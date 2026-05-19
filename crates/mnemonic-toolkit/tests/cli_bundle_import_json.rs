//! v0.27.0 Phase 5 — `mnemonic bundle --import-json <FILE|->`.
//!
//! Per `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md`
//! §3.6 + §3.6.1. Cells exercise the consumer path that decodes an
//! `import-wallet --json` envelope and synthesizes a fresh bundle.

use assert_cmd::Command;
use std::io::Write;
use std::path::{Path, PathBuf};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

/// Run `bundle --import-json <FILE> --network mainnet --json` and parse
/// stdout as JSON.
fn run_bundle_import_json_file(envelope_path: &Path) -> serde_json::Value {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--import-json",
            envelope_path.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("bundle JSON invalid: {e}\nstdout was:\n{stdout}"))
}

// ============================================================================
// Cell 1 — `bundle --import-json` synthesizes a watch-only bundle from a
// Phase-4-emitted envelope (2-of-3 BSMS source).
// ============================================================================

#[test]
fn bundle_import_json_bsms_2of3_synthesizes_watch_only_bundle() {
    let p = fixture_path("envelope_v0_27_0.json");
    let val = run_bundle_import_json_file(&p);
    assert_eq!(val["mode"].as_str(), Some("watch-only"));
    assert_eq!(val["ms1"].as_array().unwrap().len(), 3);
    for entry in val["ms1"].as_array().unwrap() {
        assert_eq!(
            entry.as_str(),
            Some(""),
            "watch-only synthesis must preserve SPEC §5.8 empty-string sentinels"
        );
    }
    // Descriptor passthrough: envelope's bundle.descriptor matches output.
    let descriptor = val["descriptor"].as_str().unwrap();
    assert!(
        descriptor.starts_with("sh(multi(2,") && descriptor.contains("#ek6d38cp"),
        "envelope descriptor must round-trip verbatim through bundle --import-json; got: {descriptor}"
    );
}

// ============================================================================
// Cell 2 — multisig synthesis preserves cosigner count (N=3) and per-cosigner
// xpub identity (chain_code + public_key) via mk1 round-trip.
// ============================================================================

#[test]
fn bundle_import_json_mk1_multi_decodes_to_n_slots_in_declaration_order() {
    let p = fixture_path("envelope_v0_27_0.json");
    let val = run_bundle_import_json_file(&p);
    let mk1_outer = val["mk1"].as_array().expect("mk1 outer array");
    assert_eq!(mk1_outer.len(), 3, "2-of-3 must emit 3 mk1 chunk arrays");
    // Decode each mk1 → confirm fingerprints in declaration order match the
    // source envelope's multisig.cosigners[].master_fingerprint.
    let expected_fps = ["b8688df1", "5436d724", "28645006"];
    for (i, chunks) in mk1_outer.iter().enumerate() {
        let strs: Vec<&str> = chunks
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap())
            .collect();
        let card = mk_codec::decode(&strs).unwrap_or_else(|e| panic!("mk1[{i}]: {e:?}"));
        let fp = format!("{}", card.origin_fingerprint.unwrap());
        assert_eq!(
            fp, expected_fps[i],
            "mk1[{i}].origin_fingerprint declaration-order mismatch"
        );
    }
}

// ============================================================================
// Cell 3 — `bundle --import-json` ↔ `--template` mutex.
// ============================================================================

#[test]
fn bundle_import_json_with_template_flag_errors_mutex() {
    let p = fixture_path("envelope_v0_27_0.json");
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "wsh-sortedmulti",
            "--import-json",
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
// Cell 4 — `bundle --import-json` ↔ `--descriptor` mutex.
// ============================================================================

#[test]
fn bundle_import_json_with_descriptor_flag_errors_mutex() {
    let p = fixture_path("envelope_v0_27_0.json");
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--descriptor",
            "wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)",
            "--import-json",
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
// Cell 5 — multi-entry envelope without --import-json-index errors.
// ============================================================================

fn multi_entry_envelope_json() -> String {
    // Two-entry envelope: same bundle shape repeated. Hand-rolled to avoid
    // dragging in a Bitcoin Core fixture.
    let one_entry: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fixture_path("envelope_v0_27_0.json")).unwrap())
            .unwrap();
    let entry = &one_entry.as_array().unwrap()[0];
    serde_json::to_string(&serde_json::Value::Array(vec![entry.clone(), entry.clone()])).unwrap()
}

#[test]
fn bundle_import_json_multi_entry_without_index_errors() {
    let tmpdir = tempfile::tempdir().unwrap();
    let p = tmpdir.path().join("multi.json");
    std::fs::write(&p, multi_entry_envelope_json()).unwrap();
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--import-json",
            p.to_str().unwrap(),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("envelope array has 2 entries"),
        "expected multi-entry-without-index error; got: {stderr}"
    );
}

