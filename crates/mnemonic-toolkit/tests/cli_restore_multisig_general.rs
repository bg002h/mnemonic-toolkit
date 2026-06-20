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

/// Independent address-equivalence oracle for the faithful arm: the toolkit's
/// REPORTED `first_addresses` (its internal derivation) must equal an INDEPENDENT
/// rust-miniscript derivation of the reconstructed descriptor STRING
/// (`into_single_descriptors` — a code path that does NOT re-enter md-codec's
/// reconstruction). Used where `assert_md1_fixed_point` is inapplicable (a
/// faithful-arm divergent card reconstructs fingerprint-ONLY origins, which
/// `bundle` won't re-ingest — a pre-existing bundle limitation, not a fidelity
/// gap; the card's per-key suffix is faithfully reconstructed regardless).
fn assert_reported_addresses_match_independent_derivation(restore_value: &Value) {
    use miniscript::descriptor::DescriptorPublicKey;
    use miniscript::{DefiniteDescriptorKey, Descriptor};
    use std::str::FromStr;
    let w = &restore_value["wallets"][0];
    let desc = w["descriptor"].as_str().unwrap();
    let reported: Vec<String> = w["first_addresses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    assert!(!reported.is_empty(), "restore reported no addresses: {desc}");
    let d = Descriptor::<DescriptorPublicKey>::from_str(desc).unwrap();
    let receive = if d.is_multipath() {
        d.clone().into_single_descriptors().unwrap().remove(0)
    } else {
        d.clone()
    };
    for (i, rep) in reported.iter().enumerate() {
        let def: Descriptor<DefiniteDescriptorKey> = if receive.has_wildcard() {
            receive.derive_at_index(i as u32).unwrap()
        } else {
            Descriptor::<DefiniteDescriptorKey>::try_from(receive.clone()).unwrap()
        };
        let indep = def.address(bitcoin::Network::Bitcoin).unwrap().to_string();
        assert_eq!(
            rep, &indep,
            "toolkit-reported address[{i}] diverged from the independent \
             rust-miniscript derivation of the reconstructed descriptor: {desc}"
        );
    }
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

/// (1b) n=1 GENERAL wsh — a SINGLE-key timelocked policy
/// `wsh(and_v(v:pk(@0),older(144)))` with NO `multi`/`sortedmulti` at the
/// trunk. Every other general-faithful cell (and STRESS-A) puts a multi at
/// the trunk; this is the only n=1 `v:pk` general restore — the shape the
/// toolkit↔Bitcoin-Core end-to-end oracle's `wsh-timelocked` row exercises
/// (`tests/bitcoind_differential.rs`). That oracle is `#[ignore]`/cron-gated,
/// so without this cell shape-6 reconstructability has NO default-CI signal
/// (`toolkit-bitcoind-end-to-end-oracle` R0-r1 I-1). Faithful (pre-v0.54.0
/// this single-key class would have collapsed) + md1 fixed-point.
#[test]
fn n1_general_wsh_timelocked_restores_faithfully() {
    let md1 = bundle_general("wsh(and_v(v:pk(@0),older(144)))");
    let v = restore_json(&md1);
    let w = &v["wallets"][0];
    let desc = w["descriptor"].as_str().unwrap();
    assert!(
        desc.contains("and_v(v:pk("),
        "must keep and_v(v:pk): {desc}"
    );
    assert!(desc.contains("older(144)"), "must keep older(144): {desc}");
    assert!(
        !desc.starts_with("wsh(pk("),
        "must NOT collapse to plain pk: {desc}"
    );
    assert_eq!(w["wallet_type"], "miniscript-policy");
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

/// (4c) consensus-masked `older()` advisory (Task 7): an md1 card carrying a
/// BIP-68 consensus-masked `older(65536)` (low-16 zero → effective 0 blocks)
/// reconstructs successfully (exit 0, descriptor printed verbatim) AND prints a
/// non-blocking advisory to stderr. `older(65536)` is bit-31-clear so it bundles
/// + parses normally; the advisory fires at the post-`from_str` Adapter-B hook.
#[test]
fn general_masked_older_emits_advisory() {
    let md1 = bundle_general("wsh(and_v(v:multi(2,@0,@1),older(65536)))");
    let mut a = restore_md1_args(&md1);
    a.push("--format".into());
    a.push("descriptor".into());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .success()
        .stdout(predicate::str::contains("older(65536)"))
        .stderr(predicate::str::contains(
            "advisory: older(65536) is consensus-masked",
        ));
}

/// (4d) clean `older()` does NOT trigger the advisory — `older(4032)` is a clean
/// block value (low-16 nonzero, no stray bits), so no `advisory: older` on stderr.
#[test]
fn general_clean_older_no_advisory() {
    let md1 = bundle_general("wsh(and_v(v:multi(2,@0,@1),older(4032)))");
    let mut a = restore_md1_args(&md1);
    a.push("--format".into());
    a.push("descriptor".into());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .success()
        .stdout(predicate::str::contains("older(4032)"))
        .stderr(predicate::str::contains("advisory: older").not());
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

/// (11) P2.3 FLIP (was: I1 refusal): a card with per-cosigner use-site overrides
/// (a STANDARD `@0` baseline, a `None`-multipath `@1` override) now restores
/// FAITHFULLY. md-codec 0.37.0's `to_miniscript_descriptor_multipath` reconstructs
/// each `@N`'s own group; the toolkit faithful arm consumes it. The `Some`/`None`
/// mix proves `@1` renders as a single-path `/*` while `@0` stays `<0;1>`.
/// md1 fixed-point is the strongest faithfulness oracle (re-bundle == original).
#[test]
fn per_key_use_site_override_some_none_mix_restores_faithfully() {
    let md1 = bundle_general("wsh(multi(2,@0/<0;1>/*,@1/*))");
    let v = restore_json(&md1);
    let desc = v["wallets"][0]["descriptor"].as_str().unwrap();
    // @0 keeps its <0;1> group; @1 is a bare single-path /* (the None override).
    assert!(
        desc.contains("<0;1>/*"),
        "@0 keeps its multipath group: {desc}"
    );
    // Exactly ONE `<0;1>` occurrence: @1 must NOT have been clobbered into one.
    assert_eq!(
        desc.matches("<0;1>").count(),
        1,
        "@1 (None override) must stay single-path /*, not be re-clobbered to <0;1>: {desc}"
    );
    assert_reported_addresses_match_independent_derivation(&v);
}

/// (11a) P2.1+P2.2 CORE: a card with a STANDARD `@0` baseline and a DIVERGENT
/// `@1` use-site override (`@1/<2;3>/*`) reconstructs FAITHFULLY — the
/// reconstructed STRING carries `@1`'s divergent `<2;3>` suffix (NOT a clobbered
/// `<0;1>`), and the divergent cosigner's receive address derives INDEPENDENTLY
/// (rust-miniscript) to the pinned golden anchored OUTSIDE the toolkit codec.
///
/// P2.1 evidence: had the plain-template arm fired, its renderer would print
/// `<0;1>` for BOTH keys (`wallet_export/pipeline.rs`) — so observing `<2;3>` in
/// the output PROVES the override card routed to the faithful arm.
#[test]
fn per_key_use_site_override_divergent_restores_faithfully() {
    let md1 = bundle_general("wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*))");
    let v = restore_json(&md1);
    let desc = v["wallets"][0]["descriptor"].as_str().unwrap();
    // @0's baseline group AND @1's divergent group both present, distinct.
    assert!(desc.contains("<0;1>/*"), "@0 keeps <0;1>: {desc}");
    assert!(
        desc.contains("<2;3>/*"),
        "@1's DIVERGENT <2;3> suffix preserved (faithful arm, not plain-template clobber): {desc}"
    );
    assert_reported_addresses_match_independent_derivation(&v);
    assert_divergent_address_independent_golden(desc);
}

/// (11b) `wsh(sortedmulti)` divergent: sortedmulti sorts per-derived-key at
/// `into_single_descriptors`; the divergent suffix must still round-trip.
#[test]
fn per_key_use_site_override_divergent_sortedmulti_restores_faithfully() {
    let md1 = bundle_general("wsh(sortedmulti(2,@0/<0;1>/*,@1/<2;3>/*))");
    let v = restore_json(&md1);
    let desc = v["wallets"][0]["descriptor"].as_str().unwrap();
    assert!(desc.starts_with("wsh(sortedmulti(2,"), "sortedmulti kept: {desc}");
    assert!(desc.contains("<0;1>/*"), "@0 keeps <0;1>: {desc}");
    assert!(desc.contains("<2;3>/*"), "@1 divergent <2;3> kept: {desc}");
    assert_reported_addresses_match_independent_derivation(&v);
}

/// (11c) `sh(wsh(multi))` (M2) divergent: the nested-witness path also routes
/// to the faithful arm and preserves the divergent suffix.
#[test]
fn per_key_use_site_override_divergent_sh_wsh_multi_restores_faithfully() {
    let md1 = bundle_general("sh(wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*)))");
    let v = restore_json(&md1);
    let desc = v["wallets"][0]["descriptor"].as_str().unwrap();
    assert!(desc.starts_with("sh(wsh(multi(2,"), "sh-wsh-multi kept: {desc}");
    assert!(desc.contains("<2;3>/*"), "@1 divergent <2;3> kept: {desc}");
    assert_reported_addresses_match_independent_derivation(&v);
}

/// (11d) bare `sh(multi)` P2SH (M1) divergent — a DISTINCT routing path:
/// `plain_template_from_tree` matches only `Wsh`/`Sh→Wsh`, so bare `sh(multi)`
/// already returns `None` → faithful arm. With an override it must STILL
/// reconstruct the divergent suffix faithfully.
#[test]
fn per_key_use_site_override_divergent_bare_sh_multi_restores_faithfully() {
    let md1 = bundle_general("sh(multi(2,@0/<0;1>/*,@1/<2;3>/*))");
    let v = restore_json(&md1);
    let desc = v["wallets"][0]["descriptor"].as_str().unwrap();
    assert!(desc.starts_with("sh(multi(2,"), "bare sh(multi) kept: {desc}");
    assert!(desc.contains("<2;3>/*"), "@1 divergent <2;3> kept: {desc}");
    assert_reported_addresses_match_independent_derivation(&v);
}

/// (11e) P2.3 guard — a TAPROOT override card (`tr(NUMS,multi_a)` with a
/// divergent `@1`) is STILL REFUSED loudly (the taproot leg is deferred:
/// `taproot_override_card(d)` guard). The error names the taproot deferral.
#[test]
fn taproot_use_site_override_still_refused() {
    let md1 = bundle_general("tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<2;3>/*))");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_md1_args(&md1))
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("taproot").and(predicate::str::contains(
                "restore-md1-taproot-use-site-override-arm",
            )),
        );
}

/// (11f) P2.3 guard — a NON-taproot override card whose `@1` override carries a
/// HARDENED wildcard (`/*h`) is STILL REFUSED loudly (`has_hardened_use_site(d)`
/// guard; watch-only cannot derive hardened). The baseline `@0` is CLEAN — the
/// hardened path is ONLY in `@1`'s override — so this exercises the
/// override-aware leg of the predicate (not just the baseline scan).
#[test]
fn override_hardened_wildcard_refused() {
    let md1 = bundle_general("wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*h))");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_md1_args(&md1))
        .assert()
        .failure()
        .stderr(predicate::str::contains("hardened use-site path"));
}

/// Independent golden anchor (SPEC I1): the divergent cosigner `@1`'s chain-0
/// idx-0 receive address, derived via rust-miniscript `into_single_descriptors`
/// on the RECONSTRUCTED descriptor (a code path that does NOT go through
/// md-codec's reconstruction), must equal a value pinned from an offline
/// derivation of `@1`'s xpub at `2/0` (its OWN alt0, chain 0). The two cosigner
/// xpubs in the reconstructed descriptor come from the fixed C0/C1 phrases, so
/// the golden is deterministic. Anchored 2026-06-19 (see test (11g) generator).
fn assert_divergent_address_independent_golden(reconstructed_desc: &str) {
    use miniscript::descriptor::DescriptorPublicKey;
    use miniscript::{DefiniteDescriptorKey, Descriptor};
    use std::str::FromStr;
    // chain-0 single descriptor (receive). into_single_descriptors() honors
    // each key's OWN multipath: @1 takes its <2;3> alt0 = child 2.
    let d = Descriptor::<DescriptorPublicKey>::from_str(reconstructed_desc).unwrap();
    let receive = d.into_single_descriptors().unwrap().remove(0);
    let def: Descriptor<DefiniteDescriptorKey> = receive.derive_at_index(0).unwrap();
    let addr = def.address(bitcoin::Network::Bitcoin).unwrap().to_string();
    // INDEPENDENT golden: the wsh(multi) script-hash address for chain0/idx0,
    // where @0 derives at <0;1>→0/0 and @1 derives at its DIVERGENT <2;3>→2/0.
    // A baseline-clobber bug (@1 at 0/0) would produce a DIFFERENT address.
    assert_eq!(
        addr, DIVERGENT_WSH_MULTI_CHAIN0_IDX0_GOLDEN,
        "divergent-suffix chain0/idx0 address drifted — @1 must derive at its OWN <2;3> alt, not the baseline <0;1>"
    );
}

/// Offline-anchored golden: chain0/idx0 of
/// `wsh(multi(2, <C0-xpub>/<0;1>/*, <C1-xpub>/<2;3>/*))`. @1's divergence makes
/// this DISTINCT from the all-baseline `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))`
/// address — the anti-clobber discriminator. Captured 2026-06-19 by test (11g).
const DIVERGENT_WSH_MULTI_CHAIN0_IDX0_GOLDEN: &str =
    "bc1qkay38njhhx0c5zx43vrfglxlj0wa7dhm7d54q9fu7gredyj8hpusnqjy9k";

/// (11g) GENERATOR + anti-vacuity: prove the divergent golden DIFFERS from the
/// all-baseline address (so the golden actually anchors divergence, not codec
/// self-agreement) AND print the value to bake into the const above. The
/// xpubs are derived from the fixed C0/C1 phrases via the toolkit's own
/// bundle — but the ADDRESS golden is computed by rust-miniscript here, an
/// INDEPENDENT path from md-codec reconstruction.
#[test]
fn divergent_golden_differs_from_baseline_and_anchors() {
    use miniscript::descriptor::DescriptorPublicKey;
    use miniscript::{DefiniteDescriptorKey, Descriptor};
    use std::str::FromStr;
    // Pull the two cosigner xpubs out of a divergent restore (their VALUE is
    // independent of the divergence; the suffix is what we vary below).
    let md1 = bundle_general("wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*))");
    let v = restore_json(&md1);
    let divergent_desc = v["wallets"][0]["descriptor"].as_str().unwrap().to_string();

    let addr_at = |desc: &str| -> String {
        let d = Descriptor::<DescriptorPublicKey>::from_str(desc).unwrap();
        let receive = d.into_single_descriptors().unwrap().remove(0);
        let def: Descriptor<DefiniteDescriptorKey> = receive.derive_at_index(0).unwrap();
        def.address(bitcoin::Network::Bitcoin).unwrap().to_string()
    };

    // The all-baseline counterpart: same xpubs, @1 at <0;1> instead of <2;3>.
    // Strip the BIP-380 `#checksum` first — the string edit invalidates it, and
    // rust-miniscript's `from_str` accepts a checksum-less descriptor.
    let strip_csum = |s: &str| s.split('#').next().unwrap().to_string();
    let baseline_desc = strip_csum(&divergent_desc).replacen("<2;3>", "<0;1>", 1);
    let divergent_addr = addr_at(&divergent_desc);
    let baseline_addr = addr_at(&baseline_desc);

    eprintln!("DIVERGENT_WSH_MULTI_CHAIN0_IDX0_GOLDEN = {divergent_addr}");
    eprintln!("(all-baseline counterpart = {baseline_addr})");
    assert_ne!(
        divergent_addr, baseline_addr,
        "the golden must ANCHOR divergence: @1's <2;3> alt0 (child 2) must yield a \
         DIFFERENT chain0/idx0 address than the baseline <0;1> alt0 (child 0)"
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
        // P2.3 broadened the message to "hardened use-site path (`/*h` wildcard
        // or a hardened multipath alternative …)" — still the hardened guard.
        .stderr(
            predicate::str::contains("hardened use-site path").and(predicate::str::contains(
                "restore-md1-per-key-use-site-and-hardened-wildcard",
            )),
        );
}
