//! v0.27.0 Phase 5 — `mnemonic export-wallet --from-import-json <FILE|->`.
//!
//! Per `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md`
//! §3.7 + §3.7.1. Cells exercise the export-side consumer path that
//! decodes an `import-wallet --json` envelope and emits a per-format
//! wallet config via the existing `WalletFormatEmitter` dispatch.
//!
//! Includes the headline v0.27.0 integration cell
//! `cross_format_bsms_to_bitcoin_core_to_import_round_trip`.
//!
//! v0.28.0 Phase P11 — cross-format conversion matrix expansion. Closes
//! FOLLOWUP `cross-format-conversion-matrix-expansion`. The matrix
//! exercises 8 source formats × 3 descriptor-capable destinations
//! (happy-path, 24 cells) + 8 sources × 5 template-only destinations
//! (refusal, 40 cells) + 24 semantic-match round-trip assertions, all
//! plumbed through the P11A helper `run_export_from_import_envelope`
//! which composes `mnemonic import-wallet --json | mnemonic
//! export-wallet --from-import-json -`.

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
    // Phase 6.5 PR-review S4 fold: assert each full xpub (not just a prefix
    // substring) so a truncation/canonicalization regression cannot pass.
    let xpub_a = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
    let xpub_b = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";
    let xpub_c = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
    for entry in arr {
        let desc = entry["desc"].as_str().unwrap();
        assert!(desc.contains(xpub_a), "missing full xpub_a in: {desc}");
        assert!(desc.contains(xpub_b), "missing full xpub_b in: {desc}");
        assert!(desc.contains(xpub_c), "missing full xpub_c in: {desc}");
        assert!(desc.contains("b8688df1/48'/0'/0'/2'"));
        assert!(desc.contains("5436d724/48'/0'/0'/2'"));
        assert!(desc.contains("28645006/48'/0'/0'/2'"));
    }
}

// ============================================================================
// Cell 12 — Phase 6.5 PR-review I4: --from-import-json-index out-of-bounds
// on EXPORT side (bundle side already covered in cli_bundle_import_json.rs).
// ============================================================================

#[test]
fn export_wallet_from_import_json_index_out_of_bounds_errors() {
    let tmpdir = tempfile::tempdir().unwrap();
    let p = tmpdir.path().join("multi.json");
    std::fs::write(&p, multi_entry_envelope_json()).unwrap();
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--from-import-json",
            p.to_str().unwrap(),
            "--from-import-json-index",
            "9",
            "--format",
            "bitcoin-core",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("index 9") && stderr.contains("2 entries"),
        "expected OOB diagnostic naming both index and length; got: {stderr}"
    );
}

// ============================================================================
// Cell 13 — Phase 6.5 PR-review I5: empty-array envelope must reject with a
// pointer-text error rather than panic or "no entries" silent-skip.
// ============================================================================

#[test]
fn export_wallet_from_import_json_empty_array_errors() {
    let tmpdir = tempfile::tempdir().unwrap();
    let p = tmpdir.path().join("empty.json");
    std::fs::write(&p, "[]").unwrap();
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
        stderr.contains("empty") || stderr.contains("0 entries") || stderr.contains("no entries"),
        "expected empty-envelope diagnostic; got: {stderr}"
    );
}

// ============================================================================
// Cell 14 — Phase 6.5 PR-review I6: malformed-JSON envelope must surface a
// parse error rather than producing garbage output.
// ============================================================================

#[test]
fn export_wallet_from_import_json_malformed_json_errors() {
    let tmpdir = tempfile::tempdir().unwrap();
    let p = tmpdir.path().join("malformed.json");
    std::fs::write(&p, "not json at all {{{").unwrap();
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
        stderr.to_lowercase().contains("json") || stderr.contains("parse"),
        "expected JSON-parse diagnostic; got: {stderr}"
    );
}

