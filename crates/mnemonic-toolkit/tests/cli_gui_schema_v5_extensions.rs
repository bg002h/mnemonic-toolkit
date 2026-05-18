//! `mnemonic gui-schema` SPEC §7 v0.24.0 Tranche B.1 extensions (schema v5).
//!
//! Pins the three additive Flag fields introduced at v0.24.0:
//!
//! - `default_value` — JSON value mirroring clap-derive's `default_value` /
//!   `default_value_t` for each flag, shaped per the flag's `kind`
//!   (`number` → integer, `text` / `path` / `dropdown` → string,
//!   `boolean` → omitted). Field is omitted from JSON when no default
//!   is declared.
//!
//! - `global` — boolean (defaulting to false / omitted) marking flags
//!   that originate from a parent-Command global declaration. The
//!   emitter propagates them into every subcommand's flag set so GUI
//!   consumers can decide whether to render globally (e.g. action-bar
//!   checkbox) or per-subcommand. Currently `--no-auto-repair` is the
//!   only global flag.
//!
//! - `secret` — boolean (defaulting to false / omitted) marking flags
//!   whose values carry sensitive material (per the authoritative
//!   `crate::secrets::flag_is_secret` predicate). GUI consumers drive
//!   paste-warn / run-confirm modals and exit-time zeroize sweeps on
//!   the union of `secret: true` flags.
//!
//! Pre-v5 / v4 cycle surfaces (disable_options) live at
//! `cli_gui_schema_v4_extensions.rs`; pre-v4 / v3 cycle surfaces live at
//! `cli_gui_schema_v3_extensions.rs`; pre-v3 baseline rules live at
//! `cli_gui_schema_conditional_rules.rs`.

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

fn find_flag<'a>(sub: &'a Value, flag_name: &str) -> &'a Value {
    sub["flags"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == flag_name)
        .unwrap_or_else(|| panic!("flag `{flag_name}` not in subcommand"))
}

// ── §7 v5 schema-version pin (integer, NOT string) ──────────────────────────

#[test]
fn v5_schema_version_is_integer_not_string() {
    // Per the v0.24.0 Tranche B.1 kickoff: schema_version is an
    // integer, not a stringified integer. Pin the wire type alongside
    // the value to catch a future regression where someone refactors
    // the field to `String`.
    let v = run_gui_schema();
    assert!(
        v["version"].is_i64() || v["version"].is_u64(),
        "schema `version` must be an integer; got {:?}",
        v["version"]
    );
    assert_eq!(
        v["version"].as_i64(),
        Some(5),
        "v0.24.0 Tranche B.1 pins version=5"
    );
}

// ── §7 v5 `default_value` field ─────────────────────────────────────────────

#[test]
fn convert_account_carries_default_value_zero_as_integer() {
    // `convert --account` is the canonical numeric-default example: clap-derive
    // `default_value = "0"` + numeric value_parser. Emitted as `Number(0)`,
    // NOT as `String("0")`.
    let v = run_gui_schema();
    let convert = find_sub(&v, "convert");
    let account = find_flag(convert, "--account");
    assert_eq!(
        account["default_value"], 0,
        "convert --account default must serialize as integer 0"
    );
    assert!(
        account["default_value"].is_number(),
        "convert --account default must be a JSON number, not a string; got {:?}",
        account["default_value"]
    );
}

#[test]
fn export_wallet_network_carries_default_value_mainnet_as_string() {
    // `export-wallet --network` is the canonical dropdown-default example.
    // String default for an enum-typed flag.
    let v = run_gui_schema();
    let xw = find_sub(&v, "export-wallet");
    let network = find_flag(xw, "--network");
    assert_eq!(network["default_value"], "mainnet");
    assert!(network["default_value"].is_string());
}

#[test]
fn export_wallet_timestamp_carries_default_value_now_as_string() {
    // `export-wallet --timestamp` defaults to "now" (parsed at runtime
    // into the current Unix epoch via parse_timestamp). The emitter
    // preserves the source string so GUIs can show "now" as the
    // placeholder text and only diverge when the user opts in.
    let v = run_gui_schema();
    let xw = find_sub(&v, "export-wallet");
    let ts = find_flag(xw, "--timestamp");
    assert_eq!(ts["default_value"], "now");
    assert!(ts["default_value"].is_string());
}

