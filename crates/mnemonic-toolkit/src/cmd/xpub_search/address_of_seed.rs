//! `mnemonic xpub-search address-of-seed` — given seed(s) and target
//! address(es), search the common paths (and, for multisig, the key-to-slot
//! map) and print each matched address's full path and the `bundle` command
//! that mints the wallet.
//!
//! SPEC: `design/SPEC_xpub_search_address_of_seed.md` (r3, approved by Brian
//! 2026-11-26 with §10 recommendations 1-9). Engine: `seed_search.rs`.
//!
//! Secrets: seeds and the passphrase come in through the sibling modes'
//! intake (`--phrase`/`--ms1`, their stdin forms and `@env:`) or a repeatable
//! `--from <node>=<value>`; they are zeroized, pinned, and dropped after the
//! account xpubs are derived. Worker threads see public keys only. The
//! printed `bundle` argv is built from references (`-`, `@env:VAR`), never
//! from a secret, and passes `redact_and_quote` as a fail-closed second check.

use super::candidate_paths::{build_candidate_paths, check_account_window, check_add_paths};
use super::seed_intake::{resolve_seed, SeedIntakeArgs};
use super::seed_search::{
    derive_child, leaf_slot, leaf_use_site, multi_family, probe_eval, scan, slot_orders,
    ChildCache, Expr, PathTuples, SourcePaths, Space, SpkKind, Template,
};
use super::{XpubSearchEnvelope, XpubSearchJson};
use crate::cmd::convert::{parse_from_input, NodeType};
use crate::derive_slot::derive_master_seed;
use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::permutation_search::{cap_decision, search_threads, CapDecision};
use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, PublicKey, ScriptBuf};
use clap::{Args, ValueEnum};
use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Duration;
use zeroize::Zeroizing;

/// md1's largest recordable path step (LP4-ext varint; SPEC §1).
const MD1_STEP_MAX: u32 = (1 << 29) - 1;
/// `restore`'s hard ceiling on realized candidates (SPEC §4).
const REALIZED_S_MAX: u128 = 1_000_000_000_000_000;
/// The child-key cache is refused above this many keys (about 4.5 GB).
const CACHE_MAX_KEYS: u128 = 1 << 26;
/// Bytes per cached child key (a `PublicKey`, rounded up).
const KEY_BYTES: u128 = 72;
/// Bytes per path entry of one key source (its `PathInfo`, the labels'
/// heap, the account xpub and the transient candidate), rounded up.
const PATH_BYTES: u128 = 320;
/// A run whose estimated memory (path lists plus the child-key cache)
/// passes this is refused before anything is built (final review M4).
const MEMORY_MAX_BYTES: u128 = 6 << 30;
/// Hits kept for the post-scan analysis; more is itself exit 4 (`cap`).
const MATCH_CAP: usize = 10_000;

#[derive(Args, Debug)]
pub struct AddressOfSeedArgs {
    /// Target address. Repeatable; at least one. Classified by its decoded
    /// scriptPubKey; it must belong to `--network`.
    #[arg(long, value_name = "ADDR", required = true)]
    pub target_address: Vec<String>,

    /// One seed: master BIP-39 phrase (inline, or `@env:VAR`). Inline is
    /// refused by the argv guard unless `--allow-argv-secret`.
    #[arg(long, value_name = "PHRASE", conflicts_with_all = ["phrase_stdin", "ms1", "ms1_stdin", "from"])]
    pub phrase: Option<String>,

    /// One seed: read the BIP-39 phrase from stdin.
    #[arg(long, conflicts_with_all = ["phrase", "ms1", "ms1_stdin", "from"])]
    pub phrase_stdin: bool,

    /// One seed: ms1 card carrying BIP-39 entropy (inline).
    #[arg(long, value_name = "MS1", conflicts_with_all = ["phrase", "phrase_stdin", "ms1_stdin", "from"])]
    pub ms1: Option<String>,

    /// One seed: read the ms1 card from stdin.
    #[arg(long, conflicts_with_all = ["phrase", "phrase_stdin", "ms1", "from"])]
    pub ms1_stdin: bool,

    /// Two or more seeds: `<phrase|ms1|entropy|seedqr>=<value|-|@env:VAR>`.
    /// Repeatable; seeds are numbered from 0 in the order given.
    #[arg(long, value_name = "NODE=VALUE")]
    pub from: Vec<String>,

    /// BIP-39 passphrase, applied to every seed. `-` reads stdin; `@env:VAR`
    /// reads an environment variable.
    #[arg(long, conflicts_with = "passphrase_stdin")]
    pub passphrase: Option<String>,

    // The `--passphrase-stdin` route (lint_argv_secret_flags anchor).
    /// Read the BIP-39 passphrase from stdin.
    #[arg(long, conflicts_with = "passphrase")]
    pub passphrase_stdin: bool,

    /// BIP-39 wordlist language. Defaults to english.
    #[arg(long, default_value = "english")]
    pub language: CliLanguage,

    /// Descriptor: a `@N` template, a keyed descriptor, or a BIP-388 policy
    /// JSON. Without one the search is single-sig (pkh, sh(wpkh), wpkh, tr).
    #[arg(long, conflicts_with_all = ["descriptor_file", "descriptor_from"])]
    pub descriptor: Option<String>,

    /// Read the descriptor from a file.
    #[arg(long, value_name = "FILE", conflicts_with_all = ["descriptor", "descriptor_from"])]
    pub descriptor_file: Option<PathBuf>,

    /// A keyless md1 template: `md1=<chunk>` (repeat per chunk) or `md1=-`
    /// (chunks on stdin). Its recorded origin paths pin the search.
    #[arg(long, value_name = "md1=CHUNK", conflicts_with_all = ["descriptor", "descriptor_file"])]
    pub descriptor_from: Vec<String>,

    /// A key that is not a seed: `@N=<xpub|[fp/path]xpub|mk1>` (slot stated)
    /// or `<xpub|[fp/path]xpub|mk1>` (slot searched). Repeatable.
    #[arg(long, value_name = "[@N=]XPUB-OR-MK1")]
    pub cosigner: Vec<String>,

    /// Lower bound of the account window (inclusive). Default 0.
    #[arg(long, default_value_t = 0)]
    pub min_account: u32,

    /// Accounts in the window starting at `--min-account` (default 20).
    #[arg(long, default_value_t = 20)]
    pub number_of_accounts: u32,

    /// Optional upper bound; the window ends at
    /// `max(min_account + number_of_accounts, max_account + 1)`.
    #[arg(long)]
    pub max_account: Option<u32>,

    /// Additional path template (`account'` substituted per account).
    /// Repeatable. Hardened steps take `'` or `h`; a template that is not a
    /// path is refused (exit 2). One that repeats a searched path is dropped.
    #[arg(long, value_name = "TEMPLATE")]
    pub add_path: Vec<String>,

    /// The index window `[0, N)` searched on each chain (not BIP-44's
    /// "N consecutive unused addresses" rule). At most 2^31; a run whose
    /// child-key cache (paths × chains × N per key) would pass 2^26 keys is
    /// refused (about 208,000 for one seed at the default paths), as is a
    /// run whose estimated memory (path lists plus that cache) passes
    /// 6144 MB.
    #[arg(long, default_value_t = 20)]
    pub gap_limit: u32,

    /// Search use-site alternative 0 only (the receive chain under `<0;1>`).
    #[arg(long)]
    pub external_only: bool,

    /// Multisig: let each seed sit at its own path (C^k path tuples) instead
    /// of every seed at the same path.
    #[arg(long)]
    pub independent_paths: bool,

    /// Add the Refugium seat paths m/270028'/coin'/n': all 256 for one key,
    /// the 8 rounds of its seat per slot for multisig.
    #[arg(long)]
    pub refugium_seats: bool,

    /// Network. Defaults to mainnet.
    #[arg(long, default_value = "mainnet")]
    pub network: CliNetwork,

    /// Emit a JSON envelope on stdout instead of the text report.
    #[arg(long)]
    pub json: bool,

    /// Positional ms1 card (one seed).
    #[arg(
        value_name = "MS1",
        num_args = 0..,
        conflicts_with_all = ["phrase", "phrase_stdin", "ms1", "ms1_stdin", "from"],
        required_unless_present_any = ["phrase", "phrase_stdin", "ms1", "ms1_stdin", "from"],
    )]
    pub positional: Vec<String>,
}

impl SeedIntakeArgs for AddressOfSeedArgs {
    fn phrase(&self) -> Option<&str> {
        self.phrase.as_deref()
    }
    fn phrase_stdin(&self) -> bool {
        self.phrase_stdin
    }
    fn ms1(&self) -> Option<&str> {
        self.ms1.as_deref()
    }
    fn ms1_stdin(&self) -> bool {
        self.ms1_stdin
    }
    fn positional(&self) -> &[String] {
        &self.positional
    }
    fn language(&self) -> CliLanguage {
        self.language
    }
}

// ---------------------------------------------------------------------------
// JSON (SPEC §6, normative)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct AddressOfSeedResult {
    pub result: &'static str,
    pub reasons: Vec<&'static str>,
    pub wallets: Vec<WalletJson>,
    pub unmatched_targets: Vec<UnmatchedJson>,
    pub seeds_without_match: Vec<usize>,
    pub searched: SearchedJson,
}

#[derive(Debug, Serialize, Clone)]
pub struct WalletJson {
    pub role: &'static str,
    pub twin_of: Option<usize>,
    pub from_user_template: bool,
    pub descriptor: String,
    pub script_type: &'static str,
    pub merged_forms: usize,
    pub slots: Vec<SlotJson>,
    pub targets: Vec<TargetJson>,
    pub distinguishing: Vec<DistinguishingJson>,
    pub bundle_command: Option<String>,
    pub bundle_command_redacted: bool,
    pub bundle_command_unavailable: Option<&'static str>,
    pub redaction_note: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SlotJson {
    pub slot: usize,
    pub source: &'static str,
    pub seed_index: Option<usize>,
    pub master_fingerprint: Option<String>,
    pub stated_fingerprint: Option<String>,
    pub origin_path: Option<String>,
    pub origin_known: bool,
    pub template: String,
    pub account: Option<u32>,
    pub refugium_seat: Option<u32>,
    pub refugium_round: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TargetJson {
    pub address: String,
    pub alternative: u32,
    pub step: Option<u32>,
    pub index: u32,
    pub paths: Vec<SlotPathJson>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SlotPathJson {
    pub slot: usize,
    /// The full path, or null for a cosigner whose origin is not known
    /// (review r1 M5: a path from an empty origin would read as a path from
    /// the master).
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DistinguishingJson {
    pub alternative: u32,
    pub index: u32,
    pub addresses: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct UnmatchedJson {
    pub address: String,
    pub reason: &'static str,
}

#[derive(Debug, Serialize)]
pub struct SearchedJson {
    pub mode: &'static str,
    pub paths: usize,
    pub child_keys: usize,
    pub candidate_scripts: Option<u64>,
    pub min_account: u32,
    pub end_account: u32,
    pub add_paths: Vec<String>,
    pub refugium_seats: bool,
    pub gap_limit: u32,
    pub alternatives: Vec<u32>,
    pub script_types: Vec<&'static str>,
    pub slot_orders: Option<u64>,
    pub independent_paths: bool,
    pub passphrase_supplied: bool,
    pub matches_total: u64,
    pub matches_kept: usize,
}

// ---------------------------------------------------------------------------
// Intake types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum SecretRef {
    Stdin,
    Env(String),
    Inline,
}

struct SeedIn {
    mnemonic: Mnemonic,
    subkey: &'static str,
    reference: SecretRef,
}

/// A path entry of one key source.
#[derive(Debug, Clone)]
struct PathInfo {
    path: DerivationPath,
    /// JSON `template` value.
    template: String,
    /// Text label, e.g. `bip84` or `--add-path m/0'`.
    label: String,
    account: Option<u32>,
    seat: Option<(u32, u32)>,
}

#[derive(Debug, Clone)]
struct CosignerIn {
    xpub: Xpub,
    fp: Option<Fingerprint>,
    path: Option<DerivationPath>,
}

/// One key source: a seed or a cosigner key.
struct Source {
    seed: Option<usize>,
    cosigner: Option<CosignerIn>,
    fingerprint: Option<Fingerprint>,
    paths: Vec<PathInfo>,
    accounts: Vec<Xpub>,
    sp: SourcePaths,
    /// Keyed mode: the fingerprint the descriptor states for this key.
    stated_fp: Option<Fingerprint>,
}

fn refuse(m: impl Into<String>) -> ToolkitError {
    ToolkitError::XpubSearchRefusal(format!("address-of-seed: {}", m.into()))
}

fn path_str(p: &DerivationPath) -> String {
    if p.is_empty() {
        "m".into()
    } else {
        format!("m/{p}")
    }
}

fn path_h(p: &DerivationPath) -> String {
    p.to_string().replace('\'', "h")
}

fn child_path(p: &DerivationPath, step: Option<u32>, idx: u32) -> String {
    let mut s = path_str(p);
    if let Some(st) = step {
        s.push_str(&format!("/{st}"));
    }
    s.push_str(&format!("/{idx}"));
    s
}

fn step_label(step: Option<u32>, alts: &[Option<u32>]) -> String {
    match step {
        None => "no chain step".into(),
        Some(s) if alts == [Some(0), Some(1)] && s == 0 => "step 0, external".into(),
        Some(s) if alts == [Some(0), Some(1)] && s == 1 => "step 1, change".into(),
        Some(s) => format!("step {s}"),
    }
}

fn network_name(n: CliNetwork) -> String {
    n.to_possible_value()
        .map(|v| v.get_name().to_string())
        .unwrap_or_else(|| "mainnet".into())
}

fn private_key_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"[a-zA-Z]prv[1-9A-HJ-NP-Za-km-z]{20,}|(^|[^1-9A-HJ-NP-Za-km-z])[5KLc9][1-9A-HJ-NP-Za-km-z]{50,51}([^1-9A-HJ-NP-Za-km-z]|$)")
            .expect("static regex")
    })
}

