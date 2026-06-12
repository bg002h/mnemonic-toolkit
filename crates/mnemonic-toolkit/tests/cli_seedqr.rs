//! CLI integration tests for `mnemonic seedqr` (v0.30.0). Target ≥30 cells.

use assert_cmd::Command;
use serde_json::Value;

const PHRASE_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
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
    mnemonic()
        .args(["seedqr", "decode", "--digits", DIGITS_12])
        .assert()
        .success()
        .stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn decode_24_word_text_mode() {
    mnemonic()
        .args(["seedqr", "decode", "--digits", DIGITS_24])
        .assert()
        .success()
        .stdout(format!("{PHRASE_24}\n"));
}

#[test]
fn decode_stdin_space_form() {
    mnemonic()
        .args(["seedqr", "decode", "--digits", "-"])
        .write_stdin(DIGITS_12)
        .assert()
        .success()
        .stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn decode_stdin_equals_form() {
    mnemonic()
        .args(["seedqr", "decode", "--digits=-"])
        .write_stdin(DIGITS_12)
        .assert()
        .success()
        .stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn decode_json_mode_12_word() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic()
        .args([
            "seedqr",
            "decode",
            "--digits",
            DIGITS_12,
            "--json-out",
            path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout("");
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
    mnemonic()
        .args([
            "seedqr",
            "decode",
            "--digits",
            DIGITS_24,
            "--json-out",
            path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout("");
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
    mnemonic()
        .args(["seedqr", "decode", "--digits", bad])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "seedqr: decode: invalid digit count",
        ));
}

#[test]
fn decode_rejects_length_49() {
    let bad = format!("{DIGITS_12}0");
    mnemonic()
        .args(["seedqr", "decode", "--digits", &bad])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "seedqr: decode: invalid digit count",
        ));
}

#[test]
fn decode_rejects_length_95() {
    let bad = &DIGITS_24[..95];
    mnemonic()
        .args(["seedqr", "decode", "--digits", bad])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn decode_rejects_length_97() {
    let bad = format!("{DIGITS_24}0");
    mnemonic()
        .args(["seedqr", "decode", "--digits", &bad])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn decode_rejects_non_digit_char() {
    let bad = "00000000000000000000000000000000000000000000000A";
    mnemonic()
        .args(["seedqr", "decode", "--digits", bad])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "seedqr: decode: invalid character",
        ));
}

#[test]
fn decode_rejects_word_index_out_of_range() {
    let bad = format!("9999{}", &DIGITS_12[4..]);
    mnemonic()
        .args(["seedqr", "decode", "--digits", &bad])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "seedqr: decode: invalid word index",
        ));
}

#[test]
fn decode_rejects_checksum_failure() {
    let bad = "000100010001000100010001000100010001000100010001";
    mnemonic()
        .args(["seedqr", "decode", "--digits", bad])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "seedqr: decode: BIP-39 checksum failure",
        ));
}

// ──────────────────────────────────────────────────────────────────────
// Encode — happy paths
// ──────────────────────────────────────────────────────────────────────

#[test]
fn encode_12_word_text_mode() {
    mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .assert()
        .success()
        .stdout(format!("{DIGITS_12}\n"));
}

#[test]
fn encode_24_word_text_mode() {
    mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_24}"))
        .assert()
        .success()
        .stdout(format!("{DIGITS_24}\n"));
}

#[test]
fn encode_stdin_space_form() {
    mnemonic()
        .args(["seedqr", "encode", "--from", "phrase=-"])
        .write_stdin(PHRASE_12)
        .assert()
        .success()
        .stdout(format!("{DIGITS_12}\n"));
}

