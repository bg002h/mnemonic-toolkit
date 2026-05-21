//! v0.31.1 — Sparrow taproot descriptor-passthrough import integration tests.
//!
//! Validates the v0.31.1 Cycle 8 path-split at `wallet_import/sparrow.rs::parse`
//! Step 6: descriptor-passthrough shape (no `@0/**` placeholder; concrete
//! `[fp/path]xpub` keys embedded directly) bypasses Step 5 substitution
//! and feeds the script directly into the existing
//! `concrete_keys_to_placeholders` → `parse_descriptor` pipeline.
//!
//! Per `wallet_export/sparrow.rs:195`, taproot SINGLESIG (Bip86) still
//! emits TEMPLATE mode (`tr(@0/**)`) and is still REFUSED (separate
//! FOLLOWUP `sparrow-taproot-singlesig-template-mode-import`). The
//! refusal test for that case lives at `cli_import_wallet_sparrow.rs:305`.

use assert_cmd::Command;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/wallet_import")
        .join(name)
}

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

// ──────────────────────────────────────────────────────────────────────
// Happy path: taproot multisig descriptor-passthrough imports
// ──────────────────────────────────────────────────────────────────────

#[test]
fn tr_multi_a_nums_2of3_imports_successfully() {
    let blob = fixture_path("sparrow-tr-multi-a-nums-2of3.json");
    mnemonic()
        .args(["import-wallet", "--format", "sparrow"])
        .arg("--blob")
        .arg(&blob)
        .args(["--json"])
        .assert()
        .success();
}

#[test]
fn tr_multi_a_nums_2of3_envelope_carries_canonical_descriptor() {
    // Validate the canonical taproot descriptor is preserved verbatim in
    // the bundle envelope (NUMS sentinel + 3 cosigners with origin
    // brackets). This documents the descriptor-passthrough pipeline's
    // contract: the `[fp/path]xpub` keys flow through unchanged.
    let blob = fixture_path("sparrow-tr-multi-a-nums-2of3.json");
    let assertion = mnemonic()
        .args(["import-wallet", "--format", "sparrow"])
        .arg("--blob")
        .arg(&blob)
        .args(["--json"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();

    // BIP-341 NUMS point (script-path-only convention).
    assert!(
        stdout.contains("50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"),
        "envelope must carry the NUMS sentinel internal-key; got: {stdout}"
    );
    // `tr(NUMS, multi_a(...))` shape with threshold 2.
    assert!(
        stdout.contains("tr(") && stdout.contains("multi_a(2,"),
        "envelope must carry the tr(...,multi_a(...)) descriptor; got: {stdout}"
    );
    // All 3 cosigners' fingerprints + xpubs preserved in the descriptor.
    assert!(
        stdout.contains("[b8688df1/87'/0'/0']")
            && stdout.contains("[28645006/87'/0'/0']")
            && stdout.contains("[5436d724/87'/0'/0']"),
        "envelope must carry all 3 cosigner origin brackets; got: {stdout}"
    );
}

#[test]
fn tr_multi_a_nums_2of3_sniffs_as_sparrow() {
    // Auto-sniff (no `--format` flag) should still detect taproot Sparrow
    // wallets as `sparrow` format. Sniff is `policyType`-based (per
    // `wallet_import/sparrow.rs::sniff` L130+); script content doesn't
    // matter. This cell guards against future sniff regressions.
    let blob = fixture_path("sparrow-tr-multi-a-nums-2of3.json");
    mnemonic()
        .args(["import-wallet"])
        .arg("--blob")
        .arg(&blob)
        .args(["--json"])
        .assert()
        .success();
}

// ──────────────────────────────────────────────────────────────────────
// Boundary: taproot SINGLESIG (Bip86) still refused (template-mode)
// ──────────────────────────────────────────────────────────────────────

#[test]
fn taproot_singlesig_template_still_refused() {
    // Cycle 8 ships taproot MULTISIG descriptor-passthrough only. Taproot
    // SINGLESIG (Bip86: `tr(@0/**)` template-mode) is still refused via
    // the path-split's `has_tr && has_at_placeholder` branch. Filed
    // forward as FOLLOWUP `sparrow-taproot-singlesig-template-mode-import`.
    let blob = r#"{
        "name":"bip86-0","network":"mainnet","policyType":"SINGLE","scriptType":"P2TR",
        "defaultPolicy":{"name":"Default","miniscript":{"script":"tr(@0/**)"}},
        "keystores":[{
            "label":"bip86-0","source":"SW_WATCH","walletModel":"SPARROW",
            "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/86'/0'/0'"},
            "extendedPublicKey":"xpub6CAYwo2AfKJy1cdFGBAgLvCrZULhEkZ9C9s4GGXwXzHvNPguMWBcVrGEDjP2ZJdX92gVWLeLrNVVmipTrKqrwMy2eT282xKEyHMbPDrcD9e"
        }]
    }"#;
    let assertion = mnemonic()
        .args(["import-wallet", "--blob", "-", "--format", "sparrow"])
        .write_stdin(blob.to_string())
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("taproot singlesig templates")
            && stderr.contains("sparrow-taproot-singlesig-template-mode-import"),
        "expected v0.31.1 narrow-refusal message for taproot singlesig templates; got: {stderr}"
    );
}

// ──────────────────────────────────────────────────────────────────────
// No-regression: existing template-mode wallets still parse
// ──────────────────────────────────────────────────────────────────────

#[test]
fn template_mode_p2wpkh_singlesig_no_regression() {
    let blob = fixture_path("sparrow-singlesig-p2wpkh.json");
    mnemonic()
        .args(["import-wallet", "--format", "sparrow"])
        .arg("--blob")
        .arg(&blob)
        .args(["--json"])
        .assert()
        .success();
}

#[test]
fn template_mode_wsh_sortedmulti_2of3_no_regression() {
    let blob = fixture_path("sparrow-multisig-2of3-p2wsh-sortedmulti.json");
    mnemonic()
        .args(["import-wallet", "--format", "sparrow"])
        .arg("--blob")
        .arg(&blob)
        .args(["--json"])
        .assert()
        .success();
}
