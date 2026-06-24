//! v0.11 ↔ codex32 BCH layer adapter, symbol-aligned per spec §3.1 / D7.
//!
//! Bypasses v0.x's byte-oriented `encode_string` / `decode_string` to avoid
//! adding an extra codex32 char per encoding due to byte-padding. Uses v0.x's
//! lower-level BCH primitives (`bch_create_checksum_regular`,
//! `bch_verify_regular`) which operate on `&[u8]` slices of 5-bit symbols.

use crate::bitstream::{BitReader, BitWriter};
use crate::error::Error;

/// Codex32 alphabet (BIP 173 lowercase). Each char = one 5-bit symbol.
const CODEX32_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// HRP for v0.11 (matches v0.x).
const HRP: &str = "md";

/// Regular-BCH checksum length, in 5-bit symbols.
pub(crate) const REGULAR_CHECKSUM_SYMBOLS: usize = 13;

/// Maximum data-symbol count for a single codex32 regular-code string.
/// The codex32 regular code is BCH(93, 80, 8): `REGULAR_DATA_SYMBOLS_MAX +
/// REGULAR_CHECKSUM_SYMBOLS == 93` (80 data + 13 checksum). Payloads exceeding
/// this cap MUST be chunked (`split()` / `--force-chunked`); a single string
/// cannot carry them. Enforced at the top of [`wrap_payload`] (cycle-4 H6).
pub(crate) const REGULAR_DATA_SYMBOLS_MAX: usize = 80;

/// Maximum total codeword length (data + checksum) for a single codex32
/// regular-code string: `REGULAR_DATA_SYMBOLS_MAX + REGULAR_CHECKSUM_SYMBOLS
/// == 93`. The generator `β` has order 93, so a word longer than this aliases
/// under the BCH decoder. Enforced on the decode boundaries (cycle-4 M4 in
/// `chunk::decode_with_correction`; cycle-4 I1 in [`unwrap_string`]).
pub(crate) const REGULAR_CODE_SYMBOLS_MAX: usize =
    REGULAR_DATA_SYMBOLS_MAX + REGULAR_CHECKSUM_SYMBOLS;

/// Pack `bit_count` bits from `payload_bytes` into 5-bit symbols. Pads the
/// final symbol with zeros if `bit_count` is not a multiple of 5. Returns
/// `ceil(bit_count / 5)` symbols. Each output u8 contains a 5-bit value.
fn bits_to_symbols(payload_bytes: &[u8], bit_count: usize) -> Result<Vec<u8>, Error> {
    let symbol_count = bit_count.div_ceil(5);
    let mut r = BitReader::with_bit_limit(payload_bytes, bit_count);
    let mut symbols = Vec::with_capacity(symbol_count);
    for _ in 0..symbol_count {
        let take = r.remaining_bits().min(5);
        let val = if take == 0 {
            0
        } else {
            r.read_bits(take)? as u8
        };
        // Left-justify within 5 bits if final symbol is short. (For decoder
        // round-trip purposes the spec defines bit-packing MSB-first into
        // 5-bit symbols, so zero-padding the LOW bits of the final symbol is
        // the canonical form.)
        let symbol = (val << (5 - take as u32)) & 0x1F;
        symbols.push(symbol);
    }
    Ok(symbols)
}

/// Convert a stream of 5-bit symbols back into byte-padded bytes (MSB-first).
fn symbols_to_bytes(symbols: &[u8]) -> Vec<u8> {
    let mut w = BitWriter::new();
    for &s in symbols {
        w.write_bits((s & 0x1F) as u64, 5);
    }
    w.into_bytes()
}

fn symbol_to_char(s: u8) -> char {
    CODEX32_ALPHABET[(s & 0x1F) as usize] as char
}

fn char_to_symbol(c: char) -> Option<u8> {
    let lc = c.to_ascii_lowercase() as u8;
    CODEX32_ALPHABET
        .iter()
        .position(|&b| b == lc)
        .map(|i| i as u8)
}

