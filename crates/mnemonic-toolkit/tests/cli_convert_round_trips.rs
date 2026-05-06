//! v0.6.1 Phase E — explicit A→B→A round-trip loop tests.
//!
//! Per the `convert-test-coverage-tightening` FOLLOWUP: v0.6.0 had one-direction
//! tests on each leg of the BIP-39 conversion graph, but no full-loop assertion
//! that A → B → A produces byte-identical A. These tests pin the symmetry of
//! the supported bidirectional pairs.
//!
//! Pattern: invoke `mnemonic convert` twice in sequence, capturing the first
//! invocation's stdout as the second invocation's input value. Compare the
//! second invocation's emission to the original input byte-for-byte.

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const TREZOR_24_ZERO_ENTROPY_HEX_64: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
const TREZOR_24_ZERO_MS1_24WORD: &str =
    "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w";

/// Helper: run `mnemonic convert` and return the value field of a single-`--to`
/// emission, stripped of the "<node>: " prefix and trailing newline.
fn convert_value(args: &[&str]) -> String {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let line = stdout.trim();
    let colon = line.find(": ").expect("convert output must be '<node>: <value>'");
    line[colon + 2..].to_string()
}

#[test]
fn round_trip_phrase_to_entropy_to_phrase() {
    let entropy = convert_value(&[
        "convert",
        "--from",
        &format!("phrase={TREZOR_24}"),
        "--to",
        "entropy",
    ]);
    assert_eq!(entropy, TREZOR_24_ZERO_ENTROPY_HEX_64);

    let phrase_back = convert_value(&[
        "convert",
        "--from",
        &format!("entropy={entropy}"),
        "--to",
        "phrase",
    ]);
    assert_eq!(
        phrase_back, TREZOR_24,
        "round-trip phrase → entropy → phrase must be byte-identical"
    );
}

#[test]
fn round_trip_entropy_to_ms1_to_entropy() {
    let ms1 = convert_value(&[
        "convert",
        "--from",
        &format!("entropy={TREZOR_24_ZERO_ENTROPY_HEX_64}"),
        "--to",
        "ms1",
    ]);
    assert_eq!(ms1, TREZOR_24_ZERO_MS1_24WORD);

    let entropy_back = convert_value(&[
        "convert",
        "--from",
        &format!("ms1={ms1}"),
        "--to",
        "entropy",
    ]);
    assert_eq!(
        entropy_back, TREZOR_24_ZERO_ENTROPY_HEX_64,
        "round-trip entropy → ms1 → entropy must be byte-identical"
    );
}

#[test]
fn round_trip_phrase_to_ms1_to_phrase_via_entropy_intermediate() {
    // ms-codec carries entropy, so phrase → ms1 traverses phrase → entropy →
    // ms1 (composite per SPEC §2). The reverse uses ms1 → entropy → phrase
    // (composite via the entropy intermediate).
    let ms1 = convert_value(&[
        "convert",
        "--from",
        &format!("phrase={TREZOR_24}"),
        "--to",
        "ms1",
    ]);
    let phrase_back = convert_value(&[
        "convert",
        "--from",
        &format!("ms1={ms1}"),
        "--to",
        "phrase",
    ]);
    assert_eq!(
        phrase_back, TREZOR_24,
        "round-trip phrase → ms1 → phrase must be byte-identical"
    );
}
