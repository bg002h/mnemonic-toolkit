//! Spec A unit 1: export-wallet under the network rule (R1.1-R1.21).
//!
//! Every expected line is a literal copied from Spec A §3 with the row's
//! fields filled in by hand (plan §3.0). Every exit-8 leg asserts stdout is
//! empty. Fixtures: `design/network-rule/fx` (public test data; commands run
//! from there so `<file>` is the name as given).

use crate::network_rule_support::{assert_refused, err, out, run, T, X};
use assert_cmd::Command;

fn ew_desc(desc: &str, net: Option<&str>, fmt: &str, extra: &[&str]) -> std::process::Output {
    let mut a = vec!["export-wallet", "--descriptor", desc];
    if let Some(n) = net {
        a.extend(["--network", n]);
    }
    a.extend(["--format", fmt, "--count", "1"]);
    a.extend(extra);
    run(&a)
}

const H1_TEST_MAIN_GIVEN: &str = "error: network mismatch: key @0 is a test-network key (tpub), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.";

#[test]
fn nr1_export_wallet_main_key_on_testnet_refused() {
    // R1.1
    let o = ew_desc(
        &format!("wpkh([73c5da0a/84h/0h/0h]{X}/<0;1>/*)"),
        Some("testnet"),
        "bitcoin-core-addresses",
        &[],
    );
    assert_refused(&o, &["error: network mismatch: key @0 is a mainnet key (xpub), but --network is testnet. Nothing was written. For a mainnet wallet, rerun with --network mainnet. For a testnet wallet, export the key again with the signer set to a test network."]);
}

#[test]
fn nr1_export_wallet_test_key_on_mainnet_refused() {
    let d = format!("wpkh([73c5da0a/84h/1h/0h]{T}/<0;1>/*)");
    // R1.2, R1.7, R1.11: every format refuses before rendering.
    for fmt in ["bitcoin-core-addresses", "bip388", "bitcoin-core"] {
        let o = ew_desc(&d, Some("mainnet"), fmt, &[]);
        assert_refused(&o, &[H1_TEST_MAIN_GIVEN]);
    }
}

#[test]
fn nr1_export_wallet_test_key_no_flag_refused() {
    // R1.3
    let o = ew_desc(
        &format!("wpkh({T}/<0;1>/*)"),
        None,
        "bitcoin-core-addresses",
        &[],
    );
    assert_refused(&o, &["error: network mismatch: key @0 is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."]);
}

const S_R14: &str = "error: network mismatch (coin type): key @0's path m/84h/1h/0h is a test-network path (coin type 1h), but key @0 is a mainnet key (xpub). Nothing was written. If the path is a mistake, export the key again at the mainnet path (coin type 0h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time.";
const S_OK_R14: &str = "warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: key @0's path m/84h/1h/0h is a test-network path; the output is for mainnet. A signer or recovery tool that looks only at mainnet paths will not find coins sent here.";

#[test]
fn nr1_export_wallet_coin_type_soft_refused_and_overridden() {
    let d14 = format!("wpkh([73c5da0a/84h/1h/0h]{X}/<0;1>/*)");
    // R1.4
    let o = ew_desc(&d14, Some("mainnet"), "bitcoin-core-addresses", &[]);
    assert_refused(&o, &[S_R14]);
    let o = ew_desc(
        &d14,
        Some("mainnet"),
        "bitcoin-core-addresses",
        &["--accept-coin-type-mismatch"],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("bc1qcr8te4k"), "{}", out(&o));
    assert!(err(&o).lines().any(|l| l == S_OK_R14), "{}", err(&o));
    // N10 through the CLI: the machine line is the last stderr line (Spec A §5).
    let m = Command::cargo_bin("mnemonic")
        .expect("binary")
        .env("MNEMONIC_NETWORK_JSON", "1")
        .args([
            "export-wallet",
            "--descriptor",
            &d14,
            "--network",
            "mainnet",
            "--format",
            "bitcoin-core-addresses",
            "--count",
            "1",
        ])
        .output()
        .expect("runs");
    assert_refused(
        &m,
        &[
            S_R14,
            r#"{"kind":"network_mismatch","class":"soft","contradictions":[{"class":"soft","text":"TX-S","a":{"what":"key @0's path m/84h/1h/0h","network":"test"},"b":{"what":"key @0","network":"main"}}]}"#,
        ],
    );

    // R1.5
    let d15 = format!("wpkh([73c5da0a/84h/0h/0h]{T}/<0;1>/*)");
    let o = ew_desc(&d15, Some("testnet"), "bitcoin-core-addresses", &[]);
    assert_refused(&o, &["error: network mismatch (coin type): key @0's path m/84h/0h/0h is a mainnet path (coin type 0h), but key @0 is a test-network key (tpub). Nothing was written. If the path is a mistake, export the key again at the test-network path (coin type 1h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for testnet and this warning is printed every time."]);
    let o = ew_desc(
        &d15,
        Some("testnet"),
        "bitcoin-core-addresses",
        &["--accept-coin-type-mismatch"],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("tb1q6rz28mc"), "{}", out(&o));
    assert!(err(&o).lines().any(|l| l == "warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: key @0's path m/84h/0h/0h is a mainnet path; the output is for testnet. A signer or recovery tool that looks only at testnet paths will not find coins sent here."), "{}", err(&o));

    // R1.6: key and run agree on mainnet; the coin disagrees with its own key.
    let o = ew_desc(&d14, None, "bitcoin-core-addresses", &[]);
    assert_refused(&o, &[S_R14]);
}

