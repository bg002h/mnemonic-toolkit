//! #28 phase 2 — `mnemonic bundle --md1-form=template` MULTISIG + general-policy
//! integration tests (P2 EMIT / Slice 1).
//!
//! Funds-safety class: a wrong template-form md1 = a wrong wallet. The gates
//! are byte-identity across keys/accounts (one engraving for thousands),
//! `md decode` round-trip (the C1 carried-origin invariant), shape refusals,
//! and the loud order-dependent warning — NOT exit-0. Every golden is anchored
//! OUTSIDE the toolkit synth path: the emitted md1 is decoded with the PUBLIC
//! `md_codec` API and reasoned about independently.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use std::str::FromStr;

// Two DISTINCT seed-sets so byte-identity must hold ACROSS keys.
const SEED_A1: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const SEED_A2: &str =
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";
const SEED_B1: &str = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
const SEED_B2: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

fn bundle(extra: &[String]) -> assert_cmd::assert::Assert {
    mnemonic().args(extra).assert()
}

fn bundle_ok(extra: &[String]) -> (String, String) {
    let out = bundle(extra).success();
    let o = out.get_output();
    (
        String::from_utf8(o.stdout.clone()).unwrap(),
        String::from_utf8(o.stderr.clone()).unwrap(),
    )
}

/// Extract the md1 string(s) from `bundle` text stdout (lines under `# md1`).
fn md1_lines(stdout: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_md1 = false;
    for line in stdout.lines() {
        if line.starts_with("# md1") {
            in_md1 = true;
            continue;
        }
        if in_md1 {
            if line.trim().is_empty() {
                in_md1 = false;
                continue;
            }
            out.push(line.trim().to_string());
        }
    }
    out
}

/// Derive a mainnet account xpub at `path_str` from a BIP-39 phrase.
fn xpub_at(phrase: &str, path_str: &str) -> (Xpub, String) {
    let secp = Secp256k1::new();
    let m = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = m.to_seed("");
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    let fp = master.fingerprint(&secp);
    let path = DerivationPath::from_str(path_str).unwrap();
    let xpriv = master.derive_priv(&secp, &path).unwrap();
    let xpub = Xpub::from_priv(&secp, &xpriv);
    (xpub, fp.to_string().to_lowercase())
}

/// Build a watch-only canonical multisig template bundle from N (phrase,path)
/// cosigners. `template` is e.g. "wsh-sortedmulti" / "wsh-multi" /
/// "sh-wsh-multi". Returns the full Assert (caller decides success/failure).
fn canonical_multisig_template_args(
    template: &str,
    threshold: &str,
    cosigners: &[(&str, &str)],
) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        template.into(),
        "--threshold".into(),
        threshold.into(),
        "--md1-form".into(),
        "template".into(),
        "--group-size".into(),
        "0".into(),
        "--no-engraving-card".into(),
    ];
    for (idx, (phrase, path)) in cosigners.iter().enumerate() {
        let (xpub, fp) = xpub_at(phrase, path);
        args.push("--slot".into());
        args.push(format!("@{idx}.xpub={xpub}"));
        args.push("--slot".into());
        args.push(format!("@{idx}.fingerprint={fp}"));
        args.push("--slot".into());
        args.push(format!("@{idx}.path={path}"));
    }
    args
}

// ===========================================================================
// Emit 1 — canonical multisig template byte-identity across seeds + accounts +
// `md decode` round-trip (elided origins).
// ===========================================================================

#[test]
fn canonical_multisig_template_byte_identical_across_seeds() {
    // Two different 2-of-2 key-sets at the SAME canonical wsh-multisig origin
    // (m/48'/0'/0'/2'). The keyless template md1 must be byte-identical.
    let key_a = "48'/0'/0'/2'";
    let args_a = canonical_multisig_template_args(
        "wsh-sortedmulti",
        "2",
        &[(SEED_A1, key_a), (SEED_A2, key_a)],
    );
    let args_b = canonical_multisig_template_args(
        "wsh-sortedmulti",
        "2",
        &[(SEED_B1, key_a), (SEED_B2, key_a)],
    );
    let (out_a, _) = bundle_ok(&args_a);
    let (out_b, _) = bundle_ok(&args_b);
    let md1_a = md1_lines(&out_a);
    let md1_b = md1_lines(&out_b);
    assert!(!md1_a.is_empty(), "canonical multisig template md1 emitted");
    assert_eq!(
        md1_a, md1_b,
        "canonical multisig template md1 must be byte-identical across DIFFERENT key-sets"
    );
}

