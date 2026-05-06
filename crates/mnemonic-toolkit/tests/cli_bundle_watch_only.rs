//! Watch-only bundle integration test.
//!
//! Decodes the bip84/mainnet fixture's mk1 to recover the xpub, then runs
//! `bundle --slot @0.xpub= --slot @0.fingerprint=` and confirms the ms1
//! section is the pinned omitted-marker.

use assert_cmd::Command;

#[test]
fn watch_only_bip84_mainnet_omits_ms1_section() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    // Compact-form mk1 strings (no spaces, no dashes).
    let mk1_lines: Vec<&str> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .collect();
    assert!(!mk1_lines.is_empty(), "fixture must contain mk1 lines");

    let card = mk_codec::decode(&mk1_lines).expect("mk1 decodes");
    let xpub_str = card.xpub.to_string();

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.xpub={xpub_str}"),
            "--slot",
            "@0.fingerprint=5436d724",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("# ms1 (omitted — xpub-only mode)"),
        "watch-only stdout must contain the ms1 omitted-marker, got:\n{}",
        stdout
    );
    assert!(stdout.contains("# mk1"), "stdout must contain # mk1 header");
    assert!(stdout.contains("# md1"), "stdout must contain # md1 header");
}
