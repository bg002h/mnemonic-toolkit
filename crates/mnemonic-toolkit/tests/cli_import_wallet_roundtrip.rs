//! Phase 4 — round-trip discipline integration tests per SPEC §7.
//!
//! Per `design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md` §4.5-§4.9. Drives
//! the `mnemonic import-wallet` + `mnemonic export-wallet` CLIs against the
//! vendored fixture corpus to validate:
//!
//! 1. **Fixture-parse smoke (BSMS + Core):** each static fixture parses
//!    via `mnemonic import-wallet --format <X> --blob <file>` and produces
//!    the expected cosigner-count + network + threshold summary.
//!
//! 2. **Cross-CLI bundle round-trip (Core only):** generate a Bitcoin Core
//!    `importdescriptors` JSON blob via `mnemonic export-wallet --format
//!    bitcoin-core`; pipe it back through `mnemonic import-wallet --format
//!    bitcoin-core`; assert the import-summary matches the original
//!    export-side inputs (cosigner count + fingerprint + network).
//!
//! 3. **Helper-side semantic round-trip (BSMS + Core):** the
//!    `canonicalize_bsms` + `canonicalize_bitcoin_core` + `unified_diff`
//!    helpers are exercised AS UNIT TESTS in `src/wallet_import/roundtrip.rs`'s
//!    `mod tests` (18 cells); the helpers themselves cannot be called from
//!    integration tests because `wallet_import` is binary-private. See the
//!    Phase 4 status report for the rationale.
//!
//! ## BSMS bundle round-trip — structurally blocked at Phase 4
//!
//! There is no `mnemonic export-wallet --format bsms` emitter in v0.25.x.
//! Bundle round-trip per plan §4.5 (`bundle → export-wallet --format bsms →
//! import-wallet --format bsms → assert match`) cannot be exercised this
//! cycle. The semantic-blob round-trip path (plan §4.7) likewise requires
//! the export-side helper. Both are tracked as future work in FOLLOWUP
//! `wallet-export-bsms-emitter` (to be filed at cycle close).
//!
//! ## `--json` envelope cell — deferred to Phase 5
//!
//! Plan §4.9's `--json` envelope `roundtrip` field cell is deferred to
//! Phase 5 per the kickoff brief: `cmd/import_wallet.rs` does not yet expose
//! `--json`. The helper-direct test
//! `roundtrip::tests::unified_diff_byte_exact_branch_short_circuits` pins
//! the underlying short-circuit behavior in lieu of an envelope-level cell.

use assert_cmd::Command;
use std::path::PathBuf;

// ---- Fixtures path helper ----

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

fn run_import_file(path: &PathBuf, format: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob"])
        .arg(path)
        .args(["--format", format])
        .assert()
}

fn run_import_stdin(blob: &str, format: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", format])
        .write_stdin(blob.to_string())
        .assert()
}

// ============================================================================
// BSMS fixture-parse cells — validate the static fixtures vendored in
// Phase 4 are well-formed (checksum + descriptor body).
// ============================================================================

#[test]
fn fixture_bsms_2line_sortedmulti_2of2_parses_clean() {
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out = run_import_file(&p, "bsms").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=2"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=2"), "stdout: {stdout}");
    assert!(stdout.contains("b8688df1"));
    assert!(stdout.contains("28645006"));
}

#[test]
fn fixture_bsms_2line_sortedmulti_2of3_parses_clean() {
    let p = fixture_path("bsms-2line-sortedmulti-2of3.txt");
    let out = run_import_file(&p, "bsms").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=3"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"));
    assert!(stdout.contains("threshold=2"));
    // 3 cosigner fingerprints all reported.
    assert!(stdout.contains("b8688df1"));
    assert!(stdout.contains("28645006"));
    assert!(stdout.contains("5436d724"));
}

#[test]
fn fixture_bsms_2line_multi_2of2_parses_clean() {
    // Bare `multi(...)` (declaration-order preserved). The toolkit
    // accepts `sh(multi(...))`; bare `multi` inside `wsh` is forbidden by
    // miniscript's malleability profile.
    let p = fixture_path("bsms-2line-multi-2of2.txt");
    let out = run_import_file(&p, "bsms").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=2"));
    assert!(stdout.contains("threshold=2"));
}

