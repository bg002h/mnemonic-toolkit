//! CLI dispatch for `mnemonic build-descriptor` (SPEC §2 + §4).
//!
//! Takes a versioned JSON `PolicyNode` spec (`--spec <FILE|->`), runs it through
//! the validation gate (`descriptor_builder::gate`), and emits a validated
//! `wsh(M)` descriptor + BIP-388 wallet-policy + a cost preview. On a gate
//! failure it emits node-addressed diagnostics and exits 2.

use std::io::{IsTerminal, Read, Write};

use clap::{Args, ValueEnum};
use serde_json::{json, Value};

use crate::cost::{self, CompareCostArgs, InputForm};
use crate::descriptor_builder::gate::{self, Diagnostic, ValidatedPolicy};
use crate::descriptor_builder::ir::SpecDoc;
use crate::descriptor_builder::schema;
use crate::derive_address::derive_receive_addresses;
use crate::error::ToolkitError;
use crate::network::CliNetwork;
use crate::wallet_export::descriptor_to_bip388_wallet_policy;

#[derive(Args, Debug)]
pub struct BuildDescriptorArgs {
    /// JSON node-tree spec: a file path, or `-` for stdin. If omitted, stdin is
    /// read when it is not a TTY.
    #[arg(long)]
    pub spec: Option<String>,

    /// Target network (default mainnet). Used for the human-view first receive
    /// address; the descriptor / bip388 / cost output is network-agnostic (the
    /// xpubs carry the network).
    #[arg(long, value_enum)]
    pub network: Option<CliNetwork>,

    /// Output a single bare artifact instead of the rich human view:
    /// `descriptor` = the concrete `wsh(M)#checksum`; `bip388` = the BIP-388
    /// wallet-policy JSON. Omit for the human view (descriptor + first address +
    /// cost table). Overridden by `--json`.
    #[arg(long, value_enum)]
    pub format: Option<CliBuildFormat>,

    /// Emit a structured JSON envelope (`{descriptor, bip388, cost, diagnostics}`)
    /// for the GUI. On a gate failure: `{diagnostics: [...]}` with exit 2.
    #[arg(long)]
    pub json: bool,

    /// Dump the versioned node-tree `--spec-schema` JSON (the grammar the GUI +
    /// presets consume) and exit; ignores all other inputs.
    #[arg(long)]
    pub spec_schema: bool,
}

/// Bare output formats (the rich human view is the no-`--format` default).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum CliBuildFormat {
    /// The concrete `wsh(M)#checksum`.
    Descriptor,
    /// The BIP-388 wallet-policy JSON.
    Bip388,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &BuildDescriptorArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // `--spec-schema` short-circuits everything.
    if args.spec_schema {
        writeln!(stdout, "{}", schema::spec_schema_string()).map_err(ToolkitError::Io)?;
        return Ok(0);
    }

    let spec_text = read_spec(args, stdin)?;
    let doc = SpecDoc::parse(&spec_text)
        .map_err(|e| ToolkitError::BuildDescriptorSpec(e.to_string()))?;

    let validated = match gate::validate(&doc) {
        Ok(vp) => vp,
        Err(diags) => {
            emit_diagnostics(&diags, args.json, stdout, stderr)?;
            return Ok(2);
        }
    };

    emit(&validated, args, stdout)?;
    Ok(0)
}

fn read_spec<R: Read>(args: &BuildDescriptorArgs, stdin: &mut R) -> Result<String, ToolkitError> {
    let read_stdin = |stdin: &mut R| -> Result<String, ToolkitError> {
        let mut buf = String::new();
        stdin
            .read_to_string(&mut buf)
            .map_err(|e| ToolkitError::BuildDescriptorSpec(format!("--spec stdin read: {e}")))?;
        Ok(buf)
    };
    match args.spec.as_deref() {
        Some("-") => read_stdin(stdin),
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| ToolkitError::BuildDescriptorSpec(format!("--spec {path}: {e}"))),
        None => {
            if std::io::stdin().is_terminal() {
                Err(ToolkitError::BuildDescriptorSpec(
                    "build-descriptor: no spec; supply --spec <FILE|-> (or pipe JSON to stdin)"
                        .to_string(),
                ))
            } else {
                read_stdin(stdin)
            }
        }
    }
}

