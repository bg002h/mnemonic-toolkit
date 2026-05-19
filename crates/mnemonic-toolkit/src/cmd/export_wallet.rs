//! `mnemonic export-wallet` subcommand.
//!
//! Realizes `design/SPEC_export_wallet_v0_8.md` (subcommand grammar, refusal
//! contract, per-vendor format emitters).

use crate::error::ToolkitError;
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::MultisigPathFamily;
use crate::template::CliTemplate;
use crate::wallet_export::{
    build_descriptor_string, script_type_from_descriptor, script_type_from_template,
    validate_watch_only, validate_watch_only_resolved, Bip388Emitter, BitcoinCoreEmitter,
    BsmsEmitter, BsmsForm, ColdcardEmitter, ElectrumEmitter, EmitInputs, GreenEmitter,
    JadeEmitter, SparrowEmitter, SpecterEmitter, TaprootInternalKey, TimestampArg,
    WalletFormatEmitter,
};
use clap::{Args, ValueEnum};
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CliExportFormat {
    #[value(name = "bitcoin-core")]
    BitcoinCore,
    #[value(name = "bip388")]
    Bip388,
    #[value(name = "coldcard")]
    Coldcard,
    #[value(name = "jade")]
    Jade,
    #[value(name = "sparrow")]
    Sparrow,
    #[value(name = "specter")]
    Specter,
    #[value(name = "electrum")]
    Electrum,
    #[value(name = "green")]
    Green,
    #[value(name = "bsms")]
    Bsms,
}

#[derive(Args, Debug)]
pub struct ExportWalletArgs {
    /// Pre-built template name. Mutually-required-one-of with --descriptor.
    #[arg(long, conflicts_with = "descriptor")]
    pub template: Option<CliTemplate>,

    /// User-supplied BIP-388 descriptor. Mutually exclusive with --template.
    #[arg(long)]
    pub descriptor: Option<String>,

    /// Multisig threshold K (1 ≤ K ≤ N).
    #[arg(long)]
    pub threshold: Option<u8>,

    /// v0.2 multisig path family (default: bip87).
    #[arg(long = "multisig-path-family", value_enum)]
    pub multisig_path_family: Option<MultisigPathFamily>,

    #[arg(long, default_value = "mainnet")]
    pub network: CliNetwork,

    /// Ignored (watch-only); kept for slot parser symmetry.
    #[arg(long)]
    pub language: Option<CliLanguage>,

    /// BIP-32 account index (default 0).
    #[arg(long, default_value = "0")]
    pub account: u32,

