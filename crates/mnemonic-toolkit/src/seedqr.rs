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

/// Library-local error. Mapped to `ToolkitError::BadInput` at the CLI
/// boundary via `cmd::seedqr::map_seedqr_error`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedqrError {
    InvalidDigits { got: usize },
    InvalidDigitChar { pos: usize, ch: char },
    InvalidWordIndex { pos: usize, idx: u16 },
    InvalidWordCount { got: usize },
    ChecksumFailure(String),
}

impl std::fmt::Display for SeedqrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeedqrError::InvalidDigits { got } => write!(
                f,
                "invalid digit count (expected 48 or 96; got {got})",
            ),
            SeedqrError::InvalidDigitChar { pos, ch } => write!(
                f,
                "invalid character at position {pos}: {ch:?}",
            ),
            SeedqrError::InvalidWordIndex { pos, idx } => write!(
                f,
                "invalid word index {idx} at position {pos} (must be 0..=2047)",
            ),
            SeedqrError::InvalidWordCount { got } => write!(
                f,
                "invalid word count: {got} (only 12 or 24 supported)",
            ),
            SeedqrError::ChecksumFailure(msg) => write!(
                f,
                "BIP-39 checksum failure: {msg}",
            ),
        }
    }
}

impl std::error::Error for SeedqrError {}

/// Decode a SeedQR numeric string into a BIP-39 phrase.
pub fn decode(input: &str) -> Result<String, SeedqrError> {
    // Strip all ASCII whitespace.
    let stripped: String = input.chars().filter(|c| !c.is_ascii_whitespace()).collect();

    // Validate length.
    let len = stripped.len();
    if len != 48 && len != 96 {
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
        // SAFETY: chunk is 4 ASCII bytes per prior digit-validation loop.
        let s = std::str::from_utf8(chunk).expect("ASCII digits");
        let idx: u16 = s.parse().expect("4 ASCII digits parse to u16");
        if idx as usize >= wordlist.len() {
            return Err(SeedqrError::InvalidWordIndex { pos: group * 4, idx });
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
    // Tokenize on whitespace, lowercase.
    let words: Vec<String> = phrase
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .collect();

    // Validate word count.
    if words.len() != 12 && words.len() != 24 {
        return Err(SeedqrError::InvalidWordCount { got: words.len() });
    }

    // Parse + checksum-validate via bip39 (also rejects invalid words).
    let normalized = words.join(" ");
    Mnemonic::parse_in(Language::English, &normalized)
        .map_err(|e| SeedqrError::ChecksumFailure(e.to_string()))?;

    // Map each word to its index via linear search.
    let wordlist = Language::English.word_list();
    let mut digits = String::with_capacity(words.len() * 4);
    for word in &words {
        let idx = wordlist
            .iter()
            .position(|w| *w == word.as_str())
            .expect("bip39::Mnemonic::parse_in already validated word membership") as u16;
        digits.push_str(&format!("{idx:04}"));
    }

    Ok(digits)
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

    #[test]
    fn decode_strips_whitespace() {
        let padded = format!(" {DIGITS_12} \n\t");
        assert_eq!(decode(&padded).unwrap(), PHRASE_12);
    }

    #[test]
    fn decode_rejects_wrong_length_47() {
        let bad = &DIGITS_12[..47];
        assert!(matches!(decode(bad), Err(SeedqrError::InvalidDigits { got: 47 })));
    }

    #[test]
    fn decode_rejects_wrong_length_49() {
        let bad = format!("{DIGITS_12}0");
        assert!(matches!(decode(&bad), Err(SeedqrError::InvalidDigits { got: 49 })));
    }

    #[test]
    fn decode_rejects_wrong_length_95() {
        let bad = &DIGITS_24[..95];
        assert!(matches!(decode(bad), Err(SeedqrError::InvalidDigits { got: 95 })));
    }

    #[test]
    fn decode_rejects_wrong_length_97() {
        let bad = format!("{DIGITS_24}0");
        assert!(matches!(decode(&bad), Err(SeedqrError::InvalidDigits { got: 97 })));
    }

    #[test]
    fn decode_rejects_non_digit_char() {
        let bad = "00000000000000000000000000000000000000000000000A";
        assert!(matches!(decode(bad), Err(SeedqrError::InvalidDigitChar { pos: 47, ch: 'A' })));
    }

    #[test]
    fn decode_rejects_word_index_out_of_range() {
        let bad = format!("9999{}", &DIGITS_12[4..]);
        assert!(matches!(decode(&bad), Err(SeedqrError::InvalidWordIndex { pos: 0, idx: 9999 })));
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
        assert!(matches!(encode(&bad), Err(SeedqrError::InvalidWordCount { got: 13 })));
    }

    #[test]
    fn encode_rejects_18_word_count() {
        let bad = "abandon ".repeat(17) + "about";
        assert!(matches!(encode(&bad), Err(SeedqrError::InvalidWordCount { got: 18 })));
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
}
