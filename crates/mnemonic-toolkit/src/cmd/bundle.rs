//! `mnemonic bundle` subcommand.
//!
//! Realizes SPEC §2.1 (full + watch-only modes), §5.1 (multi-section
//! stdout), §5.2 (engraving card stderr), §5.3 (JSON schema).

use crate::error::ToolkitError;
use crate::format::{chunk_5char, chunk_md1, BundleJson, CosignerEntry, MkField, MultisigInfo};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::MultisigPathFamily;
use crate::synthesize::Bundle;
use crate::template::CliTemplate;
use clap::Args;
use mnemonic_toolkit::mlock::pin_pages_for;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Args, Debug, Clone)]
pub struct BundleArgs {
    #[arg(long)]
    pub network: CliNetwork,

    /// Pre-built template name (single-sig or multisig). Mutually-required-one-of
    /// with --descriptor / --descriptor-file / --import-json (clap-level + runtime
    /// pre-check; v0.27.0 added the --import-json branch).
    #[arg(long, required_unless_present_any = ["descriptor", "descriptor_file", "import_json"])]
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

    /// BIP-39 mnemonic-extension passphrase ("25th word"). Empty
    /// (default) is the common case. Mutually exclusive with
    /// `--passphrase-stdin`.
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

    /// Emit a single JSON object on stdout instead of the multi-line
    /// `ms1: ... / mk1: ... / md1: ...` text form.
    #[arg(long)]
    pub json: bool,

    /// Suppress the human-readable engraving-card panel on stderr.
    /// The stdout `ms1` / `mk1` / `md1` output is unchanged. Use for
    /// piping into other tooling.
    #[arg(long = "no-engraving-card")]
    pub no_engraving_card: bool,

    /// v0.2 multisig path family (default: bip87).
    #[arg(long = "multisig-path-family", value_enum)]
    pub multisig_path_family: Option<MultisigPathFamily>,

    /// v0.2 privacy mode: suppress master fingerprint from the
    /// emitted mk1 cards + engraving card.
    #[arg(long, default_value = "false")]
    pub privacy_preserving: bool,

    /// v0.2 self-check: after emission, re-parse the emitted bundle
    /// and verify it round-trips to the same xpubs + policy.
    #[arg(long, default_value = "false")]
    pub self_check: bool,

    /// v0.2 multisig threshold K (1 ≤ K ≤ N ≤ 16). Required for
    /// multisig templates (`wsh-*`, `sh-wsh-*`, `tr-*`); refused
    /// under single-sig templates.
    #[arg(long)]
    pub threshold: Option<u8>,

    /// v0.4 unified slot input. Repeating flag — one occurrence per
    /// (slot, subkey) tuple.
    ///
    /// Grammar: `@N.<subkey>=<value>`, where N is the slot index
    /// (u8) and `<subkey>` is one of:
    ///   phrase       BIP-39 mnemonic (secret)
    ///   seedqr       48 or 96 ASCII digits encoding a BIP-39 phrase
    ///                (secret; decoded inline via seedqr::decode)
    ///   entropy      raw entropy hex (secret)
    ///   ms1          BIP-93 codex32 secret (entropy or mnemonic;
    ///                language-preserving) (secret)
    ///   xpub         BIP-32 extended public key
    ///   master_xpub  depth-0 master xpub (Coldcard singlesig only;
    ///                see SPEC_export_wallet §5.1)
    ///   fingerprint  4-byte master fingerprint (hex)
    ///   path         BIP-32 derivation path
    ///   wif          Wallet Import Format private key (secret)
    ///   xprv         BIP-32 extended private key (secret)
    ///
    /// `<value>` is the subkey's text form, or `-` to read from
    /// stdin. Single-sig templates expect `@0` only; multisig
    /// templates expect `@0..@N-1`.
    #[arg(
        long = "slot",
        action = clap::ArgAction::Append,
        value_parser = crate::slot_input::parse_slot_input,
        verbatim_doc_comment,
    )]
    pub slot: Vec<crate::slot_input::SlotInput>,

    /// v0.27.0 — synthesize a bundle from an `import-wallet --json`
    /// envelope rather than from `--template` / `--descriptor`. Accepts a
    /// file path or `-` to read the envelope from stdin. The envelope's
    /// `bundle.descriptor` carries the source-of-truth descriptor; the
    /// envelope's `bundle.mk1` chunks decode to per-cosigner xpubs +
    /// fingerprints + paths. Mutually exclusive with `--template`,
    /// `--descriptor`, `--descriptor-file`.
    ///
    /// Seed overlay (`--slot @N.phrase=`) continues to apply to
    /// cosigners where the envelope's `ms1[N] == ""` sentinel (watch-
    /// only). (`--ms1` is import-wallet's input surface, not bundle's;
    /// envelope-derived entropy arrives pre-attached as the envelope's
    /// `ms1[N] != ""`.) Supplying `--slot @N.phrase=` for a cosigner
    /// with non-empty envelope `ms1[N]` is `BadInput` (conflict).
    #[arg(
        long = "import-json",
        value_name = "FILE|-",
        conflicts_with_all = ["template", "descriptor", "descriptor_file"],
    )]
    pub import_json: Option<String>,

    /// v0.27.0 — pick a specific entry from a multi-entry envelope
    /// array (e.g., Bitcoin Core `listdescriptors` blob with multiple
    /// descriptors). Required when the envelope has > 1 entry; optional
    /// (and rejected if supplied) for single-entry envelopes? — actually
    /// SPEC §3.6 leaves single-entry usage unrestricted; passing an
    /// index for a single-entry envelope just requires `0`. Out-of-
    /// range → exit 2.
    #[arg(
        long = "import-json-index",
        value_name = "N",
        requires = "import_json",
    )]
    pub import_json_index: Option<usize>,
}

/// SPEC §6.6 byte-exact mode-violation strings. Pinned for integration tests.
pub mod mode_text {
    pub const THRESHOLD_WITHOUT_MULTISIG: &str = "--threshold is meaningful only with a multisig --template; single-sig templates ignore threshold.";
    pub const PATH_FAMILY_WITHOUT_MULTISIG: &str =
        "--multisig-path-family is meaningful only with a multisig --template.";

    // v0.3 NEW rows (SPEC §6.9). Byte-exact.
    pub const DESCRIPTOR_AND_TEMPLATE: &str = "--descriptor and --template are mutually exclusive; pick descriptor passthrough or template, not both.";
    pub const DESCRIPTOR_AND_DESCRIPTOR_FILE: &str = "--descriptor and --descriptor-file are mutually exclusive; supply the descriptor inline or via file, not both.";
    pub const DESCRIPTOR_WITH_THRESHOLD: &str = "--threshold is meaningful only with a multisig --template; descriptor mode encodes K directly.";
    pub const DESCRIPTOR_WITH_PATH_FAMILY: &str = "--multisig-path-family is meaningful only with --template; descriptor mode encodes paths directly via @i/path syntax.";
    pub const DESCRIPTOR_WITH_NONZERO_ACCOUNT: &str = "--account != 0 is meaningful only with --template; descriptor mode encodes account index in the @i origin path.";
}

pub fn run<W: Write, E: Write>(
    args: &BundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    // SPEC v0.9.0 §1 item 1 — argv-leakage closure. Run BEFORE any
    // dispatch logic so the advisory fires uniformly regardless of
    // downstream success/error. v0.26.0 §I1 fold: emit BEFORE
    // `@env:` sentinel resolution so the advisory sees the literal
    // sentinel string and can skip values that already used the
    // env-var leak-mitigation channel (sentinel-bearing flags do NOT
    // get the warning — the user already opted out of argv exposure).
    emit_secret_in_argv_advisories(args, stderr);

    // v0.26.0 §3 — resolve `@env:<VAR>` sentinels before downstream
    // consumption. Skipped when no sentinel is present to avoid an
    // unnecessary `args.clone()`.
    let env_resolved_owned;
    let args: &BundleArgs = if needs_env_sentinel_resolution(args) {
        env_resolved_owned = resolve_env_sentinels(args)?;
        &env_resolved_owned
    } else {
        args
    };
    let synthetic_args;
    let args: &BundleArgs = if needs_stdin_substitution(args) {
        synthetic_args = apply_stdin_substitutions(args, stdin)?;
        &synthetic_args
    } else {
        args
    };

    // Cycle B Phase 3a Site 1 — pin argv-string secret heap pages for the
    // remainder of the handler scope. Lands AFTER apply_stdin_substitutions
    // so the pin covers the post-substitution buffers (per SPEC §4 P3a).
    let _pin_passphrase = args
        .passphrase
        .as_ref()
        .map(|p| mnemonic_toolkit::mlock::pin_pages_for(p.as_bytes()));
    let _pin_slot_values: Vec<_> = args
        .slot
        .iter()
        .map(|s| mnemonic_toolkit::mlock::pin_pages_for(s.value.as_bytes()))
        .collect();

    // v0.27.0 — `--import-json` dispatch short-circuits before template /
    // descriptor mode pre-checks. The envelope carries everything needed
    // for a fresh `synthesize_descriptor` pass; the only relevant user
    // flags downstream are `--slot @N.phrase=` (seed overlay),
    // `--privacy-preserving`, `--json`, `--self-check`, `--no-engraving-card`.
    // Mutex with --template / --descriptor / --descriptor-file is enforced
    // by clap (`conflicts_with_all` on the `--import-json` arg).
    if args.import_json.is_some() {
        return bundle_run_from_import_json(args, stdin, stdout, stderr);
    }

    let descriptor_mode = args.descriptor.is_some() || args.descriptor_file.is_some();
    let multisig_template = args
        .template
        .as_ref()
        .map(|t| t.is_multisig())
        .unwrap_or(false);

    // SPEC §6.6 / §6.9 retained mode-violation pre-checks.
    if descriptor_mode && args.template.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--template",
            message: mode_text::DESCRIPTOR_AND_TEMPLATE,
        });
    }
    if args.descriptor.is_some() && args.descriptor_file.is_some() {
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
    if descriptor_mode && args.multisig_path_family.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--multisig-path-family",
            message: mode_text::DESCRIPTOR_WITH_PATH_FAMILY,
        });
    }
    // v0.19.0 SPEC §4.12.g — `DESCRIPTOR_WITH_NONZERO_ACCOUNT` guard is
    // canonicity-gated. The check moved into `bundle_run_unified_descriptor`
    // post-parse so canonical descriptors still refuse `--account != 0`
    // (canonical_origin's per-shape default supplies the path; user-supplied
    // account is redundant), while non-canonical descriptors consume
    // `--account N` for §4.12.b default-path inference. Pre-bundle_run_unified
    // site retains the other descriptor-mode guards (--template, --threshold,
    // --multisig-path-family) which apply uniformly regardless of canonicity.
    if args.threshold.is_some() && !multisig_template && !descriptor_mode {
        return Err(ToolkitError::ModeViolation {
            mode: "single-sig",
            flag: "--threshold",
            message: mode_text::THRESHOLD_WITHOUT_MULTISIG,
        });
    }
    if args.multisig_path_family.is_some() && !multisig_template && !descriptor_mode {
        return Err(ToolkitError::ModeViolation {
            mode: "single-sig",
            flag: "--multisig-path-family",
            message: mode_text::PATH_FAMILY_WITHOUT_MULTISIG,
        });
    }

    if descriptor_mode {
        // Read the descriptor body here (the read inside bundle_run_unified_descriptor
        // is off the Concrete early-fork path).
        let body = match (&args.descriptor, &args.descriptor_file) {
            (Some(s), None) => s.clone(),
            (None, Some(p)) => std::fs::read_to_string(p)
                .map_err(|e| ToolkitError::DescriptorParse(format!("--descriptor-file {}: {e}", p.display())))?
                .trim_end()
                .to_string(),
            _ => unreachable!("DESCRIPTOR_AND_DESCRIPTOR_FILE guard above rules out both"),
        };
        use crate::wallet_import::pipeline::{classify_descriptor_form, DescriptorForm};
        if classify_descriptor_form(&body)? == DescriptorForm::Concrete {
            return bundle_run_concrete_descriptor(args, body, stdout, stderr);
        }
        // AtN: fall through to bundle_run_unified (re-reads the file as today).
    }

    bundle_run_unified(args, stdin, stdout, stderr)
}
// ============================================================================
// v0.4.1 Phase H.5: unified --slot-driven dispatch.
// ============================================================================

