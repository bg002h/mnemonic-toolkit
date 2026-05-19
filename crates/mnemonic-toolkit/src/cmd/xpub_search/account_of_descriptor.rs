//! `mnemonic xpub-search account-of-descriptor` — P2 mode.
//!
//! SPEC: plan §4 (P2 account-of-descriptor).
//! - Seed-intake mutex (reused from P1): --phrase / --phrase-stdin / --ms1 /
//!   --ms1-stdin / positional (ms1 HRP only).
//! - Descriptor-intake polymorphism (NET-NEW): literal-xpub / md1 / BIP-388
//!   JSON shapes via auto-detect; explicit `--descriptor-from <node>=<value>`
//!   override; toolkit-@N refused.
//! - Per-cosigner search over the candidate path set; collect all matches.
//! - JSON envelope: `{"schema_version":"1","mode":"account-of-descriptor", ...}`
//!   with `matched_cosigners` array.

use super::account_search::match_descriptor_against_seed;
use super::candidate_paths::build_candidate_paths;
use super::descriptor_intake::{
    intake_from_descriptor_value, intake_from_explicit_form, DescriptorShape,
};
use super::seed_intake::{resolve_seed, SeedIntakeArgs};
use super::{XpubSearchEnvelope, XpubSearchJson};
use crate::derive_slot::derive_master_seed;
use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::secret_advisory::secret_in_argv_warning;
use bitcoin::bip32::Xpriv;
use clap::Args;
use serde::Serialize;
use std::io::{Read, Write};

#[derive(Args, Debug)]
pub struct AccountOfDescriptorArgs {
    /// Master BIP-39 phrase (inline). Emits an argv-leakage advisory; prefer
    /// --phrase-stdin for sensitive input.
    #[arg(
        long,
        value_name = "PHRASE",
        conflicts_with_all = ["phrase_stdin", "ms1", "ms1_stdin"],
    )]
    pub phrase: Option<String>,

    /// Read master BIP-39 phrase from stdin.
    #[arg(
        long,
        conflicts_with_all = ["phrase", "ms1", "ms1_stdin"],
    )]
    pub phrase_stdin: bool,

    /// ms1 card carrying BIP-39 entropy (inline). Emits an argv-leakage
    /// advisory; prefer --ms1-stdin for sensitive input.
    #[arg(
        long,
        value_name = "MS1",
        conflicts_with_all = ["phrase", "phrase_stdin", "ms1_stdin"],
    )]
    pub ms1: Option<String>,

    /// Read ms1 card from stdin (single chunk).
    #[arg(
        long,
        conflicts_with_all = ["phrase", "phrase_stdin", "ms1"],
    )]
    pub ms1_stdin: bool,

    /// BIP-39 passphrase (inline). Emits an argv-leakage advisory.
    #[arg(long, conflicts_with = "passphrase_stdin")]
    pub passphrase: Option<String>,

    /// Read BIP-39 passphrase from stdin (NULL-byte-preserving; single trailing
    /// newline stripped).
    #[arg(long, conflicts_with = "passphrase")]
    pub passphrase_stdin: bool,

    /// Wallet descriptor. Shape is auto-detected (BIP-388 JSON, md1 card,
    /// or literal-xpub descriptor). Toolkit `@N`-placeholder descriptors are
    /// refused (synthetic xpubs are non-searchable). Use
    /// `--descriptor-from <node>=<value>` to disambiguate when auto-detect
    /// picks the wrong shape.
    #[arg(long, value_name = "VALUE", conflicts_with = "descriptor_from")]
    pub descriptor: Option<String>,

    /// Explicit-form descriptor input: `<node>=<value>` where `<node>` is one
    /// of `literal` / `md1` / `bip388`. `<value>` is a literal string, or `-`
    /// to read from stdin (one chunk per line for md1; single string
    /// otherwise).
    #[arg(long, value_name = "NODE=VALUE", conflicts_with = "descriptor")]
    pub descriptor_from: Option<String>,

    /// BIP-39 wordlist language. Defaults to english.
    #[arg(long, default_value = "english")]
    pub language: CliLanguage,

    /// Network selector. Defaults to mainnet.
    #[arg(long, default_value = "mainnet")]
    pub network: CliNetwork,

    /// Lower bound of account-index iteration (inclusive). Default 0.
    #[arg(long, default_value_t = 0)]
    pub min_account: u32,

    /// Window size starting at `--min-account` (default 20).
    #[arg(long, default_value_t = 20)]
    pub number_of_accounts: u32,

    /// Optional upper bound. Effective end is
    /// `max(min_account + number_of_accounts, max_account + 1)`.
    #[arg(long)]
    pub max_account: Option<u32>,

    /// Additional derivation-path template. Repeatable. The literal token
    /// `account'` (or `account`) is substituted with each iterated account
    /// index. Templates without an `account` token are searched once at the
    /// literal path.
    #[arg(long, value_name = "TEMPLATE")]
    pub add_path: Vec<String>,

    /// Emit a JSON envelope on stdout instead of the text-form report.
    #[arg(long)]
    pub json: bool,

    /// Positional ms1 card (HRP-autodetect). BIP-39 phrase text is NOT
    /// accepted positionally (no HRP for autodetect) — use --phrase /
    /// --phrase-stdin.
    #[arg(
        value_name = "MS1",
        num_args = 0..,
        conflicts_with_all = ["phrase", "phrase_stdin", "ms1", "ms1_stdin"],
        required_unless_present_any = ["phrase", "phrase_stdin", "ms1", "ms1_stdin"],
    )]
    pub positional: Vec<String>,
}

