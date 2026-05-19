//! v0.27.1 Phase 5 drift-regression cells (per plan-doc Q6 + Q5c).
//!
//! Each cell loads a v0.27.0-captured fixture from
//! `tests/fixtures/v0_27_0_envelopes/` and pins the structural contract:
//!   - top-level key set (presence + names — locks against renames/drops/adds)
//!   - tagged fields (schema_version, mode, result)
//!   - null-emission discipline on the no-match arm (the wire-shape
//!     constraint that drove Q5a's private-constructor pivot per plan-doc R1)
//!
//! Why structural rather than byte-equality: serde_json::Value normalizes
//! map iteration alphabetically (not the toolkit's struct-declaration
//! order), so a verbatim byte-roundtrip via Value doesn't preserve
//! fixture bytes. Pinning the structural contract is the same regression-
//! guard surface without that brittleness. A future cell that drives the
//! actual toolkit binary against test fixtures + asserts byte-equal stdout
//! would be the stronger guard but requires deriving inputs from the same
//! seed-derivation pipeline the fixture-capture used; deferred to a future
//! cycle.
//!
//! Q5c (plan-doc) — fixtures are pinned to v0.27.0 forever. Future minor
//! cycles (v0.28+) that legitimately change wire shape add companion fixture
//! dirs; this file may then either gain v0.28 cells alongside the v0.27.0
//! ones, OR the v0.27.0 cells convert to `#[ignore]` with a doc comment.

use std::fs;

fn read_fixture(name: &str) -> serde_json::Value {
    let raw = fs::read_to_string(format!("tests/fixtures/v0_27_0_envelopes/{name}"))
        .expect("fixture file readable");
    serde_json::from_str(raw.trim()).expect("fixture parses as JSON")
}

/// Assert the fixture is a JSON object whose top-level keys exactly match
/// the expected set. Locks against drops, renames, and additive drift.
fn assert_top_level_key_set(v: &serde_json::Value, expected: &[&str], fixture: &str) {
    let obj = v.as_object().expect("fixture is an object");
    let mut actual: Vec<&str> = obj.keys().map(String::as_str).collect();
    actual.sort();
    let mut expected_sorted: Vec<&str> = expected.to_vec();
    expected_sorted.sort();
    assert_eq!(
        actual, expected_sorted,
        "{fixture} top-level keys drifted"
    );
}

// ============================================================================
// Path-of-xpub fixtures
// ============================================================================

#[test]
fn drift_path_of_xpub_match_structural_contract() {
    let v = read_fixture("path_of_xpub.match.json");
    assert_top_level_key_set(
        &v,
        &[
            "schema_version",
            "mode",
            "result",
            "path",
            "template",
            "account",
            "target_xpub_canonical",
            "target_xpub_variant",
            "searched_count",
        ],
        "path_of_xpub.match",
    );
    assert_eq!(v["schema_version"], "1");
    assert_eq!(v["mode"], "path-of-xpub");
    assert_eq!(v["result"], "match");
    // Match-arm contract: path/template populated, account is u32 or null
    // (None for --add-path templates without an account token).
    assert!(v["path"].is_string(), "path must be string on match");
    assert!(v["template"].is_string(), "template must be string on match");
    assert!(v["target_xpub_canonical"].is_string());
}

#[test]
fn drift_path_of_xpub_no_match_structural_contract() {
    let v = read_fixture("path_of_xpub.no_match.json");
    assert_top_level_key_set(
        &v,
        &[
            "schema_version",
            "mode",
            "result",
            "path",
            "template",
            "account",
            "target_xpub_canonical",
            "target_xpub_variant",
            "searched_count",
        ],
        "path_of_xpub.no_match",
    );
    assert_eq!(v["result"], "no_match");
    // No-match wire-shape lock (Q5a pivot's load-bearing constraint):
    // path/template/account MUST be present AS null (NOT omitted via
    // skip_serializing_if). Locks the v0.27.x discipline against silent
    // SemVer-minor shape evolution.
    assert!(v["path"].is_null(), "path must be null on no-match (not omitted)");
    assert!(v["template"].is_null(), "template must be null on no-match");
    assert!(v["account"].is_null(), "account must be null on no-match");
    assert!(v["target_xpub_canonical"].is_string(), "envelope-scope target_xpub_canonical preserved on no-match");
}

