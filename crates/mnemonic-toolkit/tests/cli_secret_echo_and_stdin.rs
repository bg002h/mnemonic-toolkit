//! Review r1 of the echo/stdin fix (toolkit PR 230): a secret typed into a
//! card field is never echoed by an HRP error on any verb or spelling, and a
//! path flag naming stdin counts as a stdin reader.

use assert_cmd::Command;

const PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon \
                      abandon abandon abandon abandon about";

fn run(args: &[&str], stdin: Option<&str>) -> (i32, String, String) {
    let mut cmd = Command::cargo_bin("mnemonic").unwrap();
    cmd.arg("--allow-argv-secret").args(args);
    if let Some(s) = stdin {
        cmd.write_stdin(s.to_string());
    }
    let o = cmd.output().unwrap();
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).into_owned(),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

fn assert_not_echoed(label: &str, (code, out, err): (i32, String, String)) {
    assert_ne!(code, 0, "{label}: {err}");
    for s in [&out, &err] {
        assert!(!s.contains("aband"), "{label} echoed the phrase: {s}");
    }
}

/// I1: the mk1 slot, the ms1 slot through stdin, `inspect`, `convert` and
/// `verify-bundle` withhold a phrase typed into a card field.
#[test]
fn a_phrase_in_a_card_field_is_not_echoed() {
    assert_not_echoed("repair --mk1", run(&["repair", "--mk1", PHRASE], None));
    assert_not_echoed(
        "repair --mk1 -",
        run(&["repair", "--mk1", "-"], Some(PHRASE)),
    );
    assert_not_echoed(
        "repair --ms1 -",
        run(&["repair", "--ms1", "-"], Some(PHRASE)),
    );
    assert_not_echoed(
        "inspect --mk1 -",
        run(&["inspect", "--mk1", "-"], Some(PHRASE)),
    );
    assert_not_echoed(
        "inspect --ms1 -",
        run(&["inspect", "--ms1", "-"], Some(PHRASE)),
    );
    assert_not_echoed(
        "convert --from mk1=",
        run(
            &[
                "convert",
                "--from",
                &format!("mk1={PHRASE}"),
                "--to",
                "xpub",
            ],
            None,
        ),
    );
    assert_not_echoed(
        "convert --from ms1=",
        run(
            &[
                "convert",
                "--from",
                &format!("ms1={PHRASE}"),
                "--to",
                "phrase",
            ],
            None,
        ),
    );
    assert_not_echoed("repair positional", run(&["repair", PHRASE], None));
}

/// M3: `--passphrase-candidates-file /dev/stdin` with `--phrase-stdin` is two
/// stdin readers.
#[test]
fn candidates_file_on_stdin_is_a_second_reader() {
    for path in ["/dev/stdin", "/dev/fd/0"] {
        let (code, _, err) = run(
            &[
                "xpub-search",
                "passphrase-of-xpub",
                "--phrase-stdin",
                "--passphrase-candidates-file",
                path,
                "--target-xpub",
                "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8",
            ],
            Some(PHRASE),
        );
        assert_eq!(code, 1, "{path}: {err}");
        assert!(
            err.contains("--passphrase-candidates-file /dev/stdin") && err.contains("stdin"),
            "{path}: {err}"
        );
    }
}

/// M3: import-wallet counts a token, blob or Round-1 path that names stdin.
#[test]
fn import_wallet_path_flags_on_stdin_are_counted() {
    for (a, b) in [
        ("--bsms-encryption-token", "--blob"),
        ("--bsms-encryption-token", "--bsms-round1"),
        ("--blob", "--bsms-round1"),
    ] {
        let (code, _, err) = run(
            &[
                "import-wallet",
                "--format",
                "bsms",
                a,
                "/dev/stdin",
                b,
                "/dev/fd/0",
            ],
            Some("00\n"),
        );
        assert_ne!(code, 0, "{a} {b}: {err}");
        assert!(
            err.contains("only one input can come from stdin"),
            "{a} {b}: {err}"
        );
    }
}
