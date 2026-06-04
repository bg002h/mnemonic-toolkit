//! v0.42.0 — `mnemonic export-wallet --format descriptor`.
//!
//! Emits the bare canonical multipath `<descriptor>#<checksum>` on one line
//! (stdout or `--output <file>`), no wallet-file wrapper. Works for single-sig
//! AND multisig (unlike `--format green`, which refuses multisig).
//!
//! See `design/SPEC_export_wallet_format_descriptor.md` §7 (tests 1-6).

use assert_cmd::Command;

/// abandon×11 + about test seed's bip84 account xpub (m/84'/0'/0').
/// Hardcoded LITERAL (SPEC R0-m1) — avoids shelling `convert` (label-prefixes
/// `xpub: …`) and the cross-binary stdout-shape coupling.
const ACCT_XPUB_BIP84: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";

/// SPEC §7 test 1 (smoke) — single-sig bip84, stdout: one line, starts
/// `wpkh(`, contains `<0;1>`, ends `#<8 alnum>\n` (single trailing newline).
#[test]
fn export_descriptor_singlesig_bip84_smoke() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            &format!("@0.xpub={ACCT_XPUB_BIP84}"),
            "--format",
            "descriptor",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();

    // Exactly one line (single trailing newline, no extra lines).
    assert_eq!(
        stdout.matches('\n').count(),
        1,
        "expected exactly one trailing newline, got: {stdout:?}"
    );
    assert!(stdout.ends_with('\n'), "must end with newline: {stdout:?}");
    let line = stdout.trim_end_matches('\n');
    assert!(!line.contains('\n'), "must be one line: {line:?}");

    assert!(line.starts_with("wpkh("), "must start with wpkh(: {line:?}");
    assert!(line.contains("<0;1>"), "must be multipath <0;1>: {line:?}");

    // Ends `#<8 alnum>`.
    let pos = line.rfind('#').expect("must carry BIP-380 #checksum");
    let csum = &line[pos + 1..];
    assert_eq!(csum.len(), 8, "checksum must be 8 chars: {csum:?}");
    assert!(
        csum.chars().all(|c| c.is_ascii_alphanumeric()),
        "checksum must be ASCII-alphanumeric: {csum:?}"
    );
}