#[test]
fn fixture_bsms_2line_decay_144_parses_clean() {
    // Decaying-multisig N=144 (1-day fallback). Testnet (704c7836).
    let p = fixture_path("bsms-2line-decay-144.txt");
    let out = run_import_file(&p, "bsms").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=2"), "stdout: {stdout}");
    assert!(stdout.contains("network=testnet"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=2"));
}

#[test]
fn fixture_bsms_1of1_singlesig_parses_clean() {
    // BIP-84 single-sig mainnet (`wpkh(...)`).
    let p = fixture_path("bsms-1of1-singlesig.txt");
    let out = run_import_file(&p, "bsms").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"));
    // single-sig → no thresh()/multi() → threshold=none.
    assert!(stdout.contains("threshold=none"), "stdout: {stdout}");
}

#[test]
fn fixture_bsms_shwsh_2of3_parses_clean() {
    // Legacy nested-segwit `sh(wsh(sortedmulti(...)))` shape.
    let p = fixture_path("bsms-shwsh-2of3.txt");
    let out = run_import_file(&p, "bsms").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=3"));
    assert!(stdout.contains("threshold=2"));
}

#[test]
fn fixture_bsms_testnet_2of2_parses_clean() {
    // tpub-based 2-of-2 with testnet coin-type (1').
    let p = fixture_path("bsms-testnet-2of2.txt");
    let out = run_import_file(&p, "bsms").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=2"));
    assert!(stdout.contains("network=testnet"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=2"));
}

#[test]
fn fixture_bsms_extra_trailing_newlines_still_parses() {
    // Vendored fixture + 3 extra trailing newlines (a common
    // wild-blob artifact). Parser must tolerate.
    let blob = std::fs::read_to_string(fixture_path("bsms-2line-sortedmulti-2of2.txt")).unwrap();
    let blob = format!("{blob}\n\n\n");
    let out = run_import_stdin(&blob, "bsms").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=2"));
}

#[test]
fn fixture_bsms_crlf_normalize_via_cli() {
    // BSMS fixture transformed to CRLF line endings. SPEC §7.3.1
    // step 1: CRLF → LF normalize.
    let blob = std::fs::read_to_string(fixture_path("bsms-2line-sortedmulti-2of2.txt")).unwrap();
    let blob_crlf = blob.replace('\n', "\r\n");
    let out = run_import_stdin(&blob_crlf, "bsms").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=2"));
    assert!(stdout.contains("network=mainnet"));
}

// ============================================================================
// Bitcoin Core fixture-parse cells.
// ============================================================================

#[test]
fn fixture_core_bip84_mainnet_parses_clean() {
    let p = fixture_path("core-bip84-mainnet.json");
    let out = run_import_file(&p, "bitcoin-core").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stdout.contains("bundles=1"));
    assert!(stdout.contains("cosigners=1"));
    assert!(stdout.contains("network=mainnet"));
    // single-sig → threshold=none.
    assert!(stdout.contains("threshold=none"), "stdout: {stdout}");
    // `timestamp` field is dropped → NOTICE on stderr.
    assert!(
        stderr.contains("dropped wallet-state fields"),
        "expected dropped-fields NOTICE; stderr was: {stderr}"
    );
    assert!(stderr.contains("timestamp"));
}

#[test]
fn fixture_core_bip49_mainnet_two_entries_emit_two_bundles() {
    // BIP-49 fixture has 2 entries (receive + change).
    let p = fixture_path("core-bip49-mainnet.json");
    let out = run_import_file(&p, "bitcoin-core").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=2"), "stdout: {stdout}");
}

#[test]
fn fixture_core_multisig_2of3_parses_clean() {
    let p = fixture_path("core-multisig-2of3.json");
    let out = run_import_file(&p, "bitcoin-core").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"));
    assert!(stdout.contains("cosigners=3"));
    assert!(stdout.contains("threshold=2"));
    assert!(stdout.contains("b8688df1"));
    assert!(stdout.contains("28645006"));
    assert!(stdout.contains("5436d724"));
}

#[test]
fn fixture_core_testnet_bip84_parses_clean() {
    let p = fixture_path("core-testnet-bip84.json");
    let out = run_import_file(&p, "bitcoin-core").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("network=testnet"), "stdout: {stdout}");
}

#[test]
fn fixture_core_multi_bip84_emit_four_bundles() {
    // Pre-existing Phase 3 fixture: 4 entries (BIP-84 + BIP-49 receive
    // + change pairs).
    let p = fixture_path("core-multi-bip84.json");
    let out = run_import_file(&p, "bitcoin-core").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=4"), "stdout: {stdout}");
}

// ============================================================================
// Cross-CLI Bitcoin Core bundle round-trip:
//
//   export-wallet --format bitcoin-core (synthesize a blob from concrete-
//   keys input) → import-wallet --format bitcoin-core (round-trip back).
//
// Plan §4.6: bundle round-trip pattern. Asserts that data produced by the
// toolkit's own export emitter is re-consumable by the toolkit's import
// parser without loss of essential properties.
// ============================================================================

// Trezor 12-word "abandon ... about" BIP-84 mainnet account 0 xpub
// (re-used from `cli_export_wallet.rs`).
const TREZOR_BIP84_XPUB: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";
const TREZOR_BIP84_FP: &str = "73c5da0a";

const COSIGNER_A_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const COSIGNER_A_FP: &str = "b8688df1";
const COSIGNER_B_XPUB: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const COSIGNER_B_FP: &str = "28645006";

/// Run `mnemonic export-wallet --format bitcoin-core ...` and return the
/// stdout JSON. Re-use the proven invocation pattern from
/// `cli_export_wallet.rs::cell_1_bitcoin_core_single_sig_wpkh_round_trip`.
fn export_core_bip84_single_sig() -> String {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_BIP84_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_BIP84_FP}"),
        ])
        .assert()
        .success();
    String::from_utf8(out.get_output().stdout.clone()).unwrap()
}

