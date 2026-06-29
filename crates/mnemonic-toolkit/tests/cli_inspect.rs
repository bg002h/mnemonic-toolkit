//! Integration tests for `mnemonic inspect` — Phase 4 v0.22.0 cycle.
//!
//! Realizes `design/IMPLEMENTATION_PLAN_repair_v0_22.md` §4.3 cells 15-17
//! plus a Phase 4 scope-substitution for cell 18 (auto-fire is wired in
//! Phase 5; here we assert that a corrupted-but-not-auto-fired input
//! surfaces the typed sibling-codec error).
//!
//! Cells:
//!   15. ms1 happy-path (text-form output structure + secret suppression)
//!   16. mk1 + md1 happy-paths (per-kind text-form structure)
//!   17. --reveal-secret gate on ms1 entropy_hex
//!   18a. Phase-4 fail-loudly: corrupted ms1 surfaces typed MsCodec error
//!        (NOT exit 5 yet — Phase 5 upgrades to auto-fire short-circuit).
//!
//! Fixtures: same toolkit-emitted bundle for `abandon × 11 about` used by
//! `cli_repair.rs` (canonical BIP-39 test phrase).

use assert_cmd::Command;
use predicates::prelude::*;

const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const VALID_MK1_CHUNK0: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
const VALID_MK1_CHUNK1: &str =
    "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";
const VALID_MD1_CHUNK0: &str =
    "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
const VALID_MD1_CHUNK1: &str =
    "md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d";
const VALID_MD1_CHUNK2: &str =
    "md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn";

// `flip_at` lived here while cell 18a (Phase 4 placeholder) was active.
// Phase 5 moved the corrupted-input cell to `cli_auto_repair.rs`, so this
// helper now lives there. cli_inspect.rs only tests happy-path / valid
// inputs, so no flip helper is needed here.

/// Cell 15: ms1 happy-path — text-form output structure + secret hidden
/// by default.
#[test]
fn cell_15_ms1_text_form_structure_secret_suppressed_by_default() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--ms1", VALID_MS1])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("kind: ms1"))
        .stdout(predicate::str::contains("tag: entr"))
        .stdout(predicate::str::contains("byte_length: 16"))
        .stdout(predicate::str::contains("bit_strength: 128"))
        // Without --reveal-secret, hex is replaced with a hint.
        .stdout(predicate::str::contains(
            "<suppressed; pass --reveal-secret",
        ))
        // Private-key-material advisory fires whenever ms1 hits stdout.
        .stderr(predicate::str::contains(
            "warning: stdout carries private key material (can spend)",
        ));
}

/// Cell 16: mk1 + md1 happy-paths — verify per-kind text structure
/// (combined cell per plan).
#[test]
fn cell_16_mk1_and_md1_text_form_structures() {
    // mk1 sub-cell: full 2-chunk bundle should yield xpub-bearing summary.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "inspect",
            "--mk1",
            VALID_MK1_CHUNK0,
            "--mk1",
            VALID_MK1_CHUNK1,
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("kind: mk1"))
        .stdout(predicate::str::contains("policy_id_stub_count:"))
        .stdout(predicate::str::contains("origin_fingerprint:"))
        .stdout(predicate::str::contains("origin_path: m/84'/0'/0'"))
        .stdout(predicate::str::contains("xpub: xpub"));

    // md1 sub-cell: 3-chunk bundle, structural summary fields.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "inspect",
            "--md1",
            VALID_MD1_CHUNK0,
            "--md1",
            VALID_MD1_CHUNK1,
            "--md1",
            VALID_MD1_CHUNK2,
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("kind: md1"))
        .stdout(predicate::str::contains("placeholder_count: 1"))
        .stdout(predicate::str::contains("tree_tag: Wpkh"))
        .stdout(predicate::str::contains("wallet_policy_mode: true"))
        .stdout(predicate::str::contains("path_decl_shape: Shared"));
}