#[test]
fn canonical_multisig_template_byte_identical_across_accounts() {
    // Same key-set, but different account index in the origin path → the
    // template md1 (origin-elided) must normalize the account away.
    let args_0 = canonical_multisig_template_args(
        "wsh-multi",
        "2",
        &[(SEED_A1, "48'/0'/0'/2'"), (SEED_A2, "48'/0'/0'/2'")],
    );
    let args_5 = canonical_multisig_template_args(
        "wsh-multi",
        "2",
        &[(SEED_A1, "48'/0'/5'/2'"), (SEED_A2, "48'/0'/5'/2'")],
    );
    let (out_0, _) = bundle_ok(&args_0);
    let (out_5, _) = bundle_ok(&args_5);
    assert_eq!(
        md1_lines(&out_0),
        md1_lines(&out_5),
        "wsh-multi template account must be normalized away (elided origin)"
    );
}

#[test]
fn canonical_multisig_template_md1_decodes_keyless() {
    let args = canonical_multisig_template_args(
        "wsh-sortedmulti",
        "2",
        &[(SEED_A1, "48'/0'/0'/2'"), (SEED_A2, "48'/0'/0'/2'")],
    );
    let (out, _) = bundle_ok(&args);
    let md1 = md1_lines(&out);
    let md1_refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let desc = md_codec::chunk::reassemble(&md1_refs).expect("canonical multisig md1 decodes");
    assert!(
        !desc.is_wallet_policy(),
        "multisig template md1 must be KEYLESS"
    );
    assert!(
        desc.tlv.pubkeys.is_none() && desc.tlv.fingerprints.is_none(),
        "multisig template md1 strips pubkeys + fingerprints"
    );
    assert_eq!(desc.n, 2, "2-of-2 → n=2 slots preserved");
    // canonical wsh-multisig is origin-elidable.
    assert!(
        md_codec::canonical_origin::canonical_origin(&desc.tree).is_some(),
        "canonical multisig template tree is origin-elidable"
    );
}

// ===========================================================================
// Emit 2 — GENERAL-POLICY template (degrade2 `wsh(or_i(...))`) decodes WITH
// carried per-`@N` origins (the C1 regression pin). The SAME shape emitted with
// EMPTY origins would FAIL `md decode` — so carrying the real origins is what
// makes it decode.
// ===========================================================================

const DEGRADE2_DESC: &str = include_str!("../../../.examples-build/degrade2.desc");

fn degrade2_desc() -> String {
    DEGRADE2_DESC.trim().to_string()
}

#[test]
fn general_policy_template_md1_decodes_with_carried_origins() {
    let desc = degrade2_desc();
    let args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--md1-form".into(),
        "template".into(),
        "--group-size".into(),
        "0".into(),
        "--no-engraving-card".into(),
        "--descriptor".into(),
        desc,
    ];
    let (out, _) = bundle_ok(&args);
    let md1 = md1_lines(&out);
    let md1_refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    // The C1 pin: a non-canonical wrapper with REAL carried origins decodes.
    let decoded = md_codec::chunk::reassemble(&md1_refs)
        .expect("general-policy template md1 must DECODE with carried per-@N origins (C1)");
    assert!(
        !decoded.is_wallet_policy(),
        "general-policy template md1 is keyless"
    );
    assert!(
        md_codec::canonical_origin::canonical_origin(&decoded.tree).is_none(),
        "degrade2 wsh(or_i(...)) is a non-canonical wrapper"
    );
    // Independent proof of the C1 mechanism: the SAME tree with EMPTY origins
    // fails `validate_explicit_origin_required` → MissingExplicitOrigin. We
    // assert the explicit-origin validation PASSES on the emitted descriptor
    // (it carries origins) — the negative is pinned in the unit test.
    md_codec::validate::validate_explicit_origin_required(&decoded)
        .expect("emitted general-policy template carries explicit origins (C1)");
}

