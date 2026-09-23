//! Stage 1a's gate: the InternalKey refactor must change NO wire bytes.
//!
//! md-codec has NO template parser — `parse::template` lives in md-cli — so
//! this follows the crate's own idiom (see `examples/dump_skeleton_keys.rs`):
//! read the VENDORED md1 wire strings under `tests/vectors/`, the same bytes
//! a SeedHammer II plate carries, decode them, re-encode, and compare. That
//! is a stronger gate than re-encoding a parsed template, because the input
//! is the real wire.
use md_codec::encode::encode_payload;

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/common/vendored.rs"
));

/// The golden is an OBJECT, not a bare array (Task 0 Step 2).
#[derive(serde::Deserialize)]
struct Golden {
    count: usize,
    tr_count: usize,
    vectors: Vec<(String, String)>,
}

#[test]
fn every_vendored_wire_vector_re_encodes_to_the_same_bytes() {
    let g: Golden = serde_json::from_str(include_str!("golden/pre_refactor_encodings.json"))
        .expect("golden parses");
    assert_eq!(g.count, 65, "vector population drifted");
    assert_eq!(
        g.tr_count, 23,
        "Body::Tr coverage drifted — this stage is about Tr"
    );
    assert_eq!(g.vectors.len(), g.count);
    for (name, want_hex) in &g.vectors {
        let chunks = load_vendored_phrase(name); // tests/vectors/<name>.phrase.txt
        // decode_vendored, NOT reassemble: 13 of the 65 are single-payload
        // strings, and the chunk reader misreads their first symbol as
        // version 2. Task 0 Step 2 has the full explanation.
        let d = decode_vendored(&chunks).unwrap_or_else(|e| panic!("{name}: decode: {e}"));
        let (bytes, _bits) = encode_payload(&d).unwrap_or_else(|e| panic!("{name}: encode: {e}"));
        assert_eq!(
            hex::encode(&bytes),
            *want_hex,
            "wire bytes changed for {name}"
        );
    }
}