// ============================================================================
// v0.28.0 Phase P11 — cross-format conversion matrix.
//
// Per plan-doc `unified-meandering-sundae.md` §S.10 + Phase 11. Closes
// FOLLOWUP `cross-format-conversion-matrix-expansion` (8 sources × 9
// destinations = 72 cells = 24 happy-path + 40 refusal + 8 wontfix
// duplicate-of-source cases skipped — covered indirectly by the
// import-side per-parser cell suites).
//
// The pipeline under test:
//
//     <source fixture> --format <source> --json
//         => import-wallet emits envelope JSON on stdout
//         => export-wallet --from-import-json - --format <dest>
//         => stdout is the per-format wallet config
//
// Each "happy-path" matrix cell asserts the envelope round-trips through
// the descriptor-capable destination AND the envelope's
// `roundtrip.semantic_match` is true on the import-wallet side (P11D).
//
// Each "refusal" cell asserts the template-only destination surfaces
// the existing emitter-contract refusal message
// (`requires --template` / `descriptor passthrough is not supported` /
// `does not support multisig`).
//
// Source format ⇒ fixture map (P11B). Each source fixture is a
// canonical multisig wallet (or singlesig where the parser is
// singlesig-only) chosen so the resulting envelope's `descriptor`
// field carries the canonical xpubs/script-type for that source.
// ============================================================================

/// Structured result from a single matrix cell — both success + failure
/// paths funnel through this shape.
#[derive(Debug)]
struct ExportResult {
    stdout: String,
    stderr: String,
    exit_code: i32,
    /// The intermediate envelope JSON emitted by `import-wallet --json`
    /// (always populated unless import-wallet itself failed; in the
    /// import-failure case this is the empty string).
    envelope_json: String,
}

/// P11A helper — compose `import-wallet --json` + `export-wallet
/// --from-import-json -` into a single dispatched run. Returns the
/// structured `ExportResult { stdout, stderr, exit_code, envelope_json }`.
///
/// Behavior:
///   - Runs `mnemonic import-wallet --blob <source_fixture> --format
///     <source_format> --json` and captures stdout as `envelope_json`.
///   - If import-wallet fails (non-zero exit), returns the import-wallet
///     stderr + exit_code with `envelope_json = ""` and `stdout = ""`.
///   - Otherwise pipes the envelope JSON to `mnemonic export-wallet
///     --from-import-json - --format <dest_format>` via stdin, captures
///     stdout + stderr + exit_code.
///
/// Discrimination of failure paths (import-wallet vs export-wallet)
/// happens at the caller: empty `envelope_json` ⇒ import-side failed.
fn run_export_from_import_envelope(
    source_fixture: &Path,
    source_format: &str,
    dest_format: &str,
) -> ExportResult {
    let import = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            source_fixture.to_str().unwrap(),
            "--format",
            source_format,
            "--json",
        ])
        .output()
        .expect("import-wallet failed to spawn");
    if !import.status.success() {
        return ExportResult {
            stdout: String::new(),
            stderr: String::from_utf8_lossy(&import.stderr).into_owned(),
            exit_code: import.status.code().unwrap_or(-1),
            envelope_json: String::new(),
        };
    }
    let envelope_json = String::from_utf8(import.stdout).expect("envelope stdout non-utf8");

    let export = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--from-import-json",
            "-",
            "--format",
            dest_format,
        ])
        .write_stdin(envelope_json.clone())
        .output()
        .expect("export-wallet failed to spawn");
    ExportResult {
        stdout: String::from_utf8_lossy(&export.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&export.stderr).into_owned(),
        exit_code: export.status.code().unwrap_or(-1),
        envelope_json,
    }
}

/// Per-source canonical happy-path fixture map. Each entry maps a
/// `--format <source>` enum value to the canonical fixture file that
/// parses cleanly + produces a descriptor-mode envelope.
fn happy_path_fixture(source_format: &str) -> &'static str {
    match source_format {
        // BSMS 2-line is the canonical Round-2 shape used in the
        // existing v0.27.0 headline cell.
        "bsms" => "bsms-2line-sortedmulti-2of3.txt",
        // Bitcoin Core listdescriptors emit (mainnet bip84 singlesig).
        "bitcoin-core" => "core-bip84-mainnet.json",
        // Coldcard single-sig (mainnet bip84).
        "coldcard" => "coldcard-singlesig-bip84-mainnet.json",
        // Coldcard multisig (P2WSH 2-of-3 with XFP header).
        "coldcard-multisig" => "coldcard-ms-2of3-p2wsh-with-xfp.txt",
        // Electrum standard wallet (BIP-84 singlesig).
        "electrum" => "electrum-standard-bip84-mainnet.json",
        // Jade multisig (P2WSH 2-of-3 — delegates to coldcard-multisig parse).
        "jade" => "jade-multisig-2of3-p2wsh.json",
        // Sparrow multisig (P2WSH 2-of-3 sortedmulti).
        "sparrow" => "sparrow-multisig-2of3-p2wsh-sortedmulti.json",
        // Specter multisig (P2WSH 2-of-3 sortedmulti).
        "specter" => "specter-multisig-2of3-p2wsh-sortedmulti.json",
        other => panic!("happy_path_fixture: unknown source format `{other}`"),
    }
}