    /// Slot input — shape `@N.<subkey>=<value>`. Repeating.
    ///
    /// `<subkey>` is one of:
    ///   phrase       BIP-39 mnemonic (secret)
    ///   entropy      raw entropy hex (secret)
    ///   xpub         BIP-32 extended public key
    ///   fingerprint  4-byte master fingerprint (hex)
    ///   path         BIP-32 derivation path
    ///   wif          Wallet Import Format private key (secret)
    ///   xprv         BIP-32 extended private key (secret)
    ///
    /// `<value>` is the subkey's text form, or `-` to read from
    /// stdin. The 7 subkeys mirror `mnemonic bundle --slot`. For a
    /// watch-only export only `xpub` and `fingerprint` are required;
    /// secret subkeys are accepted but unnecessary.
    #[arg(
        long = "slot",
        action = clap::ArgAction::Append,
        value_parser = crate::slot_input::parse_slot_input,
        verbatim_doc_comment,
    )]
    pub slot: Vec<crate::slot_input::SlotInput>,

    /// Output format. Default bitcoin-core.
    #[arg(long, value_enum, default_value = "bitcoin-core")]
    pub format: CliExportFormat,

    /// Output path. `-` (default) → stdout.
    #[arg(long, default_value = "-")]
    pub output: String,

    /// Bitcoin Core `range` field, comma-separated. Default `0,999`.
    #[arg(long, default_value = "0,999", value_parser = parse_range)]
    pub range: (u32, u32),

    /// Bitcoin Core `timestamp` field. `now` (default) or unix seconds.
    #[arg(long, default_value = "now", value_parser = parse_timestamp)]
    pub timestamp: TimestampArgValue,

    /// Bitcoin Core target version. 24 or 25 (default 25).
    #[arg(long = "bitcoin-core-version", default_value = "25")]
    pub bitcoin_core_version: u8,

    /// SPEC v0.8 §2 — wallet name/label for formats that publish one
    /// (Coldcard generic JSON, Sparrow, Specter, Electrum). Optional;
    /// defaults to `<template-human-name>-<account>` (e.g., `bip84-0`,
    /// `wsh-sortedmulti-0`) when omitted. Ignored by formats that have
    /// no name slot (Bitcoin Core / BIP-388 / Jade text / Green).
    #[arg(long = "wallet-name")]
    pub wallet_name: Option<String>,

    /// SPEC v0.8 §7 — Taproot internal-key designation for
    /// `tr-multi-a` / `tr-sortedmulti-a` templates.
    ///
    /// Accepted values:
    ///   nums  BIP-341 reference NUMS x-only point (unspendable
    ///         internal key; common default)
    ///   @N    cosigner N's xpub as the key-path internal key
    ///         (cosigner N is then removed from the multi_a leaf
    ///         set; N is a decimal index 0..=N-1)
    ///
    /// Required under `tr-multi-a` / `tr-sortedmulti-a`; refused
    /// for non-Taproot templates.
    #[arg(
        long = "taproot-internal-key",
        value_parser = parse_taproot_internal_key_arg,
        verbatim_doc_comment,
    )]
    pub taproot_internal_key: Option<TaprootInternalKey>,

    /// SPEC v0.27.0 §3.5 — BSMS Round-2 emit shape. `4-line`
    /// (BIP-129-canonical) is the default; `2-line` is the lenient
    /// excerpt symmetric with the v0.26.0 import-side parser. Ignored
    /// by every other format.
    #[arg(long = "bsms-form", value_enum, default_value = "4-line")]
    pub bsms_form: BsmsForm,

    /// v0.27.0 — emit a per-format wallet config from an
    /// `import-wallet --json` envelope rather than from `--template` /
    /// `--descriptor`. Accepts a file path or `-` to read the envelope
    /// from stdin. The envelope's `bundle.descriptor` becomes the
    /// canonical descriptor for the emitter; cosigner xpubs decode
    /// from `bundle.mk1` per SPEC §3.6.1; network derives from
    /// `bundle.network`. Mutually exclusive with `--template` and
    /// `--descriptor`. `--account` is rejected (the envelope's
    /// `bundle.account` value applies; the account is encoded in the
    /// descriptor's origin paths).
    #[arg(
        long = "from-import-json",
        value_name = "FILE|-",
        conflicts_with_all = ["template", "descriptor"],
    )]
    pub from_import_json: Option<String>,

    /// v0.27.0 — pick a specific entry from a multi-entry envelope
    /// array. Required when the envelope has > 1 entry.
    #[arg(
        long = "from-import-json-index",
        value_name = "N",
        requires = "from_import_json",
    )]
    pub from_import_json_index: Option<usize>,
}

/// SPEC v0.8 §7 parser: `nums` or `@N` (decimal index).
fn parse_taproot_internal_key_arg(s: &str) -> Result<TaprootInternalKey, String> {
    if s == "nums" {
        return Ok(TaprootInternalKey::Nums);
    }
    if let Some(n) = s.strip_prefix('@') {
        let idx: u8 = n.parse().map_err(|e| {
            format!("--taproot-internal-key {s:?}: cosigner index must be u8: {e}")
        })?;
        return Ok(TaprootInternalKey::Cosigner(idx));
    }
    Err(format!(
        "--taproot-internal-key must be 'nums' or '@N' (cosigner index); got {s:?}",
    ))
}

#[derive(Debug, Clone, Copy)]
pub struct TimestampArgValue(pub TimestampArg);

fn parse_range(s: &str) -> Result<(u32, u32), String> {
    let (a, b) = s.split_once(',').ok_or_else(|| {
        format!("--range expects '<start>,<end>' (comma-separated u32 pair); got {s:?}")
    })?;
    let start: u32 = a.parse().map_err(|e| format!("--range start: {e}"))?;
    let end: u32 = b.parse().map_err(|e| format!("--range end: {e}"))?;
    if start > end {
        return Err(format!("--range start {start} must be <= end {end}"));
    }
    Ok((start, end))
}

