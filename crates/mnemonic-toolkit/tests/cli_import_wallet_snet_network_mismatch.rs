//! cycle-5 S-NET (axis 2 / H15 + L2 + L10): the xpub-version-vs-coin-type
//! network-provenance cross-check, wired at the 7 import parsers via the shared
//! `pipeline::assert_slots_network_agrees` -> `network::assert_network_agrees`
//! helper. A hand-edited blob whose decoded xpub version bytes (mainnet `xpub`
//! / testnet `tpub`) contradict its own BIP-48 coin-type path is now rejected
//! fail-closed (`NetworkMismatch`, exit 2). This is axis 2 — distinct from the
//! H9 `--network`-vs-coin-type-class axis (exit 1, `ImportWalletNetworkClassMismatch`).
//!
//! Each parser gets (a) a RED test that exits 2 (silently accepted pre-fix) and
//! (b) a positive control proving a network-CONSISTENT input still imports
//! unchanged (the over-rejection guard).

use assert_cmd::Command;
use miniscript::descriptor::checksum::Engine as ChecksumEngine;
use std::path::PathBuf;

// Reusable fixture keys (lifted from the existing per-parser test corpora).
const MAINNET_XPUB_A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const MAINNET_XPUB_B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const MAINNET_XPUB_SPECTER: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";
const TESTNET_XPUB_A: &str = "tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC";
const TESTNET_XPUB_B: &str = "tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3";

fn checksum(desc_without_hash: &str) -> String {
    let mut eng = ChecksumEngine::new();
    eng.input(desc_without_hash).expect("ascii-only");
    eng.checksum()
}

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

/// Run import-wallet with the given format and stdin blob, return Output.
fn run_stdin(format: &str, blob: &str) -> std::process::Output {
    bin()
        .args(["import-wallet", "--format", format, "--blob", "-"])
        .write_stdin(blob.to_string())
        .output()
        .expect("spawn")
}

/// Run import-wallet against a fixture path, return Output.
fn run_fixture(format: &str, name: &str) -> std::process::Output {
    bin()
        .args([
            "import-wallet",
            "--format",
            format,
            "--blob",
            fixture(name).to_str().unwrap(),
        ])
        .output()
        .expect("spawn")
}

fn assert_network_mismatch(out: &std::process::Output, who: &str) {
    assert_eq!(
        out.status.code(),
        Some(2),
        "{who}: xpub-version-vs-coin-type mismatch must exit 2; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("network mismatch"),
        "{who}: expected NetworkMismatch message; got: {stderr}"
    );
}

// ── descriptor (site 1) ──────────────────────────────────────────────────────

#[test]
fn descriptor_tpub_on_coin_type_0_rejects() {
    let blob = format!("wpkh([704c7836/84'/0'/0']{TESTNET_XPUB_A}/<0;1>/*)\n");
    assert_network_mismatch(&run_stdin("descriptor", &blob), "descriptor");
}

#[test]
fn descriptor_xpub_on_coin_type_1_rejects() {
    // The mirror case: a mainnet xpub on a testnet coin-type path.
    let blob = format!("wpkh([704c7836/84'/1'/0']{MAINNET_XPUB_A}/<0;1>/*)\n");
    assert_network_mismatch(&run_stdin("descriptor", &blob), "descriptor (xpub-on-1)");
}

