//! `mnemonic export-wallet` subcommand.
//!
//! Realizes `design/SPEC_export_wallet_v0_7.md` §2 (grammar), §3 (refusal),
//! §4 (descriptor pipeline), §5 (Bitcoin Core importdescriptors), §6 (BIP-388
//! wallet_policy), §7 (Sparrow/Specter stubs).

use crate::error::ToolkitError;
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::MultisigPathFamily;
use crate::template::CliTemplate;
use crate::wallet_export::{
    build_descriptor_string, script_type_from_descriptor, script_type_from_template,
    validate_watch_only, validate_watch_only_resolved, Bip388Emitter, BitcoinCoreEmitter,
    ColdcardEmitter, EmitInputs, TaprootInternalKey, TimestampArg, WalletFormatEmitter,
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
    #[value(name = "sparrow")]
    Sparrow,
    #[value(name = "specter")]
    Specter,
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

    /// `@N.<subkey>=<value>` slot input, repeating.
    #[arg(long = "slot", action = clap::ArgAction::Append, value_parser = crate::slot_input::parse_slot_input)]
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

    /// SPEC v0.8 §7 — Taproot internal-key designation for `tr-multi-a` /
    /// `tr-sortedmulti-a` templates. `nums` selects the BIP-341 reference
    /// NUMS x-only point. `@N` selects cosigner N's xpub as the key-path
    /// internal key (cosigner N is then removed from the multi_a leaf set).
    #[arg(long = "taproot-internal-key", value_parser = parse_taproot_internal_key_arg)]
    pub taproot_internal_key: Option<TaprootInternalKey>,
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
    // Sparrow/Specter stubs (SPEC §7) refuse before any work.
    match args.format {
        CliExportFormat::Sparrow => {
            return Err(ToolkitError::ExportWalletFormatStub("sparrow"));
        }
        CliExportFormat::Specter => {
            return Err(ToolkitError::ExportWalletFormatStub("specter"));
        }
        _ => {}
    }

    // SPEC §3 fast-path watch-only validator on the user-supplied raw slot
    // inputs. The SPEC-mandated invariant ("runs on the resolved-slot set") is
    // additionally enforced by `validate_watch_only_resolved` after
    // `resolve_slots` returns (see template branch below).
    validate_watch_only(&args.slot)?;

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

    // Phase 0: `--wallet-name` flag not yet exposed (added in Phase 1). Default
    // is `<template-human-name>-<account>` for the template path, or
    // `"imported-descriptor"` for the descriptor path. `wallet_name_was_user_supplied`
    // is unconditionally `false` until Phase 1 wires the clap flag.
    let wallet_name_default = match template_opt {
        Some(t) => format!("{}-{}", t.human_name(), args.account),
        None => "imported-descriptor".to_string(),
    };

    let inputs = EmitInputs {
        canonical_descriptor: &canonical,
        resolved_slots: resolved_slots_ref,
        template: template_opt,
        script_type,
        network: args.network,
        account: args.account,
        threshold: threshold_opt,
        wallet_name: &wallet_name_default,
        wallet_name_was_user_supplied: false,
        taproot_internal_key: args.taproot_internal_key,
        range: args.range,
        timestamp: args.timestamp.0,
        bitcoin_core_version: args.bitcoin_core_version,
    };

    let emitted: String = match args.format {
        CliExportFormat::BitcoinCore => BitcoinCoreEmitter::emit(&inputs),
        CliExportFormat::Bip388 => Bip388Emitter::emit(&inputs),
        CliExportFormat::Coldcard => ColdcardEmitter::emit(&inputs),
        CliExportFormat::Sparrow | CliExportFormat::Specter => unreachable!("stubbed above"),
    }?;

    if args.output == "-" {
        let _ = writeln!(stdout, "{emitted}");
    } else {
        std::fs::write(&args.output, format!("{emitted}\n"))
            .map_err(|e| ToolkitError::BadInput(format!("--output {}: {e}", args.output)))?;
    }
    Ok(())
}
