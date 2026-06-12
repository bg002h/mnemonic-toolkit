//! v0.44.0 — `mnemonic restore` multisig-cosigner (FOLLOWUP
//! `restore-multisig-cosigner-scope`). A wallet-policy `md1` reconstructs the
//! concrete watch-only multisig descriptor (md1 alone); `--from`/`--cosigner`
//! are optional cross-check inputs. Scope: wsh + sh(wsh); taproot NUMS
//! multisig (tr-multi-a / tr-sortedmulti-a) reconstructed since v2 (the
//! address path routes around md-codec — `restore-multisig-taproot-reconstruction`).
//! See design/SPEC_restore_multisig_cosigner.md.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

const C0: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const C1: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const C2: &str = "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";
/// A seed that is NOT one of the three cosigners (for the mismatch cell).
const FOREIGN: &str = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";

/// Bundle a 2-of-3 multisig and return (md1 chunks, per-cosigner mk1 chunks).
fn bundle_multisig(template: &str, network: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            template,
            "--threshold",
            "2",
            "--network",
            network,
            "--slot",
            &format!("@0.phrase={C0}"),
            "--slot",
            &format!("@1.phrase={C1}"),
            "--slot",
            &format!("@2.phrase={C2}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).expect("bundle JSON");
    let md1: Vec<String> = v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    // mk1 is a per-cosigner array; each element is a String (1 chunk) or Array (chunks).
    let mk1_per: Vec<Vec<String>> = v["mk1"]
        .as_array()
        .expect("mk1 array")
        .iter()
        .map(|el| match el {
            Value::String(s) => vec![s.clone()],
            Value::Array(inner) => inner
                .iter()
                .map(|c| c.as_str().unwrap().to_string())
                .collect(),
            other => panic!("unexpected mk1 element: {other:?}"),
        })
        .collect();
    (md1, mk1_per)
}

fn restore_args(md1: &[String]) -> Vec<String> {
    let mut a = vec!["restore".to_string(), "--network".into(), "mainnet".into()];
    for c in md1 {
        a.push("--md1".into());
        a.push(c.clone());
    }
    a
}

/// (1) `--md1` alone → concrete wsh(sortedmulti) `<0;1>/*` descriptor + checksum
/// + a first receive address + UNVERIFIED.
#[test]
fn md1_alone_emits_descriptor_unverified() {
    let (md1, _) = bundle_multisig("wsh-sortedmulti", "mainnet");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .code(0)
        .stdout(
            predicate::str::contains("wsh(sortedmulti(2,")
                .and(predicate::str::contains("<0;1>/*"))
                .and(predicate::str::contains("#"))
                .and(predicate::str::contains("bc1q")),
        )
        .stderr(predicate::str::contains("UNVERIFIED"));
}

/// (2) `--md1 --from <cosigner-0 phrase>` → own position @0 inferred + labeled
/// "your seed"; the OTHER positions are NOT labeled cross-checked (C1) → PARTIAL.
#[test]
fn md1_with_own_seed_partial_only_own_verified() {
    let (md1, _) = bundle_multisig("wsh-sortedmulti", "mainnet");
    let mut a = restore_args(&md1);
    a.push("--from".into());
    a.push(format!("phrase={C0}"));
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(0)
        .stdout(
            predicate::str::contains("your seed")
                // @1 and @2 were NOT cross-checked → must NOT claim verification.
                .and(predicate::str::contains("not independently verified")),
        )
        // Only 1 of 3 cosigners independently verified → PARTIAL, not "verified".
        .stderr(predicate::str::contains("PARTIAL"));
}

/// (3) `--md1 --cosigner @1=<mk1>` → @1 cross-checked; @0/@2 NOT (C1) → PARTIAL.
#[test]
fn md1_with_cosigner_mk1_partial() {
    let (md1, mk1_per) = bundle_multisig("wsh-sortedmulti", "mainnet");
    let mut a = restore_args(&md1);
    for chunk in &mk1_per[1] {
        a.push("--cosigner".into());
        a.push(format!("@1={chunk}"));
    }
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(0)
        .stdout(
            predicate::str::contains("cross-checked")
                .and(predicate::str::contains("not independently verified")),
        )
        .stderr(predicate::str::contains("PARTIAL"));
}

/// (3c) `--json` envelope carries the C1-correct partial status + per-position
/// notes (the JSON path was part of the C1 attack surface).
#[test]
fn md1_json_partial_status_and_notes() {
    let (md1, _) = bundle_multisig("wsh-sortedmulti", "mainnet");
    let mut a = restore_args(&md1);
    a.push("--from".into());
    a.push(format!("phrase={C0}"));
    a.push("--json".into());
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(0);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).expect("restore JSON");
    assert_eq!(
        v["verification"]["status"], "partial",
        "only @0 verified → partial"
    );
    let cos = v["wallets"][0]["cosigner_keys"]
        .as_array()
        .expect("cosigner_keys");
    // @0 is the own seed; at least one other position must be flagged unverified.
    let own = cos.iter().find(|c| c["position"] == 0).expect("@0");
    assert!(
        own["note"].as_str().unwrap().contains("your seed"),
        "@0 own-seed note"
    );
    assert!(
        cos.iter().any(|c| c["note"]
            .as_str()
            .unwrap()
            .contains("not independently verified")),
        "an un-supplied position must be flagged not-independently-verified"
    );
}