use crate::bundle_unified::{detect_bundle_mode, BundleMode};
use crate::slot_input::{SlotInput, SlotSubkey};
use crate::synthesize::{synthesize_unified, ResolvedSlot};
use bitcoin::bip32::{DerivationPath, Fingerprint};
use bitcoin::secp256k1::Secp256k1;

/// v0.5.1 entry point — `--slot`-driven dispatch is the sole shape.
/// Routes through SPEC §6.6.b validate_slot_set + §3.3 detect_bundle_mode +
/// `synthesize_unified`.
fn bundle_run_unified<W: Write, E: Write>(
    args: &BundleArgs,
    _stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    use crate::bundle_unified::{pre_check_template_n, pre_check_threshold};
    use crate::slot_input::validate_slot_set;

    let slots = args.slot.clone();
    validate_slot_set(&slots)?;

    let mode = detect_bundle_mode(&slots)?;
    let n = slots
        .iter()
        .map(|s| s.index as usize)
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);

    // SPEC §6.6 row 9, 9.5, 10, 11.
    let template_str = args.template.map(|t| t.human_name());
    let multisig_template = template_str.filter(|_| {
        args.template.map(|t| t.is_multisig()).unwrap_or(false)
    });
    pre_check_threshold(args.threshold, n, multisig_template)?;
    if let Some(t) = args.template {
        pre_check_template_n(t.human_name(), t.is_multisig(), n)?;
    } else if args.descriptor.is_none() && args.descriptor_file.is_none() {
        return Err(ToolkitError::ModeViolation {
            mode: "unified-slot",
            flag: "--template / --descriptor",
            message: "missing --template or --descriptor",
        });
    }

    // v0.4.2 Phase L: descriptor mode under unified --slot dispatch.
    if args.descriptor.is_some() || args.descriptor_file.is_some() {
        return bundle_run_unified_descriptor(args, &slots, mode, stdout, stderr);
    }

    let template = args
        .template
        .ok_or_else(|| ToolkitError::BadInput("--template required for --slot dispatch".into()))?;

    // FOLLOWUP `multisig-tr-bip48-script-type-3-policy` (bless + warn): taproot
    // multisig under --multisig-path-family bip48 derives at the non-standard
    // m/48'/.../3'. Honor it, but advise on stderr at creation time.
    if let Some(w) = template
        .bip48_nonstandard_script_type_warning(args.multisig_path_family.unwrap_or_default())
    {
        let _ = writeln!(stderr, "{w}");
    }

    // Resolve slots into ResolvedSlot vec.
    let (resolved, slip0132_signals) = resolve_slots(
        &slots,
        template,
        args.network,
        args.account,
        args.language,
        args.passphrase.as_deref(),
        args.multisig_path_family.unwrap_or_default(),
    )?;

    // SPEC §4.11.b BIP-388 distinct-key check on resolved slots.
    check_resolved_slots_distinctness(&resolved)?;

    let threshold = args.threshold.unwrap_or(n as u8);

    // Mode-specific synthesis.
    let bundle = match mode {
        BundleMode::SingleSigFull
        | BundleMode::SingleSigWatchOnly
        | BundleMode::MultisigMultiSource
        | BundleMode::MultisigWatchOnly
        | BundleMode::MultisigHybrid => synthesize_unified(
            &resolved,
            template,
            threshold,
            args.network,
            args.privacy_preserving,
            args.language.unwrap_or_default().into(),
        )?,
    };

    // Emit (reuse legacy text/JSON renderer; engraving card omitted for now;
    // unified card lands in Phase I).
    emit_unified(args, &bundle, &resolved, mode, &slip0132_signals, stdout, stderr)?;

    if args.self_check {
        let entropy_bearing: Vec<bool> = resolved.iter().map(|r| r.entropy.is_some()).collect();
        self_check_bundle(&bundle, args, &entropy_bearing)?;
    }
    Ok(())
}

/// v0.4.1 H.5 BIP-388 distinct-key check on ResolvedSlot vector. Mirrors
/// `check_key_vector_distinctness` for the unified path; comparison key is
/// `(xpub.to_string(), path)` on the TYPED `DerivationPath` (v0.5 §4.11.b
/// deliberate-reversal: `h`/`'`-notation folds, so `48h/..` and `48'/..`
/// collide — converges with the descriptor-mode twin per
/// `SPEC_path_raw_bracketed_bare_unification.md` Amendment A2).
fn check_resolved_slots_distinctness(slots: &[ResolvedSlot]) -> Result<(), ToolkitError> {
    for i in 0..slots.len() {
        for j in (i + 1)..slots.len() {
            if slots[i].xpub.to_string() == slots[j].xpub.to_string()
                && slots[i].path == slots[j].path
            {
                return Err(ToolkitError::Bip388Distinctness {
                    i: i as u8,
                    j: j as u8,
                });
            }
        }
    }
    Ok(())
}

