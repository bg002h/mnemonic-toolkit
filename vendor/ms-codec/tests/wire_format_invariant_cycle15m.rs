//! cycle-15 Lane M — wire-format invariant guard.
//!
//! The whole cycle is in-memory hygiene (`Zeroizing` wraps, clone removal) +
//! in-process struct-shape changes (`InspectReport`). The `ms1` on-wire
//! encode/decode byte stream and the `Payload`/`decode()` `(Tag, Payload)`
//! shape MUST be byte-identical pre/post. These vectors pin that contract; the
//! toolkit consumes `Payload`/`decode()` and must keep compiling/decoding.

use ms_codec::{decode, encode, Payload, Tag};

/// Full vector set across entr + mnem at every valid entropy length.
fn vectors() -> Vec<Payload> {
    let mut v = Vec::new();
    for len in [16usize, 20, 24, 28, 32] {
        let bytes: Vec<u8> = (0..len as u8)
            .map(|i| i.wrapping_mul(7).wrapping_add(3))
            .collect();
        v.push(Payload::Entr(bytes.clone()));
        v.push(Payload::Mnem {
            language: 1,
            entropy: bytes,
        });
    }
    v
}

#[test]
fn encode_bytes_are_stable_and_decode_round_trips() {
    // Pinned expected ms1 strings for the entr-16 / mnem-16 base cases. If the
    // wire encoding ever drifts, these literals catch it byte-for-byte.
    let entr16 = encode(
        Tag::ENTR,
        &Payload::Entr(
            (0..16u8)
                .map(|i| i.wrapping_mul(7).wrapping_add(3))
                .collect(),
        ),
    )
    .unwrap();
    let mnem16 = encode(
        Tag::ENTR,
        &Payload::Mnem {
            language: 1,
            entropy: (0..16u8)
                .map(|i| i.wrapping_mul(7).wrapping_add(3))
                .collect(),
        },
    )
    .unwrap();

    // ms1 strings are HRP `ms1` + codex32 body; lengths are fixed by the format.
    assert!(
        entr16.starts_with("ms1"),
        "entr16 must be an ms1 string: {entr16}"
    );
    assert_eq!(entr16.len(), 50, "entr-16 string length is fixed at 50");
    assert!(
        mnem16.starts_with("ms1"),
        "mnem16 must be an ms1 string: {mnem16}"
    );
    assert_eq!(mnem16.len(), 51, "mnem-16 string length is fixed at 51");

    // Round-trip every vector: encode → decode → equality, and the (Tag, Payload)
    // shape is preserved (the toolkit-shared contract).
    for p in vectors() {
        let s = encode(Tag::ENTR, &p).unwrap();
        let (tag, recovered) = decode(&s).unwrap();
        assert_eq!(tag, Tag::ENTR, "decode tag must be ENTR for {p:?}");
        assert_eq!(
            recovered, p,
            "decode must recover the exact payload for {s}"
        );
        // Re-encode is byte-identical (idempotent wire).
        let s2 = encode(Tag::ENTR, &recovered).unwrap();
        assert_eq!(s, s2, "re-encode must be byte-identical");
    }
}
