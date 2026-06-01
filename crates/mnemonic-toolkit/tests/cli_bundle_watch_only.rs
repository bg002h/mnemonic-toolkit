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
    let fp_hex = card.origin_fingerprint.expect("fixture mk1 has origin fp").to_string();

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.xpub={xpub_str}"),
            "--slot",
            &format!("@0.fingerprint={fp_hex}"),
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
    // SPEC v0.6.1 §5.5.a — watch-only invocations (all ms1 == "" sentinel)
    // emit the WatchOnly advisory (not the private-key-material warning).
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("warning: stdout carries private key material"),
        "watch-only bundle must NOT emit the private-key-material warning; got stderr: {stderr:?}"
    );
    assert!(
        stderr.contains("note: stdout is watch-only"),
        "watch-only bundle must emit the watch-only advisory; got stderr: {stderr:?}"
    );
}

/// SPEC v0.6.1 §11 cross-cut at bundle.rs::resolve_slots — `bundle --slot
/// @0.xpub=<zpub>` (template mode) must produce a byte-identical bundle to the
/// equivalent `--slot @0.xpub=<xpub>` invocation. Proves the SLIP-0132 input
/// normalizer is wired in `bundle`, not just `convert`.
#[test]
fn watch_only_bip84_mainnet_accepts_zpub_input_via_slip0132_normalizer() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let mk1_lines: Vec<&str> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .collect();
    let card = mk_codec::decode(&mk1_lines).expect("mk1 decodes");
    let xpub_str = card.xpub.to_string();
    let fp_hex = card.origin_fingerprint.expect("fixture mk1 has origin fp").to_string();

    // The known SLIP-0132 zpub form of the canonical TREZOR_24 BIP-84 mainnet
    // account-level xpub. Pinned independently of the slip0132 module so an
    // accidental change in the prefix table in src would break this test.
    const TREZOR_24_BIP84_MAINNET_ZPUB: &str = "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S";
    assert_eq!(
        xpub_str,
        "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9",
        "fixture xpub stable; if this fails the test vector drifted"
    );

    let from_xpub = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.xpub={xpub_str}"),
            "--slot",
            &format!("@0.fingerprint={fp_hex}"),
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let from_zpub = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--slot",
            &format!("@0.xpub={TREZOR_24_BIP84_MAINNET_ZPUB}"),
            "--slot",
            &format!("@0.fingerprint={fp_hex}"),
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    assert_eq!(
        from_xpub.get_output().stdout,
        from_zpub.get_output().stdout,
        "bundle stdout must be byte-identical regardless of xpub vs. zpub input encoding"
    );
}
