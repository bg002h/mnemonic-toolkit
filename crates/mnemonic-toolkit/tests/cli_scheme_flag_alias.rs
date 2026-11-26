//! The named-wallet-scheme flag is `--scheme` (2026-11-26 rename). The old
//! spellings `--template` (bundle, verify-bundle, convert, export-wallet,
//! restore) and `--archetype` (build-descriptor) stay accepted as HIDDEN
//! aliases: each must give byte-identical stdout, stderr and exit code to the
//! new spelling, and `--help` must show `--scheme` and not the old spelling.

use assert_cmd::Command;

const A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const K1: &str = "[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";
const K2: &str = "[22222222/48h/0h/0h/2h]xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";

#[derive(Debug, PartialEq, Eq)]
struct Out {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Out {
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args(args)
        .output()
        .unwrap();
    Out {
        code: o.status.code().unwrap_or(-1),
        stdout: String::from_utf8(o.stdout).unwrap(),
        stderr: String::from_utf8(o.stderr).unwrap(),
    }
}

/// Runs `args` with `{flag}` replaced by `--scheme`, then by `old`, and
/// asserts the two runs are byte-identical. Returns the `--scheme` run.
fn assert_alias_identical(args: &[&str], old: &str) -> Out {
    let with = |f: &str| -> Vec<String> {
        args.iter()
            .map(|a| {
                if *a == "{flag}" {
                    f.to_string()
                } else {
                    a.to_string()
                }
            })
            .collect()
    };
    let new_args = with("--scheme");
    let old_args = with(old);
    let new = run(&new_args.iter().map(String::as_str).collect::<Vec<_>>());
    let old_out = run(&old_args.iter().map(String::as_str).collect::<Vec<_>>());
    assert_eq!(new, old_out, "{old} must be an exact alias of --scheme");
    new
}

/// `--help` lists `--scheme` and never prints the old spelling.
fn assert_help_shows_scheme_only(verb: &str, old: &str) {
    let o = run(&[verb, "--help"]);
    assert_eq!(o.code, 0, "{verb} --help: {}", o.stderr);
    assert!(
        o.stdout.contains("--scheme <SCHEME>"),
        "{verb} --help must list --scheme:\n{}",
        o.stdout
    );
    assert!(
        !o.stdout.contains(old),
        "{verb} --help must not print the hidden alias {old}:\n{}",
        o.stdout
    );
}

#[test]
fn bundle_template_is_a_hidden_alias_of_scheme() {
    let slot = format!("@0.phrase={A}");
    let o = assert_alias_identical(
        &[
            "bundle",
            "--network",
            "mainnet",
            "{flag}",
            "bip84",
            "--slot",
            &slot,
            "--json",
        ],
        "--template",
    );
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_help_shows_scheme_only("bundle", "--template");
}

#[test]
fn verify_bundle_template_is_a_hidden_alias_of_scheme() {
    let slot = format!("@0.phrase={A}");
    let bundle = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--scheme",
        "bip84",
        "--slot",
        &slot,
        "--json",
    ]);
    assert_eq!(bundle.code, 0, "{}", bundle.stderr);
    let v: serde_json::Value = serde_json::from_str(&bundle.stdout).unwrap();
    let strs = |k: &str| -> Vec<String> {
        v[k].as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect()
    };
    let mut args: Vec<String> = [
        "verify-bundle",
        "--network",
        "mainnet",
        "{flag}",
        "bip84",
        "--slot",
        &slot,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for (flag, key) in [("--ms1", "ms1"), ("--md1", "md1"), ("--mk1", "mk1")] {
        for s in strs(key) {
            args.push(flag.to_string());
            args.push(s);
        }
    }
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let o = assert_alias_identical(&args, "--template");
    assert_eq!(o.code, 0, "{}{}", o.stdout, o.stderr);
    assert_help_shows_scheme_only("verify-bundle", "--template");
}

#[test]
fn convert_template_is_a_hidden_alias_of_scheme() {
    let from = format!("phrase={A}");
    let o = assert_alias_identical(
        &[
            "convert", "--from", &from, "--to", "xpub", "{flag}", "bip84",
        ],
        "--template",
    );
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_help_shows_scheme_only("convert", "--template");
}

