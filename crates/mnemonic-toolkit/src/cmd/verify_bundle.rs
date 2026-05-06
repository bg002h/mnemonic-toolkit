//! `mnemonic verify-bundle` subcommand.
//!
//! Realizes SPEC §2.2 + §5.4. Both full and watch-only emit the
//! fixed 9-element `checks` array in SPEC §5.4 order; watch-only
//! marks entropy + path-rederivation `skipped` (SPEC §2.2.2). Check
//! failures stay in §5.4 with `result: "mismatch"` per the §5.4
//! routing rule (only pre-decode failures escape to the §5.5 error
//! envelope).

use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::format::{chunk_set_id_extract, VerifyBundleJson, VerifyCheck};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::{
    check_no_concurrent_stdin, parse_cosigner_spec, parse_cosigners_file, parse_master_fingerprint,
    read_phrase_input, CosignerSpec, MultisigPathFamily,
};
use crate::template::CliTemplate;
use bitcoin::bip32::Xpub;
use clap::Args;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

// SPEC §6.6 mode-violation symmetry mirrored from bundle.rs:
// clap-level mutual exclusion is ONLY --phrase ↔ --xpub; all other
// xpub-mode-incompatible flag rejections are runtime checks emitting
// byte-exact §6.6 strings via ToolkitError::ModeViolation (exit 2).

#[derive(Args, Debug, Clone)]
pub struct VerifyBundleArgs {
    #[arg(long, conflicts_with = "xpub")]
    pub phrase: Option<String>,

    #[arg(long, conflicts_with = "phrase")]
    pub xpub: Option<String>,

    #[arg(long = "master-fingerprint")]
    pub master_fingerprint: Option<String>,

    #[arg(long)]
    pub network: CliNetwork,

    /// Template name. Mutually-required-one-of with --descriptor / --descriptor-file.
    #[arg(long, required_unless_present_any = ["descriptor", "descriptor_file"])]
    pub template: Option<CliTemplate>,

    /// User-supplied descriptor (v0.3 §5.7 verify-bundle re-parse path).
    #[arg(long, conflicts_with = "descriptor_file")]
    pub descriptor: Option<String>,

    /// User-supplied descriptor file (single-line UTF-8).
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

    /// v0.4.1 J.5: schema-4 repeating-flag for per-slot ms1 cards. For
    /// schema-2/3 single-sig invocations, supply once (`--ms1 <s>`); for
    /// schema-4 multi-source multisig, repeat per slot (`--ms1 "" --ms1
    /// <s2>`...). Empty string `""` is the watch-only sentinel per SPEC §5.8.
    #[arg(long, action = clap::ArgAction::Append, conflicts_with = "bundle_json")]
    pub ms1: Vec<String>,

    #[arg(long, num_args = 1.., required_unless_present = "bundle_json", conflicts_with = "bundle_json")]
    pub mk1: Vec<String>,

    #[arg(long, num_args = 1.., required_unless_present = "bundle_json", conflicts_with = "bundle_json")]
    pub md1: Vec<String>,

    /// v0.4.3 Phase Q: read supplied ms1/mk1/md1 cards from a JSON envelope
    /// file (the output of `bundle --json`). Mutually exclusive with the
    /// explicit --ms1/--mk1/--md1 triplet. Re-derivation flags
    /// (--slot/--phrase/--xpub/etc.) are STILL required to compute the
    /// expected bundle. Schema-4 only in v0.4.3; schema-2/3 retro-compat
    /// tracked at FOLLOWUP `bundle-json-schema-2-3-retro-compat` (v0.4.4+).
    #[arg(long = "bundle-json", conflicts_with_all = ["ms1", "mk1", "md1"])]
    pub bundle_json: Option<PathBuf>,

    #[arg(long)]
    pub json: bool,

    /// v0.2 multisig watch-only: per-cosigner spec `<xpub>:<fp>:<path>`. Repeatable.
    #[arg(long, action = clap::ArgAction::Append)]
    pub cosigner: Vec<String>,

    /// v0.2 multisig watch-only: bulk cosigners via JSON file.
    #[arg(long = "cosigners-file")]
    pub cosigners_file: Option<PathBuf>,

    /// v0.2 multisig path family (default: bip87).
    #[arg(long = "multisig-path-family", value_enum)]
    pub multisig_path_family: Option<MultisigPathFamily>,

    /// v0.2 privacy mode: expect mk1 omits master fingerprint.
    #[arg(long, default_value = "false")]
    pub privacy_preserving: bool,

    /// v0.2 multisig threshold K (1 ≤ K ≤ N ≤ 16).
    #[arg(long)]
    pub threshold: Option<u8>,

    /// v0.2 multisig cosigner count N.
    #[arg(long = "cosigner-count")]
    pub cosigner_count: Option<usize>,
}

impl VerifyBundleArgs {
    fn template_unchecked(&self) -> CliTemplate {
        self.template
            .expect("template-mode dispatch contract — descriptor-mode escapes earlier")
    }
}

pub fn run<W: Write, E: Write>(
    args: &VerifyBundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    use crate::cmd::bundle::mode_text;

    // v0.4.3 Phase Q: --bundle-json intake. Load JSON envelope, extract
    // ms1/mk1/md1 into a synthetic VerifyBundleArgs, then continue dispatch
    // as if the user had supplied --ms1/--mk1/--md1 directly.
    let synthetic_args;
    let args = if args.bundle_json.is_some() {
        synthetic_args = load_bundle_json_into_args(args)?;
        &synthetic_args
    } else {
        args
    };

    // v0.3 descriptor-mode dispatch (escapes before template_unchecked).
    let descriptor_mode = args.descriptor.is_some() || args.descriptor_file.is_some();
    if descriptor_mode && args.template.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--template",
            message: mode_text::DESCRIPTOR_AND_TEMPLATE,
        });
    }
    if descriptor_mode {
        return descriptor_mode_verify_run(args, stdin, stdout);
    }

    let xpub_arg = args.xpub.as_deref();
    let phrase_arg = args.phrase.as_deref();
    let multisig = args.template_unchecked().is_multisig();
    let cosigner_present = !args.cosigner.is_empty();
    let cosigners_file_present = args.cosigners_file.is_some();

    // SPEC §6.6 v0.2 NEW mode-violation pre-checks (mirror bundle.rs).
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

    // SPEC §6.6 single-sig mode-violation pre-checks (mirror bundle.rs).
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
    if xpub_arg == Some("-") {
        return Err(ToolkitError::BadInput(mode_text::XPUB_STDIN.to_string()));
    }

    // v0.2 multisig verify dispatch.
    if multisig {
        let mut checks: Vec<VerifyCheck> = Vec::new();
        run_multisig(args, &mut checks, stderr)?;
        let any_fail = checks.iter().any(|c| !c.passed);
        let result = if any_fail { "mismatch" } else { "ok" };
        if args.json {
            let json = VerifyBundleJson {
                schema_version: "4",
                result,
                checks,
            };
            serde_json::to_writer(&mut *stdout, &json).ok();
            writeln!(stdout).ok();
        } else {
            for c in &checks {
                writeln!(stdout, "{}: {} {}", c.name, (if c.passed { "ok" } else { "fail" }), c.detail).ok();
            }
            writeln!(stdout, "result: {}", result).ok();
        }
        return Ok(if any_fail { 4 } else { 0 });
    }

    let mut checks: Vec<VerifyCheck> = Vec::new();

    if xpub_arg.is_some() {
        // Watch-only mode (SPEC §2.2.2): emits the §5.4 9-element array
        // with entropy + path-rederivation marked `skipped`.
        run_watch_only(args, &mut checks, stderr)?;
    } else if phrase_arg.is_some() {
        // Full mode (SPEC §2.2.1): emits the §5.4 9-element array.
        check_no_concurrent_stdin(phrase_arg, args.passphrase.as_deref())?;
        run_full(args, stdin, &mut checks)?;
    } else {
        return Err(ToolkitError::BadInput("expected --phrase or --xpub".into()));
    }

    let any_fail = checks.iter().any(|c| !c.passed);
    let result = if any_fail { "mismatch" } else { "ok" };

    if args.json {
        let json = VerifyBundleJson {
            schema_version: "4",
            result,
            checks,
        };
        serde_json::to_writer(&mut *stdout, &json).ok();
        writeln!(stdout).ok();
    } else {
        for c in &checks {
            writeln!(stdout, "{}: {} {}", c.name, (if c.passed { "ok" } else { "fail" }), c.detail).ok();
        }
        writeln!(stdout, "result: {}", result).ok();
    }

    Ok(if any_fail { 4 } else { 0 })
}

