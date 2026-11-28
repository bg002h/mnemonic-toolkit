//! Spec B unit 3: import-wallet under the network rule (R3.1-R3.10), and the
//! unit-8a coin-type flag leg (`nr8a_import_wallet_coin_type_flag`).
//! Expected lines are literals from Spec A §3 filled in by hand; every exit-8
//! leg asserts stdout is empty.

use crate::network_rule_support::{assert_refused, err, out, run};

fn import(blob: &str, extra: &[&str]) -> std::process::Output {
    let mut a = vec!["import-wallet", "--blob", blob, "--json"];
    a.extend_from_slice(extra);
    run(&a)
}

const CC_BTC_T: &str = "says chain BTC (mainnet), but its bip84 key is a test-network key (tpub). Nothing was written. No --network repairs a file that contradicts itself: export the file again from the Coldcard with its network set for the wallet's network.";

#[test]
fn nr3_coldcard_chain_against_key_refused() {
    // R3.1
    assert_refused(
        &import("cc_BTC_T_1.json", &[]),
        &[&format!(
            "error: network mismatch: cc_BTC_T_1.json {CC_BTC_T}"
        )],
    );
    // R3.2: TX-H4C only, no TX-S for the coin-0h path.
    assert_refused(
        &import("cc_BTC_T_0.json", &[]),
        &[&format!(
            "error: network mismatch: cc_BTC_T_0.json {CC_BTC_T}"
        )],
    );
    // R3.3
    assert_refused(
        &import("cc_XTN_X_0.json", &[]),
        &["error: network mismatch: cc_XTN_X_0.json says chain XTN (a test network), but its bip84 key is a mainnet key (xpub). Nothing was written. No --network repairs a file that contradicts itself: export the file again from the Coldcard with its network set for the wallet's network."],
    );
    // R3.5: the flag never repairs it.
    assert_refused(
        &import("cc_BTC_T_1.json", &["--network", "testnet"]),
        &[&format!(
            "error: network mismatch: cc_BTC_T_1.json {CC_BTC_T}"
        )],
    );
    assert_refused(
        &import(
            "cc_BTC_T_1.json",
            &["--network", "testnet", "--accept-coin-type-mismatch"],
        ),
        &[&format!(
            "error: network mismatch: cc_BTC_T_1.json {CC_BTC_T}"
        )],
    );
}

#[test]
fn nr3_coldcard_consistent_unchanged() {
    // R3.4
    let o = import("cc_BTC_X_0.json", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("\"network\": \"mainnet\""), "{}", out(&o));
    assert!(!err(&o).contains("network mismatch"), "{}", err(&o));
}

const JADE_H1: &str = "error: network mismatch: jade keys @0, @1 and @2 are test-network keys (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.";