#[test]
fn descriptor_consistent_mainnet_imports() {
    let blob = format!("wpkh([704c7836/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)\n");
    let out = run_stdin("descriptor", &blob);
    assert_eq!(
        out.status.code(),
        Some(0),
        "consistent mainnet must import; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn descriptor_consistent_testnet_imports() {
    let blob = format!("wpkh([704c7836/84'/1'/0']{TESTNET_XPUB_A}/<0;1>/*)\n");
    let out = run_stdin("descriptor", &blob);
    assert_eq!(out.status.code(), Some(0));
}

// ── specter (site 2) ─────────────────────────────────────────────────────────

#[test]
fn specter_xpub_on_coin_type_1_rejects() {
    let body = format!("wpkh([5436d724/84'/1'/0']{MAINNET_XPUB_SPECTER}/<0;1>/*)");
    let cs = checksum(&body);
    let blob = format!(
        "{{\n  \"label\": \"D\",\n  \"blockheight\": 800000,\n  \"descriptor\": \"{body}#{cs}\",\n  \"devices\": [{{\"type\": \"coldcard\", \"label\": \"p\"}}]\n}}\n"
    );
    assert_network_mismatch(&run_stdin("specter", &blob), "specter");
}

#[test]
fn specter_consistent_mainnet_imports() {
    let body = format!("wpkh([5436d724/84'/0'/0']{MAINNET_XPUB_SPECTER}/<0;1>/*)");
    let cs = checksum(&body);
    let blob = format!(
        "{{\n  \"label\": \"D\",\n  \"blockheight\": 800000,\n  \"descriptor\": \"{body}#{cs}\",\n  \"devices\": [{{\"type\": \"coldcard\", \"label\": \"p\"}}]\n}}\n"
    );
    let out = run_stdin("specter", &blob);
    assert_eq!(out.status.code(), Some(0));
}

// ── sparrow (site 3) ─────────────────────────────────────────────────────────

#[test]
fn sparrow_xpub_on_coin_type_1_rejects() {
    assert_network_mismatch(
        &run_fixture(
            "sparrow",
            "sparrow-singlesig-mainnet-xpub-on-cointype1.json",
        ),
        "sparrow",
    );
}

#[test]
fn sparrow_consistent_mainnet_imports() {
    let out = run_fixture("sparrow", "sparrow-singlesig-p2wpkh.json");
    assert_eq!(out.status.code(), Some(0));
}

// ── bitcoin-core (site 4) ────────────────────────────────────────────────────

#[test]
fn bitcoin_core_tpub_on_coin_type_0_rejects() {
    let body = format!("wpkh([704c7836/84'/0'/0']{TESTNET_XPUB_A}/<0;1>/*)");
    let cs = checksum(&body);
    let blob = format!(
        "{{\n  \"wallet_name\": \"x\",\n  \"descriptors\": [\n    {{\n      \"desc\": \"{body}#{cs}\",\n      \"active\": true,\n      \"internal\": false,\n      \"range\": [0, 1000]\n    }}\n  ]\n}}\n"
    );
    assert_network_mismatch(&run_stdin("bitcoin-core", &blob), "bitcoin-core");
}

#[test]
fn bitcoin_core_consistent_mainnet_imports() {
    let body = format!("wpkh([704c7836/84'/0'/0']{MAINNET_XPUB_A}/<0;1>/*)");
    let cs = checksum(&body);
    let blob = format!(
        "{{\n  \"wallet_name\": \"x\",\n  \"descriptors\": [\n    {{\n      \"desc\": \"{body}#{cs}\",\n      \"active\": true,\n      \"internal\": false,\n      \"range\": [0, 1000]\n    }}\n  ]\n}}\n"
    );
    let out = run_stdin("bitcoin-core", &blob);
    assert_eq!(out.status.code(), Some(0));
}

// ── bsms (site 5 = L10) ──────────────────────────────────────────────────────

#[test]
fn bsms_tpub_on_coin_type_0_rejects() {
    // 2-of-2 sortedmulti, testnet tpubs on a mainnet (coin-type-0) path.
    let body = format!(
        "wsh(sortedmulti(2,[704c7836/48'/0'/0'/2']{TESTNET_XPUB_A}/<0;1>/*,[97139860/48'/0'/0'/2']{TESTNET_XPUB_B}/<0;1>/*))"
    );
    let cs = checksum(&body);
    let blob = format!("BSMS 1.0\n{body}#{cs}\n");
    assert_network_mismatch(&run_stdin("bsms", &blob), "bsms");
}

#[test]
fn bsms_consistent_testnet_imports() {
    let body = format!(
        "wsh(sortedmulti(2,[704c7836/48'/1'/0'/2']{TESTNET_XPUB_A}/<0;1>/*,[97139860/48'/1'/0'/2']{TESTNET_XPUB_B}/<0;1>/*))"
    );
    let cs = checksum(&body);
    let blob = format!("BSMS 1.0\n{body}#{cs}\n");
    let out = run_stdin("bsms", &blob);
    assert_eq!(
        out.status.code(),
        Some(0),
        "consistent testnet bsms must import; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn bsms_consistent_mainnet_imports() {
    let body = format!(
        "wsh(sortedmulti(2,[704c7836/48'/0'/0'/2']{MAINNET_XPUB_A}/<0;1>/*,[97139860/48'/0'/0'/2']{MAINNET_XPUB_B}/<0;1>/*))"
    );
    let cs = checksum(&body);
    let blob = format!("BSMS 1.0\n{body}#{cs}\n");
    let out = run_stdin("bsms", &blob);
    assert_eq!(out.status.code(), Some(0));
}

// ── coldcard-multisig (site 6) ───────────────────────────────────────────────

#[test]
fn coldcard_multisig_mainnet_xpub_on_coin_type_1_rejects() {
    assert_network_mismatch(
        &run_fixture(
            "coldcard-multisig",
            "coldcard-ms-2of3-mainnet-xpub-on-cointype1.txt",
        ),
        "coldcard-multisig",
    );
}

#[test]
fn coldcard_multisig_consistent_mainnet_imports() {
    let out = run_fixture("coldcard-multisig", "coldcard-ms-2of3-p2wsh-no-xfp.txt");
    assert_eq!(
        out.status.code(),
        Some(0),
        "consistent mainnet coldcard-multisig must import; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

// ── electrum multisig (site 7 = L2) ──────────────────────────────────────────

#[test]
fn electrum_multisig_mainnet_zpub_on_coin_type_1_rejects() {
    assert_network_mismatch(
        &run_fixture(
            "electrum",
            "electrum-multisig-2of3-mainnet-zpub-on-cointype1.json",
        ),
        "electrum-multisig",
    );
}

#[test]
fn electrum_multisig_consistent_mainnet_imports() {
    let out = run_fixture("electrum", "electrum-multisig-2of3-wsh.json");
    assert_eq!(
        out.status.code(),
        Some(0),
        "consistent mainnet electrum multisig must import; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
