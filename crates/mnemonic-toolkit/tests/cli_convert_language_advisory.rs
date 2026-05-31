//! v0.37.11 — non-English BIP-39 wordlist-language advisory on `convert --to entropy`
//! (path A of the `mnem` footgun). Raw entropy drops the wordlist language. The
//! advisory fires (stderr) once when `--language` is non-English and a `entropy`
//! target is present; key-deriving targets (xprv/xpub/wif/...) do not fire.

use assert_cmd::Command;

// French / English all-zeros-entropy 12-word vectors (checksum-valid).
const FRENCH_12: &str = "abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abeille";

const ADVISORY_NEEDLE: &str = "BIP-39 seed as raw entropy";

#[test]
fn french_to_entropy_fires_advisory() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert", "--from", &format!("phrase={FRENCH_12}"), "--language", "french",
            "--to", "entropy",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains(ADVISORY_NEEDLE), "must fire: {stderr:?}");
    assert!(stderr.contains("french"), "{stderr:?}");
}

#[test]
fn french_multi_target_with_entropy_fires_once() {
    // entropy co-occurring with a key target → single advisory (targets.contains, once).
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert", "--from", &format!("phrase={FRENCH_12}"), "--language", "french",
            "--to", "xprv,entropy", "--template", "bip84", "--network", "mainnet",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(stderr.matches(ADVISORY_NEEDLE).count(), 1, "exactly once: {stderr:?}");
}

#[test]
fn french_to_xprv_no_advisory() {
    // A derived key already baked in the language — no re-recovery ambiguity.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert", "--from", &format!("phrase={FRENCH_12}"), "--language", "french",
            "--to", "xprv", "--template", "bip84", "--network", "mainnet",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(!stderr.contains(ADVISORY_NEEDLE), "key target must NOT fire: {stderr:?}");
}

#[test]
fn english_to_entropy_no_advisory() {
    let english_12 = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert", "--from", &format!("phrase={english_12}"), "--language", "english",
            "--to", "entropy",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(!stderr.contains(ADVISORY_NEEDLE), "English must NOT fire: {stderr:?}");
}

#[test]
fn to_seedqr_still_rejected_at_parse() {
    // Non-regression: `--to seedqr` is an input-only node, refused at clap parse.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert", "--from", &format!("phrase={FRENCH_12}"), "--language", "french",
            "--to", "seedqr",
        ])
        .assert()
        .failure();
}