/// Cell 17: --reveal-secret gate — default hides, flag exposes.
#[test]
fn cell_17_reveal_secret_gate_on_ms1_entropy_hex() {
    // Default: no hex.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--ms1", VALID_MS1])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("entropy_hex: <suppressed"))
        .stdout(predicate::str::contains("entropy_hex: 00000000000000000000000000000000").not());

    // With --reveal-secret: hex appears.
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--reveal-secret", "--ms1", VALID_MS1])
        .assert()
        .code(0)
        .stdout(predicate::str::contains(
            "entropy_hex: 00000000000000000000000000000000",
        ))
        .stdout(predicate::str::contains("<suppressed").not());
}

// Phase-4 placeholder `cell_18a` was superseded by Phase 5's auto-fire
// wire-up. The canonical "bad ms1 → auto-fire short-circuit exit 5" test
// lives in `tests/cli_auto_repair.rs::cell_18b_inspect_auto_fire_on_corrupted_ms1`.
// The `--no-auto-repair`-suppressed shape (typed sibling-codec error, NOT
// exit 5) is covered by `cli_auto_repair.rs::cell_22`.

/// v0.27.0 cell: assert `--json` envelope carries the current `schema_version`
/// at top level for each kind variant (v0.75.0 bumped it `"1"`→`"2"` with the
/// md1 `template` field). Closes `inspect-json-schema-version-backfill`
/// FOLLOWUP. Mirrors `cli_xpub_search_path_of_xpub::path_of_xpub_phrase_zpub_match_bip84`
/// pattern (assert `v["schema_version"] == "2"`).
#[test]
fn inspect_json_envelope_schema_version_v_0_27_0() {
    // ms1 kind
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--json", "--ms1", VALID_MS1])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let body = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(body.trim()).unwrap();
    assert_eq!(v["schema_version"], "2", "ms1 envelope schema_version");
    assert_eq!(v["kind"], "ms1");
    assert_eq!(v["byte_length"], 16);

    // mk1 kind
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "inspect",
            "--json",
            "--mk1",
            VALID_MK1_CHUNK0,
            "--mk1",
            VALID_MK1_CHUNK1,
        ])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let body = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(body.trim()).unwrap();
    assert_eq!(v["schema_version"], "2", "mk1 envelope schema_version");
    assert_eq!(v["kind"], "mk1");

    // md1 kind
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "inspect",
            "--json",
            "--md1",
            VALID_MD1_CHUNK0,
            "--md1",
            VALID_MD1_CHUNK1,
            "--md1",
            VALID_MD1_CHUNK2,
        ])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let body = String::from_utf8(out).unwrap();
    let v: serde_json::Value = serde_json::from_str(body.trim()).unwrap();
    assert_eq!(v["schema_version"], "2", "md1 envelope schema_version");
    assert_eq!(v["kind"], "md1");
}

// ============================================================================
// v0.75.0 — md1 `template:` line (default-on, text + --json) + schema v2
// ============================================================================
//
// `mnemonic inspect <md1>` now leads the md1 body with the BIP-388 keyless
// `@N` wallet-policy template, rendered by the canonical
// `md_codec::descriptor_to_template` (relocated into md-codec 0.40.0; `md`
// CLI delegates to the same fn → byte-identical). The toolkit ↔ in-crate ↔
// frozen-md-cli-0.11.2-snapshot three-way equality is asserted here; the true
// cross-binary `== md decode` parity is the MD_BIN-gated end-to-end step.

/// `(label, frozen single-string md1 [md-cli 0.11.2 KAT corpus], expected @N
/// template)`. The single-string form is re-chunked in-crate (decode →
/// split → reassemble) so the toolkit's chunked `decode_card` (reassemble-only)
/// accepts it; the rendered `template:` line is asserted equal to BOTH the
/// in-crate `descriptor_to_template` render AND this frozen expected string.
const MD1_TEMPLATE_CORPUS: &[(&str, &str, &str)] = &[
    (
        "pathological_or_i",
        "md1yzfdsssj5qqcyefnfgdsqr6zgqvzzcrfln7t3kzht2u",
        "wsh(or_i(and_v(v:pk(@0/<0;1>/*),after(1000000)),multi(2,@1/<0;1>/*,@2/<0;1>/*)))",
    ),
    (
        "tr_nums_sortedmulti_a",
        "md1yz80tgggqps8ys3psu9rrkfee0tpv2",
        "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
    ),
    ("wpkh", "md1yqpqqxqq8xtwhw4xwn4qh", "wpkh(@0/<0;1>/*)"),
];

