//! Help-text smoke tests.
//!
//! Uses substring matching rather than byte-exact: clap's exact rendering
//! varies across patch versions; locking to substrings keeps the test stable
//! while still asserting that the documented flags surface.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn top_level_help_lists_subcommands() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("bundle"))
        .stdout(predicate::str::contains("verify-bundle"));
}

#[test]
fn bundle_help_shows_slot_and_template_flags() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["bundle", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--slot"))
        .stdout(predicate::str::contains("--network"))
        .stdout(predicate::str::contains("--template"))
        .stdout(predicate::str::contains("--no-engraving-card"));
}

#[test]
fn verify_bundle_help_shows_required_flags() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["verify-bundle", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--mk1"))
        .stdout(predicate::str::contains("--md1"))
        .stdout(predicate::str::contains("--ms1"))
        .stdout(predicate::str::contains("--network"))
        .stdout(predicate::str::contains("--template"));
}
