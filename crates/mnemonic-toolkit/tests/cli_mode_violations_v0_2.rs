//! v0.2 NEW mode-violation integration tests (SPEC §6.6 v0.2 NEW rows).
//!
//! Covers byte-exact rejection text + exit-2 contract for the seven new
//! mode-violation rows added in v0.2: --xpub vs --cosigner, --cosigner vs
//! --cosigners-file, --threshold/--cosigner-count/--multisig-path-family
//! requiring a multisig template, and --privacy-preserving + --xpub.

use assert_cmd::Command;
use predicates::prelude::*;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

#[test]
fn xpub_with_cosigner_rejected_byte_exact() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--xpub",
            "xpub6...",
            "--master-fingerprint",
            "deadbeef",
            "--cosigner",
            "x:y:z",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--xpub cannot be combined with --cosigner or --cosigners-file",
        ));
}

#[test]
fn cosigner_with_cosigners_file_rejected_byte_exact() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--phrase",
            TREZOR_24,
            "--cosigner",
            "x:y:z",
            "--cosigners-file",
            "/tmp/nonexistent-cosigners.json",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--cosigner cannot be combined with --cosigners-file",
        ));
}

#[test]
fn threshold_without_multisig_rejected_byte_exact() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--phrase",
            TREZOR_24,
            "--threshold",
            "2",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--threshold is meaningful only with a multisig --template",
        ));
}

#[test]
fn cosigner_count_without_multisig_rejected_byte_exact() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--phrase",
            TREZOR_24,
            "--cosigner-count",
            "3",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--cosigner-count is meaningful only with a multisig --template",
        ));
}

#[test]
fn path_family_without_multisig_rejected_byte_exact() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--phrase",
            TREZOR_24,
            "--multisig-path-family",
            "bip48",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--multisig-path-family is meaningful only with a multisig --template",
        ));
}

#[test]
fn privacy_with_xpub_rejected_byte_exact() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--xpub",
            "xpub6...",
            "--master-fingerprint",
            "deadbeef",
            "--privacy-preserving",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--privacy-preserving with --xpub (single-sig watch-only) has no useful effect",
        ));
}

#[test]
fn xpub_with_cosigners_file_rejected_byte_exact() {
    // Mirror of xpub_with_cosigner_rejected_byte_exact but exercising the
    // --cosigners-file branch of the same union check.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--xpub",
            "xpub6...",
            "--master-fingerprint",
            "deadbeef",
            "--cosigners-file",
            "/tmp/nonexistent.json",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--xpub cannot be combined with --cosigner or --cosigners-file",
        ));
}
