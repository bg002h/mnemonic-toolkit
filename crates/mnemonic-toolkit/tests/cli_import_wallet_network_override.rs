//! v0.34.6 — `import-wallet --network` signet/regtest disambiguation override.
//! Closes `wallet-import-signet-regtest-disambiguation`.
//!
//! coin-type-1 collapses testnet/signet/regtest (SPEC §4.2 step 8); `--network`
//! re-binds within the parsed coin-type class. Cross-class is refused.
//!
//! The network rule (Spec B unit 3, R3.9; Spec A §4.1 "import-wallet, blob
//! class against run"): the run is `--network` or mainnet, so a test blob
//! with no flag is refused, and every cross-class refusal is TX-H1, exit 8,
//! stdout empty.
use assert_cmd::Command;

const H1_TEST_KEY_NOFLAG: &str = "error: network mismatch: key @0 is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.";
const H1_TEST_KEY_MAINNET: &str = "error: network mismatch: key @0 is a test-network key (tpub), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.";

#[track_caller]
fn assert_refused_with(out: &std::process::Output, line: &str) {
    assert_eq!(out.status.code(), Some(8));
    assert!(out.stdout.is_empty(), "stdout must be empty");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.lines().any(|l| l == line), "got: {stderr}");
}

const FIXTURE_BASE: &str = "tests/fixtures/wallet_import";

fn run_import(fixture: &str, network: Option<&str>) -> std::process::Output {
    let path = std::path::PathBuf::from(FIXTURE_BASE).join(fixture);
    let p = path.to_str().unwrap().to_string();
    let mut args: Vec<String> = [
        "import-wallet",
        "--format",
        "bitcoin-core",
        "--blob",
        &p,
        "--json",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    if let Some(n) = network {
        args.push("--network".to_string());
        args.push(n.to_string());
    }
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .output()
        .expect("spawn")
}

fn bundle_network(out: &std::process::Output) -> String {
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    v[0]["bundle"]["network"]
        .as_str()
        .expect("network field")
        .to_string()
}

#[test]
fn testnet_blob_default_network_is_testnet() {
    // R3.9: no --network is a mainnet run; the blob's tpub contradicts it.
    let out = run_import("core-testnet-bip84.json", None);
    assert_refused_with(&out, H1_TEST_KEY_NOFLAG);
    let out = run_import("core-testnet-bip84.json", Some("testnet"));
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(bundle_network(&out), "testnet");
}

#[test]
fn testnet_blob_override_to_signet() {
    let out = run_import("core-testnet-bip84.json", Some("signet"));
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(bundle_network(&out), "signet");
}

#[test]
fn testnet_blob_override_to_regtest() {
    let out = run_import("core-testnet-bip84.json", Some("regtest"));
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(bundle_network(&out), "regtest");
}

#[test]
fn testnet_blob_override_to_mainnet_refused() {
    let out = run_import("core-testnet-bip84.json", Some("mainnet"));
    assert_refused_with(&out, H1_TEST_KEY_MAINNET);
}

#[test]
fn mainnet_blob_override_to_signet_refused() {
    let out = run_import("core-bip84-mainnet.json", Some("signet"));
    assert_refused_with(&out, "error: network mismatch: key @0 is a mainnet key (xpub), but --network is signet. Nothing was written. For a mainnet wallet, rerun with --network mainnet. For a signet wallet, export the key again with the signer set to a test network.");
}

#[test]
fn mainnet_blob_override_to_mainnet_noop_ok() {
    let out = run_import("core-bip84-mainnet.json", Some("mainnet"));
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(bundle_network(&out), "mainnet");
}

// --- cycle-5 H9 (axis 1): the `--network` class-check must read the SAME
// per-entry network the rebind writes. Pre-fix, the guard inspected
// `parsed.first()` only, then `iter_mut()` rebound EVERY entry — so a
// heterogeneous `[Bitcoin, Testnet]` Vec whose `first()` matched the override
// passed and the Testnet entry was silently relabeled mainnet. Exit 1,
// `ImportWalletNetworkClassMismatch` (the adjacent-sibling refusal). ---

#[test]
fn mixed_mainnet_testnet_blob_override_mainnet_refused_per_entry() {
    // entry0 = mainnet (passes the old first()-only check); entry1 = testnet
    // (coin-type-1) → must be caught per-entry now.
    let out = run_import("core-mixed-mainnet-testnet.json", Some("mainnet"));
    // Every entry is checked (each ParsedImport counts its own `key @i`).
    assert_refused_with(&out, H1_TEST_KEY_MAINNET);
}

#[test]
fn homogeneous_two_mainnet_blob_override_mainnet_ok() {
    // Positive control: a homogeneous all-mainnet blob + --network mainnet must
    // STILL import (the per-entry extension does NOT over-reject a valid blob).
    let out = run_import("core-two-mainnet.json", Some("mainnet"));
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(bundle_network(&out), "mainnet");
}
