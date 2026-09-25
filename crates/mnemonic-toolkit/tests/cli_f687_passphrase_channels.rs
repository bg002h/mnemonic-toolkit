//! F-687 — `--passphrase -` is stdin and `--passphrase @env:VAR` is the
//! environment, on EVERY subcommand that declares `--passphrase`; a literal
//! argv passphrase keeps working with exactly one stderr note.
//!
//! Before F-687 (toolkit 0.104.0, measured): `--passphrase -` was refused by
//! the argv guard, and under `--allow-argv-secret` derived with the literal
//! one-character passphrase `-` at exit 0 on all twelve subcommands (exit 4, a
//! false mismatch, on `verify-bundle` and the three `xpub-search` modes);
//! `@env:VAR` was literal on `silent-payment`; `@env:VAR` kept a trailing
//! newline that the stdin forms strip; and `verify-bundle`'s keyless-template
//! path re-read an already-drained stdin for `--passphrase-stdin` (a false NO
//! MATCH, exit 4, on a matching bundle).
//!
//! The byte rule itself is pinned by `tests/vectors/passphrase_channels.json`,
//! which mnemonic-secret carries byte-identically.

use assert_cmd::Command;
use serde_json::Value;

const SEED: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const SEED_B: &str =
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";
const PW: &str = "TREZOR";
/// BIP-84 account-0 xpub of SEED + "TREZOR" (fingerprint b4e3f5ed).
const XPUB_PW: &str = "xpub6Crgkie5Rb7wDabkf4Uf6A2qnuERMA3p2QrnmHNQDrsXTaGvz9zugU38Apne8WqrcbSjdLwbhtfHrzWjNCJPVAkkNoQhMfzhBm8rKMA8KxH";
const VECTORS: &str = include_str!("vectors/passphrase_channels.json");

fn note() -> String {
    let v: Value = serde_json::from_str(VECTORS).unwrap();
    v["argv_note"].as_str().unwrap().to_string()
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    fn notes(&self) -> usize {
        let n = note();
        self.stderr.lines().filter(|l| *l == n).count()
    }
}

