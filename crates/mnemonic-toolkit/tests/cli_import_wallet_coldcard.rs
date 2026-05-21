//! v0.28.0 Phase P3C — `mnemonic import-wallet --format coldcard` integration cells.
//!
//! Per plan-doc P3C row + SPEC §11.3. Covers:
//!
//! - parse happy-path per BIP fixture (bip44 / bip49 / bip84 / bip86 + XTN testnet)
//! - sniff-positive: blob without `--format` routes to Coldcard
//! - format-mismatch: explicit `--format coldcard` against a non-Coldcard
//!   blob (BSMS / Bitcoin Core / Sparrow) → exit 1
//! - `--json` envelope: `coldcard_source_metadata` field surfaces only on
//!   Coldcard parses + carries SPEC §11.3 fields
//! - `--json` envelope: roundtrip status semantics
//! - refusal: malformed blob exits 2 `ImportWalletParse`
//! - `--select-descriptor` coerce: non-`all` value emits NOTICE + coerces to `all`
//!
//! Cells consume the fixtures at `tests/fixtures/wallet_import/coldcard-*.json`
//! created during Phase P3B.

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
// Parse happy-path cells (one per BIP fixture)
// ============================================================================

#[test]
fn coldcard_singlesig_bip84_mainnet_parses_clean() {
    let p = fixture_path("coldcard-singlesig-bip84-mainnet.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "coldcard"]).success();
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
fn coldcard_singlesig_bip49_mainnet_parses_clean() {
    let p = fixture_path("coldcard-singlesig-bip49-mainnet.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "coldcard"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
}

#[test]
fn coldcard_singlesig_bip44_mainnet_parses_clean() {
    let p = fixture_path("coldcard-singlesig-bip44-mainnet.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "coldcard"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
}

#[test]
fn coldcard_singlesig_bip86_taproot_parses_clean() {
    let p = fixture_path("coldcard-singlesig-bip86-mainnet.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "coldcard"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
}

#[test]
fn coldcard_singlesig_bip84_xtn_testnet_parses_clean() {
    let p = fixture_path("coldcard-singlesig-bip84-xtn-testnet.json");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "coldcard"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    assert!(
        stdout.contains("network=testnet"),
        "expected testnet network; stdout: {stdout}"
    );
}

// ============================================================================
// Sniff cells (auto-format detection)
// ============================================================================

#[test]
fn coldcard_sniff_detects_bip84_without_format() {
    let p = fixture_path("coldcard-singlesig-bip84-mainnet.json");
    // No --format: rely on sniff dispatch (Q3-lock: chain + xfp + ≥1 derivation marker).
    let out = run_import(&["--blob", p.to_str().unwrap()]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cosigners=1"),
        "sniff must route to ColdcardParser; stdout: {stdout}"
    );
}

#[test]
fn coldcard_sniff_detects_bip86_taproot_without_format() {
    let p = fixture_path("coldcard-singlesig-bip86-mainnet.json");
    let out = run_import(&["--blob", p.to_str().unwrap()]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
}

// ============================================================================
// Format-mismatch cells (SPEC §6.1)
// ============================================================================

#[test]
fn coldcard_with_format_mismatch_sniffed_bsms_exits_one() {
    // Explicit `--format coldcard` on a BSMS blob: sniff sees BSMS
    // → `ImportWalletFormatMismatch` (supplied=coldcard, sniffed=bsms).
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let assertion = run_import(&["--blob", p.to_str().unwrap(), "--format", "coldcard"]).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard") && stderr.contains("bsms"),
        "expected coldcard-vs-bsms format-mismatch; got: {stderr}"
    );
}

#[test]
fn coldcard_with_format_mismatch_sniffed_bitcoin_core_exits_one() {
    let core_blob =
        r#"{"wallet_name":"a","descriptors":[{"desc":"wpkh([5436d724/84'/0'/0']xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9/<0;1>/*)#00lx6ere"}]}"#;
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "coldcard"])
        .write_stdin(core_blob.to_string())
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard") && (stderr.contains("bitcoin-core") || stderr.contains("format")),
        "expected coldcard-vs-bitcoin-core format-mismatch; got: {stderr}"
    );
}

#[test]
fn coldcard_with_format_mismatch_sniffed_sparrow_exits_one() {
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
        .args(["import-wallet", "--blob", "-", "--format", "coldcard"])
        .write_stdin(sparrow_blob.to_string())
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard") && stderr.contains("sparrow"),
        "expected coldcard-vs-sparrow format-mismatch; got: {stderr}"
    );
}

// ============================================================================
// `--json` envelope cells (SPEC §7.4 + SPEC §11.3 coldcard_source_metadata)
// ============================================================================

#[test]
fn coldcard_json_envelope_includes_source_metadata_and_roundtrip() {
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
    let val: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("invalid JSON: {e}\nstdout was:\n{stdout}"));
    let arr = val.as_array().expect("--json must emit array");
    assert_eq!(arr.len(), 1, "expected one envelope");
    let env = &arr[0];

    assert_eq!(
        env.get("source_format").and_then(|v| v.as_str()),
        Some("coldcard"),
        "envelope source_format must be coldcard"
    );

    let meta = env
        .get("coldcard_source_metadata")
        .expect("coldcard_source_metadata field must be present on Coldcard envelopes");
    assert_eq!(
        meta.get("chain").and_then(|v| v.as_str()),
        Some("BTC"),
        "chain must echo blob value"
    );
    assert_eq!(
        meta.get("xfp").and_then(|v| v.as_str()),
        Some("B8688DF1"),
        "xfp echoed uppercase 8-hex"
    );
    assert_eq!(
        meta.get("bip_derivation").and_then(|v| v.as_str()),
        Some("bip84"),
        "bip_derivation reflects dominant-BIP selection"
    );
    assert_eq!(
        meta.get("raw_account").and_then(|v| v.as_u64()),
        Some(0),
        "raw_account verbatim from blob"
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
fn coldcard_json_envelope_no_coldcard_source_metadata_on_bsms() {
    // Cross-check: BSMS envelopes do NOT carry coldcard_source_metadata.
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out =
        run_import(&["--blob", p.to_str().unwrap(), "--format", "bsms", "--json"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let env = &val.as_array().unwrap()[0];
    assert!(
        env.get("coldcard_source_metadata").is_none(),
        "BSMS envelope must NOT carry coldcard_source_metadata; env: {env:?}"
    );
}

#[test]
fn coldcard_json_envelope_no_coldcard_source_metadata_on_sparrow() {
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
        env.get("coldcard_source_metadata").is_none(),
        "Sparrow envelope must NOT carry coldcard_source_metadata; env: {env:?}"
    );
}

#[test]
fn coldcard_json_envelope_bip86_taproot_carries_bip_derivation_tag() {
    let p = fixture_path("coldcard-singlesig-bip86-mainnet.json");
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
    let meta = env.get("coldcard_source_metadata").unwrap();
    assert_eq!(
        meta.get("bip_derivation").and_then(|v| v.as_str()),
        Some("bip86")
    );
}

#[test]
fn coldcard_json_envelope_xtn_carries_chain_xtn() {
    let p = fixture_path("coldcard-singlesig-bip84-xtn-testnet.json");
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
    let meta = env.get("coldcard_source_metadata").unwrap();
    assert_eq!(meta.get("chain").and_then(|v| v.as_str()), Some("XTN"));
}

// ============================================================================
// Refusal cells
// ============================================================================

#[test]
fn coldcard_malformed_json_exits_parse_error() {
    let blob = r#"{not json"#;
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "coldcard"])
        .write_stdin(blob.to_string())
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard") && stderr.contains("invalid JSON"),
        "expected coldcard parse-error citing invalid JSON; got: {stderr}"
    );
}

