//! v0.5.1 mode-violation guards. Covers the 3 retained guards under `--slot`
//! invocations: THRESHOLD_WITHOUT_MULTISIG, PATH_FAMILY_WITHOUT_MULTISIG,
//! DESCRIPTOR_AND_TEMPLATE. Each guard has a happy-path counterpart.
//!
//! The expected stderr strings below are byte-exact copies of `mode_text`
//! consts in `src/cmd/bundle.rs` — the binary crate has no `lib.rs`, so the
//! integration tests can't import them.

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const TREZOR_24_2: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";

const THRESHOLD_WITHOUT_MULTISIG: &str = "--threshold is meaningful only with a multisig --template; single-sig templates ignore threshold.";
const PATH_FAMILY_WITHOUT_MULTISIG: &str =
    "--multisig-path-family is meaningful only with a multisig --template.";
const DESCRIPTOR_AND_TEMPLATE: &str = "--descriptor and --template are mutually exclusive; pick descriptor passthrough or template, not both.";

#[test]
fn threshold_without_multisig_template_rejected() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--threshold",
            "2",
            "--no-engraving-card",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(&(format!("error: {}\n", THRESHOLD_WITHOUT_MULTISIG))),
        "{}; got {:?}",
        "stderr must be the byte-exact retained-guard text",
        stderr,
    )
}

#[test]
fn threshold_with_multisig_template_accepted() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--slot",
            &format!("@1.phrase={TREZOR_24_2}"),
            "--template",
            "wsh-sortedmulti",
            "--network",
            "mainnet",
            "--threshold",
            "2",
            "--no-engraving-card",
        ])
        .assert()
        .success();
}

#[test]
fn path_family_without_multisig_template_rejected() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--multisig-path-family",
            "bip48",
            "--no-engraving-card",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(&(format!("error: {}\n", PATH_FAMILY_WITHOUT_MULTISIG))),
        "{}; got {:?}",
        "stderr must be the byte-exact retained-guard text",
        stderr,
    )
}

#[test]
fn path_family_with_multisig_template_accepted() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--slot",
            &format!("@1.phrase={TREZOR_24_2}"),
            "--template",
            "wsh-sortedmulti",
            "--network",
            "mainnet",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--no-engraving-card",
        ])
        .assert()
        .success();
}

#[test]
fn descriptor_and_template_rejected() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--template",
            "bip84",
            "--descriptor",
            "wpkh([deadbeef/84'/0'/0']@0/<0;1>/*)",
            "--network",
            "mainnet",
            "--no-engraving-card",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(&(format!("error: {}\n", DESCRIPTOR_AND_TEMPLATE))),
        "{}; got {:?}",
        "stderr must be the byte-exact retained-guard text",
        stderr,
    )
}

#[test]
fn descriptor_without_template_accepted() {
    // H7 (cycle-2): use an UN-annotated descriptor. A `[deadbeef/...]@0` origin
    // annotation no longer silently dropped — it now FIRES the per-`@N`
    // master-fingerprint cross-check, and `deadbeef` does NOT match TREZOR_24's
    // master fp, so an annotated descriptor would (correctly) be refused. This
    // test asserts MODE acceptance (descriptor without template), not the
    // annotation behavior, so the annotation is removed.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--descriptor",
            "wpkh(@0/<0;1>/*)",
            "--network",
            "mainnet",
            "--no-engraving-card",
        ])
        .assert()
        .success();
}
