//! FOLLOWUP `multisig-tr-bip48-script-type-3-policy` (bless + warn).
//!
//! Taproot multisig under `--multisig-path-family bip48` derives at the
//! non-standard `m/48'/<coin>'/<account>'/3'` (BIP-48 standardizes only 1' and
//! 2'). The toolkit HONORS the explicit flag (exit 0, cards emitted) but emits
//! a stderr advisory at creation time (`bundle`, `export-wallet`). Standardized
//! combos (bip87 family, wsh/sh-wsh templates) stay silent.

use assert_cmd::Command;
use predicates::prelude::*;

const T24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const P1: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";

const ADVISORY: &str = "BIP-48 standardizes only script-type 1'";

#[test]
fn bundle_tr_sortedmulti_bip48_emits_advisory_and_succeeds() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "tr-sortedmulti-a",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--slot",
            &format!("@0.phrase={T24}"),
            "--slot",
            &format!("@1.phrase={P1}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success() // blessed: honored, exit 0
        .stderr(predicate::str::contains(ADVISORY));
}

#[test]
fn bundle_tr_sortedmulti_bip87_is_silent() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "tr-sortedmulti-a",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip87",
            "--slot",
            &format!("@0.phrase={T24}"),
            "--slot",
            &format!("@1.phrase={P1}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(ADVISORY).not());
}

#[test]
fn bundle_wsh_sortedmulti_bip48_is_silent() {
    // Standardized bip48 script-type 2' (wsh) — no advisory.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--slot",
            &format!("@0.phrase={T24}"),
            "--slot",
            &format!("@1.phrase={P1}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(ADVISORY).not());
}

#[test]
fn verify_bundle_tr_sortedmulti_bip48_emits_advisory() {
    // Build a tr+bip48 bundle, then verify it under the same template/family;
    // verify-bundle re-derives at m/48'/.../3' and must surface the advisory.
    let bundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "tr-sortedmulti-a",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--slot",
            &format!("@0.phrase={T24}"),
            "--slot",
            &format!("@1.phrase={P1}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let bundle: serde_json::Value =
        serde_json::from_slice(&bundle_out.get_output().stdout).unwrap();

    let mut mk1_flat: Vec<String> = Vec::new();
    for inner in bundle["mk1"].as_array().unwrap() {
        for chunk in inner.as_array().unwrap() {
            mk1_flat.push(chunk.as_str().unwrap().to_string());
        }
    }
    let arr = |k: &str| -> Vec<String> {
        bundle[k]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    };

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "tr-sortedmulti-a".into(),
        "--threshold".into(),
        "2".into(),
        "--multisig-path-family".into(),
        "bip48".into(),
        "--slot".into(),
        format!("@0.phrase={T24}"),
        "--slot".into(),
        format!("@1.phrase={P1}"),
    ];
    for m in &mk1_flat {
        args.push("--mk1".into());
        args.push(m.clone());
    }
    for m in &arr("ms1") {
        args.push("--ms1".into());
        args.push(m.clone());
    }
    for m in &arr("md1") {
        args.push("--md1".into());
        args.push(m.clone());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&refs)
        .assert()
        .stderr(predicate::str::contains(ADVISORY));
}

#[test]
fn export_wallet_tr_sortedmulti_bip48_emits_advisory() {
    // Watch-only export: derive cosigner xpubs in-test so the start is consistent.
    use bip39::Mnemonic;
    use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
    use bitcoin::secp256k1::Secp256k1;
    use std::str::FromStr;

    let secp = Secp256k1::new();
    let mut slot_args: Vec<String> = Vec::new();
    for (i, p) in [T24, P1].iter().enumerate() {
        let m = Mnemonic::parse_in(bip39::Language::English, *p).unwrap();
        let seed = m.to_seed("");
        let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
        let fp = master.fingerprint(&secp).to_string().to_lowercase();
        let path = DerivationPath::from_str("m/48'/0'/0'/3'").unwrap();
        let xpub = Xpub::from_priv(&secp, &master.derive_priv(&secp, &path).unwrap());
        slot_args.push("--slot".into());
        slot_args.push(format!("@{i}.xpub={xpub}"));
        slot_args.push("--slot".into());
        slot_args.push(format!("@{i}.fingerprint={fp}"));
        slot_args.push("--slot".into());
        slot_args.push("@{i}.path=m/48'/0'/0'/3'".replace("{i}", &i.to_string()));
    }
    let mut args: Vec<String> = [
        "export-wallet",
        "--format",
        "bip388",
        "--network",
        "mainnet",
        "--template",
        "tr-sortedmulti-a",
        "--taproot-internal-key",
        "nums",
        "--threshold",
        "2",
        "--multisig-path-family",
        "bip48",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    args.extend(slot_args);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&refs)
        .assert()
        .success()
        .stderr(predicate::str::contains(ADVISORY));
}
