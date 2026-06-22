//! SeedQR encode/decode primitives (v0.30.0 / Cycle 5).
//!
//! SeedQR is an open spec originated by SeedSigner: BIP-39 mnemonic
//! encoded as a numeric-string QR payload where each English-wordlist
//! index is rendered as a 4-digit zero-padded decimal.
//!
//! ## Scope (v0.30.0)
//!
//! - Variants: Standard SeedQR only.
//! - Word counts: 12 + 24 only.
//! - Language: English only.
//!
//! ## Error pattern
//!
//! Library-local `SeedqrError` enum with hand-rolled `impl Display`
//! (mirrors `seed_xor.rs:31-67` precedent). CLI boundary in
//! `cmd/seedqr.rs` converts via `map_seedqr_error(e, action)`.

use bip39::{Language, Mnemonic};
use zeroize::Zeroizing;

/// Library-local error. Mapped to `ToolkitError::BadInput` at the CLI
/// boundary via `cmd::seedqr::map_seedqr_error`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedqrError {
    ChecksumFailure(String),
    /// v0.32.0 — CompactSeedQR decode: input is not valid hex.
    CompactInvalidHex(String),
    /// v0.32.0 — CompactSeedQR decode: hex decodes to a byte count other
    /// than 16 (12-word) or 32 (24-word). 20/24/28 (15/18/21-word) are
    /// valid BIP-39 entropy sizes but NOT compact-supported per SeedSigner.
    CompactByteCountUnsupported {
        got: usize,
    },
    /// v0.32.0 — CompactSeedQR encode: word count other than 12 or 24.
    /// SeedSigner's CompactSeedQrEncoder handles only those two.
    CompactWordCountUnsupported {
        got: usize,
    },
    InvalidDigitChar {
        pos: usize,
        ch: char,
    },
    InvalidDigits {
        got: usize,
    },
    InvalidWordCount {
        got: usize,
    },
    InvalidWordIndex {
        pos: usize,
        idx: u16,
    },
}

impl std::fmt::Display for SeedqrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeedqrError::ChecksumFailure(msg) => write!(f, "BIP-39 checksum failure: {msg}",),
            SeedqrError::CompactInvalidHex(msg) => {
                write!(f, "compact: invalid hex: {msg}")
            }
            SeedqrError::CompactByteCountUnsupported { got } => write!(
                f,
                "compact: invalid byte count (expected 16 for 12-word or 32 for 24-word; got {got})",
            ),
            SeedqrError::CompactWordCountUnsupported { got } => write!(
                f,
                "compact: invalid word count: {got} (CompactSeedQR supports only 12 or 24)",
            ),
            SeedqrError::InvalidDigitChar { pos, ch } => {
                write!(f, "invalid character at position {pos}: {ch:?}",)
            }
            SeedqrError::InvalidDigits { got } => {
                write!(
                    f,
                    "invalid digit count (expected 48, 60, 72, 84, or 96; got {got})",
                )
            }
            SeedqrError::InvalidWordCount { got } => {
                write!(
                    f,
                    "invalid word count: {got} (only 12, 15, 18, 21, or 24 supported)",
                )
            }
            SeedqrError::InvalidWordIndex { pos, idx } => write!(
                f,
                "invalid word index {idx} at position {pos} (must be 0..=2047)",
            ),
        }
    }
}

impl std::error::Error for SeedqrError {}

