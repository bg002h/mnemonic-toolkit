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
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Args, Debug, Clone)]
pub struct BundleArgs {
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

    #[arg(long)]
    pub json: bool,

    #[arg(long = "no-engraving-card")]
    pub no_engraving_card: bool,

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

    /// v0.4 unified slot input. Repeating flag — one occurrence per
    /// (slot, subkey) tuple. Grammar: `@N.<subkey>=<value>` where N is
    /// the slot index (u8) and subkey is one of phrase / entropy / xpub /
    /// fingerprint / path / wif / xprv. Phase B lands the parser; Phase C
    /// wires it into the unified `bundle_run` dispatch.
    #[arg(long = "slot", action = clap::ArgAction::Append, value_parser = crate::slot_input::parse_slot_input)]
    pub slot: Vec<crate::slot_input::SlotInput>,
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
    // downstream success/error (the xprv-slot rejection at L470+ still
    // surfaces, but the user has already been warned about the leak).
    emit_secret_in_argv_advisories(args, stderr);
    let synthetic_args;
    let args: &BundleArgs = if needs_stdin_substitution(args) {
        synthetic_args = apply_stdin_substitutions(args, stdin)?;
        &synthetic_args
    } else {
        args
    };

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
    if descriptor_mode && args.account != 0 {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--account",
            message: mode_text::DESCRIPTOR_WITH_NONZERO_ACCOUNT,
        });
    }
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

    // Resolve slots into ResolvedSlot vec.
    let (resolved, slip0132_signals) = resolve_slots(
        &slots,
        template,
        args.network,
        args.account,
        args.language,
        args.passphrase.as_deref(),
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
        )?,
    };

    // Emit (reuse legacy text/JSON renderer; engraving card omitted for now;
    // unified card lands in Phase I).
    emit_unified(args, &bundle, &resolved, mode, &slip0132_signals, stdout, stderr)?;

    if args.self_check {
        self_check_bundle(&bundle, args)?;
    }
    Ok(())
}

