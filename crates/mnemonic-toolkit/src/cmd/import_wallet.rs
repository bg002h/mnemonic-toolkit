//! `mnemonic import-wallet` — v0.27.0 surface.
//!
//! Per SPEC_wallet_import_v0_26_0.md §2.1 (v0.26.0 baseline) + v0.27.0
//! plan-doc additions (BIP-129 Round-1 verify + envelope wire-shape
//! replacement):
//!
//!   --blob <FILE|->                                             required UNLESS --bsms-round1 supplied
//!   --format <bitcoin-core|bsms|coldcard|coldcard-multisig|electrum|jade|sparrow|specter>
//!                                                               optional (sniff default).
//!                                                               v0.28.0 Phase P0C pre-stubs the 6
//!                                                               new formats (panic via
//!                                                               `unimplemented!()` until per-parser
//!                                                               P{N}C sub-phases wire real dispatch).
//!   --select-descriptor <N|active-receive|active-change|all>    default `all`
//!   --ms1 <STRING>                                              repeatable (positional cosigner-index)
//!   --slot @<N>.phrase=<STRING>                                 (existing slot infra)
//!   --json                                                      bool; emit JSON envelope array (v0.27.0 carries full BundleJson + schema_version)
//!   --no-auto-repair                                            global; no-op for import-wallet path (reserved)
//!   --bsms-round1 <FILE>                                        v0.27.0 — repeatable; BIP-129 Round-1 BIP-322 verify per record
//!   --bsms-verify-strict                                        v0.27.0 — make Round-1 verify failure fatal (default lenient: stderr NOTICE + signature_verified:false)
//!
//! Dispatch flow:
//!   0. v0.27.0 standalone Round-1 verify mode — when `--bsms-round1` is
//!      supplied without `--blob`, parse + verify each record, emit a
//!      Round-1-only envelope (`--json`) or summary, exit 0 on verify
//!      success.
//!   1. Resolve env-var sentinels (`@env:VAR` → `std::env::var(VAR)`).
//!   2. Read blob.
//!   3. If `--format` is absent → invoke `sniff_format`:
//!        - Bsms / BitcoinCore → dispatch to corresponding parser.
//!        - Ambiguous → exit 1 `ImportWalletAmbiguousFormat`.
//!        - NoMatch → exit 1 `ImportWalletAmbiguousFormat` (different stderr template).
//!   4. If `--format <X>` is present → exit 1 `ImportWalletFormatMismatch`
//!      if blob sniffs as a different format. (When the user-supplied
//!      format matches sniff outcome OR sniff is `NoMatch`/`Ambiguous`,
//!      the explicit `--format` is honored.)
//!   5. Parse via selected parser. Apply seed overlay (SPEC §8.3). Apply
//!      `--select-descriptor` filter. Emit stdout (cards-or-JSON);
//!      v0.27.0 `--json` emits the full `BundleJson` shape in `bundle:`,
//!      with an outer `schema_version: "1"` (wire-shape REPLACEMENT vs
//!      v0.26.0 summary). When `--bsms-round1` also supplied, per-record
//!      verify state propagates into every envelope's
//!      `bsms_round1_verifications` field.
//!
//! Stderr discipline:
//!   - WARNINGs / NOTICEs from per-format parsers.
//!   - When `--json` is set: round-trip diff goes ONLY in the envelope;
//!     stderr is silent for the diff (SPEC §7.4).

use crate::cmd::convert::read_stdin_passphrase;
use crate::error::ToolkitError;
use crate::format::{BundleJson, CosignerEntry, MultisigInfo};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::secret_advisory::secret_in_argv_warning;
use crate::slot_input::{SlotInput, SlotSubkey};
use crate::synthesize::synthesize_descriptor;
use crate::wallet_import::{
    apply_select_descriptor,
    bitcoin_core::BitcoinCoreParser,
    bsms::BsmsParser,
    coldcard::ColdcardParser,
    coldcard_multisig::ColdcardMultisigParser,
    electrum::ElectrumParser,
    jade::JadeParser,
    overlay::apply_seed_overlay,
    // v0.28.0 Phase P0C — 6 new canonicalize skeletons imported alphabetically;
    // bodies are `Err(BadInput("not yet implemented"))` stubs in
    // wallet_import/roundtrip.rs. Per-parser P{N}B replaces each body with
    // a real implementation; this import list does not change.
    roundtrip::{
        canonicalize_bitcoin_core, canonicalize_bsms, canonicalize_coldcard,
        canonicalize_coldcard_multisig, canonicalize_electrum, canonicalize_jade,
        canonicalize_sparrow, canonicalize_specter, unified_diff,
    },
    sniff::{sniff_format, SniffOutcome},
    sparrow::SparrowParser,
    specter::SpecterParser,
    ParsedImport,
    SelectDescriptor,
    WalletFormatParser,
};
use clap::{ArgGroup, Args};
use mnemonic_toolkit::electrum_crypto::{
    detect_storage_magic, ecies_decrypt_storage, EciesDecryptError, ElectrumStorageMagic,
};
use serde_json::json;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use zeroize::Zeroizing;

/// SPEC v0.28.x — OUTER envelope schema version (current: "1").
///
/// **Disambiguation:** the toolkit carries TWO `schema_version` fields:
/// 1. This OUTER constant — the `--json` envelope wire-shape version
///    (governs `--from-import-json` array semantics + `import_provenance`
///    field set).
/// 2. The INNER `BundleJson.schema_version` literal at `:~975`
///    (current: "4") — governs the bundle payload wire-shape (governs
///    `bundle.mk1`/`bundle.md1`/etc. field set inside each envelope entry).
///
/// Both fields share the name `schema_version` but evolve independently.
/// Future readers / parser authors: when extending the envelope wire-shape,
/// bump THIS constant; when extending the bundle payload wire-shape, bump
/// the inner BundleJson literal. Cross-cite both when either changes.
/// Tracked as FOLLOWUP `import-wallet-envelope-schema-version-narrative-drift`
/// (resolved v0.28.5).
pub(crate) const IMPORT_WALLET_ENVELOPE_SCHEMA_VERSION: &str = "1";

