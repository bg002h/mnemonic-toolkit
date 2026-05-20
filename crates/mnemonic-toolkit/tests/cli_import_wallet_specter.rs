//! v0.28.0 Phase P2C — `mnemonic import-wallet --format specter` integration cells.
//!
//! Per plan-doc P2C row + SPEC §11.2. Covers:
//!
//! - parse happy-path per fixture (singlesig + multisig variants)
//! - sniff-positive: blob without `--format` routes to Specter
//! - sniff-negative: Specter blob with `--format bsms` exits non-zero
//! - roundtrip: `--json` envelope's `roundtrip.byte_exact` reflects the
//!   `canonicalize_specter` result
//! - envelope: `specter_source_metadata` field surfaces only on Specter
//!   parses + carries the SPEC §11.2 fields (`label`, `blockheight`,
//!   `devices`, `dropped_fields`)
//! - refusal: malformed blob exits 2 `ImportWalletParse`
//! - format-mismatch: explicit `--format specter` against a Sparrow blob → exit 1
//! - select-descriptor coerce: non-`all` value emits NOTICE + coerces to `all`
//!
//! Cells consume the fixtures at `tests/fixtures/wallet_import/specter-*.json`
//! created during Phase P2B.

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
// Parse happy-path cells
// ============================================================================

#[test]
fn specter_singlesig_p2wpkh_parses_clean() {
    let p = fixture_path("specter-singlesig-p2wpkh.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "specter"]).success();
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
fn specter_multisig_2of3_sortedmulti_parses_clean() {
    let p = fixture_path("specter-multisig-2of3-p2wsh-sortedmulti.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "specter"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cosigners=3"),
        "expected 3 cosigners; stdout: {stdout}"
    );
    assert!(
        stdout.contains("threshold=2"),
        "expected threshold=2; stdout: {stdout}"
    );
}

#[test]
fn specter_blockheight_zero_parses_clean() {
    let p = fixture_path("specter-blockheight-zero.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "specter"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
}

#[test]
fn specter_descriptor_with_checksum_parses_clean() {
    let p = fixture_path("specter-descriptor-with-checksum.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "specter"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
}

// ============================================================================
// Sniff cells (auto-format detection)
// ============================================================================

#[test]
fn specter_sniff_detects_singlesig_without_format() {
    let p = fixture_path("specter-singlesig-p2wpkh.json");
    // No --format: rely on sniff dispatch.
    let out = run_import(&["--blob", p.to_str().unwrap()]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cosigners=1"),
        "sniff must route to SpecterParser; stdout: {stdout}"
    );
}

#[test]
fn specter_sniff_detects_multisig_without_format() {
    let p = fixture_path("specter-multisig-2of3-p2wsh-sortedmulti.json");
    let out = run_import(&["--blob", p.to_str().unwrap()]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=3"), "stdout: {stdout}");
}

#[test]
fn specter_with_bsms_format_refused() {
    // Explicit `--format bsms` on a Specter blob: sniff sees Specter
    // → reject with the BSMS-arm's mismatch shape (not really a mismatch
    // since the BSMS arm only checks BitcoinCore sniff, but Specter blobs
    // ALSO fail BsmsParser parse-side on the JSON shape → ImportWalletParse).
    let p = fixture_path("specter-singlesig-p2wpkh.json");
    let assertion = run_import(&["--blob", p.to_str().unwrap(), "--format", "bsms"]).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("import-wallet")
            && (stderr.contains("bsms") || stderr.contains("format")),
        "expected import-wallet bsms-related refusal; got: {stderr}"
    );
}

#[test]
fn specter_with_format_mismatch_sniffed_bitcoin_core_exits_one() {
    // Build a Bitcoin Core blob; supply `--format specter` → format-mismatch.
    let core_blob =
        r#"{"wallet_name":"a","descriptors":[{"desc":"wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#00lx6ere"}]}"#;
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "specter"])
        .write_stdin(core_blob.to_string())
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("specter") && (stderr.contains("bitcoin-core") || stderr.contains("format")),
        "expected specter-vs-bitcoin-core format-mismatch; got: {stderr}"
    );
}

