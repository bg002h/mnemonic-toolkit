//! v0.4.3 Phase Q — `--bundle-json <file>` verify-bundle JSON intake tests.
//! Round-trips a `bundle --json` envelope through `verify-bundle --bundle-json`
//! against the same re-derivation flag set.

use assert_cmd::Command;
use std::io::Write;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

#[test]
fn verify_bundle_via_bundle_json_schema_4_round_trip() {
    // Step 1: bundle --json → write to tmp file.
    let bundle_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--phrase",
            TREZOR_24,
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(bundle_out.get_output().stdout.clone()).unwrap();

    let tmpdir = tempfile::tempdir().unwrap_or_else(|_| {
        // Fallback: write to /tmp directly if tempfile dep unavailable.
        std::process::abort();
    });
    let path = tmpdir.path().join("bundle.json");
    std::fs::File::create(&path)
        .unwrap()
        .write_all(stdout.as_bytes())
        .unwrap();

    // Step 2: verify-bundle --bundle-json <path> with same re-derivation flags.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--phrase",
            TREZOR_24,
            "--bundle-json",
            path.to_str().unwrap(),
        ])
        .assert()
        .success();
}

#[test]
fn verify_bundle_via_bundle_json_unsupported_schema_rejected() {
    let tmpdir = tempfile::tempdir().unwrap();
    let path = tmpdir.path().join("schema3.json");
    // Hand-crafted minimal schema-3 fixture (the v0.3 envelope shape with
    // ms1 as flat string, schema_version "3"). v0.4.3 must reject.
    let schema3_json = r#"{"schema_version":"3","mode":"full","network":"mainnet","template":"bip84","ms1":"ms10entrsqqqq","mk1":["mk1qstub"],"md1":["md1zstub"]}"#;
    std::fs::File::create(&path)
        .unwrap()
        .write_all(schema3_json.as_bytes())
        .unwrap();

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--phrase",
            TREZOR_24,
            "--bundle-json",
            path.to_str().unwrap(),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("schema_version 3 not supported in v0.4.3")
            && stderr.contains("bundle-json-schema-2-3-retro-compat"),
        "stderr should reject schema-3 with v0.4.4+ FOLLOWUP pointer; got: {stderr}"
    );
}

#[test]
fn verify_bundle_bundle_json_conflicts_with_ms1() {
    // clap should reject --bundle-json + --ms1 simultaneously.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--phrase",
            TREZOR_24,
            "--bundle-json",
            "/dev/null",
            "--ms1",
            "ms1stub",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("cannot be used with") || stderr.contains("conflicts") || stderr.contains("--bundle-json"),
        "clap should reject conflict; got: {stderr}"
    );
}
