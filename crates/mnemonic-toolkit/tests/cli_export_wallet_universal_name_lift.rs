//! v0.37.8 — `export-wallet --from-import-json` universal source-name lift.
//!
//! SPEC `design/SPEC_sparrow_name_universal_lift.md` §6 (test matrix).
//! Each cell rounds an import-format fixture through `import-wallet --json`
//! to produce an envelope, then re-exports the envelope WITHOUT supplying
//! `--wallet-name`. Pre-v0.37.8 the export side defaulted to the literal
//! placeholder `"imported-descriptor"` (and Specter additionally refused
//! via `MissingField::WalletName`). Post-v0.37.8 the envelope's per-format
//! `*_source_metadata.<name-key>` flows into `EmitInputs.wallet_name` AND
//! flips `wallet_name_is_non_default = true`.
//!
//! Six positive integration cells (one per name-carrying format) + one
//! Specter-target cell (was previously broken — exercises the
//! `MissingField::WalletName` unblock) + one explicit-override cell
//! (`--wallet-name` beats the lifted name). 8 cells total.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

/// Pipe a fixture through `import-wallet --json` and back through
/// `export-wallet --from-import-json -` WITHOUT supplying `--wallet-name`.
/// Returns (stdout, stderr, exit_code). Mirrors
/// `cli_export_wallet_from_import_json::run_export_from_import_envelope` but
/// keeps the no-name discipline (the lift is the point).
fn pipe_no_wallet_name(
    fixture: &Path,
    source_format: &str,
    dest_format: &str,
) -> (String, String, i32) {
    let import = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            fixture.to_str().unwrap(),
            "--format",
            source_format,
            "--json",
        ])
        .output()
        .expect("import-wallet spawn");
    assert!(
        import.status.success(),
        "import-wallet --format {source_format} failed: stderr={}",
        String::from_utf8_lossy(&import.stderr)
    );
    let envelope = import.stdout;

    let export = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--from-import-json",
            "-",
            "--format",
            dest_format,
        ])
        .write_stdin(envelope)
        .output()
        .expect("export-wallet spawn");
    (
        String::from_utf8_lossy(&export.stdout).into_owned(),
        String::from_utf8_lossy(&export.stderr).into_owned(),
        export.status.code().unwrap_or(-1),
    )
}

/// Pipe a fixture through with `--wallet-name <explicit>` — used by the
/// explicit-override cell to assert precedence: explicit beats lifted.
fn pipe_with_explicit_name(
    fixture: &Path,
    source_format: &str,
    dest_format: &str,
    wallet_name: &str,
) -> (String, String, i32) {
    let import = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            fixture.to_str().unwrap(),
            "--format",
            source_format,
            "--json",
        ])
        .output()
        .expect("import-wallet spawn");
    assert!(import.status.success());
    let envelope = import.stdout;

    let export = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--from-import-json",
            "-",
            "--format",
            dest_format,
            "--wallet-name",
            wallet_name,
        ])
        .write_stdin(envelope)
        .output()
        .expect("export-wallet spawn");
    (
        String::from_utf8_lossy(&export.stdout).into_owned(),
        String::from_utf8_lossy(&export.stderr).into_owned(),
        export.status.code().unwrap_or(-1),
    )
}

/// Sparrow target carries the wallet name in the top-level `name` field.
/// Used by 6 positive cells + 1 explicit-override.
fn sparrow_name(stdout: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(stdout).expect("sparrow output must be JSON");
    v["name"].as_str().expect("sparrow.name").to_string()
}

/// Specter target carries the wallet name in the top-level `label` field.
/// Used by the specter-target cell.
fn specter_label(stdout: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(stdout).expect("specter output must be JSON");
    v["label"].as_str().expect("specter.label").to_string()
}

// ============================================================================
// Integration cell 1/6 — sparrow source → sparrow target.
// Sparrow's `name` field round-trips into the envelope's
// `sparrow_source_metadata.label`, then lifts back into Sparrow's `name`.
// ============================================================================
#[test]
fn integration_cell_1_sparrow_source_name_lifts_into_sparrow_target() {
    let fixture = fixture_path("sparrow-multisig-2of3-p2wsh-sortedmulti.json");
    let (stdout, stderr, exit) = pipe_no_wallet_name(&fixture, "sparrow", "sparrow");
    assert_eq!(
        exit, 0,
        "sparrow→sparrow no-name round-trip must succeed; stderr={stderr}"
    );
    assert_eq!(
        sparrow_name(&stdout),
        "wsh-sortedmulti-0",
        "lifted name must equal the source fixture's `name`"
    );
}

// ============================================================================
// Integration cell 2/6 — specter source → sparrow target.
// Specter's `label` lifts via `specter_source_metadata.label`.
// ============================================================================
#[test]
fn integration_cell_2_specter_source_name_lifts_into_sparrow_target() {
    let fixture = fixture_path("specter-multisig-2of3-p2wsh-sortedmulti.json");
    let (stdout, stderr, exit) = pipe_no_wallet_name(&fixture, "specter", "sparrow");
    assert_eq!(
        exit, 0,
        "specter→sparrow no-name round-trip must succeed; stderr={stderr}"
    );
    assert_eq!(sparrow_name(&stdout), "VaultColdStorage");
}

