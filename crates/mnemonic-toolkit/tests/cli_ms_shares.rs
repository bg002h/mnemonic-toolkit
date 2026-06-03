//! ms K-of-N v0.2 Phase 3 Task 3.2 — CLI tests for `mnemonic ms-shares`.
//!
//! Realizes `design/SPEC_ms_v0_2_kofn.md` §4 (toolkit `mnemonic ms-shares`):
//!   - `split --from phrase=/entropy= --threshold K --shares N` → N ms1 shares.
//!   - `combine --share ... --to phrase|entropy|ms1` → recovered secret.
//!   - language survives a mnem split→combine (the wire language rides the
//!     secret-at-S bytes; `--to phrase` re-renders in the card language).
//!   - the combine→bundle composition the cycle is for (`combine --to entropy`
//!     piped into `bundle --slot @0.entropy=`).
//!
//! Mirrors `cli_slip39_happy_paths.rs` at parallel shape (split-or-combine
//! split into two helpers; positional-share-free `--share` repeating grammar).

use assert_cmd::Command;

const ABANDON_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

const ENTROPY_16_ZEROS_HEX: &str = "00000000000000000000000000000000";
const ENTROPY_32_ZEROS_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

fn split(args: &[&str]) -> (String, String, i32) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("ms-shares")
        .arg("split")
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
        out.status.code().unwrap_or(-1),
    )
}

fn combine(args: &[&str]) -> (String, String, i32) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("ms-shares")
        .arg("combine")
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
        out.status.code().unwrap_or(-1),
    )
}

/// Parse split's stdout into Vec<String> (one share per line; trailing newline).
fn parse_shares(stdout: &str) -> Vec<String> {
    stdout.lines().map(|l| l.to_string()).collect()
}

#[test]
fn ms_shares_split_2_of_3_entropy_round_trip() {
    let from_arg = format!("entropy={ENTROPY_32_ZEROS_HEX}");
    let (stdout, _stderr, exit) =
        split(&["--from", &from_arg, "--threshold", "2", "--shares", "3"]);
    assert_eq!(exit, 0, "split exit; stdout={stdout:?}");
    let shares = parse_shares(&stdout);
    assert_eq!(shares.len(), 3, "expected 3 shares; got {shares:?}");
    // Any 2 of 3 recombine. Default --to phrase, but entropy source → entropy
    // recovers via --to entropy.
    let (recovered, _stderr2, exit2) = combine(&[
        "--share", &shares[0], "--share", &shares[2], "--to", "entropy",
    ]);
    assert_eq!(exit2, 0, "combine exit; recovered={recovered:?}");
    assert_eq!(recovered.lines().next().unwrap(), ENTROPY_32_ZEROS_HEX);
}

#[test]
fn ms_shares_split_emits_n_shares_one_per_line_trailing_newline() {
    let from_arg = format!("entropy={ENTROPY_16_ZEROS_HEX}");
    let (stdout, _stderr, exit) =
        split(&["--from", &from_arg, "--threshold", "3", "--shares", "5"]);
    assert_eq!(exit, 0);
    assert!(stdout.ends_with('\n'), "stdout must end with newline; got {stdout:?}");
    let shares = parse_shares(&stdout);
    assert_eq!(shares.len(), 5);
    assert!(shares.iter().all(|s| s.starts_with("ms1")), "all ms1: {shares:?}");
}

#[test]
fn ms_shares_split_english_phrase_combine_to_phrase() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (stdout, _stderr, exit) =
        split(&["--from", &from_arg, "--threshold", "2", "--shares", "3"]);
    assert_eq!(exit, 0);
    let shares = parse_shares(&stdout);
    assert_eq!(shares.len(), 3);
    // English phrase source: default --to phrase recovers the phrase.
    let (recovered, _, exit2) =
        combine(&["--share", &shares[0], "--share", &shares[1]]);
    assert_eq!(exit2, 0, "combine; recovered={recovered:?}");
    assert_eq!(recovered.lines().next().unwrap(), ABANDON_12);
}