#[test]
fn coldcard_missing_chain_exits_parse_error() {
    let blob = r#"{"xfp":"B8688DF1","bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX","first":"bc1q..."}}"#;
    // Without chain, sniff also rejects → ImportWalletAmbiguousFormat.
    // With explicit --format coldcard, the parser reaches missing-chain.
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "coldcard"])
        .write_stdin(blob.to_string())
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard") && stderr.contains("chain"),
        "expected coldcard missing-chain error; got: {stderr}"
    );
}

#[test]
fn coldcard_unrecognized_chain_exits_parse_error() {
    // chain="main" (Bitcoin Core's value) explicit-format coldcard.
    let blob = r#"{"chain":"main","xfp":"B8688DF1","bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX","first":"bc1q..."}}"#;
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "coldcard"])
        .write_stdin(blob.to_string())
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard") && stderr.contains("chain"),
        "expected coldcard chain-not-BTC-or-XTN error; got: {stderr}"
    );
}

// ============================================================================
// `--select-descriptor` coerce cell (SPEC §5.3 + P3C coerce)
// ============================================================================

#[test]
fn coldcard_select_descriptor_non_all_emits_notice_and_coerces() {
    let p = fixture_path("coldcard-singlesig-bip84-mainnet.json");
    let assertion = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "coldcard",
        "--select-descriptor",
        "active-receive",
    ])
    .success();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("notice: import-wallet: coldcard:")
            && stderr.contains("--select-descriptor")
            && stderr.contains("has no effect"),
        "expected coldcard coerce NOTICE; got: {stderr}"
    );
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cosigners=1"),
        "expected parse to succeed post-coerce; got: {stdout}"
    );
}