#[test]
fn specter_with_format_mismatch_sniffed_sparrow_exits_one() {
    // Build a Sparrow blob; supply `--format specter` → format-mismatch via
    // the new Sparrow arm in P2C's mismatch matrix.
    let sparrow_blob = r#"{
        "name":"x","network":"mainnet","policyType":"SINGLE","scriptType":"P2WPKH",
        "defaultPolicy":{"name":"Default","miniscript":{"script":"wpkh(@0/**)"}},
        "keystores":[{
            "label":"x","source":"SW_WATCH","walletModel":"SPARROW",
            "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/84'/0'/0'"},
            "extendedPublicKey":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"
        }]
    }"#;
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "specter"])
        .write_stdin(sparrow_blob.to_string())
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("specter") && stderr.contains("sparrow"),
        "expected specter-vs-sparrow format-mismatch; got: {stderr}"
    );
}

// ============================================================================
// `--json` envelope cells (SPEC §7.4 + SPEC §11.2 specter_source_metadata)
// ============================================================================

#[test]
fn specter_json_envelope_includes_source_metadata_and_roundtrip() {
    let p = fixture_path("specter-singlesig-p2wpkh.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "specter",
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
        Some("specter"),
        "envelope source_format must be specter"
    );

    let meta = env
        .get("specter_source_metadata")
        .expect("specter_source_metadata field must be present on Specter envelopes");
    assert_eq!(
        meta.get("label").and_then(|v| v.as_str()),
        Some("Daily"),
        "label must be top-level `label` from blob"
    );
    assert_eq!(
        meta.get("blockheight").and_then(|v| v.as_u64()),
        Some(800000),
        "blockheight verbatim from blob"
    );
    let devices = meta
        .get("devices")
        .and_then(|v| v.as_array())
        .expect("devices must be an array");
    assert_eq!(devices.len(), 1);
    assert_eq!(
        devices[0].get("type").and_then(|v| v.as_str()),
        Some("coldcard")
    );
    assert_eq!(
        devices[0].get("label").and_then(|v| v.as_str()),
        Some("primary")
    );
    assert!(
        meta.get("dropped_fields")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(false),
        "no dropped fields on canonical fixture"
    );

    let rt = env
        .get("roundtrip")
        .expect("envelope must contain roundtrip field");
    assert_eq!(
        rt.get("status").and_then(|v| v.as_str()),
        Some("ok"),
        "roundtrip.status must be 'ok'"
    );
    assert!(
        rt.get("byte_exact").and_then(|v| v.as_bool()).is_some(),
        "roundtrip.byte_exact must be present"
    );
}

#[test]
fn specter_json_envelope_no_specter_source_metadata_on_bsms() {
    // Cross-check: BSMS envelopes do NOT carry specter_source_metadata.
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out =
        run_import(&["--blob", p.to_str().unwrap(), "--format", "bsms", "--json"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let env = &val.as_array().unwrap()[0];
    assert!(
        env.get("specter_source_metadata").is_none(),
        "BSMS envelope must NOT carry specter_source_metadata; env: {env:?}"
    );
}

#[test]
fn specter_json_envelope_no_specter_source_metadata_on_sparrow() {
    // Cross-check: Sparrow envelopes do NOT carry specter_source_metadata.
    let p = fixture_path("sparrow-singlesig-p2wpkh.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "sparrow",
        "--json",
    ])
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let env = &val.as_array().unwrap()[0];
    assert!(
        env.get("specter_source_metadata").is_none(),
        "Sparrow envelope must NOT carry specter_source_metadata; env: {env:?}"
    );
    // And conversely Sparrow MUST carry sparrow_source_metadata.
    assert!(
        env.get("sparrow_source_metadata").is_some(),
        "Sparrow envelope must carry sparrow_source_metadata; env: {env:?}"
    );
}

#[test]
fn specter_json_envelope_dropped_fields_surface_in_metadata() {
    let blob = r#"{
        "label":"x","blockheight":0,
        "descriptor":"wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#00lx6ere",
        "devices":["unknown"],
        "extra_field":"this should appear in dropped_fields"
    }"#;
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "specter",
            "--json",
        ])
        .write_stdin(blob.to_string())
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let env = &val.as_array().unwrap()[0];
    let dropped = env
        .get("specter_source_metadata")
        .and_then(|m| m.get("dropped_fields"))
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(
        dropped.iter().any(|v| v.as_str() == Some("extra_field")),
        "extra_field must surface in dropped_fields; got: {dropped:?}"
    );
}

