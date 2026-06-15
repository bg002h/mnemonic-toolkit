//! mstring-grouping P4: the toolkit's md1/ms1 intake surfaces accept a
//! comma-grouped card (comma is a SPEC §3.2 separator the codecs do NOT
//! tolerate, so it genuinely exercises `display_grouping::strip_display_separators`).

use assert_cmd::Command;

const ZERO_ENTROPY_16: &str = "00000000000000000000000000000000";

/// Insert a comma every 5 chars.
fn comma5(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && i % 5 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Unbroken ms1 for 16 zero bytes (via convert --group-size 0).
fn unbroken_ms1() -> String {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("entropy={ZERO_ENTROPY_16}"),
            "--to",
            "ms1",
            "--group-size",
            "0",
        ])
        .output()
        .unwrap();
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .find_map(|l| l.strip_prefix("ms1: "))
        .expect("ms1 line")
        .to_string()
}

#[test]
fn convert_from_ms1_accepts_comma_grouped() {
    let grouped = comma5(&unbroken_ms1());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("ms1={grouped}"),
            "--to",
            "entropy",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains(ZERO_ENTROPY_16));
}

#[test]
fn bundle_slot_ms1_accepts_comma_grouped() {
    // @N.ms1= slot intake routes through slot_ms1::resolve_ms1_slot.
    let grouped = comma5(&unbroken_ms1());
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.ms1={grouped}"),
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--no-engraving-card",
        ])
        .assert()
        .success();
}
