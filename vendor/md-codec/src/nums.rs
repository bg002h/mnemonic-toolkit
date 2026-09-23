//! The BIP-341 NUMS H-point constant — a feature-independent constant shared by
//! the `derive`-gated `to_miniscript` converter and the (ungated) `@N`-template
//! `render`er, so neither owns it and the renderer need not inherit `derive`.
//!
//! Also carries Liana's unspendable-xpub recipe (SPEC §2,
//! `design/SPEC_liana_unspendable_internal_key.md`) — wire kind 1's taproot
//! internal key. It is pure `bitcoin`-crate arithmetic over caller-supplied
//! leaf pubkeys, so it belongs here rather than in the `derive`-gated
//! `to_miniscript`: no `miniscript` dependency, and public regardless of
//! whether the `derive` feature is enabled.

use bitcoin::Network;
use bitcoin::bip32::{ChainCode, ChildNumber, Fingerprint, Xpub};
use bitcoin::hashes::{Hash, sha256};
use bitcoin::secp256k1::PublicKey;
use std::str::FromStr;

/// BIP-341 NUMS H-point x-only coordinate. Used as the internal key when
/// `Body::Tr { is_nums: true, .. }`.
///
/// Single source of truth for both [`crate::render`] (emits the literal x-only
/// hex for a NUMS-flagged taproot internal key) and `to_miniscript` (builds the
/// NUMS `DescriptorPublicKey`). Value is byte-identical to md-cli's historical
/// `parse::template::NUMS_H_POINT_X_ONLY_HEX`; sharing changes nothing.
pub(crate) const NUMS_H_POINT_X_ONLY_HEX: &str =
    "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// The `@N`-template substitution text for a wire-kind-1 (Liana unspendable)
/// taproot internal key — what a rendered `SkeletonKey` shows in place of an
/// xpub, so an operator reading a plate sees "UNSPENDABLE(liana)" rather than
/// a meaningless key (SPEC §4).
pub const LIANA_UNSPENDABLE_MARKER: &str = "UNSPENDABLE(liana)";

/// Liana's unspendable internal-key xpub (SPEC §2), reproduced from
/// `liana/src/descriptors/analysis.rs:398-430`.
///
/// `leaf_pubkeys` is the ordered sequence of the taproot tree's leaf keys'
/// 33-byte compressed public keys, **in descriptor left-to-right (wire)
/// order — not sorted, not deduplicated**, walked per key OCCURRENCE (a
/// caller building this from a tap tree must walk leaves, not index a
/// deduplicated key-slot list; §2's own discussion of `node_to_descriptor`
/// explains why those coincide for every md1-expressible descriptor but are
/// not the same operation).
///
/// Recipe: `chain_code = sha256(leaf_pubkeys[0] || leaf_pubkeys[1] || ...)`;
/// `public_key` is always the BIP-341 NUMS H-point, compressed; `depth = 0`,
/// `parent_fingerprint = 00000000`, `child_number = 0`. `network` selects the
/// rendered version bytes only (`xpub` for [`Network::Bitcoin`], `tpub`
/// otherwise) — it plays no role in the hash.
pub fn liana_unspendable_xpub(leaf_pubkeys: &[[u8; 33]], network: Network) -> Xpub {
    let mut concat = Vec::with_capacity(leaf_pubkeys.len() * 33);
    for pk in leaf_pubkeys {
        concat.extend_from_slice(pk);
    }
    let chain_code = ChainCode::from(sha256::Hash::hash(&concat).to_byte_array());

    let nums_compressed_hex = format!("02{NUMS_H_POINT_X_ONLY_HEX}");
    let public_key = PublicKey::from_str(&nums_compressed_hex)
        .expect("BIP-341 NUMS H-point is a fixed, valid compressed secp256k1 point");

    Xpub {
        network: network.into(),
        depth: 0,
        parent_fingerprint: Fingerprint::default(),
        child_number: ChildNumber::Normal { index: 0 },
        public_key,
        chain_code,
    }
}
