//! `mnemonic restore` — watch-only single-sig restore document.
//!
//! Takes secret seed material (`ms1`/`phrase`/`entropy`/`seedqr`) + an optional
//! BIP-39 passphrase and emits a watch-only "restore document" to facilitate
//! restoring a wallet on a PC: the document leads with the master fingerprint
//! (the passphrase-correctness oracle) + first receive address(es), then the
//! concrete single-sig descriptor(s) for bip44/49/84/86 (or a single
//! `--template`).
//!
//! Read-only public derivation: NO private keys reach stdout, NO signing
//! (`feedback_no_signing_read_only_derivation_boundary`). Derivation uses a
//! verification-only secp context and NEVER touches `account_xpriv`.
//!
//! Multisig restore is DEFERRED (SPEC §11 — `restore-multisig-cosigner-scope`).

use std::io::{Read, Write};
use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::bip32::{ChainCode, ChildNumber, DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::{PublicKey, Secp256k1};
use clap::Args;

use serde_json::json;

use crate::address_render::render_address_from_xpub;
use crate::cmd::convert::{
    parse_from_input, read_stdin_passphrase, read_stdin_to_string, script_type_from_template,
    NodeType,
};
use crate::cmd::export_wallet::CliExportFormat;
use crate::derive_slot::derive_bip32_from_entropy;
use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::synthesize::ResolvedSlot;
use crate::template::CliTemplate;
use crate::wallet_export::{
    self, build_descriptor_string, BsmsForm, CheckedDescriptor, EmitInputs, TaprootInternalKey,
    TimestampArg,
};
use miniscript::{translate_hash_clone, Descriptor as MsDescriptor, DescriptorPublicKey};

/// The four single-sig templates restore emits when no `--template` is given.
const ALL_SINGLE_SIG: [CliTemplate; 4] = [
    CliTemplate::Bip44,
    CliTemplate::Bip49,
    CliTemplate::Bip84,
    CliTemplate::Bip86,
];

/// SPEC §6 — the hard ceiling on `--own-account-max K` (own-account candidate
/// count). Larger → `BadInput`. A sane account-range; the subset-search space
/// grows as `C(K, own-slots)·N!`.
const OWN_ACCOUNT_MAX_CEILING: usize = 256;

/// SPEC §6 (P3 open-item) — a sane hard ceiling on the supplied cosigner-
/// candidate count `M_sup` under `--search-cosigner-subset` (opt-in over-supply).
/// The `s_opt` ceiling already catches the combinatorial blow-up, but a concrete
/// per-axis bound refuses an absurd pool early. Mirrors `OWN_ACCOUNT_MAX_CEILING`
/// — the total pool (`K_own + M_sup`) stays bounded.
const COSIGNER_SUBSET_MAX_CANDIDATES: usize = 256;

/// SPEC §6 — the hard `realized_s` ceiling: refuse (before cap calibration,
/// distinct from the time-cap) if the realized candidate count exceeds this.
/// `1e15` is ~4000 days at the #28 benchmark's ~170M cand/min — clearly refuse;
/// the operator must narrow inputs.
const REALIZED_S_MAX: u128 = 1_000_000_000_000_000;

/// `mnemonic restore` arguments.
#[derive(Args, Debug)]
pub struct RestoreArgs {
    /// Seed source: `ms1=<v>` | `phrase=<v>` | `entropy=<hex>` | `seedqr=<digits>`.
    /// Secret values support `@env:VAR` and `-` (stdin). Non-seed nodes
    /// (xpub/xprv/wif/…) are refused (restore needs a master secret).
    /// REQUIRED for single-sig restore; OPTIONAL in multisig mode (`--md1`),
    /// where it cross-checks the own cosigner position.
    #[arg(long, required_unless_present = "md1")]
    pub from: Option<String>,

    /// Multisig-cosigner restore (v0.44.0): the shared wallet-policy `md1` card
    /// chunk(s). Reconstructs the concrete watch-only multisig descriptor from
    /// the md1 ALONE; `--from`/`--cosigner` are optional cross-check inputs.
    /// wsh / sh(wsh) and taproot multisig (NUMS or a non-NUMS distinct-trunk
    /// cosigner key) plus general single-leaf/depth-1 taproot; the @-in-both
    /// shape (trunk key also a leaf key) and depth-≥2 taproot are refused.
    /// Repeat for chunked cards.
    #[arg(long)]
    pub md1: Vec<String>,

    /// Cross-check assertion (multisig mode): `@N=<mk1-chunk|xpub>` — cosigner at
    /// position `N` is this public key. Repeat the SAME `@N=` for each chunk of a
    /// multi-chunk `mk1`. A mismatch against the md1's slot is a hard error
    /// (exit 4) unless `--allow-mismatch`. Watch-only (non-secret).
    #[arg(long)]
    pub cosigner: Vec<String>,

    /// BIP-39 mnemonic-extension passphrase. `@env:VAR` supported; or
    /// `--passphrase-stdin`. Empty (default) = no passphrase.
    #[arg(long)]
    pub passphrase: Option<String>,

    /// Read the BIP-39 passphrase from stdin (conflicts with `--passphrase`).
    #[arg(long, conflicts_with = "passphrase")]
    pub passphrase_stdin: bool,

    /// BIP-39 wordlist language for `phrase=`/`seedqr=` (default english).
    /// A `mnem` ms1 card carries its own wire language; supplying a conflicting
    /// `--language` is refused.
    #[arg(long, value_enum)]
    pub language: Option<CliLanguage>,

    /// Network (default mainnet).
    #[arg(long, value_enum)]
    pub network: Option<CliNetwork>,

    /// BIP-32 account index(es). For single-sig restore + single-sig template
    /// completion this is one account (the first value is used). For a MULTISIG
    /// template completion (#28 phase 2) this is the LIST of accounts the OWN
    /// seed is used at — one own key per account (e.g. `--account 0,1,2,3` for a
    /// 4-own-slot policy); the search places each own-derived key. Default `0`.
    /// Comma-separated; whitespace tolerated.
    #[arg(long, value_delimiter = ',', default_value = "0")]
    pub account: Vec<u32>,

    /// #28 phase 1 — explicit origin derivation path for single-sig
    /// template-completion (`restore --md1 <keyless-template>`). Overrides the
    /// template's canonical `m/<purpose>'/<coin>'/<account>'` default with an
    /// arbitrary BIP-32 path (e.g. `m/84'/0'/7'`). Only meaningful for keyless
    /// single-sig template md1 restore; ignored otherwise.
    #[arg(long = "origin")]
    pub origin: Option<String>,

    /// #28 phase 1 — expected `WalletPolicyId` (hex prefix) for single-sig
    /// template-completion. Restore recomputes the `WalletPolicyId` from the
    /// completed, fully-keyed, explicit-origin wallet and matches its leading
    /// bytes against this prefix; a MISMATCH refuses loudly (exit 4). Any-length
    /// prefix; an advisory warns when shorter than 4 bytes (8 hex chars) — a
    /// collision footgun — but does NOT enforce it (the printed convenience
    /// prefix is 4 bytes). The value the `bundle --md1-form=template` advisory
    /// printed on stderr.
    #[arg(long = "expect-wallet-id")]
    pub expect_wallet_id: Option<String>,

    /// #28 phase 2 / P2 — RANGE fallback for the OWN seed's account(s) when the
    /// exact accounts are unknown: derive the own seed at every account in
    /// `0..K` and let the multisig-template OWN-ACCOUNT SUBSET-SEARCH select the
    /// subset actually used (own-only — the `--cosigner` cards must be EXACT;
    /// over-supply cosigners with `--search-cosigner-subset`). Mutually exclusive
    /// with `--account` (clap `conflicts_with` — `--own-account-max K` ALONE
    /// passes; the `--account` default is ignored). `K ≤ 256`.
    #[arg(long = "own-account-max", conflicts_with = "account")]
    pub own_account_max: Option<u32>,

    /// P3 — OPT-IN bounded cosigner-subset search. By default (OFF) a multisig
    /// template completion requires the supplied `--cosigner` cards to be EXACT
    /// (own-only — over-supplying cosigners refuses). With this flag the operator
    /// MAY over-supply `--cosigner` cards (unsure which/how many cosigners belong);
    /// the search resolves the correct cosigner subset too. The space grows to
    /// `S_opt = Σ_j C(K_own,j)·C(M_sup,N−j)·N!`, so a LONGER `--expect-wallet-id`
    /// prefix is needed (a too-short prefix refuses; `--search-address` is the
    /// recommended collision-free mode for large opt-in pools). Bounded by the §6
    /// hard ceiling + the adaptive time-cap. Mutually exclusive with `--cosigner
    /// @N=` (explicit placement). Composes with `--own-account-max` / `--account`.
    #[arg(long = "search-cosigner-subset")]
    pub search_cosigner_subset: bool,

    /// #28 phase 2 — a known receive (or change) ADDRESS of the wallet; triggers
    /// ADDRESS-SEARCH for a multisig template completion. The search finds the
    /// unique key→slot assignment whose scriptPubKey at some `(chain, index)` in
    /// the range equals this address's. Recommended over `--expect-wallet-id`
    /// (full-scriptPubKey match — collision-free).
    #[arg(long = "search-address")]
    pub search_address: Option<String>,

    /// #28 phase 2 — inclusive lower address index for `--search-address`
    /// (default 0).
    #[arg(long = "search-addr-min", default_value_t = 0)]
    pub search_addr_min: u32,

    /// #28 phase 2 — exclusive upper address index for `--search-address`
    /// (default 20). Deepen (`0..20`, then `20..40`, …) if the target is not
    /// found; a narrow range expresses "I know the index."
    #[arg(long = "search-addr-max", default_value_t = 20)]
    pub search_addr_max: u32,

    /// #28 phase 2 — which BIP-32 change-chain branch(es) `--search-address`
    /// scans: `receive` (0, default), `change` (1), or `both`.
    #[arg(long = "search-chain", value_enum, default_value_t = CliSearchChain::Receive)]
    pub search_chain: CliSearchChain,

    /// #28 phase 2 — override the 1-hour search-time ceiling for a multisig
    /// template completion. Must be ≥ the tool's printed estimated exhaustive
    /// time (a forced acknowledgment). Accepts a humantime duration (e.g. `2h`,
    /// `90min`).
    #[arg(long = "accept-search-time")]
    pub accept_search_time: Option<String>,

    /// Restrict to a single single-sig wallet type. Omit = all four
    /// (bip44/49/84/86). A multisig template is refused (restore is single-sig).
    #[arg(long, value_enum)]
    pub template: Option<CliTemplate>,

    /// Reference master fingerprint (8 lowercase hex). Mismatch → exit 4
    /// (unless `--allow-mismatch`).
    #[arg(long)]
    pub expect_fingerprint: Option<String>,

    /// Reference account xpub (requires `--template`). Mismatch → exit 4
    /// (unless `--allow-mismatch`).
    #[arg(long)]
    pub expect_xpub: Option<String>,

    /// Emit descriptors even when a reference does not match (loud banner, exit 0).
    #[arg(long)]
    pub allow_mismatch: bool,

    /// Number of first-receive addresses to show per wallet type (default 1).
    #[arg(long, default_value_t = 1)]
    pub count: u32,

    /// Emit an importable wallet-software payload (an `export-wallet` emitter:
    /// `descriptor`, `bitcoin-core`, `bip388`, `coldcard`, `sparrow`, …).
    /// REQUIRES a single `--template` (emitters are one-descriptor-in/one-out);
    /// `--format` with no `--template` (the all-4 default) → exit 2. When set,
    /// the importable PAYLOAD goes to stdout and the verification block
    /// (fingerprint / CONFIRM / descriptor / first recv) goes to stderr, so the
    /// payload pipes cleanly into wallet software. (With `--json`, the payload is
    /// embedded as the `import_payload` field instead.)
    #[arg(long, value_enum)]
    pub format: Option<CliExportFormat>,

    /// Emit a single structured JSON object on stdout instead of the text
    /// document. Seed material is NEVER echoed (redacted by construction). The
    /// `import_payload` field is present only when `--format` is also set.
    #[arg(long)]
    pub json: bool,

    /// Write the stdout content to `<FILE>` instead of standard output
    /// (`-`, the default, → stdout). The verification block / banners / advisory
    /// still go to stderr.
    #[arg(long, default_value = "-")]
    pub output: String,
}

fn bad(s: impl Into<String>) -> ToolkitError {
    ToolkitError::BadInput(s.into())
}

/// `--search-chain` value enum (#28 phase 2). Maps to the engine's
/// [`mnemonic_toolkit::permutation_search::ChainScope`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum CliSearchChain {
    /// External / receive chain only (chain 0) — the default.
    Receive,
    /// Internal / change chain only (chain 1).
    Change,
    /// Both chains (doubles the per-index search cost).
    Both,
}

impl CliSearchChain {
    fn to_scope(self) -> mnemonic_toolkit::permutation_search::ChainScope {
        match self {
            CliSearchChain::Receive => mnemonic_toolkit::permutation_search::ChainScope::Receive,
            CliSearchChain::Change => mnemonic_toolkit::permutation_search::ChainScope::Change,
            CliSearchChain::Both => mnemonic_toolkit::permutation_search::ChainScope::Both,
        }
    }
}

impl RestoreArgs {
    /// The single account index for single-sig paths (the first / only value of
    /// the `--account` list). The clap parser guarantees ≥1 element.
    fn account_primary(&self) -> u32 {
        self.account.first().copied().unwrap_or(0)
    }
}

/// One derived wallet type: its template, concrete descriptor, and first
/// receive address(es). `slot` is the watch-only `ResolvedSlot` (entropy:
/// None) retained so a `--format` emitter can rebuild `EmitInputs` for the
/// single-template case.
struct WalletRow {
    template: CliTemplate,
    account_xpub: Xpub,
    descriptor: String,
    first_recv: Vec<String>,
    slot: ResolvedSlot,
}

