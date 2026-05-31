//! v0.37.11 — non-English BIP-39 wordlist-language advisory on `bundle` (path A of
//! the `mnem` footgun). ms1 carries only the entropy, not the wordlist language; a
//! non-English seed recovered with English-defaulted software derives a different
//! wallet. The advisory fires (stderr) once per secret-bearing bundle when
//! `--language` is non-English. See `design/SPEC_non_english_seed_advisory.md`.

use assert_cmd::Command;
use predicates::prelude::*;

// Checksum-valid all-zeros-entropy vectors (generated via `convert --to phrase`).
const FRENCH_12: &str = "abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abaisser abeille";
const ENGLISH_12: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
// A second, DISTINCT French seed (0x55 entropy) for the 2-of-2 distinctness case.
const FRENCH_12_B: &str = "enrichir officier enrichir officier enrichir officier enrichir officier enrichir officier enrichir olivier";
// A known mainnet bip84 account xpub + master fingerprint (watch-only case).
const BIP84_XPUB: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";
const FP_HEX: &str = "5436d724";

const ADVISORY_NEEDLE: &str = "BIP-39 seed as an ms1 card";

#[test]
fn french_phrase_bundle_fires_advisory_once() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle", "--slot", &format!("@0.phrase={FRENCH_12}"), "--language", "french",
            "--template", "bip84", "--network", "mainnet", "--no-engraving-card",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains(ADVISORY_NEEDLE), "advisory must fire: {stderr:?}");
    assert!(stderr.contains("french"), "names the language: {stderr:?}");
    assert_eq!(
        stderr.matches(ADVISORY_NEEDLE).count(),
        1,
        "exactly once (single chokepoint): {stderr:?}"
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
fn french_bundle_json_stdout_unchanged_advisory_on_stderr() {
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
    // Advisory is on stderr only; stdout stays valid JSON.
    let _: serde_json::Value = serde_json::from_str(&stdout).expect("stdout valid JSON");
    assert!(!stdout.contains(ADVISORY_NEEDLE), "advisory must NOT be on stdout");
    assert!(stderr.contains(ADVISORY_NEEDLE), "advisory must be on stderr");
}

#[test]
fn french_multisig_2of2_fires_advisory_once() {
    // Two DISTINCT French seeds (avoids BIP-388 same-key collision). The emit lives
    // in the single `emit_unified` chokepoint → fires once, not per-cosigner.
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
    assert_eq!(
        stderr.matches(ADVISORY_NEEDLE).count(),
        1,
        "exactly once for 2-of-2 (not per-cosigner): {stderr:?}"
    );
}
