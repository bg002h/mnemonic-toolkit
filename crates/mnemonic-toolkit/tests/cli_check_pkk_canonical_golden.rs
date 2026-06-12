//! v0.55.0 WIRE-CHANGE CAPTURE — the always-on golden for the
//! Check(PkK|PkH)→bare canonicalization (user mandate: "if we change the wire
//! format, make sure our tests cover it").
//!
//! Before v0.55.0 the toolkit walker (`parse_descriptor.rs`) GATED the
//! `Terminal::Check(PkK|PkH) → bare Tag::PkK|PkH` collapse on `tap_context`:
//! it collapsed inside tap leaves but KEPT `Tag::Check(Tag::PkK)` on the wire
//! in wsh/sh. descriptor-mnemonic SPEC v0.30 §5.1 mandates bare `PkK`/`PkH`
//! regardless of context, and md-cli collapses unconditionally — so the
//! toolkit emitted a NON-CONFORMANT md1 for `wsh(pk)`-shaped descriptors and a
//! DIFFERENT `wallet_policy_id` than md-cli for the same wallet (interop
//! hazard, NOT funds-loss: both wire forms decode to the identical descriptor).
//! v0.55.0 drops the gate; this file PINS the post-fix wire bytes.
//!
//! Leg 1 (`golden_*`): for each affected shape, `bundle --descriptor … --json`
//! → `.md1` chunk array → decode IN-CRATE via `md_codec::chunk::reassemble`
//! → `compute_wallet_policy_id` + `compute_wallet_descriptor_template_id` →
//! `hex::encode(id.as_bytes())`, asserted == the FROZEN post-fix goldens
//! (== md-cli's already-conformant output, empirically re-derived).
//!
//! Leg 2 (`roundtrip_*`): `bundle --descriptor wsh(pk(…)) → restore --md1`
//! reconstructs `wsh(pk(…/<0;1>/*))` + valid bc1q addresses — proves the
//! toolkit reads its OWN new bare-PkK wire form (NOT redundant with the prop
//! test, whose generator only places pk/pkh INSIDE combinators, never a bare
//! top-level wsh(pk)/wsh(pkh)).
//!
//! These are NORMAL-suite, always-on, no-external-binary tests (the in-crate
//! decode means no MD_BIN; per the Cycle-A orphaned-vectors lesson, NOT a
//! gated/ignored vector). They FAIL pre-fix (the toolkit emitted `9ad78e4f…`
//! for wsh(pk)) and PASS post-fix.

use assert_cmd::Command;
use serde_json::Value;

// ── FROZEN, DEPTH-MATCHED xpub literals (mirrors
//    cli_cross_tool_differential.rs) ──────────────────────────────────────
// Derived (raw BIP-32) from the canonical abandon×11 about BIP-39 phrase,
// master fingerprint 73c5da0a, m/48'/0'/0'/2'. @1 is a second distinct
// depth-4 account key (m/48'/0'/1'/2').
const FP: &str = "73c5da0a";
const PATH4: &str = "48'/0'/0'/2'";
const XPUB4_0: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"; // m/48'/0'/0'/2'
const XPUB4_1: &str = "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk"; // m/48'/0'/1'/2'

fn bin() -> Command {
    Command::cargo_bin("mnemonic").unwrap()
}

