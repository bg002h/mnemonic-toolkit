//! v0.31.6 — `seedqr-digits-from-input-unification` integration tests.
//!
//! Covers BOTH surfaces:
//! - `mnemonic convert --from seedqr=<digits> --to <node>` (Option 3
//!   end-to-end wiring through classify_edge + compute_outputs).
//! - `mnemonic seedqr decode --from seedqr=<digits>` (canonical) +
//!   `--digits` deprecation warning + clap conflict + required-input.

use assert_cmd::Command;

// Canonical BIP-39 12-word zero-entropy vector: "abandon ×11 + about".
const PHRASE_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const DIGITS_12: &str = "000000000000000000000000000000000000000000000003";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

// ──────────────────────────────────────────────────────────────────────
// convert --from seedqr=
// ──────────────────────────────────────────────────────────────────────

#[test]
fn convert_from_seedqr_to_phrase_happy_path() {
    let out = mnemonic()
        .args([
            "convert",
            "--from",
            &format!("seedqr={DIGITS_12}"),
            "--to",
            "phrase",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("phrase: {PHRASE_12}\n"));
}

#[test]
fn convert_from_seedqr_to_entropy_happy_path() {
    let out = mnemonic()
        .args([
            "convert",
            "--from",
            &format!("seedqr={DIGITS_12}"),
            "--to",
            "entropy",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, "entropy: 00000000000000000000000000000000\n");
}

#[test]
fn convert_from_seedqr_to_xpub_bip84() {
    // seedqr → phrase → BIP-84 account xpub. Confirms the PBKDF2 derivation
    // path is reached (edge_uses_pbkdf2 includes Seedqr).
    let out = mnemonic()
        .args([
            "convert",
            "--from",
            &format!("seedqr={DIGITS_12}"),
            "--to",
            "xpub",
            "--template",
            "bip84",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.starts_with("xpub: xpub"),
        "expected xpub output; got: {stdout}"
    );
}

#[test]
fn convert_from_seedqr_stdin_to_phrase_happy_path() {
    // R0 M1 fold — `--from seedqr=-` consumes the digit-string from stdin.
    let out = mnemonic()
        .args(["convert", "--from", "seedqr=-", "--to", "phrase"])
        .write_stdin(DIGITS_12)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("phrase: {PHRASE_12}\n"));
}

#[test]
fn convert_from_seedqr_invalid_digits_refused() {
    let assertion = mnemonic()
        .args(["convert", "--from", "seedqr=123", "--to", "phrase"])
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("seedqr: convert: decode") && stderr.contains("invalid digit count"),
        "expected seedqr decode error; got: {stderr}"
    );
}

#[test]
fn convert_to_seedqr_rejected_by_clap() {
    // `seedqr` is intentionally absent from the `--to` PossibleValuesParser
    // list; clap rejects `--to seedqr` at parse-time (exit 2).
    let assertion = mnemonic()
        .args([
            "convert",
            "--from",
            &format!("phrase={PHRASE_12}"),
            "--to",
            "seedqr",
        ])
        .assert()
        .failure()
        .code(64); // EX_USAGE (clap parse error via the sysexits main wrapper)
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("invalid value 'seedqr' for '--to"),
        "expected clap possible-values rejection; got: {stderr}"
    );
}

// ──────────────────────────────────────────────────────────────────────
// seedqr decode --from seedqr=  +  --digits deprecation
// ──────────────────────────────────────────────────────────────────────

#[test]
fn decode_from_seedqr_happy_path() {
    mnemonic()
        .args(["seedqr", "decode", "--from", &format!("seedqr={DIGITS_12}")])
        .assert()
        .success()
        .stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn decode_from_seedqr_stdin_happy_path() {
    mnemonic()
        .args(["seedqr", "decode", "--from", "seedqr=-"])
        .write_stdin(DIGITS_12)
        .assert()
        .success()
        .stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn decode_digits_deprecation_warning() {
    let assertion = mnemonic()
        .args(["seedqr", "decode", "--digits", DIGITS_12])
        .assert()
        .success()
        .stdout(format!("{PHRASE_12}\n"));
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--digits is deprecated") && stderr.contains("--from seedqr="),
        "expected deprecation notice citing --from seedqr=; got: {stderr}"
    );
}

#[test]
fn decode_both_digits_and_from_refused_clap_conflict() {
    // R0 I3 fold — clap-level conflicts_with → exit 2 at parse-time.
    let assertion = mnemonic()
        .args([
            "seedqr",
            "decode",
            "--digits",
            DIGITS_12,
            "--from",
            &format!("seedqr={DIGITS_12}"),
        ])
        .assert()
        .failure()
        .code(64); // EX_USAGE (clap conflicts_with via the sysexits main wrapper)
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("cannot be used with"),
        "expected clap conflict error; got: {stderr}"
    );
}

#[test]
fn decode_neither_digits_nor_from_required_input() {
    let assertion = mnemonic()
        .args(["seedqr", "decode"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("seedqr decode requires an input") && stderr.contains("--from seedqr="),
        "expected required-input refusal; got: {stderr}"
    );
}

#[test]
fn decode_from_non_seedqr_node_refused() {
    let assertion = mnemonic()
        .args(["seedqr", "decode", "--from", &format!("phrase={PHRASE_12}")])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("accepts only the `seedqr` node type") && stderr.contains("got `phrase`"),
        "expected non-seedqr-node refusal; got: {stderr}"
    );
}
