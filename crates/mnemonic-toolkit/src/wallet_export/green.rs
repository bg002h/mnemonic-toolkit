//! SPEC v0.8 §10 — Blockstream Green wallet-import emitter.
//!
//! Green has no native descriptor-import file shape; the Help Center
//! documents pasting the descriptor or xpub into Green's "Import from file"
//! dialog. Reference (Zendesk-hosted, may return 403 to programmatic
//! fetchers): <https://help.blockstream.com/hc/en-us/articles/19340800530713-Set-up-watch-only-wallet>.
//!
//! Toolkit emits a thin 3-line text file:
//!
//! ```text
//! # Blockstream Green — Watch-only import (singlesig)
//! # Help: https://help.blockstream.com/hc/en-us/articles/19340800530713-Set-up-watch-only-wallet
//! <canonical-descriptor>
//! ```
//!
//! Multisig: REFUSE with pointer at Green's server-mediated multisig
//! surface (FOLLOWUPS `green-native-multisig-pending-server-support`).

use super::{EmitInputs, MissingField, WalletFormatEmitter};
use crate::error::ToolkitError;

/// SPEC v0.8 §10 — `WalletFormatEmitter` impl for `--format green`.
pub(crate) struct GreenEmitter;

impl WalletFormatEmitter for GreenEmitter {
    fn collect_missing(_inputs: &EmitInputs) -> Vec<MissingField> {
        Vec::new()
    }

    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
        // v0.28.7 — Slug 2: refuse multisig in BOTH template-mode and
        // descriptor-mode (--from-import-json). Previously the refusal was
        // gated on `inputs.template.is_some()`, which silently passed multisig
        // descriptor-mode invocations. See FOLLOWUP
        // `green-emitter-multisig-refusal-template-only` (resolved v0.28.7).
        if inputs.script_type.is_multisig() {
            return Err(ToolkitError::BadInput(
                "--format green does not support multisig — Blockstream Green's multisig setup is server-mediated (Green Multisig Shield) and not a file-import surface (tracked by FOLLOWUPS green-native-multisig-pending-server-support). Use --format bitcoin-core (descriptor) or --format sparrow for multisig watch-only.".into(),
            ));
        }
        Ok(format!(
            "# Blockstream Green — Watch-only import (singlesig)\n# Help: https://help.blockstream.com/hc/en-us/articles/19340800530713-Set-up-watch-only-wallet\n{}",
            inputs.canonical_descriptor,
        ))
    }

    fn extension() -> &'static str {
        "txt"
    }
}
