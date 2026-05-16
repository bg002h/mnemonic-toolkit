//! `mnemonic gui-schema` SPEC §6.10 conditional-applicability projection tests.
//!
//! v0.16.0 GUI conditional-applicability v1 cycle pinned per-subcommand
//! `conditional_rules` array shape, the Predicate AST tagged union per
//! §6.10.2, the Effect grammar per §6.10.3, the first-rule-wins emission
//! order per §6.10.4, and the v2 schema version bump per §6.10.6.
//!
//! v0.17.0 GUI conditional-applicability v2 cycle extends this with three
//! new Predicate kinds (slot_count_eq / slot_count_gte / slot_count_lte —
//! §6.10.2), one new Visibility variant (pin_value — §6.10.3 + §6.10.4
//! emission table), per-subcommand `meta.template_groups` (§6.10.8 — NEW),
//! and a schema-version bump 2 → 3 (§6.10.6). v3-specific surfaces have
//! their own test file at `cli_gui_schema_v3_extensions.rs`; the
//! assertions here update for the v3 version bump + the new row 12
//! (`DESCRIPTOR_WITH_NONZERO_ACCOUNT`) rule emission in `bundle`.

use assert_cmd::Command;
use serde_json::Value;

fn run_gui_schema() -> Value {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("gui-schema")
        .output()
        .expect("gui-schema exec failed");
    assert!(out.status.success(), "gui-schema must exit 0; got {out:?}");
    let stdout = String::from_utf8(out.stdout).expect("gui-schema stdout must be UTF-8");
    serde_json::from_str(&stdout).expect("gui-schema stdout must parse as JSON")
}

fn find_sub<'a>(v: &'a Value, name: &str) -> &'a Value {
    v["subcommands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == name)
        .unwrap_or_else(|| panic!("subcommand `{name}` not in schema"))
}

fn conditional_rules<'a>(v: &'a Value, sub_name: &str) -> &'a Vec<Value> {
    find_sub(v, sub_name)["conditional_rules"]
        .as_array()
        .unwrap_or_else(|| panic!("subcommand `{sub_name}` missing conditional_rules array"))
}

// ── §6.10.6 schema version bump ────────────────────────────────────────

#[test]
fn schema_version_is_three_after_v0_17_0_bump() {
    let v = run_gui_schema();
    assert_eq!(
        v["version"], 3,
        "SPEC §6.10.6: gui-schema JSON version bumps 2→3 in v0.17.0 \
         (v2 cycle — slot_count_* predicates + pin_value Visibility + \
         meta.template_groups)"
    );
}

// ── §6.10 conditional_rules field presence ─────────────────────────────

#[test]
fn every_subcommand_has_conditional_rules_array() {
    let v = run_gui_schema();
    for sub in v["subcommands"].as_array().unwrap() {
        let cr = &sub["conditional_rules"];
        assert!(
            cr.is_array(),
            "subcommand {} must have conditional_rules array (may be empty)",
            sub["name"]
        );
    }
}

// ── §6.10.7 bundle rules ───────────────────────────────────────────────

#[test]
fn bundle_emits_conditional_rules() {
    let v = run_gui_schema();
    let rules = conditional_rules(&v, "bundle");
    // v0.16.0: Bundle has 10 rules: template required-unless,
    // descriptor↔descriptor-file mutex (2 dir), passphrase↔passphrase-stdin
    // mutex (2 dir), template disabled when descriptor,
    // threshold/multisig-path-family disabled when descriptor (2),
    // threshold/multisig-path-family disabled when single-sig template (2).
    //
    // v0.17.0 (v2 cycle): adds 1 new rule —
    // `DESCRIPTOR_WITH_NONZERO_ACCOUNT` pins `--account` to 0 when
    // `--descriptor` is present (uses the new pin_value Effect per §6.10.3).
    // Detailed pin_value assertions live in cli_gui_schema_v3_extensions.rs.
    assert_eq!(rules.len(), 11, "bundle v0.17.0 rule count (v0.16.0 + 1 pin_value rule)");
}

