//! `mnemonic gui-schema` SPEC §7 contract tests.
//!
//! Companion to the `mnemonic-gui` v0.2 Phase C.2 schema-mirror contract
//! (`bg002h/mnemonic-gui` `FOLLOWUPS.md` mnemonic-gui-schema-mirror).
//!
//! The GUI overlay rejects schema docs whose `version != 1` or whose `cli`
//! string doesn't match the expected upstream identifier. These tests pin
//! both invariants and spot-check the per-subcommand flag surface so that
//! any clap-derive drift surfaces before tagging a release.

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

#[test]
fn gui_schema_exits_zero_and_parses_as_json() {
    let _ = run_gui_schema();
}

#[test]
fn gui_schema_top_level_version_is_one() {
    let v = run_gui_schema();
    assert_eq!(v["version"], 1, "SPEC §7: version pin (GUI rejects != 1)");
}

#[test]
fn gui_schema_top_level_cli_is_mnemonic() {
    let v = run_gui_schema();
    assert_eq!(v["cli"], "mnemonic", "SPEC §7: cli identifier");
}

#[test]
fn gui_schema_lists_all_six_subcommands() {
    let v = run_gui_schema();
    let subs = v["subcommands"].as_array().expect("subcommands array");
    let names: Vec<&str> = subs.iter().map(|s| s["name"].as_str().unwrap()).collect();
    // Sorted alphabetically by build_schema. v0.11.0 adds `final-word`.
    assert_eq!(
        names,
        vec![
            "bundle",
            "convert",
            "derive-child",
            "export-wallet",
            "final-word",
            "verify-bundle",
        ],
        "all 6 user-facing subcommands must appear; gui-schema + help are filtered out"
    );
}

#[test]
fn gui_schema_does_not_self_reference() {
    let v = run_gui_schema();
    let subs = v["subcommands"].as_array().unwrap();
    for s in subs {
        assert_ne!(
            s["name"], "gui-schema",
            "gui-schema must not appear in its own output (self-reference suppression)"
        );
        assert_ne!(
            s["name"], "help",
            "clap-auto-generated `help` subcommand must be filtered out"
        );
    }
}

fn find_sub<'a>(v: &'a Value, name: &str) -> &'a Value {
    v["subcommands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == name)
        .unwrap_or_else(|| panic!("subcommand `{name}` not in schema"))
}

fn find_flag<'a>(sub: &'a Value, name: &str) -> &'a Value {
    sub["flags"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == name)
        .unwrap_or_else(|| panic!("flag `{name}` not in subcommand"))
}

