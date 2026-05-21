//! v0.31.3 — `mnemonic bundle --slot @N.seedqr=<digit-string>` integration tests.
//!
//! Validates the path-split at `wallet_input::slot_input::SlotSubkey::Seedqr`
//! + the bundle.rs slot-consumer branch that decodes via `seedqr::decode`
//! and dispatches identically to the existing Phrase branch.
//!
//! Byte-equal regression cells assert the bundle envelope is identical
//! between `--slot @N.seedqr=<digits>` and `--slot @N.phrase=<phrase>`
//! where phrase is the SeedQR decode (round-trip fidelity).

use assert_cmd::Command;

const PHRASE_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const DIGITS_12: &str = "000000000000000000000000000000000000000000000003";

const PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const DIGITS_24: &str = "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000102";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

fn bundle_via_phrase(phrase: &str) -> Vec<u8> {
    let out = mnemonic()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
        ])
        .arg(format!("@0.phrase={phrase}"))
        .assert()
        .success();
    out.get_output().stdout.clone()
}

fn bundle_via_seedqr(digits: &str) -> Vec<u8> {
    let out = mnemonic()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
        ])
        .arg(format!("@0.seedqr={digits}"))
        .assert()
        .success();
    out.get_output().stdout.clone()
}

// ──────────────────────────────────────────────────────────────────────
// Happy paths — byte-equal regression
// ──────────────────────────────────────────────────────────────────────

#[test]
fn bundle_seedqr_slot_happy_path_24word() {
    let via_phrase = bundle_via_phrase(PHRASE_24);
    let via_seedqr = bundle_via_seedqr(DIGITS_24);
    assert_eq!(
        via_phrase, via_seedqr,
        "24-word bundle envelope must be byte-equal between \
         --slot @0.phrase=... and --slot @0.seedqr=... \
         (seedqr decode materializes the identical phrase)"
    );
}

#[test]
fn bundle_seedqr_slot_happy_path_12word() {
    let via_phrase = bundle_via_phrase(PHRASE_12);
    let via_seedqr = bundle_via_seedqr(DIGITS_12);
    assert_eq!(
        via_phrase, via_seedqr,
        "12-word bundle envelope must be byte-equal between \
         --slot @0.phrase=... and --slot @0.seedqr=..."
    );
}

// ──────────────────────────────────────────────────────────────────────
// Refusal paths
// ──────────────────────────────────────────────────────────────────────

#[test]
fn bundle_seedqr_slot_invalid_digit_count_refused() {
    // 47 digits (one short of valid 48) — refused by seedqr::decode.
    let bad_digits = "0".repeat(47);
    let assertion = mnemonic()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
        ])
        .arg(format!("@0.seedqr={bad_digits}"))
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("seedqr: slot @0 decode") && stderr.contains("invalid digit count"),
        "expected canonical seedqr decode error citing slot @0; got: {stderr}"
    );
}

#[test]
fn bundle_seedqr_slot_checksum_failure_refused() {
    // 48 digits, all-zeros — valid shape but invalid BIP-39 checksum
    // ("abandon" × 12 fails the BIP-39 checksum; only `abandon × 11 + about`
    // is valid).
    let bad_digits = "0".repeat(48);
    let assertion = mnemonic()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
        ])
        .arg(format!("@0.seedqr={bad_digits}"))
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("seedqr: slot @0 decode") && stderr.contains("checksum"),
        "expected canonical seedqr checksum-failure error; got: {stderr}"
    );
}

// ──────────────────────────────────────────────────────────────────────
// Stdin sentinel
// ──────────────────────────────────────────────────────────────────────

#[test]
fn bundle_seedqr_slot_stdin_sentinel_happy_path() {
    let via_seedqr_stdin = mnemonic()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            "@0.seedqr=-",
        ])
        .write_stdin(DIGITS_12)
        .assert()
        .success();
    let via_phrase = bundle_via_phrase(PHRASE_12);
    assert_eq!(
        via_seedqr_stdin.get_output().stdout,
        via_phrase,
        "@0.seedqr=- with digits via stdin must produce byte-equal bundle to \
         --slot @0.phrase=<phrase>"
    );
}

#[test]
fn bundle_seedqr_slot_double_stdin_refused() {
    // R0 I2 — single-stdin-per-invocation invariant must still fire
    // when one seedqr slot AND one phrase slot both request stdin.
    let assertion = mnemonic()
        .args([
            "bundle",
            "--template",
            "wsh-multi",
            "--threshold",
            "2",
            "--network",
            "mainnet",
            "--slot",
            "@0.seedqr=-",
            "--slot",
            "@1.phrase=-",
        ])
        .write_stdin("ignored")
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("at most one --slot @N.<secret>=- per invocation"),
        "expected single-stdin-per-invocation refusal; got: {stderr}"
    );
}
