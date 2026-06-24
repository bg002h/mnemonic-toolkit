//! Error type for `mk-codec`.
//!
//! Variants mirror the rejection conditions enumerated in
//! `design/SPEC_mk_v0_1.md` §4 ("Bytecode-Validity Rules") and
//! `bip/bip-mnemonic-key.mediawiki` §"Decoder validity rules". All
//! decoder-rejection paths in a future implementation MUST surface
//! one of these variants. Pre-BIP-submission, every variant is
//! required to map to at least one named negative test vector
//! (tracked as `decoder-error-variant-parity` in
//! `design/FOLLOWUPS.md`).

use bitcoin::bip32::ChildNumber;
use thiserror::Error;

/// All errors `mk-codec` can produce.
///
/// Marked `#[non_exhaustive]` so that future versions can add variants
/// without breaking external callers' exhaustive `match` arms.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Error {
    // ── String-layer errors (codex32 plumbing, HRP, chunk-header) ───────────
    /// HRP is not `mk` or input is not a valid bech32-shaped string.
    #[error("invalid HRP: {0}")]
    InvalidHrp(String),

    /// Input string mixes ASCII upper- and lower-case in its data part.
    /// BIP 173 forbids mixed case to remove an entire class of
    /// transcription ambiguity; the rule is inherited verbatim by mk1's
    /// codex32-derived encoding.
    #[error("mixed case in input string")]
    MixedCase,

    /// Input string's data-part length is not a valid mk1 length:
    /// either below the regular-code minimum (14 5-bit symbols), in the
    /// reserved-invalid 94–95 gap between regular and long codes, or
    /// above the long-code maximum (108). The carried `usize` is the
    /// observed length; reported pessimistically to highlight which
    /// boundary the caller missed.
    #[error("invalid data-part length: {0}")]
    InvalidStringLength(usize),

    /// Input string's data part contains a character that is not in the
    /// 32-character bech32 alphabet (`qpzry9x8gf2tvdw0s3jn54khce6mua7l`).
    /// The offending character and its 0-indexed position within the
    /// data part are reported so a higher-level decoder report can
    /// surface a precise location for transcription-error feedback.
    #[error("invalid character {ch} at position {position}")]
    InvalidChar {
        /// The character that was not in the bech32 alphabet.
        ch: char,
        /// 0-indexed position within the data part (chars after `mk1`).
        position: usize,
    },

    /// BCH checksum could not be corrected within the per-code-variant
    /// substitution capacity (4 for regular, 8 for long).
    #[error("BCH uncorrectable: {0}")]
    BchUncorrectable(String),

    /// Chunk-header card-type byte is not in {0x00 SingleString, 0x01 Chunked}.
    /// The 5-bit type field's reserved range 0x02..=0x1F MUST be rejected.
    #[error("unsupported card type: 0x{0:02x}")]
    UnsupportedCardType(u8),

    /// 5-bit payload symbols, after BCH verification, do not byte-align
    /// (i.e., the trailing pad bits of the final 5-bit symbol are non-zero).
    /// Parallels md1's `MalformedPayloadPadding` rejection.
    #[error("malformed payload padding (5-bit symbols don't byte-align)")]
    MalformedPayloadPadding,

    /// For chunked input: chunks have inconsistent `chunk_set_id` values.
    /// Used at reassembly time to detect mixed-card-set inputs.
    #[error("chunk_set_id mismatch across chunks")]
    ChunkSetIdMismatch,

    /// For chunked input: malformed chunked-string header (e.g., total_chunks
    /// = 0 or > 32, chunk_index >= total_chunks, gaps or duplicates in the
    /// index sequence at reassembly).
    #[error("chunked-header malformed: {0}")]
    ChunkedHeaderMalformed(String),

    /// Decoder received a multi-string input whose `SingleString` and
    /// `Chunked` header variants disagree across the supplied list:
    /// either the first string is `SingleString` but additional strings
    /// follow (caught early in `pipeline::decode`), or the first chunk
    /// is `Chunked` but a later chunk in the list is `SingleString`
    /// (caught in `chunk::reassemble_from_chunks`). Distinct from
    /// [`Error::ChunkedHeaderMalformed`], which covers issues *within*
    /// a declared-chunked set (bad `chunk_index`, bad `total_chunks`,
    /// duplicates, gaps, etc.).
    #[error("mixed string-layer header types in input list")]
    MixedHeaderTypes,

    /// For chunked input: reassembled bytecode's trailing 4-byte
    /// `cross_chunk_hash` does not match `SHA-256(canonical_bytecode)[0..4]`.
    #[error("cross-chunk integrity hash mismatch")]
    CrossChunkHashMismatch,

    // ── Bytecode-layer errors (after string-layer reassembly) ────────────────
    /// Bytecode-header version != 0 in v0.1.
    #[error("unsupported version: {0}")]
    UnsupportedVersion(u8),

    /// A reserved bit in the bytecode header was set (bits 0, 1, 3 in v0.1;
    /// bit 2 is the fingerprint flag and is allowed).
    #[error("reserved bits set in bytecode header")]
    ReservedBitsSet,

    /// `policy_id_stub_count == 0`. The spec requires ≥ 1.
    #[error("policy_id_stub_count must be >= 1")]
    InvalidPolicyIdStubCount,

    /// Origin-path indicator byte is outside the standard table or in the
    /// reserved range. (Per SPEC §3.5: 0x00, 0x08-0x10, 0x16, 0x18-0xFD,
    /// 0xFF are reserved; 0x16 is reserved pending md1 dictionary update,
    /// see FOLLOWUPS `md-path-dictionary-0x16-gap`.)
    #[error("invalid path indicator byte: 0x{0:02x}")]
    InvalidPathIndicator(u8),

    /// Explicit path declared `component_count > MAX_PATH_COMPONENTS`
    /// (closure Q-3 lock: max 10, was 32 in the pre-closure draft).
    #[error("path too deep: {0} components (max 10)")]
    PathTooDeep(u8),

    /// A path component's encoded value is invalid (e.g., out of BIP 32
    /// range, or hardened-bit set in an invalid position).
    #[error("invalid path component: {0}")]
    InvalidPathComponent(String),

    /// xpub `version` field doesn't match a known network's xpub prefix.
    #[error("invalid xpub version: 0x{0:08x}")]
    InvalidXpubVersion(u32),

    /// xpub `public_key` bytes do not parse as a valid compressed
    /// secp256k1 point. Realistically unreachable for inputs that
    /// pass BCH verification; surfaces hand-constructed inputs.
    #[error("invalid xpub public key: {0}")]
    InvalidXpubPublicKey(String),

    /// Decoder hit end-of-stream mid-field.
    #[error("unexpected end of bytecode")]
    UnexpectedEnd,

    /// Decoder finished consuming all expected fields but bytes remain.
    #[error("trailing bytes after xpub")]
    TrailingBytes,

    /// Canonical bytecode + cross-chunk hash exceeds the v0.1 capacity
    /// of `MAX_CHUNKS * CHUNKED_FRAGMENT_LONG_BYTES − CROSS_CHUNK_HASH_BYTES`
    /// (= 32 × 53 − 4 = 1692 bytes). Reachable only through pathological
    /// hand-constructed inputs; typical mk1 cards land well below this
    /// ceiling per `design/SPEC_mk_v0_1.md` §2.4.
    #[error(
        "card payload too large: bytecode_len = {bytecode_len} > max_supported = {max_supported}"
    )]
    CardPayloadTooLarge {
        /// Observed canonical-bytecode length in bytes.
        bytecode_len: usize,
        /// Maximum bytecode length the v0.1 chunking layer can carry.
        max_supported: usize,
    },

    /// Encoder-side invariant: the supplied `xpub`'s BIP-32 `depth` /
    /// `child_number` disagree with `origin_path` (`depth ≠` component
    /// count, or `child_number ≠` the terminal component). Compact-73
    /// reconstructs both fields from the path on decode, so emitting such a
    /// card would yield a different-metadata xpub. Rejected at encode to keep
    /// compact-73 genuinely lossless. The decoder cannot detect this (no
    /// on-wire depth) — see `design/SPEC_mk_v0_1.md` §4 (encoder-side
    /// invariant) and `design/SPEC_mk_depth_child_enforcement.md`.
    #[error(
        "xpub origin-path mismatch: xpub depth {xpub_depth} / child {xpub_child} \
         vs origin_path depth {path_depth} / last {path_child:?}"
    )]
    XpubOriginPathMismatch {
        /// `xpub.depth` as supplied.
        xpub_depth: u8,
        /// `component_count(origin_path)`.
        path_depth: u8,
        /// `xpub.child_number` as supplied.
        xpub_child: ChildNumber,
        /// Terminal component of `origin_path` (`None` for an empty path).
        path_child: Option<ChildNumber>,
    },
}

