//! Public canonical-payload (pre-chunking bytecode) round-trip tests.
//!
//! Phase P0 of the downstream-consumer plan: a `KeyCard` exposes its
//! deterministic pre-chunking bytecode as a public, round-trippable
//! byte payload via two additive methods on `KeyCard`:
//!
//! - [`KeyCard::canonical_payload_bytes`] — the canonical bytecode, byte-
//!   for-byte equal to the corpus's `expected.canonical_bytecode_hex` and
//!   independent of the per-encode random `chunk_set_id` (string layer).
//! - [`KeyCard::from_canonical_payload_bytes`] — reverses it.
//!
//! These mirror the lower-level `bytecode::encode_bytecode` /
//! `bytecode::decode_bytecode` entry points but live on the public type
//! so a downstream consumer never reaches into the `bytecode` module.
//!
//! Corpus: `src/test_vectors/v0.1.json` (each clean vector carries
//! `expected.canonical_bytecode_hex` + a pinned `input.chunk_set_id`).

use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use mk_codec::{KeyCard, decode, encode_with_chunk_set_id};
use serde_json::Value;

const VECTOR_FILE: &str = "src/test_vectors/v0.1.json";

fn vector_doc() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(VECTOR_FILE);
    let bytes = fs::read(path).expect("read src/test_vectors/v0.1.json");
    serde_json::from_slice(&bytes).expect("parse vectors JSON")
}

fn parse_hex(s: &str) -> Vec<u8> {
    hex::decode(s).expect("vector hex must decode")
}

fn vec4(bytes: &[u8]) -> [u8; 4] {
    bytes.try_into().expect("4-byte hex slice")
}

/// Rebuild the `KeyCard` from a clean vector's `input` block (mirrors the
/// helper in `tests/vectors.rs`).
fn build_card_from_input(input: &Value) -> KeyCard {
    let stubs: Vec<[u8; 4]> = input["policy_id_stubs"]
        .as_array()
        .expect("policy_id_stubs is array")
        .iter()
        .map(|v| vec4(&parse_hex(v.as_str().expect("hex string"))))
        .collect();
    let fp: Option<Fingerprint> = match &input["origin_fingerprint"] {
        Value::Null => None,
        Value::String(s) => Some(Fingerprint::from(vec4(&parse_hex(s)))),
        other => panic!("unexpected origin_fingerprint value: {other:?}"),
    };
    let path = DerivationPath::from_str(
        input["origin_path"]
            .as_str()
            .expect("origin_path is string"),
    )
    .expect("origin_path parses");
    let xpub: Xpub = input["xpub"]
        .as_str()
        .expect("xpub is string")
        .parse()
        .expect("xpub parses");
    KeyCard::new(stubs, fp, path, xpub)
}

