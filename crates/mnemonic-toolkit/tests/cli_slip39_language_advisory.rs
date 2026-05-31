//! v0.37.11 — non-English BIP-39 wordlist-language advisory on `slip39` (path A of
//! the `mnem` footgun). SLIP-39 shares encode the master entropy (the BIP-39 wordlist
//! language is dropped); `combine --to entropy` likewise. Advisory fires (stderr) on
//! `split` (always, non-English) and `combine --to entropy` (non-English).

use assert_cmd::Command;

const FRENCH_12: &str = "abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abeille";
const ENGLISH_12: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const ADVISORY_SHARES: &str = "BIP-39 seed as SLIP-39 shares";
const ADVISORY_ENTROPY: &str = "BIP-39 seed as raw entropy";

fn slip39(args: &[&str]) -> (String, String) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

fn french_shares() -> Vec<String> {
    let (out, _e) = slip39(&[
        "split", "--from", &format!("phrase={FRENCH_12}"), "--language", "french", "--group-threshold", "1", "--group", "3,2",
    ]);
    out.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

#[test]
fn split_french_fires_shares_advisory() {
    let (_o, e) = slip39(&[
        "split", "--from", &format!("phrase={FRENCH_12}"), "--language", "french", "--group-threshold", "1", "--group", "3,2",
    ]);
    assert!(e.contains(ADVISORY_SHARES), "split must advise: {e:?}");
    assert!(e.contains("french"), "{e:?}");
}

#[test]
fn split_english_no_advisory() {
    let (_o, e) = slip39(&[
        "split", "--from", &format!("phrase={ENGLISH_12}"), "--language", "english",
        "--group-threshold", "1", "--group", "3,2",
    ]);
    assert!(!e.contains(ADVISORY_SHARES), "English split must NOT advise: {e:?}");
}

#[test]
fn combine_french_to_entropy_fires_advisory() {
    let shares = french_shares();
    let (_o, e) = slip39(&[
        "combine", "--share", &shares[0], "--share", &shares[1], "--language", "french", "--to",
        "entropy",
    ]);
    assert!(e.contains(ADVISORY_ENTROPY), "combine --to entropy must advise: {e:?}");
}

#[test]
fn combine_french_to_phrase_no_entropy_advisory() {
    let shares = french_shares();
    let (_o, e) = slip39(&[
        "combine", "--share", &shares[0], "--share", &shares[1], "--language", "french", "--to",
        "phrase",
    ]);
    assert!(
        !e.contains(ADVISORY_ENTROPY),
        "combine --to phrase keeps the language → must NOT advise: {e:?}"
    );
}
