//! F-688 / F-351 / F-353 — the last argv secrets are REFUSED, not warned about.
//!
//! Before this change `ms-shares combine --share`, `slip39 combine --share`,
//! `seed-xor combine --share <node>=`, `import-wallet --ms1` and
//! `verify-bundle --ms1` ran with the secret on argv and printed only a
//! warning, unlike every other argv secret in the pre-parser guard table
//! (`argv_guard`). F-351 / F-353 had parked them because the stated remedy
//! (`--share -`) reads ONE record. The existing `@env:VAR` sentinel is the
//! per-record private channel that resolves for every one of these flags, so
//! the refusal names `-` (one record) plus `@env:VAR` (each further record).
//!
//! Test names are the row names.

use assert_cmd::Command;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// A well-formed-looking ms1 string. The refusal fires before any parse, so
/// the material never has to decode; it only has to be distinctive.
const MS1_DUMMY: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

const EXIT_REFUSED: i32 = 2;

fn run_with(args: &[&str], stdin: &str, envs: &[(&str, &str)]) -> (String, String, i32) {
    let mut cmd = Command::cargo_bin("mnemonic").unwrap();
    cmd.args(args).write_stdin(stdin.to_string());
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let out = cmd.output().unwrap();
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code().unwrap_or(-1),
    )
}

fn run(args: &[&str]) -> (String, String, i32) {
    run_with(args, "", &[])
}

/// Run with the global override so the test can mint real shares.
fn mint(args: &[&str]) -> Vec<String> {
    let mut v = vec!["--allow-argv-secret"];
    v.extend_from_slice(args);
    let (out, err, code) = run(&v);
    assert_eq!(code, 0, "mint {args:?} failed: {err}");
    out.lines().map(str::to_string).collect()
}