// ============================================================================
// Passphrase-of-xpub fixtures (symmetric to path-of-xpub)
// ============================================================================

#[test]
fn drift_passphrase_of_xpub_match_structural_contract() {
    let v = read_fixture("passphrase_of_xpub.match.json");
    assert_top_level_key_set(
        &v,
        &[
            "schema_version",
            "mode",
            "result",
            "path",
            "template",
            "account",
            "target_xpub_canonical",
            "target_xpub_variant",
            "searched_count",
        ],
        "passphrase_of_xpub.match",
    );
    assert_eq!(v["mode"], "passphrase-of-xpub");
    assert_eq!(v["result"], "match");
    assert!(v["path"].is_string());
    assert!(v["template"].is_string());
}

#[test]
fn drift_passphrase_of_xpub_no_match_structural_contract() {
    let v = read_fixture("passphrase_of_xpub.no_match.json");
    assert_eq!(v["mode"], "passphrase-of-xpub");
    assert_eq!(v["result"], "no_match");
    assert!(v["path"].is_null());
    assert!(v["template"].is_null());
    assert!(v["account"].is_null());
}

// ============================================================================
// Account-of-descriptor fixtures
// ============================================================================

#[test]
fn drift_account_of_descriptor_match_structural_contract() {
    let v = read_fixture("account_of_descriptor.match.json");
    assert_top_level_key_set(
        &v,
        &[
            "schema_version",
            "mode",
            "result",
            "matched_cosigners",
            "cosigners_total",
            "searched_count_per_cosigner",
            "descriptor_shape",
            "unspendable_internal_keys",
        ],
        "account_of_descriptor.match",
    );
    assert_eq!(v["mode"], "account-of-descriptor");
    assert_eq!(v["result"], "match");
    // matched_cosigners is non-empty on match (Q5a invariant).
    let mc = v["matched_cosigners"].as_array().expect("matched_cosigners is array");
    assert!(!mc.is_empty(), "matched_cosigners must be non-empty on match");
    // Per-cosigner structural contract.
    let inner_keys = mc[0].as_object().unwrap().keys().collect::<Vec<_>>();
    let inner_set: std::collections::BTreeSet<&str> = inner_keys.iter().map(|s| s.as_str()).collect();
    let expected: std::collections::BTreeSet<&str> = ["cosigner_index", "path", "template", "account"].into_iter().collect();
    assert_eq!(inner_set, expected, "matched_cosigners[0] keys drifted");
}

#[test]
fn drift_account_of_descriptor_no_match_structural_contract() {
    let v = read_fixture("account_of_descriptor.no_match.json");
    assert_eq!(v["mode"], "account-of-descriptor");
    assert_eq!(v["result"], "no_match");
    // matched_cosigners MUST be present + empty (Q5a invariant): not omitted.
    let mc = v["matched_cosigners"].as_array().expect("matched_cosigners is array");
    assert!(mc.is_empty(), "matched_cosigners must be empty on no-match");
    assert!(v["cosigners_total"].is_number(), "cosigners_total envelope-scope preserved on no-match");
}

// ============================================================================
// Cross-fixture invariants
// ============================================================================

#[test]
fn drift_all_fixtures_carry_schema_version_1() {
    for fixture in [
        "path_of_xpub.match.json",
        "path_of_xpub.no_match.json",
        "passphrase_of_xpub.match.json",
        "passphrase_of_xpub.no_match.json",
        "account_of_descriptor.match.json",
        "account_of_descriptor.no_match.json",
    ] {
        let v = read_fixture(fixture);
        assert_eq!(
            v["schema_version"], "1",
            "{fixture} schema_version drifted from \"1\""
        );
    }
}
