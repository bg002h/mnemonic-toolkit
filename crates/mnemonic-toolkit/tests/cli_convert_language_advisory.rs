//! v0.37.11 — non-English BIP-39 wordlist-language advisory on `convert` (path A of
//! the `mnem` footgun). Both language-agnostic targets — raw entropy AND an ms1 card
//! — drop the wordlist language. The advisory fires (stderr) when `--language` is
//! non-English and an `entropy` or `ms1` target is present; key-deriving targets
//! (xprv/xpub/wif/...) and the language-keeping `phrase` target do not fire. A
//! malformed phrase errors out before any advisory (emit is post-`compute_outputs`).

use assert_cmd::Command;

// French / English all-zeros-entropy 12-word vectors (checksum-valid).
const FRENCH_12: &str = "abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abeille";
// Same French head with a wrong final word → BIP-39 checksum failure (parse error).
const FRENCH_12_BAD_CKSUM: &str = "abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser";

const ADVISORY_NEEDLE: &str = "BIP-39 seed as raw entropy";
const ADVISORY_MS1: &str = "BIP-39 seed as an ms1 card";

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
fn french_to_ms1_emits_mnem_no_advisory() {
    // ms-mnem Phase 3 Step 5+6: `convert --language french --to ms1` now emits a
    // self-describing `mnem` ms1 card (language on-wire) → advisory SUPPRESSED.
    // The old advisory ("language-losing entr") was correct pre-Step-5; after Step 5
    // the footgun is resolved by design.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert", "--from", &format!("phrase={FRENCH_12}"), "--language", "french",
            "--to", "ms1",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // Advisory must NOT fire: the emitted ms1 is a mnem card.
    assert!(!stderr.contains(ADVISORY_MS1), "advisory must be suppressed for mnem ms1: {stderr:?}");
    // The emitted ms1 must be a mnem string (length 51 for 12-word / 16-byte entropy).
    // Output format: "ms1: <value>\n" — extract the value after "ms1: ".
    let ms1_val = stdout
        .lines()
        .find(|l| l.trim_start().starts_with("ms1:"))
        .and_then(|l| l.split_once(':').map(|x| x.1))
        .map(|s| s.trim())
        .unwrap_or_else(|| stdout.trim());
    assert_eq!(
        ms1_val.len(), 51,
        "emitted ms1 must be mnem (len=51): got len={} val={ms1_val:?}",
        ms1_val.len()
    );
}

#[test]
fn english_to_ms1_no_advisory() {
    let english_12 = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert", "--from", &format!("phrase={english_12}"), "--language", "english",
            "--to", "ms1",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(!stderr.contains(ADVISORY_MS1), "English ms1 must NOT fire: {stderr:?}");
}

#[test]
fn entropy_to_french_phrase_no_advisory() {
    // M2 (end-of-cycle R0): a `phrase` target re-encodes the entropy in `--language`
    // → the output IS the localized mnemonic, no language loss. (phrase→phrase is a
    // refused identity edge, so the language-keeping case is exercised entropy→phrase.)
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert", "--from", "entropy=00000000000000000000000000000000", "--language",
            "french", "--to", "phrase",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stdout.contains("abaisser"), "emits the French mnemonic: {stdout:?}");
    assert!(!stderr.contains(ADVISORY_NEEDLE), "phrase target must NOT fire: {stderr:?}");
    assert!(!stderr.contains(ADVISORY_MS1), "phrase target must NOT fire: {stderr:?}");
}

#[test]
fn malformed_french_phrase_errors_without_advisory() {
    // I1 (end-of-cycle R0): the advisory is emitted AFTER compute_outputs succeeds,
    // so a bad-checksum phrase exits with an error and never advises.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert", "--from", &format!("phrase={FRENCH_12_BAD_CKSUM}"), "--language", "french",
            "--to", "entropy",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(!stderr.contains(ADVISORY_NEEDLE), "must NOT advise-then-error: {stderr:?}");
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