/// Decode a SeedQR numeric string into a BIP-39 phrase.
pub fn decode(input: &str) -> Result<String, SeedqrError> {
    // Strip all ASCII whitespace. cycle-15t — the raw SeedQR digit string is
    // secret scratch; wrap in Zeroizing so it scrubs on drop (M-2: the public
    // `String` return is KEPT; only internals wrap).
    let stripped: Zeroizing<String> =
        Zeroizing::new(input.chars().filter(|c| !c.is_ascii_whitespace()).collect());

    // Validate length. v0.31.5: widened from {48, 96} to the full BIP-39
    // word-count set {48, 60, 72, 84, 96} (= 12, 15, 18, 21, 24 words × 4
    // digits/word) per `seedqr-15-18-21-word-counts` FOLLOWUP closure.
    let len = stripped.len();
    if !matches!(len, 48 | 60 | 72 | 84 | 96) {
        return Err(SeedqrError::InvalidDigits { got: len });
    }

    // Validate all ASCII digits.
    for (pos, ch) in stripped.chars().enumerate() {
        if !ch.is_ascii_digit() {
            return Err(SeedqrError::InvalidDigitChar { pos, ch });
        }
    }

    // Chunk into 4-digit groups → word indices → words.
    let wordlist = Language::English.word_list();
    let mut words: Vec<&'static str> = Vec::with_capacity(len / 4);
    for (group, chunk) in stripped.as_bytes().chunks(4).enumerate() {
        // Invariant: chunk is 4 ASCII bytes per the prior digit-validation loop.
        let s = std::str::from_utf8(chunk).expect("ASCII digits");
        let idx: u16 = s.parse().expect("4 ASCII digits parse to u16");
        if idx as usize >= wordlist.len() {
            return Err(SeedqrError::InvalidWordIndex {
                pos: group * 4,
                idx,
            });
        }
        words.push(wordlist[idx as usize]);
    }

    let phrase = words.join(" ");

    // Checksum-validate via bip39 crate.
    Mnemonic::parse_in(Language::English, &phrase)
        .map_err(|e| SeedqrError::ChecksumFailure(e.to_string()))?;

    Ok(phrase)
}

/// Encode a BIP-39 phrase into a SeedQR numeric string.
pub fn encode(phrase: &str) -> Result<String, SeedqrError> {
    // Tokenize on whitespace, lowercase. cycle-15t — each word is a secret
    // BIP-39 token; wrap in Zeroizing so the per-word copies scrub on drop
    // (M-2: public `String` return is KEPT; only internal scratch wraps).
    let words: Vec<Zeroizing<String>> = phrase
        .split_whitespace()
        .map(|w| Zeroizing::new(w.to_lowercase()))
        .collect();

    // Validate word count. v0.31.5: widened from {12, 24} to the full
    // BIP-39 word-count set per `seedqr-15-18-21-word-counts` FOLLOWUP.
    if !matches!(words.len(), 12 | 15 | 18 | 21 | 24) {
        return Err(SeedqrError::InvalidWordCount { got: words.len() });
    }

    // Parse + checksum-validate via bip39 (also rejects invalid words).
    let normalized: Zeroizing<String> = Zeroizing::new(
        words
            .iter()
            .map(|w| w.as_str())
            .collect::<Vec<_>>()
            .join(" "),
    );
    Mnemonic::parse_in(Language::English, &*normalized)
        .map_err(|e| SeedqrError::ChecksumFailure(e.to_string()))?;

    // Map each word to its index via linear search. The SeedQR digit string
    // IS the secret return-carrier; build it in Zeroizing scratch.
    let wordlist = Language::English.word_list();
    let mut digits: Zeroizing<String> = Zeroizing::new(String::with_capacity(words.len() * 4));
    for word in &words {
        let idx = wordlist
            .iter()
            .position(|w| *w == word.as_str())
            .expect("bip39::Mnemonic::parse_in already validated word membership")
            as u16;
        digits.push_str(&format!("{idx:04}"));
    }

    // I-1 / Open-Q1 — the literal return value is necessarily a bare `String`;
    // the scratch buffer scrubs, this single return-move is the documented
    // small residue window.
    Ok((*digits).clone())
}