/// (3b) ALL positions cross-checked (own seed @0 + mk1 @1 + mk1 @2) → fully
/// "verified": NO "not independently verified", NO PARTIAL/UNVERIFIED banner.
#[test]
fn md1_all_cosigners_verified_no_partial() {
    let (md1, mk1_per) = bundle_multisig("wsh-sortedmulti", "mainnet");
    let mut a = restore_args(&md1);
    a.push("--from".into());
    a.push(format!("phrase={C0}"));
    for chunk in &mk1_per[1] {
        a.push("--cosigner".into());
        a.push(format!("@1={chunk}"));
    }
    for chunk in &mk1_per[2] {
        a.push("--cosigner".into());
        a.push(format!("@2={chunk}"));
    }
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(0)
        .stdout(predicate::str::contains("not independently verified").not())
        .stderr(
            predicate::str::contains("PARTIAL")
                .not()
                .and(predicate::str::contains("UNVERIFIED").not()),
        );
}

/// (4) `--md1 --from <foreign seed>` → RestoreMismatch (exit 4).
#[test]
fn md1_with_foreign_seed_mismatch_exit4() {
    let (md1, _) = bundle_multisig("wsh-sortedmulti", "mainnet");
    let mut a = restore_args(&md1);
    a.push("--from".into());
    a.push(format!("phrase={FOREIGN}"));
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(4)
        .stderr(predicate::str::contains("MISMATCH"));
}

/// (5) `--allow-mismatch` overrides the foreign-seed mismatch → exit 0 + banner.
#[test]
fn md1_foreign_seed_allow_mismatch_exit0() {
    let (md1, _) = bundle_multisig("wsh-sortedmulti", "mainnet");
    let mut a = restore_args(&md1);
    a.push("--from".into());
    a.push(format!("phrase={FOREIGN}"));
    a.push("--allow-mismatch".into());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(0)
        .stderr(predicate::str::contains("MISMATCH (overridden)"));
}

/// (6) `sh-wsh-sortedmulti` 2-of-3 → `sh(wsh(sortedmulti(2,` (non-BIP-87 origin).
#[test]
fn sh_wsh_multisig_descriptor() {
    let (md1, _) = bundle_multisig("sh-wsh-sortedmulti", "mainnet");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .code(0)
        .stdout(predicate::str::contains("sh(wsh(sortedmulti(2,"));
}

