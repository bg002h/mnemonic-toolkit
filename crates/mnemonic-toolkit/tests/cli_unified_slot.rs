//! v0.4.1 Phase H.5 — unified `--slot @N.<subkey>=<value>` dispatch
//! integration tests. Exercises the bundle_run_unified path through the
//! actual binary.

use assert_cmd::Command;
use serde_json::Value;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

const TREZOR_FP_HEX: &str = "5436d724";

#[test]
fn unified_slot_phrase_singlesig_full_emits_schema_4() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args([
            "bundle",
            "--scheme",
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
        .arg("--allow-argv-secret")
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
        stderr.contains("missing --scheme or --descriptor")
            || stderr.contains("required unless")
            || stderr.contains("--scheme"),
        "stderr should mention missing template; got: {stderr}"
    );
}

// v0.4.2 K.1: {entropy} is now SUPPORTED via the test
// `unified_slot_entropy_singlesig_full_round_trips_against_phrase` below.
// The v0.4.1 rejection test is superseded by the round-trip-equivalence assertion.

// ---- v0.4.2 Phase K — additional slot subkey shapes ----

const TREZOR_24_ENTROPY_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
// well-known compressed-pubkey WIF (Bitcoin Core test vector).
const SAMPLE_WIF: &str = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";

#[test]
fn unified_slot_entropy_singlesig_full_round_trips_against_phrase() {
    // K.1: entropy hex form must produce a byte-identical bundle to the
    // equivalent phrase form for the same underlying seed.
    let phrase_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args([
            "bundle",
            "--scheme",
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
        .arg("--allow-argv-secret")
        .args([
            "bundle",
            "--scheme",
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
        .arg("--allow-argv-secret")
        .args([
            "bundle",
            "--scheme",
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

// Bug hunt 2026-11-25 X4: `bundle --scheme` REFUSES every `--slot @N.wif=`
// at mint (exit 1). A WIF is one private key with no chain code, but every
// template derives ranged child addresses (`/<0;1>/*`); the WIF slot used to be
// bound as a depth-0 "xpub" with a zero chain code, so the card's addresses
// were not the WIF's address. These were the v0.4.2 / Phase R success cases
// (single-sig; hybrid 2-of-3; pure-WIF 2-of-2; same WIF twice); each now
// refuses before any card is emitted. Existing WIF cards still decode
// (`cli_argv_leakage::existing_wif_mk1_card_still_decodes_via_inspect`).
const SAMPLE_WIF_2: &str = "L4rK1yDtCWekvXuE6oXD9jCYfFNV2cWRpVuPLBcCU2z8TrisoyY1";

#[test]
fn unified_slot_wif_refused_in_every_template_context() {
    let xpub = "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ";
    let single = vec![
        "--scheme".to_string(),
        "bip84".into(),
        "--slot".into(),
        format!("@0.wif={SAMPLE_WIF}"),
    ];
    let hybrid = vec![
        "--scheme".to_string(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
        "--slot".into(),
        format!("@0.phrase={TREZOR_24}"),
        "--slot".into(),
        format!("@1.wif={SAMPLE_WIF}"),
        "--slot".into(),
        format!("@2.xpub={xpub}"),
    ];
    let pure = vec![
        "--scheme".to_string(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
        "--slot".into(),
        format!("@0.wif={SAMPLE_WIF}"),
        "--slot".into(),
        format!("@1.wif={SAMPLE_WIF_2}"),
    ];
    let twice = vec![
        "--scheme".to_string(),
        "wsh-sortedmulti".into(),
        "--threshold".into(),
        "2".into(),
        "--slot".into(),
        format!("@0.wif={SAMPLE_WIF}"),
        "--slot".into(),
        format!("@1.wif={SAMPLE_WIF}"),
    ];
    for (name, case, wif_idx) in [
        ("single", single, 0),
        ("hybrid", hybrid, 1),
        ("pure", pure, 0),
        ("twice", twice, 0),
    ] {
        let out = Command::cargo_bin("mnemonic")
            .unwrap()
            .arg("--allow-argv-secret")
            .arg("bundle")
            .args(&case)
            .args(["--network", "mainnet", "--json", "--no-engraving-card"])
            .assert()
            .failure()
            .code(1);
        let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
        assert!(stdout.is_empty(), "{name}: no card on stdout; got {stdout}");
        assert!(
            stderr.contains(&format!("@{wif_idx}.wif is refused")),
            "{name}: stderr: {stderr}"
        );
    }
}

#[test]
fn unified_slot_xpub_alone_emits_partial_origin() {
    // K.4: {xpub} alone (no fingerprint, no path). Defaults applied.
    let xpub = "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args([
            "bundle",
            "--scheme",
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
    // K.4: {xpub, fingerprint} (no path). v0.5.1 defaults the path from the template.
    let xpub = "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args([
            "bundle",
            "--scheme",
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
        .arg("--allow-argv-secret")
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
        .arg("--allow-argv-secret")
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
        .code(4);
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
        .arg("--allow-argv-secret")
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
        stderr.contains("derives master fingerprint") && stderr.contains("annotation specifies"),
        "fingerprint mismatch must be reported; got: {stderr}"
    );
}
