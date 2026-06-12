//! v0.42.0 — `mnemonic export-wallet --format descriptor`.
//!
//! Emits the bare canonical multipath `<descriptor>#<checksum>` on one line
//! (stdout or `--output <file>`), no wallet-file wrapper. Works for single-sig
//! AND multisig (unlike `--format green`, which refuses multisig).
//!
//! See `design/SPEC_export_wallet_format_descriptor.md` §7 (tests 1-6):
//!   1 single-sig (smoke + exact)   4 flags-ignored
//!   2 multisig (NOT refused)       5 --output
//!   3 round-trip (from-import-json) 6 partition guard (in src unit test)
//!   + taproot via direct --descriptor passthrough (R0-M3: from-import-json
//!     refuses taproot, so taproot reaches --format descriptor only here).

use assert_cmd::Command;
use std::path::Path;

/// abandon×11 + about test seed's bip84 account xpub (m/84'/0'/0').
/// Hardcoded LITERAL (SPEC R0-m1) — avoids shelling `convert` (label-prefixes
/// `xpub: …`) and the cross-binary stdout-shape coupling.
const ACCT_XPUB_BIP84: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";

/// 2-of-N cosigner account xpubs (m/48'/0'/0'/2'), reused from the Specter
/// test fixtures. Two distinct valid account xpubs for the multisig cell.
const COSIGNER_A_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const COSIGNER_B_XPUB: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";

fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("tests/fixtures/wallet_import").join(name)
}

/// Strip the BIP-380 `#<csum>` suffix → canonical descriptor body.
/// The round-trip equivalence is "body == body (modulo checksum recompute)".
fn body(descriptor: &str) -> &str {
    descriptor
        .rsplit_once('#')
        .map(|(b, _)| b)
        .unwrap_or(descriptor)
}

/// Pull the canonical descriptor out of an `import-wallet --json` envelope.
/// The envelope is a 1-element array of bundle objects; the descriptor lives
/// at `[0].bundle.descriptor`.
fn envelope_descriptor(envelope_json: &str) -> String {
    let envelope: serde_json::Value = serde_json::from_str(envelope_json).unwrap();
    envelope[0]["bundle"]["descriptor"]
        .as_str()
        .expect("envelope [0].bundle.descriptor must be a string")
        .to_owned()
}

/// Copy of the P11A helper `run_export_from_import_envelope`
/// (`cli_export_wallet_from_import_json.rs:491`) — each `tests/*.rs` is its
/// own crate, so the helper can't be shared. Composes:
///   `import-wallet --blob <fixture> --format <src> --json`
///     → `export-wallet --from-import-json - --format <dest>`.
/// Returns `(envelope_json, export_stdout)`.
fn run_export_from_import_envelope(
    source_fixture: &Path,
    source_format: &str,
    dest_format: &str,
) -> (String, String) {
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
    assert!(
        import.status.success(),
        "import-wallet failed: {}",
        String::from_utf8_lossy(&import.stderr)
    );
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
    assert!(
        export.status.success(),
        "export-wallet --format descriptor failed: {}",
        String::from_utf8_lossy(&export.stderr)
    );
    let export_stdout = String::from_utf8(export.stdout).expect("export stdout non-utf8");
    (envelope_json, export_stdout)
}

/// SPEC §7 test 1 (smoke) — single-sig bip84, stdout: one line, starts
/// `wpkh(`, contains `<0;1>`, ends `#<8 alnum>\n` (single trailing newline).
#[test]
fn export_descriptor_singlesig_bip84_smoke() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            &format!("@0.xpub={ACCT_XPUB_BIP84}"),
            "--format",
            "descriptor",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();

    // Exactly one line (single trailing newline, no extra lines).
    assert_eq!(
        stdout.matches('\n').count(),
        1,
        "expected exactly one trailing newline, got: {stdout:?}"
    );
    assert!(stdout.ends_with('\n'), "must end with newline: {stdout:?}");
    let line = stdout.trim_end_matches('\n');
    assert!(!line.contains('\n'), "must be one line: {line:?}");

    assert!(line.starts_with("wpkh("), "must start with wpkh(: {line:?}");
    assert!(line.contains("<0;1>"), "must be multipath <0;1>: {line:?}");

    // Ends `#<8 alnum>`.
    let pos = line.rfind('#').expect("must carry BIP-380 #checksum");
    let csum = &line[pos + 1..];
    assert_eq!(csum.len(), 8, "checksum must be 8 chars: {csum:?}");
    assert!(
        csum.chars().all(|c| c.is_ascii_alphanumeric()),
        "checksum must be ASCII-alphanumeric: {csum:?}"
    );
}

/// SPEC §7 test 1 (exact) — the bare account xpub carries no master
/// fingerprint, so the toolkit emits the BIP-32 zero fingerprint `00000000`
/// in the `[fp/path]` origin (a bare xpub has no origin to lift). bip84 default
/// path m/84'/0'/0'. The exact canonical multipath line, captured from the
/// built binary.
#[test]
fn export_descriptor_singlesig_bip84_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            &format!("@0.xpub={ACCT_XPUB_BIP84}"),
            "--format",
            "descriptor",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        "wpkh([00000000/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*)#d9qwe873\n",
    );
}

