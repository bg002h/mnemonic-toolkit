//! v0.12.0 P2 — CLI happy-path tests for `mnemonic seed-xor`.
//!
//! Per SPEC §4 G2 + G3 (plain stdout). Round-trip + sortedness +
//! trailing-newline + default-language + Coldcard sizes.

use assert_cmd::Command;

const ABANDON_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

const LEGAL_TREZOR_12: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";

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
fn split_then_combine_round_trip_n2() {
    let from_arg = format!("phrase={LEGAL_TREZOR_12}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--shares",
        "2",
        "--deterministic-from-master",
    ]);
    assert_eq!(exit, 0);
    let shares: Vec<&str> = stdout.lines().collect();
    assert_eq!(shares.len(), 2);
    let s0 = format!("phrase={}", shares[0]);
    let s1 = format!("phrase={}", shares[1]);
    let (recovered, _stderr2, exit2) = combine(&["--share", &s0, "--share", &s1, "--shares", "2"]);
    assert_eq!(exit2, 0);
    let recovered_line = recovered.lines().next().unwrap();
    assert_eq!(recovered_line, LEGAL_TREZOR_12);
}

#[test]
fn split_n3_emits_3_lines_each_12_words() {
    let from_arg = format!("phrase={LEGAL_TREZOR_12}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--shares",
        "3",
        "--deterministic-from-master",
    ]);
    assert_eq!(exit, 0);
    let shares: Vec<&str> = stdout.lines().collect();
    assert_eq!(shares.len(), 3);
    for s in &shares {
        assert_eq!(s.split_whitespace().count(), 12);
    }
}

#[test]
fn split_n5_round_trip() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--shares",
        "5",
        "--deterministic-from-master",
    ]);
    assert_eq!(exit, 0);
    let shares: Vec<&str> = stdout.lines().collect();
    assert_eq!(shares.len(), 5);
    let share_flags: Vec<String> = shares.iter().map(|s| format!("phrase={s}")).collect();
    let mut combine_args = vec!["--shares", "5"];
    for sf in &share_flags {
        combine_args.push("--share");
        combine_args.push(sf.as_str());
    }
    let (recovered, _stderr2, exit2) = combine(&combine_args);
    assert_eq!(exit2, 0);
    assert_eq!(recovered.lines().next().unwrap(), ABANDON_12);
}

#[test]
fn split_default_language_is_english() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--shares",
        "2",
        "--deterministic-from-master",
    ]);
    assert_eq!(exit, 0);
    assert_eq!(stdout.lines().count(), 2);
}

#[test]
fn split_trailing_newline() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--shares",
        "2",
        "--deterministic-from-master",
    ]);
    assert_eq!(exit, 0);
    assert!(
        stdout.ends_with('\n'),
        "split stdout must end with newline; got {stdout:?}"
    );
}

#[test]
fn split_24_word_round_trip() {
    // BIP-39 zero-entropy 24-word vector
    let twenty_four = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    let from_arg = format!("phrase={twenty_four}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--shares",
        "3",
        "--deterministic-from-master",
    ]);
    assert_eq!(exit, 0);
    let shares: Vec<&str> = stdout.lines().collect();
    assert_eq!(shares.len(), 3);
    for s in &shares {
        assert_eq!(s.split_whitespace().count(), 24, "share must be 24 words");
    }
    let share_flags: Vec<String> = shares.iter().map(|s| format!("phrase={s}")).collect();
    let mut combine_args = vec!["--shares", "3"];
    for sf in &share_flags {
        combine_args.push("--share");
        combine_args.push(sf.as_str());
    }
    let (recovered, _stderr2, exit2) = combine(&combine_args);
    assert_eq!(exit2, 0);
    assert_eq!(recovered.lines().next().unwrap(), twenty_four);
}

#[test]
fn split_18_word_round_trip_coldcard_native_size() {
    // BIP-39 18-word from-zero vector
    let m = bip39::Mnemonic::from_entropy_in(bip39::Language::English, &[0u8; 24]).unwrap();
    let phrase = m.to_string();
    let from_arg = format!("phrase={phrase}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--shares",
        "2",
        "--deterministic-from-master",
    ]);
    assert_eq!(exit, 0);
    let shares: Vec<&str> = stdout.lines().collect();
    assert_eq!(shares.len(), 2);
    let s0 = format!("phrase={}", shares[0]);
    let s1 = format!("phrase={}", shares[1]);
    let (recovered, _stderr2, exit2) = combine(&["--share", &s0, "--share", &s1, "--shares", "2"]);
    assert_eq!(exit2, 0);
    assert_eq!(recovered.lines().next().unwrap(), phrase);
}

#[test]
fn split_random_round_trip_non_deterministic() {
    // Without --deterministic-from-master, the OS CSPRNG produces
    // unpredictable shares — but round-trip still works.
    let from_arg = format!("phrase={LEGAL_TREZOR_12}");
    let (stdout, _stderr, exit) = split(&["--from", &from_arg, "--shares", "3"]);
    assert_eq!(exit, 0);
    let shares: Vec<&str> = stdout.lines().collect();
    assert_eq!(shares.len(), 3);
    let share_flags: Vec<String> = shares.iter().map(|s| format!("phrase={s}")).collect();
    let mut combine_args = vec!["--shares", "3"];
    for sf in &share_flags {
        combine_args.push("--share");
        combine_args.push(sf.as_str());
    }
    let (recovered, _stderr2, exit2) = combine(&combine_args);
    assert_eq!(exit2, 0);
    assert_eq!(recovered.lines().next().unwrap(), LEGAL_TREZOR_12);
}

#[test]
fn combine_trailing_newline() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (stdout_split, _, _) = split(&[
        "--from",
        &from_arg,
        "--shares",
        "2",
        "--deterministic-from-master",
    ]);
    let shares: Vec<&str> = stdout_split.lines().collect();
    let s0 = format!("phrase={}", shares[0]);
    let s1 = format!("phrase={}", shares[1]);
    let (recovered, _, exit) = combine(&["--share", &s0, "--share", &s1, "--shares", "2"]);
    assert_eq!(exit, 0);
    assert!(recovered.ends_with('\n'));
}

#[test]
fn spanish_round_trip() {
    let m = bip39::Mnemonic::from_entropy_in(bip39::Language::Spanish, &[0u8; 16]).unwrap();
    let phrase = m.to_string();
    let from_arg = format!("phrase={phrase}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--shares",
        "2",
        "--language",
        "spanish",
        "--deterministic-from-master",
    ]);
    assert_eq!(exit, 0);
    let shares: Vec<&str> = stdout.lines().collect();
    assert_eq!(shares.len(), 2);
    let s0 = format!("phrase={}", shares[0]);
    let s1 = format!("phrase={}", shares[1]);
    let (recovered, _, exit2) = combine(&[
        "--share",
        &s0,
        "--share",
        &s1,
        "--shares",
        "2",
        "--language",
        "spanish",
    ]);
    assert_eq!(exit2, 0);
    assert_eq!(recovered.lines().next().unwrap(), phrase);
}
