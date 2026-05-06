//! `mnemonic bundle` subcommand.
//!
//! Realizes SPEC §2.1 (full + watch-only modes), §5.1 (multi-section
//! stdout), §5.2 (engraving card stderr), §5.3 (JSON schema).

use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::format::{
    chunk_5char, chunk_md1, engraving_card, BundleJson, CosignerEntry, EngravingMode, MkField,
    MultisigInfo,
};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::{
    check_no_concurrent_stdin, parse_cosigner_spec, parse_cosigners_file, parse_master_fingerprint,
    read_phrase_input, CosignerSpec, MultisigPathFamily,
};
use crate::synthesize::{
    synthesize_full, synthesize_multisig_full, synthesize_multisig_watch_only,
    synthesize_watch_only, Bundle,
};
use crate::template::CliTemplate;
use bitcoin::bip32::Xpub;
use clap::Args;
use std::io::Write;
use std::path::PathBuf;
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

    /// Pre-built template name (single-sig or multisig). Mutually-required-one-of
    /// with --descriptor / --descriptor-file (clap-level + runtime pre-check).
    #[arg(long, required_unless_present_any = ["descriptor", "descriptor_file"])]
    pub template: Option<CliTemplate>,

    /// User-supplied BIP-388 descriptor (v0.3 §2.1.10). Mutually-required-one-of
    /// with --template / --descriptor-file. XOR with --descriptor-file (clap conflicts).
    #[arg(long, conflicts_with = "descriptor_file")]
    pub descriptor: Option<String>,

    /// User-supplied BIP-388 descriptor file (v0.3 §2.1.10). Single-line UTF-8;
    /// trailing newline tolerated. XOR with --descriptor (clap conflicts).
    #[arg(long = "descriptor-file")]
    pub descriptor_file: Option<PathBuf>,

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

    /// v0.2 multisig watch-only: per-cosigner spec `<xpub>:<fp>:<path>`. Repeatable.
    #[arg(long, action = clap::ArgAction::Append)]
    pub cosigner: Vec<String>,

    /// v0.2 multisig watch-only: bulk cosigners via JSON file.
    #[arg(long = "cosigners-file")]
    pub cosigners_file: Option<PathBuf>,

    /// v0.2 multisig path family (default: bip87).
    #[arg(long = "multisig-path-family", value_enum)]
    pub multisig_path_family: Option<MultisigPathFamily>,

    /// v0.2 privacy mode: suppress master fingerprint from mk1 + engraving card.
    #[arg(long, default_value = "false")]
    pub privacy_preserving: bool,

    /// v0.2 self-check: re-parse the emitted bundle and verify it round-trips.
    #[arg(long, default_value = "false")]
    pub self_check: bool,

    /// v0.2 multisig threshold K (1 ≤ K ≤ N ≤ 16).
    #[arg(long)]
    pub threshold: Option<u8>,

    /// v0.2 multisig cosigner count N (1 ≤ K ≤ N ≤ 16).
    #[arg(long = "cosigner-count")]
    pub cosigner_count: Option<usize>,
}

impl BundleArgs {
    /// Template-mode contract: callers MUST be on the template-mode dispatch
    /// branch. Descriptor-mode escapes earlier in `run()` before any
    /// template-only helper is invoked. Panics if the contract is violated.
    fn template_unchecked(&self) -> CliTemplate {
        self.template
            .expect("template-mode dispatch contract — descriptor-mode escapes earlier")
    }
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

