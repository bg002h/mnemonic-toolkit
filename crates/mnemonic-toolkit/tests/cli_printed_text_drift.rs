//! Printed-text drift rows of design/FIX-tk-printed-text-drift.md (bughunt-1119 bh-tk-I9, I10, I11).
//! One test per RED row of section 5, named as in section 6. Public test material only.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_mnemonic");
const X: &str = "xprv9s21ZrQH143K2LBWUUQRFXhucrQqBpKdRRxNVq2zBqsx8HVqFk2uYo8kmbaLLHRdqtQpUm98uKfu3vca1LqdGhUtyoFnCNkfmXRyPXLjbKb";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn norm(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Run the binary; returns (exit code, stdout).
fn run(args: &[&str], stdin: &str) -> (i32, String) {
    let mut c = Command::new(BIN)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    let o = c.wait_with_output().unwrap();
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stdout).into_owned(),
    )
}

/// Every `e.g.,? `...`` span of a whitespace-normalised text.
fn eg_spans(t: &str) -> Vec<String> {
    let mut v = Vec::new();
    let mut rest = t;
    while let Some(i) = rest.find("e.g.") {
        let mut after = &rest[i + 4..];
        if let Some(a) = after.strip_prefix(',') {
            after = a;
        }
        rest = after;
        if let Some(a) = after.strip_prefix(" `") {
            if let Some(j) = a.find('`') {
                v.push(a[..j].to_string());
                rest = &a[j + 1..];
            }
        }
    }
    v
}

fn help(args: &[&str]) -> String {
    let mut a: Vec<&str> = args.to_vec();
    a.push("--help");
    norm(&run(&a, "").1)
}

fn app_entry(tok: &str) -> String {
    const NAMES: [&str; 9] = [
        "bip39",
        "hd-seed",
        "xprv",
        "hex",
        "password-base64",
        "password-base85",
        "dice",
        "rsa",
        "rsa-gpg",
    ];
    let (_, h) = run(&["derive-child", "--help"], "");
    let (mut inapp, mut on) = (false, false);
    let mut out: Vec<String> = Vec::new();
    for line in h.lines() {
        let t = line.trim_start();
        if t.starts_with("--application ") {
            inapp = true;
            continue;
        }
        if inapp && t.starts_with("--") {
            inapp = false;
        }
        if !inapp {
            continue;
        }
        let first = t.split_whitespace().next().unwrap_or("");
        if NAMES.contains(&first) && t[first.len()..].starts_with("  ") {
            on = first == tok;
        }
        if t.is_empty() || t.starts_with("--") {
            on = false;
        }
        if on {
            out.push(t.to_string());
        }
    }
    norm(&out.join(" "))
}

/// Lines of `file` after the first line starting with `start`, up to (excluding) the next line
/// starting with `end` (None: to the end of the file).
fn section_lines(file: &str, start: &str, end: Option<&str>) -> Vec<(usize, String)> {
    let mut v = Vec::new();
    let mut on = false;
    for (i, l) in file.lines().enumerate() {
        if !on {
            if l.starts_with(start) {
                on = true;
            }
            continue;
        }
        if end.is_some_and(|e| l.starts_with(e)) {
            break;
        }
        v.push((i + 1, l.to_string()));
    }
    v
}

/// Logical commands of the ```sh blocks: (command, expected first stdout line).
fn extract(lines: &[(usize, String)]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let (mut cur, mut exp) = (String::new(), String::new());
    let mut insh = false;
    let flush = |cur: &mut String, exp: &mut String, out: &mut Vec<(String, String)>| {
        if !cur.is_empty() {
            out.push((std::mem::take(cur), std::mem::take(exp)));
        }
        cur.clear();
        exp.clear();
    };
    for (_, l) in lines {
        if l.starts_with("```sh") {
            insh = true;
            cur.clear();
            exp.clear();
            continue;
        }
        if l.starts_with("```") {
            if insh {
                flush(&mut cur, &mut exp, &mut out);
            }
            insh = false;
            continue;
        }
        if !insh {
            continue;
        }
        let t = l.trim_start();
        if let Some(v) = t.strip_prefix("# → ") {
            if !cur.is_empty() {
                exp = v.split("  ").next().unwrap().to_string();
            }
            continue;
        }
        if t.starts_with('#') {
            continue;
        }
        if t.is_empty() {
            flush(&mut cur, &mut exp, &mut out);
            continue;
        }
        if !cur.is_empty() && (cur.ends_with('\\') || t.starts_with('|')) {
            cur.push('\n');
            cur.push_str(l);
        } else {
            flush(&mut cur, &mut exp, &mut out);
            cur = l.clone();
        }
    }
    flush(&mut cur, &mut exp, &mut out);
    out
}