#[test]
fn flags_without_clap_derive_defaults_omit_default_value() {
    // `convert --from` has no `default_value` — it's a required
    // composite-node selector. The field must be ABSENT from the JSON
    // (not present-but-null) so v4 readers (which don't know about the
    // field) round-trip cleanly and forward-compat readers don't
    // confuse "omitted" with "explicit null".
    let v = run_gui_schema();
    let convert = find_sub(&v, "convert");
    let from = find_flag(convert, "--from");
    assert!(
        from.get("default_value").is_none(),
        "flags with no clap-derive default must omit the default_value key entirely; \
         got {:?}",
        from.get("default_value")
    );
}

#[test]
fn boolean_flags_omit_default_value_even_when_clap_declares_false() {
    // Bool flags conventionally default to false; the emitter omits
    // `default_value` for all booleans (a bool flag's absence is its
    // default). Confirmed against bundle's `--passphrase-stdin`
    // (declared with `default_value_t = false` on some sibling sites,
    // implicit false here).
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    let stdin_flag = find_flag(bundle, "--passphrase-stdin");
    assert_eq!(stdin_flag["kind"], "boolean");
    assert!(
        stdin_flag.get("default_value").is_none(),
        "boolean flags must omit default_value; got {:?}",
        stdin_flag.get("default_value")
    );
}

// ── §7 v5 `global` field ────────────────────────────────────────────────────

#[test]
fn no_auto_repair_appears_with_global_true_in_every_subcommand() {
    // `--no-auto-repair` is declared `global = true` on the top-level
    // `Cli` struct. The v5 emitter propagates it into every
    // subcommand's flag set with `global: true` so GUI consumers can
    // discover the global without parsing the top-level Cli definition
    // separately. Asserts coverage across all 12 user-facing
    // subcommands (sorted alphabetically per `gui_schema.rs`'s build).
    let v = run_gui_schema();
    let names = vec![
        "bundle",
        "convert",
        "derive-child",
        "export-wallet",
        "final-word",
        "inspect",
        "repair",
        "seed-xor-combine",
        "seed-xor-split",
        "slip39-combine",
        "slip39-split",
        "verify-bundle",
    ];
    for name in &names {
        let sub = find_sub(&v, name);
        let nar = find_flag(sub, "--no-auto-repair");
        assert_eq!(
            nar["global"], true,
            "subcommand `{name}` must carry --no-auto-repair with global: true"
        );
        assert_eq!(
            nar["kind"], "boolean",
            "--no-auto-repair must be kind=boolean in {name}"
        );
    }
}

#[test]
fn non_global_flags_omit_global_field() {
    // Per the additive-skip-when-default convention: non-global flags
    // omit the `global` key entirely. `convert --account` is the
    // canonical non-global example.
    let v = run_gui_schema();
    let convert = find_sub(&v, "convert");
    let account = find_flag(convert, "--account");
    assert!(
        account.get("global").is_none(),
        "non-global flags must omit the global key; got {:?}",
        account.get("global")
    );
}

// ── §7 v5 `secret` field ────────────────────────────────────────────────────

#[test]
fn passphrase_flag_carries_secret_true_in_bundle() {
    // `--passphrase` is the canonical secret-flag example. Marked
    // secret in `crate::secrets::flag_is_secret`.
    let v = run_gui_schema();
    let bundle = find_sub(&v, "bundle");
    let pp = find_flag(bundle, "--passphrase");
    assert_eq!(pp["secret"], true);
}

#[test]
fn ms1_flag_carries_secret_true_in_repair_and_inspect() {
    // `--ms1` carries secret BIP-39 entropy material. Repair and
    // inspect subcommands both take it as a flat flag.
    let v = run_gui_schema();
    for subname in &["repair", "inspect"] {
        let sub = find_sub(&v, subname);
        let ms1 = find_flag(sub, "--ms1");
        assert_eq!(
            ms1["secret"], true,
            "{subname} --ms1 must be marked secret"
        );
    }
}

#[test]
fn share_flag_carries_secret_true_in_slip39_combine() {
    // `--share` carries SLIP-39 share material. The slip39-combine
    // subcommand takes it as a repeating flag.
    let v = run_gui_schema();
    let combine = find_sub(&v, "slip39-combine");
    let share = find_flag(combine, "--share");
    assert_eq!(share["secret"], true);
}