#[test]
fn nr3_jade_names_jade_and_exits_8() {
    // R3.6
    assert_refused(
        &import("jade_T_0.json", &[]),
        &[
            JADE_H1,
            "error: network mismatch (coin type): jade keys @0, @1 and @2's paths m/48h/0h/0h/2h, m/48h/0h/0h/2h and m/48h/0h/0h/2h are mainnet paths (coin type 0h), but jade keys @0, @1 and @2 are test-network keys (tpub). Nothing was written. If the path is a mistake, export the key again at the test-network path (coin type 1h).",
        ],
    );
    // R3.7: SOFT only; with the flag 0.
    assert_refused(
        &import("jade_X_1.json", &[]),
        &["error: network mismatch (coin type): jade keys @0, @1 and @2's paths m/48h/1h/0h/2h, m/48h/1h/0h/2h and m/48h/1h/0h/2h are test-network paths (coin type 1h), but jade keys @0, @1 and @2 are mainnet keys (xpub). Nothing was written. If the path is a mistake, export the key again at the mainnet path (coin type 0h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time."],
    );
    let o = import("jade_X_1.json", &["--accept-coin-type-mismatch"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("\"network\": \"mainnet\""), "{}", out(&o));
    assert!(
        err(&o).lines().any(|l| l == "warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: jade keys @0, @1 and @2's paths m/48h/1h/0h/2h, m/48h/1h/0h/2h and m/48h/1h/0h/2h are test-network paths; the output is for mainnet. A signer or recovery tool that looks only at mainnet paths will not find coins sent here."),
        "{}",
        err(&o)
    );
    // R3.8: TX-H2, TX-H1 without its rerun sentence, TX-S without its flag
    // sentence.
    assert_refused(
        &import("jade_mixed_0.json", &[]),
        &[
            "error: network mismatch: jade key @0 is a mainnet key (xpub), but jade key @1 is a test-network key (tpub). Nothing was written. Every key of one wallet is for one network: export the odd key again.",
            "error: network mismatch: jade key @1 is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a mainnet wallet, export the key again with the signer set to mainnet.",
            "error: network mismatch (coin type): jade key @1's path m/48h/0h/0h/2h is a mainnet path (coin type 0h), but jade key @1 is a test-network key (tpub). Nothing was written. If the path is a mistake, export the key again at the test-network path (coin type 1h).",
        ],
    );
}

#[test]
fn nr3_import_test_blob_no_flag_refused() {
    // R3.9
    assert_refused(&import("jade_T_1.json", &[]), &[JADE_H1]);
    // A test blob on its own network is unchanged.
    let o = import("jade_T_1.json", &["--network", "testnet"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("\"network\": \"testnet\""), "{}", out(&o));
    assert!(!err(&o).contains("network mismatch"), "{}", err(&o));
    // R3.10
    assert_refused(
        &import("ccms_T_0.txt", &[]),
        &[
            "error: network mismatch: coldcard-multisig keys @0, @1 and @2 are test-network keys (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.",
            "error: network mismatch (coin type): coldcard-multisig keys @0, @1 and @2's paths m/48h/0h/0h/2h, m/48h/0h/0h/2h and m/48h/0h/0h/2h are mainnet paths (coin type 0h), but coldcard-multisig keys @0, @1 and @2 are test-network keys (tpub). Nothing was written. If the path is a mistake, export the key again at the test-network path (coin type 1h).",
        ],
    );
}

#[test]
fn nr8a_import_wallet_coin_type_flag() {
    // d_x1.txt: a mainnet key at a coin-1 path (SOFT, key form).
    let s_x1 = "error: network mismatch (coin type): key @0's path m/84h/1h/0h is a test-network path (coin type 1h), but key @0 is a mainnet key (xpub). Nothing was written. If the path is a mistake, export the key again at the mainnet path (coin type 0h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time.";
    assert_refused(&import("d_x1.txt", &["--format", "descriptor"]), &[s_x1]);
    let o = import(
        "d_x1.txt",
        &["--format", "descriptor", "--accept-coin-type-mismatch"],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(
        err(&o).lines().any(|l| l == "warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: key @0's path m/84h/1h/0h is a test-network path; the output is for mainnet. A signer or recovery tool that looks only at mainnet paths will not find coins sent here."),
        "{}",
        err(&o)
    );
    // d_mixcoin.txt: two mainnet keys, one at coin 0 and one at coin 1. The
    // parser no longer refuses the mixed coin types; the coin-1 key prints
    // TX-S (I-5).
    assert_refused(
        &import("d_mixcoin.txt", &["--format", "descriptor"]),
        &["error: network mismatch (coin type): key @1's path m/48h/1h/0h/2h is a test-network path (coin type 1h), but key @1 is a mainnet key (xpub). Nothing was written. If the path is a mistake, export the key again at the mainnet path (coin type 0h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time."],
    );
    let o = import(
        "d_mixcoin.txt",
        &["--format", "descriptor", "--accept-coin-type-mismatch"],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(
        err(&o).lines().any(|l| l == "warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: key @1's path m/48h/1h/0h/2h is a test-network path; the output is for mainnet. A signer or recovery tool that looks only at mainnet paths will not find coins sent here."),
        "{}",
        err(&o)
    );
}
