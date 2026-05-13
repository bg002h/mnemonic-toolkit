//! Phase 1 argv-leakage closure — happy-path RED tests.
//!
//! 8 behavior cells covering the 9 closed flag-rows minus the xprv slot
//! (whose stdin consumption is structurally non-observable because
//! `bundle::detect_bundle_mode` rejects on the `SlotSubkey::Xprv` enum
//! tag at `bundle.rs:470`, independent of the slot value — so stdin
//! substitution produces no externally-distinguishable behavior change
//! versus the literal `-` value). The xprv `=-` route is covered
//! structurally by `lint_argv_secret_flags.rs` (asserts the
//! `slot_stdin` evidence anchor in bundle.rs) and
//! `cli_secret_in_argv_warning.rs` cell 4 (asserts inline-xprv emits
//! the secret-in-argv advisory).
//!
//! Future maintenance: when v0.5+ adds ms-codec XPRV-tag support and
//! the `bundle.rs:470` reject branch is removed, add a 9th cell here
//! mirroring cells 1-3 (`@0.xprv=-` + valid xprv on stdin → success).
//!
//! Authoritative reference: `design/SPEC_secret_memory_hygiene_v0_9_0.md`
//! §1 item 1; survey §5 toolkit table.
//!
//! Closure pairing (5 distinct impl changes → 9 flag-rows; 8 cells here):
//! 1. `slot_input.rs::parse_slot_input` `=-` parser extension covers
//!    4 behavior-observable flag-rows: cells 1-3 (bundle slot phrase /
//!    entropy / wif) + cell 4 (verify-bundle slot phrase). The xprv
//!    fifth row is structurally-covered only (see header note).
//! 2. `BundleArgs --passphrase-stdin`: cell 5
//! 3. `VerifyBundleArgs --passphrase-stdin`: cell 6
//! 4. `DeriveChildArgs --passphrase-stdin`: cell 7
//! 5. `ConvertArgs --bip38-passphrase-stdin`: cell 8

use assert_cmd::Command;
use predicates::prelude::*;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const TREZOR_24_ENTROPY_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

// SPEC_V4 from cli_convert_bip38.rs: BIP-38 spec test vector with a U+0000
// NULL byte in the NFC-normalized passphrase byte sequence. POSIX argv
// cannot carry this passphrase — only `--bip38-passphrase-stdin` can.
const BIP38_NULL_PASS: &str = "\u{03D2}\u{0301}\u{0000}\u{10400}\u{1F4A9}";
const BIP38_NULL_WIF: &str = "5Jajm8eQ22H3pGWLEVCXyvND8dQZhiQhoLJNKjYXk9roUFTMSZ4";
const BIP38_NULL_EXPECTED: &str = "6PRW5o9FLp4gJDDVqJQKJFTpMvdsSGJxMYHtHaQBF3ooa8mwD69bapcDQn";

// A valid mainnet compressed WIF (SPEC_V5 from cli_convert_bip38.rs).
const MAINNET_WIF: &str = "KwYgW8gcxj1JWJXhPSu4Fqwzfhp5Yfi42mdYmMa4XqK7NJxXUSK7";

// ============================================================================
// Cell 1 — `bundle --slot @0.phrase=-`
// ============================================================================

#[test]
fn bundle_slot_phrase_stdin_succeeds() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            "@0.phrase=-",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--no-engraving-card",
        ])
        .write_stdin(TREZOR_24.as_bytes())
        .assert()
        .success();
}

// ============================================================================
// Cell 2 — `bundle --slot @0.entropy=-`
// ============================================================================

#[test]
fn bundle_slot_entropy_stdin_succeeds() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            "@0.entropy=-",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--no-engraving-card",
        ])
        .write_stdin(TREZOR_24_ENTROPY_HEX.as_bytes())
        .assert()
        .success();
}

// ============================================================================
// Cell 3 — `bundle --slot @0.wif=-`
// ============================================================================

#[test]
fn bundle_slot_wif_stdin_succeeds() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            "@0.wif=-",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--no-engraving-card",
        ])
        .write_stdin(MAINNET_WIF.as_bytes())
        .assert()
        .success();
}

// ============================================================================
// Cell 4 — `verify-bundle --slot @0.phrase=-`
// ============================================================================

#[test]
fn verify_bundle_slot_phrase_stdin_succeeds() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let ms1 = fixture
        .lines()
        .find(|l| l.starts_with("ms1") && !l.contains(' '))
        .expect("compact ms1 line")
        .to_string();
    let mk1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    let md1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    assert!(!mk1.is_empty() && !md1.is_empty());

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--slot".into(),
        "@0.phrase=-".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
        "--ms1".into(),
        ms1,
    ];
    for s in &mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in &md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .write_stdin(TREZOR_24.as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("result: ok"));
}

// ============================================================================
// Cell 5 — `bundle --passphrase-stdin`
//
// Pipes an empty-string passphrase. The pinned bip84-mainnet fixture was
// generated without a passphrase (default ""), so a stdin-empty
// passphrase must round-trip byte-exact against it.
// ============================================================================

#[test]
fn bundle_passphrase_stdin_empty_round_trips_pinned_fixture() {
    let expected =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--passphrase-stdin",
            "--no-engraving-card",
        ])
        .write_stdin("".as_bytes())
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout, expected,
        "passphrase-stdin with empty stdin must round-trip the no-passphrase pinned fixture",
    );
}

// ============================================================================
// Cell 6 — `verify-bundle --passphrase-stdin`
// ============================================================================

#[test]
fn verify_bundle_passphrase_stdin_empty_round_trips_pinned_fixture() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let ms1 = fixture
        .lines()
        .find(|l| l.starts_with("ms1") && !l.contains(' '))
        .expect("compact ms1 line")
        .to_string();
    let mk1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    let md1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--slot".into(),
        format!("@0.phrase={TREZOR_24}"),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
        "--passphrase-stdin".into(),
        "--ms1".into(),
        ms1,
    ];
    for s in &mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in &md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .write_stdin("".as_bytes())
        .assert()
        .success()
        .stdout(predicate::str::contains("result: ok"));
}

// ============================================================================
// Cell 7 — `derive-child --passphrase-stdin`
//
// `--from phrase=<…> --passphrase-stdin` reads the BIP-39 extension
// passphrase from stdin. Pipes an empty passphrase to keep the cell
// passphrase-agnostic; the assertion is that the flag is accepted and
// the command produces a non-empty BIP-85 child entropy.
// ============================================================================

#[test]
fn derive_child_passphrase_stdin_phrase_master_succeeds() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("phrase={TREZOR_24}"),
            "--application",
            "bip39",
            "--length",
            "12",
            "--index",
            "0",
            "--passphrase-stdin",
        ])
        .write_stdin("".as_bytes())
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.trim().is_empty(),
        "derive-child --passphrase-stdin must emit a child mnemonic"
    );
}

// ============================================================================
// Cell 8 — `convert --bip38-passphrase-stdin`
//
// Closes the BIP-38 V3 NULL-byte gap. The U+0000-containing NFC
// passphrase is impossible to express on POSIX argv.
// ============================================================================

#[test]
fn convert_bip38_passphrase_stdin_null_byte_succeeds() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("wif={BIP38_NULL_WIF}"),
            "--to",
            "bip38",
            "--bip38-passphrase-stdin",
        ])
        .write_stdin(BIP38_NULL_PASS.as_bytes())
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let line = stdout.trim();
    let colon = line.find(": ").expect("convert output must be '<node>: <value>'");
    assert_eq!(&line[colon + 2..], BIP38_NULL_EXPECTED);
}
