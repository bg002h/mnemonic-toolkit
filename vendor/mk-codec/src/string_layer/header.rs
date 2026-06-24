//! 5-bit-symbol-aligned string-layer header (single-string + chunked variants).
//!
//! Per `design/SPEC_mk_v0_1.md` §2.5 and closure Q-5, mk1's string-layer
//! header lives at the bech32 5-bit symbol layer rather than the byte
//! layer. Encoders emit either a 2-symbol [`StringLayerHeader::SingleString`]
//! header (`version + type=0x00`) or an 8-symbol [`StringLayerHeader::Chunked`]
//! header (`version + type=0x01 + chunk_set_id + total_chunks + chunk_index`).
//!
//! All field widths are exactly 5 bits unless otherwise noted. The
//! `chunk_set_id` is the only multi-symbol field — 20 bits = 4 symbols.

use crate::consts::MAX_CHUNKS;
use crate::error::{Error, Result};

/// Type-byte values for the 5-bit `type` field (closure Q-5).
const TYPE_SINGLE: u8 = 0x00;
const TYPE_CHUNKED: u8 = 0x01;

/// Number of 5-bit symbols in the single-string header (`version + type`).
pub const SINGLE_HEADER_SYMBOLS: usize = 2;

/// Number of 5-bit symbols in the chunked header
/// (`version + type + 4·chunk_set_id + total_chunks + chunk_index`).
pub const CHUNKED_HEADER_SYMBOLS: usize = 8;

/// Maximum allowed value of `chunk_set_id` (20-bit field).
pub const MAX_CHUNK_SET_ID: u32 = (1 << 20) - 1;

/// Format-version field value emitted in v0.1.
pub const VERSION_V0_1: u8 = 0x00;

/// String-layer header for one mk1 chunk.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringLayerHeader {
    /// Card fits in one mk1 string; no chunking. Carries no chunk-set
    /// identifier or index because the format is unambiguous.
    SingleString {
        /// 5-bit format version (`0` in v0.1).
        version: u8,
    },
    /// One chunk in a multi-chunk encoding. All chunks of one card share
    /// the same `version`, `chunk_set_id`, and `total_chunks`; only
    /// `chunk_index` varies.
    Chunked {
        /// 5-bit format version (`0` in v0.1).
        version: u8,
        /// 20-bit per-encoding random tag for reassembly mismatch
        /// detection. Decoders compare across chunks; mismatch is
        /// rejected with [`Error::ChunkSetIdMismatch`].
        chunk_set_id: u32,
        /// Total number of chunks in this set, in `1..=MAX_CHUNKS`.
        total_chunks: u8,
        /// Zero-based index of this chunk within the set.
        chunk_index: u8,
    },
}

impl StringLayerHeader {
    /// Emit this header as a sequence of 5-bit symbols.
    ///
    /// The output length is [`SINGLE_HEADER_SYMBOLS`] (= 2) for
    /// [`StringLayerHeader::SingleString`] and [`CHUNKED_HEADER_SYMBOLS`]
    /// (= 8) for [`StringLayerHeader::Chunked`]. The caller prepends
    /// these symbols to `bytes_to_5bit(fragment)` to form a chunk's
    /// data part before BCH checksumming.
    pub fn to_5bit_symbols(self) -> Vec<u8> {
        match self {
            StringLayerHeader::SingleString { version } => {
                vec![version & 0x1F, TYPE_SINGLE]
            }
            StringLayerHeader::Chunked {
                version,
                chunk_set_id,
                total_chunks,
                chunk_index,
            } => {
                // chunk_set_id is 20 bits; pack as four 5-bit symbols
                // big-endian (bits 19..15, 14..10, 9..5, 4..0).
                let csid = chunk_set_id & MAX_CHUNK_SET_ID;
                // total_chunks is the user-facing 1..=32 count. The 5-bit
                // wire field can only hold 0..=31, so we encode `count - 1`
                // here and decode back via `wire + 1` in `from_5bit_symbols`.
                // (`design/SPEC_mk_v0_1.md` §2.5 documents the range as
                // `1..=32`; the off-by-one wire encoding is the only way
                // to honour both the 5-bit field width and the closure-
                // locked 32-chunk capacity.)
                let total_chunks_wire = (total_chunks - 1) & 0x1F;
                vec![
                    version & 0x1F,
                    TYPE_CHUNKED,
                    ((csid >> 15) & 0x1F) as u8,
                    ((csid >> 10) & 0x1F) as u8,
                    ((csid >> 5) & 0x1F) as u8,
                    (csid & 0x1F) as u8,
                    total_chunks_wire,
                    chunk_index & 0x1F,
                ]
            }
        }
    }

