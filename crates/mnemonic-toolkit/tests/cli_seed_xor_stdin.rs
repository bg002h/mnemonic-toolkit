//! v0.12.0 P2 — stdin-route tests for `mnemonic seed-xor`.
//!
//! Per SPEC §2.2 — `phrase=-` is the SOLE stdin path; for `combine` at
//! most ONE `--share` may be `phrase=-` (single stdin per invocation).

use assert_cmd::Command;

const ABANDON_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn split_stdin_route_round_trip() {
    let split_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("split")
        .arg("--from")
        .arg("phrase=-")
        .arg("--shares")
        .arg("2")
        .arg("--deterministic-from-master")
        .write_stdin(ABANDON_12)
        .output()
        .unwrap();
    assert!(split_out.status.success());
    let stdout = String::from_utf8(split_out.stdout).unwrap();
    let shares: Vec<&str> = stdout.lines().collect();
    assert_eq!(shares.len(), 2);

    // Round-trip via combine (one share via stdin, one inline)
    let s_inline = format!("phrase={}", shares[0]);
    let combine_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("combine")
        .arg("--share")
        .arg(&s_inline)
        .arg("--share")
        .arg("phrase=-")
        .arg("--shares")
        .arg("2")
        .write_stdin(shares[1])
        .output()
        .unwrap();
    assert!(combine_out.status.success());
    let recovered = String::from_utf8(combine_out.stdout).unwrap();
    assert_eq!(recovered.lines().next().unwrap(), ABANDON_12);
}

#[test]
fn combine_refuses_two_stdin_shares() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("combine")
        .arg("--share")
        .arg("phrase=-")
        .arg("--share")
        .arg("phrase=-")
        .arg("--shares")
        .arg("2")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("at most one --share value may be `-`") || stderr.contains("single stdin"),
        "must refuse multi-stdin; got: {stderr}",
    );
}

#[test]
fn split_stdin_no_argv_advisory() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("split")
        .arg("--from")
        .arg("phrase=-")
        .arg("--shares")
        .arg("2")
        .arg("--deterministic-from-master")
        .write_stdin(ABANDON_12)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains("secret material on argv"),
        "stdin route must NOT emit argv-leakage advisory; got: {stderr}",
    );
}

#[test]
fn split_stdin_tolerates_trailing_newline() {
    let with_nl = format!("{ABANDON_12}\n");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("split")
        .arg("--from")
        .arg("phrase=-")
        .arg("--shares")
        .arg("2")
        .arg("--deterministic-from-master")
        .write_stdin(with_nl)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "trailing newline must not break parse"
    );
}

#[test]
fn stdin_route_equals_inline_route_for_split() {
    let from_arg = format!("phrase={ABANDON_12}");
    let inline = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("split")
        .arg("--from")
        .arg(&from_arg)
        .arg("--shares")
        .arg("2")
        .arg("--deterministic-from-master")
        .output()
        .unwrap();
    let stdin_form = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("split")
        .arg("--from")
        .arg("phrase=-")
        .arg("--shares")
        .arg("2")
        .arg("--deterministic-from-master")
        .write_stdin(ABANDON_12)
        .output()
        .unwrap();
    assert!(inline.status.success());
    assert!(stdin_form.status.success());
    assert_eq!(
        inline.stdout, stdin_form.stdout,
        "inline and stdin routes must produce byte-identical stdout",
    );
}
