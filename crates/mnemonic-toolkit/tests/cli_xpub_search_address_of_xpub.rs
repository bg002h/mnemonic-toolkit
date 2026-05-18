//! `mnemonic xpub-search address-of-xpub` integration tests (v0.26.0 P3).
//!
//! Per `design/PLAN_v0_26_0_xpub_search.md` §10 C3 (commit-numbered phase P3).
//! Realizes plan §5 (P3 SPEC) — given an xpub (or mk1 card), scan child
//! addresses for chain ∈ {0, 1} × index ∈ [0, gap_limit) and report which
//! target addresses match, at which (chain, index).
//!
//! P3 has no seed material; auto-fire BCH repair does NOT apply.
//!
//! Test design (TDD):
//! - ~16 integration cells via `assert_cmd::Command`.
//!
//! Fixtures: BIP-39 test vector `abandon × 11 about` is the universal master.
//! All expected addresses are computed at runtime via `bitcoin` primitives
//! (no hardcoded ground-truth address constants we couldn't reproduce).

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{base58, Address, NetworkKind};
use std::str::FromStr;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Compute the account-level xpub at `account_path` for the test phrase.
fn account_xpub_at(account_path: &str) -> Xpub {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, PHRASE).unwrap();
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    let dp = DerivationPath::from_str(account_path).unwrap();
    let xpriv = master.derive_priv(&secp, &dp).unwrap();
    Xpub::from_priv(&secp, &xpriv)
}

/// Compute the testnet account-level xpub at `account_path`.
fn account_xpub_at_testnet(account_path: &str) -> Xpub {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, PHRASE).unwrap();
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Test, &seed).unwrap();
    let dp = DerivationPath::from_str(account_path).unwrap();
    let xpriv = master.derive_priv(&secp, &dp).unwrap();
    Xpub::from_priv(&secp, &xpriv)
}

/// Derive a child xpub at `chain/i` from a parent.
fn child_at(parent: &Xpub, chain: u32, i: u32) -> Xpub {
    let secp = Secp256k1::new();
    let dp = DerivationPath::from_str(&format!("m/{chain}/{i}")).unwrap();
    parent.derive_pub(&secp, &dp).unwrap()
}

/// Render a P2WPKH address.
fn p2wpkh_addr(child: &Xpub, kind: NetworkKind) -> String {
    let hrp = match kind {
        NetworkKind::Main => bitcoin::KnownHrp::Mainnet,
        NetworkKind::Test => bitcoin::KnownHrp::Testnets,
    };
    Address::p2wpkh(&child.to_pub(), hrp).to_string()
}

/// Render a P2SH-P2WPKH address.
fn p2shwpkh_addr(child: &Xpub, kind: NetworkKind) -> String {
    Address::p2shwpkh(&child.to_pub(), kind).to_string()
}

/// Render a P2PKH address.
fn p2pkh_addr(child: &Xpub, kind: NetworkKind) -> String {
    Address::p2pkh(child.to_pub(), kind).to_string()
}

/// Render a P2TR address.
fn p2tr_addr(child: &Xpub, kind: NetworkKind) -> String {
    let secp = Secp256k1::new();
    let hrp = match kind {
        NetworkKind::Main => bitcoin::KnownHrp::Mainnet,
        NetworkKind::Test => bitcoin::KnownHrp::Testnets,
    };
    Address::p2tr(&secp, child.to_x_only_pub(), None, hrp).to_string()
}

/// Re-encode an xpub as SLIP-0132 `zpub` (mainnet BIP-84 single-sig).
fn xpub_as_zpub(xp: &Xpub) -> String {
    let raw = xp.encode();
    let mut swapped = raw.to_vec();
    swapped[0..4].copy_from_slice(&[0x04, 0xB2, 0x47, 0x46]);
    base58::encode_check(&swapped)
}

/// Re-encode an xpub as SLIP-0132 `ypub` (mainnet BIP-49 single-sig).
fn xpub_as_ypub(xp: &Xpub) -> String {
    let raw = xp.encode();
    let mut swapped = raw.to_vec();
    swapped[0..4].copy_from_slice(&[0x04, 0x9D, 0x7C, 0xB2]);
    base58::encode_check(&swapped)
}