fn export_core_wsh_sortedmulti_2of2() -> String {
    // wsh-sortedmulti 2-of-2 at BIP-48 mainnet `m/48'/0'/0'/2'`.
    // Export emitter renders the canonical descriptor inside a Core-
    // shaped JSON array.
    //
    // `--format bitcoin-core` works with single-sig wpkh; for multisig
    // we use the `wsh-sortedmulti` template which feeds the same emitter.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "bitcoin-core",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={COSIGNER_A_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={COSIGNER_A_FP}"),
            "--slot",
            "@0.path=m/48'/0'/0'/2'",
            "--slot",
            &format!("@1.xpub={COSIGNER_B_XPUB}"),
            "--slot",
            &format!("@1.fingerprint={COSIGNER_B_FP}"),
            "--slot",
            "@1.path=m/48'/0'/0'/2'",
        ])
        .assert()
        .success();
    String::from_utf8(out.get_output().stdout.clone()).unwrap()
}

/// Wrap a bare-array export blob (`[ {...}, {...} ]`) into a
/// `listdescriptors`-shaped object envelope: `{ wallet_name: ...,
/// descriptors: [...] }`. The Phase 3 importer requires the object form;
/// the Phase 4 round-trip cells assume the importer's invariant and
/// adapt the export side. (Plan §7.0.b notes the importer-side rule.)
fn wrap_export_in_object_envelope(export_json: &str, wallet_name: &str) -> String {
    // Parse + re-render to handle any pretty-print variance.
    let entries: serde_json::Value = serde_json::from_str(export_json).unwrap();
    let envelope = serde_json::json!({
        "wallet_name": wallet_name,
        "descriptors": entries,
    });
    serde_json::to_string(&envelope).unwrap()
}

#[test]
fn core_bundle_roundtrip_bip84_single_sig() {
    let exported = export_core_bip84_single_sig();
    let wrapped = wrap_export_in_object_envelope(&exported, "test_bip84");
    let out = run_import_stdin(&wrapped, "bitcoin-core").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // BIP-84 exports as 2 entries (receive + change after multipath split).
    assert!(stdout.contains("bundles=2"), "stdout: {stdout}");
    // Master fingerprint propagates through both bundles.
    assert!(stdout.contains(TREZOR_BIP84_FP));
    assert!(stdout.contains("network=mainnet"));
}

#[test]
fn core_bundle_roundtrip_wsh_sortedmulti_2of2() {
    let exported = export_core_wsh_sortedmulti_2of2();
    let wrapped = wrap_export_in_object_envelope(&exported, "test_2of2");
    let out = run_import_stdin(&wrapped, "bitcoin-core").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // wsh-sortedmulti emits as 2 multipath entries (receive + change).
    assert!(stdout.contains("bundles=2"), "stdout: {stdout}");
    // Cosigner fingerprints survive the round-trip.
    assert!(stdout.contains(COSIGNER_A_FP), "stdout: {stdout}");
    assert!(stdout.contains(COSIGNER_B_FP), "stdout: {stdout}");
    assert!(stdout.contains("threshold=2"));
}

#[test]
fn core_bundle_roundtrip_export_blob_canonicalizes_against_self() {
    // Sanity: the export emitter's output, when wrapped + parsed by the
    // importer, has internally-consistent cosigner counts across both
    // multipath splits. The importer normalizes both branches; the
    // assertion is that each split bundle reports 1 cosigner.
    let exported = export_core_bip84_single_sig();
    let wrapped = wrap_export_in_object_envelope(&exported, "self_check");
    let out = run_import_stdin(&wrapped, "bitcoin-core").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout.matches("cosigners=1").count(), 4,
        "expected 2 bundle entries × (top-level alias + nested) = 4 `cosigners=1` lines; stdout: {stdout}");
}

#[test]
fn core_bundle_roundtrip_select_active_receive_filters_to_one() {
    // Apply `--select-descriptor active-receive` to a 2-entry exported
    // blob. The receive entry (`internal: false`) is filtered in.
    let exported = export_core_bip84_single_sig();
    let wrapped = wrap_export_in_object_envelope(&exported, "select_test");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bitcoin-core",
            "--select-descriptor",
            "active-receive",
        ])
        .write_stdin(wrapped)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // Only the receive (internal=false) entry survives.
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(stdout.contains("internal=false"), "stdout: {stdout}");
}

#[test]
fn core_bundle_roundtrip_export_blob_keys_present() {
    // Cross-check: the toolkit's BIP-84 export uses Trezor xpub
    // (TREZOR_BIP84_XPUB at m/84'/0'/0'). After round-trip, the
    // import-side summary must report `network=mainnet` (coin-type 0')
    // and the Trezor fingerprint.
    let exported = export_core_bip84_single_sig();
    assert!(exported.contains(TREZOR_BIP84_FP));
    assert!(exported.contains(TREZOR_BIP84_XPUB));
    let wrapped = wrap_export_in_object_envelope(&exported, "key_check");
    let out = run_import_stdin(&wrapped, "bitcoin-core").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains(TREZOR_BIP84_FP), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"));
}
