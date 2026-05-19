//! Phase 1 CLI integration tests for `mnemonic compare-cost`. SPEC §1-§5.
//! Test cells cover: smoke, timelocks, preimages, context-rewrite (multi↔
//! multi_a), capacity behavior, output formats (table + JSON), and §9 errors.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

// ── §1/§4 smoke: simple pk(A) end-to-end ────────────────────────────────────

#[test]
fn smoke_simple_pk() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "pk(A)"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Input: pk(A)"));
    assert!(stdout.contains("Wrapper comparison: wsh(M)"));
    assert!(stdout.contains("Condition"));
    assert!(stdout.contains("wsh vB"));
    assert!(stdout.contains("tr vB"));
    assert!(stdout.contains("Δ vB"));
    // Single condition expected (just A signing).
    let rows: Vec<&str> = stdout
        .lines()
        .filter(|l| l.starts_with("A "))
        .collect();
    assert_eq!(rows.len(), 1, "expected exactly 1 row labeled 'A'; got: {stdout}");
}

#[test]
fn smoke_and_v_pk_pk() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "and_v(v:pk(A),pk(B))"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    // A AND B → single row "A + B"
    assert!(stdout.contains("A + B"), "missing 'A + B' row in: {stdout}");
}

#[test]
fn smoke_or_b_pk_pk() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "or_b(pk(A),s:pk(B))"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    // or_b → 2 rows: "A" and "B"
    let a_rows: Vec<&str> = stdout.lines().filter(|l| l.starts_with("A ") || l.trim().starts_with("A ")).collect();
    let b_rows: Vec<&str> = stdout.lines().filter(|l| l.starts_with("B ")).collect();
    assert!(!a_rows.is_empty(), "missing A row in or_b output: {stdout}");
    assert!(!b_rows.is_empty(), "missing B row in or_b output: {stdout}");
}

// ── §2.1 context-rewrite: multi ↔ multi_a ──────────────────────────────────

#[test]
fn context_rewrite_multi_to_multi_a() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "multi(2,A,B,C)"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    // 2-of-3: 3 minimal conditions (A+B, A+C, B+C). Row labels reflect that.
    let row_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains('+') && !l.contains("vB") && !l.starts_with("note") && !l.starts_with("--"))
        .collect();
    assert_eq!(row_lines.len(), 3, "expected 3 rows for thresh(2,A,B,C); got: {stdout}");
}

#[test]
fn context_rewrite_thresh() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "thresh(2,pk(A),s:pk(B),s:pk(C))"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    // thresh(2,3) → 3 minimal conditions (A+B, A+C, B+C)
    let row_lines: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains('+') && !l.contains("vB") && !l.starts_with("note") && !l.starts_with("--"))
        .collect();
    assert_eq!(row_lines.len(), 3, "expected 3 rows for thresh(2, pk(A), pk(B), pk(C)); got: {stdout}");
}

// ── §4/§5 output: --feerate, --json ─────────────────────────────────────────

#[test]
fn feerate_scales_sats_columns() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "pk(A)", "--feerate", "25.0"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Feerate: 25.0 sat/vB"));
    // Find the data row (line starting with `A ` followed by whitespace).
    let row = stdout
        .lines()
        .find(|l| l.starts_with("A "))
        .expect("data row");
    // Last field is Δ sats; first numeric field after the label is wsh vB.
    let nums: Vec<i64> = row
        .split('|')
        .skip(1)
        .map(|s| s.trim().trim_start_matches('+').parse::<i64>().unwrap_or(i64::MAX))
        .collect();
    let wsh_vb = nums[0];
    let wsh_sats = nums[3];
    assert_eq!(wsh_sats, wsh_vb * 25, "sats = vbytes × 25; row: {row}");
}

#[test]
fn feerate_decimal_accepted() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "pk(A)", "--feerate", "1.5"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Feerate: 1.5 sat/vB"));
}

#[test]
fn json_envelope_shape() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "or_b(pk(A),s:pk(B))", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["subcommand"], "compare-cost");
    assert_eq!(v["input"]["form"], "miniscript");
    assert_eq!(v["input"]["value"], "or_b(pk(A),s:pk(B))");
    assert_eq!(v["feerate_sat_per_vb"], 1.0);
    let conds = v["conditions"].as_array().expect("conditions array");
    assert_eq!(conds.len(), 2, "or_b → 2 conditions");
    for c in conds {
        assert!(c["label"].is_string());
        assert!(c["wsh_vbytes"].is_number());
        assert!(c["tr_vbytes"].is_number());
        assert!(c["delta_vbytes"].is_number());
        assert!(c["wsh_sats"].is_number());
        assert!(c["tr_sats"].is_number());
        assert!(c["delta_sats"].is_number());
    }
    assert!(v["notes"].is_array(), "notes array present");
}

