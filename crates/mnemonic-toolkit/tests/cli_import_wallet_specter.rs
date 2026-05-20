//! v0.28.0 Phase P2C — `mnemonic import-wallet --format specter` integration tests.
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.2 + plan-doc
//! `/home/bcg/.claude/plans/unified-meandering-sundae.md` P2C row.
//!
//! Tests the CLI dispatch surface (cmd/import_wallet.rs) wired to
//! `SpecterParser::parse` + `canonicalize_specter`. Companion to the unit
//! coverage in `crates/mnemonic-toolkit/src/wallet_import/specter.rs::tests`
//! (which exercises the parser API directly) and
//! `crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs::tests`
//! (which exercises the canonicalize helper directly).
//!
//! Cell shapes:
//! - happy-path parse cells: --format specter + --blob <fixture> → exit 0,
//!   correct stdout summary (bundles/cosigners/network).
//! - sniff dispatch: omit --format; assert exit 0 + correct dispatch routing.
//! - sniff mismatch: --format specter against a BSMS / Core blob → exit 1
//!   `ImportWalletFormatMismatch`.
//! - `--json` envelope: parse fixture → envelope contains `source_format:
//!   "specter"`, `source_metadata.{label,blockheight,devices,dropped_fields}`,
//!   `roundtrip.{byte_exact,semantic_match,status}`.
//! - `--select-descriptor` coerce: Specter is single-descriptor → non-`all`
//!   coerces to `all` with stderr NOTICE.
//! - roundtrip not-byte-exact-but-semantic-match for blobs with extra fields.

use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

fn run_specter_file(name: &str, extra: &[&str]) -> assert_cmd::assert::Assert {
    let p = fixture_path(name);
    let mut cmd = Command::cargo_bin("mnemonic").unwrap();
    cmd.args(["import-wallet", "--blob"])
        .arg(p)
        .args(["--format", "specter"])
        .args(extra)
        .assert()
}

fn run_specter_stdin(blob: &str, extra: &[&str]) -> assert_cmd::assert::Assert {
    let mut cmd = Command::cargo_bin("mnemonic").unwrap();
    cmd.args(["import-wallet", "--blob", "-", "--format", "specter"])
        .args(extra)
        .write_stdin(blob.to_string())
        .assert()
}

fn run_auto_sniff_file(name: &str) -> assert_cmd::assert::Assert {
    let p = fixture_path(name);
    let mut cmd = Command::cargo_bin("mnemonic").unwrap();
    cmd.args(["import-wallet", "--blob"]).arg(p).assert()
}

// ============================================================================
// Happy-path parse cells (SPEC §11.2)
// ============================================================================

