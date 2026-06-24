//! Tag type — 4-byte codex32-alphabet validated type tag.

use crate::consts::TAG_ENTR;
use crate::error::{Error, Result};

/// codex32 alphabet (BIP-173 lowercase bech32 charset).
const CODEX32_ALPHABET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// 4-byte type tag. Field is private to enforce validated construction via
/// `try_new` (alphabet-checked) or `from_raw_bytes` (tooling-only, unvalidated).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tag([u8; 4]);

impl Tag {
    /// The v0.1 emit-tag for BIP-39 entropy.
    pub const ENTR: Tag = Tag(TAG_ENTR);

    /// Construct a Tag from raw 4-byte input WITHOUT alphabet validation.
    /// Reserved for tooling (e.g., `inspect()`) that needs to surface whatever
    /// bytes were observed on the wire, including alphabet violators. Encoder
    /// + decoder paths MUST go through `try_new` instead.
    pub fn from_raw_bytes(b: [u8; 4]) -> Self {
        Tag(b)
    }

    /// Construct a Tag from a 4-character string slice. Returns
    /// `Error::TagInvalidAlphabet` if any character is outside the codex32 alphabet.
    pub fn try_new(s: &str) -> Result<Self> {
        let bytes = s.as_bytes();
        if bytes.len() != 4 {
            // Length mismatch: the partial-input bytes carry no useful diagnostic
            // information (the tag wasn't even the right shape). Return an empty
            // 4-byte sentinel to keep the error variant payload simple.
            return Err(Error::TagInvalidAlphabet { got: [0; 4] });
        }
        let mut out = [0u8; 4];
        for (i, b) in bytes.iter().enumerate() {
            if !CODEX32_ALPHABET.contains(b) {
                return Err(Error::TagInvalidAlphabet {
                    got: [bytes[0], bytes[1], bytes[2], bytes[3]],
                });
            }
            out[i] = *b;
        }
        Ok(Tag(out))
    }

    /// Borrow the underlying 4 bytes.
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }

    /// View the tag as a string slice. Always succeeds for `try_new`-constructed
    /// tags (codex32 alphabet is ASCII); for `from_raw_bytes`-constructed tags
    /// containing non-UTF-8 bytes, returns "<non-utf8>".
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_or("<non-utf8>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entr_const_matches_string() {
        assert_eq!(Tag::ENTR.as_str(), "entr");
    }

    #[test]
    fn try_new_accepts_alphabet_chars() {
        // All four lowercase reserved tags should parse.
        for s in ["entr", "seed", "xprv", "mnem", "prvk"] {
            let t = Tag::try_new(s).expect(s);
            assert_eq!(t.as_str(), s);
        }
    }

    #[test]
    fn try_new_rejects_uppercase() {
        // codex32 alphabet is lowercase; uppercase bytes are rejected.
        assert!(matches!(
            Tag::try_new("ENTR"),
            Err(Error::TagInvalidAlphabet { .. })
        ));
    }

    #[test]
    fn try_new_rejects_out_of_alphabet_chars() {
        // 'b' and 'i' and 'o' are NOT in the codex32 alphabet (excluded for OCR safety).
        for s in ["beer", "iron", "oboe"] {
            assert!(
                matches!(Tag::try_new(s), Err(Error::TagInvalidAlphabet { .. })),
                "expected reject for {:?}",
                s
            );
        }
    }

    #[test]
    fn try_new_rejects_wrong_length() {
        for s in ["", "a", "ab", "abc", "abcde"] {
            assert!(
                matches!(Tag::try_new(s), Err(Error::TagInvalidAlphabet { .. })),
                "expected reject for {:?}",
                s
            );
        }
    }

    #[test]
    fn from_raw_bytes_skips_validation() {
        // Tooling-only construction path; uppercase bytes preserved.
        let t = Tag::from_raw_bytes(*b"ENTR");
        assert_eq!(t.as_bytes(), b"ENTR");
    }
}