// ── §4 cost: feerate=0 advisory ─────────────────────────────────────────────

#[test]
fn feerate_zero_emits_advisory() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "pk(A)", "--feerate", "0.0", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let notes: Vec<&str> = v["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap_or(""))
        .collect();
    assert!(
        notes.iter().any(|n| n.contains("feerate is 0")),
        "missing feerate=0 advisory in notes: {notes:?}"
    );
    let cond = &v["conditions"][0];
    assert_eq!(cond["wsh_sats"], 0);
    assert_eq!(cond["tr_sats"], 0);
}

// ── §4: delta vB sign convention (tr − wsh) ────────────────────────────────

#[test]
fn delta_sign_convention_pk() {
    // For a simple pk(A), tr's Schnorr (64B) + control block (33B) is
    // marginally bigger than wsh's ECDSA satisfaction → Δ should be positive
    // (tr more expensive than wsh).
    let out = bin()
        .args(["compare-cost", "--miniscript", "pk(A)", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let cond = &v["conditions"][0];
    let wsh_vb = cond["wsh_vbytes"].as_i64().unwrap();
    let tr_vb = cond["tr_vbytes"].as_i64().unwrap();
    let delta = cond["delta_vbytes"].as_i64().unwrap();
    assert_eq!(delta, tr_vb - wsh_vb, "Δ = tr − wsh");
    assert!(tr_vb > 0 && wsh_vb > 0);
}

// ── §4: vbyte rounding-drift advisory always present ───────────────────────

#[test]
fn vbyte_rounding_advisory_always_present() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "pk(A)", "--json"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let notes: Vec<&str> = v["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap_or(""))
        .collect();
    assert!(
        notes.iter().any(|n| n.contains("rounded individually")),
        "missing rounding-drift advisory: {notes:?}"
    );
}

// ── §9 errors: parse failure → exit 2 ──────────────────────────────────────

#[test]
fn parse_error_exit_2() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "not_a_miniscript_at_all!"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("parse error") || stderr.contains("compare-cost:"));
}

// ── §9 errors: missing required input → clap exit 64 ───────────────────────

// (`missing_miniscript_clap_error` removed in Phase 2: --miniscript is now
// optional since --descriptor competes for the same slot. The
// `no_input_flag_clap_error` cell at line ~180 covers the no-input case.)

// ── §9 errors: feerate out of range → clap exit 64 ─────────────────────────

