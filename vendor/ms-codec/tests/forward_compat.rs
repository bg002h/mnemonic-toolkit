//! SPEC §10.2 forward-compat smoke tests: prefix-byte dispatch contract.
//! 0x00 → Entr, 0x02 → Mnem (v0.2); all other non-zero prefixes →
//! Error::ReservedPrefixViolation.

use ms_codec::codex32::{Codex32String, Fe};
use ms_codec::{decode, encode, Error, Payload, PayloadKind, Tag};

#[test]
fn flipping_prefix_byte_to_0x01_rejects_with_reserved_prefix_violation() {
    // Encode a real v0.1 string.
    let entropy = vec![0xAAu8; 16];
    let _s_v01 = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();

    // Hand-build the same wire shape but with prefix byte = 0x01 (undefined).
    // Decoder MUST reject with ReservedPrefixViolation.
    let mut data = vec![0x01u8];
    data.extend_from_slice(&entropy);
    let c = Codex32String::from_seed("ms", 0, "entr", Fe::S, &data).unwrap();
    let s = c.to_string();

    assert_eq!(s.len(), 50);
    assert!(matches!(
        decode(&s),
        Err(Error::ReservedPrefixViolation { got: 0x01 })
    ));
}

#[test]
fn prefix_0x02_with_valid_mnem_payload_is_accepted() {
    // 0x02 is the v0.2 mnem-prefix; decoder must accept it and return Payload::Mnem.
    let entropy = vec![0xAAu8; 16];
    let p = Payload::Mnem {
        language: 0,
        entropy: entropy.clone(),
    };
    let s = encode(Tag::ENTR, &p).unwrap();
    assert_eq!(s.len(), 51, "mnem 16-byte → 51-char ms1");
    let (tag, recovered) = decode(&s).unwrap();
    assert_eq!(tag, Tag::ENTR);
    assert_eq!(recovered.kind(), PayloadKind::Mnem);
}

#[test]
fn all_undefined_prefix_bytes_rejected() {
    // Defense-in-depth: every prefix value that is neither 0x00 (Entr) nor
    // 0x02 (Mnem) must be rejected with ReservedPrefixViolation.
    let entropy = [0xAAu8; 16];
    for prefix in 1u8..=255 {
        if prefix == 0x02 {
            // 0x02 is now the mnem prefix — not a reserved violation.
            continue;
        }
        let mut data = vec![prefix];
        data.extend_from_slice(&entropy);
        let c = Codex32String::from_seed("ms", 0, "entr", Fe::S, &data).unwrap();
        let err = decode(&c.to_string()).unwrap_err();
        assert!(
            matches!(err, Error::ReservedPrefixViolation { got } if got == prefix),
            "prefix 0x{:02x}: expected ReservedPrefixViolation, got {:?}",
            prefix,
            err
        );
    }
}