/// Run `mnemonic restore`.
pub fn run<R: Read, W: Write, E: Write>(
    args: &RestoreArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    _no_auto_repair: bool,
) -> Result<u8, ToolkitError> {
    // `--md1` present → md1-driven reconstruction. Two ingest routes:
    //
    //   (1) #28 phase 1 — a KEYLESS SINGLE-SIG TEMPLATE md1 (`--md1-form=template`
    //       output): `!is_wallet_policy() && n==1 && canonical_origin().is_some()`.
    //       The template carries the script type + use-site; the seed (`--from`,
    //       REQUIRED here) provides the key; `--account`/`--origin` the origin.
    //       Routed to the NEW single-sig template completion below.
    //   (2) everything else (a keyed wallet-policy md1, OR a keyless MULTISIG
    //       template) → today's multisig reconstruction. run_multisig's
    //       keyless-md1 gate then correctly catches a keyless *multisig* template.
    //
    // The reassemble here mirrors run_multisig's (cheap; the cards are already in
    // memory). On a decode error we fall through to run_multisig so it owns the
    // (identical) error message.
    if !args.md1.is_empty() {
        let md1_refs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
        if let Ok(d) = md_codec::chunk::reassemble(&md1_refs) {
            let is_singlesig_template = !d.is_wallet_policy()
                && d.n == 1
                && md_codec::canonical_origin::canonical_origin(&d.tree).is_some()
                && crate::synthesize::cli_template_from_tree(&d.tree).is_some();
            if is_singlesig_template {
                return run_singlesig_template_completion(&d, args, stdin, stdout, stderr);
            }
            // (3) #28 phase 2 — a KEYLESS MULTISIG/GENERAL TEMPLATE md1
            //     (`--md1-form=template`, n≥2). The template carries the tree +
            //     use-site but NO keys; the operator's seed (`--from`) +
            //     externally-supplied cosigner `mk1`s complete a concrete
            //     watch-only wallet via the permutation-search engine. WITHOUT
            //     `--from` this still routes here and refuses (floor 1(i)),
            //     naming `--from`.
            //     - P3a: CANONICAL multisig (`canonical_origin().is_some()` —
            //       `wsh(multi/sortedmulti)`, `sh(wsh(...))`).
            //     - P3b: GENERAL/thresh policies (`canonical_origin().is_none()`
            //       — `wsh(or_i(...))`/timelocks/hashlocks, e.g. degrade2). The
            //       per-slot `path_decl` is built FRESH from the supplied keys'
            //       origins (NEVER the carried one — the C1 invariant), so the
            //       same tree-agnostic completion path handles both. The own
            //       origin honors the ACTUAL purpose (BIP-84 for degrade2), so a
            //       general template with cosigner mk1s never needs the canonical
            //       (BIP-48) fallback.
            //     A KEYED wallet-policy md1 has `is_wallet_policy()==true` → it
            //     falls through to `run_multisig` below (the full-policy path),
            //     which is correct.
            let is_multisig_template = !d.is_wallet_policy() && d.n >= 2;
            if is_multisig_template {
                return run_multisig_template_completion(&d, args, stdin, stdout, stderr);
            }
        }
        return run_multisig(args, stdin, stdout, stderr);
    }

    // Single-sig mode: `--from` is mandatory here (clap `required_unless_present
    // = "md1"` + the md1-empty check above guarantee `Some`).
    let from_raw = args
        .from
        .as_deref()
        .expect("--from is required in single-sig mode (required_unless_present = md1)");
    let from = parse_from_input(from_raw).map_err(bad)?;
    let from_uses_stdin = from.value == "-";

    // Seed-bearing nodes only — restore needs a master secret to derive from.
    if !matches!(
        from.node,
        NodeType::Ms1 | NodeType::Phrase | NodeType::Entropy | NodeType::Seedqr
    ) {
        return Err(bad(format!(
            "--from {} is not a seed source for restore (use ms1/phrase/entropy/seedqr)",
            from.node.as_str()
        )));
    }

    // Reject a multisig --template (restore is single-sig this cycle).
    if let Some(t) = args.template {
        if t.is_multisig() {
            return Err(bad(
                "restore is single-sig only; --template ∈ {bip44,bip49,bip84,bip86}",
            ));
        }
    }

    // `--expect-xpub` compares the per-template account xpub, which is only
    // unambiguous when a single `--template` is selected.
    if args.expect_xpub.is_some() && args.template.is_none() {
        return Err(ToolkitError::ModeViolation {
            mode: "restore",
            flag: "--expect-xpub",
            message:
                "--expect-xpub requires --template <bip44|bip49|bip84|bip86> (the account xpub is per-type)",
        });
    }

    // `--format` drives a single `export-wallet` emitter — one descriptor in,
    // one payload out — so it cannot straddle the all-4 default. Require a single
    // `--template` (SPEC I-A: ModeViolation exit 2, NOT BadInput exit 1).
    if args.format.is_some() && args.template.is_none() {
        return Err(ToolkitError::ModeViolation {
            mode: "restore",
            flag: "--format",
            message:
                "--format requires --template <bip44|bip49|bip84|bip86> (an importable payload is one descriptor — pick one type)",
        });
    }

    // Single-stdin-per-invocation guard (mirror convert / addresses).
    if args.passphrase_stdin && from_uses_stdin {
        return Err(bad(
            "--passphrase-stdin cannot coexist with --from <node>=- (a single stdin cannot serve both)",
        ));
    }

    // argv-leak advisories for inline secret-bearing values (mirror addresses scope).
    if !from_uses_stdin && !from.value.starts_with("@env:") {
        let node = from_raw.split('=').next().unwrap_or("");
        crate::secret_advisory::secret_in_argv_warning(
            stderr,
            &format!("--from {node}="),
            &format!("--from {node}=-"),
        );
    }
    if let Some(pp) = args.passphrase.as_deref() {
        if !pp.starts_with("@env:") {
            crate::secret_advisory::secret_in_argv_warning(
                stderr,
                "--passphrase",
                "--passphrase-stdin",
            );
        }
    }

    // Effective BIP-39 passphrase (stdin / @env: / inline).
    // cycle-14 (L22): wrap the passphrase / --from secret in Zeroizing so the
    // handler-scope local scrubs on drop (mlock-pinned below; mlock != scrub).
    let passphrase: zeroize::Zeroizing<String> =
        zeroize::Zeroizing::new(if args.passphrase_stdin {
            read_stdin_passphrase(stdin)?
        } else {
            match args.passphrase.as_deref() {
                Some(p) => crate::env_sentinel::resolve_env_var_sentinel(p, "--passphrase")?,
                None => String::new(),
            }
        });
    let passphrase_applied = !passphrase.is_empty();

    // Resolved `--from` value (stdin / @env: / literal).
    let from_value: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(if from_uses_stdin {
        read_stdin_to_string(stdin)?
    } else {
        crate::env_sentinel::resolve_env_var_sentinel(&from.value, "--from")?
    });

    let network = args.network.unwrap_or(CliNetwork::Mainnet);

    // Resolve the seed node → (entropy, derive_language). For ms1, the `mnem`
    // wire language wins (refuse-on-`--language`-conflict, exit 2).
    let (entropy, derive_language): (zeroize::Zeroizing<Vec<u8>>, bip39::Language) = match from.node
    {
        NodeType::Ms1 => {
            let res = crate::slot_ms1::resolve_ms1_slot(&from_value, args.language, 0)?;
            (res.entropy, res.derive_language)
        }
        NodeType::Phrase => {
            let language = args.language.unwrap_or_default();
            let entropy = zeroize::Zeroizing::new(
                Mnemonic::parse_in(language.into(), &*from_value)
                    .map_err(ToolkitError::Bip39)?
                    .to_entropy(),
            );
            (entropy, language.into())
        }
        NodeType::Seedqr => {
            let language = args.language.unwrap_or_default();
            let phrase = mnemonic_toolkit::seedqr::decode(&from_value)
                .map_err(|e| crate::cmd::seedqr::map_seedqr_error(e, "restore"))?;
            let entropy = zeroize::Zeroizing::new(
                Mnemonic::parse_in(language.into(), &phrase)
                    .map_err(ToolkitError::Bip39)?
                    .to_entropy(),
            );
            (entropy, language.into())
        }
        NodeType::Entropy => {
            let entropy = zeroize::Zeroizing::new(
                hex::decode(from_value.trim())
                    .map_err(|e| bad(format!("--from entropy= hex-decode: {e}")))?,
            );
            // No wordlist — language is irrelevant to derivation (english).
            (entropy, bip39::Language::English)
        }
        _ => unreachable!("seed-node guard above restricts to ms1/phrase/seedqr/entropy"),
    };

    // Pin the secret buffers for the remainder of the handler scope.
    let _pin_entropy = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);
    let _pin_pp = if passphrase.is_empty() {
        None
    } else {
        Some(mnemonic_toolkit::mlock::pin_pages_for(
            passphrase.as_bytes(),
        ))
    };

    let templates: &[CliTemplate] = match &args.template {
        Some(t) => std::slice::from_ref(t),
        None => &ALL_SINGLE_SIG,
    };

    // Derive each selected single-sig type. The master fingerprint is
    // path-independent — identical across all four — so capture it once.
    let secp = Secp256k1::verification_only();
    let mut master_fingerprint: Option<Fingerprint> = None;
    let mut rows: Vec<WalletRow> = Vec::with_capacity(templates.len());

    for &template in templates {
        let acct = derive_bip32_from_entropy(
            &entropy,
            &passphrase,
            derive_language,
            network,
            template,
            args.account_primary(),
        )?;
        master_fingerprint = Some(acct.master_fingerprint);

        let script_type = script_type_from_template(template)
            .expect("single-sig template has a ScriptType (multisig rejected above)");

        // First receive address(es): m/0/i children of the account xpub, derived
        // with a verification-only secp (watch-only by construction).
        let mut first_recv = Vec::with_capacity(args.count as usize);
        for i in 0..args.count {
            let chain = ChildNumber::from_normal_idx(0).unwrap();
            let leaf = ChildNumber::from_normal_idx(i).map_err(|_| {
                bad(format!(
                    "address index {i} out of BIP-32 normal range (0..2147483647)"
                ))
            })?;
            let dp: DerivationPath = vec![chain, leaf].into();
            let child = acct
                .account_xpub
                .derive_pub(&secp, &dp)
                .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
            first_recv.push(render_address_from_xpub(
                &secp,
                &child,
                script_type,
                network,
            ));
        }

        // Concrete descriptor. The watch-only ResolvedSlot mirrors the
        // wallet_import watch-only ctor: all 7 fields spelled, no entropy.
        let slot = ResolvedSlot {
            xpub: acct.account_xpub,
            fingerprint: acct.master_fingerprint,
            path: acct.account_path.clone(),
            entropy: None,
            master_xpub: None,
            language: None,
            _entropy_pin: None,
        };
        let descriptor = build_descriptor_string(
            template,
            std::slice::from_ref(&slot),
            1,
            network,
            args.account_primary(),
            None,
        )?;

        rows.push(WalletRow {
            template,
            account_xpub: acct.account_xpub,
            descriptor,
            first_recv,
            slot,
        });
        // NB: `acct` (and its `account_xpriv`) is dropped here — never emitted.
    }

    let master_fingerprint = master_fingerprint.expect("at least one template derived");
    let fp_str = master_fingerprint.to_string().to_lowercase();

    // ---- Verification gate (§3.4) -------------------------------------------
    // Compute the reference comparison (if any). `--expect-xpub` is gated to a
    // single `--template` above, so `rows[0]` is the only row when it is set.
    let mismatch: Option<(&'static str, String, String)> =
        if let Some(expected) = args.expect_fingerprint.as_deref() {
            let expected_norm = expected.trim().to_lowercase();
            if expected_norm != fp_str {
                Some(("fingerprint", fp_str.clone(), expected_norm))
            } else {
                None
            }
        } else if let Some(expected) = args.expect_xpub.as_deref() {
            let derived = rows[0].account_xpub.to_string();
            let expected = expected.trim().to_string();
            if expected != derived {
                Some(("xpub", derived, expected))
            } else {
                None
            }
        } else {
            None
        };

    let has_reference = args.expect_fingerprint.is_some() || args.expect_xpub.is_some();

    if let Some((reference, derived, expected)) = &mismatch {
        if !args.allow_mismatch {
            // Hard fail (exit 4) — no descriptors. The verify summary goes to
            // stderr; the typed error carries the derived-vs-expected detail.
            writeln!(stderr, "✗ MISMATCH").map_err(ToolkitError::Io)?;
            writeln!(
                stderr,
                "master fingerprint: {fp_str}  (passphrase: {})",
                if passphrase_applied {
                    "applied"
                } else {
                    "none"
                }
            )
            .map_err(ToolkitError::Io)?;
            return Err(ToolkitError::RestoreMismatch {
                reference,
                derived: derived.clone(),
                expected: expected.clone(),
                slot: None,
            });
        }
    }

    // Verification status label for the `--json` envelope (§3.5).
    let verification_status = if mismatch.is_some() {
        // Reached only with `--allow-mismatch` (the hard-fail path returned above).
        "overridden"
    } else if has_reference {
        "verified"
    } else {
        "unverified"
    };

    // ---- Importable payload (§3.5; Task 2.1) --------------------------------
    // `--format` is gated to a single `--template` above, so `rows[0]` is the
    // only row and the payload is one descriptor in / one payload out.
    let import_payload: Option<String> = if let Some(format) = args.format {
        Some(build_import_payload(
            format,
            &rows[0],
            network,
            args.account_primary(),
        )?)
    } else {
        None
    };

    // ---- Compose the stdout content (§3.5) ----------------------------------
    // The "stdout content" is JSON (when `--json`), or the importable payload
    // alone (when `--format` without `--json`), or the text verification doc.
    // It is routed to `--output <FILE>` when set, else to stdout. The
    // verification block + banners + advisory always go to stderr.
    let stdout_content: String = if args.json {
        let mut verification = json!({ "status": verification_status });
        if let Some((reference, derived, expected)) = &mismatch {
            verification["reference"] = json!(reference);
            verification["derived"] = json!(derived);
            verification["expected"] = json!(expected);
        }
        let wallets: Vec<_> = rows
            .iter()
            .map(|row| {
                json!({
                    "wallet_type": row.template.human_name(),
                    "descriptor": row.descriptor,
                    "first_addresses": row.first_recv,
                })
            })
            .collect();
        // Seed material (the `--from` value, passphrase) is NEVER serialized —
        // the envelope carries only public derivation products. `passphrase_applied`
        // is a bool, not the passphrase itself.
        let mut envelope = json!({
            "master_fingerprint": fp_str,
            "passphrase_applied": passphrase_applied,
            "network": network.human_name(),
            "verification": verification,
            "wallets": wallets,
        });
        if let Some(payload) = &import_payload {
            envelope["import_payload"] = json!(payload);
        }
        let s = serde_json::to_string(&envelope)
            .map_err(|e| bad(format!("json serialization: {e}")))?;
        format!("{s}\n")
    } else if let Some(payload) = &import_payload {
        // `--format` without `--json`: the payload alone is stdout so it pipes
        // cleanly into wallet software; the verification doc goes to stderr.
        format!("{payload}\n")
    } else {
        // Phase-1 text document.
        let mut s = String::new();
        s.push_str(&format!(
            "master fingerprint: {fp_str}  (passphrase: {})\n",
            if passphrase_applied {
                "applied"
            } else {
                "none"
            }
        ));
        s.push_str(
            "CONFIRM: this fingerprint matches the wallet you are restoring before importing any descriptor.\n",
        );
        for row in &rows {
            s.push('\n');
            s.push_str(&format!("{}:\n", template_label(row.template)));
            s.push_str(&format!("  descriptor: {}\n", row.descriptor));
            for addr in &row.first_recv {
                s.push_str(&format!("  first recv: {addr}\n"));
            }
        }
        s
    };

    // When `--format` is set (and not `--json`), the human verification doc is
    // not the stdout content — surface it on stderr so the operator can still
    // confirm the fingerprint while the payload pipes onward.
    if import_payload.is_some() && !args.json {
        writeln!(
            stderr,
            "master fingerprint: {fp_str}  (passphrase: {})",
            if passphrase_applied {
                "applied"
            } else {
                "none"
            }
        )
        .map_err(ToolkitError::Io)?;
        writeln!(
            stderr,
            "CONFIRM: this fingerprint matches the wallet you are restoring before importing the payload above."
        )
        .map_err(ToolkitError::Io)?;
        for row in &rows {
            writeln!(stderr, "{}:", template_label(row.template)).map_err(ToolkitError::Io)?;
            writeln!(stderr, "  descriptor: {}", row.descriptor).map_err(ToolkitError::Io)?;
            for addr in &row.first_recv {
                writeln!(stderr, "  first recv: {addr}").map_err(ToolkitError::Io)?;
            }
        }
    }

    // ---- Route the stdout content (stdout | --output FILE) ------------------
    if args.output == "-" {
        write!(stdout, "{stdout_content}").map_err(ToolkitError::Io)?;
    } else {
        std::fs::write(&args.output, &stdout_content)
            .map_err(|e| bad(format!("--output {}: {e}", args.output)))?;
    }

    // ---- Verification banners (stderr) --------------------------------------
    if mismatch.is_some() {
        // Reached only with `--allow-mismatch` (the hard-fail path returned above).
        writeln!(
            stderr,
            "✗ MISMATCH (overridden): derived material does NOT match the supplied reference; \
             descriptors above were produced by the passphrase you provided, NOT the expected wallet"
        )
        .map_err(ToolkitError::Io)?;
    } else if !has_reference {
        writeln!(
            stderr,
            "UNVERIFIED: no --expect-fingerprint/--expect-xpub supplied; verify the master \
             fingerprint above ({fp_str}) against your records before importing"
        )
        .map_err(ToolkitError::Io)?;
    }

    crate::secret_advisory::emit_output_class_advisory(
        crate::secret_advisory::OutputClass::WatchOnly,
        stderr,
    );

    Ok(0)
}

/// #28 phase 1 — complete a KEYLESS SINGLE-SIG TEMPLATE md1 into a concrete
/// watch-only wallet. The template (`d`, already reassembled + classified by
/// the caller) supplies the script type + use-site; the seed (`--from`,
/// MANDATORY here) supplies the key; `--account` / `--origin` supply the
/// origin.
///
/// FUNDS-SAFETY (C2): `--from` is `required_unless_present="md1"` at clap level,
/// so `restore --md1 <template>` with NO `--from` is clap-valid and would
/// mis-route to a watch-only document for nobody's wallet. This arm REJECTS a
/// missing `--from` explicitly — the seed is the key; a no-seed template
/// restore is a silent-wrong-route hole.
///
/// `--expect-wallet-id <prefix>` (optional): recomputes the `WalletPolicyId`
/// from the completed, fully-keyed, EXPLICIT-origin, presence-`0b11` descriptor
/// (via the SHARED `wallet_policy_id_for_singlesig` — the same preimage the
/// `bundle` D7 advisory printed) and matches its leading bytes; a MISMATCH
/// refuses loudly (exit 4). Note `--origin` overrides break the
/// canonical-account assumption D7 was computed under, so `--expect-wallet-id`
/// is only checked when no `--origin` is supplied (an explicit-origin wallet's
/// id is a different preimage).
fn run_singlesig_template_completion<R: Read, W: Write, E: Write>(
    d: &md_codec::Descriptor,
    args: &RestoreArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let network = args.network.unwrap_or(CliNetwork::Mainnet);

    // tree → CliTemplate (the caller already guaranteed Some).
    let template = crate::synthesize::cli_template_from_tree(&d.tree)
        .ok_or_else(|| bad("template md1 tree is not a canonical single-sig shape"))?;

    // --- (b) `--from` REQUIRED (the C2 funds-safety hole) -------------------
    let from_raw = args.from.as_deref().ok_or(ToolkitError::ModeViolation {
        mode: "restore",
        flag: "--md1",
        message: "restore of a keyless single-sig TEMPLATE md1 requires --from <seed> \
                  (the template carries no key; the seed derives it). Supply \
                  --from ms1=…/phrase=…/entropy=…/seedqr=…",
    })?;
    let from = parse_from_input(from_raw).map_err(bad)?;
    let from_uses_stdin = from.value == "-";
    if !matches!(
        from.node,
        NodeType::Ms1 | NodeType::Phrase | NodeType::Entropy | NodeType::Seedqr
    ) {
        return Err(bad(format!(
            "--from {} is not a seed source for restore (use ms1/phrase/entropy/seedqr)",
            from.node.as_str()
        )));
    }

    // Single-stdin guard (mirror the single-sig path).
    if args.passphrase_stdin && from_uses_stdin {
        return Err(bad(
            "--passphrase-stdin cannot coexist with --from <node>=- (a single stdin cannot serve both)",
        ));
    }

    // argv-leak advisory.
    if !from_uses_stdin && !from.value.starts_with("@env:") {
        let node = from_raw.split('=').next().unwrap_or("");
        crate::secret_advisory::secret_in_argv_warning(
            stderr,
            &format!("--from {node}="),
            &format!("--from {node}=-"),
        );
    }
    if let Some(pp) = args.passphrase.as_deref() {
        if !pp.starts_with("@env:") {
            crate::secret_advisory::secret_in_argv_warning(
                stderr,
                "--passphrase",
                "--passphrase-stdin",
            );
        }
    }

    // cycle-14 (L22): wrap in Zeroizing (handler-scope scrub; mlock-pinned).
    let passphrase: zeroize::Zeroizing<String> =
        zeroize::Zeroizing::new(if args.passphrase_stdin {
            read_stdin_passphrase(stdin)?
        } else {
            match args.passphrase.as_deref() {
                Some(p) => crate::env_sentinel::resolve_env_var_sentinel(p, "--passphrase")?,
                None => String::new(),
            }
        });
    let passphrase_applied = !passphrase.is_empty();

    let from_value: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(if from_uses_stdin {
        read_stdin_to_string(stdin)?
    } else {
        crate::env_sentinel::resolve_env_var_sentinel(&from.value, "--from")?
    });

    // Resolve the seed node → (entropy, derive_language). Mirrors the single-sig
    // `run` body.
    let (entropy, derive_language): (zeroize::Zeroizing<Vec<u8>>, bip39::Language) = match from.node
    {
        NodeType::Ms1 => {
            let res = crate::slot_ms1::resolve_ms1_slot(&from_value, args.language, 0)?;
            (res.entropy, res.derive_language)
        }
        NodeType::Phrase => {
            let language = args.language.unwrap_or_default();
            let entropy = zeroize::Zeroizing::new(
                Mnemonic::parse_in(language.into(), &*from_value)
                    .map_err(ToolkitError::Bip39)?
                    .to_entropy(),
            );
            (entropy, language.into())
        }
        NodeType::Seedqr => {
            let language = args.language.unwrap_or_default();
            let phrase = mnemonic_toolkit::seedqr::decode(&from_value)
                .map_err(|e| crate::cmd::seedqr::map_seedqr_error(e, "restore"))?;
            let entropy = zeroize::Zeroizing::new(
                Mnemonic::parse_in(language.into(), &phrase)
                    .map_err(ToolkitError::Bip39)?
                    .to_entropy(),
            );
            (entropy, language.into())
        }
        NodeType::Entropy => {
            let entropy = zeroize::Zeroizing::new(
                hex::decode(from_value.trim())
                    .map_err(|e| bad(format!("--from entropy= hex-decode: {e}")))?,
            );
            (entropy, bip39::Language::English)
        }
        _ => unreachable!("seed-node guard above restricts to ms1/phrase/seedqr/entropy"),
    };
    let _pin_entropy = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);
    let _pin_pp = if passphrase.is_empty() {
        None
    } else {
        Some(mnemonic_toolkit::mlock::pin_pages_for(
            passphrase.as_bytes(),
        ))
    };

    // --- (c) derive the account key at --origin OR the template default -----
    let explicit_origin = match args.origin.as_deref() {
        Some(s) => Some(
            DerivationPath::from_str(s.trim_start_matches("m/").trim_start_matches('m'))
                .or_else(|_| DerivationPath::from_str(s))
                .map_err(|e| bad(format!("--origin {s}: {e}")))?,
        ),
        None => None,
    };
    let acct = match &explicit_origin {
        Some(path) => crate::derive_slot::derive_bip32_from_entropy_at_path(
            &entropy,
            &passphrase,
            derive_language,
            network,
            path,
        )?,
        None => derive_bip32_from_entropy(
            &entropy,
            &passphrase,
            derive_language,
            network,
            template,
            args.account_primary(),
        )?,
    };
    let master_fingerprint = acct.master_fingerprint;
    let fp_str = master_fingerprint.to_string().to_lowercase();

    let script_type = script_type_from_template(template)
        .expect("template_from_tree only yields single-sig templates");

    // First receive address(es): m/0/i children of the account xpub.
    let secp = Secp256k1::verification_only();
    let mut first_recv = Vec::with_capacity(args.count as usize);
    for i in 0..args.count {
        let chain = ChildNumber::from_normal_idx(0).unwrap();
        let leaf = ChildNumber::from_normal_idx(i)
            .map_err(|_| bad(format!("address index {i} out of BIP-32 normal range")))?;
        let dp: DerivationPath = vec![chain, leaf].into();
        let child = acct
            .account_xpub
            .derive_pub(&secp, &dp)
            .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
        first_recv.push(render_address_from_xpub(
            &secp,
            &child,
            script_type,
            network,
        ));
    }

    // Concrete watch-only descriptor.
    let slot = ResolvedSlot {
        xpub: acct.account_xpub,
        fingerprint: master_fingerprint,
        path: acct.account_path.clone(),
        entropy: None,
        master_xpub: None,
        language: None,
        _entropy_pin: None,
    };
    let descriptor = build_descriptor_string(
        template,
        std::slice::from_ref(&slot),
        1,
        network,
        args.account_primary(),
        None,
    )?;

    // --- --expect-wallet-id (D7 recompute-and-match) ------------------------
    // Only checked for the canonical (no --origin) path: D7 was computed by the
    // bundle from the canonical `m/<purpose>'/<coin>'/account'` origin, so an
    // explicit --origin override is a different preimage (advise, don't match).
    if let Some(prefix_hex) = args.expect_wallet_id.as_deref() {
        if explicit_origin.is_some() {
            writeln!(
                stderr,
                "notice: --expect-wallet-id is not checked when --origin overrides the canonical \
                 account path (the wallet-id was computed for the canonical origin)."
            )
            .map_err(ToolkitError::Io)?;
        } else {
            let prefix = decode_wallet_id_prefix(prefix_hex)?;
            if prefix.len() < 4 {
                writeln!(
                    stderr,
                    "advisory: --expect-wallet-id prefix is only {} byte(s); ≥4 bytes is \
                     recommended (a short prefix is a collision footgun).",
                    prefix.len()
                )
                .map_err(ToolkitError::Io)?;
            }
            let id = crate::synthesize::wallet_policy_id_for_singlesig(
                template,
                network,
                &acct.account_xpub,
                master_fingerprint,
                args.account_primary(),
            )?;
            let id_bytes = id.as_bytes();
            if id_bytes.len() < prefix.len() || id_bytes[..prefix.len()] != prefix[..] {
                let shown = prefix.len().max(4).min(id_bytes.len());
                let derived_prefix = hex::encode(&id_bytes[..shown]);
                writeln!(stderr, "✗ WALLET-ID MISMATCH").map_err(ToolkitError::Io)?;
                return Err(ToolkitError::RestoreMismatch {
                    reference: "wallet-id",
                    derived: derived_prefix,
                    expected: prefix_hex.trim().to_lowercase(),
                    slot: None,
                });
            }
            writeln!(
                stderr,
                "✓ wallet-id verified: completed wallet matches --expect-wallet-id"
            )
            .map_err(ToolkitError::Io)?;
        }
    }

    // ---- Compose output (text or JSON) -------------------------------------
    let stdout_content: String = if args.json {
        let envelope = json!({
            "master_fingerprint": fp_str,
            "passphrase_applied": passphrase_applied,
            "network": network.human_name(),
            "completed_from": "template-md1",
            "wallets": [json!({
                "wallet_type": template.human_name(),
                "descriptor": descriptor,
                "first_addresses": first_recv,
            })],
        });
        format!(
            "{}\n",
            serde_json::to_string(&envelope)
                .map_err(|e| bad(format!("json serialization: {e}")))?
        )
    } else {
        let mut s = String::new();
        s.push_str(&format!(
            "master fingerprint: {fp_str}  (passphrase: {})\n",
            if passphrase_applied {
                "applied"
            } else {
                "none"
            }
        ));
        s.push_str(
            "CONFIRM: this fingerprint matches the wallet you are restoring before importing.\n",
        );
        s.push('\n');
        s.push_str(&format!("{}:\n", template_label(template)));
        s.push_str(&format!("  descriptor: {descriptor}\n"));
        for addr in &first_recv {
            s.push_str(&format!("  first recv: {addr}\n"));
        }
        s
    };

    if args.output == "-" {
        write!(stdout, "{stdout_content}").map_err(ToolkitError::Io)?;
    } else {
        std::fs::write(&args.output, &stdout_content)
            .map_err(|e| bad(format!("--output {}: {e}", args.output)))?;
    }

    if args.expect_wallet_id.is_none() {
        writeln!(
            stderr,
            "UNVERIFIED: no --expect-wallet-id supplied; verify the master fingerprint above \
             ({fp_str}) against your records before importing"
        )
        .map_err(ToolkitError::Io)?;
    }
    crate::secret_advisory::emit_output_class_advisory(
        crate::secret_advisory::OutputClass::WatchOnly,
        stderr,
    );
    Ok(0)
}