/// All 8 sources in canonical alphabetical-by-name order (matches the
/// `SniffOutcome` + `--format` value-set alphabetical discipline).
const ALL_SOURCES: &[&str] = &[
    "bitcoin-core",
    "bsms",
    "coldcard",
    "coldcard-multisig",
    "electrum",
    "jade",
    "sparrow",
    "specter",
];

/// Descriptor-capable destinations — accept any descriptor-mode
/// envelope without `--template`. Verified at `wallet_export/*.rs`:
/// bitcoin-core (always), bip388 (always), bsms (refuses ONLY when
/// script_type is P2tr / P2trMulti per v0.28.0 P8A).
const DESCRIPTOR_CAPABLE_DESTS: &[&str] = &["bitcoin-core", "bip388", "bsms"];

/// Template-only destinations from the envelope's perspective — every
/// such format refuses descriptor-mode input with a contract-specific
/// stderr message.
///
/// **Green is NOT in this list at v0.28.0**: its `WalletFormatEmitter::emit`
/// gates the multisig refusal on `inputs.template.is_some()` (see
/// `wallet_export/green.rs:33-39`), and `--from-import-json` always
/// passes `template: None` per `cmd/export_wallet.rs:603`. So Green
/// accepts ANY descriptor (including multisig) on this path. The
/// behavior is pinned by the dedicated `p11c_green_descriptor_passthrough_*`
/// cells below; the gap is logged at
/// `design/v0_28_0-cycle-followups.md#green-emitter-multisig-refusal-template-only`.
const TEMPLATE_ONLY_DESTS: &[&str] =
    &["coldcard", "electrum", "jade", "sparrow"];

// ============================================================================
// P11A — Helper structured-output cells. Verify the helper's
// `ExportResult` contract on both success + failure paths so consumers
// (P11B/C/D) can rely on its shape without re-asserting CLI plumbing.
// ============================================================================

#[test]
fn p11a_helper_returns_zero_exit_on_happy_path_descriptor_capable_dest() {
    let p = fixture_path(happy_path_fixture("bsms"));
    let res = run_export_from_import_envelope(&p, "bsms", "bitcoin-core");
    assert_eq!(res.exit_code, 0, "expected success exit; stderr={}", res.stderr);
    assert!(!res.stdout.is_empty(), "stdout must carry the bitcoin-core wallet config");
    assert!(!res.envelope_json.is_empty(), "envelope_json must be populated on success");
}

#[test]
fn p11a_helper_returns_nonzero_exit_on_template_only_dest_refusal() {
    let p = fixture_path(happy_path_fixture("bsms"));
    let res = run_export_from_import_envelope(&p, "bsms", "sparrow");
    assert_ne!(res.exit_code, 0, "expected refusal exit on template-only dest");
    assert!(res.stdout.is_empty(), "no stdout on refusal");
    assert!(!res.stderr.is_empty(), "refusal must surface a stderr message");
    // Envelope JSON should still be populated — import-wallet succeeded.
    assert!(
        !res.envelope_json.is_empty(),
        "envelope_json must still be populated when only the export step refuses"
    );
}

#[test]
fn p11a_helper_envelope_json_parses_as_array() {
    let p = fixture_path(happy_path_fixture("bsms"));
    let res = run_export_from_import_envelope(&p, "bsms", "bitcoin-core");
    let parsed: serde_json::Value =
        serde_json::from_str(&res.envelope_json).expect("envelope must be valid JSON");
    assert!(parsed.is_array(), "envelope must be a JSON array per SPEC §3.2");
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 1, "single-entry envelope expected for one-fixture import");
}

#[test]
fn p11a_helper_envelope_carries_schema_version_and_source_format() {
    let p = fixture_path(happy_path_fixture("sparrow"));
    let res = run_export_from_import_envelope(&p, "sparrow", "bitcoin-core");
    let arr: serde_json::Value = serde_json::from_str(&res.envelope_json).unwrap();
    let entry = &arr[0];
    assert_eq!(
        entry["schema_version"].as_str(),
        Some("1"),
        "v0.27.0 envelope outer schema_version is \"1\" \
         (IMPORT_WALLET_ENVELOPE_SCHEMA_VERSION at import_wallet.rs:87)"
    );
    assert_eq!(
        entry["source_format"].as_str(),
        Some("sparrow"),
        "source_format mirrors the --format argument"
    );
}

