//! v0.27.0 — `mnemonic import-wallet --bsms-round1 <FILE>` integration tests.
//!
//! Closes the `bsms-verify-signatures` FOLLOWUP. Per `design/agent-reports/
//! v0_27_0-phase-2-bip129-recon.md`: BIP-129 Round-1 5-line key records
//! (Signer → Coordinator) are verified via BIP-322 legacy-format ECDSA
//! recoverable signatures, base64-encoded on line 5.
//!
//! Test vectors are the published BIP-129 in-spec set (recon doc §3a):
//! - TV-1: NO_ENCRYPTION / raw pubkey, Signer 1 (`fixtures/bsms_round1/tv1-*.bsms`)
//! - TV-2: NO_ENCRYPTION / xpub, Signer 1 (`fixtures/bsms_round1/tv2-*.bsms`)
//! - TV-3: STANDARD encryption / xpub, Signer 1 (`fixtures/bsms_round1/tv3-*.bsms`)
//!
//! Cells:
//!   1. TV-1 happy path (lenient default; verified=true)
//!   2. TV-2 xpub happy path (verified=true; xpub's OWN embedded pubkey)
//!   3. TV-3 STANDARD encryption happy path (verified=true; token-in-signed-body)
//!   4. --bsms-verify-strict on verified record (exit 0; no NOTICE)
//!   5. flipped-SIG lenient default → stderr NOTICE + verified=false + exit 4 (v0.85.0 M4)
//!   6. flipped-SIG --bsms-verify-strict → exit 2 BsmsSignatureMismatch
//!   7. flipped-TOKEN --bsms-verify-strict → exit 2
//!   8. malformed line-count → exit 2 BsmsRound1Malformed
//!   9. multi-record: 2 verified records emit array of 2 verifications
//!  10. --bsms-verify-strict without --bsms-round1 → BadInput exit 2
//!  11. stdin `-` form rejected (deferred per v0.27.0 scope)
//!  12. --json envelope shape: standalone mode (no --blob)
//!  13. --json envelope shape: combined with --blob
//!  14. text-mode summary shape
//!  15. record_index propagates correctly in error messages

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

const TV1_FIXTURE: &str = "tests/fixtures/bsms_round1/tv1-no-encryption-pubkey-signer1.bsms";
const TV2_FIXTURE: &str = "tests/fixtures/bsms_round1/tv2-no-encryption-xpub-signer1.bsms";
const TV3_FIXTURE: &str = "tests/fixtures/bsms_round1/tv3-standard-xpub-signer1.bsms";

const TV1_SIGNER_PUBKEY: &str =
    "026d15412460ba0d881c21837bb999233896085a9ed4e5445bd637c10e579768ba";

/// TV-2's xpub-embedded compressed public key (the xpub's OWN
/// `public_key.serialize()` — not a child derivation). Pinning the exact
/// hex distinguishes "xpub path uses embedded pubkey" from "xpub path
/// derives a child" — the v0.27.0 PR-review S5 ask.
const TV2_SIGNER_PUBKEY: &str =
    "025fa5a6544e85c02a2c33f5090f573d9ba83ec54a852211d39f844f04b8e8b0a3";

/// Cell 1 — TV-1 NO_ENCRYPTION/pubkey Signer 1 verifies under lenient default.
#[test]
fn cell_1_tv1_no_encryption_pubkey_signer1_verifies() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--bsms-round1", TV1_FIXTURE, "--json"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"signature_verified\":true"))
        .stdout(predicate::str::contains(TV1_SIGNER_PUBKEY));
}

/// Cell 2 — TV-2 NO_ENCRYPTION/xpub Signer 1 verifies (xpub's OWN embedded pubkey).
#[test]
fn cell_2_tv2_no_encryption_xpub_signer1_verifies() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--bsms-round1", TV2_FIXTURE, "--json"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"signature_verified\":true"))
        // xpub TV-2 embedded pubkey (xpub.public_key.serialize()) — load-bearing
        // assertion that xpub path uses the OWN embedded pubkey, NOT a derived
        // child. Phase 6.5 PR-review S5 fold: pin the exact hex (not just the
        // field's presence) so a "derive child instead" regression cannot pass.
        .stdout(predicate::str::contains(TV2_SIGNER_PUBKEY));
}

/// Cell 3 — TV-3 STANDARD encryption Signer 1 verifies (TOKEN is signed-body member).
#[test]
fn cell_3_tv3_standard_encryption_xpub_signer1_verifies() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--bsms-round1", TV3_FIXTURE, "--json"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"signature_verified\":true"))
        .stdout(predicate::str::contains(
            "\"token_hex\":\"a54044308ceac9b7\"",
        ));
}