fn parse_timestamp(s: &str) -> Result<TimestampArgValue, String> {
    if s == "now" {
        Ok(TimestampArgValue(TimestampArg::Now))
    } else {
        let n: i64 = s
            .parse()
            .map_err(|e| format!("--timestamp expects 'now' or unix seconds; got {s:?}: {e}"))?;
        if n < 0 {
            return Err(format!("--timestamp unix seconds must be >= 0; got {n}"));
        }
        Ok(TimestampArgValue(TimestampArg::Unix(n)))
    }
}

/// `_stderr` is unused: export-wallet is watch-only by SPEC §3, so the
/// secret-on-stdout warning never fires; the parameter exists for callsite
/// symmetry with the other subcommands.
pub fn run<W: Write, E: Write>(
    args: &ExportWalletArgs,
    stdout: &mut W,
    _stderr: &mut E,
) -> Result<(), ToolkitError> {
    // All six formats are now real (Phase 3 promoted Specter); no stubs.

    // v0.27.0 — `--from-import-json` dispatch short-circuits before slot /
    // template / descriptor pre-checks. The envelope carries everything the
    // emitter needs (descriptor + per-cosigner xpubs/fingerprints/paths).
    // Mutex with --template / --descriptor is enforced by clap via
    // `conflicts_with_all` on the `--from-import-json` arg.
    if args.from_import_json.is_some() {
        return run_from_import_json(args, stdout);
    }

    // SPEC §3 fast-path watch-only validator on the user-supplied raw slot
    // inputs. The SPEC-mandated invariant ("runs on the resolved-slot set") is
    // additionally enforced by `validate_watch_only_resolved` after
    // `resolve_slots` returns (see template branch below).
    validate_watch_only(&args.slot)?;

    // v0.8.2 SPEC §5.1 — `master_xpub` slot subkey is now plumbed through
    // `ResolvedSlot.master_xpub` and surfaced on `EmitInputs.master_xpub_at_0`
    // for the Coldcard generic-JSON emitter. The Phase 1.9 refuse-on-supply
    // guard is retired. Other formats silently ignore the subkey per the
    // per-format ignored-input contract.

    // SPEC v0.8 §7 — `tr-multi-a` / `tr-sortedmulti-a` require
    // `--taproot-internal-key`. The flag designates the BIP-341 internal key
    // (NUMS or cosigner index) for the canonical `tr(<internal>,multi_a(K,...))`
    // construction. v0.7 refused these templates outright; v0.8 supports them
    // with the internal-key designation.
    if let Some(template) = args.template {
        if matches!(template, CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA)
            && args.taproot_internal_key.is_none()
        {
            return Err(ToolkitError::BadInput(format!(
                "--template {} requires --taproot-internal-key (use 'nums' for an unspendable BIP-341 NUMS point, or '@N' to designate cosigner N as the key-path internal key)",
                match template {
                    CliTemplate::TrMultiA => "tr-multi-a",
                    CliTemplate::TrSortedMultiA => "tr-sortedmulti-a",
                    _ => unreachable!(),
                },
            )));
        }
    }
    // SPEC v0.8 §7 — `--taproot-internal-key` is taproot-multisig-only.
    if args.taproot_internal_key.is_some()
        && !matches!(
            args.template,
            Some(CliTemplate::TrMultiA) | Some(CliTemplate::TrSortedMultiA)
        )
    {
        return Err(ToolkitError::BadInput(
            "--taproot-internal-key applies only to --template tr-multi-a / tr-sortedmulti-a".into(),
        ));
    }

    // Mutual-exclusion + minimal arg surface checks.
    if args.descriptor.is_some() && args.template.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "descriptor",
            flag: "--template",
            message: "--descriptor and --template are mutually exclusive",
        });
    }
    if args.descriptor.is_none() && args.template.is_none() {
        return Err(ToolkitError::BadInput(
            "export-wallet requires either --template or --descriptor".into(),
        ));
    }

    // `resolved_template` carries the (resolved-slots, template, k) tuple
    // forward to the bip388 branch so we don't double-call `resolve_slots`.
    let mut resolved_template: Option<(Vec<crate::synthesize::ResolvedSlot>, CliTemplate, u8)> =
        None;

    let canonical = if let Some(desc) = &args.descriptor {
        // Descriptor passthrough: parse + canonicalize via miniscript.
        use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
        use std::str::FromStr;
        let d = MsDescriptor::<DescriptorPublicKey>::from_str(desc)
            .map_err(|e| ToolkitError::DescriptorParse(format!("export-wallet --descriptor: {e}")))?;
        d.to_string()
    } else {
        let template = args.template.expect("checked above");
        // Resolve slots through the shared bundle helper. Watch-only-only at
        // this point — phrase/entropy/xprv/wif rejected by validate_watch_only.
        let (resolved, _slip0132_signals) = crate::cmd::bundle::resolve_slots(
            &args.slot,
            template,
            args.network,
            args.account,
            args.language,
            None,
        )?;
        // SPEC §3 invariant: validator runs on the resolved-slot set.
        validate_watch_only_resolved(&resolved)?;
        let n = resolved.len() as u8;
        if n == 0 {
            return Err(ToolkitError::BadInput(
                "export-wallet: at least one --slot @N.xpub=... required".into(),
            ));
        }
        // For taproot multisig with a cosigner-internal key, the multi_a leaf
        // set has N-1 cosigners (one becomes the key-path key). Default
        // threshold = N for non-taproot or NUMS-internal; = N-1 when a
        // cosigner is internal. Caller may still override via --threshold.
        let leaf_count = match (template, args.taproot_internal_key) {
            (
                CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA,
                Some(TaprootInternalKey::Cosigner(_)),
            ) => n - 1,
            _ => n,
        };
        // n=1 cosigner-internal degenerate case: removing the only cosigner
        // leaves no multi_a leaves. Refuse cleanly here rather than letting
        // miniscript reject `multi_a(0,)` with an opaque parse error.
        if leaf_count == 0 {
            return Err(ToolkitError::BadInput(
                "--taproot-internal-key @N with a single cosigner leaves no multi_a leaves; supply at least 2 cosigners (or use --taproot-internal-key nums for unspendable key-path)".into(),
            ));
        }
        let k = args.threshold.unwrap_or(leaf_count);
        if k > leaf_count {
            // Distinguish multi_a-leaf-count error (cosigner-internal taproot)
            // from the general cosigner-count error to keep the existing
            // refusal text stable for non-taproot multisig.
            let msg = if leaf_count != n {
                format!("--threshold {k} exceeds multi_a leaf count {leaf_count} (one cosigner is the taproot internal key)")
            } else {
                format!("--threshold {k} exceeds cosigner count {n}")
            };
            return Err(ToolkitError::BadInput(msg));
        }
        if matches!(template, CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA)
            && matches!(
                args.taproot_internal_key,
                Some(TaprootInternalKey::Cosigner(_))
            )
        {
            // Validate cosigner index range.
            if let Some(TaprootInternalKey::Cosigner(idx)) = args.taproot_internal_key {
                if idx >= n {
                    return Err(ToolkitError::BadInput(format!(
                        "--taproot-internal-key @{idx} out of range; only {n} cosigners supplied",
                    )));
                }
            }
        }
        let canonical = build_descriptor_string(
            template,
            &resolved,
            k,
            args.network,
            args.account,
            args.taproot_internal_key,
        )?;
        resolved_template = Some((resolved, template, k));
        canonical
    };

    // Build EmitInputs once, after slot resolution + descriptor canonicalization
    // + watch-only validation. Each WalletFormatEmitter borrows this struct.
    // SPEC v0.8 §12 — trait dispatch replaces the per-format `Value` match
    // (which was followed by `serde_json::to_string_pretty`). Each emitter now
    // returns its final `String` directly; the v0.7 byte-exact fixtures for
    // `bitcoin-core` / `bip388` remain valid because `to_string_pretty` is
    // deterministic for a given input `Value`.
    let (resolved_slots_ref, template_opt, threshold_opt): (
        &[crate::synthesize::ResolvedSlot],
        Option<CliTemplate>,
        Option<u8>,
    ) = match &resolved_template {
        Some((slots, tmpl, k)) => (slots.as_slice(), Some(*tmpl), Some(*k)),
        None => (&[], None, None),
    };

    let script_type = if let Some(t) = template_opt {
        script_type_from_template(&t)
    } else {
        // Descriptor path: parse once for script-type classification.
        use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
        use std::str::FromStr;
        let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(&canonical).map_err(|e| {
            ToolkitError::DescriptorParse(format!("export-wallet script-type derive: {e}"))
        })?;
        script_type_from_descriptor(&parsed)?
    };

    // SPEC v0.8 §2 — wallet name: user-supplied via `--wallet-name <STRING>`
    // (Phase 1.7 wiring); falls back to `<template-human-name>-<account>` for
    // the template path or `"imported-descriptor"` for the descriptor path.
    // The `wallet_name_was_user_supplied` flag lets Phase 3 SpecterEmitter
    // distinguish user-supplied from default — Specter requires explicit names.
    let wallet_name_resolved: String = match args.wallet_name.as_ref() {
        Some(name) => name.clone(),
        None => match template_opt {
            Some(t) => format!("{}-{}", t.human_name(), args.account),
            None => "imported-descriptor".to_string(),
        },
    };

    // v0.8.2 SPEC §5.1 — master_xpub plumbing. Surface @0.master_xpub= into
    // `EmitInputs.master_xpub_at_0` when slot 0 carries one (other slots'
    // master_xpub fields are not consumed by any current emitter, but
    // ResolvedSlot retains them for future per-cosigner needs).
    let master_xpub_at_0 = resolved_slots_ref.first().and_then(|s| s.master_xpub);

    let inputs = EmitInputs {
        canonical_descriptor: &canonical,
        resolved_slots: resolved_slots_ref,
        template: template_opt,
        script_type,
        network: args.network,
        account: args.account,
        threshold: threshold_opt,
        threshold_user_supplied: args.threshold.is_some(),
        wallet_name: &wallet_name_resolved,
        wallet_name_was_user_supplied: args.wallet_name.is_some(),
        taproot_internal_key: args.taproot_internal_key,
        range: args.range,
        timestamp: args.timestamp.0,
        bitcoin_core_version: args.bitcoin_core_version,
        master_xpub_at_0,
        bsms_form: args.bsms_form,
    };

    // SPEC §4 missing-info channel — every emitter exposes a per-format
    // `collect_missing` predicate; non-empty result short-circuits to the
    // deterministic refusal via `ToolkitError::ExportWalletMissingFields`
    // (which routes through `build_missing_fields_refusal`).
    let (missing, format_name): (Vec<crate::wallet_export::MissingField>, &'static str) =
        match args.format {
            CliExportFormat::BitcoinCore => (BitcoinCoreEmitter::collect_missing(&inputs), "bitcoin-core"),
            CliExportFormat::Bip388 => (Bip388Emitter::collect_missing(&inputs), "bip388"),
            CliExportFormat::Coldcard => (ColdcardEmitter::collect_missing(&inputs), "coldcard"),
            CliExportFormat::Jade => (JadeEmitter::collect_missing(&inputs), "jade"),
            CliExportFormat::Sparrow => (SparrowEmitter::collect_missing(&inputs), "sparrow"),
            CliExportFormat::Specter => (SpecterEmitter::collect_missing(&inputs), "specter"),
            CliExportFormat::Electrum => (ElectrumEmitter::collect_missing(&inputs), "electrum"),
            CliExportFormat::Green => (GreenEmitter::collect_missing(&inputs), "green"),
            CliExportFormat::Bsms => (BsmsEmitter::collect_missing(&inputs), "bsms"),
        };
    if !missing.is_empty() {
        return Err(ToolkitError::ExportWalletMissingFields {
            format: format_name,
            missing,
        });
    }

    let emitted: String = match args.format {
        CliExportFormat::BitcoinCore => BitcoinCoreEmitter::emit(&inputs),
        CliExportFormat::Bip388 => Bip388Emitter::emit(&inputs),
        CliExportFormat::Coldcard => ColdcardEmitter::emit(&inputs),
        CliExportFormat::Jade => JadeEmitter::emit(&inputs),
        CliExportFormat::Sparrow => SparrowEmitter::emit(&inputs),
        CliExportFormat::Specter => SpecterEmitter::emit(&inputs),
        CliExportFormat::Electrum => ElectrumEmitter::emit(&inputs),
        CliExportFormat::Green => GreenEmitter::emit(&inputs),
        CliExportFormat::Bsms => BsmsEmitter::emit(&inputs),
    }?;

    if args.output == "-" {
        // v0.27.0 Phase 6.5 PR-review C1 fold: propagate stdout write
        // failure (broken pipe / disk full / closed handle) as a typed
        // I/O error rather than silently exiting 0 with empty stdout.
        writeln!(stdout, "{emitted}").map_err(ToolkitError::Io)?;
    } else {
        std::fs::write(&args.output, format!("{emitted}\n"))
            .map_err(|e| ToolkitError::BadInput(format!("--output {}: {e}", args.output)))?;
    }
    Ok(())
}