// ===========================================================================
// #28 phase 2 (P3a) — keyless CANONICAL MULTISIG template completion.
//
// The funds-safety / silent-wrong-wallet core. The keyless template md1 carries
// the policy tree + use-site (incl. #25 overrides) but NO keys. The operator's
// own seed (`--from` at `--account`s) + externally-supplied cosigner `mk1`s are
// permuted across the N `@N` slots by the permutation-search engine
// (`mnemonic_toolkit::permutation_search`) until a UNIQUE assignment matches a
// recorded `--expect-wallet-id` (id-search) or a known `--search-address`
// (address-search). A wrong assignment NO-MATCHES → refuse (never silent-wrong).
//
// ORIGIN MODEL (load-bearing — see the module-level note + the report):
// the per-slot origins are BUILT FRESH from the supplied keys, NOT loaded from
// the template's carried `path_decl` (the C1 invariant). Cosigner origins come
// from each mk1's `origin_path`. The OWN origin defaults to the SAME path-family
// the cosigners use (read off a cosigner mk1, the own `--account` substituted
// into the account component) — because the toolkit's multisig emit DEFAULTS to
// BIP-87 (`m/87'/coin'/acct'`), NOT the BIP-48 `canonical_origin(tree)` the
// SPEC's worked example assumed. Honoring the cosigners' actual family is what
// reproduces a toolkit-emitted wallet regardless of `--multisig-path-family`;
// `--origin` overrides it explicitly. Either way the build is search-VERIFIED,
// so a wrong own origin only NO-MATCHES (fail-safe), never silent-wrong.
// ===========================================================================

/// A supplied candidate key for the multisig-template search: its 65-byte form,
/// master fingerprint, BUILT origin, and provenance (for the output + the
/// every-slot-supplied gate). The `(key65, origin)` PAIR is the search's
/// permutable unit.
#[derive(Clone, Debug)]
pub(crate) struct CandidateKey {
    key65: [u8; 65],
    fingerprint: Fingerprint,
    origin: DerivationPath,
    /// True for an own-seed-derived key (for the "your seed" annotation + the
    /// own-position gate); false for a cosigner mk1/xpub.
    is_own: bool,
}

/// #28 phase 2 — the ARG-STRUCT-NEUTRAL input to the shared multisig-template
/// completion engine [`complete_multisig_template`]. Both `restore` and
/// `verify-bundle` populate this from their own `*Args` (the only difference
/// between the two surfaces) and then run the IDENTICAL completion. The funds-
/// safety core (cosigner parse → per-slot origin BUILD → floors → mode → search
/// → fresh-descriptor build) lives once, behind this struct — so the two
/// surfaces can never drift on a silent-wrong-wallet decision.
pub(crate) struct MultisigCompletionCtx<'a> {
    /// The seed entropy already resolved by the caller (stdin/`@env:`/etc.).
    pub entropy: &'a [u8],
    /// The BIP-39 passphrase (already resolved; empty = none).
    pub passphrase: &'a str,
    /// The wordlist language to derive the own key under.
    pub derive_language: bip39::Language,
    /// The own account(s) the `--from` seed is used at (one own key each).
    pub own_accounts: Vec<u32>,
    /// An explicit `--origin` own-origin override (else cosigner-family default).
    pub explicit_own_origin: Option<DerivationPath>,
    /// `--cosigner` specs verbatim (assigned `@N=` or unassigned), pre-grouped.
    pub cosigner_specs: &'a [String],
    /// `--own-account-max K` — the OWN-account RANGE subset-search (P2): derive
    /// the own seed at accounts `0..K` and resolve the unique own→slot
    /// assignment over the enlarged (own-first) pool. Carried so BOTH restore +
    /// verify-bundle drive the subset-search uniformly. `K ≤ 256` (SPEC §6).
    pub own_account_max: Option<u32>,
    /// P3 — `--search-cosigner-subset`: OPT-IN bounded cosigner-subset search.
    /// When `true` the supplied `--cosigner` cards MAY exceed the wallet's true
    /// cosigner count; the search ranges over the `(own-subset, cosigner-subset,
    /// ordering)` opt-in space `S_opt` (SPEC §4.3) and resolves the correct
    /// subset. When `false` (default) the cosigner cards must be EXACT (own-only).
    /// Carried so BOTH restore + verify-bundle drive the opt-in path uniformly.
    pub search_cosigner_subset: bool,
    /// The completion mode target — `--expect-wallet-id` (id-search).
    pub expect_wallet_id: Option<String>,
    /// The completion mode target — `--search-address` (address-search).
    pub search_address: Option<String>,
    /// Address-search range `[min, max)`.
    pub search_addr_min: u32,
    pub search_addr_max: u32,
    /// Address-search chain scope.
    pub search_chain: CliSearchChain,
    /// `--accept-search-time` cap override (forced acknowledgment).
    pub accept_search_time: Option<String>,
    /// The network the recomposed wallet is on.
    pub network: CliNetwork,
}

/// #28 phase 2 — the resolved output of [`complete_multisig_template`]: the
/// freshly-built, fully-keyed completed descriptor + the supplied pool + the
/// resolved slot→pool assignment. The caller (restore: emit; verify-bundle:
/// bind + recompose + report) consumes these.
pub(crate) struct MultisigCompletionOutcome {
    /// The completed, fully-keyed `md_codec::Descriptor` (the C1-fresh build).
    pub completed: md_codec::Descriptor,
    /// The supplied candidate pool (own keys + cosigners), pre-search order.
    pub pool: Vec<CandidateKey>,
    /// `assignment[i]` = the `pool` index placed at slot `@i`.
    pub assignment: Vec<usize>,
}

impl MultisigCompletionOutcome {
    /// The slot whose pool entry is the operator's own key (for the annotation).
    pub(crate) fn own_position(&self) -> Option<usize> {
        self.assignment.iter().position(|&pi| self.pool[pi].is_own)
    }
}

/// Derive the watch-only scriptPubKey at `(chain, index)` from a parsed
/// multipath descriptor. Splits the `<0;1>` multipath, selects the `chain`
/// branch (0 = receive, 1 = change), derives at `index`, and returns the
/// scriptPubKey bytes. Used by the address-search evaluator.
fn script_pubkey_at(
    desc: &MsDescriptor<DescriptorPublicKey>,
    chain: u32,
    index: u32,
) -> Result<Vec<u8>, ToolkitError> {
    let branch = if desc.is_multipath() {
        let parts = desc
            .clone()
            .into_single_descriptors()
            .map_err(|e| bad(format!("address-search: multipath split failed: {e}")))?;
        parts
            .get(chain as usize)
            .cloned()
            .ok_or_else(|| bad(format!("address-search: no chain branch {chain}")))?
    } else if chain == 0 {
        desc.clone()
    } else {
        // A non-multipath descriptor has only the one (receive) branch.
        return Err(bad("address-search: descriptor has no change branch"));
    };
    let definite = if branch.has_wildcard() {
        branch
            .derive_at_index(index)
            .map_err(|e| bad(format!("address-search: derive_at_index({index}): {e}")))?
    } else {
        MsDescriptor::<miniscript::descriptor::DefiniteDescriptorKey>::try_from(branch)
            .map_err(|e| bad(format!("address-search: definite-key: {e}")))?
    };
    let spk = definite.script_pubkey();
    Ok(spk.to_bytes())
}

/// The family template of a cosigner origin path with the ACCOUNT component
/// blanked: i.e. the path with component index 2 (the BIP-44/48/84/87 account)
/// replaced by the own `account`. For `m/87'/0'/3'` (BIP-87) → `m/87'/0'/A'`;
/// for `m/48'/0'/3'/2'` (BIP-48) → `m/48'/0'/A'/2'`. Returns `None` when the
/// path is too shallow to carry an account component (the caller falls back to
/// `canonical_origin(tree)`).
fn own_origin_from_family(family: &DerivationPath, account: u32) -> Option<DerivationPath> {
    let mut comps: Vec<ChildNumber> = family.into_iter().copied().collect();
    // The account is the 3rd path component (index 2) in every supported
    // family (BIP-44/49/84/86 single-sig and BIP-48/87 multisig).
    if comps.len() < 3 {
        return None;
    }
    comps[2] = ChildNumber::from_hardened_idx(account).ok()?;
    Some(DerivationPath::from(comps))
}

/// #28 phase 2 — the resolved `--from` seed for a multisig-template completion:
/// the (pinned) entropy + the BIP-39 passphrase + the wordlist language. The
/// mlock pins live for the lifetime of this struct (held by the caller).
pub(crate) struct TemplateSeed {
    pub entropy: zeroize::Zeroizing<Vec<u8>>,
    // cycle-14 (L22): the resolved BIP-39 passphrase scrubs on drop.
    pub passphrase: zeroize::Zeroizing<String>,
    pub derive_language: bip39::Language,
    _pin: Option<mnemonic_toolkit::mlock::PinnedPageRange>,
    _pin_pp: Option<mnemonic_toolkit::mlock::PinnedPageRange>,
}

/// #28 phase 2 — resolve a `--from` seed string (+ passphrase) for a multisig-
/// template completion, SHARED by `restore` + `verify-bundle` so the two
/// surfaces resolve the seed (argv-leak advisories, stdin-coexist gate, `@env:`
/// / stdin handling, the seed-source node gate) byte-for-byte identically. A
/// missing `--from` returns `no_from` (the surface's own floor-1(i) message
/// naming `--from`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_template_completion_seed<E: Write>(
    from_raw: Option<&str>,
    no_from: ToolkitError,
    passphrase: Option<&str>,
    passphrase_stdin: bool,
    language: Option<CliLanguage>,
    stdin: &mut dyn Read,
    stderr: &mut E,
) -> Result<TemplateSeed, ToolkitError> {
    let from_raw = from_raw.ok_or(no_from)?;
    let from = parse_from_input(from_raw).map_err(bad)?;
    let from_uses_stdin = from.value == "-";
    if !matches!(
        from.node,
        NodeType::Ms1 | NodeType::Phrase | NodeType::Entropy | NodeType::Seedqr
    ) {
        return Err(bad(format!(
            "--from {} is not a seed source for restore (use ms1/phrase/entropy/seedqr)",
            from.node.as_str()
        )));
    }
    if passphrase_stdin && from_uses_stdin {
        return Err(bad(
            "--passphrase-stdin cannot coexist with --from <node>=- (a single stdin cannot serve both)",
        ));
    }

    // argv-leak advisories (mirror the single-sig template path).
    if !from_uses_stdin && !from.value.starts_with("@env:") {
        let node = from_raw.split('=').next().unwrap_or("");
        crate::secret_advisory::secret_in_argv_warning(
            stderr,
            &format!("--from {node}="),
            &format!("--from {node}=-"),
        );
    }
    if let Some(pp) = passphrase {
        if !pp.starts_with("@env:") {
            crate::secret_advisory::secret_in_argv_warning(
                stderr,
                "--passphrase",
                "--passphrase-stdin",
            );
        }
    }

    // --- Resolve the seed entropy --------------------------------------------
    // `stdin` is `&mut dyn Read` (unsized); reborrow as `&mut (&mut dyn Read)`
    // so the generic `<R: Read>` stdin helpers monomorphize over the sized
    // reference type.
    // cycle-14 (L22): wrap in Zeroizing (handler-scope scrub).
    let passphrase: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(if passphrase_stdin {
        read_stdin_passphrase(&mut &mut *stdin)?
    } else {
        match passphrase {
            Some(p) => crate::env_sentinel::resolve_env_var_sentinel(p, "--passphrase")?,
            None => String::new(),
        }
    });
    let from_value: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(if from_uses_stdin {
        read_stdin_to_string(&mut &mut *stdin)?
    } else {
        crate::env_sentinel::resolve_env_var_sentinel(&from.value, "--from")?
    });
    let (entropy, derive_language) = resolve_seed_entropy(&from.node, &from_value, language)?;
    let pin = Some(mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]));
    let pin_pp = (!passphrase.is_empty())
        .then(|| mnemonic_toolkit::mlock::pin_pages_for(passphrase.as_bytes()));
    Ok(TemplateSeed {
        entropy,
        passphrase,
        derive_language,
        _pin: pin,
        _pin_pp: pin_pp,
    })
}

/// `mnemonic restore --md1 <keyless CANONICAL MULTISIG template>` — complete a
/// concrete watch-only multisig wallet from `--from` (own seed) + `--cosigner`
/// (external mk1/xpub) via the permutation-search engine. The R0-heavy
/// funds-safety core; see the section banner above.
fn run_multisig_template_completion<R: Read, W: Write, E: Write>(
    d: &md_codec::Descriptor,
    args: &RestoreArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let network = args.network.unwrap_or(CliNetwork::Mainnet);

    // (The `--own-account-max` I-1 gate now lives in the SHARED
    // `complete_multisig_template` core, so both restore + verify-bundle refuse
    // it uniformly.)

    // --- Floor 1(i): `--from` is REQUIRED + resolve the seed entropy ----------
    // Factored into the SHARED helper so verify-bundle's multisig-template path
    // resolves the seed identically (same argv advisories, stdin coexist gate,
    // `@env:`/stdin handling).
    let no_from = ToolkitError::ModeViolation {
        mode: "restore",
        flag: "--md1",
        message: "restore of a keyless MULTISIG TEMPLATE md1 requires --from <seed> \
                  (the template carries no keys; the seed derives your cosigner key, and \
                  --cosigner <mk1> supplies the others). Supply \
                  --from ms1=…/phrase=…/entropy=…/seedqr=…",
    };
    let seed = resolve_template_completion_seed(
        args.from.as_deref(),
        no_from,
        args.passphrase.as_deref(),
        args.passphrase_stdin,
        args.language,
        stdin,
        stderr,
    )?;

    // --- The explicit own-origin override (#28 phase-1 `--origin`, reused). ---
    let explicit_own_origin = parse_explicit_own_origin(args.origin.as_deref())?;

    // --- Build the NEUTRAL completion ctx + run the SHARED engine ------------
    // Everything funds-safety-critical (cosigner parse → per-slot origin BUILD →
    // floors → mode → search → fresh-descriptor build) lives in
    // `complete_multisig_template`, shared byte-for-byte with `verify-bundle`.
    let ctx = MultisigCompletionCtx {
        entropy: &seed.entropy,
        passphrase: &seed.passphrase,
        derive_language: seed.derive_language,
        own_accounts: args.account.clone(),
        explicit_own_origin,
        cosigner_specs: &args.cosigner,
        own_account_max: args.own_account_max,
        search_cosigner_subset: args.search_cosigner_subset,
        expect_wallet_id: args.expect_wallet_id.clone(),
        search_address: args.search_address.clone(),
        search_addr_min: args.search_addr_min,
        search_addr_max: args.search_addr_max,
        search_chain: args.search_chain,
        accept_search_time: args.accept_search_time.clone(),
        network,
    };
    let outcome = complete_multisig_template(d, &ctx, stderr)?;
    emit_completed_multisig(
        &outcome.completed,
        &outcome.pool,
        &outcome.assignment,
        args,
        network,
        stdout,
        stderr,
    )
}

/// Parse the `--origin` own-origin override (accepts `m/…` or bare `…`).
fn parse_explicit_own_origin(s: Option<&str>) -> Result<Option<DerivationPath>, ToolkitError> {
    match s {
        Some(s) => Ok(Some(
            DerivationPath::from_str(s.trim_start_matches("m/").trim_start_matches('m'))
                .or_else(|_| DerivationPath::from_str(s))
                .map_err(|e| bad(format!("--origin {s}: {e}")))?,
        )),
        None => Ok(None),
    }
}