fn emit_diagnostics<W: Write, E: Write>(
    diags: &[Diagnostic],
    as_json: bool,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    if as_json {
        let env = json!({ "diagnostics": diags });
        writeln!(
            stdout,
            "{}",
            serde_json::to_string_pretty(&env).map_err(|e| ToolkitError::BuildDescriptorSpec(
                format!("diagnostics serialize: {e}")
            ))?
        )
        .map_err(ToolkitError::Io)?;
    } else {
        writeln!(stderr, "build-descriptor: refused — {} diagnostic(s):", diags.len())
            .map_err(ToolkitError::Io)?;
        for d in diags {
            writeln!(stderr, "  [{}] {}: {}", d.kind.as_str(), d.node_path, d.message)
                .map_err(ToolkitError::Io)?;
        }
    }
    Ok(())
}

fn emit<W: Write>(
    vp: &ValidatedPolicy,
    args: &BuildDescriptorArgs,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    // Canonical descriptor (with BIP-380 checksum) via the round-trip idiom.
    let canonical = vp.descriptor.to_string();
    let bip388 = descriptor_to_bip388_wallet_policy(&canonical)?;

    if args.json {
        let cost = cost_preview_value(vp)?;
        let env = json!({
            "descriptor": canonical,
            "bip388": bip388,
            "cost": cost,
            "diagnostics": [],
        });
        writeln!(
            stdout,
            "{}",
            serde_json::to_string_pretty(&env)
                .map_err(|e| ToolkitError::BuildDescriptorSpec(format!("envelope serialize: {e}")))?
        )
        .map_err(ToolkitError::Io)?;
        return Ok(());
    }

    match args.format {
        Some(CliBuildFormat::Descriptor) => {
            writeln!(stdout, "{canonical}").map_err(ToolkitError::Io)?;
        }
        Some(CliBuildFormat::Bip388) => {
            writeln!(
                stdout,
                "{}",
                serde_json::to_string_pretty(&bip388)
                    .map_err(|e| ToolkitError::BuildDescriptorSpec(format!("bip388 serialize: {e}")))?
            )
            .map_err(ToolkitError::Io)?;
        }
        None => emit_human(vp, args, &canonical, stdout)?,
    }
    Ok(())
}

fn emit_human<W: Write>(
    vp: &ValidatedPolicy,
    args: &BuildDescriptorArgs,
    canonical: &str,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    let network = args.network.unwrap_or(CliNetwork::Mainnet);
    writeln!(stdout, "descriptor:\n{canonical}\n").map_err(ToolkitError::Io)?;

    // First receive address (best-effort; never fails the emit).
    if let Ok(addrs) = derive_receive_addresses(&vp.descriptor, 1, network.to_bitcoin_network()) {
        if let Some(a) = addrs.first() {
            writeln!(stdout, "first receive address ({network:?}):\n{a}\n")
                .map_err(ToolkitError::Io)?;
        }
    }

    writeln!(stdout, "cost preview (wsh vs tr, per spending condition):")
        .map_err(ToolkitError::Io)?;
    let single = single_path_descriptor(vp)?;
    cost::run_compare_cost(
        &CompareCostArgs {
            input: InputForm::Descriptor(single),
            feerate_sat_per_vb: 1.0,
            max_conditions: gate::DEFAULT_PREVIEW_CAP,
            json: false,
        },
        stdout,
    )?;
    Ok(())
}

/// Single-path projection of the multipath descriptor for cost enumeration
/// (cost is path-invariant; `derive_at_index` errors on multipath — SPEC §4 I2).
fn single_path_descriptor(vp: &ValidatedPolicy) -> Result<String, ToolkitError> {
    let singles = vp.descriptor.clone().into_single_descriptors().map_err(|e| {
        ToolkitError::BuildDescriptorSpec(format!("multipath split for cost preview: {e}"))
    })?;
    singles
        .first()
        .map(|d| d.to_string())
        .ok_or_else(|| ToolkitError::BuildDescriptorSpec("multipath split produced no branch".into()))
}

fn cost_preview_value(vp: &ValidatedPolicy) -> Result<Value, ToolkitError> {
    let single = single_path_descriptor(vp)?;
    let mut buf: Vec<u8> = Vec::new();
    cost::run_compare_cost(
        &CompareCostArgs {
            input: InputForm::Descriptor(single),
            feerate_sat_per_vb: 1.0,
            max_conditions: gate::DEFAULT_PREVIEW_CAP,
            json: true,
        },
        &mut buf,
    )?;
    serde_json::from_slice(&buf)
        .map_err(|e| ToolkitError::BuildDescriptorSpec(format!("cost preview parse: {e}")))
}