/// Resolve slot inputs into ResolvedSlot vec.
/// Supported subkey shapes:
/// - {phrase} → BIP-39 derive entropy + seed + master_xpriv → xpub at template
///   path + master_fingerprint + path.
/// - {xpub, fingerprint, path} → parse all three directly.
/// - {entropy} / {wif} / {xprv-rejected} per slot_input.rs validity matrix.
///
/// Returns `(resolved, slip0132_signals)`. The signals vec carries `(slot_idx,
/// variant)` pairs in slot-index ascending order (BTreeMap iteration) for any
/// `xpub` slots whose input was a SLIP-0132 prefix variant; `emit_unified`
/// uses them to emit the SPEC §5.5.a info-line.
#[allow(clippy::type_complexity)]
pub(crate) fn resolve_slots(
    slots: &[SlotInput],
    template: CliTemplate,
    network: CliNetwork,
    account: u32,
    language: Option<CliLanguage>,
    passphrase: Option<&str>,
    multisig_path_family: MultisigPathFamily,
) -> Result<(Vec<ResolvedSlot>, Vec<(u8, &'static str)>), ToolkitError> {
    use std::collections::BTreeMap;
    let mut by_index: BTreeMap<u8, Vec<&SlotInput>> = BTreeMap::new();
    for s in slots {
        by_index.entry(s.index).or_default().push(s);
    }
    let by_index_len = by_index.len();
    let secp = Secp256k1::new();
    let mut out: Vec<ResolvedSlot> = Vec::with_capacity(by_index_len);
    // SPEC v0.6.2 §5.5.a — accumulate SLIP-0132 input-normalization signals
    // for the emit_unified info-line. BTreeMap iteration is slot-index
    // ascending → no re-sort needed downstream.
    let mut slip0132_signals: Vec<(u8, &'static str)> = Vec::new();
    // F3 fix: for multisig templates the per-cosigner derivation path comes
    // from `--multisig-path-family` (BIP-87 default → m/87'/coin'/account';
    // BIP-48 → m/48'/coin'/account'/script') — NOT `template.derivation_path`,
    // which returns the BIP-87 fallback for ALL multisig templates and so
    // silently ignored the flag for seed/entropy slots. For BIP-87 the path is
    // identical, so every pre-fix default-family bundle is byte-unchanged. For
    // single-sig this is None and the template path is used as before.
    let multisig_acct_path: Option<DerivationPath> = if template.is_multisig() {
        let script_type = template.bip48_script_type().unwrap_or(0);
        let p = multisig_path_family.default_origin_path(network, account, script_type);
        Some(DerivationPath::from_str(&p).expect("family origin paths are well-formed"))
    } else {
        None
    };
    for (idx, slot_inputs) in by_index {
        let subkeys: std::collections::BTreeSet<SlotSubkey> =
            slot_inputs.iter().map(|s| s.subkey).collect();
        if subkeys.contains(&SlotSubkey::Phrase) || subkeys.contains(&SlotSubkey::Seedqr) {
            // v0.31.3 — Seedqr digest = secret-bearing materialization of
            // a BIP-39 phrase. Decode at slot-emit time then dispatch
            // identically to the Phrase path. `is_legal_set` refuses
            // co-occurrence of Phrase + Seedqr in the same slot, so the
            // owned-String binding is unambiguous.
            let decoded_phrase: String;
            let phrase: &str = if subkeys.contains(&SlotSubkey::Seedqr) {
                let digits = slot_inputs
                    .iter()
                    .find(|s| s.subkey == SlotSubkey::Seedqr)
                    .map(|s| s.value.as_str())
                    .expect("contains() asserts presence");
                decoded_phrase = mnemonic_toolkit::seedqr::decode(digits).map_err(|e| {
                    crate::cmd::seedqr::map_seedqr_error(e, &format!("slot @{idx} decode"))
                })?;
                &decoded_phrase
            } else {
                slot_inputs
                    .iter()
                    .find(|s| s.subkey == SlotSubkey::Phrase)
                    .map(|s| s.value.as_str())
                    .expect("contains() asserts presence")
            };
            let lang = language.unwrap_or_default();
            let pass = passphrase.unwrap_or("");
            let acc = match &multisig_acct_path {
                Some(p) => crate::derive::derive_full_at_path(phrase, pass, lang, network, p)?,
                None => {
                    crate::derive::derive_full(phrase, pass, lang, network, template, account)?
                }
            };
            // v0.10.1: DerivedAccount.entropy is Zeroizing<Vec<u8>>; the
            // hand-rolled impl Drop is gone. `into_parts` remains the
            // canonical consuming-move path (returns bare Vec<u8> per the
            // caller-wrap contract — re-wrap below at the ResolvedSlot
            // ctor boundary).
            let (entropy, fingerprint, xpub, _xpriv, path) = acc.into_parts();
            let entropy_pin = Some(Rc::new(pin_pages_for(&entropy[..])));
            out.push(ResolvedSlot {
                xpub,
                fingerprint,
                path,
                // v0.10.1: ResolvedSlot.entropy migrated to Option<Zeroizing<Vec<u8>>>.
                entropy: Some(zeroize::Zeroizing::new(entropy)),
                master_xpub: None,
                language: None,
                _entropy_pin: entropy_pin,
            });
        } else if subkeys.contains(&SlotSubkey::Xpub) {
            let xpub_str = slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::Xpub)
                .map(|s| s.value.as_str())
                .expect("contains() asserts presence");
            // SPEC v0.6.1 §11 — accept SLIP-0132 prefix variants on input.
            let (xpub_str, input_variant) = crate::slip0132::normalize_xpub_prefix(xpub_str)?;
            if let Some(v) = input_variant {
                slip0132_signals.push((idx, v));
            }
            let xpub = bitcoin::bip32::Xpub::from_str(&xpub_str).map_err(|e| {
                ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e))
            })?;
            let fp_str = slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::Fingerprint)
                .map(|s| s.value.as_str());
            let fingerprint = match fp_str {
                Some(s) => Fingerprint::from_str(s).map_err(|e| {
                    ToolkitError::BadInput(format!("--slot @{idx}.fingerprint parse: {e}"))
                })?,
                None => Fingerprint::default(),
            };
            let path = match slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::Path)
            {
                Some(p) => DerivationPath::from_str(&p.value).map_err(|e| {
                    ToolkitError::BadInput(format!("--slot @{idx}.path parse: {e}"))
                })?,
                None => {
                    // v0.5.1: Path absent → fall back to the default origin path
                    // so xpub-only watch-only slots verify against fixtures built
                    // at the same path. F3 fix: for multisig templates this is the
                    // --multisig-path-family path, not the BIP-87-only template
                    // fallback.
                    match &multisig_acct_path {
                        Some(p) => p.clone(),
                        None => template.derivation_path(network, account),
                    }
                }
            };
            // v0.8.2 SPEC §5.1 — parse the optional `@N.master_xpub=` subkey
            // into a depth-0 Xpub. Only emitted by `--format coldcard`
            // singlesig (other formats silently ignore the slot per the
            // per-format ignored-input contract).
            let master_xpub = match slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::MasterXpub)
            {
                Some(m) => {
                    let (mx_str, _variant) =
                        crate::slip0132::normalize_xpub_prefix(&m.value)?;
                    let mx = bitcoin::bip32::Xpub::from_str(&mx_str).map_err(|e| {
                        ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e))
                    })?;
                    Some(mx)
                }
                None => None,
            };
            out.push(ResolvedSlot {
                xpub,
                fingerprint,
                path,
                entropy: None,
                master_xpub,
                language: None,
                _entropy_pin: None,
            });
        } else if subkeys.contains(&SlotSubkey::Entropy) {
            // K.1: {entropy} — byte-identical to phrase resolution for the same
            // underlying entropy via the shared derive_slot helper.
            let entropy_hex = slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::Entropy)
                .map(|s| s.value.as_str())
                .expect("contains() asserts presence");
            let entropy_bytes = hex::decode(entropy_hex).map_err(|e| {
                ToolkitError::BadInput(format!(
                    "--slot @{idx}.entropy hex-decode: {e}"
                ))
            })?;
            let lang = language.unwrap_or_default();
            let lang_bip39: bip39::Language = lang.into();
            let pass = passphrase.unwrap_or("");
            let acc = match &multisig_acct_path {
                Some(p) => crate::derive_slot::derive_bip32_from_entropy_at_path(
                    &entropy_bytes,
                    pass,
                    lang_bip39,
                    network,
                    p,
                )?,
                None => crate::derive_slot::derive_bip32_from_entropy(
                    &entropy_bytes,
                    pass,
                    lang_bip39,
                    network,
                    template,
                    account,
                )?,
            };
            // v0.10.1: `into_parts` returns bare Vec<u8> per caller-wrap
            // contract (Zeroizing-drives-scrub semantics live on the field).
            // The derived `entropy` is discarded here (the user-supplied
            // `entropy_bytes` is the canonical buffer for this slot);
            // the Drop on `acc` will scrub the now-orphaned husk.
            let (_acc_entropy, fingerprint, xpub, _xpriv, path) = acc.into_parts();
            let entropy_pin = Some(Rc::new(pin_pages_for(&entropy_bytes[..])));
            out.push(ResolvedSlot {
                xpub,
                fingerprint,
                path,
                // v0.10.1: ResolvedSlot.entropy migrated to Option<Zeroizing<Vec<u8>>>.
                entropy: Some(zeroize::Zeroizing::new(entropy_bytes)),
                master_xpub: None,
                language: None,
                _entropy_pin: entropy_pin,
            });
        } else if subkeys.contains(&SlotSubkey::Ms1) {
            // v0.41.0 — raw `ms1` codex32 secret. Decode + apply the
            // wire-language policy via the shared `slot_ms1` helper, then
            // derive through the SAME entropy spine the Entropy arm uses.
            // `emit_language` rides onto `ResolvedSlot.language` so the
            // re-emitted card preserves the wire language (LOAD-BEARING for
            // the verify-bundle whole-card round-trip).
            let value = slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::Ms1)
                .map(|s| s.value.as_str())
                .expect("contains() asserts presence");
            let res = crate::slot_ms1::resolve_ms1_slot(value, language, idx)?;
            let pass = passphrase.unwrap_or("");
            let acc = match &multisig_acct_path {
                Some(p) => crate::derive_slot::derive_bip32_from_entropy_at_path(
                    &res.entropy,
                    pass,
                    res.derive_language,
                    network,
                    p,
                )?,
                None => crate::derive_slot::derive_bip32_from_entropy(
                    &res.entropy,
                    pass,
                    res.derive_language,
                    network,
                    template,
                    account,
                )?,
            };
            // Discard the derived-account entropy husk (the helper-supplied
            // `res.entropy` is the canonical buffer for this slot); the Drop
            // on `acc` scrubs it.
            let (_acc_entropy, fingerprint, xpub, _xpriv, path) = acc.into_parts();
            // M4: bind the pin to a LOCAL before moving `res.entropy` — struct
            // fields eval left-to-right, `entropy:` precedes `_entropy_pin:`,
            // so an inline `pin_pages_for(&res.entropy[..])` would move-then-borrow.
            let entropy_pin = Some(Rc::new(pin_pages_for(&res.entropy[..])));
            out.push(ResolvedSlot {
                xpub,
                fingerprint,
                path,
                entropy: Some(res.entropy),
                master_xpub: None,
                language: res.emit_language,
                _entropy_pin: entropy_pin,
            });
        } else if subkeys.contains(&SlotSubkey::Wif) {
            // K.3 (v0.4.2) + R (v0.4.3): {wif} — degenerate single-key. Parse
            // WIF; use its public point as a depth-0 xpub with zero chain code
            // (BIP-32 framing accepts depth-0 with sentinel chain code;
            // non-derivable but the wallet policy slot just needs a stable
            // pubkey). v0.4.3 R: lifted the v0.4.2 single-sig-only guard;
            // wif slots are now legal in multisig contexts. BIP-388
            // distinctness applies normally — same WIF supplied for two slots
            // → identical pubkey + empty path → row 13 collision.
            let _ = by_index_len; // by_index_len no longer guards; multi-wif allowed.
            let wif_str = slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::Wif)
                .map(|s| s.value.as_str())
                .expect("contains() asserts presence");
            let priv_key = bitcoin::PrivateKey::from_wif(wif_str).map_err(|e| {
                ToolkitError::BadInput(format!("--slot @{idx}.wif parse: {e}"))
            })?;
            let pubkey = priv_key.public_key(&secp);
            // Build a depth-0 xpub from the WIF's pubkey + zero chain code.
            // The KeyCard accepts this via the standard mk-codec encoder; the
            // resulting bundle's mk1 carries the wif's pubkey verbatim.
            let xpub = bitcoin::bip32::Xpub {
                network: network.network_kind(),
                depth: 0,
                parent_fingerprint: Fingerprint::default(),
                child_number: bitcoin::bip32::ChildNumber::Normal { index: 0 },
                public_key: pubkey.inner,
                chain_code: bitcoin::bip32::ChainCode::from([0u8; 32]),
            };
            // wif slots are secret-bearing for signing but ms-codec ENTR encoding
            // takes BIP-39 entropy bytes, not raw WIF bytes. v0.4.2 emits an
            // empty-string ms1 sentinel for wif slots — analogous to the xprv
            // case. Document in SPEC §5.8 amendment block.
            out.push(ResolvedSlot {
                xpub,
                fingerprint: Fingerprint::default(),
                path: DerivationPath::default(),
                entropy: None,
                master_xpub: None,
                language: None,
                _entropy_pin: None,
            });
        } else if subkeys.contains(&SlotSubkey::Xprv) {
            // K.2: {xprv} — REJECTED in v0.4.2 per impl plan r1 review C-1.
            // Resolution requires ms-codec XPRV-tag support (cross-repo cycle).
            return Err(ToolkitError::BadInput(format!(
                "--slot @{idx}.xprv not supported in v0.4.2; deferred to v0.5+ \
                pending ms-codec XPRV-tag extension. See FOLLOWUP \
                `unified-slot-xprv-resolution-needs-ms-codec-extension`."
            )));
        } else {
            return Err(ToolkitError::BadInput(format!(
                "slot @{idx} subkey set {:?} not supported by resolve_slots; \
                this should have been caught by validate_slot_set",
                subkeys.iter().map(|s| s.as_str()).collect::<Vec<_>>()
            )));
        }
    }
    Ok((out, slip0132_signals))
}

