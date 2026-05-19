//! v0.27.0 Phase 5 — `mnemonic export-wallet --from-import-json <FILE|->`.
//!
//! Per `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md`
//! §3.7 + §3.7.1. Cells exercise the export-side consumer path that
//! decodes an `import-wallet --json` envelope and emits a per-format
//! wallet config via the existing `WalletFormatEmitter` dispatch.
//!
//! Includes the headline v0.27.0 integration cell
//! `cross_format_bsms_to_bitcoin_core_to_import_round_trip`.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

fn run_export_from_import_json(envelope_path: &Path, format: &str) -> String {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--from-import-json",
            envelope_path.to_str().unwrap(),
            "--format",
            format,
        ])
        .assert()
        .success();
    String::from_utf8(out.get_output().stdout.clone()).unwrap()
}

// ============================================================================
// Cell 1 — export to bitcoin-core listdescriptors.
// ============================================================================

#[test]
fn export_wallet_from_import_json_to_bitcoin_core_emits_valid_listdescriptors() {
    let p = fixture_path("envelope_v0_27_0.json");
    let out = run_export_from_import_json(&p, "bitcoin-core");
    // Output is a JSON array with two entries (receive + change).
    let val: serde_json::Value = serde_json::from_str(&out).unwrap();
    let arr = val.as_array().expect("bitcoin-core emit must be JSON array");
    assert_eq!(arr.len(), 2, "expected receive + change entries");
    for entry in arr {
        assert!(
            entry["desc"].as_str().unwrap().starts_with("sh(multi(2,"),
            "each desc must start with the source descriptor's outer wrapper"
        );
        assert!(entry["active"].is_boolean());
        assert!(entry["range"].is_array());
        assert!(entry["timestamp"].is_string() || entry["timestamp"].is_number());
    }
}

// ============================================================================
// Cell 2 — export to bip388 wallet-policy.
// ============================================================================

#[test]
fn export_wallet_from_import_json_to_bip388_emits_valid_wallet_policy() {
    let p = fixture_path("envelope_v0_27_0.json");
    let out = run_export_from_import_json(&p, "bip388");
    let val: serde_json::Value = serde_json::from_str(&out).unwrap();
    // bip388 wallet-policy uses `description_template` for the
    // @N-placeholder shape + `keys_info` for the per-cosigner xpubs.
    assert!(
        val["description_template"].as_str().is_some(),
        "bip388 must carry description_template field"
    );
    let keys = val["keys_info"]
        .as_array()
        .expect("bip388 must carry keys_info array");
    assert_eq!(keys.len(), 3, "2-of-3 → 3 keys");
}

// ============================================================================
// Cell 3 — jade/sparrow/coldcard/specter/electrum/green refuse
// descriptor-mode input. The wallet-import envelope is always
// descriptor-mode (Phase 4 emits with template=None), so these formats
// surface the existing per-emitter "--template required" refusal.
// Pinning this behavior at v0.27.0 prevents accidental regressions on
// the per-emitter contract.
// ============================================================================

#[test]
fn export_wallet_from_import_json_to_template_only_format_refuses_with_helpful_message() {
    let p = fixture_path("envelope_v0_27_0.json");
    // Specter / Green excluded — Specter refuses for missing wallet_name
    // (different code path); Green accepts descriptor-mode for its
    // text-emit shape.
    for fmt in ["sparrow", "jade", "coldcard", "electrum"] {
        let assertion = Command::cargo_bin("mnemonic")
            .unwrap()
            .args([
                "export-wallet",
                "--from-import-json",
                p.to_str().unwrap(),
                "--format",
                fmt,
            ])
            .assert()
            .failure();
        let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
        assert!(
            stderr.contains("requires --template") || stderr.contains("descriptor passthrough is not supported"),
            "format {fmt} must refuse descriptor-mode with the existing emitter-contract message; got: {stderr}"
        );
    }
}

// ============================================================================
// Cell 5 — mutex: --template + --from-import-json errors.
// ============================================================================

#[test]
fn export_wallet_from_import_json_with_template_errors_mutex() {
    let p = fixture_path("envelope_v0_27_0.json");
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--template",
            "wsh-sortedmulti",
            "--from-import-json",
            p.to_str().unwrap(),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("cannot be used with") || stderr.contains("conflict"),
        "clap mutex error expected; got: {stderr}"
    );
}

// ============================================================================
// Cell 6 — mutex: --descriptor + --from-import-json errors.
// ============================================================================

#[test]
fn export_wallet_from_import_json_with_descriptor_errors_mutex() {
    let p = fixture_path("envelope_v0_27_0.json");
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--descriptor",
            "wpkh(xpub.../<0;1>/*)",
            "--from-import-json",
            p.to_str().unwrap(),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("cannot be used with") || stderr.contains("conflict"),
        "clap mutex error expected; got: {stderr}"
    );
}