#[test]
fn ms_shares_split_phrase_combine_to_entropy() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (stdout, _stderr, exit) =
        split(&["--from", &from_arg, "--threshold", "2", "--shares", "2"]);
    assert_eq!(exit, 0);
    let shares = parse_shares(&stdout);
    let (recovered, _, exit2) = combine(&[
        "--share", &shares[0], "--share", &shares[1], "--to", "entropy",
    ]);
    assert_eq!(exit2, 0);
    // 12-word abandon-about = all-zero 16-byte entropy.
    assert_eq!(recovered.lines().next().unwrap(), ENTROPY_16_ZEROS_HEX);
}

#[test]
fn ms_shares_japanese_split_combine_preserves_language() {
    // Non-English mnem: the wordlist language rides the secret-at-S wire bytes
    // (Payload::Mnem). combine --to phrase must reconstruct the JA phrase, NOT
    // an English-defaulted one.
    let ja = bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &[0u8; 16])
        .unwrap()
        .to_string();
    let from_arg = format!("phrase={ja}");
    let (stdout, _stderr, exit) = split(&[
        "--from", &from_arg, "--language", "japanese", "--threshold", "2", "--shares", "3",
    ]);
    assert_eq!(exit, 0, "split; stdout={stdout:?}");
    let shares = parse_shares(&stdout);
    assert_eq!(shares.len(), 3);
    // No --language on combine: the language must come from the card (mnem),
    // not the CLI default.
    let (recovered, _, exit2) = combine(&[
        "--share", &shares[0], "--share", &shares[2], "--to", "phrase",
    ]);
    assert_eq!(exit2, 0, "combine; recovered={recovered:?}");
    assert_eq!(recovered.lines().next().unwrap(), ja);
}

#[test]
fn ms_shares_combine_to_ms1_recovers_single_string() {
    // combine --to ms1: the recovered secret re-encodes to a v0.1 single-string
    // ms1 (threshold 0). For an English phrase source it's an entr ms1.
    let from_arg = format!("phrase={ABANDON_12}");
    let (stdout, _, exit) =
        split(&["--from", &from_arg, "--threshold", "2", "--shares", "3"]);
    assert_eq!(exit, 0);
    let shares = parse_shares(&stdout);
    let (recovered, _, exit2) = combine(&[
        "--share", &shares[0], "--share", &shares[1], "--to", "ms1",
    ]);
    assert_eq!(exit2, 0, "combine --to ms1; recovered={recovered:?}");
    let ms1 = recovered.lines().next().unwrap();
    assert!(ms1.starts_with("ms1"), "recovered ms1: {ms1:?}");
    // The recovered single-string ms1 must itself decode back to the phrase via
    // `convert --from ms1 --to phrase`.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["convert", "--from", &format!("ms1={ms1}"), "--to", "phrase"])
        .output()
        .unwrap();
    assert_eq!(out.status.code().unwrap_or(-1), 0, "convert exit");
    let phrase = String::from_utf8(out.stdout).unwrap();
    // `convert` emits a labeled `phrase: <mnemonic>` line.
    assert!(
        phrase.contains(ABANDON_12),
        "recovered ms1 must convert back to the phrase; got {phrase:?}"
    );
}