#[test]
fn bundle_threshold_priority_descriptor_before_single_sig() {
    // §6.10.4 first-rule-wins requires priority-descending emission order
    // per target flag. For --threshold, the descriptor-present rule
    // (DESCRIPTOR_WITH_THRESHOLD) must precede the single-sig-template rule
    // (THRESHOLD_WITHOUT_MULTISIG); descriptor-mode is the more-specific
    // predicate.
    let v = run_gui_schema();
    let rules = conditional_rules(&v, "bundle");

    let threshold_rule_indices: Vec<usize> = rules
        .iter()
        .enumerate()
        .filter(|(_, r)| r["effect"]["flag"] == "--threshold")
        .map(|(i, _)| i)
        .collect();
    assert_eq!(threshold_rule_indices.len(), 2, "expected 2 threshold rules");

    let first_rule = &rules[threshold_rule_indices[0]];
    let second_rule = &rules[threshold_rule_indices[1]];
    // First (priority) rule's predicate must reference --descriptor.
    assert_eq!(
        first_rule["when"]["kind"], "flag_present",
        "bundle --threshold priority-1 rule must be flag_present predicate"
    );
    assert_eq!(
        first_rule["when"]["flag"], "--descriptor",
        "bundle --threshold priority-1 rule must check --descriptor presence"
    );
    // Second rule's predicate is dropdown_value_in --template.
    assert_eq!(
        second_rule["when"]["kind"], "dropdown_value_in",
        "bundle --threshold priority-2 rule must be dropdown_value_in"
    );
    assert_eq!(second_rule["when"]["flag"], "--template");
}

#[test]
fn bundle_multisig_path_family_priority_descriptor_before_single_sig() {
    let v = run_gui_schema();
    let rules = conditional_rules(&v, "bundle");
    let mpf_rules: Vec<&Value> = rules
        .iter()
        .filter(|r| r["effect"]["flag"] == "--multisig-path-family")
        .collect();
    assert_eq!(mpf_rules.len(), 2);
    assert_eq!(mpf_rules[0]["when"]["kind"], "flag_present");
    assert_eq!(mpf_rules[0]["when"]["flag"], "--descriptor");
    assert_eq!(mpf_rules[1]["when"]["kind"], "dropdown_value_in");
}

#[test]
fn bundle_template_required_unless_uses_not_any_of_predicate() {
    let v = run_gui_schema();
    let rules = conditional_rules(&v, "bundle");
    let template_required = rules
        .iter()
        .find(|r| {
            r["effect"]["flag"] == "--template"
                && r["effect"]["visibility"] == "required"
        })
        .expect("bundle must have --template Required rule");
    // §6.10.2 Not predicate: {"kind": "not", "predicate": P}
    assert_eq!(template_required["when"]["kind"], "not");
    let inner = &template_required["when"]["predicate"];
    assert_eq!(inner["kind"], "any_of");
    let predicates = inner["predicates"].as_array().unwrap();
    assert_eq!(predicates.len(), 2);
    let flags: Vec<&str> = predicates
        .iter()
        .map(|p| p["flag"].as_str().unwrap())
        .collect();
    assert!(flags.contains(&"--descriptor"));
    assert!(flags.contains(&"--descriptor-file"));
}

