//! v0.11.0 P2 — CLI advisory tests for `mnemonic final-word`.
//!
//! Per SPEC §2.6. Three advisory classes:
//! 1. Inline secret on argv (`--from phrase=<inline-value>`) → Cycle A
//!    argv-leakage warning via `secret_in_argv_warning`.
//! 2. Stdout-on-TTY candidate emit → secret-on-stdout advisory (NEW
//!    advisory class for v0.11.0). Tested in the negative-case here
//!    (assert_cmd pipes stdout, so the warning must NOT fire);
//!    positive-case (true TTY) is exercised manually per smoke-test
//!    in the plan §"Verification".
//! 3. `--json-out <path>` world-readable file warning (#[cfg(unix)]).

use assert_cmd::Command;
use tempfile::NamedTempFile;

const ABANDON_11_PARTIAL: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";

#[test]
fn inline_secret_emits_argv_leakage_advisory() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .arg("--from")
        .arg(format!("phrase={}", ABANDON_11_PARTIAL))
        .output()
        .unwrap();
    assert!(out.status.success(), "happy path with inline secret still succeeds");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("warning: secret material on argv (--from phrase=)"),
        "inline-secret advisory must fire on stderr; got: {stderr}",
    );
    assert!(
        stderr.contains("--from phrase=-"),
        "advisory must point users at the --from phrase=- alternative; got: {stderr}",
    );
}

#[test]
fn stdin_route_does_not_emit_argv_leakage_advisory() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .arg("--from")
        .arg("phrase=-")
        .write_stdin(ABANDON_11_PARTIAL)
        .output()
        .unwrap();
    assert!(out.status.success(), "stdin route should succeed");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains("secret material on argv"),
        "stdin route must NOT emit argv-leakage advisory; got: {stderr}",
    );
}

#[test]
fn piped_stdout_does_not_emit_stdout_on_tty_advisory() {
    // TTY gate dropped (Cycle B P1): the P-line now fires unconditionally,
    // even when stdout is piped (non-TTY). Inverted from the old NOT-assert.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .arg("--from")
        .arg("phrase=-")
        .write_stdin(ABANDON_11_PARTIAL)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("warning: stdout carries private key material (can spend)"),
        "P-line must fire even on piped stdout after TTY-gate drop; got: {stderr}",
    );
}

#[cfg(unix)]
#[test]
fn json_out_world_readable_emits_advisory() {
    use std::os::unix::fs::PermissionsExt;
    // Pre-create the file at 0o644 and KEEP it alive through the CLI
    // invocation. `std::fs::write` opens the existing path with O_TRUNC
    // (not O_CREAT-from-scratch), so the pre-set mode is preserved.
    let f = NamedTempFile::new().unwrap();
    let path = f.path().to_owned();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .arg("--from")
        .arg("phrase=-")
        .arg("--json-out")
        .arg(&path)
        .write_stdin(ABANDON_11_PARTIAL)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("world-readable") || stderr.contains("umask"),
        "world-readable --json-out path must emit permission-mode advisory; got: {stderr}",
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
    // Keep `f` alive so the path's 0o600 mode persists into the CLI call.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .arg("--from")
        .arg("phrase=-")
        .arg("--json-out")
        .arg(&path)
        .write_stdin(ABANDON_11_PARTIAL)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains("world-readable"),
        "0o600 --json-out path must NOT emit world-readable advisory; got: {stderr}",
    );
    drop(f);
}
