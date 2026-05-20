//! v0.28.0 Phase P1C — `mnemonic import-wallet --format sparrow` integration cells.
//!
//! Per plan-doc P1C row + SPEC §11.1. Covers:
//!
//! - parse happy-path per fixture (singlesig + multisig variants)
//! - sniff-positive: blob without `--format` routes to Sparrow
//! - sniff-negative: Sparrow blob with `--format bsms` exits non-zero
//! - roundtrip: `--json` envelope's `roundtrip.byte_exact` reflects the
//!   `canonicalize_sparrow` result
//! - envelope: `sparrow_source_metadata` field surfaces only on Sparrow
//!   parses + carries the SPEC §11.1 fields
//! - refusal: malformed blob exits 2 `ImportWalletParse`
//! - refusal: taproot blob exits 2 (P1B descriptor-passthrough deferral)
//!
//! Cells consume the fixtures at `tests/fixtures/wallet_import/sparrow-*.json`
//! created during Phase P1B.

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
fn sparrow_singlesig_p2wpkh_parses_clean() {
    let p = fixture_path("sparrow-singlesig-p2wpkh.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "sparrow"]).success();
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
fn sparrow_multisig_2of3_sortedmulti_parses_clean() {
    let p = fixture_path("sparrow-multisig-2of3-p2wsh-sortedmulti.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "sparrow"]).success();
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
fn sparrow_multisig_2of3_multi_ordered_parses_clean() {
    let p = fixture_path("sparrow-multisig-2of3-p2wsh-multi-ordered.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "sparrow"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=3"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=2"), "stdout: {stdout}");
}

#[test]
fn sparrow_singlesig_p2sh_p2wpkh_parses_clean() {
    let p = fixture_path("sparrow-singlesig-p2sh-p2wpkh.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "sparrow"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
}

// ============================================================================
// Sniff cells (auto-format detection)
// ============================================================================

#[test]
fn sparrow_sniff_detects_singlesig_without_format() {
    let p = fixture_path("sparrow-singlesig-p2wpkh.json");
    // No --format: rely on sniff dispatch.
    let out = run_import(&["--blob", p.to_str().unwrap()]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cosigners=1"),
        "sniff must route to SparrowParser; stdout: {stdout}"
    );
}

#[test]
fn sparrow_sniff_detects_multisig_without_format() {
    let p = fixture_path("sparrow-multisig-2of3-p2wsh-sortedmulti.json");
    let out = run_import(&["--blob", p.to_str().unwrap()]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=3"), "stdout: {stdout}");
}

#[test]
fn sparrow_with_bsms_format_refused() {
    // Explicit `--format bsms` on a Sparrow blob: sniff says Sparrow,
    // user says BSMS → ImportWalletParse / ImportWalletFormatMismatch
    // exit non-zero. Either exit code is acceptable acceptance for this
    // assertion; the test pins the refusal SHAPE, not the precise variant
    // (the BSMS arm's mismatch-check at site 2 only catches BitcoinCore
    // sniffs, so Sparrow blobs fall through to BsmsParser which fails on
    // the JSON shape → ImportWalletParse).
    let p = fixture_path("sparrow-singlesig-p2wpkh.json");
    let assertion = run_import(&["--blob", p.to_str().unwrap(), "--format", "bsms"]).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("import-wallet")
            && (stderr.contains("bsms") || stderr.contains("format")),
        "expected import-wallet bsms-related refusal; got: {stderr}"
    );
}

// ============================================================================
// `--json` envelope cells (SPEC §7.4 + SPEC §11.1 sparrow_source_metadata)
// ============================================================================

#[test]
fn sparrow_json_envelope_includes_source_metadata_and_roundtrip() {
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
    let val: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("invalid JSON: {e}\nstdout was:\n{stdout}"));
    let arr = val.as_array().expect("--json must emit array");
    assert_eq!(arr.len(), 1, "expected one envelope");
    let env = &arr[0];

    // source_format = "sparrow"
    assert_eq!(
        env.get("source_format").and_then(|v| v.as_str()),
        Some("sparrow"),
        "envelope source_format must be sparrow"
    );

    // sparrow_source_metadata present with SPEC §11.1 fields.
    let meta = env
        .get("sparrow_source_metadata")
        .expect("sparrow_source_metadata field must be present on Sparrow envelopes");
    assert_eq!(
        meta.get("label").and_then(|v| v.as_str()),
        Some("bip84-0"),
        "label must be top-level `name` from blob"
    );
    assert_eq!(
        meta.get("policy_type").and_then(|v| v.as_str()),
        Some("SINGLE"),
        "policy_type for SINGLE blob"
    );
    assert_eq!(
        meta.get("script_type").and_then(|v| v.as_str()),
        Some("P2WPKH"),
        "script_type verbatim from blob"
    );
    assert!(
        meta.get("dropped_fields")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(false),
        "no dropped fields on canonical fixture"
    );

    // roundtrip section: status=ok (canonicalize succeeded for a well-
    // formed Sparrow blob).
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
fn sparrow_json_envelope_no_sparrow_source_metadata_on_bsms() {
    // Cross-check: BSMS envelopes do NOT carry sparrow_source_metadata.
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "bsms", "--json"])
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let env = &val.as_array().unwrap()[0];
    assert!(
        env.get("sparrow_source_metadata").is_none(),
        "BSMS envelope must NOT carry sparrow_source_metadata; env: {env:?}"
    );
}

#[test]
fn sparrow_json_envelope_roundtrip_status_ok_on_well_formed_fixture() {
    // The Sparrow-native fixture is written in Sparrow's emit field order
    // (`name`, `network`, `policyType`, `scriptType`, `defaultPolicy`,
    // `keystores`) which is NOT alphabetical. `canonicalize_sparrow`
    // re-emits in alphabetical key order via BTreeMap — so byte_exact
    // for the Sparrow-native input is EXPECTED to be false (the
    // canonicalize is normalizing the field order). What the consumer
    // needs from this cell is the `status: "ok"` signal + the diff being
    // a key-reorder (semantic_match=true) NOT a content-change.
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
    let rt = env.get("roundtrip").unwrap();
    assert_eq!(
        rt.get("status").and_then(|v| v.as_str()),
        Some("ok"),
        "well-formed Sparrow blob must canonicalize cleanly; rt: {rt:?}"
    );
    assert_eq!(
        rt.get("semantic_match").and_then(|v| v.as_bool()),
        Some(true),
        "semantic_match must be true (key-reorder is semantic-identical); rt: {rt:?}"
    );
    // byte_exact is false because the Sparrow-native field order is not
    // alphabetical — the canonicalize re-emits with sorted keys.
    assert_eq!(
        rt.get("byte_exact").and_then(|v| v.as_bool()),
        Some(false),
        "Sparrow-native field order is not alphabetical → canonicalize reorders → byte_exact=false; rt: {rt:?}"
    );
}

#[test]
fn sparrow_json_envelope_dropped_fields_surface_in_metadata() {
    // Synthesize a blob with extra top-level fields; assert they appear in
    // sparrow_source_metadata.dropped_fields.
    let blob = r#"{
        "name":"x","network":"mainnet","policyType":"SINGLE","scriptType":"P2WPKH",
        "defaultPolicy":{"name":"Default","miniscript":{"script":"wpkh(@0/**)"}},
        "keystores":[{
            "label":"x","source":"SW_WATCH","walletModel":"SPARROW",
            "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/84'/0'/0'"},
            "extendedPublicKey":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"
        }],
        "birthDate":1717000000
    }"#;
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "sparrow", "--json"])
        .write_stdin(blob.to_string())
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let env = &val.as_array().unwrap()[0];
    let dropped = env
        .get("sparrow_source_metadata")
        .and_then(|m| m.get("dropped_fields"))
        .and_then(|v| v.as_array())
        .unwrap();
    assert!(
        dropped.iter().any(|v| v.as_str() == Some("birthDate")),
        "birthDate must surface in dropped_fields; got: {dropped:?}"
    );
}