/// NUMS H-point x-only hex (BIP-341) — the taproot internal key bundle emits
/// (v0.48.0) and restore reconstructs.
const NUMS_HEX: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// (7a) `tr-sortedmulti-a` md1 → `tr(NUMS, sortedmulti_a(2,…))` + `bc1p` first
/// address (v2 — supersedes the v0.44.0 refusal). LOAD-BEARING: the address is
/// derived from the reconstructed descriptor STRING, routing around md-codec's
/// `to_miniscript` (which errors on `SortedMultiA`) — `d.derive_address` would
/// hard-fail here (R0 v2 C1).
#[test]
fn tr_sortedmulti_a_reconstructs_nums_descriptor_and_bc1p() {
    let (md1, _) = bundle_multisig("tr-sortedmulti-a", "mainnet");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .code(0)
        .stdout(
            predicate::str::contains(format!("tr({NUMS_HEX},sortedmulti_a(2,"))
                .and(predicate::str::contains("<0;1>/*"))
                .and(predicate::str::contains("#"))
                // Golden first receive address (C0/C1/C2 fixtures, mainnet) —
                // pins the exact bc1p, catching any wrong key / cosigner order /
                // internal-key regression. Captured from a verified-correct run
                // (the construction is the proven wsh build path + NUMS).
                .and(predicate::str::contains(
                    "bc1p550zvnachy40z6hh8llka93mkm0c3635samp264ck6rfd0dcdc8s00n8c8",
                )),
        )
        .stderr(predicate::str::contains("UNVERIFIED"));
}

/// (7b) `tr-multi-a` md1 → `tr(NUMS, multi_a(2,…))` + `bc1p`.
#[test]
fn tr_multi_a_reconstructs_nums_descriptor_and_bc1p() {
    let (md1, _) = bundle_multisig("tr-multi-a", "mainnet");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .code(0)
        .stdout(
            predicate::str::contains(format!("tr({NUMS_HEX},multi_a(2,"))
                // Golden first receive address (order-significant multi_a) —
                // pins cosigner order + key + NUMS.
                .and(predicate::str::contains(
                    "bc1p6fy2e76ele2vwrhdpsu6l9cu3ayc9h6me6wgkeu78qkc5r6rpzas8e5cak",
                )),
        )
        .stderr(predicate::str::contains("UNVERIFIED"));
}

/// (8) watch-only-out: NO private material (xprv/WIF) in stdout or stderr.
#[test]
fn md1_watch_only_no_private_material() {
    let (md1, _) = bundle_multisig("wsh-sortedmulti", "mainnet");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args({
            let mut a = restore_args(&md1);
            a.push("--from".into());
            a.push(format!("phrase={C0}"));
            a
        })
        .assert()
        .code(0);
    let o = out.get_output();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(!combined.contains("xprv"), "no xprv in output");
    assert!(!combined.contains("tprv"), "no tprv in output");
}

/// (9) `--network testnet` → testnet `tpub` cosigners (the --network-authoritative
/// Xpub reconstruction; restore is the first expand_per_at_n reconstruction site).
#[test]
fn md1_testnet_emits_tpub() {
    let (md1, _) = bundle_multisig("wsh-sortedmulti", "testnet");
    let mut a = vec!["restore".to_string(), "--network".into(), "testnet".into()];
    for c in &md1 {
        a.push("--md1".into());
        a.push(c.clone());
    }
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(0)
        .stdout(predicate::str::contains("tpub").and(predicate::str::contains("xpub").not()));
}

/// (10) round-trip convergence: the restored descriptor is accepted by
/// `bundle --descriptor` (proves the reconstruction is a valid concrete
/// descriptor, incl. depth-0 cosigner xpubs).
#[test]
fn restored_descriptor_round_trips_through_bundle() {
    let (md1, _) = bundle_multisig("wsh-sortedmulti", "mainnet");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args({
            let mut a = restore_args(&md1);
            a.push("--json".into());
            a
        })
        .assert()
        .code(0);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).expect("restore JSON");
    let desc = v["wallets"][0]["descriptor"].as_str().expect("descriptor");
    // Feed the bare concrete descriptor back to bundle (A1 descriptor-form door).
    Command::cargo_bin("mnemonic")
        .unwrap()
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
}