/// Re-encode an xpub as SLIP-0132 `Zpub` (mainnet BIP-84 multisig).
fn xpub_as_multisig_zpub(xp: &Xpub) -> String {
    let raw = xp.encode();
    let mut swapped = raw.to_vec();
    swapped[0..4].copy_from_slice(&[0x02, 0xAA, 0x7E, 0xD3]);
    base58::encode_check(&swapped)
}

// ---------------------------------------------------------------------------
// Happy paths
// ---------------------------------------------------------------------------

#[test]
fn zpub_p2wpkh_match_external_0_5() {
    // BIP-84 mainnet account xpub (Trezor 12-word), zpub-prefixed. The
    // address at m/0/5 should match; chain="external", index=5.
    let acct = account_xpub_at("m/84'/0'/0'");
    let zpub = xpub_as_zpub(&acct);
    let target = p2wpkh_addr(&child_at(&acct, 0, 5), NetworkKind::Main);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &zpub,
            "--target-address",
            &target,
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("match"),
        "expected match in stdout: {stdout}"
    );
    assert!(
        stdout.contains("0/5") || stdout.contains("index=5"),
        "expected chain/index 0/5 in stdout: {stdout}"
    );
}

#[test]
fn zpub_p2wpkh_match_internal_1_3() {
    // Address derived at the internal-change chain m/1/3. Default scan covers
    // both chains; the JSON should report chain="internal", index=3.
    let acct = account_xpub_at("m/84'/0'/0'");
    let zpub = xpub_as_zpub(&acct);
    let target = p2wpkh_addr(&child_at(&acct, 1, 3), NetworkKind::Main);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &zpub,
            "--target-address",
            &target,
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    let results = json["results"].as_array().expect("results array");
    assert_eq!(results.len(), 1, "one target");
    assert_eq!(results[0]["result"], "match");
    assert_eq!(results[0]["chain"], "internal");
    assert_eq!(results[0]["index"], 3);
    assert_eq!(results[0]["script_type"], "p2wpkh");
}

#[test]
fn ypub_p2sh_p2wpkh_match() {
    // BIP-49 mainnet account xpub, ypub-prefixed. The P2SH-P2WPKH address
    // at m/0/2 should match.
    let acct = account_xpub_at("m/49'/0'/0'");
    let ypub = xpub_as_ypub(&acct);
    let target = p2shwpkh_addr(&child_at(&acct, 0, 2), NetworkKind::Main);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &ypub,
            "--target-address",
            &target,
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(json["results"][0]["result"], "match");
    assert_eq!(json["results"][0]["index"], 2);
    assert_eq!(json["results"][0]["script_type"], "p2sh-p2wpkh");
}

#[test]
fn xpub_explicit_p2pkh_match_exercises_gap_fix() {
    // v0.26.0 P3 5-site gap-fix: xpub + explicit --address-type p2pkh.
    // The xpub prefix is neutral (no SLIP-0132 signal), so --address-type
    // is required.
    let acct = account_xpub_at("m/44'/0'/0'");
    let xpub = acct.to_string();
    let target = p2pkh_addr(&child_at(&acct, 0, 7), NetworkKind::Main);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &xpub,
            "--target-address",
            &target,
            "--address-type",
            "p2pkh",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(json["results"][0]["result"], "match");
    assert_eq!(json["results"][0]["index"], 7);
    assert_eq!(json["results"][0]["script_type"], "p2pkh");
}

#[test]
fn xpub_explicit_p2tr_match() {
    // BIP-86 mainnet account xpub, neutral prefix. --address-type p2tr
    // is required (no native P2TR prefix in SLIP-0132).
    let acct = account_xpub_at("m/86'/0'/0'");
    let xpub = acct.to_string();
    let target = p2tr_addr(&child_at(&acct, 0, 4), NetworkKind::Main);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &xpub,
            "--target-address",
            &target,
            "--address-type",
            "p2tr",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(json["results"][0]["result"], "match");
    assert_eq!(json["results"][0]["index"], 4);
    assert_eq!(json["results"][0]["script_type"], "p2tr");
}

// ---------------------------------------------------------------------------
// No-match / partial-match
// ---------------------------------------------------------------------------