/// v0.4.1 unified-path emit: reuses the existing emit() / emit_multisig() text
/// rendering by adapting ResolvedSlot back into the shapes those functions
/// expect. Engraving card omitted in v0.4.1 unified path (Phase I lands the
/// unified card across both paths).
fn emit_unified<W: Write, E: Write>(
    args: &BundleArgs,
    bundle: &Bundle,
    resolved: &[ResolvedSlot],
    mode: BundleMode,
    slip0132_signals: &[(u8, &'static str)],
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let _ = mode;
    // SPEC v0.6.1 §11 + v0.6.2 §5.5.a — informational notes for SLIP-0132
    // input normalization. Slot-index ascending; both callers accumulate in
    // ascending order (BTreeMap iteration in resolve_slots; 0..n range in
    // bundle_run_unified_descriptor) so no re-sort needed here. Emitted
    // unconditionally of --json (stderr advisories follow §5.5.a).
    for (_idx, variant) in slip0132_signals.iter() {
        let _ = writeln!(stderr, "{}", crate::slip0132::render_slip0132_info_line(variant));
    }
    // ms mnem Phase 3 Step 6: re-keyed advisory — suppress iff EVERY secret-bearing
    // slot emits a self-describing `mnem` card (i.e. its effective language is
    // non-English and the card is therefore self-describing). Fire only when at
    // least one secret-bearing slot's effective language is English AND the run
    // context is non-English (that slot emits `entr`, which is language-losing
    // in a non-English run context).
    // Effective language per slot: slot.language.unwrap_or(run_language).
    // A slot emits entr iff effective_lang == English.
    // Advisory fires iff: run_language is non-English AND some secret slot's
    //   effective_lang == English (→ that slot emits entr, losing the run language).
    if bundle.any_secret_bearing() {
        let run_lang: bip39::Language = args.language.unwrap_or_default().into();
        let any_slot_emits_entr_non_english_run = run_lang != bip39::Language::English
            && resolved.iter().any(|s| {
                s.entropy.is_some()
                    && s.language.unwrap_or(run_lang) == bip39::Language::English
            });
        if any_slot_emits_entr_non_english_run {
            if let Some(msg) = crate::language::non_english_seed_advisory(
                args.language.unwrap_or_default(),
                "an ms1 card",
            ) {
                let _ = writeln!(stderr, "{msg}");
            }
        }
    }
    let n = resolved.len();
    let mode_str = if bundle.any_secret_bearing() { "full" } else { "watch-only" };
    // v0.4.2 Phase M reconciliation: legacy emit_*/descriptor_mode_emit
    // emitted origin_path with "m/" prefix (md-codec OriginPath rendering).
    // Unified path uses bitcoin DerivationPath::to_string() which omits the
    // "m/" prefix in current bitcoin lib version. Normalize for backward-
    // compatibility with cli_json_envelopes / cli_descriptor_mode tests.
    fn normalize_origin_path(p: &str) -> String {
        if p.is_empty() || p == "m" {
            "m".to_string()
        } else if p.starts_with("m/") {
            p.to_string()
        } else {
            format!("m/{}", p)
        }
    }

    // v0.5 Phase E: absent paths emit null in JSON (was Some("m") via the
    // normalize_origin_path "" → "m" branch). The empty-string sentinel
    // (v0.37.9 — from `ResolvedSlot::origin_path_bare()` for a default path) is
    // the SPEC §4.11.b absent-path marker; null is the JSON wire-format absent.
    fn origin_path_for_json(bare_path: &str) -> Option<String> {
        if bare_path.is_empty() {
            None
        } else {
            Some(normalize_origin_path(bare_path))
        }
    }

    if args.json {
        let template = args.template.map(|t| t.human_name());
        let (multisig_info, origin_path, origin_paths) = if n == 1 {
            (None, origin_path_for_json(&resolved[0].origin_path_bare()), None)
        } else {
            let cosigners: Vec<CosignerEntry> = resolved
                .iter()
                .enumerate()
                .map(|(i, s)| CosignerEntry {
                    index: i,
                    master_fingerprint: if args.privacy_preserving {
                        None
                    } else {
                        Some(s.fingerprint.to_string().to_lowercase())
                    },
                    origin_path: s.origin_path_bare(),
                    xpub: s.xpub.to_string(),
                })
                .collect();
            let threshold = args.threshold.unwrap_or(n as u8);
            // r1 review I-1 fix: derive path_family from --multisig-path-family
            // (defaults to bip87 when unset). Hardcoded "bip87" was wrong for
            // sh-wsh-* templates (which require bip48) and broke SPEC §5.6
            // cross-schema invariant for BIP-48 recovery tooling.
            let info = MultisigInfo {
                template: template.unwrap_or("descriptor"),
                threshold,
                cosigner_count: n,
                path_family: args.multisig_path_family.unwrap_or_default().human_name(),
                cosigners: cosigners.clone(),
            };
            let paths: Vec<String> = cosigners.iter().map(|c| c.origin_path.clone()).collect();
            let all_same = paths.windows(2).all(|w| w[0] == w[1]);
            if all_same {
                (Some(info), paths.first().cloned(), None)
            } else {
                (Some(info), None, Some(paths))
            }
        };
        let master_fp = if n == 1 && !args.privacy_preserving {
            Some(resolved[0].fingerprint.to_string().to_lowercase())
        } else {
            None
        };
        // v0.4.2 Phase L: descriptor presence is mutually exclusive with
        // template. emit_unified is called from both paths; pick the right
        // field based on which arg was supplied.
        let descriptor_field: Option<String> = match (&args.descriptor, &args.descriptor_file) {
            (Some(s), None) => Some(s.clone()),
            (None, Some(p)) => std::fs::read_to_string(p)
                .ok()
                .map(|s| s.trim_end().to_string()),
            _ => None,
        };
        let json = BundleJson {
            schema_version: "4",
            mode: mode_str,
            network: args.network.human_name(),
            template: if descriptor_field.is_some() { None } else { template },
            descriptor: descriptor_field,
            account: args.account,
            origin_path,
            origin_paths,
            master_fingerprint: master_fp,
            ms1: bundle.ms1.clone(),
            mk1: bundle.mk1.clone(),
            md1: bundle.md1.clone(),
            multisig: multisig_info,
            privacy_preserving: args.privacy_preserving,
        };
        serde_json::to_writer(&mut *stdout, &json).ok();
        writeln!(stdout).ok();
    } else {
        // Schema-4 text mode: emit per-slot ms1 sections (skip empty sentinels).
        // v0.4.2 Phase M reconciliation: when ALL ms1 entries are empty, emit
        // an "omitted" marker line for backward-compatibility with v0.3
        // legacy text-mode output. The marker text varies by mode.
        let any_non_empty = bundle.ms1.iter().any(|s| !s.is_empty());
        if !any_non_empty {
            let marker = if args.descriptor.is_some() || args.descriptor_file.is_some() {
                "# ms1 (omitted — descriptor watch-only mode)"
            } else if n > 1 {
                "# ms1 (omitted — multisig watch-only mode)"
            } else {
                "# ms1 (omitted — xpub-only mode)"
            };
            writeln!(stdout, "{marker}").ok();
            writeln!(stdout).ok();
        }
        for (i, ms) in bundle.ms1.iter().enumerate() {
            if ms.is_empty() {
                continue;
            }
            if n > 1 {
                writeln!(stdout, "# ms1[{i}] (entropy, BCH-checksummed)").ok();
            } else {
                writeln!(stdout, "# ms1 (entropy, BCH-checksummed)").ok();
            }
            writeln!(stdout, "{}", ms).ok();
            writeln!(stdout).ok();
            writeln!(stdout, "{}", chunk_5char(ms)).ok();
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
        // v0.4.2 Phase M reconciliation: legacy emit_multisig prefixed
        // "multisig" to the md1 header for n>1; preserve.
        let md1_header = if n > 1 {
            "# md1 (multisig wallet policy)"
        } else {
            "# md1 (wallet policy)"
        };
        writeln!(stdout, "{md1_header}").ok();
        for s in &bundle.md1 {
            writeln!(stdout, "{}", s).ok();
        }
        writeln!(stdout).ok();
        for s in &bundle.md1 {
            writeln!(stdout, "{}", chunk_md1(s)).ok();
        }
        writeln!(stdout).ok();
        // v0.4.1 Phase I: emit unified engraving card to stderr unless suppressed.
        if !args.no_engraving_card {
            let card = build_unified_card(args, bundle, resolved);
            write!(stderr, "{}", card).ok();
        }
    }
    // SPEC §5.5.a: output-class advisory — PrivateKeyMaterial when any ms1
    // slot is non-empty (BIP-39 entropy on stdout); WatchOnly otherwise
    // (all ms1 == "" sentinels per §5.8). Always fires (Option never None).
    let cls = if bundle.any_secret_bearing() {
        crate::secret_advisory::OutputClass::PrivateKeyMaterial
    } else {
        crate::secret_advisory::OutputClass::WatchOnly
    };
    crate::secret_advisory::emit_output_class_advisory(cls, stderr);
    Ok(())
}

/// Extract the multisig threshold K from a descriptor's tree, if a
/// multi-family operator (`multi` / `sortedmulti` / `multi_a` /
/// `sortedmulti_a`) or `thresh` is present. Returns `None` for pure
/// single-sig descriptors. Walks `wsh(...)`, `sh(...)`, and `tr(IK, ...)`
/// wrappings to reach the inner threshold-bearing node.
pub(crate) fn extract_multisig_threshold(node: &md_codec::tree::Node) -> Option<u8> {
    use md_codec::tree::Body;
    match &node.body {
        Body::MultiKeys { k, .. } => Some(*k),
        Body::Variable { k, .. } => Some(*k),
        Body::Children(children) => children.iter().find_map(extract_multisig_threshold),
        Body::Tr { tree: Some(inner), .. } => extract_multisig_threshold(inner),
        _ => None,
    }
}

/// v0.4.1 Phase I helper: assemble `BundleInputForCard` from the unified
/// dispatch's `ResolvedSlot` vec + `Bundle` + args, then render via
/// `engraving_card_unified`.
fn build_unified_card(
    args: &BundleArgs,
    bundle: &Bundle,
    resolved: &[ResolvedSlot],
) -> String {
    use crate::format::{engraving_card_unified, BundleInputForCard, SlotCardBlock,
        TemplateOrDescriptor};
    use crate::synthesize::derive_mk1_chunk_set_id;

    let n = resolved.len() as u8;
    let template_str: &'static str =
        args.template.map(|t| t.human_name()).unwrap_or("descriptor");

    // Compute md1 chunk_set_id from the descriptor's policy_id (re-extracted
    // from the encoded md1 strings to avoid threading the policy_id through
    // the synthesis output). The reassembled Descriptor is reused below to
    // extract the multisig threshold K — args.threshold is None on the
    // --import-json descriptor-mode path, so we must read K from the
    // descriptor body itself rather than fall back to N.
    let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
    let reassembled_descriptor = md_codec::chunk::reassemble(&md1_strs).ok();
    let md1_chunk_set_id = reassembled_descriptor
        .as_ref()
        .and_then(|d| md_codec::compute_wallet_policy_id(d).ok())
        .map(|pid| {
            let bytes = pid.as_bytes();
            format!("{:02x}{:02x}", bytes[0], bytes[1])
        })
        .unwrap_or_else(|| "????".to_string());
    let descriptor_threshold: Option<u8> = reassembled_descriptor
        .as_ref()
        .and_then(|d| extract_multisig_threshold(&d.tree));

    let per_slot: Vec<SlotCardBlock> = resolved
        .iter()
        .enumerate()
        .map(|(i, s)| {
            // Both ms1 and mk1 share the policy_id_stub-derived chunk_set_id
            // (per Phase I.1 spec note in the impl plan).
            let stub_csi_4hex = match md_codec::chunk::reassemble(&md1_strs)
                .ok()
                .and_then(|d| md_codec::compute_wallet_policy_id(&d).ok())
            {
                Some(pid) => {
                    let stub = &pid.as_bytes()[..4];
                    format!("{:05x}", derive_mk1_chunk_set_id(&[
                        stub[0], stub[1], stub[2], stub[3]
                    ]))
                }
                None => "?????".to_string(),
            };
            let ms1_card_id = if bundle.ms1.get(i).map(|s| !s.is_empty()).unwrap_or(false) {
                Some(stub_csi_4hex.clone())
            } else {
                None
            };
            SlotCardBlock {
                index: i as u8,
                ms1_card_id,
                mk1_card_id: stub_csi_4hex,
                fingerprint: if args.privacy_preserving {
                    None
                } else {
                    Some(s.fingerprint.to_string().to_lowercase())
                },
                origin_path: match s.origin_path_bare() {
                    p if p.is_empty() => None,
                    p => Some(p),
                },
            }
        })
        .collect();

    let input = BundleInputForCard {
        network: args.network.human_name(),
        template_or_descriptor: TemplateOrDescriptor::Template(template_str),
        threshold: args
            .threshold
            .or(descriptor_threshold)
            .or(if n > 1 { Some(n) } else { None }),
        n,
        language: args.language.map(|l| l.human_name()),
        passphrase_used: args.passphrase.as_ref().map(|p| !p.is_empty()).unwrap_or(false),
        privacy_preserving: args.privacy_preserving,
        per_slot,
        md1_chunk_set_id,
    };

    engraving_card_unified(&input)
}

// ============================================================================
// v0.4.2 Phase L — descriptor mode under unified --slot dispatch.
// ============================================================================

use crate::parse_descriptor::{lex_placeholders, parse_descriptor, resolve_placeholders};
use crate::synthesize::{synthesize_descriptor, CosignerKeyInfo};
use bip39::Mnemonic as Bip39Mnemonic;
use bitcoin::bip32::{Xpriv as BipXpriv, Xpub as BipXpub};
use md_codec::origin_path::PathDeclPaths;

/// v0.4.2 Phase L entry point. Reached when args.descriptor / descriptor_file
/// is supplied alongside --slot. Resolves each slot per its subkey set against
/// the per-@i annotation path from the parsed descriptor, then routes through
/// the existing synthesize_descriptor pipeline.
///
/// Phase N (binding-type merge) collapses the legacy CosignerKeyInfo into
/// ResolvedSlot; v0.4.2 Phase L continues to construct CosignerKeyInfo as a
/// bridge so synthesize_descriptor's existing signature is preserved.
fn bundle_run_unified_descriptor<W: Write, E: Write>(
    args: &BundleArgs,
    slots: &[crate::slot_input::SlotInput],
    _mode: BundleMode,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    use std::collections::BTreeMap;

    let descriptor_str = match (&args.descriptor, &args.descriptor_file) {
        (Some(s), None) => s.clone(),
        (None, Some(p)) => std::fs::read_to_string(p)
            .map_err(|e| ToolkitError::DescriptorParse(format!(
                "--descriptor-file {}: {e}",
                p.display()
            )))?
            .trim_end()
            .to_string(),
        _ => unreachable!("clap conflicts_with rules out both / pre-checks rule out neither"),
    };

    let occs = lex_placeholders(&descriptor_str)?;
    let mut resolved_placeholders = resolve_placeholders(&occs)?;
    let n = resolved_placeholders.n as usize;

    if slots.iter().map(|s| s.index as usize + 1).max().unwrap_or(0) != n {
        return Err(ToolkitError::DescriptorParse(format!(
            "descriptor has n={n} placeholders but --slot vec covers {} slots",
            slots.iter().map(|s| s.index as usize + 1).max().unwrap_or(0)
        )));
    }

    // v0.19.0 SPEC §4.12 — early canonicity classification. Probe-parse the
    // descriptor (empty keys/fingerprints — only the tree is consulted) so
    // canonicity is known before slot binding. The full parse_descriptor
    // call later (line ~1112) re-runs with populated keys/fingerprints; the
    // probe-parse is cheap because rust-miniscript caches nothing per-call
    // and the substituted form is small.
    let canonicity_probe = parse_descriptor(&descriptor_str, &[], &[])?;
    let is_non_canonical =
        md_codec::canonical_origin::canonical_origin(&canonicity_probe.tree).is_none();

    // v0.19.0 SPEC §4.12.g — DESCRIPTOR_WITH_NONZERO_ACCOUNT canonicity-gated.
    if !is_non_canonical && args.account != 0 {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--account",
            message: mode_text::DESCRIPTOR_WITH_NONZERO_ACCOUNT,
        });
    }

    // v0.19.0 SPEC §6.6 row 4 canonical-mode rejection of [Phrase, Path] /
    // [Phrase, Fingerprint, Path] subkey sets. Phase 2 slot grammar accepts
    // these pairs structurally; canonical descriptors refuse them here.
    if !is_non_canonical {
        let mut by_index_check: std::collections::BTreeMap<u8, Vec<&crate::slot_input::SlotInput>> =
            std::collections::BTreeMap::new();
        for s in slots {
            by_index_check.entry(s.index).or_default().push(s);
        }
        for (idx, slot_inputs) in &by_index_check {
            let subkeys: std::collections::BTreeSet<crate::slot_input::SlotSubkey> =
                slot_inputs.iter().map(|s| s.subkey).collect();
            let has_phrase = subkeys.contains(&crate::slot_input::SlotSubkey::Phrase);
            let has_seedqr = subkeys.contains(&crate::slot_input::SlotSubkey::Seedqr);
            let has_ms1 = subkeys.contains(&crate::slot_input::SlotSubkey::Ms1);
            let has_path = subkeys.contains(&crate::slot_input::SlotSubkey::Path);
            if (has_phrase || has_seedqr || has_ms1) && has_path {
                return Err(ToolkitError::SlotInputViolation {
                    kind: "conflict",
                    message: format!(
                        "slot @{idx} has both secret-bearing input and watch-only input; pick one per slot."
                    ),
                });
            }
        }
    }

    // v0.19.0 SPEC §4.12.b — default-path inference for non-canonical
    // descriptors. For each `@N` whose path_decl entry is empty AND that
    // has no `--slot @N.path=` override, assign `m/48'/<coin>'/<account>'/2'`
    // (BIP-48 cosigner path). The mutation produces `Divergent(vec)` with
    // `vec.len() == n`. `--slot @N.path=` overrides happen later in the
    // per-slot loop (lines 1018-1029 already handle Xpub slots; Phase 4
    // extends to Phrase slots via the canonical-mode guard above).
    let mut defaulted_indices: Vec<u8> = Vec::new();
    if is_non_canonical {
        let default_path = compute_default_origin_path(args.network, args.account);
        let mut new_paths: Vec<md_codec::origin_path::OriginPath> = match
            &resolved_placeholders.path_decl.paths
        {
            PathDeclPaths::Shared(op) => {
                if op.components.is_empty() {
                    // All slots default.
                    defaulted_indices.extend(0..(n as u8));
                    (0..n).map(|_| default_path.clone()).collect()
                } else {
                    // Shared non-empty: no defaulting; lift to Divergent
                    // for uniform downstream handling.
                    (0..n).map(|_| op.clone()).collect()
                }
            }
            PathDeclPaths::Divergent(v) => v
                .iter()
                .enumerate()
                .map(|(i, op)| {
                    if op.components.is_empty() {
                        defaulted_indices.push(i as u8);
                        default_path.clone()
                    } else {
                        op.clone()
                    }
                })
                .collect(),
        };

        // Apply per-slot `--slot @N.path=` overrides (phrase slots only;
        // the Xpub branch in the binding loop has its own path-override
        // handling). Refuse on inline-vs-slot path mismatch (row 19).
        let mut by_index_path: std::collections::BTreeMap<u8, &crate::slot_input::SlotInput> =
            std::collections::BTreeMap::new();
        for s in slots {
            if s.subkey == crate::slot_input::SlotSubkey::Path {
                by_index_path.insert(s.index, s);
            }
        }
        let mut by_index_subkeys: std::collections::BTreeMap<
            u8,
            std::collections::BTreeSet<crate::slot_input::SlotSubkey>,
        > = std::collections::BTreeMap::new();
        for s in slots {
            by_index_subkeys
                .entry(s.index)
                .or_default()
                .insert(s.subkey);
        }
        for (idx, slot_path) in &by_index_path {
            let subkeys = by_index_subkeys.get(idx).cloned().unwrap_or_default();
            // Only phrase-bearing slots route through this override path
            // (incl. v0.31.3 Seedqr materialization which decodes to phrase,
            // and v0.41.0 Ms1 which decodes to entropy). Xpub-bearing slots are
            // handled by the per-slot binding loop's existing override logic at
            // bundle.rs:1018-1029.
            if !subkeys.contains(&crate::slot_input::SlotSubkey::Phrase)
                && !subkeys.contains(&crate::slot_input::SlotSubkey::Seedqr)
                && !subkeys.contains(&crate::slot_input::SlotSubkey::Ms1)
            {
                continue;
            }
            let user_path = DerivationPath::from_str(&slot_path.value).map_err(|e| {
                ToolkitError::BadInput(format!("--slot @{idx}.path parse: {e}"))
            })?;
            let user_origin = derivation_path_to_origin(&user_path);
            // Row 19: if inline `[fp/path]@N` AND `--slot @N.path=` both
            // supplied AND non-empty AND differ → refuse.
            if !defaulted_indices.contains(idx)
                && !new_paths[*idx as usize].components.is_empty()
                && new_paths[*idx as usize] != user_origin
            {
                let inline_path = origin_to_derivation_path(&new_paths[*idx as usize])?;
                return Err(ToolkitError::SlotInputViolation {
                    kind: "path-mismatch",
                    message: format!(
                        "slot @{idx} path mismatch: --slot says {user_path}, descriptor inline [.../{inline_path}] disagrees; supply consistent values or remove one source."
                    ),
                });
            }
            new_paths[*idx as usize] = user_origin;
            // Slot-supplied path takes precedence; if it was a default,
            // remove from the notice list.
            defaulted_indices.retain(|i| i != idx);
        }

        // F4 fix: collapse identical inferred per-`@N` paths to `Shared` — the
        // canonical form `parse_descriptor` (all_paths_same) and
        // `synthesize_unified` (all_same || n==1) already use. Without this, an
        // elided-origin descriptor emitted `Divergent([p,p,p])` while the
        // explicit-origin / wallet-import path emitted `Shared(p)` for the SAME
        // wallet → byte-different md1 (cross-start non-convergence). `new_paths`
        // is non-empty (n >= 1 enforced upstream); a 1-element vec is all-same.
        let all_same = new_paths.windows(2).all(|w| w[0] == w[1]);
        resolved_placeholders.path_decl.paths = if all_same {
            PathDeclPaths::Shared(new_paths[0].clone())
        } else {
            PathDeclPaths::Divergent(new_paths)
        };
    }

    // Resolve each @i slot using the per-@i annotation path from the descriptor.
    let secp = Secp256k1::new();
    let mut by_index: BTreeMap<u8, Vec<&crate::slot_input::SlotInput>> = BTreeMap::new();
    for s in slots {
        by_index.entry(s.index).or_default().push(s);
    }

    let mut cosigners: Vec<CosignerKeyInfo> = Vec::with_capacity(n);
    let mut keys: Vec<crate::parse_descriptor::ParsedKey> = Vec::with_capacity(n);
    let mut fingerprints: Vec<crate::parse_descriptor::ParsedFingerprint> = Vec::with_capacity(n);
    // SPEC v0.6.2 §5.5.a — accumulate SLIP-0132 input-normalization signals.
    // The 0..n range loop walks slots in ascending order natively → no re-sort.
    let mut slip0132_signals: Vec<(u8, &'static str)> = Vec::new();

    for idx in 0..(n as u8) {
        let slot_inputs = by_index
            .get(&idx)
            .ok_or_else(|| ToolkitError::SlotInputViolation {
                kind: "gap",
                message: format!("--slot @{idx} missing for descriptor with n={n} placeholders"),
            })?;
        let subkeys: std::collections::BTreeSet<crate::slot_input::SlotSubkey> =
            slot_inputs.iter().map(|s| s.subkey).collect();

        // Per-@i annotation path from descriptor.
        let anno_path: bitcoin::bip32::DerivationPath =
            match &resolved_placeholders.path_decl.paths {
                PathDeclPaths::Shared(op) => origin_to_derivation_path(op)?,
                PathDeclPaths::Divergent(v) => origin_to_derivation_path(&v[idx as usize])?,
            };
        let anno_fp: Option<bitcoin::bip32::Fingerprint> =
            resolved_placeholders.fingerprint_annos[idx as usize];

        // v0.41.0 — 5-tuple widening (Plan-R0-I1): the 5th element carries the
        // per-slot emit language (Some(wire) for a mnem ms1; None for every
        // pre-ms1 arm) so the single shared push can stamp
        // `CosignerKeyInfo.language` — LOAD-BEARING for the verify-bundle
        // whole-card round-trip.
        let (xpub, fingerprint, path, ent_opt, emit_lang): (
            BipXpub,
            Fingerprint,
            DerivationPath,
            Option<Vec<u8>>,
            Option<bip39::Language>,
        ) = if subkeys.contains(&crate::slot_input::SlotSubkey::Phrase) {
            // SAFETY: third-party-blocked — `bip39::Mnemonic` +
            // `bitcoin::bip32::Xpriv` have no Drop+Zeroize. FOLLOWUPS:
            // `rust-bip39-mnemonic-zeroize-upstream`,
            // `rust-bitcoin-xpriv-zeroize-upstream`. The passphrase clone,
            // entropy Vec, and seed buffer are all `Zeroizing`-wrapped.
            let phrase = slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Phrase)
                .map(|s| s.value.as_str())
                .expect("contains() asserts presence");
            let language = args.language.unwrap_or_default();
            let passphrase: zeroize::Zeroizing<String> =
                zeroize::Zeroizing::new(args.passphrase.clone().unwrap_or_default());
            let mnemonic = Bip39Mnemonic::parse_in(language.into(), phrase)
                .map_err(ToolkitError::Bip39)?;
            let entropy = zeroize::Zeroizing::new(mnemonic.to_entropy());
            let seed = crate::derive_slot::derive_master_seed(&mnemonic, &passphrase);
            let master = BipXpriv::new_master(args.network.network_kind(), &seed[..])
                .map_err(|e| {
                    ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e))
                })?;
            let master_fp = master.fingerprint(&secp);
            // Cross-check fingerprint annotation if present.
            if let Some(anno) = anno_fp {
                if anno != master_fp {
                    return Err(ToolkitError::DescriptorParse(format!(
                        "--slot @{idx}.phrase derives master fingerprint {master_fp} but descriptor @{idx} annotation specifies {anno}"
                    )));
                }
            }
            // SAFETY: third-party-blocked — `bitcoin::bip32::Xpriv` is Copy
            // + no Drop; FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`.
            let acct_xpriv = master.derive_priv(&secp, &anno_path).map_err(|e| {
                ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e))
            })?;
            let xpub = BipXpub::from_priv(&secp, &acct_xpriv);
            (xpub, master_fp, anno_path.clone(), Some((*entropy).clone()), None)
        } else if subkeys.contains(&crate::slot_input::SlotSubkey::Xpub) {
            let xpub_str = slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Xpub)
                .map(|s| s.value.as_str())
                .expect("contains() asserts presence");
            // SPEC v0.6.1 §11 — accept SLIP-0132 prefix variants on input.
            let (xpub_str, input_variant) = crate::slip0132::normalize_xpub_prefix(xpub_str)?;
            if let Some(v) = input_variant {
                slip0132_signals.push((idx, v));
            }
            let xpub = BipXpub::from_str(&xpub_str).map_err(|e| {
                ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e))
            })?;
            let fp = slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Fingerprint)
                .and_then(|s| Fingerprint::from_str(&s.value).ok())
                .or(anno_fp)
                .unwrap_or_default();
            let path = match slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Path)
            {
                Some(p) => DerivationPath::from_str(&p.value).map_err(|e| {
                    ToolkitError::BadInput(format!("--slot @{idx}.path parse: {e}"))
                })?,
                None => anno_path.clone(),
            };
            (xpub, fp, path, None, None)
        } else if subkeys.contains(&crate::slot_input::SlotSubkey::Entropy) {
            let entropy_hex = slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Entropy)
                .map(|s| s.value.as_str())
                .expect("contains() asserts presence");
            // SAFETY: third-party-blocked — `bip39::Mnemonic` +
            // `bitcoin::bip32::Xpriv` have no Drop+Zeroize. FOLLOWUPS:
            // `rust-bip39-mnemonic-zeroize-upstream`,
            // `rust-bitcoin-xpriv-zeroize-upstream`.
            let entropy_bytes = zeroize::Zeroizing::new(hex::decode(entropy_hex).map_err(|e| {
                ToolkitError::BadInput(format!(
                    "--slot @{idx}.entropy hex-decode: {e}"
                ))
            })?);
            let language = args.language.unwrap_or_default();
            let passphrase: zeroize::Zeroizing<String> =
                zeroize::Zeroizing::new(args.passphrase.clone().unwrap_or_default());
            let mnemonic = Bip39Mnemonic::from_entropy_in(language.into(), &entropy_bytes[..])
                .map_err(ToolkitError::Bip39)?;
            let seed = crate::derive_slot::derive_master_seed(&mnemonic, &passphrase);
            let master = BipXpriv::new_master(args.network.network_kind(), &seed[..])
                .map_err(|e| {
                    ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e))
                })?;
            let master_fp = master.fingerprint(&secp);
            // SAFETY: third-party-blocked — `bitcoin::bip32::Xpriv` is Copy
            // + no Drop; FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`.
            let acct_xpriv = master.derive_priv(&secp, &anno_path).map_err(|e| {
                ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e))
            })?;
            let xpub = BipXpub::from_priv(&secp, &acct_xpriv);
            (xpub, master_fp, anno_path.clone(), Some((*entropy_bytes).clone()), None)
        } else if subkeys.contains(&crate::slot_input::SlotSubkey::Ms1) {
            // v0.41.0 — raw `ms1` codex32 secret cosigner. Decode + apply the
            // wire-language policy via the shared helper, then derive the
            // cosigner key at the descriptor-annotated `anno_path`. This loop
            // has NO bare `pass`/`network` locals (M-A): use `args.network` and
            // build the passphrase from `args.passphrase` like the Phrase /
            // Entropy arms above.
            let value = slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Ms1)
                .map(|s| s.value.as_str())
                .expect("contains() asserts presence");
            let res = crate::slot_ms1::resolve_ms1_slot(value, args.language, idx)?;
            let passphrase: zeroize::Zeroizing<String> =
                zeroize::Zeroizing::new(args.passphrase.clone().unwrap_or_default());
            let acc = crate::derive_slot::derive_bip32_from_entropy_at_path(
                &res.entropy,
                &passphrase,
                res.derive_language,
                args.network,
                &anno_path,
            )?;
            let (_acc_entropy, master_fp, xpub, _xpriv, _path) = acc.into_parts();
            (
                xpub,
                master_fp,
                anno_path.clone(),
                Some((*res.entropy).clone()),
                res.emit_language,
            )
        } else {
            return Err(ToolkitError::BadInput(format!(
                "--slot @{idx} subkey set {:?} not supported in descriptor mode in v0.4.2 \
                (xprv → v0.5+; wif → multisig FOLLOWUP; partial xpub may be supported but \
                requires full descriptor binding context — file a FOLLOWUP if needed)",
                subkeys.iter().map(|s| s.as_str()).collect::<Vec<_>>()
            )));
        };

        // v0.4.3 Phase N: per-slot entropy goes on the ResolvedSlot directly.
        // v0.10.1: CosignerKeyInfo.entropy (aliased ResolvedSlot.entropy) is
        // Option<Zeroizing<Vec<u8>>>; wrap at the field-write boundary.
        let entropy = ent_opt.clone().map(zeroize::Zeroizing::new);
        let entropy_pin = entropy.as_ref().map(|e| Rc::new(pin_pages_for(&e[..])));
        cosigners.push(CosignerKeyInfo {
            xpub,
            fingerprint,
            path,
            entropy,
            master_xpub: None,
            // v0.41.0 — per-slot emit language (Some(wire) for a mnem ms1
            // cosigner; None for every other arm). Drives the mnem-vs-entr
            // re-emit so the verify-bundle whole-card compare round-trips.
            language: emit_lang,
            _entropy_pin: entropy_pin,
        });

        keys.push(crate::parse_descriptor::ParsedKey {
            i: idx,
            payload: crate::synthesize::xpub_to_65(&xpub),
        });
        fingerprints.push(crate::parse_descriptor::ParsedFingerprint {
            i: idx,
            fp: fingerprint.to_bytes(),
        });
    }

    // SPEC §4.11.b BIP-388 distinct-key check (use bridging path: cosigners
    // already carry the typed path + entropy per slot post-v0.4.3 N alias merge).
    let dummy_binding = crate::parse_descriptor::DescriptorBinding {
        keys: keys.clone(),
        fingerprints: fingerprints.clone(),
        cosigners: cosigners.clone(),
    };
    crate::parse_descriptor::check_key_vector_distinctness(&dummy_binding)?;

    // Build md-codec Descriptor + synthesize.
    let mut descriptor = parse_descriptor(&descriptor_str, &keys, &fingerprints)?;

    // v0.19.0 SPEC §4.12.b/c — propagate the locally-mutated `path_decl`
    // (default-inference + slot-path overrides applied above) into the
    // freshly-parsed `MdDescriptor`. `parse_descriptor` re-runs
    // `resolve_placeholders` internally and would otherwise reset the
    // path_decl to its descriptor-string-derived form (empty Shared for
    // bare-`@N` non-canonical descriptors), losing the default-inference
    // mutation that `md_codec::validate_explicit_origin_required` needs to
    // accept the wire.
    if is_non_canonical {
        descriptor.path_decl.paths = resolved_placeholders.path_decl.paths.clone();
    }

    let run_language: bip39::Language = args.language.unwrap_or_default().into();
    let bundle = synthesize_descriptor(&descriptor, &cosigners, args.privacy_preserving, run_language)?;

    // Reuse emit_unified renderer (resolved must be reconstructed as
    // ResolvedSlot vec for engraving card; entropy field tracks per-slot).
    // SPEC §5.8 per-slot emission: clone entropy from each cosigners[i] so the
    // engraving-card cosigner-summary block reflects ms1 emission for every
    // phrase-bearing slot (not just @0). master_xpub: None preserves the
    // descriptor-mode invariant established above at the cosigner push.
    // ms mnem Phase 3 C1 fix: populate language on each resolved slot so that
    // emit_unified's advisory model (slot.language.unwrap_or(run_lang)) agrees
    // with the actual emitted card kind — non-English descriptor-@N slots now
    // emit mnem, so the advisory is correctly suppressed for them.
    let resolved_slots: Vec<ResolvedSlot> = cosigners
        .iter()
        .map(|c| {
            let entropy_pin = c.entropy.as_ref().map(|e| Rc::new(pin_pages_for(&e[..])));
            // Slot language: None slots inherit run_language (same unwrap_or as
            // synthesize_descriptor). This makes the advisory model in emit_unified
            // agree with the actual emitted card kind.
            let slot_language = if c.entropy.is_some() {
                Some(c.language.unwrap_or(run_language))
            } else {
                None // watch-only slots have no entropy → advisory model skips them
            };
            ResolvedSlot {
                xpub: c.xpub,
                fingerprint: c.fingerprint,
                path: c.path.clone(),
                entropy: c.entropy.clone(),
                master_xpub: None,
                language: slot_language,
                _entropy_pin: entropy_pin,
            }
        })
        .collect();

    // v0.19.0 SPEC §4.12.d — stderr info notice on default-path application.
    // Printed BEFORE the bundle to surface the assumption legibly. Suppressed
    // when no `@N` received the default (defaulted_indices is empty).
    emit_default_path_notice(stderr, &defaulted_indices, args.network, args.account)?;

    emit_unified(
        args,
        &bundle,
        &resolved_slots,
        BundleMode::SingleSigFull,
        &slip0132_signals,
        stdout,
        stderr,
    )?;

    if args.self_check {
        let entropy_bearing: Vec<bool> =
            resolved_slots.iter().map(|r| r.entropy.is_some()).collect();
        self_check_bundle(&bundle, args, &entropy_bearing)?;
    }

    Ok(())
}

