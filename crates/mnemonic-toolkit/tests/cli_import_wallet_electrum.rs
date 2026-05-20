//! v0.28.0 Phase P6 — `mnemonic import-wallet --format electrum` integration tests.
//!
//! SPEC `design/SPEC_wallet_import_v0_28_0.md` §11.6 (Electrum 4.x wallet-file
//! ingest). Per-cell coverage:
//!
//! - `--format electrum` happy path: singlesig (BIP-84 zpub + BIP-49 ypub) +
//!   multisig (2-of-3 Zpub wsh-sortedmulti).
//! - Auto-sniff path: passing no `--format` against the same fixtures routes
//!   through `sniff_format` → `SniffOutcome::Electrum` (P6A wired).
//! - SPEC §11.6.1 refusal templates: 2fa / imported / encrypted blobs emit
//!   the byte-exact stderr text + exit 2.
//! - `--json` envelope: source_metadata block carries
//!   `{seed_version, wallet_type, wallet_name, dropped_fields}`.
//! - Round-trip: `roundtrip.status: "ok"` + diff non-null (key reordering by
//!   alphabetical BTreeMap re-emit; semantic_match=true regardless).
//! - Format-mismatch: `--format electrum` against a BSMS blob errors with
//!   ImportWalletFormatMismatch.

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
// Happy paths — explicit --format electrum
// ============================================================================

#[test]
fn electrum_standard_bip84_explicit_format_succeeds() {
    let p = fixture_path("electrum-standard-bip84-mainnet.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "electrum",
    ])
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("import-wallet: bundles=1"));
    assert!(stdout.contains("bundles[0].cosigners=1"));
    assert!(stdout.contains("bundles[0].network=mainnet"));
    assert!(stdout.contains("bundles[0].threshold=none"));
    assert!(stdout.contains("5436d724"), "BIP-84 fixture fingerprint");
}

#[test]
fn electrum_standard_bip49_explicit_format_succeeds() {
    let p = fixture_path("electrum-standard-bip49-mainnet.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "electrum",
    ])
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("import-wallet: bundles=1"));
    assert!(stdout.contains("deadbeef"), "BIP-49 fixture fingerprint");
}

#[test]
fn electrum_multisig_2of3_wsh_explicit_format_succeeds() {
    let p = fixture_path("electrum-multisig-2of3-wsh.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "electrum",
    ])
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("import-wallet: bundles=1"));
    assert!(stdout.contains("bundles[0].cosigners=3"));
    assert!(stdout.contains("bundles[0].threshold=2"));
    // All three cosigner fingerprints appear in the summary.
    assert!(stdout.contains("b8688df1"));
    assert!(stdout.contains("28645006"));
    assert!(stdout.contains("5436d724"));
}

// ============================================================================
// Auto-sniff path — no --format flag
// ============================================================================

#[test]
fn electrum_standard_bip84_auto_sniff_succeeds() {
    let p = fixture_path("electrum-standard-bip84-mainnet.json");
    let out = run_import(&["--blob", p.to_str().unwrap()]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("import-wallet: bundles=1"));
}

#[test]
fn electrum_multisig_auto_sniff_succeeds() {
    let p = fixture_path("electrum-multisig-2of3-wsh.json");
    let out = run_import(&["--blob", p.to_str().unwrap()]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles[0].cosigners=3"));
}

// ============================================================================
// Refusal cells — SPEC §11.6.1 byte-exact stderr templates + exit 2
// ============================================================================

#[test]
fn electrum_2fa_refuses_with_byte_exact_stderr_and_exit_2() {
    let p = fixture_path("electrum-2fa-refused.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "electrum",
    ])
    .failure();
    assert_eq!(out.get_output().status.code(), Some(2), "exit 2 per ImportWalletParse");
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains(
            "import-wallet: electrum: 2fa wallets require TrustedCoin two-factor restoration; \
             ingest not supported"
        ),
        "expected SPEC §11.6.1 2fa byte-exact template; got: {stderr}"
    );
}

#[test]
fn electrum_imported_refuses_with_byte_exact_stderr_and_exit_2() {
    let p = fixture_path("electrum-imported-refused.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "electrum",
    ])
    .failure();
    assert_eq!(out.get_output().status.code(), Some(2));
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains(
            "import-wallet: electrum: imported-addresses wallets have no derivation chain to \
             reconstruct; ingest not supported"
        ),
        "expected SPEC §11.6.1 imported byte-exact template; got: {stderr}"
    );
}

