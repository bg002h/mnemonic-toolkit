//! G6 cross-repo `mlock.rs` invariant test (Cycle B SPEC §6 G6).
//!
//! Reads this crate's `mlock.rs` and the sibling repo's `mlock.rs`,
//! normalizes both per SPEC §6 G6 (strip `//`, `///`, `//!` comment-only
//! lines at start-of-trimmed-line; preserve `use` statements + `#[cfg]`
//! attributes + internal string-literal whitespace), and asserts
//! byte-equal across the whole module surface plus name-export parity
//! against a static MANIFEST.
//!
//! Sibling-repo path discovery:
//! - `SIBLING_REPO_PATH` env var if set (CI sets this after
//!   `actions/checkout` of the sibling repo to a known runner path).
//! - Else `$CARGO_MANIFEST_DIR/../../../mnemonic-secret` relative-default
//!   (local-dev assumes adjacent-repo layout: mnemonic-toolkit and
//!   mnemonic-secret are sibling directories on disk).
//!
//! Per Cycle B SPEC §5 + §6 G6: the `mlock` module surface is inline-
//! copied (no shared crate; "fork-and-document-pattern over shared-crate-
//! extraction" per the `mc-codex32-extraction-retired-2026-05-03`
//! precedent). This test is the regression backstop.

use std::collections::BTreeSet;
use std::path::PathBuf;

const SIBLING_REPO_REL_DEFAULT: &str = "../../../mnemonic-secret";
const SIBLING_MLOCK_RELATIVE: &str = "crates/ms-cli/src/mlock.rs";

/// Manifest of every top-level item that must remain equivalent across
/// the toolkit's `mlock.rs` and ms-cli's `mlock.rs`.
///
/// Per SPEC §6 G6 item 3 ("Manifest under test"): adding a top-level
/// item to one repo's `mlock.rs` without the corresponding update in the
/// other (and here) fails this test. This is the helper-fn-circumvention
/// mitigation.
const MANIFEST: &[&str] = &[
    "MLOCK_STATE",
    "MlockState",
    "PinnedPageRange",
    "attempts_for_test",
    "errno_to_name",
    "failure_count_for_test",
    "first_errno_for_test",
    "last_os_errno",
    "mlock_state",
    "page_size",
    "page_size_for_test",
    "pin_pages_for",
    "report_at_exit",
    "round_to_pages",
];

#[test]
#[ignore = "G6 cross-repo invariant; needs SIBLING_REPO_PATH (CI g6-invariant job) or adjacent sibling repo on disk. Run via --include-ignored."]
fn g6_mlock_normalized_source_byte_equal() {
    let own_path = own_mlock_path();
    let sibling_path = sibling_mlock_path();
    let own_src = std::fs::read_to_string(&own_path)
        .unwrap_or_else(|e| panic!("read own mlock.rs at {}: {e}", own_path.display()));
    let sibling_src = std::fs::read_to_string(&sibling_path).unwrap_or_else(|e| {
        panic!(
            "read sibling mlock.rs at {} failed: {e}.\n\
             Set SIBLING_REPO_PATH to the sibling repo root, OR keep \
             mnemonic-toolkit and mnemonic-secret as adjacent directories.",
            sibling_path.display()
        )
    });
    let own_norm = normalize(&own_src);
    let sibling_norm = normalize(&sibling_src);
    if own_norm != sibling_norm {
        panic!(
            "G6 SPEC §6 invariant violated — normalized mlock.rs differs.\n\n\
             own path:     {}\n\
             sibling path: {}\n\n\
             First differing line:\n{}\n",
            own_path.display(),
            sibling_path.display(),
            first_diff(&own_norm, &sibling_norm),
        );
    }
}

#[test]
#[ignore = "G6 cross-repo invariant; needs SIBLING_REPO_PATH (CI g6-invariant job) or adjacent sibling repo on disk. Run via --include-ignored."]
fn g6_mlock_name_exports_match_manifest() {
    let own_path = own_mlock_path();
    let sibling_path = sibling_mlock_path();
    let own_src = std::fs::read_to_string(&own_path).unwrap();
    let sibling_src = std::fs::read_to_string(&sibling_path).unwrap_or_else(|e| {
        panic!(
            "read sibling mlock.rs at {} failed: {e}.\n\
             Set SIBLING_REPO_PATH to the sibling repo root, OR keep \
             mnemonic-toolkit and mnemonic-secret as adjacent directories.",
            sibling_path.display()
        )
    });
    let own_names = extract_top_level_names(&own_src);
    let sibling_names = extract_top_level_names(&sibling_src);
    let manifest: BTreeSet<&str> = MANIFEST.iter().copied().collect();
    let own_borrowed: BTreeSet<&str> = own_names.iter().map(String::as_str).collect();
    let sibling_borrowed: BTreeSet<&str> = sibling_names.iter().map(String::as_str).collect();
    assert_eq!(
        own_borrowed, sibling_borrowed,
        "mlock.rs top-level name sets differ between repos.\n\
         own:     {own_borrowed:?}\nsibling: {sibling_borrowed:?}",
    );
    assert_eq!(
        own_borrowed, manifest,
        "MANIFEST out of sync with actual mlock.rs top-level names; \
         update MANIFEST in BOTH repos' tests/mlock_g6_invariant.rs.\n\
         actual:   {own_borrowed:?}\nmanifest: {manifest:?}",
    );
}

fn own_mlock_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/mlock.rs")
}

fn sibling_mlock_path() -> PathBuf {
    let root = match std::env::var("SIBLING_REPO_PATH") {
        Ok(s) => PathBuf::from(s),
        Err(_) => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SIBLING_REPO_REL_DEFAULT),
    };
    root.join(SIBLING_MLOCK_RELATIVE)
}

fn normalize(src: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        out.push(trimmed);
    }
    out.join("\n")
}

fn extract_top_level_names(src: &str) -> BTreeSet<String> {
    const KEYWORDS: &[&str] = &["fn ", "struct ", "enum ", "static ", "const ", "type "];
    let mut names: BTreeSet<String> = BTreeSet::new();
    for line in src.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }
        let body = trimmed
            .trim_start_matches("pub(crate)")
            .trim_start_matches("pub(super)")
            .trim_start_matches("pub")
            .trim_start();
        for kw in KEYWORDS {
            if let Some(rest) = body.strip_prefix(kw) {
                if let Some(name) = take_ident(rest) {
                    names.insert(name);
                }
                break;
            }
        }
    }
    names
}

fn take_ident(s: &str) -> Option<String> {
    let s = s.trim_start();
    let end = s
        .char_indices()
        .take_while(|&(_, c)| c.is_ascii_alphanumeric() || c == '_')
        .map(|(i, c)| i + c.len_utf8())
        .last()
        .unwrap_or(0);
    if end == 0 {
        None
    } else {
        Some(s[..end].to_string())
    }
}

fn first_diff(a: &str, b: &str) -> String {
    let a_lines: Vec<&str> = a.lines().collect();
    let b_lines: Vec<&str> = b.lines().collect();
    let max = a_lines.len().max(b_lines.len());
    for i in 0..max {
        let av = a_lines.get(i).copied().unwrap_or("<missing>");
        let bv = b_lines.get(i).copied().unwrap_or("<missing>");
        if av != bv {
            return format!(
                "L{}:\n  own:     {av}\n  sibling: {bv}\n  (own_lines={}, sibling_lines={})",
                i + 1,
                a_lines.len(),
                b_lines.len(),
            );
        }
    }
    String::from("(no per-line diff; possible trailing-newline issue)")
}
