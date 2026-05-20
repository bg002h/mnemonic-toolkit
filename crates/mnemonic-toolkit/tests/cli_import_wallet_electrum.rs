//! v0.28.0 Phase P6C — `mnemonic import-wallet --format electrum` integration cells.
//!
//! Per plan-doc P6C row + SPEC §11.6. Covers:
//!
//! - parse happy-path per fixture (standard BIP-84, standard BIP-49, multisig 2-of-3)
//! - sniff-positive: blob without `--format` routes to Electrum
//! - format-mismatch: explicit `--format electrum` against a non-Electrum
//!   blob (BSMS / Bitcoin Core / Coldcard) → exit 1
//! - `--json` envelope: `electrum_source_metadata` field surfaces only on
//!   Electrum parses + carries SPEC §11.6 fields (seed_version, wallet_type,
//!   wallet_name, dropped_fields)
//! - `--json` envelope: roundtrip status semantics
//! - refusal: 2fa / imported / encrypted fixtures exit with SPEC §11.6.1
//!   refusal templates
//!
//! Cells consume the fixtures at `tests/fixtures/wallet_import/electrum-*.json`
//! created during Phase P6B.

use assert_cmd::Command;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

fn run_import(args: &[&str]) -> assert_cmd::assert::Assert {
    let mut cmd = Command::cargo_bin("mnemonic").unwrap();
    cmd.arg("import-wallet").args(args).assert()
}

// ============================================================================
// Parse happy-path cells (one per fixture)
// ============================================================================

#[test]
fn electrum_standard_bip84_mainnet_parses_clean() {
    let p = fixture_path("electrum-standard-bip84-mainnet.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "electrum"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cosigners=1"),
        "expected single-cosigner summary; stdout: {stdout}"
    );
    assert!(
        stdout.contains("network=mainnet"),
        "expected mainnet network; stdout: {stdout}"
    );
}

#[test]
fn electrum_standard_bip49_mainnet_parses_clean() {
    let p = fixture_path("electrum-standard-bip49-mainnet.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "electrum"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
}

#[test]
fn electrum_multisig_2of3_wsh_parses_clean() {
    let p = fixture_path("electrum-multisig-2of3-wsh.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "electrum"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cosigners=3"),
        "expected 3 cosigners; stdout: {stdout}"
    );
    assert!(
        stdout.contains("network=mainnet"),
        "expected mainnet; stdout: {stdout}"
    );
}

// ============================================================================
// Sniff cells (auto-format detection)
// ============================================================================

#[test]
fn electrum_sniff_detects_standard_without_format() {
    let p = fixture_path("electrum-standard-bip84-mainnet.json");
    // No --format: sniff detects via seed_version + wallet_type.
    let out = run_import(&["--blob", p.to_str().unwrap()]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cosigners=1"),
        "sniff must route to ElectrumParser; stdout: {stdout}"
    );
}

#[test]
fn electrum_sniff_detects_multisig_without_format() {
    let p = fixture_path("electrum-multisig-2of3-wsh.json");
    let out = run_import(&["--blob", p.to_str().unwrap()]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cosigners=3"),
        "sniff must route multisig to ElectrumParser; stdout: {stdout}"
    );
}

// ============================================================================
// Format-mismatch cells (SPEC §6.1)
// ============================================================================

#[test]
fn electrum_with_format_mismatch_sniffed_bsms_exits_one() {
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let assertion = run_import(&["--blob", p.to_str().unwrap(), "--format", "electrum"]).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("electrum") && stderr.contains("bsms"),
        "expected electrum-vs-bsms format-mismatch; got: {stderr}"
    );
}

#[test]
fn electrum_with_format_mismatch_sniffed_coldcard_exits_one() {
    let p = fixture_path("coldcard-singlesig-bip84-mainnet.json");
    let assertion = run_import(&["--blob", p.to_str().unwrap(), "--format", "electrum"]).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("electrum") && stderr.contains("coldcard"),
        "expected electrum-vs-coldcard format-mismatch; got: {stderr}"
    );
}

// ============================================================================
// Refusal cells (SPEC §11.6.1)
// ============================================================================

#[test]
fn electrum_2fa_fixture_refuses_with_trustedcoin_message() {
    let p = fixture_path("electrum-2fa-refused.json");
    let assertion = run_import(&["--blob", p.to_str().unwrap(), "--format", "electrum"]).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("2fa") && stderr.contains("TrustedCoin"),
        "expected 2fa refusal with TrustedCoin reference; got: {stderr}"
    );
}

