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
const VALID_MK1_CHUNK1: &str = "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";
const VALID_MD1_CHUNK0: &str = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
const VALID_MD1_CHUNK1: &str = "md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d";
const VALID_MD1_CHUNK2: &str = "md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn";

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
        .stdout(predicate::str::contains("<suppressed; pass --reveal-secret"))
        // Private-key-material advisory fires whenever ms1 hits stdout.
        .stderr(predicate::str::contains("warning: stdout carries private key material (can spend)"));
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
        .stdout(
            predicate::str::contains("entropy_hex: 00000000000000000000000000000000").not(),
        );

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

/// v0.27.0 cell: assert `--json` envelope carries `schema_version: "1"` at
/// top level for each kind variant. Closes `inspect-json-schema-version-backfill`
/// FOLLOWUP. Mirrors `cli_xpub_search_path_of_xpub::path_of_xpub_phrase_zpub_match_bip84`
/// pattern (assert `v["schema_version"] == "1"`).
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
    assert_eq!(v["schema_version"], "1", "ms1 envelope schema_version");
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
    assert_eq!(v["schema_version"], "1", "mk1 envelope schema_version");
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
    assert_eq!(v["schema_version"], "1", "md1 envelope schema_version");
    assert_eq!(v["kind"], "md1");
}
