//! Emit the PRE-REFACTOR identity golden: `compute_wallet_policy_id`,
//! `compute_wallet_descriptor_template_id`, and the id's 12-word phrase, for
//! every vendored vector, run through UNMODIFIED `src/` code.
//!
//! r0 of the combined plan scheduled this capture INSIDE the task that
//! changes `identity.rs:90`/`:200` to `d.wire_version()`, which would pin
//! POST-change output to itself -- a test that cannot fail. This binary
//! captures identity before anything moves.
//!
//! Reads all `tests/vectors/*.phrase.txt` (all 65, via
//! `all_vendored_vector_names` -- not `keyed_phrase_files`'s 46-file
//! `keyed_` filter), decodes each via `decode_vendored`, and emits a
//! 4-tuple per vector: `[name, wallet_policy_id, template_id, phrase]`. All
//! four fields are captured because a reader that destructures only some of
//! them silently discards the rest -- in particular the 12-word phrase an
//! operator reads off steel.
//!
//! ```json
//! { "count": 65, "vectors": [["name", "policy_id_hex", "template_id_hex", "twelve word phrase"], ...] }
//! ```

use md_codec::{compute_wallet_descriptor_template_id, compute_wallet_policy_id};

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/common/vendored.rs"
));

fn main() {
    let names = all_vendored_vector_names();
    assert!(
        !names.is_empty(),
        "no tests/vectors/*.phrase.txt vectors found"
    );

    let mut vectors: Vec<(String, String, String, String)> = Vec::with_capacity(names.len());

    for name in &names {
        let chunks = load_vendored_phrase(name);
        let d: Descriptor =
            decode_vendored(&chunks).unwrap_or_else(|e| panic!("{name}: decode failed: {e}"));

        let policy_id = compute_wallet_policy_id(&d)
            .unwrap_or_else(|e| panic!("{name}: compute_wallet_policy_id failed: {e}"));
        let template_id = compute_wallet_descriptor_template_id(&d).unwrap_or_else(|e| {
            panic!("{name}: compute_wallet_descriptor_template_id failed: {e}")
        });
        let phrase = policy_id
            .to_phrase()
            .unwrap_or_else(|e| panic!("{name}: to_phrase failed: {e}"));

        vectors.push((
            name.clone(),
            hex::encode(policy_id.as_bytes()),
            hex::encode(template_id.as_bytes()),
            phrase.to_string(),
        ));
    }

    let out = serde_json::json!({
        "count": vectors.len(),
        "vectors": vectors,
    });
    println!("{out}");
}