/// Re-chunk a frozen single-string md1 into the chunked wire form the toolkit's
/// `reassemble`-based decode accepts. Returns the reassembled Descriptor (the
/// SAME object the CLI builds from these chunks) plus the chunk strings.
fn rechunk(single: &str) -> (md_codec::Descriptor, Vec<String>) {
    let d0 = md_codec::decode_md1_string(single).expect("decode frozen single md1");
    let chunks = md_codec::chunk::split(&d0).expect("split into chunked wire form");
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    let d = md_codec::chunk::reassemble(&refs).expect("reassemble chunked wire form");
    (d, chunks)
}

/// v0.75.0: `mnemonic inspect <md1>` text emits a `template:` line as the FIRST
/// md1 line, whose value equals the in-crate `descriptor_to_template` render
/// (and the frozen md-cli-0.11.2 snapshot) — for the pathological example +
/// tr(NUMS,sortedmulti_a) + wpkh.
#[test]
fn inspect_md1_text_template_line_matches_in_crate_render() {
    for (label, single, expected) in MD1_TEMPLATE_CORPUS {
        let (d, chunks) = rechunk(single);
        let rendered = md_codec::descriptor_to_template(&d)
            .unwrap_or_else(|e| panic!("[{label}] in-crate render failed: {e}"));
        assert_eq!(
            &rendered, expected,
            "[{label}] in-crate render must equal the frozen md-cli-0.11.2 snapshot"
        );

        let mut args = vec!["inspect".to_string()];
        for c in &chunks {
            args.push("--md1".to_string());
            args.push(c.clone());
        }
        let out = Command::cargo_bin("mnemonic")
            .unwrap()
            .args(&args)
            .assert()
            .code(0)
            .get_output()
            .stdout
            .clone();
        let body = String::from_utf8(out).unwrap();
        let first = body.lines().next().unwrap_or("");
        assert_eq!(
            first,
            format!("template: {expected}"),
            "[{label}] first md1 line must be the template; got stdout:\n{body}"
        );
    }
}

/// v0.75.0: `mnemonic inspect --json <md1>` carries a `template` field equal to
/// the in-crate render, and the shared envelope reports `schema_version: "2"`.
#[test]
fn inspect_md1_json_carries_template_and_schema_v2() {
    for (label, single, expected) in MD1_TEMPLATE_CORPUS {
        let (d, chunks) = rechunk(single);
        let rendered = md_codec::descriptor_to_template(&d).unwrap();

        let mut args = vec!["inspect".to_string(), "--json".to_string()];
        for c in &chunks {
            args.push("--md1".to_string());
            args.push(c.clone());
        }
        let out = Command::cargo_bin("mnemonic")
            .unwrap()
            .args(&args)
            .assert()
            .code(0)
            .get_output()
            .stdout
            .clone();
        let v: serde_json::Value =
            serde_json::from_str(String::from_utf8(out).unwrap().trim()).unwrap();
        assert_eq!(v["schema_version"], "2", "[{label}] schema_version");
        assert_eq!(v["kind"], "md1");
        assert_eq!(
            v["template"], *rendered,
            "[{label}] json template == in-crate render"
        );
        assert_eq!(
            v["template"], *expected,
            "[{label}] json template == snapshot"
        );
    }
}

