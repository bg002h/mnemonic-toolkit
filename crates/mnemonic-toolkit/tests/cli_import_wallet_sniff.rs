//! Phase 5 — `mnemonic import-wallet` sniff dispatch + `--json` envelope.
//!
//! Per `design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md` §5.2-§5.6 +
//! §5.12, plus the `--json` envelope cells deferred from Phase 4 (§4.9 +
//! §7.4 round-trip field).
//!
//! ## Sniff cells (SPEC §6)
//!
//! - `sniff_bsms_2line_detected` — pass blob without `--format`; assert
//!   exit 0 + correct BSMS parse.
//! - `sniff_core_descriptors_detected` — Core listdescriptors fixture
//!   without `--format`; assert exit 0 + Core parse.
//! - `sniff_ambiguous_with_specter_markers` — JSON containing both
//!   `descriptors` array AND `chain` vendor marker; assert exit 1
//!   "could not detect format" (Core sniff returns false; BSMS sniff
//!   returns false; verdict is NoMatch per SPEC §6.1.2 conservative rule).
//! - `sniff_format_mismatch_explicit_override` — `--format bsms` for a
//!   Core blob; assert exit 1 `ImportWalletFormatMismatch`.
//! - `sniff_no_match_random_text` — random text without `--format`;
//!   assert exit 1 + stderr "could not detect format".
//! - `sniff_path_roundtrip` — invoke without `--format` on the user's
//!   flagship BSMS fixture; assert end-to-end success.
//!
//! ## `--json` envelope cells (SPEC §7.4)
//!
//! - `sniff_json_envelope_includes_roundtrip_field` — invoke with
//!   `--json --format bitcoin-core`; parse stdout JSON; assert top-level
//!   item contains a `roundtrip` object with `byte_exact`,
//!   `semantic_match`, `diff` fields.
//! - `sniff_json_envelope_silences_stderr_diff` — invoke with `--json`;
//!   assert NO stderr diff text (envelope-only per SPEC §7.4).
//! - `sniff_no_auto_repair_flag_accepted` — invoke `--no-auto-repair
//!   --format bsms <flagship>`; assert exit 0 + flag is a recognized
//!   no-op.

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
// Sniff cells (SPEC §6)
// ============================================================================

#[test]
fn sniff_bsms_2line_detected() {
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out = run_import(&["--blob", p.to_str().unwrap()]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cosigners=2"),
        "sniff should route to BSMS parser; stdout: {stdout}"
    );
}

#[test]
fn sniff_core_descriptors_detected() {
    let p = fixture_path("core-bip84-mainnet.json");
    let out = run_import(&["--blob", p.to_str().unwrap()]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("cosigners=1"),
        "sniff should route to Bitcoin Core parser; stdout: {stdout}"
    );
}

#[test]
fn sniff_ambiguous_with_specter_markers() {
    // SPEC §6.1.2: presence of vendor-marker key (`chain`) makes the
    // Core sniff conservative-refuse → NoMatch. BSMS sniff also rejects
    // (no `BSMS 1.0\n` prefix). User must supply `--format` explicitly.
    let blob = r#"{"chain":"main","descriptors":[{"desc":"wpkh([deadbeef/84'/0'/0']xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx/0/*)#aaaaaaaa"}]}"#;
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-"])
        .write_stdin(blob.to_string())
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("could not detect format"),
        "expected NoMatch error template, stderr: {stderr}"
    );
}

#[test]
fn sniff_format_mismatch_explicit_override() {
    let p = fixture_path("core-bip84-mainnet.json");
    let assertion = run_import(&["--blob", p.to_str().unwrap(), "--format", "bsms"])
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--format bsms supplied but blob looks like bitcoin-core"),
        "expected ImportWalletFormatMismatch template, stderr: {stderr}"
    );
}

#[test]
fn sniff_no_match_random_text() {
    let blob = "not a wallet blob at all\nfoo bar baz\n";
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-"])
        .write_stdin(blob.to_string())
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("could not detect format"),
        "expected NoMatch template, stderr: {stderr}"
    );
}