/// #28 phase 2 — the SHARED, arg-struct-NEUTRAL multisig-template completion
/// engine. Given a keyless MULTISIG/general template `d` + the resolved
/// [`MultisigCompletionCtx`], runs the IDENTICAL funds-safety completion both
/// `restore` (emit) and `verify-bundle` (bind + recompose) need:
///   cosigner parse → per-slot origin BUILD (NEVER the carried `path_decl`, the
///   C1 invariant) → the every-slot + distinct-key floors → the completion mode
///   (id-search / address-search / explicit `@N=`) → the permutation search →
///   the unique assignment + a fresh fully-keyed descriptor.
///
/// On NO-MATCH / AMBIGUOUS / any floor violation it RETURNS the (refuse) error —
/// never a silent wrong wallet. `stderr` carries the explicit-mode warning + the
/// search progress line. Restore behavior is byte-identical to the pre-factor
/// inline body (the wrapper now just resolves the seed + builds the ctx + emits).
pub(crate) fn complete_multisig_template<E: Write>(
    d: &md_codec::Descriptor,
    ctx: &MultisigCompletionCtx,
    stderr: &mut E,
) -> Result<MultisigCompletionOutcome, ToolkitError> {
    use mnemonic_toolkit::permutation_search as ps;

    let network = ctx.network;
    let n = d.n as usize;

    // --- P2: `--own-account-max K` (own-account RANGE subset-search). The I-1
    // refuse gate (#28 P3a) is LIFTED — the genuine own-anchored subset-search
    // has landed (SPEC §3/§4). `Some(k)` over-supplies the OWN candidates (own
    // seed derived at accounts `0..k`); the engine resolves the unique own→slot
    // assignment over the enlarged pool (own-first). §6 hard ceiling: `K ≤ 256`.
    let own_account_max: Option<usize> = match ctx.own_account_max {
        Some(k) => {
            let k = k as usize;
            if k == 0 {
                return Err(bad(
                    "--own-account-max must be ≥ 1 (it derives the own seed at accounts 0..K)",
                ));
            }
            if k > OWN_ACCOUNT_MAX_CEILING {
                return Err(bad(format!(
                    "--own-account-max {k} exceeds the hard ceiling of {OWN_ACCOUNT_MAX_CEILING} \
                     own-account candidates: narrow the range (the subset-search space grows as \
                     C(K, own-slots)·N!)."
                )));
            }
            Some(k)
        }
        None => None,
    };

    // --- L9: the SAME early refusals the non-template `run_multisig` path
    // applies BEFORE reconstruction. Hoisted here (the SHARED completion core)
    // so BOTH `restore` and `verify-bundle` refuse uniformly — matching the I-1
    // own-account-max gate's placement.
    //   (1) ANY hardened use-site (`/*h` wildcard or a hardened multipath
    //       alternative, baseline OR per-`@N` override): watch-only cannot do
    //       hardened public derivation (BIP-32), so a reconstructed descriptor
    //       would silently render an unhardened `/*` and a derive attempt fails.
    //       Refuse EARLY with the precise, actionable message instead of an
    //       opaque downstream NO-MATCH.
    //   (2) A TAPROOT override card OUTSIDE the restorable subset (#26): a
    //       sortedmulti_a tap leaf, a non-NUMS trunk key, or a hardened use-site
    //       — these route around the faithful per-`@N` arm and would mis-render.
    //       The RESTORABLE subset — non-hardened tr(NUMS, multi_a) — is admitted.
    // Named multisig templates are non-hardened today and never reach the
    // taproot-override leg, so these are defense-in-depth / latent-form guards;
    // a future bundle form emitting a hardened canonical multisig template gets
    // the precise refusal instead of a confusing NO-MATCH.
    if md_codec::to_miniscript::has_hardened_use_site(d) {
        return Err(ToolkitError::ModeViolation {
            mode: "restore",
            flag: "--md1",
            message: "this md1 uses a hardened use-site path (`/*h` wildcard or a hardened multipath alternative, baseline or per-cosigner) — watch-only addresses cannot be derived from it, and a reconstructed descriptor would silently render an unhardened path. Faithful reconstruction is not supported. The engraved card remains a faithful backup. Tracked: restore-md1-per-key-use-site-and-hardened-wildcard",
        });
    }
    if taproot_override_card(d) && !restorable_taproot_override_card(d) {
        return Err(ToolkitError::ModeViolation {
            mode: "restore",
            flag: "--md1",
            message: "this taproot md1 carries per-cosigner use-site path overrides in a shape the toolkit cannot yet reconstruct faithfully (a sortedmulti_a tap leaf, or a non-NUMS internal/trunk key). Non-hardened tr(NUMS, multi_a(...)) override cards ARE restorable; other taproot override shapes route around the per-key reconstruction path and emitting a single shared suffix would misrepresent the wallet. The engraved card remains a faithful backup. Tracked: restore-md1-taproot-use-site-override-arm",
        });
    }

    // --- Parse `--cosigner`: unassigned (search) vs assigned `@N=` (explicit) -
    // (I-B) The phase-1 `@N=` parse read only the 65-byte key for a cross-check;
    // the template completion reads the mk1's ORIGIN too, and admits the
    // UNASSIGNED form (no `@N=`), which the search places.
    // A cosigner card may be MULTI-CHUNK; the chunks arrive as separate
    // `--cosigner` values. Group them: assigned `@N=` chunks group by `@N`;
    // unassigned chunks group greedily (accumulate until `mk_codec::decode`
    // accepts the running set — the codec rejects an incomplete chunk set,
    // naming the expected count).
    let mut any_assigned = false;
    let mut assigned_chunks: std::collections::BTreeMap<u8, Vec<String>> =
        std::collections::BTreeMap::new();
    let mut unassigned_chunks: Vec<String> = Vec::new();
    for spec in ctx.cosigner_specs {
        if spec.starts_with('@') {
            let (lhs, rhs) = spec.split_once('=').ok_or_else(|| {
                bad(format!(
                    "--cosigner @N= expects `@N=<mk1|xpub>`, got `{spec}`"
                ))
            })?;
            let nn: u8 = lhs
                .trim_start_matches('@')
                .parse()
                .map_err(|_| bad(format!("--cosigner position `{lhs}` is not `@N`")))?;
            any_assigned = true;
            assigned_chunks.entry(nn).or_default().push(rhs.to_string());
        } else {
            unassigned_chunks.push(spec.clone());
        }
    }

    let mut unassigned_cosigners: Vec<CandidateKey> = Vec::new();
    let mut assigned_cosigners: std::collections::BTreeMap<u8, CandidateKey> =
        std::collections::BTreeMap::new();
    for (nn, chunks) in &assigned_chunks {
        if *nn as usize >= n {
            return Err(bad(format!(
                "--cosigner @{nn}: position out of range (wallet has {n} slots)"
            )));
        }
        let refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        assigned_cosigners.insert(*nn, decode_cosigner_card(&refs, network)?);
    }
    // Greedy-group the unassigned chunks into cards.
    {
        let mut buf: Vec<&str> = Vec::new();
        for chunk in &unassigned_chunks {
            buf.push(chunk.as_str());
            // A bare xpub (non-mk1) is a complete single value.
            let is_mk1 = chunk.to_lowercase().starts_with("mk1");
            if !is_mk1 {
                unassigned_cosigners.push(decode_cosigner_card(&buf, network)?);
                buf.clear();
                continue;
            }
            // mk1: try to decode the running set; on success it's one card.
            if let Ok(ck) = try_decode_cosigner_card(&buf) {
                unassigned_cosigners.push(ck);
                buf.clear();
            }
            // else: incomplete chunk set — keep accumulating.
        }
        if !buf.is_empty() {
            // Leftover chunks that never completed a card → surface the codec error.
            return Err(decode_cosigner_card(&buf, network).unwrap_err());
        }
    }

    // Mixing assigned + unassigned cosigners is ambiguous (which mode?). Refuse.
    if any_assigned && !unassigned_cosigners.is_empty() {
        return Err(bad(
            "--cosigner: mix of assigned (@N=) and unassigned forms — use one or the other \
             (assigned @N= for explicit mode, unassigned for id/address search)",
        ));
    }

    // §2 open-point 7 / SPEC §5: explicit `--cosigner @N=` assignment is
    // MUTUALLY EXCLUSIVE with EITHER subset-search mode — the own-account RANGE
    // (`--own-account-max`) OR the opt-in cosigner-subset (`--search-cosigner-
    // subset`): the search PLACES keys, but @N= ASSERTS placements — combining
    // them is a contradiction. Refuse up front (BadInput).
    if any_assigned && (own_account_max.is_some() || ctx.search_cosigner_subset) {
        return Err(bad(
            "explicit @N= assignment cannot combine with subset-search \
             (--own-account-max / --search-cosigner-subset): @N= asserts the key→slot \
             placement while the subset-search resolves it. Use one or the other.",
        ));
    }

    // --- Infer the cosigner origin FAMILY (for the OWN origin default) -------
    // Any cosigner mk1 reveals the wallet's actual path family (BIP-87 vs
    // BIP-48 etc.); the own key shares it with the own --account substituted.
    let cosigner_family: Option<DerivationPath> = unassigned_cosigners
        .iter()
        .chain(assigned_cosigners.values())
        .map(|c| c.origin.clone())
        .next();

    // The explicit own-origin override (#28 phase-1 `--origin`, reused) — parsed
    // by the caller into `ctx.explicit_own_origin`.
    let explicit_own_origin = ctx.explicit_own_origin.as_ref();

    // The canonical-origin fallback (BIP-48 for canonical multisig) — used ONLY
    // in the all-own, no-`--origin`, no-cosigner-family case below. Computed
    // LAZILY: a GENERAL/thresh template (`canonical_origin().is_none()`, e.g.
    // degrade2) has no canonical origin, so an eager compute would error out
    // before checking whether `--origin`/a cosigner family already pins the own
    // origin. For the normal general case (cosigners present → `cosigner_family`
    // is `Some`) this closure is NEVER reached; when it IS reached for a general
    // template (all-own, no `--origin`, no cosigners) it yields a clear,
    // actionable error naming `--origin`.
    let canonical_fallback = || -> Result<DerivationPath, ToolkitError> {
        let op = md_codec::canonical_origin::canonical_origin(&d.tree).ok_or_else(|| {
            bad(
                "general-policy template: cannot infer the own origin family (no canonical \
                 origin, no cosigner mk1, and no --origin). Pass --origin \
                 m/<purpose>'/<coin>'/<account>' for each own key.",
            )
        })?;
        let path = origin_path_to_derivation_path(&op)?;
        // L8 — `canonical_origin(tree)` hardcodes the mainnet coin-type 0'
        // (`m/<purpose>'/0'/acct'/...`), but the bundle EMITTER writes each
        // cosigner origin at `network.coin_type()` (=1 for testnet/signet/
        // regtest). In the all-own, no-`--cosigner`, no-`--origin` case this
        // fallback is the ONLY origin source, so leaving coin at 0' on a
        // non-mainnet wallet derives every own key at the wrong path → the
        // search NEVER matches the bundle's coin-1 wallet-id (a silent
        // NO-MATCH). Substitute the network coin-type into the COIN component
        // (BIP-44/48 index 1, the second hardened element) — mirroring the
        // emitter. On mainnet (coin 0') this is the identity. md-codec's
        // `canonical_origin` is left network-agnostic (a public `.is_some()`
        // canonicity discriminator used pervasively) — the coin-type is patched
        // toolkit-side here.
        let mut comps: Vec<ChildNumber> = path.into_iter().copied().collect();
        if comps.len() >= 2 {
            comps[1] = ChildNumber::from_hardened_idx(network.coin_type()).map_err(|_| {
                bad(format!(
                    "coin-type {} out of BIP-32 hardened range",
                    network.coin_type()
                ))
            })?;
        }
        Ok(DerivationPath::from(comps))
    };

    // The own accounts. EXACT path: the `--account` LIST. OVER-SUPPLY path
    // (`--own-account-max K`): the range `0..K` (the operator does not recall
    // their own account; the subset-search resolves it). `over_supply` (own-only)
    // drives the own-anchored cardinality / enumeration; `opt_in`
    // (`--search-cosigner-subset`, P3) drives the stratified opt-in space. Both
    // build the own pool PUBLIC-ONLY and enable address-search early-exit. They
    // compose: opt-in MAY also widen the own range via `--own-account-max`.
    let over_supply = own_account_max.is_some();
    let opt_in = ctx.search_cosigner_subset;
    let own_accounts_storage: Vec<u32>;
    let own_accounts: &[u32] = if let Some(k) = own_account_max {
        own_accounts_storage = (0..k as u32).collect();
        &own_accounts_storage
    } else {
        &ctx.own_accounts
    };
    if own_accounts.is_empty() {
        return Err(bad("--account list is empty"));
    }
    // A duplicate own account would derive the SAME own key twice → a guaranteed
    // duplicate-key collision (floor 2). Reject early with a clear message.
    // (The over-supply range `0..K` is distinct by construction.)
    {
        let mut seen = std::collections::BTreeSet::new();
        for a in own_accounts {
            if !seen.insert(*a) {
                return Err(bad(format!(
                    "--account: duplicate account {a} (the own seed at one account is one key)"
                )));
            }
        }
    }

    // The own-origin closure: --origin override → the cosigner family with the
    // account substituted → the canonical (BIP-48) fallback. The fallback is
    // resolved LAZILY (only here, and only when neither --origin nor a cosigner
    // family pins the own origin) so a general template with cosigners never
    // trips the "not canonical-origin" error.
    let own_origin_for = |acct: u32| -> Result<DerivationPath, ToolkitError> {
        if let Some(o) = explicit_own_origin {
            Ok(o.clone())
        } else if let Some(fam) = &cosigner_family {
            match own_origin_from_family(fam, acct) {
                Some(o) => Ok(o),
                None => canonical_fallback(),
            }
        } else {
            // All-own multi-account, no --origin: substitute the account into
            // the canonical fallback (BIP-48 `m/48'/coin'/acct'/script'`).
            let fb = canonical_fallback()?;
            Ok(own_origin_from_family(&fb, acct).unwrap_or(fb))
        }
    };

    // --- Build the OWN candidate keys (one per own account) ------------------
    // SUBSET-SEARCH (`--own-account-max` over-supply OR `--search-cosigner-subset`
    // opt-in): derive the `K_own` own candidates via the PUBLIC-ONLY
    // `derive_accounts_xpub_only` (P0, SPEC §4.5) — the loop holds only public
    // xpubs and NEVER owns an `Xpriv` (else it would ship the un-scrubbed
    // residue). EXACT (`--account` list, no subset-search): keep the bare
    // `derive_bip32_from_entropy_at_path` byte-unchanged (the v0.60.0 path).
    let mut own_keys: Vec<CandidateKey> = Vec::with_capacity(own_accounts.len());
    if over_supply || opt_in {
        // Resolve every own origin first (shared family), then derive the shared
        // master ONCE across the range via the fan-out (own candidates share the
        // path family). The returned material is PUBLIC-ONLY.
        let origins: Vec<DerivationPath> = own_accounts
            .iter()
            .map(|&acct| own_origin_for(acct))
            .collect::<Result<_, _>>()?;
        let pubs = crate::derive_slot::derive_accounts_xpub_only(
            ctx.entropy,
            ctx.passphrase,
            ctx.derive_language,
            network,
            &origins,
        )?;
        for (origin, (account_xpub, master_fingerprint)) in origins.into_iter().zip(pubs) {
            own_keys.push(CandidateKey {
                key65: crate::synthesize::xpub_to_65(&account_xpub),
                fingerprint: master_fingerprint,
                origin,
                is_own: true,
            });
        }
    } else {
        for &acct in own_accounts {
            let origin = own_origin_for(acct)?;
            let acct_key = crate::derive_slot::derive_bip32_from_entropy_at_path(
                ctx.entropy,
                ctx.passphrase,
                ctx.derive_language,
                network,
                &origin,
            )?;
            own_keys.push(CandidateKey {
                key65: crate::synthesize::xpub_to_65(&acct_key.account_xpub),
                fingerprint: acct_key.master_fingerprint,
                origin,
                is_own: true,
            });
        }
    }

    // ---- EXPLICIT mode (all `--cosigner @N=`, no search) --------------------
    // (both subset-search modes are mutually exclusive with @N= — gated above —
    // so this only runs on the exact path.)
    if any_assigned {
        return complete_explicit_assignment(d, &own_keys, &assigned_cosigners, stderr);
    }

    // --- Build the candidate pool, OWN-FIRST ---------------------------------
    // own candidates occupy pool indices `0..k_own`, cosigner cards
    // `k_own..k_own+m` — the convention the own-anchored generator addresses.
    let k_own = own_keys.len();
    let m_cosigners = unassigned_cosigners.len();
    let mut pool: Vec<CandidateKey> = own_keys;
    pool.extend(unassigned_cosigners);

    // --- §5a premise gates (own-only) / Floor 1(ii) every slot supplied ------
    let realized_s: u128 = if opt_in {
        // OPT-IN cosigner-subset search (P3, SPEC §4.3). The cosigner count is
        // UNCERTAIN: the pool is `k_own` own + `m_cosigners` supplied cosigner
        // candidates (which MAY exceed the wallet's true M), and the search ranges
        // over `(own-subset, cosigner-subset, ordering)` per the valid j-strata.
        // `realized_s = s_opt(k_own, m_cosigners, n, sorted)` — NOT s_own, NOT n!.
        // The own-only "refuse extra cosigners" §5a gate does NOT apply here
        // (over-supplying cosigners is the whole point). §6 ceilings (refuse
        // BEFORE cap calibration, so an over-supplied pool cannot DoS):
        //   - `k_own ≤ 256` (the OWN_ACCOUNT_MAX_CEILING — enforced for
        //     `--own-account-max` above, but an `--account`-list opt-in pool
        //     reaches here without that gate, so re-assert on the realized count),
        //   - `m_cosigners ≤ 256` (a sane per-axis cosigner-candidate bound),
        //   - `s_opt ≤ REALIZED_S_MAX` (the combinatorial DoS bound),
        //   - `s_opt` overflow (None) → refuse.
        if k_own > OWN_ACCOUNT_MAX_CEILING {
            return Err(bad(format!(
                "--search-cosigner-subset: {k_own} own-account candidates exceeds the hard \
                 ceiling of {OWN_ACCOUNT_MAX_CEILING}: narrow --account / --own-account-max."
            )));
        }
        if m_cosigners > COSIGNER_SUBSET_MAX_CANDIDATES {
            return Err(bad(format!(
                "--search-cosigner-subset: {m_cosigners} supplied cosigner candidates exceeds \
                 the hard ceiling of {COSIGNER_SUBSET_MAX_CANDIDATES}: narrow the --cosigner \
                 cards (the opt-in space grows as Σ_j C(K_own,j)·C(M_sup,N−j)·N!)."
            )));
        }
        // The opt-in space is non-empty only if SOME valid j-stratum exists:
        // `j ∈ [1, min(k_own, n−1)]` with `(n − j) ≤ m_cosigners` cosigner
        // candidates. The minimum cosigners needed is `n − min(k_own, n−1)`; if
        // even the most-own stratum needs more cosigner cards than supplied, no
        // assignment can fill the slots → NO-MATCH would be the only outcome.
        // Refuse early with an actionable message (s_opt == 0).
        let sorted = crate::synthesize::is_order_independent_shape(&d.tree);
        let s = ps::s_opt(k_own, m_cosigners, n, sorted).ok_or_else(|| {
            bad("multisig template: opt-in subset-search candidate space overflow")
        })?;
        if s == 0 {
            return Err(ToolkitError::ModeViolation {
                mode: "restore",
                flag: "--search-cosigner-subset",
                message: "not enough keys to fill every slot under the opt-in cosigner-subset \
                          search: the supplied own candidates + cosigner candidates cannot fill \
                          all N slots (each assignment needs ≥1 own and ≥1 cosigner). Supply more \
                          --cosigner cards, or raise --own-account-max.",
            });
        }
        if s > REALIZED_S_MAX {
            return Err(bad(format!(
                "opt-in cosigner-subset search space ({s} candidates) exceeds the hard ceiling \
                 of {REALIZED_S_MAX}: narrow --own-account-max or supply fewer --cosigner cards."
            )));
        }
        s
    } else if over_supply {
        // OWN-ONLY subset-search. The OWN slots are `j = N − M`; the cosigner
        // cards must be EXACT (own-only — over-supply cosigners needs the P3
        // `--search-cosigner-subset`). Premise-violation table (SPEC §5a):
        //   M' > M (over-supplied cosigners): REFUSE up front (own-only).
        //   M' < M (under-supplied cosigners): NO-MATCH (cannot fill the slots).
        //   own key == a cosigner card: caught by the distinct-keys floor below.
        if m_cosigners >= n {
            // No own slot left (M' ≥ N): either over-supplied cosigners or a
            // degenerate request. Refuse with the own-only / --search-cosigner-
            // subset pointer (the §5a M'>M arm; M'==N leaves zero own slots).
            return Err(ToolkitError::ModeViolation {
                mode: "restore",
                flag: "--cosigner",
                message: "own-only subset-search needs EXACT cosigner cards (fewer than the \
                          wallet's slot count, leaving ≥1 own slot): you supplied as many or more \
                          --cosigner cards than slots. If you are unsure which/how many cosigners \
                          belong, use --search-cosigner-subset to over-supply cosigners too.",
            });
        }
        let j = n - m_cosigners; // own slots (1..=n-? ; ≥1 since m_cosigners < n)
        if k_own < j {
            // Fewer own candidates than own slots → cannot fill → NO-MATCH.
            return Err(ToolkitError::ModeViolation {
                mode: "restore",
                flag: "--own-account-max",
                message: "not enough own-account candidates to fill the own slots: raise \
                          --own-account-max, or supply the missing --cosigner card(s) (the own \
                          seed fills N − (cosigner count) slots).",
            });
        }
        // §6 hard `realized_s` ceiling (before cap calibration). s_own overflow
        // (None) → refuse.
        let sorted = crate::synthesize::is_order_independent_shape(&d.tree);
        let s = ps::s_own(k_own, j, m_cosigners, sorted)
            .ok_or_else(|| bad("multisig template: subset-search candidate space overflow"))?;
        if s > REALIZED_S_MAX {
            return Err(bad(format!(
                "own-account subset-search space ({s} candidates) exceeds the hard ceiling of \
                 {REALIZED_S_MAX}: narrow --own-account-max or supply more --cosigner cards."
            )));
        }
        s
    } else {
        // EXACT pool (v0.60.0): the supplied keys must EXACTLY fill the N slots.
        if pool.len() < n {
            return Err(ToolkitError::ModeViolation {
                mode: "restore",
                flag: "--cosigner",
                message: "not enough keys to fill every cosigner slot: the count of own keys \
                          (--from at each --account) + --cosigner keys must EQUAL the wallet's \
                          cosigner count. Supply the missing --cosigner <mk1>(s).",
            });
        }
        if pool.len() > n {
            return Err(ToolkitError::ModeViolation {
                mode: "restore",
                flag: "--account",
                message: "too many keys for the cosigner slots: the supplied own keys (one per \
                          --account) + --cosigner keys must EXACTLY equal the wallet's cosigner \
                          count. Remove the extra key(s), or specify your exact own account(s) \
                          with --account <N[,N,…]> so the supplied keys exactly fill the N slots.",
            });
        }
        // The engine enumerates exactly n! full permutations of the n pool
        // entries (pool.len() == n), so the realized space is n! = P(n, n).
        debug_assert_eq!(
            pool.len(),
            n,
            "the every-slot gate guarantees pool.len() == n"
        );
        perm_count_u128(n, n).ok_or_else(|| bad("multisig template: candidate space overflow"))?
    };

    // --- Floor 2: reject duplicate supplied keys (BEFORE the search) ---------
    // On the WHOLE pool (own candidates + cosigners) — catches an own-derived key
    // byte-identical to a cosigner card (the §5a own-as-cosigner arm) AND, for
    // the over-supply path, makes distinct subsets ⇒ distinct key SETS the
    // load-bearing collision floor (SPEC §5).
    let pool_key_blobs: Vec<[u8; 65]> = pool.iter().map(|c| c.key65).collect();
    ps::reject_duplicate_keys(&pool_key_blobs).map_err(map_search_error)?;

    // --- Select the mode + build the evaluator -------------------------------
    let id_search = ctx.expect_wallet_id.is_some();
    let addr_search = ctx.search_address.is_some();
    // SORTED carve-out (SPEC §6.1): a `sortedmulti`/`sortedmulti_a` wallet is
    // ORDER-INDEPENDENT — every key→slot permutation yields the SAME address, so
    // an address-search would find all n! placements (→ Ambiguous) even though
    // they are the SAME wallet. Collapse n! → 1 by restricting the
    // address-search to the identity placement (pool order); the address still
    // matches because order does not change a sorted wallet's scriptPubKey.
    // (id-search is NOT collapsed: `compute_wallet_policy_id` never sorts —
    // `identity.rs` — so the recorded id pins a SPECIFIC order the search must
    // still resolve. Verified: sortedmulti AB-id ≠ BA-id.)
    let sorted_shape = crate::synthesize::is_order_independent_shape(&d.tree);

    // --- The enumeration the engine ranks over (SPEC §4) ---------------------
    // OPT-IN: the stratified opt-in space `s_opt` (own-subset × cosigner-subset ×
    // ordering, SPEC §4.3). OVER-SUPPLY (own-only): the own-anchored subset space
    // `s_own` (NOT n!). For a SORTED shape BOTH subset generators drop the perm
    // factor ENUMERATION-SIDE (each subset emitted ONCE in identity order) — so
    // the v0.60.0 evaluator identity-filter must NOT also run on either subset
    // path ("identity" is per-subset there; a verbatim skip would discard every
    // non-first subset, SPEC §3 R0-r1 I-1). EXACT: the FullPermutation space, and
    // the sorted evaluator-filter stays byte-unchanged.
    let enumeration = if opt_in {
        ps::Enumeration::OptIn {
            k_own,
            m_sup: m_cosigners,
            n,
            sorted: sorted_shape,
        }
    } else if over_supply {
        let j = n - m_cosigners; // own slots (gated ≥1 above)
        ps::Enumeration::OwnAnchored {
            k_own,
            j,
            m: m_cosigners,
            sorted: sorted_shape,
        }
    } else {
        ps::Enumeration::FullPermutation { n }
    };
    // The sorted-collapse evaluator-filter applies ONLY on the EXACT path (both
    // subset paths already collapsed orderings enumeration-side).
    let apply_identity_filter = sorted_shape && !over_supply && !opt_in;

    // The fresh-descriptor builder shared by both evaluators: under an
    // assignment (slot → pool index) BUILD the keyed descriptor from the
    // permuted (key, origin, fp) triples (NEVER the carried path_decl).
    let build_candidate = |assignment: &[usize]| -> Result<md_codec::Descriptor, ToolkitError> {
        let triples: Vec<crate::synthesize::TemplateSlotKey> = assignment
            .iter()
            .map(|&pi| {
                let c = &pool[pi];
                crate::synthesize::TemplateSlotKey {
                    key65: c.key65,
                    fingerprint: c.fingerprint.to_bytes(),
                    origin: crate::synthesize::derivation_path_to_origin_path(&c.origin),
                }
            })
            .collect();
        crate::synthesize::build_keyed_template_descriptor(d, &triples)
    };

    let outcome = if id_search {
        // ---- id-search (strong-prefix sized to the realized S) -------------
        let prefix_hex = ctx.expect_wallet_id.as_deref().unwrap();
        let prefix = decode_wallet_id_prefix(prefix_hex)?;
        ps::validate_prefix_strength(prefix.len(), realized_s).map_err(map_search_error)?;
        let evaluator = |assignment: &[usize], _addr_idx: u64| -> bool {
            match build_candidate(assignment) {
                Ok(cand) => match md_codec::compute_wallet_policy_id(&cand) {
                    Ok(id) => {
                        let b = id.as_bytes();
                        b.len() >= prefix.len() && b[..prefix.len()] == prefix[..]
                    }
                    Err(_) => false,
                },
                Err(_) => false,
            }
        };
        // id-search / prefix-id NEVER gets early-exit (full-scan ambiguity
        // certification — SPEC §4.4).
        run_capped_search(
            &evaluator,
            &enumeration,
            ps::SearchMode::Id,
            realized_s,
            false,
            ctx.accept_search_time.as_deref(),
            stderr,
        )?
    } else if addr_search {
        // ---- address-search (full scriptPubKey; collision-free) ------------
        let addr_str = ctx.search_address.as_deref().unwrap();
        let target_spk = address_to_script_pubkey(addr_str, network)?;
        if ctx.search_addr_max <= ctx.search_addr_min {
            return Err(bad(
                "--search-addr-max must be greater than --search-addr-min",
            ));
        }
        let range = ps::AddressRange {
            min: ctx.search_addr_min,
            max: ctx.search_addr_max,
            chains: ctx.search_chain.to_scope(),
        };
        let scope = ctx.search_chain;
        let evaluator = |assignment: &[usize], addr_idx: u64| -> bool {
            // SORTED collapse (EXACT path only): only the identity placement is
            // evaluated (all placements are the same wallet). On the over-supply
            // path the generator already collapsed orderings enumeration-side, so
            // this filter is OFF there (else it would discard every non-first
            // subset — SPEC §3 R0-r1 I-1).
            if apply_identity_filter && !assignment.iter().enumerate().all(|(i, &v)| i == v) {
                return false;
            }
            // The engine's address_index encodes (idx << 1) | chain_bit (M2).
            let idx = (addr_idx >> 1) as u32;
            let chain_bit = (addr_idx & 1) as u32;
            // For a single-chain scope the bit is always 0; map it to the real
            // chain (0 receive / 1 change). For Both, the bit IS the chain.
            let chain = match scope {
                CliSearchChain::Receive => 0,
                CliSearchChain::Change => 1,
                CliSearchChain::Both => chain_bit,
            };
            let Ok(cand) = build_candidate(assignment) else {
                return false;
            };
            let Ok(desc_str) = candidate_descriptor_string(&cand, network) else {
                return false;
            };
            let Ok(parsed) = MsDescriptor::<DescriptorPublicKey>::from_str(&desc_str) else {
                return false;
            };
            match script_pubkey_at(&parsed, chain, idx) {
                Ok(spk) => spk == target_spk,
                Err(_) => false,
            }
        };
        // address-search is collision-free (full scriptPubKey) ⇒ a match is
        // provably unique ⇒ first-match early-exit is SAFE on EITHER subset path
        // (own-only over-supply OR opt-in cosigner-subset) — distinct subsets ⇒
        // distinct key SETS ⇒ distinct scriptPubKey (via the whole-pool
        // distinct-keys floor, SPEC §4.4 contract). The EXACT path keeps `false`
        // (byte-unchanged full-scan + 2nd-match-ambiguity certification — the I-5
        // regression guard). The sorted EXACT path also keeps `false` (it
        // collapses to a single identity candidate, so full-scan is cheap and
        // unchanged).
        let early_exit = over_supply || opt_in;
        run_capped_search(
            &evaluator,
            &enumeration,
            ps::SearchMode::Address(range),
            realized_s,
            early_exit,
            ctx.accept_search_time.as_deref(),
            stderr,
        )?
    } else {
        // ---- no mode given → refuse (SPEC §2 precedence (d)) ---------------
        return Err(ToolkitError::ModeViolation {
            mode: "restore",
            flag: "--md1",
            message: "supply a recorded --expect-wallet-id, a --search-address, or explicit \
                      --cosigner @N= assignments to complete a multisig template (the search \
                      needs a target to verify the unique key→slot assignment against).",
        });
    };

    // --- Resolve the outcome (SPEC §6.2 floors) ------------------------------
    let assignment = match outcome {
        ps::SearchOutcome::Unique { assignment, .. } => assignment,
        ps::SearchOutcome::None => {
            writeln!(stderr, "✗ NO MATCH").map_err(ToolkitError::Io)?;
            return Err(ToolkitError::RestoreMismatch {
                reference: "multisig-template-search",
                derived: "no key→slot assignment of the supplied keys".to_string(),
                expected: "the recorded wallet (--expect-wallet-id / --search-address)".to_string(),
                slot: None,
            });
        }
        ps::SearchOutcome::Ambiguous => {
            writeln!(stderr, "✗ AMBIGUOUS").map_err(ToolkitError::Io)?;
            return Err(bad(
                "multisig template completion is AMBIGUOUS: two or more key→slot assignments \
                 match — the wallet is not uniquely determined. Use --search-address (full \
                 scriptPubKey) or a longer --expect-wallet-id.",
            ));
        }
    };

    // --- Build the completed watch-only wallet (the caller emits/binds) ------
    let completed = build_candidate(&assignment)?;
    Ok(MultisigCompletionOutcome {
        completed,
        pool,
        assignment,
    })
}

