//! v0.28.0 Phase P4C — Coldcard multisig text-file parser integration tests.
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.4. Tests the library
//! boundary via the CLI scaffold (`cmd/import_wallet.rs`) extended in P4C
//! to dispatch `--format coldcard-multisig` to `ColdcardMultisigParser`
//! and to wire the auto-sniff path for the text-shape blob.
//!
//! Self-contained: no dependency on adjacent repos or external network.
//! Uses the 4 fixtures introduced in P4B:
//! - `coldcard-ms-2of3-p2wsh-with-xfp.txt` — happy-path with leading XFP header.
//! - `coldcard-ms-2of3-p2wsh-no-xfp.txt` — happy-path without leading XFP header.
//! - `coldcard-ms-3of5-p2wsh.txt` — 3-of-5 multisig.
//! - `coldcard-ms-malformed-missing-format.txt` — refused with diagnostic.

use assert_cmd::Command;

const FIX_2OF3_WITH_XFP: &str = "tests/fixtures/wallet_import/coldcard-ms-2of3-p2wsh-with-xfp.txt";
const FIX_2OF3_NO_XFP: &str = "tests/fixtures/wallet_import/coldcard-ms-2of3-p2wsh-no-xfp.txt";
const FIX_3OF5: &str = "tests/fixtures/wallet_import/coldcard-ms-3of5-p2wsh.txt";
const FIX_MALFORMED: &str = "tests/fixtures/wallet_import/coldcard-ms-malformed-missing-format.txt";

/// Run `mnemonic import-wallet --blob <path> --format coldcard-multisig`.
fn run_explicit(path: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            path,
            "--format",
            "coldcard-multisig",
        ])
        .assert()
}

/// Run `mnemonic import-wallet --blob <path>` (no `--format`); auto-sniff path.
fn run_autosniff(path: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", path])
        .assert()
}

/// Run with a synthetic blob piped on stdin.
fn run_stdin(blob: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "coldcard-multisig",
        ])
        .write_stdin(blob.to_string())
        .assert()
}

// ============================================================================
// §11.4 — happy-path parse via explicit --format coldcard-multisig
// ============================================================================

#[test]
fn coldcard_ms_2of3_with_xfp_happy_path() {
    let out = run_explicit(FIX_2OF3_WITH_XFP).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(
        stdout.contains("bundles[0].cosigners=3"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("bundles[0].network=mainnet"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("bundles[0].threshold=2"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("bundles[0].bsms_audit=none"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("bundles[0].entropy=none"),
        "stdout: {stdout}"
    );
    // Coldcard-multisig provenance is neither bsms nor bitcoin-core; the
    // `source_metadata` accessor returns None per ImportProvenance enum semantics.
    assert!(
        stdout.contains("bundles[0].source_metadata=none"),
        "stdout: {stdout}"
    );
    // Silent stderr on the with-xfp fixture (row 1 of truth table for all cosigners).
    assert!(
        !stderr.contains("warning:"),
        "stderr must be silent on with-xfp happy-path; got: {stderr}"
    );
}

#[test]
fn coldcard_ms_2of3_no_xfp_happy_path() {
    let out = run_explicit(FIX_2OF3_NO_XFP).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stdout.contains("bundles[0].cosigners=3"));
    assert!(stdout.contains("bundles[0].threshold=2"));
    assert!(!stderr.contains("warning:"));
}

#[test]
fn coldcard_ms_3of5_happy_path() {
    let out = run_explicit(FIX_3OF5).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stdout.contains("bundles[0].cosigners=5"));
    assert!(stdout.contains("bundles[0].threshold=3"));
    assert!(!stderr.contains("warning:"));
}

// ============================================================================
// §11.4 — auto-sniff: blob with Name/Policy/Format text-shape → ColdcardMultisig
// ============================================================================

#[test]
fn coldcard_ms_autosniff_2of3_with_xfp_dispatches_correctly() {
    let out = run_autosniff(FIX_2OF3_WITH_XFP).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
}

#[test]
fn coldcard_ms_autosniff_3of5_dispatches_correctly() {
    let out = run_autosniff(FIX_3OF5).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles[0].cosigners=5"));
}

// ============================================================================
// §11.4 — refusal on malformed input
// ============================================================================

#[test]
fn coldcard_ms_malformed_missing_format_refused() {
    let out = run_explicit(FIX_MALFORMED).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard-multisig"),
        "stderr must cite format; got: {stderr}"
    );
    assert!(
        stderr.contains("missing `Format:` header"),
        "stderr must cite missing field; got: {stderr}"
    );
    // SPEC §2.4 row: ImportWalletParse → exit code 2.
    let code = out.get_output().status.code().expect("exit code present");
    assert_eq!(code, 2, "ImportWalletParse must exit 2; got {code}");
}

// ============================================================================
// SPEC §11.4.1 — xfp-divergence WARNING cell (row 2 of the truth table)
// ============================================================================

#[test]
fn coldcard_ms_xfp_header_divergence_warns_byte_exact_template() {
    // Synthetic blob: top-level `XFP: DEADBEEF` disagrees with computed
    // fingerprint `34A3A4F1` from the cosigner xpub. The parser must emit
    // the SPEC §11.4.1 row-2 WARNING (byte-exact template) AND succeed
    // (header value is authoritative per the truth table).
    let xpub_a = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
    let blob = format!(
        "Name: T\n\
Policy: 1 of 1\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
XFP: DEADBEEF\n\
\n\
{xpub_a}\n"
    );
    let out = run_stdin(&blob).success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // SPEC §11.4.1 row-2 WARNING template (verbatim):
    //   warning: import-wallet: coldcard-multisig: xfp header `XFP: <hex>`
    //   disagrees with computed fingerprint `<hex>` from cosigner xpub;
    //   using blob-supplied header value as authoritative
    assert!(
        stderr.contains("warning: import-wallet: coldcard-multisig: xfp header"),
        "row-2 WARNING template missing; got: {stderr}"
    );
    assert!(
        stderr.contains("`XFP: DEADBEEF`"),
        "WARNING must cite blob's XFP header value; got: {stderr}"
    );
    assert!(
        stderr.contains("`34A3A4F1`"),
        "WARNING must cite computed fingerprint; got: {stderr}"
    );
    assert!(
        stderr.contains("using blob-supplied header value as authoritative"),
        "WARNING must cite authoritative clause; got: {stderr}"
    );
    // Parse still succeeds (row 2 uses header value).
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
}

#[test]
fn coldcard_ms_per_cosigner_xfp_divergence_warns_per_cosigner() {
    // Synthetic blob: per-cosigner `<XFP>: <xpub>` form where the XFP
    // prefix CAFEBABE disagrees with computed `34A3A4F1`. Same WARNING
    // template applies (the truth table is per-cosigner, not just per-header).
    let xpub_a = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
    let blob = format!(
        "Name: T\n\
Policy: 1 of 1\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
CAFEBABE: {xpub_a}\n"
    );
    let out = run_stdin(&blob).success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("warning: import-wallet: coldcard-multisig: xfp header"));
    assert!(stderr.contains("CAFEBABE"));
    assert!(stderr.contains("34A3A4F1"));
}

