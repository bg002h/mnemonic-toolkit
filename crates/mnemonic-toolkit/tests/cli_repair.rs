//! Integration tests for `mnemonic repair` — Phase 3 v0.22.0 cycle.
//!
//! Realizes `design/IMPLEMENTATION_PLAN_repair_v0_22.md` §4.2 (6 cells).
//! Cells:
//!   9.  text-form ms1 happy-path
//!   10. --json ms1 happy-path
//!   11. already-valid input (exit 0)
//!   12. unrepairable (TooManyErrors → exit 2)
//!   13. multi-chunk mk1 with one corrupted chunk
//!   14. stdin form (`--ms1 -`)
//!
//! Fixtures: known-valid m-format strings reused from `src/repair.rs::tests`.
//!   - VALID_MS1 was generated from the TREZOR_12_ZERO entropy via
//!     `mnemonic bundle --template bip84 --slot @0.phrase=… --json`.
//!   - VALID_MK1_LONG / VALID_MK1_REG were generated 2026-05-17 from the
//!     canonical `abandon × 11 about` phrase.

use assert_cmd::Command;
use predicates::prelude::*;

const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const VALID_MK1_LONG_CHUNK0: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
const VALID_MK1_REG_CHUNK1: &str = "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";

/// Helper: deterministically flip the bech32 character at `pos` (within
/// the data-part, i.e. after the `1` separator) to the next char in the
/// bech32 alphabet (cyclic). Mirrors the helper in `src/repair.rs::tests`.
fn flip_at(chunk: &str, pos: usize) -> String {
    const ALPHABET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let sep = chunk.rfind('1').unwrap();
    let (prefix, rest) = chunk.split_at(sep + 1);
    let mut chars: Vec<char> = rest.chars().collect();
    let was = chars[pos];
    let was_idx = ALPHABET.find(was).unwrap();
    let new_idx = (was_idx + 1) % 32;
    chars[pos] = ALPHABET.chars().nth(new_idx).unwrap();
    let mut out = String::from(prefix);
    for c in chars {
        out.push(c);
    }
    out
}

/// Cell 9: text-form ms1 happy-path.
#[test]
fn cell_9_text_form_ms1_happy_path_exit_5_with_report() {
    let bad = flip_at(VALID_MS1, 17);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--ms1", &bad])
        .assert()
        .code(5)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(
            "ms1 chunk 0: 1 correction at position 17",
        ))
        .stdout(predicate::str::contains(VALID_MS1));
}

/// Cell 10: --json ms1 happy-path.
#[test]
fn cell_10_json_form_ms1_happy_path_envelope_shape() {
    let bad = flip_at(VALID_MS1, 17);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--json", "--ms1", &bad])
        .assert()
        .code(5)
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(s.trim()).expect("valid JSON envelope");
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["kind"], "ms1");
    assert_eq!(v["corrected_chunks"][0], VALID_MS1);
    assert_eq!(v["repairs"][0]["chunk_index"], 0);
    assert_eq!(v["repairs"][0]["corrected_positions"][0]["position"], 17);
    assert_eq!(v["repairs"][0]["original_chunk"], bad);
    assert_eq!(v["repairs"][0]["corrected_chunk"], VALID_MS1);
}

/// Cell 11: already-valid input → exit 0, no report.
#[test]
fn cell_11_already_valid_ms1_exit_0_no_report() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--ms1", VALID_MS1])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("# Repair report").not())
        .stdout(predicate::str::contains(VALID_MS1));
}

/// Cell 12: unrepairable (6 errors > singleton bound) → exit 2 (Repair
/// error class) + stderr cites TooManyErrors / singleton bound.
#[test]
fn cell_12_unrepairable_exits_2_with_too_many_errors_stderr() {
    // 6 errors at well-spread positions → above t=4 singleton bound.
    let bad = [3usize, 8, 13, 18, 23, 28]
        .iter()
        .fold(VALID_MS1.to_string(), |acc, &p| flip_at(&acc, p));
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--ms1", &bad])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("too many errors"))
        .stderr(predicate::str::contains("singleton bound"));
}

/// Cell 13: multi-chunk mk1 — supply 2 chunks, corrupt the second only,
/// confirm exit 5 + both chunks emitted + only chunk 1 in the report.
#[test]
fn cell_13_multi_chunk_mk1_one_corrupted_exit_5_both_emitted() {
    let bad_chunk1 = flip_at(VALID_MK1_REG_CHUNK1, 25);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "repair",
            "--mk1",
            VALID_MK1_LONG_CHUNK0,
            "--mk1",
            &bad_chunk1,
        ])
        .assert()
        .code(5)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(
            "mk1 chunk 1: 1 correction at position 25",
        ))
        .stdout(predicate::str::contains(VALID_MK1_LONG_CHUNK0))
        .stdout(predicate::str::contains(VALID_MK1_REG_CHUNK1));
}

/// Cell 14: stdin form (`--ms1 -`) — pipe a corrupted ms1 in, expect
/// repair report + corrected output on stdout.
#[test]
fn cell_14_stdin_form_ms1_dash_reads_from_stdin() {
    let bad = flip_at(VALID_MS1, 17);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--ms1", "-"])
        .write_stdin(format!("{bad}\n"))
        .assert()
        .code(5)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(VALID_MS1));
}
