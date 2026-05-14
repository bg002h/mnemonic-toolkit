//! `mnemonic verify-bundle` subcommand.
//!
//! Realizes SPEC §2.2 + §5.4. Both full and watch-only emit the
//! fixed 9-element `checks` array in SPEC §5.4 order; watch-only
//! marks entropy + path-rederivation `skipped` (SPEC §2.2.2). Check
//! failures stay in §5.4 with `result: "mismatch"` per the §5.4
//! routing rule (only pre-decode failures escape to the §5.5 error
//! envelope).

use crate::error::ToolkitError;
use crate::format::{chunk_set_id_extract, VerifyBundleJson, VerifyCheck};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::MultisigPathFamily;
use crate::slot_input::SlotInput;
use crate::template::CliTemplate;
use clap::Args;
use mnemonic_toolkit::mlock::pin_pages_for;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Args, Debug, Clone)]
pub struct VerifyBundleArgs {
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

    /// SPEC v0.9.0 §1 item 1 — read `--passphrase` from stdin (raw,
    /// preserving NULL bytes; strips a single trailing `\r?\n`).
    /// Mutually exclusive with `--passphrase` AND with any
    /// `--slot @N.<secret>=-` (single stdin per invocation).
    /// Mirrors `convert.rs:181` precedent.
    #[arg(long = "passphrase-stdin", conflicts_with = "passphrase")]
    pub passphrase_stdin: bool,

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
    /// explicit --ms1/--mk1/--md1 triplet. Re-derivation flags (`--slot`)
    /// are STILL required to compute the expected bundle.
    #[arg(long = "bundle-json", conflicts_with_all = ["ms1", "mk1", "md1"])]
    pub bundle_json: Option<PathBuf>,

    #[arg(long)]
    pub json: bool,

    /// v0.2 multisig path family (default: bip87).
    #[arg(long = "multisig-path-family", value_enum)]
    pub multisig_path_family: Option<MultisigPathFamily>,

    /// v0.2 privacy mode: expect mk1 omits master fingerprint.
    #[arg(long, default_value = "false")]
    pub privacy_preserving: bool,

    /// v0.2 multisig threshold K (1 ≤ K ≤ N ≤ 16).
    #[arg(long)]
    pub threshold: Option<u8>,

    /// v0.4 unified slot input. Repeating flag — see `BundleArgs::slot`
    /// for grammar.
    #[arg(long = "slot", action = clap::ArgAction::Append, value_parser = crate::slot_input::parse_slot_input)]
    pub slot: Vec<SlotInput>,
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

    // SPEC v0.9.0 §1 item 1 — argv-leakage closure. Run BEFORE bundle-json
    // intake so the advisory fires uniformly even on the synthetic-args
    // intake path.
    emit_secret_in_argv_advisories(args, stderr);
    let stdin_synth;
    let args: &VerifyBundleArgs = if needs_stdin_substitution(args) {
        stdin_synth = apply_stdin_substitutions(args, stdin)?;
        &stdin_synth
    } else {
        args
    };

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

    // Cycle B Phase 3a Site 1 — pin argv-string secret heap pages for the
    // remainder of the handler scope. Lands AFTER both apply_stdin_substitutions
    // and load_bundle_json_into_args returns so the pin covers the final
    // post-substitution buffers (per SPEC §4 P3a).
    let _pin_passphrase = args
        .passphrase
        .as_ref()
        .map(|p| pin_pages_for(p.as_bytes()));
    let _pin_slot_values: Vec<_> = args
        .slot
        .iter()
        .map(|s| pin_pages_for(s.value.as_bytes()))
        .collect();

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

    let multisig = args.template_unchecked().is_multisig();

    if args.threshold.is_some() && !multisig {
        return Err(ToolkitError::ModeViolation {
            mode: "single-sig",
            flag: "--threshold",
            message: mode_text::THRESHOLD_WITHOUT_MULTISIG,
        });
    }
    if args.multisig_path_family.is_some() && !multisig {
        return Err(ToolkitError::ModeViolation {
            mode: "single-sig",
            flag: "--multisig-path-family",
            message: mode_text::PATH_FAMILY_WITHOUT_MULTISIG,
        });
    }

