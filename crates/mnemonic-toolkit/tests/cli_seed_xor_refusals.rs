//! v0.12.0 P2 — CLI refusal tests for `mnemonic seed-xor`.
//!
//! Per SPEC §2.5 — 9 refusal classes.

use assert_cmd::Command;

const ABANDON_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn split(args: &[&str]) -> (String, String, i32) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("split")
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
        out.status.code().unwrap_or(-1),
    )
}

fn combine(args: &[&str]) -> (String, String, i32) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("combine")
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
fn refusal_split_wrong_word_count() {
    // 2 words doesn't parse as BIP-39 — fails Mnemonic::parse_in before reaching word-count check
    // (which is post-parse). Either way, exit is non-zero.
    let (_, stderr, exit) = split(&["--from", "phrase=abandon abandon", "--shares", "2"]);
    assert_ne!(exit, 0);
    // bip39 error mapping fires (friendly_bip39 or default Display).
    assert!(
        !stderr.is_empty(),
        "must emit some refusal text; got empty stderr",
    );
}

#[test]
fn refusal_split_shares_less_than_2() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (_, stderr, exit) = split(&["--from", &from_arg, "--shares", "1"]);
    assert_ne!(exit, 0);
    assert!(
        stderr.contains(">= 2") || stderr.contains("must be >= 2"),
        "must mention --shares must be >= 2; got: {stderr}",
    );
}

#[test]
fn refusal_combine_cardinality_mismatch() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (out, _, _) = split(&[
        "--from",
        &from_arg,
        "--shares",
        "3",
        "--deterministic-from-master",
    ]);
    let shares: Vec<&str> = out.lines().collect();
    let s0 = format!("phrase={}", shares[0]);
    let s1 = format!("phrase={}", shares[1]);
    // Supply only 2 shares but assert 3
    let (_, stderr, exit) = combine(&["--share", &s0, "--share", &s1, "--shares", "3"]);
    assert_ne!(exit, 0);
    assert!(
        stderr.contains("requires exactly 3 --share arguments")
            || stderr.contains("--shares N requires"),
        "must mention cardinality mismatch; got: {stderr}",
    );
}

#[test]
fn refusal_combine_mixed_word_counts() {
    let twelve = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let twenty_four = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    let s12 = format!("phrase={twelve}");
    let s24 = format!("phrase={twenty_four}");
    let (_, stderr, exit) = combine(&["--share", &s12, "--share", &s24, "--shares", "2"]);
    assert_ne!(exit, 0);
    assert!(
        stderr.contains("same word count") || stderr.contains("mix of"),
        "must mention mixed lengths; got: {stderr}",
    );
}

#[test]
fn refusal_combine_invalid_bip39_checksum() {
    // A 12-word phrase with a deliberately wrong last word (not the checksum word).
    // "abandon × 11 abandon" — checksum of all-zeros is "about", so trailing "abandon" fails checksum.
    let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
    let good = format!("phrase={ABANDON_12}");
    let s_bad = format!("phrase={bad}");
    let (_, stderr, exit) = combine(&["--share", &good, "--share", &s_bad, "--shares", "2"]);
    assert_ne!(exit, 0);
    assert!(
        stderr.contains("invalid BIP-39 checksum") || stderr.to_lowercase().contains("checksum"),
        "must mention checksum failure; got: {stderr}",
    );
}

#[test]
fn refusal_combine_unknown_word() {
    let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon xyzzy";
    let s_good = format!("phrase={ABANDON_12}");
    let s_bad = format!("phrase={bad}");
    let (_, stderr, exit) = combine(&["--share", &s_good, "--share", &s_bad, "--shares", "2"]);
    assert_ne!(exit, 0);
    assert!(
        stderr.to_lowercase().contains("unknown") || stderr.contains("not in"),
        "must mention unknown word; got: {stderr}",
    );
}

#[test]
fn refusal_split_non_phrase_node() {
    // `xprv=` should refuse with the non-phrase variant message
    let (_, stderr, exit) = split(&["--from", "xprv=xprvsomething", "--shares", "2"]);
    assert_ne!(exit, 0);
    assert!(
        stderr.contains("seed-xor only accepts phrase=") || stderr.contains("only accepts"),
        "must mention phrase-only rule; got: {stderr}",
    );
}

#[test]
fn refusal_combine_non_phrase_node() {
    let (_, stderr, exit) = combine(&[
        "--share",
        "xprv=xprvsomething",
        "--share",
        "xprv=xprvsomethingelse",
        "--shares",
        "2",
    ]);
    assert_ne!(exit, 0);
    assert!(
        stderr.contains("seed-xor only accepts phrase=") || stderr.contains("only accepts"),
        "must mention phrase-only rule; got: {stderr}",
    );
}

#[test]
fn refusal_split_missing_required_flags() {
    let (_, _stderr, exit) = split(&[]);
    assert_ne!(exit, 0, "missing required --from + --shares must refuse");
}

#[test]
fn refusal_combine_missing_required_flags() {
    let (_, _stderr, exit) = combine(&[]);
    assert_ne!(exit, 0, "missing required --share + --shares must refuse");
}

#[test]
fn refusal_unknown_language() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (_, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--shares",
        "2",
        "--language",
        "klingon",
    ]);
    assert_ne!(exit, 0, "unknown --language must refuse");
}
