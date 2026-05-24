//! Integration tests for `mnemonic repair --max-indel` — Phase 5 of the
//! m-format incorrect-length (indel) recovery cycle.
//!
//! Realizes `design/IMPLEMENTATION_PLAN_m_format_incorrect_length_recovery.md`
//! §3 Phase 5 + §4.3 (exit 0/5/2; --max-indel 0 regression; clap range; md1
//! refusal; --json shape; multi-group no-skip aggregation).
//!
//! **Ambiguous (exit 4) has NO integration cell here:** the BCH checksum
//! (13-symbol regular / 15-symbol long) makes a 2-distinct-recovered collision
//! within --max-indel ≤ 4 cryptographically unreachable with real vectors
//! (~2⁻⁶⁵). Exit-4 mapping is covered by the `indel_exit_code` unit test
//! (`src/repair.rs`) + the Phase-3 engine `recover_indel_reports_ambiguous`
//! test; the Ambiguous *emit* path is covered by a unit test in
//! `src/cmd/repair.rs`.
//!
//! Fixtures: known-valid m-format strings reused from `src/repair.rs::tests`
//! / `tests/cli_repair.rs`. `MNEMONIC_FORCE_TTY=1` is set on each command for
//! deterministic advisory behavior (mirrors `cli_repair.rs` convention).

use assert_cmd::Command;
use predicates::prelude::*;

const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const MK1_C0: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
const MK1_C1: &str = "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";

/// Remove the data-part char at data-index `i` (full-string index `3 + i`,
/// after the 3-char `xx1` prefix), simulating a dropped (too-short) char.
fn drop_data(s: &str, i: usize) -> String {
    let mut out = String::from(s);
    out.remove(3 + i);
    out
}

/// Insert char `c` at data-index `i` (full-string index `3 + i`), simulating
/// an added (too-long) char.
fn ins_data(s: &str, i: usize, c: char) -> String {
    let mut out = String::from(s);
    out.insert(3 + i, c);
    out
}

/// Build a command with the deterministic-advisory TTY env set.
fn cmd() -> Command {
    let mut c = Command::cargo_bin("mnemonic").unwrap();
    c.env("MNEMONIC_FORCE_TTY", "1");
    c
}

/// Test 1 — ms1 too-long (one inserted data char) → exit 5; recovered string on stdout.
#[test]
fn ms1_too_long_recovers_exit_5() {
    let bad = ins_data(VALID_MS1, 10, 'q');
    cmd()
        .args(["repair", "--ms1", &bad, "--max-indel", "1"])
        .assert()
        .code(5)
        .stdout(predicate::str::contains(VALID_MS1));
}

/// Test 2 — ms1 too-short (one dropped data char) → exit 5; recovered string on stdout.
#[test]
fn ms1_too_short_recovers_exit_5() {
    let bad = drop_data(VALID_MS1, 1); // drop 'e' (non-'q' → BCH solves)
    cmd()
        .args(["repair", "--ms1", &bad, "--max-indel", "1"])
        .assert()
        .code(5)
        .stdout(predicate::str::contains(VALID_MS1));
}

/// Test 3 — ms1 PREFIX dropped-'m' → exit 5 (proves the HrpMismatch trigger
/// amendment, §1.7). "s10entrs…" = VALID_MS1 with the leading 'm' gone.
#[test]
fn ms1_prefix_dropped_m_recovers_exit_5() {
    let bad = VALID_MS1.strip_prefix('m').unwrap(); // "s10entrs…"
    cmd()
        .args(["repair", "--ms1", bad, "--max-indel", "1"])
        .assert()
        .code(5)
        .stdout(predicate::str::contains(VALID_MS1));
}

/// Test 4 — mk1 multi-chunk, one corrupted (too-long) chunk → exit 5;
/// recovered chunk on stdout.
#[test]
fn mk1_multichunk_one_corrupted_recovers_exit_5() {
    let bad_c1 = ins_data(MK1_C1, 12, 'q');
    cmd()
        .args(["repair", "--mk1", MK1_C0, "--mk1", &bad_c1, "--max-indel", "1"])
        .assert()
        .code(5)
        .stdout(predicate::str::contains(MK1_C1));
}

