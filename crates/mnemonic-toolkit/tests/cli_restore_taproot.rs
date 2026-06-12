//! GAP 1 — taproot restore contracts: faithful reconstruction (T3-partial,
//! v0.55.1) + the remaining refusal arms.
//!
//! `bundle --descriptor` emits a FAITHFUL md1 for taproot policies beyond
//! single-leaf NUMS `multi_a`/`sortedmulti_a`. Since v0.55.1, `restore --md1`
//! faithfully reconstructs the Display-safe subset — single-leaf and depth-1
//! two-leaf `tr(NUMS,…)` general policies — via the same general-policy arm as
//! wsh (`faithful_multisig_descriptor`); the md-codec NUMS `Single` internal
//! key passes through `ReconstructTranslator` strict-NUMS-only.
//!
//! Still refused (each pinned below, all `ModeViolation` exit 2, slug-citing):
//! - non-NUMS (cosigner) internal key (`restore-multisig-taproot-reconstruction`),
//! - depth ≥2 / ≥3 leaves — STRUCTURAL, chirality-independent: the pinned
//!   miniscript 95fdd1c mis-Displays only a LEFT-child `TapTree`, but the gate
//!   refuses right-spine shapes too (never Display-luck; lift on the
//!   miniscript #953 release — `upstream-miniscript-taptree-depth2-display-asymmetry`),
//! - `sortedmulti_a` under a `TapTree`
//!   (`md-codec-sortedmulti-a-to-miniscript-rendering-gap`).
//!
//! The reconstructed descriptor is asymmetric to the bundle input ON PURPOSE:
//! md-codec wallet-policy cards carry [chain_code‖pubkey] (no depth/parent), so
//! restore prints depth-0 `xpub661My…` keys + the `<0;1>/*` multipath + the
//! NUMS H-point HEX (not the literal `NUMS` token). Goldens derived once from
//! the binary, eyeballed, pinned (v0.49.1 precedent).
//!
//! FOLLOWUP `restore-general-and-multi-leaf-taproot-roundtrip` (T3-partial).

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

// The 3-cosigner trio, lifted from `cli_bundle_import_json.rs:312-314`: three
// distinct fingerprints, three distinct xpubs, all `[fp/87'/0'/0']…/<0;1>/*`
// watch-only (concrete `bundle --descriptor`, no seed).
const K0: &str = "[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*";
const K1: &str = "[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*";
const K2: &str = "[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*";

/// BIP-341 NUMS H-point, x-only hex — the reconstructed internal key.
const NUMS_HEX: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

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

// ─── Faithful reconstruction (T3-partial, v0.55.1) ──────────────────────────

// Goldens: derive-once-then-pin from the binary (NEVER hand-constructed by
// hex-substituting the input — the keys are md-codec depth-0 reconstructions
// `xpub661My…`, NOT the bundle-input account xpubs; v0.49.1 I2 trap).
const GOLDEN_DESC_SINGLE_LEAF: &str = "descriptor: tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,and_v(v:pk([73c5da0a/87'/0'/0']xpub661MyMwAqRbcFrooZ2966EcDmVX5MoFXZhuJqXTudvJzwBTBfPQSc5JzX52fvS18oqSdEJXJ4kTGRJ76wPWDUSNJsY5JsgVBQoD6KrbdCLL/<0;1>/*),after(12000000)))#lvyngt4k";
const GOLDEN_ADDR_SINGLE_LEAF: &str =
    "first recv: bc1pq0x9jpvsdkmw3gd87xznly7yxdgt0u4mmchhuyjqv2eckhxd0znq305vxy";
const GOLDEN_DESC_TWO_LEAF: &str = "descriptor: tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{pk([73c5da0a/87'/0'/0']xpub661MyMwAqRbcFrooZ2966EcDmVX5MoFXZhuJqXTudvJzwBTBfPQSc5JzX52fvS18oqSdEJXJ4kTGRJ76wPWDUSNJsY5JsgVBQoD6KrbdCLL/<0;1>/*),pk([b8688df1/87'/0'/0']xpub661MyMwAqRbcEnFgxHRLx7i1fnjcBPgc71qy8mVkbGXYukNGMK2XFRbAaCLYEJDUufNoBxTNa68i5MYhqmrEkfhjzgHCUEcvJBhXS5bk4RW/<0;1>/*)})#p24tk237";
const GOLDEN_ADDR_TWO_LEAF: &str =
    "first recv: bc1prgf4vyj0tgqwykeg3hzxrzk5cqtc0awjqg2dh8ksts77tuz20xpqkms0ch";