// ============================================================================
// Refusal cells
// ============================================================================

#[test]
fn sparrow_malformed_missing_script_exits_parse_error() {
    let p = fixture_path("sparrow-malformed-missing-script.json");
    let assertion = run_import(&["--blob", p.to_str().unwrap(), "--format", "sparrow"])
        .failure()
        .code(2);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("sparrow") && stderr.contains("defaultPolicy.miniscript.script"),
        "expected sparrow parse-error citing missing nested script; got: {stderr}"
    );
}

#[test]
fn sparrow_taproot_singlesig_refused() {
    // P1B taproot deferral: tr(...) scripts refused at parse-time pending
    // future descriptor-passthrough support (cycle-followup
    // `sparrow-taproot-descriptor-passthrough-import-support`).
    let blob = r#"{
        "name":"bip86-0","network":"mainnet","policyType":"SINGLE","scriptType":"P2TR",
        "defaultPolicy":{"name":"Default","miniscript":{"script":"tr(@0/**)"}},
        "keystores":[{
            "label":"bip86-0","source":"SW_WATCH","walletModel":"SPARROW",
            "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/86'/0'/0'"},
            "extendedPublicKey":"xpub6CAYwo2AfKJy1cdFGBAgLvCrZULhEkZ9C9s4GGXwXzHvNPguMWBcVrGEDjP2ZJdX92gVWLeLrNVVmipTrKqrwMy2eT282xKEyHMbPDrcD9e"
        }]
    }"#;
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "sparrow"])
        .write_stdin(blob.to_string())
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("taproot") && stderr.contains("not yet supported"),
        "expected taproot-deferred refusal; got: {stderr}"
    );
}

// ============================================================================
// Roundtrip canonicalize cell
// ============================================================================

#[test]
fn sparrow_canonicalize_drops_extra_top_level_fields() {
    // Send a non-canonical blob (extra fields + non-alphabetical order);
    // expect the parse to succeed + roundtrip envelope to report
    // byte_exact=false because the canonical form differs from the input.
    let blob = r#"{
        "policyType":"SINGLE",
        "scriptType":"P2WPKH",
        "name":"reordered",
        "network":"mainnet",
        "defaultPolicy":{"name":"Default","miniscript":{"script":"wpkh(@0/**)"}},
        "keystores":[{
            "label":"x","source":"SW_WATCH","walletModel":"SPARROW",
            "keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/84'/0'/0'"},
            "extendedPublicKey":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"
        }],
        "gapLimit":20
    }"#;
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "sparrow", "--json"])
        .write_stdin(blob.to_string())
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let env = &val.as_array().unwrap()[0];
    let rt = env.get("roundtrip").unwrap();
    // Canonicalize drops `gapLimit` + alphabetizes keys → byte_exact must
    // be false.
    assert_eq!(
        rt.get("byte_exact").and_then(|v| v.as_bool()),
        Some(false),
        "canonicalize drops extra fields → byte_exact must be false; rt: {rt:?}"
    );
    assert_eq!(
        rt.get("status").and_then(|v| v.as_str()),
        Some("ok"),
        "canonicalize still succeeded (status=ok); rt: {rt:?}"
    );
}