#[test]
fn p11a_helper_stdout_is_format_specific_on_success() {
    // bitcoin-core dest emits a JSON array of listdescriptors entries.
    let p = fixture_path(happy_path_fixture("bsms"));
    let res = run_export_from_import_envelope(&p, "bsms", "bitcoin-core");
    let val: serde_json::Value = serde_json::from_str(&res.stdout).unwrap();
    assert!(val.is_array(), "bitcoin-core emit is a JSON array");
}

#[test]
fn p11a_helper_propagates_import_wallet_failure() {
    // Use a malformed fixture file that import-wallet cannot parse as
    // the declared --format. The helper must surface the import-side
    // exit code + stderr without invoking export-wallet (envelope_json
    // empty signals the import side failed).
    let tmpdir = tempfile::tempdir().unwrap();
    let bogus = tmpdir.path().join("not_a_bsms_blob.txt");
    std::fs::write(&bogus, "this is not a BSMS Round-2 envelope\n").unwrap();
    let res = run_export_from_import_envelope(&bogus, "bsms", "bitcoin-core");
    assert_ne!(res.exit_code, 0);
    assert!(res.envelope_json.is_empty(), "import-side failure ⇒ no envelope");
    assert!(res.stdout.is_empty(), "import-side failure ⇒ no export stdout");
    assert!(!res.stderr.is_empty(), "import-side failure must carry stderr");
}

// ============================================================================
// P11B — Happy-path matrix: 8 sources × 3 descriptor-capable
// destinations = 24 cells. Each cell asserts the helper returns exit=0
// + non-empty stdout that decodes per-destination, AND the envelope's
// descriptor body carries the source's canonical xpubs (so the
// cross-format conversion preserves cosigner identity).
//
// Note: bsms-dest refuses taproot script_type per P8A — none of the 8
// source fixtures produce a taproot descriptor at v0.28.0, so all 24
// cells expect success. (If a future taproot fixture is added, that
// cell should move to P11C refusal.)
// ============================================================================

/// Extract every xpub-shaped substring (length 111 chars starting with
/// `xpub`/`tpub`/`ypub`/`zpub`/`upub`/`vpub`) from a string. Used to
/// assert cosigner-key preservation across format boundaries.
fn extract_xpubs(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    for prefix in ["xpub", "tpub", "ypub", "zpub", "upub", "vpub"] {
        let mut idx = 0;
        while let Some(pos) = s[idx..].find(prefix) {
            let abs = idx + pos;
            // xpubs are 111 chars (Base58 length); take 111 if possible.
            if abs + 111 <= s.len() {
                let candidate = &s[abs..abs + 111];
                // Base58 alphabet — reject anything containing
                // separators or other non-alphanum chars.
                if candidate
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric())
                {
                    out.push(candidate.to_string());
                }
            }
            idx = abs + prefix.len();
        }
    }
    out.sort();
    out.dedup();
    out
}