#[test]
fn encode_json_mode_12_word() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .args(["--json-out", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout("");
    let json: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(json["schema_version"], "1");
    assert_eq!(json["operation"], "encode");
    assert_eq!(json["variant"], "standard");
    assert_eq!(json["word_count"], 12);
    assert_eq!(json["phrase"], PHRASE_12);
    assert_eq!(json["digits"], DIGITS_12);
}

// ──────────────────────────────────────────────────────────────────────
// v0.32.0 — CompactSeedQR (--variant compact)
// ──────────────────────────────────────────────────────────────────────

const COMPACT_HEX_12: &str = "00000000000000000000000000000000";
const COMPACT_HEX_24: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[test]
fn encode_compact_12_word_cli() {
    mnemonic()
        .args(["seedqr", "encode", "--variant", "compact", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .assert()
        .success()
        .stdout(format!("{COMPACT_HEX_12}\n"));
}

#[test]
fn decode_compact_12_word_cli() {
    mnemonic()
        .args(["seedqr", "decode", "--variant", "compact", "--from"])
        .arg(format!("seedqr={COMPACT_HEX_12}"))
        .assert()
        .success()
        .stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn decode_compact_24_word_cli() {
    // R0 M2 — 24-word CLI happy path (64-hex).
    mnemonic()
        .args(["seedqr", "decode", "--variant", "compact", "--from"])
        .arg(format!("seedqr={COMPACT_HEX_24}"))
        .assert()
        .success()
        .stdout(format!("{PHRASE_24}\n"));
}

#[test]
fn encode_compact_json_envelope() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic()
        .args(["seedqr", "encode", "--variant", "compact", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .args(["--json-out", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout("");
    let json: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(json["operation"], "encode");
    assert_eq!(json["variant"], "compact");
    assert_eq!(json["word_count"], 12);
    assert_eq!(json["phrase"], PHRASE_12);
    assert_eq!(json["digits"], COMPACT_HEX_12); // payload field holds the hex
}

#[test]
fn compact_round_trip_via_cli() {
    let enc = mnemonic()
        .args(["seedqr", "encode", "--variant", "compact", "--from"])
        .arg(format!("phrase={PHRASE_24}"))
        .assert()
        .success();
    let hex = String::from_utf8(enc.get_output().stdout.clone())
        .unwrap()
        .trim()
        .to_string();
    mnemonic()
        .args(["seedqr", "decode", "--variant", "compact", "--from"])
        .arg(format!("seedqr={hex}"))
        .assert()
        .success()
        .stdout(format!("{PHRASE_24}\n"));
}

#[test]
fn encode_compact_rejects_15_word_cli() {
    let fifteen = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon address";
    let assertion = mnemonic()
        .args(["seedqr", "encode", "--variant", "compact", "--from"])
        .arg(format!("phrase={fifteen}"))
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("compact") && stderr.contains("only 12 or 24"),
        "expected compact word-count refusal; got: {stderr}"
    );
}

#[test]
fn decode_compact_uppercase_and_whitespace_hex() {
    // R0 M2 — hex is case-insensitive + whitespace-stripped. 16 bytes → 12-word.
    mnemonic()
        .args(["seedqr", "decode", "--variant", "compact", "--from"])
        .arg("seedqr=AABB CCDD EEFF 0011 2233 4455 6677 8899")
        .assert()
        .success();
}

#[test]
fn standard_decode_of_64_char_hex_clean_error() {
    // R0 M2 / footgun check — a 64-char all-zero compact hex under
    // --variant standard: 64 ∉ {48,60,72,84,96} → clean InvalidDigits
    // error (exit 1), NOT a panic.
    let assertion = mnemonic()
        .args(["seedqr", "decode", "--from"])
        .arg(format!("seedqr={COMPACT_HEX_24}"))
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("invalid digit count"),
        "expected clean InvalidDigits error; got: {stderr}"
    );
}

#[test]
fn encode_json_mode_24_word() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_24}"))
        .args(["--json-out", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout("");
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
    mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "seedqr: encode: invalid word count",
        ));
}

// v0.31.5 — 15/18/21-word counts are now ACCEPTED (previously refused).
// Canonical zero-entropy vectors derived via `mnemonic convert --from
// entropy=00..00 (20/24/28 bytes) --to phrase`.

#[test]
fn encode_accepts_15_word_count() {
    // 20 bytes of zeros → "abandon ×14 + address". BIP-39 index of
    // "address" is 27.
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon address";
    let expected_digits = "000000000000000000000000000000000000000000000000000000000027";
    mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={phrase}"))
        .assert()
        .success()
        .stdout(format!("{expected_digits}\n"));
}

#[test]
fn encode_accepts_18_word_count() {
    // 24 bytes of zeros → "abandon ×17 + agent". BIP-39 index 39.
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon agent";
    let expected_digits =
        "000000000000000000000000000000000000000000000000000000000000000000000039";
    mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={phrase}"))
        .assert()
        .success()
        .stdout(format!("{expected_digits}\n"));
}

#[test]
fn encode_accepts_21_word_count() {
    // 28 bytes of zeros → "abandon ×20 + admit". BIP-39 index 29.
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon admit";
    let expected_digits =
        "000000000000000000000000000000000000000000000000000000000000000000000000000000000029";
    mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={phrase}"))
        .assert()
        .success()
        .stdout(format!("{expected_digits}\n"));
}

#[test]
fn encode_json_mode_15_word() {
    // R0 I3b fold — JSON-envelope happy path for a new word count.
    // Confirms `word_count: 15` emits correctly.
    let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon address";
    let expected_digits = "000000000000000000000000000000000000000000000000000000000027";
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={phrase}"))
        .args(["--json-out", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout("");
    let json: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(json["schema_version"], "1");
    assert_eq!(json["operation"], "encode");
    assert_eq!(json["variant"], "standard");
    assert_eq!(json["word_count"], 15);
    assert_eq!(json["phrase"], phrase);
    assert_eq!(json["digits"], expected_digits);
}

#[test]
fn encode_rejects_25_word_count() {
    let bad = format!("{PHRASE_24} abandon");
    mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert()
        .failure()
        .code(1);
}

#[test]
fn encode_rejects_invalid_word() {
    let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon notaword";
    mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "seedqr: encode: BIP-39 checksum failure",
        ));
}

#[test]
fn encode_rejects_checksum_failure() {
    let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
    mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "seedqr: encode: BIP-39 checksum failure",
        ));
}

// ──────────────────────────────────────────────────────────────────────
// Round-trip
// ──────────────────────────────────────────────────────────────────────

#[test]
fn round_trip_12_word_text() {
    let encode_out = mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .assert()
        .success();
    let digits = String::from_utf8(encode_out.get_output().stdout.clone()).unwrap();
    let digits = digits.trim_end();

    mnemonic()
        .args(["seedqr", "decode", "--digits", digits])
        .assert()
        .success()
        .stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn round_trip_24_word_text() {
    let encode_out = mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_24}"))
        .assert()
        .success();
    let digits = String::from_utf8(encode_out.get_output().stdout.clone()).unwrap();
    let digits = digits.trim_end();

    mnemonic()
        .args(["seedqr", "decode", "--digits", digits])
        .assert()
        .success()
        .stdout(format!("{PHRASE_24}\n"));
}

#[test]
fn round_trip_12_word_through_json_envelope() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .args(["--json-out", path.to_str().unwrap()])
        .assert()
        .success();
    let json: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    let digits = json["digits"].as_str().unwrap();

    mnemonic()
        .args(["seedqr", "decode", "--digits", digits])
        .assert()
        .success()
        .stdout(format!("{PHRASE_12}\n"));
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
    mnemonic()
        .args(["seedqr", "decode", "--digits", DIGITS_12])
        .assert()
        .success()
        .stderr(predicates::str::contains("secret material on argv"))
        .stderr(predicates::str::contains("--digits"));
}

#[test]
fn encode_emits_argv_advisory_on_inline_form() {
    mnemonic()
        .args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .assert()
        .success()
        .stderr(predicates::str::contains("secret material on argv"))
        .stderr(predicates::str::contains("--from phrase="));
}

#[test]
fn decode_no_argv_advisory_on_stdin_form() {
    // Negate on the load-bearing template substring (per
    // secret_advisory.rs:36-38); negating on `supplied in argv` would be
    // vacuous since that substring is never emitted by any code path.
    let stderr = mnemonic()
        .args(["seedqr", "decode", "--digits", "-"])
        .write_stdin(DIGITS_12)
        .assert()
        .success();
    let stderr_bytes = stderr.get_output().stderr.clone();
    let stderr_str = String::from_utf8(stderr_bytes).unwrap();
    assert!(
        !stderr_str.contains("secret material on argv"),
        "stdin form must not emit argv-leakage advisory; got stderr: {stderr_str}"
    );
}
