//! v0.7 Phase 5 — `mnemonic export-wallet` integration tests.
//!
//! SPEC `design/SPEC_export_wallet_v0_7.md` §9: 5 mandatory + 1 conditional cell.
//! Test vectors derive from the well-known Trezor 12-word seed
//! ("abandon ... about") + a sibling "letter advice ... above" wallet to keep
//! cosigner xpubs distinct without leaking real keys.

use assert_cmd::Command;

// Trezor 12-word seed → BIP-84 mainnet account 0.
const TREZOR_BIP84_XPUB: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";
const TREZOR_BIP84_FP: &str = "73c5da0a";

// Two BIP-48 mainnet xpubs (path m/48'/0'/0'/2') for wsh-sortedmulti tests.
const COSIGNER_A_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const COSIGNER_A_FP: &str = "b8688df1";
const COSIGNER_B_XPUB: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const COSIGNER_B_FP: &str = "28645006";

/// SPEC §9 cell 1: Bitcoin Core importdescriptors round-trip with single-sig wpkh.
#[test]
fn cell_1_bitcoin_core_single_sig_wpkh_round_trip() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_BIP84_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_BIP84_FP}"),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = value.as_array().expect("Bitcoin Core output is JSON array");
    assert_eq!(arr.len(), 2, "multipath splits into 2 entries");

    // Receive entry — checksum byte-pinned.
    assert_eq!(
        arr[0]["desc"].as_str().unwrap(),
        format!("wpkh([73c5da0a/84'/0'/0']{TREZOR_BIP84_XPUB}/0/*)#wc3n3van"),
    );
    assert!(!arr[0]["internal"].as_bool().unwrap());
    assert!(arr[0]["active"].as_bool().unwrap());
    assert_eq!(arr[0]["range"][0].as_u64().unwrap(), 0);
    assert_eq!(arr[0]["range"][1].as_u64().unwrap(), 999);
    assert_eq!(arr[0]["timestamp"].as_str().unwrap(), "now");

    // Change entry.
    assert_eq!(
        arr[1]["desc"].as_str().unwrap(),
        format!("wpkh([73c5da0a/84'/0'/0']{TREZOR_BIP84_XPUB}/1/*)#lv5jvedt"),
    );
    assert!(arr[1]["internal"].as_bool().unwrap());
}

/// SPEC §9 cell 2: BIP-388 wallet_policy round-trip with multisig wsh-sortedmulti.
#[test]
fn cell_2_bip388_wallet_policy_multisig_wsh_sortedmulti() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "bip388",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={COSIGNER_A_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={COSIGNER_A_FP}"),
            "--slot",
            "@0.path=m/48'/0'/0'/2'",
            "--slot",
            &format!("@1.xpub={COSIGNER_B_XPUB}"),
            "--slot",
            &format!("@1.fingerprint={COSIGNER_B_FP}"),
            "--slot",
            "@1.path=m/48'/0'/0'/2'",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(value["name"].as_str().unwrap(), "wsh-sortedmulti");
    assert_eq!(
        value["description_template"].as_str().unwrap(),
        "wsh(sortedmulti(2,@0/**,@1/**))",
    );
    let keys = value["keys_info"].as_array().unwrap();
    assert_eq!(keys.len(), 2);
    assert_eq!(
        keys[0].as_str().unwrap(),
        format!("[{COSIGNER_A_FP}/48'/0'/0'/2']{COSIGNER_A_XPUB}"),
    );
    assert_eq!(
        keys[1].as_str().unwrap(),
        format!("[{COSIGNER_B_FP}/48'/0'/0'/2']{COSIGNER_B_XPUB}"),
    );
}

/// SPEC §9 cell 3: refusal stderr for `phrase=` slot input. Byte-exact per §3.
#[test]
fn cell_3_phrase_slot_refusal_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            "@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let expected = "error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret material, use 'mnemonic bundle'.\n";
    assert_eq!(stderr, expected);
}

/// SPEC §9 cell 4: Sparrow stub refusal stderr. Byte-exact per §7.
#[test]
fn cell_4_sparrow_stub_refusal_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "sparrow",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_BIP84_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_BIP84_FP}"),
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let expected_sparrow = "error: --format <sparrow> is deferred to v0.8 if user demand surfaces; use --format bitcoin-core or --format bip388 instead.\n";
    assert_eq!(stderr, expected_sparrow);

    // Also Specter.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "specter",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_BIP84_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_BIP84_FP}"),
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let expected_specter = "error: --format <specter> is deferred to v0.8 if user demand surfaces; use --format bitcoin-core or --format bip388 instead.\n";
    assert_eq!(stderr, expected_specter);
}

/// SPEC §9 cell 5: `--range 0,4999` override exercised in Bitcoin Core format.
#[test]
fn cell_5_range_override() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--range",
            "0,4999",
            "--slot",
            &format!("@0.xpub={TREZOR_BIP84_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_BIP84_FP}"),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = value.as_array().unwrap();
    assert_eq!(arr[0]["range"][0].as_u64().unwrap(), 0);
    assert_eq!(arr[0]["range"][1].as_u64().unwrap(), 4999);
    assert_eq!(arr[1]["range"][0].as_u64().unwrap(), 0);
    assert_eq!(arr[1]["range"][1].as_u64().unwrap(), 4999);
}

/// SPEC §9 cell 6 (CONDITIONAL): `--bitcoin-core-version 24` shape diff vs. 25.
///
/// PER SPEC §9: "if version 24 differs from 25 materially — confirm during
/// impl; if no diff, document and reduce to a single-version test." For the
/// fields the toolkit emits (`desc` / `active` / `internal` / `range` /
/// `timestamp`), Bitcoin Core 24 and 25 are wire-identical — both versions
/// accept and require this same JSON. The `--bitcoin-core-version` flag is
/// retained for future-proofing (24 vs 25 may diverge in fields the toolkit
/// does not yet emit, e.g. `next_index`); v0.7 emits the byte-identical shape
/// for both. This cell asserts that.
#[test]
fn cell_6_bitcoin_core_version_24_matches_25_for_emitted_fields() {
    let mk_args = |ver: &str| {
        vec![
            "export-wallet".to_string(),
            "--template".into(),
            "bip84".into(),
            "--network".into(),
            "mainnet".into(),
            "--bitcoin-core-version".into(),
            ver.to_string(),
            "--slot".into(),
            format!("@0.xpub={TREZOR_BIP84_XPUB}"),
            "--slot".into(),
            format!("@0.fingerprint={TREZOR_BIP84_FP}"),
        ]
    };
    let out_24 = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(mk_args("24"))
        .assert()
        .success();
    let stdout_24 = String::from_utf8(out_24.get_output().stdout.clone()).unwrap();

    let out_25 = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(mk_args("25"))
        .assert()
        .success();
    let stdout_25 = String::from_utf8(out_25.get_output().stdout.clone()).unwrap();

    assert_eq!(
        stdout_24, stdout_25,
        "Bitcoin Core 24 and 25 emit byte-identical JSON for the toolkit's \
        importdescriptors field set (desc / active / internal / range / timestamp). \
        SPEC §9 cell 6 reduces to documentation per the conditional clause."
    );
}
