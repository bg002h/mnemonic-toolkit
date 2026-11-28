//! Spec B unit 8a: the toolkit remainder under the network rule (R8a.1-R8a.24,
//! TX-R, TX-V). Expected lines are literals from Spec A §3 filled in by hand;
//! every exit-8 leg asserts stdout is empty. (The import-wallet flag leg
//! `nr8a_import_wallet_coin_type_flag` is in `cli_network_rule_import_wallet`.)

use crate::network_rule_support::{assert_refused, err, fx_text, out, run, run_s, P, T, TX_N, X};

const TX_R: &str = "warning: This rehearsal used your real seeds. Everything it engraved or printed from them is as secret as your real kit: destroy it, do not discard it.";
const TPRV: &str = "tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd";
const XPRV: &str = "xprv9s21ZrQH143K3GJpoapnV8SFfukcVBSfeCficPSGfubmSFDxo1kuHnLisriDvSnRRuL2Qrg5ggqHKNVpxR86QEC8w35uxmGoggxtQTPvfUu";
/// The test-class WIF of R8a.10 and the mainnet WIF of $P at m/84h/0h/0h/0/0.
const WIF_T: &str = "cTGhosGriPpuGA586jemcuH9pE9spwUmneMBmYYzrQEbY92DJrbo";
const WIF_X: &str = "KyZpNDKnfs94vbrwhJneDi77V6jF64PWPF8x5cdJb8ifgg2DUc9d";

/// TX-H1, a test key on a main run, `<who>` and `<run>` filled in.
fn h1_test(who: &str, run: &str) -> String {
    format!("error: network mismatch: {who} is a test-network key (tpub), but {run}. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.")
}
const NOFLAG: &str = "this run is on mainnet (no --network given)";

fn stderr_has(o: &std::process::Output, line: &str) -> bool {
    err(o).lines().any(|l| l == line)
}

fn last_stderr_line(o: &std::process::Output) -> String {
    err(o).lines().last().unwrap_or("").to_string()
}

fn phrase() -> String {
    format!("phrase={P}")
}

fn slot_phrase() -> String {
    format!("@0.phrase={P}")
}

#[test]
fn nr8a_bundle_descriptor_refused() {
    // R8a.1
    let d = format!("wpkh([73c5da0a/84h/1h/0h]{T}/<0;1>/*)");
    assert_refused(
        &run(&[
            "bundle",
            "--descriptor",
            &d,
            "--network",
            "mainnet",
            "--json",
        ]),
        &[&h1_test("key @0", "--network is mainnet")],
    );
    // R8a.2
    let d = format!("wpkh([73c5da0a/84h/0h/0h]{X}/<0;1>/*)");
    assert_refused(
        &run(&["bundle", "--descriptor", &d, "--network", "testnet", "--json"]),
        &["error: network mismatch: key @0 is a mainnet key (xpub), but --network is testnet. Nothing was written. For a mainnet wallet, rerun with --network mainnet. For a testnet wallet, export the key again with the signer set to a test network."],
    );
    // A placeholder's typed origin is a coin source too (implementation
    // note: bundle --descriptor "as export-wallet --descriptor").
    let slot = format!("@0.phrase={P}");
    for d in [
        "wpkh(@0[73c5da0a/84h/0h/0h]/<0;1>/*)",
        "wpkh([73c5da0a/84h/0h/0h]@0/<0;1>/*)",
    ] {
        let o = run(&[
            "bundle",
            "--descriptor",
            d,
            "--slot",
            &slot,
            "--network",
            "signet",
            "--allow-argv-secret",
        ]);
        assert_eq!(o.status.code(), Some(8), "{}", err(&o));
        assert!(o.stdout.is_empty(), "{}", out(&o));
        assert_eq!(
            last_stderr_line(&o),
            "error: network mismatch (coin type): key @0's path m/84h/0h/0h is a mainnet path (coin type 0h), but --network is signet. Nothing was written. For a mainnet wallet, rerun with --network mainnet. To use this path on signet as it is, add --accept-coin-type-mismatch: the output is then for signet and this warning is printed every time."
        );
    }
}