/// Encode a BIP-39 phrase into a CompactSeedQR payload as lowercase hex
/// (v0.32.0).
///
/// CompactSeedQR (SeedSigner `CompactSeedQrEncoder`) stores the raw BIP-39
/// entropy bytes in the QR's byte mode — i.e. the 11-bit word indices
/// packed with the trailing checksum bits stripped, which is exactly
/// `Mnemonic::to_entropy()`. The reference impl handles only 12-word
/// (16 bytes) and 24-word (32 bytes) seeds; other word counts are refused.
pub fn encode_compact(phrase: &str) -> Result<String, SeedqrError> {
    // cycle-15t — secret per-word + normalized-phrase scratch wraps in
    // Zeroizing (M-2: public `String` return is KEPT).
    let words: Vec<Zeroizing<String>> = phrase
        .split_whitespace()
        .map(|w| Zeroizing::new(w.to_lowercase()))
        .collect();

    if !matches!(words.len(), 12 | 24) {
        return Err(SeedqrError::CompactWordCountUnsupported { got: words.len() });
    }

    let normalized: Zeroizing<String> = Zeroizing::new(
        words
            .iter()
            .map(|w| w.as_str())
            .collect::<Vec<_>>()
            .join(" "),
    );
    let m = Mnemonic::parse_in(Language::English, &*normalized)
        .map_err(|e| SeedqrError::ChecksumFailure(e.to_string()))?;

    // The raw BIP-39 entropy is the highest-value secret here; wrap before
    // hex-encoding so the byte buffer scrubs on drop.
    let entropy: Zeroizing<Vec<u8>> = Zeroizing::new(m.to_entropy());
    Ok(hex::encode(&*entropy))
}

