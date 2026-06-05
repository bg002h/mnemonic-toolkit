//! v0.43.0 — `mnemonic restore` (single-sig core, Phase 1).
//!
//! Watch-only restore document: master fingerprint + CONFIRM line, then per-type
//! concrete descriptor + first receive address(es) for bip44/49/84/86 (or a
//! single `--template`). Tests grow across Tasks 1.2 (smoke) → 1.3-1.4 (input
//! channels + exact derivation + watch-only-out negative) → 1.5 (verify-gate).

use assert_cmd::Command;

// Trezor 12-word "abandon ... about" reference seed. Master fingerprint
// `73c5da0a` is path-independent (master xpub fingerprint, not a derived-account
// fingerprint) — asserted in-tree at `cli_export_wallet.rs:27`.
const TREZOR_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const FP_NO_PP: &str = "73c5da0a";

// bip84 single-sig multipath descriptor (with #checksum) for the no-pp seed.
const DESC_BIP84: &str = "wpkh([73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*)#hpg6d6w2";

fn bin() -> Command {
    Command::cargo_bin("mnemonic").unwrap()
}

// ---------------------------------------------------------------------------
// 1.2 smoke
// ---------------------------------------------------------------------------

#[test]
fn restore_phrase_bip84_smoke() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success(), "exit {:?}", out.status.code());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("master fingerprint:"), "stdout:\n{stdout}");
    assert!(stdout.contains(FP_NO_PP), "stdout:\n{stdout}");
    assert!(stdout.contains("wpkh("), "stdout:\n{stdout}");
    assert!(stdout.contains("<0;1>"), "stdout:\n{stdout}");
    assert!(stdout.contains(DESC_BIP84), "stdout:\n{stdout}");
}