const GOLDEN_DESC_MULTI_A_2LEAF: &str = "descriptor: tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(2,[73c5da0a/87'/0'/0']xpub661MyMwAqRbcFrooZ2966EcDmVX5MoFXZhuJqXTudvJzwBTBfPQSc5JzX52fvS18oqSdEJXJ4kTGRJ76wPWDUSNJsY5JsgVBQoD6KrbdCLL/<0;1>/*,[b8688df1/87'/0'/0']xpub661MyMwAqRbcEnFgxHRLx7i1fnjcBPgc71qy8mVkbGXYukNGMK2XFRbAaCLYEJDUufNoBxTNa68i5MYhqmrEkfhjzgHCUEcvJBhXS5bk4RW/<0;1>/*),pk([28645006/87'/0'/0']xpub661MyMwAqRbcEdy4jr5EtEhQBctfscE6a99DGLr2cW4HnnmBsXDoe3odGzRiw3hcRM5wfKcQmb7s5FjdGrR6SrExXmeopaoY9Lk7tQusDjN/<0;1>/*)})#c4ux7rlz";
const GOLDEN_ADDR_MULTI_A_2LEAF: &str =
    "first recv: bc1pw49n2w6ydsnmdcufryu6r3agpw3k4hmkhkzfr9z3wqs3xuvx74as88s86m";

/// (1) General taproot leaf `tr(NUMS, <non-multisig miniscript>)`: bundle emits
/// a faithful card (`.descriptor` round-trips EXACTLY — the literal `NUMS`
/// token is preserved on the wire, no substitution), and restore reconstructs
/// it faithfully (golden descriptor + golden bc1p first address).
#[test]
fn general_tr_leaf_restores_faithfully() {
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
        .success()
        .stdout(
            predicate::str::contains(GOLDEN_DESC_SINGLE_LEAF)
                .and(predicate::str::contains(GOLDEN_ADDR_SINGLE_LEAF)),
        );
}

/// (2) Depth-1 two-leaf taptree `tr(NUMS,{pk(K0),pk(K1)})`: bundle emits
/// faithfully (the wire round-trips), and restore reconstructs faithfully.
#[test]
fn two_leaf_taptree_restores_faithfully() {
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
        .success()
        .stdout(
            predicate::str::contains(GOLDEN_DESC_TWO_LEAF)
                .and(predicate::str::contains(GOLDEN_ADDR_TWO_LEAF)),
        );
}

/// (3) `multi_a` under a 2-leaf TapTree `tr(NUMS,{multi_a(2,K0,K1),pk(K2)})`
/// routes the GENERAL arm (NOT the single-leaf Template path — the root tag is
/// `TapTree`) and reconstructs faithfully; pins the k/threshold interaction
/// (`extract_multisig_threshold` walks `Body::Tr` → `Some(2)` here, benign).
#[test]
fn multi_a_in_2leaf_tr_restores_faithfully() {
    let desc = format!("tr(NUMS,{{multi_a(2,{K0},{K1}),pk({K2})}})");
    let (md1, emitted) = bundle_md1(&desc);
    assert_eq!(emitted, desc, "multi_a-bearing taptree must round-trip");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .success()
        .stdout(
            predicate::str::contains(GOLDEN_DESC_MULTI_A_2LEAF)
                .and(predicate::str::contains(GOLDEN_ADDR_MULTI_A_2LEAF)),
        );
}

// ─── Refusal contracts (ModeViolation, exit 2, slug-citing) ─────────────────

/// (4) Non-NUMS (cosigner) internal-key taproot multisig `tr(K2, multi_a(2,K0,K1))`
/// with a DISTINCT internal key: bundle emits, restore refuses with exit 2
/// "non-NUMS (cosigner) internal key". (A non-distinct internal key —
/// `tr(K0, multi_a(2,K0,K1))` — is rejected EARLIER by bundle's BIP-388
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

/// (5) Left-heavy 3-leaf (depth-2) taptree: bundle emits faithfully, restore
/// refuses STRUCTURALLY (exit 2) citing the upstream Display-asymmetry slug.
#[test]
fn left_heavy_3leaf_tr_refuses_depth2() {
    let desc = format!("tr(NUMS,{{{{pk({K0}),pk({K1})}},pk({K2})}})");
    let (md1, emitted) = bundle_md1(&desc);
    assert_eq!(
        emitted, desc,
        "depth-2 card must still be a faithful backup"
    );
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "upstream-miniscript-taptree-depth2-display-asymmetry",
        ));
}