/// Wrap a v0.11 payload bit stream (byte-padded with exact `bit_count`)
/// into a complete codex32 md1 string with HRP and BCH checksum, symbol-aligned.
pub fn wrap_payload(payload_bytes: &[u8], bit_count: usize) -> Result<String, Error> {
    let data_symbols = bits_to_symbols(payload_bytes, bit_count)?;
    // cycle-4 H6: enforce the regular-code 80-data-symbol cap at the lowest
    // shared chokepoint (every `wrap_payload` caller inherits it, including
    // `encode_md1_string`). An over-length single string is un-decodable under
    // the BCH(93, 80, 8) regular code, so fail closed and direct the caller to
    // chunked encoding rather than emit an aliasing-prone card.
    if data_symbols.len() > REGULAR_DATA_SYMBOLS_MAX {
        return Err(Error::PayloadTooLongForSingleString {
            data_symbols: data_symbols.len(),
            max: REGULAR_DATA_SYMBOLS_MAX,
        });
    }
    // v0.x exposes `bch_create_checksum_regular(hrp: &str, data: &[u8]) -> [u8; 13]`.
    let checksum: [u8; 13] = crate::bch::bch_create_checksum_regular(HRP, &data_symbols);

    let mut s =
        String::with_capacity(HRP.len() + 1 + data_symbols.len() + REGULAR_CHECKSUM_SYMBOLS);
    s.push_str(HRP);
    s.push('1'); // BIP 173-style HRP separator
    for sym in &data_symbols {
        s.push(symbol_to_char(*sym));
    }
    for sym in checksum.iter() {
        s.push(symbol_to_char(*sym));
    }
    Ok(s)
}

/// BIP-173: a bech32/codex32 string must NOT mix upper and lower case. Returns
/// true iff `s` (ignoring `-`/whitespace separators + digits, which are
/// case-neutral) contains BOTH an ASCII-uppercase AND an ASCII-lowercase letter.
/// All-upper, all-lower, and no-letters are fine. Mirrors mk-codec's
/// `case_check` (`string_layer/bch.rs`); shared with `chunk::parse_chunk_symbols`.
pub(crate) fn is_mixed_case(s: &str) -> bool {
    let mut has_upper = false;
    let mut has_lower = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            has_upper = true;
        } else if c.is_ascii_lowercase() {
            has_lower = true;
        }
        if has_upper && has_lower {
            return true;
        }
    }
    false
}

