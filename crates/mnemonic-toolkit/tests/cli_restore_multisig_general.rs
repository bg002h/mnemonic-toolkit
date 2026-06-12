//! v0.54.0 — `restore --md1` reconstructs a GENERAL wallet-policy descriptor
//! (timelocks/hashlocks/andor/decay) FAITHFULLY, instead of silently collapsing
//! it to plain multisig (the C1 funds-safety bug). See
//! `design/SPEC_faithful_general_policy_restore.md`.
//!
//! Oracle (byte-equality with `export-wallet --descriptor` is impossible —
//! md1 keys are depth-0): (1) the reconstructed descriptor contains the policy
//! fragments; (2) md1 fixed-point — re-bundling the reconstructed descriptor
//! reproduces the original md1; (3) addresses derive from the emitted descriptor.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

const C0: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const C1: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const C2: &str = "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";

/// Bundle a general policy and return its md1 chunks. Supplies exactly as many
/// `--slot`s as the descriptor has distinct `@N` placeholders (0..=max).
fn bundle_general(descriptor: &str) -> Vec<String> {
    bundle_general_net(descriptor, "mainnet")
}

fn bundle_general_net(descriptor: &str, network: &str) -> Vec<String> {
    let phrases = [C0, C1, C2];
    let max_n = (0u8..3)
        .filter(|n| descriptor.contains(&format!("@{n}")))
        .max()
        .expect("descriptor has at least @0");
    let mut args: Vec<String> = vec![
        "bundle".into(),
        "--descriptor".into(),
        descriptor.into(),
        "--network".into(),
        network.into(),
    ];
    for n in 0..=max_n {
        args.push("--slot".into());
        args.push(format!("@{n}.phrase={}", phrases[n as usize]));
    }
    args.push("--json".into());
    args.push("--no-engraving-card".into());
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).expect("bundle JSON");
    v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

fn restore_md1_args(md1: &[String]) -> Vec<String> {
    restore_md1_args_net(md1, "mainnet")
}

fn restore_md1_args_net(md1: &[String], network: &str) -> Vec<String> {
    let mut a = vec!["restore".to_string(), "--network".into(), network.into()];
    for c in md1 {
        a.push("--md1".into());
        a.push(c.clone());
    }
    a
}

fn restore_json(md1: &[String]) -> Value {
    let mut a = restore_md1_args(md1);
    a.push("--json".into());
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .success();
    serde_json::from_slice(&out.get_output().stdout).expect("restore JSON")
}

/// The reconstructed descriptor's md1 == the original md1 (md1 fixed-point).
fn assert_md1_fixed_point(original_md1: &[String], reconstructed_descriptor: &str) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            reconstructed_descriptor,
            "--network",
            "mainnet",
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let rebundled: Vec<String> = v["md1"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        rebundled, original_md1,
        "md1 fixed-point: re-bundle must reproduce the card"
    );
}

/// (1) and_v + older: the timelock is PRESERVED (pre-fix it collapsed to plain
/// `wsh(multi(2,…))`, dropping `older(4032)`). + label "miniscript-policy",
/// null top-level threshold, md1 fixed-point.
#[test]
fn general_and_v_older_reconstructs_faithfully() {
    let md1 = bundle_general("wsh(and_v(v:multi(2,@0,@1),older(4032)))");
    let v = restore_json(&md1);
    let w = &v["wallets"][0];
    let desc = w["descriptor"].as_str().unwrap();
    assert!(
        desc.contains("and_v(v:multi(2,"),
        "must keep the and_v(v:multi): {desc}"
    );
    assert!(
        desc.contains("older(4032)"),
        "must keep older(4032): {desc}"
    );
    assert!(
        !desc.starts_with("wsh(multi("),
        "must NOT collapse to plain multi: {desc}"
    );
    assert_eq!(w["wallet_type"], "miniscript-policy");
    assert_eq!(
        v["threshold"],
        Value::Null,
        "general policy has no single threshold"
    );
    assert!(!w["first_addresses"].as_array().unwrap().is_empty());
    assert_md1_fixed_point(&md1, desc);
}