// ============================================================================
// v0.28.6 — Legacy mk1/mk2 Coldcard wallet.json fallback (parser at
// wallet_import/coldcard.rs:460-462 with SLIP-132 prefix inference at
// :471-494). Parser implementation landed in commit 1304932 (v0.28.0
// P3-v2 cycle); this cycle adds fixture + test coverage per FOLLOWUP
// `coldcard-legacy-mk1-mk2-top-level-xpub-inference`.
// ============================================================================

#[test]
fn coldcard_legacy_mk1_xpub_prefix_infers_bip44() {
    let fixture = PathBuf::from("tests/fixtures/wallet_import")
        .join("coldcard-mk1-legacy-bip44-mainnet.json");
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--format",
            "coldcard",
            "--blob",
            fixture.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("envelope JSON");
    let descriptor = envelope[0]["bundle"]["descriptor"]
        .as_str()
        .expect("bundle.descriptor present in envelope");
    assert!(
        descriptor.starts_with("pkh("),
        "expected pkh() descriptor (BIP-44 from xpub prefix), got: {descriptor:?}"
    );
}

#[test]
fn coldcard_legacy_mk1_ypub_prefix_infers_bip49() {
    let fixture = PathBuf::from("tests/fixtures/wallet_import")
        .join("coldcard-mk1-legacy-bip49-mainnet.json");
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--format",
            "coldcard",
            "--blob",
            fixture.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("envelope JSON");
    let descriptor = envelope[0]["bundle"]["descriptor"]
        .as_str()
        .expect("bundle.descriptor present in envelope");
    assert!(
        descriptor.starts_with("sh(wpkh("),
        "expected sh(wpkh() descriptor (BIP-49 from ypub prefix), got: {descriptor:?}"
    );
}

#[test]
fn coldcard_legacy_mk1_zpub_prefix_infers_bip84() {
    let fixture = PathBuf::from("tests/fixtures/wallet_import")
        .join("coldcard-mk1-legacy-bip84-mainnet.json");
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--format",
            "coldcard",
            "--blob",
            fixture.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(assertion.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value = serde_json::from_str(&stdout).expect("envelope JSON");
    let descriptor = envelope[0]["bundle"]["descriptor"]
        .as_str()
        .expect("bundle.descriptor present in envelope");
    assert!(
        descriptor.starts_with("wpkh("),
        "expected wpkh() descriptor (BIP-84 from zpub prefix), got: {descriptor:?}"
    );
}

#[test]
fn coldcard_legacy_mk1_unrecognized_prefix_refuses() {
    use std::io::Write;
    let tmpdir = tempfile::tempdir().expect("tempdir");
    let path = tmpdir.path().join("coldcard-legacy-bad-prefix.json");
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(
        br#"{"xpub": "bogusprefix_not_a_slip132_xpub_at_all", "xfp": "5436D724", "chain": "BTC"}"#,
    )
    .unwrap();
    drop(f);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--format",
            "coldcard",
            "--blob",
            path.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("mnemonic spawn");
    assert_ne!(out.status.code(), Some(0), "must refuse, got success");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unrecognized SLIP-132 prefix"),
        "expected SLIP-132 prefix refusal, got: {stderr}"
    );
}