#[test]
fn sniff_path_roundtrip() {
    // End-to-end exercise: user passes the decaying-multisig 32768
    // fixture (their flagship blob) WITHOUT `--format`. Sniff must route
    // to BSMS and parse cleanly.
    let p = fixture_path("bsms_2line_decaying_multisig_32768.txt");
    let out = run_import(&["--blob", p.to_str().unwrap()]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // The decaying-multisig flagship is testnet (path m/48'/1'/...);
    // sniff must still parse it cleanly end-to-end.
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(
        stdout.contains("network=testnet") || stdout.contains("network=mainnet"),
        "stdout: {stdout}"
    );
}

#[test]
fn sniff_explicit_format_honored_when_blob_has_vendor_markers() {
    // SPEC §6.2: if the blob is unsniffable (NoMatch — e.g., due to
    // conservative vendor-marker rejection) but the user supplied an
    // explicit `--format`, the explicit format is honored and we
    // attempt to parse. Here the blob has a `chain` vendor marker
    // (Specter-shape) that the sniff conservatively rejects; the user
    // overrides with `--format bitcoin-core` and the Core parser
    // accepts the envelope-shape (which IS the `{ descriptors: [...] }`
    // form Core expects).
    let blob = r#"{"chain":"main","descriptors":[{"desc":"wpkh([b8688df1/84'/0'/0']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#5ql5mvwg","active":true,"internal":false}]}"#;
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "bitcoin-core"])
        .write_stdin(blob.to_string())
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
}

// ============================================================================
// `--json` envelope cells (SPEC §7.4)
// ============================================================================

#[test]
fn sniff_json_envelope_includes_roundtrip_field() {
    let p = fixture_path("core-bip84-mainnet.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "bitcoin-core",
        "--json",
    ])
    .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("invalid JSON: {e}\nstdout was:\n{stdout}"));
    let arr = val.as_array().expect("--json must emit a top-level array");
    assert_eq!(arr.len(), 1, "expected one envelope");
    let env = &arr[0];
    let rt = env
        .get("roundtrip")
        .expect("envelope must contain `roundtrip` field");
    assert!(
        rt.get("byte_exact").is_some(),
        "roundtrip.byte_exact missing"
    );
    assert!(
        rt.get("semantic_match").is_some(),
        "roundtrip.semantic_match missing"
    );
    assert!(rt.get("diff").is_some(), "roundtrip.diff missing");
    assert_eq!(
        env.get("source_format").and_then(|v| v.as_str()),
        Some("bitcoin-core")
    );
    let bundle = env.get("bundle").expect("envelope must contain `bundle`");
    let cosigners = bundle
        .get("cosigners")
        .and_then(|c| c.as_array())
        .expect("bundle.cosigners must be array");
    assert_eq!(cosigners.len(), 1);
}

#[test]
fn sniff_json_envelope_bsms_blocked_no_emitter() {
    // SPEC §7.3.1 policy: BSMS export emitter is FOLLOWUP work; the
    // envelope must non-misleadingly signal this via
    // `roundtrip.status: "blocked_no_emitter"`.
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let out = run_import(&["--blob", p.to_str().unwrap(), "--format", "bsms", "--json"]).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let env = &val.as_array().unwrap()[0];
    let rt = env.get("roundtrip").unwrap();
    assert_eq!(
        rt.get("status").and_then(|v| v.as_str()),
        Some("blocked_no_emitter"),
        "BSMS envelope must signal no emitter; rt: {rt:?}"
    );
    assert_eq!(rt.get("byte_exact").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(
        rt.get("semantic_match").and_then(|v| v.as_bool()),
        Some(false)
    );
}

#[test]
fn sniff_json_envelope_silences_stderr_diff() {
    // SPEC §7.4: when `--json` is set, the diff goes ONLY in the
    // envelope; stderr is silent for the diff. Other stderr advisories
    // (NOTICEs about dropped wallet-state fields) are still allowed.
    let p = fixture_path("core-bip84-mainnet.json");
    let out = run_import(&[
        "--blob",
        p.to_str().unwrap(),
        "--format",
        "bitcoin-core",
        "--json",
    ])
    .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // The "roundtrip not byte-exact" stderr WARNING must NOT appear
    // when --json is set; the diff is in the envelope instead.
    assert!(
        !stderr.contains("roundtrip not byte-exact"),
        "stderr should NOT carry the round-trip diff WARNING under --json; stderr: {stderr}"
    );
}

#[test]
fn sniff_no_auto_repair_flag_accepted() {
    // --no-auto-repair is a global toolkit flag, recognized by import-
    // wallet for symmetry with verify-bundle / convert / inspect. In
    // v0.26.0 it is a documented no-op for import-wallet (FOLLOWUP
    // `wallet-import-bch-correctable-fields-v0_27` will revisit). The
    // surface gate here is just "the flag is accepted, exit 0 follows".
    let p = fixture_path("bsms-2line-sortedmulti-2of2.txt");
    let _out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["--no-auto-repair", "import-wallet", "--blob"])
        .arg(&p)
        .args(["--format", "bsms"])
        .assert()
        .success();
}
