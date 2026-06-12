//! v0.6 `mnemonic convert` happy-path edges.

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const TREZOR_24_ZERO_ENTROPY_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
const TREZOR_BIP84_MAINNET_XPUB: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";
const TREZOR_12_ZERO_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const TREZOR_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn phrase_to_entropy_24word() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_24}"),
            "--to",
            "entropy",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("entropy: {TREZOR_24_ZERO_ENTROPY_HEX}\n"));
}

#[test]
fn entropy_to_phrase_12word() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            "entropy=00000000000000000000000000000000",
            "--to",
            "phrase",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("phrase: {TREZOR_12}\n"));
}

#[test]
fn phrase_to_xpub_bip84_mainnet() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_24}"),
            "--to",
            "xpub",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("xpub: {TREZOR_BIP84_MAINNET_XPUB}\n"));
}

#[test]
fn phrase_to_xpub_xprv_compound() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_24}"),
            "--to",
            "xpub,xprv",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("xpub: xpub6"));
    assert!(lines[1].starts_with("xprv: xprv9"));
}

#[test]
fn xprv_to_xpub_neuter() {
    // Derive xprv first, then neuter to xpub.
    let xprv_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_24}"),
            "--to",
            "xprv",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .success();
    let xprv_line = String::from_utf8(xprv_out.get_output().stdout.clone()).unwrap();
    let xprv = xprv_line.trim().trim_start_matches("xprv: ");

    let xpub_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["convert", "--from", &format!("xprv={xprv}"), "--to", "xpub"])
        .assert()
        .success();
    let stdout = String::from_utf8(xpub_out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("xpub: {TREZOR_BIP84_MAINNET_XPUB}\n"));
}

#[test]
fn xpub_to_fingerprint() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_BIP84_MAINNET_XPUB}"),
            "--to",
            "fingerprint",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.starts_with("fingerprint: "));
    assert_eq!(stdout.trim().len(), "fingerprint: ".len() + 8);
}

#[test]
fn entropy_to_ms1_16byte() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            "entropy=00000000000000000000000000000000",
            "--to",
            "ms1",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("ms1: {TREZOR_12_ZERO_MS1}\n"));
}

#[test]
fn ms1_to_entropy_round_trip() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("ms1={TREZOR_12_ZERO_MS1}"),
            "--to",
            "entropy",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, "entropy: 00000000000000000000000000000000\n");
}

#[test]
fn wif_to_xpub_sentinel() {
    // Bitcoin Core canonical compressed-pubkey WIF.
    let wif = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("wif={wif}"),
            "--to",
            "xpub",
            "--network",
            "mainnet",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // Depth-0 sentinel xpub starts with xpub661 (depth=0, child_number=0).
    assert!(stdout.starts_with("xpub: xpub661"));
}

#[test]
fn mk1_to_xpub_decode() {
    // Two-string mk1 fixture from bip84-mainnet vector. Whitespace-separated
    // tokens per SPEC §5.a (mk1 reads multi-string codec output).
    let mk1 = "mk1qpnd2wpqqsqek48ppe2rd4eyqvzg3vs7zfl2pe5jyqghcnaqxqq4gdatr9tn90ga6tg0purlfh9275f4pvjmck3usgpec7pzw3wvgsn9mwmd mk1qpnd2wppha4qc2sv8g58zqcpswt0zfsza3lk237tx7xeg8evycaywffzk5r3hcma55t0u0d83tguz";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("mk1={mk1}"),
            "--to",
            "xpub,fingerprint,path",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("xpub: xpub6"));
    assert!(lines[1].starts_with("fingerprint: "));
    assert_eq!(lines[1].len(), "fingerprint: ".len() + 8);
    assert!(lines[2].starts_with("path: 84'/0'/0'") || lines[2].starts_with("path: m/84'/0'/0'"));
}

#[test]
fn ms1_to_phrase_direct_edge() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("ms1={TREZOR_12_ZERO_MS1}"),
            "--to",
            "phrase",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("phrase: {TREZOR_12}\n"));
}

// SPEC-A v0.6.1 — phrase/entropy → wif at explicit leaf path.

/// Canonical BIP-84 vector: 12-word zero-entropy "abandon...about" mnemonic +
/// `m/84'/0'/0'/0/0` produces the well-known WIF
/// `KyZpNDKnfs94vbrwhJneDi77V6jF64PWPF8x5cdJb8ifgg2DUc9d` (the BIP-84-test-vector
/// first-receive privkey for the all-zeros 128-bit entropy seed).
#[test]
fn phrase_to_wif_bip84_leaf_mainnet() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--to",
            "wif",
            "--network",
            "mainnet",
            "--path",
            "m/84'/0'/0'/0/0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        "wif: KyZpNDKnfs94vbrwhJneDi77V6jF64PWPF8x5cdJb8ifgg2DUc9d\n"
    );
}