#[test]
fn specter_singlesig_p2wpkh_coldcard_parses() {
    let out = run_specter_file("specter-singlesig-p2wpkh-coldcard.json", &[]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    assert!(stdout.contains("entropy=none"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=none"), "stdout: {stdout}");
}

#[test]
fn specter_multisig_2of3_wsh_sortedmulti_parses() {
    let out = run_specter_file("specter-multisig-2of3-wsh-sortedmulti.json", &[]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains("cosigners=3"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=2"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
}

#[test]
fn specter_blockheight_zero_with_legacy_string_devices_parses() {
    let out = run_specter_file("specter-blockheight-zero.json", &[]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
}

#[test]
fn specter_with_checksum_parses() {
    let out = run_specter_file("specter-with-checksum.json", &[]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
}

// ============================================================================
// Sniff auto-dispatch cells (SPEC §6.2 + P2A sniff wiring)
// ============================================================================

#[test]
fn specter_auto_sniff_routes_to_specter_parser() {
    // Omit --format; auto-sniff routes to SpecterParser via
    // `SniffOutcome::Specter` (P2A wired the variant + sniff predicate;
    // P2C wired the cmd-side dispatch arm).
    let out = run_auto_sniff_file("specter-singlesig-p2wpkh-coldcard.json").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
}

#[test]
fn specter_auto_sniff_multisig_routes_to_specter_parser() {
    let out = run_auto_sniff_file("specter-multisig-2of3-wsh-sortedmulti.json").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=3"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=2"), "stdout: {stdout}");
}

// ============================================================================
// Sniff-mismatch cells (Site 2 — explicit --format specter against non-Specter blob)
// ============================================================================

#[test]
fn specter_format_override_against_bsms_blob_errors_mismatch() {
    // Pass --format specter for a BSMS fixture → ImportWalletFormatMismatch.
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob"])
        .arg(&p)
        .args(["--format", "specter"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("specter") && stderr.contains("bsms"),
        "expected ImportWalletFormatMismatch mentioning specter + bsms; got: {stderr}"
    );
}

#[test]
fn specter_format_override_against_bitcoin_core_blob_errors_mismatch() {
    let p = fixture_path("core-bip84-mainnet.json");
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob"])
        .arg(&p)
        .args(["--format", "specter"])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("specter") && stderr.contains("bitcoin-core"),
        "expected ImportWalletFormatMismatch mentioning specter + bitcoin-core; got: {stderr}"
    );
}

// ============================================================================
// `--json` envelope cells (SPEC §3.2 + Site 6/7 wiring)
// ============================================================================

#[test]
fn specter_json_envelope_contains_source_format_and_metadata() {
    let out = run_specter_file("specter-singlesig-p2wpkh-coldcard.json", &["--json"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let parsed: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("--json output must parse as JSON; got error {e}; stdout: {stdout}"));
    let envelope_array = parsed.as_array().expect("envelope is JSON array");
    assert_eq!(envelope_array.len(), 1, "single bundle expected: {stdout}");
    let env = &envelope_array[0];
    assert_eq!(env["source_format"], Value::String("specter".to_string()));
    assert_eq!(env["schema_version"], Value::String("1".to_string()));
    // source_metadata for specter carries label + blockheight + devices + dropped_fields.
    let meta = &env["source_metadata"];
    assert_eq!(meta["label"], Value::String("Daily Spending".to_string()));
    assert_eq!(meta["blockheight"], serde_json::json!(850000));
    let devices = meta["devices"].as_array().expect("devices is array");
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0]["type"], Value::String("coldcard".to_string()));
    assert_eq!(devices[0]["label"], Value::String("Primary Signer".to_string()));
    assert!(meta["dropped_fields"].as_array().unwrap().is_empty());
}

#[test]
fn specter_json_envelope_contains_roundtrip_object() {
    let out = run_specter_file("specter-singlesig-p2wpkh-coldcard.json", &["--json"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let parsed: Value = serde_json::from_str(&stdout).unwrap();
    let env = &parsed.as_array().unwrap()[0];
    let rt = &env["roundtrip"];
    // Roundtrip wire shape per cmd/import_wallet.rs Site 7 — same as bitcoin-core arm.
    assert!(rt.get("byte_exact").is_some(), "missing byte_exact: {rt:?}");
    assert!(rt.get("semantic_match").is_some(), "missing semantic_match: {rt:?}");
    assert!(rt.get("status").is_some(), "missing status: {rt:?}");
    assert_eq!(rt["status"], Value::String("ok".to_string()));
    assert_eq!(rt["semantic_match"], Value::Bool(true));
}

#[test]
fn specter_json_envelope_multisig_includes_three_devices() {
    let out = run_specter_file("specter-multisig-2of3-wsh-sortedmulti.json", &["--json"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let parsed: Value = serde_json::from_str(&stdout).unwrap();
    let env = &parsed.as_array().unwrap()[0];
    let devices = env["source_metadata"]["devices"].as_array().expect("devices array");
    assert_eq!(devices.len(), 3, "3 devices: {env}");
    let types: Vec<&str> = devices.iter().map(|d| d["type"].as_str().unwrap()).collect();
    assert!(types.contains(&"coldcard"));
    assert!(types.contains(&"trezor"));
    assert!(types.contains(&"ledger"));
}

#[test]
fn specter_json_envelope_legacy_string_devices_normalized_to_object() {
    // The blockheight-zero fixture uses legacy string-form `["unknown"]`.
    // The envelope wire-shape MUST be object-form regardless (envelope is
    // the canonical shape, not the input shape).
    let out = run_specter_file("specter-blockheight-zero.json", &["--json"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let parsed: Value = serde_json::from_str(&stdout).unwrap();
    let env = &parsed.as_array().unwrap()[0];
    let devices = env["source_metadata"]["devices"].as_array().unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0]["type"], Value::String("unknown".to_string()));
    assert_eq!(devices[0]["label"], Value::String(String::new()));
}

// ============================================================================
// --select-descriptor coerce cells (Site 5 wiring)
// ============================================================================

#[test]
fn specter_select_descriptor_non_default_coerces_with_notice() {
    let assertion = run_specter_file(
        "specter-singlesig-p2wpkh-coldcard.json",
        &["--select-descriptor", "active-receive"],
    )
    .success();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("notice: import-wallet: specter:")
            && stderr.contains("--select-descriptor active-receive has no effect"),
        "expected coerce NOTICE; got stderr: {stderr}"
    );
}

#[test]
fn specter_select_descriptor_all_silent() {
    // Default `all` produces no coerce NOTICE.
    let assertion =
        run_specter_file("specter-singlesig-p2wpkh-coldcard.json", &["--select-descriptor", "all"])
            .success();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("--select-descriptor"),
        "expected no coerce NOTICE on `all`; got stderr: {stderr}"
    );
}

// ============================================================================
// Stdin + stderr-discipline cells
// ============================================================================

#[test]
fn specter_stdin_blob_parses_identically_to_file_form() {
    let blob = std::fs::read_to_string(fixture_path("specter-singlesig-p2wpkh-coldcard.json"))
        .expect("fixture readable");
    let stdin_out = run_specter_stdin(&blob, &[]).success();
    let stdin_stdout = String::from_utf8(stdin_out.get_output().stdout.clone()).unwrap();
    let file_out = run_specter_file("specter-singlesig-p2wpkh-coldcard.json", &[]).success();
    let file_stdout = String::from_utf8(file_out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdin_stdout, file_stdout, "stdin + file forms must produce identical stdout");
}

#[test]
fn specter_dropped_top_level_fields_emits_notice() {
    // Inject an unknown top-level field; parse succeeds + emits NOTICE.
    let blob = r#"{
  "label": "Custom",
  "blockheight": 100,
  "descriptor": "wpkh([b8688df1/84'/0'/0']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#5ql5mvwg",
  "devices": [{"type": "coldcard", "label": ""}],
  "vendor_extension_x": "extra data",
  "metadata_y": 42
}"#;
    let assertion = run_specter_stdin(blob, &[]).success();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("dropped unrecognized top-level fields") && stderr.contains("vendor_extension_x"),
        "expected dropped-fields NOTICE; got stderr: {stderr}"
    );
}

#[test]
fn specter_invalid_json_fails_with_parse_error() {
    let blob = r#"{not even close"#;
    let assertion = run_specter_stdin(blob, &[]).failure().code(2);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("import-wallet: specter: parse error") && stderr.contains("invalid JSON"),
        "expected invalid-JSON parse error; got stderr: {stderr}"
    );
}

#[test]
fn specter_invalid_checksum_fails_with_parse_error() {
    let blob = r#"{
  "label": "x",
  "blockheight": 0,
  "descriptor": "wpkh([b8688df1/84'/0'/0']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#deadbeef",
  "devices": [{"type": "coldcard", "label": ""}]
}"#;
    let assertion = run_specter_stdin(blob, &[]).failure().code(2);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("BIP-380 checksum validation failed"),
        "expected checksum error; got: {stderr}"
    );
}
