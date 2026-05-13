//! Phase 1 argv-leakage closure — secret-in-argv stderr advisory RED tests.
//!
//! One cell per inline-secret flag-row whose stdin alternative is added
//! (or, for `convert --bip38-passphrase`, introduced) in this cycle.
//! Each cell invokes the command with the secret inline on argv and
//! asserts the new stderr advisory fires.
//!
//! Stable wording — byte-exact prefix only:
//!     "warning: secret material on argv"
//! The full message names the flag and the recommended stdin
//! alternative; tests assert only the stable prefix so impl can iterate
//! the suffix wording without test churn (cf. SPEC v0.6.1 §5.5.a
//! "secret material on stdout" precedent).
//!
//! Authoritative reference: `design/SPEC_secret_memory_hygiene_v0_9_0.md`
//! §1 item 1; survey §6 cross-cutting observation 4
//! ("Suggest a parallel `secret-in-argv` warning when clap parses an
//! inline secret value while a `=-` / stdin alternative exists").
//!
//! 9 cells matching the 9 newly-closed flag-rows in `cli_argv_leakage.rs`:
//! 1. `bundle --slot @0.phrase=<inline>`
//! 2. `bundle --slot @0.entropy=<inline>`
//! 3. `bundle --slot @0.wif=<inline>`
//! 4. `bundle --slot @0.xprv=<inline>` (warning fires even though
//!    runtime rejects xprv; advisory fires at run() entry, before any
//!    dispatch error)
//! 5. `verify-bundle --slot @0.phrase=<inline>`
//! 6. `bundle --passphrase <inline>`
//! 7. `verify-bundle --passphrase <inline>`
//! 8. `derive-child --passphrase <inline>`
//! 9. `convert --bip38-passphrase <inline>`

use assert_cmd::Command;

const ADVISORY_PREFIX: &str = "warning: secret material on argv";

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const TREZOR_24_ENTROPY_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
const MAINNET_WIF: &str = "KwYgW8gcxj1JWJXhPSu4Fqwzfhp5Yfi42mdYmMa4XqK7NJxXUSK7";
const MAINNET_XPRV: &str = "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi";

const BIP38_PASS: &str = "TestingOneTwoThree";
const BIP38_WIF: &str = "L44B5gGEpqEDRS9vVPz7QT35jcBG2r3CZwSwQ4fCewXAhAhqGVpP";

fn stderr_of(cmd: &mut Command) -> String {
    let out = cmd.assert();
    String::from_utf8(out.get_output().stderr.clone()).unwrap()
}

// ============================================================================
// Cell 1 — `bundle --slot @0.phrase=<inline>` emits the advisory
// ============================================================================

#[test]
fn bundle_slot_phrase_inline_emits_argv_advisory() {
    let stderr = stderr_of(Command::cargo_bin("mnemonic").unwrap().args([
        "bundle",
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--no-engraving-card",
    ]));
    assert!(
        stderr.contains(ADVISORY_PREFIX),
        "inline phrase must emit secret-in-argv advisory; stderr: {stderr:?}"
    );
}

// ============================================================================
// Cell 2 — `bundle --slot @0.entropy=<inline>`
// ============================================================================

#[test]
fn bundle_slot_entropy_inline_emits_argv_advisory() {
    let stderr = stderr_of(Command::cargo_bin("mnemonic").unwrap().args([
        "bundle",
        "--slot",
        &format!("@0.entropy={TREZOR_24_ENTROPY_HEX}"),
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--no-engraving-card",
    ]));
    assert!(
        stderr.contains(ADVISORY_PREFIX),
        "inline entropy must emit secret-in-argv advisory; stderr: {stderr:?}"
    );
}

// ============================================================================
// Cell 3 — `bundle --slot @0.wif=<inline>`
// ============================================================================

#[test]
fn bundle_slot_wif_inline_emits_argv_advisory() {
    let stderr = stderr_of(Command::cargo_bin("mnemonic").unwrap().args([
        "bundle",
        "--slot",
        &format!("@0.wif={MAINNET_WIF}"),
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--no-engraving-card",
    ]));
    assert!(
        stderr.contains(ADVISORY_PREFIX),
        "inline wif must emit secret-in-argv advisory; stderr: {stderr:?}"
    );
}