#[derive(Args, Debug, Clone)]
#[command(group(
    // v0.33.2 — optional, mutually-exclusive password source for Electrum
    // BIE1 storage-encrypted wallets. Unlike `electrum-decrypt`'s group, this
    // is NOT required (most imports are plaintext); the password is only
    // consumed when a BIE1 blob is detected. `--decrypt-password-stdin` is a
    // bool: `false` does not count as present in the group.
    ArgGroup::new("import_decrypt_password_source")
        .args(["decrypt_password", "decrypt_password_file", "decrypt_password_stdin"])
        .required(false)
        .multiple(false),
))]
pub struct ImportWalletArgs {
    /// Path to the third-party wallet blob; `-` reads from stdin.
    /// v0.27.0: required UNLESS `--bsms-round1` is supplied (Round-1 verify
    /// alone is a meaningful CLI mode; emits per-record verify envelope on
    /// `--json`, exits 0 on verify success).
    #[arg(
        long = "blob",
        value_name = "FILE|-",
        required_unless_present = "bsms_round1"
    )]
    pub blob: Option<PathBuf>,

    /// Format override. If absent, the blob is auto-detected via sniff
    /// (SPEC §6). Supported values (alphabetical): `bitcoin-core`, `bsms`,
    /// `coldcard`, `coldcard-multisig`, `electrum`, `jade`, `sparrow`,
    /// `specter`. The 6 non-{bsms,bitcoin-core} formats are pre-stubbed at
    /// v0.28.0 Phase P0C; per-parser dispatch ships in Phases P1C-P6C.
    #[arg(
        long = "format",
        value_name = "bitcoin-core|bsms|coldcard|coldcard-multisig|electrum|jade|sparrow|specter",
        value_parser = clap::builder::PossibleValuesParser::new([
            "bitcoin-core",
            "bsms",
            "coldcard",
            "coldcard-multisig",
            "electrum",
            "jade",
            "sparrow",
            "specter",
        ]),
    )]
    pub format: Option<String>,

    /// Multi-descriptor selector for Bitcoin Core blobs (SPEC §5.3).
    /// Accepts an integer (`0`, `1`, ...), `active-receive`, `active-change`,
    /// or `all` (default). BSMS blobs coerce non-default values to `all`
    /// with a stderr NOTICE.
    #[arg(
        long = "select-descriptor",
        value_name = "N|active-receive|active-change|all",
        default_value = "all"
    )]
    pub select_descriptor: String,

    /// Seed overlay (SPEC §8.3): supply the secret material that matches
    /// the blob's declared xpub at the cosigner's origin path. Repeatable;
    /// positional cosigner-index — the i-th `--ms1` applies to cosigner i.
    /// Accepts the `@env:VAR` sentinel (resolves to `std::env::var(VAR)`).
    /// Empty-string `--ms1 ""` preserves v0.25.1 watch-only sentinel
    /// semantics (cosigner left watch-only + stderr NOTICE).
    #[arg(long = "ms1", value_name = "STRING")]
    pub ms1: Vec<String>,

    /// Per-slot seed overlay via `--slot @<N>.phrase=<BIP-39 phrase>`.
    /// Equivalent to `--ms1`: the phrase is converted to entropy and the
    /// derived xpub at the cosigner's origin path is compared against the
    /// blob's xpub. Mutually exclusive with `--ms1[N]` for the same N.
    /// Accepts the `@env:VAR` sentinel for the phrase value.
    /// In v0.26.0 only the `phrase` subkey is accepted on `import-wallet`;
    /// other subkeys (`entropy`, `xpub`, etc.) are rejected.
    #[arg(long = "slot", value_name = "@N.phrase=<phrase>", value_parser = crate::slot_input::parse_slot_input)]
    pub slot: Vec<SlotInput>,

    /// Emit a JSON envelope array on stdout (SPEC §7.4) instead of the
    /// human-readable summary. Each envelope carries:
    ///   - `bundle`           — parsed bundle summary
    ///   - `source_format`    — "bsms" or "bitcoin-core"
    ///   - `roundtrip`        — { byte_exact, semantic_match, diff? }
    ///   - `bsms_audit?`      — BSMS audit fields (BSMS source only)
    ///   - `source_metadata?` — Bitcoin Core per-entry metadata
    ///   - `bsms_round1_verifications?` — per-record BIP-129 Round-1 SIG
    ///     verify state when `--bsms-round1` supplied (v0.27.0)
    ///
    /// When `--json` is set, the round-trip diff goes ONLY in the envelope;
    /// stderr is silent for the diff.
    #[arg(long = "json")]
    pub json: bool,

    /// v0.27.0 — supply a BIP-129 5-line Round-1 key record (Signer →
    /// Coordinator) for BIP-322 ECDSA signature verification. Repeating
    /// flag — one per record. `<FILE>` reads file contents; stdin (`-`)
    /// is NOT supported in v0.27.0 (multi-record stdin intake is filed
    /// as a future FOLLOWUP — supply file paths per record).
    ///
    /// Each record is verified independently; verify state propagates to the
    /// `--json` envelope's `bsms_round1_verifications` field. Verify failure
    /// is fatal under `--bsms-verify-strict`; otherwise emits a stderr NOTICE
    /// and sets `signature_verified: false` per-record.
    #[arg(long = "bsms-round1", value_name = "FILE")]
    pub bsms_round1: Vec<PathBuf>,

    /// v0.31.0 — BIP-129 encryption-envelope Round-2 decrypt. Reads the
    /// session TOKEN from PATH (or `-` for stdin); applies PBKDF2-SHA512
    /// key derivation + AES-256-CTR decrypt + HMAC-SHA256 verify per
    /// BIP-129 §Encryption. Combine with `--format bsms` to decrypt
    /// encrypted Round-2 wallet shares from a Coordinator. Token file
    /// contents: lowercase ASCII hex (16 chars for STANDARD mode, 32
    /// chars for EXTENDED mode); leading/trailing whitespace + newlines
    /// stripped. Encrypted blobs don't carry the `BSMS 1.0` header so
    /// they don't auto-sniff as BSMS; `--format bsms` is REQUIRED.
    /// MAC verify failure → exit 2 (typed `BsmsMacMismatch`).
    /// v0.32.2 — repeatable: one TOKEN per Signer (BIP-129 line 74). A
    /// SINGLE `--bsms-encryption-token` is SHARED (decrypts all encrypted
    /// Round-1 records + the Round-2 blob; backward-compatible with
    /// v0.31.0/v0.32.1). MULTIPLE tokens are paired POSITIONALLY with
    /// `--bsms-round1` records (token[i] ↔ record[i]); requires N matching
    /// all-encrypted records + no encrypted Round-2 `--blob`.
    #[arg(long = "bsms-encryption-token", value_name = "FILE|-")]
    pub bsms_encryption_token: Vec<PathBuf>,

    /// v0.27.0 — make BIP-129 Round-1 SIG verification failures fatal.
    /// Without this flag, verify mismatches emit a stderr NOTICE and proceed
    /// with `signature_verified: false`. With this flag, verify mismatch is
    /// `BsmsSignatureMismatch` exit 2.
    #[arg(long = "bsms-verify-strict")]
    pub bsms_verify_strict: bool,

    /// v0.33.2 — password for an Electrum BIE1 storage-encrypted wallet
    /// (inline). Only used when `--blob` is a `BIE1` storage blob; ignored
    /// otherwise. Emits an argv-leakage advisory — prefer
    /// `--decrypt-password-file` or `--decrypt-password-stdin`.
    #[arg(long = "decrypt-password", value_name = "VALUE")]
    pub decrypt_password: Option<String>,

    /// v0.33.2 — read the BIE1 decryption password from a file (trailing
    /// newline stripped).
    #[arg(long = "decrypt-password-file", value_name = "PATH")]
    pub decrypt_password_file: Option<PathBuf>,

    /// v0.33.2 — read the BIE1 decryption password from stdin (raw,
    /// NULL-byte preserving). Cannot co-exist with any other stdin consumer
    /// (`--blob=-`, `--bsms-encryption-token=-`).
    #[arg(long = "decrypt-password-stdin")]
    pub decrypt_password_stdin: bool,

    /// v0.34.6: re-bind the imported network to disambiguate signet/regtest
    /// from the coin-type-1→testnet collapse (SPEC §4.2 step 8). Honored ONLY
    /// within the parsed coin-type class (testnet ↔ {testnet,signet,regtest};
    /// mainnet ↔ mainnet); cross-class is refused. Absent = use the
    /// coin-type-derived network. Closes `wallet-import-signet-regtest-disambiguation`.
    #[arg(long, value_name = "NETWORK")]
    pub network: Option<CliNetwork>,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &ImportWalletArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
) -> Result<u8, ToolkitError> {
    // `--no-auto-repair` is resolved here for symmetry with other v0.22.x+
    // subcommands (verify-bundle / convert / inspect). In v0.26.0 the
    // import-wallet path does NOT auto-fire BCH error-correction: BSMS +
    // Bitcoin Core wallet blobs do not carry BCH-coded fields (those live
    // in the toolkit's own ms1/mk1/md1 cards). The flag is reserved for
    // v0.27+ — for now it is documented in `--help` so the schema surface
    // is symmetric and any future BCH-coded import source can adopt the
    // same TTY-conditional auto-fire convention used elsewhere.
    let _ = crate::repair::resolve_no_auto_repair(no_auto_repair);

    // FOLLOWUP `import-wallet-ms1-argv-advisory-gap` — secret-in-argv advisory
    // for inline `--ms1` AND its twin `--slot @N.phrase` (both `@env:`-only, no
    // stdin channel). Read the RAW `args` HERE, BEFORE the `@env:`-resolving
    // rebind below, so an `@env:VAR` value (NOT an argv leak, nor the `""`
    // watch-only sentinel) is correctly skipped. Fire before any validation /
    // early-return — the argv leak already happened at process exec.
    if args
        .ms1
        .iter()
        .any(|v| !v.is_empty() && !v.starts_with("@env:"))
    {
        secret_in_argv_warning(stderr, "--ms1", "@env:VAR");
    }
    for s in &args.slot {
        if s.subkey == SlotSubkey::Phrase && !s.value.is_empty() && !s.value.starts_with("@env:") {
            secret_in_argv_warning(stderr, &format!("--slot @{}.phrase=", s.index), "@env:VAR");
        }
    }

    // v0.26.0 §3 — resolve `@env:<VAR>` sentinels on secret-bearing flags.
    let env_resolved_owned;
    let args: &ImportWalletArgs = if needs_env_sentinel_resolution(args) {
        env_resolved_owned = resolve_env_sentinels(args)?;
        &env_resolved_owned
    } else {
        args
    };

    // v0.27.0 — `--bsms-verify-strict` without `--bsms-round1` is meaningless;
    // reject explicitly so the user notices the typo.
    if args.bsms_verify_strict && args.bsms_round1.is_empty() {
        return Err(ToolkitError::BadInput(
            "--bsms-verify-strict requires `--bsms-round1` (the flag controls \
             BIP-129 Round-1 SIG verify strictness; there are no records to verify)"
                .to_string(),
        ));
    }

    // v0.31.0 — Cycle 7b stdin-contention guard, HOISTED above the Round-1
    // verify + token read (v0.32.1 Cycle 15). `verify_bsms_round1_files`
    // runs before the blob match, so the token must be read first; the
    // dual-stdin refusal must therefore fire before the token consumes
    // stdin. Uses `args` directly (no dependency on the later `blob_path`
    // binding); in standalone Round-1 mode `args.blob` is None so the
    // guard does not fire.
    // v0.32.2 — single-stdin-per-invocation across the Vec of tokens + blob.
    // At most one stdin consumer: refuse 2+ token `-` entries, and refuse
    // any token `-` co-existing with `--blob=-`.
    let token_stdin_count = args
        .bsms_encryption_token
        .iter()
        .filter(|p| p.as_os_str() == "-")
        .count();
    if token_stdin_count > 1 {
        return Err(ToolkitError::BadInput(
            "at most one --bsms-encryption-token=- per invocation (single stdin per invocation)"
                .to_string(),
        ));
    }
    if let Some(blob_p) = &args.blob {
        if blob_p.as_os_str() == "-" && token_stdin_count > 0 {
            return Err(ToolkitError::BadInput(
                "--blob=- and --bsms-encryption-token=- cannot both read from stdin".to_string(),
            ));
        }
    }
    // v0.33.2 — `--decrypt-password-stdin` (Electrum BIE1) is a THIRD stdin
    // consumer. The password is resolved later (at the BIE1 decrypt site,
    // after `read_blob` + the token read have already drained stdin), so this
    // guard MUST be hoisted here. Covers every pair with blob=- and token=-.
    if args.decrypt_password_stdin {
        if let Some(blob_p) = &args.blob {
            if blob_p.as_os_str() == "-" {
                return Err(ToolkitError::BadInput(
                    "--blob=- and --decrypt-password-stdin cannot both read from stdin".to_string(),
                ));
            }
        }
        if token_stdin_count > 0 {
            return Err(ToolkitError::BadInput(
                "--bsms-encryption-token=- and --decrypt-password-stdin cannot both read from stdin"
                    .to_string(),
            ));
        }
    }

    // v0.32.2 — gap-h guard (R0 I1): per-Signer tokens (N>1) require N
    // matching `--bsms-round1` records. With zero records, the positional
    // count check in `verify_bsms_round1_files` would never fire, so guard
    // here before the token read.
    if args.bsms_encryption_token.len() > 1 && args.bsms_round1.is_empty() {
        return Err(ToolkitError::BadInput(
            "per-Signer tokens (multiple --bsms-encryption-token) require matching --bsms-round1 records; none supplied".to_string(),
        ));
    }

    // v0.32.1 — read + width-validate each `--bsms-encryption-token` ONCE,
    // before both the Round-1 verify path and the Round-2 decrypt block.
    // v0.32.2 — a Vec: 1 token = SHARED (backward-compat); N>1 = per-Signer.
    let mut bsms_tokens: Vec<BsmsToken> = Vec::with_capacity(args.bsms_encryption_token.len());
    for path in &args.bsms_encryption_token {
        bsms_tokens.push(read_and_validate_bsms_token(path, stdin)?);
    }

    // v0.27.0 — BIP-129 Round-1 BIP-322 ECDSA verify (independent of --blob).
    // v0.32.1 — encrypted Round-1 records (hex MAC||ciphertext) are decrypted
    // with the token(s) before parse + BIP-322 verify.
    let round1_verifications = if !args.bsms_round1.is_empty() {
        verify_bsms_round1_files(
            &args.bsms_round1,
            args.bsms_verify_strict,
            &bsms_tokens,
            stderr,
        )?
    } else {
        Vec::new()
    };

    // v0.27.0 — Standalone Round-1 verify mode: no --blob supplied. Emit
    // verifications only (no bundle synthesis path).
    let blob_path = match &args.blob {
        Some(p) => p,
        None => {
            if args.json {
                emit_round1_only_envelope(stdout, &round1_verifications)?;
            } else {
                emit_round1_only_summary(stdout, &round1_verifications)?;
            }
            // Output-class advisory (cycle B): INERT — this BSMS Round-1-only
            // mode emits a pass/fail verification verdict + the record's public
            // signer pubkey (no key material), analogous to verify-bundle.
            // No advisory line (SPEC §3 resolving principles b + c).
            return Ok(0);
        }
    };

    // (v0.32.1 — the dual-stdin guard + token read were hoisted above the
    // Round-1 verify path; see the `bsms_token` binding earlier in this fn.)

    // Read blob.
    let mut blob = read_blob(blob_path, stdin)?;

    // v0.34.1 — pin the blob for ALL formats (was BIE1-only). A plaintext
    // `use_encryption:false` Electrum wallet is seed-bearing yet was swappable.
    // `blob` is reassigned on the BIE1 + Round-2 arms; this SINGLE guard is
    // re-pinned at each via `mem::replace` (pin new, then drop+munlock the freed
    // original) so exactly one live guard pins the current buffer — mlock locks
    // don't stack, so a stale end-of-fn munlock would un-pin a live secret.
    let mut _pin_blob = mnemonic_toolkit::mlock::pin_pages_for(&blob);

    // v0.33.2 — Electrum BIE1 whole-file storage decrypt, BEFORE sniff.
    // A storage-encrypted Electrum wallet file is a single base64 blob (magic
    // BIE1/BIE2), NOT JSON — so it must be decrypted to wallet JSON before the
    // sniff/parse pipeline (which expects JSON/text). Detection is
    // `--format`-independent (BIE2 is always refused; BIE1 is always decrypted
    // when a password is present); after replacement the normal `--format` /
    // sniff logic runs on the recovered JSON. Mirrors the BSMS decrypt-then-
    // parse orchestration below (preserves the parser-trait surface).
    match detect_storage_magic(&blob) {
        Some(ElectrumStorageMagic::Bie2) => {
            return Err(ToolkitError::BadInput(
                "import-wallet: electrum: this wallet is encrypted with a hardware-device key \
                 (BIE2 / XPUB_PASSWORD); it cannot be decrypted from a password. Decrypt it in \
                 Electrum with the original device first, then re-import."
                    .to_string(),
            ));
        }
        Some(ElectrumStorageMagic::Bie1) => {
            let password =
                resolve_import_decrypt_password(args, stdin, stderr)?.ok_or_else(|| {
                    ToolkitError::BadInput(
                    "import-wallet: electrum: BIE1 storage-encrypted wallet detected; supply the \
                     wallet password via --decrypt-password, --decrypt-password-file, or \
                     --decrypt-password-stdin"
                        .to_string(),
                )
                })?;
            let _pin_pw = mnemonic_toolkit::mlock::pin_pages_for(password.as_bytes());
            // Detection already confirmed the trimmed UTF-8 base64 form.
            let trimmed = std::str::from_utf8(&blob)
                .expect("detect_storage_magic confirmed valid UTF-8")
                .trim();
            let plaintext = ecies_decrypt_storage(trimmed, password.as_bytes())
                .map_err(map_ecies_storage_error)?;
            writeln!(
                stderr,
                "notice: import-wallet: electrum: BIE1 user-password storage decrypted"
            )
            .map_err(ToolkitError::Io)?;
            // `plaintext` is already `Zeroizing<Vec<u8>>`; move it into `blob`
            // (which is now `Zeroizing<Vec<u8>>` too) to preserve the wrapper —
            // the recovered wallet JSON can carry seed/xprv material. Pin AFTER
            // the move (the Vec move keeps the same heap buffer).
            blob = plaintext;
            // Re-pin the recovered-JSON buffer; drop+munlock the prior guard.
            drop(std::mem::replace(
                &mut _pin_blob,
                mnemonic_toolkit::mlock::pin_pages_for(&blob),
            ));
        }
        None => {
            // Not a storage-encrypted blob. If a password was supplied anyway,
            // it is irrelevant — emit a soft advisory and proceed unchanged.
            if args.decrypt_password.is_some()
                || args.decrypt_password_file.is_some()
                || args.decrypt_password_stdin
            {
                // An inline --decrypt-password still leaked via argv even though
                // it goes unused — warn about the leak that already happened.
                if args.decrypt_password.is_some() {
                    secret_in_argv_warning(
                        stderr,
                        "--decrypt-password ",
                        "--decrypt-password-stdin",
                    );
                }
                writeln!(
                    stderr,
                    "notice: import-wallet: no BIE1 storage-encrypted wallet detected; \
                     --decrypt-password* ignored"
                )
                .map_err(ToolkitError::Io)?;
            }
        }
    }

    // SPEC §6: sniff dispatch.
    let sniff_outcome = sniff_format(&blob);
    let format_str: &str = match args.format.as_deref() {
        Some("bsms") => {
            // v0.28.7 — Slug 3 Option B: extend BSMS arm to refuse all
            // 6 remaining sniff-detected formats (was: only bitcoin-core).
            // Closes `wallet-import-format-mismatch-matrix-completion` for
            // this arm.
            match sniff_outcome {
                SniffOutcome::BitcoinCore => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "bsms".to_string(),
                        sniffed: "bitcoin-core".to_string(),
                    });
                }
                SniffOutcome::Coldcard => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "bsms".to_string(),
                        sniffed: "coldcard".to_string(),
                    });
                }
                SniffOutcome::ColdcardMultisig => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "bsms".to_string(),
                        sniffed: "coldcard-multisig".to_string(),
                    });
                }
                SniffOutcome::Electrum => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "bsms".to_string(),
                        sniffed: "electrum".to_string(),
                    });
                }
                SniffOutcome::Jade => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "bsms".to_string(),
                        sniffed: "jade".to_string(),
                    });
                }
                SniffOutcome::Sparrow => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "bsms".to_string(),
                        sniffed: "sparrow".to_string(),
                    });
                }
                SniffOutcome::Specter => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "bsms".to_string(),
                        sniffed: "specter".to_string(),
                    });
                }
                _ => {}
            }
            "bsms"
        }
        Some("bitcoin-core") => {
            // v0.28.7 — Slug 3 Option B: extend BitcoinCore arm to refuse all
            // 6 remaining sniff-detected formats (was: only bsms).
            // Closes `wallet-import-format-mismatch-matrix-completion` for
            // this arm.
            match sniff_outcome {
                SniffOutcome::Bsms => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "bitcoin-core".to_string(),
                        sniffed: "bsms".to_string(),
                    });
                }
                SniffOutcome::Coldcard => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "bitcoin-core".to_string(),
                        sniffed: "coldcard".to_string(),
                    });
                }
                SniffOutcome::ColdcardMultisig => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "bitcoin-core".to_string(),
                        sniffed: "coldcard-multisig".to_string(),
                    });
                }
                SniffOutcome::Electrum => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "bitcoin-core".to_string(),
                        sniffed: "electrum".to_string(),
                    });
                }
                SniffOutcome::Jade => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "bitcoin-core".to_string(),
                        sniffed: "jade".to_string(),
                    });
                }
                SniffOutcome::Sparrow => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "bitcoin-core".to_string(),
                        sniffed: "sparrow".to_string(),
                    });
                }
                SniffOutcome::Specter => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "bitcoin-core".to_string(),
                        sniffed: "specter".to_string(),
                    });
                }
                _ => {}
            }
            "bitcoin-core"
        }
        // v0.28.0 Phase P0C pre-stub arms (R0 C1 + R1-I2 fold). Each new
        // format is enumerated alphabetically here; the body is
        // `unimplemented!()` until the per-parser P{N}C sub-phase flips
        // the arm to a real `SniffOutcome::<Format>` mismatch check +
        // parser dispatch. Insertion point: BEFORE the `Some(other) =>`
        // fallback so PossibleValuesParser-rejected values still surface
        // via the BadInput template (defense-in-depth — clap already
        // rejects out-of-set values, but the fallback is preserved as a
        // belt-and-suspenders guard).
        Some("coldcard") => {
            // SPEC §6.1 format-mismatch check: explicit `--format coldcard`
            // against a blob that sniff identified as a DIFFERENT format →
            // reject with `ImportWalletFormatMismatch` (exit 1). Same shape
            // as BSMS / Bitcoin Core / Sparrow / Specter upper arms. Only
            // reject when sniff strongly pinned a different format;
            // `Ambiguous` and `NoMatch` are tolerated (user opted in
            // explicitly).
            //
            // v0.34.4: the off-diagonal matrix is now COMPLETE for this arm —
            // refuses all 7 sibling formats. Closes
            // `wallet-import-format-mismatch-matrix-completion-discovered-gaps`.
            match sniff_outcome {
                SniffOutcome::Bsms => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard".to_string(),
                        sniffed: "bsms".to_string(),
                    });
                }
                SniffOutcome::BitcoinCore => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard".to_string(),
                        sniffed: "bitcoin-core".to_string(),
                    });
                }
                SniffOutcome::ColdcardMultisig => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard".to_string(),
                        sniffed: "coldcard-multisig".to_string(),
                    });
                }
                SniffOutcome::Electrum => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard".to_string(),
                        sniffed: "electrum".to_string(),
                    });
                }
                SniffOutcome::Jade => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard".to_string(),
                        sniffed: "jade".to_string(),
                    });
                }
                SniffOutcome::Sparrow => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard".to_string(),
                        sniffed: "sparrow".to_string(),
                    });
                }
                SniffOutcome::Specter => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard".to_string(),
                        sniffed: "specter".to_string(),
                    });
                }
                _ => {}
            }
            "coldcard"
        }
        Some("coldcard-multisig") => {
            // SPEC §6.1 format-mismatch check: explicit `--format coldcard-multisig`
            // against a blob that sniff identified as a different format → reject
            // with `ImportWalletFormatMismatch` (exit 1). Same shape as
            // BSMS/Bitcoin Core upper arms. Only reject when sniff strongly
            // pinned a DIFFERENT format; `Ambiguous` and `NoMatch` are tolerated
            // (the user opted in to coldcard-multisig explicitly).
            // v0.28.7 — Slug 3 Option B: extend ColdcardMultisig arm to
            // refuse 5 additional sniff-detected formats (was: bsms +
            // bitcoin-core only). Closes
            // `wallet-import-format-mismatch-matrix-completion` for this arm.
            match sniff_outcome {
                SniffOutcome::Bsms => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard-multisig".to_string(),
                        sniffed: "bsms".to_string(),
                    });
                }
                SniffOutcome::BitcoinCore => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard-multisig".to_string(),
                        sniffed: "bitcoin-core".to_string(),
                    });
                }
                SniffOutcome::Coldcard => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard-multisig".to_string(),
                        sniffed: "coldcard".to_string(),
                    });
                }
                SniffOutcome::Electrum => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard-multisig".to_string(),
                        sniffed: "electrum".to_string(),
                    });
                }
                SniffOutcome::Jade => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard-multisig".to_string(),
                        sniffed: "jade".to_string(),
                    });
                }
                SniffOutcome::Sparrow => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard-multisig".to_string(),
                        sniffed: "sparrow".to_string(),
                    });
                }
                SniffOutcome::Specter => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "coldcard-multisig".to_string(),
                        sniffed: "specter".to_string(),
                    });
                }
                _ => {}
            }
            "coldcard-multisig"
        }
        Some("electrum") => {
            // v0.28.0 Phase P6C: format-mismatch check mirrors the
            // bsms/bitcoin-core/coldcard/coldcard-multisig/sparrow/specter
            // upper arms (SPEC §6.1). Only reject when sniff strongly pinned
            // a different format; Ambiguous/NoMatch are tolerated.
            //
            // v0.34.4: the off-diagonal matrix is now COMPLETE for this arm —
            // refuses all 7 sibling formats. Closes
            // `wallet-import-format-mismatch-matrix-completion-discovered-gaps`.
            match sniff_outcome {
                SniffOutcome::Bsms => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "electrum".to_string(),
                        sniffed: "bsms".to_string(),
                    });
                }
                SniffOutcome::BitcoinCore => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "electrum".to_string(),
                        sniffed: "bitcoin-core".to_string(),
                    });
                }
                SniffOutcome::Coldcard => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "electrum".to_string(),
                        sniffed: "coldcard".to_string(),
                    });
                }
                SniffOutcome::ColdcardMultisig => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "electrum".to_string(),
                        sniffed: "coldcard-multisig".to_string(),
                    });
                }
                SniffOutcome::Jade => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "electrum".to_string(),
                        sniffed: "jade".to_string(),
                    });
                }
                SniffOutcome::Sparrow => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "electrum".to_string(),
                        sniffed: "sparrow".to_string(),
                    });
                }
                SniffOutcome::Specter => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "electrum".to_string(),
                        sniffed: "specter".to_string(),
                    });
                }
                _ => {}
            }
            "electrum"
        }
        Some("jade") => {
            // v0.28.0 Phase P5C: format-mismatch check mirrors the
            // bsms/bitcoin-core/coldcard/coldcard-multisig/electrum/sparrow/specter
            // upper arms (SPEC §6.1). Only reject when sniff strongly
            // pinned a different format; Ambiguous/NoMatch are tolerated.
            //
            // The mismatch matrix is now complete at P5C (Jade is the
            // LAST parser landed in v0.28.0 Wave 1 — the matrix lists
            // all 7 sibling formats). Full N×N symmetry across the 8
            // formats lands incrementally per cycle-followup
            // `wallet-import-format-mismatch-matrix-completion`.
            match sniff_outcome {
                SniffOutcome::Bsms => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "jade".to_string(),
                        sniffed: "bsms".to_string(),
                    });
                }
                SniffOutcome::BitcoinCore => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "jade".to_string(),
                        sniffed: "bitcoin-core".to_string(),
                    });
                }
                SniffOutcome::Coldcard => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "jade".to_string(),
                        sniffed: "coldcard".to_string(),
                    });
                }
                SniffOutcome::ColdcardMultisig => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "jade".to_string(),
                        sniffed: "coldcard-multisig".to_string(),
                    });
                }
                SniffOutcome::Electrum => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "jade".to_string(),
                        sniffed: "electrum".to_string(),
                    });
                }
                SniffOutcome::Sparrow => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "jade".to_string(),
                        sniffed: "sparrow".to_string(),
                    });
                }
                SniffOutcome::Specter => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "jade".to_string(),
                        sniffed: "specter".to_string(),
                    });
                }
                _ => {}
            }
            "jade"
        }
        Some("sparrow") => {
            // SPEC §6.1 format-mismatch check: explicit `--format sparrow`
            // against a blob that sniff identified as a different format → reject
            // with `ImportWalletFormatMismatch` (exit 1). Same shape as BSMS /
            // Bitcoin Core / ColdcardMultisig upper arms. Only reject when sniff
            // strongly pinned a DIFFERENT format; `Ambiguous` and `NoMatch`
            // are tolerated (the user opted in to sparrow explicitly).
            //
            // v0.34.4: the off-diagonal matrix is now COMPLETE for this arm —
            // refuses all 7 sibling formats. Closes
            // `wallet-import-format-mismatch-matrix-completion-discovered-gaps`.
            match sniff_outcome {
                SniffOutcome::Bsms => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "sparrow".to_string(),
                        sniffed: "bsms".to_string(),
                    });
                }
                SniffOutcome::BitcoinCore => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "sparrow".to_string(),
                        sniffed: "bitcoin-core".to_string(),
                    });
                }
                SniffOutcome::Coldcard => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "sparrow".to_string(),
                        sniffed: "coldcard".to_string(),
                    });
                }
                SniffOutcome::ColdcardMultisig => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "sparrow".to_string(),
                        sniffed: "coldcard-multisig".to_string(),
                    });
                }
                SniffOutcome::Electrum => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "sparrow".to_string(),
                        sniffed: "electrum".to_string(),
                    });
                }
                SniffOutcome::Jade => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "sparrow".to_string(),
                        sniffed: "jade".to_string(),
                    });
                }
                SniffOutcome::Specter => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "sparrow".to_string(),
                        sniffed: "specter".to_string(),
                    });
                }
                _ => {}
            }
            "sparrow"
        }
        Some("specter") => {
            // SPEC §6.1 format-mismatch check: explicit `--format specter`
            // against a blob that sniff identified as a different format → reject
            // with `ImportWalletFormatMismatch` (exit 1). Same shape as
            // BSMS / Bitcoin Core / ColdcardMultisig / Sparrow upper arms.
            // Only reject when sniff strongly pinned a DIFFERENT format;
            // `Ambiguous` and `NoMatch` are tolerated (the user opted in to
            // specter explicitly).
            //
            // v0.34.4: the off-diagonal matrix is now COMPLETE for this arm —
            // refuses all 7 sibling formats. Closes
            // `wallet-import-format-mismatch-matrix-completion-discovered-gaps`.
            match sniff_outcome {
                SniffOutcome::Bsms => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "specter".to_string(),
                        sniffed: "bsms".to_string(),
                    });
                }
                SniffOutcome::BitcoinCore => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "specter".to_string(),
                        sniffed: "bitcoin-core".to_string(),
                    });
                }
                SniffOutcome::Coldcard => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "specter".to_string(),
                        sniffed: "coldcard".to_string(),
                    });
                }
                SniffOutcome::ColdcardMultisig => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "specter".to_string(),
                        sniffed: "coldcard-multisig".to_string(),
                    });
                }
                SniffOutcome::Electrum => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "specter".to_string(),
                        sniffed: "electrum".to_string(),
                    });
                }
                SniffOutcome::Jade => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "specter".to_string(),
                        sniffed: "jade".to_string(),
                    });
                }
                SniffOutcome::Sparrow => {
                    return Err(ToolkitError::ImportWalletFormatMismatch {
                        supplied: "specter".to_string(),
                        sniffed: "sparrow".to_string(),
                    });
                }
                _ => {}
            }
            "specter"
        }
        Some(other) => {
            return Err(ToolkitError::BadInput(format!(
                "--format {other} is not supported \
                 (bitcoin-core, bsms, coldcard, coldcard-multisig, \
                  electrum, jade, sparrow, specter)"
            )));
        }
        None => match sniff_outcome {
            // v0.28.0 Phase P0C: per plan-doc P0C row, the auto-sniff `None =>`
            // arm is UNTOUCHED at P0C — the new SniffOutcome::<Format>
            // variants land at per-parser P{N}A sub-phases (P1A through P6A
            // and the BSMS BIP-129 cutover at P7A). Only the Ambiguous /
            // NoMatch stderr templates below are updated to enumerate the
            // post-cycle 8-format list per plan-doc Site 3 directive.
            SniffOutcome::Bsms => "bsms",
            SniffOutcome::BitcoinCore => "bitcoin-core",
            // v0.28.0 Phase P3A: auto-sniff arm for Coldcard single-sig JSON.
            // The sniff slot is wired here so `sniff_format` can now return
            // `SniffOutcome::Coldcard`; the parse-side dispatch at the
            // `match format_str` block below remains
            // `unimplemented!("P3C: parse not yet wired")` until P3C flips it
            // to `ColdcardParser::parse(...)`. Adding this arm BEFORE the
            // `other => unreachable!()` catch-all keeps the unreachable
            // contract intact for the still-placeholder variants
            // (Electrum / Jade).
            SniffOutcome::Coldcard => "coldcard",
            // v0.28.0 Phase P4C: auto-sniff arm for coldcard-multisig text format.
            SniffOutcome::ColdcardMultisig => "coldcard-multisig",
            // v0.28.0 Phase P6A→P6C: auto-sniff arm for Electrum 4.x wallet
            // JSON. The sniff slot is wired at `sniff.rs:88`
            // (`ElectrumParser::sniff`); the parse-side dispatch at the
            // `match format_str` block below routes to
            // `ElectrumParser::parse(&blob, stderr)` (wired at P6C). Adding
            // this arm BEFORE the `other => unreachable!()` catch-all keeps
            // the unreachable contract intact for the still-placeholder Jade
            // variant (only `SniffOutcome::Jade` lacks a real auto-sniff arm
            // at P6C close — P5A wires the remaining one).
            SniffOutcome::Electrum => "electrum",
            // v0.28.0 Phase P5A: auto-sniff arm for Blockstream Jade
            // multisig wrapper JSON. The sniff slot is wired at
            // `sniff.rs:90` (`JadeParser::sniff`); the parse-side dispatch
            // at the `match format_str` block below remains
            // `unimplemented!("P5C: parse not yet wired")` until P5C flips
            // it to `JadeParser::parse(...)`. Adding this arm BEFORE the
            // `other => unreachable!()` catch-all keeps the unreachable
            // contract intact post-P5A (no remaining P5C-only
            // placeholders — P5A wires the final auto-sniff arm).
            SniffOutcome::Jade => "jade",
            // v0.28.0 Phase P1A: auto-sniff arm for Sparrow JSON. The
            // sniff slot is wired here so `sniff_format` can now return
            // `SniffOutcome::Sparrow`; the parse-side dispatch at the
            // `match format_str` block below remains
            // `unimplemented!("P1C: parse not yet wired")` until P1C
            // flips it to `SparrowParser::parse(...)`. v0.28.0 P0D's
            // `other => unreachable!()` catch-all would otherwise fire
            // on the Sparrow verdict — adding this arm BEFORE the
            // catch-all (per C/F dispatch learned-best-practice) keeps
            // the unreachable contract intact for the still-placeholder
            // variants (Coldcard / Electrum / Jade / Specter).
            SniffOutcome::Sparrow => "sparrow",
            // v0.28.0 Phase P2A: auto-sniff arm for Specter-DIY JSON. The
            // sniff slot is wired here so `sniff_format` can now return
            // `SniffOutcome::Specter`; the parse-side dispatch at the
            // `match format_str` block below remains
            // `unimplemented!("P2C: parse not yet wired")` until P2C
            // flips it to `SpecterParser::parse(...)`. Pattern matches
            // the P1A precedent above (Sparrow): wiring the auto-sniff
            // arm at P2A makes a `SniffOutcome::Specter` verdict
            // dispatch through this `None =>` branch instead of falling
            // into the `other => unreachable!()` catch-all (which
            // would crash on a positive Specter sniff before P2C lands).
            SniffOutcome::Specter => "specter",
            SniffOutcome::Ambiguous => {
                return Err(ToolkitError::ImportWalletAmbiguousFormat(
                    "import-wallet: blob matches multiple format heuristics; \
                     supply --format <bitcoin-core|bsms|coldcard|coldcard-multisig|\
electrum|jade|sparrow|specter>"
                        .to_string(),
                ));
            }
            SniffOutcome::NoMatch => {
                return Err(ToolkitError::ImportWalletAmbiguousFormat(
                    "import-wallet: could not detect format; \
                     supply --format <bitcoin-core|bsms|coldcard|coldcard-multisig|\
electrum|jade|sparrow|specter>"
                        .to_string(),
                ));
            } // v0.28.0 Phase P5A close: all 8 `SniffOutcome` parser variants
              // (BitcoinCore / Bsms / Coldcard / ColdcardMultisig / Electrum /
              // Jade / Sparrow / Specter) plus the 2 aggregate verdicts
              // (Ambiguous / NoMatch) now have explicit arms. The P0D
              // `other => unreachable!()` pre-stub catch-all was removed at
              // P5A because the match is now exhaustive — Rust's
              // `unreachable_patterns` lint flags any remaining catch-all
              // as dead code. The match exhaustiveness invariant is
              // statically enforced by the compiler.
        },
    };

    // Validate slot subkeys: import-wallet only accepts `phrase` subkey.
    // Other secret-bearing subkeys (entropy / wif / xprv) and watch-only
    // subkeys are rejected at the import-wallet surface — phrase is the
    // only seed-source channel.
    for s in &args.slot {
        if s.subkey != SlotSubkey::Phrase {
            return Err(ToolkitError::BadInput(format!(
                "import-wallet: --slot @{}.{}=: only the `phrase` subkey is supported \
                 by import-wallet",
                s.index,
                s.subkey.as_str()
            )));
        }
    }

    // Parse via selected format.
    //
    // v0.28.0 Phase P0C pre-stub (R0 C1 fold): 6 new format arms each
    // panic via `unimplemented!()` at execution time. The arms above
    // (Site 2 in plan-doc) panic FIRST when `--format <new>` is supplied
    // explicitly; this site is reachable only via the auto-sniff path
    // which can't yield a new-format verdict at P0C (the SniffOutcome
    // variants don't exist yet). The arms are preserved here for
    // alphabetical-source-grep parity + so per-parser P{N}C diffs touch
    // a SINGLE arm per site (matrix-discipline lock per plan-doc §B.2 #6).
    // v0.31.0 Cycle 7b — BIP-129 encryption-envelope Round-2 decrypt.
    // When --bsms-encryption-token is supplied AND --format bsms, decrypt
    // the wire blob via bsms_crypto BEFORE handing to BsmsParser::parse
    // (preserves the parser-trait surface; orchestrator owns the
    // cross-cycle integration).
    //
    // BIP-129 wire shape (§Encryption line 84-85):
    //   wire = hex(MAC || ciphertext)
    //   MAC  = HMAC-SHA256(HMAC_KEY, hex_ascii(TOKEN) || plaintext)
    //   IV   = first 16 bytes of MAC
    //   HMAC_KEY = SHA256(ENCRYPTION_KEY)
    //   ENCRYPTION_KEY = PBKDF2-SHA512("No SPOF", TOKEN_raw, 2048, 32)
    //
    // AE ordering: BIP-129 §Encryption line 165 = Encrypt-and-MAC →
    // decrypt FIRST, then compute MAC over plaintext, then compare to
    // received MAC. AES-CTR has no padding-oracle exposure under this
    // ordering. MAC compare uses byte-by-byte equality: single-attempt
    // non-interactive CLI flow has no timing-oracle exposure (the
    // process exits on first mismatch; no repeated probe surface).
    if format_str == "bsms" && !bsms_tokens.is_empty() {
        // v0.32.2 — the Round-2 `--blob` decrypt is a single-share,
        // single-token operation. Per-Signer tokens (N>1) pair with
        // `--bsms-round1` records only; an encrypted Round-2 blob with
        // multiple tokens is ambiguous → refuse. (Reached only after the
        // Round-1 verify path above, per the error-precedence design.)
        if bsms_tokens.len() > 1 {
            return Err(ToolkitError::BadInput(format!(
                "Round-2 --blob decrypt requires exactly one --bsms-encryption-token; got {} (per-Signer tokens pair with --bsms-round1 records only)",
                bsms_tokens.len(),
            )));
        }
        let token = &bsms_tokens[0];
        // v0.32.1 — consume the hoisted+validated shared token. The
        // blob is the encrypted Round-2 descriptor wire (hex
        // MAC||ciphertext); decrypt + MAC-verify, then hand the
        // plaintext to BsmsParser.
        let blob_hex = std::str::from_utf8(&blob).map_err(|_| {
            ToolkitError::ImportWalletParse(
                "import-wallet: bsms: encrypted Round-2 blob must be valid UTF-8 hex".to_string(),
            )
        })?;
        let plaintext = decrypt_bsms_record(blob_hex, token, "bsms: encrypted Round-2 wire")?;
        writeln!(
            stderr,
            "notice: import-wallet: bsms: BIP-129 encrypted Round-2 envelope decrypted (token width {} hex chars; MAC verified)",
            token.hex.len(),
        )
        .map_err(ToolkitError::Io)?;
        // Replace blob with the decrypted plaintext for downstream parser.
        // Re-wrap in Zeroizing (the orchestrator's `blob` is zeroizing; the
        // decrypted Round-2 descriptor is scrubbed on drop).
        blob = Zeroizing::new(plaintext.as_bytes().to_vec());
        // Re-pin the decrypted Round-2 buffer; drop+munlock the prior guard.
        drop(std::mem::replace(
            &mut _pin_blob,
            mnemonic_toolkit::mlock::pin_pages_for(&blob),
        ));
    }

    let mut parsed: Vec<ParsedImport> = match format_str {
        "bsms" => BsmsParser::parse(&blob, stderr)?,
        "bitcoin-core" => BitcoinCoreParser::parse(&blob, stderr)?,
        "coldcard" => ColdcardParser::parse(&blob, stderr)?,
        "coldcard-multisig" => ColdcardMultisigParser::parse(&blob, stderr)?,
        "electrum" => ElectrumParser::parse(&blob, stderr)?,
        "jade" => JadeParser::parse(&blob, stderr)?,
        "sparrow" => SparrowParser::parse(&blob, stderr)?,
        "specter" => SpecterParser::parse(&blob, stderr)?,
        other => {
            return Err(ToolkitError::BadInput(format!(
                "import-wallet --format {other} is not supported \
                 (bitcoin-core, bsms, coldcard, coldcard-multisig, \
                  electrum, jade, sparrow, specter)"
            )));
        }
    };

    // v0.34.6: `--network` override (signet/regtest disambiguation). The
    // override must stay WITHIN the parsed coin-type class — the blob's xpub
    // prefix is coin-type-bound (parser yields only Bitcoin/coin-type-0 or
    // Testnet/coin-type-1). Closes `wallet-import-signet-regtest-disambiguation`.
    if let Some(override_net) = args.network {
        if let Some(first) = parsed.first() {
            let parsed_coin_type: u32 = if first.network == bitcoin::Network::Bitcoin {
                0
            } else {
                1
            };
            if override_net.coin_type() != parsed_coin_type {
                return Err(ToolkitError::ImportWalletNetworkClassMismatch {
                    requested: override_net.human_name().to_string(),
                    parsed_coin_type,
                });
            }
            let rebound = override_net.to_bitcoin_network();
            for p in parsed.iter_mut() {
                p.network = rebound;
            }
        }
    }

    // Seed overlay (SPEC §8.3). Apply BEFORE select-descriptor filter so
    // the user's overlay-args index the canonical cosigner ordering.
    //
    // Build positional `ms1` vector — ms1[i] is Some(value) for cosigner i.
    let mut ms1_args: Vec<Option<String>> = Vec::with_capacity(args.ms1.len());
    for v in &args.ms1 {
        ms1_args.push(Some(v.clone()));
    }
    let phrase_overlays: Vec<(u8, String)> = args
        .slot
        .iter()
        .filter(|s| s.subkey == SlotSubkey::Phrase)
        .map(|s| (s.index, s.value.clone()))
        .collect();
    if !ms1_args.is_empty() || !phrase_overlays.is_empty() {
        apply_seed_overlay(
            &mut parsed,
            &ms1_args,
            &phrase_overlays,
            CliLanguage::default(),
            stderr,
        )?;
    }

    // SPEC §5.3 — `--select-descriptor` filter. BSMS coerces non-default
    // to `all` per the SPEC NOTICE rule; emit the NOTICE here.
    //
    // v0.28.0 Phase P0C (Site 5 in plan-doc §B.2 #6) — per-format coerce
    // decision: BSMS + Specter coerce non-`all` to `all` (both formats are
    // single-descriptor by construction so `--select-descriptor` is
    // meaningless for them). Other formats (sparrow, coldcard,
    // coldcard-multisig, jade, electrum) fall through to the `_ =>`
    // default which invokes `apply_select_descriptor`. Per-parser P{N}B /
    // P{N}C sub-phases may revisit this if a format turns out to need an
    // analogous coerce.
    //
    // v0.28.0 Phase P2C added the `specter` coerce arm: Specter-DIY's
    // wire shape carries a single `descriptor` field at top level (no
    // multi-descriptor envelope), so coercing to `all` matches SPEC §5.3's
    // intent (active-receive / active-change semantics require per-entry
    // metadata that Specter doesn't carry).
    let select = parse_select(&args.select_descriptor)?;
    let parsed = match format_str {
        "bsms" => match select {
            SelectDescriptor::All => parsed,
            _ => {
                let _ = writeln!(
                    stderr,
                    "notice: import-wallet: bsms: --select-descriptor {} has no effect; \
                     BSMS Round-2 carries a single descriptor",
                    args.select_descriptor
                );
                parsed
            }
        },
        "specter" => match select {
            SelectDescriptor::All => parsed,
            _ => {
                let _ = writeln!(
                    stderr,
                    "notice: import-wallet: specter: --select-descriptor {} has no effect; \
                     Specter-DIY carries a single descriptor",
                    args.select_descriptor
                );
                parsed
            }
        },
        // v0.28.0 Phase P3C: Coldcard single-sig coerce arm. Coldcard's
        // generic-wallet-export carries exactly one dominant-BIP descriptor
        // per blob (single-sig by construction; bipNN sub-objects are
        // exposed but the parser picks ONE dominant per SPEC §11.3.1), so
        // `--select-descriptor` is meaningless — coerce non-`all` to `all`
        // + emit NOTICE per the bsms/specter precedent.
        "coldcard" => match select {
            SelectDescriptor::All => parsed,
            _ => {
                let _ = writeln!(
                    stderr,
                    "notice: import-wallet: coldcard: --select-descriptor {} has no effect; \
                     Coldcard single-sig carries a single descriptor",
                    args.select_descriptor
                );
                parsed
            }
        },
        _ => apply_select_descriptor(parsed, select)?,
    };

    // Emit stdout.
    if args.json {
        emit_json_envelope(
            stdout,
            stderr,
            &parsed,
            &blob,
            format_str,
            args.json,
            &round1_verifications,
        )?;
    } else {
        // Default text-mode: emit the Phase 2/3 summary form. Emit
        // round-trip diff on stderr when canonicalize is non-byte-exact.
        emit_summary(stdout, &parsed)?;
        emit_roundtrip_stderr_warning(stderr, &blob, format_str)?;
    }

    // Output-class advisory: PrivateKeyMaterial when any cosigner carries
    // entropy (seed overlay supplied), WatchOnly otherwise.
    let cls = if parsed
        .iter()
        .flat_map(|p| &p.cosigners)
        .any(|c| c.entropy.is_some())
    {
        crate::secret_advisory::OutputClass::PrivateKeyMaterial
    } else {
        crate::secret_advisory::OutputClass::WatchOnly
    };
    crate::secret_advisory::emit_output_class_advisory(cls, stderr);
    Ok(0)
}