/// (2) decay vault (or_d of two multi, gated by older): both multis + the
/// timelock survive; threshold is null (NOT the first multi's k).
#[test]
fn general_decay_vault_reconstructs_faithfully() {
    let md1 = bundle_general("wsh(or_d(multi(2,@0,@1),and_v(v:multi(1,@2),older(1000))))");
    let v = restore_json(&md1);
    let desc = v["wallets"][0]["descriptor"].as_str().unwrap();
    assert!(
        desc.contains("or_d(multi(2,"),
        "outer 2-of-2 multi kept: {desc}"
    );
    assert!(
        desc.contains("multi(1,"),
        "recovery 1-of-1 multi kept: {desc}"
    );
    assert!(desc.contains("older(1000)"), "decay timelock kept: {desc}");
    assert_eq!(
        v["threshold"],
        Value::Null,
        "decay vault has no single threshold"
    );
    assert_md1_fixed_point(&md1, desc);
}

/// (3) hashlock (sha256) survives reconstruction.
#[test]
fn general_sha256_hashlock_reconstructs_faithfully() {
    let h = "926a54995ca48600920a19bf7bc502ca5f2f7d07e6f804c4f00ebf0325084dbc";
    let md1 = bundle_general(&format!("wsh(and_v(v:multi(2,@0,@1),sha256({h})))"));
    let v = restore_json(&md1);
    let desc = v["wallets"][0]["descriptor"].as_str().unwrap();
    assert!(
        desc.contains(&format!("sha256({h})")),
        "sha256 digest kept verbatim: {desc}"
    );
    assert_md1_fixed_point(&md1, desc);
}

/// (4) `--format descriptor`/`bip388` emit the FAITHFUL general descriptor.
#[test]
fn general_format_descriptor_is_faithful() {
    let md1 = bundle_general("wsh(and_v(v:multi(2,@0,@1),older(4032)))");
    let mut a = restore_md1_args(&md1);
    a.push("--format".into());
    a.push("descriptor".into());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .success()
        .stdout(predicate::str::contains("older(4032)"));
}

/// (4b) `--format coldcard` (template-requiring) REFUSES a general policy
/// (loud, exit non-zero) — it cannot represent a non-k-of-n policy.
#[test]
fn general_format_coldcard_refuses() {
    let md1 = bundle_general("wsh(and_v(v:multi(2,@0,@1),older(4032)))");
    let mut a = restore_md1_args(&md1);
    a.push("--format".into());
    a.push("coldcard".into());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .failure();
}

/// (5) pk-keyed shape: a `pkh(@N)` leaf reconstructs faithfully (md-codec
/// 0.35.1 fixed the `Check` double-wrap that previously made this a loud
/// refusal — PART 2 / `to-miniscript-check-pkh-double-wrap`). Both `pkh`
/// positions + the timelock survive.
#[test]
fn general_pkh_leaf_reconstructs_after_md_codec_fix() {
    let md1 = bundle_general("wsh(or_d(multi(2,@0,@1),and_v(v:pkh(@2),older(144))))");
    let v = restore_json(&md1);
    let desc = v["wallets"][0]["descriptor"].as_str().unwrap();
    assert!(desc.contains("or_d(multi(2,"), "outer multi kept: {desc}");
    assert!(
        desc.contains("pkh("),
        "pkh leaf kept (not collapsed): {desc}"
    );
    assert!(desc.contains("older(144)"), "timelock kept: {desc}");
    assert_md1_fixed_point(&md1, desc);
}

/// (5b) the v0.19.0 FLAGSHIP — `wsh(andor(pkh(@0),after(N),or_i(and_v(v:pkh(@1),
/// older(M)),and_v(v:pkh(@2),older(K)))))` — keys are bare `pkh(@N)` leaves
/// (NOT inside a multi). Loud-refused at toolkit v0.54.0; reconstructs after the
/// md-codec 0.35.1 pin bump, through the SAME general arm with zero toolkit code
/// change. Every key-check fragment + all three timelocks survive.
#[test]
fn flagship_pk_keyed_vault_reconstructs() {
    let md1 = bundle_general(
        "wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))",
    );
    let v = restore_json(&md1);
    let desc = v["wallets"][0]["descriptor"].as_str().unwrap();
    for frag in [
        "andor(pkh(",
        "after(12000000)",
        "or_i(and_v(v:pkh(",
        "older(4032)",
        "older(32768)",
    ] {
        assert!(desc.contains(frag), "flagship must keep {frag}: {desc}");
    }
    assert_md1_fixed_point(&md1, desc);
}

