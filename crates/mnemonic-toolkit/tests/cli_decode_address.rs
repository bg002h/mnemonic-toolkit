//! v0.36.0 — `mnemonic decode-address` integration tests. The byte-exact
//! decoding oracle is the unit suite in `src/decode_address.rs` (BIP-173/350
//! canonical vectors). These exercise the CLI surface end-to-end.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn human_output_p2wpkh() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["decode-address", "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"])
        .assert()
        .success()
        .stdout(predicate::str::contains("script_type:   p2wpkh"))
        .stdout(predicate::str::contains("witness_ver:   0"))
        .stdout(predicate::str::contains(
            "script_pubkey: 0014751e76e8199196d454941c45d1b3a323f1433bd6",
        ));
}

#[test]
fn human_output_legacy_p2pkh_has_no_witness_version() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["decode-address", "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("script_type:   p2pkh"))
        .stdout(predicate::str::contains("witness_ver:   (none; legacy)"));
}

#[test]
fn json_shape() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "decode-address",
            "bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqzk5jj0",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["valid"], true);
    assert_eq!(v["script_type"], "p2tr");
    assert_eq!(v["witness_version"], 1);
    assert!(v["script_pubkey"].as_str().unwrap().starts_with("5120"));
    assert!(v["networks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n == "mainnet"));
}

#[test]
fn testnet_address_reports_network_set() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "decode-address",
            "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let nets: Vec<&str> = v["networks"].as_array().unwrap().iter().map(|n| n.as_str().unwrap()).collect();
    assert!(nets.contains(&"testnet") && nets.contains(&"signet") && nets.contains(&"testnet4"));
    assert!(!nets.contains(&"mainnet"));
}

#[test]
fn invalid_address_nonzero_exit() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["decode-address", "not-a-real-address"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("decode-address"));
}