/// (6) Right-spine 3-leaf (depth-2) taptree ALSO refuses — the gate is
/// STRUCTURAL (chirality-independent), even though a right-spine Display
/// happens to work on 95fdd1c. Never Display-luck.
#[test]
fn right_spine_3leaf_tr_also_refuses_depth2() {
    let desc = format!("tr(NUMS,{{pk({K0}),{{pk({K1}),pk({K2})}}}})");
    let (md1, emitted) = bundle_md1(&desc);
    assert_eq!(
        emitted, desc,
        "right-spine card must still be a faithful backup"
    );
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "upstream-miniscript-taptree-depth2-display-asymmetry",
        ));
}

/// (7) `sortedmulti_a` under a 2-leaf TapTree: refused citing the md-codec
/// rendering-gap slug (single-leaf `sortedmulti_a` still routes the Template
/// path and reconstructs — pinned elsewhere).
#[test]
fn tr_sortedmulti_a_in_2leaf_refuses() {
    let desc = format!("tr(NUMS,{{sortedmulti_a(2,{K0},{K1}),pk({K2})}})");
    let (md1, emitted) = bundle_md1(&desc);
    assert_eq!(
        emitted, desc,
        "sortedmulti_a-bearing card must still be a faithful backup"
    );
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "md-codec-sortedmulti-a-to-miniscript-rendering-gap",
        ));
}

// ─── --format matrix for the general-tr arm (R0 I1) ─────────────────────────

/// (8) `--format green` on a reconstructed general-tr policy: REFUSED exit 1.
/// Without the explicit refusal, `script_type_from_descriptor` classifies a
/// general tr (no `multi_a(` substring) as `P2tr` — taproot SINGLESIG — and
/// green would emit a "singlesig" payload for a script-tree policy.
#[test]
fn general_tr_format_green_refused() {
    let desc = format!("tr(NUMS,{{pk({K0}),pk({K1})}})");
    let (md1, _emitted) = bundle_md1(&desc);
    let mut a = restore_args(&md1);
    a.push("--format".into());
    a.push("green".into());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("singlesig-only"));
}

/// (8b) The `multi_a`-bearing shape classifies `P2trMulti` instead and is
/// caught by green's EXISTING `is_multisig` refusal (a DIFFERENT message) —
/// pins the P2trMulti side of the matrix (R0-r2 m3).
#[test]
fn multi_a_2leaf_tr_format_green_refused_as_multisig() {
    let desc = format!("tr(NUMS,{{multi_a(2,{K0},{K1}),pk({K2})}})");
    let (md1, _emitted) = bundle_md1(&desc);
    let mut a = restore_args(&md1);
    a.push("--format".into());
    a.push("green".into());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("does not support multisig"));
}

/// (9) `--format bip388` on a reconstructed general-tr policy: refused loudly —
/// the NUMS internal key is a bare x-only `Single` with no `/<0;1>/*` suffix,
/// which BIP-388 wallet policies require on every key. Pins today's loud
/// refusal (NOT silent-wrong); message-quality nit may ride a future cycle.
#[test]
fn general_tr_format_bip388_refused() {
    let desc = format!("tr(NUMS,{{pk({K0}),pk({K1})}})");
    let (md1, _emitted) = bundle_md1(&desc);
    let mut a = restore_args(&md1);
    a.push("--format".into());
    a.push("bip388".into());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("/<0;1>/*"));
}

/// (10) Descriptor-driven formats emit the faithful reconstruction.
#[test]
fn general_tr_format_descriptor_and_bitcoin_core_emit() {
    let desc = format!("tr(NUMS,{{pk({K0}),pk({K1})}})");
    let (md1, _emitted) = bundle_md1(&desc);
    for fmt in ["descriptor", "bitcoin-core"] {
        let mut a = restore_args(&md1);
        a.push("--format".into());
        a.push(fmt.into());
        let out = Command::cargo_bin("mnemonic")
            .unwrap()
            .args(&a)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let payload = String::from_utf8(out).expect("utf8 payload");
        assert!(
            payload.contains(NUMS_HEX),
            "--format {fmt} payload must carry the NUMS H-point hex: {payload}"
        );
        assert!(
            !payload.contains("xprv"),
            "--format {fmt} payload must be watch-only"
        );
    }
}
