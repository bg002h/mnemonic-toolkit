//! v0.37.11 + ms-mnem Phase 3 Step 5+6 — non-English BIP-39 wordlist advisory on `bundle`.
//!
//! After Step 5, a non-English source emits a self-describing `mnem` ms1 card
//! (language preserved on-wire), so the language-losing advisory is SUPPRESSED for
//! non-English bundles. The advisory only fires when a secret-bearing slot emits
//! `entr` from a non-English run context (i.e. `slot.language == Some(English)` but
//! run_language is non-English — a rare corner case with no practical trigger today).
//!
//! See `design/SPEC_non_english_seed_advisory.md`.

use assert_cmd::Command;

// Checksum-valid all-zeros-entropy vectors (generated via `convert --to phrase`).
const FRENCH_12: &str = "abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abeille";
const ENGLISH_12: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
// A second, DISTINCT French seed (0x55 entropy) for the 2-of-2 distinctness case.
const FRENCH_12_B: &str = "enrichir officier enrichir officier enrichir officier enrichir officier enrichir officier enrichir olivier";
// A known mainnet bip84 account xpub + master fingerprint (watch-only case).
const BIP84_XPUB: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";
const FP_HEX: &str = "5436d724";

const ADVISORY_NEEDLE: &str = "BIP-39 seed as an ms1 card";

// VALID_MNEM_STR_LENGTHS for 16-byte entropy = 51 chars.
const MNEM_MS1_LEN_12WORD: usize = 51;

#[test]
fn french_phrase_bundle_emits_mnem_no_advisory() {
    // ms-mnem Phase 3 Step 5+6: a non-English phrase bundle now emits a
    // self-describing `mnem` ms1 card → advisory SUPPRESSED (no language loss).
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle", "--slot", &format!("@0.phrase={FRENCH_12}"), "--language", "french",
            "--template", "bip84", "--network", "mainnet", "--no-engraving-card",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // Advisory must NOT fire: the emitted ms1 is a mnem card (self-describing).
    assert!(!stderr.contains(ADVISORY_NEEDLE), "advisory must be suppressed for mnem emit: {stderr:?}");
    // The emitted ms1 must be a mnem card (length 51 for 12-word / 16-byte entropy).
    // In bundle text output, the ms1 value is a bare ms1... string on its own line
    // (after the "# ms1 (entropy, BCH-checksummed)" comment).
    let ms1_val = stdout
        .lines()
        .find(|l| l.starts_with("ms1") && !l.starts_with("ms1 ") && !l.contains("(entropy"))
        .unwrap_or("")
        .trim();
    assert_eq!(
        ms1_val.len(), MNEM_MS1_LEN_12WORD,
        "emitted ms1 must be mnem length {MNEM_MS1_LEN_12WORD}: got len={} val={ms1_val:?}",
        ms1_val.len()
    );
}

#[test]
fn english_phrase_bundle_no_advisory() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle", "--slot", &format!("@0.phrase={ENGLISH_12}"), "--language", "english",
            "--template", "bip84", "--network", "mainnet", "--no-engraving-card",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(!stderr.contains(ADVISORY_NEEDLE), "English must NOT fire: {stderr:?}");
}

#[test]
fn watch_only_french_bundle_no_advisory() {
    // No ms1 (no secret) → no advisory, even with --language french.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle", "--slot", &format!("@0.xpub={BIP84_XPUB}"), "--slot",
            &format!("@0.fingerprint={FP_HEX}"), "--language", "french", "--template", "bip84",
            "--network", "mainnet", "--no-engraving-card",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(!stderr.contains(ADVISORY_NEEDLE), "watch-only must NOT fire: {stderr:?}");
}

#[test]
fn french_bundle_json_stdout_valid_advisory_suppressed() {
    // ms-mnem Phase 3 Step 5+6: non-English bundle with --json → mnem card emitted,
    // advisory suppressed on stderr, stdout stays valid JSON.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle", "--slot", &format!("@0.phrase={FRENCH_12}"), "--language", "french",
            "--template", "bip84", "--network", "mainnet", "--no-engraving-card", "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // Advisory is suppressed (mnem card emitted, self-describing).
    assert!(!stderr.contains(ADVISORY_NEEDLE), "advisory must be suppressed for mnem: {stderr:?}");
    // stdout stays valid JSON.
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("stdout valid JSON");
    // The emitted ms1 must be a mnem card (length 51).
    // Bundle JSON schema: top-level "ms1": ["<value>"].
    let ms1_arr = v["ms1"].as_array().expect("top-level ms1 array");
    let ms1_val = ms1_arr[0].as_str().unwrap_or("");
    assert_eq!(
        ms1_val.len(), MNEM_MS1_LEN_12WORD,
        "emitted ms1 must be mnem length {MNEM_MS1_LEN_12WORD}: got {ms1_val:?}"
    );
}

#[test]
fn french_multisig_2of2_both_mnem_no_advisory() {
    // ms-mnem Phase 3 Step 5+6: both French cosigners emit mnem cards → advisory suppressed.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle", "--network", "mainnet", "--template", "wsh-sortedmulti", "--threshold",
            "2", "--no-engraving-card", "--language", "french", "--slot",
            &format!("@0.phrase={FRENCH_12}"), "--slot", &format!("@1.phrase={FRENCH_12_B}"),
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains(ADVISORY_NEEDLE),
        "advisory must be suppressed for all-mnem 2-of-2: {stderr:?}"
    );
}
