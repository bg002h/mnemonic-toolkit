//! `mnemonic gen-man` man-page generation tests (v0.73.0).
//!
//! `gen-man --out <DIR>` calls `clap_mangen::generate_to(Cli::command(), dir)`
//! with **NO pre-`.build()`** (SPEC §2 / C-1: a pre-build poisons output with a
//! `help` pseudo-subcommand shadow tree of ~18 spurious `*-help-*.1` pages).
//! The naive call is help-shadow-free by construction.
//!
//! These tests are the leading regression tripwire for an accidental future
//! pre-build (the NEGATIVE canary) and pin the page-set shape against the live
//! clap surface. The expected page set is derived from the `gui-schema` JSON
//! subcommand-name list (the same flattened hyphen-joined naming `generate_to`
//! uses for nested children) — NO magic integer is baked (SPEC §3 / M-2).

use assert_cmd::Command;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Run `mnemonic gen-man --out <dir>` into a fresh tempdir and return the set
/// of generated `*.1` filenames.
fn generate_man_pages(out_dir: &Path) -> BTreeSet<String> {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("gen-man")
        .arg("--out")
        .arg(out_dir)
        .output()
        .expect("gen-man exec failed");
    assert!(out.status.success(), "gen-man must exit 0; got {out:?}");
    let mut pages = BTreeSet::new();
    for entry in fs::read_dir(out_dir).expect("read out_dir") {
        let name = entry.unwrap().file_name().into_string().unwrap();
        if name.ends_with(".1") {
            pages.insert(name);
        }
    }
    pages
}

/// Flattened subcommand-leaf names from `gui-schema` JSON (excludes the
/// `gui-schema`/`help` self-filter; nested children appear hyphen-joined, e.g.
/// `seed-xor-split`). This is the oracle for the per-leaf man-page set.
fn gui_schema_leaf_names() -> Vec<String> {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("gui-schema")
        .output()
        .expect("gui-schema exec failed");
    assert!(out.status.success(), "gui-schema must exit 0; got {out:?}");
    let v: Value = serde_json::from_slice(&out.stdout).expect("gui-schema JSON");
    v["subcommands"]
        .as_array()
        .expect("subcommands array")
        .iter()
        .map(|s| s["name"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn gen_man_produces_nonempty_dot_one_set() {
    let tmp = tempfile::tempdir().unwrap();
    let pages = generate_man_pages(tmp.path());
    assert!(
        !pages.is_empty(),
        "gen-man must produce at least one *.1 page"
    );
}

#[test]
fn gen_man_root_page_has_th_header() {
    let tmp = tempfile::tempdir().unwrap();
    let pages = generate_man_pages(tmp.path());
    assert!(
        pages.contains("mnemonic.1"),
        "root page mnemonic.1 must exist; got {pages:?}"
    );
    let body = fs::read_to_string(tmp.path().join("mnemonic.1")).unwrap();
    assert!(
        body.contains(".TH"),
        "root mnemonic.1 must carry a .TH roff header"
    );
}

#[test]
fn gen_man_emits_a_page_per_gui_schema_leaf_with_distinct_nested_names() {
    let tmp = tempfile::tempdir().unwrap();
    let pages = generate_man_pages(tmp.path());
    // Every flattened gui-schema leaf must have a `mnemonic-<leaf>.1` page.
    // This proves nested children produce DISTINCT hyphen-joined filenames
    // (the three `split` children seed-xor/slip39/ms-shares do NOT collide on
    // a bare `mnemonic-split.1`).
    for leaf in gui_schema_leaf_names() {
        let expected = format!("mnemonic-{leaf}.1");
        assert!(
            pages.contains(&expected),
            "expected man page {expected} for gui-schema leaf {leaf}; got {pages:?}"
        );
    }
    // No bare collision page for the shared `split`/`combine` leaf names.
    assert!(
        !pages.contains("mnemonic-split.1"),
        "nested `split` children must NOT collapse to a single mnemonic-split.1"
    );
}

#[test]
fn gen_man_exact_page_set_walks_the_unbuilt_tree() {
    // Exact-page-set assertion derived from the live surface (NO magic integer,
    // M-2). The produced set must equal:
    //   { mnemonic.1 }                                    (root)
    //   ∪ { mnemonic-<leaf>.1 : leaf ∈ gui-schema names } (visible leaves +
    //                                                      gen-man + the
    //                                                      already-visible
    //                                                      gui-schema's leaves)
    //   ∪ { mnemonic-<parent>.1 : nested parents }        (seed-xor/slip39/
    //                                                      ms-shares/seedqr/
    //                                                      xpub-search)
    //   ∪ { mnemonic-gui-schema.1 }                       (visible, but
    //                                                      self-filtered from
    //                                                      gui-schema JSON)
    // gui-schema flattens parents away (emits only leaves), so the nested
    // parent pages + the self-filtered gui-schema page are folded in here.
    let tmp = tempfile::tempdir().unwrap();
    let pages = generate_man_pages(tmp.path());

    let mut expected: BTreeSet<String> = BTreeSet::new();
    expected.insert("mnemonic.1".to_string());
    for leaf in gui_schema_leaf_names() {
        expected.insert(format!("mnemonic-{leaf}.1"));
    }
    // Nested-parent intermediate pages that `generate_to` emits but gui-schema
    // (leaf-only) omits. Derived from the live tree's nested-parent set.
    for parent in ["seed-xor", "slip39", "ms-shares", "seedqr", "xpub-search"] {
        expected.insert(format!("mnemonic-{parent}.1"));
    }
    // gui-schema self-filters its own name from the JSON, but `generate_to`
    // (which never consults that filter) emits its page. gen-man is already
    // covered by the gui-schema leaf loop (build_schema emits it).
    expected.insert("mnemonic-gui-schema.1".to_string());

    assert_eq!(
        pages, expected,
        "produced man-page set must EXACTLY equal the walk of the live \
         (unbuilt) clap tree minus is_hide_set() and the auto `help` \
         subcommand — no extra, no missing pages"
    );
}

#[test]
fn gen_man_includes_gen_man_and_gui_schema_pages() {
    let tmp = tempfile::tempdir().unwrap();
    let pages = generate_man_pages(tmp.path());
    assert!(
        pages.contains("mnemonic-gen-man.1"),
        "gen-man is VISIBLE → its own page must be emitted; got {pages:?}"
    );
    assert!(
        pages.contains("mnemonic-gui-schema.1"),
        "gui-schema is visible → its page must be emitted; got {pages:?}"
    );
}

#[test]
fn gen_man_negative_canary_no_help_shadow_pages() {
    // NEGATIVE canary (C-1 / I-2): the naive `generate_to` call (no pre-build)
    // emits ZERO `*-help*.1` pages. A future accidental pre-`.build()` would
    // materialize the `help` pseudo-subcommand shadow tree (~18 spurious
    // `mnemonic-help.1` / `mnemonic-*-help-*.1` pages) — this test is the
    // tripwire for that regression.
    let tmp = tempfile::tempdir().unwrap();
    let pages = generate_man_pages(tmp.path());
    let help_pages: Vec<&String> = pages
        .iter()
        .filter(|p| {
            // matches `mnemonic-help.1` and any `*-help-*.1`
            let stem = p.strip_suffix(".1").unwrap_or(p);
            stem == "mnemonic-help" || stem.contains("-help-") || stem.ends_with("-help")
        })
        .collect();
    assert!(
        help_pages.is_empty(),
        "naive generate_to must emit NO *-help*.1 shadow pages (accidental \
         pre-.build() regression); found {help_pages:?}"
    );
}
