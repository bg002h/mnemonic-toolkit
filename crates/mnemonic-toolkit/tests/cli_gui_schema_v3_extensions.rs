//! `mnemonic gui-schema` SPEC §6.10 v2-cycle extensions (schema v3).
//!
//! Pins the v0.17.0 GUI conditional-applicability v2 cycle surfaces:
//!
//! - §6.10.3 + §6.10.4 — `pin_value` Visibility variant with REPLACE-value
//!   emission semantic. The bundle subcommand emits a new rule for
//!   `DESCRIPTOR_WITH_NONZERO_ACCOUNT` projecting `--account → pin_value(0)`
//!   when `--descriptor` is present (closes the v1-deferred §6.10.7 row).
//!
//! - §6.10.8 (NEW) — per-subcommand `meta.template_groups` block emitted for
//!   subcommands that consume `--template`. Block partitions `CliTemplate`
//!   variants into `single_sig` / `multisig` per
//!   `CliTemplate::is_multisig()` source-of-truth (`src/template.rs`).
//!   GUI consumer retires its hand-coded `SINGLE_SIG_TEMPLATES: &[&str]`
//!   const in favor of reading this block.
//!
//! v1 schema-version assertion (and the pre-v2 rule shapes / counts) lives
//! at `cli_gui_schema_conditional_rules.rs`. This file covers ONLY the
//! v2-cycle additions.

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

// ── §6.10.3 pin_value Effect — wire-format ─────────────────────────────────

#[test]
fn bundle_emits_account_pin_value_rule_when_descriptor_present() {
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    let rules = bundle["conditional_rules"].as_array().unwrap();
    let account_rule = rules
        .iter()
        .find(|r| r["effect"]["flag"] == "--account")
        .expect(
            "bundle v0.17.0 must emit a rule whose effect targets --account \
             (SPEC §6.10.7 row 12 — DESCRIPTOR_WITH_NONZERO_ACCOUNT)",
        );
    // Predicate: flag_present --descriptor.
    assert_eq!(
        account_rule["when"]["kind"], "flag_present",
        "row-12 predicate must be flag_present"
    );
    assert_eq!(
        account_rule["when"]["flag"], "--descriptor",
        "row-12 predicate must check --descriptor presence"
    );
    // Effect: pin_value with payload {value: 0}. Wire shape:
    //   "visibility": {"pin_value": {"value": 0}}
    let visibility = &account_rule["effect"]["visibility"];
    assert!(
        visibility.is_object(),
        "pin_value visibility MUST be a tagged-object (not a bare string) per \
         SPEC §6.10.3; got: {visibility:?}"
    );
    let pin = &visibility["pin_value"];
    assert!(
        pin.is_object(),
        "visibility.pin_value MUST be an object; got: {pin:?}"
    );
    assert_eq!(
        pin["value"], 0,
        "DESCRIPTOR_WITH_NONZERO_ACCOUNT projection must pin --account to 0; \
         the value coerces nonzero user input to 0 per §6.10.4 \
         emission table"
    );
}

#[test]
fn bare_string_visibility_round_trips_on_v3_doc() {
    // Wire back-compat: pre-existing rules whose effect uses
    // hidden/disabled/required keep the bare-string wire shape on v3
    // documents (no spurious tagged-object wrap).
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    let rules = bundle["conditional_rules"].as_array().unwrap();
    // Find any disabled-effect rule — there are many.
    let any_disabled = rules
        .iter()
        .find(|r| r["effect"]["visibility"] == "disabled")
        .expect("bundle must have at least one disabled-Visibility rule");
    assert!(
        any_disabled["effect"]["visibility"].is_string(),
        "v2-shape disabled Visibility MUST remain a bare string on v3 docs \
         (SPEC §6.10.6 back-compat guarantee); got: {:?}",
        any_disabled["effect"]["visibility"]
    );
}

// ── §6.10.8 meta.template_groups ────────────────────────────────────────────

fn meta_template_groups<'a>(v: &'a Value, sub_name: &str) -> &'a Value {
    let sub = find_sub(v, sub_name);
    &sub["meta"]["template_groups"]
}

