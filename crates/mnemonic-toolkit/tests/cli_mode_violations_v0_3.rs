//! v0.3 NEW mode-violation integration tests (SPEC §6.9 v0.3 rows 1-6 +
//! row 8). Rows 7, 9-15 are descriptor-content-aware and land in Phase C
//! integration tests.

use assert_cmd::Command;
use predicates::prelude::*;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

const SIMPLE_DESCRIPTOR: &str = "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))";

// ---- Row 1: --descriptor with --template ----
#[test]
fn descriptor_with_template_rejected() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            SIMPLE_DESCRIPTOR,
            "--template",
            "wsh-sortedmulti",
            "--network",
            "mainnet",
            "--phrase",
            TREZOR_24,
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--descriptor and --template are mutually exclusive",
        ));
}

// ---- Row 2: --descriptor with --descriptor-file (clap conflicts; exit 64) ----
#[test]
fn descriptor_and_descriptor_file_clap_conflicts() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            SIMPLE_DESCRIPTOR,
            "--descriptor-file",
            "/tmp/whatever.txt",
            "--network",
            "mainnet",
            "--phrase",
            TREZOR_24,
        ])
        .assert()
        .failure();
}

// ---- Row 3: --descriptor with --threshold ----
#[test]
fn descriptor_with_threshold_rejected() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            SIMPLE_DESCRIPTOR,
            "--threshold",
            "2",
            "--network",
            "mainnet",
            "--phrase",
            TREZOR_24,
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--threshold is meaningful only with a multisig --template",
        ));
}

// ---- Row 4: --descriptor with --cosigner-count ----
#[test]
fn descriptor_with_cosigner_count_rejected() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            SIMPLE_DESCRIPTOR,
            "--cosigner-count",
            "2",
            "--network",
            "mainnet",
            "--phrase",
            TREZOR_24,
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--cosigner-count is meaningful only with --template",
        ));
}

// ---- Row 5: --descriptor with --multisig-path-family ----
#[test]
fn descriptor_with_path_family_rejected() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            SIMPLE_DESCRIPTOR,
            "--multisig-path-family",
            "bip87",
            "--network",
            "mainnet",
            "--phrase",
            TREZOR_24,
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--multisig-path-family is meaningful only with --template",
        ));
}

// ---- Row 6: --descriptor with --account != 0 ----
#[test]
fn descriptor_with_nonzero_account_rejected() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            SIMPLE_DESCRIPTOR,
            "--account",
            "1",
            "--network",
            "mainnet",
            "--phrase",
            TREZOR_24,
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--account != 0 is meaningful only with --template",
        ));
}

// ---- Row 6 negative: --descriptor with --account 0 (default) is OK ----
#[test]
fn descriptor_with_zero_account_accepted_to_phase_c_stub() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            SIMPLE_DESCRIPTOR,
            "--account",
            "0",
            "--network",
            "mainnet",
            "--phrase",
            TREZOR_24,
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("not yet wired in v0.3 Phase B"));
}

// ---- Row 8: descriptor with no @N placeholder ----
#[test]
fn descriptor_without_placeholders_rejected() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            "wpkh(xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ/0/*)",
            "--network",
            "mainnet",
            "--phrase",
            TREZOR_24,
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("at least one @N placeholder"));
}

// ---- Clap-level: missing --template AND missing --descriptor / --descriptor-file ----
#[test]
fn neither_template_nor_descriptor_rejected_by_clap() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["bundle", "--network", "mainnet", "--phrase", TREZOR_24])
        .assert()
        .failure();
}