#[test]
fn nr1_export_wallet_main_key_on_regtest_refused() {
    // R1.8
    let o = ew_desc(
        &format!("wpkh({X}/<0;1>/*)"),
        Some("regtest"),
        "bitcoin-core-addresses",
        &[],
    );
    assert_refused(&o, &["error: network mismatch: key @0 is a mainnet key (xpub), but --network is regtest. Nothing was written. For a mainnet wallet, rerun with --network mainnet. For a regtest wallet, export the key again with the signer set to a test network."]);
}

#[test]
fn nr1_export_wallet_consistent_controls_unchanged() {
    // R1.9
    let o = ew_desc(
        &format!("wpkh([73c5da0a/84h/1h/0h]{T}/<0;1>/*)"),
        Some("signet"),
        "bitcoin-core-addresses",
        &[],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("tb1q6rz28mc"));
    assert!(!err(&o).contains("network mismatch"), "{}", err(&o));
    // R1.10: the key is a source, so no TX-N.
    let o = ew_desc(
        &format!("wpkh([73c5da0a/84h/0h/0h]{X}/<0;1>/*)"),
        None,
        "bitcoin-core-addresses",
        &[],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("bc1qcr8te4k"));
    assert!(!err(&o).contains("nothing here names"), "{}", err(&o));
    assert!(!err(&o).contains("network mismatch"), "{}", err(&o));
    // The flag with nothing to override: no line (N7).
    let o = ew_desc(
        &format!("wpkh([73c5da0a/84h/0h/0h]{X}/<0;1>/*)"),
        None,
        "bitcoin-core-addresses",
        &["--accept-coin-type-mismatch"],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(!err(&o).contains("network mismatch"), "{}", err(&o));
}

#[test]
fn nr1_export_wallet_mixed_keys_refused() {
    // R1.14
    let o = ew_desc(
        &format!("wsh(sortedmulti(1,[73c5da0a/48h/0h/0h/2h]{X}/<0;1>/*,[00000001/48h/1h/0h/2h]{T}/<0;1>/*))"),
        None,
        "bitcoin-core-addresses",
        &[],
    );
    assert_refused(&o, &[
        "error: network mismatch: key @0 is a mainnet key (xpub), but key @1 is a test-network key (tpub). Nothing was written. Every key of one wallet is for one network: export the odd key again.",
        "error: network mismatch: key @1 is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a mainnet wallet, export the key again with the signer set to mainnet.",
    ]);
}

#[test]
fn nr1_export_wallet_slot_key_refused_exit_8() {
    // R1.12: exactly one line (a template default path is no source, B1).
    let o = run(&[
        "export-wallet",
        "--template",
        "bip84",
        "--slot",
        &format!("@0.xpub={T}"),
        "--slot",
        "@0.fingerprint=00000000",
        "--format",
        "electrum",
        "--output",
        "-",
    ]);
    assert_refused(&o, &["error: network mismatch: --slot @0.xpub is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."]);
    // R1.20
    let o = run(&[
        "export-wallet",
        "--template",
        "bip84",
        "--slot",
        &format!("@0.xpub={X}"),
        "--slot",
        "@0.fingerprint=00000000",
        "--slot",
        &format!("@0.master_xpub={T}"),
        "--format",
        "coldcard",
        "--output",
        "-",
    ]);
    assert_refused(&o, &[
        "error: network mismatch: --slot @0.xpub is a mainnet key (xpub), but --slot @0.master_xpub is a test-network key (tpub). Nothing was written. Every key of one wallet is for one network: export the odd key again.",
        "error: network mismatch: --slot @0.master_xpub is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a mainnet wallet, export the key again with the signer set to mainnet.",
    ]);
}

#[test]
fn nr1_export_wallet_import_json_against_run_refused() {
    let ij = |file: &str, net: Option<&str>| {
        let mut a = vec!["export-wallet", "--from-import-json", file];
        if let Some(n) = net {
            a.extend(["--network", n]);
        }
        a.extend(["--format", "bitcoin-core-addresses", "--count", "1"]);
        run(&a)
    };
    // R1.13
    assert_refused(&ij("env_t.json", Some("mainnet")), &["error: network mismatch: env_t.json says network testnet, but --network is mainnet. Nothing was written. Rerun with --network testnet, or use the file made for mainnet."]);
    // R1.15
    assert_refused(&ij("env_x.json", Some("regtest")), &["error: network mismatch: env_x.json says network mainnet, but --network is regtest. Nothing was written. Rerun with --network mainnet, or use the file made for regtest."]);
    // Leg: no --network on env_t.json (D1: the run is mainnet).
    assert_refused(&ij("env_t.json", None), &["error: network mismatch: env_t.json says network testnet, but this run is on mainnet (no --network given). Nothing was written. Rerun with --network testnet, or use the file made for mainnet."]);
    // Control: the envelope's own network given.
    let o = ij("env_t.json", Some("testnet"));
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("tb1q6rz28mc"));
}