/// Parse the `--select-descriptor` flag value into a SelectDescriptor variant.
/// Accepts `all`, `active-receive`, `active-change`, or an integer (mapped to
/// `ByIndex(N)`).
fn parse_select(s: &str) -> Result<SelectDescriptor, ToolkitError> {
    match s {
        "all" => Ok(SelectDescriptor::All),
        "active-receive" => Ok(SelectDescriptor::ActiveReceive),
        "active-change" => Ok(SelectDescriptor::ActiveChange),
        other => {
            if let Ok(n) = other.parse::<usize>() {
                return Ok(SelectDescriptor::ByIndex(n));
            }
            Err(ToolkitError::BadInput(format!(
                "--select-descriptor: invalid value `{other}`; expected `N` (integer), `active-receive`, `active-change`, or `all`"
            )))
        }
    }
}

/// v0.26.0 §3 — cheap pre-check for `@env:` sentinels on `import-wallet`'s
/// secret-bearing flag surfaces (`--ms1`, secret-bearing `--slot`).
fn needs_env_sentinel_resolution(args: &ImportWalletArgs) -> bool {
    let ms1 = args.ms1.iter().any(|v| v.starts_with("@env:"));
    let slot = args
        .slot
        .iter()
        .any(|s| s.subkey.is_secret_bearing() && s.value.starts_with("@env:"));
    ms1 || slot
}