/// SPEC §7 test 2 — multisig is NOT refused (unlike `--format green`).
/// `wsh-sortedmulti --threshold 2` with two distinct cosigner xpubs →
/// `wsh(sortedmulti(2,…))#<csum>` on one line.
#[test]
fn export_descriptor_multisig_wsh_sortedmulti_not_refused() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--slot",
            &format!("@0.xpub={COSIGNER_A_XPUB}"),
            "--slot",
            &format!("@1.xpub={COSIGNER_B_XPUB}"),
            "--format",
            "descriptor",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();

    assert_eq!(stdout.matches('\n').count(), 1, "one line: {stdout:?}");
    let line = stdout.trim_end_matches('\n');
    assert!(
        line.starts_with("wsh(sortedmulti(2,"),
        "must be wsh(sortedmulti(2,…): {line:?}"
    );
    assert!(
        line.contains(COSIGNER_A_XPUB),
        "cosigner A present: {line:?}"
    );
    assert!(
        line.contains(COSIGNER_B_XPUB),
        "cosigner B present: {line:?}"
    );
    // multipath + checksum.
    assert!(line.contains("<0;1>"), "multipath form: {line:?}");
    let csum = &line[line.rfind('#').expect("checksum") + 1..];
    assert_eq!(csum.len(), 8, "8-char checksum: {csum:?}");
    assert!(csum.chars().all(|c| c.is_ascii_alphanumeric()));
}

/// SPEC §7 test 3 (headline) — single-sig round-trip:
/// `import-wallet <bitcoin-core fixture> --json`
///   → `export-wallet --from-import-json - --format descriptor`.
/// The emitted descriptor's canonical BODY == the envelope's `descriptor`
/// field body (modulo checksum recompute).
#[test]
fn export_descriptor_round_trip_singlesig_from_bitcoin_core() {
    let fixture = fixture_path("core-bip84-mainnet.json");
    let (envelope_json, export_stdout) =
        run_export_from_import_envelope(&fixture, "bitcoin-core", "descriptor");

    assert_eq!(
        export_stdout.matches('\n').count(),
        1,
        "one line: {export_stdout:?}"
    );
    let out_desc = export_stdout.trim_end_matches('\n');
    assert!(
        out_desc.starts_with("wpkh("),
        "single-sig wpkh: {out_desc:?}"
    );

    let env_desc = envelope_descriptor(&envelope_json);
    assert_eq!(
        body(out_desc),
        body(&env_desc),
        "round-trip body must equal envelope body (modulo checksum)"
    );
}

/// SPEC §7 test 3 (headline) — wsh-multisig round-trip:
/// `import-wallet <sparrow 2-of-3 fixture> --json`
///   → `export-wallet --from-import-json - --format descriptor`.
/// Same body-equivalence assertion, for a `wsh(sortedmulti(…))` policy.
#[test]
fn export_descriptor_round_trip_multisig_from_sparrow() {
    let fixture = fixture_path("sparrow-multisig-2of3-p2wsh-sortedmulti.json");
    let (envelope_json, export_stdout) =
        run_export_from_import_envelope(&fixture, "sparrow", "descriptor");

    assert_eq!(
        export_stdout.matches('\n').count(),
        1,
        "one line: {export_stdout:?}"
    );
    let out_desc = export_stdout.trim_end_matches('\n');
    assert!(
        out_desc.starts_with("wsh(sortedmulti(2,"),
        "2-of-3 wsh-sortedmulti: {out_desc:?}"
    );

    let env_desc = envelope_descriptor(&envelope_json);
    assert_eq!(
        body(out_desc),
        body(&env_desc),
        "multisig round-trip body must equal envelope body (modulo checksum)"
    );
}

/// SPEC §7 (taproot via passthrough; R0-M3) — taproot is refused on the
/// from-import-json leg, so it reaches `--format descriptor` only via direct
/// `--descriptor` passthrough. A minimal valid `tr(...)` multipath descriptor
/// (no taproot refusal in the passthrough path) emits unchanged.
#[test]
fn export_descriptor_taproot_via_descriptor_passthrough() {
    let tr = "tr([00000000/86'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*)#vc7af3gk";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--network",
            "mainnet",
            "--descriptor",
            tr,
            "--format",
            "descriptor",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout,
        format!("{tr}\n"),
        "taproot passthrough emits unchanged"
    );
}

/// SPEC §7 test 4 — `--range`/`--timestamp` are silently ignored for
/// `--format descriptor` (green precedent); no error, same line as without.
#[test]
fn export_descriptor_ignores_range_and_timestamp() {
    let base = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            &format!("@0.xpub={ACCT_XPUB_BIP84}"),
            "--format",
            "descriptor",
        ])
        .assert()
        .success();
    let base_out = String::from_utf8(base.get_output().stdout.clone()).unwrap();

    let with_flags = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            &format!("@0.xpub={ACCT_XPUB_BIP84}"),
            "--range",
            "0,5",
            "--timestamp",
            "1700000000",
            "--format",
            "descriptor",
        ])
        .assert()
        .success();
    let flags_out = String::from_utf8(with_flags.get_output().stdout.clone()).unwrap();

    assert_eq!(
        base_out, flags_out,
        "--range/--timestamp must be ignored — same descriptor line"
    );
}

/// SPEC §7 test 5 — `--output <file>` writes the one-line descriptor
/// (single trailing `\n` added by the write tail; emit returns no newline).
#[test]
fn export_descriptor_to_output_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("desc.txt");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            &format!("@0.xpub={ACCT_XPUB_BIP84}"),
            "--format",
            "descriptor",
            "--output",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();

    let written = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        written.matches('\n').count(),
        1,
        "file has single trailing newline: {written:?}"
    );
    let line = written.trim_end_matches('\n');
    assert!(line.starts_with("wpkh("), "file content: {line:?}");
    assert!(line.contains("<0;1>"), "multipath: {line:?}");
    assert!(line.ends_with("#d9qwe873"), "checksum suffix: {line:?}");
}
