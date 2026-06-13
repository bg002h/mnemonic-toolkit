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
//! - non-NUMS (cosigner) internal key — SUPPORTED since v0.55.3 for general
//!   single-leaf/depth-1 + distinct-trunk multisig; only the `@-in-both` shape
//!   (trunk key also a leaf key) stays refused
//!   (`restore-non-nums-tr-internal-key-also-in-leaf`, the guard lands in the
//!   next commit),
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

// Non-NUMS ("real key at the trunk") goldens — captured-once from the binary in
// Step 4 (v0.55.3). DO NOT hand-construct (md-codec depth-0 xpub661My…
// reconstructions, not the bundle-input account xpubs). The trunk MUST render
// as a REAL xpub (K2 depth-0), NOT the 50929b74… NUMS hex.
const GOLDEN_DESC_NON_NUMS_GENERAL: &str = "descriptor: tr([28645006/87'/0'/0']xpub661MyMwAqRbcEdy4jr5EtEhQBctfscE6a99DGLr2cW4HnnmBsXDoe3odGzRiw3hcRM5wfKcQmb7s5FjdGrR6SrExXmeopaoY9Lk7tQusDjN/<0;1>/*,and_v(v:pk([73c5da0a/87'/0'/0']xpub661MyMwAqRbcFrooZ2966EcDmVX5MoFXZhuJqXTudvJzwBTBfPQSc5JzX52fvS18oqSdEJXJ4kTGRJ76wPWDUSNJsY5JsgVBQoD6KrbdCLL/<0;1>/*),older(144)))#l2lh2uur";
const GOLDEN_ADDR_NON_NUMS_GENERAL: &str =
    "first recv: bc1pl6nt2ul52gjdtp5lkfgy7l34ux8yv0pc9r7tx8hkjuxrmg6x25ps0nvn6t";
const GOLDEN_DESC_NON_NUMS_MULTI_A: &str = "descriptor: tr([28645006/87'/0'/0']xpub661MyMwAqRbcEdy4jr5EtEhQBctfscE6a99DGLr2cW4HnnmBsXDoe3odGzRiw3hcRM5wfKcQmb7s5FjdGrR6SrExXmeopaoY9Lk7tQusDjN/<0;1>/*,multi_a(2,[73c5da0a/87'/0'/0']xpub661MyMwAqRbcFrooZ2966EcDmVX5MoFXZhuJqXTudvJzwBTBfPQSc5JzX52fvS18oqSdEJXJ4kTGRJ76wPWDUSNJsY5JsgVBQoD6KrbdCLL/<0;1>/*,[b8688df1/87'/0'/0']xpub661MyMwAqRbcEnFgxHRLx7i1fnjcBPgc71qy8mVkbGXYukNGMK2XFRbAaCLYEJDUufNoBxTNa68i5MYhqmrEkfhjzgHCUEcvJBhXS5bk4RW/<0;1>/*))#qtmvesaw";
const GOLDEN_ADDR_NON_NUMS_MULTI_A: &str =
    "first recv: bc1pzzpugq56ylc85m4q90gyyy7dsmdfdlzxdmz3dsvezq4946598s3shxp2j9";
