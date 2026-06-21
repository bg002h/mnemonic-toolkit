//! cycle-5 S-NET (axis 2): the network-provenance cross-check at the
//! convert + export sites (M14 / L11 / M13). All exit 2 (`NetworkMismatch`).
//!
//! - M14 `convert --xpub-prefix`: re-emitting an xpub into a `--network` family
//!   that disagrees with the xpub's OWN version bytes.
//! - L11 `convert --from wif --to xpub`: the sentinel xpub took its network
//!   from `--network`, discarding the WIF's own `pk.network`.
//! - M13 `export-wallet --from-import-json`: the envelope's declared
//!   `bundle.network` is trusted but cross-checked against the decoded xpubs.
//!
//! Each: a RED test (exit 2 + "network mismatch" message) + a consistent
//! positive control (exit 0).

use assert_cmd::Command;
use std::path::PathBuf;

const MAINNET_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const TESTNET_TPUB: &str = "tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M";

// WIFs derived from the all-`abandon` Trezor seed at m/84'/{0,1}'/0'/0/0.
const MAINNET_WIF: &str = "KyZpNDKnfs94vbrwhJneDi77V6jF64PWPF8x5cdJb8ifgg2DUc9d";
const TESTNET_WIF: &str = "cTGhosGriPpuGA586jemcuH9pE9spwUmneMBmYYzrQEbY92DJrbo";

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

fn assert_mismatch(out: &std::process::Output, who: &str) {
    assert_eq!(
        out.status.code(),
        Some(2),
        "{who}: must exit 2; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("network mismatch"),
        "{who}: expected NetworkMismatch; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// ── M14: convert --xpub-prefix ───────────────────────────────────────────────

#[test]
fn m14_tpub_reemit_into_mainnet_prefix_rejects() {
    let out = bin()
        .args([
            "convert",
            "--from",
            &format!("xpub={TESTNET_TPUB}"),
            "--to",
            "xpub",
            "--xpub-prefix",
            "zpub",
            "--network",
            "mainnet",
        ])
        .output()
        .expect("spawn");
    assert_mismatch(&out, "M14 convert --xpub-prefix");
}

#[test]
fn m14_consistent_mainnet_xpub_prefix_ok() {
    let out = bin()
        .args([
            "convert",
            "--from",
            &format!("xpub={MAINNET_XPUB}"),
            "--to",
            "xpub",
            "--xpub-prefix",
            "zpub",
            "--network",
            "mainnet",
        ])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains("zpub"));
}

// ── L11: convert --from wif --to xpub ────────────────────────────────────────

#[test]
fn l11_testnet_wif_to_xpub_mainnet_rejects() {
    let out = bin()
        .args([
            "convert",
            "--from",
            &format!("wif={TESTNET_WIF}"),
            "--to",
            "xpub",
            "--network",
            "mainnet",
        ])
        .output()
        .expect("spawn");
    assert_mismatch(&out, "L11 convert wif→xpub");
}

#[test]
fn l11_testnet_wif_to_xpub_testnet_ok() {
    let out = bin()
        .args([
            "convert",
            "--from",
            &format!("wif={TESTNET_WIF}"),
            "--to",
            "xpub",
            "--network",
            "testnet",
        ])
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The WIF's true network (testnet) → a tpub sentinel.
    assert!(String::from_utf8_lossy(&out.stdout).contains("tpub"));
}

#[test]
fn l11_mainnet_wif_to_xpub_mainnet_ok() {
    let out = bin()
        .args([
            "convert",
            "--from",
            &format!("wif={MAINNET_WIF}"),
            "--to",
            "xpub",
            "--network",
            "mainnet",
        ])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(0));
}

// ── M13: export-wallet --from-import-json ────────────────────────────────────

#[test]
fn m13_mainnet_label_testnet_keys_envelope_rejects() {
    let out = bin()
        .args([
            "export-wallet",
            "--from-import-json",
            fixture("snet-envelope-mainnet-label-testnet-keys.json")
                .to_str()
                .unwrap(),
            "--format",
            "descriptor",
        ])
        .output()
        .expect("spawn");
    assert_mismatch(&out, "M13 export --from-import-json");
}

#[test]
fn m13_consistent_testnet_envelope_ok() {
    let out = bin()
        .args([
            "export-wallet",
            "--from-import-json",
            fixture("snet-envelope-testnet-consistent.json")
                .to_str()
                .unwrap(),
            "--format",
            "descriptor",
        ])
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
