//! Versioned vector corpus replay. SHA-pinned at v0.1.0 release per RELEASE_PROCESS.md.

use ms_codec::{decode, encode, Payload, Tag};
use serde::Deserialize;

#[derive(Deserialize)]
struct Vector {
    description: String,
    #[allow(dead_code)]
    mnemonic: String,
    entropy_hex: String,
    ms1: String,
}

fn load_v01_corpus() -> Vec<Vector> {
    let raw = include_str!("vectors/v0.1.json");
    serde_json::from_str(raw).expect("v0.1.json parse")
}

fn decode_hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn v01_corpus_round_trips() {
    let corpus = load_v01_corpus();
    assert!(
        !corpus.is_empty(),
        "v0.1.json must have at least one vector"
    );

    for v in &corpus {
        let entropy = decode_hex(&v.entropy_hex);
        let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone()))
            .unwrap_or_else(|e| panic!("{}: encode failed: {:?}", v.description, e));
        assert_eq!(s, v.ms1, "{}: encoded ms1 mismatch", v.description);

        let (tag, payload) =
            decode(&v.ms1).unwrap_or_else(|e| panic!("{}: decode failed: {:?}", v.description, e));
        assert_eq!(tag, Tag::ENTR);
        assert_eq!(payload, Payload::Entr(entropy));
    }
}
