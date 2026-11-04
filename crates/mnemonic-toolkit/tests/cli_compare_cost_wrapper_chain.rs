//! Triage E9 (grade-2026-11-03-tk-v5-v7.md line 74): `compare-cost --miniscript`
//! refuses valid miniscript whose wrapper chain has two or more letters.
//!
//! Cause under test: `cost::translate::collect_abstract_labels` reads the
//! identifier after every `(` or `,` and treats it as a key label unless it is
//! one of the single-letter wrappers in `KEYWORDS`. A chain such as `ln` in
//! `or_i(pk(A),ln:pk(B))` is therefore taken for a label, replaced by that
//! label's 66-hex dummy key, and rust-miniscript then reads the hex digits as
//! wrapper letters (`unknown wrapper «9:»`).
//!
//! Row classes:
//!   * `oracle_*`   pass on unmodified code: rust-miniscript at the pin, and the
//!                  `--descriptor` path of the same command, accept every shape.
//!   * `miniscript_path_*` are the RED rows: `--miniscript <shape>` must exit 0
//!                  and print the same per-condition figures as the oracle.
//!   * `control_*`  pass before and after any fix (single-letter wrappers and
//!                  real labels must keep working).
//!
//! Every shape here is valid in BOTH Segwitv0 and Tap (the `oracle_*` rows
//! prove it), so a refusal is the toolkit's fault, not the input's.

use std::str::FromStr;

use assert_cmd::Command;
use miniscript::descriptor::DefiniteDescriptorKey;
use miniscript::{Miniscript, Segwitv0, Tap};
use serde_json::Value;

/// Compressed keys standing in for labels `A` and `B` in the concrete form.
const KA: &str = "03998512205ec6a5cdb77d5b4f7de63c560d1e846162612ee178c49e7b6cc44fb9";
const KB: &str = "03210c9382bfe049b5edd00105a2a7aba51b688fc2ad922a44bee573a4c7f97fc9";