    crate::slot_input::validate_slot_set(&args.slot)?;
    let n = args
        .slot
        .iter()
        .map(|s| s.index as usize)
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);
    let template_str = args.template.map(|t| t.human_name());
    let multisig_template_name = template_str.filter(|_| multisig);
    crate::bundle_unified::pre_check_threshold(args.threshold, n, multisig_template_name)?;
    if let Some(t) = args.template {
        crate::bundle_unified::pre_check_template_n(t.human_name(), t.is_multisig(), n)?;
    }

    let mut checks: Vec<VerifyCheck> = Vec::new();
    if multisig {
        run_multisig(args, &mut checks, stderr)?;
    } else {
        let secret_bearing_at_0 = args
            .slot
            .iter()
            .any(|s| s.index == 0 && s.subkey.is_secret_bearing());
        if secret_bearing_at_0 {
            run_full(args, &mut checks)?;
        } else {
            run_watch_only(args, &mut checks, stderr)?;
        }
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
            let status = if c.passed { "ok" } else { "fail" };
            if c.detail.is_empty() {
                writeln!(stdout, "{}: {}", c.name, status).ok();
            } else {
                writeln!(stdout, "{}: {} {}", c.name, status, c.detail).ok();
            }
        }
        writeln!(stdout, "result: {}", result).ok();
    }

    Ok(if any_fail { 4 } else { 0 })
}

