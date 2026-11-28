//! Spec A unit 7: build-descriptor under the network rule (R7.1-R7.10).
//! Expected lines are literals from Spec A §3 filled in by hand; every exit-8
//! leg asserts stdout is empty.

use crate::network_rule_support::{assert_refused, err, out, run, T1, T2, X1, X1C1, X2};

fn build(key: &str, rec: &str, extra: &[&str]) -> std::process::Output {
    let mut a = vec![
        "build-descriptor",
        "--archetype",
        "simple-timelocked-inheritance",
        "--key",
        key,
        "--recovery-key",
        rec,
        "--older",
        "144",
    ];
    a.extend(extra);
    run(&a)
}

const H1_TT_NOFLAG: &str = "error: network mismatch: keys @0 and @1 are test-network keys (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.";

#[test]
fn nr7_build_descriptor_keys_against_run_refused() {
    // R7.1
    assert_refused(&build(T1, T2, &["--network", "mainnet"]), &["error: network mismatch: keys @0 and @1 are test-network keys (tpub), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."]);
    // R7.2
    assert_refused(&build(X1, X2, &["--network", "testnet"]), &["error: network mismatch: keys @0 and @1 are mainnet keys (xpub), but --network is testnet. Nothing was written. For a mainnet wallet, rerun with --network mainnet. For a testnet wallet, export the key again with the signer set to a test network."]);
    // R7.5
    assert_refused(&build(T1, T2, &[]), &[H1_TT_NOFLAG]);
}

#[test]
fn nr7_build_descriptor_mixed_keys_refused() {
    // R7.3
    assert_refused(&build(X1, T2, &[]), &[
        "error: network mismatch: key @0 is a mainnet key (xpub), but key @1 is a test-network key (tpub). Nothing was written. Every key of one wallet is for one network: export the odd key again.",
        "error: network mismatch: key @1 is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a mainnet wallet, export the key again with the signer set to mainnet.",
    ]);
}

#[test]
fn nr7_build_descriptor_coin_type_soft() {
    // R7.4
    assert_refused(&build(X1C1, X2, &[]), &["error: network mismatch (coin type): key @0's path m/48h/1h/0h/2h is a test-network path (coin type 1h), but key @0 is a mainnet key (xpub). Nothing was written. If the path is a mistake, export the key again at the mainnet path (coin type 0h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time."]);
    let o = build(X1C1, X2, &["--accept-coin-type-mismatch"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("bc1qj7hsd"), "{}", out(&o));
    assert!(err(&o).lines().any(|l| l == "warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: key @0's path m/48h/1h/0h/2h is a test-network path; the output is for mainnet. A signer or recovery tool that looks only at mainnet paths will not find coins sent here."), "{}", err(&o));
}

#[test]
fn nr7_build_descriptor_consistent_unchanged() {
    // R7.6
    let o = build(X1, X2, &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("bc1qj7hsd"), "{}", out(&o));
    assert!(!err(&o).contains("network"), "{}", err(&o));
    // R7.10: the GUI's emit-spec argv carries the form's --network.
    let o = build(T1, T2, &["--network", "signet", "--emit-spec"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("schema_version"), "{}", out(&o));
    assert!(!err(&o).contains("network mismatch"), "{}", err(&o));
}

#[test]
fn nr7_build_descriptor_every_output_mode_refused() {
    // R7.7 (--json: no {diagnostics} envelope), R7.8, R7.9.
    for extra in [
        &["--json"][..],
        &["--format", "descriptor"],
        &["--format", "bip388"],
        &["--emit-spec"],
    ] {
        assert_refused(&build(T1, T2, extra), &[H1_TT_NOFLAG]);
    }
}
