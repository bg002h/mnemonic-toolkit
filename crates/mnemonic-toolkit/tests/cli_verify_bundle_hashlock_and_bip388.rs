//! GAP 5b — two verify-bundle coverage cells the existing suite lacked:
//!
//! (1) a HASHLOCK general policy round-trip (bundle → verify-bundle) — zero
//!     `sha256|hash256|ripemd|hash160` coverage existed across the
//!     `cli_verify_bundle*.rs` files, despite hashlocks being a shipped
//!     verify-bundle surface (`parse_descriptor.rs` has all four arms).
//! (2) BIP-388 wallet-policy JSON INTAKE for `verify-bundle --descriptor`
//!     (C2 / v0.57.0): a leading-`{` policy is auto-detected + expanded to a
//!     concrete descriptor before verifying, matching `bundle`/`export-wallet`.
//!     Three cells: a 2-of-2 sortedmulti policy bundle→verify-bundle round-trip,
//!     an n=1 wpkh policy round-trip, and a malformed `@N`-beyond-`keys_info`
//!     policy still refused LOUDLY (asserts the SPECIFIC expander message, not
//!     just exit code — exit-2 coincides pre/post-feature). Resolves FOLLOWUP
//!     `verify-bundle-bip388-policy-intake` (this file previously PINNED the
//!     refusal; cell (2) was inverted red→green by C2).
//!
//! Cell (1) is test-only NO-BUMP; cells (2a/2b/2c) ship with v0.57.0 (C2).

use assert_cmd::Command;
use std::io::Write;

const TREZOR_12_ZERO: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

// Two valid mainnet account xpubs (reused from cli_bip388_policy_intake.rs).
const A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";

/// (1) Hashlock general policy `wsh(and_v(v:sha256(H),pk(@0)))`: bundle from a
/// phrase, then verify-bundle the emitted JSON with the same phrase → ok.
/// Mirrors `cli_verify_bundle_multi_cosigner_mk1.rs::non_canonical_wsh_andor_round_trips_via_bundle_json`.
#[test]
fn hashlock_wsh_and_v_sha256_round_trips_via_bundle_json() {
    let h = "1111111111111111111111111111111111111111111111111111111111111111";
    let descriptor = format!("wsh(and_v(v:sha256({h}),pk(@0)))");

    let bundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            &descriptor,
            "--network",
            "mainnet",
            "--account",
            "0",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap();

    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("bundle.json");
    std::fs::File::create(&path)
        .unwrap()
        .write_all(stdout.as_bytes())
        .unwrap();

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--descriptor",
            &descriptor,
            "--network",
            "mainnet",
            "--account",
            "0",
            "--slot",
            &format!("@0.phrase={TREZOR_12_ZERO}"),
            "--bundle-json",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();
}

/// (2a) BIP-388 policy intake — the inverted cell (was `verify_bundle_refuses_
/// bip388_policy_json`). A 2-of-2 `wsh(sortedmulti(2,@0/**,@1/**))` wallet-policy
/// JSON (origin-annotated `keys_info`) is auto-detected (`is_bip388_policy_shape`)
/// and expanded (`expand_bip388_policy`) to a concrete descriptor BEFORE the
/// classify probe, then verified — matching bundle/export-wallet (v0.49.0).
/// Round-trip: bundle the policy (watch-only, inline keys, no slots) → JSON →
/// verify-bundle the SAME policy JSON via `--bundle-json` (cosigner cards come
/// from the envelope; no `--mk1`/`--md1`). Structural template:
/// `cli_verify_bundle_multi_cosigner_mk1.rs::audit_i10_same_xpub_two_paths_2of2_round_trips`.
#[test]
fn verify_bundle_accepts_bip388_policy_json() {
    // Identical to policy_2of2() in cli_bip388_policy_intake.rs.
    let policy = format!(
        r#"{{"name":"test-vault","description_template":"wsh(sortedmulti(2,@0/**,@1/**))","keys_info":["[704c7836/48'/0'/0'/2']{A}","[97139860/48'/0'/0'/2']{B}"]}}"#
    );

    // Watch-only bundle from the policy JSON (inline keys → no slots) → JSON.
    let bundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            &policy,
            "--network",
            "mainnet",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap();

    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("bundle.json");
    std::fs::File::create(&path)
        .unwrap()
        .write_all(stdout.as_bytes())
        .unwrap();

    let v = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--descriptor",
            &policy,
            "--network",
            "mainnet",
            "--bundle-json",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let vo = v.get_output();
    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&vo.stdout),
        String::from_utf8_lossy(&vo.stderr)
    );
    assert!(
        report.contains("result: ok"),
        "BIP-388 policy-JSON verify-bundle must be ok, got:\n{report}"
    );
}

/// (2b) Single-sig (n=1) `wpkh(@0/**)` policy round-trip — isolates the
/// expand→concrete→verify path from multisig cosigner-card machinery. Mirrors
/// `cli_bip388_policy_intake.rs::bundle_descriptor_bip388_singlesig_policy_watch_only`.
#[test]
fn verify_bundle_accepts_bip388_singlesig_policy_json() {
    // Identical to policy_singlesig() in cli_bip388_policy_intake.rs.
    let policy = format!(
        r#"{{"name":"single","description_template":"wpkh(@0/**)","keys_info":["[704c7836/84'/0'/0']{A}"]}}"#
    );

    let bundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            &policy,
            "--network",
            "mainnet",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap();

    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("bundle.json");
    std::fs::File::create(&path)
        .unwrap()
        .write_all(stdout.as_bytes())
        .unwrap();

    let v = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--descriptor",
            &policy,
            "--network",
            "mainnet",
            "--bundle-json",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();
    let vo = v.get_output();
    let report = format!(
        "{}{}",
        String::from_utf8_lossy(&vo.stdout),
        String::from_utf8_lossy(&vo.stderr)
    );
    assert!(
        report.contains("result: ok"),
        "BIP-388 single-sig policy verify-bundle must be ok, got:\n{report}"
    );
}

/// (2c) Non-vacuity guard: a malformed policy whose `description_template`
/// references `@2/**` with only TWO `keys_info` entries is still refused LOUDLY,
/// via the SHARED expander (`expand_bip388_policy` → "references @N beyond
/// keys_info"), proving the probe routes through the real expander not a bypass.
/// NOTE (R0-r1 M1): this policy exits 2 in BOTH worlds — pre-feature the
/// mixed-form classifier ("mixes @N placeholders with inline keys"), post-feature
/// the expander's residual-`@N` error. The discriminator is the MESSAGE, so we
/// assert on `"@N beyond keys_info"` (NOT a bare exit code). The expander error
/// fires before any card decode, so dummy mk1/md1 suffice.
#[test]
fn verify_bundle_refuses_bip388_policy_at_n_beyond_keys_info() {
    let bad = format!(
        r#"{{"name":"x","description_template":"wsh(multi(2,@0/**,@1/**,@2/**))","keys_info":["[704c7836/48'/0'/0'/2']{A}","[97139860/48'/0'/0'/2']{B}"]}}"#
    );
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--descriptor",
            &bad,
            "--network",
            "mainnet",
            "--mk1",
            "mk1qqq",
            "--md1",
            "md1qqq",
        ])
        .assert()
        .failure()
        .stderr(predicates::prelude::predicate::str::contains(
            "@N beyond keys_info",
        ));
}
