//! Vector-corpus integration tests.
//!
//! Walks `crates/mk-codec/src/test_vectors/v0.1.json` (the canonical
//! conformance corpus emitted by `cargo run --bin gen_mk_vectors --features gen-vectors`)
//! and asserts byte-equal round-trip equality for every vector:
//!
//! 1. `encode_bytecode(input) == expected.canonical_bytecode_hex` (byte-for-byte).
//! 2. `encode_with_chunk_set_id(input, chunk_set_id) == expected.strings`.
//! 3. `decode(expected.strings) == input` (structural equality of `KeyCard`).
//! 4. The on-disk JSON's SHA-256 matches the pinned `V0_1_SHA256` constant.
//!
//! Pinning the SHA-256 turns silent vector drift into a CI-level
//! failure: any change to the JSON file (intended or accidental)
//! produces a different hash and the test fails until both the file
//! and the pinned constant are updated together.

use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use bitcoin::NetworkKind;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use mk_codec::{KeyCard, bytecode::encode_bytecode, decode, encode_with_chunk_set_id};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Pinned SHA-256 of `crates/mk-codec/src/test_vectors/v0.1.json`.
///
/// Update via:
///
/// ```text
/// cargo run --bin gen_mk_vectors --features gen-vectors
/// sha256sum crates/mk-codec/src/test_vectors/v0.1.json
/// # paste the hex into V0_1_SHA256
/// ```
///
/// Cross-implementations validate by matching this hash plus per-vector
/// round-trip equality. Drift here means the vector corpus was modified;
/// any such change is a wire-format-relevant event and MUST be
/// reviewed before landing.
const V0_1_SHA256: &str = "ebd8f34d8d52896e07e1faef995f18ffa61d42e2a048fb2a8c11e67f120d78ff";

/// On-disk path to the canonical mk-codec vector corpus.
///
/// **Filename is intentionally stable across minor-bump family-token
/// rolls.** The corpus's `family_token` field carries the
/// `mk-codec X.Y` version (per closure Q-10: minor-version bumps roll
/// the token; patches don't), but the on-disk filename stays
/// `v0.1.json` to keep cross-repo / cross-implementation tooling
/// pointing at a single artifact location regardless of which family
/// the token has rolled to. mk-codec v0.2+ corpora carry
/// `family_token: "mk-codec 0.2"` while still living at this path;
/// v0.1.x corpora carried `"mk-codec 0.1"` at the same path. md-codec
/// follows the same convention for its own vector file.
const VECTOR_FILE: &str = "src/test_vectors/v0.1.json";

fn vector_file_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(VECTOR_FILE)
}

fn read_vector_file() -> Vec<u8> {
    fs::read(vector_file_path()).expect("read src/test_vectors/v0.1.json")
}

fn parse_hex(s: &str) -> Vec<u8> {
    hex::decode(s).expect("vector hex must decode")
}

fn vec4(bytes: &[u8]) -> [u8; 4] {
    bytes.try_into().expect("4-byte hex slice")
}

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
    let path_str = input["origin_path"]
        .as_str()
        .expect("origin_path is string");
    let path = DerivationPath::from_str(path_str).expect("origin_path parses");
    let xpub: Xpub = input["xpub"]
        .as_str()
        .expect("xpub is string")
        .parse()
        .expect("xpub parses");
    // Sanity-check the xpub network field matches the declared network.
    let declared = input["network"].as_str().expect("network is string");
    let actual = match xpub.network {
        NetworkKind::Main => "mainnet",
        NetworkKind::Test => "testnet",
    };
    assert_eq!(
        actual, declared,
        "vector network mismatch: xpub says {actual}, fixture declares {declared}"
    );
    KeyCard::new(stubs, fp, path, xpub)
}

#[test]
fn vector_file_sha256_matches_pin() {
    let bytes = read_vector_file();
    let digest = Sha256::digest(&bytes);
    let actual = hex::encode(digest);
    assert_eq!(
        actual, V0_1_SHA256,
        "src/test_vectors/v0.1.json SHA-256 drifted; if intended, regenerate via \
         `cargo run --bin gen_mk_vectors --features gen-vectors` and update \
         `V0_1_SHA256` in tests/vectors.rs"
    );
}

#[test]
fn schema_metadata_pinned() {
    // Pin schema version + family token at the top of the document so a
    // future generator can't silently bump these. Schema 2 (v0.1.1+) adds
    // optional `expected_error` per-vector for negative-vector support;
    // schema 1 (v0.1.0) corpora are still readable by this harness.
    let bytes = read_vector_file();
    let doc: Value = serde_json::from_slice(&bytes).expect("parse vectors JSON");
    assert_eq!(doc["schema"], Value::from(2u64), "schema version drift");
    assert_eq!(
        doc["family_token"].as_str().unwrap_or(""),
        "mk-codec 0.2",
        "family_token drift — see consts.rs::GENERATOR_FAMILY"
    );
}