#[test]
fn bundle_single_sig_dropdown_values_match_template_enum() {
    let v = run_gui_schema();
    let rules = conditional_rules(&v, "bundle");
    let threshold_single_sig = rules
        .iter()
        .find(|r| {
            r["effect"]["flag"] == "--threshold"
                && r["when"]["kind"] == "dropdown_value_in"
        })
        .expect("bundle --threshold single-sig rule");
    let values: Vec<&str> = threshold_single_sig["when"]["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    // Source-of-truth: CliTemplate::is_multisig() in crates/mnemonic-toolkit/
    // src/template.rs:46-56. v0.16.0 single-sig set: bip44/49/84/86.
    let expected = vec!["bip44", "bip49", "bip84", "bip86"];
    assert_eq!(values, expected, "single-sig template set");
}

// ── §6.10.7 verify-bundle rules ────────────────────────────────────────

#[test]
fn verify_bundle_emits_conditional_rules() {
    let v = run_gui_schema();
    let rules = conditional_rules(&v, "verify-bundle");
    // verify-bundle has 10 rules: template required-unless, descriptor↔
    // descriptor-file mutex (2 dir), bundle-json XOR (--ms1/--mk1/--md1)
    // (3 rules), passphrase mutex (2 dir), threshold disabled single-sig,
    // template disabled when descriptor.
    assert_eq!(
        rules.len(),
        10,
        "verify-bundle v0.16.0 rule count"
    );
}

#[test]
fn verify_bundle_bundle_json_xor_rules_target_ms1_mk1_md1() {
    let v = run_gui_schema();
    let rules = conditional_rules(&v, "verify-bundle");
    let bj_rules: Vec<&Value> = rules
        .iter()
        .filter(|r| {
            r["when"]["kind"] == "flag_present"
                && r["when"]["flag"] == "--bundle-json"
        })
        .collect();
    assert_eq!(bj_rules.len(), 3);
    let targets: Vec<&str> = bj_rules
        .iter()
        .map(|r| r["effect"]["flag"].as_str().unwrap())
        .collect();
    assert!(targets.contains(&"--ms1"));
    assert!(targets.contains(&"--mk1"));
    assert!(targets.contains(&"--md1"));
}

// ── §6.10.7 export-wallet rules ────────────────────────────────────────

#[test]
fn export_wallet_taproot_internal_key_required_and_disabled() {
    let v = run_gui_schema();
    let rules = conditional_rules(&v, "export-wallet");
    let tik_rules: Vec<&Value> = rules
        .iter()
        .filter(|r| r["effect"]["flag"] == "--taproot-internal-key")
        .collect();
    assert_eq!(
        tik_rules.len(),
        2,
        "--taproot-internal-key has both Required-on-match and Disabled-on-not"
    );
    // Required rule: template ∈ {tr-multi-a, tr-sortedmulti-a}
    let required = tik_rules
        .iter()
        .find(|r| r["effect"]["visibility"] == "required")
        .expect("Required rule");
    assert_eq!(required["when"]["kind"], "dropdown_value_in");
    let req_values: Vec<&str> = required["when"]["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(req_values, vec!["tr-multi-a", "tr-sortedmulti-a"]);
    // Disabled rule: NOT (template ∈ {tr-multi-a, tr-sortedmulti-a})
    let disabled = tik_rules
        .iter()
        .find(|r| r["effect"]["visibility"] == "disabled")
        .expect("Disabled rule");
    assert_eq!(disabled["when"]["kind"], "not");
    assert_eq!(disabled["when"]["predicate"]["kind"], "dropdown_value_in");
}

#[test]
fn export_wallet_threshold_disabled_when_single_sig() {
    let v = run_gui_schema();
    let rules = conditional_rules(&v, "export-wallet");
    let rule = rules
        .iter()
        .find(|r| {
            r["effect"]["flag"] == "--threshold"
                && r["when"]["kind"] == "dropdown_value_in"
        })
        .expect("export-wallet --threshold rule");
    assert_eq!(rule["effect"]["visibility"], "disabled");
}

// ── §6.10.7 derive-child rule ──────────────────────────────────────────

#[test]
fn derive_child_dice_sides_required_when_application_dice() {
    let v = run_gui_schema();
    let rules = conditional_rules(&v, "derive-child");
    let rule = rules
        .iter()
        .find(|r| r["effect"]["flag"] == "--dice-sides")
        .expect("derive-child --dice-sides Required rule");
    assert_eq!(rule["effect"]["visibility"], "required");
    assert_eq!(rule["when"]["kind"], "dropdown_value_in");
    assert_eq!(rule["when"]["flag"], "--application");
    let values: Vec<&str> = rule["when"]["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(values, vec!["dice"]);
}

// ── §6.10.7 convert rules ──────────────────────────────────────────────

#[test]
fn convert_has_two_passphrase_mutexes() {
    let v = run_gui_schema();
    let rules = conditional_rules(&v, "convert");
    // --passphrase ↔ --passphrase-stdin (2 dirs); --bip38-passphrase
    // ↔ --bip38-passphrase-stdin (2 dirs).
    assert_eq!(rules.len(), 4);
}

// ── §6.10.2 Predicate AST shape ────────────────────────────────────────

#[test]
fn predicate_kinds_emitted_in_snake_case() {
    let v = run_gui_schema();
    let allowed_kinds = [
        "flag_present",
        "dropdown_value_in",
        "composite_node_is",
        "positional_present",
        "all_of",
        "any_of",
        "not",
    ];
    let mut visited = 0_usize;
    for sub in v["subcommands"].as_array().unwrap() {
        for rule in sub["conditional_rules"].as_array().unwrap() {
            check_predicate_kinds(&rule["when"], &allowed_kinds);
            visited += 1;
        }
    }
    assert!(
        visited > 0,
        "test must traverse at least one rule; got {visited}"
    );
}

fn check_predicate_kinds(predicate: &Value, allowed: &[&str]) {
    let kind = predicate["kind"]
        .as_str()
        .expect("every predicate must have a `kind` string");
    assert!(
        allowed.contains(&kind),
        "predicate kind `{kind}` not in §6.10.2 vocabulary"
    );
    match kind {
        "all_of" | "any_of" => {
            for child in predicate["predicates"].as_array().unwrap() {
                check_predicate_kinds(child, allowed);
            }
        }
        "not" => check_predicate_kinds(&predicate["predicate"], allowed),
        _ => {}
    }
}

// ── §6.10.3 Effect shape ───────────────────────────────────────────────

#[test]
fn effect_visibilities_are_in_allowed_set() {
    let v = run_gui_schema();
    // §6.10.3 says Visible never appears as an Effect value. v1 cycle vocab:
    // bare-string Hidden/Disabled/Required. v2 cycle (v0.17.0 / schema v3)
    // adds the tagged-object pin_value variant. This assertion accepts both
    // wire shapes; pin_value's inner `value` payload is intentionally
    // permissive per §6.10.3 wire-format details (any JSON value), so we
    // only assert structural shape, not the value's type.
    let bare_allowed = ["hidden", "disabled", "required"];
    for sub in v["subcommands"].as_array().unwrap() {
        for rule in sub["conditional_rules"].as_array().unwrap() {
            let vis = &rule["effect"]["visibility"];
            if let Some(s) = vis.as_str() {
                assert!(
                    bare_allowed.contains(&s),
                    "bare-string Visibility `{s}` not in §6.10.3 vocabulary"
                );
                assert_ne!(s, "visible", "Visible cannot be an Effect value");
            } else if let Some(obj) = vis.as_object() {
                // v3 tagged-object: must have exactly one known tag.
                let keys: Vec<&str> = obj.keys().map(|s| s.as_str()).collect();
                assert_eq!(
                    keys.len(),
                    1,
                    "tagged-object Visibility must have exactly one tag; got: {keys:?}"
                );
                assert!(
                    keys[0] == "pin_value",
                    "tagged-object Visibility tag `{}` not in §6.10.3 v3 vocabulary",
                    keys[0]
                );
                // pin_value payload must be an object with a `value` field.
                let pin = &obj["pin_value"];
                assert!(
                    pin.is_object() && pin.get("value").is_some(),
                    "pin_value payload must be {{\"value\": <JSON>}}; got: {pin:?}"
                );
            } else {
                panic!("Visibility must be string or object; got: {vis:?}");
            }
        }
    }
}

#[test]
fn every_rule_has_rationale_and_spec_ref() {
    let v = run_gui_schema();
    for sub in v["subcommands"].as_array().unwrap() {
        for rule in sub["conditional_rules"].as_array().unwrap() {
            let rationale = rule["rationale"]
                .as_str()
                .expect("every rule must carry a rationale string");
            assert!(
                !rationale.is_empty(),
                "rationale must be non-empty for failure-message clarity"
            );
            let spec_ref = rule["spec_ref"]
                .as_str()
                .expect("every rule must carry a spec_ref string");
            assert!(
                !spec_ref.is_empty(),
                "spec_ref must be non-empty"
            );
        }
    }
}
