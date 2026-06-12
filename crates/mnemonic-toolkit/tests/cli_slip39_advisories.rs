//! v0.13.0 P2.2 — CLI advisory tests for `mnemonic slip39`.
//!
//! Per SPEC §2.6 + plan §3.3 (R0 I3 fold — 8 wiring sites for 5
//! semantic advisory classes + 1 NEW env-var class per Q2 fold).
//! Argv-leakage advisory stem byte-pinned from
//! `secret_advisory::secret_in_argv_warning`. TTY-conditional
//! advisories are negative-only here (assert_cmd always pipes;
//! positive TTY-branch coverage is out of scope for piped integration
//! tests — see plan §4.1 row + §6 risk 4).
//!
//! All tests FAIL at RED — `cmd/slip39.rs` returns a P2.1 stub
//! `ToolkitError::BadInput` until P2.2 GREEN lands the handler impl.

use assert_cmd::Command;

const ABANDON_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

const ENTROPY_16_ZEROS_HEX: &str = "00000000000000000000000000000000";

/// 32 bytes hex (64 zero chars) — used as the
/// `MNEMONIC_SLIP39_TEST_RNG` seed value.
const TEST_RNG_HEX_64: &str = "0000000000000000000000000000000000000000000000000000000000000000";

// ============================================================
// Row 1a — argv-leakage: split --from phrase= (inline)
// ============================================================

#[test]
fn advisory_split_inline_phrase_emits_argv_leakage_1a() {
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains(
            "warning: secret material on argv (--from phrase=) — pipe via --from phrase=- to avoid /proc/$PID/cmdline exposure"
        ),
        "expected row 1a stem byte-faithful; got: {stderr}"
    );
}

// ============================================================
// Row 1b — argv-leakage: split --from entropy= (inline)
// ============================================================

#[test]
fn advisory_split_inline_entropy_emits_argv_leakage_1b() {
    let from_arg = format!("entropy={ENTROPY_16_ZEROS_HEX}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains(
            "warning: secret material on argv (--from entropy=) — pipe via --from entropy=- to avoid /proc/$PID/cmdline exposure"
        ),
        "expected row 1b stem byte-faithful; got: {stderr}"
    );
}

// ============================================================
// Row 1c — argv-leakage: split --passphrase (inline)
// ============================================================

#[test]
fn advisory_split_inline_passphrase_emits_argv_leakage_1c() {
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--passphrase",
            "secret-pass",
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains(
            "warning: secret material on argv (--passphrase) — pipe via --passphrase-stdin to avoid /proc/$PID/cmdline exposure"
        ),
        "expected row 1c stem byte-faithful; got: {stderr}"
    );
}

// ============================================================
// Row 1c — R0 C1 fold pin: empty-string `--passphrase ""` STILL
// triggers the argv-leakage advisory because the structural distinction
// is "user supplied the flag" (Option::is_some), NOT "value non-empty".
// ============================================================

#[test]
fn advisory_split_empty_passphrase_still_emits_argv_leakage_per_r0_c1() {
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--passphrase",
            "",
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("warning: secret material on argv (--passphrase)"),
        "R0 C1 fold: empty-string --passphrase must still fire 1c; got: {stderr}"
    );
}

// ============================================================
// Row 1d — argv-leakage: combine --share (per-occurrence, NOT deduped)
// ============================================================

#[test]
fn advisory_combine_inline_share_per_occurrence_argv_leakage_1d() {
    // 2 inline --share occurrences → 2 advisories (per-occurrence).
    // Use any two strings; combine will likely refuse on parse, but
    // the argv-leakage advisory fires BEFORE the refusal.
    let placeholder_a = "academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic";
    let placeholder_b = "academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("combine")
        .args(["--share", placeholder_a, "--share", placeholder_b])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    let count = stderr
        .matches("warning: secret material on argv (--share)")
        .count();
    assert_eq!(
        count, 2,
        "per-occurrence not deduped — 2 --share inlines must emit 2 advisories; got {count} in: {stderr}"
    );
    // Also assert the stem byte-faithfully on at least one occurrence
    assert!(
        stderr.contains(
            "warning: secret material on argv (--share) — pipe via --share - to avoid /proc/$PID/cmdline exposure"
        ),
        "expected row 1d stem byte-faithful; got: {stderr}"
    );
}

// ============================================================
// Row 1e — argv-leakage: combine --passphrase (inline)
// ============================================================

#[test]
fn advisory_combine_inline_passphrase_emits_argv_leakage_1e() {
    // Combine likely refuses (no real shares), but the argv-leakage
    // advisory fires BEFORE parsing.
    let placeholder = "academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("combine")
        .args(["--share", placeholder, "--passphrase", "secret-pass"])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains(
            "warning: secret material on argv (--passphrase) — pipe via --passphrase-stdin to avoid /proc/$PID/cmdline exposure"
        ),
        "expected row 1e stem byte-faithful; got: {stderr}"
    );
}

// ============================================================
// NEGATIVE: split via stdin (`--from phrase=-`) does NOT emit
// argv-leakage advisory.
// ============================================================

#[test]
fn advisory_split_stdin_route_does_not_emit_argv_leakage() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            "phrase=-",
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .write_stdin(ABANDON_12)
        .output()
        .unwrap();
    assert!(out.status.success(), "expected success on stdin route");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains("secret material on argv"),
        "stdin route must NOT emit argv-leakage advisory; got: {stderr}"
    );
}