#[test]
fn nr1_export_wallet_raw_keys_no_source_tx_n_tx_t() {
    let raw = "wpkh(02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5)";
    const TX_N: &str = "warning: nothing here names a network and no --network was given: read as mainnet. For a signet, testnet or regtest wallet, rerun with --network signet, --network testnet or --network regtest.";
    // R1.16
    let o = ew_desc(raw, None, "bitcoin-core-addresses", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("bc1qq6hag6"), "{}", out(&o));
    assert!(err(&o).lines().any(|l| l == TX_N), "{}", err(&o));
    // Every format (A8): bitcoin-core too.
    let o = ew_desc(raw, None, "bitcoin-core", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(err(&o).lines().any(|l| l == TX_N), "{}", err(&o));
    // R1.17
    let o = ew_desc(raw, Some("signet"), "bitcoin-core-addresses", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("tb1qq6hag6"), "{}", out(&o));
    assert!(err(&o).lines().any(|l| l == "warning: this run is on signet, a test network, because --network says so; this descriptor names no network. A wallet holding real bitcoin is read with --network mainnet."), "{}", err(&o));
    // --network mainnet given: no line.
    let o = ew_desc(raw, Some("mainnet"), "bitcoin-core-addresses", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(!err(&o).contains("names"), "{}", err(&o));
}

#[test]
fn nr1_export_wallet_raw_key_coin_type_soft() {
    // R1.18
    let o = ew_desc(
        "wpkh([73c5da0a/84h/1h/0h]02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5)",
        None,
        "bitcoin-core-addresses",
        &[],
    );
    assert_refused(&o, &["error: network mismatch (coin type): key @0's path m/84h/1h/0h is a test-network path (coin type 1h), but this run is on mainnet (no --network given). Nothing was written. For a test-network wallet, rerun with --network signet, --network testnet or --network regtest. To use this path on mainnet as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time."]);
}

#[test]
fn nr1_export_wallet_import_json_coin_type_soft() {
    // R1.19
    let a = [
        "export-wallet",
        "--from-import-json",
        "env_x_c1.json",
        "--network",
        "mainnet",
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "1",
    ];
    assert_refused(&run(&a), &[S_R14]);
    let mut b = a.to_vec();
    b.push("--accept-coin-type-mismatch");
    let o = run(&b);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("bc1qcr8te4k"), "{}", out(&o));
    assert!(err(&o).lines().any(|l| l == S_OK_R14), "{}", err(&o));
}

#[test]
fn nr1_export_wallet_import_json_contradicts_itself() {
    // R1.21: TX-H4C only, not compared with the run.
    let o = run(&[
        "export-wallet",
        "--from-import-json",
        "env_xt.json",
        "--network",
        "testnet",
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "1",
    ]);
    assert_refused(&o, &["error: network mismatch: env_xt.json says testnet, but its bundle.descriptor key is a mainnet key (xpub). Nothing was written. No --network repairs a file that contradicts itself: export the file again from the signer with its network set for the wallet's network."]);
}
