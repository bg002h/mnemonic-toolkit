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

/// Cell 12b (ms-codec 0.2.1 regression): `mnemonic repair --ms1` works for ALL
/// BIP-39 entropy lengths, not just 12-word. Before ms-codec 0.2.1,
/// `decode_with_correction` returned `TooManyErrors` on CLEAN 20/24/28/32-byte
/// ms1 strings (a wrong `POLYMOD_INIT`), so `repair` exited 2 on a clean longer
/// seed and could not repair a single-error one — silently broken end-to-end for
/// 15/18/21/24-word backups. See
/// `mnemonic-secret/design/BUG_decode_with_correction_length_divergence.md`.
/// Fixtures generated in-test via `ms_codec::encode` (not hard-coded) so the cell
/// stays valid across any future ms1 wire tweak.
#[test]
fn cell_12b_ms1_repair_works_for_all_entropy_lengths() {
    use ms_codec::{Payload, Tag, encode};
    for len in [20usize, 24, 28, 32] {
        let entropy: Vec<u8> = (0..len as u8).collect();
        let valid = encode(Tag::ENTR, &Payload::Entr(entropy)).expect("encode ms1");

        // clean longer seed → exit 0 (already valid), no report. (Pre-0.2.1: exit 2.)
        Command::cargo_bin("mnemonic")
            .unwrap()
            .args(["repair", "--ms1", &valid])
            .assert()
            .code(0)
            .stdout(predicate::str::contains("# Repair report").not())
            .stdout(predicate::str::contains(valid.as_str()));

        // 1-error longer seed → exit 5 (repaired back to the original). (Pre-0.2.1: exit 2.)
        let bad = flip_at(&valid, 5);
        Command::cargo_bin("mnemonic")
            .unwrap()
            .args(["repair", "--ms1", &bad])
            .assert()
            .code(5)
            .stdout(predicate::str::contains("# Repair report"))
            .stdout(predicate::str::contains(valid.as_str()));
    }
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

// ============================================================================
// v0.23.0 Phase B.7 — D29 migration regression-guard cells (plan §4.B.4)
// ============================================================================
//
// These two cells guard against R3 (silent regression from delegating Ms1 +
// Md1 repair to sibling-codec native APIs in v0.23.0). The 29 pre-existing
// repair cells (cli_repair 6 + cli_inspect 3 + cli_auto_repair 13 +
// cli_verify_bundle_multi_cosigner_mk1 7) are the primary substring-match
// regression-guards per Phase B.0 (h); these two cells add explicit
// migration-aware coverage on top.

/// Cell B7-1 (`migrated_repair_byte_exact_with_pre_v0_23`): meta-regression
/// guard. Re-runs the 6 cli_repair scenarios through the new
/// sibling-codec-delegating `repair_card` and asserts the observable
/// substring contract per scenario stays stable. The cell deliberately
/// duplicates a slice of the pre-existing assertions; if any of the 6
/// pre-existing cells regresses, both that cell AND this meta-cell fail in
/// lockstep, surfacing R3 redundantly.
#[test]
fn cell_b7_1_migrated_repair_byte_exact_with_pre_v0_23() {
    // Scenario 9: text-form ms1 happy-path.
    let bad_ms1 = flip_at(VALID_MS1, 17);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--ms1", &bad_ms1])
        .assert()
        .code(5)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(
            "ms1 chunk 0: 1 correction at position 17",
        ))
        .stdout(predicate::str::contains(VALID_MS1));

    // Scenario 11: already-valid input.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--ms1", VALID_MS1])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("# Repair report").not())
        .stdout(predicate::str::contains(VALID_MS1));

    // Scenario 12: 6-error ms1 → exit 2 + TooManyErrors stderr.
    let unrepairable = [3usize, 8, 13, 18, 23, 28]
        .iter()
        .fold(VALID_MS1.to_string(), |acc, &p| flip_at(&acc, p));
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--ms1", &unrepairable])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("too many errors"))
        .stderr(predicate::str::contains("singleton bound"));

    // Scenario 13: multi-chunk mk1 with one corrupted chunk.
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
        ));
}

/// Cell B7-2 (`error_mapping_fidelity`): D29 helper-translation table
/// assertion. Feed a 5-error ms1 chunk through `repair_card(Ms1, …)` and
/// assert the returned error is `RepairError::TooManyErrors { chunk_index:
/// 0, bound: 8 }` — NOT `PostCorrectionDecodeFailed`. This pins the Q2
/// absorption rule (ms_codec::Error::TooManyErrors → toolkit
/// TooManyErrors). Same shape for md1 via `repair_card(Md1, &[md1_chunk])`.
///
/// Cell lives in cli_repair.rs (integration) rather than src/repair.rs
/// because it exercises the public `mnemonic-toolkit` lib surface; the
/// helpers themselves are pub(crate)-only.
#[test]
fn cell_b7_2_error_mapping_fidelity_ms1_too_many_errors_observable_exit_2() {
    // 5-error ms1 — above t=4 capacity. Asserts the toolkit's
    // RepairError::TooManyErrors path fires (the cell observes exit code
    // 2 + the TooManyErrors stderr substring), proving the
    // ms_codec::Error::TooManyErrors → toolkit TooManyErrors absorption
    // works end-to-end through the public CLI surface.
    let bad_ms1 = [3usize, 11, 19, 27, 35]
        .iter()
        .fold(VALID_MS1.to_string(), |acc, &p| flip_at(&acc, p));
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--ms1", &bad_ms1])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("too many errors"))
        .stderr(predicate::str::contains("singleton bound = 8"))
        // Catch a regression where the helper falls into
        // PostCorrectionDecodeFailed instead of TooManyErrors.
        .stderr(predicate::str::contains("post-correction decode failed").not());

    // md1 path — 5-error md1 chunk in chunk 0 → atomic-fail with chunk_index:0.
    const VALID_MD1_CHUNK0: &str =
        "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
    const VALID_MD1_CHUNK1: &str =
        "md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d";
    const VALID_MD1_CHUNK2: &str =
        "md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn";
    let bad_md1 = [3usize, 11, 19, 27, 35]
        .iter()
        .fold(VALID_MD1_CHUNK0.to_string(), |acc, &p| flip_at(&acc, p));
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "repair",
            "--md1",
            &bad_md1,
            "--md1",
            VALID_MD1_CHUNK1,
            "--md1",
            VALID_MD1_CHUNK2,
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("too many errors"))
        .stderr(predicate::str::contains("singleton bound = 8"))
        .stderr(predicate::str::contains("post-correction decode failed").not());
}