    /// Parse a header off the front of a 5-bit-symbol stream.
    ///
    /// Returns the parsed header and the number of symbols consumed
    /// (2 for `SingleString`, 8 for `Chunked`). The caller slices off
    /// the remainder as the fragment-payload symbols.
    ///
    /// # Errors
    ///
    /// - [`Error::UnexpectedEnd`] if `symbols` is shorter than the
    ///   minimum 2-symbol single-string header.
    /// - [`Error::UnsupportedVersion`] if the version field is non-zero
    ///   in v0.1.
    /// - [`Error::UnsupportedCardType`] if the type field is not in
    ///   `{0x00, 0x01}` (the reserved range `0x02..=0x1F` is rejected).
    /// - [`Error::ChunkedHeaderMalformed`] if a chunked header has
    ///   `total_chunks == 0`, `total_chunks > MAX_CHUNKS`, or
    ///   `chunk_index >= total_chunks`.
    pub fn from_5bit_symbols(symbols: &[u8]) -> Result<(Self, usize)> {
        if symbols.len() < SINGLE_HEADER_SYMBOLS {
            return Err(Error::UnexpectedEnd);
        }
        let version = symbols[0] & 0x1F;
        if version != VERSION_V0_1 {
            return Err(Error::UnsupportedVersion(version));
        }
        let type_byte = symbols[1] & 0x1F;
        match type_byte {
            TYPE_SINGLE => Ok((
                StringLayerHeader::SingleString { version },
                SINGLE_HEADER_SYMBOLS,
            )),
            TYPE_CHUNKED => {
                if symbols.len() < CHUNKED_HEADER_SYMBOLS {
                    return Err(Error::UnexpectedEnd);
                }
                let csid: u32 = ((symbols[2] as u32 & 0x1F) << 15)
                    | ((symbols[3] as u32 & 0x1F) << 10)
                    | ((symbols[4] as u32 & 0x1F) << 5)
                    | (symbols[5] as u32 & 0x1F);
                // total_chunks is encoded as `count - 1` on the wire (5-bit
                // field; closure-locked semantic range 1..=32). Decode back
                // by adding 1; validity range is automatically 1..=32 since
                // the 5-bit field caps at 31.
                let total_chunks = (symbols[6] & 0x1F) + 1;
                let chunk_index = symbols[7] & 0x1F;

                // Defensive: even though the off-by-one decoding makes
                // total_chunks always land in 1..=32, surface a malformed
                // error if that invariant is ever broken (e.g., by a
                // future encoder bug emitting a wire value > 31, which
                // would have been masked by the & 0x1F above).
                if total_chunks == 0 || total_chunks > MAX_CHUNKS {
                    return Err(Error::ChunkedHeaderMalformed(format!(
                        "total_chunks = {total_chunks} (must be in 1..={MAX_CHUNKS})"
                    )));
                }
                if chunk_index >= total_chunks {
                    return Err(Error::ChunkedHeaderMalformed(format!(
                        "chunk_index = {chunk_index} >= total_chunks = {total_chunks}"
                    )));
                }
                Ok((
                    StringLayerHeader::Chunked {
                        version,
                        chunk_set_id: csid,
                        total_chunks,
                        chunk_index,
                    },
                    CHUNKED_HEADER_SYMBOLS,
                ))
            }
            other => Err(Error::UnsupportedCardType(other)),
        }
    }