/// Decode a `--cosigner` card — one or more `mk1` chunks, or a single bare xpub
/// — into a `CandidateKey`. mk1 carries the origin (fingerprint + path); a bare
/// xpub has no origin so it gets an EMPTY origin (the search still verifies via
/// id/addr; the mk1 is the norm). The fallible inner is `try_decode_cosigner_card`.
fn decode_cosigner_card(
    chunks: &[&str],
    network: CliNetwork,
) -> Result<CandidateKey, ToolkitError> {
    let first = chunks.first().copied().unwrap_or("");
    if first.to_lowercase().starts_with("mk1") {
        try_decode_cosigner_card(chunks)
    } else {
        if chunks.len() != 1 {
            return Err(bad("--cosigner: a bare xpub is a single value"));
        }
        let xpub = Xpub::from_str(first).map_err(|e| bad(format!("--cosigner xpub parse: {e}")))?;
        let _ = network; // network kind is informational for a watch-only xpub
        Ok(CandidateKey {
            key65: crate::synthesize::xpub_to_65(&xpub),
            fingerprint: Fingerprint::default(),
            origin: DerivationPath::master(),
            is_own: false,
        })
    }
}

/// Decode a complete mk1 card (all its chunks) into a `CandidateKey`. Returns
/// the codec error verbatim (so the greedy grouper can detect an incomplete
/// set, and the leftover-chunks path can surface the real "received K chunks,
/// header declares total = M" message).
fn try_decode_cosigner_card(chunks: &[&str]) -> Result<CandidateKey, ToolkitError> {
    let kc = mk_codec::decode(chunks).map_err(|e| bad(format!("--cosigner mk1 decode: {e}")))?;
    Ok(CandidateKey {
        key65: crate::synthesize::xpub_to_65(&kc.xpub),
        fingerprint: kc.origin_fingerprint.unwrap_or_default(),
        origin: kc.origin_path,
        is_own: false,
    })
}

/// Parse a `--accept-search-time` duration: a positive integer with a `s`/`m`/
/// `min`/`h` suffix (e.g. `90s`, `30m`, `2h`). No suffix → seconds. The forced
/// acknowledgment for searches above the 1-hour ceiling (SPEC §6.4).
fn parse_search_duration(s: &str) -> Result<std::time::Duration, ToolkitError> {
    let s = s.trim();
    let (num_str, mult): (&str, u64) = if let Some(p) = s.strip_suffix("min") {
        (p, 60)
    } else if let Some(p) = s.strip_suffix('h') {
        (p, 3600)
    } else if let Some(p) = s.strip_suffix('m') {
        (p, 60)
    } else if let Some(p) = s.strip_suffix('s') {
        (p, 1)
    } else {
        (s, 1)
    };
    let n: u64 = num_str.trim().parse().map_err(|_| {
        bad(format!(
            "--accept-search-time `{s}`: expected e.g. 90s/30m/2h"
        ))
    })?;
    Ok(std::time::Duration::from_secs(n.saturating_mul(mult)))
}

/// P((pool), n) — the number of injective placements of `pool` distinct keys
/// into `n` slots: `pool! / (pool-n)!`. `None` on `u128` overflow or `pool < n`.
fn perm_count_u128(pool: usize, n: usize) -> Option<u128> {
    if pool < n {
        return None;
    }
    let mut p: u128 = 1;
    for i in 0..n {
        p = p.checked_mul((pool - i) as u128)?;
    }
    Some(p)
}

/// Map a `permutation_search::SearchError` to a `ToolkitError`. Floor errors
/// (duplicate keys, weak prefix, time ceiling) are funds-safety refusals (exit
/// 4); the structural ones (empty/too-large) are bad input.
fn map_search_error(e: mnemonic_toolkit::permutation_search::SearchError) -> ToolkitError {
    use mnemonic_toolkit::permutation_search::SearchError as SE;
    match e {
        SE::DuplicateKeys { .. } | SE::PrefixTooShort { .. } => ToolkitError::RestoreMismatch {
            reference: "multisig-template-floor",
            derived: e.to_string(),
            expected: "distinct cosigner keys + a strong --expect-wallet-id prefix".to_string(),
            slot: None,
        },
        SE::SearchTimeExceedsCeiling { .. } | SE::AcceptSearchTimeTooLow { .. } => {
            bad(e.to_string())
        }
        SE::EmptySearchSpace | SE::SearchSpaceTooLarge { .. } => bad(e.to_string()),
    }
}

/// Run the engine under the adaptive cap (SPEC §6.4): calibrate per-candidate
/// cost, decide silent / progress / refuse-unless-acknowledged, then search.
///
/// `enumeration` is the rank space the engine drives — `FullPermutation` for the
/// v0.60.0 EXACT pool, `OwnAnchored` for the over-supply subset-search (SPEC
/// §4). `realized_perms` is its cardinality (`n!` / `s_own`). `early_exit` is the
/// §4.4 knob: `true` ONLY for the over-supply collision-free address-search; the
/// exact path + all id/prefix-id paths pass `false` (byte-unchanged).
#[allow(clippy::too_many_arguments)]
fn run_capped_search<Ev, E: Write>(
    evaluator: &Ev,
    enumeration: &mnemonic_toolkit::permutation_search::Enumeration,
    mode: mnemonic_toolkit::permutation_search::SearchMode,
    realized_perms: u128,
    early_exit: bool,
    accept_search_time: Option<&str>,
    stderr: &mut E,
) -> Result<mnemonic_toolkit::permutation_search::SearchOutcome, ToolkitError>
where
    Ev: mnemonic_toolkit::permutation_search::CandidateEvaluator,
{
    use mnemonic_toolkit::permutation_search as ps;
    // The EXHAUSTIVE candidate count: the realized permutation count × the
    // address range (id-search has outer=1). The cap (SPEC §6.4) must estimate
    // the WHOLE scan, not just the permutation count — an address-search over a
    // wide range can dwarf the permutations.
    let outer: u128 = match mode {
        ps::SearchMode::Id => 1,
        ps::SearchMode::Address(range) => u128::from(range.outer_count()),
    };
    let realized_total = realized_perms.saturating_mul(outer);
    // Calibrate per-candidate cost on this machine, then cap.
    let per = ps::calibrate_per_candidate(evaluator, enumeration.n(), 64, 0);
    let accept = match accept_search_time {
        Some(s) => Some(parse_search_duration(s)?),
        None => None,
    };
    let total = u64::try_from(realized_total).unwrap_or(u64::MAX);
    match ps::cap_decision(total, per, accept).map_err(map_search_error)? {
        ps::CapDecision::RunSilent { .. } => {}
        ps::CapDecision::RunWithProgress { estimate } => {
            let _ = writeln!(
                stderr,
                "searching {realized_total} candidate assignment(s) (est. ≤ {estimate:?})…"
            );
        }
    }
    // The EXACT path + every id/prefix-id path pass `early_exit=false` (the
    // v0.60.0 full-scan-with-2nd-match ambiguity certification, byte-unchanged);
    // the over-supply collision-free address-search opts into `true` (SPEC §4.4).
    ps::search_enumerated(enumeration, evaluator, mode, early_exit).map_err(map_search_error)
}

/// Convert a candidate keyed `md_codec::Descriptor` to its watch-only miniscript
/// descriptor STRING via the #25 per-`@N` multipath reconstruction
/// (`faithful_multisig_descriptor`'s engine), so the address derivation reads
/// the SAME fresh assembly the id does. `pub(crate)` so `verify-bundle`'s
/// multisig-template recompose reuses the IDENTICAL engine restore emits with
/// (funds-safety parity across the two surfaces).
pub(crate) fn candidate_descriptor_string(
    cand: &md_codec::Descriptor,
    network: CliNetwork,
) -> Result<String, ToolkitError> {
    faithful_multisig_descriptor(cand, network)
}

/// EXPLICIT mode (Mode B, SPEC §4.3): all `--cosigner @N=` assigned, no search.
/// Build directly from the asserted assignment + fire the §3.4 warning. No
/// verification (operator's risk) — but the every-slot gate + distinct-keys gate
/// still apply. Returns the neutral [`MultisigCompletionOutcome`] (the caller
/// emits/binds).
fn complete_explicit_assignment<E: Write>(
    d: &md_codec::Descriptor,
    own_keys: &[CandidateKey],
    assigned: &std::collections::BTreeMap<u8, CandidateKey>,
    stderr: &mut E,
) -> Result<MultisigCompletionOutcome, ToolkitError> {
    use mnemonic_toolkit::permutation_search as ps;
    let n = d.n as usize;
    if own_keys.len() != 1 {
        return Err(bad(
            "explicit --cosigner @N= mode supports a single own account (one --account)",
        ));
    }
    let own = &own_keys[0];

    // Place keys: each @N from `assigned`, the own key at the one slot it isn't
    // covering (inferred by the slot not in `assigned`). Every slot must be
    // covered exactly once.
    let mut slots: Vec<Option<CandidateKey>> = vec![None; n];
    for (&pos, ck) in assigned {
        slots[pos as usize] = Some(ck.clone());
    }
    // The own key fills the single uncovered slot.
    let empty: Vec<usize> = (0..n).filter(|i| slots[*i].is_none()).collect();
    if empty.len() != 1 {
        return Err(ToolkitError::ModeViolation {
            mode: "restore",
            flag: "--cosigner",
            message: "explicit --cosigner @N= mode requires assigning every slot but the own \
                      one: assign N-1 cosigners with @N= and let --from fill the remaining slot.",
        });
    }
    slots[empty[0]] = Some(own.clone());

    let placed: Vec<CandidateKey> = slots.into_iter().map(|s| s.unwrap()).collect();
    // Distinct-keys floor.
    let blobs: Vec<[u8; 65]> = placed.iter().map(|c| c.key65).collect();
    ps::reject_duplicate_keys(&blobs).map_err(map_search_error)?;

    // §3.4 warning: a wrong explicit assignment is UNVERIFIED.
    let _ = writeln!(
        stderr,
        "warning: explicit --cosigner @N= mode builds the wallet from the ASSERTED key→slot \
         assignment WITHOUT verifying it against a recorded id/address. A wrong assignment \
         produces a wrong wallet silently. Record + check --expect-wallet-id or a receive address."
    );

    let triples: Vec<crate::synthesize::TemplateSlotKey> = placed
        .iter()
        .map(|c| crate::synthesize::TemplateSlotKey {
            key65: c.key65,
            fingerprint: c.fingerprint.to_bytes(),
            origin: crate::synthesize::derivation_path_to_origin_path(&c.origin),
        })
        .collect();
    let completed = crate::synthesize::build_keyed_template_descriptor(d, &triples)?;
    let assignment: Vec<usize> = (0..n).collect();
    Ok(MultisigCompletionOutcome {
        completed,
        pool: placed,
        assignment,
    })
}

/// Build + emit the completed watch-only multisig wallet (text or JSON). The
/// `assignment` maps slot → `pool` index; `pool[assignment[i]]` is the key at
/// `@i`. Prints the descriptor + first receive addresses + the completed
/// `WalletPolicyId`.
fn emit_completed_multisig<W: Write, E: Write>(
    cand: &md_codec::Descriptor,
    pool: &[CandidateKey],
    assignment: &[usize],
    args: &RestoreArgs,
    network: CliNetwork,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let descriptor = candidate_descriptor_string(cand, network)?;
    let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(&descriptor)
        .map_err(|e| bad(format!("completed descriptor parse: {e}")))?;
    if parsed.to_string() != descriptor {
        return Err(bad(
            "completed multisig descriptor does not survive a parse→print round-trip; refusing.",
        ));
    }
    let first_recv = crate::derive_address::derive_receive_addresses(
        &parsed,
        args.count,
        network.to_bitcoin_network(),
    )?;

    // The completed WalletPolicyId (surface it — funds-safety, SPEC §6.2).
    let id = md_codec::compute_wallet_policy_id(cand).map_err(ToolkitError::from)?;
    let id_hex = hex::encode(id.as_bytes());

    // Own position (for the annotation): the slot whose pool entry is_own.
    let own_pos: Option<usize> = assignment.iter().position(|&pi| pool[pi].is_own);

    let stdout_content: String = if args.json {
        let envelope = json!({
            "network": network.human_name(),
            "completed_from": "multisig-template-md1",
            "wallet_policy_id": id_hex,
            "own_position": own_pos,
            "wallets": [json!({
                "descriptor": descriptor,
                "first_addresses": first_recv,
            })],
        });
        format!(
            "{}\n",
            serde_json::to_string(&envelope)
                .map_err(|e| bad(format!("json serialization: {e}")))?
        )
    } else {
        let mut s = String::new();
        s.push_str("multisig wallet completed from template:\n");
        s.push_str(&format!("  descriptor: {descriptor}\n"));
        for addr in &first_recv {
            s.push_str(&format!("  first recv: {addr}\n"));
        }
        s
    };
    stdout
        .write_all(stdout_content.as_bytes())
        .map_err(ToolkitError::Io)?;

    let _ = writeln!(stderr, "✓ wallet-id (completed): {id_hex}");
    if let Some(p) = own_pos {
        let _ = writeln!(stderr, "  your seed completes cosigner slot @{p}");
    }
    Ok(0)
}

/// Decode a bech32/base58 address string into its scriptPubKey bytes for the
/// address-search target. The network must match `--network`.
fn address_to_script_pubkey(addr: &str, network: CliNetwork) -> Result<Vec<u8>, ToolkitError> {
    let parsed = bitcoin::Address::from_str(addr.trim())
        .map_err(|e| bad(format!("--search-address parse: {e}")))?
        .require_network(network.to_bitcoin_network())
        .map_err(|e| bad(format!("--search-address network mismatch: {e}")))?;
    Ok(parsed.script_pubkey().to_bytes())
}

/// #28 — decode an `--expect-wallet-id` hex prefix into bytes. Accepts an
/// even-length lowercase/uppercase hex string (any length ≥1 byte). Odd-length
/// or non-hex → BadInput (exit 1).
fn decode_wallet_id_prefix(s: &str) -> Result<Vec<u8>, ToolkitError> {
    let t = s.trim();
    if t.is_empty() {
        return Err(bad("--expect-wallet-id must not be empty"));
    }
    let bytes = hex::decode(t).map_err(|e| {
        bad(format!(
            "--expect-wallet-id must be an even-length hex prefix of the WalletPolicyId: {e}"
        ))
    })?;
    // The WalletPolicyId is 16 bytes; a longer "prefix" can never match (it is
    // not a prefix of a 16-byte id). Reject it here with a clear length message
    // rather than letting it fall through to the generic MISMATCH path (M2).
    if bytes.len() > 16 {
        return Err(bad(format!(
            "--expect-wallet-id prefix is {} bytes; the WalletPolicyId is only 16 bytes — \
             supply a prefix of at most 16 bytes (32 hex chars)",
            bytes.len()
        )));
    }
    Ok(bytes)
}

