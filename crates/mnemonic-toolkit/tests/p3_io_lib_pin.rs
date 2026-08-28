//! P3 row 1 — THE PIN, asserted by a call that COMPILES.
//!
//! `mnemonic-io-lib` is pinned as a REGISTRY dependency (`= "0.1.0"`) in
//! `crates/mnemonic-toolkit/Cargo.toml`. It was a `git` + `rev` pin when P3
//! landed and moved to the registry while merging F-354: a git source has no
//! published tarball, so neither `Cargo.lock`'s `checksum` nor
//! `.cargo-checksum.json`'s `package` can anchor the bytes that get vendored,
//! and `ci/repro/vendor-freshness.sh` must hand-ground every git source it
//! tolerates. The plan's gate for this row is indifferent to which of the two
//! it is, and is unchanged: it is not "the manifest has a line in it" — a
//! manifest line proves nothing about what the pinned release exposes. It is
//! that a **call compiles** against the pinned release, through the module
//! path the crate actually publishes.
//!
//! **The trap this file exists to catch.** `mnemonic_io_lib`'s `lib.rs` root
//! re-exports exactly three lines — `channel::{destination, Destination}`,
//! `exit::{write_block, WriteBlock}` and
//! `records::{no_records_guard, split_record_stream}`. `remedy` is a `pub mod`
//! with **no** root re-export, so `mnemonic_io_lib::history_purge_block` is an
//! `E0425` and only `mnemonic_io_lib::remedy::history_purge_block` resolves.
//! `mnemonic` adopts the `remedy` pair and nothing else, so that is the path
//! under test here.

use mnemonic_io_lib::remedy::{history_purge_block, history_purge_recipes};

/// The structured half. `mnemonic`'s refusal prints the block, but the block is
/// built from these, and a test that can reach the recipes can run one.
#[test]
fn pinned_revision_exposes_history_purge_recipes() {
    let recipes = history_purge_recipes("mnemonic convert");
    let shells: Vec<&str> = recipes.iter().map(|(s, _)| *s).collect();
    assert_eq!(
        shells,
        vec!["zsh", "bash", "fish"],
        "the pinned mnemonic-io-lib must publish one recipe per shell"
    );
    // The command is interpolated into the two shells that match on it, and
    // deliberately NOT into fish (see the crate's own doc comment: every fish
    // `history delete` spelling has to be handed the material to match on).
    for (shell, recipe) in &recipes {
        assert!(
            !recipe.is_empty(),
            "{shell}: the pinned revision emitted an empty recipe"
        );
        if *shell != "fish" {
            assert!(
                recipe.contains("mnemonic convert"),
                "{shell}: recipe must match on the COMMAND, got {recipe:?}"
            );
        }
    }
}

/// The printed half — the item `mnemonic`'s argv refusal emits verbatim.
#[test]
fn pinned_revision_exposes_history_purge_block() {
    let block = history_purge_block("mnemonic convert");
    assert!(
        block.contains("TO PURGE WHAT ALREADY LEAKED"),
        "the pinned block's opening line moved: {block}"
    );
    // `history -d` is NAMED in order to warn against it and must never be
    // OFFERED. Asserting the naive negative here would go RED against the
    // correct text — so assert the shape the crate's own tests assert: the
    // string appears in the block, and in no recipe.
    assert!(
        block.contains("history -d"),
        "the block must still NAME the zsh trap it warns about"
    );
    for (shell, recipe) in history_purge_recipes("mnemonic convert") {
        assert!(
            !recipe.contains("history -d"),
            "{shell}: `history -d` must never be OFFERED as a recipe"
        );
    }
}
