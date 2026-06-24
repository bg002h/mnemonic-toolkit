//! Locked constants for `mk1` per `design/SPEC_mk_v0_1.md` v0.1.
//!
//! All values are closure-locked (see
//! `docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md`).
//! Reproducer for the NUMS-derived target constants is documented in
//! the BIP draft's "Why new target constants?" section.

/// HRP for `mk1` strings (BIP 173 separator `1` follows: prefix is `mk1`).
pub const HRP: &str = "mk";

/// Domain string for NUMS-derived target constants (closure Q-1).
///
/// The string itself is the audit trail: any reader can recompute the
/// SHA-256 and verify the constants follow from it.
pub const NUMS_DOMAIN: &[u8] = b"shibbolethnumskey";

/// Top 65 bits of `SHA-256(NUMS_DOMAIN)`. Regular-code target residue.
pub const MK_REGULAR_CONST: u128 = 0x1062435f91072fa5c;

/// Top 75 bits of `SHA-256(NUMS_DOMAIN)`. Long-code target residue.
pub const MK_LONG_CONST: u128 = 0x41890d7e441cbe97273;

/// Maximum components in an explicit-path encoding (closure Q-3).
///
/// Real BIP-style derivations top out at 6 (BIP 48 multisig is 4); 10
/// gives margin without locking out plausibly real paths.
pub const MAX_PATH_COMPONENTS: u8 = 10;

/// Single-string regular-code payload bytes.
pub const SINGLE_STRING_REGULAR_BYTES: usize = 48;

/// Single-string long-code payload bytes.
pub const SINGLE_STRING_LONG_BYTES: usize = 56;

/// Chunked-fragment regular-code payload bytes per chunk.
pub const CHUNKED_FRAGMENT_REGULAR_BYTES: usize = 45;

/// Chunked-fragment long-code payload bytes per chunk.
pub const CHUNKED_FRAGMENT_LONG_BYTES: usize = 53;

/// Maximum chunks per card.
pub const MAX_CHUNKS: u8 = 32;

/// Cross-chunk integrity hash size in bytes.
pub const CROSS_CHUNK_HASH_BYTES: usize = 4;

/// Family-stable generator string (closure Q-10) for vector-corpus
/// SHA-256 anchoring. Patch-version bumps don't roll the token; minor-
/// or major-version bumps do.
pub const GENERATOR_FAMILY: &str = "mk-codec 0.2";

/// Compact-73 xpub byte size (closure Q-7).
pub const XPUB_COMPACT_BYTES: usize = 73;

/// Policy ID stub size in bytes (closure Q-2).
pub const POLICY_ID_STUB_BYTES: usize = 4;

/// Origin fingerprint size in bytes.
pub const ORIGIN_FINGERPRINT_BYTES: usize = 4;

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::hashes::{Hash, sha256};

    /// Verifies that the locked hex constants reproduce from the
    /// documented derivation rule. Catches accidental drift if either
    /// the domain string or the constants are edited without updating
    /// the other.
    #[test]
    fn nums_constants_reproduce_from_domain() {
        let digest = sha256::Hash::hash(NUMS_DOMAIN);
        let bytes = digest.as_byte_array();
        // Stage the leading 128 bits of the 256-bit digest as a
        // big-endian u128.
        let hi: u128 = u128::from_be_bytes(bytes[0..16].try_into().unwrap());

        // Top 65 bits: shift the leading 128 bits right by (128 - 65).
        let derived_regular = hi >> 63;
        assert_eq!(
            derived_regular, MK_REGULAR_CONST,
            "MK_REGULAR_CONST drift from SHA-256(NUMS_DOMAIN) top-65-bits",
        );

        // Top 75 bits: shift right by (128 - 75).
        let derived_long = hi >> 53;
        assert_eq!(
            derived_long, MK_LONG_CONST,
            "MK_LONG_CONST drift from SHA-256(NUMS_DOMAIN) top-75-bits",
        );
    }

    #[test]
    fn nums_string_differs_from_md1() {
        assert_ne!(
            NUMS_DOMAIN, b"shibbolethnums",
            "mk1 NUMS string MUST differ from md1's per closure D-10",
        );
    }

    #[test]
    fn capacity_constants_match_spec() {
        // Sanity: confirms the four capacity numbers carry the values
        // pinned in SPEC §2.4 / BIP §"Length envelope".
        assert_eq!(SINGLE_STRING_REGULAR_BYTES, 48);
        assert_eq!(SINGLE_STRING_LONG_BYTES, 56);
        assert_eq!(CHUNKED_FRAGMENT_REGULAR_BYTES, 45);
        assert_eq!(CHUNKED_FRAGMENT_LONG_BYTES, 53);
        assert_eq!(MAX_CHUNKS, 32);
    }

    #[test]
    fn xpub_compact_size_is_73() {
        // 4 (version) + 4 (parent_fingerprint) + 32 (chain_code) + 33 (public_key) = 73.
        assert_eq!(XPUB_COMPACT_BYTES, 4 + 4 + 32 + 33);
    }

    #[test]
    fn path_cap_is_ten() {
        // closure Q-3 lock; not 32.
        assert_eq!(MAX_PATH_COMPONENTS, 10);
    }
}