/// Theme-A Phase 3a entry point — `bundle --descriptor <CONCRETE>`. Accepts a
/// bare-concrete descriptor (inline `[fp/path]xpub` keys, no `@N` placeholders)
/// and synthesizes a watch-only bundle without any `--slot` inputs. SPEC §3.2.
fn bundle_run_concrete_descriptor<W: Write, E: Write>(
    args: &BundleArgs,
    body: String,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    use crate::wallet_import::pipeline::descriptor_concrete_to_resolved_slots;
    let body_no_csum =
        crate::wallet_import::json_envelope::descriptor_body_no_csum(&body, "--descriptor")?;
    let (descriptor, resolved_slots) = descriptor_concrete_to_resolved_slots(body_no_csum)?;

    // BIP-388 distinctness check — a pasted descriptor is untrusted, unlike
    // the mk1-sourced path in bundle_run_from_import_json.
    check_resolved_slots_distinctness(&resolved_slots)?;

    // Concrete-descriptor mode is always watch-only (no phrase/entropy input);
    // run_language is irrelevant but must be supplied — English is the safe default.
    let bundle = synthesize_descriptor(&descriptor, &resolved_slots, args.privacy_preserving, bip39::Language::English)?;
    let n = resolved_slots.len();
    let any_secret = resolved_slots.iter().any(|s| s.entropy.is_some()); // always false here
    let any_watch = resolved_slots.iter().any(|s| s.entropy.is_none());
    let mode = match (n, any_secret, any_watch) {
        (1, true, _) => BundleMode::SingleSigFull,
        (1, false, _) => BundleMode::SingleSigWatchOnly,
        (_, true, true) => BundleMode::MultisigHybrid,
        (_, true, false) => BundleMode::MultisigMultiSource,
        (_, false, _) => BundleMode::MultisigWatchOnly,
    };

    // slip0132 signals are not applicable to a bare-concrete descriptor
    // (the inline xpub prefix is canonical per SPEC §5.3).
    emit_unified(args, &bundle, &resolved_slots, mode, &[], stdout, stderr)?;

    if args.self_check {
        let entropy_bearing: Vec<bool> =
            resolved_slots.iter().map(|r| r.entropy.is_some()).collect();
        self_check_bundle(&bundle, args, &entropy_bearing)?;
    }

    Ok(())
}

