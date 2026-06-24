//! BIP-39 phrase rendering per spec §8.4.

use crate::error::Error;

/// A 12-word BIP-39 phrase rendering of a 128-bit identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phrase(
    /// The 12 BIP-39 words (English wordlist) derived from the 128-bit input.
    pub [String; 12],
);

impl Phrase {
    /// Render a 16-byte (128-bit) identity as a 12-word BIP-39 phrase.
    ///
    /// 128 bits of entropy is always a valid BIP-39 input, so the underlying
    /// `Mnemonic::from_entropy` cannot fail for this length.
    pub fn from_id_bytes(id: &[u8; 16]) -> Result<Self, Error> {
        let mnemonic = bip39::Mnemonic::from_entropy(id)
            .expect("128-bit entropy is always a valid BIP-39 input");
        let mut words: [String; 12] = Default::default();
        for (slot, word) in words.iter_mut().zip(mnemonic.words()) {
            *slot = word.to_string();
        }
        Ok(Phrase(words))
    }
}

impl std::fmt::Display for Phrase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phrase_deterministic() {
        let id = [0xab; 16];
        let p1 = Phrase::from_id_bytes(&id).unwrap();
        let p2 = Phrase::from_id_bytes(&id).unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn phrase_has_12_words() {
        let id = [0u8; 16];
        let p = Phrase::from_id_bytes(&id).unwrap();
        assert_eq!(p.0.len(), 12);
        for word in &p.0 {
            assert!(!word.is_empty());
        }
    }

    #[test]
    fn phrase_to_string_is_space_separated() {
        let id = [0u8; 16];
        let p = Phrase::from_id_bytes(&id).unwrap();
        let s = p.to_string();
        assert_eq!(s.split(' ').count(), 12);
    }
}
