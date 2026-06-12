//! v0.4.5 P.7: forensic-field integration tests for verify-bundle JSON envelope.
//!
//! Asserts SPEC §5.7 per-cell forensic-diagnostics rules:
//! - Pass: all forensic fields None.
//! - String mismatch: expected/actual/diff_byte_offset populated.
//! - Decode failure: decode_error populated.
//! - Watch-only short-circuit: passed=true, decode_error="skipped: watch-only slot".

use assert_cmd::Command;
use serde_json::Value;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

fn read_fixture() -> (String, Vec<String>, Vec<String>) {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let ms1 = fixture
        .lines()
        .find(|l| l.starts_with("ms1") && !l.contains(' '))
        .unwrap()
        .to_string();
    let mk1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    let md1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    (ms1, mk1, md1)
}

fn run_verify_bundle(ms1: &str, mk1: &[String], md1: &[String]) -> Value {
    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--slot".into(),
        format!("@0.phrase={TREZOR_24}"),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
        "--ms1".into(),
        ms1.into(),
        "--json".into(),
    ];
    for s in mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }
    let out = Command::cargo_bin("mnemonic").unwrap().args(&args).assert();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    serde_json::from_str(&stdout).expect("valid JSON")
}

fn run_verify_bundle_watch_only(xpub: &str, fp: &str, mk1: &[String], md1: &[String]) -> Value {
    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--slot".into(),
        format!("@0.xpub={xpub}"),
        "--slot".into(),
        format!("@0.fingerprint={fp}"),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
        "--json".into(),
    ];
    for s in mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }
    let out = Command::cargo_bin("mnemonic").unwrap().args(&args).assert();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    serde_json::from_str(&stdout).expect("valid JSON")
}

#[test]
fn happy_path_emits_no_forensic_fields() {
    let (ms1, mk1, md1) = read_fixture();
    let v = run_verify_bundle(&ms1, &mk1, &md1);
    assert_eq!(v["result"], "ok");
    let checks = v["checks"].as_array().unwrap();
    for c in checks {
        assert!(
            c["passed"].as_bool().unwrap(),
            "happy path: {} must pass",
            c["name"]
        );
        // Forensic fields are skip_serializing_if Option::is_none — absent on
        // pass means the JSON object has no expected/actual/diff/decode_error keys.
        assert!(
            c.get("expected").is_none(),
            "{}: expected absent on pass",
            c["name"]
        );
        assert!(
            c.get("actual").is_none(),
            "{}: actual absent on pass",
            c["name"]
        );
        assert!(
            c.get("diff_byte_offset").is_none(),
            "{}: diff absent on pass",
            c["name"]
        );
        assert!(
            c.get("decode_error").is_none(),
            "{}: decode_error absent on pass",
            c["name"]
        );
    }
}

#[test]
fn tampered_ms1_populates_forensic_fields() {
    let (ms1, mk1, md1) = read_fixture();
    // Tamper: insert garbage in the middle of the bech32 payload — beyond BCH
    // single-char correction radius, so ms_codec::decode rejects rather than
    // auto-corrects. This exercises the decode_error forensic-field path.
    let half = ms1.len() / 2;
    let tampered = format!("{}xxxx{}", &ms1[..half], &ms1[half + 4..]);
    let v = run_verify_bundle(&tampered, &mk1, &md1);
    assert_eq!(v["result"], "mismatch");
    let checks = v["checks"].as_array().unwrap();
    let decode = checks
        .iter()
        .find(|c| c["name"] == "ms1_decode")
        .expect("ms1_decode emitted");
    assert!(
        !decode["passed"].as_bool().unwrap(),
        "ms1_decode should fail on garbage payload"
    );
    assert!(
        decode.get("decode_error").is_some(),
        "decode_error populated on decode failure: {decode:?}"
    );
}

#[test]
fn watch_only_short_circuit_emits_decode_error() {
    // Use the bip84 fixture's mk1+md1 cards but invoke watch-only with the
    // matching xpub + fingerprint (extracted from the fixture's text dump).
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let mk1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    let md1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    // BIP-84 mainnet @ TREZOR_24 known values (from derive::tests).
    let xpub = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";
    let fp = "5436d724";
    let v = run_verify_bundle_watch_only(xpub, fp, &mk1, &md1);
    let checks = v["checks"].as_array().unwrap();
    let ms1_decode = checks
        .iter()
        .find(|c| c["name"] == "ms1_decode")
        .expect("ms1_decode emitted");
    assert!(
        ms1_decode["passed"].as_bool().unwrap(),
        "ms1_decode passes vacuously in watch-only"
    );
    assert_eq!(
        ms1_decode["decode_error"].as_str().unwrap(),
        "skipped: watch-only slot",
        "decode_error populated with the SPEC §5.7 sentinel string"
    );
    let ms1_match = checks
        .iter()
        .find(|c| c["name"] == "ms1_entropy_match")
        .expect("ms1_entropy_match emitted");
    assert!(ms1_match["passed"].as_bool().unwrap());
    assert_eq!(
        ms1_match["decode_error"].as_str().unwrap(),
        "skipped: watch-only slot"
    );
}
