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

/// Replace the data-part char at data-index `i` (full-string index `3 + i`)
/// with the next char in the bech32 alphabet (cyclic), simulating a
/// substitution (wrong-but-in-place) transcription error.
fn flip_data(s: &str, i: usize) -> String {
    const BECH32_CHARSET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let mut out: Vec<u8> = s.bytes().collect();
    let full_idx = 3 + i;
    let c = out[full_idx];
    let pos = BECH32_CHARSET.iter().position(|&b| b == c).expect("char in bech32 alphabet");
    out[full_idx] = BECH32_CHARSET[(pos + 1) % BECH32_CHARSET.len()];
    String::from_utf8(out).unwrap()
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

/// Test 9 — md1 multi-chunk indel recovery (v0.37.2; md1 un-refused). Corrupt
/// ONE chunk of a real 3-chunk md1 card; recover_indel locates + restores it.
#[test]
fn md1_multichunk_one_corrupted_recovers_exit_5() {
    const MD1_C0: &str = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
    const MD1_C1: &str = "md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d";
    const MD1_C2: &str = "md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn";
    let bad_c1 = ins_data(MD1_C1, 12, 'q'); // one inserted data char
    cmd()
        .args(["repair", "--md1", MD1_C0, "--md1", &bad_c1, "--md1", MD1_C2, "--max-indel", "1"])
        .assert()
        .code(5)
        .stdout(predicate::str::contains(MD1_C1));
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

/// Test 12 — ms1 successful indel recovery fires the secret-material stderr advisory.
/// Folds Phase-5 review Minor m-adv: the advisory was asserted to fire by code
/// inspection (`any_ms1 = true` → `secret_on_stdout_warning`) but had no
/// integration assertion. MNEMONIC_FORCE_TTY=1 ensures deterministic TTY-positive
/// advisory emission (mirrors cli_bundle_slip0132_info.rs convention).
#[test]
fn ms1_indel_recovery_fires_secret_advisory() {
    const ADVISORY: &str =
        "warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')";
    let bad = ins_data(VALID_MS1, 10, 'q');
    cmd()
        .args(["repair", "--ms1", &bad, "--max-indel", "1"])
        .assert()
        .code(5)
        .stdout(predicate::str::contains(VALID_MS1))
        .stderr(predicate::str::contains(ADVISORY));
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

// ============================================================================
// Phase 3 — --max-subst CLI surface tests
// ============================================================================

/// Phase 3 Test A — ms1 with one dropped data char PLUS one substituted data
/// char, recovered under --max-indel 1 --max-subst 1 → Unique, subst_count=1,
/// exit 4 + verify WARNING on stderr.
///
/// Construction: drop data-index 1 ('e') to make it too short by 1, then flip
/// data-index 5 (in the corrupted string) to introduce a substitution.
/// With --max-subst 1, the engine tolerates the substitution beyond the
/// placeholder, yielding a subst_count=1 candidate → exit 4 + WARNING.
#[test]
fn ms1_indel_plus_subst_exit_4_with_verify_warning() {
    // Step 1: drop data index 1 → too short by 1
    let dropped = drop_data(VALID_MS1, 1);
    // Step 2: flip data index 5 in the already-dropped string → substitution
    let bad = flip_data(&dropped, 5);
    cmd()
        .args(["repair", "--ms1", &bad, "--max-indel", "1", "--max-subst", "1"])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("verify it controls your funds"));
}

/// Phase 3 Test B — no-op notice: --max-subst 1 without --max-indel (default 0)
/// emits the "no effect without --max-indel" notice on stderr, and exits 0
/// (the string is valid — only the notice is asserted here).
#[test]
fn ms1_max_subst_without_indel_is_noop_notice() {
    cmd()
        .args(["repair", "--ms1", VALID_MS1, "--max-subst", "1"])
        .assert()
        .stderr(predicate::str::contains("no effect without --max-indel"));
}

/// Phase 3 Test C — --max-subst 0 regression: a pure-indel recovery under the
/// default --max-subst 0 still exits 5 (byte-identical to v0.37.2 behavior).
#[test]
fn ms1_max_subst_0_regression_pure_indel_exit_5() {
    let bad = ins_data(VALID_MS1, 10, 'q'); // too long by 1
    cmd()
        .args(["repair", "--ms1", &bad, "--max-indel", "1"]) // --max-subst defaults 0
        .assert()
        .code(5);
}

/// Phase 3 Test D — --max-subst 5 is rejected by clap (range 0..=4).
#[test]
fn max_subst_5_rejected_by_clap() {
    cmd()
        .args(["repair", "--ms1", VALID_MS1, "--max-indel", "1", "--max-subst", "5"])
        .assert()
        .failure();
}

/// Phase 3 Test E — --json with a subst-bearing recovery: confident == false,
/// candidates[0].subst_count == 1.
#[test]
fn ms1_indel_plus_subst_json_confident_false() {
    let dropped = drop_data(VALID_MS1, 1);
    let bad = flip_data(&dropped, 5);
    let out = cmd()
        .args(["repair", "--ms1", &bad, "--max-indel", "1", "--max-subst", "1", "--json"])
        .assert()
        .code(4)
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON envelope");
    assert_eq!(v["confident"], false, "confident should be false for subst-bearing recovery");
    assert_eq!(v["candidates"][0]["subst_count"], 1, "subst_count should be 1");
}

// ============================================================================
// Phase 4 — HrpMismatch suggestion-fallback
// ============================================================================

/// Phase 4 Test A — genuine wrong-HRP value passed to --ms1 with --max-indel 1:
/// indel search fails (it's a real mk1 string passed to --ms1, not an ms1
/// indel) and the original HrpMismatch message is surfaced instead of the
/// generic "could not be recovered within --max-indel" message.
///
/// `mk1qprsqhpqqsq3cq...` is a valid mk1 chunk passed to `--ms1`. With
/// --max-indel 1 the strict HRP pre-gate is relaxed, `repair_card` surfaces
/// `RepairError::HrpMismatch { found: "mk" }`, `is_indel_trigger` fires, but
/// `recover_indel_card` returns `Unrecoverable` (genuine HRP typo, not an
/// ms1 indel). Phase 4 falls back to the original HrpMismatch Display which
/// contains "HRP mismatch" (the stable substring always present in the
/// Display; "did you mean" only fires when found-HRP is Levenshtein-1 from
/// exactly one known HRP — "mk" is equidistant from both "ms" and "md" so no
/// suggestion appears here, but the HRP-mismatch message itself is restored).
#[test]
fn genuine_wrong_hrp_falls_back_to_suggestion_not_indel_unrecoverable() {
    cmd()
        .args(["repair", "--ms1", MK1_C0, "--max-indel", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("HRP mismatch"))
        .stderr(predicate::str::contains("could not be recovered within --max-indel").not());
}

/// Phase 4 Test B (regression) — a recoverable prefix-drop (dropping the
/// leading 'm' from a valid ms1 string) still recovers with exit 5 when
/// --max-indel 1 is set. This confirms the HrpMismatch fallback ONLY fires
/// on genuine indel-Unrecoverable cases, not on the prefix-drop case where
/// the indel engine succeeds.
#[test]
fn prefix_drop_still_recovers_under_phase4() {
    let bad = VALID_MS1.strip_prefix('m').unwrap(); // "s10entrs…"
    cmd()
        .args(["repair", "--ms1", bad, "--max-indel", "1"])
        .assert()
        .code(5)
        .stdout(predicate::str::contains(VALID_MS1));
}
