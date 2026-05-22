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
fn gui_schema_top_level_version_is_five() {
    let v = run_gui_schema();
    // v0.24.0 Tranche B.1 bump: SPEC §7 — schema version 4 → 5 for the
    // additive Flag fields {default_value, global, secret}.
    // Earlier bumps: v1→v2 at v0.16.0 (conditional_rules added); v2→v3 at
    // v0.17.0 (slot_count_* + pin_value + meta.template_groups);
    // v3→v4 at v0.18.0 (disable_options Visibility variant).
    // GUI's relaxed parse_gui_schema_json accepts version >= 1; new
    // fields consumers gate on version >= 5.
    assert_eq!(v["version"], 5, "SPEC §7: version pin v0.24.0 Tranche B.1");
}

#[test]
fn gui_schema_top_level_cli_is_mnemonic() {
    let v = run_gui_schema();
    assert_eq!(v["cli"], "mnemonic", "SPEC §7: cli identifier");
}

#[test]
fn gui_schema_lists_all_subcommands() {
    let v = run_gui_schema();
    let subs = v["subcommands"].as_array().expect("subcommands array");
    let names: Vec<&str> = subs.iter().map(|s| s["name"].as_str().unwrap()).collect();
    // Sorted alphabetically by build_schema. v0.13.0 P2.1 flattens nested
    // clap subcommands into hyphenated names: `seed-xor` becomes
    // `seed-xor-split` + `seed-xor-combine` (repairs v0.12.0 pre-existing
    // empty-flags rendering); new `slip39` enters as `slip39-split` +
    // `slip39-combine`. v0.22.0 adds `repair` (BCH error-correction) and
    // `inspect` (describe-card-contents). v0.26.0 lands three features in
    // lockstep: `compare-cost` (wsh-vs-tr per-spending-condition cost), the
    // `xpub-search` umbrella (4 flattened modes), and `import-wallet`
    // (third-party wallet blob → bundle). After all three, the list carried
    // 18 subcommands: 12 base + `compare-cost` + 4 xpub-search-* +
    // `import-wallet`. v0.30.0 adds `seedqr-decode` + `seedqr-encode`
    // (SeedQR encode/decode), bringing it to 20. v0.33.0 adds
    // `electrum-decrypt` (Electrum field-encrypted secret → plaintext);
    // total was 21. v0.34.0 adds `nostr` (wrap a nostr key as BTC
    // address/descriptor/WIF); total is now 22. gui-schema + help filtered out.
    assert_eq!(
        names,
        vec![
            "bundle",
            "compare-cost",
            "convert",
            "derive-child",
            "electrum-decrypt",
            "export-wallet",
            "final-word",
            "import-wallet",
            "inspect",
            "nostr",
            "repair",
            "seed-xor-combine",
            "seed-xor-split",
            "seedqr-decode",
            "seedqr-encode",
            "slip39-combine",
            "slip39-split",
            "verify-bundle",
            "xpub-search-account-of-descriptor",
            "xpub-search-address-of-xpub",
            "xpub-search-passphrase-of-xpub",
            "xpub-search-path-of-xpub",
        ],
        "all 22 user-facing subcommands must appear (nested split/combine \
         flattened to hyphenated names); gui-schema + help filtered out"
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
fn convert_subcommand_has_required_from_text_and_to_dropdown() {
    // --from uses a custom value_parser (FromInput parsing `<node>=<value>`),
    // so SPEC §7's lossy mapping collapses it to "text"; the GUI re-parses.
    //
    // --to was upgraded (mnemonic-toolkit post-v0.13.0) from a free-form
    // `Vec<String>` to a `PossibleValuesParser` over the 13 NODE_TYPE
    // tokens. gui-schema introspection now emits "dropdown" with the
    // choices list, matching the GUI's `schema/mnemonic.rs::NODE_TYPES`.
    let v = run_gui_schema();
    let convert = find_sub(&v, "convert");
    let from = find_flag(convert, "--from");
    assert_eq!(from["required"], true);
    assert_eq!(from["kind"], "text");
    let to = find_flag(convert, "--to");
    assert_eq!(to["required"], true);
    assert_eq!(to["kind"], "dropdown");
    let choices = to["choices"].as_array().expect("--to choices is an array");
    let choice_strs: Vec<&str> = choices.iter().filter_map(|c| c.as_str()).collect();
    assert_eq!(
        choice_strs,
        vec![
            "phrase",
            "entropy",
            "xpub",
            "xprv",
            "wif",
            "fingerprint",
            "path",
            "ms1",
            "mk1",
            "bip38",
            "minikey",
            "electrum-phrase",
            "address",
        ]
    );
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
    // SPEC v0.8: 8 vendor formats; v0.27.0 adds `bsms` (BIP-129 Round-2
    // emitter) for a total of 9; v0.28.4 adds `coldcard-multisig` alias
    // (export-side semantic precheck) for a total of 10.
    for vendor in [
        "bitcoin-core",
        "bip388",
        "coldcard",
        "coldcard-multisig",
        "jade",
        "sparrow",
        "specter",
        "electrum",
        "green",
        "bsms",
    ] {
        assert!(choices.contains(&vendor), "format must include {vendor}");
    }
    assert_eq!(choices.len(), 10);
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
