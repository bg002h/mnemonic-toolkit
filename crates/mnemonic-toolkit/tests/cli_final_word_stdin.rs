//! v0.11.0 P2 — CLI stdin-route tests for `mnemonic final-word`.
//!
//! Per SPEC §2.2 + R0 round 1 C1 resolution: `--from phrase=-` is the
//! SOLE stdin path (no paired `--phrase-stdin` flag). These tests
//! exercise the stdin route end-to-end and confirm it is functionally
//! equivalent to the inline `--from phrase=<value>` path.

use assert_cmd::Command;

const ABANDON_11_PARTIAL: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";

const BEEF_11_PARTIAL: &str = "beef beef beef beef beef beef beef beef beef beef beef";

#[test]
fn stdin_route_abandon_11_emits_128_sorted_lines() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .arg("--from")
        .arg("phrase=-")
        .arg("--language")
        .arg("english")
        .write_stdin(ABANDON_11_PARTIAL)
        .output()
        .unwrap();
    assert!(out.status.success(), "stdin route should succeed");
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        128,
        "abandon×11 via stdin must emit 128 candidates"
    );
    let mut sorted = lines.clone();
    sorted.sort();
    assert_eq!(lines, sorted, "stdout must be lexicographically sorted");
    assert!(
        lines.iter().any(|w| *w == "about"),
        "stdin route preserves abandon×11 canonical Trezor anchor (about)",
    );
}

#[test]
fn stdin_route_beef_11_emits_128_sorted_lines() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .arg("--from")
        .arg("phrase=-")
        .arg("--language")
        .arg("english")
        .write_stdin(BEEF_11_PARTIAL)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        128,
        "beef×11 via stdin must emit 128 candidates"
    );
}

#[test]
fn stdin_route_equals_inline_route_byte_for_byte() {
    // The same partial, routed via stdin vs argv, must yield byte-identical
    // stdout. Stderr will DIFFER (inline route emits the argv-leakage
    // advisory) — we compare stdout only.
    let stdin_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .arg("--from")
        .arg("phrase=-")
        .arg("--language")
        .arg("english")
        .write_stdin(ABANDON_11_PARTIAL)
        .output()
        .unwrap();
    let inline_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .arg("--from")
        .arg(format!("phrase={}", ABANDON_11_PARTIAL))
        .arg("--language")
        .arg("english")
        .output()
        .unwrap();
    assert!(stdin_out.status.success());
    assert!(inline_out.status.success());
    assert_eq!(
        stdin_out.stdout, inline_out.stdout,
        "stdin and inline routes must produce byte-identical stdout",
    );
}

#[test]
fn stdin_route_with_trailing_newline_in_input() {
    // Real-world stdin typically has a trailing newline; the parser must
    // tolerate it. (split_whitespace handles this naturally.)
    let with_nl = format!("{}\n", ABANDON_11_PARTIAL);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .arg("--from")
        .arg("phrase=-")
        .arg("--language")
        .arg("english")
        .write_stdin(with_nl)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "trailing newline in stdin must not break parse"
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 128);
}

#[test]
fn stdin_route_with_extra_whitespace() {
    // Tab + double-space + leading space — split_whitespace tolerates all.
    let messy = format!("  {}  ", ABANDON_11_PARTIAL.replace(' ', "\t"));
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .arg("--from")
        .arg("phrase=-")
        .arg("--language")
        .arg("english")
        .write_stdin(messy)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "messy whitespace must not break parse"
    );
    assert_eq!(String::from_utf8(out.stdout).unwrap().lines().count(), 128);
}

#[test]
fn stdin_route_does_not_emit_argv_advisory() {
    // Sanity: stdin route must NOT trip the inline-secret advisory.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .arg("--from")
        .arg("phrase=-")
        .arg("--language")
        .arg("english")
        .write_stdin(ABANDON_11_PARTIAL)
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains("secret material on argv"),
        "stdin route must NOT emit argv-leakage advisory; got: {stderr}",
    );
}
