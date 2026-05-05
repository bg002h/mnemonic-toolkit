//! `mnemonic bundle` subcommand.
//!
//! Realizes SPEC §2.1 (full + watch-only modes), §5.1 (multi-section
//! stdout), §5.2 (engraving card stderr), §5.3 (JSON schema).

use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::format::{chunk_5char, chunk_md1, engraving_card, BundleJson, EngravingMode};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::{check_no_concurrent_stdin, parse_master_fingerprint, read_phrase_input};
use crate::synthesize::{synthesize_full, synthesize_watch_only, Bundle};
use crate::template::CliTemplate;
use bitcoin::bip32::Xpub;
use clap::Args;
use std::io::Write;
use std::str::FromStr;

// SPEC §6.6 requires byte-exact rejection text + exit code 2 for the
// xpub-mode-incompatible flag set. clap's `conflicts_with` would exit 64
// with clap's default usage error and overwrite the SPEC text. So we
// declare ONLY `--phrase` ↔ `--xpub` as mutually-exclusive at the clap
// level (which is the intent — pick a mode); --passphrase / --language /
// --master-fingerprint compatibility is enforced at runtime in `run()`
// with the exact §6.6 text and exit code 2 via ToolkitError::ModeViolation.

#[derive(Args, Debug)]
pub struct BundleArgs {
    #[arg(long, conflicts_with = "xpub")]
    pub phrase: Option<String>,

    #[arg(long, conflicts_with = "phrase")]
    pub xpub: Option<String>,

    #[arg(long = "master-fingerprint")]
    pub master_fingerprint: Option<String>,

    #[arg(long)]
    pub network: CliNetwork,

    #[arg(long)]
    pub template: CliTemplate,

    #[arg(long)]
    pub language: Option<CliLanguage>,

    #[arg(long)]
    pub passphrase: Option<String>,

    /// BIP-32 account index (default 0). Non-zero values produce md1 with
    /// PathDeclPaths::Divergent per SPEC §4.2.
    #[arg(long, default_value = "0")]
    pub account: u32,

    #[arg(long)]
    pub json: bool,

    #[arg(long = "no-engraving-card")]
    pub no_engraving_card: bool,
}

/// SPEC §6.6 byte-exact mode-violation strings. Pinned for integration tests.
pub mod mode_text {
    pub const PASSPHRASE_WITH_XPUB: &str = "--passphrase is incompatible with --xpub: the xpub is already a post-passphrase derivation product (the passphrase is baked into the xpub at engrave time).";
    pub const LANGUAGE_WITH_XPUB: &str =
        "--language is meaningful only with --phrase; xpub-only mode does not consult any wordlist";
    pub const XPUB_NEEDS_FINGERPRINT: &str = "--xpub requires --master-fingerprint (xpub mode needs the master fingerprint to populate mk1's origin)";
    pub const FINGERPRINT_WITHOUT_XPUB: &str =
        "--master-fingerprint is meaningful only with --xpub";
    pub const XPUB_STDIN: &str =
        "--xpub does not accept stdin (-); pass the xpub literally on argv";
}

pub fn run<W: Write, E: Write>(
    args: &BundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let phrase_arg = args.phrase.as_deref();
    let xpub_arg = args.xpub.as_deref();

    // SPEC §6.6 mode-violation pre-checks (BEFORE mode dispatch so the
    // exit code is 2 + byte-exact text, not clap's 64 + default text).
    if xpub_arg.is_some() && args.passphrase.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only",
            flag: "--passphrase",
            message: mode_text::PASSPHRASE_WITH_XPUB,
        });
    }
    if xpub_arg.is_some() && args.language.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only",
            flag: "--language",
            message: mode_text::LANGUAGE_WITH_XPUB,
        });
    }
    if xpub_arg.is_some() && args.master_fingerprint.is_none() {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only",
            flag: "--xpub",
            message: mode_text::XPUB_NEEDS_FINGERPRINT,
        });
    }
    if xpub_arg.is_none() && args.master_fingerprint.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "full",
            flag: "--master-fingerprint",
            message: mode_text::FINGERPRINT_WITHOUT_XPUB,
        });
    }

    // Mode dispatch.
    if let Some(xpub_str) = xpub_arg {
        if xpub_str == "-" {
            return Err(ToolkitError::BadInput(mode_text::XPUB_STDIN.to_string()));
        }
        bundle_watch_only(args, xpub_str, stdout, stderr)
    } else if phrase_arg.is_some() {
        check_no_concurrent_stdin(phrase_arg, args.passphrase.as_deref())?;
        bundle_full(args, stdin, stdout, stderr)
    } else {
        Err(ToolkitError::BadInput("expected --phrase or --xpub".into()))
    }
}

