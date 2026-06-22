//! v0.28.0 Phase P5C — Blockstream Jade wallet-import integration tests.
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.5. Tests the library
//! boundary via the CLI scaffold (`cmd/import_wallet.rs`) extended in P5C
//! to dispatch `--format jade` to `JadeParser` and to wire the auto-sniff
//! path for the Jade JSON wrapper shape.
//!
//! Self-contained: no dependency on adjacent repos or external network.
//! Uses the 3 fixtures introduced in P5B:
//! - `jade-multisig-2of3-p2wsh.json` — happy-path 2-of-3 P2WSH (Jade
//!   wrapper around the Coldcard-multisig text).
//! - `jade-singlesig-refused.json` — bogus singlesig fragment; refused
//!   via the delegated coldcard-multisig parser.
//! - `jade-malformed-json.json` — invalid JSON top-level; refused with
//!   `jade`-citing diagnostic.

use assert_cmd::Command;

const FIX_2OF3: &str = "tests/fixtures/wallet_import/jade-multisig-2of3-p2wsh.json";
const FIX_SINGLESIG: &str = "tests/fixtures/wallet_import/jade-singlesig-refused.json";
const FIX_MALFORMED: &str = "tests/fixtures/wallet_import/jade-malformed-json.json";

/// Run `mnemonic import-wallet --blob <path> --format jade`.
fn run_explicit(path: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", path, "--format", "jade"])
        .assert()
}

/// Run `mnemonic import-wallet --blob <path>` (no `--format`); auto-sniff path.
fn run_autosniff(path: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", path])
        .assert()
}

/// Run `mnemonic import-wallet --blob <path> --format jade --json`.
fn run_explicit_json(path: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            path,
            "--format",
            "jade",
            "--json",
        ])
        .assert()
}

// ============================================================================
// §11.5 — happy-path parse via explicit --format jade
// ============================================================================

#[test]
fn jade_2of3_happy_path_explicit_format() {
    let out = run_explicit(FIX_2OF3).success();
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
    // Jade provenance is neither bsms nor bitcoin-core; the
    // `source_metadata` accessor returns None per ImportProvenance semantics.
    assert!(
        stdout.contains("bundles[0].source_metadata=none"),
        "stdout: {stdout}"
    );
    // Silent stderr on happy-path (XFP truth-table row 1 across all cosigners).
    assert!(
        !stderr.contains("warning:"),
        "stderr must be silent on happy-path; got: {stderr}"
    );
}

// ============================================================================
// §11.5 — auto-sniff: JSON blob with top-level `multisig_file` → Jade
// ============================================================================

#[test]
fn jade_autosniff_2of3_dispatches_correctly() {
    let out = run_autosniff(FIX_2OF3).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(
        stdout.contains("bundles[0].cosigners=3"),
        "stdout: {stdout}"
    );
}

#[test]
fn jade_autosniff_does_not_co_fire_with_coldcard_multisig() {
    // The inner Coldcard-multisig text IS a valid Coldcard-multisig
    // blob in isolation. But wrapping it inside a JSON `{multisig_file: ...}`
    // envelope changes the sniff dispatch: the OUTER JSON sniffs as Jade
    // (top-level `multisig_file` field present) and NOT as
    // ColdcardMultisig (which sniffs on bare text-shape header lines).
    // This is the load-bearing disambiguation per SPEC §11.5 — the Jade
    // wrapper must NOT trigger Ambiguous.
    let out = run_autosniff(FIX_2OF3).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stdout.contains("bundles=1"));
    // No "ambiguous" / "matches multiple format heuristics" template.
    assert!(
        !stderr.to_lowercase().contains("ambiguous"),
        "stderr must not cite ambiguity; got: {stderr}"
    );
}

// ============================================================================
// §11.5 — refusal on singlesig-shaped Jade wrapper (delegated parser fails)
// ============================================================================

#[test]
fn jade_singlesig_fixture_refused() {
    let out = run_explicit(FIX_SINGLESIG).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // Delegated coldcard-multisig parser surfaces its native error
    // template (missing `Policy:` header). This is by design — the user
    // sees the underlying §11.4 diagnostic verbatim.
    assert!(
        stderr.contains("coldcard-multisig") || stderr.contains("Policy"),
        "delegated parser error must surface; got: {stderr}"
    );
}

// ============================================================================
// §11.5 — refusal on malformed JSON (sniff fails OR parse fails)
// ============================================================================

#[test]
fn jade_malformed_json_explicit_format_refused() {
    let out = run_explicit(FIX_MALFORMED).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("jade") && stderr.contains("JSON"),
        "stderr must cite jade format + JSON; got: {stderr}"
    );
}

#[test]
fn jade_malformed_json_autosniff_nomatch() {
    // Without explicit --format, the malformed JSON sniffs as NoMatch
    // (Jade sniff returns false on invalid JSON). The user-facing error
    // is the "could not detect format" template — not a jade-specific
    // diagnostic.
    let out = run_autosniff(FIX_MALFORMED).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("could not detect format") || stderr.contains("ambiguous"),
        "stderr should cite sniff-failure template; got: {stderr}"
    );
}

// ============================================================================
// §11.5 — JSON envelope shape (P5C wiring)
// ============================================================================