// ===========================================================================
// Emit 3 — refusals: tr(sortedmulti_a) and hardened use-site.
// ===========================================================================

#[test]
fn template_form_refuses_tr_sortedmulti_a() {
    // tr-sortedmulti-a renders through the md-codec to_miniscript gap — refuse.
    let args = canonical_multisig_template_args(
        "tr-sortedmulti-a",
        "2",
        &[(SEED_A1, "48'/0'/0'/3'"), (SEED_A2, "48'/0'/0'/3'")],
    );
    bundle(&args).failure().code(2);
}

#[test]
fn template_form_refuses_hardened_use_site() {
    // A hardened use-site (`/*h`) cannot be derived from an xpub at restore →
    // refuse the template (#25 / has_hardened_use_site).
    let args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--md1-form".into(),
        "template".into(),
        "--group-size".into(),
        "0".into(),
        "--no-engraving-card".into(),
        "--descriptor".into(),
        "wsh(multi(2,@0/*h,@1/*h))".into(),
        "--slot".into(),
        format!("@0.phrase={SEED_A1}"),
        "--slot".into(),
        format!("@1.phrase={SEED_A2}"),
    ];
    bundle(&args).failure().code(2);
}

#[test]
fn template_form_admits_tr_nums_multi_a() {
    // tr-multi-a (NUMS internal key, multi_a tap leaf) is the SHIPPED restorable
    // taproot multisig shape — must be ADMITTED and decode keyless.
    let args = canonical_multisig_template_args(
        "tr-multi-a",
        "2",
        &[(SEED_A1, "48'/0'/0'/3'"), (SEED_A2, "48'/0'/0'/3'")],
    );
    let (out, _) = bundle_ok(&args);
    let md1 = md1_lines(&out);
    let md1_refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let desc = md_codec::chunk::reassemble(&md1_refs).expect("tr-multi-a template md1 decodes");
    assert!(!desc.is_wallet_policy(), "tr-multi-a template is keyless");
    assert_eq!(desc.n, 2);
}

// ===========================================================================
// Emit 4 — the loud order-dependent warning (N!) fires for an order-dependent
// shape, softened/absent for sortedmulti.
// ===========================================================================

#[test]
fn order_dependent_multisig_template_emits_loud_warning() {
    // wsh-multi (unsorted) is ORDER-DEPENDENT → the loud N! warning must fire.
    let args = canonical_multisig_template_args(
        "wsh-multi",
        "2",
        &[(SEED_A1, "48'/0'/0'/2'"), (SEED_A2, "48'/0'/0'/2'")],
    );
    let (_stdout, stderr) = bundle_ok(&args);
    // N = 2 distinct slots → 2! = 2 assignments.
    assert!(
        stderr.contains("2!") || stderr.contains("2 assignment") || stderr.contains("assignments"),
        "order-dependent multisig template warning must state the N! assignment count: {stderr}"
    );
    assert!(
        stderr.to_lowercase().contains("only one assignment"),
        "warning must say only one assignment reproduces this wallet: {stderr}"
    );
}

#[test]
fn sortedmulti_template_softens_order_warning() {
    // sortedmulti is order-INDEPENDENT → no loud N! warning.
    let args = canonical_multisig_template_args(
        "wsh-sortedmulti",
        "2",
        &[(SEED_A1, "48'/0'/0'/2'"), (SEED_A2, "48'/0'/0'/2'")],
    );
    let (_stdout, stderr) = bundle_ok(&args);
    assert!(
        !stderr.to_lowercase().contains("only one assignment"),
        "sortedmulti is order-independent — must NOT print the loud single-assignment warning: {stderr}"
    );
}