// ============================================================
// Row 2 — K-of-N stdout-on-TTY advisory (NEGATIVE: piped → silent)
// ============================================================

#[test]
fn advisory_k_of_n_stdout_silent_when_piped() {
    // TTY gate dropped (Cycle B P1): the P-line now fires unconditionally,
    // even when stdout is piped (non-TTY). Inverted from the old NOT-assert.
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("warning: stdout carries private key material (can spend)"),
        "P-line must fire even on piped stdout after TTY-gate drop; got: {stderr}"
    );
}

// ============================================================
// Row 3 — Combine reconstructed-secret stdout-on-TTY (NEGATIVE: piped)
// ============================================================

#[test]
fn advisory_combine_reconstructed_silent_when_piped() {
    // TTY gate dropped (Cycle B P1): the P-line now fires unconditionally,
    // even when stdout is piped (non-TTY). Inverted from the old NOT-assert.
    let from_arg = format!("phrase={ABANDON_12}");
    let split_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .output()
        .unwrap();
    if !split_out.status.success() {
        panic!(
            "slip39 split failed: {}",
            String::from_utf8_lossy(&split_out.stderr)
        );
    }
    let split_stdout = String::from_utf8(split_out.stdout).unwrap();
    let shares: Vec<&str> = split_stdout.lines().filter(|l| !l.is_empty()).collect();
    assert!(shares.len() >= 2, "expected >=2 shares from 3,2 group");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("combine")
        .args(["--share", shares[0], "--share", shares[1]])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("warning: stdout carries private key material (can spend)"),
        "P-line must fire even on piped stdout after TTY-gate drop; got: {stderr}"
    );
}

// ============================================================
// Row 4 — `--json-out` to world-readable path emits advisory
// ============================================================

#[cfg(unix)]
#[test]
fn advisory_json_out_world_readable_emits_row_4() {
    use std::os::unix::fs::PermissionsExt;
    let f = tempfile::NamedTempFile::new().unwrap();
    let path = f.path().to_owned();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
            "--json-out",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(out.status.success(), "json-out split must succeed");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("world-readable") || stderr.contains("umask"),
        "world-readable --json-out must emit row 4 advisory; got: {stderr}"
    );
    drop(f);
}

#[cfg(unix)]
#[test]
fn advisory_json_out_0o600_does_not_emit_row_4() {
    use std::os::unix::fs::PermissionsExt;
    let f = tempfile::NamedTempFile::new().unwrap();
    let path = f.path().to_owned();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
            "--json-out",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains("world-readable"),
        "0o600 --json-out must NOT emit row 4 advisory; got: {stderr}"
    );
    drop(f);
}

// ============================================================
// Row 5 — G9 iteration-exponent threshold (E >= 5)
// ============================================================

#[test]
fn advisory_iteration_exponent_5_emits_g9_row_5() {
    // R0 I-2 fold — byte-pin more of the stem (was 3 substrings; now 5)
    // to guard against regressions that drop the Trezor reference or
    // the E>=10 hardware warning. Plan §3.3 row 5 vs SPEC §2.6 row 5
    // disagree on the `<iters> × ` (space) vs `<iters>×` (no space)
    // form; this test pins the plan's space-separated form, and the
    // plan §5 P2.2 GREEN SPEC-patches list grows a §2.6 row 5
    // reconciliation patch (per R0 I-2 fold) to land in lockstep.
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--iteration-exponent",
            "5",
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("warning: --iteration-exponent E=5 yields 320000 × PBKDF2-HMAC-SHA-256 iterations"),
        "E=5 G9 advisory must byte-pin the lead phrase with the plan §3.3 space-separated form; got: {stderr}"
    );
    assert!(
        stderr.contains("Trezor's reference uses E=1 (20000 iters) as default"),
        "G9 advisory must include the Trezor reference; got: {stderr}"
    );
    assert!(
        stderr.contains("E >= 10 may exceed 30s on weak hardware"),
        "G9 advisory must include the E>=10 hardware warning; got: {stderr}"
    );
}

#[test]
fn advisory_iteration_exponent_4_does_not_emit_g9_row_5() {
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--iteration-exponent",
            "4",
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains("PBKDF2-HMAC-SHA-256"),
        "E=4 (below threshold) must NOT emit row 5 advisory; got: {stderr}"
    );
}

// ============================================================
// Row 6 — `MNEMONIC_SLIP39_TEST_RNG` env-var always-on insecurity
// advisory (per Q2 fold).
// ============================================================

#[test]
fn advisory_env_var_test_rng_set_emits_row_6_always_on() {
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_SLIP39_TEST_RNG", TEST_RNG_HEX_64)
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains(
            "warning: MNEMONIC_SLIP39_TEST_RNG set — output is deterministic and INSECURE; do not use for real shares"
        ),
        "env-var TEST_RNG set must emit row 6 always-on insecurity advisory byte-faithfully; got: {stderr}"
    );
}

#[test]
fn advisory_env_var_test_identifier_alone_emits_row_6_always_on() {
    // Per plan §3.4 Q2: when EITHER env-var is set, the same advisory
    // stem fires (the canonical anchor names TEST_RNG even if only
    // _IDENTIFIER is set).
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_SLIP39_TEST_IDENTIFIER", "12345")
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("MNEMONIC_SLIP39_TEST_RNG set")
            && stderr.contains("deterministic")
            && stderr.contains("INSECURE"),
        "env-var _IDENTIFIER alone must still emit row 6 advisory (canonical TEST_RNG anchor); got: {stderr}"
    );
}
