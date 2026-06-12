//! GAP 5b — two verify-bundle coverage cells the existing suite lacked:
//!
//! (1) a HASHLOCK general policy round-trip (bundle → verify-bundle) — zero
//!     `sha256|hash256|ripemd|hash160` coverage existed across the
//!     `cli_verify_bundle*.rs` files, despite hashlocks being a shipped
//!     verify-bundle surface (`parse_descriptor.rs` has all four arms).
//! (2) a pinned-REFUSAL contract for a BIP-388 wallet-policy JSON fed to
//!     `verify-bundle --descriptor` — `bundle`/`export-wallet` auto-detect +
//!     expand a leading-`{` policy, but `verify_bundle.rs` has no
//!     `is_bip388_policy_shape` probe, so it refuses today. Pinning the current
//!     refusal documents the intake asymmetry; FOLLOWUP
//!     `verify-bundle-bip388-policy-intake` tracks the feature (which would flip
//!     this cell red-then-green).
//!
//! NO-BUMP (test-only).

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

/// (2) Pinned refusal: a BIP-388 wallet-policy JSON fed to
/// `verify-bundle --descriptor` is NOT auto-detected/expanded (unlike
/// bundle/export-wallet) — the leading-`{` policy carries BOTH `@N` template
/// placeholders AND inline keys, tripping the mixed-form classifier. Exit 2,
/// "descriptor mixes @N placeholders with inline keys; use one form". The
/// classify error fires before any card decode, so dummy mk1/md1 suffice.
/// FOLLOWUP `verify-bundle-bip388-policy-intake` would flip this red→green.
#[test]
fn verify_bundle_refuses_bip388_policy_json() {
    let policy = format!(
        r#"{{"name":"test-vault","description_template":"wsh(sortedmulti(2,@0/**,@1/**))","keys_info":["[704c7836/48'/0'/0'/2']{A}","[97139860/48'/0'/0'/2']{B}"]}}"#
    );
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--descriptor",
            &policy,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={A}"),
            "--slot",
            &format!("@1.xpub={B}"),
            "--mk1",
            "mk1qqq",
            "--md1",
            "md1qqq",
        ])
        .assert()
        .code(2)
        .stderr(predicates::prelude::predicate::str::contains(
            "mixes @N placeholders with inline keys",
        ));
}