/// v0.4.1 H.5 BIP-388 distinct-key check on ResolvedSlot vector. Mirrors
/// `check_key_vector_distinctness` for the unified path; comparison key
/// is `(xpub.to_string(), path_raw)` raw-string equality per SPEC §4.11.b.
fn check_resolved_slots_distinctness(slots: &[ResolvedSlot]) -> Result<(), ToolkitError> {
    for i in 0..slots.len() {
        for j in (i + 1)..slots.len() {
            if slots[i].xpub.to_string() == slots[j].xpub.to_string()
                && slots[i].path_raw == slots[j].path_raw
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
    for (idx, slot_inputs) in by_index {
        let subkeys: std::collections::BTreeSet<SlotSubkey> =
            slot_inputs.iter().map(|s| s.subkey).collect();
        if subkeys.contains(&SlotSubkey::Phrase) {
            let phrase = slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::Phrase)
                .map(|s| s.value.as_str())
                .expect("contains() asserts presence");
            let lang = language.unwrap_or_default();
            let pass = passphrase.unwrap_or("");
            let acc = crate::derive::derive_full(
                phrase, pass, lang, network, template, account,
            )?;
            let path_raw = acc.account_path.to_string();
            out.push(ResolvedSlot {
                xpub: acc.account_xpub,
                fingerprint: acc.master_fingerprint,
                path: acc.account_path,
                path_raw,
                entropy: Some(acc.entropy),
                master_xpub: None,
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
            let (path, path_raw) = match slot_inputs
                .iter()
                .find(|s| s.subkey == SlotSubkey::Path)
            {
                Some(p) => {
                    let parsed = DerivationPath::from_str(&p.value).map_err(|e| {
                        ToolkitError::BadInput(format!("--slot @{idx}.path parse: {e}"))
                    })?;
                    (parsed, p.value.clone())
                }
                None => {
                    // v0.5.1: Path absent → fall back to template's per-network
                    // origin path so xpub-only watch-only slots can verify
                    // against fixtures built at the same path.
                    let dp = template.derivation_path(network, account);
                    let raw = dp.to_string();
                    (dp, raw)
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
                path_raw,
                entropy: None,
                master_xpub,
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
            let pass = passphrase.unwrap_or("");
            let acc = crate::derive_slot::derive_bip32_from_entropy(
                &entropy_bytes, pass, lang, network, template, account,
            )?;
            let path_raw = acc.account_path.to_string();
            out.push(ResolvedSlot {
                xpub: acc.account_xpub,
                fingerprint: acc.master_fingerprint,
                path: acc.account_path,
                path_raw,
                entropy: Some(entropy_bytes),
                master_xpub: None,
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
                network: network.network_kind().into(),
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
                path_raw: String::new(),
                entropy: None,
                master_xpub: None,
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
    // normalize_origin_path "" → "m" branch). path_raw.is_empty() is the
    // SPEC §4.11.b absent-path sentinel; null is the JSON wire-format absent.
    fn origin_path_for_json(path_raw: &str) -> Option<String> {
        if path_raw.is_empty() {
            None
        } else {
            Some(normalize_origin_path(path_raw))
        }
    }

    if args.json {
        let template = args.template.map(|t| t.human_name());
        let (multisig_info, origin_path, origin_paths) = if n == 1 {
            (None, origin_path_for_json(&resolved[0].path_raw), None)
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
                    origin_path: normalize_origin_path(&s.path_raw),
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
    // SPEC v0.6.1 §5.5.a: secret-on-stdout warning — last stderr write,
    // matches convert.rs §7 byte-exactly. Fires only when at least one ms1
    // slot is non-empty (BIP-39 entropy is on stdout); watch-only invocations
    // (all ms1 == "" sentinels per §5.8) suppress it.
    if bundle.any_secret_bearing() {
        let _ = writeln!(
            stderr,
            "warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')"
        );
    }
    Ok(())
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
    // the synthesis output).
    let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
    let md1_chunk_set_id = match md_codec::chunk::reassemble(&md1_strs)
        .ok()
        .and_then(|d| md_codec::compute_wallet_policy_id(&d).ok())
    {
        Some(pid) => {
            let bytes = pid.as_bytes();
            format!("{:02x}{:02x}", bytes[0], bytes[1])
        }
        None => "????".to_string(),
    };

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
                origin_path: if s.path_raw.is_empty() {
                    None
                } else {
                    Some(s.path_raw.clone())
                },
            }
        })
        .collect();

    let input = BundleInputForCard {
        network: args.network.human_name(),
        template_or_descriptor: TemplateOrDescriptor::Template(template_str),
        threshold: args.threshold.or(if n > 1 { Some(n) } else { None }),
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
    let resolved_placeholders = resolve_placeholders(&occs)?;
    let n = resolved_placeholders.n as usize;

    if slots.iter().map(|s| s.index as usize + 1).max().unwrap_or(0) != n {
        return Err(ToolkitError::DescriptorParse(format!(
            "descriptor has n={n} placeholders but --slot vec covers {} slots",
            slots.iter().map(|s| s.index as usize + 1).max().unwrap_or(0)
        )));
    }

    // Resolve each @i slot using the per-@i annotation path from the descriptor.
    let secp = Secp256k1::new();
    let mut by_index: BTreeMap<u8, Vec<&crate::slot_input::SlotInput>> = BTreeMap::new();
    for s in slots {
        by_index.entry(s.index).or_default().push(s);
    }

    let mut cosigners: Vec<CosignerKeyInfo> = Vec::with_capacity(n);
    let mut entropy_at_0: Option<Vec<u8>> = None;
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

        let (xpub, fingerprint, path, path_raw, ent_opt) = if subkeys
            .contains(&crate::slot_input::SlotSubkey::Phrase)
        {
            let phrase = slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Phrase)
                .map(|s| s.value.as_str())
                .expect("contains() asserts presence");
            let language = args.language.unwrap_or_default();
            let passphrase = args.passphrase.clone().unwrap_or_default();
            let mnemonic = Bip39Mnemonic::parse_in(language.into(), phrase)
                .map_err(ToolkitError::Bip39)?;
            let entropy = mnemonic.to_entropy();
            let seed = mnemonic.to_seed(&passphrase);
            let master = BipXpriv::new_master(args.network.network_kind(), &seed)
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
            let acct_xpriv = master.derive_priv(&secp, &anno_path).map_err(|e| {
                ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e))
            })?;
            let xpub = BipXpub::from_priv(&secp, &acct_xpriv);
            (xpub, master_fp, anno_path.clone(), anno_path.to_string(), Some(entropy))
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
            let (path, path_raw) = match slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Path)
            {
                Some(p) => {
                    let parsed = DerivationPath::from_str(&p.value).map_err(|e| {
                        ToolkitError::BadInput(format!("--slot @{idx}.path parse: {e}"))
                    })?;
                    (parsed, p.value.clone())
                }
                None => (anno_path.clone(), anno_path.to_string()),
            };
            (xpub, fp, path, path_raw, None)
        } else if subkeys.contains(&crate::slot_input::SlotSubkey::Entropy) {
            let entropy_hex = slot_inputs
                .iter()
                .find(|s| s.subkey == crate::slot_input::SlotSubkey::Entropy)
                .map(|s| s.value.as_str())
                .expect("contains() asserts presence");
            let entropy_bytes = hex::decode(entropy_hex).map_err(|e| {
                ToolkitError::BadInput(format!(
                    "--slot @{idx}.entropy hex-decode: {e}"
                ))
            })?;
            let language = args.language.unwrap_or_default();
            let passphrase = args.passphrase.clone().unwrap_or_default();
            let mnemonic = Bip39Mnemonic::from_entropy_in(language.into(), &entropy_bytes)
                .map_err(ToolkitError::Bip39)?;
            let seed = mnemonic.to_seed(&passphrase);
            let master = BipXpriv::new_master(args.network.network_kind(), &seed)
                .map_err(|e| {
                    ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e))
                })?;
            let master_fp = master.fingerprint(&secp);
            let acct_xpriv = master.derive_priv(&secp, &anno_path).map_err(|e| {
                ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e))
            })?;
            let xpub = BipXpub::from_priv(&secp, &acct_xpriv);
            (xpub, master_fp, anno_path.clone(), anno_path.to_string(), Some(entropy_bytes))
        } else {
            return Err(ToolkitError::BadInput(format!(
                "--slot @{idx} subkey set {:?} not supported in descriptor mode in v0.4.2 \
                (xprv → v0.5+; wif → multisig FOLLOWUP; partial xpub may be supported but \
                requires full descriptor binding context — file a FOLLOWUP if needed)",
                subkeys.iter().map(|s| s.as_str()).collect::<Vec<_>>()
            )));
        };

        // v0.4.3 Phase N: per-slot entropy goes on the ResolvedSlot directly.
        cosigners.push(CosignerKeyInfo {
            xpub,
            fingerprint,
            path,
            path_raw,
            entropy: ent_opt.clone(),
            master_xpub: None,
        });
        if idx == 0 {
            entropy_at_0 = ent_opt;
        }

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
    // already carry path_raw + entropy per slot post-v0.4.3 N alias merge).
    let _ = &entropy_at_0; // entropy is on cosigners[0] already; remove this binding
    let dummy_binding = crate::parse_descriptor::DescriptorBinding {
        keys: keys.clone(),
        fingerprints: fingerprints.clone(),
        cosigners: cosigners.clone(),
    };
    crate::parse_descriptor::check_key_vector_distinctness(&dummy_binding)?;

    // Build md-codec Descriptor + synthesize.
    let descriptor = parse_descriptor(&descriptor_str, &keys, &fingerprints)?;
    let bundle = synthesize_descriptor(
        &descriptor,
        &cosigners,
        entropy_at_0.as_deref(),
        args.privacy_preserving,
    )?;

    // Reuse emit_unified renderer (resolved must be reconstructed as
    // ResolvedSlot vec for engraving card; entropy field tracks per-slot).
    let resolved_slots: Vec<ResolvedSlot> = cosigners
        .iter()
        .enumerate()
        .map(|(i, c)| ResolvedSlot {
            xpub: c.xpub,
            fingerprint: c.fingerprint,
            path: c.path.clone(),
            path_raw: c.path_raw.clone(),
            entropy: if i == 0 { entropy_at_0.clone() } else { None },
            master_xpub: None,
        })
        .collect();

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
        self_check_bundle(&bundle, args)?;
    }

    Ok(())
}

/// Convert a md-codec OriginPath to bitcoin::bip32::DerivationPath. Required
/// because the resolved descriptor placeholder carries the path in md-codec
/// shape but the binding logic operates on bitcoin types.
fn origin_to_derivation_path(
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

/// Does the current invocation require stdin consumption for slot_stdin
/// or passphrase_stdin? Returns false when no stdin work is needed,
/// letting `run()` skip the clone-into-synthetic step.
fn needs_stdin_substitution(args: &BundleArgs) -> bool {
    args.passphrase_stdin || args.slot.iter().any(|s| s.is_stdin_sentinel())
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
