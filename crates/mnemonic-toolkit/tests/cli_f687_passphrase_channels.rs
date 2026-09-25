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
        cases.len() >= 42,
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
        // F-687b ruling 2: exactly `empty_warnings` empty-channel warnings.
        let empties = r
            .stderr
            .lines()
            .filter(|l| {
                l.starts_with("warning: --passphrase from ")
                    && l.ends_with(" is empty; proceeding with the EMPTY passphrase")
            })
            .count() as u64;
        if empties != case["empty_warnings"].as_u64().unwrap() {
            failures.push(format!(
                "{name}: {empties} empty warnings; stderr:\n{}",
                r.stderr
            ));
        }
        // F-687b ruling 3: stdin is a pipe in every case, so never a prompt.
        if r.stderr.contains(v["prompt"].as_str().unwrap().trim_end()) {
            failures.push(format!("{name}: prompted on a non-terminal:\n{}", r.stderr));
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
        } else if let Some(code) = case["expect"]["exit_code"].as_i64() {
            if i64::from(r.code) != code || !r.stdout.is_empty() {
                failures.push(format!(
                    "{name}: want exit {code}, got rc {} stdout {:?} stderr {:?}",
                    r.code, r.stdout, r.stderr
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

// ---------------------------------------------------------------------------
// Fold 1 (review f687-review.md).
// ---------------------------------------------------------------------------

/// M1 + M4: an input PATH that is stdin — by name, or the same file as fd 0 —
/// and `bundle --import-json -` are stdin readers. Before fold 1,
/// `silent-payment --secret-file /dev/stdin --passphrase -` derived the
/// NO-passphrase wallet at exit 0, and `bundle --import-json -` swallowed the
/// JSON as the passphrase.
#[test]
fn a_path_that_is_stdin_is_a_second_stdin_reader() {
    let dir = tempfile::tempdir().unwrap();
    let seed_file = dir.path().join("seed.txt");
    std::fs::write(&seed_file, format!("{SEED}\n")).unwrap();
    let seed_path = seed_file.to_str().unwrap();
    let cases: Vec<Vec<String>> = vec![
        s(&["silent-payment", "--secret-file", "/dev/stdin"]),
        s(&["silent-payment", "--secret-file", "/dev/fd/0"]),
        s(&[
            "bundle",
            "--network",
            "mainnet",
            "--import-json",
            "-",
            "--slot",
            "@0.phrase=@env:F687_SEED",
        ]),
        s(&[
            "bundle",
            "--network",
            "mainnet",
            "--descriptor-file",
            "/dev/stdin",
        ]),
        s(&[
            "verify-bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            "@0.phrase=@env:F687_SEED",
            "--bundle-json",
            "/dev/stdin",
        ]),
    ];
    for base in &cases {
        for pp in [&["--passphrase", "-"][..], &["--passphrase-stdin"][..]] {
            let r = run(&with(base, pp), format!("{SEED}\n").as_bytes(), &[]);
            assert!(
                r.code != 0 && r.stdout.is_empty() && r.stderr.contains("stdin"),
                "{base:?} {pp:?}: rc {} stderr {}",
                r.code,
                r.stderr
            );
        }
    }
    // By INODE: stdin redirected from a file, and the same file named as the
    // input path.
    // fd 0 IS the file (a real `< seed.txt` redirect, not a pipe).
    let o = std::process::Command::new(assert_cmd::cargo::cargo_bin("mnemonic"))
        .args([
            "silent-payment",
            "--secret-file",
            seed_path,
            "--passphrase",
            "-",
        ])
        .stdin(std::fs::File::open(&seed_file).unwrap())
        .output()
        .unwrap();
    assert!(!o.status.success() && o.stdout.is_empty(), "{:?}", o);
    // Control: the same path with the passphrase NOT on stdin is fine.
    let r = run(
        &s(&[
            "silent-payment",
            "--secret-file",
            seed_path,
            "--passphrase",
            "@env:F687_PP",
        ]),
        b"",
        &[("F687_PP", PW)],
    );
    assert_eq!(r.code, 0, "{}", r.stderr);
}

/// N1: a non-UTF-8 `@env:` value is "not valid UTF-8", not "not set"; a
/// non-UTF-8 argv value is a usage error (64), not a panic (101), and is not
/// echoed.
#[cfg(unix)]
#[test]
fn non_utf8_input_is_refused_by_name_not_panicked() {
    use std::os::unix::ffi::OsStrExt;
    let bad = std::ffi::OsStr::from_bytes(b"TRE\xffZOR");
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("F687_SEED", SEED)
        .env("F687_PP", bad)
        .args([
            "convert",
            "--from",
            "phrase=@env:F687_SEED",
            "--to",
            "fingerprint",
            "--template",
            "bip84",
            "--passphrase",
            "@env:F687_PP",
        ])
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert!(
        err.contains("F687_PP") && err.contains("not valid UTF-8"),
        "{err}"
    );
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("F687_SEED", SEED)
        .args([
            "convert",
            "--from",
            "phrase=@env:F687_SEED",
            "--to",
            "fingerprint",
            "--template",
            "bip84",
            "--allow-argv-secret",
            "--passphrase",
        ])
        .arg(bad)
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(64), "{err}");
    assert!(
        err.contains("not valid UTF-8") && !err.contains("ZOR"),
        "{err}"
    );
}

/// N2: the slip39 two-stdin refusal names the spelling the operator typed.
#[test]
fn slip39_refusal_names_the_dash_spelling() {
    let r = run(
        &s(&[
            "slip39",
            "combine",
            "--share",
            "-",
            "--share",
            "@env:F687_SEED",
            "--passphrase",
            "-",
        ]),
        b"x",
        &[],
    );
    assert!(r.code != 0, "{}", r.stderr);
    assert!(r.stderr.contains("and --passphrase -)"), "{}", r.stderr);
}

// ---------------------------------------------------------------------------
// F-687b ruling 3: the terminal prompt. A real pseudo-terminal is the only way
// to exercise it: every other test pipes stdin, which is the no-prompt case.
// ---------------------------------------------------------------------------

/// What a pty run produced.
#[cfg(target_os = "linux")]
#[allow(dead_code)]
struct PtyRun {
    code: Option<i32>,
    /// The signal that killed the child, if one did.
    signal: Option<i32>,
    stdout: String,
    stderr: String,
    /// Everything the terminal DISPLAYED (the line discipline's echo).
    shown: Vec<u8>,
    /// Was ECHO on in the terminal's mode after the child was gone?
    echo_after: bool,
    /// F-687c: input still queued on the terminal after the child was gone,
    /// i.e. what the shell would read next and run.
    left_for_shell: Vec<u8>,
}

/// Run `bin argv` on a fresh pty that is the child's CONTROLLING terminal
/// (setsid + TIOCSCTTY, so a typed Ctrl-C really sends SIGINT). Waits for
/// `prompt` on stderr, then types `typed` on the master. On a timeout the
/// child is KILLED before the test fails, so no process is left blocked on
/// the pty (review M5).
#[cfg(target_os = "linux")]
fn run_on_a_terminal(
    bin: &std::path::Path,
    argv: &[&str],
    env: &[(&str, &str)],
    prompt: &str,
    typed: &[u8],
) -> PtyRun {
    run_on_a_terminal_writes(bin, argv, env, prompt, &[typed])
}

/// [`run_on_a_terminal`], typing each of `writes` as a separate write (the
/// second and later ones are type-ahead after the first).
#[cfg(target_os = "linux")]
fn run_on_a_terminal_writes(
    bin: &std::path::Path,
    argv: &[&str],
    env: &[(&str, &str)],
    prompt: &str,
    writes: &[&[u8]],
) -> PtyRun {
    use std::io::{Read, Write};
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    // SAFETY: posix_openpt/grantpt/unlockpt/ptsname on a fd we own.
    let (mut master, name) = unsafe {
        let m = libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY);
        assert!(m >= 0, "posix_openpt");
        assert_eq!(libc::grantpt(m), 0);
        assert_eq!(libc::unlockpt(m), 0);
        let name = std::ffi::CStr::from_ptr(libc::ptsname(m))
            .to_str()
            .unwrap()
            .to_string();
        (std::fs::File::from_raw_fd(m), name)
    };
    let open_slave = || {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NOCTTY)
            .open(&name)
            .unwrap()
    };
    use std::os::unix::fs::OpenOptionsExt;
    // Our own handle on the slave, to read the terminal mode back afterwards.
    let keep = open_slave();
    let mut cmd = std::process::Command::new(bin);
    cmd.args(argv)
        .stdin(open_slave())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    for (k, v) in env {
        cmd.env(k, v);
    }
    // SAFETY: async-signal-safe calls only, between fork and exec.
    unsafe {
        cmd.pre_exec(|| {
            if libc::setsid() < 0 || libc::ioctl(0, libc::TIOCSCTTY as _, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = cmd.spawn().unwrap();
    let mut err = child.stderr.take().unwrap();
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let prompt_owned = prompt.to_string();
    let reader = std::thread::spawn(move || {
        let mut got = Vec::new();
        let mut b = [0u8; 1];
        let mut sent = false;
        while err.read(&mut b).unwrap_or(0) == 1 {
            got.push(b[0]);
            if !sent && String::from_utf8_lossy(&got).contains(&prompt_owned) {
                let _ = tx.send(());
                sent = true;
            }
        }
        String::from_utf8_lossy(&got).into_owned()
    });
    if rx.recv_timeout(std::time::Duration::from_secs(20)).is_err() {
        let _ = child.kill();
        let _ = child.wait();
        panic!("the prompt {prompt:?} never appeared on stderr (child killed)");
    }
    for w in writes {
        // An EMPTY write is a 40 ms pause: it lets the child finish reading
        // the line and enter the drain before the next write arrives.
        if w.is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(40));
            continue;
        }
        master.write_all(w).unwrap();
        master.flush().unwrap();
    }
    let out = child.wait_with_output().unwrap();
    let stderr = reader.join().unwrap();
    // SAFETY: fcntl/tcgetattr on fds we own.
    let echo_after = unsafe {
        let fd = master.as_raw_fd();
        libc::fcntl(
            fd,
            libc::F_SETFL,
            libc::fcntl(fd, libc::F_GETFL) | libc::O_NONBLOCK,
        );
        let mut t: libc::termios = std::mem::zeroed();
        assert_eq!(libc::tcgetattr(keep.as_raw_fd(), &mut t), 0, "tcgetattr");
        t.c_lflag & libc::ECHO != 0
    };
    // What would the shell read next? Switch our slave handle to
    // non-blocking, non-canonical reads and take everything queued.
    // SAFETY: termios/read on the slave fd we own.
    let left_for_shell = unsafe {
        let fd = keep.as_raw_fd();
        let mut t: libc::termios = std::mem::zeroed();
        libc::tcgetattr(fd, &mut t);
        t.c_lflag &= !libc::ICANON;
        t.c_cc[libc::VMIN] = 0;
        t.c_cc[libc::VTIME] = 0;
        libc::tcsetattr(fd, libc::TCSANOW, &t);
        let mut left = Vec::new();
        let mut b = [0u8; 256];
        loop {
            let n = libc::read(fd, b.as_mut_ptr().cast(), b.len());
            if n <= 0 {
                break;
            }
            left.extend_from_slice(&b[..n as usize]);
        }
        left
    };
    let mut shown = Vec::new();
    let mut buf = [0u8; 256];
    while let Ok(n) = master.read(&mut buf) {
        if n == 0 {
            break;
        }
        shown.extend_from_slice(&buf[..n]);
    }
    PtyRun {
        code: out.status.code(),
        signal: out.status.signal(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr,
        shown,
        echo_after,
        left_for_shell,
    }
}

// ---------------------------------------------------------------------------
// F-687b: the operator's three rulings (2026-09-25) and F-689.
// ---------------------------------------------------------------------------

/// BIP-38 test vector 1 (no EC multiply, uncompressed).
const BIP38_WIF: &str = "5KN7MzqK5wt2TP1fQCYyHBtDrXdJuXbUzm4A9rKAteGu3Qi5CVR";
const BIP38_PW: &str = "TestingOneTwoThree";
const BIP38_OUT: &str = "bip38: 6PRVWUbkzzsbcVac2qwfssoUJAN1Xhrg6bNk8J7Nzm5H7kxEbn2Nh2ZoGg";
/// Electrum field-encryption test vector (`cli_electrum_decrypt.rs`).
const EL_CT: &str = "ABEiM0RVZneImaq7zN3u/zY0181f7qAY/NWiVQFLdHE=";
const EL_PW: &str = "test-password";
const BIE1: &str = "tests/fixtures/wallet_import/electrum-bie1-storage-bip84.txt";

fn empties(stderr: &str, flag: &str) -> usize {
    stderr
        .lines()
        .filter(|l| l.starts_with(&format!("warning: {flag} from ")) && l.contains(" is empty; "))
        .count()
}

/// Ruling 1: `--bip38-passphrase` follows the same rule through the same
/// resolver: `-`, `@env:` (one newline stripped), the stdin flag and a
/// literal all give the BIP-38 vector; only the literal gets the note; an
/// empty private value warns once (ruling 2).
#[test]
fn bip38_passphrase_takes_every_form() {
    let base = s(&["convert", "--from", "wif=@env:F687_W", "--to", "bip38"]);
    let w = [("F687_W", BIP38_WIF)];
    let lf = format!("{BIP38_PW}\n");
    for (label, extra, stdin, env, notes) in [
        (
            "stdin flag",
            vec!["--bip38-passphrase-stdin"],
            lf.as_str(),
            vec![],
            0usize,
        ),
        (
            "dash",
            vec!["--bip38-passphrase", "-"],
            lf.as_str(),
            vec![],
            0,
        ),
        ("dash =", vec!["--bip38-passphrase=-"], BIP38_PW, vec![], 0),
        (
            "env",
            vec!["--bip38-passphrase", "@env:F687_BP"],
            "",
            vec![("F687_BP", lf.as_str())],
            0,
        ),
        (
            "literal",
            vec!["--allow-argv-secret", "--bip38-passphrase", BIP38_PW],
            "",
            vec![],
            1,
        ),
    ] {
        let mut e = w.to_vec();
        e.extend(env);
        let r = run(&with(&base, &extra), stdin.as_bytes(), &e);
        assert_eq!(r.code, 0, "{label}: {}", r.stderr);
        assert!(
            r.stdout.lines().any(|l| l == BIP38_OUT),
            "{label}: {}",
            r.stdout
        );
        let n = r
            .stderr
            .lines()
            .filter(|l| l.starts_with("warning: secret material on argv (--bip38-passphrase)"))
            .count();
        assert_eq!(n, notes, "{label}: {}", r.stderr);
        assert_eq!(empties(&r.stderr, "--bip38-passphrase"), 0, "{label}");
    }
    // Empty from stdin: warned once, proceeds (never refused).
    let r = run(&with(&base, &["--bip38-passphrase", "-"]), b"\n", &w);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert_eq!(empties(&r.stderr, "--bip38-passphrase"), 1, "{}", r.stderr);
    // One stdin: `--bip38-passphrase -` beside `--from wif=-`, or beside a
    // stdin `--passphrase`, is refused before anything is read.
    for argv in [
        s(&[
            "convert",
            "--from",
            "wif=-",
            "--to",
            "bip38",
            "--bip38-passphrase",
            "-",
        ]),
        s(&[
            "convert",
            "--from",
            "phrase=@env:F687_SEED",
            "--to",
            "bip38",
            "--template",
            "bip84",
            "--passphrase",
            "-",
            "--bip38-passphrase",
            "-",
        ]),
    ] {
        let r = run(&argv, format!("{BIP38_WIF}\n").as_bytes(), &[]);
        assert!(r.code != 0 && r.stdout.is_empty(), "{argv:?}: {}", r.stderr);
        assert!(r.stderr.contains("--bip38-passphrase -"), "{}", r.stderr);
    }
}

/// Ruling 1: `--decrypt-password` on `electrum-decrypt` and `import-wallet`.
#[test]
fn decrypt_password_takes_every_form() {
    let base = s(&["electrum-decrypt", "--ciphertext", EL_CT]);
    let lf = format!("{EL_PW}\n");
    for (label, extra, stdin, env, notes) in [
        (
            "stdin flag",
            vec!["--decrypt-password-stdin"],
            lf.as_str(),
            vec![],
            0usize,
        ),
        (
            "dash",
            vec!["--decrypt-password", "-"],
            lf.as_str(),
            vec![],
            0,
        ),
        (
            "env",
            vec!["--decrypt-password", "@env:F687_DP"],
            "",
            vec![("F687_DP", lf.as_str())],
            0,
        ),
        (
            "literal",
            vec!["--allow-argv-secret", "--decrypt-password", EL_PW],
            "",
            vec![],
            1,
        ),
    ] {
        let r = run(&with(&base, &extra), stdin.as_bytes(), &env);
        assert_eq!(r.code, 0, "{label}: {}", r.stderr);
        assert_eq!(r.stdout.trim(), "hello world", "{label}");
        let n = r
            .stderr
            .lines()
            .filter(|l| l.starts_with("warning: secret material on argv (--decrypt-password)"))
            .count();
        assert_eq!(n, notes, "{label}: {}", r.stderr);
    }
    // Empty from the environment: warned once, then the (wrong) password
    // fails closed, as any wrong password does.
    let r = run(
        &with(&base, &["--decrypt-password", "@env:F687_DP"]),
        b"",
        &[("F687_DP", "")],
    );
    assert_eq!(empties(&r.stderr, "--decrypt-password"), 1, "{}", r.stderr);
    assert!(
        r.stderr.contains("environment variable F687_DP"),
        "{}",
        r.stderr
    );
    // One stdin.
    let r = run(
        &s(&[
            "electrum-decrypt",
            "--ciphertext",
            "-",
            "--decrypt-password",
            "-",
        ]),
        b"x",
        &[],
    );
    assert!(r.code != 0 && r.stderr.contains("stdin"), "{}", r.stderr);

    // import-wallet: BIE1 storage decrypts through `-` and `@env:`.
    for (extra, stdin, env) in [
        (vec!["--decrypt-password", "-"], "satoshi\n", vec![]),
        (
            vec!["--decrypt-password", "@env:F687_DP"],
            "",
            vec![("F687_DP", "satoshi")],
        ),
    ] {
        let r = run(
            &with(&s(&["import-wallet", "--blob", BIE1]), &extra),
            stdin.as_bytes(),
            &env,
        );
        assert_eq!(r.code, 0, "{extra:?}: {}", r.stderr);
        assert!(
            r.stderr.contains("BIE1 user-password storage decrypted"),
            "{}",
            r.stderr
        );
        assert!(
            !r.stderr.contains("secret material on argv"),
            "{}",
            r.stderr
        );
    }
    let r = run(
        &s(&["import-wallet", "--blob", "-", "--decrypt-password", "-"]),
        b"x",
        &[],
    );
    assert!(
        r.code != 0 && r.stderr.contains("--decrypt-password -"),
        "{}",
        r.stderr
    );
}

/// F-689: `verify-bundle --ms1 -` reads the ms1 from stdin and verifies
/// `result: ok` against a matching bundle. Before, the `-` was stripped to
/// `""` (the watch-only sentinel) and the bundle reported a false mismatch.
#[test]
fn verify_bundle_ms1_dash_reads_stdin() {
    let r = run(
        &s(&[
            "bundle",
            "--slot",
            "@0.phrase=@env:F687_SEED",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ]),
        b"",
        &[],
    );
    assert_eq!(r.code, 0, "{}", r.stderr);
    let line = |p: &str| -> Vec<String> {
        r.stdout
            .lines()
            .filter(|l| l.starts_with(p))
            .map(str::to_string)
            .collect()
    };
    let ms1 = line("ms1")[0].clone();
    let mut base = s(&[
        "verify-bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        "@0.phrase=@env:F687_SEED",
    ]);
    let mut tail = vec!["--".to_string()];
    tail.extend(line("mk1"));
    tail.extend(line("md1"));
    let ok = |r: &Run| r.stdout.lines().any(|l| l == "result: ok");
    // Control: the ms1 on argv verifies.
    let mut lit = base.clone();
    lit.extend(["--allow-argv-secret".into(), "--ms1".into(), ms1.clone()]);
    lit.extend(tail.clone());
    assert!(ok(&run(&lit, b"", &[])));
    // `--ms1 -`.
    base.extend(["--ms1".to_string(), "-".to_string()]);
    let mut dash = base.clone();
    dash.extend(tail.clone());
    let r = run(&dash, format!("{ms1}\n").as_bytes(), &[]);
    assert!(ok(&r), "rc {}: {}\n{}", r.code, r.stdout, r.stderr);
    // A wrong ms1 on stdin still mismatches (the `-` really is read).
    let r = run(
        &dash,
        b"ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7g\n",
        &[],
    );
    assert!(!ok(&r), "{}", r.stdout);
    // Empty stdin is refused, never the watch-only sentinel.
    let r = run(&dash, b"", &[]);
    assert!(
        r.code != 0 && r.stderr.contains("--ms1 -: stdin was empty"),
        "{}",
        r.stderr
    );
    // One stdin: beside `--passphrase -`, or twice.
    let mut two = base.clone();
    two.extend(["--passphrase".into(), "-".into()]);
    two.extend(tail.clone());
    let r = run(&two, format!("{ms1}\n").as_bytes(), &[]);
    assert!(r.code != 0 && r.stderr.contains("--ms1 -"), "{}", r.stderr);
    let mut twice = base.clone();
    twice.extend(["--ms1".into(), "-".into()]);
    twice.extend(tail);
    let r = run(&twice, format!("{ms1}\n").as_bytes(), &[]);
    assert!(
        r.code != 0 && r.stderr.contains("at most one --ms1 -"),
        "{}",
        r.stderr
    );
}

/// Ruling 3: on a terminal, prompt on stderr, echo off, read ONE line.
#[cfg(target_os = "linux")]
#[test]
fn a_terminal_gets_a_prompt_and_no_echo() {
    let bin = assert_cmd::cargo::cargo_bin("mnemonic");
    let fp = [
        "convert",
        "--from",
        "phrase=@env:F687_SEED",
        "--to",
        "fingerprint",
        "--template",
        "bip84",
    ];
    for pp in [&["--passphrase", "-"][..], &["--passphrase-stdin"][..]] {
        let argv: Vec<&str> = fp.iter().chain(pp.iter()).copied().collect();
        let r = run_on_a_terminal(
            &bin,
            &argv,
            &[("F687_SEED", SEED)],
            "Enter passphrase: ",
            b"TREZOR\n",
        );
        assert_eq!(r.code, Some(0), "{pp:?}: {}", r.stderr);
        assert!(
            r.stdout.contains("fingerprint: b4e3f5ed"),
            "{pp:?}: {}",
            r.stdout
        );
        assert!(
            !r.stderr.contains("input will be visible"),
            "echo could not be disabled: {}",
            r.stderr
        );
        let shown = String::from_utf8_lossy(&r.shown);
        assert!(
            !shown.contains("TREZOR"),
            "the terminal echoed the passphrase: {shown:?}"
        );
        // Review M4: the terminal mode is RESTORED after a normal exit.
        assert!(r.echo_after, "{pp:?}: echo left OFF after the run");
    }
    // The other flags prompt with their own name.
    let argv = [
        "convert",
        "--from",
        "wif=@env:F687_W",
        "--to",
        "bip38",
        "--bip38-passphrase",
        "-",
    ];
    let typed = format!("{BIP38_PW}\n");
    let r = run_on_a_terminal(
        &bin,
        &argv,
        &[("F687_W", BIP38_WIF)],
        "Enter BIP-38 passphrase: ",
        typed.as_bytes(),
    );
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    assert!(r.stdout.contains(BIP38_OUT), "{}", r.stdout);
    assert!(r.echo_after);
}

/// Ctrl-C at the prompt: the process dies of SIGINT (the conventional
/// status) AND the terminal has echo back on. Ctrl-D at an empty prompt: the
/// empty warning starts on its own line (review N1), echo restored.
#[cfg(target_os = "linux")]
#[test]
fn ctrl_c_and_ctrl_d_at_the_prompt_leave_a_working_terminal() {
    let bin = assert_cmd::cargo::cargo_bin("mnemonic");
    let argv = [
        "convert",
        "--from",
        "phrase=@env:F687_SEED",
        "--to",
        "fingerprint",
        "--template",
        "bip84",
        "--passphrase",
        "-",
    ];
    let env = [("F687_SEED", SEED)];
    let r = run_on_a_terminal(&bin, &argv, &env, "Enter passphrase: ", b"TRE\x03");
    assert_eq!(
        r.signal,
        Some(libc::SIGINT),
        "code {:?}: {}",
        r.code,
        r.stderr
    );
    assert!(r.stdout.is_empty());
    assert!(r.echo_after, "Ctrl-C left echo OFF");
    let r = run_on_a_terminal(&bin, &argv, &env, "Enter passphrase: ", b"\x04");
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    assert!(r.stdout.contains("fingerprint: 73c5da0a"), "{}", r.stdout);
    assert!(
        r.stderr
            .contains("Enter passphrase: \nwarning: --passphrase from stdin is empty"),
        "{:?}",
        r.stderr
    );
    assert!(r.echo_after);
}

/// F-689 follow-up: `verify-bundle --ms1 -` on a terminal prompts and hides
/// the ms1 (seed material) exactly like a passphrase.
#[cfg(target_os = "linux")]
#[test]
fn verify_bundle_ms1_dash_prompts_on_a_terminal() {
    let r = run(
        &s(&[
            "bundle",
            "--slot",
            "@0.phrase=@env:F687_SEED",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ]),
        b"",
        &[],
    );
    let ms1 = r
        .stdout
        .lines()
        .find(|l| l.starts_with("ms1"))
        .unwrap()
        .to_string();
    let mut argv: Vec<String> = s(&[
        "verify-bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        "@0.phrase=@env:F687_SEED",
        "--ms1",
        "-",
        "--",
    ]);
    argv.extend(
        r.stdout
            .lines()
            .filter(|l| l.starts_with("mk1") || l.starts_with("md1"))
            .map(str::to_string),
    );
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    let bin = assert_cmd::cargo::cargo_bin("mnemonic");
    let typed = format!("{ms1}\n");
    let r = run_on_a_terminal(
        &bin,
        &argv,
        &[("F687_SEED", SEED)],
        "Enter ms1: ",
        typed.as_bytes(),
    );
    assert_eq!(r.code, Some(0), "{}\n{}", r.stdout, r.stderr);
    assert!(r.stdout.lines().any(|l| l == "result: ok"), "{}", r.stdout);
    assert!(
        !String::from_utf8_lossy(&r.shown).contains(&ms1[5..]),
        "the ms1 was echoed"
    );
    assert!(r.echo_after);
}

/// Review M1-M3: separators-only `--ms1 -` is refused (never the watch-only
/// `""`); `import-wallet --blob /dev/stdin` beside a stdin password, and
/// `verify-bundle --ms1 -` beside `--descriptor-file /dev/stdin`, are two
/// stdin readers.
#[test]
fn fold1_stdin_gaps_are_refused() {
    for stdin in [&b"-\n"[..], b"---", b" - - \n"] {
        let r = run(
            &s(&[
                "verify-bundle",
                "--network",
                "mainnet",
                "--template",
                "bip84",
                "--slot",
                "@0.phrase=@env:F687_SEED",
                "--ms1",
                "-",
                "--",
                "mk1x",
            ]),
            stdin,
            &[],
        );
        assert!(
            r.code != 0
                && r.stderr
                    .contains("--ms1 -: stdin was empty or only separators"),
            "{stdin:?}: {}",
            r.stderr
        );
        assert!(!r.stderr.contains("watch-only via empty"), "{}", r.stderr);
    }
    for pw in [
        &["--decrypt-password", "-"][..],
        &["--decrypt-password-stdin"][..],
    ] {
        let r = run(
            &with(&s(&["import-wallet", "--blob", "/dev/stdin"]), pw),
            b"satoshi\n",
            &[],
        );
        assert!(
            r.code != 0 && r.stderr.contains("cannot both read from stdin"),
            "{pw:?}: {}",
            r.stderr
        );
    }
    let r = run(
        &s(&[
            "verify-bundle",
            "--network",
            "mainnet",
            "--descriptor-file",
            "/dev/stdin",
            "--slot",
            "@0.phrase=@env:F687_SEED",
            "--ms1",
            "-",
            "--",
            "mk1x",
        ]),
        b"x\n",
        &[],
    );
    assert!(
        r.code != 0 && r.stderr.contains("--descriptor-file /dev/stdin"),
        "{}",
        r.stderr
    );
}

// ---------------------------------------------------------------------------
// F-687c (operator ruling 2026-09-25): on a terminal, input pending after the
// prompted line -- a multi-line paste or type-ahead -- is read, shown on
// stderr and discarded, so the shell never runs it.
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
#[test]
fn pasted_and_typed_ahead_lines_are_drained_and_shown() {
    let bin = assert_cmd::cargo::cargo_bin("mnemonic");
    let argv = [
        "convert",
        "--from",
        "phrase=@env:F687_SEED",
        "--to",
        "fingerprint",
        "--template",
        "bip84",
        "--passphrase",
        "-",
    ];
    let env = [("F687_SEED", SEED)];
    let p = "Enter passphrase: ";
    let label = "note: discarded";
    for (what, writes, lines, text) in [
        // F-687d: each discarded line is MASKED (<= 8 chars shown as is,
        // two-space indent); see drain_preview.json.
        ("paste", vec![&b"TREZOR\nline2\nline3\n"[..]], 2usize, "  line2\n  line3"),
        ("type-ahead", vec![&b"TREZOR\n"[..], b"ls -la\n"], 1, "  ls -la"),
        ("partial line", vec![&b"TREZOR\npartial"[..]], 1, "  partial"),
        (
            "a pasted 12-word seed",
            vec![&b"TREZOR\nabandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about\n"[..]],
            1,
            "  abandon \u{2026} (12 words, 93 chars)",
        ),
        ("an escape sequence", vec![&b"TREZOR\n\x1b[2J\n"[..]], 1, "  ?"),
        (
            "multi-byte text",
            vec!["TREZOR\n\u{43f}\u{430}\u{440}\u{43e}\u{43b}\u{44c}-\u{441}\u{435}\u{43a}\u{440}\u{435}\u{442}\n".as_bytes()],
            1,
            "  \u{43f}\u{430}\u{440}\u{43e}\u{43b}\u{44c}-\u{441}\u{2026} (1 word, 13 chars)",
        ),
    ] {
        let r = run_on_a_terminal_writes(&bin, &argv, &env, p, &writes);
        assert_eq!(r.code, Some(0), "{what}: {}", r.stderr);
        assert!(
            r.stdout.contains("fingerprint: b4e3f5ed"),
            "{what}: {}",
            r.stdout
        );
        assert!(
            r.stderr.contains(&format!(
                "{label} {lines} line(s) typed after the passphrase (not run, not used):\n{text}\n"
            )),
            "{what}: {:?}",
            r.stderr
        );
        assert!(
            r.left_for_shell.is_empty(),
            "{what}: left for the shell: {:?}",
            r.left_for_shell
        );
        // Masked: never the seed's tail, never a raw escape byte.
        assert!(!r.stderr.contains("about"), "{what}: the full line leaked: {}", r.stderr);
        assert!(!r.stderr.contains('\u{1b}'), "{what}: a raw ESC reached stderr");
        assert!(r.echo_after, "{what}: echo left off");
        assert!(
            !String::from_utf8_lossy(&r.shown).contains("line2"),
            "{what}: echoed"
        );
    }
    // Nothing pending: no note, nothing left.
    let r = run_on_a_terminal(&bin, &argv, &env, p, b"TREZOR\n");
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    assert!(!r.stderr.contains(label), "{}", r.stderr);
    assert!(r.left_for_shell.is_empty());
    assert!(r.echo_after);
    // Every prompt drains: the BIP-38 prompt names its own noun.
    let bargv = [
        "convert",
        "--from",
        "wif=@env:F687_W",
        "--to",
        "bip38",
        "--bip38-passphrase",
        "-",
    ];
    let typed = format!("{BIP38_PW}\nstray\n");
    let r = run_on_a_terminal(
        &bin,
        &bargv,
        &[("F687_W", BIP38_WIF)],
        "Enter BIP-38 passphrase: ",
        typed.as_bytes(),
    );
    assert!(r.stdout.contains(BIP38_OUT), "{}", r.stdout);
    assert!(
        r.stderr
            .contains("1 line(s) typed after the BIP-38 passphrase"),
        "{}",
        r.stderr
    );
    assert!(r.left_for_shell.is_empty());
    // A pipe is unchanged: read to EOF under the byte rule, no drain, no note.
    let r = run(&s(&argv), b"TREZOR\nline2\n", &[]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(!r.stderr.contains(label), "{}", r.stderr);
    // ...the passphrase is the whole stream "TREZOR\nline2", not "TREZOR".
    assert!(!r.stdout.contains("b4e3f5ed"), "{}", r.stdout);
}

/// `verify-bundle --ms1 -` on a terminal drains too, naming the ms1.
#[cfg(target_os = "linux")]
#[test]
fn verify_bundle_ms1_prompt_drains_a_paste() {
    let r = run(
        &s(&[
            "bundle",
            "--slot",
            "@0.phrase=@env:F687_SEED",
            "--network",
            "mainnet",
            "--template",
            "bip84",
        ]),
        b"",
        &[],
    );
    let ms1 = r
        .stdout
        .lines()
        .find(|l| l.starts_with("ms1"))
        .unwrap()
        .to_string();
    let mut argv: Vec<String> = s(&[
        "verify-bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        "@0.phrase=@env:F687_SEED",
        "--ms1",
        "-",
        "--",
    ]);
    argv.extend(
        r.stdout
            .lines()
            .filter(|l| l.starts_with("mk1") || l.starts_with("md1"))
            .map(str::to_string),
    );
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    let bin = assert_cmd::cargo::cargo_bin("mnemonic");
    let typed = format!("{ms1}\necho pwn\n");
    let r = run_on_a_terminal(
        &bin,
        &argv,
        &[("F687_SEED", SEED)],
        "Enter ms1: ",
        typed.as_bytes(),
    );
    assert!(
        r.stdout.lines().any(|l| l == "result: ok"),
        "{}\n{}",
        r.stdout,
        r.stderr
    );
    assert!(
        r.stderr
            .contains("1 line(s) typed after the ms1 (not run, not used):\n  echo pwn\n"),
        "{}",
        r.stderr
    );
    assert!(r.left_for_shell.is_empty(), "{:?}", r.left_for_shell);
    assert!(r.echo_after);
}

/// F-687c: a Ctrl-C arriving DURING the drain (40 ms after Enter; the drain
/// listens for 100 ms) still restores the terminal and exits by SIGINT. With
/// ISIG off in the drain the ^C would be read as data and the run would exit
/// 0 -- which is what makes this test fail on that mutation.
#[cfg(target_os = "linux")]
#[test]
fn ctrl_c_during_the_drain_leaves_a_working_terminal() {
    let (bin, argv, env) = drain_target();
    let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
    let env: Vec<(&str, &str)> = env.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let r = run_on_a_terminal_writes(
        &bin,
        &argv,
        &env,
        "Enter passphrase: ",
        &[b"TREZOR\n", b"", b"\x03"],
    );
    assert_eq!(
        r.signal,
        Some(libc::SIGINT),
        "code {:?}: {}",
        r.code,
        r.stderr
    );
    assert!(r.stdout.is_empty(), "{}", r.stdout);
    assert!(r.echo_after, "a signal during the drain left echo OFF");
}

#[cfg(target_os = "linux")]
fn drain_target() -> (std::path::PathBuf, Vec<String>, Vec<(String, String)>) {
    (
        assert_cmd::cargo::cargo_bin("mnemonic"),
        s(&[
            "convert",
            "--from",
            "phrase=@env:F687_SEED",
            "--to",
            "fingerprint",
            "--template",
            "bip84",
            "--passphrase",
            "-",
        ]),
        vec![("F687_SEED".to_string(), SEED.to_string())],
    )
}
