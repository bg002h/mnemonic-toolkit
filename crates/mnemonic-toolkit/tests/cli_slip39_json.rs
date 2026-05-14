//! v0.13.0 P2.2 — JSON envelope tests for `mnemonic slip39 --json-out`.
//!
//! Per SPEC §2.3 + §4 G4 (env-var SHA-pin per Q2 fold; field order per
//! R0 N4 fold — `shares` LAST in `SplitGroupEntry`). Mirrors
//! `cli_seed_xor_json.rs` shape; the SHA-pinned anchor tests below
//! drive determinism via the `MNEMONIC_SLIP39_TEST_RNG` +
//! `MNEMONIC_SLIP39_TEST_IDENTIFIER` env-var wedge.
//!
//! At RED, the EXPECTED_SHA placeholder values below are `64*'0'`;
//! they are captured at GREEN once the envelope serializer is wired,
//! same pattern as `cli_seed_xor_json.rs:194,209`.

use assert_cmd::Command;
use serde_json::Value;
use tempfile::NamedTempFile;

const ABANDON_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

const TEST_RNG_HEX_64: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";
const TEST_RNG_HEX_64_ANCHOR_2: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";

/// Helper: split with `--json-out`; returns `(body_string, parsed_json,
/// exit_code)`. No env-vars set (production path; random identifier).
fn split_with_json_out(args: &[&str]) -> (String, Value, i32) {
    let f = NamedTempFile::new().unwrap();
    let path = f.path().to_owned();
    drop(f);
    let mut cmd_args = vec!["slip39", "split"];
    cmd_args.extend_from_slice(args);
    cmd_args.push("--json-out");
    let path_str = path.to_string_lossy().into_owned();
    cmd_args.push(&path_str);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&cmd_args)
        .output()
        .unwrap();
    let exit = out.status.code().unwrap_or(-1);
    let body = std::fs::read_to_string(&path).expect("json-out file must exist");
    let parsed: Value = serde_json::from_str(&body).expect("body must be valid JSON");
    (body, parsed, exit)
}

#[test]
fn json_split_schema_version_is_one() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (_, parsed, exit) = split_with_json_out(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 0);
    assert_eq!(parsed["schema_version"], "1");
}

#[test]
fn json_split_operation_is_split() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (_, parsed, _exit) = split_with_json_out(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(parsed["operation"], "split");
}

#[test]
fn json_split_envelope_top_level_fields() {
    let from_arg = format!("phrase={ABANDON_12}");
    let (_, parsed, exit) = split_with_json_out(&[
        "--from",
        &from_arg,
        "--iteration-exponent",
        "0",
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 0);
    assert!(parsed["identifier"].is_u64(), "identifier must be a u64");
    assert_eq!(parsed["iteration_exponent"], 0);
    assert_eq!(parsed["group_threshold"], 1);
    let groups = parsed["groups"].as_array().unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["member_count"], 3);
    assert_eq!(groups[0]["member_threshold"], 2);
    let shares = groups[0]["shares"].as_array().unwrap();
    assert_eq!(shares.len(), 3);
}

#[test]
fn json_split_group_entry_field_order_shares_last() {
    // R0 N4 fold: `shares` MUST appear LAST in `SplitGroupEntry` to
    // mirror `seed_xor.rs:352-361` precedent.
    let from_arg = format!("phrase={ABANDON_12}");
    let (body, _parsed, exit) = split_with_json_out(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 0);
    let mc_pos = body
        .find("\"member_count\"")
        .expect("member_count field present");
    let mt_pos = body
        .find("\"member_threshold\"")
        .expect("member_threshold field present");
    let shares_pos = body.find("\"shares\"").expect("shares field present");
    assert!(
        mc_pos < mt_pos,
        "member_count must precede member_threshold; got mc={mc_pos} mt={mt_pos} in:\n{body}"
    );
    assert!(
        mt_pos < shares_pos,
        "member_threshold must precede shares (R0 N4 fold: shares LAST); got mt={mt_pos} shares={shares_pos} in:\n{body}"
    );
}

