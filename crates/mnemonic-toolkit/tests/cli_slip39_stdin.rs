//! v0.13.0 P2.2 — stdin-route tests for `mnemonic slip39`.
//!
//! Per SPEC §2.2 + plan §3.2 row 18 — N+1 stdin candidates (N `--share`
//! slots + `--from -` at split + `--passphrase-stdin`). Single stdin
//! consumer per invocation. Three pairwise refusal classes covered
//! exhaustively here; the canonical row-18 stem is also covered in
//! `cli_slip39_refusals.rs`.
//!
//! R0 Note 2 fold — silent-correctness pin: `--passphrase-stdin` uses
//! `read_stdin_passphrase` (preserves trailing whitespace + NULL),
//! NOT `read_stdin_to_string` (which `.trim()`s). Mismatching these is
//! silently wrong — a passphrase with trailing whitespace would yield
//! a different EMS at recovery.

use assert_cmd::Command;

const ABANDON_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

// ============================================================
// Happy path — split `--from phrase=-` round-trip
// ============================================================

#[test]
fn stdin_split_from_dash_round_trip() {
    let split_out = Command::cargo_bin("mnemonic")
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
    assert!(split_out.status.success(), "split via stdin must succeed");
    let stdout = String::from_utf8(split_out.stdout).unwrap();
    let shares: Vec<String> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    assert_eq!(shares.len(), 3);
    let combine_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("combine")
        .args([
            "--share",
            &shares[0],
            "--share",
            &shares[1],
            "--to",
            "phrase",
            "--language",
            "english",
        ])
        .output()
        .unwrap();
    assert!(combine_out.status.success());
    let recovered = String::from_utf8(combine_out.stdout).unwrap();
    assert_eq!(recovered.lines().next().unwrap(), ABANDON_12);
}

// ============================================================
// Happy path — combine with one `--share -` and N-1 inline
// ============================================================

#[test]
fn stdin_combine_one_share_dash_round_trip() {
    // Split first (no stdin) to get shares.
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
    assert!(split_out.status.success());
    let stdout = String::from_utf8(split_out.stdout).unwrap();
    let shares: Vec<String> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    let inline_share = &shares[0];
    let stdin_share = &shares[1];
    let combine_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("combine")
        .args([
            "--share",
            inline_share,
            "--share",
            "-",
            "--to",
            "phrase",
            "--language",
            "english",
        ])
        .write_stdin(stdin_share.clone())
        .output()
        .unwrap();
    assert!(
        combine_out.status.success(),
        "combine via stdin share must succeed"
    );
    let recovered = String::from_utf8(combine_out.stdout).unwrap();
    assert_eq!(recovered.lines().next().unwrap(), ABANDON_12);
}

// ============================================================
// Happy path — split with `--passphrase-stdin`
// ============================================================

#[test]
fn stdin_split_passphrase_stdin_round_trip() {
    let pp = "test-passphrase";
    let from_arg = format!("phrase={ABANDON_12}");
    let split_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--passphrase-stdin",
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .write_stdin(pp)
        .output()
        .unwrap();
    assert!(split_out.status.success());
    let stdout = String::from_utf8(split_out.stdout).unwrap();
    let shares: Vec<String> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    let combine_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("combine")
        .args([
            "--share",
            &shares[0],
            "--share",
            &shares[1],
            "--passphrase",
            pp,
            "--to",
            "phrase",
            "--language",
            "english",
        ])
        .output()
        .unwrap();
    assert!(combine_out.status.success());
    let recovered = String::from_utf8(combine_out.stdout).unwrap();
    assert_eq!(recovered.lines().next().unwrap(), ABANDON_12);
}

// ============================================================
// Happy path — combine with `--passphrase-stdin`
// ============================================================

#[test]
fn stdin_combine_passphrase_stdin_round_trip() {
    let pp = "test-passphrase";
    let from_arg = format!("phrase={ABANDON_12}");
    let split_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--passphrase",
            pp,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .output()
        .unwrap();
    assert!(split_out.status.success());
    let stdout = String::from_utf8(split_out.stdout).unwrap();
    let shares: Vec<String> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    let combine_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("combine")
        .args([
            "--share",
            &shares[0],
            "--share",
            &shares[1],
            "--passphrase-stdin",
            "--to",
            "phrase",
            "--language",
            "english",
        ])
        .write_stdin(pp)
        .output()
        .unwrap();
    assert!(combine_out.status.success());
    let recovered = String::from_utf8(combine_out.stdout).unwrap();
    assert_eq!(recovered.lines().next().unwrap(), ABANDON_12);
}

// ============================================================
// Refusal pairwise (a) — `--passphrase-stdin` + `--share -` (combine)
// ============================================================

