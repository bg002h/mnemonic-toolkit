//! SPEC §10.2 BIP-39 round-trip integration: take an English BIP-39 mnemonic,
//! extract entropy, encode as ms1 entr, decode, re-derive the mnemonic,
//! confirm string-exact match. Catches any entropy-bit-misalignment regression.

use bip39::{Language, Mnemonic};
use ms_codec::{decode, encode, Payload, Tag};

#[test]
fn bip39_12_word_round_trip_english() {
    let phrase = "abandon abandon abandon abandon abandon abandon \
                  abandon abandon abandon abandon abandon about";
    let mnemonic = Mnemonic::parse_in(Language::English, phrase).unwrap();
    let entropy = mnemonic.to_entropy();
    assert_eq!(entropy.len(), 16, "12 words = 128 bits = 16 bytes");

    let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
    assert_eq!(s.len(), 50, "12-word entr = 50-char ms1 string");

    let (tag, recovered_payload) = decode(&s).unwrap();
    assert_eq!(tag, Tag::ENTR);
    let Payload::Entr(recovered_entropy) = recovered_payload else {
        panic!("expected Payload::Entr after decode");
    };
    assert_eq!(recovered_entropy, entropy);

    let recovered_mnemonic =
        Mnemonic::from_entropy_in(Language::English, &recovered_entropy).unwrap();
    assert_eq!(recovered_mnemonic.to_string(), phrase);
}

#[test]
fn bip39_24_word_round_trip_english() {
    let phrase = "abandon abandon abandon abandon abandon abandon \
                  abandon abandon abandon abandon abandon abandon \
                  abandon abandon abandon abandon abandon abandon \
                  abandon abandon abandon abandon abandon art";
    let mnemonic = Mnemonic::parse_in(Language::English, phrase).unwrap();
    let entropy = mnemonic.to_entropy();
    assert_eq!(entropy.len(), 32);

    let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
    assert_eq!(s.len(), 75);

    let (_tag, recovered_payload) = decode(&s).unwrap();
    let Payload::Entr(recovered_entropy) = recovered_payload else {
        panic!("expected Payload::Entr after decode");
    };
    assert_eq!(recovered_entropy, entropy);

    let recovered_mnemonic =
        Mnemonic::from_entropy_in(Language::English, &recovered_entropy).unwrap();
    assert_eq!(recovered_mnemonic.to_string(), phrase);
}

#[test]
fn bip39_random_entropy_round_trips_at_all_word_counts() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Deterministic pseudo-random entropy from a fixed seed (no rand dep needed).
    fn det_bytes(seed: u64, len: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(len);
        let mut h = seed;
        while out.len() < len {
            let mut hasher = DefaultHasher::new();
            h.hash(&mut hasher);
            let v = hasher.finish().to_le_bytes();
            out.extend_from_slice(&v);
            h = h.wrapping_add(0x9E3779B97F4A7C15);
        }
        out.truncate(len);
        out
    }

    for (word_count, byte_len) in [(12usize, 16usize), (15, 20), (18, 24), (21, 28), (24, 32)] {
        let entropy = det_bytes(0xDEADBEEF + word_count as u64, byte_len);
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy).unwrap();
        let original_phrase = mnemonic.to_string();
        assert_eq!(original_phrase.split_whitespace().count(), word_count);

        let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
        let (_tag, recovered_payload) = decode(&s).unwrap();
        let Payload::Entr(recovered_entropy) = recovered_payload else {
            panic!("expected Payload::Entr after decode");
        };
        assert_eq!(recovered_entropy, entropy);

        let recovered_mnemonic =
            Mnemonic::from_entropy_in(Language::English, &recovered_entropy).unwrap();
        assert_eq!(recovered_mnemonic.to_string(), original_phrase);
    }
}