/// v0.27.0 Phase 5 entry point — `bundle --import-json <FILE|->`. Consumes
/// an `import-wallet --json` envelope (SPEC §3.2 wire shape; Phase 4 ship)
/// and synthesizes a fresh bundle. Per plan §3.6.
///
/// Pipeline:
/// 1. Read envelope (file or stdin) → parse via `parse_import_json_envelopes`.
/// 2. Extract descriptor (`bundle.descriptor`) + decode `bundle.mk1` chunks
///    into `Vec<ResolvedSlot>` per §3.6.1.
/// 3. Decode envelope's `bundle.ms1[i] != ""` entries → attach entropy to
///    `resolved_slots[i]` (envelope-derived seed-bearing state).
/// 4. Apply user seed overlay (`--slot @N.phrase=`) on slots where
///    envelope `ms1[i] == ""`; conflict-on-non-empty-envelope-ms1
///    is `BadInput` exit 1 (per plan Q5 / §3.1.3). (`--ms1` is
///    import-wallet's input surface, not bundle's — see the asymmetry
///    note after the overlay loop in `bundle_run_from_import_json`.)
/// 5. Parse descriptor via `concrete_keys_to_placeholders` →
///    `parse_descriptor::parse_descriptor` (same path bundle_run_unified_descriptor
///    follows post-substitute).
/// 6. `synthesize_descriptor(descriptor, resolved_slots, privacy_preserving)`
///    → `Bundle`.
/// 7. Determine `BundleMode` from `resolved_slots` entropy state +
///    cosigner count; route through `emit_unified` (existing
///    text/JSON/engraving renderer).
fn bundle_run_from_import_json<W: Write, E: Write>(
    args: &BundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    use crate::wallet_import::json_envelope::{
        cli_network_from_str, descriptor_body_no_csum, envelope_to_resolved_slots,
        parse_import_json_envelopes, read_import_json_arg,
    };
    use crate::wallet_import::pipeline::concrete_keys_to_placeholders;
    use bitcoin::secp256k1::Secp256k1;
    use zeroize::Zeroizing;

    let value = args
        .import_json
        .as_ref()
        .expect("caller checked --import-json.is_some()");
    let raw = read_import_json_arg(value, stdin, "--import-json")?;
    let envelope =
        parse_import_json_envelopes(&raw, args.import_json_index, "--import-json")?;

    // §3.6 — envelope.bundle.descriptor is the source-of-truth for the
    // descriptor; descriptor-mode synthesis applies. v0.27.0 wallet-
    // import path always emits Some.
    let descriptor_with_csum = envelope
        .bundle
        .descriptor
        .as_deref()
        .ok_or_else(|| {
            ToolkitError::BadInput(
                "--import-json: envelope.bundle.descriptor is null; v0.27.0 wallet-import \
                 path always emits the descriptor string verbatim"
                    .to_string(),
            )
        })?;
    let descriptor_body = descriptor_body_no_csum(descriptor_with_csum, "--import-json")?;

    // Decode mk1 chunks per §3.6.1 → ResolvedSlots (entropy=None). v0.27.1
    // Phase 2 I5 fold: stderr carries the origin_fingerprint substitution
    // NOTICE if any mk1 card omits the master fingerprint.
    let mut resolved_slots = envelope_to_resolved_slots(&envelope, stderr)?;

    // Network: env-derived. Used for entropy → xpub derivation in the
    // seed-overlay step + as the cross-check against args.network when
    // the user supplied one explicitly (--network defaults silently to
    // mainnet on the bundle subcommand; we use envelope.network as the
    // source-of-truth for the consumer path).
    let envelope_network = cli_network_from_str(&envelope.bundle.network)?;
    let secp = Secp256k1::new();

    // §3.6 — decode envelope's ms1[i] != "" entries first (envelope-
    // declared seed-bearing slots). These are equivalent to v0.26.0
    // "full" import-wallet's seed-overlay-on-emit state.
    let n = resolved_slots.len();
    if envelope.bundle.ms1.len() != n {
        return Err(ToolkitError::BadInput(format!(
            "--import-json: envelope.bundle.ms1 length {} disagrees with mk1 cosigner count {n}",
            envelope.bundle.ms1.len()
        )));
    }
    for (i, ms1_str) in envelope.bundle.ms1.iter().enumerate() {
        if ms1_str.is_empty() {
            continue;
        }
        let (_tag, payload) = ms_codec::decode(ms1_str).map_err(|e| {
            ToolkitError::BadInput(format!(
                "--import-json: envelope.bundle.ms1[{i}] decode failed: {e:?}"
            ))
        })?;
        // ms mnem Phase 3 (R2-I6): bind both Entr and Mnem payloads.
        // Populate slot.language from the wire for mnem cards (emit-only path —
        // this flow does NOT re-derive; xpub came from the envelope's mk1 chunk).
        //
        // C2 regression fix: an Entr wire card is language-AGNOSTIC and MUST stay
        // Entr on re-emit regardless of --language. Set language=English explicitly
        // so synthesize_descriptor's unwrap_or(run_language) never fabricates a
        // non-English language for it. An entr card emitted with English language
        // produces the Entr payload (synthesize_descriptor's emit_lang == English
        // branch → Payload::Entr), preserving the wire shape faithfully.
        let (entropy_bytes, slot_lang) = match payload {
            ms_codec::Payload::Entr(bytes) => (Zeroizing::new(bytes), bip39::Language::English),
            ms_codec::Payload::Mnem { entropy, language: wire_lang, .. } => {
                let lang = crate::language::wire_code_to_bip39(wire_lang)
                    .map_err(|e| ToolkitError::BadInput(format!(
                        "--import-json: envelope.bundle.ms1[{i}] {e}"
                    )))?;
                (Zeroizing::new(entropy), lang)
            }
            _ => {
                return Err(ToolkitError::BadInput(format!(
                    "--import-json: envelope.bundle.ms1[{i}] payload is not entropy"
                )));
            }
        };
        resolved_slots[i].entropy = Some(entropy_bytes);
        resolved_slots[i].language = Some(slot_lang);
    }

    // §3.6 + Q5 — apply user seed overlay (--slot @N.phrase=) on
    // watch-only cosigners. Conflict on non-empty envelope ms1[i] is
    // BadInput (exit 1). (--ms1 is import-wallet's input surface, not
    // bundle's — see the asymmetry note after this loop.)
    for (i, user_ms1) in args.slot.iter().filter_map(|s| {
        if s.subkey == crate::slot_input::SlotSubkey::Phrase {
            Some((s.index as usize, &s.value))
        } else {
            None
        }
    }) {
        if i >= n {
            return Err(ToolkitError::BadInput(format!(
                "--slot @{i}.phrase=: cosigner index out of range (envelope has {n} cosigners)"
            )));
        }
        if !envelope.bundle.ms1[i].is_empty() {
            return Err(ToolkitError::BadInput(format!(
                "--slot @{i}.phrase=: envelope already carries entropy for cosigner {i} \
                 (bundle.ms1[{i}] != \"\"); supply overlay only for watch-only slots"
            )));
        }
        // Resolve phrase → entropy → derive xpub at resolved_slots[i].path → verify match.
        let language = args.language.unwrap_or_default();
        // SAFETY: third-party-blocked — `bip39::Mnemonic` has no Drop+Zeroize.
        // FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`.
        let mnemonic = bip39::Mnemonic::parse_in(language.into(), user_ms1).map_err(|e| {
            ToolkitError::BadInput(format!(
                "--slot @{i}.phrase=: BIP-39 parse error: {e}"
            ))
        })?;
        let entropy = Zeroizing::new(mnemonic.to_entropy());
        let passphrase: Zeroizing<String> =
            Zeroizing::new(args.passphrase.clone().unwrap_or_default());
        let seed = crate::derive_slot::derive_master_seed(&mnemonic, &passphrase);
        // SAFETY: third-party-blocked — `bitcoin::bip32::Xpriv` is Copy + no
        // Drop; FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`.
        let master = BipXpriv::new_master(envelope_network.network_kind(), &seed[..])
            .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
        // SAFETY: third-party-blocked — `bitcoin::bip32::Xpriv` is Copy + no
        // Drop; FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`.
        let child = master
            .derive_priv(&secp, &resolved_slots[i].path)
            .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
        let derived_xpub = BipXpub::from_priv(&secp, &child);
        // Path-only-mk1-encoded xpub may differ from the user's derivation
        // by reconstructed depth+child_number — compare via (parent_fp,
        // chain_code, public_key) tuple per the Phase 4 holistic-review
        // mk-codec depth/child reconstruction note.
        if derived_xpub.parent_fingerprint != resolved_slots[i].xpub.parent_fingerprint
            || derived_xpub.chain_code != resolved_slots[i].xpub.chain_code
            || derived_xpub.public_key != resolved_slots[i].xpub.public_key
        {
            return Err(ToolkitError::ImportWalletSeedMismatch {
                cosigner_index: i,
                derived_xpub: derived_xpub.to_string(),
                blob_xpub: resolved_slots[i].xpub.to_string(),
                path: resolved_slots[i].origin_path_bare(),
            });
        }
        resolved_slots[i].entropy = Some(entropy);
        // C2 regression fix: set language explicitly from the user's --language
        // so synthesize_descriptor's unwrap_or(run_language) is moot for this
        // slot — the slot already carries the correct target language.
        resolved_slots[i].language = Some(language.into());
    }

    // --ms1 positional overlay (vs --slot @N.phrase=). Repeat with the
    // ms_codec entropy decode + same xpub-equivalence check.
    // BundleArgs doesn't currently expose --ms1 (that's import-wallet's
    // surface); the seed-overlay channel on `bundle --import-json` is
    // --slot @N.phrase= per the existing bundle subcommand convention.
    // Document the asymmetry in the import-wallet --json envelope's
    // overlay model — `--ms1` is the import-wallet-side surface and
    // already attaches entropy via Phase 4 emit → bundle.ms1[i] != "".
    // No `--ms1` arg on BundleArgs to handle here.

    // §3.6 — parse descriptor: concrete-keys → @N placeholders → md_codec.
    let (placeholder_form, parsed_keys, parsed_fps) =
        concrete_keys_to_placeholders(descriptor_body)?;
    let descriptor = crate::parse_descriptor::parse_descriptor(
        &placeholder_form,
        &parsed_keys,
        &parsed_fps,
    )
    .map_err(|e| {
        ToolkitError::DescriptorParse(format!(
            "--import-json: descriptor re-parse failed: {}",
            e.message()
        ))
    })?;

    // Synthesize. run_language defaults to English; import-json slots that
    // carry Some(wire_lang) override via unwrap_or in synthesize_descriptor,
    // so import-json behavior is unchanged by this parameter.
    let run_language_import: bip39::Language = args.language.unwrap_or_default().into();
    let bundle = synthesize_descriptor(&descriptor, &resolved_slots, args.privacy_preserving, run_language_import)?;

    // Determine BundleMode from resolved_slots state.
    let any_secret = resolved_slots.iter().any(|s| s.entropy.is_some());
    let any_watch = resolved_slots.iter().any(|s| s.entropy.is_none());
    let mode = match (n, any_secret, any_watch) {
        (1, true, _) => BundleMode::SingleSigFull,
        (1, false, _) => BundleMode::SingleSigWatchOnly,
        (_, true, true) => BundleMode::MultisigHybrid,
        (_, true, false) => BundleMode::MultisigMultiSource,
        (_, false, _) => BundleMode::MultisigWatchOnly,
    };

    // Emit. emit_unified derives its `descriptor` field from
    // `args.descriptor` / `args.descriptor_file`; the --import-json path
    // doesn't populate either, so we clone the args struct and inject
    // the envelope's descriptor (including #<csum>) so the emit-side
    // `descriptor` field round-trips faithfully through the new envelope.
    // This mirrors `cmd::bundle::run`'s descriptor-mode dispatch
    // pre-conditions; clap-mutex `conflicts_with_all` is bypassed at
    // this layer (the synthetic args is internal, not user-facing).
    let mut emit_args = args.clone();
    emit_args.descriptor = Some(descriptor_with_csum.to_string());
    // slip0132 signals are not applicable to envelope-sourced bundles
    // (the envelope's xpub field is canonical per SPEC §5.3).
    emit_unified(&emit_args, &bundle, &resolved_slots, mode, &[], stdout, stderr)?;

    if args.self_check {
        let entropy_bearing: Vec<bool> =
            resolved_slots.iter().map(|r| r.entropy.is_some()).collect();
        self_check_bundle(&bundle, args, &entropy_bearing)?;
    }

    Ok(())
}