#[test]
fn ms_shares_combine_to_entropy_composes_into_bundle() {
    // The composition the cycle is for: split a secret, recombine via
    // `combine --to entropy`, feed the recovered entropy into `bundle` as a
    // slot source → a valid 3-card bundle. (NOTE: `bundle` has no `ms1` slot
    // subkey; the realizable seed-overlay subkeys are `entropy`/`phrase`.)
    let from_arg = format!("entropy={ENTROPY_16_ZEROS_HEX}");
    let (stdout, _, exit) =
        split(&["--from", &from_arg, "--threshold", "2", "--shares", "3"]);
    assert_eq!(exit, 0);
    let shares = parse_shares(&stdout);
    let (recovered, _, exit2) = combine(&[
        "--share", &shares[1], "--share", &shares[2], "--to", "entropy",
    ]);
    assert_eq!(exit2, 0);
    let entropy_hex = recovered.lines().next().unwrap();
    assert_eq!(entropy_hex, ENTROPY_16_ZEROS_HEX);

    // Feed into bundle as a seed-overlay entropy slot.
    let slot = format!("@0.entropy={entropy_hex}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle", "--network", "mainnet", "--template", "bip84", "--slot", &slot,
        ])
        .output()
        .unwrap();
    let b_stdout = String::from_utf8(out.stdout).unwrap();
    let b_stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(
        out.status.code().unwrap_or(-1),
        0,
        "bundle exit; stdout={b_stdout:?} stderr={b_stderr:?}"
    );
    // A valid bundle emits ms1/mk1/md1 lines.
    assert!(b_stdout.contains("ms1"), "bundle stdout has ms1: {b_stdout:?}");
    assert!(b_stdout.contains("mk1"), "bundle stdout has mk1: {b_stdout:?}");
    assert!(b_stdout.contains("md1"), "bundle stdout has md1: {b_stdout:?}");
}

#[test]
fn ms_shares_split_emits_private_key_material_advisory() {
    let from_arg = format!("entropy={ENTROPY_16_ZEROS_HEX}");
    let (_stdout, stderr, exit) =
        split(&["--from", &from_arg, "--threshold", "2", "--shares", "3"]);
    assert_eq!(exit, 0);
    // OutputClass::PrivateKeyMaterial advisory fires unconditionally (Cycle B P1).
    assert!(
        stderr.to_lowercase().contains("secret") || stderr.contains("private"),
        "split must emit a PrivateKeyMaterial advisory; got stderr={stderr:?}"
    );
}

#[test]
fn ms_shares_combine_emits_private_key_material_advisory() {
    let from_arg = format!("entropy={ENTROPY_16_ZEROS_HEX}");
    let (stdout, _, exit) =
        split(&["--from", &from_arg, "--threshold", "2", "--shares", "2"]);
    assert_eq!(exit, 0);
    let shares = parse_shares(&stdout);
    let (_recovered, stderr, exit2) = combine(&[
        "--share", &shares[0], "--share", &shares[1], "--to", "entropy",
    ]);
    assert_eq!(exit2, 0);
    assert!(
        stderr.to_lowercase().contains("secret") || stderr.contains("private"),
        "combine must emit a PrivateKeyMaterial advisory; got stderr={stderr:?}"
    );
}

#[test]
fn ms_shares_split_inline_from_emits_argv_advisory() {
    let from_arg = format!("entropy={ENTROPY_16_ZEROS_HEX}");
    let (_stdout, stderr, exit) =
        split(&["--from", &from_arg, "--threshold", "2", "--shares", "3"]);
    assert_eq!(exit, 0);
    // Inline (non-stdin) --from value emits an argv-leakage advisory.
    assert!(
        stderr.contains("argv") || stderr.contains("cmdline") || stderr.contains("--from"),
        "inline --from should warn argv-leak; got stderr={stderr:?}"
    );
}

#[test]
fn ms_shares_split_rejects_bad_threshold() {
    // --threshold 1 is outside 2..=9 → InvalidThreshold → BadInput exit 1.
    let from_arg = format!("entropy={ENTROPY_16_ZEROS_HEX}");
    let (_stdout, _stderr, exit) =
        split(&["--from", &from_arg, "--threshold", "1", "--shares", "3"]);
    assert_ne!(exit, 0, "threshold=1 must be rejected");
}

#[test]
fn ms_shares_split_rejects_shares_below_threshold() {
    // --shares 2 < --threshold 3 → InvalidShareCount.
    let from_arg = format!("entropy={ENTROPY_16_ZEROS_HEX}");
    let (_stdout, _stderr, exit) =
        split(&["--from", &from_arg, "--threshold", "3", "--shares", "2"]);
    assert_ne!(exit, 0, "shares < threshold must be rejected");
}
