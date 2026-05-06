//! v0.4.1 Phase H.5 — unified `--slot @N.<subkey>=<value>` dispatch
//! integration tests. Exercises the bundle_run_unified path through the
//! actual binary.

use assert_cmd::Command;
use serde_json::Value;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

// Trezor canonical 24-word seed → BIP-84 mainnet account-0 xpub.
const TREZOR_BIP84_XPUB: &str = "zpub6jftahH18ngZxRZS6QPhWZAjzmK3HQjJfPZG7HzgaXwj1S3MaSCZmCsX8s8Pn7Z5XZ2D8wgXjYtAS65g7HFwy3WL6vSXKM4UCEMEnzXAtQF";
const TREZOR_FP_HEX: &str = "5436d724";

#[test]
fn unified_slot_phrase_singlesig_full_emits_schema_4() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={}", TREZOR_24),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["schema_version"], "4");
    assert_eq!(v["mode"], "full");
    assert_eq!(v["template"], "bip84");
    // SPEC §5.8: ms1 is MsField (length-1 array for single-sig full).
    assert!(v["ms1"].is_array());
    assert_eq!(v["ms1"].as_array().unwrap().len(), 1);
    assert!(v["ms1"][0].as_str().unwrap().starts_with("ms1"));
    assert!(v["mk1"].is_array());
}

#[test]
fn unified_slot_missing_template_or_descriptor_rejected() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={}", TREZOR_24),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("missing --template or --descriptor")
            || stderr.contains("required unless")
            || stderr.contains("--template"),
        "stderr should mention missing template; got: {stderr}"
    );
}

#[test]
fn unified_slot_unsupported_subkey_shape_rejected_with_followup_pointer() {
    // {entropy} alone is in the SPEC §6.6.b validity matrix but v0.4.1
    // unified resolution defers it to v0.4.2 per FOLLOWUP.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            "@0.entropy=00000000000000000000000000000000",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("v0.4.1 unified --slot dispatch supports") || stderr.contains("v0.4.2"),
        "stderr should reference v0.4.1/v0.4.2 deferral; got: {stderr}"
    );
}

// Smoke check: --slot still triggers the unified path even with row-6 conflict
// (--phrase + --slot @0.phrase=) — confirms expand_legacy_to_slots fires.
#[test]
fn unified_slot_phrase_collides_with_legacy_phrase_emits_row6() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--phrase",
            TREZOR_24,
            "--slot",
            &format!("@0.phrase={}", TREZOR_24),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--phrase deprecated; cannot combine with --slot @0.phrase="),
        "stderr should match SPEC §6.6 row 6; got: {stderr}"
    );
    let _ = TREZOR_BIP84_XPUB;
    let _ = TREZOR_FP_HEX;
}