/// Run `mnemonic <argv>` with the seeds in `F687_SEED` / `F687_SEED_B`, so no
/// seed ever needs argv.
fn run(argv: &[String], stdin: &[u8], env: &[(&str, &str)]) -> Run {
    let mut c = Command::cargo_bin("mnemonic").unwrap();
    c.env("F687_SEED", SEED).env("F687_SEED_B", SEED_B);
    for (k, v) in env {
        c.env(k, v);
    }
    c.args(argv).write_stdin(stdin.to_vec());
    let o = c.output().unwrap();
    Run {
        code: o.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

fn with(base: &[String], extra: &[&str]) -> Vec<String> {
    let mut v = base.to_vec();
    v.extend(extra.iter().map(|x| x.to_string()));
    v
}

// ---------------------------------------------------------------------------
// The vector file.
// ---------------------------------------------------------------------------

#[test]
fn every_vector_case_holds() {
    let v: Value = serde_json::from_str(VECTORS).unwrap();
    assert_eq!(v["seed_phrase"], SEED);
    let base = s(&[
        "convert",
        "--from",
        "phrase=@env:F687_SEED",
        "--to",
        "fingerprint",
        "--template",
        "bip84",
    ]);
    let cases = v["cases"].as_array().unwrap();
    assert!(
        cases.len() >= 25,
        "the vector file lost cases: {}",
        cases.len()
    );
    let mut failures = Vec::new();
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let mut argv = base.clone();
        if case["allow_argv_secret"].as_bool().unwrap_or(false) {
            argv.push("--allow-argv-secret".into());
        }
        argv.extend(
            case["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| a.as_str().unwrap().to_string()),
        );
        let env: Vec<(String, String)> = case["env"]
            .as_object()
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let mut c = Command::cargo_bin("mnemonic").unwrap();
        c.env("F687_SEED", SEED);
        for (k, v) in &env {
            c.env(k, v);
        }
        for k in case["unset"].as_array().into_iter().flatten() {
            c.env_remove(k.as_str().unwrap());
        }
        let o = c
            .args(&argv)
            .write_stdin(case["stdin"].as_str().unwrap().as_bytes().to_vec())
            .output()
            .unwrap();
        let r = Run {
            code: o.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
        };
        if r.notes() as u64 != case["argv_notes"].as_u64().unwrap() {
            failures.push(format!(
                "{name}: {} argv notes; stderr:\n{}",
                r.notes(),
                r.stderr
            ));
        }
        let fp = r
            .stdout
            .lines()
            .find_map(|l| l.strip_prefix("fingerprint: "))
            .map(str::to_string);
        if let Some(want) = case["expect"]["fingerprint"].as_str() {
            if r.code != 0 || fp.as_deref() != Some(want) {
                failures.push(format!(
                    "{name}: want {want}, got rc {} fp {fp:?}; stderr:\n{}",
                    r.code, r.stderr
                ));
            }
        } else {
            let needle = case["expect"]["error_contains"].as_str().unwrap();
            if r.code == 0 || !r.stderr.contains(needle) || !r.stdout.is_empty() {
                failures.push(format!(
                    "{name}: want an error naming {needle:?}, got rc {} stdout {:?} stderr {:?}",
                    r.code, r.stdout, r.stderr
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}

// ---------------------------------------------------------------------------
// Every subcommand that declares --passphrase.
// ---------------------------------------------------------------------------

/// `(name, argv without any passphrase, argv of a SECOND stdin reader)`.
fn commands(dir: &std::path::Path) -> Vec<(&'static str, Vec<String>, Vec<String>)> {
    let secret = dir.join("secret.txt");
    std::fs::write(&secret, format!("{SEED}\n")).unwrap();
    // A matching bundle (made with the stdin flag) for verify-bundle.
    let r = run(
        &s(&[
            "bundle",
            "--slot",
            "@0.phrase=@env:F687_SEED",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--passphrase-stdin",
            "--json",
        ]),
        PW.as_bytes(),
        &[],
    );
    assert_eq!(r.code, 0, "{}", r.stderr);
    let bundle_json = dir.join("bundle.json");
    std::fs::write(&bundle_json, &r.stdout).unwrap();
    let bj = bundle_json.to_str().unwrap().to_string();
    let secret = secret.to_str().unwrap().to_string();
    let xs = |mode: &str| -> Vec<String> {
        let mut v = s(&["xpub-search", mode]);
        if mode == "account-of-descriptor" {
            v.extend(s(&["--descriptor", &format!("wpkh({XPUB_PW}/0/*)")]));
        } else {
            v.extend(s(&["--target-xpub", XPUB_PW]));
        }
        v
    };
    let with_phrase = |mut v: Vec<String>, stdin: bool| {
        if stdin {
            v.push("--phrase-stdin".into());
        } else {
            v.extend(s(&["--phrase", "@env:F687_SEED"]));
        }
        v
    };
    vec![
        (
            "addresses",
            s(&[
                "addresses",
                "--from",
                "phrase=@env:F687_SEED",
                "--address-type",
                "p2wpkh",
                "--count",
                "1",
            ]),
            s(&[
                "addresses",
                "--from",
                "phrase=-",
                "--address-type",
                "p2wpkh",
                "--count",
                "1",
            ]),
        ),
        (
            "restore",
            s(&[
                "restore",
                "--from",
                "phrase=@env:F687_SEED",
                "--template",
                "bip84",
            ]),
            s(&["restore", "--from", "phrase=-", "--template", "bip84"]),
        ),
        (
            "derive-child",
            s(&[
                "derive-child",
                "--from",
                "phrase=@env:F687_SEED",
                "--application",
                "bip39",
                "--length",
                "12",
                "--index",
                "0",
            ]),
            s(&[
                "derive-child",
                "--from",
                "phrase=-",
                "--application",
                "bip39",
                "--length",
                "12",
                "--index",
                "0",
            ]),
        ),
        (
            "bundle",
            s(&[
                "bundle",
                "--slot",
                "@0.phrase=@env:F687_SEED",
                "--network",
                "mainnet",
                "--template",
                "bip84",
            ]),
            s(&[
                "bundle",
                "--slot",
                "@0.phrase=-",
                "--network",
                "mainnet",
                "--template",
                "bip84",
            ]),
        ),
        (
            "convert",
            s(&[
                "convert",
                "--from",
                "phrase=@env:F687_SEED",
                "--to",
                "xpub",
                "--template",
                "bip84",
            ]),
            s(&[
                "convert",
                "--from",
                "phrase=-",
                "--to",
                "xpub",
                "--template",
                "bip84",
            ]),
        ),
        (
            "silent-payment",
            s(&["silent-payment", "--secret-file", &secret]),
            s(&["silent-payment", "--secret-stdin"]),
        ),
        (
            "slip39 split",
            s(&[
                "slip39",
                "split",
                "--from",
                "phrase=@env:F687_SEED",
                "--group-threshold",
                "1",
                "--group",
                "3,2",
            ]),
            s(&[
                "slip39",
                "split",
                "--from",
                "phrase=-",
                "--group-threshold",
                "1",
                "--group",
                "3,2",
            ]),
        ),
        ("slip39 combine", Vec::new(), Vec::new()), // filled by the caller (needs shares)
        (
            "verify-bundle",
            s(&[
                "verify-bundle",
                "--network",
                "mainnet",
                "--template",
                "bip84",
                "--slot",
                "@0.phrase=@env:F687_SEED",
                "--bundle-json",
                &bj,
            ]),
            s(&[
                "verify-bundle",
                "--network",
                "mainnet",
                "--template",
                "bip84",
                "--slot",
                "@0.phrase=-",
                "--bundle-json",
                &bj,
            ]),
        ),
        (
            "xpub-search path-of-xpub",
            with_phrase(xs("path-of-xpub"), false),
            with_phrase(xs("path-of-xpub"), true),
        ),
        (
            "xpub-search account-of-descriptor",
            with_phrase(xs("account-of-descriptor"), false),
            with_phrase(xs("account-of-descriptor"), true),
        ),
        (
            "xpub-search passphrase-of-xpub",
            with_phrase(xs("passphrase-of-xpub"), false),
            with_phrase(xs("passphrase-of-xpub"), true),
        ),
    ]
}

fn share_lines(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .take(2)
        .map(str::to_string)
        .collect()
}

/// What a run MEANS, for comparison across channels. SLIP-39 split output is
/// randomised, so it is compared by what its first two shares combine to under
/// the reference passphrase.
fn meaning(name: &str, r: &Run) -> String {
    if name == "slip39 split" {
        let shares = share_lines(&r.stdout);
        assert_eq!(shares.len(), 2, "{}", r.stdout);
        let c = run(
            &s(&[
                "slip39",
                "combine",
                "--share",
                "@env:S0",
                "--share",
                "@env:S1",
                "--passphrase-stdin",
            ]),
            PW.as_bytes(),
            &[("S0", &shares[0]), ("S1", &shares[1])],
        );
        assert_eq!(c.code, 0, "{}", c.stderr);
        return c.stdout;
    }
    r.stdout.clone()
}

#[test]
fn every_subcommand_honours_dash_env_and_literal_alike() {
    let dir = tempfile::tempdir().unwrap();
    let mut cmds = commands(dir.path());
    // slip39 combine needs shares made under the reference passphrase.
    let split = run(
        &with(&cmds[6].1, &["--passphrase-stdin"]),
        PW.as_bytes(),
        &[],
    );
    let shares = share_lines(&split.stdout);
    let share_env = [("S0", shares[0].as_str()), ("S1", shares[1].as_str())];
    cmds[7].1 = s(&[
        "slip39", "combine", "--share", "@env:S0", "--share", "@env:S1",
    ]);

    let mut failures = Vec::new();
    for (name, base, _) in &cmds {
        let env: Vec<(&str, &str)> = if *name == "slip39 combine" {
            share_env.to_vec()
        } else {
            vec![]
        };
        let reference = run(&with(base, &["--passphrase-stdin"]), PW.as_bytes(), &env);
        if reference.code != 0 {
            failures.push(format!(
                "{name}: reference rc {}: {}",
                reference.code, reference.stderr
            ));
            continue;
        }
        let want = meaning(name, &reference);
        let none = run(base, b"", &env);
        if meaning(name, &none) == want {
            failures.push(format!(
                "{name}: the passphrase does not change the output; the test proves nothing"
            ));
        }
        let mut env_pp = env.clone();
        env_pp.push(("F687_PP", PW));
        // The byte rule: a trailing newline in the variable is stripped, as on
        // stdin — on every subcommand, not only where `passphrase_input`
        // resolves directly (convert/derive-child/bundle/verify-bundle
        // resolve `@env:` in their own pre-pass).
        let mut env_pp_lf = env.clone();
        env_pp_lf.push(("F687_PP", "TREZOR\n"));
        for (label, argv, stdin, e, notes) in [
            (
                "--passphrase -",
                with(base, &["--passphrase", "-"]),
                PW.as_bytes(),
                &env,
                0usize,
            ),
            (
                "--passphrase - + LF",
                with(base, &["--passphrase", "-"]),
                b"TREZOR\n".as_slice(),
                &env,
                0,
            ),
            (
                "@env:",
                with(base, &["--passphrase", "@env:F687_PP"]),
                b"".as_slice(),
                &env_pp,
                0,
            ),
            (
                "@env: + LF",
                with(base, &["--passphrase", "@env:F687_PP"]),
                b"".as_slice(),
                &env_pp_lf,
                0,
            ),
            (
                "literal",
                with(base, &["--allow-argv-secret", "--passphrase", PW]),
                b"".as_slice(),
                &env,
                1,
            ),
        ] {
            let r = run(&argv, stdin, e);
            if r.code != 0 {
                failures.push(format!("{name} {label}: rc {}: {}", r.code, r.stderr));
                continue;
            }
            if meaning(name, &r) != want {
                failures.push(format!(
                    "{name} {label}: a different result than --passphrase-stdin"
                ));
            }
            if r.notes() != notes {
                failures.push(format!(
                    "{name} {label}: {} argv notes, want {notes}:\n{}",
                    r.notes(),
                    r.stderr
                ));
            }
            if r.stderr.contains(PW) {
                failures.push(format!("{name} {label}: stderr echoes the passphrase"));
            }
            if *name == "verify-bundle" && !r.stdout.lines().any(|l| l == "result: ok") {
                failures.push(format!("{name} {label}: not `result: ok`:\n{}", r.stdout));
            }
        }
        if reference.notes() != 0 {
            failures.push(format!("{name}: --passphrase-stdin printed the argv note"));
        }
        // Unset variable: an error naming it, never the empty passphrase.
        let r = run(
            &with(base, &["--passphrase", "@env:F687_NEVER_SET"]),
            b"",
            &env,
        );
        if r.code == 0 || !r.stderr.contains("F687_NEVER_SET") {
            failures.push(format!(
                "{name}: unset @env: gave rc {}: {}",
                r.code, r.stderr
            ));
        }
        // `-` and the flag together: one passphrase, one stdin.
        let r = run(
            &with(base, &["--passphrase", "-", "--passphrase-stdin"]),
            PW.as_bytes(),
            &env,
        );
        if r.code == 0 || !r.stdout.is_empty() {
            failures.push(format!(
                "{name}: `--passphrase - --passphrase-stdin` accepted (rc {})",
                r.code
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}

/// One stdin per invocation: `--passphrase -` (and the flag) beside another
/// input on stdin is refused before anything is derived.
#[test]
fn a_second_stdin_reader_is_refused_for_both_spellings() {
    let dir = tempfile::tempdir().unwrap();
    let mut failures = Vec::new();
    for (name, _, second) in commands(dir.path()) {
        let second = if name == "slip39 combine" {
            s(&[
                "slip39",
                "combine",
                "--share",
                "-",
                "--share",
                "@env:F687_SEED",
            ])
        } else {
            second
        };
        for pp in [&["--passphrase", "-"][..], &["--passphrase-stdin"][..]] {
            let r = run(&with(&second, pp), format!("{SEED}\n").as_bytes(), &[]);
            if r.code == 0 || !r.stdout.is_empty() || !r.stderr.contains("stdin") {
                failures.push(format!("{name} {pp:?}: rc {} stderr {}", r.code, r.stderr));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}

// ---------------------------------------------------------------------------
// verify-bundle's keyless multisig TEMPLATE path (the shared
// resolve_template_completion_seed helper, reached with ALREADY-substituted
// args).
// ---------------------------------------------------------------------------

fn section(stdout: &str, header: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut on = false;
    for l in stdout.lines() {
        if l.starts_with(header) {
            on = true;
            continue;
        }
        if on && l.trim().is_empty() {
            on = false;
        }
        if on && (l.starts_with("mk1") || l.starts_with("md1")) {
            out.push(l.trim().to_string());
        }
    }
    out
}

#[test]
fn verify_bundle_template_completion_takes_every_passphrase_form() {
    let xpub_of = |env: &str, pw: Option<&str>| -> (String, String) {
        let mut argv = s(&[
            "convert",
            "--from",
            &format!("phrase=@env:{env}"),
            "--to",
            "xpub,fingerprint",
            "--template",
            "wsh-sortedmulti",
        ]);
        if pw.is_some() {
            argv.push("--passphrase-stdin".into());
        }
        let r = run(&argv, pw.unwrap_or("").as_bytes(), &[]);
        assert_eq!(r.code, 0, "{}", r.stderr);
        let get = |k: &str| {
            r.stdout
                .lines()
                .find_map(|l| l.strip_prefix(k))
                .unwrap()
                .to_string()
        };
        (get("xpub: "), get("fingerprint: "))
    };
    let (xa, fa) = xpub_of("F687_SEED", Some(PW));
    assert_eq!(fa, "b4e3f5ed");
    let (xb, fb) = xpub_of("F687_SEED_B", None);
    let common = s(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--group-size",
        "0",
        "--no-engraving-card",
        "--slot",
        &format!("@0.xpub={xa}"),
        "--slot",
        &format!("@0.fingerprint={fa}"),
        "--slot",
        "@0.path=87'/0'/0'",
        "--slot",
        &format!("@1.xpub={xb}"),
        "--slot",
        &format!("@1.fingerprint={fb}"),
        "--slot",
        "@1.path=87'/0'/0'",
    ]);
    let t = run(&with(&common, &["--md1-form", "template"]), b"", &[]);
    assert_eq!(t.code, 0, "{}", t.stderr);
    let wid = t
        .stderr
        .lines()
        .find(|l| l.contains("wallet-id (hex)"))
        .and_then(|l| l.rsplit(':').next())
        .unwrap()
        .trim()
        .to_string();
    let p = run(&with(&common, &["--md1-form", "policy"]), b"", &[]);
    assert_eq!(p.code, 0, "{}", p.stderr);
    let mut argv = s(&["verify-bundle", "--network", "mainnet"]);
    for c in section(&t.stdout, "# md1") {
        argv.extend(["--md1".to_string(), c]);
    }
    for c in section(&t.stdout, "# mk1") {
        argv.extend(["--mk1".to_string(), c]);
    }
    for c in section(&p.stdout, "# mk1[1]") {
        argv.extend(["--cosigner".to_string(), c]);
    }
    argv.extend(s(&[
        "--from",
        "phrase=@env:F687_SEED",
        "--expect-wallet-id",
        &wid,
    ]));

    // Control: no passphrase is the wrong seed, so NO MATCH (exit 4).
    let none = run(&argv, b"", &[]);
    assert_eq!(none.code, 4, "control: {}\n{}", none.stdout, none.stderr);

    for (label, extra, stdin, env, notes) in [
        (
            "--passphrase-stdin",
            vec!["--passphrase-stdin"],
            PW,
            vec![],
            0usize,
        ),
        (
            "--passphrase -",
            vec!["--passphrase", "-"],
            "TREZOR\n",
            vec![],
            0,
        ),
        (
            "@env:",
            vec!["--passphrase", "@env:F687_PP"],
            "",
            vec![("F687_PP", PW)],
            0,
        ),
        (
            "literal",
            vec!["--allow-argv-secret", "--passphrase", PW],
            "",
            vec![],
            1,
        ),
    ] {
        let r = run(&with(&argv, &extra), stdin.as_bytes(), &env);
        assert_eq!(r.code, 0, "{label}: {}\n{}", r.stdout, r.stderr);
        assert!(
            r.stdout
                .lines()
                .any(|l| l.starts_with("OK (multisig template recomposed)")),
            "{label}: {}",
            r.stdout
        );
        assert_eq!(r.notes(), notes, "{label}: {}", r.stderr);
    }
}

/// Pre-F-687, xpub-search let `--phrase-stdin` drain stdin and then searched
/// with the EMPTY passphrase: a false "no match" (exit 4) for the right
/// passphrase. Now refused, for both passphrase spellings.
#[test]
fn xpub_search_phrase_stdin_and_passphrase_stdin_are_refused() {
    for mode in ["path-of-xpub", "passphrase-of-xpub"] {
        for pp in [&["--passphrase-stdin"][..], &["--passphrase", "-"][..]] {
            let argv = with(
                &s(&[
                    "xpub-search",
                    mode,
                    "--target-xpub",
                    XPUB_PW,
                    "--phrase-stdin",
                ]),
                pp,
            );
            let r = run(&argv, format!("{SEED}\n").as_bytes(), &[]);
            assert_eq!(r.code, 1, "{mode} {pp:?}: {}", r.stderr);
            assert!(
                r.stderr.contains("only one input can come from stdin"),
                "{}",
                r.stderr
            );
        }
    }
}
