//! v0.3 descriptor-mode end-to-end integration tests.
//!
//! Exercises bundle + verify-bundle for the four mode pairs:
//! full single-sig, watch-only single-sig, full multisig, watch-only multisig.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

const TREZOR_FP_HEX: &str = "5436d724";

#[test]
fn descriptor_full_singlesig_bip84_emits_valid_bundle() {
    let descriptor = format!("wpkh(@0[{TREZOR_FP_HEX}/84'/0'/0']/<0;1>/*)");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            &descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["schema_version"], "4");
    assert_eq!(v["mode"], "full");
    assert_eq!(v["template"], Value::Null);
    assert_eq!(v["descriptor"].as_str().unwrap(), descriptor);
    // SPEC §5.8 schema-4: ms1 is MsField (length-N array); single-sig full = ["ms1..."].
    assert!(v["ms1"].is_array());
    assert!(v["ms1"][0].as_str().unwrap().starts_with("ms1"));
    assert!(v["mk1"].is_array() && !v["mk1"].as_array().unwrap().is_empty());
    assert!(v["md1"].is_array() && !v["md1"].as_array().unwrap().is_empty());
    assert_eq!(v["multisig"], Value::Null);
}

#[test]
fn descriptor_watch_only_singlesig_emits_no_ms1() {
    // Use the bip84 xpub for the trezor-24 seed (well-known test vector).
    let xpub = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";
    let descriptor = "wpkh(@0/<0;1>/*)";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={xpub}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_FP_HEX}"),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["mode"], "watch-only");
    // SPEC §5.8 schema-4: single-sig watch-only ms1 = [""] (length-1 with sentinel).
    assert_eq!(v["ms1"], serde_json::json!([""]));
    assert_eq!(v["descriptor"].as_str().unwrap(), descriptor);
}

#[test]
fn descriptor_bundle_round_trips_through_verify_bundle() {
    let descriptor = format!("wpkh(@0[{TREZOR_FP_HEX}/84'/0'/0']/<0;1>/*)");

    // Step 1: emit bundle.
    let bundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            &descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--json",
        ])
        .assert()
        .success();
    let bundle_stdout = String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap();
    let bundle: Value = serde_json::from_str(&bundle_stdout).expect("valid JSON");
    let ms1 = bundle["ms1"][0].as_str().unwrap();
    let mk1: Vec<&str> = bundle["mk1"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let md1: Vec<&str> = bundle["md1"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    // Step 2: verify-bundle.
    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--descriptor".into(),
        descriptor.clone(),
        "--network".into(),
        "mainnet".into(),
        "--slot".into(),
        format!("@0.phrase={TREZOR_24}"),
        "--ms1".into(),
        ms1.into(),
        "--json".into(),
    ];
    for s in &mk1 {
        args.push("--mk1".into());
        args.push((*s).into());
    }
    for s in &md1 {
        args.push("--md1".into());
        args.push((*s).into());
    }
    let verify_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success();
    let verify_stdout = String::from_utf8(verify_out.get_output().stdout.clone()).unwrap();
    let verify: Value = serde_json::from_str(&verify_stdout).expect("valid JSON");
    assert_eq!(verify["schema_version"], "4");
    assert_eq!(verify["result"], "ok");
    let checks = verify["checks"].as_array().unwrap();
    assert!(checks.iter().all(|c| c["passed"] == true));
}

#[test]
fn descriptor_verify_bundle_detects_tampered_mk1() {
    let descriptor = format!("wpkh(@0[{TREZOR_FP_HEX}/84'/0'/0']/<0;1>/*)");

    // Emit, then mangle mk1 before verify.
    let bundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            &descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--json",
        ])
        .assert()
        .success();
    let bundle: Value =
        serde_json::from_str(&String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap())
            .unwrap();
    let ms1 = bundle["ms1"][0].as_str().unwrap();
    // Use a clearly-different mk1 (truncated) that decodes-but-mismatches.
    let bad_mk1 = "mk1qpnd2wpqqsqek48ppe2rd4eyqvzg3vs7zfl2pe5jyqghcnaqxqq4gdatr9tn90ga6tg0purlfh9275f4pvjmck3usgpec7pzw3wvgsn9mwm0";
    let md1: Vec<&str> = bundle["md1"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--descriptor".into(),
        descriptor,
        "--network".into(),
        "mainnet".into(),
        "--slot".into(),
        format!("@0.phrase={TREZOR_24}"),
        "--ms1".into(),
        ms1.into(),
        "--mk1".into(),
        bad_mk1.into(),
        "--json".into(),
    ];
    for s in &md1 {
        args.push("--md1".into());
        args.push((*s).into());
    }
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .failure()
        .code(4)
        .stdout(predicate::str::contains(r#""result":"mismatch""#));
}

/// SPEC v0.6.1 §11 cross-cut at bundle.rs::bundle_run_unified_descriptor —
/// `bundle --descriptor "wpkh(@0/<0;1>/*)" --slot @0.xpub=<zpub>` (descriptor
/// mode) must produce a byte-identical bundle to the equivalent xpub
/// invocation. Proves the SLIP-0132 input normalizer is wired in the
/// descriptor-mode branch (bundle.rs:853), not just the template-mode branch.
#[test]
fn descriptor_watch_only_singlesig_accepts_zpub_input_via_slip0132_normalizer() {
    let xpub = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";
    // Canonical zpub form per BIP-84 reference vector
    // https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki
    let zpub = "zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4wAcvPhXNfE3EfH1r1ADqtfSdVCToUG868RvUUkgDKf31mGDtKsAYz2oz2AGutZYs";
    let descriptor = "wpkh(@0/<0;1>/*)";

    let from_xpub = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={xpub}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_FP_HEX}"),
            "--json",
        ])
        .assert()
        .success();
    let from_zpub = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={zpub}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_FP_HEX}"),
            "--json",
        ])
        .assert()
        .success();
    assert_eq!(
        from_xpub.get_output().stdout,
        from_zpub.get_output().stdout,
        "descriptor-mode bundle stdout must be byte-identical regardless of xpub vs. zpub input encoding"
    );
}