/// Find the named clean vector and return its `(input, expected)` blocks.
fn clean_vector<'a>(doc: &'a Value, name: &str) -> (&'a Value, &'a Value) {
    let v = doc["vectors"]
        .as_array()
        .expect("vectors is array")
        .iter()
        .find(|v| v["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("clean vector {name} not found"));
    assert!(
        v["expected_error"].is_null(),
        "{name} is a negative vector, not clean"
    );
    (&v["input"], &v["expected"])
}

/// Decode a clean vector's pinned mk1 strings back into a `KeyCard`.
fn decode_vector(expected: &Value) -> KeyCard {
    let strings: Vec<String> = expected["strings"]
        .as_array()
        .expect("strings is array")
        .iter()
        .map(|v| v.as_str().expect("string").to_string())
        .collect();
    let parts: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    decode(&parts).expect("vector strings decode")
}

/// Representative clean vectors: a 1-stub-with-fp card, a no-fp card, and
/// two genuinely multi-chunk (3-string) cards.
const REPRESENTATIVE: &[&str] = &[
    "V1_bip48_mainnet_1_stub_with_fp",
    "V4_bip84_mainnet_1_stub_no_fp",
    "V5_explicit_path_4_components_with_fp", // 3 chunks
    "V7_max_path_components_no_fp",          // 3 chunks
];

/// 1. Vector match — `decode(strings)?.canonical_payload_bytes()?` equals
///    the corpus's pinned `expected.canonical_bytecode_hex`, byte-for-byte.
#[test]
fn canonical_payload_matches_corpus_hex() {
    let doc = vector_doc();
    for name in REPRESENTATIVE {
        let (_input, expected) = clean_vector(&doc, name);
        let card = decode_vector(expected);
        let actual = card
            .canonical_payload_bytes()
            .unwrap_or_else(|e| panic!("[{name}] canonical_payload_bytes failed: {e}"));
        let want = parse_hex(
            expected["canonical_bytecode_hex"]
                .as_str()
                .expect("canonical_bytecode_hex is string"),
        );
        assert_eq!(
            actual, want,
            "[{name}] canonical_payload_bytes != pinned canonical_bytecode_hex"
        );
    }
}

/// 2. Bytecode round-trip — `from_canonical_payload_bytes(&card.canonical_payload_bytes()?)?`
///    reproduces an equal `KeyCard`.
#[test]
fn canonical_payload_round_trips() {
    let doc = vector_doc();
    for name in REPRESENTATIVE {
        let (input, _expected) = clean_vector(&doc, name);
        let card = build_card_from_input(input);
        let bytes = card
            .canonical_payload_bytes()
            .unwrap_or_else(|e| panic!("[{name}] canonical_payload_bytes failed: {e}"));
        let recovered = KeyCard::from_canonical_payload_bytes(&bytes)
            .unwrap_or_else(|e| panic!("[{name}] from_canonical_payload_bytes failed: {e}"));
        assert_eq!(
            recovered, card,
            "[{name}] from_canonical_payload_bytes(canonical_payload_bytes) != original"
        );
    }
}

/// 3. Cross-`chunk_set_id` determinism (load-bearing): the SAME card encoded
///    under two distinct `chunk_set_id`s yields DIFFERENT mk1 strings, but
///    the canonical payload recovered from either set is IDENTICAL — i.e. the
///    bytecode is invariant to the random string-layer framing.
#[test]
fn canonical_payload_is_chunk_set_id_invariant() {
    let doc = vector_doc();
    // Use a multi-chunk card so the 20-bit chunk_set_id actually appears in
    // the chunked-string headers and makes the two string sets differ.
    let (input, _expected) = clean_vector(&doc, "V5_explicit_path_4_components_with_fp");
    let card = build_card_from_input(input);

    let id_a: u32 = 0x0_00AA;
    let id_b: u32 = 0x0_FF55;

    let strings_a = encode_with_chunk_set_id(&card, id_a).expect("encode A");
    let strings_b = encode_with_chunk_set_id(&card, id_b).expect("encode B");
    assert!(
        strings_a.len() > 1 && strings_b.len() > 1,
        "test card must be multi-chunk for chunk_set_id to appear on the wire"
    );
    assert_ne!(
        strings_a, strings_b,
        "distinct chunk_set_id must produce distinct mk1 strings"
    );

    let parts_a: Vec<&str> = strings_a.iter().map(|s| s.as_str()).collect();
    let parts_b: Vec<&str> = strings_b.iter().map(|s| s.as_str()).collect();
    let payload_a = decode(&parts_a)
        .expect("decode A")
        .canonical_payload_bytes()
        .expect("payload A");
    let payload_b = decode(&parts_b)
        .expect("decode B")
        .canonical_payload_bytes()
        .expect("payload B");

    assert_eq!(
        payload_a, payload_b,
        "canonical payload must be invariant to the random chunk_set_id framing"
    );
}

/// 4. Empty / invalid bytes are rejected cleanly (a clean `Err`, no panic).
#[test]
fn from_canonical_payload_bytes_rejects_garbage() {
    // Empty input: the decoder hits end-of-stream reading the header byte.
    assert!(
        KeyCard::from_canonical_payload_bytes(&[]).is_err(),
        "empty input must be rejected"
    );

    // A single stray byte: a malformed header / truncated stream.
    assert!(
        KeyCard::from_canonical_payload_bytes(&[0xFF]).is_err(),
        "1-byte garbage must be rejected"
    );

    // A short all-zero blob: not a well-formed canonical payload.
    assert!(
        KeyCard::from_canonical_payload_bytes(&[0u8; 8]).is_err(),
        "short zero blob must be rejected"
    );

    // Trailing-byte corruption of a real payload must also be rejected.
    let doc = vector_doc();
    let (input, _expected) = clean_vector(&doc, "V1_bip48_mainnet_1_stub_with_fp");
    let card = build_card_from_input(input);
    let mut bytes = card.canonical_payload_bytes().expect("payload");
    bytes.push(0xAB); // one extra byte after the xpub
    assert!(
        KeyCard::from_canonical_payload_bytes(&bytes).is_err(),
        "trailing-byte corruption must be rejected"
    );
}
