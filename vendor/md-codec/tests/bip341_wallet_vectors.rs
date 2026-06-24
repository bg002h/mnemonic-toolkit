//! BIP-341 wallet-test-vectors.json — taproot output-key + bech32m
//! address pinning.
//!
//! md-codec's wallet-policy descriptor pipeline compiles `tr(K, {…})`
//! templates by delegating taproot tree assembly + output-key tweaking
//! to `bitcoin` (the `rust-bitcoin` crate). BIP-86's reference vectors
//! exercise only the *key-spend-only* path (`scriptTree == null`);
//! script-tree assembly + nested-branch merkle computation are
//! unverified against an upstream-authoritative source at v0.7.1.
//!
//! This file pins the canonical
//! `https://github.com/bitcoin/bips/blob/master/bip-0341/wallet-test-vectors.json`
//! `scriptPubKey` array (7 entries) against md-codec's transitive
//! `bitcoin v0.32` taproot path. A future `bitcoin` bump that silently
//! drifts taproot tweaking (e.g., a leaf-version semantics change, a
//! merkle-root construction order change) fails these cells.
//!
//! Fixture pin: `tests/vectors/bip341-wallet-test-vectors.json`,
//! `sha256sum = 403e19fb81dd1f31e745699216308f61fb403774b2aafa87b631b8f7c042d37f`.
//! If the SHA changes, the fixture was re-fetched and may carry a
//! different vector set; rebase this test against the new SHA before
//! merging.
//!
//! The companion `keyPathSpending` array (1 vector) is
//! OUT-OF-SCOPE-PER-LAYER — no Schnorr signing surface in the
//! constellation; filed as FOLLOWUP
//! `bip341-keypath-signing-vector-coverage`.
//!
//! Cycle: v0.8.0 BIP-vector adoption.
//! SPEC: `mnemonic-toolkit/design/SPEC_test_vector_audit_v0_8_0.md` §2.
//! Phase: 1.

use bitcoin::key::{Secp256k1, UntweakedPublicKey};
use bitcoin::secp256k1::XOnlyPublicKey;
use bitcoin::taproot::{LeafVersion, TaprootBuilder, TaprootSpendInfo};
use bitcoin::{Address, KnownHrp, ScriptBuf};

use serde_json::Value;

const FIXTURE: &str = include_str!("vectors/bip341-wallet-test-vectors.json");
const FIXTURE_SHA256: &str = "403e19fb81dd1f31e745699216308f61fb403774b2aafa87b631b8f7c042d37f";

fn vectors() -> Vec<Value> {
    let root: Value = serde_json::from_str(FIXTURE).expect("fixture parses");
    root["scriptPubKey"]
        .as_array()
        .expect("scriptPubKey array")
        .clone()
}

#[test]
fn fixture_sha256_pin() {
    use bitcoin::hashes::{Hash, sha256};
    let actual = sha256::Hash::hash(FIXTURE.as_bytes()).to_string();
    assert_eq!(
        actual, FIXTURE_SHA256,
        "BIP-341 fixture SHA drifted; vectors may have changed upstream — \
         re-audit before bumping FIXTURE_SHA256",
    );
}

#[test]
fn scriptpubkey_array_length_is_seven() {
    // SPEC §2 invariant: BIP-341 `scriptPubKey` array length is 7.
    // If upstream adds an 8th vector, this guard fires and forces a
    // cycle-bump to extend the test cells below to match.
    assert_eq!(vectors().len(), 7);
}

/// Recursively walk BIP-341's scriptTree representation and emit
/// (depth, script, leaf_version) tuples in DFS order. The BIP shape is:
///
/// - `null` → no leaves (key-spend only path)
/// - `{id, script, leafVersion}` → single leaf at the current depth
/// - `[left, right]` → branch; recurse into both children at depth+1
///
/// `bitcoin::taproot::TaprootBuilder::add_leaf_with_ver` consumes
/// exactly this DFS order to reconstruct the merkle tree.
fn walk_tree(tree: &Value, depth: u8, leaves: &mut Vec<(u8, ScriptBuf, LeafVersion)>) {
    if tree.is_null() {
        return;
    }
    if let Some(arr) = tree.as_array() {
        assert_eq!(arr.len(), 2, "branch nodes must have exactly 2 children");
        walk_tree(&arr[0], depth + 1, leaves);
        walk_tree(&arr[1], depth + 1, leaves);
        return;
    }
    let obj = tree.as_object().expect("leaf is an object");
    let script_hex = obj["script"].as_str().expect("script is hex string");
    let leaf_ver_u8 = obj["leafVersion"].as_u64().expect("leafVersion is u8") as u8;
    let script = ScriptBuf::from_bytes(hex::decode(script_hex).expect("script hex parses"));
    let leaf_ver = LeafVersion::from_consensus(leaf_ver_u8)
        .unwrap_or_else(|_| panic!("leaf version {leaf_ver_u8:#x} rejected by bitcoin v0.32"));
    leaves.push((depth, script, leaf_ver));
}