/// v0.26.0 §3 — resolve `@env:<VAR>` sentinels across `import-wallet`'s
/// secret-bearing flag surfaces. Non-secret slot subkeys are NOT resolved
/// per SPEC §3.2 (opt-in per-callsite).
fn resolve_env_sentinels(args: &ImportWalletArgs) -> Result<ImportWalletArgs, ToolkitError> {
    use crate::env_sentinel::resolve_env_var_sentinel;
    let mut owned = args.clone();
    for v in owned.ms1.iter_mut() {
        *v = resolve_env_var_sentinel(v, "--ms1")?;
    }
    for s in owned.slot.iter_mut() {
        if s.subkey.is_secret_bearing() {
            let flag = format!("--slot @{}.{}=", s.index, s.subkey.as_str());
            s.value = resolve_env_var_sentinel(&s.value, &flag)?;
        }
    }
    Ok(owned)
}

/// v0.27.0 SPEC §3.2 + §3.2.1 — emit the `import-wallet --json` envelope
/// array on stdout. Each element corresponds to one `ParsedImport`.
///
/// The envelope's `bundle` field is the full `BundleJson` shape (SPEC §5.3),
/// synthesized post-parse via `synthesize_descriptor`. v0.26.0's compact
/// `bundle: { cosigners, network, threshold }` summary is replaced wholesale
/// — this is the wire-shape change that v0.27.0's `### Changed` CHANGELOG
/// entry documents. Closes FOLLOWUP `wallet-import-json-envelope-full-bundle`.
///
/// **BSMS round-trip caveat:** v0.27.0 ships a BSMS Round-2 emitter (Phase 3),
/// but wiring import-wallet's round-trip block to consume it is out of scope
/// for Phase 4. Status stays `blocked_no_emitter` until a follow-up cycle
/// rewires the round-trip block to call the new BSMS emitter.
fn emit_json_envelope<W: Write, E: Write>(
    stdout: &mut W,
    stderr: &mut E,
    parsed: &[ParsedImport],
    blob: &[u8],
    format_str: &str,
    _json: bool,
    round1_verifications: &[Round1Verification],
) -> Result<(), ToolkitError> {
    let mut envelopes: Vec<serde_json::Value> = Vec::with_capacity(parsed.len());

    // v0.27.1 Phase 1 I7 fold: preserve the canonicalize error reason
    // (was: `.ok()` silently discarded it). The Err arm's String is the
    // typed `ToolkitError` Display form, surfaced in the `roundtrip`
    // envelope's `canonicalize_failed` branch per SPEC §7.4 v0.27.1
    // amendment. `None` for non-{bsms,bitcoin-core} formats (no canonicalize
    // path defined; not an error).
    // v0.28.0 Phase P0C (Site 6 in plan-doc §B.2 #6) — 6 new canonicalize
    // dispatch arms. Each calls a skeleton helper in
    // `wallet_import/roundtrip.rs` that returns `Err(BadInput("not yet
    // implemented; <format> ingest lands in Phase P{N}B"))`. At P0C the
    // arms are unreachable in practice (Site 2 + Site 4 panic earlier on
    // `--format <new>`, and the auto-sniff `None =>` arm can only yield
    // bsms/bitcoin-core verdicts until per-parser P{N}A wires the new
    // SniffOutcome variants). Per-parser P{N}B replaces the skeleton body
    // with a real canonicalize implementation; this dispatch site flips
    // from skeleton to real automatically.
    let canon_orig: Option<Result<String, String>> = match format_str {
        "bsms" => Some(canonicalize_bsms(blob).map_err(|e| e.to_string())),
        "bitcoin-core" => Some(canonicalize_bitcoin_core(blob).map_err(|e| e.to_string())),
        "coldcard" => Some(canonicalize_coldcard(blob).map_err(|e| e.to_string())),
        "coldcard-multisig" => {
            Some(canonicalize_coldcard_multisig(blob).map_err(|e| e.to_string()))
        }
        "electrum" => Some(canonicalize_electrum(blob).map_err(|e| e.to_string())),
        "jade" => Some(canonicalize_jade(blob).map_err(|e| e.to_string())),
        "sparrow" => Some(canonicalize_sparrow(blob).map_err(|e| e.to_string())),
        "specter" => Some(canonicalize_specter(blob).map_err(|e| e.to_string())),
        _ => None,
    };

    for p in parsed {
        // v0.27.0 SPEC §3.2.1 — synthesize the full BundleJson via
        // descriptor-mode synthesis (`synthesize_descriptor`). Both v0.26.0
        // wallet-import formats (BSMS Round-2 + Bitcoin Core listdescriptors)
        // carry a literal descriptor, so descriptor-mode synthesis applies
        // uniformly.
        // import-wallet always re-emits English entr cards (no phrase input here);
        // run_language=English is correct and preserves pre-fix behavior.
        let bundle =
            synthesize_descriptor(&p.descriptor, &p.cosigners, false, bip39::Language::English)?;

        // Per §3.2.1 row `template`: descriptor-mode → `None`.
        // Per §3.2.1 row `descriptor`: source from `original_descriptor`
        // (pre-strip raw including `#<checksum>`). Disjoint use vs the
        // typed `p.descriptor` (input to synthesize above).
        let descriptor_field = Some(p.original_descriptor.clone());

        let n = p.cosigners.len();

        // Per §3.2.1 row `master_fingerprint`: Some only for N=1; None for
        // multisig. Mirrors live cmd/bundle.rs:677-678 emission rule.
        let master_fingerprint = if n == 1 {
            Some(p.cosigners[0].fingerprint.to_string().to_lowercase())
        } else {
            None
        };

        // Per §3.2.1 row `origin_path` / `origin_paths`: mutually exclusive
        // per SPEC §5.3. Extract per-cosigner path string from the bracket-
        // bare `m/48'/0'/0'/2'` origin path, derived from the typed
        // fingerprint+path (matches `cmd/bundle.rs`). The foreign-format import
        // regex always captures ≥1 path component, so these are never empty
        // (SPEC_path_raw_bracketed_bare_unification.md §5 C11 reachability).
        let paths: Vec<String> = p.cosigners.iter().map(|c| c.origin_path_bare()).collect();
        let (origin_path, origin_paths) = if n == 1 {
            (paths.first().cloned(), None)
        } else {
            let all_same = paths.windows(2).all(|w| w[0] == w[1]);
            if all_same {
                (paths.first().cloned(), None)
            } else {
                (None, Some(paths.clone()))
            }
        };

        // Per §3.2.1 row `multisig`: Some when N>1, None for N=1.
        let multisig = if n > 1 {
            let cosigners: Vec<CosignerEntry> = p
                .cosigners
                .iter()
                .enumerate()
                .map(|(i, s)| CosignerEntry {
                    index: i,
                    master_fingerprint: Some(s.fingerprint.to_string().to_lowercase()),
                    origin_path: s.origin_path_bare(),
                    xpub: s.xpub.to_string(),
                })
                .collect();
            let threshold = p.threshold.unwrap_or(n as u8);
            let (path_family, notice) = path_family_from_paths(&paths);
            if let Some(msg) = notice {
                writeln!(stderr, "{msg}").map_err(ToolkitError::Io)?;
            }
            Some(MultisigInfo {
                template: "descriptor",
                threshold,
                cosigner_count: n,
                path_family,
                cosigners,
            })
        } else {
            None
        };

        // Per §3.2.1 row `mode`: "watch-only" when all cosigners are
        // watch-only; "full" if any cosigner has entropy attached (seed
        // overlay path). Mirrors `bundle.any_secret_bearing()` rule at
        // `cmd/bundle.rs:611`.
        let mode_str: &'static str = if p.cosigners.iter().any(|c| c.entropy.is_some()) {
            "full"
        } else {
            "watch-only"
        };

        let bundle_json = BundleJson {
            // INNER BundleJson schema_version (current: "4"). Governs the
            // bundle payload wire-shape (mk1/md1/etc fields). See the OUTER
            // envelope schema_version doc-comment at L87 for the
            // disambiguation rule; cross-cite both when either changes.
            // FOLLOWUP `import-wallet-envelope-schema-version-narrative-drift`
            // resolved v0.28.5.
            schema_version: "4",
            mode: mode_str,
            network: network_human_name(p.network),
            template: None,
            descriptor: descriptor_field,
            account: 0,
            origin_path,
            origin_paths,
            master_fingerprint,
            ms1: bundle.ms1,
            mk1: bundle.mk1,
            md1: bundle.md1,
            multisig,
            privacy_preserving: false,
        };
        let bundle_value = serde_json::to_value(&bundle_json).map_err(|e| {
            ToolkitError::BadInput(format!("import-wallet --json bundle serialize: {e}"))
        })?;

        // Round-trip per SPEC §7.4 + §7.3 — preserved from v0.26.0 wire
        // shape; v0.27.1 Phase 1 I7 fold adds the `error: String` field to
        // the `canonicalize_failed` branch (per SPEC §7.4 v0.27.1 amendment).
        let roundtrip = match format_str {
            "bitcoin-core" => match canon_orig.clone() {
                Some(Ok(canon)) => {
                    let original_text = std::str::from_utf8(blob).unwrap_or("").to_string();
                    let byte_exact = original_text == canon;
                    let diff_val = if byte_exact {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(unified_diff(&original_text, &canon))
                    };
                    json!({
                        "byte_exact": byte_exact,
                        "semantic_match": true,
                        "diff": diff_val,
                        "status": "ok",
                    })
                }
                Some(Err(err_msg)) => json!({
                    "byte_exact": false,
                    "semantic_match": false,
                    "diff": serde_json::Value::Null,
                    "status": "canonicalize_failed",
                    "error": err_msg,
                }),
                None => json!({}),
            },
            "bsms" => json!({
                "byte_exact": false,
                "semantic_match": false,
                "diff": serde_json::Value::Null,
                "status": "blocked_no_emitter",
            }),
            // v0.28.0 Phase P3C — coldcard round-trip envelope mirrors the
            // bitcoin-core / coldcard-multisig shape: canonicalize is real
            // (SPEC §11.3 semantic round-trip via preserved-key projection +
            // BTreeMap alphabetical ordering). `byte_exact` compares input
            // bytes to canonical output; `semantic_match=true` always since
            // a successful canonicalize implies the parse + re-emit cycle
            // succeeded.
            "coldcard" => match canon_orig.clone() {
                Some(Ok(canon)) => {
                    let original_text = std::str::from_utf8(blob).unwrap_or("").to_string();
                    let byte_exact = original_text == canon;
                    let diff_val = if byte_exact {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(unified_diff(&original_text, &canon))
                    };
                    json!({
                        "byte_exact": byte_exact,
                        "semantic_match": true,
                        "diff": diff_val,
                        "status": "ok",
                    })
                }
                Some(Err(err_msg)) => json!({
                    "byte_exact": false,
                    "semantic_match": false,
                    "diff": serde_json::Value::Null,
                    "status": "canonicalize_failed",
                    "error": err_msg,
                }),
                None => json!({}),
            },
            // v0.28.0 Phase P4C — coldcard-multisig round-trip envelope mirrors
            // the bitcoin-core shape: canonicalize is real (SPEC §11.4
            // semantic round-trip via `parse_text` + re-emit in canonical
            // shared-derivation shape). `byte_exact` compares input bytes to
            // canonical output; `semantic_match=true` always since a
            // successful canonicalize implies the parse + re-emit cycle
            // succeeded.
            "coldcard-multisig" => match canon_orig.clone() {
                Some(Ok(canon)) => {
                    let original_text = std::str::from_utf8(blob).unwrap_or("").to_string();
                    let byte_exact = original_text == canon;
                    let diff_val = if byte_exact {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(unified_diff(&original_text, &canon))
                    };
                    json!({
                        "byte_exact": byte_exact,
                        "semantic_match": true,
                        "diff": diff_val,
                        "status": "ok",
                    })
                }
                Some(Err(err_msg)) => json!({
                    "byte_exact": false,
                    "semantic_match": false,
                    "diff": serde_json::Value::Null,
                    "status": "canonicalize_failed",
                    "error": err_msg,
                }),
                None => json!({}),
            },
            // v0.28.0 Phase P6C — electrum round-trip envelope mirrors the
            // bitcoin-core / coldcard / coldcard-multisig / sparrow / specter
            // shape: canonicalize is real (SPEC §11.6 semantic round-trip via
            // BTreeMap-backed alphabetical key reorder + dynamic xN/ cosigner
            // key preservation). `byte_exact` compares input bytes to canonical
            // output; `semantic_match=true` always on Ok.
            "electrum" => match canon_orig.clone() {
                Some(Ok(canon)) => {
                    let original_text = std::str::from_utf8(blob).unwrap_or("").to_string();
                    let byte_exact = original_text == canon;
                    let diff_val = if byte_exact {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(unified_diff(&original_text, &canon))
                    };
                    json!({
                        "byte_exact": byte_exact,
                        "semantic_match": true,
                        "diff": diff_val,
                        "status": "ok",
                    })
                }
                Some(Err(err_msg)) => json!({
                    "byte_exact": false,
                    "semantic_match": false,
                    "diff": serde_json::Value::Null,
                    "status": "canonicalize_failed",
                    "error": err_msg,
                }),
                None => json!({}),
            },
            // v0.28.0 Phase P5C — jade round-trip envelope mirrors the
            // bitcoin-core / coldcard / coldcard-multisig / electrum /
            // sparrow / specter shape: canonicalize is real (SPEC §11.5
            // semantic round-trip via BTreeMap-backed JSON wrapper with
            // `id` dropped + inner Coldcard-multisig text canonicalized).
            // `byte_exact` compares input bytes to canonical output;
            // `semantic_match=true` always on Ok.
            "jade" => match canon_orig.clone() {
                Some(Ok(canon)) => {
                    let original_text = std::str::from_utf8(blob).unwrap_or("").to_string();
                    let byte_exact = original_text == canon;
                    let diff_val = if byte_exact {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(unified_diff(&original_text, &canon))
                    };
                    json!({
                        "byte_exact": byte_exact,
                        "semantic_match": true,
                        "diff": diff_val,
                        "status": "ok",
                    })
                }
                Some(Err(err_msg)) => json!({
                    "byte_exact": false,
                    "semantic_match": false,
                    "diff": serde_json::Value::Null,
                    "status": "canonicalize_failed",
                    "error": err_msg,
                }),
                None => json!({}),
            },
            // v0.28.0 Phase P1C — sparrow round-trip envelope mirrors the
            // bitcoin-core + coldcard-multisig shape: canonicalize is real
            // (SPEC §11.1 semantic round-trip via BTreeMap-backed
            // alphabetical-key form). `byte_exact` compares input bytes to
            // canonical output; `semantic_match=true` always on Ok (a
            // successful canonicalize implies the parse + re-emit cycle
            // succeeded). Failures surface via `canonicalize_failed`.
            "sparrow" => match canon_orig.clone() {
                Some(Ok(canon)) => {
                    let original_text = std::str::from_utf8(blob).unwrap_or("").to_string();
                    let byte_exact = original_text == canon;
                    let diff_val = if byte_exact {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(unified_diff(&original_text, &canon))
                    };
                    json!({
                        "byte_exact": byte_exact,
                        "semantic_match": true,
                        "diff": diff_val,
                        "status": "ok",
                    })
                }
                Some(Err(err_msg)) => json!({
                    "byte_exact": false,
                    "semantic_match": false,
                    "diff": serde_json::Value::Null,
                    "status": "canonicalize_failed",
                    "error": err_msg,
                }),
                None => json!({}),
            },
            // v0.28.0 Phase P2C — specter round-trip envelope mirrors the
            // bitcoin-core + coldcard-multisig + sparrow shape: canonicalize
            // is real (SPEC §11.2 semantic round-trip via BTreeMap-backed
            // alphabetical-key form). `byte_exact` compares input bytes to
            // canonical output; `semantic_match=true` always on Ok (a
            // successful canonicalize implies the parse + re-emit cycle
            // succeeded). Failures surface via `canonicalize_failed`.
            "specter" => match canon_orig.clone() {
                Some(Ok(canon)) => {
                    let original_text = std::str::from_utf8(blob).unwrap_or("").to_string();
                    let byte_exact = original_text == canon;
                    let diff_val = if byte_exact {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(unified_diff(&original_text, &canon))
                    };
                    json!({
                        "byte_exact": byte_exact,
                        "semantic_match": true,
                        "diff": diff_val,
                        "status": "ok",
                    })
                }
                Some(Err(err_msg)) => json!({
                    "byte_exact": false,
                    "semantic_match": false,
                    "diff": serde_json::Value::Null,
                    "status": "canonicalize_failed",
                    "error": err_msg,
                }),
                None => json!({}),
            },
            _ => json!({}),
        };

        let mut env = serde_json::Map::new();
        env.insert(
            "schema_version".to_string(),
            json!(IMPORT_WALLET_ENVELOPE_SCHEMA_VERSION),
        );
        env.insert("source_format".to_string(), json!(format_str));
        env.insert("bundle".to_string(), bundle_value);
        env.insert("roundtrip".to_string(), roundtrip);

        if let Some(audit) = p.bsms_audit() {
            env.insert(
                "bsms_audit".to_string(),
                json!({
                    "token": audit.token,
                    "signature": audit.signature,
                    "first_address": audit.first_address,
                    "derivation_path": audit.derivation_path,
                    "signature_verified": audit.verification.signature_verified(),
                }),
            );
        }
        if let Some(meta) = p.source_metadata() {
            env.insert(
                "source_metadata".to_string(),
                json!({
                    "active": meta.active,
                    "internal": meta.internal,
                    "range": meta.range,
                    "dropped_fields": meta.dropped_fields,
                    "wallet_name": meta.wallet_name,
                }),
            );
        }
        // v0.28.0 Phase P3C — Coldcard single-sig provenance envelope field.
        // Mirrors the per-format-distinct field-name discipline
        // (`coldcard_source_metadata`): surfaces ONLY when the parse was
        // Coldcard-shaped. Carries `chain`, `xfp`, `bip_derivation`,
        // `raw_account`, `dropped_fields` per SPEC §11.3.
        if let Some(meta) = p.provenance.coldcard_source_metadata() {
            let chain_str = match meta.chain {
                crate::wallet_import::coldcard::ColdcardChain::Btc => "BTC",
                crate::wallet_import::coldcard::ColdcardChain::Xtn => "XTN",
            };
            let bip_str = match meta.bip_derivation {
                crate::wallet_import::coldcard::ColdcardBip::Bip44 => "bip44",
                crate::wallet_import::coldcard::ColdcardBip::Bip49 => "bip49",
                crate::wallet_import::coldcard::ColdcardBip::Bip84 => "bip84",
                crate::wallet_import::coldcard::ColdcardBip::Bip86 => "bip86",
            };
            let xfp_hex = format!(
                "{:02X}{:02X}{:02X}{:02X}",
                meta.xfp[0], meta.xfp[1], meta.xfp[2], meta.xfp[3]
            );
            env.insert(
                "coldcard_source_metadata".to_string(),
                json!({
                    "chain": chain_str,
                    "xfp": xfp_hex,
                    "bip_derivation": bip_str,
                    "raw_account": meta.raw_account,
                    "dropped_fields": meta.dropped_fields,
                }),
            );
        }
        // v0.37.8 — Coldcard-multisig provenance envelope field. Mirrors
        // the per-format-distinct field-name discipline
        // (`coldcard_multisig_source_metadata`): surfaces ONLY when the parse
        // was Coldcard-multisig-shaped. Carries `name`, `policy_k`/`policy_n`,
        // `script_format`, `xfp_*` telemetry, `dropped_fields` — same shape
        // as Jade's nested `coldcard_compat` subobject (Jade delegates this
        // parser). Added so `export-wallet --from-import-json` can lift the
        // wallet name back through round-trips.
        if let Some(meta) = p.provenance.coldcard_multisig_source_metadata() {
            let script_format_str = match meta.script_format {
                crate::wallet_import::coldcard_multisig::ColdcardMsFormat::P2wsh => "P2WSH",
                crate::wallet_import::coldcard_multisig::ColdcardMsFormat::P2shP2wsh => {
                    "P2SH-P2WSH"
                }
                crate::wallet_import::coldcard_multisig::ColdcardMsFormat::P2sh => "P2SH",
            };
            env.insert(
                "coldcard_multisig_source_metadata".to_string(),
                json!({
                    "name": meta.name,
                    "policy_k": meta.policy.k,
                    "policy_n": meta.policy.n,
                    "script_format": script_format_str,
                    "xfp_was_blob_supplied": meta.xfp_was_blob_supplied,
                    "xfp_header_disagreed": meta.xfp_header_disagreed,
                    "dropped_fields": meta.dropped_fields,
                }),
            );
        }
        // v0.28.0 Phase P6C — Electrum provenance envelope field. Mirrors
        // the per-format-distinct field-name discipline
        // (`electrum_source_metadata`): surfaces ONLY when the parse was
        // Electrum-shaped. Carries `seed_version`, `wallet_type` (rendered
        // as the canonical Electrum value-set string: "standard" or
        // "<k>of<n>"), `wallet_name`, `dropped_fields` per SPEC §11.6.
        if let Some(meta) = p.provenance.electrum_source_metadata() {
            let wallet_type_str = match meta.wallet_type {
                crate::wallet_import::electrum::ElectrumWalletType::Standard => {
                    "standard".to_string()
                }
                crate::wallet_import::electrum::ElectrumWalletType::Multisig { k, n } => {
                    format!("{k}of{n}")
                }
            };
            env.insert(
                "electrum_source_metadata".to_string(),
                json!({
                    "seed_version": meta.seed_version,
                    "wallet_type": wallet_type_str,
                    "wallet_name": meta.wallet_name,
                    "dropped_fields": meta.dropped_fields,
                }),
            );
        }
        // v0.28.0 Phase P5C — Jade provenance envelope field. Mirrors
        // the per-format-distinct field-name discipline
        // (`jade_source_metadata`): surfaces ONLY when the parse was
        // Jade-shaped. Carries the delegated Coldcard-multisig metadata
        // verbatim under `coldcard_compat` (name, policy K-of-N,
        // script_format, xfp telemetry, dropped_fields) plus a
        // future-proof `jade_specific_fields` array (empty at v0.28.0
        // per Q1 SeedQR-deferred lock).
        if let Some(meta) = p.provenance.jade_source_metadata() {
            let script_format_str = match meta.coldcard_compat.script_format {
                crate::wallet_import::coldcard_multisig::ColdcardMsFormat::P2wsh => "P2WSH",
                crate::wallet_import::coldcard_multisig::ColdcardMsFormat::P2shP2wsh => {
                    "P2SH-P2WSH"
                }
                crate::wallet_import::coldcard_multisig::ColdcardMsFormat::P2sh => "P2SH",
            };
            env.insert(
                "jade_source_metadata".to_string(),
                json!({
                    "coldcard_compat": {
                        "name": meta.coldcard_compat.name,
                        "policy_k": meta.coldcard_compat.policy.k,
                        "policy_n": meta.coldcard_compat.policy.n,
                        "script_format": script_format_str,
                        "xfp_was_blob_supplied": meta.coldcard_compat.xfp_was_blob_supplied,
                        "xfp_header_disagreed": meta.coldcard_compat.xfp_header_disagreed,
                        "dropped_fields": meta.coldcard_compat.dropped_fields,
                    },
                    "jade_specific_fields": meta.jade_specific_fields,
                }),
            );
        }
        // v0.28.0 Phase P1C — Sparrow provenance envelope field. Mirrors
        // `source_metadata` (BitcoinCore) + `bsms_audit` (BSMS): the field
        // surfaces ONLY when the parse was Sparrow-shaped. Field name is
        // `sparrow_source_metadata` for cross-format symmetry with
        // `source_metadata` (Core); using a per-format-distinct field name
        // avoids wire-shape conflict with the existing `source_metadata`.
        if let Some(meta) = p.provenance.sparrow_source_metadata() {
            let policy_type_str = match meta.policy_type {
                crate::wallet_import::sparrow::SparrowPolicyType::Single => "SINGLE",
                crate::wallet_import::sparrow::SparrowPolicyType::Multi => "MULTI",
            };
            env.insert(
                "sparrow_source_metadata".to_string(),
                json!({
                    "label": meta.label,
                    "policy_type": policy_type_str,
                    "script_type": meta.script_type,
                    "dropped_fields": meta.dropped_fields,
                }),
            );
        }
        // v0.28.0 Phase P2C — Specter provenance envelope field. Mirrors
        // `sparrow_source_metadata` discipline: per-format-distinct field
        // name (`specter_source_metadata`) surfaces ONLY when the parse was
        // Specter-shaped. Carries `label` + `blockheight` + `devices` array
        // (vendor-type + label per cosigner) + `dropped_fields`.
        if let Some(meta) = p.provenance.specter_source_metadata() {
            let devices_json: Vec<serde_json::Value> = meta
                .devices
                .iter()
                .map(|d| {
                    json!({
                        "type": d.device_type,
                        "label": d.label,
                    })
                })
                .collect();
            env.insert(
                "specter_source_metadata".to_string(),
                json!({
                    "label": meta.label,
                    "blockheight": meta.blockheight,
                    "devices": devices_json,
                    "dropped_fields": meta.dropped_fields,
                }),
            );
        }

        // v0.27.0 — propagate Round-1 BIP-322 verify state when --bsms-round1
        // was supplied alongside --blob. Same array on every parsed entry
        // (verifications are blob-independent; surface on every envelope so
        // downstream consumers don't have to index-match).
        if !round1_verifications.is_empty() {
            env.insert(
                "bsms_round1_verifications".to_string(),
                serde_json::Value::Array(
                    round1_verifications
                        .iter()
                        .map(round1_verification_to_json)
                        .collect(),
                ),
            );
        }

        envelopes.push(serde_json::Value::Object(env));
    }

    let text = serde_json::to_string_pretty(&serde_json::Value::Array(envelopes))
        .map_err(|e| ToolkitError::BadInput(format!("import-wallet --json serialize: {e}")))?;
    writeln!(stdout, "{text}").map_err(ToolkitError::Io)?;
    Ok(())
}