#[test]
fn json_split_top_level_field_order_pins_schema_version_first() {
    // Top-level field order per SPEC §2.3: schema_version, operation,
    // identifier, iteration_exponent, group_threshold, groups.
    let from_arg = format!("phrase={ABANDON_12}");
    let (body, _parsed, exit) = split_with_json_out(&[
        "--from",
        &from_arg,
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
    assert_eq!(exit, 0);
    let sv_pos = body.find("\"schema_version\"").unwrap();
    let op_pos = body.find("\"operation\"").unwrap();
    let id_pos = body.find("\"identifier\"").unwrap();
    let gt_pos = body.find("\"group_threshold\"").unwrap();
    let g_pos = body.find("\"groups\"").unwrap();
    assert!(sv_pos < op_pos, "schema_version < operation");
    assert!(op_pos < id_pos, "operation < identifier");
    assert!(id_pos < gt_pos, "identifier < group_threshold");
    assert!(gt_pos < g_pos, "group_threshold < groups (groups LAST)");
}

#[test]
fn json_split_plain_stdout_coexists_with_json_out() {
    let f = NamedTempFile::new().unwrap();
    let path = f.path().to_owned();
    drop(f);
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
            "--json-out",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let non_blank_lines = stdout.lines().filter(|l| !l.is_empty()).count();
    assert_eq!(
        non_blank_lines, 3,
        "plain stdout must still emit 3 share lines alongside --json-out; got: {stdout:?}"
    );
}

#[test]
fn json_combine_envelope_shape_entropy_output() {
    // Round-trip via env-var-driven deterministic split to get shares,
    // then combine with --json-out --to entropy.
    let f_split = NamedTempFile::new().unwrap();
    let split_path = f_split.path().to_owned();
    drop(f_split);
    let from_arg = format!("phrase={ABANDON_12}");
    let split_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_SLIP39_TEST_RNG", TEST_RNG_HEX_64)
        .env("MNEMONIC_SLIP39_TEST_IDENTIFIER", "12345")
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
            "--json-out",
        ])
        .arg(&split_path)
        .output()
        .unwrap();
    if !split_out.status.success() {
        // RED — handler not yet implemented; pin assertion at split
        let stderr = String::from_utf8(split_out.stderr).unwrap();
        panic!("split failed (expected at RED); stderr={stderr}");
    }
    let split_body = std::fs::read_to_string(&split_path).unwrap();
    let split_parsed: Value = serde_json::from_str(&split_body).unwrap();
    let shares: Vec<String> = split_parsed["groups"][0]["shares"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect();

    let f_combine = NamedTempFile::new().unwrap();
    let combine_path = f_combine.path().to_owned();
    drop(f_combine);
    let combine_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("combine")
        .args(["--share", &shares[0], "--share", &shares[1], "--json-out"])
        .arg(&combine_path)
        .output()
        .unwrap();
    assert!(combine_out.status.success());
    let body = std::fs::read_to_string(&combine_path).unwrap();
    let parsed: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["schema_version"], "1");
    assert_eq!(parsed["operation"], "combine");
    assert_eq!(parsed["identifier"], 12345);
    assert_eq!(parsed["iteration_exponent"], 0);
    assert_eq!(parsed["output_shape"], "entropy");
    assert!(
        parsed["entropy_hex"].is_string(),
        "entropy_hex must be a string when --to entropy; got {:?}",
        parsed["entropy_hex"]
    );
    assert!(
        parsed["phrase"].is_null(),
        "phrase must be null when --to entropy; got {:?}",
        parsed["phrase"]
    );
}

#[test]
fn json_combine_envelope_shape_phrase_output() {
    let f_split = NamedTempFile::new().unwrap();
    let split_path = f_split.path().to_owned();
    drop(f_split);
    let from_arg = format!("phrase={ABANDON_12}");
    let split_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_SLIP39_TEST_RNG", TEST_RNG_HEX_64)
        .env("MNEMONIC_SLIP39_TEST_IDENTIFIER", "12345")
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--group-threshold",
            "1",
            "--group",
            "3,2",
            "--json-out",
        ])
        .arg(&split_path)
        .output()
        .unwrap();
    assert!(split_out.status.success());
    let split_body = std::fs::read_to_string(&split_path).unwrap();
    let split_parsed: Value = serde_json::from_str(&split_body).unwrap();
    let shares: Vec<String> = split_parsed["groups"][0]["shares"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect();

    let f_combine = NamedTempFile::new().unwrap();
    let combine_path = f_combine.path().to_owned();
    drop(f_combine);
    let combine_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("slip39")
        .arg("combine")
        .args([
            "--share",
            &shares[0],
            "--share",
            &shares[1],
            "--to",
            "phrase",
            "--language",
            "english",
            "--json-out",
        ])
        .arg(&combine_path)
        .output()
        .unwrap();
    assert!(combine_out.status.success());
    let body = std::fs::read_to_string(&combine_path).unwrap();
    let parsed: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["output_shape"], "phrase");
    assert_eq!(parsed["phrase"], ABANDON_12);
    assert!(
        parsed["entropy_hex"].is_null(),
        "entropy_hex must be null when --to phrase"
    );
}

// ============================================================
// G4 SHA-pin anchors via env-var wedge (Q2 fold). EXPECTED_SHA values
// are captured at GREEN and updated post-GREEN.
// ============================================================