    // v0.2 NEW rows (SPEC §6.6 v0.2 NEW table). Byte-exact.
    pub const XPUB_AND_COSIGNER: &str = "--xpub cannot be combined with --cosigner or --cosigners-file; pick single-sig (--xpub) or multisig (--cosigner/--cosigners-file) but not both.";
    pub const COSIGNER_AND_COSIGNERS_FILE: &str = "--cosigner cannot be combined with --cosigners-file; supply cosigners via flag-repetition or file, not both.";
    pub const THRESHOLD_WITHOUT_MULTISIG: &str = "--threshold is meaningful only with a multisig --template; single-sig templates ignore threshold.";
    pub const COSIGNER_COUNT_WITHOUT_MULTISIG: &str =
        "--cosigner-count is meaningful only with a multisig --template.";
    pub const PATH_FAMILY_WITHOUT_MULTISIG: &str =
        "--multisig-path-family is meaningful only with a multisig --template.";
    pub const PRIVACY_WITH_XPUB: &str = "--privacy-preserving with --xpub (single-sig watch-only) has no useful effect: --xpub mode requires --master-fingerprint and the bundle's md1 binds that fingerprint into tlv.fingerprints; suppressing it from mk1 only would produce an inconsistent bundle. Drop --privacy-preserving or switch to multisig watch-only mode.";
    // §6.6 row 7 — reserved for Phase C+ when v0.3+ templates may lack
    // an account-position. Currently never emitted (all v0.2 templates have
    // an account position).
    #[allow(dead_code)]
    pub const ACCOUNT_INCOMPATIBLE_TEMPLATE: &str = "--account is incompatible with the selected --template (template lacks an account-position in its standard path).";

    // v0.3 NEW rows (SPEC §6.9). Byte-exact.
    pub const DESCRIPTOR_AND_TEMPLATE: &str = "--descriptor and --template are mutually exclusive; pick descriptor passthrough or template, not both.";
    pub const DESCRIPTOR_AND_DESCRIPTOR_FILE: &str = "--descriptor and --descriptor-file are mutually exclusive; supply the descriptor inline or via file, not both.";
    pub const DESCRIPTOR_WITH_THRESHOLD: &str = "--threshold is meaningful only with a multisig --template; descriptor mode encodes K directly.";
    pub const DESCRIPTOR_WITH_COSIGNER_COUNT: &str = "--cosigner-count is meaningful only with --template; descriptor mode encodes N from @i placeholder count.";
    pub const DESCRIPTOR_WITH_PATH_FAMILY: &str = "--multisig-path-family is meaningful only with --template; descriptor mode encodes paths directly via @i/path syntax.";
    pub const DESCRIPTOR_WITH_NONZERO_ACCOUNT: &str = "--account != 0 is meaningful only with --template; descriptor mode encodes account index in the @i origin path.";
}

