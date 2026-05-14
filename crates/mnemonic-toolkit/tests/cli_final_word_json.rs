//! v0.11.0 P2 — JSON envelope tests for `mnemonic final-word --json-out`.
//!
//! Per SPEC §2.3 + §4 G3. Schema `v1`. Pinned via SHA-256 over the
//! canonical-sort output for two anchor vectors.

use assert_cmd::Command;
use serde_json::Value;
use tempfile::NamedTempFile;

const ABANDON_11_PARTIAL: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";

const BEEF_11_PARTIAL: &str =
    "beef beef beef beef beef beef beef beef beef beef beef";

fn invoke_with_json_out(partial: &str, language: &str) -> (String, Value, i32) {
    let f = NamedTempFile::new().unwrap();
    let path = f.path().to_owned();
    // Close the handle so the CLI can write to it on platforms with
    // exclusive-write semantics.
    drop(f);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .arg("--from")
        .arg(format!("phrase={}", partial))
        .arg("--language")
        .arg(language)
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
fn envelope_schema_version_is_one() {
    let (_, parsed, exit) = invoke_with_json_out(ABANDON_11_PARTIAL, "english");
    assert_eq!(exit, 0);
    assert_eq!(parsed["schema_version"], "1");
}

#[test]
fn envelope_has_no_feature_namespace_tag() {
    // R0 round 1 I4 decision: omit `feature` field for envelope-shape
    // consistency with existing `bundle --json` / `convert --json`.
    let (_, parsed, _exit) = invoke_with_json_out(ABANDON_11_PARTIAL, "english");
    assert!(
        parsed.get("feature").is_none(),
        "envelope must NOT contain a `feature` field (SPEC §2.3 decision)",
    );
}

#[test]
fn envelope_fields_complete_n12() {
    let (_, parsed, exit) = invoke_with_json_out(ABANDON_11_PARTIAL, "english");
    assert_eq!(exit, 0);
    assert_eq!(parsed["language"], "english");
    assert_eq!(parsed["partial_word_count"], 11);
    assert_eq!(parsed["target_word_count"], 12);
    assert_eq!(parsed["candidate_count"], 128);
    let cands = parsed["candidates"].as_array().unwrap();
    assert_eq!(cands.len(), 128);
    let abandon_strs: Vec<&str> = cands.iter().map(|v| v.as_str().unwrap()).collect();
    assert!(abandon_strs.contains(&"about"));
    let mut sorted = abandon_strs.clone();
    sorted.sort();
    assert_eq!(abandon_strs, sorted);
}

#[test]
fn envelope_language_is_kebab_case_for_multiword() {
    // Construct a 12-word Spanish phrase + drop last to test the
    // `simplified-chinese`-style kebab-case rendering.
    let m = bip39::Mnemonic::from_entropy_in(bip39::Language::SimplifiedChinese, &[0u8; 16]).unwrap();
    let words: Vec<&str> = m.words().collect();
    let partial: String = words[..11].join(" ");
    let (_, parsed, exit) = invoke_with_json_out(&partial, "simplifiedchinese");
    assert_eq!(exit, 0);
    assert_eq!(parsed["language"], "simplified-chinese");
}

#[test]
fn envelope_target_word_count_n24() {
    let m = bip39::Mnemonic::from_entropy_in(bip39::Language::English, &[0u8; 32]).unwrap();
    let words: Vec<&str> = m.words().collect();
    let partial: String = words[..23].join(" ");
    let (_, parsed, exit) = invoke_with_json_out(&partial, "english");
    assert_eq!(exit, 0);
    assert_eq!(parsed["partial_word_count"], 23);
    assert_eq!(parsed["target_word_count"], 24);
    assert_eq!(parsed["candidate_count"], 8);
}

/// Plain stdout MUST still be emitted alongside --json-out (the JSON is
/// a side-effect, not a stdout-replacement).
#[test]
fn json_out_does_not_suppress_plain_stdout() {
    let f = NamedTempFile::new().unwrap();
    let path = f.path().to_owned();
    drop(f);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("final-word")
        .arg("--from")
        .arg(format!("phrase={}", ABANDON_11_PARTIAL))
        .arg("--language")
        .arg("english")
        .arg("--json-out")
        .arg(&path)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 128, "plain stdout still emits 128 lines");
}

/// SHA-pin: byte-equal canonical envelope for `abandon × 11` anchor.
/// This is the regression backstop. Computed at GREEN time; pinned here.
#[test]
fn anchor_abandon_11_envelope_sha_pin() {
    let (body, _parsed, exit) = invoke_with_json_out(ABANDON_11_PARTIAL, "english");
    assert_eq!(exit, 0);
    let trimmed = body.trim_end_matches('\n');
    use bitcoin::hashes::{sha256, Hash};
    let h = sha256::Hash::hash(trimmed.as_bytes());
    let actual = format!("{}", h);
    // PIN POST-GREEN: replace placeholder with the captured SHA.
    const EXPECTED: &str = "P2_PLACEHOLDER_PIN_AT_GREEN";
    assert_eq!(
        actual, EXPECTED,
        "JSON envelope SHA-pin drift for abandon×11; if schema or sort \
         changed intentionally, update EXPECTED",
    );
}

#[test]
fn anchor_beef_11_envelope_sha_pin() {
    let (body, _parsed, exit) = invoke_with_json_out(BEEF_11_PARTIAL, "english");
    assert_eq!(exit, 0);
    let trimmed = body.trim_end_matches('\n');
    use bitcoin::hashes::{sha256, Hash};
    let h = sha256::Hash::hash(trimmed.as_bytes());
    let actual = format!("{}", h);
    const EXPECTED: &str = "P2_PLACEHOLDER_PIN_AT_GREEN";
    assert_eq!(actual, EXPECTED, "JSON envelope SHA-pin drift for beef×11");
}