#[test]
fn json_split_g4_anchor_1_sha_pin_with_test_rng_env_var() {
    let f = NamedTempFile::new().unwrap();
    let path = f.path().to_owned();
    drop(f);
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_SLIP39_TEST_RNG", TEST_RNG_HEX_64)
        .env("MNEMONIC_SLIP39_TEST_IDENTIFIER", "12345")
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--iteration-exponent",
            "0",
            "--group-threshold",
            "1",
            "--group",
            "3,2",
            "--json-out",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(out.status.success(), "split must succeed for anchor 1");
    let body = std::fs::read_to_string(&path).unwrap();
    use bitcoin::hashes::{sha256, Hash};
    let h = sha256::Hash::hash(body.as_bytes());
    let actual = format!("{}", h);
    // Captured at GREEN 2026-05-14. ABANDON_12 phrase + identifier
    // 12345 + TEST_RNG seed 0×64 + iteration_exponent 0 + group-thresh
    // 1 + --group 3,2 → SHA below.
    const EXPECTED: &str =
        "df7f6cc9dadb52c51ca2b7889443142dd742e946e61104b76dd3d5e3dac96688";
    assert_eq!(
        actual, EXPECTED,
        "G4 SHA-pin drift anchor 1 (ABANDON_12, identifier=12345, TEST_RNG=0×64); if schema changed intentionally, update EXPECTED"
    );
    // Plan §6 risk 2 mitigation: pin the always-on insecurity advisory.
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains(
            "warning: MNEMONIC_SLIP39_TEST_RNG set — output is deterministic and INSECURE; do not use for real shares"
        ),
        "env-var-wedge SHA-pin tests MUST also pin the insecurity advisory; got: {stderr}"
    );
}

#[test]
fn json_split_g4_anchor_2_sha_pin_different_env_vars() {
    let f = NamedTempFile::new().unwrap();
    let path = f.path().to_owned();
    drop(f);
    let from_arg = format!("phrase={ABANDON_12}");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_SLIP39_TEST_RNG", TEST_RNG_HEX_64_ANCHOR_2)
        .env("MNEMONIC_SLIP39_TEST_IDENTIFIER", "32767")
        .arg("slip39")
        .arg("split")
        .args([
            "--from",
            &from_arg,
            "--iteration-exponent",
            "1",
            "--group-threshold",
            "1",
            "--group",
            "3,2",
            "--json-out",
        ])
        .arg(&path)
        .output()
        .unwrap();
    assert!(out.status.success(), "split must succeed for anchor 2");
    let body = std::fs::read_to_string(&path).unwrap();
    use bitcoin::hashes::{sha256, Hash};
    let h = sha256::Hash::hash(body.as_bytes());
    let actual = format!("{}", h);
    // Captured at GREEN 2026-05-14. ABANDON_12 + identifier 32767
    // + TEST_RNG seed 0x11×64 + iteration_exponent 1 + group-thresh
    // 1 + --group 3,2 → SHA below.
    const EXPECTED: &str =
        "33c28c6b828d7a3c48ff583884f9c65c0efd9616421bf61a64ef571fccad2e7c";
    assert_eq!(
        actual, EXPECTED,
        "G4 SHA-pin drift anchor 2 (ABANDON_12, identifier=32767, TEST_RNG=0x11×64, E=1)"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("MNEMONIC_SLIP39_TEST_RNG set"),
        "anchor 2 insecurity advisory pin; got: {stderr}"
    );
}

#[test]
fn json_split_g4_env_var_yields_deterministic_output() {
    // Run twice with identical env-vars → identical JSON output.
    // (Independent regression-check on the determinism mechanism;
    // does not depend on SHA pinning.)
    let from_arg = format!("phrase={ABANDON_12}");
    let make_run = || {
        let f = NamedTempFile::new().unwrap();
        let path = f.path().to_owned();
        drop(f);
        let out = Command::cargo_bin("mnemonic")
            .unwrap()
            .env("MNEMONIC_SLIP39_TEST_RNG", TEST_RNG_HEX_64)
            .env("MNEMONIC_SLIP39_TEST_IDENTIFIER", "777")
            .arg("slip39")
            .arg("split")
            .args([
                "--from",
                &from_arg,
                "--group-threshold",
                "1",
                "--group",
                "3,2",
                "--json-out",
            ])
            .arg(&path)
            .output()
            .unwrap();
        assert!(out.status.success());
        std::fs::read_to_string(&path).unwrap()
    };
    let body_a = make_run();
    let body_b = make_run();
    assert_eq!(
        body_a, body_b,
        "env-var-wedge must yield byte-identical JSON output across runs"
    );
}
