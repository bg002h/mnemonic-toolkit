//! cycle-5 S-NET (L1), as reversed by erratum E1 (SPEC_network_rule unit 7):
//! the two pins that warned or inferred now refuse (exit 8). Original header:
//! `build-descriptor`'s human first-address preview was the
//! ONLY network-disposition that WARNS instead of rejecting — the canonical /
//! bip388 deliverables are network-agnostic and byte-correct; only the optional
//! preview's HRP/label depends on `--network`. New behavior:
//!   - `--network` supplied AND disagrees with the keys' version bytes → stderr
//!     WARNING + exit 0 (preview still rendered for `--network`).
//!   - `--network` omitted → inferred from the keys (preview HRP correct by
//!     default), no warning.
//!   - consistent → no warning.

use assert_cmd::Command;

// Two distinct TESTNET tpubs (decode to NetworkKind::Test).
const TPUB_K1: &str = "[704c7836/84h/1h/0h]tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M";
const TPUB_K2: &str = "[97139860/84h/1h/0h]tpubDC9Go1KDateW3gS8VXZ6DD1Xu7PgoTdPcf1MX9Z6qVLiHbaeDJ78swPyuQ8YQY19QjtrzkfkZSXwqCcb7XArtid1iLq8Vy55Ydfm4giZh6X";

// Two distinct MAINNET xpubs (decode to NetworkKind::Main).
const XPUB_K1: &str = "[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";
const XPUB_K2: &str = "[97139860/48h/0h/0h/2h]xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";

fn build(k1: &str, k2: &str, network: Option<&str>) -> std::process::Output {
    let mut args: Vec<String> = vec![
        "build-descriptor".into(),
        "--archetype".into(),
        "simple-timelocked-inheritance".into(),
        "--key".into(),
        k1.into(),
        "--recovery-key".into(),
        k2.into(),
        "--older".into(),
        "144".into(),
    ];
    if let Some(n) = network {
        args.push("--network".into());
        args.push(n.into());
    }
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .output()
        .expect("spawn")
}

/// Erratum E1 (Spec A unit 7) reverses the two L1 pins below: a key of
/// another network than the run is refused (exit 8, TX-H1), and no `--network`
/// means mainnet (no inference from the keys).
#[test]
fn l1_tpub_with_network_mainnet_warns_not_rejects() {
    let out = build(TPUB_K1, TPUB_K2, Some("mainnet"));
    assert_eq!(
        out.status.code(),
        Some(8),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&out.stderr).lines().collect::<Vec<_>>(),
        ["error: network mismatch: keys @0 and @1 are test-network keys (tpub), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."]
    );
}

#[test]
fn l1_tpub_network_omitted_infers_testnet_preview() {
    let out = build(TPUB_K1, TPUB_K2, None);
    assert_eq!(
        out.status.code(),
        Some(8),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&out.stderr).lines().collect::<Vec<_>>(),
        ["error: network mismatch: keys @0 and @1 are test-network keys (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."]
    );
}

#[test]
fn l1_tpub_with_network_testnet_no_warn() {
    // Consistent: tpub keys + --network testnet → no warning, tb1 preview.
    let out = build(TPUB_K1, TPUB_K2, Some("testnet"));
    assert_eq!(out.status.code(), Some(0));
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("warning"),
        "consistent network must not warn; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("tb1"));
}

#[test]
fn l1_mainnet_xpub_network_omitted_defaults_mainnet_preview() {
    // Mainnet xpub keys, no --network → bc1 preview (inferred mainnet), no warn.
    let out = build(XPUB_K1, XPUB_K2, None);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Mainnet") && stdout.contains("bc1"),
        "expected mainnet bc1 preview; stdout: {stdout}"
    );
    assert!(!String::from_utf8_lossy(&out.stderr).contains("warning"));
}