/// Build the importable wallet-software payload for a single template via the
/// `export-wallet` `WalletFormatEmitter` dispatch (§3.5; Task 2.1).
///
/// Mirrors the 16-field `EmitInputs` ctor + dispatch in `cmd::export_wallet::run`
/// (`export_wallet.rs`). NOTE: `EmitInputs.script_type` is
/// `wallet_export::WalletScriptType` — a DIFFERENT enum from the
/// `convert::ScriptType` used for address rendering — so we use
/// `wallet_export::script_type_from_template`, not the convert-side helper.
fn build_import_payload(
    format: CliExportFormat,
    row: &WalletRow,
    network: CliNetwork,
    account: u32,
) -> Result<String, ToolkitError> {
    let script_type = wallet_export::script_type_from_template(&row.template);
    let wallet_name = format!("{}-{}", row.template.human_name(), account);
    let inputs = EmitInputs {
        canonical_descriptor: CheckedDescriptor::new(&row.descriptor)?,
        resolved_slots: std::slice::from_ref(&row.slot),
        template: Some(row.template),
        script_type,
        network,
        account,
        // Single-sig: no multisig threshold.
        threshold: None,
        threshold_user_supplied: false,
        master_xpub_at_0: row.slot.master_xpub,
        wallet_name: &wallet_name,
        wallet_name_is_non_default: false,
        taproot_internal_key: None,
        range: (0, 999),
        // v0.47.3: genesis rescan (`0`) — the correct anchor for a recovery
        // workflow; matches export-wallet's default. restore has no --timestamp
        // flag. SPEC_timestamp_default_zero.
        timestamp: TimestampArg::Unix(0),
        bitcoin_core_version: 25,
        bsms_form: BsmsForm::default(),
    };

    // Shared 4-way dispatch (collect_missing-first → emit) via the canonical
    // `emit_payload` helper (FOLLOWUP `restore-emit-dispatch-3way-dedup`; recon
    // corrected "3-way" → "4-way"). This reuses the export-wallet missing-info
    // channel verbatim (so e.g. `--format specter` refuses identically) AND
    // unifies the single-sig `coldcard-multisig` refusal: it now routes through
    // the helper's 6-variant template `_ =>` arm ("requires a multisig
    // --template …") instead of the old restore-specific "requires a multisig
    // wallet" string — exit 1 (BadInput) either way (the upfront single-sig
    // gate at the top of `run` already rejects multisig `--template`).
    crate::cmd::export_wallet::emit_payload(&inputs, format)
}

/// §3 outcome for a `Tag::Tr` wallet-policy md1: which reconstruction arm,
/// and the internal ("trunk") key to thread (NUMS or a real cosigner key).
enum TaprootRestore {
    /// Single-leaf `multi_a`/`sortedmulti_a` — the byte-identical template
    /// path (`build_descriptor_string`). NUMS or distinct-trunk Cosigner(idx).
    Template(CliTemplate, TaprootInternalKey),
    /// General single-leaf or depth-1 two-leaf `tr(<internal>,…)` policy — the
    /// faithful arm (`faithful_multisig_descriptor`), v0.55.1 (T3-partial of
    /// FOLLOWUP `restore-general-and-multi-leaf-taproot-roundtrip`); v0.55.3
    /// extends it to a non-NUMS (real cosigner) trunk key.
    GeneralFaithful(TaprootInternalKey),
}

/// Classify a taproot wallet-policy md1 tree for restore. The single-leaf
/// `multi_a`/`sortedmulti_a` Template path stays byte-identical (routing
/// around md-codec's `to_miniscript`, which errors on a root `SortedMultiA`);
/// the GeneralFaithful arm re-enters `to_miniscript` via
/// `faithful_multisig_descriptor`, so its blockers are pre-gated here.
/// Supports `is_nums:true` (NUMS) AND `is_nums:false` (real cosigner trunk
/// key), the latter for general single-leaf/depth-1 (route-around) and
/// distinct-trunk multisig (Template); the `@-in-both` shape (trunk key also a
/// leaf key) refuses (`restore-non-nums-tr-internal-key-also-in-leaf`).
///
/// The GeneralFaithful arm is gated CONSERVATIVELY + STRUCTURALLY (never on
/// Display behavior):
/// - depth ≥2 (any `TapTree` child of a `TapTree`) refuses — the pinned
///   miniscript 95fdd1c mis-Displays a LEFT-child `TapTree` (`{{a,b,c}}`),
///   and a right-spine shape that happens to Display fine must not create a
///   Display-luck accepted set (FOLLOWUP
///   `upstream-miniscript-taptree-depth2-display-asymmetry`; lift the gate
///   when the miniscript #953 fix releases);
/// - `sortedmulti_a` anywhere under a `TapTree` refuses — md-codec's
///   `to_miniscript` cannot render it as a non-root tap leaf (FOLLOWUP
///   `md-codec-sortedmulti-a-to-miniscript-rendering-gap`).
fn classify_taproot_restore(tree: &md_codec::tree::Node) -> Result<TaprootRestore, ToolkitError> {
    use md_codec::tree::Body;
    let (inner, internal_key) = match &tree.body {
        Body::Tr {
            is_nums: true,
            tree: Some(inner),
            ..
        } => (inner, TaprootInternalKey::Nums),
        Body::Tr {
            is_nums: false,
            key_index,
            tree: Some(inner),
        } => {
            // Read the real trunk key off the wire — no inference. (key_index
            // is a 0..n placeholder index into the cosigner table; u8, and
            // TaprootInternalKey::Cosigner is also u8 — no cast.)
            (inner, TaprootInternalKey::Cosigner(*key_index))
        }
        Body::Tr { tree: None, .. } => {
            return Err(bad(
                "--md1 taproot tree has no script leaf (keypath-only tr is single-sig, not multisig)",
            ));
        }
        _ => {
            return Err(bad(
                "--md1: internal error — taproot handler on a non-Tr tree",
            ))
        }
    };
    match inner.tag {
        md_codec::Tag::MultiA => {
            refuse_at_in_both(&internal_key, inner)?;
            Ok(TaprootRestore::Template(
                CliTemplate::TrMultiA,
                internal_key,
            ))
        }
        md_codec::Tag::SortedMultiA => {
            refuse_at_in_both(&internal_key, inner)?;
            Ok(TaprootRestore::Template(
                CliTemplate::TrSortedMultiA,
                internal_key,
            ))
        }
        _ => {
            if subtree_contains_sortedmulti_a(inner) {
                return Err(ToolkitError::ModeViolation {
                    mode: "restore",
                    flag: "--md1",
                    message: "taproot md1 carries sortedmulti_a under a tap-script tree — md-codec cannot yet render it back as a non-root tap leaf (FOLLOWUP md-codec-sortedmulti-a-to-miniscript-rendering-gap); the engraved card remains a faithful backup",
                });
            }
            ensure_taptree_depth_le_one(inner)?;
            Ok(TaprootRestore::GeneralFaithful(internal_key))
        }
    }
}

/// Refuse the `@-in-both` shape `tr(@i, multi_a/sortedmulti_a(k, …@i…))` where
/// the non-NUMS trunk key index is ALSO one of the leaf key indices. This is a
/// STRUCTURAL classify-time precondition — NEVER a post-reconstruction Display
/// check — and it is the funds-safety crux of the non-NUMS taproot cycle.
///
/// WHY structural, not Display: the Template path's `Cosigner(idx)` mode
/// reconstructs the leaf as `{all cosigners EXCEPT idx}` WITHOUT lowering `k`
/// (`wallet_export/pipeline.rs:134-156`). For an `@-in-both` card it therefore
/// emits a leaf that has dropped the trunk key. When the original leaf had `n ≥
/// 3` keys, the dropped-trunk leaf is still a VALID `k ≤ n` multisig, so the
/// reconstruction SUCCEEDS and prints a DIFFERENT, silently-wrong multisig at a
/// DIFFERENT address. The Display-fidelity guard (`restore.rs`, parse→print
/// before address derivation) provably CANNOT catch this: the Template path's
/// output is its own re-print (`pipeline.rs:28-31` `from_str().to_string()`), so
/// a wrong-but-self-consistent leaf passes parse→print. The only safe net is to
/// refuse the shape here, before any reconstruction. (For `n = 2` the dropped-
/// trunk leaf happens to be a `k > n` multisig that miniscript rejects
/// downstream — but that is coincidental, not a guarantee, so the guard refuses
/// every `@-in-both` shape uniformly.)
///
/// NUMS trunks (`is_nums:true` → `TaprootInternalKey::Nums`) are not in a
/// cosigner slot, so they never trip this. General-arm leaves never reach this
/// helper (they reconstruct via the route-around, which reads the ACTUAL tree).
fn refuse_at_in_both(
    internal_key: &TaprootInternalKey,
    leaf: &md_codec::tree::Node,
) -> Result<(), ToolkitError> {
    use md_codec::tree::Body;
    // Cosigner(u8); indices: Vec<u8> — all u8, no casts.
    if let TaprootInternalKey::Cosigner(i) = internal_key {
        if let Body::MultiKeys { indices, .. } = &leaf.body {
            if indices.iter().any(|&idx| idx == *i) {
                return Err(ToolkitError::ModeViolation {
                    mode: "restore",
                    flag: "--md1",
                    message: "taproot md1 has a non-NUMS internal (trunk) key that is also a leaf key (@-in-both) — the engraved card is a faithful backup, but the toolkit will not reconstruct this shape: the trunk key already spends unilaterally via the key path, so re-using it inside the script-path multisig is a degenerate construction. Refusing rather than emit a silently-different multisig (WONTFIX restore-non-nums-tr-internal-key-also-in-leaf)",
                });
            }
        }
    }
    Ok(())
}

/// `true` iff `Tag::SortedMultiA` occurs anywhere in the subtree (the §3
/// pre-gate for the GeneralFaithful arm — a clear refusal instead of
/// md-codec's converter-internal "must be a tap-leaf root child" error).
/// A single-leaf root `SortedMultiA` never reaches this (Template arm first).
fn subtree_contains_sortedmulti_a(n: &md_codec::tree::Node) -> bool {
    use md_codec::tree::Body;
    if n.tag == md_codec::Tag::SortedMultiA {
        return true;
    }
    match &n.body {
        Body::Children(c) => c.iter().any(subtree_contains_sortedmulti_a),
        Body::Variable { children, .. } => children.iter().any(subtree_contains_sortedmulti_a),
        Body::Tr { tree, .. } => tree.as_deref().is_some_and(subtree_contains_sortedmulti_a),
        _ => false,
    }
}

/// Refuse a tap-script tree of depth ≥2 — STRUCTURAL on the md1 Node tree
/// (never on Display behavior; see `classify_taproot_restore`). md-codec
/// taptrees are strictly binary, so "no `TapTree` child of a `TapTree`" ⟺
/// depth ≤1 ⟺ ≤2 leaves. Spine-only walk: a `TapTree` under a non-TapTree
/// leaf is not constructible (md-codec decode errors first).
fn ensure_taptree_depth_le_one(inner: &md_codec::tree::Node) -> Result<(), ToolkitError> {
    use md_codec::tree::Body;
    if inner.tag != md_codec::Tag::TapTree {
        // A single general leaf — no tree nesting possible.
        return Ok(());
    }
    // md-codec decode guarantees a TapTree body is EXACTLY 2 children
    // (tree.rs `read_node` Tag::TapTree arm), so the ≠2 refusal below is
    // defensive-only — but a malformed tree must REFUSE, never be silently
    // treated as a leaf (unlike the test-only `count_tap_leaves` pattern).
    let children = match &inner.body {
        Body::Children(c) if c.len() == 2 => c,
        _ => {
            return Err(ToolkitError::ModeViolation {
                mode: "restore",
                flag: "--md1",
                message: "taproot md1 tap-script tree node is malformed (a TapTree must carry exactly 2 children); refusing to reconstruct",
            })
        }
    };
    if children.iter().any(|c| c.tag == md_codec::Tag::TapTree) {
        return Err(ToolkitError::ModeViolation {
            mode: "restore",
            flag: "--md1",
            message: "taproot tree depth ≥2 (≥3 leaves) is not yet restorable — the pinned miniscript mis-prints nested taptrees (FOLLOWUP upstream-miniscript-taptree-depth2-display-asymmetry); the engraved card remains a faithful backup",
        });
    }
    Ok(())
}

/// Build the importable wallet payload for a MULTISIG `restore --md1 --format`
/// (FOLLOWUP `restore-multisig-format-payloads`). Mirrors `export-wallet`'s
/// `EmitInputs` (`export_wallet.rs:560-577`) using the reconstructed
/// (`template`, `slots`, `k`, `descriptor`); the dispatch goes through the
/// shared `emit_payload` helper (FOLLOWUP `restore-emit-dispatch-3way-dedup`,
/// the former 4-way dedup). `threshold_user_supplied: true` is LOAD-BEARING:
/// `k` from the md1 is authoritative, and `sparrow.rs` `collect_missing`
/// refuses a multisig template (`MissingField::Threshold`) when it is false.
///
/// `taproot_internal_key` is `Some(Nums)` or `Some(Cosigner(idx))` for a
/// taproot md1 (threaded from the §3 classification), `None` for wsh/sh-wsh —
/// so the `--format` payload's emitted descriptor carries the correct internal
/// key. (Non-NUMS real-trunk support: v0.55.3.)
#[allow(clippy::too_many_arguments)]
fn build_multisig_import_payload(
    format: CliExportFormat,
    template: Option<CliTemplate>,
    slots: &[ResolvedSlot],
    k: Option<u8>,
    descriptor: &str,
    network: CliNetwork,
    account: u32,
    taproot_internal_key: Option<TaprootInternalKey>,
) -> Result<String, ToolkitError> {
    // General arm (`template == None`): descriptor-mode `EmitInputs` mirroring
    // `export-wallet --descriptor` — `script_type_from_descriptor` + the
    // `"imported-descriptor"` default name. Descriptor-driven formats
    // (bitcoin-core/descriptor/bsms) emit FAITHFULLY; `bip388` emits faithfully
    // for a multipath (`/<0;1>/*`) card and refuses a wildcard-only one (BIP-388
    // wallet policies require the multipath suffix) — and refuses a general-tr
    // card too (the NUMS internal key is a bare x-only `Single` with no
    // multipath suffix). Template-requiring k-of-n formats
    // (coldcard/jade/electrum/sparrow) refuse via their existing
    // `template`/`is_multisig` branches; `specter` refuses via its
    // `collect_missing → MissingField::WalletName` path (the general arm's
    // default `"imported-descriptor"` name is rejected), not a template gate.
    // `green` needs the EXPLICIT refusal
    // below for the general-tr arm (R0 I1, v0.55.1):
    // `script_type_from_descriptor` classifies a general tr without a
    // `multi_a(` substring as `P2tr` — taproot SINGLESIG — so green's
    // `is_multisig` gate would otherwise EMIT a "singlesig" payload for a
    // tap-script-tree policy. (The wsh-general arm classifies `P2wshMulti`
    // and the multi_a-bearing tr arm `P2trMulti` — both already refused by
    // green's own gate.)
    let (script_type, wallet_name) = match template {
        Some(t) => (
            wallet_export::script_type_from_template(&t),
            format!("{}-{}", t.human_name(), account),
        ),
        None => {
            let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(descriptor)
                .map_err(|e| bad(format!("--md1 reconstructed descriptor parse: {e}")))?;
            let script_type = wallet_export::script_type_from_descriptor(&parsed)?;
            if format == CliExportFormat::Green
                && script_type == wallet_export::WalletScriptType::P2tr
            {
                return Err(ToolkitError::BadInput(
                    "--format green cannot emit a taproot policy descriptor — Green's file-import surface is singlesig-only, and this md1 restores a tap-script-tree policy. Use --format bitcoin-core or --format descriptor for a watch-only import.".into(),
                ));
            }
            if format == CliExportFormat::Bip388
                && matches!(
                    script_type,
                    wallet_export::WalletScriptType::P2tr
                        | wallet_export::WalletScriptType::P2trMulti
                )
            {
                return Err(ToolkitError::BadInput(
                    "--format bip388 cannot express this taproot policy as a BIP-388 wallet policy — a tap-script-tree reconstructed via the general route-around has no named-template form. Use --format descriptor or --format bitcoin-core for a watch-only import. (A distinct-trunk tr-multisig md1 DOES export bip388 via its template path.)".into(),
                ));
            }
            (script_type, "imported-descriptor".to_string())
        }
    };
    let inputs = EmitInputs {
        canonical_descriptor: CheckedDescriptor::new(descriptor)?,
        resolved_slots: slots,
        template,
        script_type,
        network,
        account,
        threshold: k,
        threshold_user_supplied: k.is_some(),
        master_xpub_at_0: slots.first().and_then(|s| s.master_xpub),
        wallet_name: &wallet_name,
        wallet_name_is_non_default: false,
        taproot_internal_key,
        range: (0, 999),
        // v0.47.3: genesis rescan (`0`) — the correct anchor for a recovery
        // workflow; matches export-wallet's default. restore has no --timestamp
        // flag. SPEC_timestamp_default_zero.
        timestamp: TimestampArg::Unix(0),
        bitcoin_core_version: 25,
        bsms_form: BsmsForm::default(),
    };

    // Shared 4-way dispatch (collect_missing-first → emit) via the canonical
    // `emit_payload` helper — byte-identical to the former inline copy,
    // INCLUDING the coldcard-multisig six-variant CliTemplate match.
    crate::cmd::export_wallet::emit_payload(&inputs, format)
}

fn template_label(t: CliTemplate) -> &'static str {
    match t {
        CliTemplate::Bip44 => "bip44 (legacy P2PKH)",
        CliTemplate::Bip49 => "bip49 (nested segwit P2SH-P2WPKH)",
        CliTemplate::Bip84 => "bip84 (native segwit P2WPKH)",
        CliTemplate::Bip86 => "bip86 (taproot P2TR)",
        // Multisig templates are rejected before any WalletRow is built.
        _ => "multisig",
    }
}

// ============================================================================
// Multisig-cosigner restore (v0.44.0; SPEC_restore_multisig_cosigner.md)
// ============================================================================

/// Build a `bitcoin::bip32::Xpub` from md-codec's 65-byte `[chain_code‖pubkey]`
/// form + the `--network`-authoritative `NetworkKind` (R0-r1 I2 — the md1 is
/// network-agnostic; md-codec's own reconstruction hardcodes `Main`). Depth-0.
fn xpub_from_65_bytes(bytes: &[u8; 65], network: CliNetwork) -> Result<Xpub, ToolkitError> {
    let chain_code = ChainCode::from(<[u8; 32]>::try_from(&bytes[0..32]).unwrap());
    let public_key = PublicKey::from_slice(&bytes[32..65])
        .map_err(|e| bad(format!("--md1 cosigner pubkey decode: {e}")))?;
    Ok(Xpub {
        network: network.network_kind(),
        depth: 0,
        parent_fingerprint: Fingerprint::default(),
        child_number: ChildNumber::Normal { index: 0 },
        public_key,
        chain_code,
    })
}

/// Convert md-codec's `OriginPath` to a `bitcoin` `DerivationPath` (inverse of
/// `synthesize::derivation_path_to_origin_path`). Reads the per-`@N` origin (do
/// NOT hardcode BIP-87 — sh(wsh) is `m/48'/coin'/account'/1'`).
fn origin_path_to_derivation_path(
    op: &md_codec::origin_path::OriginPath,
) -> Result<DerivationPath, ToolkitError> {
    let mut comps: Vec<ChildNumber> = Vec::with_capacity(op.components.len());
    for c in &op.components {
        let cn = if c.hardened {
            ChildNumber::from_hardened_idx(c.value)
        } else {
            ChildNumber::from_normal_idx(c.value)
        }
        .map_err(|_| {
            bad(format!(
                "--md1 origin component {} out of BIP-32 range",
                c.value
            ))
        })?;
        comps.push(cn);
    }
    Ok(comps.into())
}

/// Translator that applies the ONE caveat md-codec's
/// `to_miniscript_descriptor_multipath` leaves to the consumer: md-codec
/// hardcodes the `Main` network on its rendered xpubs. This corrects each key's
/// network kind to `--network` and passes the per-`@N` multipath GROUP through
/// unchanged (the builder already set it from each key's OWN
/// `ExpandedKey.use_site_path`, P2.2 — no baseline re-clobber). NUMS `Single`
/// internal keys pass through untouched (strict-NUMS refusal preserved).
struct ReconstructTranslator {
    network: CliNetwork,
}

/// The BIP-341 NUMS H-point as an `XOnlyPublicKey` (parsed from the shared
/// `cost::NUMS_XONLY_HEX` const; infallible on the known-good literal).
fn nums_xonly() -> bitcoin::secp256k1::XOnlyPublicKey {
    bitcoin::secp256k1::XOnlyPublicKey::from_str(crate::cost::NUMS_XONLY_HEX)
        .expect("the NUMS H-point hex literal is a valid x-only point")
}

impl miniscript::Translator<DescriptorPublicKey> for ReconstructTranslator {
    type TargetPk = DescriptorPublicKey;
    type Error = ToolkitError;

    fn pk(&mut self, pk: &DescriptorPublicKey) -> Result<DescriptorPublicKey, ToolkitError> {
        use miniscript::descriptor::SinglePubKey;
        // A `Single` key appears in exactly one card rendering: the BIP-341
        // NUMS H-point internal key of a `tr(NUMS,…)` policy (md-codec
        // `build_nums_internal_key` is the only `Single` producer; every
        // policy key is an `XPub`/`MultiXPub`). Pass it through UNCHANGED iff
        // it IS the H-point — x-only equality, never string matching — and
        // never promote it to multipath/network. Any other `Single` cannot
        // come from a toolkit wallet-policy card → refuse (strict-NUMS, v0.55.1).
        if let DescriptorPublicKey::Single(s) = pk {
            if matches!(&s.key, SinglePubKey::XOnly(x) if *x == nums_xonly()) {
                return Ok(pk.clone());
            }
            return Err(bad(
                "--md1 reconstruction: unexpected non-NUMS single key in wallet policy",
            ));
        }
        // `to_miniscript_descriptor_multipath` (P2.2) already assembled each
        // key with its OWN per-`@N` multipath group from
        // `ExpandedKey.use_site_path` — so the ONLY remaining caveat is the
        // hardcoded `Main` network. Correct the network kind in place and pass
        // the group (and wildcard/origin) through UNCHANGED — NO baseline
        // re-clobber. Be total (R0-r1 M6) — never panic: a wallet-policy key is
        // always `MultiXPub` (with a group) or `XPub` (a `None`-multipath
        // override / wildcard-only key); anything else → refuse.
        match pk {
            DescriptorPublicKey::MultiXPub(x) => {
                let mut x = x.clone();
                x.xkey.network = self.network.network_kind();
                Ok(DescriptorPublicKey::MultiXPub(x))
            }
            DescriptorPublicKey::XPub(x) => {
                let mut x = x.clone();
                x.xkey.network = self.network.network_kind();
                Ok(DescriptorPublicKey::XPub(x))
            }
            _ => Err(bad(
                "--md1 reconstruction: unexpected non-XPub key in wallet policy",
            )),
        }
    }