const GOLDEN_DESC_NON_NUMS_SORTEDMULTI_A: &str = "descriptor: tr([28645006/87'/0'/0']xpub661MyMwAqRbcEdy4jr5EtEhQBctfscE6a99DGLr2cW4HnnmBsXDoe3odGzRiw3hcRM5wfKcQmb7s5FjdGrR6SrExXmeopaoY9Lk7tQusDjN/<0;1>/*,sortedmulti_a(2,[73c5da0a/87'/0'/0']xpub661MyMwAqRbcFrooZ2966EcDmVX5MoFXZhuJqXTudvJzwBTBfPQSc5JzX52fvS18oqSdEJXJ4kTGRJ76wPWDUSNJsY5JsgVBQoD6KrbdCLL/<0;1>/*,[b8688df1/87'/0'/0']xpub661MyMwAqRbcEnFgxHRLx7i1fnjcBPgc71qy8mVkbGXYukNGMK2XFRbAaCLYEJDUufNoBxTNa68i5MYhqmrEkfhjzgHCUEcvJBhXS5bk4RW/<0;1>/*))#jpfr0tgx";
// Intentionally identical to GOLDEN_ADDR_NON_NUMS_MULTI_A: sortedmulti_a(2,K0,K1)
// and multi_a(2,K0,K1) produce the same script when {K0,K1} is already in sorted
// order at index 0, so the derived first receive address matches.
const GOLDEN_ADDR_NON_NUMS_SORTEDMULTI_A: &str =
    "first recv: bc1pzzpugq56ylc85m4q90gyyy7dsmdfdlzxdmz3dsvezq4946598s3shxp2j9";

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

// ─── Non-NUMS ("real key at the trunk") faithful reconstruction (v0.55.3) ────

/// (N1) Non-NUMS GENERAL single-leaf tr(D, and_v(v:pk(B),older(N))): the trunk
/// is a real cosigner key (live key-path spend). bundle emits a faithful
/// is_nums:false card; restore reconstructs the descriptor (real trunk key) +
/// a receive address. Golden captured-once from the binary (v0.49.1 precedent).
#[test]
fn non_nums_general_tr_leaf_restores_faithfully() {
    // K2 distinct from the leaf key K0 → not @-in-both.
    let desc = format!("tr({K2},and_v(v:pk({K0}),older(144)))");
    let (md1, emitted) = bundle_md1(&desc);
    assert!(!md1.is_empty(), "non-NUMS general-tr card must be emitted");
    assert_eq!(emitted, desc, "non-NUMS general-tr must round-trip on the wire");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .success()
        .stdout(
            predicate::str::contains(GOLDEN_DESC_NON_NUMS_GENERAL)
                .and(predicate::str::contains(GOLDEN_ADDR_NON_NUMS_GENERAL)),
        );
}

/// (N2) Non-NUMS DISTINCT-trunk multisig tr(D, multi_a(2,B,C)): trunk D NOT a
/// leaf key. Template path + Cosigner(idx). Golden captured-once.
#[test]
fn non_nums_distinct_trunk_multi_a_restores_faithfully() {
    let desc = format!("tr({K2},multi_a(2,{K0},{K1}))");
    let (md1, emitted) = bundle_md1(&desc);
    assert!(!md1.is_empty(), "non-NUMS multisig card must be emitted");
    assert_eq!(emitted, desc, "non-NUMS multisig must round-trip on the wire");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .success()
        .stdout(
            predicate::str::contains(GOLDEN_DESC_NON_NUMS_MULTI_A)
                .and(predicate::str::contains(GOLDEN_ADDR_NON_NUMS_MULTI_A)),
        );
}

/// (N3) Non-NUMS DISTINCT-trunk sortedmulti_a tr(D, sortedmulti_a(2,B,C)):
/// Template path (TrSortedMultiA) routes AROUND md-codec's SortedMultiA gap.
#[test]
fn non_nums_distinct_trunk_sortedmulti_a_restores_faithfully() {
    let desc = format!("tr({K2},sortedmulti_a(2,{K0},{K1}))");
    let (md1, emitted) = bundle_md1(&desc);
    assert!(!md1.is_empty(), "non-NUMS sortedmulti_a card must be emitted");
    assert_eq!(emitted, desc, "non-NUMS sortedmulti_a must round-trip on the wire");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .success()
        .stdout(
            predicate::str::contains(GOLDEN_DESC_NON_NUMS_SORTEDMULTI_A)
                .and(predicate::str::contains(GOLDEN_ADDR_NON_NUMS_SORTEDMULTI_A)),
        );
}

// ─── Refusal contracts (ModeViolation, exit 2, slug-citing) ─────────────────

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

// ─── @-in-both structural guard (funds-safety crux, v0.55.3) ────────────────

