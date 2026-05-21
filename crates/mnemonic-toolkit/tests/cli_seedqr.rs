//! CLI integration tests for `mnemonic seedqr` (v0.30.0). Target ≥30 cells.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

const PHRASE_12: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const DIGITS_12: &str = "000000000000000000000000000000000000000000000003";

const PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const DIGITS_24: &str = "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000102";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

// ──────────────────────────────────────────────────────────────────────
// Decode — happy paths
// ──────────────────────────────────────────────────────────────────────

#[test]
fn decode_12_word_text_mode() {
    mnemonic().args(["seedqr", "decode", "--digits", DIGITS_12])
        .assert().success().stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn decode_24_word_text_mode() {
    mnemonic().args(["seedqr", "decode", "--digits", DIGITS_24])
        .assert().success().stdout(format!("{PHRASE_24}\n"));
}

#[test]
fn decode_stdin_space_form() {
    mnemonic().args(["seedqr", "decode", "--digits", "-"])
        .write_stdin(DIGITS_12)
        .assert().success().stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn decode_stdin_equals_form() {
    mnemonic().args(["seedqr", "decode", "--digits=-"])
        .write_stdin(DIGITS_12)
        .assert().success().stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn decode_json_mode_12_word() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic().args(["seedqr", "decode", "--digits", DIGITS_12, "--json-out", path.to_str().unwrap()])
        .assert().success().stdout("");
    let json: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(json["schema_version"], "1");
    assert_eq!(json["operation"], "decode");
    assert_eq!(json["variant"], "standard");
    assert_eq!(json["word_count"], 12);
    assert_eq!(json["phrase"], PHRASE_12);
    assert_eq!(json["digits"], DIGITS_12);
}

#[test]
fn decode_json_mode_24_word() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic().args(["seedqr", "decode", "--digits", DIGITS_24, "--json-out", path.to_str().unwrap()])
        .assert().success().stdout("");
    let json: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(json["word_count"], 24);
    assert_eq!(json["phrase"], PHRASE_24);
    assert_eq!(json["digits"], DIGITS_24);
}

// ──────────────────────────────────────────────────────────────────────
// Decode — refusals
// ──────────────────────────────────────────────────────────────────────

#[test]
fn decode_rejects_length_47() {
    let bad = &DIGITS_12[..47];
    mnemonic().args(["seedqr", "decode", "--digits", bad])
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: decode: invalid digit count"));
}

#[test]
fn decode_rejects_length_49() {
    let bad = format!("{DIGITS_12}0");
    mnemonic().args(["seedqr", "decode", "--digits", &bad])
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: decode: invalid digit count"));
}

#[test]
fn decode_rejects_length_95() {
    let bad = &DIGITS_24[..95];
    mnemonic().args(["seedqr", "decode", "--digits", bad])
        .assert().failure().code(1);
}

#[test]
fn decode_rejects_length_97() {
    let bad = format!("{DIGITS_24}0");
    mnemonic().args(["seedqr", "decode", "--digits", &bad])
        .assert().failure().code(1);
}

#[test]
fn decode_rejects_non_digit_char() {
    let bad = "00000000000000000000000000000000000000000000000A";
    mnemonic().args(["seedqr", "decode", "--digits", bad])
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: decode: invalid character"));
}

#[test]
fn decode_rejects_word_index_out_of_range() {
    let bad = format!("9999{}", &DIGITS_12[4..]);
    mnemonic().args(["seedqr", "decode", "--digits", &bad])
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: decode: invalid word index"));
}

#[test]
fn decode_rejects_checksum_failure() {
    let bad = "000100010001000100010001000100010001000100010001";
    mnemonic().args(["seedqr", "decode", "--digits", bad])
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: decode: BIP-39 checksum failure"));
}

// ──────────────────────────────────────────────────────────────────────
// Encode — happy paths
// ──────────────────────────────────────────────────────────────────────

#[test]
fn encode_12_word_text_mode() {
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .assert().success().stdout(format!("{DIGITS_12}\n"));
}

#[test]
fn encode_24_word_text_mode() {
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_24}"))
        .assert().success().stdout(format!("{DIGITS_24}\n"));
}

#[test]
fn encode_stdin_space_form() {
    mnemonic().args(["seedqr", "encode", "--from", "phrase=-"])
        .write_stdin(PHRASE_12)
        .assert().success().stdout(format!("{DIGITS_12}\n"));
}

