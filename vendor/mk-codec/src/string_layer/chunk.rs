//! Stream chunking + cross-chunk integrity hash for mk1 multi-string cards.
//!
//! Per `design/SPEC_mk_v0_1.md` §2.6, the canonical bytecode is suffixed
//! with a 4-byte `cross_chunk_hash` (= `SHA-256(canonical_bytecode)[0..4]`)
//! before splitting into chunk fragments; the hash is verified at
//! reassembly. This catches dropped, reordered, or substituted chunks
//! that the per-chunk BCH layer alone cannot detect.

use bitcoin::hashes::{Hash, sha256};

use crate::consts::{CHUNKED_FRAGMENT_LONG_BYTES, CROSS_CHUNK_HASH_BYTES, MAX_CHUNKS};
use crate::error::{Error, Result};
use crate::string_layer::header::{MAX_CHUNK_SET_ID, StringLayerHeader, VERSION_V0_1};

/// Maximum canonical-bytecode length that can be chunked under v0.1.
///
/// Equals `MAX_CHUNKS * CHUNKED_FRAGMENT_LONG_BYTES − CROSS_CHUNK_HASH_BYTES`
/// (= 32 * 53 − 4 = 1692). Bytecodes longer than this cannot be encoded
/// as a single mk1 card and the encoder returns
/// [`Error::CardPayloadTooLarge`].
pub const MAX_CHUNKABLE_BYTECODE: usize =
    (MAX_CHUNKS as usize) * CHUNKED_FRAGMENT_LONG_BYTES - CROSS_CHUNK_HASH_BYTES;

/// One chunk's worth of split output: a parsed header + its fragment bytes.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkFragment {
    /// The string-layer header that prefixes this chunk on the wire.
    pub header: StringLayerHeader,
    /// The raw fragment payload bytes for this chunk.
    pub fragment: Vec<u8>,
}

/// Split canonical bytecode into chunks, appending the cross-chunk integrity hash.
///
/// The split target is `CHUNKED_FRAGMENT_LONG_BYTES` (= 53 bytes) per
/// fragment so each chunk lands in long-code BCH territory under typical
/// mk1 sizes; the last fragment may be shorter, in which case the
/// pipeline auto-falls-back to regular code per
/// [`encode_5bit_to_string`][crate::string_layer::bch::encode_5bit_to_string].
///
/// Returns [`Error::CardPayloadTooLarge`] if the bytecode (plus the 4-byte
/// hash) exceeds `MAX_CHUNKS * CHUNKED_FRAGMENT_LONG_BYTES`.
///
/// # Determinism
///
/// The output is byte-deterministic in `(canonical_bytecode, chunk_set_id)`:
/// callers passing the same arguments produce the same chunk sequence,
/// which is the property Phase 6 relies on for vector regeneration.
pub fn split_into_chunks(
    canonical_bytecode: &[u8],
    chunk_set_id: u32,
) -> Result<Vec<ChunkFragment>> {
    if chunk_set_id > MAX_CHUNK_SET_ID {
        return Err(Error::ChunkedHeaderMalformed(format!(
            "chunk_set_id {chunk_set_id:#x} exceeds 20-bit field"
        )));
    }
    if canonical_bytecode.len() > MAX_CHUNKABLE_BYTECODE {
        return Err(Error::CardPayloadTooLarge {
            bytecode_len: canonical_bytecode.len(),
            max_supported: MAX_CHUNKABLE_BYTECODE,
        });
    }

    // Stream = bytecode || SHA-256(bytecode)[0..4]
    let hash = sha256::Hash::hash(canonical_bytecode);
    let mut stream = Vec::with_capacity(canonical_bytecode.len() + CROSS_CHUNK_HASH_BYTES);
    stream.extend_from_slice(canonical_bytecode);
    stream.extend_from_slice(&hash.to_byte_array()[..CROSS_CHUNK_HASH_BYTES]);

    let frag_size = CHUNKED_FRAGMENT_LONG_BYTES;
    let total: usize = stream.len().div_ceil(frag_size).max(1);
    debug_assert!(
        total <= MAX_CHUNKS as usize,
        "capacity check above guarantees this"
    );
    let total_chunks_u8: u8 = total as u8;

    let mut chunks = Vec::with_capacity(total);
    for i in 0..total {
        let start = i * frag_size;
        let end = ((i + 1) * frag_size).min(stream.len());
        let fragment = stream[start..end].to_vec();
        let header = StringLayerHeader::Chunked {
            version: VERSION_V0_1,
            chunk_set_id,
            total_chunks: total_chunks_u8,
            chunk_index: i as u8,
        };
        chunks.push(ChunkFragment { header, fragment });
    }
    Ok(chunks)
}

