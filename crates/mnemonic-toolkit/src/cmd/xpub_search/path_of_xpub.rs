//! `mnemonic xpub-search path-of-xpub` — P1 mode.
//!
//! SPEC: plan §3 (P1 path-of-xpub).
//! - Seed-intake mutex: --phrase / --phrase-stdin / --ms1 / --ms1-stdin /
//!   positional (ms1 HRP only).
//! - Target intake: bare SLIP-0132 xpub OR mk1 card.
//! - Candidate iteration: BIP-44/49/84/86 + BIP-48 multisig × account range +
//!   `--add-path` extensions.
//! - First-match-wins.
//! - JSON envelope: `{"schema_version":"1","mode":"path-of-xpub", ...}` with
//!   tag = "mode" + flatten body.

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
pub struct PathOfXpubArgs {
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

    /// Read BIP-39 passphrase from stdin (NULL-byte-preserving; single trailing newline stripped).
    #[arg(long, conflicts_with = "passphrase")]
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

impl SeedIntakeArgs for PathOfXpubArgs {
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

/// Per-mode JSON body for path-of-xpub. Top-level fields are flattened into
/// the `XpubSearchEnvelope` JSON: `schema_version` + `mode` + `result` tag +
/// variant fields.
///
/// v0.29.0 SemVer-minor wire-shape break: converted from struct with nullable
/// optional fields to tagged enum. `#[serde(tag = "result")]` emits
/// `"result": "match"` / `"result": "no_match"` discriminator and drops the
/// null-emitting optional fields from the no-match variant.
/// Closes FOLLOWUP `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution`.
#[derive(Debug, Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub enum PathOfXpubResult {
    /// Match: all search fields populated.
    Match {
        /// Matched derivation path (e.g. `"m/84'/0'/0'"`).
        path: String,
        /// Matched template name (`"bip84"`, `"bip48-wsh"`, or the literal
        /// `--add-path` string).
        template: String,
        /// Matched account index (None for `--add-path` templates without an
        /// account token).
        account: Option<u32>,
        /// The target xpub after SLIP-0132 normalization.
        target_xpub_canonical: String,
        /// The original SLIP-0132 prefix when the target was alt-prefixed; null
        /// for canonical xpub/tpub input or for mk1-card input.
        target_xpub_variant: Option<&'static str>,
        /// Count of candidates exhausted (paths × templates × add-paths).
        searched_count: usize,
    },
    /// No match: envelope-scope fields preserved; search fields absent.
    NoMatch {
        /// The target xpub after SLIP-0132 normalization.
        target_xpub_canonical: String,
        /// The original SLIP-0132 prefix when the target was alt-prefixed; null
        /// for canonical xpub/tpub input or for mk1-card input.
        target_xpub_variant: Option<&'static str>,
        /// Count of candidates exhausted (paths × templates × add-paths).
        searched_count: usize,
    },
}

/// Construct a `PathOfXpubResult::Match` variant.
///
/// `account` remains `Option<u32>` per existing semantics (None for
/// `--add-path` templates without an account token).
pub(super) fn build_path_match(
    path: String,
    template: String,
    account: Option<u32>,
    target_xpub_canonical: String,
    target_xpub_variant: Option<&'static str>,
    searched_count: usize,
) -> PathOfXpubResult {
    PathOfXpubResult::Match {
        path,
        template,
        account,
        target_xpub_canonical,
        target_xpub_variant,
        searched_count,
    }
}

/// Construct a `PathOfXpubResult::NoMatch` variant.
/// Envelope-scope fields (canonical xpub, variant, searched count) are preserved.
pub(super) fn build_path_no_match(
    target_xpub_canonical: String,
    target_xpub_variant: Option<&'static str>,
    searched_count: usize,
) -> PathOfXpubResult {
    PathOfXpubResult::NoMatch {
        target_xpub_canonical,
        target_xpub_variant,
        searched_count,
    }
}

pub fn run_path_of_xpub<R: Read, W: Write, E: Write>(
    args: &PathOfXpubArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
) -> Result<u8, ToolkitError> {
    // 1) Resolve seed (mutex + parse + ms1 auto-fire short-circuit).
    let mnemonic = resolve_seed(args, stdin, stdout, stderr, no_auto_repair)?;

    // 2) Resolve passphrase. Inline emits argv-leak advisory. Wrapped in
    //    Zeroizing<String> so the heap buffer scrubs on drop (plan §3.6
    //    secret hygiene; mirrors `derive_child.rs:137-151` precedent).
    let passphrase: zeroize::Zeroizing<String> = if args.passphrase_stdin {
        // Re-use convert's stdin reader (preserves embedded NULL bytes).
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
        // when the user routed through the env-var channel (the literal
        // passphrase is not in argv).
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
    // Pin passphrase heap pages for handler scope (mirrors derive_child.rs:159).
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

    // 6) Pin entropy bytes for the candidate-iteration phase. mlock the
    //    seed buffer for the loop lifetime per plan §9.2 (mirrors
    //    derive_slot.rs:82 precedent; works cross-platform via
    //    `mlock::pin_pages_for`'s no-op fallback on non-unix).
    let _seed_pin = mnemonic_toolkit::mlock::pin_pages_for(&seed[..]);

    // 7) Search.
    let matched = match_xpub_against_paths(&master_xprv, &candidates, &target_xpub_65);

    // 8) Emit + return.
    match matched {
        Some(m) => {
            if args.json {
                let envelope = XpubSearchEnvelope {
                    schema_version: "1",
                    body: XpubSearchJson::PathOfXpub(build_path_match(
                        format!("m/{}", m.path),
                        m.template_name.clone(),
                        m.account,
                        target_xpub_canonical.clone(),
                        target_variant,
                        searched_count,
                    )),
                };
                let body = serde_json::to_string(&envelope).map_err(|e| {
                    ToolkitError::BadInput(format!("path-of-xpub JSON serialize: {e}"))
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
                // Emit no-match envelope on stdout for the --json caller, then
                // return the typed no-match error so the exit code routes to 4.
                let envelope = XpubSearchEnvelope {
                    schema_version: "1",
                    body: XpubSearchJson::PathOfXpub(build_path_no_match(
                        target_xpub_canonical.clone(),
                        target_variant,
                        searched_count,
                    )),
                };
                let body = serde_json::to_string(&envelope).map_err(|e| {
                    ToolkitError::BadInput(format!("path-of-xpub JSON serialize: {e}"))
                })?;
                writeln!(stdout, "{body}").map_err(ToolkitError::Io)?;
            }
            Err(ToolkitError::XpubSearchNoMatch {
                mode: "path-of-xpub",
                searched: searched_count,
            })
        }
    }
}
