//! The BIP-341 NUMS H-point constant — a feature-independent constant shared by
//! the `derive`-gated `to_miniscript` converter and the (ungated) `@N`-template
//! `render`er, so neither owns it and the renderer need not inherit `derive`.

/// BIP-341 NUMS H-point x-only coordinate. Used as the internal key when
/// `Body::Tr { is_nums: true, .. }`.
///
/// Single source of truth for both [`crate::render`] (emits the literal x-only
/// hex for a NUMS-flagged taproot internal key) and `to_miniscript` (builds the
/// NUMS `DescriptorPublicKey`). Value is byte-identical to md-cli's historical
/// `parse::template::NUMS_H_POINT_X_ONLY_HEX`; sharing changes nothing.
pub(crate) const NUMS_H_POINT_X_ONLY_HEX: &str =
    "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";