    translate_hash_clone!(DescriptorPublicKey);
}

// The taproot use-site-override classification predicates (`taproot_override_card`
// / `restorable_taproot_override_card`) moved to the crate-root
// `taproot_override_classify` module so `unrestorable_advisory` can reach them
// under `cfg(fuzzing)` (where `cmd` is absent) — FOLLOWUP
// `fuzz-build-broken-unrestorable-advisory-references-bin-only-cmd`. Re-exported
// here (preserving the `cmd::restore::…` path + `pub(crate)` visibility) so every
// in-module caller, the classify-reroute, and the truth-table tests below
// (`use super::*`) resolve the bare names unchanged. Logic is byte-identical —
// a pure relocation, no behavior change.
pub(crate) use crate::taproot_override_classify::{
    restorable_taproot_override_card, taproot_override_card,
};

/// Reconstruct the faithful concrete watch-only descriptor STRING from a general
/// (non-plain-template) wallet-policy md1, PRESERVING the full policy tree
/// (timelocks/hashlocks/andor/decay/…). This is the C1 fix: md-codec's
/// `to_miniscript_descriptor` already renders the faithful descriptor — keep it
/// (with the network/multipath `translate_pk` pass) instead of discarding it into
/// a plain-multi template. Errors (the `pk(@N)`/`pkh(@N)` double-Check shape,
/// PART 2) surface a CLEAR refusal naming the md-codec follow-up — never silent.
fn faithful_multisig_descriptor(
    d: &md_codec::Descriptor,
    network: CliNetwork,
) -> Result<String, ToolkitError> {
    // P2.2/C2: the multipath builder assembles each `@N`'s OWN per-key group
    // (from `ExpandedKey.use_site_path`, where `@N` == Vec position) — so a
    // divergent `@1/<2;3>/*` reconstructs to ITS group, not the baseline
    // `<0;1>` collapse the old single-path `to_miniscript_descriptor(d, 0)`
    // produced. The `ReconstructTranslator` below is now network-correction ONLY.
    let ms0 = md_codec::to_miniscript::to_miniscript_descriptor_multipath(d).map_err(|e| {
        // A `cannot wrap a fragment of type B` error is the known `pk(@N)`/
        // `pkh(@N)` double-Check shape (PART 2); other errors are unrelated, so
        // attribute the slug conditionally rather than blaming it for everything.
        let hint = if e.to_string().contains("cannot wrap") {
            " — this md1 encodes a key-check fragment the current md-codec cannot yet render \
             back (tracked as `to-miniscript-check-pkh-double-wrap`)"
        } else {
            ""
        };
        bad(format!(
            "--md1 → descriptor: {e}{hint}. The engraved card remains a faithful backup."
        ))
    })?;
    let mut t = ReconstructTranslator { network };
    let translated = ms0.translate_pk(&mut t).map_err(|e| match e {
        miniscript::TranslateErr::TranslatorErr(te) => te,
        miniscript::TranslateErr::OuterError(oe) => bad(format!("--md1 reconstruction: {oe}")),
    })?;
    Ok(translated.to_string())
}

/// Return `Some(template)` ONLY for a strictly-plain `wsh/sh-wsh(multi|sortedmulti)`
/// md1 with IDENTITY key indices and the standard `<0;1>` use-site — the shape the
/// existing `build_descriptor_string` path reconstructs byte-for-byte. Everything
/// else (general policy, duplicate/non-identity indices, non-standard/`None`
/// use-site) returns `None` → the faithful arm. Deliberately does NOT use
/// `template_from_descriptor` (its `Wsh(_) => WshMulti` collapse IS the C1 bug).
fn plain_template_from_tree(
    node: &md_codec::tree::Node,
    use_site: &md_codec::use_site_path::UseSitePath,
) -> Option<CliTemplate> {
    use md_codec::tree::Body;
    use md_codec::Tag;

    // Standard `<0;1>/*` use-site only; anything else (incl. `None`) → faithful.
    if *use_site != md_codec::use_site_path::UseSitePath::standard_multipath() {
        return None;
    }
    // A plain multi/sortedmulti leaf with identity indices. `Some(true)` =
    // sortedmulti, `Some(false)` = multi, `None` = not-plain (→ faithful arm,
    // incl. duplicate/non-identity indices `build_descriptor_string` would drop).
    fn plain_leaf(n: &md_codec::tree::Node) -> Option<bool> {
        match (&n.tag, &n.body) {
            (Tag::Multi | Tag::SortedMulti, Body::MultiKeys { indices, .. }) => {
                let identity = indices.iter().enumerate().all(|(i, &ix)| ix as usize == i);
                identity.then_some(matches!(n.tag, Tag::SortedMulti))
            }
            _ => None,
        }
    }
    match (&node.tag, &node.body) {
        (Tag::Wsh, Body::Children(c)) if c.len() == 1 => plain_leaf(&c[0]).map(|sorted| {
            if sorted {
                CliTemplate::WshSortedMulti
            } else {
                CliTemplate::WshMulti
            }
        }),
        (Tag::Sh, Body::Children(c)) if c.len() == 1 => match (&c[0].tag, &c[0].body) {
            (Tag::Wsh, Body::Children(gc)) if gc.len() == 1 => plain_leaf(&gc[0]).map(|sorted| {
                if sorted {
                    CliTemplate::ShWshSortedMulti
                } else {
                    CliTemplate::ShWshMulti
                }
            }),
            _ => None,
        },
        _ => None,
    }
}

/// One reconstructed cosigner position for the restore document.
struct CosignerInfo {
    idx: u8,
    fingerprint: Fingerprint,
    origin: DerivationPath,
    /// 65-byte canonical key form, for cross-check comparison.
    key65: [u8; 65],
    /// Cross-check verdict label (set during the cross-check pass).
    note: &'static str,
}

/// `mnemonic restore --md1 …` — reconstruct the concrete watch-only multisig
/// descriptor from a wallet-policy md1; cross-check `--from`/`--cosigner`.
fn run_multisig<R: Read, W: Write, E: Write>(
    args: &RestoreArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let network = args.network.unwrap_or(CliNetwork::Mainnet);

    // `--expect-xpub`/`--template` are single-sig-only here. `--format` IS
    // supported in multisig mode (v0.45.0) — emitted below via
    // `build_multisig_import_payload`.
    if args.expect_xpub.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "restore",
            flag: "--expect-xpub",
            message: "--expect-xpub is single-sig only; multisig cross-check uses --from / --cosigner @N=",
        });
    }
    if let Some(t) = args.template {
        if !t.is_multisig() {
            return Err(ToolkitError::ModeViolation {
                mode: "restore",
                flag: "--template",
                message: "--template (single-sig) does not apply in multisig --md1 mode; remove it",
            });
        }
    }

    // --- 1. Reassemble the md1 card(s) ---
    let md1_refs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
    let d =
        md_codec::chunk::reassemble(&md1_refs).map_err(|e| bad(format!("--md1 decode: {e}")))?;

    // --- 2. Gate: wallet-policy requirement (taproot multisig handled in §3) ---
    if !d.is_wallet_policy() {
        return Err(ToolkitError::ModeViolation {
            mode: "restore",
            flag: "--md1",
            message: "--md1 is template-only (no concrete cosigner keys); multisig restore needs a wallet-policy md1 (the toolkit emits these for every cosigner set)",
        });
    }

    // Use-site fidelity guard (P2.3, narrowed): faithful per-`@N` reconstruction
    // (C1 routing + C2 multipath builder) now restores non-taproot, non-hardened
    // override cards correctly — so the blanket override refusal is gone. TWO
    // residual shapes still RECONSTRUCT WRONG/UNDERIVABLE and must refuse loudly
    // (the funds-safety class this fix closes); both predicates are SHARED with
    // the engrave-surface advisory (`unrestorable_advisory.rs`) so the advisory
    // fires IFF restore refuses (exact parity).
    //
    //  (1) ANY hardened use-site (baseline OR override; `/*h` wildcard OR a
    //      hardened multipath alt) — watch-only cannot derive hardened, and a
    //      reconstructed descriptor would silently render an unhardened `/*`.
    //  (2) A TAPROOT override card OUTSIDE the restorable subset (#26): a
    //      `sortedmulti_a` leaf (md-codec render gap), a non-NUMS real trunk key
    //      (D7 out of scope), or a hardened use-site — these still route around
    //      the faithful per-`@N` arm and would mis-render, so they refuse. The
    //      RESTORABLE subset — non-hardened `tr(NUMS, multi_a)` — is ADMITTED
    //      (`restorable_taproot_override_card`) and reaches the P2.2 reroute.
    if md_codec::to_miniscript::has_hardened_use_site(&d) {
        return Err(ToolkitError::ModeViolation {
            mode: "restore",
            flag: "--md1",
            message: "this md1 uses a hardened use-site path (`/*h` wildcard or a hardened multipath alternative, baseline or per-cosigner) — watch-only addresses cannot be derived from it, and a reconstructed descriptor would silently render an unhardened path. Faithful reconstruction is not supported. The engraved card remains a faithful backup. Tracked: restore-md1-per-key-use-site-and-hardened-wildcard",
        });
    }
    if taproot_override_card(&d) && !restorable_taproot_override_card(&d) {
        return Err(ToolkitError::ModeViolation {
            mode: "restore",
            flag: "--md1",
            message: "this taproot md1 carries per-cosigner use-site path overrides in a shape the toolkit cannot yet reconstruct faithfully (a sortedmulti_a tap leaf, or a non-NUMS internal/trunk key). Non-hardened tr(NUMS, multi_a(...)) override cards ARE restorable; other taproot override shapes route around the per-key reconstruction path and emitting a single shared suffix would misrepresent the wallet. The engraved card remains a faithful backup. Tracked: restore-md1-taproot-use-site-override-arm",
        });
    }

    // --- 3. Classify: template + (taproot) NUMS internal key. ---
    // Taproot md1 (`Tag::Tr`): `classify_taproot_restore` 3-ways the tree —
    // single-leaf `multi_a`/`sortedmulti_a` → the byte-identical Template path
    // (routing AROUND md-codec's `to_miniscript`, which errors on a root
    // `SortedMultiA`; the toolkit's own miniscript rev 95fdd1c HAS
    // `Terminal::SortedMultiA`); general single-leaf / depth-1 two-leaf
    // `tr(NUMS,…)` → GeneralFaithful (`template_opt = None`, falls through the
    // SAME general-policy machinery as wsh below, v0.55.1; non-NUMS real-trunk
    // reconstructs since v0.55.3); depth ≥2 / `sortedmulti_a`-under-TapTree /
    // `@-in-both` (trunk key also a leaf key) → loud structural refusals.
    // wsh/sh-wsh keep `to_miniscript_descriptor`. `template_opt = Some(_)`
    // ONLY for a strictly-plain `wsh/sh-wsh(multi|sortedmulti)` (or
    // single-leaf taproot multi_a/sortedmulti_a) md1 → the existing
    // byte-for-byte `build_descriptor_string` path. `None` = a GENERAL policy
    // (timelocks/hashlocks/andor/decay/…) → `faithful_multisig_descriptor`,
    // which keeps the full tree instead of silently collapsing it to plain
    // multisig (the C1 funds-safety fix). Discrimination is STRUCTURAL on the
    // md1 tree, NOT `template_from_descriptor` (its `Wsh(_) => WshMulti` arm IS
    // the collapse bug).
    let is_taproot = d.tree.tag == md_codec::Tag::Tr;
    let (template_opt, tap_internal_key): (Option<CliTemplate>, Option<TaprootInternalKey>) =
        if is_taproot && restorable_taproot_override_card(&d) {
            // P2.2 (#26): a RESTORABLE taproot override card — `tr(NUMS, multi_a)`
            // with per-`@N` divergent suffixes, non-hardened. The single-leaf
            // `multi_a` Template path hardcodes one shared `<0;1>` suffix per key
            // and would silently collapse `@1`'s divergent alt — so FORCE the
            // faithful arm (`template_opt = None`), exactly as the non-taproot
            // override path below does. The internal key is NUMS by the
            // predicate's `is_nums:true` conjunct. The faithful arm routes through
            // `faithful_multisig_descriptor` → md-codec's multipath builder, which
            // reconstructs each `@N`'s OWN group. (`classify_taproot_restore` is
            // tree-only and cannot see overrides, so the verdict is computed here
            // at the call site, mirroring the non-taproot override reroute.)
            (None, Some(TaprootInternalKey::Nums))
        } else if is_taproot {
            match classify_taproot_restore(&d.tree)? {
                TaprootRestore::Template(t, ik) => (Some(t), Some(ik)),
                TaprootRestore::GeneralFaithful(ik) => (None, Some(ik)),
            }
        } else if d.tlv.use_site_path_overrides.is_some() {
            // C1 (P2.1): an override card carries per-`@N` divergent suffixes the
            // plain-template renderer cannot express (it hardcodes `<0;1>` per
            // key). Force the faithful arm (`template_opt = None`), which
            // reconstructs each `@N`'s OWN group via the md-codec multipath
            // builder. (Taproot override cards never reach here — pre-refused by
            // the use-site guard above.)
            (None, None)
        } else {
            (plain_template_from_tree(&d.tree, &d.use_site_path), None)
        };
    // The "is multisig" hard-gate applies ONLY to the plain arm (a plain
    // multi/sortedmulti tree always carries a threshold). The general arm does
    // NOT require `k` — it routes to `faithful_multisig_descriptor` regardless
    // (R0-r1 I1: the cryptic k-gate must not pre-empt the clear general refusal).
    let k_opt: Option<u8> = crate::cmd::bundle::extract_multisig_threshold(&d.tree);

    // --- 4. Build cosigner slots from the wallet-policy keys ---
    let expanded = md_codec::canonicalize::expand_per_at_n(&d)
        .map_err(|e| bad(format!("--md1 expand: {e}")))?;
    let mut slots: Vec<ResolvedSlot> = Vec::with_capacity(expanded.len());
    let mut cosigners: Vec<CosignerInfo> = Vec::with_capacity(expanded.len());
    for e in &expanded {
        // The `is_wallet_policy()` gate guarantees `Some`; handle `None`
        // defensively rather than `unwrap` (R0-r2).
        let key65 = e
            .xpub
            .ok_or_else(|| bad(format!("--md1 cosigner @{} has no concrete pubkey", e.idx)))?;
        let fp_bytes = e
            .fingerprint
            .ok_or_else(|| bad(format!("--md1 cosigner @{} has no fingerprint", e.idx)))?;
        let xpub = xpub_from_65_bytes(&key65, network)?;
        let fingerprint = Fingerprint::from(fp_bytes);
        let origin = origin_path_to_derivation_path(&e.origin_path)?;
        slots.push(ResolvedSlot {
            xpub,
            fingerprint,
            path: origin.clone(),
            entropy: None,
            master_xpub: None,
            language: None,
            _entropy_pin: None,
        });
        cosigners.push(CosignerInfo {
            idx: e.idx,
            fingerprint,
            origin,
            key65,
            note: "unverified",
        });
    }

    // Plain arm: existing `build_descriptor_string` (byte-for-byte unchanged —
    // `tap_internal_key` is `Some(ik)` for taproot, `None` for non-taproot,
    // exactly as before). General arm: the faithful reconstruction.
    let descriptor = match template_opt {
        Some(template) => build_descriptor_string(
            template,
            &slots,
            k_opt.expect("plain/taproot template arm always carries a threshold"),
            network,
            args.account_primary(),
            tap_internal_key,
        )?,
        None => faithful_multisig_descriptor(&d, network)?,
    };

    // --- 5. First receive address(es), chain 0. ---
    // Taproot AND the general arm derive from the reconstructed descriptor STRING
    // via the toolkit's miniscript (self-consistency: print and address agree).
    // The plain wsh/sh-wsh arm keeps the md-codec tree path. `d.derive_address`
    // re-enters md-codec's `to_miniscript` which errors on `SortedMultiA`, so the
    // string path is mandatory for taproot; for the general arm it guarantees the
    // address matches the FAITHFUL descriptor we print (R0 v2 C1 / crux 4).
    let first_recv: Vec<String> = if is_taproot || template_opt.is_none() {
        let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(&descriptor)
            .map_err(|e| bad(format!("--md1 descriptor parse: {e}")))?;
        // Display-fidelity guard (v0.55.1, R0 Q4): the reconstructed
        // descriptor must survive its own parse→print round-trip — the only
        // guard against a PARSEABLE-but-wrong Display infidelity in the
        // pinned miniscript (the known depth-2 taptree bug is structurally
        // pre-gated in §3; this catches any future parseable variant). The
        // template-tr arm cannot false-refuse here: `build_descriptor_string`
        // output is already `to_string()` of a parsed descriptor
        // (Display-stable by construction), as is the faithful arm's.
        if parsed.to_string() != descriptor {
            return Err(bad(
                "--md1 internal error: the reconstructed descriptor does not survive a parse→print round-trip (miniscript Display infidelity); refusing rather than print a possibly-unfaithful descriptor. The engraved card remains a faithful backup.",
            ));
        }
        // Consensus-masked older() advisory (Adapter B, fail-closed): a bit-31
        // or zero-16-bit card would have errored at `from_str` above before
        // reaching here, so only the `Masked` consequence can fire. Non-blocking.
        let adv = crate::timelock_advisory::older_advisories_descriptor(&parsed);
        crate::timelock_advisory::emit_advisories(&adv, stderr);
        crate::derive_address::derive_receive_addresses(
            &parsed,
            args.count,
            network.to_bitcoin_network(),
        )?
    } else {
        let mut v = Vec::with_capacity(args.count as usize);
        for i in 0..args.count {
            let addr = d
                .derive_address(0, i, network.to_bitcoin_network())
                .map_err(|e| bad(format!("first receive address @{i}: {e}")))?;
            v.push(addr.assume_checked().to_string());
        }
        v
    };

    // --- 6. Cross-check (own seed via --from; cosigners via --cosigner @N=) ---
    let mut mismatch: Option<(&'static str, String, String, Option<u8>)> = None;
    let has_reference = args.from.is_some() || !args.cosigner.is_empty();
    // Positions whose key was INDEPENDENTLY validated (own seed + each passing
    // `--cosigner @N`). C1: ONLY these may be labeled verified — never blanket-
    // label the positions that were not actually cross-checked.
    let mut verified_positions: std::collections::BTreeSet<u8> = std::collections::BTreeSet::new();

    // 6a. own seed (--from) → infer position by 65-byte match.
    let mut own_pos: Option<u8> = None;
    if let Some(from_raw) = args.from.as_deref() {
        let from = parse_from_input(from_raw).map_err(bad)?;
        let from_uses_stdin = from.value == "-";
        if !matches!(
            from.node,
            NodeType::Ms1 | NodeType::Phrase | NodeType::Entropy | NodeType::Seedqr
        ) {
            return Err(bad(format!(
                "--from {} is not a seed source for restore (use ms1/phrase/entropy/seedqr)",
                from.node.as_str()
            )));
        }
        if args.passphrase_stdin && from_uses_stdin {
            return Err(bad(
                "--passphrase-stdin cannot coexist with --from <node>=- (a single stdin cannot serve both)",
            ));
        }
        if !from_uses_stdin && !from.value.starts_with("@env:") {
            let node = from_raw.split('=').next().unwrap_or("");
            crate::secret_advisory::secret_in_argv_warning(
                stderr,
                &format!("--from {node}="),
                &format!("--from {node}=-"),
            );
        }
        if let Some(pp) = args.passphrase.as_deref() {
            if !pp.starts_with("@env:") {
                crate::secret_advisory::secret_in_argv_warning(
                    stderr,
                    "--passphrase",
                    "--passphrase-stdin",
                );
            }
        }
        // cycle-14 (L22): wrap in Zeroizing (handler-scope scrub).
        let passphrase: zeroize::Zeroizing<String> =
            zeroize::Zeroizing::new(if args.passphrase_stdin {
                read_stdin_passphrase(stdin)?
            } else {
                match args.passphrase.as_deref() {
                    Some(p) => crate::env_sentinel::resolve_env_var_sentinel(p, "--passphrase")?,
                    None => String::new(),
                }
            });
        let from_value: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(if from_uses_stdin {
            read_stdin_to_string(stdin)?
        } else {
            crate::env_sentinel::resolve_env_var_sentinel(&from.value, "--from")?
        });
        let (entropy, derive_language) =
            resolve_seed_entropy(&from.node, &from_value, args.language)?;
        let _pin = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);
        // M1: pin the passphrase too (parity with the single-sig `run` path).
        let _pin_pp = (!passphrase.is_empty())
            .then(|| mnemonic_toolkit::mlock::pin_pages_for(passphrase.as_bytes()));

        // Derive the own key at each cosigner's origin; the 65-byte match is the
        // own position (stronger than a master-fp match, R0-r1 M3).
        for c in &cosigners {
            let acct = crate::derive_slot::derive_bip32_from_entropy_at_path(
                &entropy,
                &passphrase,
                derive_language,
                network,
                &c.origin,
            )?;
            if crate::synthesize::xpub_to_65(&acct.account_xpub) == c.key65 {
                own_pos = Some(c.idx);
                verified_positions.insert(c.idx);
                break;
            }
        }
        if own_pos.is_none() {
            // The supplied seed is not a cosigner of this wallet.
            let derived_fp = {
                // Recompute master fp once for the message (path-independent).
                let acct = crate::derive_slot::derive_bip32_from_entropy_at_path(
                    &entropy,
                    &passphrase,
                    derive_language,
                    network,
                    &cosigners[0].origin,
                )?;
                acct.master_fingerprint.to_string().to_lowercase()
            };
            mismatch = Some((
                "cosigner-seed",
                format!("seed master fp {derived_fp}"),
                "a cosigner of this md1 wallet".to_string(),
                None,
            ));
        }
    }

    // 6b. explicit cosigner assertions (--cosigner @N=mk1|xpub).
    if mismatch.is_none() && !args.cosigner.is_empty() {
        // Group values by position N.
        let mut by_pos: std::collections::BTreeMap<u8, Vec<String>> =
            std::collections::BTreeMap::new();
        for spec in &args.cosigner {
            let (lhs, rhs) = spec
                .split_once('=')
                .ok_or_else(|| bad(format!("--cosigner expects @N=<mk1|xpub>, got `{spec}`")))?;
            let n: u8 = lhs
                .trim_start_matches('@')
                .parse()
                .map_err(|_| bad(format!("--cosigner position `{lhs}` is not `@N`")))?;
            by_pos.entry(n).or_default().push(rhs.to_string());
        }
        for (n, values) in &by_pos {
            let c = cosigners.iter().find(|c| c.idx == *n).ok_or_else(|| {
                bad(format!(
                    "--cosigner @{n}: position out of range (wallet has {} cosigners)",
                    cosigners.len()
                ))
            })?;
            // mk1 (multi-chunk) vs a single raw xpub. Case-insensitive PROBE
            // (v0.53.3 audit M11); originals pass to mk-codec, the case
            // authority (it lowercase-normalizes; rejects mixed).
            let supplied65: [u8; 65] = if values.iter().all(|v| v.to_lowercase().starts_with("mk1"))
            {
                let refs: Vec<&str> = values.iter().map(|v| v.as_str()).collect();
                let kc = mk_codec::decode(&refs)
                    .map_err(|e| bad(format!("--cosigner @{n} mk1 decode: {e}")))?;
                crate::synthesize::xpub_to_65(&kc.xpub)
            } else if values.len() == 1 {
                let xpub = Xpub::from_str(&values[0])
                    .map_err(|e| bad(format!("--cosigner @{n} xpub parse: {e}")))?;
                crate::synthesize::xpub_to_65(&xpub)
            } else {
                return Err(bad(format!(
                    "--cosigner @{n}: multiple values must all be mk1 chunks, or a single xpub"
                )));
            };
            if supplied65 != c.key65 {
                mismatch = Some((
                    "cosigner-key",
                    format!("supplied key for @{n}"),
                    format!(
                        "md1 cosigner @{n} ({})",
                        c.fingerprint.to_string().to_lowercase()
                    ),
                    Some(*n),
                ));
                break;
            }
            verified_positions.insert(*n);
        }
    }

    // --- 7. Mismatch hard-gate (exit 4) unless --allow-mismatch ---
    if let Some((reference, derived, expected, slot)) = &mismatch {
        if !args.allow_mismatch {
            writeln!(stderr, "✗ MISMATCH").map_err(ToolkitError::Io)?;
            return Err(ToolkitError::RestoreMismatch {
                reference,
                derived: derived.clone(),
                expected: expected.clone(),
                slot: *slot,
            });
        }
    }

    // Annotate per-cosigner notes — C1: ONLY positions in `verified_positions`
    // (own seed + each passing `--cosigner @N`) are labeled verified; every other
    // position is "from md1 (not independently verified)" even when SOME other
    // position WAS cross-checked. Never present an unchecked key as verified.
    for c in cosigners.iter_mut() {
        c.note = if Some(c.idx) == own_pos {
            "← your seed (verified)"
        } else if verified_positions.contains(&c.idx) {
            "cross-checked"
        } else {
            "from md1 (not independently verified)"
        };
    }

    // Overall status: "verified" ONLY when EVERY cosigner position was validated;
    // "partial" when some (but not all) were; else "unverified" / "overridden".
    let all_verified = cosigners
        .iter()
        .all(|c| verified_positions.contains(&c.idx));
    let verification_status = if mismatch.is_some() {
        "overridden"
    } else if !has_reference {
        "unverified"
    } else if all_verified {
        "verified"
    } else {
        "partial"
    };

    // Build the importable payload when `--format` is set (v0.45.0). Computed
    // AFTER the step-7 mismatch hard-gate, so a non-overridden MISMATCH exits 4
    // before any payload is emitted (with `--allow-mismatch` the payload is the
    // md1's authoritative wallet + the overridden banner, mirroring single-sig).
    let import_payload: Option<String> = match args.format {
        Some(f) => Some(build_multisig_import_payload(
            f,
            template_opt,
            &slots,
            k_opt,
            &descriptor,
            network,
            args.account_primary(),
            tap_internal_key,
        )?),
        None => None,
    };

    // Labels (R0-r1 I4): a general policy is NOT "k-of-n multisig" (and for a
    // decay vault `extract_multisig_threshold` returns only the FIRST k, so the
    // top-level threshold is misleading). All four label sites switch on the arm.
    let n_cosigners = cosigners.len();
    // Top-level `threshold` is the WALLET's k-of-n threshold — meaningful only
    // for a plain multisig. A general policy has no single threshold (a decay
    // vault has several; `k_opt` would report only the first), so it is null.
    let threshold_field: Option<u8> = if template_opt.is_some() { k_opt } else { None };
    let (header_label, wallet_type_label): (String, String) = match (template_opt, k_opt) {
        (Some(_), Some(k)) => (
            format!("{k}-of-{n_cosigners} multisig restore"),
            format!("{k}-of-{n_cosigners} multisig"),
        ),
        _ => {
            let noun = if n_cosigners == 1 {
                "cosigner"
            } else {
                "cosigners"
            };
            (
                format!("miniscript policy restore ({n_cosigners} {noun})"),
                "miniscript-policy".to_string(),
            )
        }
    };

    // --- 8. Compose stdout content (payload | json | text) + route to --output ---
    let stdout_content: String = if args.json {
        let cos: Vec<_> = cosigners
            .iter()
            .map(|c| {
                json!({
                    "position": c.idx,
                    "fingerprint": c.fingerprint.to_string().to_lowercase(),
                    "origin": c.origin.to_string(),
                    "note": c.note,
                })
            })
            .collect();
        let mut verification = json!({ "status": verification_status });
        if let Some((reference, derived, expected, slot)) = &mismatch {
            verification["reference"] = json!(reference);
            verification["derived"] = json!(derived);
            verification["expected"] = json!(expected);
            verification["slot"] = json!(slot);
        }
        let mut envelope = json!({
            "mode": "multisig",
            "network": network.human_name(),
            "threshold": threshold_field,
            "cosigners": cosigners.len(),
            "verification": verification,
            "wallets": [json!({
                "wallet_type": wallet_type_label,
                "descriptor": descriptor,
                "first_addresses": first_recv,
                "cosigner_keys": cos,
            })],
        });
        if let Some(payload) = &import_payload {
            envelope["import_payload"] = json!(payload);
        }
        format!(
            "{}\n",
            serde_json::to_string(&envelope)
                .map_err(|e| bad(format!("json serialization: {e}")))?
        )
    } else if let Some(payload) = &import_payload {
        // `--format` without `--json`: the payload alone is stdout so it pipes
        // cleanly into wallet software; the verification doc goes to stderr below.
        format!("{payload}\n")
    } else {
        let mut s = String::new();
        s.push_str(&format!("{header_label}\n"));
        s.push_str(
            "CONFIRM: verify each cosigner fingerprint against your records before importing.\n",
        );
        s.push_str(&format!("  descriptor: {descriptor}\n"));
        for addr in &first_recv {
            s.push_str(&format!("  first recv: {addr}\n"));
        }
        for c in &cosigners {
            s.push_str(&format!(
                "  cosigner @{}: {} [{}]  {}\n",
                c.idx,
                c.fingerprint.to_string().to_lowercase(),
                c.origin,
                c.note
            ));
        }
        s
    };

    if args.output == "-" {
        write!(stdout, "{stdout_content}").map_err(ToolkitError::Io)?;
    } else {
        std::fs::write(&args.output, &stdout_content)
            .map_err(|e| bad(format!("--output {}: {e}", args.output)))?;
    }

    // When `--format` is set (and not `--json`), the human verification doc is
    // NOT the stdout content — surface it on stderr so the operator can confirm
    // each cosigner fingerprint while the payload pipes onward (mirror single-sig).
    if import_payload.is_some() && !args.json {
        writeln!(stderr, "{header_label}").map_err(ToolkitError::Io)?;
        writeln!(
            stderr,
            "CONFIRM: verify each cosigner fingerprint against your records before importing the payload above."
        )
        .map_err(ToolkitError::Io)?;
        writeln!(stderr, "  descriptor: {descriptor}").map_err(ToolkitError::Io)?;
        for addr in &first_recv {
            writeln!(stderr, "  first recv: {addr}").map_err(ToolkitError::Io)?;
        }
        for c in &cosigners {
            writeln!(
                stderr,
                "  cosigner @{}: {} [{}]  {}",
                c.idx,
                c.fingerprint.to_string().to_lowercase(),
                c.origin,
                c.note
            )
            .map_err(ToolkitError::Io)?;
        }
    }

    // --- 9. Verification banners (stderr) ---
    if mismatch.is_some() {
        writeln!(
            stderr,
            "✗ MISMATCH (overridden): a supplied cross-check key does NOT match the md1 wallet; \
             the descriptor above is the md1's wallet, NOT what your --from/--cosigner asserted"
        )
        .map_err(ToolkitError::Io)?;
    } else if !has_reference {
        writeln!(
            stderr,
            "UNVERIFIED: no --from/--cosigner cross-check supplied; verify each cosigner \
             fingerprint above against your records before importing"
        )
        .map_err(ToolkitError::Io)?;
    } else if !all_verified {
        // C1: some cosigners were cross-checked, others were not. Name the
        // unverified positions so the user does not over-trust the document.
        let unverified: Vec<String> = cosigners
            .iter()
            .filter(|c| !verified_positions.contains(&c.idx))
            .map(|c| format!("@{}", c.idx))
            .collect();
        writeln!(
            stderr,
            "PARTIAL: cross-checked {}/{} cosigners; positions {} were NOT independently \
             verified — confirm their fingerprints against your records before importing",
            verified_positions.len(),
            cosigners.len(),
            unverified.join(", ")
        )
        .map_err(ToolkitError::Io)?;
    }

    // Cycle Y — LOUD funds-safety warning for a CUSTOM use-site on a
    // tr(NUMS,multi_a) card. This shape RESTORES FAITHFULLY (#26, admitted at the
    // restorable-taproot-override arm above) but has no known wallet precedent, so
    // a misconfigured user risks PERMANENT LOSS OF FUNDS. Proceed-and-warn (NOT
    // refuse): the reconstruction above is unchanged; `Ok(0)` below. Single-sourced
    // via `custom_use_site_nums_taproot_card`, so engrave and restore cannot drift.
    let fs = crate::unrestorable_advisory::funds_safety_advisories(&d);
    crate::unrestorable_advisory::emit_funds_safety_advisories(&fs, stderr);

    crate::secret_advisory::emit_output_class_advisory(
        crate::secret_advisory::OutputClass::WatchOnly,
        stderr,
    );

    Ok(0)
}

