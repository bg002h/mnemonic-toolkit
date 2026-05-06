//! Verify-bundle watch-only round-trip integration test.
//!
//! Confirms SPEC §2.2.2 stderr warning is emitted alongside the round-trip
//! "result: ok" verdict.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn verify_bundle_watch_only_bip84_mainnet_round_trip() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let mk1_lines: Vec<&str> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .collect();
    let md1_lines: Vec<&str> = fixture
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' ') && !l.contains('-'))
        .collect();
    assert!(!mk1_lines.is_empty() && !md1_lines.is_empty());

    let card = mk_codec::decode(&mk1_lines).expect("mk1 decodes");
    let xpub_str = card.xpub.to_string();

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--xpub".into(),
        xpub_str,
        "--master-fingerprint".into(),
        "5436d724".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
    ];
    for s in &mk1_lines {
        args.push("--mk1".into());
        args.push((*s).into());
    }
    for s in &md1_lines {
        args.push("--md1".into());
        args.push((*s).into());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success()
        .stdout(predicate::str::contains("result: ok"))
        .stderr(predicate::str::contains(
            "watch-only verify-bundle does not verify",
        ));
}

// v0.5 SPEC §5.7 case 1: watch-only with user-supplied (spurious) --ms1.
// The helper's watch-only short-circuit absorbs supplied --ms1 silently; the
// run still reports result: ok (closes FOLLOWUP verify-bundle-watch-only-
// spurious-ms1-handling).
#[test]
fn verify_bundle_watch_only_spurious_ms1_silently_absorbed_v0_5() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let mk1_lines: Vec<&str> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .collect();
    let md1_lines: Vec<&str> = fixture
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' ') && !l.contains('-'))
        .collect();
    let ms1_line: &str = fixture
        .lines()
        .find(|l| l.starts_with("ms1") && !l.contains(' '))
        .expect("compact ms1 line in fixture");

    let card = mk_codec::decode(&mk1_lines).expect("mk1 decodes");
    let xpub_str = card.xpub.to_string();

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--xpub".into(),
        xpub_str,
        "--master-fingerprint".into(),
        "5436d724".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
        // Spurious --ms1 supply — should be silently absorbed in watch-only mode.
        "--ms1".into(),
        ms1_line.into(),
    ];
    for s in &mk1_lines {
        args.push("--mk1".into());
        args.push((*s).into());
    }
    for s in &md1_lines {
        args.push("--md1".into());
        args.push((*s).into());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success()
        .stdout(predicate::str::contains("result: ok"));
}