#[test]
fn non_secret_flags_omit_secret_field() {
    // Per the additive-skip-when-default convention: non-secret flags
    // omit the `secret` key entirely.
    let v = run_gui_schema();
    let convert = find_sub(&v, "convert");
    let account = find_flag(convert, "--account");
    assert!(
        account.get("secret").is_none(),
        "non-secret flags must omit the secret key; got {:?}",
        account.get("secret")
    );
    // --mk1 / --md1 are pubkey / descriptor material, not secret.
    let repair = find_sub(&v, "repair");
    let mk1 = find_flag(repair, "--mk1");
    let md1 = find_flag(repair, "--md1");
    assert!(
        mk1.get("secret").is_none(),
        "--mk1 (xpub) must NOT be marked secret"
    );
    assert!(
        md1.get("secret").is_none(),
        "--md1 (descriptor) must NOT be marked secret"
    );
}

// ── §7 v5 secret-flag full enumeration ──────────────────────────────────────

#[test]
fn secret_flag_enumeration_matches_authoritative_predicate() {
    // Spot-check that every flag name in the toolkit's
    // `secrets::flag_is_secret` enumeration that appears in the
    // emitted schema actually carries `secret: true`. Iterates every
    // flag in every subcommand and asserts wire-shape consistency
    // with the predicate. This is the toolkit-internal half of the
    // drift gate; the GUI-side mirror lives at
    // `mnemonic-gui/src/secrets.rs` (Tranche B.3 wires up the
    // cross-repo drift cell).
    let v = run_gui_schema();
    for sub in v["subcommands"].as_array().unwrap() {
        for flag in sub["flags"].as_array().unwrap() {
            let name = flag["name"].as_str().unwrap();
            let emitted_secret = flag.get("secret").and_then(|v| v.as_bool()).unwrap_or(false);
            let expected_secret = mnemonic_toolkit::secrets::flag_is_secret(name);
            assert_eq!(
                emitted_secret,
                expected_secret,
                "subcommand `{}` flag `{}`: emitted secret={} but predicate says {}",
                sub["name"], name, emitted_secret, expected_secret
            );
        }
    }
}

// ── §7 v5 global-vs-local flag-id disjointness invariant (v0.25.0 Phase 3) ──

#[test]
fn global_local_id_disjointness_invariant_holds_in_current_schema() {
    // v0.25.0 Phase 3 — the gui-schema emitter's
    // `assert_global_local_id_disjointness` debug-assert enforces that no
    // subcommand declares a local flag whose clap-derive identifier
    // collides with a root-level `global = true` flag. The toolkit-internal
    // helper checks IDs; this test exercises the END-TO-END wire shape via
    // the emitted JSON: for every subcommand, the set of `global: true`
    // flag names must be disjoint from the set of `global: false / omitted`
    // flag names.
    //
    // This positive-invariant cell runs in BOTH debug AND release builds —
    // it does NOT depend on `debug_assertions` to fire. It validates the
    // shipped wire contract regardless of how the emitter implements the
    // partition internally.
    //
    // The companion negative-control cell
    // (`global_local_collision_triggers_debug_assert`) is
    // `#[cfg(debug_assertions)]`-gated and exercises the actual
    // `debug_assert!` panic path.
    let v = run_gui_schema();
    for sub in v["subcommands"].as_array().unwrap() {
        let mut globals: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        let mut locals: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        for flag in sub["flags"].as_array().unwrap() {
            let name = flag["name"].as_str().unwrap().to_string();
            let is_global = flag
                .get("global")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if is_global {
                globals.insert(name);
            } else {
                locals.insert(name);
            }
        }
        let intersection: Vec<&String> = globals.intersection(&locals).collect();
        assert!(
            intersection.is_empty(),
            "subcommand `{}`: global flag names overlap local flag names: {:?}",
            sub["name"],
            intersection
        );
    }
}

// The negative-control cell `global_local_collision_triggers_debug_assert`
// lives as a unit test inside `crates/mnemonic-toolkit/src/cmd/gui_schema.rs`
// — it must call the `pub(crate) assert_global_local_id_disjointness`
// helper directly, which is not reachable from this integration-test crate.
// The helper's `pub(crate)` visibility is intentional (the function is an
// internal invariant guard, not public API).
