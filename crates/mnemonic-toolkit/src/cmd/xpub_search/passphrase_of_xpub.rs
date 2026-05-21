//! `mnemonic xpub-search passphrase-of-xpub` — P4 mode.
//!
//! SPEC: plan §6 (P4 passphrase-of-xpub).
//! - P4 is **P1 + a fixed mandatory passphrase**. Re-derive master via
//!   `derive_master_seed(mnemonic, passphrase)`, then invoke the same
//!   `match_xpub_against_paths` primitive over the standard BIP-44/49/84/86
//!   single-sig + BIP-48 multisig templates × account range + `--add-path`.
//! - Semantic difference from P1: P1 asks "what path produced this xpub?";
//!   P4 asks "does this specific passphrase produce this xpub (at some
//!   standard path)?". Clap enforces the passphrase group required.
//! - Stderr advisory emitted on every invocation per plan §6.4 (before the
//!   search starts) — points users at `--add-path` / `path-of-xpub` for
//!   non-standard paths.
//! - JSON envelope: `{"schema_version":"1","mode":"passphrase-of-xpub",...}`
//!   — same shape as P1 with `mode` substituted (plan §6.5). Separate
//!   `PassphraseOfXpubResult` struct keeps future divergence clean.
//!
//! P4 deliberately does NOT do (MVP per plan §6.3):
//! - No `--passphrases-file <path>` brute-force.
//! - No streaming candidates from stdin.
//! - No generated passphrase wordlists.
//!
//! Filed as FOLLOWUP `xpub-search-passphrase-bruteforce` for v0.27+.

use super::candidate_paths::build_candidate_paths;
use super::path_search::match_xpub_against_paths;
use super::seed_intake::{resolve_seed, SeedIntakeArgs};
use super::target_intake::resolve_target_xpub;
use super::{XpubSearchEnvelope, XpubSearchJson};
use crate::derive_slot::derive_master_seed;
use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::secret_advisory::secret_in_argv_warning;
use crate::synthesize::xpub_to_65;
use bitcoin::bip32::Xpriv;
use clap::Args;
use serde::Serialize;
use std::io::{Read, Write};

#[derive(Args, Debug)]
pub struct PassphraseOfXpubArgs {
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
    /// **Mandatory** in passphrase-of-xpub (one of --passphrase /
    /// --passphrase-stdin must be supplied; see plan §6.1).
    #[arg(
        long,
        conflicts_with = "passphrase_stdin",
        required_unless_present = "passphrase_stdin",
    )]
    pub passphrase: Option<String>,

    /// Read BIP-39 passphrase from stdin (NULL-byte-preserving; single
    /// trailing newline stripped). **Mandatory** in passphrase-of-xpub.
    #[arg(
        long,
        conflicts_with = "passphrase",
        required_unless_present = "passphrase",
    )]
    pub passphrase_stdin: bool,

    /// Target xpub. Accepts any SLIP-0132 prefix (xpub/tpub/ypub/Ypub/zpub/
    /// Zpub/upub/Upub/vpub/Vpub) or an mk1 bech32 card carrying an xpub.
    #[arg(long, value_name = "XPUB-OR-MK1")]
    pub target_xpub: String,

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

impl SeedIntakeArgs for PassphraseOfXpubArgs {
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

/// Per-mode JSON body for passphrase-of-xpub. Same shape as
/// `PathOfXpubResult` per plan §6.5; kept as a separate type so future
/// divergence stays clean.
///
/// v0.29.0 SemVer-minor wire-shape break: converted from struct to tagged
/// enum. Mirrors `PathOfXpubResult` conversion.
#[derive(Debug, Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum PassphraseOfXpubResult {
    /// Match: all search fields populated.
    Match {
        /// Matched derivation path (e.g. `"m/84'/0'/0'"`).
        path: String,
        /// Matched template name.
        template: String,
        /// Matched account index (None for `--add-path` templates without an
        /// account token).
        account: Option<u32>,
        /// The target xpub after SLIP-0132 normalization.
        target_xpub_canonical: String,
        /// The original SLIP-0132 prefix when alt-prefixed; null otherwise.
        target_xpub_variant: Option<&'static str>,
        /// Count of candidates exhausted (paths × templates × add-paths).
        searched_count: usize,
    },
    /// No match: envelope-scope fields preserved; search fields absent.
    NoMatch {
        /// The target xpub after SLIP-0132 normalization.
        target_xpub_canonical: String,
        /// The original SLIP-0132 prefix when alt-prefixed; null otherwise.
        target_xpub_variant: Option<&'static str>,
        /// Count of candidates exhausted.
        searched_count: usize,
    },
}

/// Construct a `PassphraseOfXpubResult::Match` variant.
pub(super) fn build_passphrase_match(
    path: String,
    template: String,
    account: Option<u32>,
    target_xpub_canonical: String,
    target_xpub_variant: Option<&'static str>,
    searched_count: usize,
) -> PassphraseOfXpubResult {
    PassphraseOfXpubResult::Match {
        path,
        template,
        account,
        target_xpub_canonical,
        target_xpub_variant,
        searched_count,
    }
}

/// Construct a `PassphraseOfXpubResult::NoMatch` variant.
pub(super) fn build_passphrase_no_match(
    target_xpub_canonical: String,
    target_xpub_variant: Option<&'static str>,
    searched_count: usize,
) -> PassphraseOfXpubResult {
    PassphraseOfXpubResult::NoMatch {
        target_xpub_canonical,
        target_xpub_variant,
        searched_count,
    }
}

