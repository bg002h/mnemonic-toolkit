//! `mnemonic gui-schema` SPEC §6.10 v3-cycle extensions (schema v4).
//!
//! Pins the v0.18.0 v3-cycle GUI conditional-applicability surfaces
//! that REMAIN after the v0.18.1 row 10/11 rollback:
//!
//! - §6.10.3 + §6.10.4 — `disable_options` Visibility variant remains
//!   a valid v4 grammar variant. v0.18.0 introduced it with two rules
//!   (rows 10 + 11) that turned out to have a transient-state UX flaw
//!   (row 11 disabled multisig templates at slot_count==1, the natural
//!   intermediate state when building UP to multisig). v0.18.1 reverted
//!   both rules; the grammar variant + GUI consumer remain in place
//!   for future cycles. Row 10/11 closure migrated to GUI-internal
//!   warning banner (Option A pattern matching row 8) in mnemonic-gui
//!   v0.7.2; CLI rows 10/11 stay the authoritative gate per §6.6.
//!
//! - Schema version is v4 (set by v0.18.0; no roll-back to v3 because
//!   the disable_options grammar is still defined; v3 consumers would
//!   still fail-CLOSED on disable_options if any future rule emits it).
//!
//! Pre-v4 / v3 cycle surfaces (pin_value Effect + meta.template_groups)
//! live at `cli_gui_schema_v3_extensions.rs`; pre-v3 rule shapes /
//! counts live at `cli_gui_schema_conditional_rules.rs`.

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

// ── v0.18.1 — row 10/11 rollback: no disable_options rules emitted ──────────

#[test]
fn bundle_emits_no_disable_options_rules_after_v0_18_1_rollback() {
    // v0.18.1 reverted rows 10 + 11 (disable_options for single-sig /
    // multisig --template values). Multi-row template/slot_count
    // mismatch UX migrated to a GUI-internal warning banner (Option A
    // pattern; see mnemonic-gui v0.7.2 release notes). The grammar
    // variant remains defined; no rule emits it after the rollback.
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    let rules = bundle["conditional_rules"].as_array().unwrap();
    let count_disable_options = rules
        .iter()
        .filter(|r| {
            r["effect"]["visibility"].is_object()
                && r["effect"]["visibility"]["disable_options"].is_object()
        })
        .count();
    assert_eq!(
        count_disable_options, 0,
        "v0.18.1 rollback: bundle must emit ZERO disable_options rules; \
         row 10/11 UX migrated to GUI-internal warning banner. \
         Got {count_disable_options} rules."
    );
}

// ── Back-compat round-trips on v4 doc ───────────────────────────────────────

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
            "bundle v0.18.1 must still contain the v3 pin_value rule (row 12 \
             DESCRIPTOR_WITH_NONZERO_ACCOUNT — no regression)",
        );
    let pin = &pin_value_rule["effect"]["visibility"]["pin_value"];
    assert_eq!(
        pin["value"], 0,
        "v3 pin_value rule must continue to pin --account to 0 on v4 docs"
    );
}

// ── §6.10.6 v4 schema-version pin (updated to v5 in v0.24.0 Tranche B.1) ───

#[test]
fn schema_version_pinned_at_five_after_v0_24_0_tranche_b1() {
    // v0.24.0 Tranche B.1 bumped 4 → 5 for the additive Flag fields
    // {default_value, global, secret}. The v4 disable_options grammar
    // variant remains defined (just unused by any rule); v3/v4
    // consumers would still fail-CLOSED on those features if a future
    // rule reintroduces them. Pinning v5 here documents the current
    // schema-version pin; bump in lockstep with future schema cycles.
    let v = run_gui_schema();
    assert_eq!(
        v["version"], 5,
        "schema version pinned at v5 after v0.24.0 Tranche B.1 \
         (additive Flag fields: default_value, global, secret)"
    );
}

#[test]
fn bundle_conditional_rules_count_is_eleven_at_v0_18_1() {
    // Anti-regression on the rule count. v0.18.0 ramped to 13 (added
    // rows 10 + 11). v0.18.1 reverts to 11 (rolls back rows 10 + 11).
    // v0.17.1 baseline was also 11. Explicit equality assertion at
    // v0.18.1 catches re-introduction or accidental drift.
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    let rules = bundle["conditional_rules"].as_array().unwrap();
    assert_eq!(
        rules.len(),
        11,
        "bundle v0.18.1 must emit exactly 11 conditional_rules \
         (v0.17.1 baseline; v0.18.0's +2 disable_options rules reverted). \
         Got: {} rules",
        rules.len()
    );
}
