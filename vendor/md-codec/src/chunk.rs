//! Chunk header per SPEC v0.30 §2.2.
//!
//! Encodes the 37-bit chunked wire-format header. First-symbol layout
//! MSB-first: `[v3][v2][v1][v0][chunked]` (4-bit version + 1-bit chunked-flag).
//! Remainder: 20-bit chunk-set-id + 6-bit count-minus-1 + 6-bit index.
//! Total = 4 + 1 + 20 + 6 + 6 = 37 bits.
//!
//! v0.34.0: also hosts [`decode_with_correction`] — the BCH-error-correcting
//! decode entry point. Per chunk: parse → polymod-residue → (if non-zero)
//! call [`crate::bch_decode::decode_regular_errors`] → apply corrections →
//! re-encode → forward to [`reassemble`]. Atomic per plan §1 D28: any chunk
//! exceeding the BCH `t = 4` capacity fails the whole call without partial
//! output.

use crate::bitstream::{BitReader, BitWriter};
use crate::codex32::REGULAR_CODE_SYMBOLS_MAX;
use crate::error::Error;
use crate::header::Header;

/// Wire header for a single chunk in a chunked v0.30 payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkHeader {
    /// Wire-format version (4 bits). v0.30 = 4.
    pub version: u8,
    /// 20-bit chunk-set identifier shared by all chunks in a set.
    pub chunk_set_id: u32,
    /// Total number of chunks in the set; valid range `1..=64`.
    pub count: u8,
    /// Zero-based index of this chunk within the set; must be `< count`.
    pub index: u8,
}

impl ChunkHeader {
    /// Encode the chunk header into `w` as 37 bits.
    ///
    /// Returns an error if `count`, `index`, or `chunk_set_id` are out of range.
    pub fn write(&self, w: &mut BitWriter) -> Result<(), Error> {
        if !(1..=64).contains(&(self.count as u32)) {
            return Err(Error::ChunkCountOutOfRange { count: self.count });
        }
        if self.index >= self.count {
            return Err(Error::ChunkIndexOutOfRange {
                index: self.index,
                count: self.count,
            });
        }
        if self.chunk_set_id >= (1 << 20) {
            return Err(Error::ChunkSetIdOutOfRange {
                id: self.chunk_set_id,
            });
        }
        w.write_bits(u64::from(self.version & 0b1111), 4);
        w.write_bits(1, 1); // chunked = 1
        w.write_bits(u64::from(self.chunk_set_id), 20);
        w.write_bits((self.count - 1) as u64, 6); // count-1 offset
        w.write_bits(u64::from(self.index), 6);
        Ok(())
    }