/// v0.19.0 SPEC §4.12.b — compute the default origin path
/// `m/48'/<coin>'/<account>'/2'` for non-canonical descriptors with bare
/// `@N` placeholders. `<coin>` derives from `--network` (mainnet → `0'`,
/// testnet/signet/regtest → `1'` per BIP-44); `<account>` consumes
/// `--account N`. Public so verify-bundle's descriptor mode can mirror
/// the same default-inference per SPEC §4.11.c (symmetric verify-bundle
/// enforcement).
pub fn compute_default_origin_path(
    network: crate::network::CliNetwork,
    account: u32,
) -> md_codec::origin_path::OriginPath {
    use md_codec::origin_path::{OriginPath, PathComponent};
    OriginPath {
        components: vec![
            PathComponent {
                hardened: true,
                value: 48,
            },
            PathComponent {
                hardened: true,
                value: network.coin_type(),
            },
            PathComponent {
                hardened: true,
                value: account,
            },
            PathComponent {
                hardened: true,
                value: 2,
            },
        ],
    }
}

/// Convert a `bitcoin::bip32::DerivationPath` to `md_codec::origin_path::OriginPath`.
/// Used to fold `--slot @N.path=` user input into `path_decl.paths` for
/// non-canonical-descriptor default-inference override. Public for the
/// same symmetric-verify-bundle reason as `compute_default_origin_path`.
pub fn derivation_path_to_origin(
    dp: &DerivationPath,
) -> md_codec::origin_path::OriginPath {
    use bitcoin::bip32::ChildNumber;
    use md_codec::origin_path::{OriginPath, PathComponent};
    OriginPath {
        components: dp
            .into_iter()
            .map(|c| match c {
                ChildNumber::Normal { index } => PathComponent {
                    hardened: false,
                    value: *index,
                },
                ChildNumber::Hardened { index } => PathComponent {
                    hardened: true,
                    value: *index,
                },
            })
            .collect(),
    }
}