fn run_full(
    args: &VerifyBundleArgs,
    checks: &mut Vec<VerifyCheck>,
) -> Result<(), ToolkitError> {
    let template = args.template_unchecked();
    // verify-bundle does not surface SLIP-0132 input-normalization signals.
    // SPEC `design/SPEC_convert_v0_6.md` §11 v0.7 amendment (Option B): checker
    // semantics suppress info-lines to avoid breaking script callers parsing
    // VERIFIED/MISMATCH stderr line-by-line.
    let (resolved, _slip0132_signals) = crate::cmd::bundle::resolve_slots(
        &args.slot,
        template,
        args.network,
        args.account,
        args.language,
        args.passphrase.as_deref(),
    )?;
    let n = resolved.len() as u8;
    let threshold = args.threshold.unwrap_or(n);
    let expected = crate::synthesize::synthesize_unified(
        &resolved,
        template,
        threshold,
        args.network,
        args.privacy_preserving,
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
    // parse error so the user always sees it, even if the supplied xpub is
    // malformed.
    writeln!(
        stderr,
        "warning: watch-only verify-bundle does not verify --slot @0.xpub= is actually at the"
    )
    .ok();
    writeln!(
        stderr,
        "warning: claimed BIP path m/<purpose>'/<coin>'/0' (no master seed available"
    )
    .ok();
    writeln!(
        stderr,
        "warning: for re-derivation). Use --slot @0.phrase= mode for end-to-end verification."
    )
    .ok();

    let template = args.template_unchecked();
    // verify-bundle does not surface SLIP-0132 input-normalization signals.
    // SPEC `design/SPEC_convert_v0_6.md` §11 v0.7 amendment (Option B): checker
    // semantics suppress info-lines to avoid breaking script callers parsing
    // VERIFIED/MISMATCH stderr line-by-line.
    let (resolved, _slip0132_signals) = crate::cmd::bundle::resolve_slots(
        &args.slot,
        template,
        args.network,
        args.account,
        args.language,
        args.passphrase.as_deref(),
    )?;
    let n = resolved.len() as u8;
    let threshold = args.threshold.unwrap_or(n);
    let expected = crate::synthesize::synthesize_unified(
        &resolved,
        template,
        threshold,
        args.network,
        args.privacy_preserving,
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
    let any_secret = args
        .slot
        .iter()
        .any(|s| s.subkey.is_secret_bearing());
    let any_watch_only = args
        .slot
        .iter()
        .any(|s| s.subkey.is_watch_only());
    let watch_only_multi = !any_secret && any_watch_only;

    if watch_only_multi {
        // SPEC §2.2.2 multisig watch-only stderr warning.
        writeln!(
            stderr,
            "warning: watch-only multisig verify-bundle does not verify --slot xpubs are at the"
        )
        .ok();
        writeln!(
            stderr,
            "warning: claimed BIP path (no per-cosigner master seed available for re-derivation)."
        )
        .ok();
        writeln!(
            stderr,
            "warning: Use --slot @N.phrase= mode for end-to-end verification of self-multisig backups."
        )
        .ok();
    }

    let template = args.template_unchecked();
    // verify-bundle does not surface SLIP-0132 input-normalization signals.
    // SPEC `design/SPEC_convert_v0_6.md` §11 v0.7 amendment (Option B): checker
    // semantics suppress info-lines to avoid breaking script callers parsing
    // VERIFIED/MISMATCH stderr line-by-line.
    let (resolved, _slip0132_signals) = crate::cmd::bundle::resolve_slots(
        &args.slot,
        template,
        args.network,
        args.account,
        args.language,
        args.passphrase.as_deref(),
    )?;
    let n = resolved.len() as u8;
    let threshold = args.threshold.unwrap_or(1);
    let expected = crate::synthesize::synthesize_unified(
        &resolved,
        template,
        threshold,
        args.network,
        args.privacy_preserving,
    )?;
    let _ = n;

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
    _stdin: &mut dyn std::io::Read,
    stdout: &mut W,
) -> Result<u8, ToolkitError> {
    use crate::parse_descriptor::{
        check_key_vector_distinctness, lex_placeholders, parse_descriptor, resolve_placeholders,
        DescriptorBinding, ParsedFingerprint, ParsedKey,
    };
    use crate::synthesize::{synthesize_descriptor, xpub_to_65, CosignerKeyInfo};

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
    let descriptor_resolved =
        resolve_placeholders(&occs).map_err(|e| ToolkitError::DescriptorReparseFailed {
            detail: e.message(),
        })?;
    let n = descriptor_resolved.n as usize;

    crate::slot_input::validate_slot_set(&args.slot)?;
    let template = args
        .template
        .unwrap_or(crate::template::CliTemplate::Bip84);
    // verify-bundle does not surface SLIP-0132 input-normalization signals.
    // SPEC `design/SPEC_convert_v0_6.md` §11 v0.7 amendment (Option B): checker
    // semantics suppress info-lines to avoid breaking script callers parsing
    // VERIFIED/MISMATCH stderr line-by-line.
    let (resolved_slots, _slip0132_signals) = crate::cmd::bundle::resolve_slots(
        &args.slot,
        template,
        args.network,
        args.account,
        args.language,
        args.passphrase.as_deref(),
    )?;

    if resolved_slots.len() != n {
        return Err(ToolkitError::DescriptorReparseFailed {
            detail: format!(
                "descriptor has n={n} placeholders but --slot vec covers {} slots",
                resolved_slots.len()
            ),
        });
    }

    let mut keys: Vec<ParsedKey> = Vec::with_capacity(n);
    let mut fingerprints: Vec<ParsedFingerprint> = Vec::with_capacity(n);
    let mut cosigners: Vec<CosignerKeyInfo> = Vec::with_capacity(n);
    // SPEC v0.9.0 §1 item 2 — entropy_at_0 is the cloned @0 entropy
    // used downstream for verification; wrap in Zeroizing.
    let mut entropy_at_0: Option<zeroize::Zeroizing<Vec<u8>>> = None;
    for (i, slot) in resolved_slots.iter().enumerate() {
        keys.push(ParsedKey {
            i: i as u8,
            payload: xpub_to_65(&slot.xpub),
        });
        fingerprints.push(ParsedFingerprint {
            i: i as u8,
            fp: slot.fingerprint.to_bytes(),
        });
        let entropy = slot.entropy.clone();
        let entropy_pin = entropy.as_ref().map(|e| Rc::new(pin_pages_for(&e[..])));
        cosigners.push(CosignerKeyInfo {
            xpub: slot.xpub,
            fingerprint: slot.fingerprint,
            path: slot.path.clone(),
            path_raw: slot.path_raw.clone(),
            entropy,
            master_xpub: slot.master_xpub,
            _entropy_pin: entropy_pin,
        });
        if i == 0 {
            // v0.10.1: slot.entropy is now Option<Zeroizing<Vec<u8>>>; its
            // clone matches entropy_at_0's declared type natively. No map.
            entropy_at_0 = slot.entropy.clone();
        }
    }

    let binding = DescriptorBinding {
        keys: keys.clone(),
        fingerprints: fingerprints.clone(),
        cosigners: cosigners.clone(),
    };

    // SPEC §4.11.c symmetric verify-bundle enforcement: re-wrap to the verify-bundle
    // exit-4 variant so v0.2 self-multisig artifacts fail with the §4.11.c stderr.
    if let Err(ToolkitError::Bip388Distinctness { .. }) = check_key_vector_distinctness(&binding) {
        return Err(ToolkitError::Bip388VerifyDistinctness);
    }

    let descriptor = parse_descriptor(&descriptor_str, &keys, &fingerprints)
        .map_err(|e| ToolkitError::DescriptorReparseFailed {
            detail: e.message(),
        })?;
    let expected = synthesize_descriptor(
        &descriptor,
        &cosigners,
        entropy_at_0.as_ref().map(|z| &z[..]),
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
            let status = if c.passed { "ok" } else { "fail" };
            if c.detail.is_empty() {
                writeln!(stdout, "{}: {}", c.name, status).ok();
            } else {
                writeln!(stdout, "{}: {} {}", c.name, status, c.detail).ok();
            }
        }
        writeln!(stdout, "result: {}", result_str).ok();
    }
    Ok(if any_fail { 4 } else { 0 })
}

// ============================================================================
// SPEC v0.9.0 §1 item 1 — argv-leakage closure helpers (mirror bundle.rs)
// ============================================================================

/// Per-occurrence `secret-in-argv` stderr advisory emission for
/// `verify-bundle`. Mirrors `cmd/bundle.rs` shape (one advisory per
/// (flag, slot-index) site).
fn emit_secret_in_argv_advisories<E: std::io::Write>(args: &VerifyBundleArgs, stderr: &mut E) {
    use crate::secret_advisory::secret_in_argv_warning;
    for s in &args.slot {
        if s.subkey.is_secret_bearing() && !s.is_stdin_sentinel() {
            let flag = format!("--slot @{}.{}=", s.index, s.subkey.as_str());
            let alt = format!("--slot @{}.{}=-", s.index, s.subkey.as_str());
            secret_in_argv_warning(stderr, &flag, &alt);
        }
    }
    if args.passphrase.is_some() {
        secret_in_argv_warning(stderr, "--passphrase", "--passphrase-stdin");
    }
}

fn needs_stdin_substitution(args: &VerifyBundleArgs) -> bool {
    args.passphrase_stdin || args.slot.iter().any(|s| s.is_stdin_sentinel())
}

fn apply_stdin_substitutions(
    args: &VerifyBundleArgs,
    stdin: &mut dyn std::io::Read,
) -> Result<VerifyBundleArgs, ToolkitError> {
    let mut owned = args.clone();
    let has_slot_stdin = owned.slot.iter().any(|s| s.is_stdin_sentinel());
    if owned.passphrase_stdin && has_slot_stdin {
        return Err(ToolkitError::BadInput(
            "--passphrase-stdin cannot be used with --slot @N.<secret>=- (single stdin per invocation)"
                .into(),
        ));
    }
    if owned.passphrase_stdin {
        let mut buf = String::new();
        stdin
            .read_to_string(&mut buf)
            .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
        if buf.ends_with('\n') {
            buf.pop();
            if buf.ends_with('\r') {
                buf.pop();
            }
        }
        owned.passphrase = Some(buf);
    } else if has_slot_stdin {
        crate::slot_input::apply_slot_stdin(&mut owned.slot, stdin)?;
    }
    Ok(owned)
}

/// v0.4.3 Phase Q: load a `bundle --json` envelope file and synthesize
/// a VerifyBundleArgs with the extracted ms1/mk1/md1 vecs populated. Other
/// args (re-derivation flags --slot/--phrase/etc) are preserved from the
/// caller's args. v0.5: schema-version peek-and-reject deleted; envelopes
/// that don't match the v0.5 schema-4 shape fail at the underlying field
/// extraction (serde-style errors).
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
/// Per-cosigner mapping outcome. v0.5 SPEC §5.7 mk1-mapping diagnostic.
/// Precedence when multiple modes apply: `XpubNotInPolicy > DecodeFailed > NotSupplied`.
#[derive(Debug)]
enum MappingFailure {
    NotSupplied,
    DecodeFailed(String),
    XpubNotInPolicy,
}

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

    // Group supplied.mk1 by chunk_set_id; remember per-group decode outcome
    // (Ok(card) or Err(message)) so the mapping diagnostic can distinguish
    // DecodeFailed from NotSupplied.
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
    let supplied_decoded: Vec<Result<mk_codec::KeyCard, String>> = groups
        .iter()
        .map(|g| mk_codec::decode(g).map_err(|e| format!("{:?}", e)))
        .collect();

    // Decode supplied.md1 once for cosigner-mapping by tlv.pubkeys.
    let supplied_md1_strs: Vec<&str> = supplied.md1.iter().map(|s| s.as_str()).collect();
    let supplied_md_decoded = md_codec::chunk::reassemble(&supplied_md1_strs);

    // B.2: positional fallback condition refactored to match for clarity.
    let needs_positional_fallback = match supplied_md_decoded.as_ref() {
        Err(_) => true,
        Ok(d) => d.tlv.pubkeys.is_none(),
    };

    // Map decoded supplied groups → cosigner positions, tracking failure modes.
    // B.4: Vec<Result<&KeyCard, MappingFailure>> with precedence enforcement.
    let mut card_for_cosigner: Vec<Result<&mk_codec::KeyCard, MappingFailure>> =
        (0..n).map(|_| Err(MappingFailure::NotSupplied)).collect();

    if !needs_positional_fallback {
        let desc = supplied_md_decoded.as_ref().expect("Ok per needs_positional_fallback");
        let pubkeys = desc.tlv.pubkeys.as_ref().expect("Some per needs_positional_fallback");
        // First pass: place decoded groups into matching cosigner slots by xpub.
        for (gi, decode_res) in supplied_decoded.iter().enumerate() {
            if let Ok(card) = decode_res {
                let want = crate::synthesize::xpub_to_65(&card.xpub);
                // Prefer slot gi if it matches.
                if let Some((_, b)) = pubkeys.get(gi) {
                    if b == &want && matches!(card_for_cosigner[gi], Err(MappingFailure::NotSupplied)) {
                        card_for_cosigner[gi] = Ok(card);
                        continue;
                    }
                }
                // Otherwise scan for first unfilled matching slot.
                if let Some((idx, _)) = pubkeys.iter().find(|(slot, b)| {
                    b == &want && matches!(card_for_cosigner[*slot as usize], Err(MappingFailure::NotSupplied))
                }) {
                    card_for_cosigner[*idx as usize] = Ok(card);
                } else {
                    // Decoded successfully but xpub not in any policy slot.
                    // Promote any NotSupplied slot to XpubNotInPolicy (precedence).
                    for slot in card_for_cosigner.iter_mut() {
                        if matches!(slot, Err(MappingFailure::NotSupplied)) {
                            *slot = Err(MappingFailure::XpubNotInPolicy);
                            break;
                        }
                    }
                }
            }
        }
        // Second pass: any remaining group with DecodeFailed promotes a NotSupplied slot.
        // Precedence: XpubNotInPolicy > DecodeFailed > NotSupplied.
        for decode_res in &supplied_decoded {
            if let Err(msg) = decode_res {
                for slot in card_for_cosigner.iter_mut() {
                    if matches!(slot, Err(MappingFailure::NotSupplied)) {
                        *slot = Err(MappingFailure::DecodeFailed(msg.clone()));
                        break;
                    }
                }
            }
        }
    } else {
        // Positional fallback: position-i decoded card → Ok; per-position decode error → DecodeFailed.
        for (i, slot) in card_for_cosigner.iter_mut().enumerate().take(n) {
            match supplied_decoded.get(i) {
                Some(Ok(c)) => *slot = Ok(c),
                Some(Err(msg)) => *slot = Err(MappingFailure::DecodeFailed(msg.clone())),
                None => {} // stays NotSupplied
            }
        }
    }

    // 6N per-cosigner emission.
    #[allow(clippy::needless_range_loop)]
    for i in 0..n {
        let exp_ms1 = expected.ms1.get(i).map(|s| s.as_str()).unwrap_or("");
        let watch_only_slot = exp_ms1.is_empty();
        let sup_ms1 = supplied.ms1.get(i).map(|s| s.as_str());

        // SPEC §5.7 four-case ms1_decode[i] + ms1_entropy_match[i].
        if watch_only_slot {
            // Case 1: watch-only slot — pass-vacuously regardless of supplied.
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
                    // Case 2: full-mode, supplied present, decodes Ok.
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
                    // Case 3: full-mode, supplied present, decodes Err.
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
            // Case 4: full-mode, supplied absent. v0.5 SPEC §5.7 — passed: false.
            checks.push(VerifyCheck {
                name: format!("ms1_decode[{}]", i),
                passed: false,
                detail: format!("cosigner[{}] ms1 expected (full-mode bundle) but not supplied", i),
                decode_error: Some(format!(
                    "error: ms1[{}] expected (full-mode bundle) but not supplied",
                    i
                )),
                ..Default::default()
            });
            checks.push(VerifyCheck {
                name: format!("ms1_entropy_match[{}]", i),
                passed: false,
                detail: format!("cosigner[{}] ms1 not supplied", i),
                decode_error: Some(format!("skipped: ms1[{}] not supplied", i)),
                ..Default::default()
            });
        }

        // mk1_decode[i] + mk1_xpub_match[i] + mk1_fingerprint_match[i] + mk1_path_match[i].
        let sup_card_result = &card_for_cosigner[i];
        let exp_card = expected_mk1_per_cos.get(i).and_then(|o| o.as_ref());
        match (sup_card_result, exp_card) {
            (Ok(sup), Some(exp)) => {
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
            (Err(failure), _) => {
                // SPEC §5.7 mk1-mapping diagnostic: distinguish three failure modes.
                let (detail, decode_error) = match failure {
                    MappingFailure::NotSupplied => (
                        format!("cosigner[{}] mk1 not supplied", i),
                        format!("skipped: mk1[{}] not supplied", i),
                    ),
                    MappingFailure::DecodeFailed(msg) => (
                        format!("cosigner[{}] mk1 decode failed", i),
                        msg.clone(),
                    ),
                    MappingFailure::XpubNotInPolicy => (
                        format!("cosigner[{}] supplied mk1 card xpub absent from descriptor policy", i),
                        "supplied mk1 card xpub absent from descriptor policy".to_string(),
                    ),
                };
                checks.push(VerifyCheck {
                    name: format!("mk1_decode[{}]", i),
                    passed: false,
                    detail,
                    decode_error: Some(decode_error),
                    ..Default::default()
                });
                // Cascade-skip dependent checks: passed=true (vacuous-skip; no oracle).
                for nm in &["mk1_xpub_match", "mk1_fingerprint_match", "mk1_path_match"] {
                    checks.push(VerifyCheck {
                        name: format!("{}[{}]", nm, i),
                        passed: true,
                        detail: format!("cosigner[{}] mk1 decode failed; cannot evaluate", i),
                        decode_error: Some(format!("skipped: mk1[{}] decode failed", i)),
                        ..Default::default()
                    });
                }
            }
            (Ok(_), None) => {
                // Expected card unavailable (legacy MkField::Single beyond i=0): treat as
                // unknown — supplied card decoded but no comparison oracle.
                checks.push(VerifyCheck {
                    name: format!("mk1_decode[{}]", i),
                    passed: true,
                    detail: format!("cosigner[{}] mk1 decoded; no expected oracle", i),
                    ..Default::default()
                });
                for nm in &["mk1_xpub_match", "mk1_fingerprint_match", "mk1_path_match"] {
                    checks.push(VerifyCheck {
                        name: format!("{}[{}]", nm, i),
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
                // md1_xpub_match (B.3: SPEC §5.7 multiset semantics, sort-then-compare).
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
                let mut exp_sorted = exp_pubs.clone();
                let mut act_sorted = act_pubs.clone();
                exp_sorted.sort();
                act_sorted.sort();
                let pubkeys_match = exp_sorted == act_sorted;
                if pubkeys_match {
                    checks.push(VerifyCheck {
                        name: "md1_xpub_match".into(),
                        passed: true,
                        detail: format!("all {} pubkeys match expected (multiset)", exp_pubs.len()),
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
    use std::str::FromStr;

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

    #[test]
    fn helper_multisig_full_emits_3plus6n_checks_in_spec_order() {
        // B.1: full-mode multisig fixture. Reuses watch-only synthesis for the
        // mk1+md1 (distinct cosigners → distinct chunk_set_ids → grouping works)
        // then manually populates expected.ms1 with two distinct non-empty ms1
        // strings derived from synthesize_full(seed_a/seed_b). The unit-test
        // scope is emit_multisig_checks behavior in isolation, not synthesis.
        use crate::parse::{CosignerSpec, MultisigPathFamily};
        use crate::synthesize::{synthesize_full, synthesize_multisig_watch_only};
        use bitcoin::bip32::DerivationPath;
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str("m/48'/0'/0'/2'").unwrap();
        let m_a = Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let entropy_a = m_a.to_entropy();
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
        let entropy_b = m_b.to_entropy();
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
        let mut expected = synthesize_multisig_watch_only(
            &cosigners,
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            0,
            MultisigPathFamily::default(),
            false,
        )
        .unwrap();
        // Manually populate per-cosigner ms1 with non-empty strings (full-mode shape).
        let bundle_a = synthesize_full(
            &entropy_a, fp_a, xpub_a, CliTemplate::Bip84, CliNetwork::Mainnet, 0,
        )
        .unwrap();
        let bundle_b = synthesize_full(
            &entropy_b, fp_b, xpub_b, CliTemplate::Bip84, CliNetwork::Mainnet, 0,
        )
        .unwrap();
        expected.ms1 = vec![bundle_a.ms1[0].clone(), bundle_b.ms1[0].clone()];
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
        assert_eq!(checks.len(), 6 * n + 3, "multisig must emit 3+6N checks (N={n})");
        // Substantive ms1 happy-path: case 2 (decodes Ok + byte-equal) for both slots.
        for i in 0..n {
            let dec = checks.iter().find(|c| c.name == format!("ms1_decode[{i}]")).unwrap();
            assert!(dec.passed, "case 2 ms1_decode[{i}] must pass");
            let mat = checks
                .iter()
                .find(|c| c.name == format!("ms1_entropy_match[{i}]"))
                .unwrap();
            assert!(mat.passed, "case 2 ms1_entropy_match[{i}] must pass on byte-identical");
        }
    }

    #[test]
    fn helper_multisig_missing_ms1_emits_passed_false_per_spec_5_7_case_4() {
        // B.5: SPEC §5.7 case 4 — full-mode bundle with no supplied ms1 → passed=false.
        use crate::parse::{CosignerSpec, MultisigPathFamily};
        use crate::synthesize::{synthesize_full, synthesize_multisig_watch_only};
        use bitcoin::bip32::DerivationPath;
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str("m/48'/0'/0'/2'").unwrap();
        let m_a = Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let entropy_a = m_a.to_entropy();
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
        let entropy_b = m_b.to_entropy();
        let seed_b = m_b.to_seed("");
        let master_b = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed_b).unwrap();
        let xpriv_b = master_b.derive_priv(&secp, &path).unwrap();
        let xpub_b = Xpub::from_priv(&secp, &xpriv_b);
        let fp_b = master_b.fingerprint(&secp);
        let cosigners = vec![
            CosignerSpec { xpub: xpub_a, master_fingerprint: fp_a, path: Some(path.clone()) },
            CosignerSpec { xpub: xpub_b, master_fingerprint: fp_b, path: Some(path.clone()) },
        ];
        let mut expected = synthesize_multisig_watch_only(
            &cosigners,
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            0,
            MultisigPathFamily::default(),
            false,
        )
        .unwrap();
        let bundle_a = synthesize_full(
            &entropy_a, fp_a, xpub_a, CliTemplate::Bip84, CliNetwork::Mainnet, 0,
        )
        .unwrap();
        let bundle_b = synthesize_full(
            &entropy_b, fp_b, xpub_b, CliTemplate::Bip84, CliNetwork::Mainnet, 0,
        )
        .unwrap();
        expected.ms1 = vec![bundle_a.ms1[0].clone(), bundle_b.ms1[0].clone()];
        // Supply EMPTY ms1 to trigger case 4.
        let supplied_ms1: Vec<String> = vec![];
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
        for i in 0..2 {
            let dec = checks.iter().find(|c| c.name == format!("ms1_decode[{i}]")).unwrap();
            assert!(!dec.passed, "case 4 ms1_decode[{i}] must fail (passed=false)");
            assert_eq!(
                dec.decode_error.as_deref().unwrap(),
                &format!("error: ms1[{i}] expected (full-mode bundle) but not supplied")
            );
            let mat = checks
                .iter()
                .find(|c| c.name == format!("ms1_entropy_match[{i}]"))
                .unwrap();
            assert!(!mat.passed, "case 4 ms1_entropy_match[{i}] must fail");
        }
    }
}
