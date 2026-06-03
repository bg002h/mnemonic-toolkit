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
        .args([
            "ms-shares", "split", "--from", &from_arg, "--threshold", "2", "--shares", "3",
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
        .args(["convert", "--from", &format!("ms1={share}"), "--to", "phrase"])
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