#[test]
fn bundle_emits_meta_template_groups() {
    let v = run_gui_schema();
    let groups = meta_template_groups(&v, "bundle");
    assert!(
        groups.is_object(),
        "bundle.meta.template_groups MUST be an object per §6.10.8; got: \
         {groups:?}"
    );
    let single_sig: Vec<&str> = groups["single_sig"]
        .as_array()
        .expect("template_groups.single_sig must be an array")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let multisig: Vec<&str> = groups["multisig"]
        .as_array()
        .expect("template_groups.multisig must be an array")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    // Source-of-truth: CliTemplate::is_multisig() (template.rs:46-56).
    // single-sig set at v0.17.0:
    assert_eq!(
        single_sig,
        vec!["bip44", "bip49", "bip84", "bip86"],
        "single_sig template group must match !CliTemplate::is_multisig()"
    );
    // multisig set per !single-sig partition of CliTemplate::value_variants():
    // sh-wsh-multi, sh-wsh-sortedmulti, wsh-multi, wsh-sortedmulti,
    // tr-multi-a, tr-sortedmulti-a. (Ordering preserved from
    // value_variants() iteration order; we compare as sets to avoid pinning
    // declaration order, which is a brittle invariant.)
    let expected_multisig_set: std::collections::BTreeSet<&str> = [
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
    let actual_multisig_set: std::collections::BTreeSet<&str> =
        multisig.iter().copied().collect();
    assert_eq!(
        actual_multisig_set, expected_multisig_set,
        "multisig template group must match CliTemplate::is_multisig() set"
    );
}

#[test]
fn verify_bundle_emits_meta_template_groups() {
    // verify-bundle consumes --template; meta block must be present.
    let v = run_gui_schema();
    let groups = meta_template_groups(&v, "verify-bundle");
    assert!(
        groups.is_object(),
        "verify-bundle.meta.template_groups MUST be an object"
    );
}

#[test]
fn export_wallet_emits_meta_template_groups() {
    // export-wallet consumes --template; meta block must be present.
    let v = run_gui_schema();
    let groups = meta_template_groups(&v, "export-wallet");
    assert!(
        groups.is_object(),
        "export-wallet.meta.template_groups MUST be an object"
    );
}

#[test]
fn derive_child_omits_meta_template_groups() {
    // v0.17.1 P0: derive-child does NOT consume `--template` (the subcommand
    // has zero `--template` references in `crates/mnemonic-toolkit/src/cmd/
    // derive_child.rs`). The previous test cell at this position
    // (`derive_child_emits_meta_template_groups`) enshrined the wrong
    // invariant — `build_subcommand_meta`'s spurious match-arm inclusion of
    // `derive-child` emitted a meta.template_groups block on a subcommand
    // with no `--template` widget. The v0.17.1 fix removes the match arm;
    // this negative cell guards against re-introduction.
    // Tracks FOLLOWUP `gui-schema-derive-child-meta-template-groups-spurious`.
    let v = run_gui_schema();
    let derive_child = find_sub(&v, "derive-child");
    let template_groups = &derive_child["meta"]["template_groups"];
    assert!(
        template_groups.is_null() || derive_child["meta"].is_null(),
        "derive-child subcommand MUST NOT emit meta.template_groups (no \
         --template flag in derive_child.rs). Got: {template_groups:?}"
    );
}

#[test]
fn subcommands_without_template_flag_omit_meta_template_groups() {
    // §6.10.8 requires meta.template_groups for subcommands that consume
    // `--template`. Subcommands that do NOT consume `--template` MUST omit
    // the block entirely (no empty `single_sig: []` / `multisig: []` stub).
    let v = run_gui_schema();
    // `convert` does not consume `--template` (it operates on xpub
    // network-prefix transforms).
    let convert = find_sub(&v, "convert");
    let convert_template_groups = &convert["meta"]["template_groups"];
    assert!(
        convert_template_groups.is_null() || convert["meta"].is_null(),
        "convert subcommand MUST NOT emit meta.template_groups (convert does \
         not consume --template). Got: {convert_template_groups:?}"
    );
}

// ── §6.10.6 v3 schema-version regression guard ──────────────────────────────

#[test]
fn v3_schema_includes_all_v2_cycle_surfaces() {
    // Smoke test that the v3-cycle features ship together. Catches the case
    // where someone bumps version 2→3 but forgets one of the new emissions
    // (e.g., bumps version but doesn't add meta.template_groups).
    let v = run_gui_schema();
    assert_eq!(v["version"], 3, "schema version must be v3");
    // pin_value rule on bundle:
    let bundle = find_sub(&v, "bundle");
    let rules = bundle["conditional_rules"].as_array().unwrap();
    let has_pin_value_rule = rules.iter().any(|r| {
        r["effect"]["visibility"].is_object()
            && r["effect"]["visibility"]["pin_value"].is_object()
    });
    assert!(
        has_pin_value_rule,
        "bundle must contain at least one pin_value rule (row 12 \
         DESCRIPTOR_WITH_NONZERO_ACCOUNT)"
    );
    // meta block on a template-consuming subcommand:
    assert!(
        bundle["meta"]["template_groups"].is_object(),
        "bundle must have meta.template_groups"
    );
}