/// 16-byte all-zero entropy + the same path; matches the phrase-source vector
/// (the entropy IS the all-zero 128-bit entropy that decodes to TREZOR_12).
#[test]
fn entropy_to_wif_bip84_leaf_mainnet() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            "entropy=00000000000000000000000000000000",
            "--to",
            "wif",
            "--network",
            "mainnet",
            "--path",
            "m/84'/0'/0'/0/0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        "wif: KyZpNDKnfs94vbrwhJneDi77V6jF64PWPF8x5cdJb8ifgg2DUc9d\n"
    );
}

// ============================================================================
// Phase E — convert-test-coverage-tightening (FOLLOWUP closure)
// 6 missing direct edges per the v0.6.0 post-release coverage audit.
// ============================================================================

const TREZOR_24_ZERO_ENTROPY_HEX_64: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
const TREZOR_24_BIP84_MAINNET_XPRV: &str = "xprv9xoJSXoA4FrCpc2ufjps9Bwd7MHDCSLTbQxzdtDtxPv1Tx7KhF8riMNbQx1PqgUaTf2VjXLVBbw1WqZbeTRdmF4Di8o3Xz7t9LRizh5WxEP";
const TREZOR_24_MASTER_FINGERPRINT: &str = "5436d724";
const TREZOR_24_ZERO_MS1_24WORD: &str =
    "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w";
/// hash160 of the WIF sentinel-xpub pubkey (depth=0, chain_code=0, the
/// `bundle.rs::resolve_slots` Wif sentinel pattern shared by `convert`).
const SAMPLE_WIF: &str = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";
const SAMPLE_WIF_SENTINEL_FINGERPRINT: &str = "751e76e8";

#[test]
fn phrase_to_ms1_composite_via_entropy() {
    // Composite edge phrase → entropy → ms1; v0.6.0 covered entropy → ms1
    // directly but had no phrase-source assertion.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_24}"),
            "--to",
            "ms1",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("ms1: {TREZOR_24_ZERO_MS1_24WORD}\n"));
}

#[test]
fn entropy_to_xpub_bip84_mainnet() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("entropy={TREZOR_24_ZERO_ENTROPY_HEX_64}"),
            "--to",
            "xpub",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("xpub: {TREZOR_BIP84_MAINNET_XPUB}\n"));
}

#[test]
fn entropy_to_xprv_bip84_mainnet() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("entropy={TREZOR_24_ZERO_ENTROPY_HEX_64}"),
            "--to",
            "xprv",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("xprv: {TREZOR_24_BIP84_MAINNET_XPRV}\n"));
}

#[test]
fn entropy_to_fingerprint_bip84_mainnet() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("entropy={TREZOR_24_ZERO_ENTROPY_HEX_64}"),
            "--to",
            "fingerprint",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("fingerprint: {TREZOR_24_MASTER_FINGERPRINT}\n")
    );
}

#[test]
fn xprv_to_fingerprint_account_xpriv() {
    // Fingerprint of the account-level xprv's xpub (NOT the master fingerprint).
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xprv={TREZOR_24_BIP84_MAINNET_XPRV}"),
            "--to",
            "fingerprint",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // Account-xpub fingerprint, computed by hashing the account-xpub's pubkey.
    // Same value emitted by `--from xpub --to fingerprint` against the
    // matching xpub fixture.
    assert!(stdout.starts_with("fingerprint: "));
    assert_eq!(stdout.trim().len(), "fingerprint: ".len() + 8);
    let fp = stdout.trim().trim_start_matches("fingerprint: ");
    // Cross-check against the xpub-source emission for byte-for-byte equality.
    let xpub_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_BIP84_MAINNET_XPUB}"),
            "--to",
            "fingerprint",
        ])
        .assert()
        .success();
    let xpub_fp = String::from_utf8(xpub_out.get_output().stdout.clone())
        .unwrap()
        .trim()
        .trim_start_matches("fingerprint: ")
        .to_string();
    assert_eq!(
        fp, xpub_fp,
        "xprv→fingerprint must equal xpub→fingerprint for the same account-level key"
    );
}

#[test]
fn wif_to_fingerprint_co_tested_with_wif_to_xpub() {
    // Co-tested with wif → xpub (sentinel) per architect r1 L-3: same setup,
    // same WIF, two derived outputs verified together via compound --to.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("wif={SAMPLE_WIF}"),
            "--to",
            "xpub,fingerprint",
            "--network",
            "mainnet",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("xpub: xpub661"));
    assert_eq!(
        lines[1],
        format!("fingerprint: {SAMPLE_WIF_SENTINEL_FINGERPRINT}")
    );
}

/// SPEC-A §8 invariant — `--passphrase` is meaningful for phrase/entropy → wif
/// (the edge traverses PBKDF2: phrase → seed → master → derive → leaf privkey).
/// The "passphrase ignored" warning MUST NOT fire on this edge.
#[test]
fn phrase_to_wif_passphrase_does_not_emit_ignored_warning() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--to",
            "wif",
            "--network",
            "mainnet",
            "--path",
            "m/84'/0'/0'/0/0",
            "--passphrase",
            "TREZOR",
        ])
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("--passphrase ignored"),
        "ignored-passphrase warning must not fire on PBKDF2-bearing wif edge; got stderr: {stderr:?}"
    );
}