/// Decode a CompactSeedQR hex payload into a BIP-39 phrase (v0.32.0).
///
/// Strips ASCII whitespace, hex-decodes (case-insensitive), refuses byte
/// counts other than 16 (12-word) / 32 (24-word) — note 20/24/28 are valid
/// BIP-39 entropy sizes (15/18/21-word) but NOT compact-supported per
/// SeedSigner — then recomputes the BIP-39 checksum to produce the phrase.
pub fn decode_compact(input: &str) -> Result<String, SeedqrError> {
    // cycle-15t — raw hex-digit scratch + hex-decoded raw entropy wrap in
    // Zeroizing (the decoded `bytes` are the highest-value secret here).
    let stripped: Zeroizing<String> =
        Zeroizing::new(input.chars().filter(|c| !c.is_ascii_whitespace()).collect());

    let bytes: Zeroizing<Vec<u8>> = Zeroizing::new(
        hex::decode(&*stripped).map_err(|e| SeedqrError::CompactInvalidHex(e.to_string()))?,
    );

    if !matches!(bytes.len(), 16 | 32) {
        return Err(SeedqrError::CompactByteCountUnsupported { got: bytes.len() });
    }

    let m = Mnemonic::from_entropy_in(Language::English, &bytes[..])
        .map_err(|e| SeedqrError::ChecksumFailure(e.to_string()))?;

    Ok(m.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Canonical BIP-39 12-word test vector (Trezor): all-abandon-about.
    // "about" BIP-39 index 3 (zero-based; verified against English wordlist
    // file: line 4 = "about").
    const PHRASE_12: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const DIGITS_12: &str = "000000000000000000000000000000000000000000000003";

    // Canonical BIP-39 24-word test vector (Trezor): all-abandon-art.
    // "art" BIP-39 index 102 (zero-based; verified against English wordlist
    // file: line 103 = "art"). 92 zeros + "0102" = 96 digits.
    const PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    const DIGITS_24: &str = "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000102";

    // v0.31.5 canonical BIP-39 15-word zero-entropy vector. Derived
    // empirically via `mnemonic convert --from entropy=00..00 (20 bytes)
    // --to phrase`. Last word "address" = BIP-39 index 27.
    const PHRASE_15: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon address";
    const DIGITS_15: &str = "000000000000000000000000000000000000000000000000000000000027";

    // v0.31.5 canonical BIP-39 18-word zero-entropy vector. 24 bytes of
    // zeros → last word "agent" = BIP-39 index 39.
    const PHRASE_18: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon agent";
    const DIGITS_18: &str =
        "000000000000000000000000000000000000000000000000000000000000000000000039";

    // v0.31.5 canonical BIP-39 21-word zero-entropy vector. 28 bytes of
    // zeros → last word "admit" = BIP-39 index 29.
    const PHRASE_21: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon admit";
    const DIGITS_21: &str =
        "000000000000000000000000000000000000000000000000000000000000000000000000000000000029";

    // v0.32.0 CompactSeedQR vectors. Payload = raw BIP-39 entropy bytes
    // as hex. All-zero entropy: 12-word = 16 zero bytes (32 hex chars);
    // 24-word = 32 zero bytes (64 hex chars).
    const COMPACT_HEX_12: &str = "00000000000000000000000000000000";
    const COMPACT_HEX_24: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn encode_compact_12_word() {
        assert_eq!(encode_compact(PHRASE_12).unwrap(), COMPACT_HEX_12);
    }

    #[test]
    fn encode_compact_24_word() {
        assert_eq!(encode_compact(PHRASE_24).unwrap(), COMPACT_HEX_24);
    }

    #[test]
    fn decode_compact_12_word() {
        assert_eq!(decode_compact(COMPACT_HEX_12).unwrap(), PHRASE_12);
    }

    #[test]
    fn decode_compact_24_word() {
        assert_eq!(decode_compact(COMPACT_HEX_24).unwrap(), PHRASE_24);
    }

    #[test]
    fn round_trip_compact_12_word() {
        let hex = encode_compact(PHRASE_12).unwrap();
        assert_eq!(decode_compact(&hex).unwrap(), PHRASE_12);
    }

    #[test]
    fn round_trip_compact_24_word() {
        let hex = encode_compact(PHRASE_24).unwrap();
        assert_eq!(decode_compact(&hex).unwrap(), PHRASE_24);
    }

    #[test]
    fn encode_compact_rejects_15_word() {
        // 15-word is a valid BIP-39 + Standard-SeedQR shape, but
        // CompactSeedQR (SeedSigner) supports only 12/24.
        assert!(matches!(
            encode_compact(PHRASE_15),
            Err(SeedqrError::CompactWordCountUnsupported { got: 15 })
        ));
    }

    #[test]
    fn decode_compact_rejects_invalid_hex() {
        assert!(matches!(
            decode_compact("zz00"),
            Err(SeedqrError::CompactInvalidHex(_))
        ));
    }

    #[test]
    fn decode_compact_rejects_20_byte_count() {
        // 20 bytes = 15-word entropy size; valid BIP-39 entropy but NOT
        // compact-supported. Must be caught BEFORE from_entropy_in.
        let twenty_bytes_hex = "00".repeat(20);
        assert!(matches!(
            decode_compact(&twenty_bytes_hex),
            Err(SeedqrError::CompactByteCountUnsupported { got: 20 })
        ));
    }

    #[test]
    fn decode_compact_accepts_uppercase_and_whitespace() {
        // hex crate is case-insensitive; decode_compact strips ASCII whitespace.
        let upper_spaced = "AA BB CC DD EE FF 00 11 22 33 44 55 66 77 88 99";
        // 16 bytes → 12-word phrase (whatever it decodes to; just must succeed).
        let phrase = decode_compact(upper_spaced).unwrap();
        assert_eq!(phrase.split_whitespace().count(), 12);
    }

    #[test]
    fn decode_12_word_canonical() {
        assert_eq!(decode(DIGITS_12).unwrap(), PHRASE_12);
    }

    #[test]
    fn decode_24_word_canonical() {
        assert_eq!(decode(DIGITS_24).unwrap(), PHRASE_24);
    }

    #[test]
    fn encode_12_word_canonical() {
        assert_eq!(encode(PHRASE_12).unwrap(), DIGITS_12);
    }

    #[test]
    fn encode_24_word_canonical() {
        assert_eq!(encode(PHRASE_24).unwrap(), DIGITS_24);
    }

    #[test]
    fn round_trip_12_word() {
        let encoded = encode(PHRASE_12).unwrap();
        assert_eq!(decode(&encoded).unwrap(), PHRASE_12);
    }

    #[test]
    fn round_trip_24_word() {
        let encoded = encode(PHRASE_24).unwrap();
        assert_eq!(decode(&encoded).unwrap(), PHRASE_24);
    }

    // v0.31.5 — happy paths for 15/18/21-word BIP-39-valid phrases.

    #[test]
    fn decode_15_word_canonical() {
        assert_eq!(decode(DIGITS_15).unwrap(), PHRASE_15);
    }

    #[test]
    fn encode_15_word_canonical() {
        assert_eq!(encode(PHRASE_15).unwrap(), DIGITS_15);
    }

    #[test]
    fn round_trip_15_word() {
        let encoded = encode(PHRASE_15).unwrap();
        assert_eq!(decode(&encoded).unwrap(), PHRASE_15);
    }

    #[test]
    fn decode_18_word_canonical() {
        assert_eq!(decode(DIGITS_18).unwrap(), PHRASE_18);
    }

    #[test]
    fn encode_18_word_canonical() {
        assert_eq!(encode(PHRASE_18).unwrap(), DIGITS_18);
    }

    #[test]
    fn round_trip_18_word() {
        let encoded = encode(PHRASE_18).unwrap();
        assert_eq!(decode(&encoded).unwrap(), PHRASE_18);
    }

    #[test]
    fn decode_21_word_canonical() {
        assert_eq!(decode(DIGITS_21).unwrap(), PHRASE_21);
    }

    #[test]
    fn encode_21_word_canonical() {
        assert_eq!(encode(PHRASE_21).unwrap(), DIGITS_21);
    }

    #[test]
    fn round_trip_21_word() {
        let encoded = encode(PHRASE_21).unwrap();
        assert_eq!(decode(&encoded).unwrap(), PHRASE_21);
    }

    #[test]
    fn decode_strips_whitespace() {
        let padded = format!(" {DIGITS_12} \n\t");
        assert_eq!(decode(&padded).unwrap(), PHRASE_12);
    }

    #[test]
    fn decode_rejects_wrong_length_47() {
        let bad = &DIGITS_12[..47];
        assert!(matches!(
            decode(bad),
            Err(SeedqrError::InvalidDigits { got: 47 })
        ));
    }

    #[test]
    fn decode_rejects_wrong_length_49() {
        let bad = format!("{DIGITS_12}0");
        assert!(matches!(
            decode(&bad),
            Err(SeedqrError::InvalidDigits { got: 49 })
        ));
    }

    #[test]
    fn decode_rejects_wrong_length_95() {
        let bad = &DIGITS_24[..95];
        assert!(matches!(
            decode(bad),
            Err(SeedqrError::InvalidDigits { got: 95 })
        ));
    }

    #[test]
    fn decode_rejects_wrong_length_97() {
        let bad = format!("{DIGITS_24}0");
        assert!(matches!(
            decode(&bad),
            Err(SeedqrError::InvalidDigits { got: 97 })
        ));
    }

    #[test]
    fn decode_rejects_non_digit_char() {
        let bad = "00000000000000000000000000000000000000000000000A";
        assert!(matches!(
            decode(bad),
            Err(SeedqrError::InvalidDigitChar { pos: 47, ch: 'A' })
        ));
    }

    #[test]
    fn decode_rejects_word_index_out_of_range() {
        let bad = format!("9999{}", &DIGITS_12[4..]);
        assert!(matches!(
            decode(&bad),
            Err(SeedqrError::InvalidWordIndex { pos: 0, idx: 9999 })
        ));
    }

    #[test]
    fn decode_rejects_checksum_failure() {
        // 12 valid word indices but indices that don't checksum.
        let bad = "000100010001000100010001000100010001000100010001";
        assert!(matches!(decode(bad), Err(SeedqrError::ChecksumFailure(_))));
    }

    #[test]
    fn encode_rejects_13_word_count() {
        let bad = format!("{PHRASE_12} abandon");
        assert!(matches!(
            encode(&bad),
            Err(SeedqrError::InvalidWordCount { got: 13 })
        ));
    }

    // v0.31.5 — `encode_rejects_18_word_count` removed: the old cell
    // synthesized an 18-word string and asserted refusal. v0.31.5 widens
    // the gate to accept 12/15/18/21/24, so 18-word inputs now go
    // through the BIP-39-checksum-validation path. Replaced by:
    // - `encode_18_word_canonical` + `decode_18_word_canonical` +
    //   `round_trip_18_word` (happy-path coverage above).
    // - `encode_rejects_22_word_count` below — exercises a new
    //   not-in-{12,15,18,21,24} boundary value.

    #[test]
    fn encode_rejects_22_word_count() {
        // v0.31.5 — 22 words is between two valid sizes (21 and 24) so
        // it lands in the new gate's refusal arm. Confirms the
        // gate-widening did not silently accept arbitrary word counts.
        let bad = "abandon ".repeat(21) + "about";
        assert!(matches!(
            encode(&bad),
            Err(SeedqrError::InvalidWordCount { got: 22 })
        ));
    }

    #[test]
    fn encode_rejects_invalid_word() {
        let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon notaword";
        // bip39::Mnemonic::parse_in's invalid-word error collapses into
        // SeedqrError::ChecksumFailure (with the underlying diagnostic
        // preserved).
        assert!(matches!(encode(bad), Err(SeedqrError::ChecksumFailure(_))));
    }

    #[test]
    fn encode_rejects_checksum_failure() {
        let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
        assert!(matches!(encode(bad), Err(SeedqrError::ChecksumFailure(_))));
    }

    // ========================================================================
    // cycle-15t Slug-3 — seedqr internal-scratch zeroize (M-2: public `String`
    // returns are KEPT; only the internal scratch wraps in Zeroizing).
    // ========================================================================

    /// T7 — public-API-stability fence (stays GREEN): the four `pub fn` STILL
    /// return `Result<String, SeedqrError>` (we did NOT widen the public
    /// return → no SemVer break). A widened return fails the fn-pointer
    /// coercion at compile time.
    #[test]
    fn t7_public_returns_stay_string() {
        let _d: fn(&str) -> Result<String, SeedqrError> = decode;
        let _e: fn(&str) -> Result<String, SeedqrError> = encode;
        let _ec: fn(&str) -> Result<String, SeedqrError> = encode_compact;
        let _dc: fn(&str) -> Result<String, SeedqrError> = decode_compact;
    }

    /// T8 — internal-scratch evidence (RED until the wraps land): the secret
    /// scratch in `seedqr.rs` is `Zeroizing`-wrapped — the `decode`/`decode_compact`
    /// raw-digit `stripped`, the per-word `words`/`normalized`/`digits`, and the
    /// `decode_compact` hex-decoded `bytes` (highest-value: raw entropy). The
    /// anchors are assembled at runtime so this test's own source does NOT
    /// self-match the `src.contains(...)` checks.
    #[test]
    fn t8_internal_scratch_is_zeroizing() {
        let src = std::fs::read_to_string("src/seedqr.rs").expect("read src/seedqr.rs");
        let zs = format!("Zeroizing<{}>", "String");
        let zv = format!("Zeroizing<{}>", "Vec<u8>");
        // raw-digit scratch (decode + decode_compact).
        assert!(
            src.contains(&format!("let stripped: {zs}")),
            "T8: `stripped` raw-digit scratch must be Zeroizing<String>"
        );
        // per-word vecs (encode + encode_compact).
        assert!(
            src.contains(&format!("let words: Vec<{zs}>")),
            "T8: per-word `words` must be Vec<Zeroizing<String>>"
        );
        // normalized join (encode + encode_compact).
        assert!(
            src.contains(&format!("let normalized: {zs}")),
            "T8: `normalized` must be Zeroizing<String>"
        );
        // SeedQR digit secret return-carrier (encode).
        assert!(
            src.contains(&format!("let mut digits: {zs}")),
            "T8: `digits` must be Zeroizing<String>"
        );
        // decode_compact hex-decoded raw entropy (highest-value scratch).
        assert!(
            src.contains(&format!("let bytes: {zv}")),
            "T8: decode_compact `bytes` must be Zeroizing<Vec<u8>>"
        );
    }
}
