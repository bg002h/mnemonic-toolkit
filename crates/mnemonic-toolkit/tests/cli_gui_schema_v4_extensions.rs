//! `mnemonic gui-schema` SPEC §6.10 v3-cycle extensions (schema v4).
//!
//! Pins the v0.18.0 GUI conditional-applicability v3 cycle surfaces:
//!
//! - §6.10.3 + §6.10.4 — `disable_options` Visibility variant with
//!   schema-time-only semantic (no argv-emission impact). The bundle
//!   subcommand emits two new rules: row 10 (`slot_count_gte: 2` →
//!   disable single-sig templates) + row 11 (`slot_count_eq: 1` →
//!   disable multisig templates). Closes the v2-deferred §6.10.7 rows
//!   9/10/11 partition (the dropdown-option-disable Effect vocabulary
//!   gap previously tracked at FOLLOWUP
//!   `gui-schema-effect-on-dropdown-options-vocab`). Row 9 closes
//!   GUI-side via `NumberMax::FromSlotCount` — no toolkit wire change.
//!
//! v1/v2/v3 schema-version assertions (and the pre-v4 rule shapes /
//! counts) live at `cli_gui_schema_conditional_rules.rs` +
//! `cli_gui_schema_v3_extensions.rs`. This file covers ONLY the
//! v3-cycle additions.

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

// ── §6.10.3 disable_options Effect — wire-format ───────────────────────────

#[test]
fn bundle_emits_disable_options_rule_row_10_when_slot_count_gte_2() {
    // SPEC §6.6 row 10: single-sig --template with N > 1 slots is invalid;
    // GUI projection disables single-sig template options when slot_count >= 2.
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    let rules = bundle["conditional_rules"].as_array().unwrap();
    let row_10 = rules
        .iter()
        .find(|r| {
            r["effect"]["flag"] == "--template"
                && r["when"]["kind"] == "slot_count_gte"
                && r["when"]["value"] == 2
        })
        .expect(
            "bundle v0.18.0 must emit a rule with predicate slot_count_gte: 2 \
             whose effect targets --template (SPEC §6.10.7 row 10 — \
             SINGLE_SIG_TEMPLATE_WITH_MULTISIG_SLOTS)",
        );
    // Effect: disable_options with values list. Wire shape:
    //   "visibility": {"disable_options": {"values": [...]}}
    let visibility = &row_10["effect"]["visibility"];
    assert!(
        visibility.is_object(),
        "disable_options visibility MUST be a tagged-object (not a bare \
         string) per SPEC §6.10.3; got: {visibility:?}"
    );
    let disable = &visibility["disable_options"];
    assert!(
        disable.is_object(),
        "visibility.disable_options MUST be an object (not a bare array) \
         per SPEC §6.10.3 inner-key convention; got: {disable:?}"
    );
    let values: Vec<&str> = disable["values"]
        .as_array()
        .expect("disable_options.values must be an array")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    // Row 10 disables single-sig templates when slot_count >= 2.
    // Source-of-truth: CliTemplate::is_multisig()==false.
    let expected: std::collections::BTreeSet<&str> =
        ["bip44", "bip49", "bip84", "bip86"].iter().copied().collect();
    let actual: std::collections::BTreeSet<&str> = values.iter().copied().collect();
    assert_eq!(
        actual, expected,
        "row 10 must disable the single-sig template set"
    );
}

#[test]
fn bundle_emits_disable_options_rule_row_11_when_slot_count_eq_1() {
    // SPEC §6.6 row 11: multisig --template with N == 1 slot is invalid;
    // GUI projection disables multisig template options when slot_count == 1.
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    let rules = bundle["conditional_rules"].as_array().unwrap();
    let row_11 = rules
        .iter()
        .find(|r| {
            r["effect"]["flag"] == "--template"
                && r["when"]["kind"] == "slot_count_eq"
                && r["when"]["value"] == 1
        })
        .expect(
            "bundle v0.18.0 must emit a rule with predicate slot_count_eq: 1 \
             whose effect targets --template (SPEC §6.10.7 row 11 — \
             MULTISIG_TEMPLATE_WITH_SINGLE_SLOT)",
        );
    let visibility = &row_11["effect"]["visibility"];
    let disable = &visibility["disable_options"];
    assert!(
        disable.is_object(),
        "row 11 visibility.disable_options MUST be an object; got: {disable:?}"
    );
    let values: Vec<&str> = disable["values"]
        .as_array()
        .expect("disable_options.values must be an array")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    // Row 11 disables multisig templates when slot_count == 1.
    // Source-of-truth: CliTemplate::is_multisig()==true.
    let expected: std::collections::BTreeSet<&str> = [
        "sh-wsh-multi",
        "sh-wsh-sortedmulti",
        "wsh-multi",
        "wsh-sortedmulti",
        "tr-multi-a",
        "tr-sortedmulti-a",
    ]
    .iter()
    .copied()
    .collect();
    let actual: std::collections::BTreeSet<&str> = values.iter().copied().collect();
    assert_eq!(
        actual, expected,
        "row 11 must disable the multisig template set"
    );
}

