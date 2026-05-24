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

// ── v0.36.1 Phase 1: --passphrase / --passphrase-stdin ──────────────────────

fn sp_json(args: &[&str]) -> serde_json::Value {
    let out = Command::cargo_bin("mnemonic").unwrap().args(args)
        .assert().success().get_output().stdout.clone();
    serde_json::from_slice(&out).unwrap()
}

#[test]
fn passphrase_changes_derived_address() {
    let no_pass = sp_json(&["silent-payment", "--secret", PHRASE, "--json"]);
    let with_pass = sp_json(&["silent-payment", "--secret", PHRASE, "--passphrase", "TREZOR", "--json"]);
    assert_eq!(no_pass["address"], BASE_SP);
    assert_ne!(with_pass["address"], no_pass["address"], "passphrase must change the SP address");
    assert!(with_pass["address"].as_str().unwrap().starts_with("sp1q"));
}

#[test]
fn passphrase_stdin_matches_inline() {
    let inline = sp_json(&["silent-payment", "--secret", PHRASE, "--passphrase", "hunter2", "--json"]);
    let via_stdin = Command::cargo_bin("mnemonic").unwrap()
        .args(["silent-payment", "--secret", PHRASE, "--passphrase-stdin", "--json"])
        .write_stdin("hunter2") // no trailing newline → exact
        .assert().success().get_output().stdout.clone();
    let via_stdin: serde_json::Value = serde_json::from_slice(&via_stdin).unwrap();
    assert_eq!(inline["address"], via_stdin["address"]);
}

#[test]
fn passphrase_stdin_and_secret_stdin_refused() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["silent-payment", "--secret-stdin", "--passphrase-stdin", "--json"])
        .write_stdin(PHRASE)
        .assert().failure()
        .stderr(predicate::str::contains("single stdin per invocation"));
}

#[test]
fn passphrase_conflicts_with_passphrase_stdin() {
    // clap-level conflict (ArgGroup-independent pairwise conflict).
    Command::cargo_bin("mnemonic").unwrap()
        .args(["silent-payment", "--secret", PHRASE, "--passphrase", "x", "--passphrase-stdin"])
        .assert().failure();
}

#[test]
fn xprv_input_with_passphrase_warns_and_ignores() {
    // xprv is the master → passphrase has no effect; address == xprv-no-passphrase.
    let plain = sp_json(&["silent-payment", "--secret", ROOT_XPRV, "--json"]);
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["silent-payment", "--secret", ROOT_XPRV, "--passphrase", "ignored", "--json"])
        .assert().success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("--passphrase ignored"), "expected ignore-warning; got: {stderr}");
    let with_pass: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(plain["address"], with_pass["address"], "xprv address must be passphrase-independent");
}

// ── v0.36.1 Phase 2: --change-address (BIP-352 m=0) ─────────────────────────

#[test]
fn change_address_emits_tagged_m0_distinct_from_base() {
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["silent-payment", "--secret", PHRASE, "--change-address"])
        .assert().success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains(BASE_SP), "base address still emitted (additive)");
    assert!(s.to_lowercase().contains("change"), "change line present + tagged");
    assert!(s.contains("never hand out"), "footgun guard present");
    // the change_addr line carries a sp1q… distinct from the base
    let change_line = s.lines().find(|l| l.contains("change_addr")).unwrap();
    assert!(change_line.contains("sp1q") && !change_line.contains(BASE_SP));
}

#[test]
fn change_address_json_has_value_and_warning() {
    let v = sp_json(&["silent-payment", "--secret", PHRASE, "--change-address", "--json"]);
    let ch = v["change_address"].as_str().unwrap();
    assert!(ch.starts_with("sp1q"));
    assert_ne!(v["change_address"], v["address"], "m=0 change != base");
    assert!(v["change_address_warning"].as_str().unwrap().to_lowercase().contains("never publish"));
}

#[test]
fn change_address_absent_by_default() {
    let v = sp_json(&["silent-payment", "--secret", PHRASE, "--json"]);
    assert!(v.get("change_address").is_none());
    assert!(v.get("change_address_warning").is_none());
}

#[test]
fn change_address_composes_with_passphrase() {
    // change addr inherits the passphrase wallet (differs from no-passphrase change).
    let no_pass = sp_json(&["silent-payment", "--secret", PHRASE, "--change-address", "--json"]);
    let with_pass = sp_json(&["silent-payment", "--secret", PHRASE, "--passphrase", "TREZOR", "--change-address", "--json"]);
    assert_ne!(no_pass["change_address"], with_pass["change_address"]);
}

#[test]
fn label_zero_still_refused_alongside_change_address_flag() {
    // --label 0 stays refused; --change-address is the only m=0 route.
    Command::cargo_bin("mnemonic").unwrap()
        .args(["silent-payment", "--secret", PHRASE, "--label", "0"])
        .assert().failure()
        .stderr(predicate::str::contains("reserved BIP-352 change label"));
}
