//! Prints `<name>\t<key>` for every vendored keyed vector.
//! Plan 1b's evidence table is a transcript of THIS -- one implementation of
//! the key, never a second one in Python.
//!
//! Reads the actual md1 WIRE CHUNK SET vendored at
//! `crates/md-codec/tests/vectors/keyed_*.phrase.txt` -- the same bytes a
//! SeedHammer II plate carries -- and decodes it through the crate's own
//! `chunk::reassemble` / `skeleton` / `skeleton_key`, the ONE implementation
//! of the key. `tests/skeleton_key_conformance.rs` (Task 5's gate) proves
//! this agrees with the OTHER route -- the descriptor the same vendored
//! record carries; this binary never recomputes the key a second way, and
//! plan 1b's table must not either.

use std::path::{Path, PathBuf};

use md_codec::chunk::reassemble;
use md_codec::skeleton::{skeleton, skeleton_key};

fn conformance_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/vectors"))
}

fn keyed_phrase_files(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("keyed_") && n.ends_with(".phrase.txt"))
        })
        .collect();
    out.sort();
    out
}

fn main() {
    let dir = conformance_dir();
    let files = keyed_phrase_files(&dir);
    assert!(
        !files.is_empty(),
        "no keyed_*.phrase.txt vectors found under {}",
        dir.display()
    );
    for path in files {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_else(|| panic!("{}: unreadable file stem", path.display()));
        let name = stem
            .strip_suffix(".phrase")
            .unwrap_or_else(|| panic!("{}: expected a <name>.phrase.txt file", path.display()));

        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let mut lines = text.lines();
        let header = lines
            .next()
            .unwrap_or_else(|| panic!("{name}: empty phrase file"));
        assert!(
            header.starts_with("chunk-set-id:"),
            "{name}: expected a chunk-set header, got {header:?}"
        );
        let chunks: Vec<&str> = lines.filter(|l| !l.is_empty()).collect();
        assert!(!chunks.is_empty(), "{name}: no chunk lines");

        let d = reassemble(&chunks).unwrap_or_else(|e| panic!("{name}: reassemble: {e}"));
        let s = skeleton(&d).unwrap_or_else(|e| panic!("{name}: skeleton: {e}"));
        let key = skeleton_key(&s);
        println!("{name}\t{}", key.as_str());
    }
}