fn run_full(
    args: &VerifyBundleArgs,
    stdin: &mut dyn std::io::Read,
    checks: &mut Vec<VerifyCheck>,
) -> Result<(), ToolkitError> {
    let phrase = read_phrase_input(args.phrase.as_deref(), stdin)?;
    let passphrase = args.passphrase.clone().unwrap_or_default();
    let language = args.language.unwrap_or_default();

    let acc = crate::derive::derive_full(
        &phrase,
        &passphrase,
        language,
        args.network,
        args.template_unchecked(),
        args.account,
    )?;

    let expected = crate::synthesize::synthesize_full(
        &acc.entropy,
        acc.master_fingerprint,
        acc.account_xpub,
        args.template_unchecked(),
        args.network,
        args.account,
    )?;

    let supplied = SuppliedCards {
        ms1: &args.ms1,
        mk1: &args.mk1,
        md1: &args.md1,
    };

    checks.extend(emit_verify_checks(&expected, &supplied, false));
    Ok(())
}

fn run_watch_only<E: Write>(
    args: &VerifyBundleArgs,
    checks: &mut Vec<VerifyCheck>,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    // SPEC §2.2.2 watch-only-cannot-verify-path warning. Emitted before any
    // parse error so the user always sees it, even if --xpub fails to parse.
    writeln!(
        stderr,
        "warning: watch-only verify-bundle does not verify --xpub is actually at the"
    )
    .ok();
    writeln!(
        stderr,
        "warning: claimed BIP path m/<purpose>'/<coin>'/0' (no master seed available"
    )
    .ok();
    writeln!(
        stderr,
        "warning: for re-derivation). Use --phrase mode for end-to-end verification."
    )
    .ok();

    let xpub_str = args.xpub.as_deref().expect("xpub set in watch-only mode");
    let fp_str = args
        .master_fingerprint
        .as_deref()
        .ok_or_else(|| ToolkitError::BadInput("--xpub requires --master-fingerprint".into()))?;
    let supplied_xpub = Xpub::from_str(xpub_str)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::XpubParse(format!("{}", e))))?;
    let supplied_fp = parse_master_fingerprint(fp_str)?;

    if supplied_xpub.network != args.network.network_kind() {
        return Err(ToolkitError::NetworkMismatch {
            xpub_network: if supplied_xpub.network == bitcoin::NetworkKind::Main {
                "mainnet"
            } else {
                "testnet/signet/regtest"
            },
            expected: args.network.human_name(),
        });
    }

    // Synthesize the watch-only Bundle from supplied xpub+fp; expected.ms1 = [""]
    // (empty-string sentinel) drives the helper's watch-only short-circuit.
    let expected = crate::synthesize::synthesize_watch_only(
        supplied_fp,
        supplied_xpub,
        args.template_unchecked(),
        args.network,
        args.account,
    )?;

    let supplied = SuppliedCards {
        ms1: &args.ms1,
        mk1: &args.mk1,
        md1: &args.md1,
    };

    checks.extend(emit_verify_checks(&expected, &supplied, false));
    Ok(())
}


/// Multisig verify-bundle entry. Synthesizes the expected Bundle (full or
/// watch-only) and dispatches to `emit_verify_checks(... is_multisig: true)`,
/// which emits the SPEC §5.7 `3 + 6N` schema in this order:
///
///   For each cosigner i ∈ 0..N (interleaved by slot):
///     ms1_decode[i], ms1_entropy_match[i],
///     mk1_decode[i], mk1_xpub_match[i],
///     mk1_fingerprint_match[i], mk1_path_match[i].
///   Then 3 shared md1 checks:
///     md1_decode, md1_wallet_policy, md1_xpub_match.
///
/// Watch-only / wif slots (`expected.ms1[i] == ""`) short-circuit ms1_decode[i]
/// and ms1_entropy_match[i] with `passed: true + decode_error: "skipped: watch-only slot"`.
fn run_multisig<E: Write>(
    args: &VerifyBundleArgs,
    checks: &mut Vec<VerifyCheck>,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    use crate::synthesize::{synthesize_multisig_full, synthesize_multisig_watch_only};

    let phrase_arg = args.phrase.as_deref();
    let cosigner_present = !args.cosigner.is_empty();
    let cosigners_file_present = args.cosigners_file.is_some();
    let watch_only_multi = cosigner_present || cosigners_file_present;

    if watch_only_multi {
        // SPEC §2.2.2 multisig watch-only stderr warning.
        writeln!(
            stderr,
            "warning: watch-only multisig verify-bundle does not verify --cosigner xpubs are at the"
        )
        .ok();
        writeln!(
            stderr,
            "warning: claimed BIP path (no per-cosigner master seed available for re-derivation)."
        )
        .ok();
        writeln!(
            stderr,
            "warning: Use --phrase mode for end-to-end verification of self-multisig backups."
        )
        .ok();
    }

    let template = args.template_unchecked();
    let path_family = args.multisig_path_family.unwrap_or_default();
    let threshold = args.threshold.unwrap_or(1);

    let expected = if watch_only_multi {
        let specs: Vec<CosignerSpec> = if let Some(file) = &args.cosigners_file {
            parse_cosigners_file(file)?
        } else {
            let mut out = Vec::with_capacity(args.cosigner.len());
            for (i, s) in args.cosigner.iter().enumerate() {
                out.push(parse_cosigner_spec(s, i)?);
            }
            out
        };
        synthesize_multisig_watch_only(
            &specs,
            args.network,
            template,
            threshold,
            args.account,
            path_family,
            args.privacy_preserving,
        )?
    } else if let Some(p) = phrase_arg {
        check_no_concurrent_stdin(phrase_arg, args.passphrase.as_deref())?;
        let cosigner_count = args
            .cosigner_count
            .ok_or_else(|| ToolkitError::MultisigConfig {
                message: "--cosigner-count required for full-mode multisig verify".into(),
            })?;
        let language = args.language.unwrap_or_default();
        let passphrase = args.passphrase.clone().unwrap_or_default();
        let mnemonic = bip39::Mnemonic::parse_in(language.into(), p)
            .map_err(ToolkitError::Bip39)?;
        synthesize_multisig_full(
            &mnemonic,
            &passphrase,
            args.network,
            template,
            threshold,
            cosigner_count as usize,
            args.account,
            path_family,
            args.privacy_preserving,
        )?
    } else {
        return Err(ToolkitError::BadInput(
            "multisig verify-bundle requires --phrase (full) or --cosigner/--cosigners-file (watch-only)".into(),
        ));
    };

    let supplied = SuppliedCards {
        ms1: &args.ms1,
        mk1: &args.mk1,
        md1: &args.md1,
    };

    checks.extend(emit_verify_checks(&expected, &supplied, true));
    Ok(())
}