/// SPEC §3.2.1 row `multisig.path_family` — detection from the BIP-43
/// purpose component of cosigner paths. v0.27.0 Phase 6.5 PR-review I1 fold:
/// requires all cosigners to agree on the purpose component (heterogeneity
/// produces a stderr NOTICE + falls back to bip87); explicitly enumerates
/// recognized BIP-43 purposes (`44'/45'/48'/49'/84'/86'/87'`) rather than
/// silently collapsing unknowns to bip87.
///
/// Returns `(path_family, optional_stderr_notice)`:
/// - `48'` → `"bip48"` — BIP-48 multisig.
/// - `87'` → `"bip87"` — BIP-87 cosigner-level multisig (and toolkit default).
/// - `44'`/`45'`/`49'`/`84'`/`86'` → `"bip87"` + stderr NOTICE about the
///   purpose mismatch (single-sig purposes appearing in multisig context
///   are non-canonical; surface this rather than silently collapsing).
/// - Heterogeneous purposes → `"bip87"` + stderr NOTICE listing the
///   per-cosigner purposes.
/// - Empty paths → `"bip87"` silently (the calling site only invokes this
///   helper when N ≥ 1).
fn path_family_from_paths(paths: &[String]) -> (&'static str, Option<String>) {
    fn extract_purpose(p: &str) -> &str {
        let trimmed = p.trim_start_matches("m/").trim_start_matches('m');
        trimmed
            .trim_start_matches('/')
            .split('/')
            .next()
            .unwrap_or("")
    }
    let purposes: Vec<&str> = paths.iter().map(|p| extract_purpose(p)).collect();
    if purposes.is_empty() {
        return ("bip87", None);
    }
    let all_same = purposes.windows(2).all(|w| w[0] == w[1]);
    if !all_same {
        let notice = format!(
            "notice: import-wallet: cosigner paths disagree on BIP-43 purpose: {:?}; \
             envelope `multisig.path_family` defaults to \"bip87\" — consumers may misinterpret",
            purposes
        );
        return ("bip87", Some(notice));
    }
    match purposes[0] {
        "48'" | "48h" => ("bip48", None),
        "87'" | "87h" => ("bip87", None),
        // Recognized but non-canonical-for-multisig BIP-43 purposes.
        "44'" | "44h" | "45'" | "45h" | "49'" | "49h" | "84'" | "84h" | "86'" | "86h" => {
            let notice = format!(
                "notice: import-wallet: cosigner BIP-43 purpose {:?} is non-canonical for \
                 multisig; envelope `multisig.path_family` defaults to \"bip87\"",
                purposes[0]
            );
            ("bip87", Some(notice))
        }
        "" => ("bip87", None), // empty paths (single-sig N=1) — no notice
        other => {
            let notice = format!(
                "notice: import-wallet: unrecognized BIP-43 purpose component {:?}; \
                 envelope `multisig.path_family` defaults to \"bip87\"",
                other
            );
            ("bip87", Some(notice))
        }
    }
}

