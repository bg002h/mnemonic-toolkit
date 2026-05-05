//! `--self-check` happy-path test (Phase E.2).
//!
//! Self-check on a freshly-emitted bundle MUST succeed; a failure indicates a
//! synthesis/verify inconsistency. Fixtures live at
//! `tests/vectors/v0_2/{bip84,wsh-sortedmulti}-mainnet-0-false-true.txt`.

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

#[test]
fn bundle_self_check_passes_for_canonical_seed_singlesig() {
    let expected = std::fs::read_to_string("tests/vectors/v0_2/bip84-mainnet-0-false-true.txt")
        .expect("fixture");
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
            "--self-check",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, expected, "self-check single-sig fixture mismatch");
}

#[test]
fn bundle_self_check_passes_for_canonical_seed_multisig() {
    let expected =
        std::fs::read_to_string("tests/vectors/v0_2/wsh-sortedmulti-mainnet-0-false-true.txt")
            .expect("fixture");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--phrase",
            TREZOR_24,
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--cosigner-count",
            "3",
            "--self-check",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, expected, "self-check multisig fixture mismatch");
}