/// Build a `tr(@trunk, multi_a(k, <indices>))` wallet-policy `Descriptor`
/// DIRECTLY via md-codec's public tree types — the `@-in-both` family where the
/// non-NUMS trunk key index is ALSO one of the leaf indices.
///
/// This shape CANNOT go through `bundle --descriptor`: intake rejects it at the
/// BIP-388 distinct-key gate ("slot @i and slot @j resolve to identical (xpub,
/// path)"). So the md1 is constructed by hand. Mirrors the direct-construction
/// precedent in `template.rs` (`tr_sortedmulti_a_2_of_2_round_trips_via_md_codec`):
/// the canonical 65-byte synthetic xpub filler (chain_code `[0x42;32]` ‖ SEC1
/// compressed secp256k1 generator G — passes md-codec's `validate_xpub_bytes`),
/// a shared `48'/0'/0'/2'` origin, and `UseSitePath::standard_multipath()`.
///
/// `tlv.pubkeys` is populated with one entry PER slot (`n` total, all the same
/// filler content — the chain_code prefix is unvalidated so distinct slots are
/// allowed) so `is_wallet_policy()` returns true. The card MUST clear the step-2
/// wallet-policy gate (`restore.rs:1163`) and reach `classify_taproot_restore`,
/// else it would trip the WRONG "template-only" refusal and the test would pass
/// for the wrong reason (R0-r3 m1).
///
/// The payload encodes cleanly: `validate_placeholder_usage` registers the trunk
/// index from the `Tr` body first, then the leaf indices (skipping the already-
/// seen trunk), so the `first_occurrences` are canonical and validation passes
/// for `indices = [0..n]` with `trunk = 0` (R0-confirmed).
fn build_at_in_both_descriptor(
    n: u8,
    k: u8,
    leaf_indices: Vec<u8>,
    tag: md_codec::Tag,
) -> md_codec::Descriptor {
    use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
    use md_codec::tree::{Body, Node};
    use md_codec::use_site_path::UseSitePath;
    use md_codec::{Descriptor, Tag, TlvSection};

    // Canonical 65-byte synthetic xpub filler: chain_code = [0x42;32], pubkey =
    // SEC1 compressed secp256k1 generator G (the precedent in `template.rs`).
    let mut xpub_bytes = [0u8; 65];
    xpub_bytes[0..32].copy_from_slice(&[0x42; 32]);
    xpub_bytes[32] = 0x02;
    xpub_bytes[33..].copy_from_slice(&[
        0x79, 0xBE, 0x66, 0x7E, 0xF9, 0xDC, 0xBB, 0xAC, 0x55, 0xA0, 0x62, 0x95, 0xCE, 0x87, 0x0B,
        0x07, 0x02, 0x9B, 0xFC, 0xDB, 0x2D, 0xCE, 0x28, 0xD9, 0x59, 0xF2, 0x81, 0x5B, 0x16, 0xF8,
        0x17, 0x98,
    ]);

    // The @-in-both shape: trunk @0 (is_nums:false) is ALSO a leaf index.
    let tree = Node {
        tag: Tag::Tr,
        body: Body::Tr {
            is_nums: false,
            key_index: 0,
            tree: Some(Box::new(Node {
                tag,
                body: Body::MultiKeys {
                    k,
                    indices: leaf_indices,
                },
            })),
        },
    };

    let path = OriginPath {
        components: vec![
            PathComponent {
                hardened: true,
                value: 48,
            },
            PathComponent {
                hardened: true,
                value: 0,
            },
            PathComponent {
                hardened: true,
                value: 0,
            },
            PathComponent {
                hardened: true,
                value: 2,
            },
        ],
    };

    // One filler fingerprint + pubkey per slot (synthetic, distinct fingerprints).
    let fingerprints: Vec<(u8, [u8; 4])> = (0..n).map(|i| (i, [i, 0xBB, 0xCC, 0xDD])).collect();
    let pubkeys: Vec<(u8, [u8; 65])> = (0..n).map(|i| (i, xpub_bytes)).collect();

    Descriptor {
        n,
        path_decl: PathDecl {
            n,
            paths: PathDeclPaths::Shared(path),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: Some(fingerprints),
            // Non-empty → is_wallet_policy() == true (clears the step-2 gate).
            pubkeys: Some(pubkeys),
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    }
}

/// (N4) @-in-both refusal — the FUNDS-SAFETY RED-proof: `tr(@0, multi_a(2, @0,
/// @1, @2))`, where the non-NUMS trunk key index `@0` is ALSO a leaf index. The
/// Template `Cosigner(idx)` shortcut reconstructs the leaf as `{all cosigners
/// EXCEPT idx}` WITHOUT lowering `k`, so without the guard it emits
/// `multi_a(2, @1, @2)` — dropping the trunk key. For an `n ≥ 3` leaf this is a
/// VALID 2-of-2 (k ≤ n): the reconstruction SUCCEEDS (exit 0) and prints a
/// DIFFERENT, silently-wrong multisig at a DIFFERENT address. The Display-
/// fidelity guard CANNOT catch this — the Template path's output is its own
/// re-print (parse→print of the rendered string), so a wrong-but-self-consistent
/// leaf passes. The protection MUST therefore be a STRUCTURAL classify-time
/// precondition. (RED-proven: with the guard removed, this card exits 0 with the
/// trunk-dropped `multi_a(2, @1, @2)`; the plan's 2-of-2 shape is NOT a valid
/// RED — dropping @0 there yields a 2-of-1 that k>n rejects downstream, see the
/// `_2of2` cell below.)
///
/// `bundle --descriptor` rejects this shape at intake (BIP-388 distinct-key
/// gate), so the md1 is built directly via md_codec (`build_at_in_both_descriptor`).
#[test]
fn at_in_both_tr_refuses_structurally() {
    // n=3, 2-of-3: dropping the trunk @0 leaves a VALID 2-of-2 → the dangerous
    // exit-0 silent-wrong reconstruction the structural guard must prevent.
    let d = build_at_in_both_descriptor(3, 2, vec![0, 1, 2], md_codec::Tag::MultiA);
    let chunks = md_codec::chunk::split(&d).expect("split @-in-both md1");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&chunks))
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("restore-non-nums-tr-internal-key-also-in-leaf")
                .and(predicate::str::contains("also a leaf key")),
        );
}