pub fn run<W: Write, E: Write>(
    args: &BundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    // SPEC §6.9 v0.3 mode-violation pre-check ladder, rows 1-6 (flag-combination
    // checks; rows 7-15 fire after descriptor parse, inside descriptor_mode_run).
    // Evaluated TOP-TO-BOTTOM; first triggered row fires.
    let descriptor_mode = args.descriptor.is_some() || args.descriptor_file.is_some();
    if descriptor_mode && args.template.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--template",
            message: mode_text::DESCRIPTOR_AND_TEMPLATE,
        });
    }
    if args.descriptor.is_some() && args.descriptor_file.is_some() {
        // clap conflicts_with usually rejects this; runtime backstop for
        // direct API callers that bypass clap.
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--descriptor-file",
            message: mode_text::DESCRIPTOR_AND_DESCRIPTOR_FILE,
        });
    }
    if descriptor_mode && args.threshold.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--threshold",
            message: mode_text::DESCRIPTOR_WITH_THRESHOLD,
        });
    }
    if descriptor_mode && args.cosigner_count.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--cosigner-count",
            message: mode_text::DESCRIPTOR_WITH_COSIGNER_COUNT,
        });
    }
    if descriptor_mode && args.multisig_path_family.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--multisig-path-family",
            message: mode_text::DESCRIPTOR_WITH_PATH_FAMILY,
        });
    }
    if descriptor_mode && args.account != 0 {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--account",
            message: mode_text::DESCRIPTOR_WITH_NONZERO_ACCOUNT,
        });
    }

    // Descriptor-mode dispatch (Phase B stub; Phase C lands synthesis).
    if descriptor_mode {
        return descriptor_mode_run(args, stdin, stdout, stderr);
    }

    let phrase_arg = args.phrase.as_deref();
    let xpub_arg = args.xpub.as_deref();
    let multisig = args.template_unchecked().is_multisig();
    let cosigner_present = !args.cosigner.is_empty();
    let cosigners_file_present = args.cosigners_file.is_some();

    // SPEC §6.6 v0.2 NEW mode-violation pre-checks (BEFORE single-sig checks).
    if xpub_arg.is_some() && (cosigner_present || cosigners_file_present) {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only",
            flag: "--cosigner/--cosigners-file",
            message: mode_text::XPUB_AND_COSIGNER,
        });
    }
    if cosigner_present && cosigners_file_present {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only-multisig",
            flag: "--cosigners-file",
            message: mode_text::COSIGNER_AND_COSIGNERS_FILE,
        });
    }
    if args.threshold.is_some() && !multisig {
        return Err(ToolkitError::ModeViolation {
            mode: "single-sig",
            flag: "--threshold",
            message: mode_text::THRESHOLD_WITHOUT_MULTISIG,
        });
    }
    if args.cosigner_count.is_some() && !multisig {
        return Err(ToolkitError::ModeViolation {
            mode: "single-sig",
            flag: "--cosigner-count",
            message: mode_text::COSIGNER_COUNT_WITHOUT_MULTISIG,
        });
    }
    if args.multisig_path_family.is_some() && !multisig {
        return Err(ToolkitError::ModeViolation {
            mode: "single-sig",
            flag: "--multisig-path-family",
            message: mode_text::PATH_FAMILY_WITHOUT_MULTISIG,
        });
    }
    if args.privacy_preserving && xpub_arg.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only",
            flag: "--privacy-preserving",
            message: mode_text::PRIVACY_WITH_XPUB,
        });
    }
    // §6.6 row "ACCOUNT_INCOMPATIBLE_TEMPLATE": never fires for v0.2's
    // templates (all have an account-position in their standard path).
    // TODO: revisit when v0.3+ adds template families that lack one.

    // SPEC §6.6 single-sig mode-violation pre-checks (BEFORE mode dispatch so the
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

    // v0.2 multisig mode dispatch.
    if multisig {
        let threshold = args.threshold.ok_or_else(|| ToolkitError::MultisigConfig {
            message: "--threshold required for multisig templates".into(),
        })?;
        let path_family = args.multisig_path_family.unwrap_or_default();

        if cosigner_present || cosigners_file_present {
            return bundle_multisig_watch_only(args, threshold, path_family, stdout, stderr);
        }
        if phrase_arg.is_some() {
            check_no_concurrent_stdin(phrase_arg, args.passphrase.as_deref())?;
            let cosigner_count =
                args.cosigner_count
                    .ok_or_else(|| ToolkitError::MultisigConfig {
                        message: "--cosigner-count required for full-mode multisig".into(),
                    })?;
            return bundle_multisig_full(
                args,
                threshold,
                cosigner_count,
                path_family,
                stdin,
                stdout,
                stderr,
            );
        }
        return Err(ToolkitError::BadInput(
            "multisig bundle requires --phrase (full mode) or --cosigner/--cosigners-file (watch-only)".into(),
        ));
    }

    // Single-sig mode dispatch.
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
        args.template_unchecked(),
        args.account,
    )?;
    let bundle = synthesize_full(
        &acc.entropy,
        acc.master_fingerprint,
        acc.account_xpub,
        args.template_unchecked(),
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
            args.template_unchecked().human_name(),
            &args
                .template_unchecked()
                .origin_path_str(args.network, args.account),
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
        args.template_unchecked()
            .origin_path_str(args.network, args.account),
    )?;

    if args.self_check {
        self_check_bundle(&bundle, args)?;
    }
    Ok(())
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

    // §4.8 watch-only account-index hazard (emitted only when --account is at its
    // default 0; user may not realize the default). v0.2 makes --account user-tunable.
    if args.account == 0 {
        writeln!(
            stderr,
            "warning: --account defaults to 0; if your xpub was derived at a non-zero"
        )
        .ok();
        writeln!(
            stderr,
            "warning: account, pass --account <N> to match. Default may not align with"
        )
        .ok();
        writeln!(
            stderr,
            "warning: the supplied xpub's actual derivation account."
        )
        .ok();
    }

    let bundle = synthesize_watch_only(
        fp,
        xpub,
        args.template_unchecked(),
        args.network,
        args.account,
    )?;

    let card_text = if args.no_engraving_card {
        None
    } else {
        Some(engraving_card(
            args.network.human_name(),
            args.template_unchecked().human_name(),
            &args
                .template_unchecked()
                .origin_path_str(args.network, args.account),
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
        args.template_unchecked()
            .origin_path_str(args.network, args.account),
    )?;

    if args.self_check {
        self_check_bundle(&bundle, args)?;
    }
    Ok(())
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
        // v0.2: schema_version "2"; bundle.mk1 already typed as MkField (Single
        // for single-sig matches v0.1 flat shape via #[serde(untagged)]; Multi
        // for multisig). multisig: None for single-sig. origin_path populated;
        // origin_paths: None (single-sig is always shared-with-itself).
        let json = BundleJson {
            schema_version: "3",
            mode,
            network: args.network.human_name(),
            template: Some(args.template_unchecked().human_name()),
            descriptor: None,
            account: args.account,
            origin_path: Some(origin_path),
            origin_paths: None,
            master_fingerprint: Some(master_fp.to_string()),
            ms1: bundle.ms1.clone(),
            mk1: bundle.mk1.clone(),
            md1: bundle.md1.clone(),
            engraving_card: engraving_text.map(|s| s.to_string()),
            multisig: None,
            privacy_preserving: args.privacy_preserving,
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

        match &bundle.mk1 {
            MkField::Single(mk1) => {
                writeln!(stdout, "# mk1 (xpub + origin)").ok();
                for s in mk1 {
                    writeln!(stdout, "{}", s).ok();
                }
                writeln!(stdout).ok();
                for s in mk1 {
                    writeln!(stdout, "{}", chunk_5char(s)).ok();
                }
                writeln!(stdout).ok();
            }
            MkField::Multi(per_cosigner) => {
                // SPEC §5.1 multisig: per-cosigner `# mk1[<i>]` headers.
                for (i, chunks) in per_cosigner.iter().enumerate() {
                    writeln!(stdout, "# mk1[{}] (cosigner {} xpub + origin)", i, i).ok();
                    for s in chunks {
                        writeln!(stdout, "{}", s).ok();
                    }
                    writeln!(stdout).ok();
                    for s in chunks {
                        writeln!(stdout, "{}", chunk_5char(s)).ok();
                    }
                    writeln!(stdout).ok();
                }
            }
        }

        let md1_label = if matches!(bundle.mk1, MkField::Multi(_)) {
            "# md1 (multisig wallet policy)"
        } else {
            "# md1 (wallet policy)"
        };
        writeln!(stdout, "{}", md1_label).ok();
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

/// SPEC §4.1 SELF-MULTISIG WARNING (byte-exact, non-suppressible).
/// Emitted BEFORE the bundle stdout block per SPEC §4.1 ordering rule.
pub const SELF_MULTISIG_WARNING: &str = "\
warning: full-mode multisig (--cosigner-count > 1) derives all N cosigner xpubs from one
warning: seed at one path; all N cosigner cards are byte-identical interchangeable copies.
warning: For production multi-device multisig, use --cosigner watch-only mode with distinct
warning: cosigner xpubs from distinct seeds.
";

#[allow(clippy::too_many_arguments)]
fn bundle_multisig_full<W: Write, E: Write>(
    args: &BundleArgs,
    threshold: u8,
    cosigner_count: usize,
    path_family: MultisigPathFamily,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let phrase = read_phrase_input(args.phrase.as_deref(), stdin)?;
    let passphrase = args.passphrase.clone().unwrap_or_default();
    let language = args.language.unwrap_or_default();

    // Stderr ordering (SPEC §4.1 + §5.2): SELF-MULTISIG WARNING FIRST when N>1
    // (BEFORE language/passphrase warnings to satisfy "before bundle stdout block").
    if cosigner_count > 1 {
        write!(stderr, "{}", SELF_MULTISIG_WARNING).ok();
    }
    if args.language.is_none() {
        writeln!(stderr, "warning: --language defaulting to english; record the wordlist language alongside the engraved cards.").ok();
    }
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

    let mnemonic =
        bip39::Mnemonic::parse_in(language.into(), &phrase).map_err(ToolkitError::Bip39)?;

    let bundle = synthesize_multisig_full(
        &mnemonic,
        &passphrase,
        args.network,
        args.template_unchecked(),
        threshold,
        cosigner_count,
        args.account,
        path_family,
        args.privacy_preserving,
    )?;

    // Build MultisigInfo for JSON + engraving card.
    let script_type = args.template_unchecked().bip48_script_type().unwrap_or(0);
    let path_str = path_family.default_origin_path(args.network, args.account, script_type);
    use bitcoin::bip32::Xpriv;
    use bitcoin::secp256k1::Secp256k1;
    let secp = Secp256k1::new();
    let seed = mnemonic.to_seed(&passphrase);
    let master = Xpriv::new_master(args.network.network_kind(), &seed)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
    let master_fp = master.fingerprint(&secp);
    let master_fp_str = master_fp.to_string().to_lowercase();
    let derive_path = bitcoin::bip32::DerivationPath::from_str(&path_str)
        .map_err(|e| ToolkitError::BadInput(format!("path parse {}: {}", path_str, e)))?;
    let xpriv = master
        .derive_priv(&secp, &derive_path)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
    let xpub = bitcoin::bip32::Xpub::from_priv(&secp, &xpriv);
    let xpub_str = xpub.to_string();

    let cosigners_meta: Vec<CosignerEntry> = (0..cosigner_count)
        .map(|i| CosignerEntry {
            index: i,
            master_fingerprint: if args.privacy_preserving {
                None
            } else {
                Some(master_fp_str.clone())
            },
            origin_path: path_str.clone(),
            xpub: xpub_str.clone(),
        })
        .collect();
    let multisig_info = MultisigInfo {
        template: args.template_unchecked().human_name(),
        threshold,
        cosigner_count,
        path_family: path_family.human_name(),
        cosigners: cosigners_meta,
    };

    let card_text = if args.no_engraving_card {
        None
    } else {
        Some(engraving_card(
            args.network.human_name(),
            args.template_unchecked().human_name(),
            &path_str,
            &master_fp_str,
            args.account,
            EngravingMode::FullMultisig {
                language: language.human_name(),
                passphrase_used: !passphrase.is_empty(),
                multisig_info: &multisig_info,
                account: args.account,
                paths_shared: true,
            },
        ))
    };

    emit_multisig(
        args,
        &bundle,
        card_text.as_deref(),
        "full",
        Some(multisig_info),
        stdout,
        stderr,
    )?;

    if args.self_check {
        self_check_bundle(&bundle, args)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn bundle_multisig_watch_only<W: Write, E: Write>(
    args: &BundleArgs,
    threshold: u8,
    path_family: MultisigPathFamily,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    // Parse cosigner specs.
    let mut cosigners: Vec<CosignerSpec> = if let Some(file) = &args.cosigners_file {
        parse_cosigners_file(file)?
    } else {
        let mut out = Vec::with_capacity(args.cosigner.len());
        for (i, s) in args.cosigner.iter().enumerate() {
            out.push(parse_cosigner_spec(s, i)?);
        }
        out
    };
    if cosigners.is_empty() {
        return Err(ToolkitError::MultisigConfig {
            message: "no cosigners supplied".into(),
        });
    }

    // Resolve per-cosigner paths so we can emit them into MultisigInfo even
    // when they were defaulted from the path family.
    let script_type = args.template_unchecked().bip48_script_type().unwrap_or(0);
    let default_path_str = path_family.default_origin_path(args.network, args.account, script_type);
    let default_path = bitcoin::bip32::DerivationPath::from_str(&default_path_str)
        .map_err(|e| ToolkitError::BadInput(format!("default path parse: {}", e)))?;

    let resolved_paths: Vec<bitcoin::bip32::DerivationPath> = cosigners
        .iter()
        .map(|c| c.path.clone().unwrap_or_else(|| default_path.clone()))
        .collect();

    // §4.8 per-cosigner depth advisory.
    let expected_depth = match path_family {
        MultisigPathFamily::Bip48 => 4u8,
        MultisigPathFamily::Bip87 => 3u8,
    };
    for (i, c) in cosigners.iter().enumerate() {
        if c.xpub.depth != expected_depth {
            writeln!(
                stderr,
                "warning: cosigner @{} xpub depth is {}; expected {} for {} paths.",
                i,
                c.xpub.depth,
                expected_depth,
                path_family.human_name(),
            )
            .ok();
        }
    }

    // Synthesize.
    let bundle = synthesize_multisig_watch_only(
        &cosigners,
        args.network,
        args.template_unchecked(),
        threshold,
        args.account,
        path_family,
        args.privacy_preserving,
    )?;

    // Build MultisigInfo.
    let cosigner_count = cosigners.len();
    let cosigners_meta: Vec<CosignerEntry> = cosigners
        .iter_mut()
        .zip(resolved_paths.iter())
        .enumerate()
        .map(|(i, (c, p))| CosignerEntry {
            index: i,
            master_fingerprint: if args.privacy_preserving {
                None
            } else {
                Some(c.master_fingerprint.to_string().to_lowercase())
            },
            origin_path: p.to_string(),
            xpub: c.xpub.to_string(),
        })
        .collect();
    let multisig_info = MultisigInfo {
        template: args.template_unchecked().human_name(),
        threshold,
        cosigner_count,
        path_family: path_family.human_name(),
        cosigners: cosigners_meta,
    };

    let paths_shared = resolved_paths.windows(2).all(|w| w[0] == w[1]);

    let card_text = if args.no_engraving_card {
        None
    } else {
        Some(engraving_card(
            args.network.human_name(),
            args.template_unchecked().human_name(),
            &default_path_str,
            "(per-cosigner)",
            args.account,
            EngravingMode::WatchOnlyMultisig {
                multisig_info: &multisig_info,
                account: args.account,
                paths_shared,
            },
        ))
    };

    emit_multisig(
        args,
        &bundle,
        card_text.as_deref(),
        "watch-only",
        Some(multisig_info),
        stdout,
        stderr,
    )?;

    if args.self_check {
        self_check_bundle(&bundle, args)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn emit_multisig<W: Write, E: Write>(
    args: &BundleArgs,
    bundle: &Bundle,
    engraving_text: Option<&str>,
    mode: &'static str,
    multisig_info: Option<MultisigInfo>,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    if args.json {
        // SPEC §5.3 multisig envelope shape:
        //   - origin_path / origin_paths discriminated by path-decl shape.
        //   - master_fingerprint = null for multisig OR --privacy-preserving.
        let (origin_path, origin_paths) = if let Some(info) = multisig_info.as_ref() {
            let paths: Vec<String> = info
                .cosigners
                .iter()
                .map(|c| c.origin_path.clone())
                .collect();
            let all_same = paths.windows(2).all(|w| w[0] == w[1]);
            if all_same {
                (paths.first().cloned(), None)
            } else {
                (None, Some(paths))
            }
        } else {
            (None, None)
        };
        let json = BundleJson {
            schema_version: "3",
            mode,
            network: args.network.human_name(),
            template: Some(args.template_unchecked().human_name()),
            descriptor: None,
            account: args.account,
            origin_path,
            origin_paths,
            // Multisig OR privacy: top-level master_fingerprint is null per SPEC §5.3.
            master_fingerprint: None,
            ms1: bundle.ms1.clone(),
            mk1: bundle.mk1.clone(),
            md1: bundle.md1.clone(),
            engraving_card: engraving_text.map(|s| s.to_string()),
            multisig: multisig_info,
            privacy_preserving: args.privacy_preserving,
        };
        serde_json::to_writer(&mut *stdout, &json).ok();
        writeln!(stdout).ok();
    } else {
        if let Some(ms1) = bundle.ms1.as_deref() {
            writeln!(stdout, "# ms1 (entropy, BCH-checksummed)").ok();
            writeln!(stdout, "{}", ms1).ok();
            writeln!(stdout).ok();
            writeln!(stdout, "{}", chunk_5char(ms1)).ok();
            writeln!(stdout).ok();
        } else {
            writeln!(stdout, "# ms1 (omitted — multisig watch-only mode)").ok();
            writeln!(stdout).ok();
        }

        if let MkField::Multi(per_cosigner) = &bundle.mk1 {
            for (i, chunks) in per_cosigner.iter().enumerate() {
                writeln!(stdout, "# mk1[{}] (cosigner {} xpub + origin)", i, i).ok();
                for s in chunks {
                    writeln!(stdout, "{}", s).ok();
                }
                writeln!(stdout).ok();
                for s in chunks {
                    writeln!(stdout, "{}", chunk_5char(s)).ok();
                }
                writeln!(stdout).ok();
            }
        }

        writeln!(stdout, "# md1 (multisig wallet policy)").ok();
        for s in &bundle.md1 {
            writeln!(stdout, "{}", s).ok();
        }
        writeln!(stdout).ok();
        for s in &bundle.md1 {
            writeln!(stdout, "{}", chunk_md1(s)).ok();
        }
        writeln!(stdout).ok();

        if let Some(text) = engraving_text {
            write!(stderr, "{}", text).ok();
        }
    }
    Ok(())
}

/// Self-check (SPEC §2.1.9): re-decode the emitted bundle and verify cross-binding.
/// Used by `--self-check`. Emits exit 4 BundleMismatch with `card =
/// "self-check[<failed>]"` per SPEC §2.1.9.
pub fn self_check_bundle(bundle: &Bundle, args: &BundleArgs) -> Result<(), ToolkitError> {
    // md1 decode.
    let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
    let desc =
        md_codec::chunk::reassemble(&md1_strs).map_err(|e| ToolkitError::BundleMismatch {
            card: "self-check[md1_decode]".into(),
            message: format!("{:?}", e),
        })?;
    if !desc.is_wallet_policy() {
        return Err(ToolkitError::BundleMismatch {
            card: "self-check[md1_wallet_policy]".into(),
            message: "descriptor is not in wallet-policy mode".into(),
        });
    }
    let pid =
        md_codec::compute_wallet_policy_id(&desc).map_err(|e| ToolkitError::BundleMismatch {
            card: "self-check[stub_linkage]".into(),
            message: format!("policy_id compute: {:?}", e),
        })?;
    let expected_stub: [u8; 4] = pid.as_bytes()[..4].try_into().unwrap();

    match &bundle.mk1 {
        MkField::Single(mk1) => {
            let mk1_strs: Vec<&str> = mk1.iter().map(|s| s.as_str()).collect();
            let card = mk_codec::decode(&mk1_strs).map_err(|e| ToolkitError::BundleMismatch {
                card: "self-check[mk1_decode]".into(),
                message: format!("{:?}", e),
            })?;
            if !card.policy_id_stubs.iter().any(|s| *s == expected_stub) {
                return Err(ToolkitError::BundleMismatch {
                    card: "self-check[stub_linkage]".into(),
                    message: "mk1 policy_id_stubs do not include descriptor's stub".into(),
                });
            }
            if !args.privacy_preserving && card.origin_fingerprint.is_none() {
                return Err(ToolkitError::BundleMismatch {
                    card: "self-check[mk1_fingerprint_match]".into(),
                    message: "mk1 missing origin_fingerprint but --privacy-preserving not set"
                        .into(),
                });
            }
            if args.privacy_preserving && card.origin_fingerprint.is_some() {
                return Err(ToolkitError::BundleMismatch {
                    card: "self-check[mk1_fingerprint_match]".into(),
                    message: "mk1 has origin_fingerprint but --privacy-preserving was set".into(),
                });
            }
        }
        MkField::Multi(per_cosigner) => {
            // Decode each card-set; verify all share the same stubs list.
            let mut decoded_cards: Vec<mk_codec::KeyCard> = Vec::with_capacity(per_cosigner.len());
            for (i, chunks) in per_cosigner.iter().enumerate() {
                let strs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
                let card = mk_codec::decode(&strs).map_err(|e| ToolkitError::BundleMismatch {
                    card: format!("self-check[mk1_decode[{}]]", i),
                    message: format!("{:?}", e),
                })?;
                decoded_cards.push(card);
            }
            let first_stubs = &decoded_cards[0].policy_id_stubs;
            for (i, c) in decoded_cards.iter().enumerate().skip(1) {
                if &c.policy_id_stubs != first_stubs {
                    return Err(ToolkitError::BundleMismatch {
                        card: format!("self-check[stub_linkage[{}]]", i),
                        message: "policy_id_stubs differ across cosigner cards".into(),
                    });
                }
            }
            if !first_stubs.iter().any(|s| *s == expected_stub) {
                return Err(ToolkitError::BundleMismatch {
                    card: "self-check[stub_linkage]".into(),
                    message: "mk1 policy_id_stubs do not include descriptor's stub".into(),
                });
            }
        }
    }
    Ok(())
}

/// Phase B stub for descriptor-mode dispatch. Lex's the descriptor (so SPEC §6.9
/// row 8 — empty / no @N — fires as a DescriptorParse error from lex_placeholders),
/// then errors out indicating Phase C lands the synthesis. Phase C will replace
/// this stub with full pipeline: parse_descriptor → key sourcing → synthesize_descriptor.
fn descriptor_mode_run<W: Write, E: Write>(
    args: &BundleArgs,
    _stdin: &mut dyn std::io::Read,
    _stdout: &mut W,
    _stderr: &mut E,
) -> Result<(), ToolkitError> {
    let descriptor = match (&args.descriptor, &args.descriptor_file) {
        (Some(s), None) => s.clone(),
        (None, Some(p)) => std::fs::read_to_string(p)
            .map_err(|e| {
                ToolkitError::DescriptorParse(format!("--descriptor-file {}: {e}", p.display()))
            })?
            .trim_end()
            .to_string(),
        _ => unreachable!("pre-check ladder rejects all other combos"),
    };
    // Row 8: SPEC §6.9 "descriptor must contain at least one @N placeholder."
    let _ = crate::parse_descriptor::lex_placeholders(&descriptor)?;
    Err(ToolkitError::DescriptorParse(
        "descriptor mode is not yet wired in v0.3 Phase B; Phase C will land the synthesis path"
            .into(),
    ))
}
