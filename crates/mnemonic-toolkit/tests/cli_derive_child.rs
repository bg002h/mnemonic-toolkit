//! v0.7 Phase 6 — `mnemonic derive-child` integration tests.
//!
//! SPEC `design/SPEC_derive_child_v0_7.md` §6: 6 reference-vector cells
//! (BIP-39×2, HD-Seed WIF, XPRV, HEX, PWD BASE64, PWD BASE85) + 3 refusal
//! cells (unsupported app, bip39 length out-of-range, hd-seed length
//! not-applicable).
//!
//! All reference vectors come verbatim from BIP-85 §"Test Vectors"
//! (<https://github.com/bitcoin/bips/blob/master/bip-0085.mediawiki#test-vectors>),
//! all using the spec-provided master xprv.

use assert_cmd::Command;

const MASTER_XPRV: &str =
    "xprv9s21ZrQH143K2LBWUUQRFXhucrQqBpKdRRxNVq2zBqsx8HVqFk2uYo8kmbaLLHRdqtQpUm98uKfu3vca1LqdGhUtyoFnCNkfmXRyPXLjbKb";

/// SPEC §6 cell 1 — BIP-85 BIP-39 12-English-word reference vector.
/// Path m/83696968'/39'/0'/12'/0' → "girl mad pet galaxy egg matter matrix prison refuse sense ordinary nose".
#[test]
fn cell_1_bip39_12_words_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "bip39",
            "--length",
            "12",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        "girl mad pet galaxy egg matter matrix prison refuse sense ordinary nose\n",
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// SPEC §6 cell 2 — BIP-85 BIP-39 18-English-word reference vector.
/// Path m/83696968'/39'/0'/18'/0'.
#[test]
fn cell_2_bip39_18_words_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "bip39",
            "--length",
            "18",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        "near account window bike charge season chef number sketch tomorrow excuse sniff circle vital hockey outdoor supply token\n",
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// SPEC §6 cell 3 — BIP-85 HD-Seed WIF reference vector.
/// Path m/83696968'/2'/0' → WIF Kzyv4uF39d4Jrw2W7UryTHwZr1zQVNk4dAFyqE6BuMrMh1Za7uhp.
/// `--length` is required at clap level for SPEC §2 grammar-uniformity but
/// ignored at validation when `0` (the sentinel); any non-zero value
/// triggers the SPEC §7 not-applicable refusal (cell 9).
#[test]
fn cell_3_hd_seed_wif_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "hd-seed",
            "--length",
            "0",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        "Kzyv4uF39d4Jrw2W7UryTHwZr1zQVNk4dAFyqE6BuMrMh1Za7uhp\n",
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// SPEC §6 cell 4 — BIP-85 XPRV reference vector.
/// Path m/83696968'/32'/0'. `--length 0` sentinel per SPEC §2 (see cell 3).
#[test]
fn cell_4_xprv_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "xprv",
            "--length",
            "0",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        "xprv9s21ZrQH143K2srSbCSg4m4kLvPMzcWydgmKEnMmoZUurYuBuYG46c6P71UGXMzmriLzCCBvKQWBUv3vPB3m1SATMhp3uEjXHJ42jFg7myX\n",
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// SPEC §6 cell 5 — BIP-85 HEX reference vector.
/// Path m/83696968'/128169'/64'/0' → 64 hex bytes per spec.
#[test]
fn cell_5_hex_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "hex",
            "--length",
            "64",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        "492db4698cf3b73a5a24998aa3e9d7fa96275d85724a91e71aa2d645442f878555d078fd1f1f67e368976f04137b1f7a0d19232136ca50c44614af72b5582a5c\n",
    );
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// SPEC §6 cell 6a — BIP-85 PWD BASE64 reference vector.
/// Path m/83696968'/707764'/21'/0' → "dKLoepugzdVJvdL56ogNV".
#[test]
fn cell_6a_pwd_base64_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "password-base64",
            "--length",
            "21",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, "dKLoepugzdVJvdL56ogNV\n");
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// SPEC §6 cell 6b — BIP-85 PWD BASE85 reference vector.
/// Path m/83696968'/707785'/12'/0' → "_s`{TW89)i4`".
#[test]
fn cell_6b_pwd_base85_reference_vector() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "password-base85",
            "--length",
            "12",
            "--index",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, "_s`{TW89)i4`\n");
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on stdout"),
        "secret-on-stdout warning expected; got stderr: {stderr:?}"
    );
}

/// SPEC §7 — refusal: --application rsa is out-of-scope for v0.7. Byte-exact stderr.
#[test]
fn cell_7_unsupported_application_rsa_refusal() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "rsa",
            "--length",
            "32",
            "--index",
            "0",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr.trim(),
        "error: --application <rsa|rsa-gpg|dice> is out-of-scope for v0.7 \
         (rsa crate not in tree; dice is niche). Tracked for v0.8+.",
    );
}

/// SPEC §7 — refusal: --length 16 invalid for bip39 (valid is 12|15|18|21|24).
#[test]
fn cell_8_bip39_length_out_of_range_refusal() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "bip39",
            "--length",
            "16",
            "--index",
            "0",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr.trim(),
        "error: --length 16 out of range for --application bip39 (valid: 12 | 15 | 18 | 21 | 24 words)",
    );
}

/// SPEC §7 — refusal: --length not applicable for hd-seed (output is fixed-size).
#[test]
fn cell_9_hd_seed_length_not_applicable_refusal() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "hd-seed",
            "--length",
            "32",
            "--index",
            "0",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr.trim(),
        "error: --length not applicable for --application <hd-seed|xprv> (output is fixed-size)",
    );
}

/// SPEC §7 — refusal: --length not applicable for xprv (output is fixed-size).
/// Mirrors cell 9 for the xprv branch of the not-applicable family.
#[test]
fn cell_9b_xprv_length_not_applicable_refusal() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "derive-child",
            "--from",
            &format!("xprv={MASTER_XPRV}"),
            "--application",
            "xprv",
            "--length",
            "32",
            "--index",
            "0",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr.trim(),
        "error: --length not applicable for --application <hd-seed|xprv> (output is fixed-size)",
    );
}
