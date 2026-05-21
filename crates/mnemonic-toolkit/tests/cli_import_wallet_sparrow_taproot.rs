//! v0.31.1 + v0.31.2 — Sparrow taproot import integration tests.
//!
//! Validates BOTH branches of the path-split at
//! `wallet_import/sparrow.rs::parse` Step 6:
//!
//! - v0.31.1 Cycle 8: descriptor-passthrough shape (no `@0/**`
//!   placeholder; concrete `[fp/path]xpub` keys embedded directly)
//!   bypasses Step 5 substitution and feeds the script directly into
//!   the existing `concrete_keys_to_placeholders` → `parse_descriptor`
//!   pipeline. Covers taproot MULTISIG (`tr-multi-a` /
//!   `tr-sortedmulti-a` per `wallet_export/sparrow.rs:215-219`).
//!
//! - v0.31.2 Cycle 9: taproot SINGLESIG template-mode (Bip86:
//!   `tr(@0/**)` per `wallet_export/sparrow.rs:195`) routes through
//!   the standard substitution branch, producing
//!   `tr([fp/86'/0'/0']xpub.../<0;1>/*)`. Cycle 8's narrow refusal was
//!   removed (FOLLOWUP `sparrow-taproot-singlesig-template-mode-import`
//!   closed).

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
// v0.31.2 — taproot SINGLESIG (Bip86) template-mode happy path
// ──────────────────────────────────────────────────────────────────────

#[test]
fn taproot_singlesig_template_imports_via_substitution() {
    // v0.31.2 Cycle 9: taproot SINGLESIG (Bip86: `tr(@0/**)` template-mode)
    // now joins the general template-mode substitution path. The
    // `@0/**` placeholder is replaced with the concrete
    // `[5436d724/86'/0'/0']xpub.../<0;1>/*` form and the resulting
    // descriptor is parsed cleanly. Closes
    // `sparrow-taproot-singlesig-template-mode-import`.
    let blob = fixture_path("sparrow-singlesig-p2tr.json");
    let assertion = mnemonic()
        .args(["import-wallet", "--format", "sparrow"])
        .arg("--blob")
        .arg(&blob)
        .args(["--json"])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("[5436d724/86'/0'/0']")
            && stdout.contains("xpub6CAYwo2AfKJy1cdFGBAgLvCrZULhEkZ9C9s4GGXwXzHvNPguMWBcVrGEDjP2ZJdX92gVWLeLrNVVmipTrKqrwMy2eT282xKEyHMbPDrcD9e"),
        "expected substituted taproot descriptor with concrete origin+xpub; got: {stdout}"
    );
    assert!(
        stdout.contains("tr(") && stdout.contains("<0;1>/*"),
        "expected tr() wrapping + multipath suffix; got: {stdout}"
    );
}

#[test]
fn taproot_singlesig_envelope_blocked_by_wallet_import_taproot_internal_key() {
    // Boundary cell — documents an ORTHOGONAL prior FOLLOWUP gap that
    // Cycle 9 does NOT address: `wallet-import-taproot-internal-key`.
    //
    // The v0.31.2 wallet_import/sparrow path now produces a clean JSON
    // envelope for taproot singlesig (verified by
    // `taproot_singlesig_template_imports_via_substitution` above).
    // However, the export-from-envelope path
    // (`export-wallet --from-import-json`) refuses ALL taproot envelopes
    // because the envelope wire-shape doesn't surface the BIP-341
    // internal-key designation (NUMS sentinel vs raw xonly). This is the
    // same boundary that applies to taproot MULTISIG (Cycle 8) envelopes.
    //
    // Cycle 9 ships taproot singlesig IMPORT only — re-emission via
    // `--from-import-json` is gated on the separate FOLLOWUP
    // `wallet-import-taproot-internal-key` shipping the envelope
    // wire-shape extension.
    let blob = fixture_path("sparrow-singlesig-p2tr.json");

    // Import → JSON envelope works.
    let round1 = mnemonic()
        .args(["import-wallet", "--format", "sparrow"])
        .arg("--blob")
        .arg(&blob)
        .args(["--json"])
        .assert()
        .success();
    let envelope = String::from_utf8(round1.get_output().stdout.clone()).unwrap();
    assert!(
        envelope.contains("\"source_format\": \"sparrow\""),
        "envelope must declare source_format=sparrow; got: {envelope}"
    );

    // Re-emit via --from-import-json refuses (taproot-envelope gap).
    let assertion = mnemonic()
        .args([
            "export-wallet",
            "--format",
            "sparrow",
            "--from-import-json",
            "-",
        ])
        .write_stdin(envelope)
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("taproot") && stderr.contains("wallet-import-taproot-internal-key"),
        "expected wallet-import-taproot-internal-key refusal message; got: {stderr}"
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
