//! Stage 1b task 5 (SPEC §4, rows 2 and 3): BOTH keyless render modes emit
//! `LIANA_UNSPENDABLE_MARKER` for a wire-kind-1 internal key, never the raw
//! NUMS hex. §4's own words: "The derived xpub **cannot** appear in a
//! keyless form — its chain code is a function of the seated leaf keys,
//! which a template-only card does not carry. The marker is the only
//! correct spelling, not a shortcut."
//!
//! This is `render.rs`'s half of G-1's four gating `NumsPoint |
//! LianaUnspendable` or-patterns (`render.rs:192`): before this task both
//! kinds rendered the identical NUMS hex in every mode, keyed or not.

mod common;

use md_codec::nums::LIANA_UNSPENDABLE_MARKER;
use md_codec::render::{descriptor_to_abstract_template, descriptor_to_template};

#[test]
fn kind_1_emits_the_marker_in_literal_mode() {
    let d = common::tr_liana_unspendable_two_leaves();
    let t = descriptor_to_template(&d).unwrap();
    assert!(
        t.contains(LIANA_UNSPENDABLE_MARKER),
        "Mode::Literal must emit the marker for a wire-kind-1 internal key: got {t}"
    );
}

#[test]
fn kind_1_emits_the_marker_in_abstract_mode() {
    let d = common::tr_liana_unspendable_two_leaves();
    let t = descriptor_to_abstract_template(&d).unwrap();
    assert!(
        t.contains(LIANA_UNSPENDABLE_MARKER),
        "Mode::Abstract must emit the marker for a wire-kind-1 internal key: got {t}"
    );
}

#[test]
fn kind_0_does_not_emit_the_marker_in_either_mode() {
    // Differential control: the SAME two-leaf shape at kind 0
    // (`tr_nums_two_leaves`) must never carry the marker text — proves the
    // two kinds render DIFFERENTLY rather than the marker being emitted
    // unconditionally regardless of `internal_key`.
    let d = common::tr_nums_two_leaves();
    let lit = descriptor_to_template(&d).unwrap();
    let abs = descriptor_to_abstract_template(&d).unwrap();
    assert!(!lit.contains(LIANA_UNSPENDABLE_MARKER), "got {lit}");
    assert!(!abs.contains(LIANA_UNSPENDABLE_MARKER), "got {abs}");
}

#[test]
fn kind_0_and_kind_1_templates_differ() {
    // A direct A/B: same tap-tree shape, only the internal key kind differs.
    // If the G-1 or-pattern at render.rs:192 were reverted (both kinds
    // folded back onto the NUMS-hex branch), this pair would render
    // IDENTICALLY and this assertion would fail.
    let nums = descriptor_to_template(&common::tr_nums_two_leaves()).unwrap();
    let liana = descriptor_to_template(&common::tr_liana_unspendable_two_leaves()).unwrap();
    assert_ne!(nums, liana, "kind 0 and kind 1 must render differently");
}