/// Resolve a seed `--from` node + value to (entropy, derive-language), mirroring
/// the single-sig `run` block (ms1 wire-language wins; entropy/seedqr/phrase).
fn resolve_seed_entropy(
    node: &NodeType,
    from_value: &str,
    language: Option<CliLanguage>,
) -> Result<(zeroize::Zeroizing<Vec<u8>>, bip39::Language), ToolkitError> {
    Ok(match node {
        NodeType::Ms1 => {
            let res = crate::slot_ms1::resolve_ms1_slot(from_value, language, 0)?;
            (res.entropy, res.derive_language)
        }
        NodeType::Phrase => {
            let lang = language.unwrap_or_default();
            let entropy = zeroize::Zeroizing::new(
                Mnemonic::parse_in(lang.into(), from_value)
                    .map_err(ToolkitError::Bip39)?
                    .to_entropy(),
            );
            (entropy, lang.into())
        }
        NodeType::Seedqr => {
            let lang = language.unwrap_or_default();
            let phrase = mnemonic_toolkit::seedqr::decode(from_value)
                .map_err(|e| crate::cmd::seedqr::map_seedqr_error(e, "restore"))?;
            let entropy = zeroize::Zeroizing::new(
                Mnemonic::parse_in(lang.into(), &phrase)
                    .map_err(ToolkitError::Bip39)?
                    .to_entropy(),
            );
            (entropy, lang.into())
        }
        NodeType::Entropy => {
            let entropy = zeroize::Zeroizing::new(
                hex::decode(from_value.trim())
                    .map_err(|e| bad(format!("--from entropy= hex-decode: {e}")))?,
            );
            (entropy, bip39::Language::English)
        }
        _ => unreachable!("seed-node guard restricts to ms1/phrase/seedqr/entropy"),
    })
}

#[cfg(test)]
mod taproot_override_predicate_tests {
    //! P2.1 truth table for `restorable_taproot_override_card` — the SINGLE
    //! shared predicate that partitions every `taproot_override_card(d)` into
    //! {reroute→faithful} vs {loud-refuse+advisory}. Each `Descriptor` is built
    //! from a REAL md1 card (generated offline via `mnemonic bundle` over the
    //! fixed C0/C1/C2 phrases) and reassembled through `md_codec::chunk` — the
    //! identical wire path `restore --md1` walks — so the predicate sees exactly
    //! the on-the-wire tree/TLV shape, not a hand-forged `Descriptor` literal.
    use super::*;

    fn desc(cards: &[&str]) -> md_codec::Descriptor {
        md_codec::chunk::reassemble(cards).expect("reassemble md1 cards")
    }

    // `tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<2;3>/*))` — divergent override, NUMS
    // internal, plain MultiA leaf, non-hardened → RESTORABLE (the unlock).
    const NUMS_MULTI_A_OVERRIDE: &[&str] = &[
        "md1frh62pspq2tvyyy4qqxquszzs95czskp0prnchdq4hp5gmug4cyja6p372zc9gwrh7h9q2hq95869eapttw6g",
        "md1frh62psvxs7s5yl6smtcxjz9m806prlsm794tkxqs6806lhaeh6reknylagmwyjycf8044xgymjflqyj4xuqg",
        "md1frh62psnt9flsdlkvt6f6cthyl98fejsahhtp2x7t365s9qhgfvt63yacv0jzrws489wwl2qk383gdnvgkmwn",
        "md1frh62psmcse69e0qvuhzq6k5jt8ymyydynzrv4kudj9m56mcqpxckrlzeq339uepg0p7q8x8rh5sd2v5hk",
    ];
    // Same shape but `sortedmulti_a` leaf — md-codec cannot render it back as a
    // non-root tap leaf (umbrella gap) → NOT restorable.
    const NUMS_SORTEDMULTI_A_OVERRIDE: &[&str] = &[
        "md1ftf38pspq2tvyyy4qqxqujzzs95czskp0prnchdq4hp5gmug4cyja6p372zc9gwrh7h9q2hqlafphjqhy6vu7",
        "md1ftf38psvxs7s5yl6smtcxjz9m806prlsm794tkxqs6806lhaeh6reknylagmwyjycf8044xg7stmtpsjl5fdj",
        "md1ftf38psnt9flsdlkvt6f6cthyl98fejsahhtp2x7t365s9qhgfvt63yacv0jzrws489wwl2qv67ruv8vzywrf",
        "md1ftf38psmcse69e0qvuhzq6k5jt8ymyydynzrv4kudj9m56mcqpxckrlzeq339uepg0p7q6gzlrcel29yrh",
    ];
    // `tr(@0,multi_a(2,@1/<0;1>/*,@2/<2;3>/*))` — real (non-NUMS) trunk key (D7
    // out of scope) → NOT restorable.
    const NON_NUMS_MULTI_A_OVERRIDE: &[&str] = &[
        "md1f3sl6zspqjtvyyy5qgjqgtqxnkqqdgzskp0npeutks2dcdzxlrzsezsqc27rchwsv0jskq40meejhx8ptl2",
        "md1f3sl6zsdgwrh7h9q2hyxs7s5yl6smtcxjz9m806prlsm794tkxqs6806lhaeh6reknylagqkr2s9n7c2vsc",
        "md1f3sl6zsndcjgnpya7k5edv487ph7e30f8tpwunu5knn9pm0wkz5duhr4fq2pwsjch4zfmsq6dryjtwrel8g",
        "md1f3sl6zsmrussm59fetnh6s7yxw3wtcr89csx44yjeexeprfycsm9dhrv3waxk7qqfk9slcwmfzgkfetgnvw",
        "md1f3sl6z3zeq339uepg0plpz2zll50ju3dcmghtxtfv0y025ltk2vc8a3ex8yqnc896wtrlv4g04rwua8nzh8",
        "md1f3sl6z3fhqdghjmksz3ry92d3gv4ejtmu9f0zxf3clxvtlnnv86xy4qee32ay5gp9lt69yuy5m4",
    ];
    // `tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<2;3>/*h))` — hardened alt in @1's
    // override; watch-only cannot derive it → NOT restorable.
    const NUMS_MULTI_A_HARDENED_OVERRIDE: &[&str] = &[
        "md1f36rfpspq2tvyyy4qqxquszzs95czshp0prnchdq4hp5gmug4cyja6p372zc9gwrh7h9q2hqrnqxdtcr2cxyl",
        "md1f36rfpsvxs7s5yl6smtcxjz9m806prlsm794tkxqs6806lhaeh6reknylagmwyjycf8044xgwcc03gsp4pm5n",
        "md1f36rfpsnt9flsdlkvt6f6cthyl98fejsahhtp2x7t365s9qhgfvt63yacv0jzrws489wwl2qujdhx98lg3u6g",
        "md1f36rfpsmcse69e0qvuhzq6k5jt8ymyydynzrv4kudj9m56mcqpxckrlzeq339uepg0p7qk49ams3vfwqr0",
    ];
    // `tr(NUMS,multi_a(2,@0,@1))` — NUMS plain MultiA but NO use-site override
    // → `taproot_override_card` false → predicate false (not the override leg).
    const NUMS_MULTI_A_NO_OVERRIDE: &[&str] = &[
        "md1fmvwjpspq2tvyyy5qwgppgtcgu79mg9dcdzxlz9wpyhwsv0jskp2rsal4egz4ep5859pq3wt7la86whlwl",
        "md1fmvwjpstl2rd0q6gghvalgy07r0ck4wcczrgalt7lhxlg0x6vnl4rdcjgnpya7k5edv4qwl2zn759k60rg",
        "md1fmvwjpsnlqmlvch5n4shwf72wnn9pm0wkz5duhr4fq2pwsjch4zfmsclyyxap2w2ua75qhvcxtekagrcz2",
        "md1fmvwjpsmcse69e0qvuhzq6k5jt8ymyydynzrv4kudj9m56mcqpxckrlzeq339uepg0p7qv90lpeqtn6vyf",
    ];

    #[test]
    fn restorable_nums_multi_a_override_is_true() {
        let d = desc(NUMS_MULTI_A_OVERRIDE);
        // Sanity: the shape IS a taproot override card (the blanket #25 set).
        assert!(taproot_override_card(&d), "must be a taproot override card");
        assert!(
            restorable_taproot_override_card(&d),
            "non-hardened tr(NUMS,multi_a) override is the restorable subset"
        );
    }

    #[test]
    fn sortedmulti_a_override_is_not_restorable() {
        let d = desc(NUMS_SORTEDMULTI_A_OVERRIDE);
        assert!(taproot_override_card(&d), "must be a taproot override card");
        assert!(
            !restorable_taproot_override_card(&d),
            "sortedmulti_a leaf has no md-codec renderer — must NOT be admitted"
        );
    }

    #[test]
    fn non_nums_trunk_override_is_not_restorable() {
        let d = desc(NON_NUMS_MULTI_A_OVERRIDE);
        assert!(taproot_override_card(&d), "must be a taproot override card");
        assert!(
            !restorable_taproot_override_card(&d),
            "non-NUMS real-trunk internal key is D7 out of scope — must NOT be admitted"
        );
    }

    #[test]
    fn hardened_override_is_not_restorable() {
        let d = desc(NUMS_MULTI_A_HARDENED_OVERRIDE);
        assert!(taproot_override_card(&d), "must be a taproot override card");
        assert!(
            md_codec::to_miniscript::has_hardened_use_site(&d),
            "fixture must actually carry a hardened use-site"
        );
        assert!(
            !restorable_taproot_override_card(&d),
            "a hardened use-site override is unrestorable for watch-only — must NOT be admitted"
        );
    }

    #[test]
    fn non_override_taproot_is_not_restorable() {
        let d = desc(NUMS_MULTI_A_NO_OVERRIDE);
        assert!(
            !taproot_override_card(&d),
            "no use-site overrides → not a taproot override card at all"
        );
        assert!(
            !restorable_taproot_override_card(&d),
            "the predicate is gated on taproot_override_card → false when there is no override"
        );
    }
}
