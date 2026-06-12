//! v0.13.0 P2.2 — CLI happy-path tests for `mnemonic slip39`.
//!
//! Per SPEC §4 G3 (plain stdout shape) + G6 (Cycle A/B advisories — only
//! the absence-of-advisory aspect; positive advisory tests live in
//! `cli_slip39_advisories.rs`). Round-trip + trailing-newline +
//! blank-line group separator + entropy/phrase output shapes +
//! non-default `--passphrase` round-trip + hidden-interaction pin
//! (`--language` is silent when `--from entropy=`).
//!
//! Mirrors `cli_seed_xor_happy_paths.rs` (260 LOC) at parallel shape.
//! All tests FAIL at RED — `cmd/slip39.rs` returns a P2.1 stub
//! `ToolkitError::BadInput` until P2.2 GREEN lands the handler impl.

use assert_cmd::Command;

const ABANDON_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

const ABANDON_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

const ENTROPY_16_ZEROS_HEX: &str = "00000000000000000000000000000000";
const ENTROPY_32_ZEROS_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

fn split(args: &[&str]) -> (String, String, i32) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
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
        .arg("slip39")
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

/// Parse split's stdout into Vec<Vec<String>> (groups → shares).
/// SPEC §2.2 split: shares one per line; groups separated by blank line;
/// trailing newline.
fn parse_split_stdout(stdout: &str) -> Vec<Vec<String>> {
    let mut groups: Vec<Vec<String>> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for line in stdout.lines() {
        if line.is_empty() {
            if !current.is_empty() {
                groups.push(std::mem::take(&mut current));
            }
        } else {
            current.push(line.to_string());
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

#[test]
fn slip39_split_2_of_3_single_group_round_trip_via_entropy() {
    let from_arg = format!("entropy={ENTROPY_32_ZEROS_HEX}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 0, "split exit; stdout={stdout:?}");
    let groups = parse_split_stdout(&stdout);
    assert_eq!(groups.len(), 1, "expected 1 group; got {groups:?}");
    assert_eq!(
        groups[0].len(),
        3,
        "expected 3 shares in group; got {:?}",
        groups[0]
    );
    let (recovered, _stderr2, exit2) =
        combine(&["--share", &groups[0][0], "--share", &groups[0][1]]);
    assert_eq!(exit2, 0);
    assert_eq!(
        recovered.lines().next().unwrap(),
        ENTROPY_32_ZEROS_HEX,
        "round-trip entropy mismatch",
    );
}

#[test]
fn slip39_split_minimal_2_of_2_member_threshold_round_trip() {
    // Plan §4.1 happy-path "1-of-1" interpretation: simplest non-refused
    // config. Note: the library accepts `--group N,T` only when
    // `T >= 2` OR `(N=1, T=1)`; the `--group N,1` (N>1) case is lib-
    // refused as "duplicate-share, no recovery benefit" (slip39/mod.rs:134),
    // and `--group 1,1` is CLI-refused via SPEC §2.5 row 5. The minimal
    // valid config is `--group 2,2` (2 of 2 members within a single
    // group — both shares needed). The test asserts round-trip via
    // BOTH shares (the minimal recovery set).
    let from_arg = format!("phrase={ABANDON_12}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "1",
        "--group",
        "2,2",
    ]);
    assert_eq!(exit, 0);
    let groups = parse_split_stdout(&stdout);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].len(), 2);
    let (recovered, _, exit2) = combine(&[
        "--share",
        &groups[0][0],
        "--share",
        &groups[0][1],
        "--to",
        "phrase",
        "--language",
        "english",
    ]);
    assert_eq!(exit2, 0);
    assert_eq!(recovered.lines().next().unwrap(), ABANDON_12);
}

#[test]
fn slip39_split_1_of_2_groups_either_group_recovers() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "1",
        "--group",
        "3,2",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 0);
    let groups = parse_split_stdout(&stdout);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].len(), 3);
    assert_eq!(groups[1].len(), 3);
    let (rec_a, _, exit_a) = combine(&[
        "--share",
        &groups[0][0],
        "--share",
        &groups[0][1],
        "--to",
        "phrase",
        "--language",
        "english",
    ]);
    assert_eq!(exit_a, 0);
    assert_eq!(rec_a.lines().next().unwrap(), ABANDON_12);
    let (rec_b, _, exit_b) = combine(&[
        "--share",
        &groups[1][0],
        "--share",
        &groups[1][1],
        "--to",
        "phrase",
        "--language",
        "english",
    ]);
    assert_eq!(exit_b, 0);
    assert_eq!(rec_b.lines().next().unwrap(), ABANDON_12);
}