    /// Returns `true` if this header is the `Chunked` variant.
    pub fn is_chunked(self) -> bool {
        matches!(self, StringLayerHeader::Chunked { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_string_round_trip() {
        let h = StringLayerHeader::SingleString { version: 0 };
        let symbols = h.to_5bit_symbols();
        assert_eq!(symbols.len(), SINGLE_HEADER_SYMBOLS);
        let (parsed, consumed) = StringLayerHeader::from_5bit_symbols(&symbols).unwrap();
        assert_eq!(parsed, h);
        assert_eq!(consumed, SINGLE_HEADER_SYMBOLS);
    }

    #[test]
    fn chunked_round_trip() {
        let h = StringLayerHeader::Chunked {
            version: 0,
            chunk_set_id: 0xABCDE,
            total_chunks: 5,
            chunk_index: 3,
        };
        let symbols = h.to_5bit_symbols();
        assert_eq!(symbols.len(), CHUNKED_HEADER_SYMBOLS);
        let (parsed, consumed) = StringLayerHeader::from_5bit_symbols(&symbols).unwrap();
        assert_eq!(parsed, h);
        assert_eq!(consumed, CHUNKED_HEADER_SYMBOLS);
    }

    #[test]
    fn chunked_round_trip_max_csid() {
        // Top-of-range chunk_set_id (all 20 bits set) packs and unpacks correctly.
        let h = StringLayerHeader::Chunked {
            version: 0,
            chunk_set_id: MAX_CHUNK_SET_ID,
            total_chunks: MAX_CHUNKS,
            chunk_index: MAX_CHUNKS - 1,
        };
        let (parsed, _) = StringLayerHeader::from_5bit_symbols(&h.to_5bit_symbols()).unwrap();
        assert_eq!(parsed, h);
    }

    #[test]
    fn chunked_round_trip_zero_csid() {
        // Bottom-of-range chunk_set_id (zero) packs and unpacks correctly.
        let h = StringLayerHeader::Chunked {
            version: 0,
            chunk_set_id: 0,
            total_chunks: 1,
            chunk_index: 0,
        };
        let (parsed, _) = StringLayerHeader::from_5bit_symbols(&h.to_5bit_symbols()).unwrap();
        assert_eq!(parsed, h);
    }

    #[test]
    fn parse_rejects_truncated_input() {
        // Empty and 1-symbol inputs cannot encode a full single-string header.
        assert!(matches!(
            StringLayerHeader::from_5bit_symbols(&[]),
            Err(Error::UnexpectedEnd)
        ));
        assert!(matches!(
            StringLayerHeader::from_5bit_symbols(&[0]),
            Err(Error::UnexpectedEnd)
        ));
        // Truncated chunked header (type=0x01 declared, but fewer than 8 symbols).
        let symbols = vec![0u8, TYPE_CHUNKED, 0, 0, 0];
        assert!(matches!(
            StringLayerHeader::from_5bit_symbols(&symbols),
            Err(Error::UnexpectedEnd)
        ));
    }

    #[test]
    fn parse_rejects_unsupported_version() {
        let symbols = vec![1u8, TYPE_SINGLE];
        assert!(matches!(
            StringLayerHeader::from_5bit_symbols(&symbols),
            Err(Error::UnsupportedVersion(1))
        ));
    }

    #[test]
    fn parse_rejects_reserved_card_type() {
        // Reserved type byte 0x02..=0x1F MUST be rejected.
        for ct in 0x02u8..=0x1F {
            let symbols = vec![0u8, ct];
            let r = StringLayerHeader::from_5bit_symbols(&symbols);
            assert!(
                matches!(r, Err(Error::UnsupportedCardType(c)) if c == ct),
                "card type 0x{ct:02x} not rejected"
            );
        }
    }

    #[test]
    fn wire_total_chunks_zero_decodes_to_one() {
        // The 5-bit `total_chunks` field is encoded as `count - 1` per the
        // off-by-one note in `to_5bit_symbols`, so wire value 0 represents
        // a single-chunk encoding. (`SingleString` is wire-defined for
        // forward compatibility per SPEC §2.4 but unreachable for v0.1
        // encoders — a `Chunked(total=1)` is a defined-but-rare shape
        // produced only by hand-constructed test inputs at the header layer.)
        let h = StringLayerHeader::Chunked {
            version: 0,
            chunk_set_id: 0,
            total_chunks: 1,
            chunk_index: 0,
        };
        let symbols = h.to_5bit_symbols();
        assert_eq!(symbols[6], 0, "wire encoding of total_chunks=1 must be 0");
        let (parsed, _) = StringLayerHeader::from_5bit_symbols(&symbols).unwrap();
        assert_eq!(parsed, h);
    }

    #[test]
    fn parse_rejects_chunk_index_at_or_above_total_chunks() {
        let h = StringLayerHeader::Chunked {
            version: 0,
            chunk_set_id: 0,
            total_chunks: 3,
            chunk_index: 0,
        };
        let mut symbols = h.to_5bit_symbols();
        symbols[7] = 3; // chunk_index >= total_chunks (=3)
        assert!(matches!(
            StringLayerHeader::from_5bit_symbols(&symbols),
            Err(Error::ChunkedHeaderMalformed(_))
        ));
    }

    #[test]
    fn is_chunked_discriminator() {
        assert!(!StringLayerHeader::SingleString { version: 0 }.is_chunked());
        assert!(
            StringLayerHeader::Chunked {
                version: 0,
                chunk_set_id: 0,
                total_chunks: 1,
                chunk_index: 0,
            }
            .is_chunked()
        );
    }
}
