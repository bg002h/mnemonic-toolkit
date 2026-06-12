//! v0.11.0 P2 — CLI happy-path tests for `mnemonic final-word`.
//!
//! Per SPEC §4 G2 (plain stdout output) + plan §"Phase 2".
//! Tests the canonical N=12/15/18/21/24 round-trip plus the two
//! user-locked anchor vectors (abandon × 11 + beef × 11).
//! Plain stdout is the default; JSON tests live in
//! `cli_final_word_json.rs`.
//!
//! Per `feedback_default_cargo_test_runs_sibling_dependent_tests`: this
//! file has no external-state dependency (the binary is built from the
//! workspace itself), so no `#[ignore]` gating is needed.

use assert_cmd::Command;
use std::io::Write;
use std::process::Stdio;

const ABANDON_11_PARTIAL: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";

const BEEF_11_PARTIAL: &str = "beef beef beef beef beef beef beef beef beef beef beef";

/// Helper: invoke `mnemonic final-word --from phrase=<inline>` and
/// return (stdout-lines, stderr-as-string, exit-code).
fn invoke_inline(partial: &str, language: Option<&str>) -> (Vec<String>, String, i32) {
    let mut cmd = Command::cargo_bin("mnemonic").unwrap();
    cmd.arg("final-word")
        .arg("--from")
        .arg(format!("phrase={}", partial));
    if let Some(l) = language {
        cmd.arg("--language").arg(l);
    }
    let out = cmd.output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    let exit = out.status.code().unwrap_or(-1);
    let lines: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();
    (lines, stderr, exit)
}

#[test]
fn n12_abandon_11_emits_128_sorted_lines() {
    let (lines, _stderr, exit) = invoke_inline(ABANDON_11_PARTIAL, Some("english"));
    assert_eq!(exit, 0, "expected success");
    assert_eq!(lines.len(), 128, "N=12 must emit 128 candidates");
    let mut sorted = lines.clone();
    sorted.sort();
    assert_eq!(lines, sorted, "stdout must be lexicographically sorted");
    assert!(
        lines.iter().any(|w| w == "about"),
        "abandon×11 canonical Trezor vector must include 'about'",
    );
}

#[test]
fn n12_beef_11_emits_128_sorted_lines_including_beef() {
    let (lines, _stderr, exit) = invoke_inline(BEEF_11_PARTIAL, Some("english"));
    assert_eq!(exit, 0);
    assert_eq!(lines.len(), 128);
    assert!(
        lines.iter().any(|w| w == "beef"),
        "beef×11 anchor: 'beef' is in the candidate set (pinned at P1 GREEN)",
    );
}

fn invoke_with_partial_words(partial: &str) -> (usize, i32) {
    let (lines, _stderr, exit) = invoke_inline(partial, Some("english"));
    (lines.len(), exit)
}

/// For per-N round-trip we use bip39::Mnemonic::from_entropy_in to
/// construct a valid mnemonic at test runtime, drop the last word,
/// invoke the CLI, and verify the original last word is in the set.
fn round_trip(entropy_len_bytes: usize, expected_size: usize) {
    let entropy = vec![0u8; entropy_len_bytes];
    let m = bip39::Mnemonic::from_entropy_in(bip39::Language::English, &entropy).unwrap();
    let words: Vec<&str> = m.words().collect();
    let last = words.last().copied().unwrap().to_string();
    let partial: String = words[..words.len() - 1].join(" ");
    let (count, exit) = invoke_with_partial_words(&partial);
    assert_eq!(exit, 0, "round-trip N={} bytes", entropy_len_bytes);
    assert_eq!(
        count, expected_size,
        "N derived from {}-byte entropy: expected {expected_size} candidates",
        entropy_len_bytes,
    );

    // Verify the original word appears via a second invocation that prints.
    let (lines, _stderr, _exit) = invoke_inline(&partial, Some("english"));
    assert!(
        lines.iter().any(|w| w == &last),
        "round-trip: original last word '{}' must appear in CLI output",
        last,
    );
}

#[test]
fn n12_zero_entropy_round_trip() {
    round_trip(16, 128);
}

#[test]
fn n15_zero_entropy_round_trip() {
    round_trip(20, 64);
}

#[test]
fn n18_zero_entropy_round_trip() {
    round_trip(24, 32);
}

#[test]
fn n21_zero_entropy_round_trip() {
    round_trip(28, 16);
}

#[test]
fn n24_zero_entropy_round_trip() {
    round_trip(32, 8);
}

#[test]
fn stdout_has_trailing_newline_after_last_candidate() {
    let mut cmd = Command::cargo_bin("mnemonic").unwrap();
    let out = cmd
        .arg("final-word")
        .arg("--from")
        .arg(format!("phrase={}", ABANDON_11_PARTIAL))
        .arg("--language")
        .arg("english")
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.ends_with('\n'),
        "stdout must terminate with a trailing newline; got: {:?}",
        stdout.chars().rev().take(5).collect::<String>(),
    );
}

#[test]
fn language_default_is_english_when_omitted() {
    let mut cmd = Command::cargo_bin("mnemonic").unwrap();
    let out = cmd
        .arg("final-word")
        .arg("--from")
        .arg(format!("phrase={}", ABANDON_11_PARTIAL))
        .output()
        .unwrap();
    assert!(out.status.success(), "default language should succeed");
    let stdout = String::from_utf8(out.stdout).unwrap();
    let count = stdout.lines().count();
    assert_eq!(count, 128, "default language=english on abandon×11 → 128");
}

#[test]
fn spanish_partial_yields_spanish_candidates() {
    let m = bip39::Mnemonic::from_entropy_in(bip39::Language::Spanish, &[0u8; 16]).unwrap();
    let words: Vec<&str> = m.words().collect();
    let partial: String = words[..11].join(" ");
    let (lines, _stderr, exit) = invoke_inline(&partial, Some("spanish"));
    assert_eq!(exit, 0);
    assert_eq!(lines.len(), 128);
    let spanish_set: std::collections::BTreeSet<&'static str> = bip39::Language::Spanish
        .word_list()
        .iter()
        .copied()
        .collect();
    for line in &lines {
        assert!(
            spanish_set.contains(line.as_str()),
            "spanish candidate '{line}' must be in Spanish wordlist",
        );
    }
}

/// Suppress unused-import warning when invoking via stdio shape.
fn _suppress_warnings() {
    let _w: Box<dyn Write> = Box::new(std::io::stderr());
    let _s = Stdio::piped();
}