#[test]
fn nr8a_bundle_slot_wif_refused() {
    // R8a.10
    let slot = format!("@0.wif={WIF_T}");
    let o = run(&[
        "bundle",
        "--template",
        "bip84",
        "--slot",
        &slot,
        "--network",
        "mainnet",
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(
        last_stderr_line(&o),
        "error: network mismatch: the WIF key is a test-network key (version byte 239), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."
    );
    // The mainnet WIF on signet and on regtest.
    let slot = format!("@0.wif={WIF_X}");
    for net in ["signet", "regtest"] {
        let o = run(&[
            "bundle",
            "--template",
            "bip84",
            "--slot",
            &slot,
            "--network",
            net,
            "--allow-argv-secret",
        ]);
        assert_eq!(o.status.code(), Some(8), "{}", err(&o));
        assert!(o.stdout.is_empty(), "{}", out(&o));
        assert_eq!(
            last_stderr_line(&o),
            format!("error: network mismatch: the WIF key is a mainnet key (version byte 128), but --network is {net}. Nothing was written. For a mainnet wallet, rerun with --network mainnet. For a {net} wallet, export the key again with the signer set to a test network.")
        );
    }
}

#[test]
fn nr8a_slot_path_coin_type_soft() {
    // R8a.11 (export-wallet, key form)
    let xs = format!("@0.xpub={T}");
    let base = [
        "export-wallet",
        "--template",
        "bip84",
        "--slot",
        &xs,
        "--slot",
        "@0.fingerprint=73c5da0a",
        "--slot",
        "@0.path=m/84h/0h/0h",
        "--network",
        "signet",
        "--format",
        "bitcoin-core-addresses",
        "--count",
        "1",
    ];
    assert_refused(
        &run(&base),
        &["error: network mismatch (coin type): the path m/84h/0h/0h given with --slot @0.path is a mainnet path (coin type 0h), but --slot @0.xpub is a test-network key (tpub). Nothing was written. If the path is a mistake, export the key again at the test-network path (coin type 1h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for signet and this warning is printed every time."],
    );
    let mut with = base.to_vec();
    with.push("--accept-coin-type-mismatch");
    let o = run(&with);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("tb1q6rz28mc"), "{}", out(&o));
    assert!(
        stderr_has(&o, "warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: the path m/84h/0h/0h given with --slot @0.path is a mainnet path; the output is for signet. A signer or recovery tool that looks only at signet paths will not find coins sent here."),
        "{}",
        err(&o)
    );
    // R8a.12 (bundle, run form)
    let sp = slot_phrase();
    let o = run(&[
        "bundle",
        "--template",
        "bip84",
        "--slot",
        &sp,
        "--slot",
        "@0.path=m/84h/0h/0h",
        "--network",
        "signet",
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(
        last_stderr_line(&o),
        "error: network mismatch (coin type): the path m/84h/0h/0h given with --slot @0.path is a mainnet path (coin type 0h), but --network is signet. Nothing was written. For a mainnet wallet, rerun with --network mainnet. To use this path on signet as it is, add --accept-coin-type-mismatch: the output is then for signet and this warning is printed every time."
    );
}

const S_PATH_0_ON_TESTNET: &str = "error: network mismatch (coin type): the path m/84h/0h/0h given with --path is a mainnet path (coin type 0h), but --network is testnet. Nothing was written. For a mainnet wallet, rerun with --network mainnet. To use this path on testnet as it is, add --accept-coin-type-mismatch: the output is then for testnet and this warning is printed every time.";

#[test]
fn nr8a_convert_path_coin_type_soft() {
    let ph = phrase();
    // R8a.3
    let base = [
        "convert",
        "--from",
        &ph,
        "--to",
        "xpub",
        "--path",
        "m/84h/0h/0h",
        "--network",
        "testnet",
        "--allow-argv-secret",
    ];
    let o = run(&base);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(last_stderr_line(&o), S_PATH_0_ON_TESTNET);
    let mut with = base.to_vec();
    with.push("--accept-coin-type-mismatch");
    let o = run(&with);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("tpubDCxX2s"), "{}", out(&o));
    assert!(
        stderr_has(&o, "warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: the path m/84h/0h/0h given with --path is a mainnet path; the output is for testnet. A signer or recovery tool that looks only at testnet paths will not find coins sent here."),
        "{}",
        err(&o)
    );
    // R8a.4
    let o = run(&[
        "convert",
        "--from",
        &ph,
        "--to",
        "xpub",
        "--path",
        "m/84h/1h/0h",
        "--network",
        "mainnet",
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(
        last_stderr_line(&o),
        "error: network mismatch (coin type): the path m/84h/1h/0h given with --path is a test-network path (coin type 1h), but --network is mainnet. Nothing was written. For a test-network wallet, rerun with --network signet, --network testnet or --network regtest. To use this path on mainnet as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time."
    );
    // R8a.17
    let base = [
        "convert",
        "--from",
        &ph,
        "--to",
        "address",
        "--script-type",
        "p2wpkh",
        "--path",
        "m/84h/1h/0h/0/0",
        "--network",
        "mainnet",
        "--allow-argv-secret",
    ];
    let o = run(&base);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(
        last_stderr_line(&o),
        "error: network mismatch (coin type): the path m/84h/1h/0h/0/0 given with --path is a test-network path (coin type 1h), but --network is mainnet. Nothing was written. For a test-network wallet, rerun with --network signet, --network testnet or --network regtest. To use this path on mainnet as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time."
    );
    let mut with = base.to_vec();
    with.push("--accept-coin-type-mismatch");
    let o = run(&with);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("bc1q6rz28mc"), "{}", out(&o));
}

#[test]
fn nr8a_convert_xpub_to_xpub_refused() {
    // R8a.5
    let f = format!("xpub={T}");
    assert_refused(
        &run(&[
            "convert",
            "--from",
            &f,
            "--to",
            "xpub",
            "--network",
            "mainnet",
        ]),
        &[&h1_test("--from xpub", "--network is mainnet")],
    );
}

#[test]
fn nr8a_convert_every_key_source_refused() {
    // R8a.13
    let f = format!("xprv={TPRV}");
    let o = run(&[
        "convert",
        "--from",
        &f,
        "--to",
        "xpub",
        "--network",
        "mainnet",
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(
        last_stderr_line(&o),
        "error: network mismatch: --from xprv is a test-network key (tprv), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."
    );
    // R8a.14
    let f = format!("xpub={T}");
    assert_refused(
        &run(&["convert", "--from", &f, "--to", "xpub"]),
        &[&h1_test("--from xpub", NOFLAG)],
    );
    // R8a.15
    let mk = format!("mk1={}", fx_text("mk_T_c1.txt").replace('\n', " "));
    assert_refused(
        &run(&["convert", "--from", &mk, "--to", "xpub"]),
        &[&h1_test("--from mk1", NOFLAG)],
    );
    // R8a.16: a target that names no network refuses too.
    assert_refused(
        &run(&["convert", "--from", &f, "--to", "fingerprint"]),
        &[&h1_test("--from xpub", NOFLAG)],
    );
}

#[test]
fn nr8a_mk1_own_coin_type_soft() {
    // R8a.24
    let mk = format!("mk1={}", fx_text("mk_X_c1.txt").replace('\n', " "));
    let s = "error: network mismatch (coin type): --from mk1's path m/84h/1h/0h is a test-network path (coin type 1h), but --from mk1 is a mainnet key (xpub). Nothing was written. If the path is a mistake, export the key again at the mainnet path (coin type 0h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time.";
    assert_refused(&run(&["convert", "--from", &mk, "--to", "xpub"]), &[s]);
    let o = run(&[
        "convert",
        "--from",
        &mk,
        "--to",
        "xpub",
        "--accept-coin-type-mismatch",
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("xpub6CatWd"), "{}", out(&o));
    assert!(
        stderr_has(&o, "warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: --from mk1's path m/84h/1h/0h is a test-network path; the output is for mainnet. A signer or recovery tool that looks only at mainnet paths will not find coins sent here."),
        "{}",
        err(&o)
    );
    // Leg: restore of a keyless multisig template with a cosigner mk1 whose
    // path coin disagrees with its key (refused before the completion runs).
    let k0 = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
    let k1 = "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk";
    let mut b: Vec<String> = [
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--md1-form",
        "template",
        "--group-size",
        "0",
        "--no-engraving-card",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for (i, (k, a)) in [(k0, 0), (k1, 1)].iter().enumerate() {
        b.extend([
            "--slot".into(),
            format!("@{i}.xpub={k}"),
            "--slot".into(),
            format!("@{i}.fingerprint=73c5da0a"),
            "--slot".into(),
            format!("@{i}.path=m/48h/0h/{a}h/2h"),
        ]);
    }
    let o = run_s(&b);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let mut r: Vec<String> = vec!["restore".into()];
    for l in out(&o).lines().filter(|l| l.starts_with("md1")) {
        r.extend(["--md1".into(), l.to_string()]);
    }
    r.extend(["--from".into(), phrase(), "--allow-argv-secret".into()]);
    for l in fx_text("mk_X_c1.txt").lines() {
        r.extend(["--cosigner".into(), format!("@1={l}")]);
    }
    let o = run_s(&r);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(
        last_stderr_line(&o),
        "error: network mismatch (coin type): --cosigner @1's mk1 card's path m/84h/1h/0h is a test-network path (coin type 1h), but --cosigner @1's mk1 card is a mainnet key (xpub). Nothing was written. If the path is a mistake, export the key again at the mainnet path (coin type 0h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time."
    );
}

#[test]
fn nr8a_restore_origin_coin_type_soft() {
    // R8a.6 (single-sig restore)
    let ph = phrase();
    let o = run(&[
        "restore",
        "--from",
        &ph,
        "--origin",
        "m/84h/0h/0h",
        "--network",
        "testnet",
        "--template",
        "bip84",
        "--allow-argv-secret",
        "--count",
        "1",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(
        last_stderr_line(&o),
        "error: network mismatch (coin type): the path m/84h/0h/0h given with --origin is a mainnet path (coin type 0h), but --network is testnet. Nothing was written. For a mainnet wallet, rerun with --network mainnet. To use this path on testnet as it is, add --accept-coin-type-mismatch: the output is then for testnet and this warning is printed every time."
    );
    // Template leg: a keyless single-sig template md1 with --origin.
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--md1-form",
        "template",
        "--slot",
        &format!("@0.xpub={X}"),
        "--slot",
        "@0.fingerprint=73c5da0a",
        "--slot",
        "@0.path=m/84h/0h/0h",
        "--no-engraving-card",
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let mut r: Vec<String> = vec!["restore".into()];
    for l in out(&o).lines().filter(|l| l.starts_with("md1")) {
        r.extend(["--md1".into(), l.to_string()]);
    }
    r.extend(
        [
            "--from",
            &ph,
            "--allow-argv-secret",
            "--origin",
            "m/84h/1h/0h",
            "--count",
            "1",
        ]
        .iter()
        .map(|s| s.to_string()),
    );
    let o = run_s(&r);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(
        last_stderr_line(&o),
        "error: network mismatch (coin type): the path m/84h/1h/0h given with --origin is a test-network path (coin type 1h), but this run is on mainnet (no --network given). Nothing was written. For a test-network wallet, rerun with --network signet, --network testnet or --network regtest. To use this path on mainnet as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time."
    );
}

#[test]
fn nr8a_restore_expect_xpub_network_first() {
    let ph = phrase();
    let h1 = h1_test("--expect-xpub", NOFLAG);
    // R8a.22
    let o = run(&[
        "restore",
        "--from",
        &ph,
        "--template",
        "bip84",
        "--expect-xpub",
        T,
        "--allow-argv-secret",
        "--count",
        "1",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(last_stderr_line(&o), h1);
    assert!(!err(&o).contains("MISMATCH"), "{}", err(&o));
    // R8a.23: --allow-mismatch does not pass it.
    let o = run(&[
        "restore",
        "--from",
        &ph,
        "--template",
        "bip84",
        "--expect-xpub",
        T,
        "--allow-mismatch",
        "--allow-argv-secret",
        "--count",
        "1",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(last_stderr_line(&o), h1);
}

#[test]
fn nr8a_xpub_search_refuses_before_search() {
    // R8a.7
    let o = run(&[
        "xpub-search",
        "path-of-xpub",
        "--phrase",
        P,
        "--target-xpub",
        T,
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(last_stderr_line(&o), h1_test("--target-xpub", NOFLAG));
    // passphrase-of-xpub, same arm.
    let o = run(&[
        "xpub-search",
        "passphrase-of-xpub",
        "--phrase",
        P,
        "--target-xpub",
        T,
        "--passphrase",
        "x",
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(last_stderr_line(&o), h1_test("--target-xpub", NOFLAG));
    // R8a.8
    assert_refused(
        &run(&[
            "xpub-search", "address-of-xpub", "--xpub", X, "--target-address",
            "tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl", "--address-type", "p2wpkh",
        ]),
        &["error: network mismatch: --target-address tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl is a signet or testnet address, but this run is on mainnet (no --network given). Nothing was written. Give an address of mainnet."],
    );
}

#[test]
fn nr8a_xpub_search_account_of_descriptor_key_refused() {
    // R8a.20
    let d = format!("wpkh([73c5da0a/84h/1h/0h]{T}/<0;1>/*)");
    let o = run(&[
        "xpub-search",
        "account-of-descriptor",
        "--phrase",
        P,
        "--descriptor",
        &d,
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(last_stderr_line(&o), h1_test("key @0", NOFLAG));
    // --descriptor-from too.
    let lit = format!("literal={d}");
    let o = run(&[
        "xpub-search",
        "account-of-descriptor",
        "--phrase",
        P,
        "--descriptor-from",
        &lit,
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(last_stderr_line(&o), h1_test("key @0", NOFLAG));
}

#[test]
fn nr8a_nostr_pubkey_prints_tx_n() {
    // R8a.9
    let o = run(&[
        "nostr",
        "--pubkey",
        "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("bc1pmfr3p9j"), "{}", out(&o));
    assert_eq!(last_stderr_line(&o), TX_N);
    // A test flag prints TX-T.
    let o = run(&[
        "nostr",
        "--pubkey",
        "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
        "--network",
        "signet",
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(last_stderr_line(&o), "warning: this run is on signet, a test network, because --network says so; this key names no network. A wallet holding real bitcoin is read with --network mainnet.");
    // --network mainnet: no line.
    let o = run(&[
        "nostr",
        "--pubkey",
        "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
        "--network",
        "mainnet",
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(!err(&o).contains("names no network"), "{}", err(&o));
    assert!(!err(&o).contains("names a network"), "{}", err(&o));
}

const NOSTR_SECRET: &str = "0000000000000000000000000000000000000000000000000000000000000001";

#[test]
fn nr8a_nostr_secret_prints_tx_n() {
    let o = run(&["nostr", "--secret", NOSTR_SECRET, "--allow-argv-secret"]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    // The key advisory, then TX-N, last; never TX-R.
    assert_eq!(last_stderr_line(&o), TX_N);
    let o = run(&[
        "nostr",
        "--secret",
        NOSTR_SECRET,
        "--allow-argv-secret",
        "--network",
        "testnet",
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(last_stderr_line(&o), "warning: this run is on testnet, a test network, because --network says so; this key names no network. A wallet holding real bitcoin is read with --network mainnet.");
    assert!(!err(&o).contains("rehearsal"), "{}", err(&o));
}

#[test]
fn nr8a_derive_child_tprv_no_flag_refused() {
    // R8a.18
    let f = format!("xprv={XPRV}");
    let o = run(&[
        "derive-child",
        "--from",
        &f,
        "--application",
        "xprv",
        "--length",
        "0",
        "--index",
        "0",
        "--network",
        "testnet",
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(
        last_stderr_line(&o),
        "error: network mismatch: --from xprv is a mainnet key (xprv), but --network is testnet. Nothing was written. For a mainnet wallet, rerun with --network mainnet. For a testnet wallet, export the key again with the signer set to a test network."
    );
    // R8a.19
    let f = format!("xprv={TPRV}");
    let o = run(&[
        "derive-child",
        "--from",
        &f,
        "--application",
        "hex",
        "--length",
        "16",
        "--index",
        "0",
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(
        last_stderr_line(&o),
        "error: network mismatch: --from xprv is a test-network key (tprv), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."
    );
}

#[test]
fn nr8a_tx_r_on_secret_output_test_run() {
    // R8a.21 (convert)
    let ph = phrase();
    let o = run(&[
        "convert",
        "--from",
        &ph,
        "--to",
        "xprv",
        "--path",
        "m",
        "--network",
        "testnet",
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("tprv8Zgx"), "{}", out(&o));
    assert_eq!(last_stderr_line(&o), TX_R);
    // The same on mainnet prints no TX-R.
    let o = run(&[
        "convert",
        "--from",
        &ph,
        "--to",
        "xprv",
        "--path",
        "m",
        "--network",
        "mainnet",
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(!err(&o).contains(TX_R), "{}", err(&o));
    // derive-child
    let f = format!("xprv={TPRV}");
    let o = run(&[
        "derive-child",
        "--from",
        &f,
        "--application",
        "xprv",
        "--length",
        "0",
        "--index",
        "0",
        "--network",
        "testnet",
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(last_stderr_line(&o), TX_R);
    // bundle
    let sp = slot_phrase();
    let o = run(&[
        "bundle",
        "--template",
        "bip84",
        "--slot",
        &sp,
        "--network",
        "signet",
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(last_stderr_line(&o), TX_R);
    // silent-payment
    let o = run(&[
        "silent-payment",
        "--secret",
        P,
        "--network",
        "signet",
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert_eq!(last_stderr_line(&o), TX_R);
}

/// `verify-bundle --template bip84 --slot @0.phrase=$P` over a bundle file's
/// mk1/md1 lines, plus `extra`.
fn verify(file: &str, extra: &[&str]) -> std::process::Output {
    let text = fx_text(file);
    let mut a: Vec<String> = [
        "verify-bundle",
        "--allow-argv-secret",
        "--template",
        "bip84",
        "--slot",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    a.push(slot_phrase());
    for l in text.lines().filter(|l| l.starts_with("mk1")) {
        a.extend(["--mk1".into(), l.to_string()]);
    }
    for l in text.lines().filter(|l| l.starts_with("md1")) {
        a.extend(["--md1".into(), l.to_string()]);
    }
    a.extend(extra.iter().map(|s| s.to_string()));
    run_s(&a)
}

fn ms1_of_signet_bundle() -> String {
    fx_text("vb_signet_bip84.txt")
        .lines()
        .find(|l| l.starts_with("ms1"))
        .expect("ms1 line")
        .to_string()
}

const TX_V_MAINNET: &str = "note: every mismatch above is a coin-type or key-version row: the bundle may be for another network than --network mainnet; rerun with the network the bundle was made for.";

#[test]
fn nr8a_verify_bundle_network_refused_exit_8() {
    let ms1 = ms1_of_signet_bundle();
    let o = verify(
        "vb_signet_bip84.txt",
        &["--network", "mainnet", "--ms1", &ms1],
    );
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(
        last_stderr_line(&o),
        "error: network mismatch: the mk1 card is a test-network key (tpub), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."
    );
}

#[test]
fn nr8a_verify_bundle_cause_line() {
    // Positive: coin 5 is no source, the key is a mainnet xpub; exactly the
    // four network rows fail, then TX-V; exit 4 unchanged.
    let ms1 = ms1_of_signet_bundle();
    let o = verify(
        "vb_mainnet_coin5.txt",
        &["--network", "mainnet", "--ms1", &ms1],
    );
    assert_eq!(o.status.code(), Some(4), "{}", err(&o));
    let s = out(&o);
    let failed: Vec<&str> = s
        .lines()
        .filter(|l| l.contains(": fail"))
        .map(|l| l.split(':').next().unwrap())
        .collect();
    assert_eq!(
        failed,
        [
            "mk1_xpub_match",
            "mk1_path_match",
            "md1_xpub_match",
            "md1_origin_match"
        ],
        "{s}"
    );
    assert!(out(&o).contains("result: mismatch"), "{}", out(&o));
    assert_eq!(last_stderr_line(&o), TX_V_MAINNET);
    // Negative: only ms1_decode fails (no --ms1): no TX-V.
    let o = verify("vb_signet_bip84.txt", &["--network", "signet"]);
    assert_eq!(o.status.code(), Some(4), "{}", err(&o));
    assert!(
        !err(&o).contains("note: every mismatch above"),
        "{}",
        err(&o)
    );
    // The consistent run: exit 0, no TX-V.
    let o = verify(
        "vb_signet_bip84.txt",
        &["--network", "signet", "--ms1", &ms1],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(
        !err(&o).contains("note: every mismatch above"),
        "{}",
        err(&o)
    );
}

#[test]
fn nr8a_bundle_import_json_contradicts_itself() {
    // Plan unit 8a (I-7): the envelope's TX-H4C and TX-S at bundle
    // --import-json, as export-wallet --from-import-json (R1.19, R1.21).
    assert_refused(
        &run(&["bundle", "--import-json", "env_xt.json", "--network", "testnet"]),
        &["error: network mismatch: env_xt.json says testnet, but its bundle.descriptor key is a mainnet key (xpub). Nothing was written. No --network repairs a file that contradicts itself: export the file again from the signer with its network set for the wallet's network."],
    );
    let a = [
        "bundle",
        "--import-json",
        "env_x_c1.json",
        "--network",
        "mainnet",
    ];
    assert_refused(
        &run(&a),
        &["error: network mismatch (coin type): key @0's path m/84h/1h/0h is a test-network path (coin type 1h), but key @0 is a mainnet key (xpub). Nothing was written. If the path is a mistake, export the key again at the mainnet path (coin type 0h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time."],
    );
    let mut b = a.to_vec();
    b.push("--accept-coin-type-mismatch");
    let o = run(&b);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("mk1"), "{}", out(&o));
    assert!(
        stderr_has(&o, "warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: key @0's path m/84h/1h/0h is a test-network path; the output is for mainnet. A signer or recovery tool that looks only at mainnet paths will not find coins sent here."),
        "{}",
        err(&o)
    );
}

#[test]
fn nr8b_inferring_arms_refuse_without_flag() {
    // R8b.1
    let from = format!("xpub={T}");
    assert_refused(
        &run(&[
            "addresses",
            "--from",
            &from,
            "--count",
            "1",
            "--address-type",
            "p2wpkh",
        ]),
        &[&h1_test("--from xpub", NOFLAG)],
    );
    // R8b.4
    assert_refused(
        &run(&[
            "convert",
            "--from",
            &from,
            "--to",
            "address",
            "--script-type",
            "p2wpkh",
            "--path",
            "m/0/0",
        ]),
        &[&h1_test("--from xpub", NOFLAG)],
    );
    // A test key on its own network is unchanged.
    let o = run(&[
        "addresses",
        "--from",
        &from,
        "--count",
        "1",
        "--address-type",
        "p2wpkh",
        "--network",
        "testnet",
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("tb1q6rz28mc"), "{}", out(&o));
}

#[test]
fn nr8b_existing_refusals_exit_8_one_wording() {
    const MAIN: &str = "--network is mainnet";
    let from = format!("xpub={T}");
    // R8b.2: addresses with --network (was exit 1).
    assert_refused(
        &run(&[
            "addresses",
            "--from",
            &from,
            "--count",
            "1",
            "--address-type",
            "p2wpkh",
            "--network",
            "mainnet",
        ]),
        &[&h1_test("--from xpub", MAIN)],
    );
    // convert --xpub-prefix (was exit 2).
    assert_refused(
        &run(&[
            "convert",
            "--from",
            &from,
            "--to",
            "xpub",
            "--xpub-prefix",
            "zpub",
            "--network",
            "mainnet",
        ]),
        &[&h1_test("--from xpub", MAIN)],
    );
    // convert wif->xpub (was exit 2): TX-H5.
    let wif = format!("wif={WIF_T}");
    let o = run(&[
        "convert",
        "--from",
        &wif,
        "--to",
        "xpub",
        "--network",
        "mainnet",
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(
        last_stderr_line(&o),
        "error: network mismatch: the WIF key is a test-network key (version byte 239), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."
    );
    // bundle --slot @N.xpub= (was CosignerSpec, exit 1).
    let slot = format!("@0.xpub={T}");
    assert_refused(
        &run(&[
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            &slot,
            "--slot",
            "@0.fingerprint=73c5da0a",
        ]),
        &[&h1_test("--slot @0.xpub", MAIN)],
    );
    // bundle --import-json (was BadInput, exit 1): TX-H4.
    assert_refused(
        &run(&["bundle", "--import-json", "env_t.json", "--network", "mainnet"]),
        &["error: network mismatch: env_t.json says network testnet, but --network is mainnet. Nothing was written. Rerun with --network testnet, or use the file made for mainnet."],
    );
    // silent-payment (was exit 2).
    let o = run(&[
        "silent-payment",
        "--secret",
        TPRV,
        "--network",
        "mainnet",
        "--allow-argv-secret",
    ]);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(
        last_stderr_line(&o),
        "error: network mismatch: --secret is a test-network key (tprv), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."
    );
    // verify-bundle watch-only slot (was exit 1): merged with the mk1 card
    // of the same class (B5).
    let text = fx_text("vb_signet_bip84.txt");
    let mut a: Vec<String> = [
        "verify-bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        &slot,
        "--slot",
        "@0.fingerprint=73c5da0a",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for l in text
        .lines()
        .filter(|l| l.starts_with("mk1") || l.starts_with("md1"))
    {
        a.extend([format!("--{}", &l[..3]), l.to_string()]);
    }
    assert_refused(
        &run_s(&a),
        &["error: network mismatch: the mk1 card and --slot @0.xpub are test-network keys (tpub), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."],
    );
    // restore --search-address, multisig template completion (was exit 1):
    // TX-H3.
    let k0 = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
    let k1 = "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk";
    let mut b: Vec<String> = [
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--md1-form",
        "template",
        "--group-size",
        "0",
        "--no-engraving-card",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for (i, k) in [k0, k1].iter().enumerate() {
        b.extend([
            "--slot".into(),
            format!("@{i}.xpub={k}"),
            "--slot".into(),
            format!("@{i}.fingerprint=73c5da0a"),
            "--slot".into(),
            format!("@{i}.path=m/48h/0h/{i}h/2h"),
        ]);
    }
    let o = run_s(&b);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let mut r: Vec<String> = vec!["restore".into()];
    for l in out(&o).lines().filter(|l| l.starts_with("md1")) {
        r.extend(["--md1".into(), l.to_string()]);
    }
    r.extend([
        "--from".into(),
        phrase(),
        "--allow-argv-secret".into(),
        "--search-address".into(),
        "tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl".into(),
    ]);
    let o = run_s(&r);
    assert_eq!(o.status.code(), Some(8), "{}", err(&o));
    assert!(o.stdout.is_empty(), "{}", out(&o));
    assert_eq!(
        last_stderr_line(&o),
        "error: network mismatch: --search-address tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl is a signet or testnet address, but this run is on mainnet (no --network given). Nothing was written. Rerun with --network signet or --network testnet, or give an address of mainnet."
    );
}
