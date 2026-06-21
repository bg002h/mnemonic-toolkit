//! cycle-5 S-NET (L1): `build-descriptor`'s human first-address preview is the
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

#[test]
fn l1_tpub_with_network_mainnet_warns_not_rejects() {
    // RED: the deliverable is network-agnostic, so this must WARN (stderr) and
    // EXIT 0 — never reject.
    let out = build(TPUB_K1, TPUB_K2, Some("mainnet"));
    assert_eq!(
        out.status.code(),
        Some(0),
        "L1 must WARN not reject; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("warning")
            && stderr.contains("disagrees with descriptor keys")
            && stderr.contains("testnet"),
        "expected the network-disagreement WARN; got: {stderr}"
    );
}

#[test]
fn l1_tpub_network_omitted_infers_testnet_preview() {
    // No `--network`: the preview HRP must be inferred from the tpub keys
    // (tb1…), NOT the historical Mainnet default — and NO warning.
    let out = build(TPUB_K1, TPUB_K2, None);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Testnet") && stdout.contains("tb1"),
        "expected inferred-testnet preview (tb1…); stdout: {stdout}"
    );
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("warning"),
        "no warning when --network is omitted"
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