/// v0.27.0 Phase 5 entry — `export-wallet --from-import-json <FILE|->`.
/// Consumes an `import-wallet --json` envelope (SPEC §3.2 wire shape;
/// Phase 4 ship) and emits a per-format wallet config. Per plan §3.7 +
/// §3.7.1 (16-field EmitInputs contract).
fn run_from_import_json<W: Write>(
    args: &ExportWalletArgs,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    use crate::wallet_export::script_type_from_descriptor;
    use crate::wallet_import::json_envelope::{
        cli_network_from_str, descriptor_body_no_csum, envelope_to_resolved_slots,
        parse_import_json_envelopes,
    };

    // §3.7 Q6 — `--account` on the --from-import-json path is BadInput
    // (the envelope's bundle.account is authoritative; the user supplying
    // a separate account index is ambiguous / contradictory).
    if args.account != 0 {
        return Err(ToolkitError::BadInput(
            "--account is meaningful only with --template / --descriptor; \
             --from-import-json reads the account from the envelope (bundle.account)"
                .to_string(),
        ));
    }

    let value = args
        .from_import_json
        .as_ref()
        .expect("caller checked --from-import-json.is_some()");
    let raw = if value == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(ToolkitError::Io)?;
        buf
    } else {
        std::fs::read_to_string(value).map_err(|e| {
            ToolkitError::BadInput(format!("--from-import-json: read {value}: {e}"))
        })?
    };
    let envelope = parse_import_json_envelopes(
        &raw,
        args.from_import_json_index,
        "--from-import-json",
    )?;

    let descriptor_with_csum = envelope
        .bundle
        .descriptor
        .as_deref()
        .ok_or_else(|| {
            ToolkitError::BadInput(
                "--from-import-json: envelope.bundle.descriptor is null; v0.27.0 \
                 wallet-import path always emits the descriptor string verbatim"
                    .to_string(),
            )
        })?;
    // canonicalize: store the descriptor body without `#<csum>` so the
    // miniscript parse below succeeds. BIP-380 checksum validated up-
    // front; failure is `BadInput` (Phase 5 R0 I1 fold) rather than
    // silently passing through to a downstream miniscript parse error.
    let canonical_descriptor =
        descriptor_body_no_csum(descriptor_with_csum, "--from-import-json")?.to_string();

    // Decode mk1 → ResolvedSlots per §3.6.1.
    let resolved_slots = envelope_to_resolved_slots(&envelope)?;

    // Derive network from envelope.
    let network = cli_network_from_str(&envelope.bundle.network)?;

    // Script-type from the parsed descriptor (canonical form sans checksum).
    use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
    use std::str::FromStr;
    let parsed_ms = MsDescriptor::<DescriptorPublicKey>::from_str(&canonical_descriptor)
        .map_err(|e| {
            ToolkitError::DescriptorParse(format!(
                "--from-import-json: descriptor parse for script-type derivation: {e}"
            ))
        })?;
    let script_type = script_type_from_descriptor(&parsed_ms)?;

    // Wallet name: --wallet-name explicit OR default `imported-descriptor`
    // (same as the descriptor-mode path at run() line ~385-391).
    let wallet_name_resolved: String = args
        .wallet_name
        .clone()
        .unwrap_or_else(|| "imported-descriptor".to_string());

    // Threshold from envelope's bundle.multisig.threshold (None for N=1).
    let threshold = envelope.bundle.multisig.as_ref().map(|m| m.threshold);

    let inputs = EmitInputs {
        canonical_descriptor: &canonical_descriptor,
        resolved_slots: &resolved_slots,
        // template is always None for descriptor-mode (envelope is
        // always descriptor-mode per §3.2.1).
        template: None,
        script_type,
        network,
        account: envelope.bundle.account,
        threshold,
        threshold_user_supplied: false, // envelope-derived, not user-supplied
        wallet_name: &wallet_name_resolved,
        wallet_name_was_user_supplied: args.wallet_name.is_some(),
        // taproot internal key: v0.27.0 wallet-import path doesn't surface
        // taproot internal-key designation; reject taproot envelopes here
        // (file FOLLOWUP `wallet-import-taproot-internal-key` for v0.28+).
        taproot_internal_key: None,
        range: args.range,
        timestamp: args.timestamp.0,
        bitcoin_core_version: args.bitcoin_core_version,
        master_xpub_at_0: None,
        bsms_form: args.bsms_form,
    };

    // §3.7 — per-format missing-info channel + dispatch. Mirror the
    // template-path dispatch at run() line ~422-449.
    let (missing, format_name): (Vec<crate::wallet_export::MissingField>, &'static str) =
        match args.format {
            CliExportFormat::BitcoinCore => {
                (BitcoinCoreEmitter::collect_missing(&inputs), "bitcoin-core")
            }
            CliExportFormat::Bip388 => (Bip388Emitter::collect_missing(&inputs), "bip388"),
            CliExportFormat::Coldcard => (ColdcardEmitter::collect_missing(&inputs), "coldcard"),
            CliExportFormat::Jade => (JadeEmitter::collect_missing(&inputs), "jade"),
            CliExportFormat::Sparrow => (SparrowEmitter::collect_missing(&inputs), "sparrow"),
            CliExportFormat::Specter => (SpecterEmitter::collect_missing(&inputs), "specter"),
            CliExportFormat::Electrum => (ElectrumEmitter::collect_missing(&inputs), "electrum"),
            CliExportFormat::Green => (GreenEmitter::collect_missing(&inputs), "green"),
            CliExportFormat::Bsms => (BsmsEmitter::collect_missing(&inputs), "bsms"),
        };
    if !missing.is_empty() {
        return Err(ToolkitError::ExportWalletMissingFields {
            format: format_name,
            missing,
        });
    }

    let emitted: String = match args.format {
        CliExportFormat::BitcoinCore => BitcoinCoreEmitter::emit(&inputs),
        CliExportFormat::Bip388 => Bip388Emitter::emit(&inputs),
        CliExportFormat::Coldcard => ColdcardEmitter::emit(&inputs),
        CliExportFormat::Jade => JadeEmitter::emit(&inputs),
        CliExportFormat::Sparrow => SparrowEmitter::emit(&inputs),
        CliExportFormat::Specter => SpecterEmitter::emit(&inputs),
        CliExportFormat::Electrum => ElectrumEmitter::emit(&inputs),
        CliExportFormat::Green => GreenEmitter::emit(&inputs),
        CliExportFormat::Bsms => BsmsEmitter::emit(&inputs),
    }?;

    if args.output == "-" {
        // v0.27.0 Phase 6.5 PR-review C1 fold: propagate stdout write
        // failure (broken pipe / disk full / closed handle) as a typed
        // I/O error rather than silently exiting 0 with empty stdout.
        writeln!(stdout, "{emitted}").map_err(ToolkitError::Io)?;
    } else {
        std::fs::write(&args.output, format!("{emitted}\n"))
            .map_err(|e| ToolkitError::BadInput(format!("--output {}: {e}", args.output)))?;
    }
    Ok(())
}