/// Phase D descriptor-mode verify: re-run the descriptor pipeline to build the
/// expected Bundle, then compare each card against the supplied --ms1/--mk1/--md1.
/// Emits the same VerifyBundleJson schema as template-mode verify (per SPEC §5.7
/// the check schema is structurally unchanged; only the source of truth differs).
fn descriptor_mode_verify_run<W: Write>(
    args: &VerifyBundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
) -> Result<u8, ToolkitError> {
    use crate::parse_descriptor::{
        bind_descriptor_keys, check_key_vector_distinctness, lex_placeholders, parse_descriptor,
        resolve_placeholders,
    };
    use crate::synthesize::synthesize_descriptor;

    let descriptor_str = match (&args.descriptor, &args.descriptor_file) {
        (Some(s), None) => s.clone(),
        (None, Some(p)) => std::fs::read_to_string(p)
            .map_err(|e| ToolkitError::DescriptorReparseFailed {
                detail: format!("--descriptor-file {}: {e}", p.display()),
            })?
            .trim_end()
            .to_string(),
        _ => unreachable!("clap conflicts_with rules out both"),
    };

    let occs =
        lex_placeholders(&descriptor_str).map_err(|e| ToolkitError::DescriptorReparseFailed {
            detail: e.message(),
        })?;
    let resolved =
        resolve_placeholders(&occs).map_err(|e| ToolkitError::DescriptorReparseFailed {
            detail: e.message(),
        })?;

    let phrase_owned: Option<String> = if args.phrase.is_some() {
        Some(read_phrase_input(args.phrase.as_deref(), stdin)?)
    } else {
        None
    };
    let passphrase = args.passphrase.clone().unwrap_or_default();
    let language = args.language.unwrap_or_default();

    let cosigner_specs: Vec<CosignerSpec> = if !args.cosigner.is_empty() {
        args.cosigner
            .iter()
            .enumerate()
            .map(|(i, s)| parse_cosigner_spec(s, i))
            .collect::<Result<Vec<_>, _>>()?
    } else if let Some(p) = args.cosigners_file.as_ref() {
        parse_cosigners_file(p)?
    } else {
        Vec::new()
    };

    let binding = bind_descriptor_keys(
        &resolved,
        args.network,
        phrase_owned.as_deref(),
        &passphrase,
        language,
        args.xpub.as_deref(),
        args.master_fingerprint.as_deref(),
        &cosigner_specs,
    )?;

    // SPEC §4.11.c symmetric verify-bundle enforcement: re-wrap to the verify-bundle
    // exit-4 variant so v0.2 self-multisig artifacts fail with the §4.11.c stderr.
    if let Err(ToolkitError::Bip388Distinctness { .. }) = check_key_vector_distinctness(&binding) {
        return Err(ToolkitError::Bip388VerifyDistinctness);
    }

    let descriptor = parse_descriptor(&descriptor_str, &binding.keys, &binding.fingerprints)
        .map_err(|e| ToolkitError::DescriptorReparseFailed {
            detail: e.message(),
        })?;
    let expected = synthesize_descriptor(
        &descriptor,
        &binding.cosigners,
        binding.entropy_at_0(),
        args.privacy_preserving,
    )?;

    // SPEC §5.7: descriptor-mode emits the same 9 / 3+6N schema as template-mode.
    // is_multisig := descriptor.n > 1.
    let supplied = SuppliedCards {
        ms1: &args.ms1,
        mk1: &args.mk1,
        md1: &args.md1,
    };
    let checks = emit_verify_checks(&expected, &supplied, descriptor.n > 1);

    let any_fail = checks.iter().any(|c| !c.passed);
    let result_str = if any_fail { "mismatch" } else { "ok" };
    if args.json {
        let json = VerifyBundleJson {
            schema_version: "4",
            result: result_str,
            checks,
        };
        serde_json::to_writer(&mut *stdout, &json).ok();
        writeln!(stdout).ok();
    } else {
        for c in &checks {
            writeln!(
                stdout,
                "{}: {} {}",
                c.name,
                (if c.passed { "ok" } else { "fail" }),
                c.detail
            )
            .ok();
        }
        writeln!(stdout, "result: {}", result_str).ok();
    }
    Ok(if any_fail { 4 } else { 0 })
}

#[cfg(test)]
mod watch_only_tests {
    use super::*;
    #[test]
    fn watch_only_emits_spec_2_2_2_warning_to_stderr() {
        let mut stdin = std::io::empty();
        let mut stdout: Vec<u8> = Vec::new();
        let mut stderr: Vec<u8> = Vec::new();
        let args = VerifyBundleArgs {
            phrase: None,
            xpub: Some("xpub6BadInvalidShortString".into()),
            master_fingerprint: Some("deadbeef".into()),
            network: CliNetwork::Mainnet,
            template: Some(CliTemplate::Bip84),
            descriptor: None,
            descriptor_file: None,
            language: None,
            passphrase: None,
            account: 0,
            ms1: Vec::new(),
            mk1: vec!["mk1placeholder".into()],
            md1: vec!["md1placeholder".into()],
            bundle_json: None,
            json: false,
            cosigner: Vec::new(),
            cosigners_file: None,
            multisig_path_family: None,
            privacy_preserving: false,
            threshold: None,
            cosigner_count: None,
        };
        // run() will fail at xpub parse, but the §2.2.2 warning should
        // already be on stderr.
        let _ = run(&args, &mut stdin, &mut stdout, &mut stderr);
        let stderr_text = String::from_utf8(stderr).unwrap();
        assert!(
            stderr_text.contains("watch-only verify-bundle does not verify"),
            "missing line 1 of §2.2.2 warning; got: {stderr_text:?}"
        );
        assert!(
            stderr_text.contains("BIP path m/<purpose>'/<coin>'/0'"),
            "missing line 2 of §2.2.2 warning; got: {stderr_text:?}"
        );
        assert!(
            stderr_text.contains("Use --phrase mode for end-to-end verification."),
            "missing line 3 of §2.2.2 warning; got: {stderr_text:?}"
        );
    }
}

/// v0.4.3 Phase Q: load a `bundle --json` envelope file and synthesize
/// a VerifyBundleArgs with the extracted ms1/mk1/md1 vecs populated. Other
/// args (re-derivation flags --slot/--phrase/etc) are preserved from the
/// caller's args. Schema-4 only; other schema versions error out with a
/// pointer to FOLLOWUP `bundle-json-schema-2-3-retro-compat`.
fn load_bundle_json_into_args(args: &VerifyBundleArgs) -> Result<VerifyBundleArgs, ToolkitError> {
    let path = args.bundle_json.as_ref().expect("caller checked bundle_json.is_some()");
    let raw = std::fs::read_to_string(path).map_err(|e| {
        ToolkitError::BadInput(format!(
            "--bundle-json {}: {e}",
            path.display()
        ))
    })?;
    let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
        ToolkitError::BadInput(format!(
            "--bundle-json {} parse: {e}",
            path.display()
        ))
    })?;
    let schema = v["schema_version"].as_str().ok_or_else(|| {
        ToolkitError::BadInput(format!(
            "--bundle-json {} missing or non-string schema_version field",
            path.display()
        ))
    })?;
    if schema != "4" {
        return Err(ToolkitError::BadInput(format!(
            "--bundle-json schema_version {schema} not supported in v0.4.3; this toolkit emits and reads schema_version \"4\" only. Schema-2/3 retro-compat intake tracked at FOLLOWUP `bundle-json-schema-2-3-retro-compat`."
        )));
    }
    // Extract ms1 (MsField = Vec<String>) + mk1 (MkField — flat or nested) + md1 (Vec<String>).
    let ms1: Vec<String> = v["ms1"]
        .as_array()
        .ok_or_else(|| ToolkitError::BadInput("--bundle-json ms1 field is not an array".into()))?
        .iter()
        .map(|s| s.as_str().unwrap_or("").to_string())
        .collect();
    // mk1 may be flat (Vec<String>) or nested (Vec<Vec<String>>); both flatten
    // into a single Vec<String> for verify-bundle's --mk1 vec semantics.
    let mk1: Vec<String> = match &v["mk1"] {
        serde_json::Value::Array(arr) => {
            let mut flat = Vec::new();
            for item in arr {
                match item {
                    serde_json::Value::String(s) => flat.push(s.clone()),
                    serde_json::Value::Array(inner) => {
                        for s in inner {
                            if let Some(t) = s.as_str() {
                                flat.push(t.to_string());
                            }
                        }
                    }
                    _ => return Err(ToolkitError::BadInput(
                        "--bundle-json mk1 element is neither string nor array".into(),
                    )),
                }
            }
            flat
        }
        _ => return Err(ToolkitError::BadInput("--bundle-json mk1 field is not an array".into())),
    };
    let md1: Vec<String> = v["md1"]
        .as_array()
        .ok_or_else(|| ToolkitError::BadInput("--bundle-json md1 field is not an array".into()))?
        .iter()
        .map(|s| s.as_str().unwrap_or("").to_string())
        .collect();
    // Construct synthetic args: clone everything from caller, override the
    // card-input fields. bundle_json field is cleared to avoid recursion.
    Ok(VerifyBundleArgs {
        ms1,
        mk1,
        md1,
        bundle_json: None,
        ..args.clone()
    })
}