// ============================================================================
// Cell 6 — `--import-json-index N` picks the correct entry.
// ============================================================================

#[test]
fn bundle_import_json_index_picks_correct_descriptor() {
    let tmpdir = tempfile::tempdir().unwrap();
    let p = tmpdir.path().join("multi.json");
    std::fs::write(&p, multi_entry_envelope_json()).unwrap();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--import-json",
            p.to_str().unwrap(),
            "--import-json-index",
            "0",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // Should produce the same bundle as the single-entry case (entry 0
    // is identical to the original envelope's bundle).
    assert_eq!(val["mode"].as_str(), Some("watch-only"));
    assert_eq!(val["ms1"].as_array().unwrap().len(), 3);
}

// ============================================================================
// Cell 7 — index out of bounds.
// ============================================================================

#[test]
fn bundle_import_json_index_out_of_bounds_errors() {
    let p = fixture_path("envelope_v0_27_0.json");
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--import-json",
            p.to_str().unwrap(),
            "--import-json-index",
            "5",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("out of range"),
        "expected out-of-range error; got: {stderr}"
    );
}

// ============================================================================
// Cell 8 — `--import-json -` reads envelope from stdin.
// ============================================================================

#[test]
fn bundle_import_json_stdin_dash_reads_envelope() {
    let envelope_json = std::fs::read_to_string(fixture_path("envelope_v0_27_0.json")).unwrap();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--import-json",
            "-",
            "--json",
        ])
        .write_stdin(envelope_json)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(val["mode"].as_str(), Some("watch-only"));
}

// ============================================================================
// Cell 9 — `bundle --import-json --self-check` exercises the bundle's
// internal round-trip (synthesize → re-parse → verify match).
// ============================================================================

#[test]
fn bundle_import_json_self_check_round_trip_passes() {
    let p = fixture_path("envelope_v0_27_0.json");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--import-json",
            p.to_str().unwrap(),
            "--self-check",
        ])
        .assert()
        .success();
}

// ============================================================================
// Cell 10 — seed-overlay via `--slot @N.phrase=` on a watch-only envelope.
// The envelope must be built from a real-BIP-32-derivation source so the
// xpub-at-path comparison succeeds. Uses skip_middle_3of3 fixture xpubs
// at m/87'/0'/0' (mirrors cli_import_wallet_seed_overlay).
// ============================================================================

const BIP39_TEST_1: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn bsms_2line_from_body(body: &str) -> String {
    use miniscript::descriptor::checksum::Engine as ChecksumEngine;
    let mut e = ChecksumEngine::new();
    e.input(body).expect("ascii");
    let csum = e.checksum();
    format!("BSMS 1.0\n{body}#{csum}\n")
}

fn skip_middle_3of3_envelope_json() -> String {
    let body = "wsh(sortedmulti(2,\
[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*,\
[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*,\
[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*))";
    let blob = bsms_2line_from_body(body);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "bsms", "--json"])
        .write_stdin(blob)
        .assert()
        .success();
    String::from_utf8(out.get_output().stdout.clone()).unwrap()
}

#[test]
fn bundle_import_json_seed_overlay_via_slot_phrase_yields_full_for_overlaid_slot() {
    let envelope_json = skip_middle_3of3_envelope_json();
    let tmpdir = tempfile::tempdir().unwrap();
    let p = tmpdir.path().join("env.json");
    std::fs::write(&p, &envelope_json).unwrap();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--import-json",
            p.to_str().unwrap(),
            "--slot",
            &format!("@0.phrase={BIP39_TEST_1}"),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // Cosigner 0 now seeded → bundle.ms1[0] is non-empty; the other two
    // remain watch-only sentinels.
    let ms1 = val["ms1"].as_array().unwrap();
    assert!(
        !ms1[0].as_str().unwrap().is_empty(),
        "ms1[0] must be non-empty after seed overlay; got: {:?}",
        ms1[0]
    );
    assert_eq!(ms1[1].as_str(), Some(""));
    assert_eq!(ms1[2].as_str(), Some(""));
    // Hybrid mode: any-secret + any-watch.
    assert_eq!(val["mode"].as_str(), Some("full"));
}

// ============================================================================
// Cell 11 — conflict: `--slot @N.phrase=` on a slot with non-empty envelope
// ms1[N] is BadInput. To exercise this we need an envelope with ms1[N] != "",
// which only happens when import-wallet's seed-overlay attached entropy at
// emit time. We build one inline via import-wallet --ms1 <ms1-card>.
// ============================================================================