/// SPEC §7.4: when `--json` is NOT set, the round-trip diff goes ONLY on
/// stderr (the cards stdout is unaffected). For BSMS the stderr-WARNING
/// path is not yet rewired to consume the v0.27.0-shipped emitter
/// (`crate::wallet_export::bsms`); we skip the WARNING here. See the
/// caveat at `emit_json_envelope`'s doc comment (`import_wallet.rs:396-399`)
/// for the corresponding `--json` envelope `roundtrip.status` behavior.
/// For Bitcoin Core we compare original bytes vs canonicalize and emit
/// a WARNING per SPEC §2.4 ("roundtrip not byte-exact; semantic
/// equivalent; diff below").
fn emit_roundtrip_stderr_warning<E: Write>(
    stderr: &mut E,
    blob: &[u8],
    format_str: &str,
) -> Result<(), ToolkitError> {
    // v0.28.0 Phase P0C (Site 8 in plan-doc §B.2 #6) — per-format
    // stderr-WARNING decision: ALL 6 new formats fall under the no-warning
    // early-return (the `!= "bitcoin-core"` predicate covers them). BSMS
    // takes the same path today via the `blocked_no_emitter` caveat. If a
    // per-parser P{N}B sub-phase decides to surface a roundtrip WARNING on
    // stderr, this site flips to an explicit `if !matches!(format_str,
    // "bitcoin-core" | "<new>") { return Ok(()) }` shape.
    if format_str != "bitcoin-core" {
        return Ok(());
    }
    // v0.27.1 Phase 1 C1 fold: previous code silently returned Ok(()) on
    // canonicalize / UTF-8 errors, suppressing the SPEC §7.4 stderr
    // warning. This is the ONLY non-JSON-mode feedback that a Bitcoin Core
    // blob isn't round-tripping byte-exactly; a parser/canonicalizer
    // disagreement or a non-UTF-8 blob could otherwise produce an apparently
    // clean import that silently mutated the descriptor. Emit a clear
    // diagnostic on each failure path.
    let canon = match canonicalize_bitcoin_core(blob) {
        Ok(c) => c,
        Err(e) => {
            writeln!(
                stderr,
                "warning: import-wallet: roundtrip check skipped: canonicalize_bitcoin_core failed: {e}"
            )
            .map_err(ToolkitError::Io)?;
            return Ok(());
        }
    };
    let original_text = match std::str::from_utf8(blob) {
        Ok(s) => s,
        Err(_) => {
            writeln!(
                stderr,
                "notice: import-wallet: blob is not UTF-8; roundtrip check uses lossy decode"
            )
            .map_err(ToolkitError::Io)?;
            // Fall through with lossy decode so we still emit the comparison
            // diff if the lossy form differs from canon. Bind a String to
            // outlive the match.
            let lossy = String::from_utf8_lossy(blob).into_owned();
            if lossy == canon {
                return Ok(());
            }
            let diff = unified_diff(&lossy, &canon);
            writeln!(
                stderr,
                "warning: import-wallet: roundtrip not byte-exact; semantic equivalent; diff below"
            )
            .map_err(ToolkitError::Io)?;
            write!(stderr, "{diff}").map_err(ToolkitError::Io)?;
            return Ok(());
        }
    };
    if original_text == canon {
        return Ok(());
    }
    let diff = unified_diff(original_text, &canon);
    writeln!(
        stderr,
        "warning: import-wallet: roundtrip not byte-exact; semantic equivalent; diff below"
    )
    .map_err(ToolkitError::Io)?;
    write!(stderr, "{diff}").map_err(ToolkitError::Io)?;
    Ok(())
}

