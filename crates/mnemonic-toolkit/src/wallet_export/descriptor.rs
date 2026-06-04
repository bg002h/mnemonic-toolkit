//! Bare canonical descriptor emitter: `<descriptor>#<checksum>` on one line,
//! no wallet-file wrapper. Works for single-sig AND multisig (unlike `green`,
//! which is Green-wallet-targeted and refuses multisig). The descriptor + its
//! BIP-380 checksum are already computed in `EmitInputs.canonical_descriptor`.
use super::{EmitInputs, MissingField, WalletFormatEmitter};
use crate::error::ToolkitError;

/// `WalletFormatEmitter` impl for `--format descriptor`.
pub(crate) struct DescriptorEmitter;

impl WalletFormatEmitter for DescriptorEmitter {
    fn collect_missing(_inputs: &EmitInputs) -> Vec<MissingField> {
        Vec::new()
    }

    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
        // NO trailing `\n` — the dispatch tail (writeln! / format!("{emitted}\n"),
        // export_wallet.rs) adds it (matches green.rs). `CheckedDescriptor`'s
        // Display impl yields the canonical multipath `<descriptor>#<checksum>`.
        Ok(inputs.canonical_descriptor.to_string())
    }

    fn extension() -> &'static str {
        "txt"
    }
}
