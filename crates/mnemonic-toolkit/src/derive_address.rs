//! First-address derivation for BIP-129 BSMS Round-2 line 4 + import-side
//! audit verification.
//!
//! Per `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md` §3.5.
//! Used by:
//! - `wallet_export::bsms::BsmsEmitter` — line 4 of the 4-line Round-2 emit.
//! - `wallet_import::bsms::BsmsParser` — informational WARNING when the
//!   6-line audit field's declared first-address disagrees with the
//!   toolkit's locally-derived first-address (closes FOLLOWUP
//!   `bsms-first-address-verify` at `design/FOLLOWUPS.md:2083`).
//!
//! Canonical derivation point: receive branch index 0, address index 0
//! (i.e., `/0/0` relative to the descriptor's cosigner xpubs). For multipath
//! `<0;1>/*` descriptors, this splits to the receive branch and renders at
//! definite index 0. For non-multipath descriptors with wildcards, it
//! derives at definite index 0 directly. Non-wildcard descriptors render
//! their unique address.

use crate::error::ToolkitError;
use miniscript::descriptor::{DefiniteDescriptorKey, DescriptorPublicKey};
use miniscript::Descriptor;

/// Derive the wallet's first address at `/0/0` for a watch-only descriptor.
/// Reserved for non-taproot descriptors (BIP-386 is outside BIP-129
/// prerequisites); the caller must reject `tr(...)` before invoking.
pub(crate) fn derive_first_address(
    descriptor: &Descriptor<DescriptorPublicKey>,
    network: bitcoin::Network,
) -> Result<String, ToolkitError> {
    // Multipath `<0;1>/*` descriptors must be split before derivation.
    // `into_single_descriptors()` returns one entry per multipath alternative
    // in declaration order; for the canonical `<0;1>` shape that means
    // [receive, change]. We render from receive (index 0).
    let single = if descriptor.is_multipath() {
        let mut parts = descriptor.clone().into_single_descriptors().map_err(|e| {
            ToolkitError::DescriptorParse(format!(
                "first-address: multipath split failed: {e}"
            ))
        })?;
        if parts.is_empty() {
            return Err(ToolkitError::DescriptorParse(
                "first-address: multipath split produced no branches".into(),
            ));
        }
        parts.remove(0)
    } else {
        descriptor.clone()
    };

    let definite: Descriptor<DefiniteDescriptorKey> = if single.has_wildcard() {
        single.derive_at_index(0).map_err(|e| {
            ToolkitError::DescriptorParse(format!(
                "first-address: derive_at_index(0) failed: {e}"
            ))
        })?
    } else {
        // Non-wildcard branch (e.g., user supplied a concrete `/0/0`
        // descriptor). Convert directly to definite-key form.
        Descriptor::<DefiniteDescriptorKey>::try_from(single).map_err(|e| {
            ToolkitError::DescriptorParse(format!(
                "first-address: definite-key conversion failed: {e}"
            ))
        })?
    };

    let address = definite.address(network).map_err(|e| {
        ToolkitError::DescriptorParse(format!("first-address: render failed: {e}"))
    })?;
    Ok(address.to_string())
}

// Unit-test coverage lives at the integration layer because the helper
// requires real (valid-checksum) test xpubs. See
// `tests/cli_export_wallet_bsms.rs::bsms_4line_first_address_byte_exact_against_descriptor_derivation`
// for the byte-exact cross-check against an independent miniscript derivation.