#[test]
fn jade_json_envelope_emits_canonical_shape() {
    let out = run_explicit_json(FIX_2OF3).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();

    // Envelope carries the per-format `jade_source_metadata` field
    // (vs. coldcard_source_metadata / electrum_source_metadata /
    // sparrow_source_metadata / specter_source_metadata for the other
    // formats).
    assert!(
        stdout.contains("\"jade_source_metadata\""),
        "envelope must carry jade_source_metadata field; got: {stdout}"
    );
    // The Coldcard-compat sub-object carries the SPEC §11.4 telemetry
    // verbatim.
    assert!(
        stdout.contains("\"coldcard_compat\""),
        "jade_source_metadata.coldcard_compat must be present; got: {stdout}"
    );
    // `jade_specific_fields` is empty at v0.28.0 (SeedQR deferred).
    assert!(
        stdout.contains("\"jade_specific_fields\": []"),
        "jade_specific_fields must be empty array at v0.28.0; got: {stdout}"
    );
    // source_format pinned at "jade" (NOT "coldcard-multisig" — load-bearing
    // distinction per SPEC §11.5).
    assert!(
        stdout.contains("\"source_format\": \"jade\""),
        "envelope must source_format=jade; got: {stdout}"
    );
    // schema_version pinned at "1".
    assert!(
        stdout.contains("\"schema_version\": \"1\""),
        "envelope must schema_version=\"1\"; got: {stdout}"
    );
    // roundtrip envelope is now real (P5C wiring).
    assert!(
        stdout.contains("\"status\": \"ok\""),
        "roundtrip status must be ok; got: {stdout}"
    );
    assert!(
        stdout.contains("\"semantic_match\": true"),
        "roundtrip semantic_match must be true; got: {stdout}"
    );
}

#[test]
fn jade_provenance_distinct_from_coldcard_multisig() {
    // Load-bearing distinction per SPEC §11.5: the Jade ingest path
    // produces `source_format: "jade"` in the envelope — NOT
    // `coldcard-multisig` — even though the inner parse delegates.
    // The user-facing wire-shape must surface the actual format the
    // user supplied (Jade), not the implementation-detail delegate.
    let out = run_explicit_json(FIX_2OF3).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("\"source_format\": \"jade\""));
    // Negative assertion: source_format is NOT coldcard-multisig.
    assert!(
        !stdout.contains("\"source_format\": \"coldcard-multisig\""),
        "source_format must NOT be coldcard-multisig — Jade provenance \
         is distinct per SPEC §11.5; got: {stdout}"
    );
    // The Jade envelope carries `jade_source_metadata`, NOT
    // `coldcard_source_metadata`.
    assert!(
        !stdout.contains("\"coldcard_source_metadata\""),
        "Jade envelope must NOT carry coldcard_source_metadata field; \
         got: {stdout}"
    );
}

// ============================================================================
// §11.5 — format-mismatch matrix completion (P5C)
// ============================================================================

#[test]
fn jade_format_mismatch_against_bsms_blob() {
    // Explicit `--format jade` against a BSMS blob → ImportWalletFormatMismatch.
    let bsms_path = "tests/fixtures/wallet_import/bsms-2line-sortedmulti-2of2.txt";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", bsms_path, "--format", "jade"])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("jade") && stderr.contains("bsms"),
        "stderr must cite format mismatch (supplied=jade vs sniffed=bsms); \
         got: {stderr}"
    );
}

#[test]
fn jade_format_mismatch_against_coldcard_multisig_blob() {
    // Explicit `--format jade` against a bare Coldcard-multisig blob
    // → ImportWalletFormatMismatch (the bare text is NOT wrapped in
    // Jade JSON).
    let cc_path = "tests/fixtures/wallet_import/coldcard-ms-2of3-p2wsh-with-xfp.txt";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", cc_path, "--format", "jade"])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("jade") && stderr.contains("coldcard-multisig"),
        "stderr must cite format mismatch (supplied=jade vs sniffed=coldcard-multisig); \
         got: {stderr}"
    );
}

// ============================================================================
// cycle-13a #14 (H14-b / M-1 blast-radius) — the depth-gated master-fp refusal
// fires on the JADE import surface too, since `wallet_import/jade.rs` delegates
// the inner `multisig_file` to the shared `coldcard_multisig::parse_text`.
// ============================================================================

#[test]
fn import_jade_depth_gt0_no_xfp_refuses() {
    // Jade `get_registered_multisig` reply whose inner `multisig_file` carries
    // a DEPTH-4 account xpub with NO XFP (header or per-line) → the shared
    // parser refuses (master fp unrecoverable from an account xpub), exit 2.
    let xpub_a = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
    let multisig_file = format!(
        "Name: T\nPolicy: 1 of 1\nDerivation: m/48'/0'/0'/2'\nFormat: P2WSH\n{xpub_a}\n"
    );
    let envelope = serde_json::json!({
        "id": "jade-test-depth-refuse",
        "multisig_name": "T",
        "multisig_file": multisig_file,
    })
    .to_string();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "jade"])
        .write_stdin(envelope)
        .assert()
        .failure();
    let code = out.get_output().status.code().expect("exit code present");
    assert_eq!(code, 2, "H14-b refusal must exit 2 (ImportWalletParse); got {code}");
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("depth") && stderr.contains("master fingerprint"),
        "Jade refusal must cite depth + master fingerprint (delegated H14-b); got: {stderr}"
    );
}