// ============================================================================
// SPEC §6.1 — format mismatch (explicit --format vs sniffed)
// ============================================================================

#[test]
fn coldcard_ms_format_mismatch_against_bsms_blob_rejected() {
    // BSMS blob with `--format coldcard-multisig` must surface
    // ImportWalletFormatMismatch (exit 1) per SPEC §6.1.
    let bsms_blob = "BSMS 1.0\nwsh(sortedmulti(2,[deadbeef/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,[cafebabe/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*))#00000000\n";
    let out = run_stdin(bsms_blob).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard-multisig") && stderr.contains("bsms"),
        "stderr must cite mismatch between coldcard-multisig (supplied) and bsms (sniffed); \
         got: {stderr}"
    );
}

#[test]
fn coldcard_ms_format_mismatch_against_bitcoin_core_blob_rejected() {
    let core_blob = r#"{"wallet_name":"x","descriptors":[{"desc":"wpkh([deadbeef/84'/0'/0']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#00000000"}]}"#;
    let out = run_stdin(core_blob).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard-multisig") && stderr.contains("bitcoin-core"),
        "stderr must cite mismatch; got: {stderr}"
    );
}

// ============================================================================
// --json envelope shape for coldcard-multisig
// ============================================================================

#[test]
fn coldcard_ms_json_envelope_emits_canonical_shape() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            FIX_2OF3_NO_XFP,
            "--format",
            "coldcard-multisig",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("envelope is JSON");
    let arr = v.as_array().expect("top-level envelope is an array");
    assert_eq!(arr.len(), 1, "single multisig wallet → single envelope");
    let env = &arr[0];
    assert_eq!(env["schema_version"], "1");
    assert_eq!(env["source_format"], "coldcard-multisig");
    let roundtrip = &env["roundtrip"];
    assert_eq!(
        roundtrip["status"], "ok",
        "canonicalize succeeds on the no-xfp-header fixture"
    );
    assert_eq!(
        roundtrip["semantic_match"], true,
        "semantic_match always true on canonicalize-ok"
    );
}

// ============================================================================
// SPEC §6.2 — auto-sniff dispatch via sniff_format → SniffOutcome::ColdcardMultisig
// ============================================================================

#[test]
fn coldcard_ms_autosniff_does_not_co_fire_with_bsms_or_core() {
    // Sanity: the 4 fixtures all auto-sniff to `coldcard-multisig` (not
    // `Ambiguous`, not `NoMatch`). This is a per-parser sniff signature
    // smoke; the cross-format-matrix Phase P11 will exhaustively verify.
    for fix in [FIX_2OF3_WITH_XFP, FIX_2OF3_NO_XFP, FIX_3OF5] {
        let _out = run_autosniff(fix).success();
    }
}