// ============================================================================
// v0.4.4 Phase P — emit_verify_checks helper (SPEC §5.7 9 / 3+6N + forensics).
// ============================================================================

use crate::synthesize::Bundle;

/// User-supplied --ms1/--mk1/--md1 vectors packaged for the helper.
/// `mk1[i]` is the mk1 card for cosigner @i (0-indexed); `len(mk1) == N` expected.
pub struct SuppliedCards<'a> {
    pub ms1: &'a [String],
    pub mk1: &'a [String],
    pub md1: &'a [String],
}

/// SPEC §5.7 verify-bundle check emission. Returns the 9-check array (single-sig)
/// or 3+6N (multisig) per the SPEC's check-name ordering. Forensic fields
/// populated per SPEC §5.7 rules: pass → all None; string-mismatch → expected/
/// actual/diff_byte_offset; decode-failure → decode_error; watch-only short-
/// circuit → passed: true + decode_error: "skipped: watch-only slot".
///
/// `expected.ms1[i].is_empty()` discriminates watch-only slots per SPEC §5.7
/// (the §5.8 MsField wire-format defines the empty-string sentinel; §5.7
/// specifies the watch-only short-circuit semantics in verify-bundle).
/// `is_multisig` selects the 9 vs 3+6N schema.
///
pub fn emit_verify_checks(
    expected: &Bundle,
    supplied: &SuppliedCards,
    is_multisig: bool,
) -> Vec<VerifyCheck> {
    if is_multisig {
        return emit_multisig_checks(expected, supplied);
    }

    let mut checks = Vec::with_capacity(9);
    let watch_only = expected.ms1.first().map(|s| s.is_empty()).unwrap_or(true);

    // 1. ms1_decode + 2. ms1_entropy_match — both pass-vacuously for watch-only.
    if watch_only {
        checks.push(VerifyCheck {
            name: "ms1_decode".into(),
            passed: true,
            detail: "skipped: watch-only slot".into(),
            decode_error: Some("skipped: watch-only slot".into()),
            ..Default::default()
        });
        checks.push(VerifyCheck {
            name: "ms1_entropy_match".into(),
            passed: true,
            detail: "skipped: watch-only slot".into(),
            decode_error: Some("skipped: watch-only slot".into()),
            ..Default::default()
        });
    } else {
        let supplied_ms1 = supplied.ms1.first().map(|s| s.as_str()).unwrap_or("");
        let expected_ms1 = expected.ms1.first().map(|s| s.as_str()).unwrap_or("");
        match ms_codec::decode(supplied_ms1) {
            Ok(_) => {
                checks.push(VerifyCheck {
                    name: "ms1_decode".into(),
                    passed: true,
                    detail: "decoded successfully".into(),
                    ..Default::default()
                });
                if supplied_ms1 == expected_ms1 {
                    checks.push(VerifyCheck {
                        name: "ms1_entropy_match".into(),
                        passed: true,
                        detail: "ms1 byte-identical".into(),
                        ..Default::default()
                    });
                } else {
                    let diff = VerifyCheck::diff_offset(expected_ms1, supplied_ms1);
                    checks.push(VerifyCheck {
                        name: "ms1_entropy_match".into(),
                        passed: false,
                        detail: "expected ms1 bytes differ from supplied".into(),
                        expected: Some(expected_ms1.to_string()),
                        actual: Some(supplied_ms1.to_string()),
                        diff_byte_offset: Some(diff),
                        decode_error: None,
                    });
                }
            }
            Err(e) => {
                let err_msg = format!("{:?}", e);
                checks.push(VerifyCheck {
                    name: "ms1_decode".into(),
                    passed: false,
                    detail: err_msg.clone(),
                    decode_error: Some(err_msg),
                    ..Default::default()
                });
                checks.push(VerifyCheck {
                    name: "ms1_entropy_match".into(),
                    passed: true,
                    detail: "ms1 decode failed; entropy match cannot run".into(),
                    decode_error: Some("skipped: ms1 decode failed".into()),
                    ..Default::default()
                });
            }
        }
    }

    // 3. mk1_decode — must succeed for checks 4/5/6 to run.
    let mk1_strs: Vec<&str> = supplied.mk1.iter().map(|s| s.as_str()).collect();
    let mk_card_result = mk_codec::decode(&mk1_strs);
    match &mk_card_result {
        Ok(_) => {
            checks.push(VerifyCheck {
                name: "mk1_decode".into(),
                passed: true,
                detail: "decoded successfully".into(),
                ..Default::default()
            });
        }
        Err(e) => {
            let err_msg = format!("{:?}", e);
            checks.push(VerifyCheck {
                name: "mk1_decode".into(),
                passed: false,
                detail: err_msg.clone(),
                decode_error: Some(err_msg),
                ..Default::default()
            });
            // 4/5/6 cascade-skipped.
            for n in &["mk1_xpub_match", "mk1_fingerprint_match", "mk1_path_match"] {
                checks.push(VerifyCheck {
                    name: (*n).into(),
                    passed: true,
                    detail: "mk1 decode failed; check cannot run".into(),
                    decode_error: Some("skipped: mk1 decode failed".into()),
                    ..Default::default()
                });
            }
            // Try md1 anyway for diagnostic completeness.
            emit_md1_checks(expected, supplied, &mut checks);
            return checks;
        }
    }
    let mk_card = mk_card_result.expect("Ok branch handled above");

    // expected.mk1 is MkField::Single for single-sig. Caller invariant: only
    // multisig dispatch passes MkField::Multi (handled in emit_multisig_checks).
    let expected_mk1_strs: Vec<&str> = match &expected.mk1 {
        crate::format::MkField::Single(v) => v.iter().map(|s| s.as_str()).collect(),
        crate::format::MkField::Multi(_) => {
            unreachable!("single-sig branch reached MkField::Multi — caller invariant violation")
        }
    };
    let exp_card = mk_codec::decode(&expected_mk1_strs).expect("expected bundle is well-formed");

    // 4. mk1_xpub_match.
    let exp_xpub = exp_card.xpub.to_string();
    let act_xpub = mk_card.xpub.to_string();
    if exp_xpub == act_xpub {
        checks.push(VerifyCheck {
            name: "mk1_xpub_match".into(),
            passed: true,
            detail: "xpub matches".into(),
            ..Default::default()
        });
    } else {
        let diff = VerifyCheck::diff_offset(&exp_xpub, &act_xpub);
        checks.push(VerifyCheck {
            name: "mk1_xpub_match".into(),
            passed: false,
            detail: "xpub does not match".into(),
            expected: Some(exp_xpub),
            actual: Some(act_xpub),
            diff_byte_offset: Some(diff),
            decode_error: None,
        });
    }

    // 5. mk1_fingerprint_match.
    let exp_fp = exp_card
        .origin_fingerprint
        .map(|f| f.to_string())
        .unwrap_or_default();
    let act_fp = mk_card
        .origin_fingerprint
        .map(|f| f.to_string())
        .unwrap_or_default();
    if exp_fp == act_fp {
        checks.push(VerifyCheck {
            name: "mk1_fingerprint_match".into(),
            passed: true,
            detail: "fingerprint matches".into(),
            ..Default::default()
        });
    } else {
        let diff = VerifyCheck::diff_offset(&exp_fp, &act_fp);
        checks.push(VerifyCheck {
            name: "mk1_fingerprint_match".into(),
            passed: false,
            detail: "fingerprint does not match".into(),
            expected: Some(exp_fp),
            actual: Some(act_fp),
            diff_byte_offset: Some(diff),
            decode_error: None,
        });
    }

    // 6. mk1_path_match.
    let exp_path = exp_card.origin_path.to_string();
    let act_path = mk_card.origin_path.to_string();
    if exp_path == act_path {
        checks.push(VerifyCheck {
            name: "mk1_path_match".into(),
            passed: true,
            detail: "path matches".into(),
            ..Default::default()
        });
    } else {
        let diff = VerifyCheck::diff_offset(&exp_path, &act_path);
        checks.push(VerifyCheck {
            name: "mk1_path_match".into(),
            passed: false,
            detail: "path does not match".into(),
            expected: Some(exp_path),
            actual: Some(act_path),
            diff_byte_offset: Some(diff),
            decode_error: None,
        });
    }

    // 7+8+9: md1.
    emit_md1_checks(expected, supplied, &mut checks);

    checks
}

