//! v0.6 `mnemonic convert` --json envelope shape (SPEC §6).

use assert_cmd::Command;
use serde_json::Value;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const TREZOR_BIP84_MAINNET_XPUB: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";

#[test]
fn json_secret_from_node_omits_from_value() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_24}"),
            "--to",
            "entropy",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["from_node"], "phrase");
    assert!(
        v.get("from_value").is_none() || v["from_value"].is_null(),
        "secret-bearing from_node must omit from_value (privacy hygiene per §6.a)"
    );
    let to = v["to"].as_array().unwrap();
    assert_eq!(to.len(), 1);
    assert_eq!(to[0]["node"], "entropy");
    assert_eq!(
        to[0]["value"],
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
}

#[test]
fn json_public_from_node_includes_from_value() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_BIP84_MAINNET_XPUB}"),
            "--to",
            "fingerprint",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["from_node"], "xpub");
    assert_eq!(v["from_value"], TREZOR_BIP84_MAINNET_XPUB);
}

#[test]
fn json_compound_to_preserves_argument_order() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_24}"),
            "--to",
            "fingerprint,xpub,entropy",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("valid JSON");
    let to = v["to"].as_array().unwrap();
    assert_eq!(to.len(), 3);
    assert_eq!(to[0]["node"], "fingerprint");
    assert_eq!(to[1]["node"], "xpub");
    assert_eq!(to[2]["node"], "entropy");
}