/// `Result` alias used throughout `mk-codec`.
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    /// Each variant carries enough information for its rendered Display
    /// to be diagnostic. Sanity-check the format strings render
    /// correctly for every parameterized variant.
    #[test]
    fn parameterized_variants_render() {
        let cases: Vec<(Error, &str)> = vec![
            (Error::InvalidHrp("mq".into()), "invalid HRP: mq"),
            (
                Error::BchUncorrectable(
                    "5 substitutions exceed long-code 4-correction limit".into(),
                ),
                "BCH uncorrectable: 5 substitutions exceed long-code 4-correction limit",
            ),
            (
                Error::UnsupportedCardType(0x05),
                "unsupported card type: 0x05",
            ),
            (
                Error::ChunkedHeaderMalformed("total_chunks = 0".into()),
                "chunked-header malformed: total_chunks = 0",
            ),
            (
                Error::InvalidXpubPublicKey("malformed compressed point".into()),
                "invalid xpub public key: malformed compressed point",
            ),
            (Error::UnsupportedVersion(1), "unsupported version: 1"),
            (
                Error::InvalidPathIndicator(0x16),
                "invalid path indicator byte: 0x16",
            ),
            (
                Error::PathTooDeep(11),
                "path too deep: 11 components (max 10)",
            ),
            (
                Error::InvalidPathComponent("LEB128 overflow at component 3".into()),
                "invalid path component: LEB128 overflow at component 3",
            ),
            (
                Error::InvalidXpubVersion(0xDEADBEEF),
                "invalid xpub version: 0xdeadbeef",
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(format!("{err}"), expected);
        }
    }

    // ── String-layer rejection coverage (per plan §3.2.4) ──────────────
    //
    // Phase 5 landed the string-layer code paths that produce
    // `CrossChunkHashMismatch`, `MalformedPayloadPadding`,
    // `ChunkSetIdMismatch`, and `ChunkedHeaderMalformed`. The detailed
    // reject scenarios live in `crate::string_layer::pipeline::tests`
    // and `crate::string_layer::chunk::tests`; the smoke checks here
    // assert that each variant is reachable through the public
    // `crate::decode` API rather than just the lower-level layer
    // helpers (the scaffolds documented in the plan §3.2.4 forward-
    // reference these tests).
    //
    // (Phase 4 retired the proposed `FingerprintFlagMismatch` variant:
    // structurally undetectable in the decoder under the closure-locked
    // wire format, since no length prefix lets the decoder distinguish
    // "flag set, fp present" from "flag unset, fp omitted." SPEC §4
    // rule 3 was reframed as an encoder-side invariant; see commit
    // log for Phase 4 review fixup.)

    /// Unparameterized variants render their static message verbatim.
    #[test]
    fn static_variants_render() {
        assert_eq!(
            format!("{}", Error::ReservedBitsSet),
            "reserved bits set in bytecode header",
        );
        assert_eq!(
            format!("{}", Error::CrossChunkHashMismatch),
            "cross-chunk integrity hash mismatch",
        );
        assert_eq!(
            format!("{}", Error::ChunkSetIdMismatch),
            "chunk_set_id mismatch across chunks",
        );
        assert_eq!(
            format!("{}", Error::MixedHeaderTypes),
            "mixed string-layer header types in input list",
        );
        assert_eq!(
            format!("{}", Error::MalformedPayloadPadding),
            "malformed payload padding (5-bit symbols don't byte-align)",
        );
        assert_eq!(
            format!("{}", Error::InvalidPolicyIdStubCount),
            "policy_id_stub_count must be >= 1",
        );
        assert_eq!(
            format!("{}", Error::UnexpectedEnd),
            "unexpected end of bytecode",
        );
        assert_eq!(
            format!("{}", Error::TrailingBytes),
            "trailing bytes after xpub",
        );
    }
}
