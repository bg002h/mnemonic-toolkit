//! v0.12.0 P2 — CLI advisory tests for `mnemonic seed-xor`.
//!
//! Per SPEC §2.6 — 5 advisory classes.

use assert_cmd::Command;
use tempfile::NamedTempFile;

const ABANDON_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn zero_entropy_15_word() -> String {
    bip39::Mnemonic::from_entropy_in(bip39::Language::English, &[0u8; 20])
        .unwrap()
        .to_string()
}

#[test]
fn split_inline_emits_argv_leakage_advisory() {
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("split")
        .arg("--from")
        .arg(&from_arg)
        .arg("--shares")
        .arg("2")
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("warning: secret material on argv (--from phrase=)"),
        "split inline must emit argv-leakage advisory; got: {stderr}",
    );
    assert!(stderr.contains("--from phrase=-"));
}

#[test]
fn split_stdin_does_not_emit_argv_advisory() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("split")
        .arg("--from")
        .arg("phrase=-")
        .arg("--shares")
        .arg("2")
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
fn combine_inline_share_emits_argv_leakage_advisory_per_share() {
    let s = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("combine")
        .arg("--share")
        .arg(&s)
        .arg("--share")
        .arg(&s)
        .arg("--shares")
        .arg("2")
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    // 2 inline shares → 2 argv-leakage advisories (per-occurrence, NOT deduped)
    let count = stderr.matches("warning: secret material on argv (--share phrase=)").count();
    assert_eq!(count, 2, "must emit per-occurrence advisory; got {count} advisories in: {stderr}");
}

#[test]
fn piped_stdout_does_not_emit_kofn_tty_advisory() {
    // assert_cmd pipes stdout → IsTerminal false → no K-of-N advisory
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
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
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains("Seed XOR shares on stdout"),
        "K-of-N TTY advisory must NOT fire when stdout is piped; got: {stderr}",
    );
}

#[test]
fn piped_stdout_combine_does_not_emit_tty_advisory() {
    let from_arg = format!("phrase={ABANDON_12}");
    let split_out = Command::cargo_bin("mnemonic")
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
    let split_stdout = String::from_utf8(split_out.stdout).unwrap();
    let shares: Vec<&str> = split_stdout.lines().collect();
    let s0 = format!("phrase={}", shares[0]);
    let s1 = format!("phrase={}", shares[1]);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("combine")
        .arg("--share")
        .arg(&s0)
        .arg("--share")
        .arg(&s1)
        .arg("--shares")
        .arg("2")
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains("combined phrase is secret material"),
        "combine TTY advisory must NOT fire when stdout is piped; got: {stderr}",
    );
}

#[test]
fn deterministic_with_15_word_emits_toolkit_only_advisory() {
    let phrase_15 = zero_entropy_15_word();
    let from_arg = format!("phrase={phrase_15}");
    let out = Command::cargo_bin("mnemonic")
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
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("--deterministic-from-master with 15-word input is toolkit-only"),
        "15-word + deterministic must trigger toolkit-only advisory; got: {stderr}",
    );
}

#[test]
fn deterministic_with_12_word_does_not_emit_toolkit_only_advisory() {
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
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
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains("toolkit-only"),
        "12-word + deterministic must NOT trigger toolkit-only advisory; got: {stderr}",
    );
}

#[cfg(unix)]
#[test]
fn json_out_world_readable_emits_advisory() {
    use std::os::unix::fs::PermissionsExt;
    let f = NamedTempFile::new().unwrap();
    let path = f.path().to_owned();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("split")
        .arg("--from")
        .arg(&from_arg)
        .arg("--shares")
        .arg("2")
        .arg("--deterministic-from-master")
        .arg("--json-out")
        .arg(&path)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("world-readable") || stderr.contains("umask"),
        "world-readable --json-out must emit advisory; got: {stderr}",
    );
    drop(f);
}

#[cfg(unix)]
#[test]
fn json_out_0o600_does_not_emit_advisory() {
    use std::os::unix::fs::PermissionsExt;
    let f = NamedTempFile::new().unwrap();
    let path = f.path().to_owned();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("split")
        .arg("--from")
        .arg(&from_arg)
        .arg("--shares")
        .arg("2")
        .arg("--deterministic-from-master")
        .arg("--json-out")
        .arg(&path)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains("world-readable"),
        "0o600 --json-out must NOT emit advisory; got: {stderr}",
    );
    drop(f);
}