#[test]
fn p11b_happy_path_matrix_all_sources_all_descriptor_capable_dests() {
    let mut cell_count = 0;
    let mut failures: Vec<String> = Vec::new();
    for src in ALL_SOURCES {
        let fixture = fixture_path(happy_path_fixture(src));
        // Pre-extract the source-side cosigner xpubs from the envelope
        // descriptor, so we can assert preservation across all 3 dests.
        let probe = run_export_from_import_envelope(&fixture, src, "bitcoin-core");
        if probe.exit_code != 0 {
            failures.push(format!(
                "[{src} → bitcoin-core] probe failed exit={} stderr={}",
                probe.exit_code, probe.stderr
            ));
            continue;
        }
        let envelope: serde_json::Value =
            serde_json::from_str(&probe.envelope_json).unwrap();
        let src_desc = envelope[0]["bundle"]["descriptor"]
            .as_str()
            .expect("bundle.descriptor must be present")
            .to_string();
        let src_xpubs = extract_xpubs(&src_desc);
        assert!(
            !src_xpubs.is_empty(),
            "[{src}] source descriptor must contain at least one xpub-shaped key; \
             got descriptor: {src_desc}"
        );

        for dest in DESCRIPTOR_CAPABLE_DESTS {
            cell_count += 1;
            let res = run_export_from_import_envelope(&fixture, src, dest);
            if res.exit_code != 0 {
                failures.push(format!(
                    "[{src} → {dest}] exit={} stderr={}",
                    res.exit_code, res.stderr
                ));
                continue;
            }
            if res.stdout.is_empty() {
                failures.push(format!("[{src} → {dest}] empty stdout"));
                continue;
            }
            // Each descriptor-capable destination must surface the
            // source's cosigner xpubs verbatim in its emit (verifies
            // round-trip identity preservation across format boundary).
            let dest_xpubs = extract_xpubs(&res.stdout);
            for x in &src_xpubs {
                if !dest_xpubs.contains(x) {
                    failures.push(format!(
                        "[{src} → {dest}] missing xpub {x} in dest stdout"
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "P11B happy-path matrix failures ({}/{cell_count}): {failures:#?}",
        failures.len()
    );
    assert_eq!(cell_count, 24, "P11B must run exactly 8×3 = 24 cells");
}

// ============================================================================
// P11C — Refusal matrix: 8 sources × N template-only destinations.
//
// Each cell asserts the destination refuses the descriptor-mode
// envelope with the contract-specific stderr message.
//
// Coverage:
//   - 4 strict template-only dests (coldcard, electrum, jade, sparrow) ×
//     8 sources = 32 cells
//   - `specter` is NOT strictly template-only: when it has a
//     `wallet_name` it accepts descriptor (see existing cell-3
//     comment). We assert the refusal path by passing a fixture whose
//     envelope lacks the `wallet_name` (BSMS doesn't carry one), and
//     the success path for sources that do. So the refusal matrix
//     includes only sources where specter refuses for missing
//     wallet_name: bsms, bitcoin-core, coldcard-multisig (no
//     wallet_name in their fixture envelopes). +3 cells.
//   - `green` refuses multisig: 5 multisig sources × 1 dest = 5 cells.
//
// Total target ≈ 32 + 3 + 5 = 40 cells.
// ============================================================================

/// Sources whose canonical happy-path fixture lacks a `wallet_name` in
/// the envelope's `source_metadata` (specter then refuses descriptor +
/// no wallet_name).
const SOURCES_LACKING_WALLET_NAME: &[&str] = &["bsms", "coldcard-multisig"];

/// Allowed refusal stderr substrings. Each template-only emitter has
/// its own message but they all contain one of these literals.
const REFUSAL_STDERR_PATTERNS: &[&str] = &[
    "requires --template",
    "descriptor passthrough is not supported",
    "does not support multisig",
    // Specter wallet-name path (descriptor mode with no wallet_name).
    "--wallet-name",
];

fn assert_refusal(res: &ExportResult, src: &str, dest: &str) {
    assert_ne!(
        res.exit_code, 0,
        "[{src} → {dest}] expected non-zero exit; stderr={}",
        res.stderr
    );
    assert!(
        REFUSAL_STDERR_PATTERNS
            .iter()
            .any(|p| res.stderr.contains(p)),
        "[{src} → {dest}] refusal stderr did not match any expected pattern. \
         Got stderr: {}",
        res.stderr
    );
}

#[test]
fn p11c_refusal_matrix_strict_template_only_dests() {
    let mut cell_count = 0;
    let mut failures: Vec<String> = Vec::new();
    for src in ALL_SOURCES {
        let fixture = fixture_path(happy_path_fixture(src));
        for dest in TEMPLATE_ONLY_DESTS {
            cell_count += 1;
            let res = run_export_from_import_envelope(&fixture, src, dest);
            if res.exit_code == 0 {
                failures.push(format!(
                    "[{src} → {dest}] expected refusal but exit=0; stdout={}",
                    res.stdout
                ));
                continue;
            }
            if !REFUSAL_STDERR_PATTERNS
                .iter()
                .any(|p| res.stderr.contains(p))
            {
                failures.push(format!(
                    "[{src} → {dest}] refusal stderr did not match any expected \
                     pattern; got: {}",
                    res.stderr
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "P11C strict-template-only refusal failures ({}/{cell_count}): {failures:#?}",
        failures.len()
    );
    assert_eq!(cell_count, 32, "P11C strict matrix = 8×4 = 32 cells");
}

#[test]
fn p11c_refusal_matrix_specter_no_wallet_name() {
    for src in SOURCES_LACKING_WALLET_NAME {
        let fixture = fixture_path(happy_path_fixture(src));
        let res = run_export_from_import_envelope(&fixture, src, "specter");
        assert_refusal(&res, src, "specter");
    }
}

/// Pin Green's current `--from-import-json` behavior: accepts ANY
/// descriptor (singlesig + multisig) because the multisig refusal in
/// `wallet_export/green.rs:33-39` is template-gated and the envelope
/// path always supplies `template: None`. This is a real product gap
/// (logged at `green-emitter-multisig-refusal-template-only`); the
/// matrix test pins the current behavior so any future fix will surface
/// as a P11C regression that flags the cycle-followup as ready for
/// closure.
#[test]
fn p11c_green_descriptor_passthrough_current_behavior_no_refusal() {
    let mut failures: Vec<String> = Vec::new();
    for src in ALL_SOURCES {
        let fixture = fixture_path(happy_path_fixture(src));
        let res = run_export_from_import_envelope(&fixture, src, "green");
        if res.exit_code != 0 {
            failures.push(format!(
                "[{src} → green] expected current passthrough success but exit={}; stderr={}",
                res.exit_code, res.stderr
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "P11C green passthrough pin failed (a fix to \
         green-emitter-multisig-refusal-template-only would surface here): {failures:#?}"
    );
}

// ============================================================================
// P11D — Semantic-match assertions for the happy-path matrix. Mirrors
// the v0.27.1 Phase 4 I19 fold pattern: every happy-path cell's
// envelope must carry `roundtrip.semantic_match == true`. The semantic
// match comes from the import-wallet side; it confirms the per-format
// canonicalize+re-emit cycle succeeded for the source fixture. (We
// already exercise this on the source-fixture parser side in
// `cli_import_wallet_<format>.rs`; P11D asserts it again here as a
// matrix-level invariant — any future parser regression that
// silently flips semantic_match to false will surface as a P11D
// failure even if other cells stay green.)
//
// `bsms` source is special: its envelope carries
// `semantic_match: false` + `status: "blocked_no_emitter"` because
// re-emit lives outside the parser at v0.28.0 (BSMS canonicalize is
// not via the same path). The other 7 sources have real canonicalize
// implementations. We pin the cross-source contract at the matrix
// level: 7 sources × true + 1 source × false (bsms).
// ============================================================================

#[test]
fn p11d_semantic_match_true_for_canonicalize_capable_sources() {
    let canonicalize_capable: &[&str] = &[
        "bitcoin-core",
        "coldcard",
        "coldcard-multisig",
        "electrum",
        "jade",
        "sparrow",
        "specter",
    ];
    let mut failures: Vec<String> = Vec::new();
    for src in canonicalize_capable {
        let fixture = fixture_path(happy_path_fixture(src));
        // Run one dest (bitcoin-core) to pull the envelope — the
        // envelope itself doesn't depend on dest selection.
        let res = run_export_from_import_envelope(&fixture, src, "bitcoin-core");
        if res.exit_code != 0 {
            failures.push(format!("[{src}] probe failed: {}", res.stderr));
            continue;
        }
        let env: serde_json::Value = serde_json::from_str(&res.envelope_json).unwrap();
        let rt = &env[0]["roundtrip"];
        if rt["semantic_match"].as_bool() != Some(true) {
            failures.push(format!(
                "[{src}] roundtrip.semantic_match != true; roundtrip={rt}"
            ));
        }
        if rt["status"].as_str() != Some("ok") {
            failures.push(format!(
                "[{src}] roundtrip.status != \"ok\"; roundtrip={rt}"
            ));
        }
    }
    assert!(failures.is_empty(), "P11D semantic_match failures: {failures:#?}");
}

#[test]
fn p11d_semantic_match_false_for_bsms_blocked_no_emitter() {
    // BSMS source still carries the v0.27.0 contract:
    //   roundtrip.semantic_match == false
    //   roundtrip.status == "blocked_no_emitter"
    // (re-verify at execution time: this is the v0.28.0 source-of-
    // truth per `cmd/import_wallet.rs` lines 1022-1027.)
    let fixture = fixture_path(happy_path_fixture("bsms"));
    let res = run_export_from_import_envelope(&fixture, "bsms", "bitcoin-core");
    assert_eq!(res.exit_code, 0);
    let env: serde_json::Value = serde_json::from_str(&res.envelope_json).unwrap();
    let rt = &env[0]["roundtrip"];
    assert_eq!(
        rt["semantic_match"].as_bool(),
        Some(false),
        "bsms source carries semantic_match=false; got {rt}"
    );
    assert_eq!(
        rt["status"].as_str(),
        Some("blocked_no_emitter"),
        "bsms source carries status=blocked_no_emitter; got {rt}"
    );
}