/// (6) wildcard-only keys (R0-r1 I3, `multipath == None`): a `--descriptor`
/// bundle produces `xpub/*` (no `<0;1>` group). The faithful arm must
/// reconstruct it WITHOUT fabricating `<0;1>` (pass the XPub through unchanged
/// shape) — and addresses still derive. (The canonical `<0;1>` plain-multisig
/// path is exercised byte-for-byte by the 13+12 `--template` goldens.)
#[test]
fn general_wildcard_only_multipath_none_reconstructs_without_fabricating_multipath() {
    let md1 = bundle_general("wsh(and_v(v:multi(2,@0,@1),older(4032)))");
    let v = restore_json(&md1);
    let w = &v["wallets"][0];
    let desc = w["descriptor"].as_str().unwrap();
    assert!(
        desc.contains("/*)"),
        "wildcard-only keys kept as /*: {desc}"
    );
    assert!(
        !desc.contains("<0;1>"),
        "must NOT fabricate a <0;1> multipath: {desc}"
    );
    assert!(
        !w["first_addresses"].as_array().unwrap().is_empty(),
        "addresses derive"
    );
    assert_md1_fixed_point(&md1, desc);
}

/// (7) canonical `<0;1>/*` multipath general policy — exercises the Translator's
/// `MultiXPub` arm (the `Some(alts)` branch): keys reconstruct as `<0;1>/*`.
#[test]
fn general_multipath_0_1_reconstructs_with_multixpub_arm() {
    let md1 = bundle_general("wsh(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*),older(50)))");
    let v = restore_json(&md1);
    let desc = v["wallets"][0]["descriptor"].as_str().unwrap();
    assert!(
        desc.contains("<0;1>/*"),
        "MultiXPub <0;1> multipath kept: {desc}"
    );
    assert!(desc.contains("older(50)"), "timelock kept: {desc}");
    assert_md1_fixed_point(&md1, desc);
}

/// (8) testnet: the `--network`-correct key kind (tpub) + a testnet address —
/// guards the translator's `xkey.network` correction (md1 is network-agnostic;
/// md-codec hardcodes Main).
#[test]
fn general_testnet_network_corrected() {
    let md1 = bundle_general_net("wsh(and_v(v:multi(2,@0,@1),older(50)))", "testnet");
    let mut a = restore_md1_args_net(&md1, "testnet");
    a.push("--json".into());
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let w = &v["wallets"][0];
    assert!(
        w["descriptor"].as_str().unwrap().contains("tpub"),
        "testnet → tpub keys"
    );
    assert!(
        w["first_addresses"][0].as_str().unwrap().starts_with("tb1"),
        "testnet → tb1 address"
    );
}

/// (9) `after()` absolute timelock survives.
#[test]
fn general_after_reconstructs_faithfully() {
    let md1 = bundle_general("wsh(and_v(v:multi(2,@0,@1),after(800000)))");
    let v = restore_json(&md1);
    let desc = v["wallets"][0]["descriptor"].as_str().unwrap();
    assert!(desc.contains("after(800000)"), "after kept: {desc}");
    assert_md1_fixed_point(&md1, desc);
}

/// (10) legacy `sh(multi)` reconstructs faithfully (impl-review M2: was a loud
/// refusal pre-fix at `template_from_descriptor`'s `ShInner::Ms` arm).
#[test]
fn general_legacy_sh_multi_reconstructs() {
    let md1 = bundle_general("sh(multi(2,@0,@1))");
    let v = restore_json(&md1);
    let desc = v["wallets"][0]["descriptor"].as_str().unwrap();
    assert!(
        desc.starts_with("sh(multi(2,"),
        "legacy sh(multi) kept: {desc}"
    );
    assert_md1_fixed_point(&md1, desc);
}

/// (11) impl-review I1: a card with per-cosigner use-site overrides (cosigners
/// not sharing one multipath suffix) is REFUSED loudly — md-codec would render
/// one shared suffix, silently misrepresenting the wallet.
#[test]
fn per_key_use_site_override_refused() {
    let md1 = bundle_general("wsh(multi(2,@0/<0;1>/*,@1/*))");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_md1_args(&md1))
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("per-cosigner use-site").and(predicate::str::contains(
                "restore-md1-per-key-use-site-and-hardened-wildcard",
            )),
        );
}

/// (12) impl-review I2: a hardened wildcard (`/*h`) is REFUSED loudly (can't
/// derive watch-only addresses; would silently render an unhardened `/*`).
#[test]
fn hardened_wildcard_refused() {
    let md1 = bundle_general("wsh(multi(2,@0/*h,@1/*h))");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_md1_args(&md1))
        .assert()
        .failure()
        .stderr(predicate::str::contains("hardened wildcard"));
}
