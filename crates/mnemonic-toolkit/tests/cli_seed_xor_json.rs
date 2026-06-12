//! v0.12.0 P2 — JSON envelope tests for `mnemonic seed-xor --json-out`.
//!
//! Per SPEC §2.3 + §4 G4. Schema `v1`. SHA-pinned over canonical
//! deterministic split anchors (post-GREEN capture).

use assert_cmd::Command;
use serde_json::Value;
use tempfile::NamedTempFile;

const ABANDON_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

const LEGAL_TREZOR_12: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";

fn split_with_json_out(phrase: &str, shares: usize) -> (String, Value, i32) {
    let f = NamedTempFile::new().unwrap();
    let path = f.path().to_owned();
    drop(f);
    let from_arg = format!("phrase={phrase}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("split")
        .arg("--from")
        .arg(&from_arg)
        .arg("--shares")
        .arg(shares.to_string())
        .arg("--deterministic-from-master")
        .arg("--json-out")
        .arg(&path)
        .output()
        .unwrap();
    let exit = out.status.code().unwrap_or(-1);
    let body = std::fs::read_to_string(&path).expect("json-out file must exist");
    let parsed: Value = serde_json::from_str(&body).expect("body must be valid JSON");
    (body, parsed, exit)
}

#[test]
fn split_envelope_schema_version_is_one() {
    let (_, parsed, exit) = split_with_json_out(ABANDON_12, 2);
    assert_eq!(exit, 0);
    assert_eq!(parsed["schema_version"], "1");
}

#[test]
fn split_envelope_operation_is_split() {
    let (_, parsed, _exit) = split_with_json_out(ABANDON_12, 2);
    assert_eq!(parsed["operation"], "split");
}

#[test]
fn split_envelope_fields_complete() {
    let (_, parsed, exit) = split_with_json_out(ABANDON_12, 3);
    assert_eq!(exit, 0);
    assert_eq!(parsed["language"], "english");
    assert_eq!(parsed["word_count"], 12);
    assert_eq!(parsed["share_count"], 3);
    assert_eq!(parsed["deterministic"], true);
    let shares = parsed["shares"].as_array().unwrap();
    assert_eq!(shares.len(), 3);
    for s in shares {
        let words = s.as_str().unwrap().split_whitespace().count();
        assert_eq!(words, 12);
    }
}

#[test]
fn split_envelope_24_word() {
    let twenty_four = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    let (_, parsed, exit) = split_with_json_out(twenty_four, 2);
    assert_eq!(exit, 0);
    assert_eq!(parsed["word_count"], 24);
}

#[test]
fn split_envelope_random_marks_deterministic_false() {
    // Without --deterministic-from-master flag
    let f = NamedTempFile::new().unwrap();
    let path = f.path().to_owned();
    drop(f);
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("split")
        .arg("--from")
        .arg(&from_arg)
        .arg("--shares")
        .arg("2")
        .arg("--json-out")
        .arg(&path)
        .output()
        .unwrap();
    assert!(out.status.success());
    let body = std::fs::read_to_string(&path).unwrap();
    let parsed: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["deterministic"], false);
}

#[test]
fn split_json_does_not_suppress_plain_stdout() {
    let f = NamedTempFile::new().unwrap();
    let path = f.path().to_owned();
    drop(f);
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("split")
        .arg("--from")
        .arg(&from_arg)
        .arg("--shares")
        .arg("2")
        .arg("--deterministic-from-master")
        .arg("--json-out")
        .arg(&path)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        stdout.lines().count(),
        2,
        "plain stdout still emits 2 lines"
    );
}

#[test]
fn combine_envelope_shape() {
    // Split first to get shares
    let f_split = NamedTempFile::new().unwrap();
    let split_path = f_split.path().to_owned();
    drop(f_split);
    let from_arg = format!("phrase={ABANDON_12}");
    let _split_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("split")
        .arg("--from")
        .arg(&from_arg)
        .arg("--shares")
        .arg("2")
        .arg("--deterministic-from-master")
        .arg("--json-out")
        .arg(&split_path)
        .output()
        .unwrap();
    let split_body = std::fs::read_to_string(&split_path).unwrap();
    let split_parsed: Value = serde_json::from_str(&split_body).unwrap();
    let shares: Vec<&str> = split_parsed["shares"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect();

    // Now combine with json-out
    let f_combine = NamedTempFile::new().unwrap();
    let combine_path = f_combine.path().to_owned();
    drop(f_combine);
    let s0 = format!("phrase={}", shares[0]);
    let s1 = format!("phrase={}", shares[1]);
    let combine_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("seed-xor")
        .arg("combine")
        .arg("--share")
        .arg(&s0)
        .arg("--share")
        .arg(&s1)
        .arg("--shares")
        .arg("2")
        .arg("--json-out")
        .arg(&combine_path)
        .output()
        .unwrap();
    assert!(combine_out.status.success());
    let body = std::fs::read_to_string(&combine_path).unwrap();
    let parsed: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["schema_version"], "1");
    assert_eq!(parsed["operation"], "combine");
    assert_eq!(parsed["language"], "english");
    assert_eq!(parsed["word_count"], 12);
    assert_eq!(parsed["share_count"], 2);
    assert_eq!(parsed["phrase"], ABANDON_12);
}

#[test]
fn anchor_abandon_12_envelope_sha_pin() {
    let (body, _parsed, exit) = split_with_json_out(ABANDON_12, 2);
    assert_eq!(exit, 0);
    use bitcoin::hashes::{sha256, Hash};
    let h = sha256::Hash::hash(body.as_bytes());
    let actual = format!("{}", h);
    // Pinned at P2 GREEN (2026-05-14).
    const EXPECTED: &str = "d368c70aabb6d3bab7d75b79f8a61a8340db6ac94c57250db6354fe235861af3";
    assert_eq!(
        actual, EXPECTED,
        "JSON envelope SHA-pin drift for abandon×12 N=2 deterministic; if schema/sort/algorithm changed intentionally, update EXPECTED",
    );
}

#[test]
fn anchor_trezor_12_envelope_sha_pin() {
    let (body, _parsed, exit) = split_with_json_out(LEGAL_TREZOR_12, 3);
    assert_eq!(exit, 0);
    use bitcoin::hashes::{sha256, Hash};
    let h = sha256::Hash::hash(body.as_bytes());
    let actual = format!("{}", h);
    // Pinned at P2 GREEN (2026-05-14).
    const EXPECTED: &str = "85d53f7e83db167b1223b8b23bbe2baca060e7aefad50f6034b5b65750883871";
    assert_eq!(
        actual, EXPECTED,
        "JSON envelope SHA-pin drift for legal×... 12-word Trezor vector N=3"
    );
}