#[test]
fn slip39_split_2_of_3_groups_4_tier_hierarchy_round_trip() {
    let from_arg = format!("phrase={ABANDON_24}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "2",
        "--group",
        "3,2",
        "--group",
        "3,2",
        "--group",
        "5,3",
    ]);
    assert_eq!(exit, 0);
    let groups = parse_split_stdout(&stdout);
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0].len(), 3);
    assert_eq!(groups[1].len(), 3);
    assert_eq!(groups[2].len(), 5);
    // Recover from group 0 (2 shares satisfies T=2) + group 1 (2 shares)
    let (recovered, _, exit2) = combine(&[
        "--share",
        &groups[0][0],
        "--share",
        &groups[0][1],
        "--share",
        &groups[1][0],
        "--share",
        &groups[1][1],
        "--to",
        "phrase",
        "--language",
        "english",
    ]);
    assert_eq!(exit2, 0);
    assert_eq!(recovered.lines().next().unwrap(), ABANDON_24);
}

#[test]
fn slip39_split_24_word_phrase_round_trip_via_phrase() {
    let from_arg = format!("phrase={ABANDON_24}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 0);
    let groups = parse_split_stdout(&stdout);
    let (recovered, _, exit2) = combine(&[
        "--share",
        &groups[0][0],
        "--share",
        &groups[0][1],
        "--to",
        "phrase",
        "--language",
        "english",
    ]);
    assert_eq!(exit2, 0);
    assert_eq!(recovered.lines().next().unwrap(), ABANDON_24);
}

#[test]
fn slip39_split_12_word_phrase_16_byte_entropy_round_trip() {
    let from_arg = format!("entropy={ENTROPY_16_ZEROS_HEX}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 0);
    let groups = parse_split_stdout(&stdout);
    let (recovered, _, exit2) = combine(&["--share", &groups[0][0], "--share", &groups[0][1]]);
    assert_eq!(exit2, 0);
    assert_eq!(recovered.lines().next().unwrap(), ENTROPY_16_ZEROS_HEX);
}

#[test]
fn slip39_split_trailing_newline_and_blank_line_group_separator() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "1",
        "--group",
        "3,2",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 0);
    assert!(
        stdout.ends_with('\n'),
        "split stdout must end with newline; got {stdout:?}"
    );
    let lines: Vec<&str> = stdout.lines().collect();
    let blank_count = lines.iter().filter(|l| l.is_empty()).count();
    let non_blank_count = lines.iter().filter(|l| !l.is_empty()).count();
    assert_eq!(
        non_blank_count, 6,
        "expected 6 share lines (2 groups × 3 shares); got lines={lines:?}"
    );
    assert_eq!(
        blank_count, 1,
        "expected exactly 1 blank-line separator between 2 groups; got lines={lines:?}"
    );
}

#[test]
fn slip39_split_with_passphrase_round_trip() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--passphrase",
        "test-passphrase",
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 0);
    let groups = parse_split_stdout(&stdout);
    let (recovered, _, exit2) = combine(&[
        "--share",
        &groups[0][0],
        "--share",
        &groups[0][1],
        "--passphrase",
        "test-passphrase",
        "--to",
        "phrase",
        "--language",
        "english",
    ]);
    assert_eq!(exit2, 0);
    assert_eq!(recovered.lines().next().unwrap(), ABANDON_12);
}

#[test]
fn slip39_split_default_language_is_english() {
    // No `--language` flag → default english parsing of `--from phrase=`.
    let from_arg = format!("phrase={ABANDON_12}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 0);
    // Round-trip; default --to entropy yields the 16-byte all-zero hex
    let groups = parse_split_stdout(&stdout);
    let (recovered, _, exit2) = combine(&["--share", &groups[0][0], "--share", &groups[0][1]]);
    assert_eq!(exit2, 0);
    assert_eq!(recovered.lines().next().unwrap(), ENTROPY_16_ZEROS_HEX);
}

#[test]
fn slip39_split_spanish_phrase_round_trip() {
    let spanish = bip39::Mnemonic::from_entropy_in(bip39::Language::Spanish, &[0u8; 16])
        .unwrap()
        .to_string();
    let from_arg = format!("phrase={spanish}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--language",
        "spanish",
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 0);
    let groups = parse_split_stdout(&stdout);
    let (recovered, _, exit2) = combine(&[
        "--share",
        &groups[0][0],
        "--share",
        &groups[0][1],
        "--to",
        "phrase",
        "--language",
        "spanish",
    ]);
    assert_eq!(exit2, 0);
    assert_eq!(recovered.lines().next().unwrap(), spanish);
}

#[test]
fn slip39_split_with_language_flag_is_silent_for_entropy_input() {
    // Plan §6 risk 5 / hidden-interaction pin: `--language` is silent
    // when `--from entropy=` per SPEC §2.2. The combination round-trips
    // identically to the no-language variant.
    let from_arg = format!("entropy={ENTROPY_32_ZEROS_HEX}");
    let (stdout, _stderr, exit) = split(&[
        "--from",
        &from_arg,
        "--language",
        "spanish",
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 0);
    let groups = parse_split_stdout(&stdout);
    // Recover with default --to entropy; `--language` was silent on
    // input, so output hex must equal the original.
    let (recovered, _, exit2) = combine(&["--share", &groups[0][0], "--share", &groups[0][1]]);
    assert_eq!(exit2, 0);
    assert_eq!(recovered.lines().next().unwrap(), ENTROPY_32_ZEROS_HEX);
}