impl SeedIntakeArgs for AccountOfDescriptorArgs {
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

/// Per-mode JSON body for account-of-descriptor.
#[derive(Debug, Serialize)]
pub struct AccountOfDescriptorResult {
    /// `"match"` or `"no_match"`.
    pub result: &'static str,
    /// Matched cosigners. Empty array on no-match.
    pub matched_cosigners: Vec<MatchedCosignerJson>,
    /// Total cosigner positions in the descriptor.
    pub cosigners_total: usize,
    /// Number of candidates exhausted per cosigner (templates × accounts +
    /// add-paths).
    pub searched_count_per_cosigner: usize,
    /// Detected descriptor shape: `literal_xpub` / `md1` / `bip388_json`.
    pub descriptor_shape: DescriptorShape,
    /// Cosigners flagged as `unspendable_internal_key: true` (e.g. taproot
    /// NUMS sentinel). Empty array when none.
    pub unspendable_internal_keys: Vec<usize>,
}

#[derive(Debug, Serialize)]
pub struct MatchedCosignerJson {
    pub cosigner_index: usize,
    pub path: String,
    pub template: String,
    pub account: Option<u32>,
}

/// v0.27.1 Phase 5a API-discipline scaffolding for `AccountOfDescriptorResult`.
/// Mirrors `path_of_xpub::build_path_match` discipline; the `matched_cosigners`
/// vec is required non-empty for match and is pinned to `vec![]` on no-match.
/// Tracked by FOLLOWUP
/// `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution`.
pub(super) fn build_account_match(
    matched_cosigners: Vec<MatchedCosignerJson>,
    cosigners_total: usize,
    searched_count_per_cosigner: usize,
    descriptor_shape: DescriptorShape,
    unspendable_internal_keys: Vec<usize>,
) -> AccountOfDescriptorResult {
    AccountOfDescriptorResult {
        result: "match",
        matched_cosigners,
        cosigners_total,
        searched_count_per_cosigner,
        descriptor_shape,
        unspendable_internal_keys,
    }
}

pub(super) fn build_account_no_match(
    cosigners_total: usize,
    searched_count_per_cosigner: usize,
    descriptor_shape: DescriptorShape,
    unspendable_internal_keys: Vec<usize>,
) -> AccountOfDescriptorResult {
    AccountOfDescriptorResult {
        result: "no_match",
        matched_cosigners: Vec::new(),
        cosigners_total,
        searched_count_per_cosigner,
        descriptor_shape,
        unspendable_internal_keys,
    }
}

pub fn run_account_of_descriptor<R: Read, W: Write, E: Write>(
    args: &AccountOfDescriptorArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
) -> Result<u8, ToolkitError> {
    // Mutex: --descriptor xor --descriptor-from.
    if args.descriptor.is_none() && args.descriptor_from.is_none() {
        return Err(ToolkitError::BadInput(
            "supply --descriptor <value> or --descriptor-from <node>=<value>".into(),
        ));
    }
    // For per-mode design, deduce account window first so default-path
    // inference + candidate range are uniformly parameterized.
    let candidates = build_candidate_paths(
        args.min_account,
        args.number_of_accounts,
        args.max_account,
        &args.add_path,
        args.network,
    );
    let searched_count = candidates.len();

    // 1) Resolve descriptor intake. Note: this MUST run before
    //    `resolve_seed` consumes `stdin`, since `--descriptor-from md1=-`
    //    also reads from stdin. Both consuming stdin would be ambiguous;
    //    by convention if `--descriptor-from <node>=-` is the requested
    //    descriptor route, the seed intake cannot also be a stdin form.
    //    (clap's conflict-graph doesn't model this; the user gets a
    //    "stdin already drained" empty-payload error on the second read
    //    if they try to combine them.)
    let descriptor_intake = if let Some(spec) = &args.descriptor_from {
        let (node, value) = spec.split_once('=').ok_or_else(|| {
            ToolkitError::BadInput(format!(
                "--descriptor-from expects <node>=<value> form; got `{spec}`"
            ))
        })?;
        intake_from_explicit_form(
            node,
            value,
            args.network,
            args.min_account,
            stdin,
            stderr,
        )?
    } else {
        let value = args.descriptor.as_deref().expect("checked above");
        intake_from_descriptor_value(value, args.network, args.min_account, stderr)?
    };

    // 2) Resolve seed intake.
    let mnemonic = resolve_seed(args, stdin, stdout, stderr, no_auto_repair)?;

    // 3) Resolve passphrase (mirrors path_of_xpub.rs:178-198).
    let passphrase: zeroize::Zeroizing<String> = if args.passphrase_stdin {
        let mut buf: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(String::new());
        stdin
            .read_to_string(&mut buf)
            .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
        if buf.ends_with('\n') {
            buf.pop();
            if buf.ends_with('\r') {
                buf.pop();
            }
        }
        buf
    } else if let Some(p) = &args.passphrase {
        // v0.26.0 §3 — resolve `@env:<VAR>` sentinel; skip argv-leak advisory
        // when the user routed through the env-var channel.
        if p.starts_with("@env:") {
            let resolved = crate::env_sentinel::resolve_env_var_sentinel(p, "--passphrase")?;
            zeroize::Zeroizing::new(resolved)
        } else {
            secret_in_argv_warning(stderr, "--passphrase", "--passphrase-stdin");
            zeroize::Zeroizing::new(p.clone())
        }
    } else {
        zeroize::Zeroizing::new(String::new())
    };
    let _passphrase_pin = mnemonic_toolkit::mlock::pin_pages_for(passphrase.as_bytes());

    // 4) Derive master xprv.
    let seed = derive_master_seed(&mnemonic, passphrase.as_str());
    let master_xprv = Xpriv::new_master(args.network.network_kind(), &seed[..])
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
    let _seed_pin = mnemonic_toolkit::mlock::pin_pages_for(&seed[..]);

    // 5) Per-cosigner search.
    let matches =
        match_descriptor_against_seed(&master_xprv, &descriptor_intake.cosigners, &candidates);

    let cosigners_total = descriptor_intake.cosigners.len();
    let unspendable: Vec<usize> = descriptor_intake
        .cosigners
        .iter()
        .filter(|c| c.is_nums)
        .map(|c| c.idx)
        .collect();

    // 6) Emit result.
    if !matches.is_empty() {
        if args.json {
            let matched_cosigners = matches
                .iter()
                .map(|m| MatchedCosignerJson {
                    cosigner_index: m.cosigner_index,
                    path: format!("m/{}", m.path),
                    template: m.template.clone(),
                    account: m.account,
                })
                .collect();
            let envelope = XpubSearchEnvelope {
                schema_version: "1",
                body: XpubSearchJson::AccountOfDescriptor(build_account_match(
                    matched_cosigners,
                    cosigners_total,
                    searched_count,
                    descriptor_intake.shape,
                    unspendable.clone(),
                )),
            };
            let body = serde_json::to_string(&envelope).map_err(|e| {
                ToolkitError::BadInput(format!("account-of-descriptor JSON serialize: {e}"))
            })?;
            writeln!(stdout, "{body}").map_err(ToolkitError::Io)?;
        } else {
            for m in &matches {
                writeln!(
                    stdout,
                    "match: cosigner @{}  m/{}  (template={}, account={})",
                    m.cosigner_index,
                    m.path,
                    m.template,
                    m.account
                        .map(|a| a.to_string())
                        .unwrap_or_else(|| "n/a".to_string()),
                )
                .map_err(ToolkitError::Io)?;
            }
            writeln!(
                stdout,
                "cosigners total: {cosigners_total}; matched: {}",
                matches.len()
            )
            .map_err(ToolkitError::Io)?;
            writeln!(
                stdout,
                "searched: {searched_count} candidates × {cosigners_total} cosigners"
            )
            .map_err(ToolkitError::Io)?;
        }
        Ok(0)
    } else {
        if args.json {
            let envelope = XpubSearchEnvelope {
                schema_version: "1",
                body: XpubSearchJson::AccountOfDescriptor(build_account_no_match(
                    cosigners_total,
                    searched_count,
                    descriptor_intake.shape,
                    unspendable,
                )),
            };
            let body = serde_json::to_string(&envelope).map_err(|e| {
                ToolkitError::BadInput(format!("account-of-descriptor JSON serialize: {e}"))
            })?;
            writeln!(stdout, "{body}").map_err(ToolkitError::Io)?;
        }
        Err(ToolkitError::XpubSearchNoMatch {
            mode: "account-of-descriptor",
            searched: searched_count.saturating_mul(cosigners_total.max(1)),
        })
    }
}