/// Test 5 — already-valid ms1 + --max-indel 1 → exit 0 (no indel needed; the
/// normal repair_card Ok path with zero corrections).
#[test]
fn ms1_already_valid_with_max_indel_exit_0() {
    cmd()
        .args(["repair", "--ms1", VALID_MS1, "--max-indel", "1"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains(VALID_MS1));
}

/// Test 6 — unrecoverable: 3 inserted chars but --max-indel 1 → exit 2; stderr
/// cites the indel-unrecoverable message ("within --max-indel").
#[test]
fn ms1_unrecoverable_within_budget_exit_2() {
    // Three inserted chars (remove from high index first so the indices stay
    // valid as we splice). Use ins_data left-to-right at spread indices.
    let mut bad = ins_data(VALID_MS1, 10, 'q');
    bad = ins_data(&bad, 20, 'p');
    bad = ins_data(&bad, 30, 'z');
    cmd()
        .args(["repair", "--ms1", &bad, "--max-indel", "1"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("within --max-indel"));
}

/// Test 7 — --max-indel 0 regression: a too-long ms1 with the flag at 0 must
/// NOT enter indel mode — identical to today's behavior (exit 2, a repair
/// error, NOT an indel message). The key default-path-untouched guard.
#[test]
fn max_indel_0_is_unchanged_no_indel_mode() {
    let bad = ins_data(VALID_MS1, 10, 'q');
    cmd()
        .args(["repair", "--ms1", &bad, "--max-indel", "0"])
        .assert()
        .code(2)
        // A normal repair-error path (NOT indel). An inserted char shifts the
        // tail → far over t=4 → TooManyErrors.
        .stderr(predicate::str::contains("too many errors"))
        .stderr(predicate::str::contains("within --max-indel").not());
}

/// Test 8 — --max-indel 5 → clap usage error (exit 2); stderr mentions the 0..=4 range.
#[test]
fn max_indel_5_is_clap_usage_error() {
    cmd()
        .args(["repair", "--ms1", VALID_MS1, "--max-indel", "5"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("0..=4"));
}

/// Test 9 — md1 refusal → exit 1 (BadInput); stderr cites the not-yet-supported
/// message. A short md1 triggers (parse fails → trigger), then refusal.
#[test]
fn md1_indel_refusal_exit_1() {
    cmd()
        .args(["repair", "--md1", "md1xxxx", "--max-indel", "1"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "not yet supported for chunked md1",
        ));
}

/// Test 10 — --json unique shape: status "unique" + candidates[0].recovered == VALID_MS1.
#[test]
fn json_unique_envelope_shape() {
    let bad = ins_data(VALID_MS1, 10, 'q');
    let out = cmd()
        .args(["repair", "--ms1", &bad, "--max-indel", "1", "--json"])
        .assert()
        .code(5)
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON envelope");
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["status"], "unique");
    assert_eq!(v["candidates"][0]["recovered"], VALID_MS1);
}

/// Test 11 — multi-group both emit → exit 5; stdout contains BOTH the recovered
/// VALID_MS1 (indel path) AND the valid passthrough mk1 chunks (normal path).
/// Guards R0 I1 — no group is skipped by the indel branch's control flow.
#[test]
fn multi_group_both_emit_exit_5() {
    let bad_ms1 = ins_data(VALID_MS1, 10, 'q');
    cmd()
        .args([
            "repair",
            "--ms1",
            &bad_ms1,
            "--mk1",
            MK1_C0,
            "--mk1",
            MK1_C1,
            "--max-indel",
            "1",
        ])
        .assert()
        .code(5)
        .stdout(predicate::str::contains(VALID_MS1))
        .stdout(predicate::str::contains(MK1_C0))
        .stdout(predicate::str::contains(MK1_C1));
}