// ============================================================================
// Cell 4 — `bundle --slot @0.xprv=<inline>`
//
// Runtime rejects the xprv slot per bundle.rs:470-476 (v0.4.2 deferral);
// the advisory must still fire because it runs at parse-success / run()
// entry, BEFORE dispatch reaches the xprv-subkey reject branch.
// ============================================================================

#[test]
fn bundle_slot_xprv_inline_emits_argv_advisory_even_on_runtime_reject() {
    let stderr = stderr_of(Command::cargo_bin("mnemonic").unwrap().args([
        "bundle",
        "--slot",
        &format!("@0.xprv={MAINNET_XPRV}"),
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--no-engraving-card",
    ]));
    assert!(
        stderr.contains(ADVISORY_PREFIX),
        "inline xprv must emit secret-in-argv advisory before runtime reject; stderr: {stderr:?}"
    );
    assert!(
        stderr.contains("not supported in v0.4.2; deferred to v0.5+"),
        "xprv runtime rejection must still fire; stderr: {stderr:?}"
    );
}

// ============================================================================
// Cell 5 — `verify-bundle --slot @0.phrase=<inline>`
// ============================================================================

#[test]
fn verify_bundle_slot_phrase_inline_emits_argv_advisory() {
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

    let stderr = stderr_of(Command::cargo_bin("mnemonic").unwrap().args(&args));
    assert!(
        stderr.contains(ADVISORY_PREFIX),
        "inline phrase on verify-bundle must emit secret-in-argv advisory; stderr: {stderr:?}"
    );
}

// ============================================================================
// Cell 6 — `bundle --passphrase <inline>`
// ============================================================================

#[test]
fn bundle_passphrase_inline_emits_argv_advisory() {
    let stderr = stderr_of(Command::cargo_bin("mnemonic").unwrap().args([
        "bundle",
        "--slot",
        &format!("@0.phrase={TREZOR_24}"),
        "--passphrase",
        "swordfish",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--no-engraving-card",
    ]));
    assert!(
        stderr.contains(ADVISORY_PREFIX) && stderr.contains("--passphrase"),
        "inline --passphrase on bundle must emit advisory naming the flag; stderr: {stderr:?}"
    );
}

// ============================================================================
// Cell 7 — `verify-bundle --passphrase <inline>`
//
// The passphrase that bundle was generated with must match verify-bundle's
// `--passphrase` for the cards to round-trip. The pinned fixture is
// generated with no passphrase, so the wrong passphrase here surfaces a
// `result: mismatch`. We don't care about the verify-bundle outcome here —
// only that the advisory fires.
// ============================================================================

#[test]
fn verify_bundle_passphrase_inline_emits_argv_advisory() {
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
        "--passphrase".into(),
        "swordfish".into(),
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

    let stderr = stderr_of(Command::cargo_bin("mnemonic").unwrap().args(&args));
    assert!(
        stderr.contains(ADVISORY_PREFIX) && stderr.contains("--passphrase"),
        "inline --passphrase on verify-bundle must emit advisory; stderr: {stderr:?}"
    );
}

// ============================================================================
// Cell 8 — `derive-child --passphrase <inline>`
// ============================================================================

#[test]
fn derive_child_passphrase_inline_emits_argv_advisory() {
    let stderr = stderr_of(Command::cargo_bin("mnemonic").unwrap().args([
        "derive-child",
        "--from",
        &format!("phrase={TREZOR_24}"),
        "--application",
        "bip39",
        "--length",
        "12",
        "--index",
        "0",
        "--passphrase",
        "swordfish",
    ]));
    assert!(
        stderr.contains(ADVISORY_PREFIX) && stderr.contains("--passphrase"),
        "inline --passphrase on derive-child must emit advisory; stderr: {stderr:?}"
    );
}

// ============================================================================
// Cell 9 — `convert --bip38-passphrase <inline>`
// ============================================================================

#[test]
fn convert_bip38_passphrase_inline_emits_argv_advisory() {
    let stderr = stderr_of(Command::cargo_bin("mnemonic").unwrap().args([
        "convert",
        "--from",
        &format!("wif={BIP38_WIF}"),
        "--to",
        "bip38",
        "--bip38-passphrase",
        BIP38_PASS,
    ]));
    assert!(
        stderr.contains(ADVISORY_PREFIX) && stderr.contains("--bip38-passphrase"),
        "inline --bip38-passphrase must emit advisory naming the flag; stderr: {stderr:?}"
    );
}
