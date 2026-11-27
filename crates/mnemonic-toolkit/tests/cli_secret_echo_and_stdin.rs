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

/// Hex entropy with a `1` in it: its prefix (`3a`) reaches the HRP gate, as
/// a phrase (no `1`, spaces) mostly does not.
const HEX: &str = "3a1c9e5b7d2f40816c9ab3e4f5d60718";

fn assert_withheld(label: &str, needle: &str, (code, out, err): (i32, String, String)) {
    assert_ne!(code, 0, "{label}: {err}");
    for s in [&out, &err] {
        assert!(!s.contains(needle), "{label} echoed the secret: {s}");
    }
    assert!(
        out.contains("withheld") || err.contains("withheld"),
        "{label} did not reach the HRP diagnostic: {err}"
    );
}

/// I1 (review r1) and M2 (final review): a secret typed into a card field
/// reaches the HRP diagnostic and is withheld, on argv and stdin, for the
/// mk1 and ms1 slots, `inspect`, `convert` and a positional.
#[test]
fn a_secret_in_a_card_field_is_not_echoed() {
    // A phrase reaches the mk1 decoder's InvalidHrp (the whole input) and the
    // positional UnknownHrp.
    assert_withheld(
        "repair --mk1 phrase",
        "aband",
        run(&["repair", "--mk1", PHRASE], None),
    );
    assert_withheld(
        "inspect --mk1 - phrase",
        "aband",
        run(&["inspect", "--mk1", "-"], Some(PHRASE)),
    );
    assert_withheld(
        "convert --from mk1=phrase",
        "aband",
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
    assert_withheld(
        "repair positional phrase",
        "aband",
        run(&["repair", PHRASE], None),
    );
    // Hex with a `1` reaches the flag gate and repair's HrpMismatch.
    for (label, args, stdin) in [
        (
            "repair --ms1 - hex",
            vec!["repair", "--ms1", "-"],
            Some(HEX),
        ),
        (
            "repair --mk1 - hex",
            vec!["repair", "--mk1", "-"],
            Some(HEX),
        ),
        (
            "inspect --mk1 - hex",
            vec!["inspect", "--mk1", "-"],
            Some(HEX),
        ),
        ("repair --ms1 hex", vec!["repair", "--ms1", HEX], None),
        ("repair positional hex", vec!["repair", HEX], None),
    ] {
        assert_withheld(label, "3a1c", run(&args, stdin));
    }
}

/// M2 (final review): verify-bundle's decode rows render the friendly
/// message, which withholds a secret found in a card slot of the bundle.
#[test]
fn verify_bundle_decode_rows_withhold_a_secret() {
    let seed = "@0.phrase=abandon abandon abandon abandon abandon abandon abandon \
                abandon abandon abandon abandon about";
    let clean = run(
        &[
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            seed,
            "--account",
            "0",
            "--json",
            "--no-engraving-card",
        ],
        None,
    );
    assert_eq!(clean.0, 0, "{}", clean.2);
    for (field, value) in [("mk1", "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong")] {
        let mut v: serde_json::Value = serde_json::from_str(&clean.1).unwrap();
        v[field] = serde_json::json!([value]);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("b.json");
        std::fs::write(&path, serde_json::to_string(&v).unwrap()).unwrap();
        for json in [false, true] {
            let mut args = vec![
                "--no-auto-repair",
                "verify-bundle",
                "--network",
                "mainnet",
                "--template",
                "bip84",
                "--slot",
                seed,
                "--bundle-json",
                path.to_str().unwrap(),
            ];
            if json {
                args.push("--json");
            }
            assert_withheld(
                &format!("verify-bundle {field} json={json}"),
                "zoo",
                run(&args, None),
            );
        }
    }
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

/// I1 (final review): `restore --cosigner` withholds a value with no `=`
/// (a seed card or phrase pasted into the cosigner field).
#[test]
fn restore_cosigner_withholds_a_bare_value() {
    let seed = format!("@0.phrase={PHRASE}");
    let (code, out, err) = run(
        &[
            "bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            &seed,
            "--account",
            "0",
            "--json",
            "--no-engraving-card",
        ],
        None,
    );
    assert_eq!(code, 0, "{err}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let mut base: Vec<String> = vec![
        "restore".into(),
        "--from".into(),
        format!("phrase={PHRASE}"),
    ];
    for m in v["md1"].as_array().unwrap() {
        base.push("--md1".into());
        base.push(m.as_str().unwrap().into());
    }
    for (value, needle) in [
        (
            "ms10entrsqplh7lml0alh7lml0alh7lml0als5cclar2zmksh6",
            "ms10entrs",
        ),
        ("@1legal winner thank year wave sausage", "legal"),
    ] {
        let mut a: Vec<&str> = base.iter().map(String::as_str).collect();
        a.extend(["--cosigner", value]);
        let (code, out, err) = run(&a, None);
        assert_ne!(code, 0, "{err}");
        assert!(err.contains("input withheld"), "{err}");
        assert!(!err.contains(needle) && !out.contains(needle), "{err}");
    }
}

/// M3 (final review): bundle counts a stdin slot against a path flag that
/// names stdin; verify-bundle counts `--from <source>=-` likewise.
#[test]
fn bundle_and_verify_bundle_count_every_stdin_reader() {
    for path_flag in ["--descriptor-file", "--import-json"] {
        let (code, _, err) = run(
            &[
                "bundle",
                "--network",
                "mainnet",
                path_flag,
                "/dev/stdin",
                "--slot",
                "@0.phrase=-",
            ],
            Some(PHRASE),
        );
        assert_ne!(code, 0, "{path_flag}: {err}");
        assert!(
            err.contains("only one input can come from stdin"),
            "{path_flag}: {err}"
        );
    }
    let (code, _, err) = run(
        &[
            "verify-bundle",
            "--network",
            "mainnet",
            "--from",
            "phrase=-",
            "--bundle-json",
            "/dev/stdin",
        ],
        Some(PHRASE),
    );
    assert_ne!(code, 0, "{err}");
    assert!(
        err.contains("--from <source>=-") && err.contains("only one input"),
        "{err}"
    );
}
