//! `KeyCard` — the in-memory representation of a decoded MK card.
//!
//! Field semantics mirror the wire-format payload from
//! `design/SPEC_mk_v0_1.md` §3.2. The bytecode-layer encode/decode
//! lives in [`crate::bytecode`] (Phase 4); the string-layer wrapper
//! (BCH + chunking) wires up the public `encode`/`decode` functions
//! below in Phase 5.

use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};

use crate::error::Result;

/// In-memory representation of one decoded MK card.
///
/// Per closure Q-8, `origin_fingerprint` is `Option<Fingerprint>`:
/// a card encoded with the bytecode-header fingerprint flag unset
/// (privacy-preserving mode) reconstructs to a `KeyCard` with
/// `origin_fingerprint = None`.
///
/// `#[non_exhaustive]` so future versions can add fields without
/// breaking external constructors.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyCard {
    /// Policy ID stubs declaring which MD-encoded policy template(s)
    /// this xpub is intended to serve. Each stub is the top 4 bytes
    /// of the policy's `SHA-256(canonical_bytecode)`. The vector is
    /// guaranteed non-empty after a successful `decode` (the decoder
    /// rejects `count == 0` with `Error::InvalidPolicyIdStubCount`).
    pub policy_id_stubs: Vec<[u8; 4]>,

    /// Master-key fingerprint identifying the seed from which `xpub`
    /// was derived. Verbatim from BIP 380 origin notation `[fp/...]`.
    /// Optional per closure Q-8: encoders MAY omit (set bytecode-header
    /// bit 2 = 0) for the privacy-preserving mode.
    pub origin_fingerprint: Option<Fingerprint>,

    /// Derivation path from master to `xpub`. Encoded on the wire
    /// either via a 1-byte standard-path indicator (BIP 44/49/84/86/
    /// 48-segwit/48-nested/87 + testnet variants) or via the explicit
    /// `0xFE` escape hatch with LEB128 components.
    pub origin_path: DerivationPath,

    /// The BIP 32 extended public key. The wire format carries a
    /// 73-byte compact form (per closure Q-7); the in-memory `Xpub`
    /// is reconstructed at decode time using the locked rule:
    ///
    /// ```text
    /// depth        := component_count(origin_path)
    /// child_number := last_component(origin_path),
    ///                 or Normal{0} when origin_path is empty (depth-0 / no-path key)
    /// ```
    pub xpub: Xpub,
}

impl KeyCard {
    /// Construct a `KeyCard` from its four owned fields.
    ///
    /// `KeyCard` is `#[non_exhaustive]` so that future versions can
    /// add fields without breaking external callers; the constructor
    /// stays stable across additions because new fields land with
    /// `Default`-compatible values or new constructors.
    ///
    /// # Field invariants enforced at encode time
    ///
    /// `KeyCard::new` is intentionally permissive — field-level
    /// validation lives in [`crate::encode`] / [`crate::bytecode::encode_bytecode`].
    /// In particular:
    ///
    /// - `policy_id_stubs` MUST be non-empty; the encoder rejects an
    ///   empty vector with [`crate::Error::InvalidPolicyIdStubCount`]
    ///   (per `design/SPEC_mk_v0_1.md` §4 rule 3).
    /// - `origin_path` MUST have at most [`crate::MAX_PATH_COMPONENTS`]
    ///   = 10 components when an explicit-path encoding would be used;
    ///   exceeding that yields [`crate::Error::PathTooDeep`].
    ///
    /// Callers that want a fail-fast constructor should validate
    /// these invariants before calling `new`, or simply rely on the
    /// encoder's rejection.
    pub fn new(
        policy_id_stubs: Vec<[u8; 4]>,
        origin_fingerprint: Option<Fingerprint>,
        origin_path: DerivationPath,
        xpub: Xpub,
    ) -> Self {
        Self {
            policy_id_stubs,
            origin_fingerprint,
            origin_path,
            xpub,
        }
    }
}

/// Encode a `KeyCard` into one or more `mk1`-prefixed strings.
///
/// Multi-chunk encodings draw a fresh 20-bit `chunk_set_id` from the
/// system CSPRNG. Use [`encode_with_chunk_set_id`] for byte-deterministic
/// output (vector regeneration, conformance tests).
pub fn encode(card: &KeyCard) -> Result<Vec<String>> {
    crate::string_layer::encode(card)
}

/// Like [`encode`], with an explicit `chunk_set_id` override.
///
/// `chunk_set_id` MUST fit in 20 bits (`0..=0x000F_FFFF`); otherwise
/// returns [`crate::Error::ChunkedHeaderMalformed`]. The override is
/// only consulted on the chunked path; single-string encodings have no
/// `chunk_set_id` field.
pub fn encode_with_chunk_set_id(card: &KeyCard, chunk_set_id: u32) -> Result<Vec<String>> {
    crate::string_layer::encode_with_chunk_set_id(card, chunk_set_id)
}

/// Decode one or more `mk1`-prefixed strings into a `KeyCard`.
pub fn decode(strings: &[&str]) -> Result<KeyCard> {
    crate::string_layer::decode(strings)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sanity check: type signatures compile and the public API
    /// surface matches what the lib.rs re-exports expect. Real
    /// round-trip coverage at this layer lands in Phase 6.
    #[test]
    fn types_compile() {
        let _f: fn(&KeyCard) -> Result<Vec<String>> = encode;
        let _g: fn(&[&str]) -> Result<KeyCard> = decode;
    }
}