#[test]
fn feerate_negative_clap_error() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "pk(A)", "--feerate", "-1.0"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64), "stderr: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn feerate_too_large_clap_error() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "pk(A)", "--feerate", "1000000"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64), "stderr: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn feerate_at_upper_bound_accepted() {
    // SPEC §1: --feerate value_parser upper bound at 10_000.0. Pins
    // boundary acceptance (inclusive) so future bounds-tightening doesn't
    // silently move the gate.
    let out = bin()
        .args(["compare-cost", "--miniscript", "pk(A)", "--feerate", "10000.0"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn feerate_just_above_upper_bound_rejected() {
    // Mirror of the boundary cell: 10000.0 + ULP rejected.
    let out = bin()
        .args(["compare-cost", "--miniscript", "pk(A)", "--feerate", "10000.1"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64), "stderr: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn descriptor_plaintext_shows_both_input_and_extracted() {
    // Regression: for --descriptor input where the wrapper differs from
    // the inner M (e.g. wsh(pk(K))), the plaintext header MUST surface
    // both the original (`Input:`) and the stripped (`Extracted:`)
    // forms so users can see what wrapper they typed. (Pre-fold the
    // wrapper was silently dropped from plaintext output while JSON
    // showed both as `input.value` + `extracted_miniscript`.)
    let desc = format!("wsh(pk({KEY_A}))");
    let out = bin()
        .args(["compare-cost", "--descriptor", &desc])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&format!("Input:     wsh(pk({KEY_A}))")),
        "missing Input: <descriptor> line; got: {stdout}"
    );
    assert!(
        stdout.contains(&format!("Extracted: pk({KEY_A})")),
        "missing Extracted: <inner-M> line; got: {stdout}"
    );
}

#[test]
fn or_b_rejects_non_minimal_both_keys_row() {
    // SPEC §3.3 step 5 minimality: a configuration is minimal iff dropping
    // any single asset breaks `plan()`. For `or_b(pk(A), s:pk(B))`, the
    // configuration `{A, B}` is NOT minimal (either alone satisfies, so
    // dropping A still plans via B and vice versa). Direct positive +
    // negative assertion on the row set; complements smoke_or_b_pk_pk
    // which only positive-asserts the A and B rows exist.
    let out = bin()
        .args(["compare-cost", "--miniscript", "or_b(pk(A),s:pk(B))", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let labels: Vec<String> = v["conditions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["label"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(labels.len(), 2, "or_b minimality expects exactly 2 rows; got: {labels:?}");
    assert!(labels.iter().any(|l| l == "A"), "missing A row: {labels:?}");
    assert!(labels.iter().any(|l| l == "B"), "missing B row: {labels:?}");
    // Negative: no joint-asset row labeled like "A + B" should appear.
    assert!(
        !labels.iter().any(|l| l.contains('+')),
        "is_minimal failure: non-minimal joint-asset row leaked: {labels:?}"
    );
}

#[test]
#[ignore = "slow: enumerates ~462 minimal configs to hit soft-cap threshold"]
fn soft_cap_advisory_fires_when_rows_exceed_threshold() {
    // SPEC §3.3 step 7: soft warn-trail advisory fires when
    // rows.len() >= min(SOFT_THRESHOLD=256, --max-conditions).
    // C(11, 6) = 462 minimal configs > 256. Set --max-conditions=10000
    // to bypass the eager precheck (4 * 2^11 = 8192 < 10000).
    // Ignored by default (462 plan() calls × 2 sides takes a few seconds);
    // opt in via `cargo test -- --include-ignored soft_cap`.
    let policy = "thresh(6, pk(A),pk(B),pk(C),pk(D),pk(E),pk(F),pk(G),pk(H),pk(I),pk(J),pk(K))";
    let out = bin()
        .args([
            "compare-cost",
            "--miniscript",
            policy,
            "--max-conditions",
            "10000",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let notes: Vec<String> = v["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap().to_string())
        .collect();
    assert!(
        notes.iter().any(|n| n.contains("soft threshold") || n.contains("256")),
        "expected soft-cap advisory in notes; got: {notes:?}"
    );
}

#[test]
fn max_conditions_zero_clap_error() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "pk(A)", "--max-conditions", "0"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64), "stderr: {}", String::from_utf8_lossy(&out.stderr));
}

// ── §3.3 step 1: hard cap precheck → exit 3 ───────────────────────────────

#[test]
fn hard_cap_too_small_for_n_keys_exit_3() {
    // 4-key thresh, no timelocks/preimages: n_abs × n_rel = 1 × 1 = 1, so
    // raw = 1 × 2^4 = 16; set --max-conditions=8 to force precheck failure.
    let out = bin()
        .args([
            "compare-cost",
            "--miniscript",
            "thresh(2,pk(A),s:pk(B),s:pk(C),s:pk(D))",
            "--max-conditions",
            "8",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("exceed --max-conditions"),
        "wrong error text: {stderr}"
    );
}

// ── §3 timelock-bearing fragments enumerate properly ──────────────────────

#[test]
fn timelock_and_v_produces_one_row_with_older_label() {
    let out = bin()
        .args(["compare-cost", "--miniscript", "and_v(v:pk(A),older(144))"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("A + older(blocks)"),
        "expected label 'A + older(blocks)' in: {stdout}"
    );
}

#[test]
fn timelock_or_d_produces_two_rows_one_unlocked_one_timelocked() {
    // SPEC §5 hero example.
    let out = bin()
        .args([
            "compare-cost",
            "--miniscript",
            "or_d(pk(A),and_v(v:pk(B),older(144)))",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Two rows expected: "A" (timelock unneeded) and "B + older(blocks)".
    assert!(stdout.contains("A "), "missing 'A' row: {stdout}");
    assert!(
        stdout.contains("B + older(blocks)"),
        "missing 'B + older(blocks)' row: {stdout}"
    );
}

// ── §3 user-labeled rows (C1 fold from R0 review) ───────────────────────────

#[test]
fn user_labels_preserved_in_output_z_before_a() {
    // Use labels Z and A — input-order is Z then A; rows should reflect user
    // labels, not AST-traversal-order indices A/B.
    let out = bin()
        .args([
            "compare-cost",
            "--miniscript",
            "or_b(pk(Z),s:pk(A))",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Both Z and A should appear as standalone single-signer rows.
    assert!(stdout.contains("Z "), "user label 'Z' missing: {stdout}");
    assert!(stdout.contains("A "), "user label 'A' missing: {stdout}");
}

// ── Phase 2: --descriptor wrapper stripping (SPEC §2) ──────────────────────

const KEY_A: &str = "02998512205ec6a5cdb77d5b4f7de63c560d1e846162612ee178c49e7b6cc44fb9";
const KEY_B: &str = "03999999999999999999999999999999999999999999999999999999999999999d";

#[test]
fn descriptor_wsh_strips_wrapper() {
    let desc = format!("wsh(pk({KEY_A}))");
    let out = bin()
        .args(["compare-cost", "--descriptor", &desc, "--json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["input"]["form"], "descriptor");
    let conds = v["conditions"].as_array().unwrap();
    assert_eq!(conds.len(), 1);
    let notes: Vec<&str> = v["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap_or(""))
        .collect();
    assert!(
        notes.iter().any(|n| n.contains("concrete keys")),
        "missing concrete-keys advisory: {notes:?}"
    );
}

#[test]
fn descriptor_sh_wsh_strips_both_wrappers() {
    let desc = format!("sh(wsh(pk({KEY_A})))");
    let out = bin()
        .args(["compare-cost", "--descriptor", &desc])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("key[0]"), "expected single row in: {stdout}");
}

#[test]
fn descriptor_wsh_or_b_produces_two_rows() {
    let desc = format!("wsh(or_b(pk({KEY_A}),s:pk({KEY_B})))");
    let out = bin()
        .args(["compare-cost", "--descriptor", &desc, "--json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["conditions"].as_array().unwrap().len(), 2);
}

#[test]
fn descriptor_tr_refused_exit_3() {
    let out = bin()
        .args(["compare-cost", "--descriptor", "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0)"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stderr = String::from_utf8_lossy(&out.stdout) + String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("tr-input") || stderr.contains("FOLLOWUP"),
        "expected tr-input FOLLOWUP message: {stderr}"
    );
}

#[test]
fn descriptor_wsh_wildcard_xpub_materializes_at_index_zero() {
    // Wildcard xpub descriptor — exercises the strip.rs `derive_at_index(0)`
    // branch that's distinct from the bare-pubkey TryFrom branch.
    let desc = "wsh(pk([d34db33f/48h/0h/0h/2h]xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8/0/*))";
    let out = bin()
        .args(["compare-cost", "--descriptor", desc, "--json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let conds = v["conditions"].as_array().unwrap();
    assert_eq!(conds.len(), 1, "single-pk wildcard wsh has one minimal row");
}

#[test]
fn descriptor_wsh_with_timelock_descriptor_input_enumerates_paths() {
    // Verify descriptor-input path correctly feeds the timelock enumeration:
    // or_d should produce 2 rows (key signs alone OR key2+older).
    let desc = format!("wsh(or_d(pk({KEY_A}),and_v(v:pk({KEY_B}),older(144))))");
    let out = bin()
        .args(["compare-cost", "--descriptor", &desc, "--json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let conds = v["conditions"].as_array().unwrap();
    assert_eq!(conds.len(), 2, "or_d with timelock → 2 minimal rows");
    let labels: Vec<&str> = conds.iter().map(|c| c["label"].as_str().unwrap()).collect();
    assert!(
        labels.iter().any(|l| l.contains("older(blocks)")),
        "expected an older(blocks) row: {labels:?}"
    );
}

#[test]
fn descriptor_pkh_refused_exit_3() {
    let desc = format!("pkh({KEY_A})");
    let out = bin()
        .args(["compare-cost", "--descriptor", &desc])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3), "stderr: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn descriptor_and_miniscript_conflict_exit_64() {
    let desc = format!("wsh(pk({KEY_A}))");
    let out = bin()
        .args(["compare-cost", "--descriptor", &desc, "--miniscript", "pk(A)"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64), "stderr: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn no_input_flag_clap_error() {
    let out = bin().args(["compare-cost"]).output().unwrap();
    // Now that --miniscript is optional, this hits our explicit "supply one"
    // error → exit 2 (BadInput) under the toolkit's mapping.
    let code = out.status.code().unwrap_or(-1);
    assert!(code == 1 || code == 2 || code == 64, "expected exit 1/2/64 for missing input; got {code}: {}", String::from_utf8_lossy(&out.stderr));
}

// ── §3 preimage enumeration ────────────────────────────────────────────────

#[test]
fn sha256_preimage_required_emits_preimage_row_and_advisory() {
    // and_v(v:pk(A), sha256(H)) — both A and a preimage of H are required.
    let hash_hex = "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9";
    let ms = format!("and_v(v:pk(A),sha256({hash_hex}))");
    let out = bin()
        .args(["compare-cost", "--miniscript", &ms, "--json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let conds = v["conditions"].as_array().unwrap();
    assert_eq!(conds.len(), 1, "exactly one minimal row: A + preimage");
    let label = conds[0]["label"].as_str().unwrap();
    assert!(label.contains('A'), "label must include A: got {label}");
    assert!(label.contains("preimage"), "label must include preimage: got {label}");
    let notes: Vec<&str> = v["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap_or(""))
        .collect();
    assert!(
        notes.iter().any(|n| n.contains("hash-preimage fragments")),
        "missing hash-preimage advisory: {notes:?}"
    );
}

// ── §3 per-kind timelock saturation (R1 C1 fold) ────────────────────────────

#[test]
fn absolute_mtp_time_lock_satisfies() {
    // after(N >= 500_000_000) is an MTP-time lock and requires the
    // time-kind saturation in Assets — height-kind cannot satisfy.
    let out = bin()
        .args([
            "compare-cost",
            "--miniscript",
            "and_v(v:pk(A),after(1500000000))",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("after(time)"),
        "expected after(time) label for MTP-time abs-lock: {stdout}"
    );
}

#[test]
fn relative_512s_time_lock_satisfies() {
    // older(N | TIME_LOCK_FLAG) — N=1 | 0x00400000 = 4194305 = ~512 seconds.
    let out = bin()
        .args([
            "compare-cost",
            "--miniscript",
            "and_v(v:pk(A),older(4194305))",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("older(512s)"),
        "expected older(512s) label for 512s-rel-lock: {stdout}"
    );
}

// ── Phase 3: stdin fallback (SPEC §2 row "stdin") ──────────────────────────

#[test]
fn stdin_classifies_miniscript_input() {
    let out = bin()
        .args(["compare-cost"])
        .write_stdin("pk(A)\n")
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Input: pk(A)"));
}

#[test]
fn stdin_classifies_descriptor_input() {
    let desc = format!("wsh(pk({KEY_A}))");
    let out = bin()
        .args(["compare-cost", "--json"])
        .write_stdin(format!("{desc}\n"))
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["input"]["form"], "descriptor");
}

#[test]
fn stdin_malformed_input_exits_parse_error() {
    let out = bin()
        .args(["compare-cost"])
        .write_stdin("not_a_thing!!\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "stderr: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn flag_wins_over_stdin() {
    // When --miniscript is supplied, stdin is ignored.
    let out = bin()
        .args(["compare-cost", "--miniscript", "pk(A)"])
        .write_stdin("THIS_SHOULD_BE_IGNORED\n")
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Input: pk(A)"));
}

#[test]
fn stdin_empty_input_exits_bad_input() {
    let out = bin()
        .args(["compare-cost"])
        .write_stdin("")
        .output()
        .unwrap();
    let code = out.status.code().unwrap_or(-1);
    assert!(
        code == 1 || code == 2,
        "expected exit 1 (BadInput) or 2 (parse); got {code}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn stdin_uses_first_nonblank_line() {
    let out = bin()
        .args(["compare-cost"])
        .write_stdin("\n\n  \npk(A)\nignored_second_line\n")
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Input: pk(A)"));
}

// ── §1 --help renders ──────────────────────────────────────────────────────

#[test]
fn help_renders() {
    bin()
        .args(["compare-cost", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--miniscript"))
        .stdout(predicate::str::contains("--feerate"))
        .stdout(predicate::str::contains("--max-conditions"))
        .stdout(predicate::str::contains("--json"));
}