#[test]
fn electrum_encrypted_refuses_with_byte_exact_stderr_and_followup_slug() {
    let p = fixture_path("electrum-encrypted-refused.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "electrum",
    ])
    .failure();
    assert_eq!(out.get_output().status.code(), Some(2));
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains(
            "import-wallet: electrum: encrypted wallet files require decrypting via \
             'electrum --decrypt-wallet' first; encrypted ingest not yet supported"
        ),
        "expected SPEC §11.6.1 encrypted byte-exact template; got: {stderr}"
    );
    assert!(
        stderr.contains("wallet-import-electrum-encrypted"),
        "FOLLOWUP slug must be referenced in stderr template; got: {stderr}"
    );
}

#[test]
fn electrum_2fa_via_auto_sniff_still_routes_to_refusal_arm() {
    // SPEC §11.6: sniff is positive for 2fa wallets (routing-decision, not
    // admission-decision). Auto-sniff must reach the parse arm which then
    // surfaces the §11.6.1 template.
    let p = fixture_path("electrum-2fa-refused.json");
    let out = run_import(&["--blob", p.to_str().unwrap()]).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("2fa wallets require TrustedCoin"),
        "auto-sniff must route 2fa to §11.6.1 template, not 'could not detect format'; \
         got: {stderr}"
    );
}

// ============================================================================
// --json envelope: source_metadata + roundtrip surface
// ============================================================================

#[test]
fn electrum_json_envelope_carries_source_metadata() {
    let p = fixture_path("electrum-standard-bip84-mainnet.json");
    let out = run_import(&[
        "--json",
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "electrum",
    ])
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON envelope");
    let arr = v.as_array().expect("envelope is array");
    assert_eq!(arr.len(), 1);
    let env = &arr[0];

    // source_format
    assert_eq!(
        env.get("source_format").and_then(|v| v.as_str()),
        Some("electrum")
    );

    // source_metadata block
    let sm = env
        .get("source_metadata")
        .expect("source_metadata present");
    assert_eq!(sm.get("seed_version").and_then(|v| v.as_u64()), Some(17));
    assert_eq!(
        sm.get("wallet_type").and_then(|v| v.as_str()),
        Some("standard")
    );
    assert!(sm.get("wallet_name").unwrap().is_null());
    assert!(
        sm.get("dropped_fields").unwrap().as_array().unwrap().is_empty(),
        "BIP-84 fixture has no runtime-state fields"
    );
}

#[test]
fn electrum_json_envelope_multisig_carries_kofn_wallet_type() {
    let p = fixture_path("electrum-multisig-2of3-wsh.json");
    let out = run_import(&[
        "--json",
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "electrum",
    ])
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let sm = &v[0]["source_metadata"];
    assert_eq!(
        sm.get("wallet_type").and_then(|v| v.as_str()),
        Some("2of3"),
        "multisig wallet_type must render as <k>of<n> per SPEC §11.6"
    );
}

#[test]
fn electrum_json_envelope_roundtrip_status_ok() {
    let p = fixture_path("electrum-standard-bip84-mainnet.json");
    let out = run_import(&[
        "--json",
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "electrum",
    ])
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let rt = &v[0]["roundtrip"];
    assert_eq!(rt.get("status").and_then(|v| v.as_str()), Some("ok"));
    assert_eq!(
        rt.get("semantic_match").and_then(|v| v.as_bool()),
        Some(true),
        "canonicalize_electrum is semantic-preserving"
    );
}

// ============================================================================
// Format-mismatch surface
// ============================================================================

#[test]
fn explicit_format_electrum_against_bsms_blob_yields_format_mismatch() {
    // BSMS blob with --format electrum → ImportWalletFormatMismatch (exit 1
    // per ImportWalletFormatMismatch's Display impl).
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "electrum",
    ])
    .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("electrum") && stderr.contains("bsms"),
        "expected format-mismatch citing both supplied + sniffed; got: {stderr}"
    );
}

#[test]
fn explicit_format_electrum_against_bitcoin_core_blob_yields_format_mismatch() {
    let p = fixture_path("core-bip84-mainnet.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "electrum",
    ])
    .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("electrum") && stderr.contains("bitcoin-core"),
        "expected format-mismatch citing both supplied + sniffed; got: {stderr}"
    );
}