#[test]
fn general_policy_template_warns_about_spending_role() {
    // A general policy (degrade2) has ASYMMETRIC semantics: a wrong assignment
    // changes each key's SPENDING ROLE, not just the address.
    let args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--md1-form".into(),
        "template".into(),
        "--group-size".into(),
        "0".into(),
        "--no-engraving-card".into(),
        "--descriptor".into(),
        degrade2_desc(),
    ];
    let (_stdout, stderr) = bundle_ok(&args);
    assert!(
        stderr.to_lowercase().contains("spending role"),
        "general-policy template warning must name the asymmetric spending-role caveat: {stderr}"
    );
}

// ===========================================================================
// Emit 5 — D7 WalletPolicyId print for a multisig template (full 16-byte hex +
// 4-byte prefix) on stderr.
// ===========================================================================

#[test]
fn multisig_template_prints_wallet_policy_id() {
    let cosigners = &[(SEED_A1, "48'/0'/0'/2'"), (SEED_A2, "48'/0'/0'/2'")];
    let args = canonical_multisig_template_args("wsh-sortedmulti", "2", cosigners);
    let (stdout, stderr) = bundle_ok(&args);
    assert!(
        stderr.contains("wallet-id"),
        "multisig template must print the WalletPolicyId on stderr: {stderr}"
    );
    assert!(
        !stdout.contains("wallet-id"),
        "D7 advisory must NOT pollute stdout (the engraved cards)"
    );

    // FUNDS-SAFETY anchor: the printed wallet-id must be the REAL WalletPolicyId
    // of this wallet — equal to the policy-form md1's binding id, recomputed
    // INDEPENDENTLY via md_codec. (No stale/wrong id.)
    let mut policy_args = canonical_multisig_template_args("wsh-sortedmulti", "2", cosigners);
    // swap --md1-form template → policy.
    let pos = policy_args.iter().position(|a| a == "template").unwrap();
    policy_args[pos] = "policy".into();
    let (policy_out, _) = bundle_ok(&policy_args);
    let policy_md1 = md1_lines(&policy_out);
    let policy_refs: Vec<&str> = policy_md1.iter().map(|s| s.as_str()).collect();
    let policy_desc = md_codec::chunk::reassemble(&policy_refs).unwrap();
    assert!(policy_desc.is_wallet_policy());
    let expect_id = md_codec::compute_wallet_policy_id(&policy_desc).unwrap();
    let expect_hex = hex::encode(expect_id.as_bytes());
    assert!(
        stderr.contains(&expect_hex),
        "D7 printed wallet-id must equal the independent policy-form WalletPolicyId {expect_hex}; stderr: {stderr}"
    );
}

#[test]
fn multisig_template_bundle_self_check_passes() {
    // The card↔template-id binding must hold across all N mk1 cards for a
    // multisig template (self-check recomputes the template-id stub).
    let mut args = canonical_multisig_template_args(
        "wsh-sortedmulti",
        "2",
        &[(SEED_A1, "48'/0'/0'/2'"), (SEED_A2, "48'/0'/0'/2'")],
    );
    args.push("--self-check".into());
    bundle(&args).success();
}

// ===========================================================================
// Emit 8 — non-regression: single-sig --md1-form=template + --md1-form=policy
// are byte-identical to pre-P2 (covered in cli_bundle_md1_template_form.rs;
// here we pin that the multisig path did not perturb single-sig template emit).
// ===========================================================================

#[test]
fn single_sig_template_still_emits_after_multisig_admission() {
    let (out, _) = bundle_ok(&[
        "bundle".into(),
        "--template".into(),
        "bip84".into(),
        "--network".into(),
        "mainnet".into(),
        "--md1-form".into(),
        "template".into(),
        "--group-size".into(),
        "0".into(),
        "--slot".into(),
        format!("@0.phrase={SEED_A1}"),
    ]);
    let md1 = md1_lines(&out);
    let md1_refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let desc = md_codec::chunk::reassemble(&md1_refs).expect("single-sig template still decodes");
    assert!(!desc.is_wallet_policy());
    assert_eq!(desc.n, 1);
}