/// Reassemble canonical bytecode from a list of parsed chunks.
///
/// Validates SPEC §4 rules 11–13 in order:
///
/// 1. All chunks must be `Chunked` (mixing with `SingleString` is rejected).
/// 2. All chunks share `chunk_set_id` and `total_chunks`
///    ([`Error::ChunkSetIdMismatch`], [`Error::ChunkedHeaderMalformed`]).
/// 3. `chunk_index` values cover `0..total_chunks` exactly once
///    ([`Error::ChunkedHeaderMalformed`] on gaps, duplicates, or out-of-range).
/// 4. The reassembled stream's trailing 4-byte cross-chunk hash matches
///    `SHA-256(reassembled_bytecode)[0..4]` ([`Error::CrossChunkHashMismatch`]).
///
/// Chunks may arrive in any order; this function sorts internally.
pub fn reassemble_from_chunks(chunks: Vec<ChunkFragment>) -> Result<Vec<u8>> {
    if chunks.is_empty() {
        return Err(Error::ChunkedHeaderMalformed(
            "empty chunk list".to_string(),
        ));
    }

    // All chunks must be `Chunked` (no `SingleString` allowed at this entry).
    let (set_id, total) = match chunks[0].header {
        StringLayerHeader::Chunked {
            chunk_set_id,
            total_chunks,
            ..
        } => (chunk_set_id, total_chunks),
        StringLayerHeader::SingleString { .. } => {
            return Err(Error::ChunkedHeaderMalformed(
                "single-string header in multi-chunk reassembly".to_string(),
            ));
        }
    };

    let total_usize = total as usize;
    if chunks.len() != total_usize {
        return Err(Error::ChunkedHeaderMalformed(format!(
            "received {} chunks, header declares total_chunks = {total}",
            chunks.len()
        )));
    }

    // Place each chunk into a slot indexed by chunk_index; reject duplicates
    // and gaps by tracking which slots are filled.
    let mut slots: Vec<Option<Vec<u8>>> = (0..total_usize).map(|_| None).collect();
    for chunk in chunks {
        match chunk.header {
            StringLayerHeader::Chunked {
                version: _,
                chunk_set_id,
                total_chunks,
                chunk_index,
            } => {
                if chunk_set_id != set_id {
                    return Err(Error::ChunkSetIdMismatch);
                }
                if total_chunks != total {
                    return Err(Error::ChunkedHeaderMalformed(format!(
                        "total_chunks disagrees across chunks: saw {total} and {total_chunks}"
                    )));
                }
                let idx = chunk_index as usize;
                if idx >= total_usize {
                    return Err(Error::ChunkedHeaderMalformed(format!(
                        "chunk_index {idx} >= total_chunks {total}"
                    )));
                }
                if slots[idx].is_some() {
                    return Err(Error::ChunkedHeaderMalformed(format!(
                        "duplicate chunk_index {idx}"
                    )));
                }
                slots[idx] = Some(chunk.fragment);
            }
            StringLayerHeader::SingleString { .. } => {
                // A `SingleString` header at any non-leading position in
                // a chunked set is a header-types-disagree error, not a
                // chunked-internal malformation. Emitted as
                // [`Error::MixedHeaderTypes`] for symmetry with the
                // forward-direction reject in `pipeline::decode`.
                return Err(Error::MixedHeaderTypes);
            }
        }
    }

    // Concatenate fragments in chunk_index order.
    let mut stream = Vec::new();
    for (i, slot) in slots.into_iter().enumerate() {
        let frag =
            slot.ok_or_else(|| Error::ChunkedHeaderMalformed(format!("missing chunk_index {i}")))?;
        stream.extend_from_slice(&frag);
    }

    // Verify cross-chunk hash. Stream layout: bytecode || hash[0..4].
    if stream.len() < CROSS_CHUNK_HASH_BYTES {
        return Err(Error::ChunkedHeaderMalformed(
            "reassembled stream shorter than 4-byte cross-chunk hash".to_string(),
        ));
    }
    let split = stream.len() - CROSS_CHUNK_HASH_BYTES;
    let bytecode = &stream[..split];
    let recovered_hash = &stream[split..];
    let computed = sha256::Hash::hash(bytecode);
    if recovered_hash != &computed.to_byte_array()[..CROSS_CHUNK_HASH_BYTES] {
        return Err(Error::CrossChunkHashMismatch);
    }
    Ok(bytecode.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_bytecode(len: usize) -> Vec<u8> {
        // Deterministic but not all-zero, so the cross-chunk hash exercises
        // the SHA-256 path rather than the trivial digest.
        (0..len).map(|i| (i & 0xFF) as u8).collect()
    }

    #[test]
    fn split_then_reassemble_round_trip_short() {
        let bc = fixture_bytecode(60);
        let chunks = split_into_chunks(&bc, 0x12345).unwrap();
        // 60 + 4 = 64 stream bytes → ceil(64/53) = 2 chunks.
        assert_eq!(chunks.len(), 2);
        let recovered = reassemble_from_chunks(chunks).unwrap();
        assert_eq!(recovered, bc);
    }

    #[test]
    fn split_then_reassemble_round_trip_typical_mk1_card_size() {
        // 84 bytes ≈ typical 1-stub mainnet card with std-table indicator
        // and fingerprint present (per SPEC §3.2 worked example).
        let bc = fixture_bytecode(84);
        let chunks = split_into_chunks(&bc, 0xABCDE).unwrap();
        // 84 + 4 = 88 → ceil(88/53) = 2 chunks.
        assert_eq!(chunks.len(), 2);
        let recovered = reassemble_from_chunks(chunks).unwrap();
        assert_eq!(recovered, bc);
    }

    #[test]
    fn split_at_capacity_uses_max_chunks() {
        let bc = fixture_bytecode(MAX_CHUNKABLE_BYTECODE);
        let chunks = split_into_chunks(&bc, 0x55555).unwrap();
        assert_eq!(chunks.len(), MAX_CHUNKS as usize);
        let recovered = reassemble_from_chunks(chunks).unwrap();
        assert_eq!(recovered, bc);
    }

    #[test]
    fn split_rejects_oversized_bytecode() {
        let bc = vec![0u8; MAX_CHUNKABLE_BYTECODE + 1];
        let r = split_into_chunks(&bc, 0);
        assert!(matches!(r, Err(Error::CardPayloadTooLarge { .. })));
    }

    #[test]
    fn split_rejects_chunk_set_id_above_20_bits() {
        let bc = fixture_bytecode(60);
        let r = split_into_chunks(&bc, 0x10_0000);
        assert!(matches!(r, Err(Error::ChunkedHeaderMalformed(_))));
    }

    #[test]
    fn reassemble_accepts_out_of_order_chunks() {
        let bc = fixture_bytecode(150);
        let mut chunks = split_into_chunks(&bc, 0).unwrap();
        chunks.reverse();
        let recovered = reassemble_from_chunks(chunks).unwrap();
        assert_eq!(recovered, bc);
    }

    #[test]
    fn reassemble_rejects_chunk_set_id_mismatch() {
        let bc = fixture_bytecode(150);
        let mut chunks = split_into_chunks(&bc, 0x12345).unwrap();
        // Tamper the second chunk's chunk_set_id.
        if let StringLayerHeader::Chunked {
            ref mut chunk_set_id,
            ..
        } = chunks[1].header
        {
            *chunk_set_id = 0x00001;
        }
        assert!(matches!(
            reassemble_from_chunks(chunks),
            Err(Error::ChunkSetIdMismatch)
        ));
    }

    #[test]
    fn reassemble_rejects_cross_chunk_hash_mismatch() {
        let bc = fixture_bytecode(150);
        let mut chunks = split_into_chunks(&bc, 0).unwrap();
        // Flip a byte inside the FIRST chunk's payload — this falls in
        // the bytecode region, so the recomputed SHA-256 will differ.
        chunks[0].fragment[0] ^= 0x01;
        assert!(matches!(
            reassemble_from_chunks(chunks),
            Err(Error::CrossChunkHashMismatch)
        ));
    }

    #[test]
    fn reassemble_rejects_duplicate_chunk_index() {
        let bc = fixture_bytecode(150);
        let mut chunks = split_into_chunks(&bc, 0).unwrap();
        // Force two chunks to claim chunk_index = 0.
        if let StringLayerHeader::Chunked {
            ref mut chunk_index,
            ..
        } = chunks[1].header
        {
            *chunk_index = 0;
        }
        assert!(matches!(
            reassemble_from_chunks(chunks),
            Err(Error::ChunkedHeaderMalformed(_))
        ));
    }

    #[test]
    fn reassemble_rejects_missing_chunk() {
        let bc = fixture_bytecode(150);
        let mut chunks = split_into_chunks(&bc, 0).unwrap();
        // Drop the last chunk; reassembly must reject.
        chunks.pop();
        assert!(matches!(
            reassemble_from_chunks(chunks),
            Err(Error::ChunkedHeaderMalformed(_))
        ));
    }

    #[test]
    fn reassemble_rejects_empty_chunk_list() {
        assert!(matches!(
            reassemble_from_chunks(vec![]),
            Err(Error::ChunkedHeaderMalformed(_))
        ));
    }

    #[test]
    fn split_one_chunk_when_stream_fits_in_53_bytes() {
        // Bytecode 49 + 4-byte hash = 53 bytes → exactly fills one fragment.
        let bc = fixture_bytecode(49);
        let chunks = split_into_chunks(&bc, 0).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].fragment.len(), 53);
        let recovered = reassemble_from_chunks(chunks).unwrap();
        assert_eq!(recovered, bc);
    }

    #[test]
    fn split_handles_empty_bytecode() {
        // Degenerate but defined: 0 bytes → stream is just the 4-byte hash.
        let bc: Vec<u8> = vec![];
        let chunks = split_into_chunks(&bc, 0).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].fragment.len(), CROSS_CHUNK_HASH_BYTES);
        let recovered = reassemble_from_chunks(chunks).unwrap();
        assert_eq!(recovered, bc);
    }
}