#[test]
fn electrum_imported_fixture_refuses_with_derivation_chain_message() {
    let p = fixture_path("electrum-imported-refused.json");
    let assertion = run_import(&["--blob", p.to_str().unwrap(), "--format", "electrum"]).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("imported-addresses") && stderr.contains("derivation chain"),
        "expected imported-addresses refusal; got: {stderr}"
    );
}

#[test]
fn electrum_encrypted_fixture_refuses_with_decrypt_wallet_message() {
    let p = fixture_path("electrum-encrypted-refused.json");
    let assertion = run_import(&["--blob", p.to_str().unwrap(), "--format", "electrum"]).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("encrypted") && stderr.contains("decrypt-wallet"),
        "expected encrypted refusal; got: {stderr}"
    );
}

// ============================================================================
// `--json` envelope cells (SPEC §11.6 electrum_source_metadata + roundtrip)
// ============================================================================

#[test]
fn electrum_json_envelope_includes_source_metadata_and_roundtrip() {
    let p = fixture_path("electrum-standard-bip84-mainnet.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "electrum",
        "--json",
    ])
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("invalid JSON: {e}\nstdout was:\n{stdout}"));
    let arr = val.as_array().expect("--json must emit array");
    assert_eq!(arr.len(), 1, "expected one envelope");
    let env = &arr[0];

    assert_eq!(
        env.get("source_format").and_then(|v| v.as_str()),
        Some("electrum"),
        "envelope source_format must be electrum"
    );

    let meta = env
        .get("electrum_source_metadata")
        .expect("electrum_source_metadata field must be present on Electrum envelopes");
    assert_eq!(
        meta.get("seed_version").and_then(|v| v.as_u64()),
        Some(17),
        "seed_version echoes blob"
    );
    assert_eq!(
        meta.get("wallet_type").and_then(|v| v.as_str()),
        Some("standard"),
        "wallet_type echoes blob"
    );
    assert_eq!(
        meta.get("wallet_name").and_then(|v| v.as_str()),
        Some("Daily"),
        "wallet_name carried from keystore.label"
    );

    let rt = env
        .get("roundtrip")
        .expect("envelope must contain roundtrip field");
    assert_eq!(
        rt.get("status").and_then(|v| v.as_str()),
        Some("ok"),
        "roundtrip.status must be 'ok'"
    );
}

#[test]
fn electrum_json_envelope_multisig_carries_kofn_wallet_type() {
    let p = fixture_path("electrum-multisig-2of3-wsh.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "electrum",
        "--json",
    ])
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let env = &val.as_array().unwrap()[0];
    let meta = env.get("electrum_source_metadata").unwrap();
    assert_eq!(
        meta.get("wallet_type").and_then(|v| v.as_str()),
        Some("2of3"),
        "multisig wallet_type must be canonical <k>of<n>"
    );
}

#[test]
fn electrum_json_envelope_no_electrum_source_metadata_on_bsms() {
    // Cross-check: BSMS envelopes do NOT carry electrum_source_metadata.
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out =
        run_import(&["--blob", p.to_str().unwrap(), "--format", "bsms", "--json"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let env = &val.as_array().unwrap()[0];
    assert!(
        env.get("electrum_source_metadata").is_none(),
        "BSMS envelope must NOT carry electrum_source_metadata; env: {env:?}"
    );
}

#[test]
fn electrum_json_envelope_no_electrum_source_metadata_on_coldcard() {
    let p = fixture_path("coldcard-singlesig-bip84-mainnet.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "coldcard",
        "--json",
    ])
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let env = &val.as_array().unwrap()[0];
    assert!(
        env.get("electrum_source_metadata").is_none(),
        "Coldcard envelope must NOT carry electrum_source_metadata; env: {env:?}"
    );
}

// ============================================================================
// Malformed-blob cells
// ============================================================================

#[test]
fn electrum_malformed_json_exits_parse_error() {
    let blob = r#"{not json"#;
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "electrum"])
        .write_stdin(blob.to_string())
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("electrum") && stderr.contains("invalid JSON"),
        "expected electrum parse-error citing invalid JSON; got: {stderr}"
    );
}