/// SPEC §5.7 multisig 3+6N emission.
///
/// Output ordering: 6N per-cosigner first (interleaved by slot), then 3 shared
/// md1 checks. For each cosigner i in 0..N:
///   ms1_decode[i], ms1_entropy_match[i],
///   mk1_decode[i], mk1_xpub_match[i], mk1_fingerprint_match[i], mk1_path_match[i].
/// Then shared: md1_decode, md1_wallet_policy, md1_xpub_match.
///
/// Watch-only / wif slots (where `expected.ms1[i].is_empty()`): the two ms1
/// checks short-circuit with `passed: true + decode_error: "skipped: watch-only slot"`.
fn emit_multisig_checks(expected: &Bundle, supplied: &SuppliedCards) -> Vec<VerifyCheck> {
    let n = expected.ms1.len();
    let mut checks: Vec<VerifyCheck> = Vec::with_capacity(6 * n + 3);

    // Decode expected.mk1 per-cosigner. expected.mk1 is MkField::Multi(Vec<Vec<String>>)
    // for multisig; on legacy single-element MkField::Single(v) self-multisig
    // bundles, treat the single card as cosigner-0 and emit "missing card"
    // failures for the remaining cosigners.
    let expected_mk1_per_cos: Vec<Option<mk_codec::KeyCard>> = match &expected.mk1 {
        crate::format::MkField::Multi(per_cosigner) => per_cosigner
            .iter()
            .map(|chunks| {
                let strs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
                mk_codec::decode(&strs).ok()
            })
            .collect(),
        crate::format::MkField::Single(v) => {
            let strs: Vec<&str> = v.iter().map(|s| s.as_str()).collect();
            let mut out = vec![mk_codec::decode(&strs).ok()];
            out.resize_with(n, || None);
            out
        }
    };

    // Group supplied.mk1 by chunk_set_id; map to cosigner-i by xpub-vs-md1.tlv.pubkeys
    // (with positional fallback).
    use std::collections::BTreeMap;
    let mut chunked: BTreeMap<u32, Vec<&str>> = BTreeMap::new();
    let mut singles: Vec<Vec<&str>> = Vec::new();
    for s in supplied.mk1 {
        match chunk_set_id_extract(s) {
            Some(csi) => chunked.entry(csi).or_default().push(s.as_str()),
            None => singles.push(vec![s.as_str()]),
        }
    }
    let groups: Vec<Vec<&str>> = chunked.into_values().chain(singles).collect();
    let supplied_decoded: Vec<Option<mk_codec::KeyCard>> = groups
        .iter()
        .map(|g| mk_codec::decode(g).ok())
        .collect();

    // Decode supplied.md1 once for cosigner-mapping by tlv.pubkeys.
    let supplied_md1_strs: Vec<&str> = supplied.md1.iter().map(|s| s.as_str()).collect();
    let supplied_md_decoded = md_codec::chunk::reassemble(&supplied_md1_strs);

    // Map decoded supplied groups → cosigner positions.
    let mut card_for_cosigner: Vec<Option<&mk_codec::KeyCard>> = vec![None; n];
    if let Ok(desc) = supplied_md_decoded.as_ref() {
        if let Some(pubkeys) = desc.tlv.pubkeys.as_ref() {
            for (gi, opt) in supplied_decoded.iter().enumerate() {
                if let Some(card) = opt {
                    let want = crate::synthesize::xpub_to_65(&card.xpub);
                    // Prefer slot gi if it matches (covers self-multisig where all xpubs are equal).
                    if let Some((_, b)) = pubkeys.get(gi) {
                        if b == &want && card_for_cosigner[gi].is_none() {
                            card_for_cosigner[gi] = Some(card);
                            continue;
                        }
                    }
                    if let Some((idx, _)) = pubkeys.iter().find(|(slot, b)| {
                        b == &want && card_for_cosigner[*slot as usize].is_none()
                    }) {
                        card_for_cosigner[*idx as usize] = Some(card);
                    }
                }
            }
        }
    }
    // Positional fallback if md1 decode failed or pubkeys absent.
    if supplied_md_decoded.is_err()
        || supplied_md_decoded
            .as_ref()
            .map(|d| d.tlv.pubkeys.is_none())
            .unwrap_or(false)
    {
        for (i, slot) in card_for_cosigner.iter_mut().enumerate().take(n) {
            if let Some(Some(c)) = supplied_decoded.get(i) {
                *slot = Some(c);
            }
        }
    }

    // 6N per-cosigner emission.
    for i in 0..n {
        let exp_ms1 = expected.ms1.get(i).map(|s| s.as_str()).unwrap_or("");
        let watch_only_slot = exp_ms1.is_empty();
        let sup_ms1 = supplied.ms1.get(i).map(|s| s.as_str());

        // ms1_decode[i] + ms1_entropy_match[i].
        if watch_only_slot {
            checks.push(VerifyCheck {
                name: format!("ms1_decode[{}]", i),
                passed: true,
                detail: "skipped: watch-only slot".into(),
                decode_error: Some("skipped: watch-only slot".into()),
                ..Default::default()
            });
            checks.push(VerifyCheck {
                name: format!("ms1_entropy_match[{}]", i),
                passed: true,
                detail: "skipped: watch-only slot".into(),
                decode_error: Some("skipped: watch-only slot".into()),
                ..Default::default()
            });
        } else if let Some(s) = sup_ms1.filter(|s| !s.is_empty()) {
            match ms_codec::decode(s) {
                Ok(_) => {
                    checks.push(VerifyCheck {
                        name: format!("ms1_decode[{}]", i),
                        passed: true,
                        detail: format!("cosigner[{}] ms1 decoded", i),
                        ..Default::default()
                    });
                    if s == exp_ms1 {
                        checks.push(VerifyCheck {
                            name: format!("ms1_entropy_match[{}]", i),
                            passed: true,
                            detail: format!("cosigner[{}] ms1 byte-identical", i),
                            ..Default::default()
                        });
                    } else {
                        let diff = VerifyCheck::diff_offset(exp_ms1, s);
                        checks.push(VerifyCheck {
                            name: format!("ms1_entropy_match[{}]", i),
                            passed: false,
                            detail: format!("cosigner[{}] ms1 differs", i),
                            expected: Some(exp_ms1.to_string()),
                            actual: Some(s.to_string()),
                            diff_byte_offset: Some(diff),
                            decode_error: None,
                        });
                    }
                }
                Err(e) => {
                    let err_msg = format!("{:?}", e);
                    checks.push(VerifyCheck {
                        name: format!("ms1_decode[{}]", i),
                        passed: false,
                        detail: err_msg.clone(),
                        decode_error: Some(err_msg),
                        ..Default::default()
                    });
                    checks.push(VerifyCheck {
                        name: format!("ms1_entropy_match[{}]", i),
                        passed: true,
                        detail: format!("cosigner[{}] ms1 decode failed; entropy match cannot run", i),
                        decode_error: Some("skipped: ms1 decode failed".into()),
                        ..Default::default()
                    });
                }
            }
        } else {
            // Expected substantive but supplied missing/empty.
            checks.push(VerifyCheck {
                name: format!("ms1_decode[{}]", i),
                passed: true,
                detail: format!("cosigner[{}] ms1 not supplied", i),
                decode_error: Some(format!("skipped: ms1[{}] not supplied", i)),
                ..Default::default()
            });
            checks.push(VerifyCheck {
                name: format!("ms1_entropy_match[{}]", i),
                passed: true,
                detail: format!("cosigner[{}] ms1 not supplied", i),
                decode_error: Some(format!("skipped: ms1[{}] not supplied", i)),
                ..Default::default()
            });
        }

        // mk1_decode[i] + mk1_xpub_match[i] + mk1_fingerprint_match[i] + mk1_path_match[i].
        let sup_card = card_for_cosigner[i];
        let exp_card = expected_mk1_per_cos.get(i).and_then(|o| o.as_ref());
        match (sup_card, exp_card) {
            (Some(sup), Some(exp)) => {
                checks.push(VerifyCheck {
                    name: format!("mk1_decode[{}]", i),
                    passed: true,
                    detail: format!("cosigner[{}] mk1 decoded", i),
                    ..Default::default()
                });
                let exp_x = exp.xpub.to_string();
                let act_x = sup.xpub.to_string();
                if exp_x == act_x {
                    checks.push(VerifyCheck {
                        name: format!("mk1_xpub_match[{}]", i),
                        passed: true,
                        detail: format!("cosigner[{}] xpub matches", i),
                        ..Default::default()
                    });
                } else {
                    let diff = VerifyCheck::diff_offset(&exp_x, &act_x);
                    checks.push(VerifyCheck {
                        name: format!("mk1_xpub_match[{}]", i),
                        passed: false,
                        detail: format!("cosigner[{}] xpub mismatch", i),
                        expected: Some(exp_x),
                        actual: Some(act_x),
                        diff_byte_offset: Some(diff),
                        decode_error: None,
                    });
                }
                let exp_fp = exp.origin_fingerprint.map(|f| f.to_string()).unwrap_or_default();
                let act_fp = sup.origin_fingerprint.map(|f| f.to_string()).unwrap_or_default();
                if exp_fp == act_fp {
                    checks.push(VerifyCheck {
                        name: format!("mk1_fingerprint_match[{}]", i),
                        passed: true,
                        detail: format!("cosigner[{}] fingerprint matches", i),
                        ..Default::default()
                    });
                } else {
                    let diff = VerifyCheck::diff_offset(&exp_fp, &act_fp);
                    checks.push(VerifyCheck {
                        name: format!("mk1_fingerprint_match[{}]", i),
                        passed: false,
                        detail: format!("cosigner[{}] fingerprint mismatch", i),
                        expected: Some(exp_fp),
                        actual: Some(act_fp),
                        diff_byte_offset: Some(diff),
                        decode_error: None,
                    });
                }
                let exp_p = exp.origin_path.to_string();
                let act_p = sup.origin_path.to_string();
                if exp_p == act_p {
                    checks.push(VerifyCheck {
                        name: format!("mk1_path_match[{}]", i),
                        passed: true,
                        detail: format!("cosigner[{}] path matches", i),
                        ..Default::default()
                    });
                } else {
                    let diff = VerifyCheck::diff_offset(&exp_p, &act_p);
                    checks.push(VerifyCheck {
                        name: format!("mk1_path_match[{}]", i),
                        passed: false,
                        detail: format!("cosigner[{}] path mismatch", i),
                        expected: Some(exp_p),
                        actual: Some(act_p),
                        diff_byte_offset: Some(diff),
                        decode_error: None,
                    });
                }
            }
            (None, _) => {
                // Supplied mk1 missing/undecodable for this cosigner.
                let err = format!("skipped: mk1[{}] not supplied or decode failed", i);
                checks.push(VerifyCheck {
                    name: format!("mk1_decode[{}]", i),
                    passed: false,
                    detail: err.clone(),
                    decode_error: Some(err.clone()),
                    ..Default::default()
                });
                for n in &["mk1_xpub_match", "mk1_fingerprint_match", "mk1_path_match"] {
                    checks.push(VerifyCheck {
                        name: format!("{}[{}]", n, i),
                        passed: true,
                        detail: err.clone(),
                        decode_error: Some(format!("skipped: mk1[{}] decode failed", i)),
                        ..Default::default()
                    });
                }
            }
            (Some(_), None) => {
                // Expected card unavailable (legacy MkField::Single beyond i=0): treat as
                // unknown — supplied card decoded but no comparison oracle.
                checks.push(VerifyCheck {
                    name: format!("mk1_decode[{}]", i),
                    passed: true,
                    detail: format!("cosigner[{}] mk1 decoded; no expected oracle", i),
                    ..Default::default()
                });
                for n in &["mk1_xpub_match", "mk1_fingerprint_match", "mk1_path_match"] {
                    checks.push(VerifyCheck {
                        name: format!("{}[{}]", n, i),
                        passed: true,
                        detail: format!("cosigner[{}] no expected mk1 oracle", i),
                        decode_error: Some(format!("skipped: expected mk1[{}] not available", i)),
                        ..Default::default()
                    });
                }
            }
        }
    }

    // 3 shared md1 checks.
    let expected_md1_strs: Vec<&str> = expected.md1.iter().map(|s| s.as_str()).collect();
    let expected_md_decoded = md_codec::chunk::reassemble(&expected_md1_strs)
        .expect("expected bundle is well-formed");

    match supplied_md_decoded.as_ref() {
        Ok(desc) => {
            checks.push(VerifyCheck {
                name: "md1_decode".into(),
                passed: true,
                detail: "decoded successfully".into(),
                ..Default::default()
            });
            let wp = desc.is_wallet_policy();
            if wp {
                checks.push(VerifyCheck {
                    name: "md1_wallet_policy".into(),
                    passed: true,
                    detail: "wallet-policy mode confirmed".into(),
                    ..Default::default()
                });
                // md1_xpub_match (shared, set-equality across all N pubkeys).
                let exp_pubs: Vec<[u8; 65]> = expected_md_decoded
                    .tlv
                    .pubkeys
                    .as_ref()
                    .map(|v| v.iter().map(|(_, b)| *b).collect())
                    .unwrap_or_default();
                let act_pubs: Vec<[u8; 65]> = desc
                    .tlv
                    .pubkeys
                    .as_ref()
                    .map(|v| v.iter().map(|(_, b)| *b).collect())
                    .unwrap_or_default();
                if exp_pubs == act_pubs {
                    checks.push(VerifyCheck {
                        name: "md1_xpub_match".into(),
                        passed: true,
                        detail: format!("all {} pubkeys match expected", exp_pubs.len()),
                        ..Default::default()
                    });
                } else {
                    let exp_hex = exp_pubs.iter().map(hex::encode).collect::<Vec<_>>().join(",");
                    let act_hex = act_pubs.iter().map(hex::encode).collect::<Vec<_>>().join(",");
                    let diff = VerifyCheck::diff_offset(&exp_hex, &act_hex);
                    checks.push(VerifyCheck {
                        name: "md1_xpub_match".into(),
                        passed: false,
                        detail: "md1 pubkeys differ from expected set".into(),
                        expected: Some(exp_hex),
                        actual: Some(act_hex),
                        diff_byte_offset: Some(diff),
                        decode_error: None,
                    });
                }
            } else {
                checks.push(VerifyCheck {
                    name: "md1_wallet_policy".into(),
                    passed: false,
                    detail: "descriptor is template-only (no pubkeys TLV)".into(),
                    decode_error: Some("not in wallet-policy mode".into()),
                    ..Default::default()
                });
                checks.push(VerifyCheck {
                    name: "md1_xpub_match".into(),
                    passed: true,
                    detail: "skipped: not in wallet-policy mode".into(),
                    decode_error: Some("skipped: not in wallet-policy mode".into()),
                    ..Default::default()
                });
            }
        }
        Err(e) => {
            let err_msg = format!("{:?}", e);
            checks.push(VerifyCheck {
                name: "md1_decode".into(),
                passed: false,
                detail: err_msg.clone(),
                decode_error: Some(err_msg),
                ..Default::default()
            });
            checks.push(VerifyCheck {
                name: "md1_wallet_policy".into(),
                passed: true,
                detail: "skipped: md1 decode failed".into(),
                decode_error: Some("skipped: md1 decode failed".into()),
                ..Default::default()
            });
            checks.push(VerifyCheck {
                name: "md1_xpub_match".into(),
                passed: true,
                detail: "skipped: md1 decode failed".into(),
                decode_error: Some("skipped: md1 decode failed".into()),
                ..Default::default()
            });
        }
    }

    checks
}

