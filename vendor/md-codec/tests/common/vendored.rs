// Shared vendored-vector reader, spliced into THREE separate crate roots.
//
// `examples/` and `tests/` are separate crate roots -- `use` across them
// does not compile -- so this file is never compiled on its own. It is
// pulled in verbatim with
// `include!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/common/vendored.rs"))`
// from `examples/dump_encodings.rs`, `examples/dump_ids.rs`, and (a later
// task) `tests/internal_key_refactor.rs` alike. One reader, three
// consumers, no divergence -- and no `pub mod testsupport` leaking into the
// shipped crate.
//
// Cargo only auto-discovers `.rs` files that are DIRECT children of
// `tests/` as separate integration-test binaries, so `tests/common/` is
// never built as a target of its own; it only exists to be spliced in.
//
// `Descriptor` is imported HERE, not in each consumer -- `include!` splices
// this text at the call site, so a consumer that also wrote
// `use md_codec::Descriptor;` would collide with this one (E0252).

use md_codec::Descriptor;

/// Directory holding the vendored md1 wire strings, one `<name>.phrase.txt`
/// per vector -- the same bytes a SeedHammer II plate carries.
fn vendored_vectors_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/vectors"))
}

/// Every vector name under `tests/vectors/*.phrase.txt`, sorted. ALL of
/// them -- unlike `dump_skeleton_keys.rs`'s `keyed_phrase_files`, this does
/// NOT filter to `keyed_*` (that yields 46; the corpus this reads is 65).
///
/// `#[allow(dead_code)]`: not every spliced-in consumer calls this -- e.g.
/// `tests/internal_key_refactor.rs` iterates the golden's own vector list
/// instead of re-listing the vectors directory, so this function is dead
/// code in that compilation unit specifically.
#[allow(dead_code)]
fn all_vendored_vector_names() -> Vec<String> {
    let dir = vendored_vectors_dir();
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter_map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_suffix(".phrase.txt"))
                .map(str::to_string)
        })
        .collect();
    names.sort();
    names
}

/// Read the vendored md1 wire string(s) for vector `name`, as the exact
/// strings a SeedHammer II plate would carry.
///
/// Two shapes exist in the corpus: a chunk SET (a `chunk-set-id:` header
/// line followed by one md1 string per chunk) and a single-payload vector
/// (no header; the entire one-line file IS the sole chunk). `decode_vendored`
/// below dispatches on which shape it got.
fn load_vendored_phrase(name: &str) -> Vec<String> {
    let path = vendored_vectors_dir().join(format!("{name}.phrase.txt"));
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut lines = text.lines();
    let first = lines
        .next()
        .unwrap_or_else(|| panic!("{name}: empty phrase file"));

    if first.starts_with("chunk-set-id:") {
        let chunks: Vec<String> = lines.filter(|l| !l.is_empty()).map(String::from).collect();
        assert!(
            !chunks.is_empty(),
            "{name}: chunk-set-id header with no chunk lines"
        );
        chunks
    } else {
        // No header: the file is a single-payload md1 string, one line.
        let extra: Vec<&str> = lines.filter(|l| !l.is_empty()).collect();
        assert!(
            extra.is_empty(),
            "{name}: no chunk-set-id header but {} extra line(s) -- expected exactly one",
            extra.len()
        );
        vec![first.to_string()]
    }
}

/// Decode a vendored vector, choosing the reader by shape. A chunk SET goes
/// through `reassemble`; a single-payload string goes through
/// `decode_md1_string`, whose auto-dispatch reads bit 0 of the first symbol.
/// Using `reassemble` for both is what made 13 of 65 look like "version 2".
fn decode_vendored(chunks: &[String]) -> Result<Descriptor, md_codec::Error> {
    if chunks.len() == 1 {
        md_codec::decode::decode_md1_string(&chunks[0])
    } else {
        let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
        md_codec::chunk::reassemble(&refs)
    }
}