/// Unwrap a v0.11 md1 string into (byte-padded payload bytes, symbol-aligned bit count).
///
/// The returned `symbol_aligned_bit_count = 5 × data_symbol_count`. This is
/// the EXACT bit length carried by the codex32 BCH layer (rounded up to the
/// next 5-bit boundary from the actual payload). The caller uses this as
/// `decode_payload`'s `bit_len` so the v11 decoder's TLV-rollback only sees
/// ≤4 bits of trailing zero-padding (well under the 7-bit threshold).
pub fn unwrap_string(s: &str) -> Result<(Vec<u8>, usize), Error> {
    // BIP-173: reject mixed-case input (all-upper / all-lower both OK, the
    // latter canonicalized below). md-codec was the one constellation codec
    // that leniently accepted mixed case; mk-codec + ms-codec reject it.
    if is_mixed_case(s) {
        return Err(Error::Codex32DecodeError(
            "string mixes upper and lower case (BIP-173 forbids mixed case)".to_string(),
        ));
    }
    // 1. Strip HRP + separator.
    let prefix = format!("{}1", HRP);
    if !s.to_ascii_lowercase().starts_with(&prefix) {
        return Err(Error::Codex32DecodeError(format!(
            "string does not start with HRP {prefix}"
        )));
    }
    let symbols_str = &s[prefix.len()..];

    // 2. Char-to-symbol decode (tolerate visual separators per D11).
    let mut symbols = Vec::with_capacity(symbols_str.len());
    for c in symbols_str.chars() {
        if c.is_whitespace() || c == '-' {
            continue;
        }
        let sym = char_to_symbol(c).ok_or_else(|| {
            Error::Codex32DecodeError(format!("character {c:?} not in codex32 alphabet"))
        })?;
        symbols.push(sym);
    }

    // cycle-4 I1 (§5.2.3): reject an over-93-symbol codeword BEFORE the
    // length-agnostic BCH verify. A clean (residue==0) over-length word is
    // BCH-verifiable but structurally out-of-domain for the regular code
    // (β has order 93). Symmetric with the too-short floor below; fail-closed
    // so a non-correcting `decode` cannot accept an out-of-domain payload.
    if symbols.len() > REGULAR_CODE_SYMBOLS_MAX {
        return Err(Error::StringSymbolCountOutOfRange {
            symbols: symbols.len(),
            max: REGULAR_CODE_SYMBOLS_MAX,
        });
    }

    // 3. BCH-verify.
    if !crate::bch::bch_verify_regular(HRP, &symbols) {
        return Err(Error::Codex32DecodeError(
            "BCH checksum verification failed".into(),
        ));
    }

    // 4. Strip the 13-symbol checksum.
    if symbols.len() < REGULAR_CHECKSUM_SYMBOLS {
        return Err(Error::Codex32DecodeError(
            "string too short for BCH checksum".into(),
        ));
    }
    let data_symbols = &symbols[..symbols.len() - REGULAR_CHECKSUM_SYMBOLS];
    let bit_count = 5 * data_symbols.len();

    // 5. Convert symbols → byte-padded bytes.
    Ok((symbols_to_bytes(data_symbols), bit_count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_unwrap_round_trip_57_bits() {
        // Synthetic 57-bit payload (mimics BIP 84 single-sig length).
        let mut w = BitWriter::new();
        w.write_bits(0xDEAD_BEEF_CAFE_BABE_u64 >> 7, 57);
        let bytes = w.into_bytes();
        let s = wrap_payload(&bytes, 57).unwrap();
        // HRP "md1" (3 chars) + 12 data symbols + 13 checksum = 28 chars.
        assert_eq!(s.len(), 28);
        assert!(s.starts_with("md1"));
        let (out_bytes, out_bits) = unwrap_string(&s).unwrap();
        // Symbol-aligned bit count = 5 * 12 = 60 (≥ 57 by ≤4 padding bits).
        assert_eq!(out_bits, 60);
        // First 7 bytes match exactly; last byte's high bits match (low bits = padding).
        assert_eq!(&out_bytes[..7], &bytes[..7]);
        assert_eq!(out_bytes[7] & 0x80, bytes[7] & 0x80);
    }

    /// Critical: covers an N-byte chunk whose round-trip would mismatch under
    /// byte-aligned `bytes.len() * 8` accounting. N=3 is the smallest such case
    /// (encoder writes 8 bytes; symbol-aligned packing produces 13 symbols which
    /// unpack to 9 bytes — but symbol_aligned_bit_count = 65 stays the right
    /// reference).
    #[test]
    fn wrap_unwrap_n3_chunk_byte_count_recovers_correctly() {
        // Chunk-format wire: 37-bit header + 8*3 = 24-bit payload = 61 bits.
        let bit_count = 37 + 24;
        let mut w = BitWriter::new();
        w.write_bits(0x1FFF_FFFF_FFFF_u64, 37); // arbitrary header bits
        w.write_bits(0x00AA_BBCC_u64, 24);
        let bytes = w.into_bytes();
        assert_eq!(bytes.len(), 8); // ceil(61/8)
        let s = wrap_payload(&bytes, bit_count).unwrap();
        let (_out_bytes, out_bits) = unwrap_string(&s).unwrap();
        // Symbol-aligned bit count = 5 * ceil(61/5) = 5 * 13 = 65.
        assert_eq!(out_bits, 65);
        // (out_bits - 37) / 8 = (65 - 37) / 8 = 3 → 3 chunk-payload bytes recovered.
        let recovered_payload_byte_count = (out_bits - 37) / 8;
        assert_eq!(recovered_payload_byte_count, 3);
    }

    #[test]
    fn unwrap_rejects_non_md_string() {
        assert!(unwrap_string("xx1qpz9r4cy7").is_err());
    }

    #[test]
    fn unwrap_tolerates_visual_separators() {
        let mut w = BitWriter::new();
        w.write_bits(0b1010, 4);
        let bytes = w.into_bytes();
        let s = wrap_payload(&bytes, 4).unwrap();
        let mut grouped = String::new();
        for (i, c) in s.chars().enumerate() {
            grouped.push(c);
            if i == 3 {
                grouped.push('-');
            }
            if i == 8 {
                grouped.push(' ');
            }
        }
        let _ = unwrap_string(&grouped).unwrap();
    }

    // ── H6 (cycle-4): encode-side 80-data-symbol cap ─────────────────────────
    // The codex32 regular code is BCH(93, 80, 8): a single string carries at
    // most 80 data symbols + 13 checksum = 93. `wrap_payload` is the lowest
    // shared chokepoint; it MUST reject an over-80-data-symbol payload rather
    // than emit an un-decodable / aliasing-prone single string.

    #[test]
    fn wrap_payload_rejects_over_80_data_symbols() {
        // 405 bits → ceil(405/5) = 81 data symbols (one past the cap).
        let bit_count = 81 * 5;
        let mut w = BitWriter::new();
        // Fill with arbitrary non-zero bits, 32 at a time.
        let mut remaining = bit_count;
        while remaining > 0 {
            let take = remaining.min(32);
            w.write_bits(0xDEAD_BEEF_u64 & ((1u64 << take) - 1), take);
            remaining -= take;
        }
        let bytes = w.into_bytes();
        let got = wrap_payload(&bytes, bit_count);
        assert_eq!(
            got,
            Err(Error::PayloadTooLongForSingleString {
                data_symbols: 81,
                max: 80,
            }),
            "81 data symbols must be rejected with the typed cap error"
        );
    }

    #[test]
    fn wrap_payload_accepts_exactly_80_data_symbols() {
        // 400 bits → ceil(400/5) = 80 data symbols (the maximal LEGAL value).
        let bit_count = 80 * 5;
        let mut w = BitWriter::new();
        let mut remaining = bit_count;
        while remaining > 0 {
            let take = remaining.min(32);
            w.write_bits(0x1234_5678_u64 & ((1u64 << take) - 1), take);
            remaining -= take;
        }
        let bytes = w.into_bytes();
        let s = wrap_payload(&bytes, bit_count).expect("80 data symbols is in-domain");
        // HRP "md1" (3) + 80 data + 13 checksum = 96 chars (93-symbol codeword).
        assert_eq!(s.chars().count(), 3 + 80 + REGULAR_CHECKSUM_SYMBOLS);
    }

    // ── I1 (cycle-4, §5.2.3): non-correcting decode 93-symbol-codeword cap ────
    // `unwrap_string` (the `decode_md1_string` primitive) BCH-verifies via the
    // length-agnostic `bch_verify_regular` and only had a too-SHORT floor. A
    // CLEAN (residue==0, BCH-valid) over-93-symbol md1 must fail closed, not
    // decode an out-of-domain payload.

    /// Build a CLEAN (BCH-valid, residue==0) md1 string with `data_symbols`
    /// arbitrary data symbols, bypassing `wrap_payload`'s H6 cap by calling the
    /// raw BCH primitive directly. Used to forge over-93-codeword strings.
    fn clean_md1_with_data_symbols(data_symbols: usize) -> String {
        let data: Vec<u8> = (0..data_symbols).map(|i| (i as u8) & 0x1F).collect();
        let checksum = crate::bch::bch_create_checksum_regular(HRP, &data);
        let mut s = String::new();
        s.push_str(HRP);
        s.push('1');
        for &sym in data.iter().chain(checksum.iter()) {
            s.push(symbol_to_char(sym));
        }
        s
    }

    #[test]
    fn unwrap_string_rejects_clean_over_93_symbol_string() {
        // 90 data + 13 checksum = 103 codeword symbols (> 93), residue == 0.
        let s = clean_md1_with_data_symbols(90);
        let codeword_symbols = 90 + REGULAR_CHECKSUM_SYMBOLS;
        assert_eq!(codeword_symbols, 103);
        match crate::decode::decode_md1_string(&s) {
            Err(Error::StringSymbolCountOutOfRange { symbols, max }) => {
                assert_eq!(symbols, codeword_symbols);
                assert_eq!(max, 93);
            }
            other => panic!(
                "clean over-93-symbol string must be rejected with StringSymbolCountOutOfRange, got {other:?}"
            ),
        }
    }

    #[test]
    fn unwrap_string_accepts_exactly_93_symbol_codeword() {
        // 80 data + 13 checksum = 93 codeword symbols (the maximal legal value).
        let s = clean_md1_with_data_symbols(80);
        assert_eq!(s.chars().count(), 3 + 80 + REGULAR_CHECKSUM_SYMBOLS);
        let (_bytes, bit_count) =
            unwrap_string(&s).expect("a 93-symbol legal codeword must still decode");
        assert_eq!(bit_count, 5 * 80);
    }
}