#[test]
fn every_vector_round_trips() {
    let bytes = read_vector_file();
    let doc: Value = serde_json::from_slice(&bytes).expect("parse vectors JSON");
    let vectors = doc["vectors"].as_array().expect("vectors is array");
    assert!(!vectors.is_empty(), "vector corpus must not be empty");

    let mut clean_count = 0usize;
    let mut negative_count = 0usize;

    for vector in vectors {
        let name = vector["name"]
            .as_str()
            .expect("vector.name is string")
            .to_string();
        match &vector["expected_error"] {
            Value::Null => {
                clean_count += 1;
                exercise_clean_vector(&name, vector);
            }
            Value::String(expected_err) => {
                negative_count += 1;
                exercise_negative_vector(&name, vector, expected_err);
            }
            other => panic!("[{name}] expected_error must be null or string; got {other:?}"),
        }
    }

    // Pin v0.1.1's vector counts as floors (17 clean + 22 negative = 39).
    // The `>=` form lets v0.1.x patch releases add vectors without
    // breaking the harness; tightening to `==` is a v0.2-grade decision.
    assert!(clean_count >= 18, "clean-vector count regressed");
    assert!(negative_count >= 22, "negative-vector count regressed");
}

fn exercise_clean_vector(name: &str, vector: &Value) {
    let input = &vector["input"];
    let expected = &vector["expected"];

    let card = build_card_from_input(input);

    // 1. Bytecode round-trip — every byte must match the pinned hex.
    let actual_bytecode =
        encode_bytecode(&card).unwrap_or_else(|e| panic!("[{name}] encode_bytecode failed: {e}"));
    let expected_bytecode = parse_hex(
        expected["canonical_bytecode_hex"]
            .as_str()
            .expect("canonical_bytecode_hex is string"),
    );
    assert_eq!(
        actual_bytecode, expected_bytecode,
        "[{name}] canonical_bytecode_hex drifted from encoder output"
    );

    // 2. String-set round-trip — pinned chunk_set_id makes this byte-stable.
    let chunk_set_id = u32::try_from(input["chunk_set_id"].as_u64().expect("chunk_set_id is u64"))
        .expect("chunk_set_id fits in u32");
    let actual_strings = encode_with_chunk_set_id(&card, chunk_set_id)
        .unwrap_or_else(|e| panic!("[{name}] encode_with_chunk_set_id failed: {e}"));
    let expected_strings: Vec<String> = expected["strings"]
        .as_array()
        .expect("strings is array")
        .iter()
        .map(|v| v.as_str().expect("string").to_string())
        .collect();
    assert_eq!(
        actual_strings, expected_strings,
        "[{name}] mk1 string set drifted from encoder output"
    );

    // 3. total_chunks invariant — the metadata field equals the
    //    actual emitted string count.
    let expected_total = u64::try_from(actual_strings.len()).unwrap();
    assert_eq!(
        expected["total_chunks"].as_u64().unwrap_or(0),
        expected_total,
        "[{name}] total_chunks metadata disagrees with strings.len()"
    );

    // 4. Decode round-trip — produce the same KeyCard back.
    let recovered_strs: Vec<&str> = actual_strings.iter().map(|s| s.as_str()).collect();
    let recovered_card =
        decode(&recovered_strs).unwrap_or_else(|e| panic!("[{name}] decode failed: {e}"));
    assert_eq!(
        recovered_card, card,
        "[{name}] decoded KeyCard differs from original"
    );

    // 5. Decoder-correction field is `clean` for clean vectors.
    assert_eq!(
        expected["decoder_correction"].as_str().unwrap_or(""),
        "clean",
        "[{name}] decoder_correction is not 'clean' for a clean vector"
    );
}

fn exercise_negative_vector(name: &str, vector: &Value, expected_err: &str) {
    let strings: Vec<String> = vector["input"]["strings"]
        .as_array()
        .expect("[name] negative.input.strings is array")
        .iter()
        .map(|v| v.as_str().expect("string").to_string())
        .collect();
    let parts: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    match decode(&parts) {
        Err(e) => {
            let actual = format!("{e}");
            assert_eq!(
                actual, expected_err,
                "[{name}] decoder error rendering drifted from pinned `expected_error`"
            );
        }
        Ok(card) => panic!(
            "[{name}] expected `Err({expected_err})`; decoder accepted input as KeyCard {card:?}"
        ),
    }
}

// Exhaustiveness gate (every `Error` variant has a negative vector or an
// explicit exemption) lives in `tests/error_coverage.rs` as of v0.2.0.
// It uses a strum::EnumIter-driven mirror enum, mirroring md-codec's
// `crates/md-codec/tests/error_coverage.rs` precedent. The mirror-enum
// pattern sidesteps the `#[non_exhaustive]` constraint that blocks
// integration-test compile-time exhaustive matching on the source enum.