/// v0.19.0 SPEC §4.12.d — emit the stderr info notice naming the `@N`
/// indices that received the default path. Format byte-exact per SPEC.
fn emit_default_path_notice<E: Write>(
    stderr: &mut E,
    defaulted_indices: &[u8],
    network: crate::network::CliNetwork,
    account: u32,
) -> Result<(), ToolkitError> {
    if defaulted_indices.is_empty() {
        return Ok(());
    }
    let idx_list = defaulted_indices
        .iter()
        .map(|i| format!("@{i}"))
        .collect::<Vec<_>>()
        .join(",");
    let coin = network.coin_type();
    writeln!(
        stderr,
        "info: non-canonical descriptor; defaulting origin path for {idx_list} to m/48'/{coin}'/{account}'/2' (BIP-48 cosigner path). Override per-placeholder with [fp/path]@N or --slot @N.path=m/..."
    )
    .map_err(|e| ToolkitError::BadInput(format!("stderr write: {e}")))?;
    Ok(())
}

/// Convert a md-codec OriginPath to bitcoin::bip32::DerivationPath. Required
/// because the resolved descriptor placeholder carries the path in md-codec
/// shape but the binding logic operates on bitcoin types. Public for
/// symmetric verify-bundle (§4.11.c) which mirrors the descriptor-mode
/// binding loop and needs the same conversion.
pub fn origin_to_derivation_path(
    op: &md_codec::origin_path::OriginPath,
) -> Result<DerivationPath, ToolkitError> {
    let s = if op.components.is_empty() {
        "m".to_string()
    } else {
        let mut s = String::from("m");
        for c in &op.components {
            s.push('/');
            s.push_str(&c.value.to_string());
            if c.hardened {
                s.push('\'');
            }
        }
        s
    };
    DerivationPath::from_str(&s).map_err(|e| {
        ToolkitError::DescriptorParse(format!("descriptor @N annotation path parse failed: {e}"))
    })
}

pub fn self_check_bundle(
    bundle: &Bundle,
    args: &BundleArgs,
    entropy_bearing: &[bool],
) -> Result<(), ToolkitError> {
    // Phase 1 (RED): ms1 validation not yet implemented — see Phase 2.
    let _ = entropy_bearing;
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

// ============================================================================
// SPEC v0.9.0 §1 item 1 — argv-leakage closure helpers
// ============================================================================

/// Per-occurrence `secret-in-argv` stderr advisory emission. One advisory
/// per inline-secret site (per (flag, slot-index) pair) so the user sees
/// every leak site, not just the first. Emits to stderr only — no
/// argv-leakage data is logged or persisted.
fn emit_secret_in_argv_advisories<E: std::io::Write>(args: &BundleArgs, stderr: &mut E) {
    use crate::secret_advisory::secret_in_argv_warning;
    for s in &args.slot {
        if s.subkey.is_secret_bearing()
            && !s.is_stdin_sentinel()
            && !s.value.starts_with("@env:")
        {
            let flag = format!("--slot @{}.{}=", s.index, s.subkey.as_str());
            let alt = format!("--slot @{}.{}=-", s.index, s.subkey.as_str());
            secret_in_argv_warning(stderr, &flag, &alt);
        }
    }
    if let Some(pp) = args.passphrase.as_deref() {
        if !pp.starts_with("@env:") {
            secret_in_argv_warning(stderr, "--passphrase", "--passphrase-stdin");
        }
    }
}

/// Does the current invocation require stdin consumption for slot_stdin
/// or passphrase_stdin? Returns false when no stdin work is needed,
/// letting `run()` skip the clone-into-synthetic step.
fn needs_stdin_substitution(args: &BundleArgs) -> bool {
    args.passphrase_stdin || args.slot.iter().any(|s| s.is_stdin_sentinel())
}

/// v0.26.0 §3 — cheap pre-check for `@env:` sentinels on `bundle`'s
/// secret-bearing flag surfaces (`--passphrase`, secret-bearing `--slot`).
fn needs_env_sentinel_resolution(args: &BundleArgs) -> bool {
    let pp = args
        .passphrase
        .as_deref()
        .map(|v| v.starts_with("@env:"))
        .unwrap_or(false);
    let slot = args
        .slot
        .iter()
        .any(|s| s.subkey.is_secret_bearing() && s.value.starts_with("@env:"));
    pp || slot
}

/// v0.26.0 §3 — resolve `@env:<VAR>` sentinels across `bundle`'s
/// secret-bearing flag surfaces. Non-secret slot subkeys are NOT resolved
/// per SPEC §3.2 (opt-in per-callsite).
fn resolve_env_sentinels(args: &BundleArgs) -> Result<BundleArgs, ToolkitError> {
    use crate::env_sentinel::resolve_env_var_sentinel;
    let mut owned = args.clone();
    if let Some(pp) = owned.passphrase.as_ref() {
        owned.passphrase = Some(resolve_env_var_sentinel(pp, "--passphrase")?);
    }
    for s in owned.slot.iter_mut() {
        if s.subkey.is_secret_bearing() {
            let flag = format!("--slot @{}.{}=", s.index, s.subkey.as_str());
            s.value = resolve_env_var_sentinel(&s.value, &flag)?;
        }
    }
    Ok(owned)
}

/// Clone `args` into an owned `BundleArgs` and apply the stdin
/// substitution(s) (single-stdin-per-invocation: at most one of
/// `--passphrase-stdin` OR `--slot @N.<secret>=-` may be present).
fn apply_stdin_substitutions(
    args: &BundleArgs,
    stdin: &mut dyn std::io::Read,
) -> Result<BundleArgs, ToolkitError> {
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

#[cfg(test)]
mod self_check_ms1_tests {
    use super::*;

    // All-zero-entropy 24-word vector (mirrors synthesize.rs tests).
    const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    fn minimal_bundle_args() -> BundleArgs {
        BundleArgs {
            network: CliNetwork::Mainnet,
            template: None,
            descriptor: None,
            descriptor_file: None,
            language: None,
            passphrase: None,
            passphrase_stdin: false,
            account: 0,
            json: false,
            no_engraving_card: true,
            multisig_path_family: None,
            privacy_preserving: false,
            self_check: false,
            threshold: None,
            slot: vec![],
            import_json: None,
            import_json_index: None,
        }
    }

    /// A real 2-of-3 self-multisig bundle (md1/mk1/ms1 all valid + decodable).
    fn multisig_bundle() -> Bundle {
        let m = bip39::Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        crate::synthesize::synthesize_multisig_full(
            &m,
            "",
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            3,
            0,
            MultisigPathFamily::Bip87,
            false,
        )
        .unwrap()
    }

    /// RED→GREEN: self-check must DETECT a regressed ms1 emission (the @0-only
    /// reversion: ms1[0] populated, ms1[1+] wrongly cleared). RED against the
    /// pre-fix self_check_bundle (which ignores ms1 → returns Ok).
    #[test]
    fn self_check_detects_at0_only_ms1_regression() {
        let args = minimal_bundle_args();
        let entropy_bearing = vec![true, true, true]; // 3 phrase-bearing cosigners

        // Sanity: a correct full-mode multisig bundle self-checks Ok.
        let good = multisig_bundle();
        assert_eq!(good.ms1.len(), 3);
        assert!(
            good.ms1.iter().all(|s| !s.is_empty()),
            "fixture must emit a non-empty ms1 per cosigner"
        );
        assert!(
            self_check_bundle(&good, &args, &entropy_bearing).is_ok(),
            "a correct full-mode multisig must pass self-check"
        );

        // Regress: clear ms1[1] (the @0-only emission reversion).
        let mut bad = multisig_bundle();
        bad.ms1[1] = String::new();
        let r = self_check_bundle(&bad, &args, &entropy_bearing);
        assert!(
            r.is_err(),
            "self-check MUST detect the @0-only ms1 regression (ms1[1] cleared); got {r:?}"
        );
    }

    /// GREEN guard: a watch-only shape (all-empty ms1, no entropy-bearing slots)
    /// must PASS self-check (the check must not be over-eager).
    #[test]
    fn self_check_passes_watch_only_all_empty_ms1() {
        let args = minimal_bundle_args();
        let mut b = multisig_bundle();
        for s in b.ms1.iter_mut() {
            *s = String::new();
        }
        let entropy_bearing = vec![false, false, false];
        assert!(
            self_check_bundle(&b, &args, &entropy_bearing).is_ok(),
            "all-empty ms1 with no entropy-bearing slots (watch-only) must pass self-check"
        );
    }
}