#[test]
fn disable_options_wire_shape_uses_inner_values_key() {
    // Pin the {"disable_options": {"values": [...]}} wire form (inner-key,
    // not bare-array). Future-readers + parity with v3 pin_value precedent
    // {"pin_value": {"value": V}}. Catches the regression where someone
    // "simplifies" the wire shape to a bare-array {"disable_options": [...]}.
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    let rules = bundle["conditional_rules"].as_array().unwrap();
    let any_disable_options = rules
        .iter()
        .find(|r| {
            r["effect"]["visibility"].is_object()
                && r["effect"]["visibility"]["disable_options"].is_object()
        })
        .expect("bundle v0.18.0 must contain at least one disable_options rule");
    let payload = &any_disable_options["effect"]["visibility"]["disable_options"];
    assert!(
        payload["values"].is_array(),
        "disable_options payload MUST have an inner `values` array per the \
         SPEC §6.10.3 inner-key convention (mirrors pin_value's inner \
         `value` key). Got: {payload:?}"
    );
    // Reject the bare-array shape: visibility.disable_options must NOT be
    // an array directly.
    assert!(
        !any_disable_options["effect"]["visibility"]["disable_options"].is_array(),
        "disable_options MUST NOT use the bare-array shape \
         {{\"disable_options\": [...]}}; SPEC §6.10.3 requires the \
         inner-key shape {{\"disable_options\": {{\"values\": [...]}}}}"
    );
}

#[test]
fn bare_string_visibility_round_trips_on_v4_doc() {
    // Wire back-compat: pre-existing rules whose effect uses
    // hidden/disabled/required keep the bare-string wire shape on v4
    // documents (no spurious tagged-object wrap).
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    let rules = bundle["conditional_rules"].as_array().unwrap();
    let any_disabled = rules
        .iter()
        .find(|r| r["effect"]["visibility"] == "disabled")
        .expect("bundle must have at least one disabled-Visibility rule");
    assert!(
        any_disabled["effect"]["visibility"].is_string(),
        "v2-shape disabled Visibility MUST remain a bare string on v4 docs \
         (SPEC §6.10.6 back-compat guarantee); got: {:?}",
        any_disabled["effect"]["visibility"]
    );
}

#[test]
fn pin_value_visibility_round_trips_on_v4_doc() {
    // Wire back-compat: v3 pin_value rules keep the tagged-object wire
    // shape {"pin_value": {"value": V}} unchanged on v4 documents.
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    let rules = bundle["conditional_rules"].as_array().unwrap();
    let pin_value_rule = rules
        .iter()
        .find(|r| {
            r["effect"]["visibility"].is_object()
                && r["effect"]["visibility"]["pin_value"].is_object()
        })
        .expect(
            "bundle v0.18.0 must still contain the v3 pin_value rule (row 12 \
             DESCRIPTOR_WITH_NONZERO_ACCOUNT — no v4 regression)",
        );
    let pin = &pin_value_rule["effect"]["visibility"]["pin_value"];
    assert_eq!(
        pin["value"], 0,
        "v3 pin_value rule must continue to pin --account to 0 on v4 docs"
    );
}

// ── §6.10.6 v4 schema-version regression guard ──────────────────────────────

#[test]
fn v4_schema_includes_all_v3_cycle_surfaces() {
    // Smoke test that the v3-cycle features ship together. Catches the case
    // where someone bumps version 3→4 but forgets to add the new
    // disable_options emissions.
    let v = run_gui_schema();
    assert_eq!(v["version"], 4, "schema version must be v4");
    let bundle = find_sub(&v, "bundle");
    let rules = bundle["conditional_rules"].as_array().unwrap();
    let has_disable_options_rule = rules.iter().any(|r| {
        r["effect"]["visibility"].is_object()
            && r["effect"]["visibility"]["disable_options"].is_object()
    });
    assert!(
        has_disable_options_rule,
        "bundle must contain at least one disable_options rule (rows 10/11 \
         from SPEC §6.10.7 v3-cycle closing list)"
    );
}

#[test]
fn bundle_conditional_rules_count_is_thirteen_at_v0_18_0() {
    // Anti-regression on the rule count. A regression that drops a rule
    // would otherwise pass the v3-cycle's existing per-subcommand floor of
    // 11 silently — explicit equality assertion at v0.18.0 catches it.
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    let rules = bundle["conditional_rules"].as_array().unwrap();
    assert_eq!(
        rules.len(),
        13,
        "bundle v0.18.0 must emit exactly 13 conditional_rules \
         (v0.17.1 baseline 11 + 2 new disable_options rules for rows 10/11). \
         Got: {} rules",
        rules.len()
    );
}