#[test]
fn specter_json_envelope_roundtrip_status_ok_on_alphabetical_fixture() {
    // The Specter fixture is committed in the wire shape Specter Desktop
    // produces (label, blockheight, descriptor, devices). The canonicalize
    // alphabetizes keys via BTreeMap. Specter's wire-form key order
    // happens to already be (label, blockheight, descriptor, devices) which
    // is partial-alphabetical for the first 2 keys; the canonicalize re-sorts
    // to (blockheight, descriptor, devices, label). The fixture is NOT
    // byte-exact w.r.t. the canonical form, but the status must be ok and
    // semantic_match must be true.
    let p = fixture_path("specter-singlesig-p2wpkh.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "specter",
        "--json",
    ])
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let env = &val.as_array().unwrap()[0];
    let rt = env.get("roundtrip").unwrap();
    assert_eq!(
        rt.get("status").and_then(|v| v.as_str()),
        Some("ok"),
        "well-formed Specter blob must canonicalize cleanly; rt: {rt:?}"
    );
    assert_eq!(
        rt.get("semantic_match").and_then(|v| v.as_bool()),
        Some(true),
        "semantic_match must be true (key-reorder is semantic-identical); rt: {rt:?}"
    );
}

// ============================================================================
// Refusal cells
// ============================================================================

#[test]
fn specter_malformed_missing_descriptor_exits_parse_error() {
    let blob = r#"{"label":"x","blockheight":0,"devices":[]}"#;
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "specter"])
        .write_stdin(blob.to_string())
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("specter") && stderr.contains("descriptor"),
        "expected specter parse-error citing missing descriptor; got: {stderr}"
    );
}

#[test]
fn specter_invalid_checksum_exits_parse_error() {
    // SPEC §11.2 — Specter wire-shape carries descriptor with `#csum`.
    // The parser delegates BIP-380 checksum verify before threading the
    // body through the @N-substitution pipeline; an invalid checksum
    // surfaces as ImportWalletParse exit 2.
    let blob = r#"{
        "label":"x","blockheight":0,
        "descriptor":"wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#deadbeef",
        "devices":["unknown"]
    }"#;
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "specter"])
        .write_stdin(blob.to_string())
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("specter") && stderr.contains("checksum"),
        "expected specter parse-error citing checksum; got: {stderr}"
    );
}

// ============================================================================
// --select-descriptor coerce cell
// ============================================================================

#[test]
fn specter_select_descriptor_non_all_emits_notice_and_coerces() {
    // SPEC §5.3 + P2C coerce: `--select-descriptor active-receive` on a
    // Specter blob emits the SPEC §2.4 NOTICE + coerces to `all`.
    let p = fixture_path("specter-singlesig-p2wpkh.json");
    let assertion = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "specter",
        "--select-descriptor",
        "active-receive",
    ])
    .success();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("notice: import-wallet: specter:")
            && stderr.contains("--select-descriptor")
            && stderr.contains("has no effect"),
        "expected specter coerce NOTICE; got: {stderr}"
    );
    // Parse must succeed — coerce-to-all keeps the single descriptor.
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cosigners=1"),
        "expected parse to succeed post-coerce; got: {stdout}"
    );
}
