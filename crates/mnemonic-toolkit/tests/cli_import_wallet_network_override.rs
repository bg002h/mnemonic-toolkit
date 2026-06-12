//! v0.34.6 — `import-wallet --network` signet/regtest disambiguation override.
//! Closes `wallet-import-signet-regtest-disambiguation`.
//!
//! coin-type-1 collapses testnet/signet/regtest (SPEC §4.2 step 8); `--network`
//! re-binds within the parsed coin-type class. Cross-class is refused.
use assert_cmd::Command;

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
    let out = run_import("core-testnet-bip84.json", None);
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
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("coin-type"),
        "expected coin-type-class mismatch; got: {stderr}"
    );
}

#[test]
fn mainnet_blob_override_to_signet_refused() {
    let out = run_import("core-bip84-mainnet.json", Some("signet"));
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn mainnet_blob_override_to_mainnet_noop_ok() {
    let out = run_import("core-bip84-mainnet.json", Some("mainnet"));
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(bundle_network(&out), "mainnet");
}