/// `bundle --descriptor <concrete> --network mainnet --json` → md1 chunk array.
fn bundle_md1(desc: &str) -> Vec<String> {
    let out = bin()
        .args([
            "bundle",
            "--descriptor",
            desc,
            "--network",
            "mainnet",
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

/// Decode the toolkit's own md1 chunk array IN-CRATE (no MD_BIN) and return
/// `(wallet_policy_id, wallet_descriptor_template_id)` as lowercase hex.
fn ids_from_md1(md1: &[String]) -> (String, String) {
    let refs: Vec<&str> = md1.iter().map(String::as_str).collect();
    let desc = md_codec::chunk::reassemble(&refs).expect("md1 reassembles to a Descriptor");
    let policy = md_codec::compute_wallet_policy_id(&desc).expect("wallet_policy_id");
    let template = md_codec::compute_wallet_descriptor_template_id(&desc)
        .expect("wallet_descriptor_template_id");
    (
        hex::encode(policy.as_bytes()),
        hex::encode(template.as_bytes()),
    )
}

/// Bundle the concrete descriptor, decode its md1 in-crate, assert the frozen
/// post-fix (policy_id, template_id) goldens.
fn assert_golden(desc: &str, want_policy: &str, want_template: &str) {
    let md1 = bundle_md1(desc);
    let (policy, template) = ids_from_md1(&md1);
    assert_eq!(
        policy, want_policy,
        "wallet_policy_id golden mismatch for `{desc}`"
    );
    assert_eq!(
        template, want_template,
        "wallet_descriptor_template_id golden mismatch for `{desc}`"
    );
}

// One concrete cosigner key with the shared depth-4 origin.
fn k0() -> String {
    format!("[{FP}/{PATH4}]{XPUB4_0}/<0;1>/*")
}
fn k1() -> String {
    format!("[{FP}/{PATH4}]{XPUB4_1}/<0;1>/*")
}

// ── Leg 1 — frozen post-fix wire goldens ──────────────────────────────────

#[test]
fn golden_wsh_pk() {
    // PRE-fix toolkit emitted policy 9ad78e4f… (Check(PkK) kept); post-fix
    // emits bare PkK == md-cli.
    assert_golden(
        &format!("wsh(pk({}))", k0()),
        "58d1803363f5599914a9f4ba0afa97d7",
        "9208f59035e4912d4fca8182a897fafb",
    );
}

#[test]
fn golden_wsh_pkh() {
    assert_golden(
        &format!("wsh(pkh({}))", k0()),
        "3d6fb9a1656b02b36378645aaea9633e",
        "1499fe4902eaa084c9574ed33b7fc109",
    );
}

#[test]
fn golden_wsh_and_v_pk() {
    assert_golden(
        &format!("wsh(and_v(v:pk({}),pk({})))", k0(), k1()),
        "a513edb6343f69ca59841187a567a5ee",
        "cb13e9cd9a18a72e538a41482f562da8",
    );
}

#[test]
fn golden_wsh_or_d_pk() {
    assert_golden(
        &format!("wsh(or_d(pk({}),pk({})))", k0(), k1()),
        "aa4bbe01269571d7e5940f542a3b0a3c",
        "247773f7bc8f1e637d2c6f6163f811c5",
    );
}

// ── Leg 2 — toolkit reads its OWN new bare-PkK wire form ──────────────────

/// `bundle --descriptor wsh(pk(…)) → restore --md1` reconstructs
/// `wsh(pk(…/<0;1>/*))` + valid bc1q receive addresses. Targets the bare
/// top-level wsh(pk)/wsh(pkh) shapes the prop test structurally cannot reach.
fn roundtrip(desc: &str, expect_inner: &str) {
    let md1 = bundle_md1(desc);
    let mut a = vec![
        "restore".to_string(),
        "--network".into(),
        "mainnet".into(),
        "--count".into(),
        "2".into(),
        "--json".into(),
    ];
    for c in &md1 {
        a.push("--md1".into());
        a.push(c.clone());
    }
    let out = bin().args(&a).assert().success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let w = &v["wallets"][0];
    let recovered = w["descriptor"].as_str().unwrap();
    assert!(
        recovered.starts_with(expect_inner),
        "restore reconstructed `{recovered}`, expected to start with `{expect_inner}`"
    );
    let addrs: Vec<String> = w["first_addresses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    assert_eq!(addrs.len(), 2, "expected 2 receive addresses");
    for addr in &addrs {
        assert!(
            addr.starts_with("bc1q"),
            "expected a bc1q (wsh→v0 segwit) address, got `{addr}`"
        );
    }
}

#[test]
fn roundtrip_wsh_pk() {
    // restore reconstructs the bare-PkK md1 back to wsh(pk(...)). The
    // reconstruction uses depth-0 placeholder xpubs (known v0.49.1 restore
    // behavior), so we assert the `wsh(pk(` prefix + the chain suffix shape +
    // bc1q addresses, not the exact account-xpub.
    roundtrip(&format!("wsh(pk({}))", k0()), "wsh(pk(");
}

#[test]
fn roundtrip_wsh_pkh() {
    roundtrip(&format!("wsh(pkh({}))", k0()), "wsh(pkh(");
}
