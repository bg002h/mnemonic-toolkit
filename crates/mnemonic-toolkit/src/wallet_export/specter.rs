//! SPEC v0.8 §8 — Specter Desktop wallet-import emitter.
//!
//! Format reference (canonical import-shape authority, NOT the REST GET
//! schema): <https://github.com/cryptoadvance/specter-desktop/blob/master/src/cryptoadvance/specter/util/wallet_importer.py>.
//!
//! Shape: a single JSON object with four fields in this order:
//! `label`, `blockheight`, `descriptor`, `devices`.
//!
//! - `label`: the user-supplied `--wallet-name` (REQUIRED — SPEC §13 R1-L1
//!   hardening: Specter's UX requires a wallet label, and emitting an empty
//!   string produces a wallet that displays as blank in the Specter UI;
//!   `SpecterEmitter::collect_missing` returns `MissingField::WalletName`
//!   when `--wallet-name` was not user-supplied).
//! - `blockheight`: `0` by default; the `--blockheight <N>` flag is
//!   deferred to FOLLOWUPS.
//! - `descriptor`: canonical BIP-380 descriptor WITH `#<checksum>` suffix
//!   (Specter expects the checksum; cross-verifies against the bitcoin-core
//!   branch byte-exact via `pipeline::build_descriptor_string`).
//! - `devices`: array of vendor strings; length = cosigner count for
//!   multisig (1 for singlesig). The toolkit emits `"unknown"` placeholders
//!   because cosigner-vendor metadata is not threaded through the codecs.

use super::{EmitInputs, MissingField, WalletFormatEmitter};
use crate::error::ToolkitError;
use serde::Serialize;

/// SPEC v0.8 §8 — `WalletFormatEmitter` impl for `--format specter`.
pub(crate) struct SpecterEmitter;

impl WalletFormatEmitter for SpecterEmitter {
    fn collect_missing(inputs: &EmitInputs) -> Vec<MissingField> {
        // SPEC §13 R1-L1: Specter requires explicit `--wallet-name`.
        let mut out = Vec::new();
        if !inputs.wallet_name_was_user_supplied {
            out.push(MissingField::WalletName);
        }
        out
    }

    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
        emit_specter_wallet_json(inputs)
    }

    fn extension() -> &'static str {
        "json"
    }
}

#[derive(Serialize)]
struct SpecterWallet<'a> {
    label: &'a str,
    blockheight: u32,
    descriptor: &'a str,
    devices: Vec<&'static str>,
}

/// SPEC v0.8 §8 — Specter Desktop wallet JSON emitter.
pub(crate) fn emit_specter_wallet_json(inputs: &EmitInputs) -> Result<String, ToolkitError> {
    // Cosigner count: 1 for singlesig (descriptor-passthrough may have 0
    // resolved slots — fall back to 1 in that case since Specter expects
    // at least one device entry).
    let n = inputs.resolved_slots.len().max(1);
    let devices: Vec<&'static str> = vec!["unknown"; n];

    let wallet = SpecterWallet {
        label: inputs.wallet_name,
        blockheight: 0,
        descriptor: inputs.canonical_descriptor,
        devices,
    };

    serde_json::to_string_pretty(&wallet)
        .map_err(|e| ToolkitError::BadInput(format!("--format specter: serialize: {e}")))
}