#[test]
fn stdin_refusal_pairwise_a_combine_passphrase_stdin_plus_share_dash() {
    let placeholder = "academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic academic";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("combine")
        .args(["--share", placeholder, "--share", "-", "--passphrase-stdin"])
        .write_stdin("anything")
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(out.status.code(), Some(1), "exit 1; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)"
        ),
        "pairwise (a) must emit row 18 stem; got: {stderr}"
    );
}

// ============================================================
// Refusal pairwise (b) — `--passphrase-stdin` + `--from -` (split)
// ============================================================

#[test]
fn stdin_refusal_pairwise_b_split_passphrase_stdin_plus_from_dash() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            "phrase=-",
            "--passphrase-stdin",
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        .write_stdin("anything")
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(out.status.code(), Some(1), "exit 1; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)"
        ),
        "pairwise (b) must emit row 18 stem; got: {stderr}"
    );
}

// ============================================================
// Refusal pairwise (c) — two distinct `--share -` slots (combine)
// ============================================================

#[test]
fn stdin_refusal_pairwise_c_combine_two_share_dash() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("combine")
        .args(["--share", "-", "--share", "-"])
        .write_stdin("anything")
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(out.status.code(), Some(1), "exit 1; stderr={stderr:?}");
    assert!(
        stderr.contains(
            "slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)"
        ),
        "pairwise (c) must emit row 18 stem; got: {stderr}"
    );
}

// ============================================================
// R0 Note 2 silent-correctness pin — `--passphrase-stdin` MUST use
// `read_stdin_passphrase` (preserves trailing whitespace + NULL),
// NOT `read_stdin_to_string` (strips).
//
// Construction: split with passphrase containing trailing spaces via
// stdin; recover with the SAME passphrase via stdin. If the handler
// were (wrongly) using `read_stdin_to_string`, the trailing spaces
// would be stripped at split AND at combine — round-trip would work
// because BOTH sides strip identically. The bug only surfaces under
// asymmetric usage. To exercise THIS specific bug, we round-trip via
// passphrase-stdin on split + inline `--passphrase "<value with
// trailing space>"` on combine: the inline path preserves the
// trailing space; if `--passphrase-stdin` stripped, the EMS at split
// would differ from the EMS expected by combine → DigestVerificationFailed.
// ============================================================

#[test]
fn stdin_passphrase_stdin_preserves_trailing_whitespace_r0_note_2() {
    // Passphrase with trailing single space.
    let pp_with_trailing_space = "secret-pass ";
    let from_arg = format!("phrase={ABANDON_12}");
    // Split via --passphrase-stdin.
    let split_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--passphrase-stdin",
            "--group-threshold",
            "1",
            "--group",
            "3,2",
        ])
        // Write passphrase + trailing \n (the stdin-passphrase reader
        // strips ONLY a single trailing \r?\n — the trailing space
        // is preserved).
        .write_stdin(format!("{pp_with_trailing_space}\n"))
        .output()
        .unwrap();
    assert!(split_out.status.success(), "split must succeed");
    let split_stdout = String::from_utf8(split_out.stdout).unwrap();
    let shares: Vec<String> = split_stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    // Combine with inline --passphrase (the trailing space is
    // preserved exactly in the argv slot).
    let combine_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("combine")
        .args([
            "--share",
            &shares[0],
            "--share",
            &shares[1],
            "--passphrase",
            pp_with_trailing_space,
            "--to",
            "phrase",
            "--language",
            "english",
        ])
        .output()
        .unwrap();
    let combine_stderr = String::from_utf8(combine_out.stderr.clone()).unwrap();
    // R0 I-3 fold — clap-delivery precondition: pin the inline
    // combine's stderr contains the row-1e argv-leakage advisory,
    // proving clap delivered the byte-exact `"secret-pass "` (with
    // trailing space) to the handler. Without this precondition,
    // clap auto-trimming the trailing argv space (unlikely but
    // possible across clap versions) would render the entire test
    // premise invalid AND silently mask the foot-gun.
    assert!(
        combine_stderr.contains("warning: secret material on argv (--passphrase)"),
        "R0 I-3 precondition: inline --passphrase must surface row-1e \
         argv-leakage advisory (proves clap delivered the value to the \
         handler — if missing, clap may have stripped the trailing \
         space and the silent-correctness pin is invalid); got: {combine_stderr}"
    );
    assert!(
        combine_out.status.success(),
        "combine with byte-identical passphrase must succeed (R0 Note 2 pin); stderr={combine_stderr:?}"
    );
    let recovered = String::from_utf8(combine_out.stdout).unwrap();
    assert_eq!(recovered.lines().next().unwrap(), ABANDON_12);
}