#[test]
fn no_match_within_gap_limit_returns_exit_4() {
    // Target an address far outside the default gap-limit window. The
    // default gap-limit is 20; target at index 100 should not be found.
    let acct = account_xpub_at("m/84'/0'/0'");
    let zpub = xpub_as_zpub(&acct);
    let target = p2wpkh_addr(&child_at(&acct, 0, 100), NetworkKind::Main);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &zpub,
            "--target-address",
            &target,
        ])
        .assert()
        .failure()
        .code(4);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("address-of-xpub"),
        "stderr should reference mode address-of-xpub: {stderr}"
    );
}

#[test]
fn external_only_skips_internal_chain() {
    // Target on internal chain m/1/3. With --external-only, internal chain
    // is skipped → no match. Also pin the JSON-envelope's scanned_internal:0
    // payload that the manual prose advertises.
    let acct = account_xpub_at("m/84'/0'/0'");
    let zpub = xpub_as_zpub(&acct);
    let target = p2wpkh_addr(&child_at(&acct, 1, 3), NetworkKind::Main);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &zpub,
            "--target-address",
            &target,
            "--external-only",
            "--json",
        ])
        .assert()
        .failure()
        .code(4);
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    let r0 = &json["results"][0];
    assert_eq!(r0["result"], "no_match");
    assert_eq!(
        r0["scanned_external"], 20,
        "external scan covers the default gap_limit=20 window"
    );
    assert_eq!(
        r0["scanned_internal"], 0,
        "--external-only suppresses internal-chain scanning"
    );
}

#[test]
fn gap_limit_50_extends_scan() {
    // Default gap-limit is 20; target at index 35 fails by default but
    // matches with --gap-limit 50.
    let acct = account_xpub_at("m/84'/0'/0'");
    let zpub = xpub_as_zpub(&acct);
    let target = p2wpkh_addr(&child_at(&acct, 0, 35), NetworkKind::Main);

    // Default → no match
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &zpub,
            "--target-address",
            &target,
        ])
        .assert()
        .failure()
        .code(4);

    // --gap-limit 50 → match
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &zpub,
            "--target-address",
            &target,
            "--gap-limit",
            "50",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(json["results"][0]["result"], "match");
    assert_eq!(json["results"][0]["index"], 35);
    assert_eq!(json["gap_limit"], 50);
}

// ---------------------------------------------------------------------------
// Multi-target
// ---------------------------------------------------------------------------

#[test]
fn multi_target_all_match_exit_0() {
    // Two targets, both within gap-limit. Both match → exit 0.
    let acct = account_xpub_at("m/84'/0'/0'");
    let zpub = xpub_as_zpub(&acct);
    let t1 = p2wpkh_addr(&child_at(&acct, 0, 2), NetworkKind::Main);
    let t2 = p2wpkh_addr(&child_at(&acct, 1, 8), NetworkKind::Main);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &zpub,
            "--target-address",
            &t1,
            "--target-address",
            &t2,
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    let results = json["results"].as_array().expect("array");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["result"], "match");
    assert_eq!(results[1]["result"], "match");
}

#[test]
fn multi_target_partial_unmatch_returns_exit_4() {
    // Two targets, one within gap-limit, one outside. Partial match → exit 4.
    let acct = account_xpub_at("m/84'/0'/0'");
    let zpub = xpub_as_zpub(&acct);
    let t1 = p2wpkh_addr(&child_at(&acct, 0, 2), NetworkKind::Main);
    let t2_unmatch = p2wpkh_addr(&child_at(&acct, 0, 99), NetworkKind::Main);

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &zpub,
            "--target-address",
            &t1,
            "--target-address",
            &t2_unmatch,
        ])
        .assert()
        .failure()
        .code(4);
}

// ---------------------------------------------------------------------------
// Refusals / errors
// ---------------------------------------------------------------------------

#[test]
fn multisig_zpub_prefix_refused_exit_1() {
    // Multisig SLIP-0132 prefix (Zpub) is refused with a pointer to
    // account-of-descriptor.
    let acct = account_xpub_at("m/48'/0'/0'/2'");
    let zpub_multi = xpub_as_multisig_zpub(&acct);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &zpub_multi,
            "--target-address",
            "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("single-sig only"),
        "expected the load-bearing 'single-sig only' wording in the refusal: {stderr}"
    );
    assert!(
        stderr.contains("account-of-descriptor"),
        "expected pointer to account-of-descriptor: {stderr}"
    );
}