#[test]
fn bundle_subcommand_has_network_flag_as_dropdown() {
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    let network = find_flag(bundle, "--network");
    assert_eq!(network["required"], true);
    assert_eq!(network["kind"], "dropdown");
    let choices: Vec<&str> = network["choices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    assert_eq!(choices, vec!["mainnet", "testnet", "signet", "regtest"]);
}

#[test]
fn bundle_subcommand_has_template_dropdown_with_v0_2_multisig_templates() {
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    let template = find_flag(bundle, "--template");
    assert_eq!(template["kind"], "dropdown");
    let choices: Vec<&str> = template["choices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    // 4 single-sig (bip44/49/84/86) + 6 multisig (wsh-{multi,sortedmulti},
    // sh-wsh-{multi,sortedmulti}, tr-{multi-a,sortedmulti-a}).
    assert!(choices.contains(&"bip84"));
    assert!(choices.contains(&"wsh-sortedmulti"));
    assert!(choices.contains(&"tr-sortedmulti-a"));
    assert_eq!(choices.len(), 10);
}

#[test]
fn bundle_subcommand_has_boolean_flags() {
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    for flag in ["--json", "--no-engraving-card", "--privacy-preserving", "--self-check"] {
        let f = find_flag(bundle, flag);
        assert_eq!(f["kind"], "boolean", "{flag} must be kind=boolean");
        assert!(f["choices"].is_null(), "{flag} must have null choices");
    }
}

#[test]
fn bundle_subcommand_has_numeric_account_and_threshold() {
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    assert_eq!(find_flag(bundle, "--account")["kind"], "number");
    assert_eq!(find_flag(bundle, "--threshold")["kind"], "number");
}

#[test]
fn bundle_subcommand_has_path_descriptor_file() {
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    assert_eq!(find_flag(bundle, "--descriptor-file")["kind"], "path");
}

#[test]
fn convert_subcommand_has_required_from_and_to_as_text() {
    // --from and --to use custom value_parsers (FromInput / ToInput), so
    // SPEC §7's lossy mapping collapses them to "text". The GUI re-parses.
    let v = run_gui_schema();
    let convert = find_sub(&v, "convert");
    let from = find_flag(convert, "--from");
    assert_eq!(from["required"], true);
    assert_eq!(from["kind"], "text");
    let to = find_flag(convert, "--to");
    assert_eq!(to["required"], true);
    assert_eq!(to["kind"], "text");
}

#[test]
fn derive_child_has_four_required_flags() {
    let v = run_gui_schema();
    let dc = find_sub(&v, "derive-child");
    for name in ["--from", "--application", "--length", "--index"] {
        let f = find_flag(dc, name);
        assert_eq!(f["required"], true, "{name} must be required");
    }
    // Numeric ones.
    assert_eq!(find_flag(dc, "--length")["kind"], "number");
    assert_eq!(find_flag(dc, "--index")["kind"], "number");
}

#[test]
fn export_wallet_has_format_dropdown_with_eight_vendors() {
    let v = run_gui_schema();
    let ew = find_sub(&v, "export-wallet");
    let fmt = find_flag(ew, "--format");
    assert_eq!(fmt["kind"], "dropdown");
    let choices: Vec<&str> = fmt["choices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    // SPEC v0.8: 8 vendor formats.
    for vendor in [
        "bitcoin-core",
        "bip388",
        "coldcard",
        "jade",
        "sparrow",
        "specter",
        "electrum",
        "green",
    ] {
        assert!(choices.contains(&vendor), "format must include {vendor}");
    }
    assert_eq!(choices.len(), 8);
}

#[test]
fn verify_bundle_has_bundle_json_path_flag() {
    let v = run_gui_schema();
    let vb = find_sub(&v, "verify-bundle");
    let bj = find_flag(vb, "--bundle-json");
    // verify_bundle::VerifyBundleArgs declares `--bundle-json` as `PathBuf`,
    // so SPEC §7 maps it to kind=path.
    assert_eq!(bj["kind"], "path");
}

#[test]
fn all_flags_carry_required_field_as_bool() {
    let v = run_gui_schema();
    for sub in v["subcommands"].as_array().unwrap() {
        for f in sub["flags"].as_array().unwrap() {
            assert!(
                f["required"].is_boolean(),
                "{}::{} required must be bool",
                sub["name"],
                f["name"]
            );
            // SPEC §7: choices is non-null only when kind == dropdown.
            if f["kind"] == "dropdown" {
                assert!(
                    f["choices"].is_array(),
                    "{}::{} kind=dropdown must have choices array",
                    sub["name"],
                    f["name"]
                );
            } else {
                assert!(
                    f["choices"].is_null(),
                    "{}::{} kind={} must have null choices",
                    sub["name"],
                    f["name"],
                    f["kind"]
                );
            }
        }
    }
}

#[test]
fn kind_values_are_in_spec_enum() {
    let v = run_gui_schema();
    let allowed = ["text", "boolean", "number", "dropdown", "path"];
    for sub in v["subcommands"].as_array().unwrap() {
        for f in sub["flags"].as_array().unwrap() {
            let k = f["kind"].as_str().unwrap();
            assert!(
                allowed.contains(&k),
                "kind {k:?} for {}::{} not in SPEC §7 enum",
                sub["name"],
                f["name"]
            );
        }
    }
}
