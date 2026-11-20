//! A bare argv value with no `=` on a node-shaped flag (`--from`, `--slot`,
//! `seed-xor combine --share`) used to skip the pre-parser guard (it keys on
//! `<node>=`), reach clap's value parser, and be ECHOED in clap's exit-64
//! "invalid value" error. The guard now refuses it with the same channel text.
//!
//! Test names are the row names.

use assert_cmd::Command;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

const EXIT_REFUSED: i32 = 2;

fn run(args: &[&str]) -> (String, String, i32) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .write_stdin("")
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code().unwrap_or(-1),
    )
}

fn assert_refused_without_echo(args: &[&str], channel: &str) {
    let (stdout, stderr, code) = run(args);
    assert_eq!(
        code, EXIT_REFUSED,
        "expected the argv refusal at exit {EXIT_REFUSED}; stderr:\n{stderr}"
    );
    assert!(stdout.is_empty(), "stdout must be empty:\n{stdout}");
    assert!(
        stderr.contains("Refused BEFORE the command line was parsed"),
        "{stderr}"
    );
    assert!(
        stderr.contains(channel),
        "refusal must name `{channel}`:\n{stderr}"
    );
    assert!(
        !stderr.contains("abandon"),
        "the refusal or clap echoed the secret:\n{stderr}"
    );
}

#[test]
fn seed_xor_combine_shapeless_share_is_refused_without_echo() {
    assert_refused_without_echo(
        &["seed-xor", "combine", "--share", PHRASE, "--share", PHRASE],
        "--share phrase=-",
    );
}

#[test]
fn convert_shapeless_from_is_refused_without_echo() {
    assert_refused_without_echo(
        &["convert", "--from", PHRASE, "--to", "xpub"],
        "--from phrase=-",
    );
}

#[test]
fn inline_equals_shapeless_from_is_refused_without_echo() {
    let from = format!("--from={PHRASE}");
    assert_refused_without_echo(&["convert", &from, "--to", "xpub"], "--from phrase=-");
}

#[test]
fn bundle_shapeless_slot_is_refused_without_echo() {
    assert_refused_without_echo(&["bundle", "--slot", PHRASE], "--slot @N.phrase=-");
}

#[test]
fn verify_bundle_shapeless_from_is_refused_without_echo() {
    assert_refused_without_echo(&["verify-bundle", "--from", PHRASE], "--from phrase=-");
}

#[test]
fn shapeless_private_channel_values_are_not_refused_by_the_guard() {
    for v in ["-", "@env:SOME_VAR"] {
        let (_, stderr, _) = run(&["convert", "--from", v, "--to", "xpub"]);
        assert!(
            !stderr.contains("Refused BEFORE the command line was parsed"),
            "`{v}` is a private channel, not material:\n{stderr}"
        );
    }
}

#[test]
fn word_card_bare_public_card_is_not_refused_by_the_guard() {
    let (_, stderr, _) = run(&["word-card", "--from", "mk1qqqqqqqq"]);
    assert!(
        !stderr.contains("Refused BEFORE the command line was parsed"),
        "word-card --from takes a public bare card:\n{stderr}"
    );
}

#[test]
fn the_override_admits_a_shapeless_value() {
    let (_, stderr, _) = run(&[
        "--allow-argv-secret",
        "convert",
        "--from",
        PHRASE,
        "--to",
        "xpub",
    ]);
    assert!(
        !stderr.contains("Refused BEFORE the command line was parsed"),
        "{stderr}"
    );
}
