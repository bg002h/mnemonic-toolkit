//! GAP 1-T1 — taproot restore-refusal contracts.
//!
//! `bundle --descriptor` emits a FAITHFUL md1 for taproot policies BEYOND
//! single-leaf NUMS `multi_a`/`sortedmulti_a`, but `restore --md1` reconstructs
//! only those two — every other `Tag::Tr` md1 hits a refusal arm in
//! `src/cmd/restore.rs::taproot_template_and_internal_key`. That is the
//! engrave-but-can't-mechanically-restore class: the wire round-trips byte-exact
//! (md-codec), so the steel card is a faithful backup recoverable by a future
//! toolkit / a human — only the mechanical reconstruction lags.
//!
//! These cells pin the THREE bundle-reachable refusal arms (restore.rs:689 /
//! :710) + the wire faithfulness (`.descriptor` round-trips exactly). Pinning
//! the refusal CONTRACT keeps a future restore-walker change from silently
//! turning a clean refusal into a wrong reconstruction or a panic.
//!
//! FOLLOWUP `restore-general-and-multi-leaf-taproot-roundtrip`: T1 = these
//! contracts; T3 = faithful reconstruction + the `tree:None` keypath-only arm
//! (the only one needing a direct wire fixture). NO-BUMP (test-only).

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

// The 3-cosigner trio, lifted from `cli_bundle_import_json.rs:312-314`: three
// distinct fingerprints, three distinct xpubs, all `[fp/87'/0'/0']…/<0;1>/*`
// watch-only (concrete `bundle --descriptor`, no seed).
const K0: &str = "[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*";
const K1: &str = "[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*";
const K2: &str = "[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*";

/// `bundle --descriptor <desc> --network mainnet --json` → (md1 chunks, emitted `.descriptor`).
fn bundle_md1(desc: &str) -> (Vec<String>, String) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            desc,
            "--network",
            "mainnet",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).expect("bundle --json output");
    let chunks: Vec<String> = v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|c| c.as_str().expect("md1 chunk str").to_string())
        .collect();
    let descriptor = v["descriptor"]
        .as_str()
        .expect("descriptor str")
        .to_string();
    (chunks, descriptor)
}

/// `restore --network mainnet --md1 <chunk>` per chunk (mirrors `cli_restore_multisig.rs`).
fn restore_args(md1: &[String]) -> Vec<String> {
    let mut a = vec!["restore".to_string(), "--network".into(), "mainnet".into()];
    for c in md1 {
        a.push("--md1".into());
        a.push(c.clone());
    }
    a
}

/// (1) General taproot leaf `tr(NUMS, <non-multisig miniscript>)`: bundle emits a
/// faithful card (`.descriptor` round-trips EXACTLY — the literal `NUMS` token is
/// preserved on the wire, no substitution), but restore refuses with exit 2
/// "not a recognized multisig" (restore.rs:710).
#[test]
fn general_tr_leaf_bundles_faithfully_but_restore_refuses() {
    let desc = format!("tr(NUMS,and_v(v:pk({K0}),after(12000000)))");
    let (md1, emitted) = bundle_md1(&desc);
    assert!(!md1.is_empty(), "card must be emitted (faithful backup)");
    assert_eq!(
        emitted, desc,
        "the emitted descriptor must round-trip EXACTLY (literal NUMS preserved)"
    );
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("not a recognized multisig"));
}

/// (2) Multi-leaf taptree `tr(NUMS,{leaf_a,leaf_b})`: bundle emits faithfully
/// (the wire round-trips, depth-1 / Display-safe), restore refuses (same arm —
/// a `TapTree`-tagged inner is not multi_a/sortedmulti_a). The GAP-1 sub-4
/// engrave-but-can't-restore contract incl. the wire-faithfulness leg.
#[test]
fn multi_leaf_taptree_bundles_faithfully_but_restore_refuses() {
    let desc = format!("tr(NUMS,{{pk({K0}),pk({K1})}})");
    let (md1, emitted) = bundle_md1(&desc);
    assert!(!md1.is_empty(), "multi-leaf card must be emitted");
    assert_eq!(
        emitted, desc,
        "multi-leaf descriptor must round-trip EXACTLY"
    );
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("not a recognized multisig"));
}

/// (3) Non-NUMS (cosigner) internal-key taproot multisig `tr(K2, multi_a(2,K0,K1))`
/// with a DISTINCT internal key: bundle emits, restore refuses with exit 2
/// "non-NUMS (cosigner) internal key" (restore.rs:689). (A non-distinct internal
/// key — `tr(K0, multi_a(2,K0,K1))` — is rejected EARLIER by bundle's BIP-388
/// distinct-key gate; the distinct K2 reaches the restore arm.)
#[test]
fn cosigner_internal_key_tr_bundles_but_restore_refuses_non_nums() {
    let desc = format!("tr({K2},multi_a(2,{K0},{K1}))");
    let (md1, _emitted) = bundle_md1(&desc);
    assert!(!md1.is_empty(), "cosigner-IK card must be emitted");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("non-NUMS (cosigner) internal key"));
}