/// Cell 4 — --bsms-verify-strict on verified record: exit 0; no NOTICE.
#[test]
fn cell_4_verify_strict_on_verified_record_succeeds_silently() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--bsms-round1",
            TV1_FIXTURE,
            "--bsms-verify-strict",
            "--json",
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"signature_verified\":true"))
        .stderr(predicate::str::contains("notice:").not());
}

/// Cell 5 — flipped SIG lenient default: stderr NOTICE + envelope
/// verified=false + exit 4 (v0.85.0 M4 — `any(Failed)` in lenient mode now
/// exits 4 "VERIFY-ME", not 0; the report/envelope is still fully emitted).
#[test]
fn cell_5_flipped_sig_lenient_default_emits_notice_and_verified_false() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let fixture = std::fs::read_to_string(TV1_FIXTURE).unwrap();
    // Flip last payload byte of the base64 SIG (change last '4' before '=' to '5').
    let bad = fixture.replace("q0s6im4=", "q0s6im5=");
    std::fs::write(tmp.path(), bad).unwrap();

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--bsms-round1",
            tmp.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .code(4)
        .stdout(predicate::str::contains("\"signature_verified\":false"))
        .stderr(predicate::str::contains(
            "notice: import-wallet: --bsms-round1: signature verification failed",
        ));
}

/// Cell 6 — flipped SIG under --bsms-verify-strict: exit 2 BsmsSignatureMismatch.
#[test]
fn cell_6_flipped_sig_strict_errors_signature_mismatch_exit_2() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let fixture = std::fs::read_to_string(TV1_FIXTURE).unwrap();
    let bad = fixture.replace("q0s6im4=", "q0s6im5=");
    std::fs::write(tmp.path(), bad).unwrap();

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--bsms-round1",
            tmp.path().to_str().unwrap(),
            "--bsms-verify-strict",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "BIP-129 signature verification failed",
        ));
}

/// Cell 7 — flipped TOKEN under --bsms-verify-strict: exit 2 (TOKEN is signed-body member).
#[test]
fn cell_7_flipped_token_strict_errors_signature_mismatch() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let fixture = std::fs::read_to_string(TV1_FIXTURE).unwrap();
    // NO_ENCRYPTION mode: token "00" → "ff" (still valid 2-hex shape).
    let bad = fixture.replace("\n00\n", "\nff\n");
    std::fs::write(tmp.path(), bad).unwrap();

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--bsms-round1",
            tmp.path().to_str().unwrap(),
            "--bsms-verify-strict",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "BIP-129 signature verification failed",
        ));
}

/// Cell 8 — malformed line count: exit 2 BsmsRound1Malformed.
#[test]
fn cell_8_malformed_line_count_errors_round1_malformed() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    // Strip the SIG line — only 4 lines remain.
    let bad = "BSMS 1.0\n00\n[59865f44/48'/0'/0'/2']026d15412460ba0d881c21837bb999233896085a9ed4e5445bd637c10e579768ba\nSigner 1 key\n";
    std::fs::write(tmp.path(), bad).unwrap();

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--bsms-round1",
            tmp.path().to_str().unwrap(),
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("BIP-129 Round-1 record malformed"));
}

/// Cell 9 — multi-record verify: 2 verified records emit array of 2 verifications.
#[test]
fn cell_9_multi_record_verify_emits_array_of_two() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--bsms-round1",
            TV1_FIXTURE,
            "--bsms-round1",
            TV2_FIXTURE,
            "--json",
        ])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let body = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(body.trim()).unwrap();
    let verifications = v["bsms_round1_verifications"].as_array().unwrap();
    assert_eq!(verifications.len(), 2, "two verifications expected");
    assert_eq!(verifications[0]["index"], 0);
    assert_eq!(verifications[0]["signature_verified"], true);
    assert_eq!(verifications[1]["index"], 1);
    assert_eq!(verifications[1]["signature_verified"], true);
}

/// Cell 10 — --bsms-verify-strict without --bsms-round1 is meaningless; BadInput exit 1.
#[test]
fn cell_10_verify_strict_without_round1_errors_bad_input() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--bsms-verify-strict",
            "--blob",
            "tests/fixtures/wallet_import/bsms-1of1-singlesig.txt",
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "--bsms-verify-strict requires `--bsms-round1`",
        ));
}

/// Cell 11 — stdin `-` form rejected (deferred per v0.27.0 scope); BadInput exit 1.
#[test]
fn cell_11_stdin_dash_rejected_in_v_0_27_0() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--bsms-round1", "-"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "--bsms-round1 -: stdin input deferred",
        ));
}