fn bundle_full<W: Write, E: Write>(
    args: &BundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let phrase = read_phrase_input(args.phrase.as_deref(), stdin)?;
    let passphrase = args.passphrase.clone().unwrap_or_default();
    let language = args.language.unwrap_or_default();

    // Stderr: language defaulting warning (SPEC §5.2 ordering rule 1).
    if args.language.is_none() {
        writeln!(stderr, "warning: --language defaulting to english; record the wordlist language alongside the engraved cards.").ok();
    }
    // Stderr: passphrase warning (rule 2).
    if !passphrase.is_empty() {
        writeln!(
            stderr,
            "warning: --passphrase set; the passphrase is NOT engraved on any card and must"
        )
        .ok();
        writeln!(
            stderr,
            "warning: be remembered separately. A forgotten passphrase is unrecoverable from"
        )
        .ok();
        writeln!(stderr, "warning: the engraved bundle.").ok();
    }

    let acc = crate::derive::derive_full(
        &phrase,
        &passphrase,
        language,
        args.network,
        args.template,
        args.account,
    )?;
    let bundle = synthesize_full(
        &acc.entropy,
        acc.master_fingerprint,
        acc.account_xpub,
        args.template,
        args.network,
        args.account,
    )?;

    let card_text = if args.no_engraving_card {
        None
    } else {
        let mode = if passphrase.is_empty() {
            EngravingMode::FullNoPassphrase {
                language: language.human_name(),
            }
        } else {
            EngravingMode::FullWithPassphrase {
                language: language.human_name(),
            }
        };
        Some(engraving_card(
            args.network.human_name(),
            args.template.human_name(),
            &args.template.origin_path_str(args.network, args.account),
            &acc.master_fingerprint.to_string().to_lowercase(),
            args.account,
            mode,
        ))
    };

    emit(
        args,
        &bundle,
        card_text.as_deref(),
        &acc.master_fingerprint.to_string().to_lowercase(),
        "full",
        stdout,
        stderr,
        args.template.origin_path_str(args.network, args.account),
    )
}

fn bundle_watch_only<W: Write, E: Write>(
    args: &BundleArgs,
    xpub_str: &str,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let fp_str = args.master_fingerprint.as_deref().ok_or_else(|| {
        ToolkitError::BadInput(
            "--xpub requires --master-fingerprint (xpub mode needs the master fingerprint to populate mk1's origin)"
                .into(),
        )
    })?;
    let fp = parse_master_fingerprint(fp_str)?;
    let xpub = Xpub::from_str(xpub_str)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::XpubParse(format!("{}", e))))?;

    // §4.3 network/xpub cross-check.
    if xpub.network != args.network.network_kind() {
        return Err(ToolkitError::NetworkMismatch {
            xpub_network: if xpub.network == bitcoin::NetworkKind::Main {
                "mainnet"
            } else {
                "testnet/signet/regtest"
            },
            expected: args.network.human_name(),
        });
    }

    // §4.8 watch-only depth advisory.
    if xpub.depth != 3 {
        writeln!(
            stderr,
            "warning: --xpub depth is {}; expected 3 for canonical BIP-44/49/84/86 paths.",
            xpub.depth
        )
        .ok();
        writeln!(
            stderr,
            "warning: Bundle will still be emitted; verify your wallet uses a non-standard path."
        )
        .ok();
    }

    // §4.8 watch-only account-index hazard (always emitted in watch-only).
    writeln!(
        stderr,
        "warning: watch-only mode hardcodes account=0; if your xpub was derived"
    )
    .ok();
    writeln!(
        stderr,
        "warning: at a non-zero account, the bundle's path will not match. Use"
    )
    .ok();
    writeln!(stderr, "warning: v0.2's --account flag once available.").ok();

    let bundle = synthesize_watch_only(fp, xpub, args.template, args.network, args.account)?;

    let card_text = if args.no_engraving_card {
        None
    } else {
        Some(engraving_card(
            args.network.human_name(),
            args.template.human_name(),
            &args.template.origin_path_str(args.network, args.account),
            &fp.to_string().to_lowercase(),
            args.account,
            EngravingMode::WatchOnly,
        ))
    };

    emit(
        args,
        &bundle,
        card_text.as_deref(),
        &fp.to_string().to_lowercase(),
        "watch-only",
        stdout,
        stderr,
        args.template.origin_path_str(args.network, args.account),
    )
}

#[allow(clippy::too_many_arguments)]
fn emit<W: Write, E: Write>(
    args: &BundleArgs,
    bundle: &Bundle,
    engraving_text: Option<&str>,
    master_fp: &str,
    mode: &'static str,
    stdout: &mut W,
    stderr: &mut E,
    origin_path: String,
) -> Result<(), ToolkitError> {
    if args.json {
        let json = BundleJson {
            schema_version: "1",
            mode,
            network: args.network.human_name(),
            template: args.template.human_name(),
            account: args.account,
            origin_path,
            master_fingerprint: master_fp.to_string(),
            ms1: bundle.ms1.as_deref(),
            mk1: &bundle.mk1,
            md1: &bundle.md1,
            engraving_card: engraving_text.map(|s| s.to_string()),
        };
        serde_json::to_writer(&mut *stdout, &json).ok();
        writeln!(stdout).ok();
    } else {
        // Multi-section text output (SPEC §5.1).
        if let Some(ms1) = bundle.ms1.as_deref() {
            writeln!(stdout, "# ms1 (entropy, BCH-checksummed)").ok();
            writeln!(stdout, "{}", ms1).ok();
            writeln!(stdout).ok();
            writeln!(stdout, "{}", chunk_5char(ms1)).ok();
            writeln!(stdout).ok();
        } else {
            writeln!(stdout, "# ms1 (omitted — xpub-only mode)").ok();
            writeln!(stdout).ok();
        }

        writeln!(stdout, "# mk1 (xpub + origin)").ok();
        for s in &bundle.mk1 {
            writeln!(stdout, "{}", s).ok();
        }
        writeln!(stdout).ok();
        for s in &bundle.mk1 {
            writeln!(stdout, "{}", chunk_5char(s)).ok();
        }
        writeln!(stdout).ok();

        writeln!(stdout, "# md1 (wallet policy)").ok();
        for s in &bundle.md1 {
            writeln!(stdout, "{}", s).ok();
        }
        writeln!(stdout).ok();
        for s in &bundle.md1 {
            writeln!(stdout, "{}", chunk_md1(s)).ok();
        }
        writeln!(stdout).ok();

        if let Some(text) = engraving_text {
            // Stderr ordering: warnings already emitted; engraving card last.
            write!(stderr, "{}", text).ok();
        }
    }
    Ok(())
}
