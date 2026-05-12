//! SPEC v0.8 §6 — Blockstream Jade wallet-import emitter.
//!
//! Jade's multisig registration format
//! (`register_multisig.multisig_file` alternative; reference:
//! <https://github.com/Blockstream/Jade/blob/master/docs/index.rst>) accepts
//! the same text shape Coldcard's multisig export produces. We therefore
//! delegate the multisig path directly to `emit_coldcard_multisig_text`
//! (SPEC §5.2 emitter) and refuse the cases Jade does not support:
//! - Singlesig: Jade selects address type on-device after seed restore; no
//!   native file-import surface exists for watch-only singlesig.
//! - Taproot multisig (tr-multi-a / tr-sortedmulti-a): pending Jade firmware
//!   support — tracked by FOLLOWUPS `jade-tr-multi-a-pending-firmware`.

use super::coldcard::emit_coldcard_multisig_text;
use super::{EmitInputs, MissingField, WalletFormatEmitter};
use crate::error::ToolkitError;

/// SPEC v0.8 §6 — `WalletFormatEmitter` impl for `--format jade`. Multisig
/// delegates byte-identical to Coldcard §5.2; singlesig + taproot-multisig
/// refuse.
pub(crate) struct JadeEmitter;

impl WalletFormatEmitter for JadeEmitter {
    fn collect_missing(_inputs: &EmitInputs) -> Vec<MissingField> {
        Vec::new()
    }

    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
        use crate::template::CliTemplate;
        let template = inputs.template.ok_or_else(|| {
            ToolkitError::BadInput(
                "--format jade requires --template (wsh-multi / wsh-sortedmulti / sh-wsh-multi / sh-wsh-sortedmulti); descriptor passthrough is not supported by Jade's file-import surface".into(),
            )
        })?;
        match template {
            // Multisig wsh / sh-wsh: delegate byte-identical to Coldcard.
            CliTemplate::WshMulti
            | CliTemplate::WshSortedMulti
            | CliTemplate::ShWshMulti
            | CliTemplate::ShWshSortedMulti => emit_coldcard_multisig_text(inputs),
            // Taproot multisig: refuse pending firmware support.
            CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA => Err(ToolkitError::BadInput(format!(
                "--format jade does not yet support --template {} — Blockstream Jade firmware does not currently ingest taproot multisig wallet config (tracked by FOLLOWUPS jade-tr-multi-a-pending-firmware). Use --format bitcoin-core (descriptor) or --format sparrow for taproot multisig watch-only setup.",
                template.human_name(),
            ))),
            // Singlesig templates: refuse with the §6 byte-exact pointer.
            CliTemplate::Bip44
            | CliTemplate::Bip49
            | CliTemplate::Bip84
            | CliTemplate::Bip86 => Err(ToolkitError::BadInput(
                "error: mnemonic export-wallet --format jade emits multisig wallet config only; for singlesig setups Jade reads the seed on-device. Use --format coldcard for a singlesig JSON or --format bitcoin-core for a descriptor.".into(),
            )),
        }
    }

    fn extension() -> &'static str {
        "txt"
    }
}