#[test]
fn xpub_without_address_type_refused() {
    // Neutral xpub (no SLIP-0132 prefix signal) requires --address-type.
    let acct = account_xpub_at("m/84'/0'/0'");
    let xpub = acct.to_string();

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &xpub,
            "--target-address",
            "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--address-type"),
        "expected --address-type refusal: {stderr}"
    );
}

#[test]
fn network_inference_from_tpub_prefix() {
    // tpub neutral testnet prefix → inferred testnet. A testnet bech32
    // address at the right path matches.
    let acct = account_xpub_at_testnet("m/84'/1'/0'");
    let tpub = {
        // Encode as native tpub (testnet bip32 neutral).
        let raw = acct.encode();
        let mut swapped = raw.to_vec();
        swapped[0..4].copy_from_slice(&[0x04, 0x35, 0x87, 0xCF]);
        base58::encode_check(&swapped)
    };
    let target = p2wpkh_addr(&child_at(&acct, 0, 0), NetworkKind::Test);
    assert!(
        target.starts_with("tb1"),
        "expected testnet bech32 prefix: {target}"
    );

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &tpub,
            "--target-address",
            &target,
            "--address-type",
            "p2wpkh",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(json["results"][0]["result"], "match");
}

#[test]
fn network_signet_override_disambiguates_testnet_version_byte() {
    // The xpub version byte collapses testnet / signet / regtest into one
    // family; `--network signet` overrides the inferred testnet default.
    // Address rendering is byte-identical for the signet / testnet HRP
    // (both use `tb1` for native segwit), so the match-rendered address
    // is unchanged — but the override codepath at
    // `address_of_xpub.rs::run_address_of_xpub` step 5 IS exercised.
    let acct = account_xpub_at_testnet("m/84'/1'/0'");
    let tpub = {
        let raw = acct.encode();
        let mut swapped = raw.to_vec();
        swapped[0..4].copy_from_slice(&[0x04, 0x35, 0x87, 0xCF]);
        base58::encode_check(&swapped)
    };
    let target = p2wpkh_addr(&child_at(&acct, 0, 0), NetworkKind::Test);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &tpub,
            "--target-address",
            &target,
            "--address-type",
            "p2wpkh",
            "--network",
            "signet",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    assert_eq!(json["results"][0]["result"], "match");
    assert_eq!(json["results"][0]["chain"], "external");
    assert_eq!(json["results"][0]["index"], 0);
}

#[test]
fn invalid_xpub_returns_exit_1() {
    // Garbage xpub → BadInput → exit 1.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            "xpub-not-valid",
            "--target-address",
            "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu",
        ])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn json_envelope_shape_byte_exact() {
    // Pin the JSON envelope's top-level structure for a single-target match.
    let acct = account_xpub_at("m/84'/0'/0'");
    let zpub = xpub_as_zpub(&acct);
    let target = p2wpkh_addr(&child_at(&acct, 0, 0), NetworkKind::Main);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &zpub,
            "--target-address",
            &target,
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    // Top-level shape (plan §5.5).
    assert_eq!(json["schema_version"], "1");
    assert_eq!(json["mode"], "address-of-xpub");
    assert!(json["xpub_canonical"].is_string());
    assert_eq!(json["xpub_variant"], "zpub");
    assert_eq!(json["gap_limit"], 20);
    // Per-target shape.
    let r0 = &json["results"][0];
    assert_eq!(r0["target"], target);
    assert_eq!(r0["result"], "match");
    assert_eq!(r0["chain"], "external");
    assert_eq!(r0["index"], 0);
    assert_eq!(r0["script_type"], "p2wpkh");
}

#[test]
fn no_match_json_envelope_has_scan_counts() {
    // No-match result should carry scanned_external + scanned_internal counts.
    let acct = account_xpub_at("m/84'/0'/0'");
    let zpub = xpub_as_zpub(&acct);
    let target = p2wpkh_addr(&child_at(&acct, 0, 99), NetworkKind::Main);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "address-of-xpub",
            "--xpub",
            &zpub,
            "--target-address",
            &target,
            "--json",
        ])
        .assert()
        .failure()
        .code(4);
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid JSON");
    let r0 = &json["results"][0];
    assert_eq!(r0["result"], "no_match");
    assert_eq!(r0["scanned_external"], 20);
    assert_eq!(r0["scanned_internal"], 20);
}
