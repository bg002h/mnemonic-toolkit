//! cycle-13 Lane C · M1 — `import-wallet --json` must decode the REAL BIP-32
//! account from the single-sig origin path into `bundle.account`, so that
//! `export-wallet --from-import-json` re-emits the correct origin
//! (`m/84'/0'/<account>'`) on the template emitters (sparrow / electrum /
//! coldcard) rather than the hardcoded `m/84'/0'/0'`.
//!
//! Pre-fix: `cmd/import_wallet.rs` hardcoded `account: 0` in `BundleJson`, so
//! the single-sig template emitters rebuilt the origin via
//! `template.origin_path_str(network, account=0)` → a wallet imported at
//! account 5 silently re-emitted `m/84'/0'/0'` (xpub/addresses still correct;
//! declared origin wrong → PSBT key-origin matching / account discovery fails).
//! Multisig uses per-slot `origin_path_bare()` → unaffected.

use assert_cmd::Command;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

/// Run `import-wallet --json` on a fixture and return the parsed envelope.
fn import_envelope(fixture: &str, format: &str) -> serde_json::Value {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            fixture_path(fixture).to_str().unwrap(),
            "--format",
            format,
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    serde_json::from_str(&stdout).unwrap()
}

/// Export an envelope (passed via stdin) to `format`, returning stdout.
fn export_from_envelope(envelope: &serde_json::Value, format: &str) -> String {
    let envelope_json = serde_json::to_string(envelope).unwrap();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--from-import-json",
            "-",
            "--format",
            format,
        ])
        .write_stdin(envelope_json)
        .assert()
        .success();
    String::from_utf8(out.get_output().stdout.clone()).unwrap()
}

// ============================================================================
// Cell 1 — `import-wallet --json` of a single-sig wallet at account 5 must
// decode `bundle.account == 5` (was hardcoded 0).
// ============================================================================

#[test]
fn import_singlesig_account_5_decodes_bundle_account() {
    let env = import_envelope("coldcard-mk1-legacy-bip84-mainnet-account-5.json", "coldcard");
    let bundle = &env[0]["bundle"];
    // Sanity: the origin path the import side recorded is account-5.
    assert_eq!(
        bundle["origin_path"].as_str(),
        Some("m/84'/0'/5'"),
        "origin_path must reflect the imported account; got {bundle}"
    );
    assert_eq!(
        bundle["account"].as_u64(),
        Some(5),
        "bundle.account must decode the real BIP-32 account (3rd hardened \
         component) from the single-sig origin, not the hardcoded 0; got {bundle}"
    );
}

// ============================================================================
// Cell 2 — round-trip: export-wallet --from-import-json re-emits the correct
// account-5 origin on the template emitters (sparrow / electrum).
// ============================================================================

#[test]
fn export_from_import_json_singlesig_account_5_reemits_account_5_origin_sparrow() {
    let env = import_envelope("coldcard-mk1-legacy-bip84-mainnet-account-5.json", "coldcard");
    let out = export_from_envelope(&env, "sparrow");
    assert!(
        out.contains("84'/0'/5'"),
        "sparrow re-emit must carry the account-5 origin m/84'/0'/5'; got: {out}"
    );
    assert!(
        !out.contains("84'/0'/0'"),
        "sparrow re-emit must NOT carry the account-0 origin m/84'/0'/0'; got: {out}"
    );
}

#[test]
fn export_from_import_json_singlesig_account_5_reemits_account_5_origin_electrum() {
    let env = import_envelope("coldcard-mk1-legacy-bip84-mainnet-account-5.json", "coldcard");
    let out = export_from_envelope(&env, "electrum");
    assert!(
        out.contains("84'/0'/5'"),
        "electrum re-emit must carry the account-5 origin m/84'/0'/5'; got: {out}"
    );
    assert!(
        !out.contains("84'/0'/0'"),
        "electrum re-emit must NOT carry the account-0 origin m/84'/0'/0'; got: {out}"
    );
}

// ============================================================================
// Cell 3 — guard: account-0 single-sig still round-trips to account 0
// (no off-by-one / mis-decode regression).
// ============================================================================

#[test]
fn import_singlesig_account_0_stays_account_0() {
    let env = import_envelope("coldcard-singlesig-bip84-mainnet.json", "coldcard");
    let bundle = &env[0]["bundle"];
    assert_eq!(
        bundle["account"].as_u64(),
        Some(0),
        "account-0 single-sig must stay account 0; got {bundle}"
    );
    let out = export_from_envelope(&env, "sparrow");
    assert!(
        out.contains("84'/0'/0'"),
        "account-0 sparrow re-emit must carry m/84'/0'/0'; got: {out}"
    );
}

// ============================================================================
// Cell 4 — multisig is unaffected: bundle.account stays 0 (multisig uses
// per-slot origin paths, not bundle.account).
// ============================================================================

#[test]
fn import_multisig_bundle_account_unaffected() {
    let env = import_envelope("coldcard-ms-2of3-p2wsh-with-xfp.txt", "coldcard-multisig");
    let bundle = &env[0]["bundle"];
    // Multisig has no single `origin_path` account semantics — bundle.account
    // must remain 0 (the fix only decodes the single-sig case).
    assert_eq!(
        bundle["account"].as_u64(),
        Some(0),
        "multisig bundle.account must stay 0 (per-slot origins drive multisig); got {bundle}"
    );
}