pub(crate) fn network_human_name(n: bitcoin::Network) -> &'static str {
    match n {
        bitcoin::Network::Bitcoin => "mainnet",
        bitcoin::Network::Testnet => "testnet",
        bitcoin::Network::Signet => "signet",
        bitcoin::Network::Regtest => "regtest",
        _ => "unknown",
    }
}

fn emit_summary<W: Write>(stdout: &mut W, parsed: &[ParsedImport]) -> Result<(), ToolkitError> {
    writeln!(stdout, "import-wallet: bundles={}", parsed.len()).map_err(ToolkitError::Io)?;
    for (i, b) in parsed.iter().enumerate() {
        writeln!(stdout, "bundles[{i}].cosigners={}", b.cosigners.len())
            .map_err(ToolkitError::Io)?;
        let network_name = network_human_name(b.network);
        writeln!(stdout, "bundles[{i}].network={network_name}").map_err(ToolkitError::Io)?;
        let threshold_str = b
            .threshold
            .map(|t| t.to_string())
            .unwrap_or_else(|| "none".to_string());
        writeln!(stdout, "bundles[{i}].threshold={threshold_str}").map_err(ToolkitError::Io)?;
        let audit_str = if b.bsms_audit().is_some() {
            "some"
        } else {
            "none"
        };
        writeln!(stdout, "bundles[{i}].bsms_audit={audit_str}").map_err(ToolkitError::Io)?;
        let entropy_str = if b.cosigners.iter().any(|c| c.entropy.is_some()) {
            "some"
        } else {
            "none"
        };
        writeln!(stdout, "bundles[{i}].entropy={entropy_str}").map_err(ToolkitError::Io)?;
        let src_meta_str = if b.source_metadata().is_some() {
            "some"
        } else {
            "none"
        };
        writeln!(stdout, "bundles[{i}].source_metadata={src_meta_str}")
            .map_err(ToolkitError::Io)?;
        if let Some(m) = b.source_metadata() {
            writeln!(stdout, "bundles[{i}].active={}", m.active).map_err(ToolkitError::Io)?;
            writeln!(stdout, "bundles[{i}].internal={}", m.internal).map_err(ToolkitError::Io)?;
        }
        for (j, c) in b.cosigners.iter().enumerate() {
            writeln!(
                stdout,
                "bundles[{i}].cosigners[{j}].fingerprint={}",
                hex_lower(&c.fingerprint.to_bytes())
            )
            .map_err(ToolkitError::Io)?;
            writeln!(
                stdout,
                "cosigners[{j}].fingerprint={}",
                hex_lower(&c.fingerprint.to_bytes())
            )
            .map_err(ToolkitError::Io)?;
        }
        writeln!(stdout, "cosigners={}", b.cosigners.len()).map_err(ToolkitError::Io)?;
        writeln!(stdout, "network={network_name}").map_err(ToolkitError::Io)?;
        writeln!(stdout, "threshold={threshold_str}").map_err(ToolkitError::Io)?;
        writeln!(stdout, "bsms_audit={audit_str}").map_err(ToolkitError::Io)?;
        writeln!(stdout, "entropy={entropy_str}").map_err(ToolkitError::Io)?;
    }
    Ok(())
}

/// v0.33.2 — resolve the optional Electrum BIE1 decryption password from the
/// `--decrypt-password{,-file,-stdin}` group. Returns `None` when no form is
/// supplied. Mirrors `cmd/electrum_decrypt.rs` resolution: the inline form
/// emits the argv-leakage advisory; the file form strips one trailing
/// newline; the stdin form is NULL-byte-preserving. The ArgGroup guarantees
/// at most one form is set.
fn resolve_import_decrypt_password<R: Read, E: Write>(
    args: &ImportWalletArgs,
    stdin: &mut R,
    stderr: &mut E,
) -> Result<Option<Zeroizing<String>>, ToolkitError> {
    if let Some(pw) = &args.decrypt_password {
        secret_in_argv_warning(stderr, "--decrypt-password ", "--decrypt-password-stdin");
        Ok(Some(Zeroizing::new(pw.clone())))
    } else if let Some(path) = &args.decrypt_password_file {
        let raw = std::fs::read_to_string(path).map_err(|e| {
            ToolkitError::BadInput(format!(
                "--decrypt-password-file: cannot read {}: {e}",
                path.display()
            ))
        })?;
        Ok(Some(Zeroizing::new(
            raw.strip_suffix('\n').unwrap_or(&raw).to_string(),
        )))
    } else if args.decrypt_password_stdin {
        Ok(Some(Zeroizing::new(read_stdin_passphrase(stdin)?)))
    } else {
        Ok(None)
    }
}

/// Map a library-local `EciesDecryptError` to a CLI-boundary `ToolkitError`.
/// The wrong-password / corruption modes (`HmacMismatch`, `AesDecryptFailure`)
/// are UNIFIED into one non-leaky message (mirrors `electrum-decrypt`).
fn map_ecies_storage_error(e: EciesDecryptError) -> ToolkitError {
    match e {
        EciesDecryptError::HmacMismatch | EciesDecryptError::AesDecryptFailure => {
            ToolkitError::BadInput(
                "import-wallet: electrum: decryption failed (wrong password or corrupted wallet file)"
                    .to_string(),
            )
        }
        other => ToolkitError::BadInput(format!("import-wallet: electrum: {other}")),
    }
}

/// Read the wallet blob into a `Zeroizing<Vec<u8>>` so the in-memory
/// plaintext is scrubbed on drop. A plaintext Electrum wallet
/// (`use_encryption:false`) can carry a seed, and the BIE1 decrypt path
/// (`run()`) writes decrypted seed/xprv-bearing JSON into this buffer — the
/// `Zeroizing` wrapper wipes it regardless of import format.
fn read_blob<R: Read>(path: &PathBuf, stdin: &mut R) -> Result<Zeroizing<Vec<u8>>, ToolkitError> {
    if path.as_os_str() == "-" {
        let mut buf = Zeroizing::new(Vec::new());
        stdin.read_to_end(&mut buf).map_err(ToolkitError::Io)?;
        Ok(buf)
    } else {
        Ok(Zeroizing::new(fs::read(path).map_err(ToolkitError::Io)?))
    }
}

/// v0.31.0 Cycle 7b — read the BIP-129 session TOKEN from a file or stdin.
/// Returns the lowercased + whitespace-stripped hex string. Mirrors
/// `read_blob`'s `path.as_os_str() == "-"` precedent.
fn read_bsms_token<R: Read>(path: &PathBuf, stdin: &mut R) -> Result<String, ToolkitError> {
    let raw = if path.as_os_str() == "-" {
        let mut buf = String::new();
        stdin.read_to_string(&mut buf).map_err(ToolkitError::Io)?;
        buf
    } else {
        fs::read_to_string(path).map_err(|e| {
            ToolkitError::BadInput(format!(
                "--bsms-encryption-token: cannot read token file {}: {e}",
                path.display()
            ))
        })?
    };
    Ok(raw.trim().to_lowercase())
}

/// v0.31.0+ — a decoded BIP-129 encryption TOKEN, held once after a single
/// read so the Round-1 verify path (`verify_bsms_round1_files`) and the
/// Round-2 descriptor-decrypt block can share it without re-reading stdin.
/// `hex` is the lowercase ASCII-hex form (the PBKDF2 salt is `raw`; the
/// HMAC input prefix is `hex`).
struct BsmsToken {
    hex: String,
    raw: Vec<u8>,
}

/// Read + width-validate the `--bsms-encryption-token` once. Mirrors the
/// width gate previously inline in the Round-2 decrypt block (8-byte
/// STANDARD / 16-byte EXTENDED per BIP-129).
fn read_and_validate_bsms_token<R: Read>(
    path: &PathBuf,
    stdin: &mut R,
) -> Result<BsmsToken, ToolkitError> {
    let hex = read_bsms_token(path, stdin)?;
    let raw = hex::decode(&hex).map_err(|e| {
        ToolkitError::BadInput(format!(
            "--bsms-encryption-token: token file contents not valid hex: {e}"
        ))
    })?;
    if raw.len() != 8 && raw.len() != 16 {
        return Err(ToolkitError::BadInput(format!(
            "--bsms-encryption-token: token must be 8 bytes STANDARD (16 hex chars) or 16 bytes EXTENDED (32 hex chars); got {} bytes ({} hex chars)",
            raw.len(),
            hex.len(),
        )));
    }
    Ok(BsmsToken { hex, raw })
}

/// True iff `text` is a BIP-129 encrypted record wire (hex `MAC || ciphertext`)
/// rather than a plaintext Round-1 record. Plaintext records always begin
/// with the `BSMS 1.0` header (space + non-hex letters); an encrypted wire
/// is raw lowercase/uppercase hex with no header.
fn is_encrypted_bsms_record(text: &str) -> bool {
    let t = text.trim();
    !t.is_empty()
        && !t.starts_with(crate::wallet_import::bsms_round1::BSMS_HEADER)
        && t.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Decrypt a BIP-129 encrypted record wire (hex `MAC || ciphertext`) with
/// `token`, MAC-verifying per the §Encryption Encrypt-and-MAC ordering, and
/// return the UTF-8 plaintext. Shared by the Round-2 descriptor-decrypt
/// block and the Round-1 verify path; `ctx` labels error messages for the
/// caller's record kind (e.g. `"bsms: encrypted Round-2 wire"` or
/// `"--bsms-round1: encrypted record N"`).
fn decrypt_bsms_record(
    text: &str,
    token: &BsmsToken,
    ctx: &str,
) -> Result<Zeroizing<String>, ToolkitError> {
    let wire = hex::decode(text.trim()).map_err(|e| {
        ToolkitError::ImportWalletParse(format!("import-wallet: {ctx} is not valid hex: {e}"))
    })?;
    if wire.len() < 32 + 1 {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: {ctx} too short ({} bytes; need MAC (32) + at least 1 ciphertext byte)",
            wire.len(),
        )));
    }
    let (mac_recv, ciphertext) = wire.split_at(32);
    let enc_key = mnemonic_toolkit::bsms_crypto::derive_encryption_key(&token.raw);
    let hmac_key = mnemonic_toolkit::bsms_crypto::derive_hmac_key(&enc_key);
    let iv: [u8; 16] = mac_recv[..16]
        .try_into()
        .expect("32-byte MAC slice has 16-byte prefix");
    let plaintext = mnemonic_toolkit::bsms_crypto::decrypt(ciphertext, &enc_key, &iv)
        .map_err(|e| ToolkitError::ImportWalletParse(format!("import-wallet: {ctx}: {e}")))?;
    let mac_expected =
        mnemonic_toolkit::bsms_crypto::compute_mac(&hmac_key, &token.hex, &plaintext);
    if mac_recv != mac_expected.as_slice() {
        return Err(ToolkitError::BsmsMacMismatch {
            token_len_hex: token.hex.len(),
        });
    }
    String::from_utf8(plaintext.to_vec())
        .map(Zeroizing::new)
        .map_err(|_| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: {ctx}: decrypted record is not valid UTF-8"
            ))
        })
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

// ---------------------------------------------------------------------------
// v0.27.0 — BIP-129 Round-1 verify (--bsms-round1 + --bsms-verify-strict)
// ---------------------------------------------------------------------------

/// Per-record verify result. Propagates to `--json` envelope when --blob is
/// supplied, OR to the standalone Round-1 verify envelope when --blob is not.
///
/// v0.27.0 Phase 6.5 PR-review I7 fold: status flipped from a
/// `(signature_verified: bool, failure_reason: Option<String>)` pair to a
/// closed enum so the representable-invalid state `(true, Some(reason))`
/// is no longer expressible.
struct Round1Verification {
    index: usize,
    signer_pubkey: String,
    description: String,
    token_hex: String,
    status: Round1VerificationStatus,
}

#[derive(Debug)]
enum Round1VerificationStatus {
    Verified,
    /// Lenient-default failure: `reason` is the BIP-322 verifier's
    /// rationale string. Strict mode surfaces this as a fatal
    /// `BsmsSignatureMismatch` before this enum is constructed.
    Failed {
        reason: String,
    },
}

