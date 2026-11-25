//! ms K-of-N v0.2 Phase 3 Task 3.3 — consume-path threshold-dispatch.
//!
//! Realizes `design/SPEC_ms_v0_2_kofn.md` §4 (R0-m3): a toolkit consume path
//! (`inspect` / `convert --from ms1=`) handed ONE share of a K-of-N set must
//! surface the friendly "this is a K-of-N share; use `mnemonic ms-shares
//! combine`" message (NOT "unhandled ms_codec::Error variant"), with the
//! mapped exit code — NOT the generic exit-1 wildcard fall-through.

use assert_cmd::Command;

const ENTROPY_16_ZEROS_HEX: &str = "00000000000000000000000000000000";

/// Produce a single distributed share string via `ms-shares split`.
fn one_share() -> String {
    let from_arg = format!("entropy={ENTROPY_16_ZEROS_HEX}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args([
            "ms-shares",
            "split",
            "--from",
            &from_arg,
            "--threshold",
            "2",
            "--shares",
            "3",
            // mstring-grouping P4: this test exercises share-DETECTION on
            // intake; keep the share unbroken (default split is now space/5).
            "--group-size",
            "0",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code().unwrap_or(-1), 0, "split must succeed");
    let stdout = String::from_utf8(out.stdout).unwrap();
    stdout.lines().next().unwrap().to_string()
}

#[test]
fn inspect_of_a_share_surfaces_friendly_message() {
    let share = one_share();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args(["inspect", "--ms1", &share])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let exit = out.status.code().unwrap_or(-1);
    let combined = format!("{stdout}{stderr}");
    assert!(
        !combined.contains("unhandled ms_codec::Error variant"),
        "inspect must NOT fall through to the wildcard; got exit={exit} stdout={stdout:?} stderr={stderr:?}"
    );
    assert!(
        combined.contains("ms-shares combine") || combined.to_lowercase().contains("share"),
        "inspect of a share must point at `ms-shares combine`; got exit={exit} combined={combined:?}"
    );
    // Mapped exit code: a format/usage class (NOT 0 success).
    assert_ne!(exit, 0, "inspecting a single share is not a success");
}

#[test]
fn convert_from_share_surfaces_friendly_message_and_mapped_exit() {
    let share = one_share();
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args([
            "convert",
            "--from",
            &format!("ms1={share}"),
            "--to",
            "phrase",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let exit = out.status.code().unwrap_or(-1);
    let combined = format!("{stdout}{stderr}");
    assert!(
        !combined.contains("unhandled ms_codec::Error variant"),
        "convert must NOT fall through to the wildcard; got exit={exit} combined={combined:?}"
    );
    assert!(
        combined.contains("ms-shares combine"),
        "convert of a share must point at `mnemonic ms-shares combine`; got exit={exit} combined={combined:?}"
    );
    // ms_codec_exit_code maps IsShareNotSingleString to a format/usage class
    // (2), distinct from the generic exit-1 wildcard fall-through.
    assert_eq!(exit, 2, "IsShareNotSingleString → exit 2; got {exit}");
}

// ---- bh-tk-ms-s-index: an index-`s` ms1 string is the secret itself ----

/// Synthetic index-`s` string (threshold 2, id `test`); not a funded key.
const S2: &str = "ms12testsqqqqqqqqqqqqqqqqqqqqqqqqqqqqkwe5qlrp6ykwa";
/// Synthetic index-`a` string (threshold 2, id `test`).
const A2: &str = "ms12testaqqqqqqqqqqqqqqqqqqqqqqqqqqqqzfvk7urqpuwd0";
/// `S2` with its last character substituted (`a` -> `q`).
const S2B: &str = "ms12testsqqqqqqqqqqqqqqqqqqqqqqqqqqqqkwe5qlrp6ykwq";
/// `A2` with its last character substituted (`0` -> `q`).
const A2B: &str = "ms12testaqqqqqqqqqqqqqqqqqqqqqqqqqqqqzfvk7urqpuwdq";

const S_INDEX_TEXT: &str = "ms1 this is the secret share of a K-of-N set (threshold '2', index 's'); it is the secret itself, not a distributed share, and `mnemonic ms-shares combine` does not take it";
const S_INDEX_REPAIR_B: &str = "correction UNVERIFIED — a corrected codex32 secret (share index 's' of a K-of-N set) cannot be self-verified; it is the secret itself, not a distributed share, and `mnemonic ms-shares combine` does not take it; BIP-93 recommends confirming a corrected codex32 string";
const S_INDEX_REPAIR_C: &str = "correction UNVERIFIED — a corrected K-of-N share cannot be self-verified; recombine it with the other shares (mnemonic ms-shares combine --share - --share @env:VAR …, one share on stdin and each further share in an exported variable) and confirm the recovered seed's address/xpub against a known-good copy before use; leave out the corrected string at share index 's': it is the secret itself, not a distributed share, and `mnemonic ms-shares combine` does not take it; BIP-93 recommends confirming a corrected codex32 string";
const PK_ADVISORY: &str = "warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')";

fn run_stdin(args: &[&str], stdin: &str) -> (i32, String, String) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .write_stdin(stdin.to_string())
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

#[test]
fn bh_tk_ms_s_index_convert() {
    let (code, so, se) = run_stdin(
        &["convert", "--from", "ms1=-", "--to", "phrase"],
        &format!("{S2}\n"),
    );
    assert_eq!(code, 2);
    assert_eq!(so, "");
    assert_eq!(se, format!("error: {S_INDEX_TEXT}\n"));
}

#[test]
fn bh_tk_ms_s_index_inspect() {
    let (code, so, se) = run_stdin(&["inspect", "--ms1", "-"], &format!("{S2}\n"));
    assert_eq!(code, 2);
    assert_eq!(so, "");
    assert_eq!(se, format!("error: {S_INDEX_TEXT}\n"));
}

#[test]
fn bh_tk_ms_s_index_uppercase() {
    let (code, _so, se) = run_stdin(
        &["convert", "--from", "ms1=-", "--to", "phrase"],
        &format!("{}\n", S2.to_uppercase()),
    );
    assert_eq!(code, 2);
    assert_eq!(se, format!("error: {S_INDEX_TEXT}\n"));
}

#[test]
fn bh_tk_ms_s_index_control_distributed() {
    let (code, _so, se) = run_stdin(
        &["convert", "--from", "ms1=-", "--to", "phrase"],
        &format!("{A2}\n"),
    );
    assert_eq!(code, 2);
    assert_eq!(
        se,
        "error: ms1 this is ONE of a K-of-N share set (threshold '2', index 'a'); use `mnemonic ms-shares combine` to recombine 2 shares\n"
    );
}

#[test]
fn bh_tk_ms_s_index_repair() {
    let (code, so, se) = run_stdin(&["repair", "--ms1", "-"], &format!("{S2B}\n"));
    assert_eq!(code, 4);
    assert_eq!(
        so,
        format!(
            "# Repair report\n#   ms1 chunk 0: 1 correction at position 46: 'q' -> 'a'\n{S2}\n"
        )
    );
    assert_eq!(se, format!("repair: {S_INDEX_REPAIR_B}\n{PK_ADVISORY}\n"));
}

#[test]
fn bh_tk_ms_s_index_repair_mixed() {
    let (code, _so, se) = run_stdin(&["repair", "--allow-argv-secret", S2B, A2B], "");
    assert_eq!(code, 4);
    // stderr opens with two argv-secret warning lines, so `contains`, not `starts_with`.
    assert!(
        se.contains(&format!("repair: {S_INDEX_REPAIR_C}\n")),
        "stderr: {se}"
    );
}
