//! JSON envelope schema integration tests for both subcommands.

use assert_cmd::Command;
use serde_json::Value;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

#[test]
fn bundle_json_schema_field_order() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--phrase",
            TREZOR_24,
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("bundle stdout is valid JSON");
    assert_eq!(v["schema_version"], "4");
    assert_eq!(v["mode"], "full");
    assert_eq!(v["network"], "mainnet");
    assert_eq!(v["template"], "bip84");
    assert_eq!(v["account"], 0);
    assert_eq!(v["origin_path"], "m/84'/0'/0'");
    assert_eq!(v["master_fingerprint"], "5436d724");
    // SPEC §5.8 schema-4: ms1 is length-N MsField; single-sig full = ["ms1..."].
    assert!(v["ms1"].is_array(), "ms1 is MsField (Vec<String>)");
    assert!(v["ms1"][0].as_str().unwrap().starts_with("ms1"));
    assert!(v["mk1"].is_array() && !v["mk1"].as_array().unwrap().is_empty());
    assert!(v["md1"].is_array() && !v["md1"].as_array().unwrap().is_empty());
}

#[test]
fn verify_bundle_json_emits_9_checks_in_spec_order() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let ms1 = fixture
        .lines()
        .find(|l| l.starts_with("ms1") && !l.contains(' '))
        .unwrap()
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
        "--phrase".into(),
        TREZOR_24.into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
        "--ms1".into(),
        ms1,
        "--json".into(),
    ];
    for s in &mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in &md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("verify-bundle stdout is valid JSON");
    assert_eq!(v["schema_version"], "4");
    assert_eq!(v["result"], "ok");
    let checks = v["checks"].as_array().expect("checks is an array");
    assert_eq!(checks.len(), 9, "9 checks emitted (SPEC §2.2)");
    let names: Vec<&str> = checks.iter().map(|c| c["name"].as_str().unwrap()).collect();
    assert_eq!(
        names,
        vec![
            "ms1_entropy_match",
            "mk1_decode",
            "mk1_xpub_match",
            "mk1_fingerprint_match",
            "mk1_path_match",
            "md1_decode",
            "md1_wallet_policy",
            "md1_xpub_match",
            "stub_linkage",
        ]
    );
}