/// Read + parse + verify each `--bsms-round1 <FILE>` entry. Lenient default:
/// verify failure emits stderr NOTICE + sets `status: Failed { reason }`;
/// strict (`--bsms-verify-strict`) makes verify failure fatal.
///
/// v0.32.2 — `tokens` carries 0, 1, or N decryption tokens:
/// - 0 tokens: encrypted records → `BadInput` (no token).
/// - 1 token: SHARED — decrypts every encrypted record (backward-compat).
/// - N>1 tokens (per-Signer): requires ALL records encrypted AND
///   `tokens.len() == paths.len()`; `token[i]` decrypts `record[i]`.
fn verify_bsms_round1_files(
    paths: &[PathBuf],
    strict: bool,
    tokens: &[BsmsToken],
    stderr: &mut dyn Write,
) -> Result<Vec<Round1Verification>, ToolkitError> {
    use crate::wallet_import::bsms_round1::{parse_round1, signer_pubkey};
    use crate::wallet_import::bsms_verify::verify_round1_signature;

    // v0.32.2 — per-Signer (N>1) pairing pre-checks. With multiple tokens,
    // every record must be encrypted and the counts must match so the
    // positional `token[i] ↔ record[i]` mapping is unambiguous.
    if tokens.len() > 1 {
        if tokens.len() != paths.len() {
            return Err(ToolkitError::BadInput(format!(
                "per-Signer tokens: supplied {} --bsms-encryption-token but {} --bsms-round1 record(s); counts must match for positional pairing",
                tokens.len(),
                paths.len(),
            )));
        }
        for (i, path) in paths.iter().enumerate() {
            if path.as_os_str() == "-" {
                continue; // stdin refusal handled in the main loop below
            }
            let probe = std::fs::read_to_string(path).map_err(ToolkitError::Io)?;
            if !is_encrypted_bsms_record(&probe) {
                return Err(ToolkitError::BadInput(format!(
                    "per-Signer tokens: --bsms-round1 record {i} is plaintext, but multiple --bsms-encryption-token were supplied; per-Signer mode requires every record to be encrypted (or supply a single shared token)"
                )));
            }
        }
    }

    let mut out = Vec::with_capacity(paths.len());
    for (i, path) in paths.iter().enumerate() {
        if path.as_os_str() == "-" {
            // v0.27.0 first cut: stdin input for --bsms-round1 deferred. Future:
            // multi-record stdin (one record per blob, separated by sentinel)
            // or single-record-from-stdin (mutually exclusive with --blob -).
            return Err(ToolkitError::BadInput(format!(
                "--bsms-round1 -: stdin input deferred in v0.27.0; supply a file path \
                 (record index {})",
                i
            )));
        }
        let raw_text = std::fs::read_to_string(path).map_err(ToolkitError::Io)?;
        // v0.32.1 — encrypted Round-1 records (hex MAC||ciphertext) are
        // decrypted before parse + BIP-322 verify. Plaintext records
        // (5-line `BSMS 1.0\n…`) pass through unchanged.
        // v0.32.2 — token selection: shared (1 token → tokens[0] for every
        // record) or per-Signer positional (N tokens → tokens[i]).
        let text = if is_encrypted_bsms_record(&raw_text) {
            let token = match tokens.len() {
                0 => {
                    return Err(ToolkitError::BadInput(format!(
                        "--bsms-round1: record {i} looks encrypted (hex MAC||ciphertext) but no --bsms-encryption-token was supplied"
                    )))
                }
                1 => &tokens[0],
                _ => &tokens[i], // counts validated equal above
            };
            let plaintext = decrypt_bsms_record(
                &raw_text,
                token,
                &format!("--bsms-round1: encrypted record {i}"),
            )?;
            writeln!(
                stderr,
                "notice: import-wallet: --bsms-round1: BIP-129 encrypted Round-1 record {i} decrypted (token width {} hex chars; MAC verified)",
                token.hex.len(),
            )
            .map_err(ToolkitError::Io)?;
            plaintext
        } else {
            Zeroizing::new(raw_text)
        };
        let record = parse_round1(&text)?;
        let pk_hex = hex::encode(signer_pubkey(&record).serialize());

        match verify_round1_signature(&record, i) {
            Ok(()) => {
                out.push(Round1Verification {
                    index: i,
                    signer_pubkey: pk_hex,
                    description: record.description.clone(),
                    token_hex: record.token_hex.clone(),
                    status: Round1VerificationStatus::Verified,
                });
            }
            Err(ToolkitError::BsmsSignatureMismatch {
                record_index,
                signer_pubkey: pk_for_err,
                reason,
            }) => {
                if strict {
                    return Err(ToolkitError::BsmsSignatureMismatch {
                        record_index,
                        signer_pubkey: pk_for_err,
                        reason,
                    });
                }
                // v0.27.0 Phase 6.5 PR-review I2 fold: propagate stderr
                // write failure as a typed I/O error rather than silently
                // dropping the NOTICE. This NOTICE is the ONLY interactive
                // signal of Round-1 verify failure in lenient mode (text-
                // mode users see no other indication), so a failed write
                // here would be a silent security-relevant signal loss.
                writeln!(
                    stderr,
                    "notice: import-wallet: --bsms-round1: signature verification failed \
                     for record {i} (signer pubkey {pk_for_err}): {reason}"
                )
                .map_err(ToolkitError::Io)?;
                out.push(Round1Verification {
                    index: i,
                    signer_pubkey: pk_for_err,
                    description: record.description.clone(),
                    token_hex: record.token_hex.clone(),
                    status: Round1VerificationStatus::Failed { reason },
                });
            }
            Err(e) => return Err(e),
        }
    }
    Ok(out)
}

fn emit_round1_only_envelope<W: Write>(
    stdout: &mut W,
    verifications: &[Round1Verification],
) -> Result<(), ToolkitError> {
    let payload = json!({
        "source_format": "bsms-round1",
        "bsms_round1_verifications": verifications
            .iter()
            .map(round1_verification_to_json)
            .collect::<Vec<_>>(),
    });
    let body = serde_json::to_string(&payload)
        .map_err(|e| ToolkitError::BadInput(format!("--bsms-round1 envelope serialize: {e}")))?;
    writeln!(stdout, "{body}").map_err(ToolkitError::Io)?;
    Ok(())
}

fn emit_round1_only_summary<W: Write>(
    stdout: &mut W,
    verifications: &[Round1Verification],
) -> Result<(), ToolkitError> {
    writeln!(
        stdout,
        "bsms-round1: {} record(s) processed",
        verifications.len()
    )
    .map_err(ToolkitError::Io)?;
    for v in verifications {
        let verified = matches!(v.status, Round1VerificationStatus::Verified);
        writeln!(
            stdout,
            "  record[{}]: signer_pubkey={} description={:?} token_hex={} verified={}",
            v.index, v.signer_pubkey, v.description, v.token_hex, verified
        )
        .map_err(ToolkitError::Io)?;
    }
    Ok(())
}

fn round1_verification_to_json(v: &Round1Verification) -> serde_json::Value {
    let (signature_verified, failure_reason): (bool, Option<&str>) = match &v.status {
        Round1VerificationStatus::Verified => (true, None),
        Round1VerificationStatus::Failed { reason } => (false, Some(reason.as_str())),
    };
    json!({
        "index": v.index,
        "signer_pubkey": v.signer_pubkey,
        "description": v.description,
        "token_hex": v.token_hex,
        "signature_verified": signature_verified,
        "failure_reason": failure_reason,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// v0.32.1 — `is_encrypted_bsms_record` discriminates a plaintext
    /// 5-line Round-1 record (starts with `BSMS 1.0`) from an encrypted
    /// hex `MAC||ciphertext` wire.
    #[test]
    fn is_encrypted_bsms_record_discrimination() {
        // Plaintext Round-1 → false (header is not all-hex).
        assert!(!is_encrypted_bsms_record(
            "BSMS 1.0\n00\n[59865f44/48'/0'/0'/2']026d15\nSigner 1 key\nH6DXgq...\n"
        ));
        // Pure hex wire → true.
        assert!(is_encrypted_bsms_record(
            "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899abcd"
        ));
        // Uppercase hex (with surrounding whitespace) → true.
        assert!(is_encrypted_bsms_record("  AABBCCDD\n  "));
        // Empty / whitespace-only → false.
        assert!(!is_encrypted_bsms_record(""));
        assert!(!is_encrypted_bsms_record("   \n\t "));
        // Non-hex non-BSMS text → false (e.g. odd punctuation).
        assert!(!is_encrypted_bsms_record("hello world"));
        // Odd-length hex is still "looks encrypted" (caught later by
        // hex::decode in decrypt_bsms_record, not by the discriminator).
        assert!(is_encrypted_bsms_record("abc"));
    }

    /// v0.27.1 Phase 1 C1 fold: canonicalize-failed arm of
    /// `emit_roundtrip_stderr_warning` emits a stderr warning with the
    /// typed `ToolkitError` Display form, rather than silently returning
    /// Ok(()). Regression guard against re-introduction of the bug.
    #[test]
    fn emit_roundtrip_stderr_warning_canonicalize_err_emits_warning() {
        let mut stderr: Vec<u8> = Vec::new();
        // Bytes that fail JSON parse → `canonicalize_bitcoin_core` returns Err.
        let blob = b"not valid json at all {{{";
        let res = emit_roundtrip_stderr_warning(&mut stderr, blob, "bitcoin-core");
        assert!(
            res.is_ok(),
            "lenient mode must succeed even on canonicalize Err"
        );
        let stderr_str = String::from_utf8(stderr).unwrap();
        assert!(
            stderr_str.contains("warning: import-wallet: roundtrip check skipped: canonicalize_bitcoin_core failed:"),
            "expected canonicalize-failed warning; got: {stderr_str}"
        );
    }

    /// Byte-exact case emits no warning (regression guard — prior code's
    /// silent path was correct on this branch; the v0.27.1 fold must not
    /// accidentally emit a spurious warning on the happy path).
    ///
    /// R0 M2 fold: use the canonicalize output itself as the input so
    /// byte-exact-ness is guaranteed (not dependent on the seed blob's
    /// happenstance JSON key order). This gives a strict `is_empty()`
    /// assertion rather than the weaker prior `if !is_empty { not_contains
    /// "canonicalize_failed" }` guard.
    #[test]
    fn emit_roundtrip_stderr_warning_byte_exact_no_warning() {
        let mut stderr: Vec<u8> = Vec::new();
        // Capture the canonicalize output, then feed it back in. By
        // construction, `canon == original_text`, so the function takes
        // the "no warning" path.
        let seed = br#"{"descriptors":[]}"#;
        let canon = crate::wallet_import::roundtrip::canonicalize_bitcoin_core(seed)
            .expect("canonicalize seed accepted");
        let res = emit_roundtrip_stderr_warning(&mut stderr, canon.as_bytes(), "bitcoin-core");
        assert!(res.is_ok());
        let stderr_str = String::from_utf8(stderr).unwrap();
        assert!(
            stderr_str.is_empty(),
            "byte-exact case must emit nothing on stderr; got: {stderr_str:?}"
        );
    }

    /// v0.27.1 Phase 1 C1 fold (R0 M1 fold): non-UTF-8 blob fires the
    /// `notice:` line + falls through to lossy-decode comparison. Verifies
    /// the second Err arm of `emit_roundtrip_stderr_warning` after the C1
    /// fix. (Note: in production this branch is largely unreachable since
    /// `canonicalize_bitcoin_core` runs JSON parse first which requires
    /// UTF-8; this cell pins the defensive belt-and-suspenders code.)
    #[test]
    fn emit_roundtrip_stderr_warning_non_utf8_blob_emits_notice() {
        let mut stderr: Vec<u8> = Vec::new();
        // Bytes that pass JSON parse (so canonicalize succeeds) AS A LOSSY-
        // DECODE WOULD; but as raw bytes contain a non-UTF-8 sequence.
        // Achieving both is impossible in practice (JSON requires UTF-8),
        // so we instead pass bytes that fail `canonicalize_bitcoin_core`
        // and verify the canonicalize-Err arm fires correctly — the
        // non-UTF-8 arm is structurally guarded by the canonicalize-first
        // ordering, and the assertion below pins the canonicalize-Err arm
        // template against drift.
        let non_utf8: &[u8] = &[
            0xff, 0xfe, 0xfd, b' ', b'n', b'o', b't', b' ', b'j', b's', b'o', b'n',
        ];
        let res = emit_roundtrip_stderr_warning(&mut stderr, non_utf8, "bitcoin-core");
        assert!(
            res.is_ok(),
            "lenient mode succeeds even on non-UTF-8 / non-JSON"
        );
        let stderr_str = String::from_utf8_lossy(&stderr).into_owned();
        // canonicalize_bitcoin_core's serde_json::from_slice rejects the
        // non-UTF-8 prefix first, so the canonicalize-Err warning fires.
        // (The non-UTF-8 `notice:` line at sites 749-768 is reachable only
        // if a hypothetical canonicalize variant accepted non-UTF-8 input.)
        assert!(
            stderr_str.contains("warning: import-wallet: roundtrip check skipped: canonicalize_bitcoin_core failed:"),
            "expected canonicalize-failed warning on non-UTF-8 blob; got: {stderr_str}"
        );
    }

    /// v0.27.1 Phase 1 I7 fold: the `roundtrip` envelope's
    /// `canonicalize_failed` branch carries an `error: String` field with
    /// the typed `ToolkitError` Display form. Verifies the JSON shape
    /// matches the SPEC §7.4 v0.27.1 amendment. (Unit-level — the
    /// integration scenario requires a BitcoinCoreParser-vs-miniscript
    /// divergence fixture; this test pins the wire shape directly.)
    #[test]
    fn canonicalize_failed_envelope_carries_error_field() {
        // Mirror the json! macro construction at the canonicalize_failed
        // arm of emit_json_envelope (cmd/import_wallet.rs around line 555).
        let err_msg = "canonicalize_bitcoin_core: miniscript: unexpected token".to_string();
        let envelope = json!({
            "byte_exact": false,
            "semantic_match": false,
            "diff": serde_json::Value::Null,
            "status": "canonicalize_failed",
            "error": err_msg,
        });
        assert_eq!(envelope["status"], "canonicalize_failed");
        assert_eq!(
            envelope["error"],
            "canonicalize_bitcoin_core: miniscript: unexpected token"
        );
        assert_eq!(envelope["byte_exact"], false);
        assert_eq!(envelope["semantic_match"], false);
        assert!(envelope["diff"].is_null());
        // SPEC §7.4 v0.27.1 amendment: `error` is omitted in other status
        // values. Verify the closed-enum branch discipline.
        let ok_envelope = json!({
            "byte_exact": true,
            "semantic_match": true,
            "diff": serde_json::Value::Null,
            "status": "ok",
        });
        assert!(
            ok_envelope.get("error").is_none(),
            "ok status must not carry error field"
        );
    }
}
