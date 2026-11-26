//! Final review M2 (spec R2): a setting is judged before any input is read,
//! so a bad setting is 64 whatever the input is (good, bad or not UTF-8).

use assert_cmd::Command;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Inputs every row is run against: none read at all would matter.
fn inputs() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("a valid phrase", format!("{PHRASE}\n").into_bytes()),
        ("garbage", b"garbage\n".to_vec()),
        ("not UTF-8", b"\xff\xfe\n".to_vec()),
    ]
}

fn assert_64_for_every_input(name: &str, args: &[&str]) {
    for (what, bytes) in inputs() {
        let out = Command::cargo_bin("mnemonic")
            .expect("binary built")
            .args(args)
            .write_stdin(bytes)
            .output()
            .expect("spawn");
        assert_eq!(
            out.status.code(),
            Some(64),
            "{name} with {what} on stdin: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            out.stdout.is_empty(),
            "{name} with {what}: nothing on stdout"
        );
    }
}

#[test]
fn final_m2_ms_shares_split_threshold_and_shares_are_64_whatever_the_input() {
    for (t, n) in [("1", "3"), ("3", "2"), ("10", "12"), ("2", "32")] {
        for from in ["phrase=-", "entropy=-"] {
            assert_64_for_every_input(
                &format!("ms-shares split --from {from} --threshold {t} --shares {n}"),
                &[
                    "ms-shares",
                    "split",
                    "--from",
                    from,
                    "--threshold",
                    t,
                    "--shares",
                    n,
                ],
            );
        }
    }
}

#[test]
fn final_m2_derive_child_length_is_64_whatever_the_input() {
    for (app, len) in [
        ("bip39", "13"),
        ("hd-seed", "3"),
        ("xprv", "3"),
        ("hex", "15"),
        ("password-base64", "19"),
        ("password-base85", "81"),
        ("dice", "0"),
    ] {
        assert_64_for_every_input(
            &format!("derive-child --application {app} --length {len}"),
            &[
                "derive-child",
                "--from",
                "phrase=-",
                "--application",
                app,
                "--length",
                len,
                "--index",
                "0",
            ],
        );
    }
    // dice without --dice-sides is a missing setting, not an input failure.
    assert_64_for_every_input(
        "derive-child --application dice without --dice-sides",
        &[
            "derive-child",
            "--from",
            "phrase=-",
            "--application",
            "dice",
            "--length",
            "5",
            "--index",
            "0",
        ],
    );
}

/// The R2 audit's finds (final review M2): settings that were judged only
/// after the input was read.
#[test]
fn final_m2_audit_settings_are_64_whatever_the_input() {
    assert_64_for_every_input(
        "derive-child --index 2^31",
        &[
            "derive-child",
            "--from",
            "phrase=-",
            "--application",
            "bip39",
            "--length",
            "12",
            "--index",
            "2147483648",
        ],
    );
    assert_64_for_every_input(
        "addresses --range 5,2",
        &[
            "addresses",
            "--from",
            "phrase=-",
            "--address-type",
            "p2wpkh",
            "--range",
            "5,2",
        ],
    );
    assert_64_for_every_input(
        "addresses --from xpub=- --account 1",
        &[
            "addresses",
            "--from",
            "xpub=-",
            "--address-type",
            "p2wpkh",
            "--account",
            "1",
        ],
    );
    for group in [
        &["--group", "2,3", "--group", "2,3", "--group-threshold", "3"][..],
        &["--group", "3,1", "--group-threshold", "1"][..],
        &[
            "--group",
            "3,2",
            "--group-threshold",
            "1",
            "--iteration-exponent",
            "16",
        ][..],
    ] {
        let mut a = vec!["slip39", "split", "--from", "phrase=-"];
        a.extend_from_slice(group);
        assert_64_for_every_input(&format!("slip39 split {group:?}"), &a);
    }
}