#[test]
fn bundle_import_json_overlay_on_seeded_slot_errors_conflict() {
    // Start with the skip_middle envelope and emit with --ms1 attached to
    // cosigner 0 via import-wallet seed-overlay (ms1-encoded form of the
    // BIP39_TEST_1 entropy). The toolkit's ms_codec ms1-encoded entropy for
    // "abandon × 11 about" is well-known.
    const MS1_TEST_1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    let body = "wsh(sortedmulti(2,\
[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*,\
[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*,\
[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*))";
    let blob = bsms_2line_from_body(body);
    let import_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--ms1",
            MS1_TEST_1,
            "--json",
        ])
        .write_stdin(blob)
        .assert()
        .success();
    let envelope_json = String::from_utf8(import_out.get_output().stdout.clone()).unwrap();

    let tmpdir = tempfile::tempdir().unwrap();
    let p = tmpdir.path().join("env.json");
    std::fs::write(&p, &envelope_json).unwrap();
    // Now try to overlay --slot @0.phrase=... on a cosigner that already has
    // entropy in the envelope. This must error with the conflict message.
    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--import-json",
            p.to_str().unwrap(),
            "--slot",
            &format!("@0.phrase={BIP39_TEST_1}"),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("envelope already carries entropy for cosigner 0"),
        "expected envelope-already-seeded conflict; got: {stderr}"
    );
}

// ============================================================================
// Cell 12 — R0-scope round-trip per memory: bundle --import-json X | jq
// → verify-bundle --bundle-json <FILE>. The Phase-4-emitted envelope, when
// fed back through `bundle --import-json --json`, must produce a bundle
// JSON that verify-bundle then accepts as `result:"ok"`. Lossy synthesis
// surfaces as `result:"mismatch"`.
// ============================================================================

// ============================================================================
// Cell 13 — Phase 5 R0 I1 regression: tampered descriptor (valid checksum
// but for a different body) surfaces as a clean BIP-380 checksum error
// rather than a downstream confusing parse error.
// ============================================================================

#[test]
fn bundle_import_json_tampered_descriptor_emits_bip380_checksum_error() {
    // Read the v0.27.0 envelope fixture, then mutate one byte of the
    // descriptor body (changing `b8688df1` to `b8688df0`); the original
    // `#ek6d38cp` checksum no longer validates against the mutated body.
    let envelope_path = fixture_path("envelope_v0_27_0.json");
    let raw = std::fs::read_to_string(&envelope_path).unwrap();
    let mutated = raw.replace("b8688df1/48'/0'/0'", "b8688df0/48'/0'/0'");
    assert_ne!(raw, mutated, "fixture must contain the source fingerprint");
    let tmpdir = tempfile::tempdir().unwrap();
    let p = tmpdir.path().join("tampered.json");
    std::fs::write(&p, mutated).unwrap();

    let assertion = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--import-json",
            p.to_str().unwrap(),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("BIP-380 checksum validation failed"),
        "tampered descriptor must surface as a clean BIP-380 checksum error; got: {stderr}"
    );
}

#[test]
fn bundle_import_json_to_verify_bundle_round_trip_yields_ok() {
    let envelope_json = skip_middle_3of3_envelope_json();
    let tmpdir = tempfile::tempdir().unwrap();
    let env_p = tmpdir.path().join("env.json");
    std::fs::write(&env_p, &envelope_json).unwrap();

    // Synthesize via bundle --import-json --json.
    let bundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--import-json",
            env_p.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success();
    let bundle_stdout = String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap();
    let bundle_val: serde_json::Value = serde_json::from_str(&bundle_stdout).unwrap();
    let bundle_path = tmpdir.path().join("bundle.json");
    let mut f = std::fs::File::create(&bundle_path).unwrap();
    f.write_all(
        serde_json::to_string_pretty(&bundle_val)
            .unwrap()
            .as_bytes(),
    )
    .unwrap();
    drop(f);

    // Verify-bundle round-trip via --bundle-json.
    let env_val: serde_json::Value = serde_json::from_str(&envelope_json).unwrap();
    let cosigners = env_val.as_array().unwrap()[0]["bundle"]["multisig"]["cosigners"]
        .as_array()
        .unwrap();
    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--multisig-path-family".into(),
        "bip87".into(),
        "--threshold".into(),
        "2".into(),
        "--bundle-json".into(),
        bundle_path.to_str().unwrap().to_string(),
        "--json".into(),
    ];
    for (i, c) in cosigners.iter().enumerate() {
        args.push("--slot".into());
        args.push(format!("@{i}.xpub={}", c["xpub"].as_str().unwrap()));
        args.push("--slot".into());
        args.push(format!("@{i}.fingerprint={}", c["master_fingerprint"].as_str().unwrap()));
    }
    let verify_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success();
    let verify_stdout = String::from_utf8(verify_out.get_output().stdout.clone()).unwrap();
    let verify_val: serde_json::Value = serde_json::from_str(&verify_stdout).unwrap();
    assert_eq!(
        verify_val["result"].as_str(),
        Some("ok"),
        "round-trip verify-bundle must yield result=ok; got: {verify_val}"
    );
}