/// (N4b) @-in-both refusal — the degenerate 2-of-2 shape `tr(@0, multi_a(2, @0,
/// @1))`. Also `@-in-both`, so the guard refuses it identically (slug + "also a
/// leaf key"). NOTE: without the guard this shape does NOT exit 0 — dropping @0
/// from a 2-key leaf yields `multi_a(2, @1)` (2-of-1), which miniscript rejects
/// downstream as k>n (exit 2, a DIFFERENT message). So it is not a funds-safety
/// RED on its own (the genuine RED is the `n ≥ 3` cell above); this cell pins
/// that the STRUCTURAL guard catches the shape at classify time regardless, with
/// the correct slug — never relying on the coincidental downstream k>n catch.
#[test]
fn at_in_both_tr_2of2_refuses_structurally() {
    let d = build_at_in_both_descriptor(2, 2, vec![0, 1], md_codec::Tag::MultiA);
    let chunks = md_codec::chunk::split(&d).expect("split @-in-both 2-of-2 md1");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&chunks))
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("restore-non-nums-tr-internal-key-also-in-leaf")
                .and(predicate::str::contains("also a leaf key")),
        );
}

/// (N4c) Same @-in-both shape but with a SortedMultiA leaf — proves the guard
/// covers BOTH Template arms (catches a one-arm regression).
#[test]
fn at_in_both_sortedmulti_a_refuses_structurally() {
    let d = build_at_in_both_descriptor(3, 2, vec![0, 1, 2], md_codec::Tag::SortedMultiA);
    let chunks = md_codec::chunk::split(&d).expect("split @-in-both sortedmulti_a md1");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&chunks))
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("restore-non-nums-tr-internal-key-also-in-leaf")
                .and(predicate::str::contains("also a leaf key")),
        );
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

