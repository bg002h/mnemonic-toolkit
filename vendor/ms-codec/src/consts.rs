//! v0.1 wire-format constants.
//!
//! **Naming convention:** ASCII byte literals (`b'0'`, `b's'`) are used for
//! values whose semantic meaning is the *character* on the wire (threshold
//! digit, share-index letter); hex literals (`0x00`) are used for values
//! whose semantic meaning is the *byte* on the wire (the reserved-prefix
//! byte). Both produce `u8`; the form chosen reflects which mental model
//! is more natural at the use site.

/// HRP for ms1 strings (BIP-93 codex32 HRP).
pub const HRP: &str = "ms";

/// BIP-93 separator character.
pub const SEPARATOR: char = '1';

/// v0.1 reserved-prefix byte (becomes the v0.2 type discriminator).
pub const RESERVED_PREFIX: u8 = 0x00;

/// v0.1 emit-side threshold value (ASCII).
pub const THRESHOLD_V01: u8 = b'0';

/// v0.1 emit-side share-index value (ASCII; "s" denotes the unshared secret per BIP-93).
pub const SHARE_INDEX_V01: u8 = b's';

/// Short codex32 checksum length in characters.
pub const CHECKSUM_LEN_SHORT: usize = 13;

/// Allowed v0.1 entr entropy byte lengths (bijective with BIP-39 word counts {12,15,18,21,24}).
pub const VALID_ENTR_LENGTHS: &[usize] = &[16, 20, 24, 28, 32];

/// Allowed v0.1 total ms1 string lengths (HRP+sep+threshold+id+share+payload+cksum).
/// Computed: 9 fixed + ceil((entropy_bytes + 1) * 8 / 5) payload symbols + 13 cksum.
pub const VALID_STR_LENGTHS: &[usize] = &[50, 56, 62, 69, 75];

/// 4-byte type tag — v0.1 emit (also accept).
pub const TAG_ENTR: [u8; 4] = *b"entr";

/// v0.2 mnem-prefix byte (type discriminator for Mnem payloads).
pub const MNEM_PREFIX: u8 = 0x02;

/// Allowed v0.2 mnem total ms1 string lengths (byte-aligned: prefix + lang + entropy).
/// Computed: 9 fixed + ceil((entropy_bytes + 2) * 8 / 5) payload symbols + 13 cksum.
pub const VALID_MNEM_STR_LENGTHS: &[usize] = &[51, 58, 64, 70, 77];

/// BIP-39 wordlist language names indexed by language byte (0 = English).
/// This order MUST match ms-cli's `CliLanguage` declaration order (Phase 2 depends on it).
pub const MNEM_LANGUAGE_NAMES: [&str; 10] = [
    "english",
    "japanese",
    "korean",
    "spanish",
    "chinese-simplified",
    "chinese-traditional",
    "french",
    "italian",
    "czech",
    "portuguese",
];

/// 4-byte type tags reserved-not-emitted in v0.1 (decoder rejects).
/// `mnem` is no longer reserved-not-emitted: it is emitted in v0.2+ as Payload::Mnem.
pub const RESERVED_NOT_EMITTED_V01: &[[u8; 4]] = &[*b"seed", *b"xprv", *b"prvk"];

/// Anti-collision blocklist for the random 4-char `id` of a v0.2 K-of-N
/// share-set (SPEC_ms_v0_2_kofn §2 consts / design-review I4). A share-set's
/// `id` is random-per-set; re-roll while it lands in this set so a share-set
/// `id` never collides with a v0.1 type-tag-shaped value.
///
/// **DISTINCT from `RESERVED_NOT_EMITTED_V01`** (the decoder-reject set, which
/// dropped `mnem` in Cycle 1): `mnem` MUST stay in this id-blocklist.
pub const RESERVED_ID_BLOCKLIST: &[[u8; 4]] = &[*b"entr", *b"seed", *b"xprv", *b"mnem", *b"prvk"];

#[cfg(test)]
mod tests {
    use super::*;

    /// Locks the bijection between VALID_ENTR_LENGTHS and VALID_STR_LENGTHS so
    /// that a future edit to one without the other fails CI loudly.
    /// Formula per SPEC §2.4: total = 9 fixed (HRP+sep+threshold+id+share) +
    /// ceil((entropy_bytes + 1) * 8 / 5) payload symbols + 13 short checksum.
    #[test]
    fn valid_str_lengths_match_entr_lengths_via_bijection() {
        assert_eq!(VALID_ENTR_LENGTHS.len(), VALID_STR_LENGTHS.len());
        for (i, &entropy_bytes) in VALID_ENTR_LENGTHS.iter().enumerate() {
            let data_bits = (entropy_bytes + 1) * 8; // +1 for the 0x00 prefix byte
            let payload_symbols = data_bits.div_ceil(5);
            let total = 9 + payload_symbols + CHECKSUM_LEN_SHORT;
            assert_eq!(
                total, VALID_STR_LENGTHS[i],
                "entropy {} B -> expected str.len {}, got {} (bijection drift)",
                entropy_bytes, VALID_STR_LENGTHS[i], total
            );
        }
    }

    /// Locks the bijection between VALID_ENTR_LENGTHS and VALID_MNEM_STR_LENGTHS.
    /// Mnem payload = [0x02 prefix] + [lang byte] + entropy = entropy_bytes + 2 bytes.
    /// Formula: total = 9 fixed + ceil((entropy_bytes + 2) * 8 / 5) payload symbols
    /// + 13 short checksum.
    #[test]
    fn valid_mnem_str_lengths_match_entr_lengths_via_bijection() {
        assert_eq!(VALID_ENTR_LENGTHS.len(), VALID_MNEM_STR_LENGTHS.len());
        for (i, &entropy_bytes) in VALID_ENTR_LENGTHS.iter().enumerate() {
            let data_bits = (entropy_bytes + 2) * 8; // +2 for 0x02 prefix + lang byte
            let payload_symbols = data_bits.div_ceil(5);
            let total = 9 + payload_symbols + CHECKSUM_LEN_SHORT;
            assert_eq!(
                total, VALID_MNEM_STR_LENGTHS[i],
                "entropy {} B -> expected mnem str.len {}, got {} (bijection drift)",
                entropy_bytes, VALID_MNEM_STR_LENGTHS[i], total
            );
        }
    }
}
