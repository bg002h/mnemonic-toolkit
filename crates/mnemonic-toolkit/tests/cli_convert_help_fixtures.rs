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

#[test]
fn top_level_help_lists_convert() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("convert"));
}
