//! Payload type — v0.2: Entr (BIP-39 entropy) and Mnem (BIP-39 mnemonic with language).

use crate::consts::VALID_ENTR_LENGTHS;
use crate::error::{Error, Result};
use crate::tag::Tag;

/// v0.2 payload kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PayloadKind {
    /// BIP-39 entropy (16/20/24/28/32 B).
    Entr,
    /// BIP-39 mnemonic entropy with wordlist language tag (16/20/24/28/32 B entropy).
    Mnem,
}

/// v0.1 payload.
///
/// **Caller-wrap contract (SPEC v0.9.0 §1 item 2):** the `Vec<u8>` inside
/// `Payload::Entr` is NOT zeroize-wrapped — widening the public type to
/// `Zeroizing<Vec<u8>>` is a breaking change deferred indefinitely per
/// SPEC §3 OOS-2. Callers MUST wrap the byte buffer at the use site
/// (e.g., `let bytes = Zeroizing::new((*p.as_bytes()).to_vec());`)
/// so that the secret-material lifetime ends with a scrubbed drop.
/// ms-codec internally minimizes the un-scrubbed lifetime: encode + decode
/// path locals are `Zeroizing<Vec<u8>>`; only the public `Payload::Entr`
/// boundary is unwrapped.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Payload {
    /// BIP-39 entropy. Length MUST be in {16, 20, 24, 28, 32} bytes
    /// (bijective with BIP-39 word counts {12, 15, 18, 21, 24}).
    ///
    /// **Caller responsibility:** ms-codec does NOT check the statistical
    /// quality of these bytes. Callers are responsible for sourcing entropy
    /// from a vetted CSPRNG, or from a BIP-39 mnemonic the user already trusts.
    /// FIPS-style entropy-quality checks would slow encoding and provide false
    /// assurance — they cannot detect attacker-supplied "pseudo-random" seeds
    /// crafted to pass standard randomness tests. See SPEC §3.6.
    ///
    /// **Caller-wrap reminder:** wrap this `Vec<u8>` in `Zeroizing` at the
    /// use site so it scrubs on drop. ms-codec cannot wrap this for you
    /// without a breaking public-API change.
    Entr(Vec<u8>),
    /// BIP-39 mnemonic entropy with wordlist language tag. On-wire payload:
    /// `[0x02][language_byte][entropy:N]` where `language_byte` indexes into
    /// `consts::MNEM_LANGUAGE_NAMES` (0 = English, 1 = Japanese, …, 9 = Portuguese).
    /// Entropy length MUST be in {16, 20, 24, 28, 32} bytes.
    ///
    /// **Caller-wrap reminder:** wrap `entropy` in `Zeroizing` at the use site.
    Mnem {
        /// BIP-39 wordlist language index (0..=9).
        language: u8,
        /// BIP-39 entropy bytes (16/20/24/28/32 B).
        entropy: Vec<u8>,
    },
}

impl Payload {
    /// Validate the payload's intrinsic structure (byte length for Entr/Mnem;
    /// language code range for Mnem).
    /// Encoder MUST call this before emitting; decoder calls it after extracting
    /// the payload bytes following the prefix byte.
    pub fn validate(&self) -> Result<()> {
        match self {
            Payload::Entr(data) => {
                if !VALID_ENTR_LENGTHS.contains(&data.len()) {
                    return Err(Error::PayloadLengthMismatch {
                        tag: *Tag::ENTR.as_bytes(),
                        expected: VALID_ENTR_LENGTHS,
                        got: data.len(),
                    });
                }
                Ok(())
            }
            Payload::Mnem { language, entropy } => {
                if *language >= 10 {
                    return Err(Error::MnemUnknownLanguage(*language));
                }
                if !VALID_ENTR_LENGTHS.contains(&entropy.len()) {
                    return Err(Error::PayloadLengthMismatch {
                        tag: *Tag::ENTR.as_bytes(),
                        expected: VALID_ENTR_LENGTHS,
                        got: entropy.len(),
                    });
                }
                Ok(())
            }
        }
    }

    /// The PayloadKind discriminant.
    pub fn kind(&self) -> PayloadKind {
        match self {
            Payload::Entr(_) => PayloadKind::Entr,
            Payload::Mnem { .. } => PayloadKind::Mnem,
        }
    }

    /// Borrow the inner entropy byte slice.
    /// For `Payload::Mnem`, returns the entropy bytes only (without prefix or language byte).
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Payload::Entr(data) => data,
            Payload::Mnem { entropy, .. } => entropy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Mnem failing tests (written before impl per TDD) ---

    #[test]
    fn mnem_valid_language_and_entropy_accepts() {
        let p = Payload::Mnem {
            language: 1,
            entropy: vec![0u8; 16],
        };
        assert!(matches!(p.validate(), Ok(())));
    }

    #[test]
    fn mnem_language_10_rejects() {
        let p = Payload::Mnem {
            language: 10,
            entropy: vec![0u8; 16],
        };
        assert!(matches!(p.validate(), Err(Error::MnemUnknownLanguage(10))));
    }

    #[test]
    fn mnem_language_0x10_rejects() {
        let p = Payload::Mnem {
            language: 0x10,
            entropy: vec![0u8; 16],
        };
        assert!(matches!(
            p.validate(),
            Err(Error::MnemUnknownLanguage(0x10))
        ));
    }

    #[test]
    fn mnem_bad_entropy_length_rejects() {
        let p = Payload::Mnem {
            language: 0,
            entropy: vec![0u8; 17],
        };
        assert!(matches!(
            p.validate(),
            Err(Error::PayloadLengthMismatch { .. })
        ));
    }

    #[test]
    fn mnem_kind_returns_mnem() {
        let p = Payload::Mnem {
            language: 0,
            entropy: vec![0u8; 16],
        };
        assert_eq!(p.kind(), PayloadKind::Mnem);
    }

    // --- Entr tests (pre-existing) ---

    #[test]
    fn entr_accepts_all_bip39_lengths() {
        for len in [16usize, 20, 24, 28, 32] {
            let p = Payload::Entr(vec![0u8; len]);
            p.validate()
                .unwrap_or_else(|e| panic!("expected ok for len {}, got {:?}", len, e));
        }
    }

    #[test]
    fn entr_rejects_off_by_one_lengths() {
        for len in [15usize, 17, 19, 21, 23, 25, 31, 33] {
            let p = Payload::Entr(vec![0u8; len]);
            assert!(
                matches!(p.validate(), Err(Error::PayloadLengthMismatch { .. })),
                "expected reject for len {}",
                len
            );
        }
    }

    #[test]
    fn entr_rejects_zero_length() {
        let p = Payload::Entr(vec![]);
        assert!(matches!(
            p.validate(),
            Err(Error::PayloadLengthMismatch { .. })
        ));
    }

    #[test]
    fn kind_returns_entr() {
        assert_eq!(Payload::Entr(vec![0u8; 16]).kind(), PayloadKind::Entr);
    }
}