/// v0.75.0: ms1/mk1 inspect bodies are UNCHANGED — no `template` line/field —
/// but the shared `--json` envelope still versions to `schema_version: "2"`.
#[test]
fn inspect_ms1_mk1_bodies_have_no_template_but_envelope_is_schema_v2() {
    // ms1 text: no template line.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--ms1", VALID_MS1])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let body = String::from_utf8(out).unwrap();
    assert!(
        !body.contains("template:"),
        "ms1 text must not carry a template line; got:\n{body}"
    );

    // ms1 json: schema_version "2", no template field.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--json", "--ms1", VALID_MS1])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8(out).unwrap().trim()).unwrap();
    assert_eq!(v["schema_version"], "2", "ms1 envelope schema_version");
    assert!(
        v["template"].is_null(),
        "ms1 json must not carry a template field"
    );

    // mk1 text: no template line.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "inspect",
            "--mk1",
            VALID_MK1_CHUNK0,
            "--mk1",
            VALID_MK1_CHUNK1,
        ])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let body = String::from_utf8(out).unwrap();
    assert!(
        !body.contains("template:"),
        "mk1 text must not carry a template line; got:\n{body}"
    );

    // mk1 json: schema_version "2", no template field.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "inspect",
            "--json",
            "--mk1",
            VALID_MK1_CHUNK0,
            "--mk1",
            VALID_MK1_CHUNK1,
        ])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8(out).unwrap().trim()).unwrap();
    assert_eq!(v["schema_version"], "2", "mk1 envelope schema_version");
    assert!(
        v["template"].is_null(),
        "mk1 json must not carry a template field"
    );
}

// ============================================================================
// Audit M3 — inline-ms1 secret-in-argv advisory (inspect intake)
// ============================================================================
//
// `resolve_groups` (src/repair.rs) fires `secret_in_argv_warning` per
// occurrence on the RAW pre-expansion values: unconditionally for every
// non-`-` `--ms1` flag value, and for each positional that HRP-classifies
// as ms1. mk1/md1 values (public material) and stdin-expanded chunks
// never fire.

const ARGV_ADVISORY_PREFIX: &str = "warning: secret material on argv";

/// RED (M3): `inspect --ms1 <inline>` puts a secret on argv — the advisory
/// must fire, naming the flag and the `--ms1 -` stdin alternative.
#[test]
fn inspect_inline_ms1_fires_secret_in_argv_advisory() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--ms1", VALID_MS1])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("warning: secret material on argv (--ms1)"),
        "inline --ms1 must fire the argv advisory; stderr: {stderr:?}"
    );
    assert!(
        stderr.contains("pipe via --ms1 -"),
        "advisory must point at the --ms1 - stdin alternative; stderr: {stderr:?}"
    );
}

/// RED (M3, per-occurrence): `--ms1 <inline>` PLUS a positional ms1 in one
/// invocation → TWO advisories, one per occurrence with distinct labels.
/// (`--ms1` is single-occurrence clap-side, so the second inline source
/// must be a positional.)
#[test]
fn inspect_mixed_inline_flag_and_positional_ms1_fire_per_occurrence() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--ms1", VALID_MS1, VALID_MS1])
        .assert()
        .get_output()
        .clone();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(
        stderr
            .matches("warning: secret material on argv (--ms1)")
            .count(),
        1,
        "flag occurrence fires exactly once; stderr: {stderr:?}"
    );
    assert_eq!(
        stderr
            .matches("warning: secret material on argv (positional ms1)")
            .count(),
        1,
        "positional occurrence fires exactly once; stderr: {stderr:?}"
    );
}

/// Guard (M3): `--ms1 -` (stdin route — the advisory's own recommendation)
/// must NOT fire, and the stdin-expanded chunk must not retro-fire either.
#[test]
fn inspect_ms1_stdin_dash_does_not_fire_argv_advisory() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", "--ms1", "-"])
        .write_stdin(format!("{VALID_MS1}\n"))
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains(ARGV_ADVISORY_PREFIX),
        "--ms1 - (stdin) must not fire the argv advisory; stderr: {stderr:?}"
    );
}

/// Guard (M3): positional mk1/md1-only intake (public material) never fires.
#[test]
fn inspect_positional_mk1_md1_only_does_not_fire_argv_advisory() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "inspect",
            VALID_MK1_CHUNK0,
            VALID_MK1_CHUNK1,
            VALID_MD1_CHUNK0,
            VALID_MD1_CHUNK1,
            VALID_MD1_CHUNK2,
        ])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        !stderr.contains(ARGV_ADVISORY_PREFIX),
        "mk1/md1 positionals are public material — no argv advisory; stderr: {stderr:?}"
    );
}