/// (row name, shape with labels A/B). Uppercase `A`/`B` occur only as labels.
const SHAPES: &[(&str, &str)] = &[
    ("ln", "or_i(pk(A),ln:pk(B))"),
    ("tv", "or_i(pk(A),tv:pk(B))"),
    ("sdv", "and_b(pk(A),sdv:older(10))"),
    ("snl", "and_b(pk(A),snl:older(10))"),
    ("sj", "and_b(pk(A),sj:pk(B))"),
];

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Run {
    let out = bin().args(args).output().expect("spawn mnemonic");
    Run {
        code: out.status.code().expect("exit code"),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

fn shape(name: &str) -> &'static str {
    SHAPES
        .iter()
        .find(|(n, _)| *n == name)
        .expect("known shape")
        .1
}

/// Segwitv0 form: labels become compressed keys.
fn concrete_segwit(s: &str) -> String {
    s.replace('A', KA).replace('B', KB)
}

/// Tap form: labels become x-only keys (drop the parity byte).
fn concrete_tap(s: &str) -> String {
    s.replace('A', &KA[2..]).replace('B', &KB[2..])
}

/// `(wsh_vbytes, tr_vbytes)` per condition, in output order.
fn figures(json_stdout: &str) -> Vec<(i64, i64)> {
    let v: Value = serde_json::from_str(json_stdout).expect("compare-cost --json parses");
    v["conditions"]
        .as_array()
        .expect("conditions array")
        .iter()
        .map(|c| {
            (
                c["wsh_vbytes"].as_i64().expect("wsh_vbytes"),
                c["tr_vbytes"].as_i64().expect("tr_vbytes"),
            )
        })
        .collect()
}

/// Oracle figures: the `--descriptor wsh(<concrete>)` path, which hands the
/// string straight to rust-miniscript.
fn oracle_figures(s: &str) -> Vec<(i64, i64)> {
    let desc = format!("wsh({})", concrete_segwit(s));
    let r = run(&["compare-cost", "--descriptor", &desc, "--json"]);
    assert_eq!(
        r.code, 0,
        "oracle --descriptor refused {desc}: stderr={} ",
        r.stderr
    );
    let f = figures(&r.stdout);
    assert!(!f.is_empty(), "oracle produced no conditions for {desc}");
    f
}

fn assert_miniscript_path_matches_oracle(name: &str) {
    let s = shape(name);
    let want = oracle_figures(s);
    let r = run(&["compare-cost", "--miniscript", s, "--json"]);
    assert_eq!(
        r.code, 0,
        "`compare-cost --miniscript '{s}'` must accept valid miniscript (the \
         --descriptor path and rust-miniscript do); got exit {} stderr={:?}",
        r.code, r.stderr
    );
    assert_eq!(
        figures(&r.stdout),
        want,
        "figures for '{s}' differ from the --descriptor oracle"
    );
}

// ── oracle rows: pass on unmodified code ────────────────────────────────────

#[test]
fn oracle_rust_miniscript_parses_every_shape_in_both_contexts() {
    for (name, s) in SHAPES {
        let seg = concrete_segwit(s);
        let tap = concrete_tap(s);
        Miniscript::<DefiniteDescriptorKey, Segwitv0>::from_str(&seg)
            .unwrap_or_else(|e| panic!("{name}: Segwitv0 parse of {seg} failed: {e}"));
        Miniscript::<DefiniteDescriptorKey, Tap>::from_str(&tap)
            .unwrap_or_else(|e| panic!("{name}: Tap parse of {tap} failed: {e}"));
    }
}

#[test]
fn oracle_descriptor_path_accepts_every_shape() {
    for (_, s) in SHAPES {
        let _ = oracle_figures(s);
    }
}

// ── RED rows: --miniscript with a multi-letter wrapper chain ────────────────

#[test]
fn miniscript_path_accepts_ln_chain() {
    assert_miniscript_path_matches_oracle("ln");
}

#[test]
fn miniscript_path_accepts_tv_chain() {
    assert_miniscript_path_matches_oracle("tv");
}

#[test]
fn miniscript_path_accepts_sdv_chain() {
    assert_miniscript_path_matches_oracle("sdv");
}

#[test]
fn miniscript_path_accepts_snl_chain() {
    assert_miniscript_path_matches_oracle("snl");
}

#[test]
fn miniscript_path_accepts_sj_chain() {
    assert_miniscript_path_matches_oracle("sj");
}

/// The same chain over concrete hex keys (no abstract labels at all) fails
/// today too, because the scan still takes `ln` for a label.
#[test]
fn miniscript_path_accepts_ln_chain_with_concrete_keys() {
    let s = concrete_segwit(shape("ln"));
    let want = oracle_figures(shape("ln"));
    let r = run(&["compare-cost", "--miniscript", &s, "--json"]);
    assert_eq!(
        r.code, 0,
        "concrete-key chain refused: stderr={:?}",
        r.stderr
    );
    assert_eq!(figures(&r.stdout), want);
}

// ── controls: pass before and after the fix ─────────────────────────────────

#[test]
fn control_single_letter_wrappers_still_parse() {
    for s in [
        "or_i(pk(A),l:pk(B))",
        "or_i(pk(A),n:pk(B))",
        "and_b(pk(A),s:pk(B))",
    ] {
        let r = run(&["compare-cost", "--miniscript", s]);
        assert_eq!(r.code, 0, "{s}: stderr={:?}", r.stderr);
    }
}

/// Real labels must still be substituted after any fix: `Alice` and `Bob` are
/// labels (no `:` follows them), not wrapper chains.
#[test]
fn control_real_labels_still_substituted() {
    let r = run(&[
        "compare-cost",
        "--miniscript",
        "or_b(pk(Alice),s:pk(Bob))",
        "--json",
    ]);
    assert_eq!(r.code, 0, "stderr={:?}", r.stderr);
    let v: Value = serde_json::from_str(&r.stdout).unwrap();
    let labels: Vec<&str> = v["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["label"].as_str().unwrap())
        .collect();
    assert_eq!(labels, vec!["Alice", "Bob"]);
}

/// A genuinely invalid chain must still be refused (typecheck), exit 2, and
/// must not be accepted by the fix: `dv:pk(B)` needs a zero-input operand.
#[test]
fn control_invalid_chain_still_refused() {
    let r = run(&["compare-cost", "--miniscript", "or_i(pk(A),sdv:pk(B))"]);
    assert_eq!(r.code, 2, "stdout={} stderr={:?}", r.stdout, r.stderr);
}