/// Run every command naming `needle` (and not matching `skip`); returns (runnable, failing, report).
fn run_section(lines: &[(usize, String)], needle: &str, skip: &str) -> (usize, usize, String) {
    let bindir = Path::new(BIN).parent().unwrap().to_path_buf();
    let path = format!(
        "{}:{}",
        bindir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let (mut n, mut bad, mut rep) = (0, 0, String::new());
    for (cmd, exp) in extract(lines) {
        if !cmd.contains(needle) || cmd.contains(skip) {
            continue;
        }
        let k = [cmd.find("| xxd"), cmd.find("| qrencode")]
            .into_iter()
            .flatten()
            .min();
        let mut cmd = k.map_or(cmd.clone(), |k| cmd[..k].to_string());
        cmd = cmd
            .trim_end_matches(|c: char| c.is_whitespace() || c == '\\')
            .to_string();
        n += 1;
        let o = Command::new("bash")
            .args(["-o", "pipefail", "-c", &cmd])
            .env("PATH", &path)
            .output()
            .unwrap();
        let out = String::from_utf8_lossy(&o.stdout)
            .trim_end_matches('\n')
            .to_string();
        let first = out.lines().next().unwrap_or("").to_string();
        let rc = o.status.code().unwrap_or(-1);
        if rc != 0 || out.is_empty() || (!exp.is_empty() && first != exp) {
            bad += 1;
            rep.push_str(&format!(
                "  [{}] rc={rc} out={first:.48} expect={exp:.48} err={:.110}\n",
                norm(&cmd),
                String::from_utf8_lossy(&o.stderr)
            ));
        }
    }
    (n, bad, rep)
}

/// sh-block lines of the section whose secret is given inline: `--from <kind>=<not '-'>` or `--digits <not '-'>`.
fn inline_lines(lines: &[(usize, String)], kinds: &[&str], digits: bool) -> Vec<String> {
    let mut insh = false;
    let mut v = Vec::new();
    for (n, l) in lines {
        if l.starts_with("```sh") {
            insh = true;
            continue;
        }
        if l.starts_with("```") {
            insh = false;
            continue;
        }
        if !insh {
            continue;
        }
        let nondash = |p: &str| {
            l.match_indices(p)
                .any(|(i, _)| l[i + p.len()..].chars().next().is_some_and(|c| c != '-'))
        };
        if kinds.iter().any(|k| nondash(&format!("--from {k}=")))
            || (digits && nondash("--digits "))
        {
            v.push(format!("{n}: {l}"));
        }
    }
    v
}

fn seedqr_section() -> Vec<(usize, String)> {
    section_lines(
        &read("docs/manual/src/40-cli-reference/41-mnemonic.md"),
        "## `mnemonic seedqr`",
        Some("## "),
    )
}

fn bip85_file() -> String {
    read("docs/manual/src/30-workflows/38-bip85-children.md")
}

#[test]
fn h1_compare_cost_help_example_runs() {
    let spans = eg_spans(&help(&["compare-cost"]));
    let ex = spans.first().expect("compare-cost --help has an e.g. span");
    let (rc, out) = run(&["compare-cost", "--miniscript", ex], "");
    assert_eq!(rc, 0, "example [{ex}] rc={rc}");
    assert_eq!(out.lines().next().unwrap(), "Input: or_b(pk(A),s:pk(B))");
}

#[test]
fn h2_help_eg_examples_inventory() {
    let list = read("docs/manual/tests/cli-subcommands.list");
    let mut got: Vec<String> = Vec::new();
    for l in list.lines() {
        let mut w = l.split_whitespace();
        if w.next() != Some("mnemonic") {
            continue;
        }
        let sub: Vec<&str> = w.collect();
        for s in eg_spans(&help(&sub)) {
            got.push(format!(
                "{}: {s}",
                if sub.is_empty() {
                    "<root>".into()
                } else {
                    sub.join(" ")
                }
            ));
        }
    }
    got.sort();
    let mut want: Vec<String> = [
        "compare-cost: or_b(pk(A),s:pk(B))",
        "convert: --to mk1",
        "export-wallet: bip84-0",
        "gen-man: mnemonic-seed-xor-split.1",
        "restore: --account 0,1,2,3",
        "restore: 2h",
        "restore: m/84'/0'/7'",
        "verify-bundle: 2h",
        "verify-bundle: m/84'/0'/7'",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    want.sort();
    assert_eq!(
        got, want,
        "a --help text gained or lost an `e.g.` example; if it is a runnable command add a run row like h1, then update this list"
    );
}

#[test]
fn d1_derive_child_help_hd_seed_is_application_2() {
    assert_eq!(
        app_entry("hd-seed"),
        "hd-seed BIP-85 application 2' — derives a private key, printed as a compressed WIF, at m/83696968'/2'/<index>' (`--length 0`)"
    );
}

#[test]
fn d2_derive_child_help_xprv_is_application_32() {
    assert_eq!(
        app_entry("xprv"),
        "xprv BIP-85 application 32' — derives a child xprv at m/83696968'/32'/<index>' (`--length 0`)"
    );
}

#[test]
fn d3_derive_child_help_dice_is_runnable_with_bip85_range() {
    assert_eq!(
        app_entry("dice"),
        "dice BIP-85 application 89101' — derives `--length` rolls (1..=2^31-1) of a `--dice-sides`-sided die"
    );
}

#[test]
fn d4_derive_child_help_bip39_lists_five_lengths() {
    assert_eq!(
        app_entry("bip39"),
        "bip39 BIP-85 application 39' — derives a child BIP-39 mnemonic of `--length` words (12 / 15 / 18 / 21 / 24)"
    );
}

#[test]
fn d5_derive_child_help_rsa_gpg_is_application_828365() {
    assert_eq!(
        app_entry("rsa-gpg"),
        "rsa-gpg BIP-85 application 828365' (RSA-GPG key scheme); REFUSED at runtime"
    );
}

#[test]
fn d6_derive_child_dice_bounds_match_binary() {
    let dice = |sides: &str, len: &str| {
        run(
            &[
                "derive-child",
                "--from",
                "xprv=-",
                "--application",
                "dice",
                "--dice-sides",
                sides,
                "--length",
                len,
                "--index",
                "0",
            ],
            X,
        )
        .0
    };
    assert_ne!(dice("6", "2147483648"), 0, "rolls 2^31");
    assert_eq!(dice("2147483647", "3"), 0, "sides 2^31-1");
    assert_ne!(dice("2147483648", "3"), 0, "sides 2^31");
    let h = help(&["derive-child"]);
    let i = h.find("rolls (1..=2^31-1)").expect("help rolls bound");
    assert!(
        h[i..].contains("Range: 2..=2^31-1"),
        "help dice-sides bound"
    );
}

fn derive(app: &str, len: &str) -> String {
    run(
        &[
            "derive-child",
            "--from",
            "xprv=-",
            "--application",
            app,
            "--length",
            len,
            "--index",
            "0",
        ],
        X,
    )
    .1
}

#[test]
fn p1_hd_seed_bip85_vector_application_2() {
    assert_eq!(
        derive("hd-seed", "0").trim(),
        "Kzyv4uF39d4Jrw2W7UryTHwZr1zQVNk4dAFyqE6BuMrMh1Za7uhp"
    );
}

#[test]
fn p2_xprv_bip85_vector_application_32() {
    assert_eq!(
        derive("xprv", "0").trim(),
        "xprv9s21ZrQH143K2srSbCSg4m4kLvPMzcWydgmKEnMmoZUurYuBuYG46c6P71UGXMzmriLzCCBvKQWBUv3vPB3m1SATMhp3uEjXHJ42jFg7myX"
    );
}

#[test]
fn p3_dice_100_rolls_and_bip39_15_21_run() {
    let (rc, o) = run(
        &[
            "derive-child",
            "--from",
            "xprv=-",
            "--application",
            "dice",
            "--dice-sides",
            "6",
            "--length",
            "100",
            "--index",
            "0",
        ],
        X,
    );
    assert_eq!(rc, 0);
    assert_eq!(o.split([',', '\n']).filter(|s| !s.is_empty()).count(), 100);
    assert_eq!(derive("bip39", "15").split_whitespace().count(), 15);
    assert_eq!(derive("bip39", "21").split_whitespace().count(), 21);
}

#[test]
fn s1_manual_seedqr_commands_run_as_printed() {
    let (n, bad, rep) = run_section(&seedqr_section(), "mnemonic seedqr", "--json-out");
    assert!(bad == 0 && n == 8, "runnable={n} failing={bad}\n{rep}");
}

#[test]
fn s2_manual_seedqr_no_inline_secret() {
    let v = inline_lines(&seedqr_section(), &["phrase", "seedqr"], true);
    assert!(v.is_empty(), "inline secrets: {v:?}");
}

#[test]
fn s3_manual_seedqr_argv_paragraph_says_refused() {
    let sec = norm(
        &seedqr_section()
            .iter()
            .map(|(_, l)| l.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    );
    assert!(
        !sec.contains("to avoid the advisory") && !sec.contains("argv-leakage advisory"),
        "old advisory text"
    );
    assert!(sec.contains("(#argv-refusal)"), "no link to #argv-refusal");
}

#[test]
fn b1_manual_bip85_commands_run_as_printed() {
    let f = bip85_file();
    let lines = section_lines(&f, "# Deterministic child secrets", None);
    let (n, bad, rep) = run_section(&lines, "mnemonic derive-child", "<");
    assert!(bad == 0 && n == 5, "runnable={n} failing={bad}\n{rep}");
}

#[test]
fn b2_manual_bip85_no_inline_secret() {
    let f = bip85_file();
    let lines = section_lines(&f, "# Deterministic child secrets", None);
    let v = inline_lines(&lines, &["phrase", "xprv"], false);
    assert!(v.is_empty(), "inline secrets: {v:?}");
}

#[test]
fn b3_manual_bip85_application_facts() {
    let f = bip85_file();
    let mut problems = Vec::new();
    for want in [
        "| `bip39` | new BIP-39 phrase (10 BIP-85-coded languages) | `12`, `15`, `18`, `21`, `24` |",
        "| `hd-seed` | private key as a compressed WIF | `0` (sentinel) |",
        "| `dice` | deterministic dice rolls | `1..=2^31-1` |",
        "  `--passphrase @env:VAR` for a passphrase: stdin is already taken,",
        "  (12 / 15 / 18 / 21 / 24 words; 10 BIP-85 languages) for an air-gapped",
    ] {
        if !f.lines().any(|l| l == want) {
            problems.push(format!("missing: {want}"));
        }
    }
    for gone in [
        "HD-seed bytes",
        "64-byte HD-seed",
        "9 BIP-85",
        "land in v0.9",
        "1..=10000",
    ] {
        if f.contains(gone) {
            problems.push(format!("still present: {gone}"));
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}

#[test]
fn g1_gui_manual_hd_seed_and_xprv_facts() {
    let g = read("docs/manual-gui/src/40-mnemonic/46-derive-child.md");
    let section = |s: &str| {
        let mut on = false;
        let mut v = Vec::new();
        for l in g.lines() {
            if !on && l.starts_with(s) {
                on = true;
                v.push(l);
                continue;
            }
            if on && l.starts_with("### ") {
                break;
            }
            if on {
                v.push(l);
            }
        }
        norm(&v.join("\n"))
    };
    let hd = section("### `hd-seed` {#mnemonic-derive-child-application-hd-seed}");
    let xp = section("### `xprv` {#mnemonic-derive-child-application-xprv}");
    assert_eq!(hd, "### `hd-seed` {#mnemonic-derive-child-application-hd-seed} Emit a child private key as a compressed WIF (BIP-85 application 2', path `m/83696968'/2'/<index>'`). `--length` must be 0. `--network` selects the mainnet (`K…` / `L…`) or test-network (`c…`) WIF prefix.");
    assert_eq!(xp, "### `xprv` {#mnemonic-derive-child-application-xprv} Emit a child xprv at depth 0 (BIP-85 application 32', path `m/83696968'/32'/<index>'`). `--length` must be 0. `--network` selects the prefix.");
}