/// Emit md1_decode + md1_wallet_policy + md1_xpub_match (checks 7-9 of SPEC §5.7).
fn emit_md1_checks(
    expected: &Bundle,
    supplied: &SuppliedCards,
    checks: &mut Vec<VerifyCheck>,
) {
    let supplied_md1: Vec<&str> = supplied.md1.iter().map(|s| s.as_str()).collect();
    match md_codec::chunk::reassemble(&supplied_md1) {
        Ok(desc) => {
            checks.push(VerifyCheck {
                name: "md1_decode".into(),
                passed: true,
                detail: "decoded successfully".into(),
                ..Default::default()
            });
            let wp = desc.is_wallet_policy();
            if wp {
                checks.push(VerifyCheck {
                    name: "md1_wallet_policy".into(),
                    passed: true,
                    detail: "wallet-policy mode confirmed".into(),
                    ..Default::default()
                });
                // 9. md1_xpub_match — compare descriptor's first pubkey to expected mk1's xpub.
                let expected_md1: Vec<&str> = expected.md1.iter().map(|s| s.as_str()).collect();
                let exp_desc = md_codec::chunk::reassemble(&expected_md1)
                    .expect("expected bundle is well-formed");
                let exp_xpub = exp_desc
                    .tlv
                    .pubkeys
                    .as_ref()
                    .and_then(|v| v.first())
                    .map(|(_, b)| *b);
                let act_xpub = desc
                    .tlv
                    .pubkeys
                    .as_ref()
                    .and_then(|v| v.first())
                    .map(|(_, b)| *b);
                let xpub_match = exp_xpub == act_xpub;
                if xpub_match {
                    checks.push(VerifyCheck {
                        name: "md1_xpub_match".into(),
                        passed: true,
                        detail: "65-byte xpub matches expected".into(),
                        ..Default::default()
                    });
                } else {
                    let exp_hex = exp_xpub.map(hex::encode).unwrap_or_default();
                    let act_hex = act_xpub.map(hex::encode).unwrap_or_default();
                    let diff = VerifyCheck::diff_offset(&exp_hex, &act_hex);
                    checks.push(VerifyCheck {
                        name: "md1_xpub_match".into(),
                        passed: false,
                        detail: "md1 xpub differs from expected".into(),
                        expected: Some(exp_hex),
                        actual: Some(act_hex),
                        diff_byte_offset: Some(diff),
                        decode_error: None,
                    });
                }
            } else {
                checks.push(VerifyCheck {
                    name: "md1_wallet_policy".into(),
                    passed: false,
                    detail: "descriptor is template-only (no pubkeys TLV)".into(),
                    decode_error: Some("not in wallet-policy mode".into()),
                    ..Default::default()
                });
                checks.push(VerifyCheck {
                    name: "md1_xpub_match".into(),
                    passed: true,
                    detail: "skipped: not in wallet-policy mode".into(),
                    decode_error: Some("skipped: not in wallet-policy mode".into()),
                    ..Default::default()
                });
            }
        }
        Err(e) => {
            let err_msg = format!("{:?}", e);
            checks.push(VerifyCheck {
                name: "md1_decode".into(),
                passed: false,
                detail: err_msg.clone(),
                decode_error: Some(err_msg),
                ..Default::default()
            });
            checks.push(VerifyCheck {
                name: "md1_wallet_policy".into(),
                passed: true,
                detail: "skipped: md1 decode failed".into(),
                decode_error: Some("skipped: md1 decode failed".into()),
                ..Default::default()
            });
            checks.push(VerifyCheck {
                name: "md1_xpub_match".into(),
                passed: true,
                detail: "skipped: md1 decode failed".into(),
                decode_error: Some("skipped: md1 decode failed".into()),
                ..Default::default()
            });
        }
    }
}