/// Derive (tweaked output x-only pubkey, bech32m mainnet address) for a
/// single BIP-341 wallet-test-vector entry.
fn derive(vector: &Value) -> (XOnlyPublicKey, String) {
    let secp = Secp256k1::verification_only();
    let internal_hex = vector["given"]["internalPubkey"]
        .as_str()
        .expect("internalPubkey hex");
    let internal_bytes = hex::decode(internal_hex).expect("internalPubkey hex parses");
    let internal: UntweakedPublicKey =
        XOnlyPublicKey::from_slice(&internal_bytes).expect("32-byte x-only pubkey");

    let tree = &vector["given"]["scriptTree"];
    if tree.is_null() {
        // Key-spend only path. `Address::p2tr(secp, internal, None, hrp)`
        // computes the BIP-86 / BIP-341 §"if no scripts" tweak and
        // wraps it into a bech32m address. Output key extracted via
        // the same tweak so we can assert against `intermediary.tweakedPubkey`.
        let addr = Address::p2tr(&secp, internal, None, KnownHrp::Mainnet);
        // Reconstruct the tweaked output key for direct comparison.
        let info: TaprootSpendInfo = TaprootBuilder::new()
            .finalize(&secp, internal)
            .expect("empty builder finalizes for key-spend-only");
        let tweaked = info.output_key().to_x_only_public_key();
        return (tweaked, addr.to_string());
    }

    let mut leaves = Vec::new();
    walk_tree(tree, 0, &mut leaves);
    assert!(!leaves.is_empty(), "non-null tree must yield ≥1 leaf");
    let mut builder = TaprootBuilder::new();
    for (depth, script, leaf_ver) in leaves {
        builder = builder
            .add_leaf_with_ver(depth, script, leaf_ver)
            .expect("DFS order valid for the BIP-341 tree shape");
    }
    let info: TaprootSpendInfo = builder
        .finalize(&secp, internal)
        .expect("script-tree builder finalizes against internal key");
    let tweaked = info.output_key().to_x_only_public_key();
    let addr = Address::p2tr_tweaked(info.output_key(), KnownHrp::Mainnet);
    (tweaked, addr.to_string())
}

fn assert_vector(i: usize) {
    let vectors = vectors();
    let v = &vectors[i];
    let (tweaked, addr) = derive(v);
    let expected_tweaked_hex = v["intermediary"]["tweakedPubkey"]
        .as_str()
        .expect("intermediary.tweakedPubkey hex");
    let expected_addr = v["expected"]["bip350Address"]
        .as_str()
        .expect("expected.bip350Address");
    let actual_tweaked_hex = hex::encode(tweaked.serialize());
    assert_eq!(
        actual_tweaked_hex, expected_tweaked_hex,
        "vector {i} tweaked pubkey mismatch"
    );
    assert_eq!(addr, expected_addr, "vector {i} bech32m address mismatch");
}

#[test]
fn vector_0_key_spend_only() {
    // BIP-341 wallet-test-vector 0: `scriptTree: null` (key-spend
    // only). Equivalent to BIP-86's reference flow.
    assert_vector(0);
}

#[test]
fn vector_1_single_leaf() {
    assert_vector(1);
}

#[test]
fn vector_2_single_leaf() {
    assert_vector(2);
}

#[test]
fn vector_3_balanced_two_leaves() {
    // First multi-leaf vector; `[leaf, leaf]` at depth 1 each.
    assert_vector(3);
}

#[test]
fn vector_4_balanced_two_leaves() {
    assert_vector(4);
}

#[test]
fn vector_5_balanced_two_leaves() {
    assert_vector(5);
}

#[test]
fn vector_6_unbalanced_left_leaf_right_subtree() {
    // `[leaf, [leaf, leaf]]` — left at depth 1, right branch's two
    // leaves at depth 2 each. Exercises the asymmetric-depth merkle
    // path.
    assert_vector(6);
}