/// Stderr advisory emitted on every passphrase-of-xpub invocation (plan §6.4).
/// Points users at `--add-path` / `path-of-xpub` for non-standard paths.
const STDERR_ADVISORY: &str = "note: passphrase verification searches the standard \
BIP-44/49/84/86 + BIP-48 templates × account range; if the wallet uses a \
non-standard path, supply --add-path or use `xpub-search path-of-xpub` to \
find the path first.";

pub fn run_passphrase_of_xpub<R: Read, W: Write, E: Write>(
    args: &PassphraseOfXpubArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
) -> Result<u8, ToolkitError> {
    // 0) Emit the unconditional stderr advisory FIRST (plan §6.4 "always
    //    emit … before search starts"). Errors in writeln are non-fatal —
    //    if stderr is unreachable the user already has bigger problems.
    let _ = writeln!(stderr, "{STDERR_ADVISORY}");

    // 1) Resolve seed (mutex + parse + ms1 auto-fire short-circuit).
    let mnemonic = resolve_seed(args, stdin, stdout, stderr, no_auto_repair)?;

    // 2) Resolve mandatory passphrase. Clap enforces "exactly one of
    //    --passphrase / --passphrase-stdin"; we still defensively handle the
    //    fall-through to keep the shape symmetric with path_of_xpub. Inline
    //    emits argv-leak advisory. Wrapped in Zeroizing<String> so the heap
    //    buffer scrubs on drop (plan §6 secret hygiene reuse of §3.6 contract;
    //    mirrors `path_of_xpub.rs:178-198`).
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
        // Defensive: clap's required_unless_present pair makes this
        // unreachable in practice (the `required_unless_present` on both
        // fields forms a mandatory `passphrase | passphrase_stdin` group).
        return Err(ToolkitError::BadInput(
            "passphrase-of-xpub requires --passphrase <VALUE> or --passphrase-stdin".into(),
        ));
    };
    // Pin passphrase heap pages for handler scope.
    let _passphrase_pin = mnemonic_toolkit::mlock::pin_pages_for(passphrase.as_bytes());

    // 3) Resolve target xpub (mk1-or-slip0132 dispatch).
    let (target_xpub, target_variant) = resolve_target_xpub(&args.target_xpub)?;
    let target_xpub_65 = xpub_to_65(&target_xpub);
    let target_xpub_canonical = target_xpub.to_string();

    // 4) Derive master xprv from (mnemonic, passphrase).
    let seed = derive_master_seed(&mnemonic, passphrase.as_str());
    let master_xprv = Xpriv::new_master(args.network.network_kind(), &seed[..])
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;

    // 5) Build candidate paths.
    let candidates = build_candidate_paths(
        args.min_account,
        args.number_of_accounts,
        args.max_account,
        &args.add_path,
        args.network,
    );
    let searched_count = candidates.len();

    // 6) Pin entropy bytes for the candidate-iteration phase.
    let _seed_pin = mnemonic_toolkit::mlock::pin_pages_for(&seed[..]);

    // 7) Search (passphrase-verification: does THIS passphrase produce the
    //    target xpub at some standard path? Same primitive as P1).
    let matched = match_xpub_against_paths(&master_xprv, &candidates, &target_xpub_65);

    // 8) Emit + return.
    match matched {
        Some(m) => {
            if args.json {
                let envelope = XpubSearchEnvelope {
                    schema_version: "1",
                    body: XpubSearchJson::PassphraseOfXpub(build_passphrase_match(
                        format!("m/{}", m.path),
                        m.template_name.clone(),
                        m.account,
                        target_xpub_canonical.clone(),
                        target_variant,
                        searched_count,
                    )),
                };
                let body = serde_json::to_string(&envelope).map_err(|e| {
                    ToolkitError::BadInput(format!("passphrase-of-xpub JSON serialize: {e}"))
                })?;
                writeln!(stdout, "{body}").map_err(ToolkitError::Io)?;
            } else {
                writeln!(
                    stdout,
                    "match: m/{}  (template={}, account={})",
                    m.path,
                    m.template_name,
                    m.account
                        .map(|a| a.to_string())
                        .unwrap_or_else(|| "n/a".to_string()),
                )
                .map_err(ToolkitError::Io)?;
                writeln!(
                    stdout,
                    "target-xpub: {}{}",
                    target_xpub_canonical,
                    match target_variant {
                        Some(v) => format!(" (normalized from {v}; variant={v})"),
                        None => String::new(),
                    },
                )
                .map_err(ToolkitError::Io)?;
                writeln!(stdout, "searched: {searched_count} candidate paths")
                    .map_err(ToolkitError::Io)?;
            }
            Ok(0)
        }
        None => {
            if args.json {
                let envelope = XpubSearchEnvelope {
                    schema_version: "1",
                    body: XpubSearchJson::PassphraseOfXpub(build_passphrase_no_match(
                        target_xpub_canonical.clone(),
                        target_variant,
                        searched_count,
                    )),
                };
                let body = serde_json::to_string(&envelope).map_err(|e| {
                    ToolkitError::BadInput(format!("passphrase-of-xpub JSON serialize: {e}"))
                })?;
                writeln!(stdout, "{body}").map_err(ToolkitError::Io)?;
            }
            Err(ToolkitError::XpubSearchNoMatch {
                mode: "passphrase-of-xpub",
                searched: searched_count,
            })
        }
    }
}