/// (9) `--format bip388` on a reconstructed general-tr policy: refused loudly.
/// Since v0.55.3 the refusal is the EXPLICIT taproot route-around guard in the
/// `None` (general) arm of `build_multisig_import_payload` — a tap-script-tree
/// reconstructed via the route-around has no named-template form, so it cannot
/// be expressed as a BIP-388 wallet policy. (Previously this was an INCIDENTAL
/// failure: the NUMS internal key is a bare x-only `Single` with no `/<0;1>/*`
/// suffix; that incidental mechanism does NOT catch a non-NUMS multipath trunk,
/// which is the hole the unified explicit guard closes.) Exit 1 (BadInput).
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
        .stderr(predicate::str::contains("BIP-388 wallet policy"));
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

// ─── Non-NUMS taproot `--format` matrix (v0.55.3) ───────────────────────────

/// (N5) Non-NUMS general-tr `--format bip388` → refused (the general
/// route-around arm cannot express a tap-script tree as a BIP-388 wallet
/// policy). Exit 1 (BadInput). This closes the non-NUMS multipath-trunk hole:
/// the trunk is a real multipath xpub, so the incidental no-multipath refusal
/// (which catches the NUMS x-only Single) does NOT fire here.
#[test]
fn non_nums_general_tr_format_bip388_refused() {
    let desc = format!("tr({K2},and_v(v:pk({K0}),older(144)))");
    let (md1, _e) = bundle_md1(&desc);
    let mut a = restore_args(&md1);
    a.push("--format".into());
    a.push("bip388".into());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("BIP-388 wallet policy"));
}

/// (N6) Non-NUMS DISTINCT-trunk multisig `--format bip388` → SUCCEEDS (Template
/// path + `bip388.rs` `Cosigner(idx)` arm emits `tr(@idx/**,multi_a(k,…))`).
/// The Some(t) Template branch never reaches the route-around guard.
#[test]
fn non_nums_distinct_trunk_multi_a_format_bip388_succeeds() {
    let desc = format!("tr({K2},multi_a(2,{K0},{K1}))");
    let (md1, _e) = bundle_md1(&desc);
    let mut a = restore_args(&md1);
    a.push("--format".into());
    a.push("bip388".into());
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    // The trunk cosigner is at @-some-index; the emitted policy is a tr(@i/**,…).
    assert!(
        s.contains("tr(@") && s.contains("multi_a("),
        "bip388 wallet policy must carry tr(@idx/**,multi_a(…)): {s}"
    );
}

/// (N7) Non-NUMS `--format descriptor` / `bitcoin-core` → emit faithfully (both
/// the general route-around AND the distinct-trunk Template arm).
#[test]
fn non_nums_format_descriptor_and_bitcoin_core_emit() {
    for desc in [
        format!("tr({K2},and_v(v:pk({K0}),older(144)))"),
        format!("tr({K2},multi_a(2,{K0},{K1}))"),
    ] {
        let (md1, _e) = bundle_md1(&desc);
        for fmt in ["descriptor", "bitcoin-core"] {
            let mut a = restore_args(&md1);
            a.push("--format".into());
            a.push(fmt.into());
            Command::cargo_bin("mnemonic")
                .unwrap()
                .args(&a)
                .assert()
                .success();
        }
    }
}

/// (N8) Non-NUMS general-tr `--format green` → refused (existing P2tr green
/// gate in the `None` branch).
#[test]
fn non_nums_general_tr_format_green_refused() {
    let desc = format!("tr({K2},and_v(v:pk({K0}),older(144)))");
    let (md1, _e) = bundle_md1(&desc);
    let mut a = restore_args(&md1);
    a.push("--format".into());
    a.push("green".into());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .failure();
}