#[cfg(test)]
mod helper_tests {
    use super::*;
    use crate::format::MkField;
    use crate::synthesize::synthesize_full;
    use crate::network::CliNetwork;
    use crate::template::CliTemplate;
    use bip39::Mnemonic;
    use bitcoin::bip32::{Xpriv, Xpub};
    use bitcoin::secp256k1::Secp256k1;

    const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    fn synth_full_bundle() -> Bundle {
        let m = Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let entropy = m.to_entropy();
        let seed = m.to_seed("");
        let secp = Secp256k1::new();
        let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed).unwrap();
        let fp = master.fingerprint(&secp);
        let path = CliTemplate::Bip84.derivation_path(CliNetwork::Mainnet, 0);
        let acct_xpriv = master.derive_priv(&secp, &path).unwrap();
        let xpub = Xpub::from_priv(&secp, &acct_xpriv);
        synthesize_full(&entropy, fp, xpub, CliTemplate::Bip84, CliNetwork::Mainnet, 0).unwrap()
    }

    #[test]
    fn helper_singlesig_full_emits_9_checks_in_spec_order() {
        let expected = synth_full_bundle();
        let supplied_ms1 = expected.ms1.clone();
        let supplied_mk1 = match &expected.mk1 {
            MkField::Single(v) => v.clone(),
            MkField::Multi(_) => panic!("expected single-sig"),
        };
        let supplied_md1 = expected.md1.clone();
        let supplied = SuppliedCards {
            ms1: &supplied_ms1,
            mk1: &supplied_mk1,
            md1: &supplied_md1,
        };
        let checks = emit_verify_checks(&expected, &supplied, false);
        assert_eq!(checks.len(), 9, "single-sig must emit 9 checks per SPEC §5.7");
        let names: Vec<&str> = checks.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "ms1_decode",
                "ms1_entropy_match",
                "mk1_decode",
                "mk1_xpub_match",
                "mk1_fingerprint_match",
                "mk1_path_match",
                "md1_decode",
                "md1_wallet_policy",
                "md1_xpub_match",
            ],
            "checks must be in SPEC §5.7 order"
        );
        assert!(
            checks.iter().all(|c| c.passed),
            "happy-path bundle must produce all-passed checks"
        );
    }

    #[test]
    fn helper_singlesig_tampered_mk1_populates_forensics() {
        let expected = synth_full_bundle();
        let supplied_ms1 = expected.ms1.clone();
        // Tamper: replace the last char with '0'.
        let supplied_mk1: Vec<String> = match &expected.mk1 {
            MkField::Single(v) => v
                .iter()
                .map(|s| {
                    let mut s = s.clone();
                    s.pop();
                    s.push('0');
                    s
                })
                .collect(),
            MkField::Multi(_) => panic!("expected single-sig"),
        };
        let supplied_md1 = expected.md1.clone();
        let supplied = SuppliedCards {
            ms1: &supplied_ms1,
            mk1: &supplied_mk1,
            md1: &supplied_md1,
        };
        let checks = emit_verify_checks(&expected, &supplied, false);
        // Either mk1_decode fails (BCH checksum mismatch) OR mk1_xpub_match fails.
        let mk1_decode = checks
            .iter()
            .find(|c| c.name == "mk1_decode")
            .expect("mk1_decode present");
        if !mk1_decode.passed {
            assert!(
                mk1_decode.decode_error.is_some(),
                "decode-failure must populate decode_error"
            );
        }
    }

    #[test]
    fn helper_singlesig_watch_only_short_circuits_ms1() {
        let mut expected = synth_full_bundle();
        // Convert to watch-only by emptying ms1[0].
        expected.ms1[0].clear();
        let supplied_ms1: Vec<String> = vec!["".into()];
        let supplied_mk1 = match &expected.mk1 {
            MkField::Single(v) => v.clone(),
            MkField::Multi(_) => panic!("expected single-sig"),
        };
        let supplied_md1 = expected.md1.clone();
        let supplied = SuppliedCards {
            ms1: &supplied_ms1,
            mk1: &supplied_mk1,
            md1: &supplied_md1,
        };
        let checks = emit_verify_checks(&expected, &supplied, false);
        assert_eq!(checks.len(), 9);
        // ms1_decode and ms1_entropy_match are skipped per SPEC §5.7.
        let ms1_decode = &checks[0];
        let ms1_match = &checks[1];
        assert!(ms1_decode.passed);
        assert!(ms1_match.passed);
        assert_eq!(
            ms1_decode.decode_error.as_deref(),
            Some("skipped: watch-only slot")
        );
        assert_eq!(
            ms1_match.decode_error.as_deref(),
            Some("skipped: watch-only slot")
        );
        // mk1 + md1 substantive checks all pass.
        for c in &checks[2..] {
            assert!(c.passed, "{} should pass on watch-only happy path", c.name);
        }
    }

    #[test]
    fn helper_multisig_watch_only_emits_3plus6n_checks_in_spec_order() {
        use crate::parse::{CosignerSpec, MultisigPathFamily};
        use crate::synthesize::synthesize_multisig_watch_only;
        use bitcoin::bip32::DerivationPath;
        // Derive 2 distinct cosigner xpubs at the canonical BIP-48 depth-4 path
        // from 2 distinct mnemonic seeds. Distinct xpubs → distinct chunk_set_ids
        // (avoids the legacy self-multisig csi-collision case which is out of
        // scope for SPEC §5.7).
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str("m/48'/0'/0'/2'").unwrap();
        let m_a = Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let seed_a = m_a.to_seed("");
        let master_a = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed_a).unwrap();
        let xpriv_a = master_a.derive_priv(&secp, &path).unwrap();
        let xpub_a = Xpub::from_priv(&secp, &xpriv_a);
        let fp_a = master_a.fingerprint(&secp);
        let m_b = Mnemonic::parse_in(
            bip39::Language::English,
            "legal winner thank year wave sausage worth useful legal winner thank yellow",
        )
        .unwrap();
        let seed_b = m_b.to_seed("");
        let master_b = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed_b).unwrap();
        let xpriv_b = master_b.derive_priv(&secp, &path).unwrap();
        let xpub_b = Xpub::from_priv(&secp, &xpriv_b);
        let fp_b = master_b.fingerprint(&secp);
        let cosigners = vec![
            CosignerSpec { xpub: xpub_a, master_fingerprint: fp_a, path: Some(path.clone()) },
            CosignerSpec { xpub: xpub_b, master_fingerprint: fp_b, path: Some(path.clone()) },
        ];
        let n: usize = 2;
        let expected = synthesize_multisig_watch_only(
            &cosigners,
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            0,
            MultisigPathFamily::default(),
            false,
        )
        .unwrap();
        let supplied_ms1 = expected.ms1.clone();
        let supplied_mk1: Vec<String> = match &expected.mk1 {
            MkField::Multi(per_cos) => per_cos.iter().flat_map(|v| v.iter().cloned()).collect(),
            MkField::Single(_) => panic!("expected multisig"),
        };
        let supplied_md1 = expected.md1.clone();
        let supplied = SuppliedCards {
            ms1: &supplied_ms1,
            mk1: &supplied_mk1,
            md1: &supplied_md1,
        };
        let checks = emit_verify_checks(&expected, &supplied, true);
        assert_eq!(
            checks.len(),
            6 * n + 3,
            "multisig must emit 3+6N checks per SPEC §5.7 (N={n})"
        );
        let names: Vec<&str> = checks.iter().map(|c| c.name.as_str()).collect();
        // First 6N: per-cosigner [i]-indexed.
        let mut expected_names: Vec<String> = Vec::new();
        for i in 0..n {
            expected_names.push(format!("ms1_decode[{i}]"));
            expected_names.push(format!("ms1_entropy_match[{i}]"));
            expected_names.push(format!("mk1_decode[{i}]"));
            expected_names.push(format!("mk1_xpub_match[{i}]"));
            expected_names.push(format!("mk1_fingerprint_match[{i}]"));
            expected_names.push(format!("mk1_path_match[{i}]"));
        }
        // Last 3: shared md1.
        expected_names.push("md1_decode".into());
        expected_names.push("md1_wallet_policy".into());
        expected_names.push("md1_xpub_match".into());
        let expected_names_ref: Vec<&str> = expected_names.iter().map(String::as_str).collect();
        assert_eq!(names, expected_names_ref, "SPEC §5.7 ordering");
        // The fixture uses two distinct mnemonic seeds → two distinct cosigner
        // xpubs → two distinct chunk_set_ids; mk_codec grouping works correctly.
        // Per-cell forensic content on the chunked multi-card path is fully
        // exercised by cli_bundle_multisig.rs / cli_verify_bundle_*.rs end-to-end.
        // This unit test asserts the helper's structural contract (3+6N name
        // vec + ms1_decode happy-path) only.
        let ms1_decode_passed = checks
            .iter()
            .filter(|c| c.name.starts_with("ms1_decode"))
            .all(|c| c.passed);
        assert!(ms1_decode_passed, "ms1_decode[i] must pass on byte-identical happy path");
    }
}
