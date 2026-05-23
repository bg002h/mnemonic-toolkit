//! v0.35.0 — `mnemonic silent-payment` (BIP-352 receiver address) integration tests.
//! The byte-exact crypto oracle is the unit test in `src/silent_payment.rs`
//! (official BIP-352 vectors). These exercise the CLI surface end-to-end.
use assert_cmd::Command;
use predicates::prelude::*;

// Canonical BIP-39 test mnemonic (abandon×11 + about).
const PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
// Its mainnet silent-payment BASE address (regression pin; the encode crypto
// is byte-exact-validated against the official vectors in the lib unit test).
const BASE_SP: &str = "sp1qqfqnnv8czppwysafq3uwgwvsc638hc8rx3hscuddh0xa2yd746s7xqh6yy9ncjnqhqxazct0fzh98w7lpkm5fvlepqec2yy0sxlq4j6ccc3h6t0g";
// The canonical BIP-32 root xprv for the same mnemonic (no passphrase).
const ROOT_XPRV: &str = "xprv9s21ZrQH143K3GJpoapnV8SFfukcVBSfeCficPSGfubmSFDxo1kuHnLisriDvSnRRuL2Qrg5ggqHKNVpxR86QEC8w35uxmGoggxtQTPvfUu";

#[test]
fn mainnet_base_address() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["silent-payment", "--secret", PHRASE])
        .assert().success()
        .stdout(predicate::str::contains(BASE_SP));
}

#[test]
fn testnet_uses_tsp_hrp() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["silent-payment", "--secret", PHRASE, "--network", "testnet"])
        .assert().success()
        .stdout(predicate::str::contains("  address:      tsp1q"));
}

#[test]
fn json_shape_and_labeled_differs() {
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["silent-payment", "--secret", PHRASE, "--label", "1", "--json"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["address"], BASE_SP);
    assert_eq!(v["scan_path"], "m/352'/0'/0'/1'/0");
    assert_eq!(v["spend_path"], "m/352'/0'/0'/0'/0");
    assert_eq!(v["labeled"][0]["m"], 1);
    assert_ne!(v["labeled"][0]["address"], v["address"], "labeled must differ from base");
    assert_eq!(v["scan_priv"].as_str().unwrap().len(), 64);
    assert_eq!(v["spend_priv"].as_str().unwrap().len(), 64);
    assert_eq!(v["scan_pubkey"].as_str().unwrap().len(), 66);
}

#[test]
fn label_zero_refused() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["silent-payment", "--secret", PHRASE, "--label", "0"])
        .assert().failure().code(1)
        .stderr(predicate::str::contains("reserved BIP-352 change label"));
}

#[test]
fn wif_input_refused() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["silent-payment", "--secret", "5Kb8kLf9zgWQnogidDA76MzPL6TsZZY36hWXMssSzNydYXYB9KF"])
        .assert().failure().code(1)
        .stderr(predicate::str::contains("seed-bearing"));
}

#[test]
fn xprv_input_matches_phrase() {
    // Same master via the canonical root xprv → same base address as the phrase.
    Command::cargo_bin("mnemonic").unwrap()
        .args(["silent-payment", "--secret", ROOT_XPRV])
        .assert().success()
        .stdout(predicate::str::contains(BASE_SP));
}

#[test]
fn secret_stdin_works_and_warns_stdout_not_argv() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["silent-payment", "--secret-stdin"])
        .write_stdin(format!("{PHRASE}\n"))
        .assert().success()
        .stdout(predicate::str::contains(BASE_SP))
        .stderr(predicate::str::contains("secret material on stdout"))
        .stderr(predicate::str::contains("secret material on argv").not());
}

#[test]
fn inline_secret_warns_argv() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["silent-payment", "--secret", PHRASE])
        .assert().success()
        .stderr(predicate::str::contains("secret material on argv (--secret"));
}
