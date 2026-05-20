//! v0.28.0 Phase P3C — Coldcard single-sig wallet.json import integration
//! tests.
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.3 + §11.3.1. Exercises
//! the library boundary via the CLI scaffold (`cmd/import_wallet.rs`)
//! extended in this phase to dispatch `--format coldcard` to
//! `ColdcardParser` + emit Coldcard-specific source-metadata + roundtrip
//! envelopes per SPEC §7.4.
//!
//! Self-contained: no dependency on adjacent repos or external network.
//! Fixtures live at `tests/fixtures/wallet_import/coldcard-*.json` (5
//! files: BIP-44/49/84/86 mainnet + BIP-84 testnet).

use assert_cmd::Command;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

/// Run import-wallet against a fixture file with explicit `--format coldcard`.
fn run_coldcard_explicit(name: &str) -> assert_cmd::assert::Assert {
    let p = fixture_path(name);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob"])
        .arg(&p)
        .args(["--format", "coldcard"])
        .assert()
}

/// Run import-wallet against a fixture file via auto-sniff (no `--format`).
fn run_coldcard_autosniff(name: &str) -> assert_cmd::assert::Assert {
    let p = fixture_path(name);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob"])
        .arg(&p)
        .assert()
}

/// Run with --json flag to get the envelope output for source-metadata
/// assertions.
fn run_coldcard_json(name: &str) -> assert_cmd::assert::Assert {
    let p = fixture_path(name);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob"])
        .arg(&p)
        .args(["--format", "coldcard", "--json"])
        .assert()
}

// ============================================================================
// Happy path per BIP variant
// ============================================================================

#[test]
fn coldcard_bip84_mainnet_explicit_format_summary() {
    let out = run_coldcard_explicit("coldcard-bip84-mainnet.json").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    assert!(stdout.contains("entropy=none"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=none"), "stdout: {stdout}");
}

#[test]
fn coldcard_bip49_mainnet_explicit_format_summary() {
    let out = run_coldcard_explicit("coldcard-bip49-mainnet.json").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"));
    assert!(stdout.contains("cosigners=1"));
    assert!(stdout.contains("network=mainnet"));
}

#[test]
fn coldcard_bip44_mainnet_explicit_format_summary() {
    let out = run_coldcard_explicit("coldcard-bip44-mainnet.json").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"));
    assert!(stdout.contains("network=mainnet"));
}

#[test]
fn coldcard_bip84_testnet_explicit_format_summary() {
    let out = run_coldcard_explicit("coldcard-bip84-testnet.json").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"));
    assert!(stdout.contains("network=testnet"), "stdout: {stdout}");
}

#[test]
fn coldcard_bip86_mainnet_explicit_format_summary() {
    let out = run_coldcard_explicit("coldcard-bip86-mainnet.json").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"));
    assert!(stdout.contains("network=mainnet"));
}

// ============================================================================
// Auto-sniff dispatch (no --format)
// ============================================================================

#[test]
fn coldcard_bip84_mainnet_auto_sniff_dispatches_coldcard() {
    let out = run_coldcard_autosniff("coldcard-bip84-mainnet.json").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "auto-sniff must dispatch coldcard; stdout: {stdout}");
}

#[test]
fn coldcard_bip86_mainnet_auto_sniff_dispatches_coldcard() {
    let out = run_coldcard_autosniff("coldcard-bip86-mainnet.json").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"));
}

// ============================================================================
// --json envelope assertions
// ============================================================================

#[test]
fn coldcard_json_envelope_carries_coldcard_metadata_block() {
    let out = run_coldcard_json("coldcard-bip84-mainnet.json").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let env: serde_json::Value =
        serde_json::from_str(&stdout).expect("--json must produce parseable JSON");
    let arr = env.as_array().expect("--json envelope is an array");
    assert_eq!(arr.len(), 1, "single-sig coldcard → 1 envelope entry");
    let e = &arr[0];
    assert_eq!(
        e.get("source_format").and_then(|v| v.as_str()),
        Some("coldcard"),
        "source_format must be `coldcard`; got: {e}"
    );
    let meta = e
        .get("coldcard_metadata")
        .expect("coldcard_metadata field must be present");
    assert_eq!(meta.get("chain").and_then(|v| v.as_str()), Some("BTC"));
    assert_eq!(meta.get("xfp").and_then(|v| v.as_str()), Some("5436D724"));
    assert_eq!(
        meta.get("bip_derivation").and_then(|v| v.as_str()),
        Some("bip84")
    );
    assert_eq!(meta.get("account").and_then(|v| v.as_u64()), Some(0));
    // dropped_fields includes the bip84 block's `_pub` + `first` + `name`.
    let dropped = meta
        .get("dropped_fields")
        .and_then(|v| v.as_array())
        .expect("dropped_fields must be an array");
    let dropped_str: Vec<&str> = dropped.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        dropped_str.iter().any(|s| s.contains("first")),
        "dropped_fields should mention `first`; got: {dropped_str:?}"
    );
    assert!(
        dropped_str.iter().any(|s| s.contains("_pub")),
        "dropped_fields should mention `_pub`; got: {dropped_str:?}"
    );
}