/// Cell 12 — --json envelope shape: standalone mode (no --blob). Validates
/// the standalone-Round-1-verify envelope produces the expected shape.
#[test]
fn cell_12_standalone_round1_json_envelope_shape() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--bsms-round1", TV1_FIXTURE, "--json"])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let body = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(body.trim()).unwrap();
    assert_eq!(v["source_format"], "bsms-round1");
    let verifications = v["bsms_round1_verifications"].as_array().unwrap();
    assert_eq!(verifications.len(), 1);
    assert_eq!(verifications[0]["index"], 0);
    assert_eq!(verifications[0]["signer_pubkey"], TV1_SIGNER_PUBKEY);
    assert_eq!(verifications[0]["description"], "Signer 1 key");
    assert_eq!(verifications[0]["token_hex"], "00");
    assert_eq!(verifications[0]["signature_verified"], true);
    assert!(verifications[0]["failure_reason"].is_null());
}

/// Cell 13 — --json envelope shape: combined with --blob.
/// Both Round-1 verify state AND parsed bundle envelope appear in output.
#[test]
fn cell_13_combined_blob_and_round1_json_envelope_shape() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "tests/fixtures/wallet_import/bsms-1of1-singlesig.txt",
            "--bsms-round1",
            TV1_FIXTURE,
            "--json",
        ])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let body = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(body.trim()).unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    // BSMS parsed bundle envelope shape preserved.
    assert!(arr[0]["bundle"].is_object());
    assert_eq!(arr[0]["source_format"], "bsms");
    // Round-1 verifications attached.
    let verifications = arr[0]["bsms_round1_verifications"].as_array().unwrap();
    assert_eq!(verifications.len(), 1);
    assert_eq!(verifications[0]["signature_verified"], true);
}

/// Cell 14 — text-mode summary (no --json) on standalone verify.
#[test]
fn cell_14_text_mode_summary_shape() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--bsms-round1", TV1_FIXTURE])
        .assert()
        .code(0)
        .stdout(predicate::str::contains(
            "bsms-round1: 1 record(s) processed",
        ))
        .stdout(predicate::str::contains("record[0]:"))
        .stdout(predicate::str::contains("verified=true"));
}

/// Cell 15 — record_index propagates correctly: middle of multi-record fails.
#[test]
fn cell_15_record_index_propagates_in_multi_record_error() {
    let tmp_bad = tempfile::NamedTempFile::new().unwrap();
    let fixture = std::fs::read_to_string(TV1_FIXTURE).unwrap();
    let bad = fixture.replace("q0s6im4=", "q0s6im5=");
    std::fs::write(tmp_bad.path(), bad).unwrap();

    // Order: TV-2 (good, index 0), TV-1-corrupt (bad, index 1), TV-3 (good, index 2)
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--bsms-round1",
            TV2_FIXTURE,
            "--bsms-round1",
            tmp_bad.path().to_str().unwrap(),
            "--bsms-round1",
            TV3_FIXTURE,
            "--bsms-verify-strict",
        ])
        .assert()
        .code(2)
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8(out).unwrap();
    assert!(
        stderr.contains("record 1"),
        "record_index 1 must appear in error; got: {stderr}"
    );
}

/// v0.85.0 M4 — combined `--blob` + `--bsms-round1` mode, lenient default,
/// with a Failed record: the `:1363` tail return must apply the SAME
/// `any(Failed)` → exit 4 rule as the standalone early-return (cell 5).
/// Without this test the combined-mode arm ships untested — cell 13 (below)
/// uses a verified record, so a revert of the `:1363` check would still
/// pass the rest of the suite. Blob = cell 13's fixture (BSMS 1-of-1
/// singlesig); Round-1 record = cell 5's flipped-SIG TV1 tempfile.
#[test]
fn combined_blob_round1_lenient_failed_exits_4() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let fixture = std::fs::read_to_string(TV1_FIXTURE).unwrap();
    let bad = fixture.replace("q0s6im4=", "q0s6im5=");
    std::fs::write(tmp.path(), bad).unwrap();

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "tests/fixtures/wallet_import/bsms-1of1-singlesig.txt",
            "--bsms-round1",
            tmp.path().to_str().unwrap(),
            "--json",
        ])
        .assert()
        .code(4)
        .get_output()
        .stdout
        .clone();
    let body = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(body.trim()).unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    // The parsed bundle envelope is still fully emitted (lenient mode does
    // not abort import) — only the exit code changed.
    assert!(arr[0]["bundle"].is_object());
    assert_eq!(arr[0]["source_format"], "bsms");
    let verifications = arr[0]["bsms_round1_verifications"].as_array().unwrap();
    assert_eq!(verifications.len(), 1);
    assert_eq!(verifications[0]["signature_verified"], false);
}

/// Helper: silence unused-import warnings on `Write` import.
#[allow(dead_code)]
fn _silence_unused_write_import() {
    let _ = std::io::sink().write_all(b"");
}