// ============================================================================
// Integration cell 3/6 — jade source → sparrow target.
// Jade's `multisig_name` lifts via the nested
// `jade_source_metadata.coldcard_compat.name`. This cell is the production
// guard for the `walk_str` nested-path traversal (the unit cell guards the
// helper directly; this guards the full end-to-end pipe).
// ============================================================================
#[test]
fn integration_cell_3_jade_source_name_lifts_via_nested_coldcard_compat_path() {
    let fixture = fixture_path("jade-multisig-2of3-p2wsh.json");
    let (stdout, stderr, exit) = pipe_no_wallet_name(&fixture, "jade", "sparrow");
    assert_eq!(
        exit, 0,
        "jade→sparrow no-name round-trip must succeed; stderr={stderr}"
    );
    assert_eq!(sparrow_name(&stdout), "TestMs2of3");
}

// ============================================================================
// Integration cell 4/6 — electrum source → sparrow target.
// Electrum's `wallet_name` lifts via `electrum_source_metadata.wallet_name`.
// Note electrum→sparrow is the singlesig-on-singlesig branch (bip84).
// ============================================================================
#[test]
fn integration_cell_4_electrum_source_name_lifts_into_sparrow_target() {
    let fixture = fixture_path("electrum-standard-bip84-mainnet.json");
    let (stdout, stderr, exit) = pipe_no_wallet_name(&fixture, "electrum", "sparrow");
    assert_eq!(
        exit, 0,
        "electrum→sparrow no-name round-trip must succeed; stderr={stderr}"
    );
    assert_eq!(sparrow_name(&stdout), "Daily");
}

// ============================================================================
// Integration cell 5/6 — bitcoin-core source → sparrow target.
// Core's `wallet_name` lifts via the (un-prefixed) `source_metadata.
// wallet_name`. The lift's tie-break order probes this slot first
// per `resolved_wallet_name` priority, but per-envelope at-most-one
// population means tie-break doesn't fire on real inputs.
// ============================================================================
#[test]
fn integration_cell_5_bitcoin_core_source_name_lifts_into_sparrow_target() {
    let fixture = fixture_path("core-bip84-mainnet.json");
    let (stdout, stderr, exit) = pipe_no_wallet_name(&fixture, "bitcoin-core", "sparrow");
    assert_eq!(
        exit, 0,
        "bitcoin-core→sparrow no-name round-trip must succeed; stderr={stderr}"
    );
    assert_eq!(sparrow_name(&stdout), "bip84_mainnet");
}

// ============================================================================
// Integration cell 6/6 — coldcard-multisig source → sparrow target.
// Coldcard-MS's `Name:` header lifts via the TOP-LEVEL
// `coldcard_multisig_source_metadata.name` (distinct from Jade's NESTED
// `jade_source_metadata.coldcard_compat.name` even though they parse the
// same metadata struct — Jade delegates and wraps).
// ============================================================================
#[test]
fn integration_cell_6_coldcard_multisig_source_name_lifts_into_sparrow_target() {
    let fixture = fixture_path("coldcard-ms-2of3-p2wsh-with-xfp.txt");
    let (stdout, stderr, exit) = pipe_no_wallet_name(&fixture, "coldcard-multisig", "sparrow");
    assert_eq!(
        exit, 0,
        "coldcard-multisig→sparrow no-name round-trip must succeed; stderr={stderr}"
    );
    assert_eq!(sparrow_name(&stdout), "TestMs2of3");
}

// ============================================================================
// Specter-target cell — pre-v0.37.8 ANY `--from-import-json` to specter
// without `--wallet-name` exited 2 with `MissingField::WalletName`. The
// universal-lift unblocks every source format whose envelope carries a
// name. This cell is the production guard against that unblock regressing.
// ============================================================================
#[test]
fn specter_target_no_wallet_name_lifts_from_sparrow_source_succeeds() {
    let fixture = fixture_path("sparrow-multisig-2of3-p2wsh-sortedmulti.json");
    let (stdout, stderr, exit) = pipe_no_wallet_name(&fixture, "sparrow", "specter");
    assert_eq!(
        exit, 0,
        "specter target with NO --wallet-name MUST succeed when the envelope carries \
         a liftable name (pre-v0.37.8 this exited 2 via MissingField::WalletName); \
         stderr={stderr}"
    );
    assert_eq!(
        specter_label(&stdout),
        "wsh-sortedmulti-0",
        "lifted name must reach Specter's `label` field"
    );
}

// ============================================================================
// Explicit-override cell — `--wallet-name <X>` ALWAYS beats the envelope's
// lifted name. Without this guard the lift could silently dictate the
// emitted name even when the user supplied an override on the CLI.
// ============================================================================
#[test]
fn explicit_wallet_name_overrides_envelope_lifted_name() {
    let fixture = fixture_path("sparrow-multisig-2of3-p2wsh-sortedmulti.json");
    let (stdout, stderr, exit) =
        pipe_with_explicit_name(&fixture, "sparrow", "sparrow", "ExplicitOverride");
    assert_eq!(
        exit, 0,
        "explicit-name override must succeed; stderr={stderr}"
    );
    assert_eq!(
        sparrow_name(&stdout),
        "ExplicitOverride",
        "explicit --wallet-name MUST beat lifted name `wsh-sortedmulti-0`"
    );
}
