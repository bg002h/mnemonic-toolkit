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
    build_descriptor_string, format_bip388_wallet_policy,
    format_bitcoin_core_importdescriptors, validate_watch_only,
    validate_watch_only_resolved, TimestampArg,
};
use clap::{Args, ValueEnum};
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CliExportFormat {
    #[value(name = "bitcoin-core")]
    BitcoinCore,
    #[value(name = "bip388")]
    Bip388,
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

    // Refuse `tr-multi-a` / `tr-sortedmulti-a` — `tr(multi_a(...))` is malformed
    // grammar (miniscript requires `tr(<internal-key>, <taptree>)`); proper
    // construction needs an internal-key designation (NUMS vs key-path key)
    // and is deferred to v0.8.
    if let Some(template) = args.template {
        match template {
            CliTemplate::TrMultiA => {
                return Err(ToolkitError::ExportWalletTaprootMultisigUnsupported(
                    "tr-multi-a",
                ));
            }
            CliTemplate::TrSortedMultiA => {
                return Err(ToolkitError::ExportWalletTaprootMultisigUnsupported(
                    "tr-sortedmulti-a",
                ));
            }
            _ => {}
        }
    }

    // BIP-388 + `--descriptor` passthrough are mutually unsupported in v0.7
    // (BIP-388 needs `@N/**` placeholders; --descriptor substitutes concrete
    // xpubs at the canonical layer). Fire this refusal BEFORE the canonical
    // descriptor parse so the user gets the more-specific message rather than
    // a parse error.
    if args.format == CliExportFormat::Bip388 && args.descriptor.is_some() {
        return Err(ToolkitError::BadInput(
            "--format bip388 requires --template (descriptor passthrough not supported in v0.7)".into(),
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
        let k = args.threshold.unwrap_or(n);
        if k > n {
            return Err(ToolkitError::BadInput(format!(
                "--threshold {k} exceeds cosigner count {n}"
            )));
        }
        // SPEC §5.5.a — verify-bundle Option B: suppress slip0132 info-line on
        // export-wallet (consistent with verify-bundle's read-only checker
        // semantics; see self-review report).
        let canonical =
            build_descriptor_string(template, &resolved, k, args.network, args.account)?;
        resolved_template = Some((resolved, template, k));
        canonical
    };

    let value = match args.format {
        CliExportFormat::BitcoinCore => format_bitcoin_core_importdescriptors(
            &canonical,
            args.range,
            args.timestamp.0,
            args.bitcoin_core_version,
        )?,
        CliExportFormat::Bip388 => {
            // BIP-388: render template + slots directly so description_template
            // uses @N/** placeholders (canonical descriptor with concrete xpubs
            // does not). `--descriptor` passthrough was refused above; reaching
            // here implies the template branch ran and `resolved_template` is
            // populated.
            let (resolved, template, k) = resolved_template.expect(
                "Bip388 reached without resolved_template; --descriptor refused upstream",
            );
            format_bip388_wallet_policy(template, &resolved, k, args.network, args.account)?
        }
        CliExportFormat::Sparrow | CliExportFormat::Specter => unreachable!("stubbed above"),
    };

    let serialized = serde_json::to_string_pretty(&value)
        .map_err(|e| ToolkitError::BadInput(format!("export-wallet json: {e}")))?;

    if args.output == "-" {
        let _ = writeln!(stdout, "{serialized}");
    } else {
        std::fs::write(&args.output, format!("{serialized}\n"))
            .map_err(|e| ToolkitError::BadInput(format!("--output {}: {e}", args.output)))?;
    }
    Ok(())
}