// ============================================================================
// Cell 7 — --account != 0 with --from-import-json is BadInput.
// ============================================================================

#[test]
fn export_wallet_from_import_json_with_account_errors() {
    let p = fixture_path("envelope_v0_27_0.json");
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--from-import-json",
            p.to_str().unwrap(),
            "--account",
            "1",
            "--format",
            "bitcoin-core",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--from-import-json reads the account from the envelope"),
        "expected --account-with-from-import-json error; got: {stderr}"
    );
}

// ============================================================================
// Cell 8 — multi-entry envelope without --from-import-json-index errors.
// ============================================================================

fn multi_entry_envelope_json() -> String {
    let one_entry: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fixture_path("envelope_v0_27_0.json")).unwrap())
            .unwrap();
    let entry = &one_entry.as_array().unwrap()[0];
    serde_json::to_string(&serde_json::Value::Array(vec![entry.clone(), entry.clone()])).unwrap()
}

#[test]
fn export_wallet_from_import_json_multi_descriptor_requires_index() {
    let tmpdir = tempfile::tempdir().unwrap();
    let p = tmpdir.path().join("multi.json");
    std::fs::write(&p, multi_entry_envelope_json()).unwrap();
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--from-import-json",
            p.to_str().unwrap(),
            "--format",
            "bitcoin-core",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("envelope array has 2 entries"),
        "expected multi-entry error; got: {stderr}"
    );
}

// ============================================================================
// Cell 9 — --from-import-json-index picks the correct entry.
// ============================================================================

#[test]
fn export_wallet_from_import_json_index_picks_correct_entry() {
    let tmpdir = tempfile::tempdir().unwrap();
    let p = tmpdir.path().join("multi.json");
    std::fs::write(&p, multi_entry_envelope_json()).unwrap();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--from-import-json",
            p.to_str().unwrap(),
            "--from-import-json-index",
            "1",
            "--format",
            "bitcoin-core",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("sh(multi(2,"));
}

// ============================================================================
// Cell 10 — `--from-import-json -` reads envelope from stdin.
// ============================================================================

#[test]
fn export_wallet_from_import_json_stdin_dash_reads_envelope() {
    let envelope_json = std::fs::read_to_string(fixture_path("envelope_v0_27_0.json")).unwrap();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--from-import-json",
            "-",
            "--format",
            "bitcoin-core",
        ])
        .write_stdin(envelope_json)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("sh(multi(2,"));
}

// ============================================================================
// Cell 11 — INTEGRATION (cross-phase headline). Per plan §4.5:
// `cross_format_bsms_to_<X>_round_trip`: start from a BSMS Round-2 blob;
// import-wallet --json → export-wallet --from-import-json → assert output
// parses semantically and matches the source descriptor + cosigner xpubs.
//
// We pick Bitcoin Core as the export target (Sparrow / Jade /
// Specter / Electrum / Green descriptor-mode compat varies; Bitcoin Core
// is the most canonical multi-cosigner multisig consumer). Round-trip
// semantic-match: descriptor body + cosigner xpubs preserved verbatim.
// ============================================================================

#[test]
fn cross_format_bsms_to_bitcoin_core_to_import_round_trip() {
    let bsms = "BSMS 1.0\nsh(multi(2,[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,[5436d724/48'/0'/0'/2']xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx/<0;1>/*,[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*))#ek6d38cp\n";

    // Step 1: import-wallet → envelope JSON.
    let import_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "bsms", "--json"])
        .write_stdin(bsms)
        .assert()
        .success();
    let envelope_json = String::from_utf8(import_out.get_output().stdout.clone()).unwrap();

    // Step 2: export-wallet --from-import-json - → bitcoin-core JSON.
    let export_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--from-import-json",
            "-",
            "--format",
            "bitcoin-core",
        ])
        .write_stdin(envelope_json)
        .assert()
        .success();
    let core_json = String::from_utf8(export_out.get_output().stdout.clone()).unwrap();
    let core_val: serde_json::Value = serde_json::from_str(&core_json).unwrap();
    let arr = core_val.as_array().expect("bitcoin-core JSON array");
    assert_eq!(arr.len(), 2, "expected receive + change descriptors");

    // Step 3: assert each Bitcoin Core descriptor body contains all 3
    // cosigner xpubs from the BSMS source + the 48'/0'/0'/2' origin paths.
    for entry in arr {
        let desc = entry["desc"].as_str().unwrap();
        assert!(desc.contains("xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJA"));
        assert!(desc.contains("xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62"));
        assert!(desc.contains("xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxi"));
        assert!(desc.contains("b8688df1/48'/0'/0'/2'"));
        assert!(desc.contains("5436d724/48'/0'/0'/2'"));
        assert!(desc.contains("28645006/48'/0'/0'/2'"));
    }
}