    /// Decode a chunk header (37 bits) from `r`.
    ///
    /// Returns [`Error::WireVersionMismatch`] if the 4-bit version field
    /// is not `WF_REDESIGN_VERSION` per SPEC §2.5 (e.g., v0.x chunked
    /// payloads where version=0 in the first 3 wire bits become version=0
    /// or version=1 under the v0.30 4-bit read depending on prior bits).
    /// Returns [`Error::ChunkHeaderChunkedFlagMissing`] if the chunked-flag
    /// bit is not set after the version check passes.
    pub fn read(r: &mut BitReader) -> Result<Self, Error> {
        let version = r.read_bits(4)? as u8;
        if version != Header::WF_REDESIGN_VERSION {
            return Err(Error::WireVersionMismatch { got: version });
        }
        let chunked = r.read_bits(1)? != 0;
        if !chunked {
            return Err(Error::ChunkHeaderChunkedFlagMissing);
        }
        let chunk_set_id = r.read_bits(20)? as u32;
        let count = (r.read_bits(6)? + 1) as u8;
        let index = r.read_bits(6)? as u8;
        Ok(Self {
            version,
            chunk_set_id,
            count,
            index,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::Header;

    #[test]
    fn chunk_header_round_trip() {
        let h = ChunkHeader {
            version: Header::WF_REDESIGN_VERSION,
            chunk_set_id: 0xABCDE,
            count: 3,
            index: 1,
        };
        let mut w = BitWriter::new();
        h.write(&mut w).unwrap();
        // 4 + 1 + 20 + 6 + 6 = 37 bits
        assert_eq!(w.bit_len(), 37);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(ChunkHeader::read(&mut r).unwrap(), h);
    }

    #[test]
    fn chunk_header_count_64_round_trip() {
        let h = ChunkHeader {
            version: Header::WF_REDESIGN_VERSION,
            chunk_set_id: 0,
            count: 64,
            index: 63,
        };
        let mut w = BitWriter::new();
        h.write(&mut w).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(ChunkHeader::read(&mut r).unwrap(), h);
    }

    #[test]
    fn chunk_header_count_zero_rejected() {
        let h = ChunkHeader {
            version: Header::WF_REDESIGN_VERSION,
            chunk_set_id: 0,
            count: 0,
            index: 0,
        };
        let mut w = BitWriter::new();
        assert!(matches!(
            h.write(&mut w),
            Err(Error::ChunkCountOutOfRange { count: 0 })
        ));
    }

    /// SPEC v0.30 §2.5 v0.x rejection for chunk-header path. A wire crafted
    /// with version=0 and chunked-flag=1 (the v0.30-layout interpretation of
    /// what a v0.x chunked first-symbol becomes when reordered) must be
    /// rejected with `WireVersionMismatch { got: 0 }`.
    #[test]
    fn chunk_header_rejects_v0x_version() {
        // Construct first 5 bits MSB-first: [v3=0][v2=0][v1=0][v0=0][chunked=1]
        //   = 0b00001 (numeric 1)
        // Pad with 32 zero bits (chunk_set_id + count-1 + index) to reach
        // the full 37-bit chunk header length. 37 bits packed MSB-first into
        // 5 bytes (with 3 trailing zero bits beyond the bit limit).
        // Easier: use BitWriter to build the wire deterministically.
        let mut w = BitWriter::new();
        w.write_bits(0, 4); // version = 0 (v0.x)
        w.write_bits(1, 1); // chunked = 1
        w.write_bits(0, 20); // chunk_set_id
        w.write_bits(0, 6); // count-1
        w.write_bits(0, 6); // index
        assert_eq!(w.bit_len(), 37);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert!(matches!(
            ChunkHeader::read(&mut r),
            Err(Error::WireVersionMismatch { got: 0 })
        ));
    }
}

use crate::identity::Md1EncodingId;

/// Derive the 20-bit chunk-set-id from a [`Md1EncodingId`] by taking the
/// top 20 bits of the underlying 16-byte hash, MSB-first.
///
/// The chunk-set-id groups chunks belonging to the same encoded payload.
/// Returned value is in the range `0..=0xFFFFF`.
pub fn derive_chunk_set_id(id: &Md1EncodingId) -> u32 {
    // First 20 bits of Md1EncodingId[0..16], MSB-first.
    let bytes = id.as_bytes();
    ((bytes[0] as u32) << 12) | ((bytes[1] as u32) << 4) | ((bytes[2] as u32) >> 4)
}

#[cfg(test)]
mod chunk_set_id_tests {
    use super::*;

    #[test]
    fn derive_chunk_set_id_deterministic() {
        let mut bytes = [0u8; 16];
        bytes[0] = 0xab;
        bytes[1] = 0xcd;
        bytes[2] = 0xe1;
        bytes[3] = 0x23;
        let id = Md1EncodingId::new(bytes);
        let csid_a = derive_chunk_set_id(&id);
        let csid_b = derive_chunk_set_id(&id);
        assert_eq!(csid_a, csid_b);
    }

    #[test]
    fn derive_chunk_set_id_msb_first_extraction() {
        // bytes[0]=0xAB, [1]=0xCD, [2]=0xEF: top 20 bits = 0xABCDE
        let mut bytes = [0u8; 16];
        bytes[0] = 0xAB;
        bytes[1] = 0xCD;
        bytes[2] = 0xEF;
        let id = Md1EncodingId::new(bytes);
        assert_eq!(derive_chunk_set_id(&id), 0xABCDE);
    }
}

use crate::encode::Descriptor;

/// Per-chunk payload *sizing* budget (in payload bits) that [`split`] uses to
/// choose the chunk count: `count = ceil(padded_payload_bits / 320)`. It is 64
/// data symbols (64 × 5 = 320 bits), deliberately BELOW the codex32 regular
/// single-string data cap of 80 symbols / 400 bits (enforced by
/// [`crate::codex32::wrap_payload`]), so each chunk's 37-bit header fits
/// alongside the fragment inside one regular-code codeword.
///
/// NOTE: this is the chunk-*sizing* budget, NOT the single-string threshold.
/// A payload that fits ≤ 400 bits is emitted as ONE string; only a payload
/// exceeding the 400-bit single-string cap (or an explicit `--force-chunked`)
/// is split — and once split, chunks are sized by this 320-bit budget.
pub const SINGLE_STRING_PAYLOAD_BIT_LIMIT: usize = 64 * 5;

/// Split a [`Descriptor`] into N codex32 md1 strings, each carrying a chunk
/// header and a slice of the canonical payload.
///
/// Algorithm:
/// 1. Encode the full payload (`encode_payload`).
/// 2. Compute [`crate::identity::Md1EncodingId`]; derive `ChunkSetId`.
/// 3. Choose chunk count N such that each chunk fits in codex32 long form
///    after adding the 37-bit chunk header.
/// 4. Split the payload into N approximately-equal byte-boundary slices.
/// 5. For each chunk i: prepend chunk header (37 bits), wrap via codex32 with
///    the chunked-flag bit set, emit md1 string.
///
/// Note: `bytes_per_chunk` could be 0 if `payload_bytes` were empty, but the
/// encoder validates `n ≥ 1` so the payload is always non-empty.
pub fn split(d: &Descriptor) -> Result<Vec<String>, Error> {
    use crate::bitstream::BitWriter;
    use crate::encode::encode_payload;
    use crate::identity::compute_md1_encoding_id;

    let (payload_bytes, _payload_bits) = encode_payload(d)?;

    // Compute ChunkSetId from full-encoding hash.
    let md1_id = compute_md1_encoding_id(d)?;
    let chunk_set_id = derive_chunk_set_id(&md1_id);

    // Choose chunk count from payload byte count (≤7 bits of trailing
    // codex32-padding are tolerated by the reassembled-stream TLV-rollback).
    let payload_bit_count_for_sizing = payload_bytes.len() * 8;
    let chunks_needed = payload_bit_count_for_sizing.div_ceil(SINGLE_STRING_PAYLOAD_BIT_LIMIT);
    if chunks_needed > 64 {
        return Err(Error::ChunkCountExceedsMax {
            needed: chunks_needed,
        });
    }
    let count: u8 = if chunks_needed == 0 {
        1
    } else {
        chunks_needed as u8
    };

    // Split payload into `count` byte-boundary slices.
    let bytes_per_chunk = payload_bytes.len().div_ceil(count as usize);

    let mut chunks = Vec::with_capacity(count as usize);
    for index in 0..count {
        let start_byte = (index as usize) * bytes_per_chunk;
        let end_byte = ((index as usize + 1) * bytes_per_chunk).min(payload_bytes.len());
        let chunk_payload_bytes = &payload_bytes[start_byte..end_byte];

        // Build per-chunk wire: 37-bit chunk header + chunk-payload bytes
        // (full 8 bits per byte, no further fractional content). Chunk's
        // exact bit count = 37 + 8 × |chunk_payload_bytes|.
        let header = ChunkHeader {
            version: Header::WF_REDESIGN_VERSION,
            chunk_set_id,
            count,
            index,
        };
        let mut w = BitWriter::new();
        header.write(&mut w)?;
        for byte in chunk_payload_bytes {
            w.write_bits(u64::from(*byte), 8);
        }
        let chunk_bit_count = 37 + 8 * chunk_payload_bytes.len();
        let bytes = w.into_bytes();
        let s = crate::codex32::wrap_payload(&bytes, chunk_bit_count)?;
        chunks.push(s);
    }
    Ok(chunks)
}

/// Reassemble a [`Descriptor`] from N md1 codex32 strings (strict:
/// byte-identical to pre-P0 behavior). Delegates to
/// [`reassemble_with_opts`] with the default (strict) options.
///
/// Algorithm:
/// 1. Unwrap each string via the codex32 layer (verifies BCH per chunk).
/// 2. Parse the 37-bit chunk header from each.
/// 3. Validate consistency: same version, chunk_set_id, count.
/// 4. Sort by index; verify `0..count-1` with no gaps.
/// 5. Concatenate per-chunk payload bytes.
/// 6. Decode the reassembled payload via
///    [`crate::decode::decode_payload`].
/// 7. Verify the reassembled payload's derived chunk-set-id matches the
///    chunk-set-id present in every chunk header (cross-chunk integrity).
pub fn reassemble(strings: &[&str]) -> Result<Descriptor, Error> {
    reassemble_with_opts(strings, crate::decode::DecodeOpts::default())
}

/// Reassemble a [`Descriptor`] from N md1 codex32 strings, honoring
/// `opts` (P0 partial-decode; see [`crate::decode::DecodeOpts`] for the
/// contract). Same algorithm as [`reassemble`], except step 6 decodes via
/// [`crate::decode::decode_payload_with_opts`] instead of the strict
/// primitive.
///
/// INVARIANT (funds-load-bearing): `opts.allow_unresolved_origin` relaxes
/// ONLY the origin-gate outcome of the step-6 decode call. Every check
/// ABOVE that call (per-chunk BCH via `unwrap_string`, chunk-header
/// consistency, index-gap) and the derived-chunk-set-id / content-id
/// check BELOW it (step 7) stay enforced UNCONDITIONALLY regardless of
/// `opts` — a chunk set with a doctored chunk-set-id still rejects with
/// `Error::ChunkSetIdMismatch` even when `allow_unresolved_origin: true`.
pub fn reassemble_with_opts(
    strings: &[&str],
    opts: crate::decode::DecodeOpts,
) -> Result<Descriptor, Error> {
    use crate::bitstream::BitReader;
    use crate::codex32::unwrap_string;
    use crate::decode::decode_payload_with_opts;
    use crate::identity::compute_md1_encoding_id;

    if strings.is_empty() {
        return Err(Error::ChunkSetEmpty);
    }

    // Unwrap each, parse 37-bit chunk header, then read whole payload bytes.
    // Use the symbol-aligned bit count returned by `unwrap_string` (NOT
    // `bytes.len() * 8`, which would over-estimate by up to 7 bits and break
    // round-trip for chunks where symbol-padding plus byte-padding crosses a
    // byte boundary — e.g. N=3, N=8, etc.).
    let mut parsed: Vec<(ChunkHeader, Vec<u8>)> = Vec::with_capacity(strings.len());
    for s in strings {
        let (bytes, symbol_aligned_bit_count) = unwrap_string(s)?;
        let mut r = BitReader::with_bit_limit(&bytes, symbol_aligned_bit_count);
        let header = ChunkHeader::read(&mut r)?;
        // Per encoder contract: chunk wire is exactly 37 + 8N bits. The
        // symbol-aligned bit count is `ceil((37+8N)/5) * 5`, which is in
        // [37+8N, 37+8N+4]. So `(symbol_aligned_bit_count - 37) / 8`
        // (floor) recovers exactly N.
        let payload_byte_count = (symbol_aligned_bit_count - 37) / 8;
        let mut chunk_payload_bytes = Vec::with_capacity(payload_byte_count);
        for _ in 0..payload_byte_count {
            let v = r.read_bits(8)? as u8;
            chunk_payload_bytes.push(v);
        }
        // Trailing ≤4 symbol-padding bits remain in r; discard.
        parsed.push((header, chunk_payload_bytes));
    }

    // Validate consistency.
    let (h0, _) = &parsed[0];
    let expected_count = h0.count;
    let expected_csid = h0.chunk_set_id;
    let expected_version = h0.version;
    for (h, _) in &parsed {
        if h.count != expected_count
            || h.chunk_set_id != expected_csid
            || h.version != expected_version
        {
            return Err(Error::ChunkSetInconsistent);
        }
    }
    if parsed.len() != expected_count as usize {
        return Err(Error::ChunkSetIncomplete {
            got: parsed.len(),
            expected: expected_count as usize,
        });
    }

    // Sort by index; verify 0..count-1 with no gaps.
    parsed.sort_by_key(|(h, _)| h.index);
    for (i, (h, _)) in parsed.iter().enumerate() {
        if h.index as usize != i {
            return Err(Error::ChunkIndexGap {
                expected: i as u8,
                got: h.index,
            });
        }
    }

    // Concatenate chunk payload bytes.
    let mut full_bytes = Vec::new();
    for (_, chunk_bytes) in &parsed {
        full_bytes.extend_from_slice(chunk_bytes);
    }

    // Decode payload, honoring `opts` (P0.2). bit_len = bytes.len() * 8;
    // TLV-rollback handles trailing padding.
    let descriptor = decode_payload_with_opts(&full_bytes, full_bytes.len() * 8, opts)?;

    // Cross-chunk integrity check — UNCONDITIONAL regardless of `opts`
    // (the content-id oracle; P0.2 funds-load-bearing invariant).
    let md1_id = compute_md1_encoding_id(&descriptor)?;
    let derived_csid = derive_chunk_set_id(&md1_id);
    if derived_csid != expected_csid {
        return Err(Error::ChunkSetIdMismatch {
            expected: expected_csid,
            derived: derived_csid,
        });
    }

    Ok(descriptor)
}

// ---------------------------------------------------------------------------
// v0.34.0: BCH-error-correcting decode (plan §1 D22 + §2.B.1).
// ---------------------------------------------------------------------------

/// Per-correction report emitted by [`decode_with_correction`]. One entry
/// per repaired character. `position` is 0-indexed into the codex32
/// data-part (i.e. the characters following the `md1` HRP + separator);
/// `was` is the original (corrupted) char from the input; `now` is the
/// corrected char.
///
/// Atomic per plan §1 D28: when [`decode_with_correction`] succeeds the
/// returned vector aggregates corrections across all chunks; chunks that
/// were already valid contribute nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrectionDetail {
    /// 0-indexed position of the chunk in the caller's `&[&str]` slice.
    pub chunk_index: usize,
    /// 0-indexed position of the corrected character within the chunk's
    /// data-part (post-HRP-and-separator).
    pub position: usize,
    /// The original (corrupted) character at this position.
    pub was: char,
    /// The corrected character at this position.
    pub now: char,
}

/// Local codex32 alphabet (BIP 173 lowercase). Each char = one 5-bit
/// symbol. Duplicated from `codex32.rs` (which keeps it private) so this
/// module doesn't widen the codex32 public surface; the mapping is
/// constant per BIP 173.
const CODEX32_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// BIP 173 separator character between HRP and data-part for md1 strings.
const HRP_PREFIX: &str = "md1";

/// Parse a single md1 chunk into its 5-bit data-part symbol vector.
/// Returns the data-with-checksum symbols (i.e. all symbols after `md1`).
/// Visual separators (whitespace + `-`) are stripped per codex32 convention.
fn parse_chunk_symbols(chunk: &str, chunk_index: usize) -> Result<Vec<u8>, Error> {
    // BIP-173: reject mixed-case (per chunk). The correction path rejects too —
    // case is lowercased before symbol mapping, so a case-flip is a zero-symbol-
    // error event never in the BCH channel; a wholesale mixed-case string is a
    // malformed encoding, not noise to correct. (Mirrors mk-codec's correcting
    // decode, which rejects MixedCase before correction.)
    if crate::codex32::is_mixed_case(chunk) {
        return Err(Error::Codex32DecodeError(format!(
            "chunk {chunk_index}: string mixes upper and lower case (BIP-173 forbids mixed case)"
        )));
    }
    let lower = chunk.to_ascii_lowercase();
    if !lower.starts_with(HRP_PREFIX) {
        return Err(Error::Codex32DecodeError(format!(
            "chunk {chunk_index}: string does not start with HRP md1"
        )));
    }
    let rest = &lower[HRP_PREFIX.len()..];
    let mut symbols: Vec<u8> = Vec::with_capacity(rest.len());
    for c in rest.chars() {
        if c.is_whitespace() || c == '-' {
            continue;
        }
        let lc = c as u8;
        let sym = CODEX32_ALPHABET
            .iter()
            .position(|&b| b == lc)
            .ok_or_else(|| {
                Error::Codex32DecodeError(format!(
                    "chunk {chunk_index}: character {c:?} not in codex32 alphabet"
                ))
            })? as u8;
        symbols.push(sym);
    }
    Ok(symbols)
}

/// Re-encode a 5-bit data-part symbol vector as a complete md1 string.
fn encode_chunk_string(data_with_checksum: &[u8]) -> String {
    let mut out = String::with_capacity(HRP_PREFIX.len() + data_with_checksum.len());
    out.push_str(HRP_PREFIX);
    for &v in data_with_checksum {
        out.push(CODEX32_ALPHABET[(v & 0x1F) as usize] as char);
    }
    out
}

/// BCH-error-correcting decode for a chunk-set of md1 strings.
///
/// Per plan §1 Q1 lock — full-decode semantics: this is the single entry
/// point that callers needing both "did anything get repaired?" AND "the
/// fully-decoded descriptor" should use.
///
/// Algorithm:
/// 1. For each chunk, parse the data-part into 5-bit symbols and compute
///    the BCH polymod residue (`hrp_expand("md") || data_with_checksum`)
///    XOR'd against [`crate::bch::MD_REGULAR_CONST`].
/// 2. Residue `== 0` ⇒ chunk passes through unchanged.
/// 3. Residue `!= 0` ⇒ invoke
///    [`crate::bch_decode::decode_regular_errors`]. If `None`, return
///    `Err(Error::TooManyErrors { chunk_index, bound: 8 })` per plan §2.B.4
///    D29 error-mapping table.
/// 4. Apply corrections to the chunk's symbol vector, re-encode as a
///    fresh md1 string, and record one [`CorrectionDetail`] per repaired
///    character.
/// 5. After ALL chunks have been processed (any single uncorrectable
///    chunk aborts atomically per plan §1 D28), forward the corrected
///    chunk strings to [`reassemble`] to produce the [`Descriptor`].
///
/// On success returns `(Descriptor, Vec<CorrectionDetail>)`. The
/// correction-detail vector is in (`chunk_index` ascending,
/// `position` ascending within chunk) order; an empty vector means every
/// input chunk was already a valid codeword.
pub fn decode_with_correction(
    strings: &[&str],
) -> Result<(Descriptor, Vec<CorrectionDetail>), Error> {
    if strings.is_empty() {
        return Err(Error::ChunkSetEmpty);
    }

    let mut corrected_strings: Vec<String> = Vec::with_capacity(strings.len());
    // Track the post-correction 5-bit symbol vector of the first string so the
    // single-string detection pre-pass below can inspect bit 0 of the first
    // symbol (the chunked-flag per SPEC v0.30 §2.3) without re-parsing the
    // wrapped string.
    let mut first_corrected_symbols: Option<Vec<u8>> = None;
    let mut all_details: Vec<CorrectionDetail> = Vec::new();

    for (chunk_index, chunk) in strings.iter().enumerate() {
        let symbols = parse_chunk_symbols(chunk, chunk_index)?;

        // cycle-4 M4: reject any chunk longer than the codex32 regular code's
        // 93-symbol codeword BEFORE the residue/correction logic. β has order
        // 93, so degrees d and d+93 alias in chien_search for an over-93-symbol
        // word — the correcting decoder would otherwise mis-correct at an
        // aliased root. This precedes the residue==0 pass-through, so a clean
        // over-length md1 is rejected on `repair` too (the correct domain gate;
        // composes with H6's encode cap). Fail-closed.
        if symbols.len() > REGULAR_CODE_SYMBOLS_MAX {
            return Err(Error::ChunkSymbolCountOutOfRange {
                chunk_index,
                symbols: symbols.len(),
                max: REGULAR_CODE_SYMBOLS_MAX,
            });
        }

        // Polymod residue against md1's target constant.
        let mut input = crate::bch::hrp_expand("md");
        input.extend_from_slice(&symbols);
        let residue = crate::bch::polymod_run(&input) ^ crate::bch::MD_REGULAR_CONST;

        if residue == 0 {
            // Already valid — pass through unchanged.
            corrected_strings.push((*chunk).to_string());
            if chunk_index == 0 {
                first_corrected_symbols = Some(symbols);
            }
            continue;
        }

        // Attempt BCH correction.
        let (positions, magnitudes) =
            crate::bch_decode::decode_regular_errors(residue, symbols.len()).ok_or(
                Error::TooManyErrors {
                    chunk_index,
                    bound: 8,
                },
            )?;

        // Apply corrections; record (was, now) chars per position.
        let mut corrected = symbols.clone();
        let mut details: Vec<CorrectionDetail> = Vec::with_capacity(positions.len());
        for (&pos, &mag) in positions.iter().zip(&magnitudes) {
            if pos >= corrected.len() {
                // Defensive: chien_search bounded pos to [0, L); but a
                // pathological 5+-error pattern could in principle skirt
                // that. Treat as uncorrectable per Q2 absorption rules.
                return Err(Error::TooManyErrors {
                    chunk_index,
                    bound: 8,
                });
            }
            let was_byte = corrected[pos];
            let now_byte = was_byte ^ mag;
            let was = CODEX32_ALPHABET[(was_byte & 0x1F) as usize] as char;
            let now = CODEX32_ALPHABET[(now_byte & 0x1F) as usize] as char;
            details.push(CorrectionDetail {
                chunk_index,
                position: pos,
                was,
                now,
            });
            corrected[pos] = now_byte;
        }

        // Defensive re-verify (catches pathological 5+-error patterns
        // that happen to produce a degree-≤4 locator with 4 valid roots).
        let mut verify_input = crate::bch::hrp_expand("md");
        verify_input.extend_from_slice(&corrected);
        let verify_residue = crate::bch::polymod_run(&verify_input) ^ crate::bch::MD_REGULAR_CONST;
        if verify_residue != 0 {
            return Err(Error::TooManyErrors {
                chunk_index,
                bound: 8,
            });
        }

        corrected_strings.push(encode_chunk_string(&corrected));
        if chunk_index == 0 {
            first_corrected_symbols = Some(corrected);
        }
        all_details.extend(details);
    }

    // v0.35.0: single-string auto-dispatch per SPEC v0.30 §2.3. The first
    // 5-bit symbol of the corrected payload carries the chunked-flag in
    // bit 0 (0 = single-payload, 1 = chunked). When the sole input string
    // decodes (post-BCH correction) as non-chunked, route it through the
    // single-payload decode path rather than `reassemble`. When it
    // decodes as chunked, fall through to the existing `reassemble`
    // path — which naturally surfaces `ChunkSetIncomplete { got: 1,
    // expected: count }` for any `count > 1` (the "chunked-bit set but
    // only one chunk supplied" ambiguity edge per plan §2.D.1) while
    // preserving the legitimate count==1 chunked-of-1 case shipped in
    // v0.34.0.
    if strings.len() == 1 {
        // `first_corrected_symbols` is populated by the loop above (both
        // the residue==0 pass-through and the correction-applied paths
        // populate it for `chunk_index == 0`).
        let symbols = first_corrected_symbols
            .as_ref()
            .expect("loop populates first_corrected_symbols when strings.len() >= 1");
        let chunked_flag = symbols.first().map(|s| s & 0x01).unwrap_or(1);
        if chunked_flag == 0 {
            // Non-chunked: decode via the single-payload path. The
            // corrected string passes BCH-verify (proven by the defensive
            // re-verify above; or by residue == 0 in the pass-through
            // branch), so `decode_md1_string` will not re-fail at the
            // codex32 layer.
            let descriptor = crate::decode::decode_md1_string(&corrected_strings[0])?;
            return Ok((descriptor, all_details));
        }
        // chunked_flag == 1: fall through to `reassemble` below.
    }

    // Hand corrected strings to the existing reassembly path.
    let corrected_refs: Vec<&str> = corrected_strings.iter().map(|s| s.as_str()).collect();
    let descriptor = reassemble(&corrected_refs)?;
    Ok((descriptor, all_details))
}
