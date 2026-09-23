//! Emit the PRE-REFACTOR encodings golden: `encode_payload` output for
//! every vendored vector, run through UNMODIFIED `src/` code.
//!
//! This is Task 0's whole point -- an internal-key-refactor plan replaces a
//! struct field pair with a sum type and must change ZERO wire bytes. This
//! binary captures the "before" so a later task can diff against it.
//!
//! Reads all `tests/vectors/*.phrase.txt` (all 65 -- unlike
//! `dump_skeleton_keys.rs`'s `keyed_phrase_files`, NOT filtered to
//! `keyed_*`), decodes each via `decode_vendored` (which dispatches chunk
//! set vs. single payload -- see that function's doc comment in
//! `tests/common/vendored.rs`), then `encode_payload`s the result back down
//! and hex-encodes the bytes.
//!
//! `tr_count` walks the whole tree, not just the root -- though in practice
//! `Tag::Tr` is always the top-level tag for these vectors, since BIP 388
//! does not permit nesting a taproot descriptor inside another wrapper.
//!
//! ```json
//! { "count": 65, "tr_count": 23, "vectors": [["name", "hex"], ...] }
//! ```

use md_codec::encode_payload;
use md_codec::tree::{Body, Node};

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/common/vendored.rs"
));

/// True iff `node`, or any node reachable from it, carries `Body::Tr`.
fn contains_tr(node: &Node) -> bool {
    if matches!(node.body, Body::Tr { .. }) {
        return true;
    }
    match &node.body {
        Body::Children(children) => children.iter().any(contains_tr),
        Body::Variable { children, .. } => children.iter().any(contains_tr),
        _ => false,
    }
}

fn main() {
    let names = all_vendored_vector_names();
    assert!(
        !names.is_empty(),
        "no tests/vectors/*.phrase.txt vectors found"
    );

    let mut vectors: Vec<(String, String)> = Vec::with_capacity(names.len());
    let mut tr_count = 0usize;

    for name in &names {
        let chunks = load_vendored_phrase(name);
        let d: Descriptor =
            decode_vendored(&chunks).unwrap_or_else(|e| panic!("{name}: decode failed: {e}"));

        if contains_tr(&d.tree) {
            tr_count += 1;
        }

        let (bytes, _total_bits) =
            encode_payload(&d).unwrap_or_else(|e| panic!("{name}: encode_payload failed: {e}"));
        vectors.push((name.clone(), hex::encode(bytes)));
    }

    let out = serde_json::json!({
        "count": vectors.len(),
        "tr_count": tr_count,
        "vectors": vectors,
    });
    println!("{out}");
}
