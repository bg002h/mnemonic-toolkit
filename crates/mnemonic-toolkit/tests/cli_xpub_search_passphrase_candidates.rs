//! `mnemonic xpub-search passphrase-of-xpub --passphrase-candidates-file` —
//! candidate-list passphrase scan (FOLLOWUP `xpub-search-passphrase-bruteforce`,
//! file-only scope). Loops the existing single-passphrase oracle over a text
//! file (one candidate per line, no argv exposure), aborts on first match,
//! reports the matching FILE LINE NUMBER to stdout (the passphrase only in
//! `--json`). See design/SPEC_xpub_search_passphrase_candidates_file.md.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::NetworkKind;
use predicates::prelude::*;
use serde_json::Value;
use std::io::Write;
use std::str::FromStr;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// The known passphrase whose bip84 xpub we target.
const SECRET: &str = "satoshi";

fn xpub_at(path: &str, passphrase: &str) -> Xpub {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, PHRASE).unwrap();
    let seed = mnemonic.to_seed(passphrase);
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    let dp = DerivationPath::from_str(path).unwrap();
    let xpriv = master.derive_priv(&secp, &dp).unwrap();
    Xpub::from_priv(&secp, &xpriv)
}

/// Write `contents` to a temp file and return it (kept alive by the caller).
fn candidates_file(contents: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(contents.as_bytes()).unwrap();
    f.flush().unwrap();
    f
}

fn target_bip84() -> String {
    xpub_at("m/84'/0'/0'", SECRET).to_string()
}

// ── Cell 1 — hit: SECRET is line 3 of the file → exit 0, reports line 3 ──────
#[test]
fn candidates_hit_reports_matching_line() {
    let file = candidates_file("decoy1\ndecoy2\nsatoshi\ndecoy4\n");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase-candidates-file",
            file.path().to_str().unwrap(),
            "--target-xpub",
            &target_bip84(),
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .stdout(predicate::str::contains("3")); // the 1-indexed matching line
}

// ── Cell 2 — hit --json: matched_candidate_line + matched_passphrase ─────────
#[test]
fn candidates_hit_json_carries_line_and_passphrase() {
    let file = candidates_file("decoy1\nsatoshi\n");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase-candidates-file",
            file.path().to_str().unwrap(),
            "--target-xpub",
            &target_bip84(),
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    // tagged enum: result == "match"
    assert_eq!(v["result"], "match");
    assert_eq!(v["matched_candidate_line"], 2);
    assert_eq!(v["matched_passphrase"], SECRET);
}

// ── Cell 3 — miss: exit 4 (Exhausted), candidates_tried == non-blank lines ───
#[test]
fn candidates_miss_exit4_candidates_tried() {
    let file = candidates_file("nope1\nnope2\nnope3\n");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase-candidates-file",
            file.path().to_str().unwrap(),
            "--target-xpub",
            &target_bip84(),
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(4);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["result"], "no_match");
    assert_eq!(v["candidates_tried"], 3);
}

// ── Cell 4 — abort-on-first: SECRET twice → reports the FIRST occurrence ─────
#[test]
fn candidates_abort_on_first_occurrence() {
    let file = candidates_file("x\nsatoshi\ny\nsatoshi\n"); // lines 2 and 4
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase-candidates-file",
            file.path().to_str().unwrap(),
            "--target-xpub",
            &target_bip84(),
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["matched_candidate_line"], 2);
}

// ── Cell 5 — blank lines are skipped (don't count toward candidates_tried) ───
#[test]
fn candidates_blank_lines_skipped() {
    // 2 real candidates (lines 1,3); line 2 blank. Neither matches → tried==2.
    let file = candidates_file("nope1\n\nnope2\n");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase-candidates-file",
            file.path().to_str().unwrap(),
            "--target-xpub",
            &target_bip84(),
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(4);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["candidates_tried"], 2);
}

// ── Cell 6 — exact bytes: a trailing-space candidate is literal ──────────────
#[test]
fn candidates_exact_bytes_trailing_space() {
    // Target uses passphrase "pw " (trailing space). A file line "pw " must
    // match it; a file with only "pw" (trimmed) must NOT.
    let target_sp = xpub_at("m/84'/0'/0'", "pw ").to_string();
    let hit = candidates_file("pw \n");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase-candidates-file",
            hit.path().to_str().unwrap(),
            "--target-xpub",
            &target_sp,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0);
    let miss = candidates_file("pw\n");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase-candidates-file",
            miss.path().to_str().unwrap(),
            "--target-xpub",
            &target_sp,
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(4);
}

// ── Cell 7 — mutex: --passphrase + --passphrase-candidates-file → exit 64 ────
#[test]
fn candidates_mutex_with_passphrase_exit_64() {
    let file = candidates_file("satoshi\n");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase",
            "satoshi",
            "--passphrase-candidates-file",
            file.path().to_str().unwrap(),
            "--target-xpub",
            &target_bip84(),
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(64);
}

// ── Cell 8 — required-one-of: none of the 3 sources → exit 64 ────────────────
#[test]
fn candidates_none_of_passphrase_sources_exit_64() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &target_bip84(),
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(64);
}

// ── Cell 9 — empty file (all blank) → exit 4, candidates_tried == 0 ──────────
#[test]
fn candidates_empty_file_exit4_zero_tried() {
    let file = candidates_file("\n\n");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase-candidates-file",
            file.path().to_str().unwrap(),
            "--target-xpub",
            &target_bip84(),
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(4);
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(v["candidates_tried"], 0);
}

// ── Cell 10 — secret hygiene: default (non-json) stdout does NOT echo SECRET ─
#[test]
fn candidates_default_stdout_does_not_echo_passphrase() {
    let file = candidates_file("decoy\nsatoshi\n");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase-candidates-file",
            file.path().to_str().unwrap(),
            "--target-xpub",
            &target_bip84(),
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0);
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains(SECRET),
        "default stdout must not echo the passphrase:\n{stdout}"
    );
}

// ── Cell 11 — missing file → exit 1 (BadInput) ──────────────────────────────
#[test]
fn candidates_missing_file_exit_1() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase-candidates-file",
            "/no/such/candidates/file",
            "--target-xpub",
            &target_bip84(),
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(1);
}

// ── Cell 12 — sensitivity advisory on stderr (impl-review I1 / SPEC §2) ──────
#[test]
fn candidates_emits_sensitivity_advisory_on_stderr() {
    let file = candidates_file("decoy\nsatoshi\n");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "passphrase-of-xpub",
            "--phrase-stdin",
            "--passphrase-candidates-file",
            file.path().to_str().unwrap(),
            "--target-xpub",
            &target_bip84(),
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .stderr(predicate::str::contains("treat as sensitive"));
}