fn assert_refused(args: &[&str], material: &str, channel: &str, what: &str) {
    let (stdout, stderr, code) = run(args);
    assert_eq!(
        code, EXIT_REFUSED,
        "{what}: expected the argv refusal at exit {EXIT_REFUSED}; stderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "{what}: stdout must be empty:\n{stdout}");
    assert!(
        stderr.contains("Refused BEFORE the command line was parsed"),
        "{what}: {stderr}"
    );
    assert!(
        stderr.contains(channel),
        "{what}: refusal must name `{channel}`:\n{stderr}"
    );
    assert!(
        !stderr.contains(material),
        "{what}: the refusal echoed the secret:\n{stderr}"
    );
}

fn ms_shares() -> Vec<String> {
    mint(&[
        "ms-shares",
        "split",
        "--from",
        &format!("phrase={PHRASE}"),
        "--threshold",
        "2",
        "--shares",
        "3",
    ])
}

fn slip39_shares() -> Vec<String> {
    mint(&[
        "slip39",
        "split",
        "--from",
        &format!("phrase={PHRASE}"),
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ])
    .into_iter()
    .filter(|l| !l.is_empty())
    .collect()
}

fn xor_shares() -> Vec<String> {
    mint(&[
        "seed-xor",
        "split",
        "--from",
        &format!("phrase={PHRASE}"),
        "--shares",
        "3",
    ])
}

// ---------------------------------------------------------------------------
// RED rows: the argv form is refused
// ---------------------------------------------------------------------------

#[test]
fn ms_shares_combine_share_on_argv_is_refused() {
    let s = ms_shares();
    assert_refused(
        &["ms-shares", "combine", "--share", &s[0], "--share", &s[1]],
        &s[0],
        "--share @env:VAR",
        "ms-shares combine --share",
    );
}

#[test]
fn slip39_combine_share_on_argv_is_refused() {
    let s = slip39_shares();
    assert_refused(
        &["slip39", "combine", "--share", &s[0], "--share", &s[1]],
        &s[0],
        "--share @env:VAR",
        "slip39 combine --share",
    );
}

#[test]
fn seed_xor_combine_share_on_argv_is_refused() {
    let s = xor_shares();
    let a = format!("phrase={}", s[0]);
    let b = format!("phrase={}", s[1]);
    let c = format!("phrase={}", s[2]);
    assert_refused(
        &[
            "seed-xor", "combine", "--shares", "3", "--share", &a, "--share", &b, "--share", &c,
        ],
        &s[0],
        "--share phrase=@env:VAR",
        "seed-xor combine --share",
    );
}

#[test]
fn import_wallet_ms1_on_argv_is_refused() {
    assert_refused(
        &[
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--ms1",
            MS1_DUMMY,
        ],
        MS1_DUMMY,
        "--ms1 @env:VAR",
        "import-wallet --ms1",
    );
}

#[test]
fn verify_bundle_ms1_on_argv_is_refused() {
    assert_refused(
        &[
            "verify-bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--ms1",
            MS1_DUMMY,
        ],
        MS1_DUMMY,
        "--ms1 -",
        "verify-bundle --ms1",
    );
}

// ---------------------------------------------------------------------------
// GREEN controls: the channels the refusal names resolve and are not refused
// ---------------------------------------------------------------------------

#[test]
fn ms_shares_combine_env_and_stdin_channels_recover_the_phrase() {
    let s = ms_shares();
    let (out, err, code) = run_with(
        &[
            "ms-shares",
            "combine",
            "--share",
            "-",
            "--share",
            "@env:F688_S1",
        ],
        &s[0],
        &[("F688_S1", &s[1])],
    );
    assert_eq!(code, 0, "{err}");
    assert_eq!(out.lines().next(), Some(PHRASE));
    assert!(!err.contains("Refused"), "{err}");
}

#[test]
fn slip39_combine_env_and_stdin_channels_recover_the_phrase() {
    let s = slip39_shares();
    let (out, err, code) = run_with(
        &[
            "slip39",
            "combine",
            "--share",
            "-",
            "--share",
            "@env:F688_S1",
        ],
        &s[0],
        &[("F688_S1", &s[1])],
    );
    assert_eq!(code, 0, "{err}");
    // slip39 combine prints the master secret as hex; the abandon phrase's
    // 128-bit entropy is all zeros.
    assert_eq!(out.lines().next(), Some("00000000000000000000000000000000"));
}

#[test]
fn seed_xor_combine_env_and_stdin_channels_recover_the_phrase() {
    let s = xor_shares();
    let (out, err, code) = run_with(
        &[
            "seed-xor",
            "combine",
            "--shares",
            "3",
            "--share",
            "phrase=-",
            "--share",
            "phrase=@env:F688_S1",
            "--share",
            "phrase=@env:F688_S2",
        ],
        &s[0],
        &[("F688_S1", &s[1]), ("F688_S2", &s[2])],
    );
    assert_eq!(code, 0, "{err}");
    assert_eq!(out.lines().next(), Some(PHRASE));
}

#[test]
fn ms1_env_channel_is_not_refused_on_import_wallet_and_verify_bundle() {
    for args in [
        vec![
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bsms",
            "--ms1",
            "@env:F688_MS1",
        ],
        vec![
            "verify-bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--ms1",
            "@env:F688_MS1",
        ],
    ] {
        let (_, err, _) = run_with(&args, "", &[("F688_MS1", MS1_DUMMY)]);
        assert!(
            !err.contains("Refused"),
            "{args:?} refused its own channel: {err}"
        );
        assert!(!err.contains(MS1_DUMMY), "{args:?} echoed the card: {err}");
    }
}

#[test]
fn verify_bundle_ms1_stdin_channel_is_not_refused() {
    let (_, err, _) = run_with(
        &[
            "verify-bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--ms1",
            "-",
        ],
        MS1_DUMMY,
        &[],
    );
    assert!(!err.contains("Refused"), "{err}");
}

#[test]
fn the_override_still_admits_every_refused_form() {
    let s = ms_shares();
    let (out, err, code) = run(&[
        "--allow-argv-secret",
        "ms-shares",
        "combine",
        "--share",
        &s[0],
        "--share",
        &s[1],
    ]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out.lines().next(), Some(PHRASE));
}

/// The replacement the refusal names must itself be clean: `@env:VAR` and `-`
/// are private channels, so the tool must not then warn that the secret is on
/// argv (it warned before, because it judged the value after resolution).
#[test]
fn the_named_channels_do_not_warn_about_argv() {
    let s = ms_shares();
    let (_, err, code) = run_with(
        &[
            "ms-shares",
            "combine",
            "--share",
            "-",
            "--share",
            "@env:F688_S1",
        ],
        &s[0],
        &[("F688_S1", &s[1])],
    );
    assert_eq!(code, 0, "{err}");
    assert!(!err.contains("secret material on argv"), "{err}");

    let s = slip39_shares();
    let (_, err, code) = run_with(
        &[
            "slip39",
            "combine",
            "--share",
            "-",
            "--share",
            "@env:F688_S1",
        ],
        &s[0],
        &[("F688_S1", &s[1])],
    );
    assert_eq!(code, 0, "{err}");
    assert!(!err.contains("secret material on argv"), "{err}");

    let s = xor_shares();
    let (_, err, code) = run_with(
        &[
            "seed-xor",
            "combine",
            "--shares",
            "3",
            "--share",
            "phrase=-",
            "--share",
            "phrase=@env:F688_S1",
            "--share",
            "phrase=@env:F688_S2",
        ],
        &s[0],
        &[("F688_S1", &s[1]), ("F688_S2", &s[2])],
    );
    assert_eq!(code, 0, "{err}");
    assert!(!err.contains("secret material on argv"), "{err}");
}
