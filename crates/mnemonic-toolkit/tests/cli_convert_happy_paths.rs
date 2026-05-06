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
    assert_eq!(
        stdout,
        format!("entropy: {TREZOR_24_ZERO_ENTROPY_HEX}\n")
    );
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