#[test]
fn export_wallet_template_is_a_hidden_alias_of_scheme() {
    let o = run(&[
        "convert",
        "--from",
        &format!("phrase={A}"),
        "--to",
        "xpub",
        "--scheme",
        "bip84",
    ]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    let xpub = o
        .stdout
        .trim()
        .strip_prefix("xpub: ")
        .expect("xpub: line")
        .to_string();
    let slot = format!("@0.xpub={xpub}");
    let o = assert_alias_identical(
        &[
            "export-wallet",
            "{flag}",
            "bip84",
            "--slot",
            &slot,
            "--format",
            "descriptor",
        ],
        "--template",
    );
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_help_shows_scheme_only("export-wallet", "--template");
}

#[test]
fn restore_template_is_a_hidden_alias_of_scheme() {
    let from = format!("phrase={A}");
    let o = assert_alias_identical(
        &["restore", "--from", &from, "{flag}", "bip84"],
        "--template",
    );
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_help_shows_scheme_only("restore", "--template");
}

#[test]
fn build_descriptor_archetype_is_a_hidden_alias_of_scheme() {
    let o = assert_alias_identical(
        &[
            "build-descriptor",
            "{flag}",
            "simple-timelocked-inheritance",
            "--key",
            K1,
            "--recovery-key",
            K2,
            "--older",
            "65535",
        ],
        "--archetype",
    );
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert_help_shows_scheme_only("build-descriptor", "--archetype");
}

/// A runtime refusal about a missing scheme names `--scheme`, not `--template`.
#[test]
fn remedies_name_scheme() {
    let o = run(&["export-wallet", "--format", "descriptor"]);
    assert_ne!(o.code, 0);
    assert!(o.stderr.contains("--scheme"), "{}", o.stderr);
    assert!(!o.stderr.contains("--template"), "{}", o.stderr);
}

/// `xpub-search --add-path` takes a derivation path pattern, not a template.
#[test]
fn xpub_search_add_path_value_name_is_path() {
    for mode in [
        "path-of-xpub",
        "passphrase-of-xpub",
        "account-of-descriptor",
    ] {
        let o = run(&["xpub-search", mode, "--help"]);
        assert_eq!(o.code, 0, "{mode}: {}", o.stderr);
        assert!(
            o.stdout.contains("--add-path <PATH>"),
            "{mode}:\n{}",
            o.stdout
        );
        assert!(!o.stdout.contains("TEMPLATE"), "{mode}:\n{}", o.stdout);
    }
}

/// `gui-schema` lists `--scheme` and none of the hidden aliases.
#[test]
fn gui_schema_lists_scheme_not_aliases() {
    let o = run(&["gui-schema"]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    let v: serde_json::Value = serde_json::from_str(&o.stdout).unwrap();
    for sub in v["subcommands"].as_array().unwrap() {
        let name = sub["name"].as_str().unwrap();
        let flags: Vec<&str> = sub["flags"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["name"].as_str().unwrap())
            .collect();
        for old in ["template", "--template", "archetype", "--archetype"] {
            assert!(!flags.contains(&old), "{name}: lists {old}: {flags:?}");
        }
        if [
            "bundle",
            "verify-bundle",
            "convert",
            "export-wallet",
            "restore",
            "build-descriptor",
        ]
        .contains(&name)
        {
            assert!(
                flags.iter().any(|f| f.trim_start_matches('-') == "scheme"),
                "{name}: {flags:?}"
            );
        }
    }
    assert!(
        !o.stdout.contains("\"--template\""),
        "conditional rules must name --scheme"
    );
}

/// Error paths: under the old spelling a refusal is byte-identical (stdout,
/// stderr, exit code) to the same refusal under `--scheme`, and names `--scheme`.
#[test]
fn refusals_are_identical_under_the_alias() {
    let cases: &[(&[&str], &str)] = &[
        // clap conflict: a scheme beside --descriptor.
        (
            &[
                "export-wallet",
                "{flag}",
                "bip84",
                "--descriptor",
                "wpkh(@0/<0;1>/*)",
            ],
            "--template",
        ),
        // clap value error: an unknown scheme name.
        (
            &["bundle", "--network", "mainnet", "{flag}", "bogus"],
            "--template",
        ),
        // clap conflict: a scheme beside --spec.
        (
            &[
                "build-descriptor",
                "{flag}",
                "kofn-recovery",
                "--spec",
                "x.json",
            ],
            "--archetype",
        ),
        // runtime refusal: a multisig scheme for single-sig restore.
        (
            &["restore", "--from", "phrase=-", "{flag}", "wsh-sortedmulti"],
            "--template",
        ),
    ];
    for (args, old) in cases {
        let o = assert_alias_identical(args, old);
        assert_ne!(o.code, 0, "{args:?} must be refused: {}", o.stdout);
        assert!(o.stderr.contains("--scheme"), "{args:?}: {}", o.stderr);
        assert!(!o.stderr.contains(old), "{args:?}: {}", o.stderr);
    }
}
