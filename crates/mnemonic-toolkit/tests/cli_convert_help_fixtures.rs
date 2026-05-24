//! v0.6 `mnemonic convert --help` smoke test.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn convert_help_lists_required_flags() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["convert", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--from"))
        .stdout(predicate::str::contains("--to"))
        .stdout(predicate::str::contains("--network"))
        .stdout(predicate::str::contains("--template"))
        .stdout(predicate::str::contains("--json"));
}

// v0.36.0: lock the `--from` node enumeration so the documented `entropy`
// row can't silently regress. (The R0 review of v0.36.0 found the controller
// had wrongly believed `entropy` was missing from the help — a grep blind-spot;
// the row at convert.rs:175 was present all along. This pins it.)
#[test]
fn convert_help_documents_entropy_node() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["convert", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("entropy"));
}

#[test]
fn top_level_help_lists_convert() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("convert"));
}