#[test]
fn encode_json_mode_12_word() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .args(["--json-out", path.to_str().unwrap()])
        .assert().success().stdout("");
    let json: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(json["schema_version"], "1");
    assert_eq!(json["operation"], "encode");
    assert_eq!(json["variant"], "standard");
    assert_eq!(json["word_count"], 12);
    assert_eq!(json["phrase"], PHRASE_12);
    assert_eq!(json["digits"], DIGITS_12);
}

#[test]
fn encode_json_mode_24_word() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_24}"))
        .args(["--json-out", path.to_str().unwrap()])
        .assert().success().stdout("");
    let json: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(json["word_count"], 24);
    assert_eq!(json["phrase"], PHRASE_24);
    assert_eq!(json["digits"], DIGITS_24);
}

// ──────────────────────────────────────────────────────────────────────
// Encode — refusals
// ──────────────────────────────────────────────────────────────────────

#[test]
fn encode_rejects_non_phrase_node_xpub() {
    mnemonic().args(["seedqr", "encode", "--from", "xpub=xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz"])
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr encode only accepts phrase="));
}

#[test]
fn encode_rejects_13_word_count() {
    let bad = format!("{PHRASE_12} abandon");
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: encode: invalid word count"));
}

#[test]
fn encode_rejects_15_word_count() {
    let bad = "abandon ".repeat(14) + "about";
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert().failure().code(1);
}

#[test]
fn encode_rejects_18_word_count() {
    let bad = "abandon ".repeat(17) + "about";
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert().failure().code(1);
}

#[test]
fn encode_rejects_21_word_count() {
    let bad = "abandon ".repeat(20) + "about";
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert().failure().code(1);
}

#[test]
fn encode_rejects_25_word_count() {
    let bad = format!("{PHRASE_24} abandon");
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert().failure().code(1);
}

#[test]
fn encode_rejects_invalid_word() {
    let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon notaword";
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: encode: BIP-39 checksum failure"));
}

#[test]
fn encode_rejects_checksum_failure() {
    let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: encode: BIP-39 checksum failure"));
}

// ──────────────────────────────────────────────────────────────────────
// Round-trip
// ──────────────────────────────────────────────────────────────────────

#[test]
fn round_trip_12_word_text() {
    let encode_out = mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .assert().success();
    let digits = String::from_utf8(encode_out.get_output().stdout.clone()).unwrap();
    let digits = digits.trim_end();

    mnemonic().args(["seedqr", "decode", "--digits", digits])
        .assert().success().stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn round_trip_24_word_text() {
    let encode_out = mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_24}"))
        .assert().success();
    let digits = String::from_utf8(encode_out.get_output().stdout.clone()).unwrap();
    let digits = digits.trim_end();

    mnemonic().args(["seedqr", "decode", "--digits", digits])
        .assert().success().stdout(format!("{PHRASE_24}\n"));
}

#[test]
fn round_trip_12_word_through_json_envelope() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .args(["--json-out", path.to_str().unwrap()])
        .assert().success();
    let json: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    let digits = json["digits"].as_str().unwrap();

    mnemonic().args(["seedqr", "decode", "--digits", digits])
        .assert().success().stdout(format!("{PHRASE_12}\n"));
}

// ──────────────────────────────────────────────────────────────────────
// Argv-leakage advisory
// ──────────────────────────────────────────────────────────────────────

#[test]
fn decode_emits_argv_advisory_on_inline_form() {
    // Assert on the load-bearing template substring per
    // secret_advisory.rs:36-38: `"warning: secret material on argv (...)"`.
    // Asserting on `--digits` alone would be too loose; on `supplied in argv`
    // would be vacuous (substring doesn't appear in the template).
    mnemonic().args(["seedqr", "decode", "--digits", DIGITS_12])
        .assert().success()
        .stderr(predicates::str::contains("secret material on argv"))
        .stderr(predicates::str::contains("--digits"));
}

#[test]
fn encode_emits_argv_advisory_on_inline_form() {
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .assert().success()
        .stderr(predicates::str::contains("secret material on argv"))
        .stderr(predicates::str::contains("--from phrase="));
}

#[test]
fn decode_no_argv_advisory_on_stdin_form() {
    // Negate on the load-bearing template substring (per
    // secret_advisory.rs:36-38); negating on `supplied in argv` would be
    // vacuous since that substring is never emitted by any code path.
    let stderr = mnemonic().args(["seedqr", "decode", "--digits", "-"])
        .write_stdin(DIGITS_12)
        .assert().success();
    let stderr_bytes = stderr.get_output().stderr.clone();
    let stderr_str = String::from_utf8(stderr_bytes).unwrap();
    assert!(
        !stderr_str.contains("secret material on argv"),
        "stdin form must not emit argv-leakage advisory; got stderr: {stderr_str}"
    );
}