#[test]
fn coldcard_json_envelope_carries_roundtrip_status_ok() {
    let out = run_coldcard_json("coldcard-bip84-mainnet.json").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let env: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = env.as_array().unwrap();
    let rt = arr[0].get("roundtrip").expect("roundtrip field must be present");
    assert_eq!(
        rt.get("status").and_then(|v| v.as_str()),
        Some("ok"),
        "round-trip must succeed; got: {rt}"
    );
    assert_eq!(
        rt.get("semantic_match").and_then(|v| v.as_bool()),
        Some(true),
        "semantic_match must be true; got: {rt}"
    );
}

#[test]
fn coldcard_json_envelope_bundle_carries_descriptor() {
    let out = run_coldcard_json("coldcard-bip84-mainnet.json").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let env: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = env.as_array().unwrap();
    let bundle = arr[0]
        .get("bundle")
        .expect("bundle field must be present in coldcard envelope");
    let descriptor = bundle
        .get("descriptor")
        .and_then(|v| v.as_str())
        .expect("bundle.descriptor must be a string for coldcard");
    assert!(
        descriptor.starts_with("wpkh("),
        "BIP-84 → wpkh wrapper; descriptor: {descriptor}"
    );
    assert!(
        descriptor.contains("/<0;1>/*"),
        "multipath suffix must be present; descriptor: {descriptor}"
    );
}

#[test]
fn coldcard_json_envelope_bip86_emits_tr_descriptor() {
    let out = run_coldcard_json("coldcard-bip86-mainnet.json").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let env: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = env.as_array().unwrap();
    let descriptor = arr[0]
        .get("bundle")
        .and_then(|b| b.get("descriptor"))
        .and_then(|v| v.as_str())
        .unwrap();
    assert!(
        descriptor.starts_with("tr("),
        "BIP-86 → tr wrapper; got: {descriptor}"
    );
}

// ============================================================================
// Stderr notice for dropped fields
// ============================================================================

#[test]
fn coldcard_emits_stderr_notice_on_dropped_fields() {
    let out = run_coldcard_explicit("coldcard-bip84-mainnet.json").success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("dominant-BIP bip84"),
        "stderr should mention dominant-BIP selection; got: {stderr}"
    );
    assert!(
        stderr.contains("dropped fields"),
        "stderr should mention dropped fields; got: {stderr}"
    );
}

// ============================================================================
// Refusal cases
// ============================================================================

#[test]
fn coldcard_explicit_format_against_bsms_blob_returns_format_mismatch() {
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob"])
        .arg(&p)
        .args(["--format", "coldcard"])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard") && stderr.contains("bsms"),
        "FormatMismatch must mention both supplied + sniffed formats; got: {stderr}"
    );
}

#[test]
fn coldcard_explicit_format_against_bitcoin_core_blob_returns_format_mismatch() {
    let p = fixture_path("core-bip84-mainnet.json");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob"])
        .arg(&p)
        .args(["--format", "coldcard"])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard") && stderr.contains("bitcoin-core"),
        "FormatMismatch must mention both formats; got: {stderr}"
    );
}

#[test]
fn coldcard_malformed_json_returns_parse_error() {
    // Pipe an invalid JSON blob to test the JSON-parse error path.
    let blob = b"{not valid json}";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "coldcard"])
        .write_stdin(&blob[..])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("invalid JSON") || stderr.contains("parse error"),
        "malformed JSON should surface parse error; got: {stderr}"
    );
}

#[test]
fn coldcard_missing_bip_block_returns_parse_error() {
    let blob = br#"{"chain":"BTC","xfp":"5436D724","account":0}"#;
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "coldcard"])
        .write_stdin(&blob[..])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // Sniff yields NoMatch (clause 3 requires at-least-one-of bip* / xpub /
    // bip48_*), but explicit --format coldcard bypasses sniff. Parse fails
    // at the dominant-BIP selection step with the "no recognized BIP" error.
    assert!(
        stderr.contains("no recognized BIP-derivation block")
            || stderr.contains("could not detect format")
            || stderr.contains("matches multiple"),
        "missing-bip-block should surface clear refusal; got: {stderr}"
    );
}

// ============================================================================
// Roundtrip parity (canonicalize regression guard)
// ============================================================================

#[test]
fn coldcard_roundtrip_byte_exact_when_blob_in_canonical_form() {
    // The fixture file is already pretty-printed alphabetically (BTreeMap
    // canonicalize order); however, the source blob includes `_pub` +
    // `first` fields that canonicalize drops. The byte_exact flag will be
    // FALSE for this case (since _pub + first are present in input but
    // absent in canon output). Pin this behavior — round-trip surfaces a
    // diff but semantic_match remains TRUE.
    let out = run_coldcard_json("coldcard-bip84-mainnet.json").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let env: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = env.as_array().unwrap();
    let rt = arr[0].get("roundtrip").unwrap();
    assert_eq!(
        rt.get("byte_exact").and_then(|v| v.as_bool()),
        Some(false),
        "byte_exact should be false (fixture has _pub + first; canon drops them)"
    );
    assert_eq!(
        rt.get("semantic_match").and_then(|v| v.as_bool()),
        Some(true),
        "semantic_match should remain true after dropping ephemeral fields"
    );
    // A diff field should be present (unified-diff non-empty).
    let diff = rt.get("diff").expect("diff field present");
    assert!(
        diff.is_string() && !diff.as_str().unwrap().is_empty(),
        "diff must be a non-empty unified-diff string; got: {diff}"
    );
}
