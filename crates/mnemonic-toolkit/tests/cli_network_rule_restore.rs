//! Spec B unit 2: restore `--md1` under the network rule (R2.1-R2.9).
//! Expected lines are literals from Spec A §3 filled in by hand; every exit-8
//! leg asserts stdout is empty.

use crate::network_rule_support::{assert_refused, cos, err, fx_text, mdargs, out, run_s, P, TX_N};

fn restore(md: &str, extra: &[&str]) -> std::process::Output {
    let mut a = vec!["restore".to_string()];
    a.extend(mdargs(md));
    a.extend(extra.iter().map(|s| s.to_string()));
    a.extend(["--count".to_string(), "1".to_string()]);
    run_s(&a)
}

fn restore_cos(md: &str, cosigners: Vec<String>, extra: &[&str]) -> std::process::Output {
    let mut a = vec!["restore".to_string()];
    a.extend(mdargs(md));
    a.extend(cosigners);
    a.extend(extra.iter().map(|s| s.to_string()));
    a.extend(["--count".to_string(), "1".to_string()]);
    run_s(&a)
}

fn signet_cards() -> Vec<String> {
    let mut c = cos("m_signet.txt", 8, 9, 0);
    c.extend(cos("m_signet.txt", 12, 13, 1));
    c
}

fn mainnet_cards() -> Vec<String> {
    let mut c = cos("m_mainnet.txt", 8, 9, 0);
    c.extend(cos("m_mainnet.txt", 12, 13, 1));
    c
}

const S_SIGNET_NOFLAG: &str = "error: network mismatch (coin type): keys @0 and @1's paths m/87h/1h/0h and m/87h/1h/0h are test-network paths (coin type 1h), but this run is on mainnet (no --network given). Nothing was written. For a test-network wallet, rerun with --network signet, --network testnet or --network regtest. To use this path on mainnet as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time.";

#[test]
fn nr2_restore_md1_coin_type_against_run_refused() {
    // R2.1
    assert_refused(&restore("m_signet.txt", &[]), &[S_SIGNET_NOFLAG]);
    // R2.2
    assert_refused(&restore("m_signet.txt", &["--network", "mainnet"]), &["error: network mismatch (coin type): keys @0 and @1's paths m/87h/1h/0h and m/87h/1h/0h are test-network paths (coin type 1h), but --network is mainnet. Nothing was written. For a test-network wallet, rerun with --network signet, --network testnet or --network regtest. To use this path on mainnet as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time."]);
    let o = restore(
        "m_signet.txt",
        &["--network", "mainnet", "--accept-coin-type-mismatch"],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("bc1qvqnan"), "{}", out(&o));
    assert!(err(&o).lines().any(|l| l == "warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: keys @0 and @1's paths m/87h/1h/0h and m/87h/1h/0h are test-network paths; the output is for mainnet. A signer or recovery tool that looks only at mainnet paths will not find coins sent here."), "{}", err(&o));
    // R2.3
    assert_refused(&restore("m_mainnet.txt", &["--network", "testnet"]), &["error: network mismatch (coin type): keys @0 and @1's paths m/87h/0h/0h and m/87h/0h/0h are mainnet paths (coin type 0h), but --network is testnet. Nothing was written. For a mainnet wallet, rerun with --network mainnet. To use this path on testnet as it is, add --accept-coin-type-mismatch: the output is then for testnet and this warning is printed every time."]);
    // R2.9: a seed typed now does not exempt the md1's coin types.
    let from = format!("phrase={P}");
    assert_refused(
        &restore("m_signet.txt", &["--from", &from, "--allow-argv-secret"]),
        &[S_SIGNET_NOFLAG],
    );
}

#[test]
fn nr2_restore_md1_cosigner_version_refused_flag_ignored() {
    // R2.4: TX-H1 only (both coins agree with their tpub key, named on the
    // HARD line); the flag does not pass it.
    let h1 = "error: network mismatch: --cosigner @0's and @1's mk1 cards are test-network keys (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.";
    assert_refused(&restore_cos("m_signet.txt", signet_cards(), &[]), &[h1]);
    assert_refused(
        &restore_cos(
            "m_signet.txt",
            signet_cards(),
            &["--accept-coin-type-mismatch"],
        ),
        &[h1],
    );
    // R2.7
    assert_refused(
        &restore_cos("m_mainnet.txt", mainnet_cards(), &["--network", "signet"]),
        &["error: network mismatch: --cosigner @0's and @1's mk1 cards are mainnet keys (xpub), but --network is signet. Nothing was written. For a mainnet wallet, rerun with --network mainnet. For a signet wallet, export the key again with the signer set to a test network."],
    );
    // SOFT only (I-10b): xpub cosigners on the run's class, the md1 slots at
    // coin 1: one merged TX-S key form; the flag lets it through.
    let x = fx_text("m_signet_xcos.txt");
    let xl: Vec<&str> = x.lines().collect();
    let xcos = vec![
        "--cosigner".to_string(),
        format!("@0={}", xl[0]),
        "--cosigner".to_string(),
        format!("@1={}", xl[1]),
    ];
    assert_refused(&restore_cos("m_signet.txt", xcos.clone(), &[]), &["error: network mismatch (coin type): keys @0 and @1's paths m/87h/1h/0h and m/87h/1h/0h are test-network paths (coin type 1h), but --cosigner @0's and @1's xpubs are mainnet keys (xpub). Nothing was written. If the path is a mistake, make the md1 card again at the mainnet path (coin type 0h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time."]);
    let o = restore_cos("m_signet.txt", xcos, &["--accept-coin-type-mismatch"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("cross-checked"), "{}", out(&o));
    assert!(err(&o).lines().any(|l| l == "warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: keys @0 and @1's paths m/87h/1h/0h and m/87h/1h/0h are test-network paths; the output is for mainnet. A signer or recovery tool that looks only at mainnet paths will not find coins sent here."), "{}", err(&o));
}

#[test]
fn nr2_restore_md1_controls_unchanged() {
    // R2.5
    let o = restore("m_signet.txt", &["--network", "signet"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("tb1qvqnan"), "{}", out(&o));
    assert!(!err(&o).contains("network mismatch"), "{}", err(&o));
    // R2.6
    let o = restore("m_mainnet.txt", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("bc1q7cty6"), "{}", out(&o));
    assert!(!err(&o).contains("network mismatch"), "{}", err(&o));
    // R2.8
    let o = restore_cos("m_signet.txt", signet_cards(), &["--network", "signet"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("cross-checked"), "{}", out(&o));
    assert!(!err(&o).contains("network mismatch"), "{}", err(&o));
}

#[test]
fn nr2_restore_md1_no_source_tx_n_tx_t() {
    // A keyed md1 at m/45h: no N1 coin, no key version.
    let o = restore("m_bip45.txt", &[]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(err(&o).lines().any(|l| l == TX_N), "{}", err(&o));
    let o = restore("m_bip45.txt", &["--network", "signet"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(err(&o).lines().any(|l| l == "warning: this run is on signet, a test network, because --network says so; this card names no network. A wallet holding real bitcoin is read with --network mainnet."), "{}", err(&o));
    assert!(!err(&o).contains(TX_N), "{}", err(&o));
}
