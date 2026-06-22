//! H12 (cycle-1) — descriptor-mode taproot multisig defaults the BIP-48
//! script-type origin component to `3'` (P2TR), not `2'` (P2WSH).
//!
//! `compute_default_origin_path` (`cmd/bundle.rs`) hardcoded `2'` regardless of
//! script type, so for `tr(NUMS, multi_a/sortedmulti_a)` every origin-elided
//! cosigner key landed in the `2'` (P2WSH) subtree instead of `3'` (P2TR).
//! Result: every address diverged from what BIP-48 coordinators
//! (Sparrow/Coldcard/Jade) re-derive at `3'` → un-cosignable wallets. `3'`=P2TR
//! is a documented de-facto interop convention, confirmed by the differential
//! oracle (`bitcoind_differential.rs`).
//!
//! These tests prove: (a) the emitted per-cosigner `origin_path` is
//! `m/48'/<coin>'/<account>'/3'`; (b) the mk-decoded cosigner xpub equals the
//! INDEPENDENT `3'` derivation (obtained via an explicit `--slot @N.path=3'`
//! override), NOT the `2'` derivation; (c) the stderr default-path notice
//! renders the actual `3'` component; (d) non-taproot wsh/sh(wsh) are
//! unaffected (no spurious `3'`).

use assert_cmd::Command;
use serde_json::Value;

// Two valid 12-word BIP-39 test seeds (distinct cosigners).
const SEED_A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const SEED_B: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";

/// `bundle --network regtest --account 0 --descriptor <d> --slot @N.phrase …`
/// (+ optional per-slot extra args) → parsed `--json` Value + raw stderr.
fn bundle_taproot(descriptor: &str, extra: &[&str]) -> (Value, String) {
    let mut args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "regtest".into(),
        "--account".into(),
        "0".into(),
        "--descriptor".into(),
        descriptor.into(),
        "--slot".into(),
        format!("@0.phrase={SEED_A}"),
        "--slot".into(),
        format!("@1.phrase={SEED_B}"),
        "--no-engraving-card".into(),
        "--json".into(),
    ];
    for e in extra {
        args.push((*e).to_string());
    }
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("bundle JSON invalid: {e}\nstdout:\n{stdout}"));
    (v, stderr)
}

fn cosigner_origins(v: &Value) -> Vec<String> {
    v["multisig"]["cosigners"]
        .as_array()
        .expect("multisig.cosigners array")
        .iter()
        .map(|c| {
            c["origin_path"]
                .as_str()
                .expect("origin_path str")
                .to_string()
        })
        .collect()
}

fn cosigner_xpubs(v: &Value) -> Vec<String> {
    v["multisig"]["cosigners"]
        .as_array()
        .expect("multisig.cosigners array")
        .iter()
        .map(|c| c["xpub"].as_str().expect("xpub str").to_string())
        .collect()
}

/// Core H12 assertion: taproot multi_a defaults every origin-elided cosigner
/// to `m/48'/1'/0'/3'` (regtest coin-type 1), and the mk-decoded cosigner xpub
/// equals the INDEPENDENT `3'` derivation (NOT the `2'` derivation).
#[test]
fn taproot_multi_a_defaults_origin_to_3prime() {
    let (v, stderr) = bundle_taproot("tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))", &[]);

    let origins = cosigner_origins(&v);
    assert_eq!(
        origins,
        vec!["m/48'/1'/0'/3'", "m/48'/1'/0'/3'"],
        "taproot multi_a cosigner origins must default to the 3' (P2TR) subtree"
    );
    assert_eq!(
        v["origin_path"].as_str(),
        Some("m/48'/1'/0'/3'"),
        "top-level origin_path must also be 3'"
    );

    // Independent oracle: the SAME bundle with an explicit @N.path=3' override
    // produces the canonical `3'` xpubs. The DEFAULT (no override) xpubs must
    // equal these — proving the keys live in the 3' subtree, not just a label.
    let (v3, _) = bundle_taproot(
        "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))",
        &[
            "--slot",
            "@0.path=m/48'/1'/0'/3'",
            "--slot",
            "@1.path=m/48'/1'/0'/3'",
        ],
    );
    assert_eq!(
        cosigner_xpubs(&v),
        cosigner_xpubs(&v3),
        "default taproot cosigner xpubs must equal the explicit-3' derivation"
    );

    // …and DIFFER from the `2'` derivation (the pre-H12 wrong subtree).
    let (v2, _) = bundle_taproot(
        "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))",
        &[
            "--slot",
            "@0.path=m/48'/1'/0'/2'",
            "--slot",
            "@1.path=m/48'/1'/0'/2'",
        ],
    );
    assert_ne!(
        cosigner_xpubs(&v),
        cosigner_xpubs(&v2),
        "default taproot cosigner xpubs must NOT equal the 2' (wrong) subtree derivation"
    );

    // The stderr default-path notice renders the actual 3' component.
    assert!(
        stderr.contains("m/48'/1'/0'/3'"),
        "default-path notice must render the 3' component; got stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("m/48'/1'/0'/2' (BIP-48 cosigner path)"),
        "default-path notice must NOT render the stale 2' for a taproot descriptor; got:\n{stderr}"
    );
}

/// `tr(NUMS, sortedmulti_a(...))` likewise defaults to `3'`.
#[test]
fn taproot_sortedmulti_a_defaults_origin_to_3prime() {
    let (v, _) = bundle_taproot("tr(NUMS,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*))", &[]);
    assert_eq!(
        cosigner_origins(&v),
        vec!["m/48'/1'/0'/3'", "m/48'/1'/0'/3'"],
        "taproot sortedmulti_a cosigner origins must default to the 3' subtree"
    );
}

/// Clean-negative: a non-taproot wsh(sortedmulti) descriptor is CANONICAL —
/// its origin is elided (empty), and crucially it is NOT given a spurious `3'`
/// (the taproot detection must be specific to `Tag::Tr`).
#[test]
fn wsh_sortedmulti_not_given_3prime() {
    let (v, _) = bundle_taproot("wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))", &[]);
    for op in cosigner_origins(&v) {
        assert!(
            !op.ends_with("/3'"),
            "wsh cosigner origin must NOT be the taproot 3' subtree; got {op}"
        );
    }
}

/// Clean-negative: sh(wsh(sortedmulti)) likewise is not given a spurious `3'`.
#[test]
fn sh_wsh_sortedmulti_not_given_3prime() {
    let (v, _) = bundle_taproot("sh(wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*)))", &[]);
    for op in cosigner_origins(&v) {
        assert!(
            !op.ends_with("/3'"),
            "sh(wsh) cosigner origin must NOT be the taproot 3' subtree; got {op}"
        );
    }
}