fn refuse_private(flag: &str, value: &str) -> Result<(), ToolkitError> {
    if private_key_re().is_match(value) {
        return Err(refuse(format!(
            "{flag} holds private-key material (an xprv/tprv-style key or WIF); only public keys \
             are accepted here (the value is not repeated)"
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// run
// ---------------------------------------------------------------------------

pub fn run_address_of_seed<R: Read, W: Write, E: Write>(
    args: &AddressOfSeedArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
) -> Result<u8, ToolkitError> {
    check_account_window(args.min_account, args.number_of_accounts, args.max_account)?;
    check_add_paths(
        "address-of-seed",
        args.min_account,
        args.number_of_accounts,
        args.max_account,
        &args.add_path,
    )?;
    if u64::from(args.gap_limit) > 1 << 31 || args.gap_limit == 0 {
        return Err(refuse(
            "--gap-limit must be between 1 and 2147483648 (2^31)",
        ));
    }
    // One stdin reader (SPEC §2).
    let from_stdin: Vec<bool> = args
        .from
        .iter()
        .map(|f| f.split_once('=').is_some_and(|(_, v)| v == "-"))
        .collect();
    let mut readers: Vec<(bool, String)> = vec![
        (
            args.phrase_stdin || args.ms1_stdin,
            "--phrase-stdin/--ms1-stdin".into(),
        ),
        (
            crate::passphrase_input::reads_stdin(args.passphrase.as_deref(), args.passphrase_stdin),
            crate::passphrase_input::stdin_spelling(args.passphrase_stdin).into(),
        ),
        (
            args.descriptor_from.iter().any(|d| d == "md1=-"),
            "--descriptor-from md1=-".into(),
        ),
    ];
    for (i, s) in from_stdin.iter().enumerate() {
        readers.push((*s, format!("--from #{i} =-")));
    }
    let rd: Vec<(bool, &str)> = readers.iter().map(|(b, s)| (*b, s.as_str())).collect();
    crate::passphrase_input::refuse_second_stdin(&rd)
        .map_err(|e| refuse(e.to_string().trim_start_matches("error: ").to_string()))?;

    let network = args.network;
    let btc_network = network.to_bitcoin_network();

    // Targets (SPEC §2 "Network").
    let mut targets: Vec<(String, ScriptBuf, SpkKind)> = Vec::new();
    for t in &args.target_address {
        let a = Address::from_str(t.trim())
            .map_err(|e| ToolkitError::BadInput(format!("--target-address {t}: {e}")))?;
        if !a.is_valid_for_network(btc_network) {
            return Err(ToolkitError::NetworkMismatch {
                decoded_network: "another network",
                expected_network: network.human_name(),
                context: "--target-address",
            });
        }
        let a = a.assume_checked();
        let spk = a.script_pubkey();
        let kind = SpkKind::of_script(&spk).ok_or_else(|| {
            ToolkitError::BadInput(format!(
                "--target-address {t}: not a searchable output type"
            ))
        })?;
        // A repeated target (also in another case) is one target (final
        // review M1: a second copy would never match and read `unmatched`).
        if let Some((first, _, _)) = targets.iter().find(|x| x.1 == spk) {
            writeln!(
                stderr,
                "note: --target-address {} repeats {first}; it is searched once",
                t.trim()
            )
            .map_err(ToolkitError::Io)?;
            continue;
        }
        targets.push((t.trim().to_string(), spk, kind));
    }
    let target_map: HashMap<ScriptBuf, usize> = targets
        .iter()
        .enumerate()
        .map(|(i, (_, s, _))| (s.clone(), i))
        .collect();

    // Private keys in --descriptor / --cosigner (SPEC §2, §7).
    if let Some(d) = &args.descriptor {
        refuse_private("--descriptor", d)?;
    }
    for c in &args.cosigner {
        // An mk1 card is public by construction, and its bech32 text can
        // contain an `xprv`-shaped run (review r1 N8); mk-codec decodes
        // public keys only.
        let value = c
            .split_once('=')
            .filter(|_| c.starts_with('@'))
            .map_or(c.as_str(), |x| x.1);
        if !value.trim().to_lowercase().starts_with("mk1") {
            refuse_private("--cosigner", c)?;
        }
    }

    // Descriptor intake (before reading any secret).
    let descriptor_text: Option<String> = if let Some(d) = &args.descriptor {
        Some(d.clone())
    } else if let Some(f) = &args.descriptor_file {
        let s = std::fs::read_to_string(f)
            .map_err(|e| ToolkitError::BadInput(format!("--descriptor-file: {e}")))?;
        refuse_private("--descriptor-file", &s)?;
        Some(s.trim().to_string())
    } else {
        None
    };
    let md1_intake: Option<Md1Template> = if args.descriptor_from.is_empty() {
        None
    } else {
        Some(md1_template(&args.descriptor_from, stdin)?)
    };
    if descriptor_text.is_none() && md1_intake.is_none() && !args.cosigner.is_empty() {
        return Err(refuse(
            "--cosigner needs --descriptor: a single-sig wallet has no cosigner",
        ));
    }

    // Seeds.
    let seeds = intake_seeds(args, stdin, stdout, stderr, no_auto_repair)?;
    for i in 0..seeds.len() {
        for j in (i + 1)..seeds.len() {
            if seeds[i].mnemonic.to_entropy() == seeds[j].mnemonic.to_entropy() {
                return Err(refuse(format!(
                    "seed {i} and seed {j} are the same seed; one seed in two slots is not \
                     searched in v1"
                )));
            }
        }
    }
    crate::passphrase_input::emit_argv_note(
        args.passphrase.as_deref(),
        args.passphrase_stdin,
        stderr,
    );
    let passphrase: Zeroizing<String> = crate::passphrase_input::resolve_or_empty(
        args.passphrase.as_deref(),
        args.passphrase_stdin,
        stdin,
    )?;
    let _pp_pin = mnemonic_toolkit::mlock::pin_pages_for(passphrase.as_bytes());
    let passphrase_ref: Option<SecretRef> =
        if args.passphrase_stdin || args.passphrase.as_deref() == Some("-") {
            Some(SecretRef::Stdin)
        } else if let Some(p) = args.passphrase.as_deref() {
            if let Some(v) = p.strip_prefix("@env:") {
                Some(SecretRef::Env(v.to_string()))
            } else {
                Some(SecretRef::Inline)
            }
        } else {
            None
        };

    // Master keys: derive, keep the fingerprints, drop the xprvs after the
    // account xpubs are derived (below).
    let secp = Secp256k1::new();
    let mut masters: Vec<Xpriv> = Vec::with_capacity(seeds.len());
    let mut fps: Vec<Fingerprint> = Vec::with_capacity(seeds.len());
    for s in &seeds {
        let seed = derive_master_seed(&s.mnemonic, passphrase.as_str());
        let _pin = mnemonic_toolkit::mlock::pin_pages_for(&seed[..]);
        let m = Xpriv::new_master(network.network_kind(), &seed[..])
            .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
        fps.push(m.fingerprint(&secp));
        masters.push(m);
    }

    let ctx = Ctx {
        args,
        network,
        targets: &targets,
        target_map: &target_map,
        seeds: &seeds,
        fps: &fps,
        passphrase_ref,
        passphrase_supplied: !passphrase.is_empty(),
    };

    let mut job = if let Some(text) = descriptor_text {
        let text = if text.trim_start().starts_with('{') {
            text
        } else {
            strip_checksum(&text)?
        };
        if text.trim_start().starts_with('{') {
            let expanded = crate::wallet_import::pipeline::expand_bip388_policy(&text)?;
            refuse_private("--descriptor", &expanded)?;
            keyed_job(&ctx, &expanded, &masters, stderr)?
        } else if Regex::new(r"@\d").expect("static").is_match(&text) {
            template_job(&ctx, &text, None)?
        } else {
            keyed_job(&ctx, &text, &masters, stderr)?
        }
    } else if let Some(md) = md1_intake {
        template_job(&ctx, &md.text, Some(&md))?
    } else {
        single_sig_job(&ctx)?
    };
    // The estimate is printed, and the memory ceiling checked, before any
    // account key or child key is derived (final review M4).
    print_plan(&ctx, &job, &masters, stderr)?;
    derive_accounts(&mut job.sources, &masters)?;
    drop(masters);

    execute(&ctx, job, stdout, stderr)
}

struct Ctx<'a> {
    args: &'a AddressOfSeedArgs,
    network: CliNetwork,
    targets: &'a [(String, ScriptBuf, SpkKind)],
    target_map: &'a HashMap<ScriptBuf, usize>,
    seeds: &'a [SeedIn],
    fps: &'a [Fingerprint],
    passphrase_ref: Option<SecretRef>,
    passphrase_supplied: bool,
}

// ---------------------------------------------------------------------------
// Seed intake
// ---------------------------------------------------------------------------

fn intake_seeds<R: Read, W: Write, E: Write>(
    args: &AddressOfSeedArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
) -> Result<Vec<SeedIn>, ToolkitError> {
    if args.from.is_empty() {
        let mnemonic = resolve_seed(args, stdin, stdout, stderr, no_auto_repair)?;
        let (subkey, reference) = if let Some(p) = &args.phrase {
            match p.strip_prefix("@env:") {
                Some(v) => ("phrase", SecretRef::Env(v.to_string())),
                None => ("phrase", SecretRef::Inline),
            }
        } else if args.phrase_stdin {
            ("phrase", SecretRef::Stdin)
        } else if args.ms1_stdin {
            ("ms1", SecretRef::Stdin)
        } else {
            ("ms1", SecretRef::Inline)
        };
        return Ok(vec![SeedIn {
            mnemonic,
            subkey,
            reference,
        }]);
    }
    let mut out = Vec::with_capacity(args.from.len());
    for raw in &args.from {
        let f = parse_from_input(raw).map_err(ToolkitError::BadInput)?;
        let subkey = match f.node {
            NodeType::Phrase => "phrase",
            NodeType::Ms1 => "ms1",
            NodeType::Entropy => "entropy",
            NodeType::Seedqr => "seedqr",
            other => {
                return Err(refuse(format!(
                    "--from {} is not a seed (use phrase, ms1, entropy or seedqr); a public key \
                     goes on --cosigner",
                    other.as_str()
                )))
            }
        };
        let (reference, value): (SecretRef, Zeroizing<String>) = if f.value == "-" {
            let mut buf = Zeroizing::new(String::new());
            stdin
                .read_to_string(&mut buf)
                .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
            (SecretRef::Stdin, Zeroizing::new(buf.trim().to_string()))
        } else if let Some(v) = f.value.strip_prefix("@env:") {
            (
                SecretRef::Env(v.to_string()),
                Zeroizing::new(crate::env_sentinel::resolve_env_var_sentinel(
                    &f.value, "--from",
                )?),
            )
        } else {
            crate::secret_advisory::secret_in_argv_warning(
                stderr,
                &format!("--from {subkey}="),
                &format!("--from {subkey}=-"),
            );
            (SecretRef::Inline, Zeroizing::new(f.value.clone()))
        };
        let _pin = mnemonic_toolkit::mlock::pin_pages_for(value.as_bytes());
        let lang: bip39::Language = args.language.into();
        let mnemonic = match f.node {
            NodeType::Phrase => {
                let norm = Zeroizing::new(value.split_whitespace().collect::<Vec<_>>().join(" "));
                Mnemonic::parse_in(lang, norm.as_str()).map_err(ToolkitError::Bip39)?
            }
            NodeType::Seedqr => {
                // The decode error can name a character and its position in
                // the secret; it is not repeated (final review N3).
                let phrase = Zeroizing::new(
                    mnemonic_toolkit::seedqr::decode(value.trim()).map_err(|_| {
                        ToolkitError::BadInput(
                            "--from seedqr=: not a valid SeedQR (the value is not repeated)".into(),
                        )
                    })?,
                );
                Mnemonic::parse_in(lang, phrase.as_str()).map_err(ToolkitError::Bip39)?
            }
            NodeType::Entropy => {
                // The decode error names a character and its position in
                // the secret; it is not repeated (review r1 N3).
                let e = Zeroizing::new(hex::decode(value.trim()).map_err(|_| {
                    ToolkitError::BadInput(
                        "--from entropy=: not an even-length hex string (the value is not \
                         repeated)"
                            .into(),
                    )
                })?);
                Mnemonic::from_entropy_in(lang, &e).map_err(ToolkitError::Bip39)?
            }
            _ => {
                let res = crate::slot_ms1::resolve_ms1_slot(value.trim(), Some(args.language), 0)?;
                Mnemonic::from_entropy_in(res.derive_language, &res.entropy)
                    .map_err(ToolkitError::Bip39)?
            }
        };
        out.push(SeedIn {
            mnemonic,
            subkey,
            reference,
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// md1 intake (keyless template)
// ---------------------------------------------------------------------------

struct Md1Template {
    text: String,
    /// Per slot, the recorded origin path (non-empty) and fingerprint.
    origins: Vec<(Option<DerivationPath>, Option<Fingerprint>)>,
}

fn md1_template<R: Read>(values: &[String], stdin: &mut R) -> Result<Md1Template, ToolkitError> {
    let mut chunks: Vec<String> = Vec::new();
    for v in values {
        let Some(c) = v.strip_prefix("md1=") else {
            return Err(refuse("--descriptor-from takes md1=<chunk> or md1=-"));
        };
        if c == "-" {
            let mut s = String::new();
            stdin
                .read_to_string(&mut s)
                .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
            chunks.extend(s.split_whitespace().map(str::to_string));
        } else {
            chunks.extend(c.split_whitespace().map(str::to_string));
        }
    }
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    let d = md_codec::chunk::reassemble(&refs).map_err(ToolkitError::from)?;
    if d.tlv.pubkeys.as_ref().is_some_and(|p| !p.is_empty()) {
        return Err(refuse(
            "a keyed md1 is not taken on --descriptor-from in v1; pass its descriptor as text \
             on --descriptor (FOLLOWUP address-of-seed-keyed-md1-intake)",
        ));
    }
    let text = md_codec::descriptor_to_template(&d).map_err(ToolkitError::from)?;
    let n = d.n as usize;
    let mut origins = vec![(None, None); n];
    for (i, o) in origins.iter_mut().enumerate() {
        let op = match &d.path_decl.paths {
            md_codec::origin_path::PathDeclPaths::Shared(p) => p.clone(),
            md_codec::origin_path::PathDeclPaths::Divergent(v) => v[i].clone(),
        };
        if !op.components.is_empty() {
            o.0 = Some(crate::cmd::bundle::origin_to_derivation_path(&op)?);
        }
        if let Some(fps) = &d.tlv.fingerprints {
            if let Some((_, fp)) = fps.iter().find(|(j, _)| *j as usize == i) {
                o.1 = Some(Fingerprint::from(*fp));
            }
        }
    }
    Ok(Md1Template { text, origins })
}

// ---------------------------------------------------------------------------
// Cosigner intake
// ---------------------------------------------------------------------------

fn parse_cosigner_key(v: &str) -> Result<CosignerIn, ToolkitError> {
    let v = v.trim();
    if v.to_lowercase().starts_with("mk1") {
        let toks: Vec<&str> = v.split_whitespace().collect();
        let card = mk_codec::decode(&toks).map_err(ToolkitError::from)?;
        return Ok(CosignerIn {
            xpub: card.xpub,
            fp: card.origin_fingerprint,
            path: Some(card.origin_path),
        });
    }
    let (fp, path, key) = if let Some(rest) = v.strip_prefix('[') {
        let (orig, key) = rest
            .split_once(']')
            .ok_or_else(|| refuse("--cosigner: unterminated `[fp/path]` origin"))?;
        let (fp, p) = match orig.split_once('/') {
            Some((f, p)) => (f, Some(p)),
            None => (orig, None),
        };
        let fp = Fingerprint::from_str(fp)
            .map_err(|_| refuse("--cosigner: the origin fingerprint is not 8 hex digits"))?;
        let path = match p {
            Some(p) => Some(
                DerivationPath::from_str(&format!("m/{p}"))
                    .map_err(|e| refuse(format!("--cosigner: origin path: {e}")))?,
            ),
            None => None,
        };
        (Some(fp), path, key)
    } else {
        (None, None, v)
    };
    let (canon, _) = crate::slip0132::normalize_xpub_prefix(key)?;
    let xpub = Xpub::from_str(&canon).map_err(ToolkitError::from)?;
    Ok(CosignerIn { xpub, fp, path })
}

/// `(slot, key)` per cosigner. Placed mk1 chunks for one slot are joined;
/// unplaced mk1 chunks, one or more per `--cosigner` value, are grouped by
/// the chunk-set id in their string-layer headers and each group is
/// reassembled (SPEC §2; review r1 I6). A group that does not reassemble is
/// refused (exit 2).
fn intake_cosigners(values: &[String]) -> Result<Vec<(Option<usize>, CosignerIn)>, ToolkitError> {
    use mk_codec::string_layer::{decode_string, StringLayerHeader};
    let mut placed: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    let mut out = Vec::new();
    // Unplaced mk1 groups, in first-seen order: (chunk-set id or a
    // single-string marker, chunks).
    let mut groups: Vec<(Option<u32>, Vec<String>)> = Vec::new();
    for v in values {
        if let Some(rest) = v.strip_prefix('@') {
            let (n, val) = rest
                .split_once('=')
                .ok_or_else(|| refuse("--cosigner @N=<key>: missing `=`"))?;
            let n: usize = n
                .parse()
                .map_err(|_| refuse(format!("--cosigner @{n}=: not a slot number")))?;
            // Placed mk1 chunks of one slot are joined; any other second
            // value for a slot is a conflict (final review M6).
            let vals = placed.entry(n).or_default();
            let is_mk1 = |x: &str| x.trim().to_lowercase().starts_with("mk1");
            let joinable = vals.is_empty() || (is_mk1(val) && vals.iter().all(|x| is_mk1(x)));
            if !joinable {
                return Err(refuse(format!("@{n} is given two keys")));
            }
            vals.push(val.to_string());
        } else if v.trim().to_lowercase().starts_with("mk1") {
            for tok in v.split_whitespace() {
                let header = decode_string(tok)
                    .map_err(ToolkitError::from)
                    .and_then(|d| {
                        StringLayerHeader::from_5bit_symbols(d.data())
                            .map(|(h, _)| h)
                            .map_err(ToolkitError::from)
                    })?;
                let id = match header {
                    StringLayerHeader::Chunked { chunk_set_id, .. } => Some(chunk_set_id),
                    _ => None,
                };
                match groups.iter_mut().find(|(g, _)| id.is_some() && *g == id) {
                    Some((_, chunks)) => chunks.push(tok.to_string()),
                    None => groups.push((id, vec![tok.to_string()])),
                }
            }
        } else {
            out.push((None, parse_cosigner_key(v)?));
        }
    }
    for (_, chunks) in groups {
        out.push((None, parse_cosigner_key(&chunks.join(" "))?));
    }
    for (n, vals) in placed {
        out.push((Some(n), parse_cosigner_key(&vals.join(" "))?));
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Jobs
// ---------------------------------------------------------------------------

/// One template searched over one space.
struct Unit {
    tpl_idx: usize,
    space: Space,
    /// Single-sig: the seed this unit searches (its own group).
    group: usize,
}

struct Job {
    mode: &'static str,
    templates: Vec<Template>,
    /// The template the user gave (multisig), for `from_user_template`.
    user_template: Option<usize>,
    units: Vec<Unit>,
    sources: Vec<Source>,
    single_sig: bool,
    /// Standard path count (JSON `searched.paths`).
    std_paths: usize,
    /// Keyed mode: seed derivations made to place seeds.
    keyed_derivations: usize,
    end_account: u32,
    add_paths: Vec<String>,
    md1_pinned: bool,
}

/// Build each seed's standard path list (with the analytic `--add-path`
/// dedupe) and, if asked, the 256 Refugium seat paths.
fn standard_paths(ctx: &Ctx) -> Result<(Vec<PathInfo>, Vec<String>, u32), ToolkitError> {
    let a = ctx.args;
    let add: Vec<String> = a
        .add_path
        .iter()
        .filter(|t| !add_path_repeats_standard(t, a, ctx.network))
        // An --add-path equal to a seat path is dropped when the seats are
        // searched anyway (review r1 M4: it would report two wallets).
        .filter(|t| !(a.refugium_seats && is_seat_path(t, ctx.network)))
        .cloned()
        .collect();
    let cps = build_candidate_paths(
        a.min_account,
        a.number_of_accounts,
        a.max_account,
        &add,
        ctx.network,
    );
    let end = {
        let from_n = a.min_account.saturating_add(a.number_of_accounts);
        match a.max_account {
            Some(m) => from_n.max(m.saturating_add(1)),
            None => from_n,
        }
    };
    early_ceiling(ctx, cps.len())?;
    // An --add-path entry whose path an earlier entry already names (the
    // same template twice, or `m/0'` beside `m/0h`) is dropped (final review
    // M2: it would report the one wallet twice, a false `split`).
    let mut seen: std::collections::HashSet<DerivationPath> = std::collections::HashSet::new();
    let mut contributed: BTreeSet<String> = BTreeSet::new();
    let list: Vec<PathInfo> = cps
        .iter()
        .filter(|c| {
            let new = seen.insert(c.path.clone());
            if new {
                contributed.insert(c.template_name.clone());
            }
            new
        })
        .map(|c| {
            let (template, label) = match c.template_name.as_str() {
                n @ ("bip44" | "bip49" | "bip84" | "bip86" | "bip87") => {
                    (n.to_string(), n.to_string())
                }
                "bip48-sh-wsh" => ("bip48".into(), "bip48, script type 1'".into()),
                "bip48-wsh" => ("bip48".into(), "bip48, script type 2'".into()),
                "bip48-tr-multi-a" => ("bip48".into(), "bip48, script type 3'".into()),
                "bip45" => (
                    "bundle-sh-default".into(),
                    "bundle bare-sh default m/45'".into(),
                ),
                other => ("add-path".into(), format!("--add-path {other}")),
            };
            let account = if template == "bundle-sh-default" || template == "add-path" {
                None
            } else {
                c.account
            };
            PathInfo {
                path: c.path.clone(),
                template,
                label,
                account,
                seat: None,
            }
        })
        .collect();
    let mut listed: BTreeSet<String> = BTreeSet::new();
    let add: Vec<String> = add
        .into_iter()
        .filter(|t| contributed.contains(t) && listed.insert(t.clone()))
        .collect();
    Ok((list, add, end))
}

/// The ceilings checked before any path list is materialised: the window
/// alone can name billions of paths.
fn early_ceiling(ctx: &Ctx, c: usize) -> Result<(), ToolkitError> {
    let k = ctx.seeds.len().max(1) as u32;
    let gap = u128::from(ctx.args.gap_limit);
    let c = c as u128 + if ctx.args.refugium_seats { 256 } else { 0 };
    let tuples = if ctx.args.independent_paths && k > 1 {
        c.checked_pow(k)
    } else {
        Some(c)
    };
    let scripts = tuples.and_then(|t| t.checked_mul(gap));
    match scripts {
        Some(s) if s <= REALIZED_S_MAX && s <= u128::from(u64::MAX) => {}
        _ => {
            return Err(refuse(
                "the candidate count is above the ceiling of 10^15 (or past the u64 range); \
                 narrow --number-of-accounts / --max-account, --gap-limit, drop \
                 --independent-paths, or pin slots and paths (--cosigner @N=, [fp]@N, \
                 [fp/path]@N)",
            ))
        }
    }
    let keys = c * u128::from(k) * 2 * gap;
    if keys > CACHE_MAX_KEYS {
        return Err(refuse(format!(
            "the child-key cache would hold {keys} keys (more than {CACHE_MAX_KEYS}); narrow \
             --gap-limit or the account window"
        )));
    }
    memory_ceiling(c * u128::from(k), keys)
}

/// Refuse a run whose estimated memory passes `MEMORY_MAX_BYTES`.
fn memory_ceiling(paths: u128, keys: u128) -> Result<(), ToolkitError> {
    let bytes = paths * PATH_BYTES + keys * KEY_BYTES;
    if bytes > MEMORY_MAX_BYTES {
        return Err(refuse(format!(
            "the search would need an estimated {} MB of memory ({paths} path entries and a child-key \
             cache of {keys} keys), above the ceiling of {} MB; narrow --number-of-accounts / \
             --max-account or --gap-limit",
            bytes >> 20,
            MEMORY_MAX_BYTES >> 20
        )));
    }
    Ok(())
}

/// The child steps the scan caches (each template's use-site steps over the
/// searched alternatives), in first-seen order.
fn cached_steps(ctx: &Ctx, templates: &[Template]) -> Vec<Option<u32>> {
    let mut steps: Vec<Option<u32>> = Vec::new();
    for t in templates {
        let a = alts_used(ctx, t);
        for sa in &t.slot_alts {
            for st in sa.iter().take(a) {
                if !steps.contains(st) {
                    steps.push(*st);
                }
            }
        }
    }
    steps
}

/// Print the estimate before any account or child key is derived (final
/// review M4): candidates, scripts, the derivations and the child-key cache
/// with their time (sampled on a few derivations), and the memory; refuse
/// a run above the memory ceiling. The scan's own time follows once the
/// keys exist (it is measured on them).
fn print_plan<E: Write>(
    ctx: &Ctx,
    job: &Job,
    masters: &[Xpriv],
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let gap = u128::from(ctx.args.gap_limit);
    let mut cands: u128 = 0;
    let mut scripts: u128 = 0;
    for u in &job.units {
        let mut sp = u.space.clone();
        sp.sources = job.sources.iter().map(|s| s.sp.clone()).collect();
        let c = u128::from(sp.candidates().unwrap_or(0));
        cands += c;
        scripts += c * alts_used(ctx, &job.templates[u.tpl_idx]) as u128 * gap;
    }
    let nsteps = cached_steps(ctx, &job.templates).len() as u128;
    let paths: u128 = job.sources.iter().map(|s| s.paths.len() as u128).sum();
    let keys = paths * nsteps * gap;
    memory_ceiling(paths, keys)?;
    let derivs: u128 = job
        .sources
        .iter()
        .filter(|s| s.seed.is_some())
        .map(|s| s.paths.len() as u128)
        .sum();
    // Sample up to 4 account derivations of the first seed, then 16 child
    // derivations of the last account key.
    let secp = Secp256k1::new();
    let (mut per_path, mut per_child) = (Duration::ZERO, Duration::ZERO);
    if let Some(s) = job.sources.iter().find(|s| s.seed.is_some()) {
        let m = &masters[s.seed.unwrap_or(0)];
        let n = s.paths.len().min(4);
        let t0 = std::time::Instant::now();
        let mut last: Option<Xpub> = None;
        for p in s.paths.iter().take(n) {
            if let Ok(x) = m.derive_priv(&secp, &p.path) {
                last = Some(Xpub::from_priv(&secp, &x));
            }
        }
        if n > 0 {
            per_path = t0.elapsed() / n as u32;
        }
        if let Some(x) = last {
            let t1 = std::time::Instant::now();
            for i in 0..16 {
                let _ = x.ckd_pub(&secp, ChildNumber::Normal { index: i });
            }
            per_child = t1.elapsed() / 16;
        }
    }
    let threads = search_threads().max(1) as f64;
    let derive_s =
        per_path.as_secs_f64() * derivs as f64 + per_child.as_secs_f64() * keys as f64 / threads;
    let mem = paths * PATH_BYTES + keys * KEY_BYTES;
    writeln!(
        stderr,
        "estimate: {cands} candidate(s), {scripts} script(s) over index window [0, {gap}); \
         {derivs} account key derivation(s) and a child-key cache of {keys} keys, estimated \
         {derive_s:.1} s to derive; memory estimated {} MB (ceiling {} MB)",
        (mem >> 20).max(1),
        MEMORY_MAX_BYTES >> 20
    )
    .map_err(ToolkitError::Io)
}

fn seat_paths(network: CliNetwork) -> Vec<PathInfo> {
    (0..256u32)
        .map(|n| {
            let path = DerivationPath::from(vec![
                ChildNumber::Hardened { index: 270_028 },
                ChildNumber::Hardened {
                    index: network.coin_type(),
                },
                ChildNumber::Hardened { index: n },
            ]);
            PathInfo {
                path,
                template: "refugium-seat".into(),
                label: format!("Refugium seat {}, round {}", n % 32, n / 32),
                account: None,
                seat: Some((n % 32, n / 32)),
            }
        })
        .collect()
}

/// `m/270028'/coin'/n'` with `n < 256`: one of the Refugium seat paths.
fn is_seat_path(tmpl: &str, network: CliNetwork) -> bool {
    DerivationPath::from_str(tmpl).is_ok_and(|p| {
        let v: Vec<ChildNumber> = p.into_iter().copied().collect();
        matches!(v.as_slice(), [
            ChildNumber::Hardened { index: 270_028 },
            ChildNumber::Hardened { index: c },
            ChildNumber::Hardened { index: n },
        ] if *c == network.coin_type() && *n < 256)
    })
}

/// SPEC §3 dedupe: an `--add-path` that would repeat a standard path is
/// dropped analytically (never by materialising the window).
fn add_path_repeats_standard(tmpl: &str, a: &AddressOfSeedArgs, network: CliNetwork) -> bool {
    let lo = a.min_account;
    let hi = {
        let from_n = lo.saturating_add(a.number_of_accounts);
        match a.max_account {
            Some(m) => from_n.max(m.saturating_add(1)),
            None => from_n,
        }
    };
    let coin = network.coin_type();
    let is_std = |p: &DerivationPath| -> bool {
        let v: Vec<ChildNumber> = p.into_iter().copied().collect();
        let h = |c: &ChildNumber| match c {
            ChildNumber::Hardened { index } => Some(*index),
            _ => None,
        };
        match v.as_slice() {
            [p0] => h(p0) == Some(45),
            [p0, c, acc] => {
                matches!(h(p0), Some(44 | 49 | 84 | 86 | 87))
                    && h(c) == Some(coin)
                    && h(acc).is_some_and(|x| x >= lo && x < hi)
            }
            [p0, c, acc, st] => {
                h(p0) == Some(48)
                    && h(c) == Some(coin)
                    && h(acc).is_some_and(|x| x >= lo && x < hi)
                    && matches!(h(st), Some(1..=3))
            }
            _ => false,
        }
    };
    if tmpl.contains("account") {
        if lo >= hi {
            return false;
        }
        // Equal to a family template iff it equals the family path at two
        // accounts (or the one account of a single-account window).
        let sub = |acc: u32| {
            DerivationPath::from_str(&tmpl.replacen("account'", &format!("{acc}'"), 1).replacen(
                "account",
                &acc.to_string(),
                1,
            ))
            .ok()
        };
        let probes: Vec<u32> = if hi - lo >= 2 {
            vec![lo, hi - 1]
        } else {
            vec![lo]
        };
        // A token in the account position yields standard paths only when
        // the token sits exactly at the account step of a family.
        let token_at_account = tmpl.contains("account'")
            && probes
                .iter()
                .all(|acc| sub(*acc).is_some_and(|p| is_std(&p)));
        token_at_account && {
            // Same family at both probes and the token is the account level.
            let p0 = sub(probes[0]);
            p0.is_some_and(|p| {
                let v: Vec<ChildNumber> = p.into_iter().copied().collect();
                v.len() >= 3 && v[2] == ChildNumber::Hardened { index: probes[0] }
            })
        }
    } else {
        DerivationPath::from_str(tmpl).is_ok_and(|p| is_std(&p))
    }
}

fn seed_source(
    i: usize,
    fps: &[Fingerprint],
    paths: Vec<PathInfo>,
    std_len: usize,
    seat_base: Option<usize>,
    n: usize,
) -> Source {
    Source {
        seed: Some(i),
        cosigner: None,
        fingerprint: Some(fps[i]),
        paths,
        accounts: Vec::new(),
        sp: SourcePaths {
            std_len,
            seat_base,
            pin: vec![None; n],
        },
        stated_fp: None,
    }
}

fn cosigner_source(c: CosignerIn, n: usize) -> Source {
    let (template, label) = if c.path.is_some() {
        ("cosigner".to_string(), "cosigner".to_string())
    } else {
        (
            "cosigner".to_string(),
            "cosigner, origin not given".to_string(),
        )
    };
    let path = c.path.clone().unwrap_or_default();
    Source {
        seed: None,
        fingerprint: c.fp,
        accounts: vec![c.xpub],
        paths: vec![PathInfo {
            path,
            template,
            label,
            account: None,
            seat: None,
        }],
        sp: SourcePaths {
            std_len: 1,
            seat_base: None,
            pin: vec![None; n],
        },
        stated_fp: c.fp,
        cosigner: Some(c),
    }
}

fn derive_accounts(sources: &mut [Source], masters: &[Xpriv]) -> Result<(), ToolkitError> {
    let secp = Secp256k1::new();
    for s in sources.iter_mut() {
        if let Some(i) = s.seed {
            let m = &masters[i];
            s.accounts = s
                .paths
                .iter()
                .map(|p| {
                    m.derive_priv(&secp, &p.path)
                        .map(|x| Xpub::from_priv(&secp, &x))
                        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))
                })
                .collect::<Result<_, _>>()?;
        }
    }
    Ok(())
}

const SINGLE_SIG_TEMPLATES: [&str; 4] = [
    "pkh(@0/<0;1>/*)",
    "sh(wpkh(@0/<0;1>/*))",
    "wpkh(@0/<0;1>/*)",
    "tr(@0/<0;1>/*)",
];

fn single_sig_job(ctx: &Ctx) -> Result<Job, ToolkitError> {
    let (mut std, add, end) = standard_paths(ctx)?;
    if ctx.args.refugium_seats {
        std.extend(seat_paths(ctx.network));
    }
    let c = std.len();
    let templates: Vec<Template> = SINGLE_SIG_TEMPLATES
        .iter()
        .map(|t| Template::parse(t).map_err(|e| tpl_refuse(&e, 0, &[])))
        .collect::<Result<_, _>>()?;
    let sources: Vec<Source> = (0..ctx.seeds.len())
        .map(|i| seed_source(i, ctx.fps, std.clone(), c, None, 1))
        .collect();
    let kinds: BTreeSet<SpkKind> = ctx.targets.iter().map(|t| t.2).collect();
    let mut units = Vec::new();
    for s in 0..sources.len() {
        for (ti, t) in templates.iter().enumerate() {
            if !kinds.contains(&t.kind) {
                continue;
            }
            units.push(Unit {
                tpl_idx: ti,
                group: s,
                space: Space {
                    n: 1,
                    pinned_key: vec![Some(s)],
                    free_slots: vec![],
                    free_keys: vec![],
                    orders: vec![1],
                    segments: vec![PathTuples::Shared { c: c as u64 }],
                    sources: Vec::new(),
                },
            });
        }
    }
    if units.is_empty() {
        return Err(refuse(
            "no target can be produced by a single-sig script (pkh, sh(wpkh), wpkh, tr); a \
             32-byte v0 (P2WSH) target needs --descriptor",
        ));
    }
    precheck_counts(ctx, &units, &sources, &templates)?;
    Ok(Job {
        mode: "search",
        templates,
        user_template: None,
        units,
        sources,
        single_sig: true,
        std_paths: c,
        keyed_derivations: 0,
        end_account: end,
        add_paths: add,
        md1_pinned: false,
    })
}

fn template_job(ctx: &Ctx, text: &str, md1: Option<&Md1Template>) -> Result<Job, ToolkitError> {
    // A literal key beside `@N` placeholders becomes a placed cosigner slot
    // after the last placeholder (review r1 I2): the evaluator translates
    // slots only, so a literal key left in place would match nothing.
    let placeholders = Regex::new(r"@(\d+)")
        .expect("static")
        .captures_iter(text)
        .filter_map(|c| c[1].parse::<usize>().ok())
        .max()
        .map_or(0, |m| m + 1);
    let (lifted_text, lifted, typed) = templatise_keys(text, placeholders)?;
    let tpl = Template::parse(&lifted_text).map_err(|e| tpl_refuse(&e, placeholders, &typed))?;
    let n = tpl.n;
    let (std, add, end) = standard_paths(ctx)?;
    let mut std = std;
    let seats_in_std = ctx.args.refugium_seats && n == 1;
    if seats_in_std {
        std.extend(seat_paths(ctx.network));
    }
    let c = std.len();
    let k = ctx.seeds.len();
    let mut cos = intake_cosigners(&ctx.args.cosigner)?;
    for (j, k) in lifted.into_iter().enumerate() {
        cos.push((Some(placeholders + j), k));
    }

    // Sources: seeds then cosigners.
    let mut sources: Vec<Source> = (0..k)
        .map(|i| seed_source(i, ctx.fps, std.clone(), c, None, n))
        .collect();
    let mut pinned_key: Vec<Option<usize>> = vec![None; n];
    let mut unplaced_cos: Vec<usize> = Vec::new();
    for (slot, ci) in cos {
        let id = sources.len();
        sources.push(cosigner_source(ci, n));
        match slot {
            Some(s) if s >= n => {
                return Err(refuse(format!(
                    "--cosigner @{s}=: the descriptor has slots @0 to @{}",
                    n - 1
                )))
            }
            Some(s) => {
                if pinned_key[s].is_some() {
                    return Err(refuse(format!("@{s} is given two keys")));
                }
                pinned_key[s] = Some(id);
            }
            None => unplaced_cos.push(id),
        }
    }
    // Stated origins: `[fp]@N` pins a seed (or a cosigner with that
    // fingerprint); a path pins the slot's path.
    let mut slot_fp: Vec<Option<Fingerprint>> = tpl.slot_fp.clone();
    let mut slot_path: Vec<Option<DerivationPath>> = tpl.slot_origin.clone();
    if let Some(md) = md1 {
        for (i, (p, f)) in md.origins.iter().enumerate() {
            if slot_path[i].is_none() {
                slot_path[i] = p.clone();
            }
            if slot_fp[i].is_none() {
                slot_fp[i] = *f;
            }
        }
    }
    let mut pinned_seed_or_cos = |slot: usize, fp: Fingerprint| -> Result<(), ToolkitError> {
        if pinned_key[slot].is_some() {
            return Ok(());
        }
        if let Some(i) = ctx.fps.iter().position(|f| *f == fp) {
            if pinned_key.contains(&Some(i)) {
                return Err(refuse(format!(
                    "seed {i} ({fp}) is stated in two slots; one seed in two slots is not \
                     searched in v1"
                )));
            }
            pinned_key[slot] = Some(i);
            return Ok(());
        }
        if let Some(pos) = unplaced_cos
            .iter()
            .position(|id| sources[*id].stated_fp == Some(fp))
        {
            pinned_key[slot] = Some(unplaced_cos.remove(pos));
            return Ok(());
        }
        if md1.is_some() {
            // A card's recorded fingerprint without a matching seed is a hint.
            return Ok(());
        }
        Err(refuse(format!(
            "@{slot} states fingerprint {fp}, which matches no seed given (seed fingerprints: {}) \
             and no --cosigner",
            ctx.fps
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )))
    };
    for (slot, fp) in slot_fp.iter().enumerate() {
        if let Some(fp) = fp {
            pinned_seed_or_cos(slot, *fp)?;
        }
    }
    // Path pins: append the pinned path to every seed's list, once per
    // distinct path (equal pins share an index, so the collapse test of
    // `slot_orders` compares paths, not positions).
    for (slot, p) in slot_path.iter().enumerate() {
        if let Some(p) = p {
            for s in sources.iter_mut().filter(|s| s.seed.is_some()) {
                let pinned_before = (0..slot)
                    .filter_map(|o| s.sp.pin[o])
                    .find(|i| s.paths[*i].path == *p);
                if let Some(idx) = pinned_before {
                    s.sp.pin[slot] = Some(idx);
                    continue;
                }
                let idx = s.paths.len();
                s.paths.push(PathInfo {
                    path: p.clone(),
                    template: if md1.is_some() {
                        "md1".into()
                    } else {
                        "stated".into()
                    },
                    label: if md1.is_some() {
                        "recorded on the md1 card".into()
                    } else {
                        "stated in the descriptor".into()
                    },
                    account: None,
                    seat: None,
                });
                s.sp.pin[slot] = Some(idx);
            }
        }
    }
    // Refugium seats for multisig: per slot 8 rounds.
    if ctx.args.refugium_seats && n > 1 {
        for s in sources.iter_mut().filter(|s| s.seed.is_some()) {
            s.sp.seat_base = Some(s.paths.len());
            s.paths.extend(seat_paths(ctx.network));
        }
    }
    let free_slots: Vec<usize> = (0..n).filter(|s| pinned_key[*s].is_none()).collect();
    let mut free_keys: Vec<usize> = (0..k).filter(|i| !pinned_key.contains(&Some(*i))).collect();
    free_keys.extend(unplaced_cos.iter().copied());
    if free_keys.len() > free_slots.len() {
        return Err(refuse(format!(
            "more keys than slots: {} key(s) for {} free slot(s) of the descriptor",
            free_keys.len(),
            free_slots.len()
        )));
    }
    if free_keys.len() < free_slots.len() {
        let missing: Vec<String> = free_slots.iter().map(|s| format!("@{s}")).collect();
        return Err(refuse(format!(
            "a slot without a key: {} free slot(s) ({}) but {} key(s) to place; give every slot a \
             seed or a --cosigner",
            free_slots.len(),
            missing.join(", "),
            free_keys.len()
        )));
    }
    // Path tuples. A seed whose slot is pinned and path-pinned takes no
    // path digit (review r1 M7: its digit would repeat every assignment).
    let seed_ids: Vec<usize> = (0..k)
        .filter(|i| {
            !pinned_key
                .iter()
                .enumerate()
                .any(|(slot, p)| *p == Some(*i) && slot_path[slot].is_some())
        })
        .collect();
    let any_unpinned_path_seed_slot = (0..n).any(|slot| {
        slot_path[slot].is_none()
            && match pinned_key[slot] {
                Some(s) => sources[s].seed.is_some(),
                None => free_keys.iter().any(|s| sources[*s].seed.is_some()),
            }
    });
    let mut segments = Vec::new();
    if any_unpinned_path_seed_slot {
        if ctx.args.independent_paths && seed_ids.len() > 1 {
            segments.push(PathTuples::Independent {
                seeds: seed_ids.clone(),
                c: c as u64,
            });
        } else {
            segments.push(PathTuples::Shared { c: c as u64 });
        }
        if ctx.args.refugium_seats && n > 1 {
            segments.push(PathTuples::Refugium { seeds: seed_ids });
        }
    } else {
        segments.push(PathTuples::Shared { c: 1 });
    }
    let src_paths: Vec<SourcePaths> = sources.iter().map(|s| s.sp.clone()).collect();
    let orders = slot_orders(&tpl, &free_slots, &free_keys, &segments, &src_paths)
        .ok_or_else(|| refuse("too many free slots"))?;
    let kinds: BTreeSet<SpkKind> = ctx.targets.iter().map(|t| t.2).collect();
    if !kinds.contains(&tpl.kind) {
        return Err(refuse(format!(
            "no target can be produced by this descriptor ({}); every target is another output \
             type",
            tpl.script_type
        )));
    }
    let units = vec![Unit {
        tpl_idx: 0,
        group: 0,
        space: Space {
            n,
            pinned_key,
            free_slots,
            free_keys,
            orders,
            segments,
            sources: Vec::new(),
        },
    }];
    let templates = vec![tpl];
    precheck_counts(ctx, &units, &sources, &templates)?;
    Ok(Job {
        mode: "search",
        templates,
        user_template: Some(0),
        units,
        sources,
        single_sig: n == 1,
        std_paths: c,
        keyed_derivations: 0,
        end_account: end,
        add_paths: add,
        md1_pinned: md1.is_some(),
    })
}

/// A literal extended public key with an optional `[fp/path]` origin.
fn key_regex() -> &'static Regex {
    static KEY: OnceLock<Regex> = OnceLock::new();
    KEY.get_or_init(|| {
        Regex::new(
            r"(?:\[([0-9a-fA-F]{8})((?:/\d+['h]?)*)\])?([a-zA-Z]pub[1-9A-HJ-NP-Za-km-z]{100,112})",
        )
        .expect("static regex")
    })
}

/// A template refusal in the user's terms (final review M5): one `error:`
/// prefix, and each `@N` the toolkit made from a typed key (`N >= first`)
/// named by that key, since the user never typed the placeholder.
fn tpl_refuse(e: &str, first: usize, typed: &[String]) -> ToolkitError {
    let mut m = e;
    while let Some(r) = m.strip_prefix("error: ") {
        m = r;
    }
    let mut named = false;
    let out = Regex::new(r"@(\d+)")
        .expect("static")
        .replace_all(m, |c: &regex::Captures| {
            match c[1]
                .parse::<usize>()
                .ok()
                .and_then(|n| n.checked_sub(first))
                .and_then(|j| typed.get(j))
            {
                Some(k) => {
                    named = true;
                    let short: String = k.chars().take(16).collect();
                    format!("the key {short}…")
                }
                None => c[0].to_string(),
            }
        })
        .into_owned();
    if named {
        refuse(format!(
            "{out} (this descriptor shape cannot be put on a card, so `bundle` cannot mint it \
             and it is not searched)"
        ))
    } else {
        refuse(out)
    }
}

/// Replace every literal key of `text` by `@{first + i}` (one slot per
/// distinct key, in order of first appearance); return the rewritten text
/// the keys with their stated origins, and the keys as typed.
#[allow(clippy::type_complexity)]
fn templatise_keys(
    text: &str,
    first: usize,
) -> Result<(String, Vec<CosignerIn>, Vec<String>), ToolkitError> {
    let mut keys: Vec<CosignerIn> = Vec::new();
    let mut key_strs: Vec<String> = Vec::new();
    let mut bad: Option<ToolkitError> = None;
    let templ = key_regex()
        .replace_all(text, |c: &regex::Captures| {
            let k = c[3].to_string();
            let i = match key_strs.iter().position(|x| *x == k) {
                Some(i) => i,
                None => {
                    let parsed = (|| -> Result<CosignerIn, ToolkitError> {
                        let (canon, _) = crate::slip0132::normalize_xpub_prefix(&k)?;
                        let xpub = Xpub::from_str(&canon).map_err(ToolkitError::from)?;
                        let fp = c
                            .get(1)
                            .map(|m| Fingerprint::from_str(m.as_str()))
                            .transpose()
                            .map_err(|_| refuse("bad origin fingerprint"))?;
                        let path = match c.get(2).map(|m| m.as_str()).filter(|s| !s.is_empty()) {
                            Some(p) => Some(
                                DerivationPath::from_str(&format!("m{p}"))
                                    .map_err(|e| refuse(format!("origin path: {e}")))?,
                            ),
                            None => None,
                        };
                        Ok(CosignerIn { xpub, fp, path })
                    })();
                    match parsed {
                        Ok(ci) => {
                            keys.push(ci);
                            key_strs.push(k);
                            key_strs.len() - 1
                        }
                        Err(e) => {
                            bad.get_or_insert(e);
                            0
                        }
                    }
                }
            };
            format!("@{}", first + i)
        })
        .into_owned();
    if let Some(e) = bad {
        return Err(e);
    }
    Ok((templ, keys, key_strs))
}

/// Verify a BIP-380 checksum on the text as given and strip it (review r1
/// I3: the keys are rewritten to `@N` after this, which would change it).
fn strip_checksum(text: &str) -> Result<String, ToolkitError> {
    let t = text.trim();
    if !t.contains('#') {
        return Ok(t.to_string());
    }
    miniscript::descriptor::checksum::verify_checksum(t)
        .map(str::to_string)
        .map_err(|e| refuse(format!("--descriptor checksum: {e}")))
}

fn keyed_job<E: Write>(
    ctx: &Ctx,
    text: &str,
    masters: &[Xpriv],
    stderr: &mut E,
) -> Result<Job, ToolkitError> {
    let expanded = crate::parse_descriptor::expand_literal_double_star(text).into_owned();
    let (templ, keys, typed) = templatise_keys(&expanded, 0)?;
    if keys.is_empty() {
        return Err(refuse(
            "--descriptor has neither @N placeholders nor public keys",
        ));
    }
    if !ctx.args.cosigner.is_empty() {
        return Err(refuse(
            "--cosigner is for @N templates; a keyed descriptor already states every key",
        ));
    }
    let tpl = Template::parse(&templ).map_err(|e| tpl_refuse(&e, 0, &typed))?;
    let n = tpl.n;
    let (mut std, add, end) = standard_paths(ctx)?;
    // A key without an origin is compared with the seat paths too when
    // they are asked for (review r1 M10).
    if ctx.args.refugium_seats {
        std.extend(seat_paths(ctx.network));
    }
    let secp = Secp256k1::new();
    // Place seeds by key derivation (SPEC §2), fingerprint only a hint.
    let mut placed: Vec<Option<(usize, PathInfo)>> = vec![None; n];
    let mut derivations = 0usize;
    for (si, m) in masters.iter().enumerate() {
        let mut hit: Option<(usize, PathInfo)> = None;
        // Stated paths first, keys whose stated fp matches this seed first.
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by_key(|i| keys[*i].fp != Some(ctx.fps[si]));
        for i in order {
            if let Some(p) = &keys[i].path {
                derivations += 1;
                let x = m
                    .derive_priv(&secp, p)
                    .map(|x| Xpub::from_priv(&secp, &x))
                    .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
                if x.public_key == keys[i].xpub.public_key
                    && x.chain_code == keys[i].xpub.chain_code
                {
                    hit = Some((
                        i,
                        PathInfo {
                            path: p.clone(),
                            template: "keyed".into(),
                            label: "stated in the descriptor".into(),
                            account: None,
                            seat: None,
                        },
                    ));
                    break;
                }
            } else {
                for pi in &std {
                    derivations += 1;
                    let x = m
                        .derive_priv(&secp, &pi.path)
                        .map(|x| Xpub::from_priv(&secp, &x))
                        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
                    if x.public_key == keys[i].xpub.public_key {
                        hit = Some((i, pi.clone()));
                        break;
                    }
                }
                if hit.is_some() {
                    break;
                }
            }
        }
        match hit {
            Some((i, pi)) => {
                if placed[i].is_some() {
                    return Err(refuse(format!("two seeds equal key @{i}")));
                }
                if let Some(st) = keys[i].fp {
                    if st != ctx.fps[si] {
                        writeln!(
                            stderr,
                            "note: key @{i} states {st}; this seed is {}; the printed command \
                             uses the seed's real fingerprint",
                            ctx.fps[si]
                        )
                        .map_err(ToolkitError::Io)?;
                    }
                }
                placed[i] = Some((si, pi));
            }
            None => {
                return Err(refuse(format!(
                    "seed {si} ({}) equals no key of the descriptor: tried each key's stated \
                     origin path, and the {} standard (and asked-for seat) paths for keys without \
                     one",
                    ctx.fps[si],
                    std.len()
                )))
            }
        }
    }
    // Sources: one per slot (seed or watch-only key), all pinned.
    let mut sources: Vec<Source> = Vec::new();
    let mut seed_src: Vec<Option<usize>> = vec![None; ctx.seeds.len()];
    for (i, p) in placed.iter().enumerate() {
        if let Some((si, pi)) = p {
            seed_src[*si] = Some(i);
            let mut s = seed_source(*si, ctx.fps, vec![pi.clone()], 1, None, n);
            s.stated_fp = keys[i].fp;
            sources.push(s);
        } else {
            sources.push(cosigner_source(keys[i].clone(), n));
        }
    }
    let pinned_key: Vec<Option<usize>> = (0..n).map(Some).collect();
    let kinds: BTreeSet<SpkKind> = ctx.targets.iter().map(|t| t.2).collect();
    if !kinds.contains(&tpl.kind) {
        return Err(refuse(format!(
            "no target can be produced by this descriptor ({})",
            tpl.script_type
        )));
    }
    let units = vec![Unit {
        tpl_idx: 0,
        group: 0,
        space: Space {
            n,
            pinned_key,
            free_slots: vec![],
            free_keys: vec![],
            orders: vec![1],
            segments: vec![PathTuples::Shared { c: 1 }],
            sources: Vec::new(),
        },
    }];
    let templates = vec![tpl];
    Ok(Job {
        mode: "keyed",
        templates,
        user_template: Some(0),
        units,
        sources,
        single_sig: n == 1,
        std_paths: std.len(),
        keyed_derivations: derivations,
        end_account: end,
        add_paths: add,
        md1_pinned: false,
    })
}

fn alts_used(ctx: &Ctx, tpl: &Template) -> usize {
    if ctx.args.external_only {
        1
    } else {
        tpl.alternatives()
    }
}

/// SPEC §4 ceilings: checked arithmetic, refuse above 10^15 candidate
/// scripts or past `u64`; refuse a child-key cache past `CACHE_MAX_KEYS`.
fn precheck_counts(
    ctx: &Ctx,
    units: &[Unit],
    sources: &[Source],
    templates: &[Template],
) -> Result<(), ToolkitError> {
    let mut scripts: u128 = 0;
    let gap = u128::from(ctx.args.gap_limit);
    for u in units {
        let mut sp = u.space.clone();
        sp.sources = sources.iter().map(|s| s.sp.clone()).collect();
        let c = sp.candidates().ok_or_else(|| {
            refuse(
                "the candidate count is past the u64 range; narrow --number-of-accounts / \
                 --max-account, drop --independent-paths, or pin slots and paths",
            )
        })?;
        let a = alts_used(ctx, &templates[u.tpl_idx]) as u128;
        scripts = scripts.saturating_add(u128::from(c) * a * gap);
    }
    if scripts > u128::from(u64::MAX) || scripts > REALIZED_S_MAX {
        return Err(refuse(format!(
            "{scripts} candidate scripts is above the ceiling of 10^15; narrow \
             --number-of-accounts / --max-account, --gap-limit, drop --independent-paths, or pin \
             slots and paths (--cosigner @N=, [fp]@N, [fp/path]@N)"
        )));
    }
    let steps = 2u128;
    let keys: u128 = sources
        .iter()
        .map(|s| s.paths.len() as u128 * steps * gap)
        .sum();
    if keys > CACHE_MAX_KEYS {
        return Err(refuse(format!(
            "the child-key cache would hold {keys} keys (more than {CACHE_MAX_KEYS}); narrow \
             --gap-limit or the account window"
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Execute: estimate, scan, analyse, report
// ---------------------------------------------------------------------------

/// A found or twin wallet before rendering.
#[derive(Clone)]
struct Wallet {
    tpl_text: String,
    script_type: &'static str,
    assign: Vec<(usize, usize)>,
    /// (target, alt, idx), ascending.
    hits: Vec<(usize, u32, u32)>,
    merged_forms: usize,
    group: usize,
    from_user: bool,
}

impl Wallet {
    fn target_set(&self) -> BTreeSet<usize> {
        self.hits.iter().map(|h| h.0).collect()
    }
}

fn execute<W: Write, E: Write>(
    ctx: &Ctx,
    job: Job,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let threads = search_threads();
    let steps = cached_steps(ctx, &job.templates);
    let accounts: Vec<Vec<Xpub>> = job.sources.iter().map(|s| s.accounts.clone()).collect();
    let cache = ChildCache::build(&accounts, &steps, ctx.args.gap_limit, threads);
    let src_paths: Vec<SourcePaths> = job.sources.iter().map(|s| s.sp.clone()).collect();

    // Estimate (stderr).
    let mut total_cands: u64 = 0;
    let mut total_scripts: u64 = 0;
    let mut est = Duration::ZERO;
    let mut spaces: Vec<Space> = Vec::new();
    for u in &job.units {
        let mut sp = u.space.clone();
        sp.sources = src_paths.clone();
        let tpl = &job.templates[u.tpl_idx];
        let a = alts_used(ctx, tpl);
        // Fail loud (review r1 I2): a template the evaluator cannot
        // translate would match nothing and print a false no-match.
        if !probe_eval(&sp, tpl, &cache, a) {
            return Err(refuse(format!(
                "internal: {} cannot be evaluated with the keys given; nothing was searched",
                tpl.text
            )));
        }
        let c = sp.candidates().unwrap_or(0);
        total_cands = total_cands.saturating_add(c);
        total_scripts = total_scripts.saturating_add(c * a as u64 * u64::from(ctx.args.gap_limit));
        let per = super::seed_search::calibrate(&sp, tpl, &cache, ctx.target_map, a, 8);
        est += match cap_decision(c, per, threads, None) {
            Ok(CapDecision::RunSilent { estimate } | CapDecision::RunWithProgress { estimate }) => {
                estimate
            }
            Err(_) => Duration::ZERO,
        };
        spaces.push(sp);
    }
    let orders = if job.single_sig {
        1
    } else {
        job.units[0].space.max_orders()
    };
    writeln!(
        stderr,
        "scan estimate: {total_cands} candidate(s), {total_scripts} script(s) over {} \
         alternative(s) and index window [0, {}); {} slot order(s); child-key cache {} keys ({} \
         KB); estimated {:.1} s on {threads} thread(s)",
        job.templates
            .iter()
            .map(|t| alts_used(ctx, t))
            .max()
            .unwrap_or(1),
        ctx.args.gap_limit,
        orders,
        cache.len(),
        cache.len() * 33 / 1024,
        est.as_secs_f64()
    )
    .map_err(ToolkitError::Io)?;
    if job.end_account > MD1_STEP_MAX {
        writeln!(
            stderr,
            "note: the account window reaches 2^29 or above; a match there is reported but no card \
             can record it"
        )
        .map_err(ToolkitError::Io)?;
    }
    let progress = est >= crate::permutation_search::SILENT_THRESHOLD;

    // Scan.
    let mut wallets_raw: RawWallets = BTreeMap::new();
    let mut matches_total: u64 = 0;
    let mut matches_kept: usize = 0;
    let mut capped = false;
    for (u, sp) in job.units.iter().zip(&spaces) {
        let tpl = &job.templates[u.tpl_idx];
        let a = alts_used(ctx, tpl);
        let mut prog = |done: u64, total: u64| {
            let _ = writeln!(stderr, "progress: {done}/{total} candidates");
        };
        let res = scan(
            sp,
            tpl,
            &cache,
            ctx.target_map,
            a,
            threads,
            MATCH_CAP,
            progress.then_some(Duration::from_secs(10)),
            &mut prog,
        );
        matches_total += res.total_hits;
        matches_kept += res.hits.len();
        capped |= res.capped;
        for h in res.hits {
            if let Some(assign) = sp.decode(h.rank) {
                wallets_raw
                    .entry((u.tpl_idx, u.group, assign))
                    .or_default()
                    .insert((h.target, h.alt, h.index));
            }
        }
    }

    // Merge by symmetry (§5).
    let mut merged: BTreeMap<(usize, String), Vec<Wallet>> = BTreeMap::new();
    for ((ti, group, assign), hits) in wallets_raw {
        let tpl = &job.templates[ti];
        let key = canonical_label(&tpl.text, &assign);
        let mut hv: Vec<(usize, u32, u32)> = Vec::new();
        for h in hits {
            if !hv.iter().any(|x| x.0 == h.0) {
                hv.push(h);
            }
        }
        merged.entry((ti, key)).or_default().push(Wallet {
            tpl_text: tpl.text.clone(),
            script_type: tpl.script_type,
            assign,
            hits: hv,
            merged_forms: 0,
            group,
            from_user: job.user_template == Some(ti),
        });
    }
    let mut found: Vec<Wallet> = merged
        .into_values()
        .map(|mut v| {
            v.sort_by(|a, b| a.assign.cmp(&b.assign));
            let n = v.len();
            let mut w = v.swap_remove(0);
            w.merged_forms = n - 1;
            w
        })
        .collect();
    found.sort_by_key(|w| (w.group, w.hits.first().map(|h| (h.1, h.2, h.0))));

    let analysis = Analysis {
        ctx,
        job: &job,
        cache: &cache,
    };
    analysis.report(
        found,
        matches_total,
        matches_kept,
        capped,
        total_scripts,
        stdout,
        stderr,
    )
}

/// Instantiate the template's key leaves with (source, path, use-site)
/// labels and canonicalise under the known symmetries.
fn canonical_label(text: &str, assign: &[(usize, usize)]) -> String {
    let Ok(e) = Expr::parse(text) else {
        return format!("{assign:?}");
    };
    let inst = e.map_leaves(&|leaf| match leaf_slot(leaf) {
        Some(i) if i < assign.len() => {
            format!("k{}p{}{}", assign[i].0, assign[i].1, leaf_use_site(leaf))
        }
        _ => leaf.to_string(),
    });
    inst.canonical().render()
}

struct Analysis<'a> {
    ctx: &'a Ctx<'a>,
    job: &'a Job,
    cache: &'a ChildCache,
}

struct Twin {
    wallet: Wallet,
    /// Every target the twin produces in the window.
    covers: BTreeSet<usize>,
}

impl Analysis<'_> {
    fn spk(
        &self,
        tpl: &Template,
        assign: &[(usize, usize)],
        alt: usize,
        idx: u32,
    ) -> Option<ScriptBuf> {
        let secp = Secp256k1::verification_only();
        let mut keys: Vec<PublicKey> = Vec::with_capacity(tpl.n);
        for (slot, (src, path)) in assign.iter().enumerate() {
            let step = tpl.slot_alts[slot][alt];
            // The scan's child-key cache when it holds the key (every index
            // of the window), else a derivation.
            let cached = self
                .cache
                .steps
                .iter()
                .position(|s| *s == step)
                .filter(|_| idx < self.cache.gap);
            keys.push(match cached {
                Some(si) => self.cache.get(*src, *path, si, idx),
                None => derive_child(&secp, &self.job.sources[*src].accounts[*path], step, idx)?,
            });
        }
        tpl.eval(&keys)
    }

    /// True when `tpl`+`assign` produces each of the wallet's matched
    /// targets at that target's own (alternative, index): the twin check's
    /// "one script per target" (final review I2: no index cap).
    fn fits_hits(&self, tpl: &Template, assign: &[(usize, usize)], w: &Wallet) -> bool {
        w.hits.iter().all(|(t, a, i)| {
            self.spk(tpl, assign, *a as usize, *i).as_ref() == Some(&self.ctx.targets[*t].1)
        })
    }

    fn alts(&self, tpl: &Template) -> usize {
        alts_used(self.ctx, tpl)
    }

    /// (target, alt, idx) the wallet `tpl`+`assign` produces in the whole
    /// index window (served from the child-key cache; no cap).
    fn coverage(&self, tpl: &Template, assign: &[(usize, usize)]) -> Vec<(usize, u32, u32)> {
        let mut out = Vec::new();
        for a in 0..self.alts(tpl) {
            for i in 0..self.ctx.args.gap_limit {
                if let Some(s) = self.spk(tpl, assign, a, i) {
                    if let Some(t) = self.ctx.target_map.get(&s) {
                        if !out.iter().any(|x: &(usize, u32, u32)| x.0 == *t) {
                            out.push((*t, a as u32, i));
                        }
                    }
                }
            }
        }
        out
    }

    /// SPEC §5 twin check, per fragment, over the wallet's matched targets.
    fn twin(&self, w: &Wallet) -> (Option<Twin>, String) {
        let Ok(expr) = Expr::parse(&w.tpl_text) else {
            return (None, "none".into());
        };
        let frags = expr.multi_fragments();
        if frags.is_empty() || w.assign.len() < 2 {
            return (None, "none (no multi fragment)".into());
        }
        let Ok(base) = Template::parse(&w.tpl_text) else {
            return (None, "none".into());
        };
        let mut fitting: Vec<(Vec<usize>, Expr)> = Vec::new();
        let mut why: Vec<String> = Vec::new();
        for f in &frags {
            let Some(rep) = self.fragment_twin(&expr, f, &base, w) else {
                // Visible skip (final review N7): a sorted fragment with an
                // argument that is not a slot gets no twin check.
                why.push("a fragment with an argument that is not a slot was not checked".into());
                continue;
            };
            let mut e2 = expr.clone();
            *e2.at_mut(f) = rep.clone();
            let Ok(t2) = Template::parse(&e2.render()) else {
                continue;
            };
            if self.fits_hits(&t2, &w.assign, w) {
                fitting.push((f.clone(), rep));
            } else if let Expr::Node { name, .. } = expr.at(f) {
                why.push(if matches!(multi_family(name), Some((true, _))) {
                    "no multi order fits every target".to_string()
                } else {
                    "the sorted form does not fit every target".to_string()
                });
            }
        }
        if fitting.is_empty() {
            return (
                None,
                format!(
                    "none ({})",
                    why.first()
                        .cloned()
                        .unwrap_or_else(|| "no twin fits".into())
                ),
            );
        }
        let mut e2 = expr.clone();
        for (f, rep) in &fitting {
            *e2.at_mut(f) = rep.clone();
        }
        let text = e2.render();
        let Ok(t2) = Template::parse(&text) else {
            return (None, "none".into());
        };
        if !self.fits_hits(&t2, &w.assign, w) {
            return (
                None,
                "none (the fragments' twins do not fit together)".into(),
            );
        }
        // The twin's own hits: the wallet's (at their own indices), then any
        // further target it produces in the window.
        let mut cov: Vec<(usize, u32, u32)> = w.hits.clone();
        for h in self.coverage(&t2, &w.assign) {
            if !cov.iter().any(|x| x.0 == h.0) {
                cov.push(h);
            }
        }
        cov.sort();
        let covers: BTreeSet<usize> = cov.iter().map(|x| x.0).collect();
        let note = if frags.len() > 1 {
            format!("{text} also fits every target (several fragments: other mixtures also fit)")
        } else {
            format!("{text} also fits every target")
        };
        (
            Some(Twin {
                wallet: Wallet {
                    tpl_text: text,
                    script_type: t2.script_type,
                    assign: w.assign.clone(),
                    hits: cov,
                    merged_forms: 0,
                    group: w.group,
                    from_user: false,
                },
                covers,
            }),
            note,
        )
    }

    fn fragment_twin(&self, expr: &Expr, f: &[usize], base: &Template, w: &Wallet) -> Option<Expr> {
        let Expr::Node { name, args } = expr.at(f) else {
            return None;
        };
        let (sorted, tap) = multi_family(name)?;
        let (prefix, b) = match name.rfind(':') {
            Some(p) => (&name[..=p], &name[p + 1..]),
            None => ("", name.as_str()),
        };
        if !sorted {
            let nb = b.replacen("multi", "sortedmulti", 1);
            return Some(Expr::Node {
                name: format!("{prefix}{nb}"),
                args: args.clone(),
            });
        }
        // Sorted: the order that sorts the arguments' child keys at the first
        // matched target; the coverage check then decides whether it fits
        // every matched target (one order for all, else no `multi` twin).
        let (_, alt, idx) = *w.hits.first()?;
        let secp = Secp256k1::verification_only();
        let mut keyed: Vec<(Vec<u8>, Expr)> = Vec::new();
        for a in args.iter().skip(1) {
            let Expr::Leaf(l) = a else {
                return None;
            };
            let slot = leaf_slot(l)?;
            let (src, path) = w.assign.get(slot)?;
            let step = base.slot_alts[slot][alt as usize];
            let k = derive_child(&secp, &self.job.sources[*src].accounts[*path], step, idx)?;
            let bytes = if tap {
                k.inner.x_only_public_key().0.serialize().to_vec()
            } else {
                k.inner.serialize().to_vec()
            };
            keyed.push((bytes, a.clone()));
        }
        keyed.sort_by(|x, y| x.0.cmp(&y.0));
        let mut nargs = vec![args[0].clone()];
        nargs.extend(keyed.into_iter().map(|x| x.1));
        let nb = b.replacen("sortedmulti", "multi", 1);
        Some(Expr::Node {
            name: format!("{prefix}{nb}"),
            args: nargs,
        })
    }

    /// The first three (alt, idx) where two wallets differ (§5).
    fn distinguishing(&self, a: &Wallet, b: &Wallet) -> Vec<(u32, u32, String, String)> {
        let (Ok(ta), Ok(tb)) = (Template::parse(&a.tpl_text), Template::parse(&b.tpl_text)) else {
            return Vec::new();
        };
        let net = self.ctx.network.to_bitcoin_network();
        let render = |s: &ScriptBuf| {
            Address::from_script(s, net)
                .map(|x| x.to_string())
                .unwrap_or_else(|_| s.to_hex_string())
        };
        let mut out = Vec::new();
        let alts = self.alts(&ta);
        let gap = self.ctx.args.gap_limit.min(1000);
        let probe = |alt: usize, range: std::ops::Range<u32>, out: &mut Vec<_>| {
            for i in range {
                if out.len() >= 3 {
                    return;
                }
                let (Some(x), Some(y)) = (
                    self.spk(&ta, &a.assign, alt, i),
                    self.spk(&tb, &b.assign, alt, i),
                ) else {
                    continue;
                };
                if x != y {
                    out.push((alt as u32, i, render(&x), render(&y)));
                }
            }
        };
        probe(0, 0..gap, &mut out);
        for alt in 1..alts {
            probe(alt, 0..gap, &mut out);
        }
        if out.len() < 3 {
            probe(0, gap..1000, &mut out);
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn report<W: Write, E: Write>(
        &self,
        found: Vec<Wallet>,
        matches_total: u64,
        matches_kept: usize,
        capped: bool,
        total_scripts: u64,
        stdout: &mut W,
        stderr: &mut E,
    ) -> Result<u8, ToolkitError> {
        let ctx = self.ctx;
        let job = self.job;
        let ntargets = ctx.targets.len();
        let all: BTreeSet<usize> = (0..ntargets).collect();
        let union: BTreeSet<usize> = found.iter().flat_map(|w| w.target_set()).collect();

        let mut reasons: BTreeSet<usize> = BTreeSet::new(); // index into REASONS
        const REASONS: [&str; 6] = [
            "twin",
            "split",
            "unmatched",
            "path_unrecordable",
            "origin_unknown",
            "cap",
        ];
        // Groups: per seed in single-sig, one group otherwise.
        let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (i, w) in found.iter().enumerate() {
            groups.entry(w.group).or_default().push(i);
        }
        let mut full_pairs: Vec<(usize, usize)> = Vec::new();
        for idxs in groups.values() {
            if idxs.len() > 1 {
                reasons.insert(1);
                let gu: BTreeSet<usize> =
                    idxs.iter().flat_map(|i| found[*i].target_set()).collect();
                let full: Vec<usize> = idxs
                    .iter()
                    .copied()
                    .filter(|i| found[*i].target_set() == gu)
                    .collect();
                if full.len() >= 2 {
                    full_pairs.push((full[0], full[1]));
                }
            }
        }
        if union != all {
            reasons.insert(2);
        }
        if capped {
            reasons.insert(5);
        }

        // Twins.
        let mut twin_notes: Vec<String> = vec![String::new(); found.len()];
        let mut twins: Vec<(usize, Twin)> = Vec::new();
        if !job.single_sig {
            let mut seen: BTreeSet<String> = BTreeSet::new();
            for (i, w) in found.iter().enumerate() {
                let (t, note) = self.twin(w);
                twin_notes[i] = note;
                if let Some(t) = t {
                    reasons.insert(0);
                    let key = canonical_label(&t.wallet.tpl_text, &t.wallet.assign);
                    if seen.insert(key) {
                        twins.push((i, t));
                    }
                }
            }
        }
        let max_found = found
            .iter()
            .map(|w| w.target_set().len())
            .max()
            .unwrap_or(0);
        // Order: twins covering more than any found wallet first, then each
        // found wallet followed by its twin.
        let mut order: Vec<(&'static str, Wallet, Option<usize>)> = Vec::new(); // role, wallet, found index
        let mut super_twin_text: Option<&'static str> = None;
        for (fi, t) in &twins {
            if t.covers.len() > max_found {
                super_twin_text = Some(if t.covers == all {
                    "the sorted twin of your template matches all targets"
                } else {
                    "a twin of your template matches more targets than any searched candidate"
                });
                order.push(("twin", t.wallet.clone(), Some(*fi)));
            }
        }
        for (fi, w) in found.iter().enumerate() {
            order.push(("found", w.clone(), Some(fi)));
            for (ti, t) in &twins {
                if *ti == fi && t.covers.len() <= max_found {
                    order.push(("twin", t.wallet.clone(), Some(fi)));
                }
            }
        }
        // Positions of found wallets in the output.
        let pos_of_found: BTreeMap<usize, usize> = order
            .iter()
            .enumerate()
            .filter(|(_, (r, _, _))| *r == "found")
            .map(|(p, (_, _, fi))| (fi.unwrap_or(0), p))
            .collect();

        // Render wallets.
        let mut wallets_json: Vec<WalletJson> = Vec::new();
        for (role, w, fi) in &order {
            let mut wj = self.wallet_json(w, role)?;
            if *role == "twin" {
                wj.twin_of = fi.and_then(|f| pos_of_found.get(&f).copied());
            }
            if let Some(why) = wj.bundle_command_unavailable {
                reasons.insert(if why == "path_unrecordable" { 3 } else { 4 });
            }
            if wj.slots.iter().any(|s| !s.origin_known) {
                reasons.insert(4);
            }
            wallets_json.push(wj);
        }
        // Distinguishing for each (found, twin) pair, and the first two full
        // wallets of a split.
        let mut pairs: Vec<(usize, usize)> = Vec::new();
        for (p, (role, _, fi)) in order.iter().enumerate() {
            if *role == "twin" {
                if let Some(fp) = fi.and_then(|f| pos_of_found.get(&f).copied()) {
                    pairs.push((fp, p));
                }
            }
        }
        for (a, b) in &full_pairs {
            if let (Some(pa), Some(pb)) = (pos_of_found.get(a), pos_of_found.get(b)) {
                pairs.push((*pa, *pb));
            }
        }
        let mut diff_text: Vec<DiffPair> = Vec::new();
        for (a, b) in &pairs {
            let d = self.distinguishing(&order[*a].1, &order[*b].1);
            for (x, y) in [(*a, *b), (*b, *a)] {
                if wallets_json[x].distinguishing.is_empty() {
                    wallets_json[x].distinguishing = d
                        .iter()
                        .map(|(alt, idx, ax, bx)| {
                            let (ax, bx) = if x == *a { (ax, bx) } else { (bx, ax) };
                            let mut m = BTreeMap::new();
                            m.insert(x.to_string(), ax.clone());
                            m.insert(y.to_string(), bx.clone());
                            DistinguishingJson {
                                alternative: *alt,
                                index: *idx,
                                addresses: m,
                            }
                        })
                        .collect();
                }
            }
            diff_text.push((*a, *b, d));
        }

        // Unmatched.
        let searched_kinds: BTreeSet<SpkKind> = job
            .units
            .iter()
            .map(|u| job.templates[u.tpl_idx].kind)
            .collect();
        let unmatched: Vec<UnmatchedJson> = ctx
            .targets
            .iter()
            .enumerate()
            .filter(|(i, _)| !union.contains(i))
            .map(|(_, t)| UnmatchedJson {
                address: t.0.clone(),
                reason: if searched_kinds.contains(&t.2) {
                    "not_found"
                } else {
                    "type_not_searched"
                },
            })
            .collect();
        let seeds_without_match: Vec<usize> = if job.single_sig && job.templates.len() > 1 {
            (0..ctx.seeds.len())
                .filter(|s| !groups.contains_key(s))
                .collect()
        } else {
            Vec::new()
        };

        // SPEC §6: `reasons` is empty for `no_match` (review r1 M1).
        if found.is_empty() {
            reasons.clear();
        }
        let reasons_v: Vec<&'static str> = reasons.iter().map(|i| REASONS[*i]).collect();
        let result = if found.is_empty() {
            "no_match"
        } else if reasons_v.is_empty() {
            "match"
        } else {
            "unverified"
        };
        let mut script_types: Vec<&'static str> = job
            .units
            .iter()
            .map(|u| job.templates[u.tpl_idx].script_type)
            .collect();
        script_types.dedup();
        let alts_max = job
            .templates
            .iter()
            .map(|t| alts_used(ctx, t))
            .max()
            .unwrap_or(1);
        let searched = SearchedJson {
            mode: job.mode,
            paths: if job.mode == "keyed" {
                job.keyed_derivations
            } else {
                job.std_paths
            },
            child_keys: self.cache.len(),
            candidate_scripts: if job.single_sig {
                None
            } else {
                Some(total_scripts)
            },
            min_account: ctx.args.min_account,
            end_account: job.end_account,
            add_paths: job.add_paths.clone(),
            refugium_seats: ctx.args.refugium_seats,
            gap_limit: ctx.args.gap_limit,
            alternatives: (0..alts_max as u32).collect(),
            script_types,
            slot_orders: if job.mode == "keyed" {
                None
            } else if job.single_sig {
                Some(1)
            } else {
                Some(job.units[0].space.max_orders())
            },
            independent_paths: ctx.args.independent_paths,
            passphrase_supplied: ctx.passphrase_supplied,
            matches_total,
            matches_kept,
        };
        // The shared no-match error labels its count "paths searched".
        let searched_paths = searched.paths;

        if ctx.args.json {
            let env = XpubSearchEnvelope {
                schema_version: "1",
                body: XpubSearchJson::AddressOfSeed(AddressOfSeedResult {
                    result,
                    reasons: reasons_v.clone(),
                    wallets: wallets_json.clone(),
                    unmatched_targets: unmatched,
                    seeds_without_match,
                    searched,
                }),
            };
            let body = serde_json::to_string(&env)
                .map_err(|e| ToolkitError::BadInput(format!("address-of-seed JSON: {e}")))?;
            writeln!(stdout, "{body}").map_err(ToolkitError::Io)?;
        } else {
            self.text_report(
                &order,
                &wallets_json,
                &twin_notes,
                &diff_text,
                &unmatched,
                &seeds_without_match,
                &searched,
                super_twin_text,
                &reasons_v,
                stdout,
                stderr,
            )?;
        }
        match result {
            "match" => Ok(0),
            "no_match" => Err(ToolkitError::XpubSearchNoMatch {
                mode: "address-of-seed",
                searched: searched_paths,
            }),
            _ => Err(ToolkitError::XpubSearchUnverified { reasons: reasons_v }),
        }
    }

    fn wallet_json(&self, w: &Wallet, role: &'static str) -> Result<WalletJson, ToolkitError> {
        let ctx = self.ctx;
        let job = self.job;
        let tpl = Template::parse(&w.tpl_text).map_err(refuse)?;
        let mut slots = Vec::new();
        for (slot, (src, path)) in w.assign.iter().enumerate() {
            let s = &job.sources[*src];
            let pi = &s.paths[*path];
            let origin_known = s.cosigner.as_ref().is_none_or(|c| c.path.is_some());
            slots.push(SlotJson {
                slot,
                source: if s.seed.is_some() { "seed" } else { "cosigner" },
                seed_index: s.seed,
                master_fingerprint: s.fingerprint.map(|f| f.to_string()),
                stated_fingerprint: s.stated_fp.map(|f| f.to_string()),
                origin_path: if origin_known {
                    Some(path_str(&pi.path))
                } else {
                    None
                },
                origin_known,
                template: pi.template.clone(),
                account: pi.account,
                refugium_seat: pi.seat.map(|x| x.0),
                refugium_round: pi.seat.map(|x| x.1),
            });
        }
        let targets = w
            .hits
            .iter()
            .map(|(t, alt, idx)| TargetJson {
                address: ctx.targets[*t].0.clone(),
                alternative: *alt,
                step: tpl.slot_alts[0][*alt as usize],
                index: *idx,
                paths: w
                    .assign
                    .iter()
                    .enumerate()
                    .map(|(slot, (src, path))| {
                        let s = &job.sources[*src];
                        let known = s.cosigner.as_ref().is_none_or(|c| c.path.is_some());
                        SlotPathJson {
                            slot,
                            path: known.then(|| {
                                child_path(
                                    &s.paths[*path].path,
                                    tpl.slot_alts[slot][*alt as usize],
                                    *idx,
                                )
                            }),
                        }
                    })
                    .collect(),
            })
            .collect();
        let bc = self.bundle_command(w, &tpl);
        Ok(WalletJson {
            role,
            twin_of: None,
            from_user_template: w.from_user,
            descriptor: bc.descriptor,
            script_type: w.script_type,
            merged_forms: w.merged_forms,
            slots,
            targets,
            distinguishing: Vec::new(),
            bundle_command: bc.command,
            bundle_command_redacted: bc.redacted,
            bundle_command_unavailable: bc.unavailable,
            redaction_note: bc.note,
        })
    }

    fn bundle_command(&self, w: &Wallet, tpl: &Template) -> BundleCmd {
        let ctx = self.ctx;
        let job = self.job;
        let mut unavailable: Option<&'static str> = None;
        let mut placeholder = false;
        // Paths recordable by md1? A seed at the master `m` is not either:
        // `bundle` cannot record the master as a key origin (SPEC §1 "not
        // covered"; review r1 M2), so it is reported like a step past 2^29.
        for (src, path) in &w.assign {
            let p = &job.sources[*src].paths[*path].path;
            if job.sources[*src].seed.is_some() && p.is_empty() {
                unavailable = Some("path_unrecordable");
            }
            if p.into_iter().any(|c| match c {
                ChildNumber::Hardened { index } | ChildNumber::Normal { index } => {
                    *index > MD1_STEP_MAX
                }
            }) {
                unavailable = Some("path_unrecordable");
            }
        }
        // Descriptor with inline origins for seed slots.
        let assign = &w.assign;
        let desc = match Expr::parse(&w.tpl_text) {
            Ok(e) => e
                .map_leaves(&|leaf| match leaf_slot(leaf) {
                    Some(i) if i < assign.len() => {
                        let (src, path) = assign[i];
                        let s = &job.sources[src];
                        let us = leaf_use_site(leaf);
                        match s.fingerprint.filter(|_| s.seed.is_some()) {
                            Some(fp) if s.paths[path].path.is_empty() => format!("[{fp}]@{i}{us}"),
                            Some(fp) => {
                                format!("[{fp}/{}]@{i}{us}", path_h(&s.paths[path].path))
                            }
                            None => format!("@{i}{us}"),
                        }
                    }
                    _ => leaf.to_string(),
                })
                .render(),
            Err(_) => w.tpl_text.clone(),
        };
        let mut argv: Vec<String> = vec![
            "mnemonic".into(),
            "bundle".into(),
            "--network".into(),
            network_name(ctx.network),
            "--descriptor".into(),
            desc.clone(),
        ];
        if ctx.args.language != CliLanguage::English {
            if let Some(v) = ctx.args.language.to_possible_value() {
                argv.push("--language".into());
                argv.push(v.get_name().to_string());
            }
        }
        // References: stdin goes to the first inline value only if nothing
        // else in the printed command reads stdin.
        let seeds_used: Vec<usize> = {
            let mut v: Vec<usize> = assign
                .iter()
                .filter_map(|(s, _)| job.sources[*s].seed)
                .collect();
            v.sort_unstable();
            v
        };
        let mut stdin_taken = seeds_used
            .iter()
            .any(|i| ctx.seeds[*i].reference == SecretRef::Stdin)
            || ctx.passphrase_ref == Some(SecretRef::Stdin);
        let mut converted: Vec<String> = Vec::new();
        let mut seed_ref: BTreeMap<usize, String> = BTreeMap::new();
        for i in &seeds_used {
            let r = match &ctx.seeds[*i].reference {
                SecretRef::Stdin => "-".to_string(),
                SecretRef::Env(v) => format!("@env:{v}"),
                SecretRef::Inline => {
                    if !stdin_taken {
                        stdin_taken = true;
                        converted.push(format!("seed {i} on stdin (`-`)"));
                        "-".into()
                    } else {
                        converted.push(format!("seed {i} in $MNEMONIC_SEED_{i}"));
                        format!("@env:MNEMONIC_SEED_{i}")
                    }
                }
            };
            seed_ref.insert(*i, r);
        }
        let pp_ref = match &ctx.passphrase_ref {
            None => None,
            Some(SecretRef::Stdin) => Some("-".to_string()),
            Some(SecretRef::Env(v)) => Some(format!("@env:{v}")),
            Some(SecretRef::Inline) => {
                if !stdin_taken {
                    converted.push("the passphrase on stdin (`-`)".into());
                    Some("-".into())
                } else {
                    converted.push("the passphrase in $MNEMONIC_PASSPHRASE".into());
                    Some("@env:MNEMONIC_PASSPHRASE".into())
                }
            }
        };
        for (slot, (src, _)) in assign.iter().enumerate() {
            let s = &job.sources[*src];
            if let Some(i) = s.seed {
                argv.push("--slot".into());
                argv.push(format!("@{slot}.{}={}", ctx.seeds[i].subkey, seed_ref[&i]));
            } else if let Some(c) = &s.cosigner {
                argv.push("--slot".into());
                argv.push(format!("@{slot}.xpub={}", c.xpub));
                if let Some(fp) = c.fp {
                    argv.push("--slot".into());
                    argv.push(format!("@{slot}.fingerprint={fp}"));
                }
                match &c.path {
                    Some(p) => {
                        argv.push("--slot".into());
                        argv.push(format!("@{slot}.path=m/{}", path_h(p)));
                    }
                    None => {
                        // SPEC §6 bare cosigner: the canonical origin when the
                        // xpub's own header fits it, else a placeholder.
                        let canon = canonical_origin(&tpl.text, ctx.network);
                        let v: Vec<ChildNumber> = canon.into_iter().copied().collect();
                        let fits = usize::from(c.xpub.depth) == v.len()
                            && v.last() == Some(&c.xpub.child_number);
                        argv.push("--slot".into());
                        if fits {
                            argv.push(format!("@{slot}.path=m/{}", path_h(&canon)));
                        } else {
                            argv.push(format!("@{slot}.path=<origin of this xpub>"));
                            placeholder = true;
                        }
                        if unavailable.is_none() {
                            unavailable = if fits { None } else { Some("origin_unknown") };
                        }
                    }
                }
            }
        }
        if let Some(r) = pp_ref {
            argv.push("--passphrase".into());
            argv.push(r);
        }
        let (quoted, redacted) = crate::cmd::bundle::redact_and_quote(&argv, &[], None);
        debug_assert!(!redacted, "the bundle argv is built from references only");
        let note = if converted.is_empty() {
            None
        } else {
            Some(format!(
                "supply {} when you run this command; never type a seed or passphrase on the \
                 command line (the argv guard refuses it)",
                converted.join(", ")
            ))
        };
        let command = match unavailable {
            Some(_) => None,
            None => Some(quoted.clone()),
        };
        BundleCmd {
            descriptor: desc,
            command,
            placeholder_command: if placeholder { Some(quoted) } else { None },
            redacted,
            unavailable,
            note,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn text_report<W: Write, E: Write>(
        &self,
        order: &[(&'static str, Wallet, Option<usize>)],
        wallets: &[WalletJson],
        twin_notes: &[String],
        diffs: &[DiffPair],
        unmatched: &[UnmatchedJson],
        seeds_without_match: &[usize],
        searched: &SearchedJson,
        super_twin: Option<&'static str>,
        reasons: &[&'static str],
        out: &mut W,
        err: &mut E,
    ) -> Result<(), ToolkitError> {
        let io = ToolkitError::Io;
        let ctx = self.ctx;
        let job = self.job;
        if let Some(s) = super_twin {
            writeln!(out, "unverified: {s}").map_err(io)?;
        }
        for (p, ((role, w, fi), wj)) in order.iter().zip(wallets).enumerate() {
            let tpl = Template::parse(&w.tpl_text).map_err(refuse)?;
            let tag = if *role == "twin" {
                // Numbered by output position, as the "unverified" lines are
                // (review r1 M8).
                let of = fi
                    .and_then(|f| {
                        order
                            .iter()
                            .position(|(r, _, x)| *r == "found" && *x == Some(f))
                    })
                    .map_or(0, |pos| pos + 1);
                format!("twin (of wallet {of}): ")
            } else {
                String::new()
            };
            if tpl.n == 1 {
                let s = &wj.slots[0];
                for t in &wj.targets {
                    writeln!(out, "{tag}match: {}", t.address).map_err(io)?;
                    writeln!(
                        out,
                        "  path:        {}",
                        t.paths[0].path.as_deref().unwrap_or("–")
                    )
                    .map_err(io)?;
                    writeln!(
                        out,
                        "  chain:       alternative {} ({})      index: {}",
                        t.alternative,
                        step_label(t.step, &tpl.slot_alts[0]),
                        t.index
                    )
                    .map_err(io)?;
                }
                let (src, path) = w.assign[0];
                let pi = &job.sources[src].paths[path];
                let acct = match pi.account {
                    Some(a) => format!("{a}   ({}, {})", pi.label, path_str(&pi.path)),
                    None => format!("–   ({}, {})", pi.label, path_str(&pi.path)),
                };
                writeln!(out, "  account:     {acct}").map_err(io)?;
                writeln!(out, "  script type: {}", wj.script_type).map_err(io)?;
                writeln!(out, "  slots:       @0 = {}", slot_who(s)).map_err(io)?;
                if let Some(n) = nonstandard_note(&pi.template, &pi.path, wj.script_type) {
                    writeln!(out, "  note: {n}").map_err(io)?;
                }
            } else {
                writeln!(
                    out,
                    "{tag}{}: {}   script type: {}",
                    if *role == "twin" { "wallet" } else { "match" },
                    w.tpl_text,
                    wj.script_type
                )
                .map_err(io)?;
                for s in &wj.slots {
                    let (src, path) = w.assign[s.slot];
                    let pi = &job.sources[src].paths[path];
                    let where_ = if s.origin_known {
                        path_str(&pi.path)
                    } else {
                        "origin not given".into()
                    };
                    let acct = match pi.account {
                        Some(a) => format!("  account {a} ({})", pi.label),
                        None => format!("  ({})", pi.label),
                    };
                    writeln!(
                        out,
                        "  slot @{}:     {}  {where_}{acct}",
                        s.slot,
                        slot_who(s)
                    )
                    .map_err(io)?;
                }
                for t in &wj.targets {
                    writeln!(out, "  target:      {}", t.address).map_err(io)?;
                    writeln!(
                        out,
                        "               alternative {} ({})  index {}",
                        t.alternative,
                        step_label(t.step, &tpl.slot_alts[0]),
                        t.index
                    )
                    .map_err(io)?;
                    let ps: Vec<String> = t
                        .paths
                        .iter()
                        .map(|x| match &x.path {
                            Some(p) => format!("@{} {p}", x.slot),
                            None => {
                                let rest = child_path(
                                    &DerivationPath::master(),
                                    tpl.slot_alts[x.slot][t.alternative as usize],
                                    t.index,
                                );
                                format!(
                                    "@{} <origin unknown>{}",
                                    x.slot,
                                    rest.trim_start_matches('m')
                                )
                            }
                        })
                        .collect();
                    writeln!(out, "               {}", ps.join("   ")).map_err(io)?;
                }
                if *role == "found" {
                    if let Some(f) = fi {
                        writeln!(out, "  twin:        {}", twin_notes[*f]).map_err(io)?;
                    }
                }
            }
            if wj.merged_forms > 0 {
                writeln!(
                    out,
                    "  note: {} other form(s) give the same addresses (a different wallet-policy \
                     id); the command uses the template as you gave it",
                    wj.merged_forms
                )
                .map_err(io)?;
            }
            let _ = p;
            match (&wj.bundle_command, wj.bundle_command_unavailable) {
                (Some(c), _) => {
                    let assumed = wj.slots.iter().any(|s| !s.origin_known);
                    if assumed {
                        writeln!(
                            out,
                            "  note: the cosigner path is assumed: the canonical origin for this \
                             shape; your xpub carries no origin, and its header shows only its \
                             depth and last step, not the account (an xpub at account 1 fits \
                             too): check the path against your records"
                        )
                        .map_err(io)?;
                    }
                    writeln!(out, "to mint this wallet, run:").map_err(io)?;
                    writeln!(out, "  {c}").map_err(io)?;
                    if let Some(n) = &wj.redaction_note {
                        writeln!(err, "note: {n}").map_err(io)?;
                    }
                }
                (None, Some("path_unrecordable")) => {
                    let at_master = w.assign.iter().any(|(src, path)| {
                        job.sources[*src].seed.is_some()
                            && job.sources[*src].paths[*path].path.is_empty()
                    });
                    writeln!(
                        out,
                        "  {}; no card can be made for this wallet",
                        if at_master {
                            "the master key m cannot be recorded as a key origin"
                        } else {
                            "md1 cannot record a path step above 536870911"
                        }
                    )
                    .map_err(io)?;
                }
                (None, _) => {
                    let cmd = self.bundle_command(w, &tpl).placeholder_command;
                    writeln!(
                        out,
                        "  the cosigner's origin is not known and its header does not fit the \
                         canonical origin; only your records can fill it, and `bundle` refuses \
                         the command until they do:"
                    )
                    .map_err(io)?;
                    if let Some(c) = cmd {
                        writeln!(out, "  {c}").map_err(io)?;
                    }
                }
            }
        }
        for (a, b, d) in diffs {
            let wa = &wallets[*a];
            let wb = &wallets[*b];
            let tgt: Vec<String> = wa.targets.iter().map(|t| short(&t.address)).collect();
            writeln!(out, "unverified: two wallets give {}", tgt.join(", ")).map_err(io)?;
            let label = |w: &WalletJson| {
                if w.from_user_template && w.role == "found" {
                    "your template"
                } else if w.role == "twin" {
                    "its twin"
                } else {
                    "another order"
                }
            };
            writeln!(out, "  wallet {} ({}): {}", a + 1, label(wa), wa.descriptor).map_err(io)?;
            writeln!(
                out,
                "  wallet {} ({}):      {}",
                b + 1,
                label(wb),
                wb.descriptor
            )
            .map_err(io)?;
            writeln!(out, "  they differ at:").map_err(io)?;
            let tpl = Template::parse(&order[*a].1.tpl_text).map_err(refuse)?;
            for (alt, idx, x, y) in d {
                let name = match tpl.slot_alts[0][*alt as usize] {
                    Some(0) if tpl.slot_alts[0] == [Some(0), Some(1)] => format!("receive {idx}"),
                    Some(1) if tpl.slot_alts[0] == [Some(0), Some(1)] => format!("change {idx}"),
                    _ => format!("alternative {alt} index {idx}"),
                };
                writeln!(out, "    {name}: wallet {} {x}", a + 1).map_err(io)?;
                writeln!(out, "    {}  wallet {} {y}", " ".repeat(name.len()), b + 1)
                    .map_err(io)?;
            }
            writeln!(
                out,
                "  whichever of these your wallet shows decides it; or rerun with that address as a \
                 second --target-address."
            )
            .map_err(io)?;
        }
        for u in unmatched {
            writeln!(
                out,
                "unmatched: {} ({})",
                u.address,
                if u.reason == "type_not_searched" {
                    "no searched script produces this output type"
                } else {
                    "not found in the searched set"
                }
            )
            .map_err(io)?;
        }
        for s in seeds_without_match {
            writeln!(out, "seed {s}: no target found").map_err(io)?;
        }
        if !reasons.is_empty() && !wallets.is_empty() {
            writeln!(
                out,
                "unverified ({}): verify before you rely on it",
                reasons.join(", ")
            )
            .map_err(io)?;
        }
        // Searched set.
        let n_wallets = wallets.iter().filter(|w| w.role == "found").count();
        if wallets.is_empty() {
            writeln!(out, "no match.").map_err(io)?;
        }
        let cands = match searched.candidate_scripts {
            Some(c) => format!(", {c} candidate scripts"),
            None => String::new(),
        };
        writeln!(
            out,
            "searched: {} child keys on {} paths{cands}, index window [0, {}); {} wallet(s) match",
            searched.child_keys, searched.paths, searched.gap_limit, n_wallets
        )
        .map_err(io)?;
        if wallets.is_empty() {
            self.no_match_detail(searched, out)?;
        }
        let _ = ctx;
        Ok(())
    }

    fn no_match_detail<W: Write>(&self, s: &SearchedJson, out: &mut W) -> Result<(), ToolkitError> {
        let io = ToolkitError::Io;
        let ctx = self.ctx;
        writeln!(
            out,
            "  path families: bip44, bip49, bip84, bip86, bip48 (script types 1', 2', 3'), bip87 \
             for accounts [{}, {}); m/45' (the bundle bare-sh default, not BIP-45 cosigner \
             branches)",
            s.min_account, s.end_account
        )
        .map_err(io)?;
        if !s.add_paths.is_empty() {
            writeln!(out, "  --add-path: {}", s.add_paths.join(", ")).map_err(io)?;
        }
        writeln!(
            out,
            "  Refugium seats: {}; script types: {}; alternatives: {:?}; index window [0, {}); \
             slot orders: {}; paths: {}; passphrase: {}",
            if s.refugium_seats {
                "included"
            } else {
                "not included"
            },
            s.script_types.join(", "),
            s.alternatives,
            s.gap_limit,
            s.slot_orders
                .map(|x| x.to_string())
                .unwrap_or_else(|| "–".into()),
            if s.independent_paths {
                "independent"
            } else {
                "shared"
            },
            if s.passphrase_supplied {
                "given"
            } else {
                "not given"
            }
        )
        .map_err(io)?;
        if self.job.md1_pinned {
            writeln!(
                out,
                "hint: the paths came from the md1 card and may be `bundle`'s defaults; pass the \
                 template as text without origins to search the full path set"
            )
            .map_err(io)?;
        }
        writeln!(
            out,
            "hint: widen with --number-of-accounts / --max-account, --gap-limit, --add-path, \
             --refugium-seats, then --independent-paths (narrow the accounts first)"
        )
        .map_err(io)?;
        writeln!(
            out,
            "hint: common paths outside the set: --add-path \"m/0'\" (BIP-32 default account), \
             --add-path \"m/84'/{c}'/2147483644'\" to 2147483646' (Samourai; a match there is \
             reported but cannot be put on a card)",
            c = ctx.network.coin_type()
        )
        .map_err(io)?;
        if !ctx.passphrase_supplied {
            writeln!(
                out,
                "hint: a passphrase-protected wallet needs --passphrase; `xpub-search \
                 passphrase-of-xpub` tests candidates against a known xpub"
            )
            .map_err(io)?;
        }
        if ctx
            .targets
            .iter()
            .any(|t| matches!(t.2, SpkKind::P2sh | SpkKind::P2tr))
        {
            writeln!(
                out,
                "hint: a 3… or bc1p… address may be a script the search was not given: a taproot \
                 script wallet, or a taproot multisig whose internal key is not the toolkit's \
                 NUMS (BIP-341's H + rG), is found only with its exact descriptor"
            )
            .map_err(io)?;
        }
        Ok(())
    }
}

/// (template, group, assignment) → matched (target, alt, idx).
type RawWallets = BTreeMap<(usize, usize, Vec<(usize, usize)>), BTreeSet<(usize, u32, u32)>>;
/// Two wallets' output positions and their first differing (alt, idx, a, b).
type DiffPair = (usize, usize, Vec<(u32, u32, String, String)>);

struct BundleCmd {
    descriptor: String,
    command: Option<String>,
    placeholder_command: Option<String>,
    redacted: bool,
    unavailable: Option<&'static str>,
    note: Option<String>,
}

fn slot_who(s: &SlotJson) -> String {
    match (s.seed_index, &s.master_fingerprint) {
        (Some(i), Some(fp)) => format!("seed {i} ({fp})"),
        (_, Some(fp)) => format!("cosigner ({fp})"),
        _ => "cosigner".into(),
    }
}

fn short(a: &str) -> String {
    if a.len() > 20 {
        format!("{}…{}", &a[..10], &a[a.len() - 6..])
    } else {
        a.to_string()
    }
}

/// `bundle`'s canonical origin for the template's shape (account 0).
fn canonical_origin(text: &str, network: CliNetwork) -> DerivationPath {
    use crate::template::DefaultOriginFamily as F;
    let t = text.trim();
    let fam = if t.starts_with("sh(wsh(") {
        F::Bip48 { script_type: 1 }
    } else if t.starts_with("sh(") || t.starts_with("tr(") {
        F::Bip87
    } else {
        F::Bip48 { script_type: 2 }
    };
    fam.derivation_path(network, 0)
}

fn nonstandard_note(template: &str, path: &DerivationPath, script_type: &str) -> Option<String> {
    let std = match template {
        "bip44" => "p2pkh",
        "bip49" => "p2sh-p2wpkh",
        "bip84" => "p2wpkh",
        "bip86" => "p2tr",
        "bip48" | "bip87" => "",
        _ => return None,
    };
    if std == script_type {
        return None;
    }
    let num = match path.into_iter().next() {
        Some(ChildNumber::Hardened { index }) => *index,
        _ => return None,
    };
    Some(format!(
        "{script_type} at a BIP-{num} path is not what BIP-{num} describes"
    ))
}
