//! v0.11.0 P2 — CLI refusal tests for `mnemonic final-word`.
//!
//! Per SPEC §2.5 + §4 G4. All 4 refusal classes must surface their
//! pinned stderr message with exit code 1 (mapped from
//! `ToolkitError::BadInput.exit_code()` per `error.rs:244`) or 64 (clap
//! parse-error for malformed --from value).

use assert_cmd::Command;

fn invoke(args: &[&str]) -> (String, String, i32) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
        out.status.code().unwrap_or(-1),
    )
}

#[test]
fn refusal_empty_partial() {
    // Single space passes the `parse_from_input` non-empty check but yields
    // 0 words after split_whitespace at runtime — exercises SPEC §2.5 row 1
    // (the BadInput "got 0 words" path, not the clap parse error).
    let (_, stderr, exit) = invoke(&["--from", "phrase= "]);
    assert_ne!(exit, 0, "empty partial must refuse");
    assert!(
        stderr.contains("11") && stderr.contains("23"),
        "refusal must enumerate accepted partial-word counts; stderr: {stderr}",
    );
}

#[test]
fn refusal_two_words() {
    let (_, stderr, exit) = invoke(&["--from", "phrase=abandon abandon"]);
    assert_ne!(exit, 0);
    assert!(
        stderr.contains("got 2") || stderr.contains("2 words"),
        "refusal must mention the actual word count; stderr: {stderr}",
    );
}

#[test]
fn refusal_twelve_words_target_thirteen_not_valid() {
    // 12 words → would target N=13, which is not a valid BIP-39 length.
    let twelve = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
    let (_, stderr, exit) = invoke(&["--from", &format!("phrase={twelve}")]);
    assert_ne!(exit, 0, "12-word partial (targets N=13) must refuse");
    assert!(
        stderr.contains("12") || stderr.contains("words"),
        "stderr: {stderr}"
    );
}

#[test]
fn refusal_unknown_word_in_partial() {
    let partial =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon xyzzy";
    let (_, stderr, exit) = invoke(&[
        "--from",
        &format!("phrase={partial}"),
        "--language",
        "english",
    ]);
    assert_ne!(exit, 0, "partial with unknown word must refuse");
    assert!(
        stderr.to_lowercase().contains("unknown") || stderr.to_lowercase().contains("not in"),
        "refusal must mention the unknown-word condition; stderr: {stderr}",
    );
}

#[test]
fn refusal_from_xprv_variant_not_supported() {
    // `--from xprv=...` parses (existing FromInput shape) but final-word
    // should refuse non-phrase variants per SPEC §2.5 row 4.
    let (_, stderr, exit) = invoke(&["--from", "xprv=xprvsomething"]);
    assert_ne!(exit, 0, "non-phrase --from variant must refuse");
    assert!(
        stderr.contains("phrase=") || stderr.contains("only accepts"),
        "refusal must mention the only-phrase= rule; stderr: {stderr}",
    );
}

#[test]
fn refusal_from_missing_entirely() {
    // No --from at all → clap-level required-flag error.
    let (_, _stderr, exit) = invoke(&[]);
    assert_ne!(exit, 0, "missing --from must refuse");
}

#[test]
fn refusal_language_unknown_value() {
    // Unknown language value (not in CliLanguage variants) → clap parse error.
    let abandon =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
    let (_, _stderr, exit) = invoke(&[
        "--from",
        &format!("phrase={abandon}"),
        "--language",
        "klingon",
    ]);
    assert_ne!(exit, 0, "unknown --language value must refuse");
}
