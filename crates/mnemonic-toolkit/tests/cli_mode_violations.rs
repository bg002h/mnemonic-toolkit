//! Mode-violation integration tests (Task 5.3).
//!
//! Covers SPEC §6.6 byte-exact rejections + clap-default exit-64 boundaries.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn passphrase_with_xpub_rejected_byte_exact() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--xpub",
            "xpub6...",
            "--master-fingerprint",
            "deadbeef",
            "--passphrase",
            "x",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--passphrase is incompatible with --xpub: the xpub is already a post-passphrase derivation product",
        ));
}

#[test]
fn language_with_xpub_rejected() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--xpub",
            "xpub6...",
            "--master-fingerprint",
            "deadbeef",
            "--language",
            "english",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--language is meaningful only with --phrase",
        ));
}

#[test]
fn xpub_stdin_rejected() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--xpub",
            "-",
            "--master-fingerprint",
            "deadbeef",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("--xpub does not accept stdin"));
}

#[test]
fn fingerprint_short_rejected_byte_exact() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--xpub",
            "xpub6...",
            "--master-fingerprint",
            "dead",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "--master-fingerprint must be 8 hex chars (e.g., deadbeef)",
        ));
}

#[test]
fn xpub_without_fingerprint_byte_exact() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--xpub",
            "xpub6...",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--xpub requires --master-fingerprint (xpub mode needs the master fingerprint to populate mk1's origin)",
        ));
}

#[test]
fn fingerprint_without_xpub_byte_exact() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--phrase",
            "x",
            "--master-fingerprint",
            "deadbeef",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--master-fingerprint is meaningful only with --xpub",
        ));
}

#[test]
fn phrase_with_xpub_collision_clap_exit_64() {
    // SPEC §6.6 leaves --phrase ↔ --xpub collision to clap's mutually-exclusive
    // group → exit 64 with clap's default text. Locks the boundary so byte-
    // exact mode-violation rules never accidentally co-opt this row.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--phrase",
            "x",
            "--xpub",
            "y",
            "--master-fingerprint",
            "deadbeef",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ])
        .assert()
        .failure()
        .code(64);
}

#[test]
fn verify_bundle_no_engraving_card_flag_rejected() {
    // verify-bundle does not emit an engraving card; the flag must not exist.
    // clap-derive auto-rejects unknown flags; main.rs maps that to exit 64.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--no-engraving-card",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--mk1",
            "x",
            "--md1",
            "x",
        ])
        .assert()
        .failure()
        .code(64);
}
