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

// v0.4.2 K.1: {entropy} is now SUPPORTED via the test
// `unified_slot_entropy_singlesig_full_round_trips_against_phrase` below.
// The v0.4.1 rejection test is superseded by the round-trip-equivalence assertion.

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

// ---- v0.4.2 Phase K — additional slot subkey shapes ----

const TREZOR_24_ENTROPY_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";
// well-known compressed-pubkey WIF (Bitcoin Core test vector).
const SAMPLE_WIF: &str = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";

#[test]
fn unified_slot_entropy_singlesig_full_round_trips_against_phrase() {
    // K.1: entropy hex form must produce a byte-identical bundle to the
    // equivalent phrase form for the same underlying seed.
    let phrase_out = Command::cargo_bin("mnemonic")
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
    let phrase_stdout = String::from_utf8(phrase_out.get_output().stdout.clone()).unwrap();

    let entropy_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.entropy={}", TREZOR_24_ENTROPY_HEX),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let entropy_stdout = String::from_utf8(entropy_out.get_output().stdout.clone()).unwrap();

    assert_eq!(
        phrase_stdout, entropy_stdout,
        "entropy hex form must produce byte-identical JSON envelope to phrase form for same seed"
    );
}

#[test]
fn unified_slot_xprv_rejected_with_followup_pointer() {
    // K.2: xprv DEFERRED to v0.5+ per impl plan r1 review C-1.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            "@0.xprv=xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("not supported in v0.4.2") && stderr.contains("ms-codec XPRV-tag"),
        "xprv must be rejected with v0.5+ deferral pointer; got: {stderr}"
    );
}

#[test]
fn unified_slot_wif_singlesig_emits_valid_bundle() {
    // K.3: wif degenerate single-key in single-sig context.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.wif={}", SAMPLE_WIF),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["schema_version"], "4");
    // wif slot emits empty-string ms1 sentinel (v0.4.2 documented behavior).
    assert_eq!(v["ms1"], serde_json::json!([""]));
    assert!(v["mk1"].is_array());
}

// v0.4.3 Phase R: wif slots in multisig contexts are now SUPPORTED
// (previously rejected with v0.4.3 deferral pointer in v0.4.2).
// The v0.4.2 rejection test is replaced by 3 R tests below.

// Two distinct WIFs (Bitcoin Core compressed-pubkey test vector + a flipped
// variant). Same 32-byte private-key test vector both times would produce
// the same xpub, hence we use two genuinely-distinct WIFs to avoid
// triggering BIP-388 row 13.
const SAMPLE_WIF_2: &str = "L4rK1yDtCWekvXuE6oXD9jCYfFNV2cWRpVuPLBcCU2z8TrisoyY1";

#[test]
fn unified_slot_wif_in_multisig_2_of_3() {
    // Phase R: hybrid 2-of-3 with phrase + wif + xpub cosigners.
    let xpub = "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={}", TREZOR_24),
            "--slot",
            &format!("@1.wif={}", SAMPLE_WIF),
            "--slot",
            &format!("@2.xpub={}", xpub),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["schema_version"], "4");
    // ms1: [secret, "", ""] — only @0 is secret-bearing.
    assert!(v["ms1"][0].as_str().unwrap().starts_with("ms1"));
    assert_eq!(v["ms1"][1], "");
    assert_eq!(v["ms1"][2], "");
}

#[test]
fn unified_slot_wif_alone_in_2_of_2() {
    // Phase R: pure wif multisig (degenerate but legal). Two DISTINCT WIFs.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.wif={}", SAMPLE_WIF),
            "--slot",
            &format!("@1.wif={}", SAMPLE_WIF_2),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["schema_version"], "4");
    // Both wif slots are watch-only at the ms1 level (empty sentinels).
    assert_eq!(v["ms1"], serde_json::json!(["", ""]));
}

#[test]
fn unified_slot_same_wif_twice_emits_bip388_row13() {
    // Phase R: BIP-388 distinct-key conformance under wif multisig. Same
    // WIF supplied for @0 AND @1 → identical pubkey + empty path tuples
    // → SPEC §6.6 row 13 fires (exit 2).
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.wif={}", SAMPLE_WIF),
            "--slot",
            &format!("@1.wif={}", SAMPLE_WIF),
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: BIP-388 distinct-key violation: slot @0 and slot @1 resolve to identical (xpub, path)\n",
        "same WIF twice in multisig must trigger SPEC §6.6 row 13 byte-exactly"
    );
}

#[test]
fn unified_slot_xpub_alone_emits_partial_origin() {
    // K.4: {xpub} alone (no fingerprint, no path). Defaults applied.
    let xpub = "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={}", xpub),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["schema_version"], "4");
    assert_eq!(v["mode"], "watch-only");
    assert_eq!(v["ms1"], serde_json::json!([""]));
}

#[test]
fn unified_slot_xpub_with_fingerprint_no_path() {
    // K.4: {xpub, fingerprint} (no path). Default empty path applied.
    let xpub = "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={}", xpub),
            "--slot",
            "@0.fingerprint=deadbeef",
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["mode"], "watch-only");
    assert_eq!(v["master_fingerprint"], "deadbeef");
}

// ---- v0.4.2 Phase L — descriptor mode under unified --slot ----

#[test]
fn unified_slot_descriptor_singlesig_phrase_full() {
    let descriptor = format!("wpkh(@0[{TREZOR_FP_HEX}/84'/0'/0']/<0;1>/*)");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            &descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={}", TREZOR_24),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["schema_version"], "4");
    assert_eq!(v["mode"], "full");
    assert_eq!(v["template"], Value::Null);
    assert_eq!(v["descriptor"].as_str().unwrap(), descriptor);
    assert!(v["ms1"].is_array());
    assert!(v["ms1"][0].as_str().unwrap().starts_with("ms1"));
}

#[test]
fn unified_slot_descriptor_watch_only_xpub_via_slot() {
    let xpub = "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ";
    let descriptor = "wpkh(@0/<0;1>/*)";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={}", xpub),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["mode"], "watch-only");
    assert_eq!(v["ms1"], serde_json::json!([""]));
}

#[test]
fn unified_slot_descriptor_phrase_fingerprint_mismatch_rejected() {
    // Descriptor annotation says fingerprint deadbeef but TREZOR_24 produces
    // 5436d724 — must be rejected as cross-check failure.
    let descriptor = "wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={}", TREZOR_24),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("derives master fingerprint")
            && stderr.contains("annotation specifies"),
        "fingerprint mismatch must be reported; got: {stderr}"
    );
}
