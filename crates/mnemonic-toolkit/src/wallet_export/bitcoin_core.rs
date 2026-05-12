//! SPEC §5 — Bitcoin Core `importdescriptors` JSON emitter.

use super::TimestampArg;
use crate::error::ToolkitError;
use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
use serde_json::{json, Value};
use std::str::FromStr;

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
