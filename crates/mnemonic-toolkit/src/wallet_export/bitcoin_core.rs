//! SPEC §5 — Bitcoin Core `importdescriptors` JSON emitter.

use super::{EmitInputs, MissingField, TimestampArg, WalletFormatEmitter};
use crate::error::ToolkitError;
use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
use serde_json::{json, Value};
use std::str::FromStr;

/// SPEC v0.8 §12 — `WalletFormatEmitter` impl for `--format bitcoin-core`.
/// Thin-wraps `format_bitcoin_core_importdescriptors` (a `Value` builder) with
/// `serde_json::to_string_pretty` to return the final `String`. The byte-exact
/// v0.7 fixture for `--format bitcoin-core` remains valid because
/// `to_string_pretty` is deterministic for a given `Value` input.
pub(crate) struct BitcoinCoreEmitter;

impl WalletFormatEmitter for BitcoinCoreEmitter {
    fn collect_missing(_inputs: &EmitInputs) -> Vec<MissingField> {
        // Bitcoin Core `importdescriptors` takes the canonical descriptor
        // as-is; missing fields surface as descriptor-parse errors upstream
        // rather than as §4 missing-info refusals.
        Vec::new()
    }

    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
        let value = format_bitcoin_core_importdescriptors(
            inputs.canonical_descriptor,
            inputs.range,
            inputs.timestamp,
            inputs.bitcoin_core_version,
        )?;
        serde_json::to_string_pretty(&value)
            .map_err(|e| ToolkitError::BadInput(format!("export-wallet json: {e}")))
    }

    fn extension() -> &'static str {
        "json"
    }
}

/// SPEC §5: emit Bitcoin Core `importdescriptors` JSON. Multipath `<0;1>`
/// splits into 2 entries (receive `internal: false`, change `internal: true`).
pub(crate) fn format_bitcoin_core_importdescriptors(
    canonical_descriptor: &str,
    range: (u32, u32),
    timestamp: TimestampArg,
    _bitcoin_core_version: u8,
) -> Result<Value, ToolkitError> {
    let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(canonical_descriptor)
        .map_err(|e| ToolkitError::DescriptorParse(format!("export-wallet re-parse: {e}")))?;

    let entries: Vec<Value> = if parsed.is_multipath() {
        let parts = parsed
            .clone()
            .into_single_descriptors()
            .map_err(|e| ToolkitError::DescriptorParse(format!("multipath split: {e}")))?;
        if parts.len() != 2 {
            return Err(ToolkitError::DescriptorParse(format!(
                "expected 2 multipath splits (receive/change), got {}",
                parts.len()
            )));
        }
        parts
            .into_iter()
            .enumerate()
            .map(|(i, p)| {
                json!({
                    "desc": p.to_string(),
                    "active": true,
                    "internal": i == 1,
                    "range": [range.0, range.1],
                    "timestamp": timestamp.to_json(),
                })
            })
            .collect()
    } else {
        vec![json!({
            "desc": parsed.to_string(),
            "active": true,
            "internal": false,
            "range": [range.0, range.1],
            "timestamp": timestamp.to_json(),
        })]
    };

    Ok(Value::Array(entries))
}
