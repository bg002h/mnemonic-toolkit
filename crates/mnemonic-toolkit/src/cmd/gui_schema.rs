//! `mnemonic gui-schema` subcommand — emit SPEC §7 GUI-overlay schema JSON.
//!
//! Companion to the `mnemonic-gui` v0.2 Phase C.2 contract
//! (`bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `mnemonic-gui-schema-mirror`).
//!
//! Walks the clap `Command` tree via `clap::CommandFactory` and serializes a
//! machine-readable schema of every existing subcommand's flag surface to
//! stdout as JSON. The GUI consumes this schema to render forms. The
//! envelope `version` field is an integer; v5 (v0.24.0+) is the current
//! emission. `schema_check::parse_gui_schema_json` accepts any `version >= 1`
//! and ignores additive per-flag fields, so older GUI builds still parse
//! the envelope (the per-flag `default_value`/`global`/`secret` fields
//! added in v5 simply pass through unread).
//!
//! ## SPEC §7 contract
//!
//! ```json
//! {
//!   "version": 5,
//!   "cli": "mnemonic",
//!   "subcommands": [
//!     {
//!       "name": "bundle",
//!       "flags":       [ { "name", "required", "kind", "choices": [..] | null,
//!                          "default_value"?: any, "global"?: bool, "secret"?: bool } ],
//!       "positionals": [ { "name", "required", "repeating" } ]
//!     }
//!   ]
//! }
//! ```
//!
//! ## kind mapping
//!
//! | Rust type / clap annotation                          | kind         | choices            |
//! |------------------------------------------------------|--------------|--------------------|
//! | `bool` (`ArgAction::SetTrue`)                        | `boolean`    | null               |
//! | numeric `value_parser` (i64/u32/u64/u8/...)          | `number`     | null               |
//! | enum w/ `value_enum` or `PossibleValuesParser`       | `dropdown`   | array of variants  |
//! | `PathBuf` / `Path`                                   | `path`       | null               |
//! | everything else (`String`, custom value_parsers, …)  | `text`       | null               |
//!
//! The mapping is intentionally lossy for complex GUI variants
//! (NodeValueComposite / TaggedOrIndexed / Range / Timestamp) per the
//! SPEC §7 contract — those collapse to `"text"` upstream and the GUI
//! re-parses client-side.
//!
//! Self-reference is suppressed: the `gui-schema` subcommand itself is
//! filtered out of its own output.

use crate::error::ToolkitError;
use crate::template::CliTemplate;
use clap::{Args, Command, ValueEnum};
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;
use std::io::Write;

#[derive(Args, Debug)]
pub struct GuiSchemaArgs {
    /// v0.20.0 F2 — classify a descriptor as `canonical` or `non-canonical` per
    /// md-codec's `canonical_origin` table. Mutually exclusive with the default
    /// JSON-schema emission; when set, the JSON schema is suppressed and a
    /// single line of stdout (`canonical\n` or `non-canonical\n`) is printed.
    /// Exit 0 on parse success; exit 2 on parse failure.
    #[arg(long, value_name = "DESCRIPTOR")]
    pub classify_descriptor: Option<String>,
}

#[derive(Serialize, Debug)]
struct Schema {
    version: u32,
    cli: String,
    subcommands: Vec<Subcommand>,
}

#[derive(Serialize, Debug)]
struct Subcommand {
    name: String,
    flags: Vec<Flag>,
    positionals: Vec<Positional>,
    /// SPEC §6.10 conditional-applicability projection. Empty array for
    /// subcommands without per-frame visibility constraints. Always present
    /// (never omitted) so v2 consumers can rely on the field's presence.
    conditional_rules: Vec<ConditionalRule>,
    /// SPEC §6.10.8 (NEW v3) per-subcommand meta-fields block. Currently
    /// contains the `template_groups` block for subcommands that consume
    /// `--template`; future v3 cycles add more fields here additively
    /// (additive at the field level — no schema-version bump required for
    /// new meta keys; only Predicate/Effect changes bump the version).
    /// Empty BTreeMap serializes as omitted (no `meta` key in JSON) so
    /// subcommands without meta surfaces remain byte-identical with v2.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    meta: BTreeMap<String, serde_json::Value>,
}

/// SPEC §6.10 ConditionalRule projection. See SPEC §6.10.1–§6.10.7.
#[derive(Serialize, Debug)]
struct ConditionalRule {
    rationale: String,
    spec_ref: String,
    when: Predicate,
    effect: Effect,
}

/// SPEC §6.10.2 Predicate AST. Tagged JSON union via serde's internal tag.
///
/// v0.17.0 / schema v3 adds the three slot_count_* variants for slot-grid-
/// dependent predicates. These exist as predicate-machinery for future Effect-
/// grammar extensions; no v0.17 rule currently uses them. See SPEC §6.10.2
/// closing paragraph + §6.10.7 closing list for the deferred-effect status.
#[derive(Serialize, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Predicate {
    FlagPresent {
        flag: String,
    },
    DropdownValueIn {
        flag: String,
        values: Vec<String>,
    },
    #[allow(dead_code)]
    CompositeNodeIs {
        flag: String,
        node: String,
    },
    #[allow(dead_code)]
    PositionalPresent {
        index: usize,
    },
    #[allow(dead_code)]
    AllOf {
        predicates: Vec<Predicate>,
    },
    #[allow(dead_code)]
    AnyOf {
        predicates: Vec<Predicate>,
    },
    Not {
        predicate: Box<Predicate>,
    },
    #[allow(dead_code)]
    SlotCountEq {
        value: usize,
    },
    #[allow(dead_code)]
    SlotCountGte {
        value: usize,
    },
    #[allow(dead_code)]
    SlotCountLte {
        value: usize,
    },
}

/// SPEC §6.10.3 Effect.
#[derive(Serialize, Debug)]
struct Effect {
    flag: String,
    visibility: VisibilityProjection,
}

/// SPEC §6.10.3 VisibilityProjection. `Visible` is the implicit default and
/// never appears as an Effect value.
///
/// v1 cycle (v0.16.0) used `Disabled` exclusively to match the existing GUI
/// hand-coding pattern (`mnemonic-gui/src/form/conditional.rs` uses
/// `Visibility::Disabled` for both structurally-inapplicable and user-mutex
/// cases). `Hidden` is reserved for a future cycle that distinguishes the two
/// UX classes per SPEC §6.10.3's "Hidden = structurally non-applicable"
/// framing.
///
/// v2 cycle (v0.17.0) adds `PinValue { value }` — a data-carrying variant
/// with REPLACE-value emission semantic (the GUI emits `--name <V>` using the
/// pinned value V, distinct from hidden/disabled which suppress emission).
/// `Copy` is dropped from the derive set because `serde_json::Value` is not
/// `Copy`. Custom `Serialize` impl preserves the bare-string wire shape for
/// `Hidden`/`Disabled`/`Required` (v2 back-compat per SPEC §6.10.6) and emits
/// `PinValue` as a tagged-object `{"pin_value": {"value": V}}`.
#[derive(Debug, Clone)]
enum VisibilityProjection {
    #[allow(dead_code)]
    Hidden,
    Disabled,
    Required,
    PinValue {
        value: serde_json::Value,
    },
    // v0.18.1 — DisableOptions remains a defined v4 grammar variant
    // even after the row 10/11 rollback (no rule currently constructs
    // it). Kept for forward-compat: future cycles may identify
    // contexts where greying dropdown options serves UX better than
    // an inline warning. SPEC §6.10.3 documents the wire shape.
    #[allow(dead_code)]
    DisableOptions {
        values: Vec<String>,
    },
}

impl Serialize for VisibilityProjection {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        match self {
            Self::Hidden => ser.serialize_str("hidden"),
            Self::Disabled => ser.serialize_str("disabled"),
            Self::Required => ser.serialize_str("required"),
            Self::PinValue { value } => {
                // Wire shape per SPEC §6.10.3 v3:
                //   {"pin_value": {"value": V}}
                let mut outer = ser.serialize_map(Some(1))?;
                let mut inner = serde_json::Map::new();
                inner.insert("value".to_string(), value.clone());
                outer.serialize_entry("pin_value", &inner)?;
                outer.end()
            }
            Self::DisableOptions { values } => {
                // Wire shape per SPEC §6.10.3 v4:
                //   {"disable_options": {"values": [<string>, ...]}}
                // The inner-key form (rather than bare-array) mirrors the v3
                // pin_value precedent and leaves room for future per-Effect
                // metadata without a wire-shape break.
                let mut outer = ser.serialize_map(Some(1))?;
                let mut inner = serde_json::Map::new();
                inner.insert(
                    "values".to_string(),
                    serde_json::Value::Array(
                        values
                            .iter()
                            .map(|v| serde_json::Value::String(v.clone()))
                            .collect(),
                    ),
                );
                outer.serialize_entry("disable_options", &inner)?;
                outer.end()
            }
        }
    }
}

#[derive(Serialize, Debug)]
struct Flag {
    name: String,
    required: bool,
    kind: String,
    choices: Option<Vec<String>>,
    /// v0.24.0 schema v5 — flag's clap-derive `default_value` if any, mapped
    /// to a JSON value whose shape matches `kind`. Omitted when the clap-derive
    /// site declares no default (or when the default is uninteresting, e.g.
    /// a boolean flag's implicit `false`). See `extract_default_value` for
    /// the per-kind mapping rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    default_value: Option<serde_json::Value>,
    /// v0.24.0 schema v5 — true iff the flag is a parent-Command global flag
    /// (clap-derive `global = true`) that propagates into this subcommand.
    /// Omitted when false (defaulted). Currently emits true for
    /// `--no-auto-repair` across all subcommands.
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    global: bool,
    /// v0.24.0 schema v5 — true iff the flag carries secret material (per
    /// the authoritative `secrets::flag_is_secret` predicate). Omitted when
    /// false (defaulted). GUI consumers use this to drive paste-warn /
    /// run-confirm modals and exit-time zeroize sweeps.
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    secret: bool,
}

#[derive(Serialize, Debug)]
struct Positional {
    name: String,
    required: bool,
    repeating: bool,
}

/// SPEC §6.10.7-derived single-sig template list. Derived from
/// `CliTemplate::is_multisig()` (source-of-truth at `template.rs:46-56`) so the
/// projection stays in sync with template-enum additions automatically. Used
/// by `dropdown_value_in("--template", SINGLE_SIG)` predicates in
/// bundle/verify-bundle/export-wallet rules.
fn single_sig_template_values() -> Vec<String> {
    CliTemplate::value_variants()
        .iter()
        .filter(|t| !t.is_multisig())
        .filter_map(|t| t.to_possible_value().map(|p| p.get_name().to_string()))
        .collect()
}

/// SPEC §6.10.8-derived multisig template list. Mirror of
/// `single_sig_template_values` for the multisig partition. Used by the
/// per-subcommand `meta.template_groups` emission (v0.17.0 / schema v3).
fn multisig_template_values() -> Vec<String> {
    CliTemplate::value_variants()
        .iter()
        .filter(|t| t.is_multisig())
        .filter_map(|t| t.to_possible_value().map(|p| p.get_name().to_string()))
        .collect()
}

/// SPEC §6.10.8 — per-subcommand `meta` block. Returns the meta map for
/// subcommands that have one (currently: any subcommand that consumes
/// `--template`); empty map for the rest (which serializes as omitted).
///
/// Source-of-truth for the template-consumer list: this match is hand-coded
/// to mirror the subcommands whose clap-derive `#[arg]` set includes
/// `--template`. Adding a new template-consuming subcommand requires
/// extending this match in lockstep — the drift gate
/// (`mnemonic-gui/tests/gui_schema_conditional_drift.rs`) catches divergence.
fn build_subcommand_meta(name: &str) -> BTreeMap<String, serde_json::Value> {
    let mut meta = BTreeMap::new();
    // v0.17.1 P0: derive-child REMOVED from this match arm. Although v0.17.0
    // emitted a meta.template_groups block for derive-child, that subcommand
    // has zero `--template` references in `crates/mnemonic-toolkit/src/cmd/
    // derive_child.rs` — the block was spurious. Negative-cell guard at
    // `tests/cli_gui_schema_v3_extensions.rs::derive_child_omits_meta_template_groups`.
    if matches!(name, "bundle" | "verify-bundle" | "export-wallet") {
        meta.insert(
            "template_groups".to_string(),
            serde_json::json!({
                "single_sig": single_sig_template_values(),
                "multisig":   multisig_template_values(),
            }),
        );
    }
    meta
}

/// SPEC §6.10.7-derived taproot multi-leaf template list. Identifies templates
/// that REQUIRE a separate internal key (`--taproot-internal-key`) when
/// emitting export-wallet vendor surfaces. Hardcoded here; future template
/// additions in this class should land in lockstep via Template enum +
/// `is_taproot_with_internal_key()` predicate.
fn taproot_internal_key_template_values() -> Vec<String> {
    vec!["tr-multi-a".to_string(), "tr-sortedmulti-a".to_string()]
}

/// SPEC §6.10 — hand-encoded `conditional_rules` per subcommand. Returns the
/// rules in priority-descending order per target flag (§6.10.4 first-rule-wins
/// requires this). Subcommands without rules return an empty Vec.
fn build_subcommand_conditional_rules(name: &str) -> Vec<ConditionalRule> {
    match name {
        "bundle" => bundle_conditional_rules(),
        "verify-bundle" => verify_bundle_conditional_rules(),
        "export-wallet" => export_wallet_conditional_rules(),
        "convert" => convert_conditional_rules(),
        "derive-child" => derive_child_conditional_rules(),
        _ => Vec::new(),
    }
}

fn bundle_conditional_rules() -> Vec<ConditionalRule> {
    let single_sig = single_sig_template_values();
    vec![
        // --template Required-unless-descriptor (existing GUI encoding).
        // SPEC §6.6 "Required first-class flag" framing; cmd/bundle.rs:25
        // clap-derive `required_unless_present_any`.
        ConditionalRule {
            rationale: "--template is required unless --descriptor or \
                        --descriptor-file is supplied (bundle's primary-mode \
                        selection invariant)."
                .to_string(),
            spec_ref: "SPEC §6.6 row 3 (negative form); cmd/bundle.rs clap-derive".to_string(),
            when: Predicate::Not {
                predicate: Box::new(Predicate::AnyOf {
                    predicates: vec![
                        Predicate::FlagPresent {
                            flag: "--descriptor".to_string(),
                        },
                        Predicate::FlagPresent {
                            flag: "--descriptor-file".to_string(),
                        },
                    ],
                }),
            },
            effect: Effect {
                flag: "--template".to_string(),
                visibility: VisibilityProjection::Required,
            },
        },
        // --descriptor ↔ --descriptor-file mutex (existing GUI encoding;
        // cmd/bundle.rs::mode_text::DESCRIPTOR_AND_DESCRIPTOR_FILE).
        ConditionalRule {
            rationale: "--descriptor and --descriptor-file are mutually exclusive."
                .to_string(),
            spec_ref: "SPEC §6.6 row 2 sibling; bundle.rs::mode_text::\
                       DESCRIPTOR_AND_DESCRIPTOR_FILE"
                .to_string(),
            when: Predicate::FlagPresent {
                flag: "--descriptor".to_string(),
            },
            effect: Effect {
                flag: "--descriptor-file".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        ConditionalRule {
            rationale: "--descriptor and --descriptor-file are mutually exclusive \
                        (symmetric direction)."
                .to_string(),
            spec_ref: "SPEC §6.6 row 2 sibling; bundle.rs::mode_text::\
                       DESCRIPTOR_AND_DESCRIPTOR_FILE"
                .to_string(),
            when: Predicate::FlagPresent {
                flag: "--descriptor-file".to_string(),
            },
            effect: Effect {
                flag: "--descriptor".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --passphrase ↔ --passphrase-stdin mutex (existing GUI encoding;
        // cmd/bundle.rs:51 conflicts_with).
        ConditionalRule {
            rationale: "--passphrase and --passphrase-stdin are mutually exclusive \
                        (secret-source selection)."
                .to_string(),
            spec_ref: "cmd/bundle.rs:51 clap-derive conflicts_with".to_string(),
            when: Predicate::FlagPresent {
                flag: "--passphrase".to_string(),
            },
            effect: Effect {
                flag: "--passphrase-stdin".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        ConditionalRule {
            rationale: "--passphrase and --passphrase-stdin are mutually exclusive \
                        (symmetric direction)."
                .to_string(),
            spec_ref: "cmd/bundle.rs:51 clap-derive conflicts_with".to_string(),
            when: Predicate::FlagPresent {
                flag: "--passphrase-stdin".to_string(),
            },
            effect: Effect {
                flag: "--passphrase".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --template disabled when --descriptor present (NEW v0.16.0;
        // cmd/bundle.rs::mode_text::DESCRIPTOR_AND_TEMPLATE).
        ConditionalRule {
            rationale: "--template is incompatible with --descriptor; descriptor \
                        passthrough mode supplies its own wallet structure."
                .to_string(),
            spec_ref: "SPEC §6.6 row 2; bundle.rs::mode_text::DESCRIPTOR_AND_TEMPLATE"
                .to_string(),
            when: Predicate::FlagPresent {
                flag: "--descriptor".to_string(),
            },
            effect: Effect {
                flag: "--template".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --threshold disabled when --descriptor present (NEW v0.16.0;
        // priority-1 of two --threshold rules — more-specific predicate).
        ConditionalRule {
            rationale: "--threshold is incompatible with --descriptor; descriptor \
                        encodes its own threshold."
                .to_string(),
            spec_ref: "bundle.rs::mode_text::DESCRIPTOR_WITH_THRESHOLD".to_string(),
            when: Predicate::FlagPresent {
                flag: "--descriptor".to_string(),
            },
            effect: Effect {
                flag: "--threshold".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --threshold disabled when template is single-sig (NEW v0.16.0;
        // priority-2 of two --threshold rules — less-specific predicate).
        ConditionalRule {
            rationale: "--threshold is meaningful only with a multisig --template; \
                        single-sig templates ignore threshold."
                .to_string(),
            spec_ref: "SPEC §6.6 row T1; bundle.rs::mode_text::THRESHOLD_WITHOUT_MULTISIG"
                .to_string(),
            when: Predicate::DropdownValueIn {
                flag: "--template".to_string(),
                values: single_sig.clone(),
            },
            effect: Effect {
                flag: "--threshold".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --multisig-path-family disabled when --descriptor present (NEW v0.16.0;
        // priority-1 of two --multisig-path-family rules).
        ConditionalRule {
            rationale: "--multisig-path-family is incompatible with --descriptor; \
                        descriptor encodes paths directly via @i/path syntax."
                .to_string(),
            spec_ref: "bundle.rs::mode_text::DESCRIPTOR_WITH_PATH_FAMILY".to_string(),
            when: Predicate::FlagPresent {
                flag: "--descriptor".to_string(),
            },
            effect: Effect {
                flag: "--multisig-path-family".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --multisig-path-family disabled when template is single-sig (NEW v0.16.0;
        // priority-2 of two rules).
        ConditionalRule {
            rationale: "--multisig-path-family is meaningful only with a multisig \
                        --template; single-sig templates ignore it."
                .to_string(),
            spec_ref: "SPEC §6.6 row T2; bundle.rs::mode_text::PATH_FAMILY_WITHOUT_MULTISIG"
                .to_string(),
            when: Predicate::DropdownValueIn {
                flag: "--template".to_string(),
                values: single_sig,
            },
            effect: Effect {
                flag: "--multisig-path-family".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --account pinned to 0 when --descriptor present (NEW v0.17.0;
        // SPEC §6.6 row 12 — DESCRIPTOR_WITH_NONZERO_ACCOUNT). Uses the
        // v3-cycle pin_value Effect: GUI emits `--account 0` regardless of
        // user input, coercing nonzero values to 0 per §6.10.4 emission
        // table. Closes the v1-cycle DEFERRED entry in §6.10.7.
        ConditionalRule {
            rationale: "--account is incompatible with --descriptor; descriptor \
                        encodes account in its @i/origin path. The GUI pins \
                        --account to 0 (coercing any user-typed nonzero value) \
                        rather than disabling the widget entirely, so the \
                        emitted argv is descriptor-compatible without user \
                        intervention. See SPEC §6.10.4 emission-mapping table \
                        for pin_value semantics."
                .to_string(),
            spec_ref: "SPEC §6.6 row 12; bundle.rs::mode_text::\
                       DESCRIPTOR_WITH_NONZERO_ACCOUNT"
                .to_string(),
            when: Predicate::FlagPresent {
                flag: "--descriptor".to_string(),
            },
            effect: Effect {
                flag: "--account".to_string(),
                visibility: VisibilityProjection::PinValue {
                    value: serde_json::json!(0),
                },
            },
        },
        // v0.18.1 — rows 10 + 11 disable_options emission REMOVED.
        // v0.18.0 introduced two rules: slot_count_gte: 2 → disable
        // single-sig --template options (row 10); slot_count_eq: 1 →
        // disable multisig --template options (row 11). Row 11 was a
        // design flaw: slot_count==1 is the natural transient state
        // when building UP to multisig (user adds slots one at a time),
        // and disabling multisig at that transient state prevents the
        // user from selecting their intended template before completing
        // the slot setup. Symmetry made row 10 face the same UX
        // friction during multisig→single-sig template switches.
        // v0.18.1 + GUI v0.7.2 replace both with a GUI-internal warning
        // banner (Option A pattern, like row 8 contiguity check):
        // dropdown renders all options normally; an inline warning fires
        // when the chosen --template + slot_count combination is
        // invalid, with text suggesting which side to adjust. CLI rows
        // 10 + 11 remain the authoritative gate per §6.6.
        //
        // VisibilityProjection::DisableOptions remains a valid v4
        // grammar variant; just unused by any rule until a future cycle
        // identifies a context where greying dropdown options serves UX
        // better than an inline warning.
    ]
}

fn verify_bundle_conditional_rules() -> Vec<ConditionalRule> {
    let single_sig = single_sig_template_values();
    vec![
        // --template Required-unless-descriptor (existing GUI encoding).
        ConditionalRule {
            rationale: "--template is required unless --descriptor or \
                        --descriptor-file is supplied."
                .to_string(),
            spec_ref: "SPEC §6.7; cmd/verify_bundle.rs clap-derive".to_string(),
            when: Predicate::Not {
                predicate: Box::new(Predicate::AnyOf {
                    predicates: vec![
                        Predicate::FlagPresent {
                            flag: "--descriptor".to_string(),
                        },
                        Predicate::FlagPresent {
                            flag: "--descriptor-file".to_string(),
                        },
                    ],
                }),
            },
            effect: Effect {
                flag: "--template".to_string(),
                visibility: VisibilityProjection::Required,
            },
        },
        // --descriptor ↔ --descriptor-file mutex (existing).
        ConditionalRule {
            rationale: "--descriptor and --descriptor-file are mutually exclusive."
                .to_string(),
            spec_ref: "cmd/verify_bundle.rs clap-derive conflicts_with".to_string(),
            when: Predicate::FlagPresent {
                flag: "--descriptor".to_string(),
            },
            effect: Effect {
                flag: "--descriptor-file".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        ConditionalRule {
            rationale: "--descriptor and --descriptor-file are mutually exclusive \
                        (symmetric direction)."
                .to_string(),
            spec_ref: "cmd/verify_bundle.rs clap-derive conflicts_with".to_string(),
            when: Predicate::FlagPresent {
                flag: "--descriptor-file".to_string(),
            },
            effect: Effect {
                flag: "--descriptor".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --bundle-json XOR (--ms1, --mk1, --md1) (existing GUI encoding).
        ConditionalRule {
            rationale: "--bundle-json is mutually exclusive with the explicit \
                        --ms1/--mk1/--md1 triplet; supplies the same data via \
                        JSON envelope."
                .to_string(),
            spec_ref: "SPEC §6.7 v0.4.3 amendment; cmd/verify_bundle.rs:67 \
                       conflicts_with_all"
                .to_string(),
            when: Predicate::FlagPresent {
                flag: "--bundle-json".to_string(),
            },
            effect: Effect {
                flag: "--ms1".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        ConditionalRule {
            rationale: "--bundle-json is mutually exclusive with --mk1."
                .to_string(),
            spec_ref: "SPEC §6.7 v0.4.3 amendment; cmd/verify_bundle.rs:67 \
                       conflicts_with_all"
                .to_string(),
            when: Predicate::FlagPresent {
                flag: "--bundle-json".to_string(),
            },
            effect: Effect {
                flag: "--mk1".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        ConditionalRule {
            rationale: "--bundle-json is mutually exclusive with --md1."
                .to_string(),
            spec_ref: "SPEC §6.7 v0.4.3 amendment; cmd/verify_bundle.rs:67 \
                       conflicts_with_all"
                .to_string(),
            when: Predicate::FlagPresent {
                flag: "--bundle-json".to_string(),
            },
            effect: Effect {
                flag: "--md1".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --passphrase ↔ --passphrase-stdin mutex (existing).
        ConditionalRule {
            rationale: "--passphrase and --passphrase-stdin are mutually exclusive."
                .to_string(),
            spec_ref: "cmd/verify_bundle.rs:51 clap-derive conflicts_with".to_string(),
            when: Predicate::FlagPresent {
                flag: "--passphrase".to_string(),
            },
            effect: Effect {
                flag: "--passphrase-stdin".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        ConditionalRule {
            rationale: "--passphrase and --passphrase-stdin are mutually exclusive \
                        (symmetric direction)."
                .to_string(),
            spec_ref: "cmd/verify_bundle.rs:51 clap-derive conflicts_with".to_string(),
            when: Predicate::FlagPresent {
                flag: "--passphrase-stdin".to_string(),
            },
            effect: Effect {
                flag: "--passphrase".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --threshold disabled when template is single-sig (NEW v0.16.0).
        ConditionalRule {
            rationale: "--threshold is meaningful only with a multisig --template."
                .to_string(),
            spec_ref: "SPEC §6.6 row T1 (mirror for verify-bundle)".to_string(),
            when: Predicate::DropdownValueIn {
                flag: "--template".to_string(),
                values: single_sig,
            },
            effect: Effect {
                flag: "--threshold".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --template disabled when --descriptor present (NEW v0.16.0).
        ConditionalRule {
            rationale: "--template is incompatible with --descriptor."
                .to_string(),
            spec_ref: "SPEC §6.6 row 2 (mirror); cmd/verify_bundle.rs \
                       conflicts_with"
                .to_string(),
            when: Predicate::FlagPresent {
                flag: "--descriptor".to_string(),
            },
            effect: Effect {
                flag: "--template".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
    ]
}

fn export_wallet_conditional_rules() -> Vec<ConditionalRule> {
    let single_sig = single_sig_template_values();
    let tr_internal_key = taproot_internal_key_template_values();
    vec![
        // --template ↔ --descriptor mutex (existing GUI encoding).
        ConditionalRule {
            rationale: "--template and --descriptor are mutually exclusive in \
                        export-wallet (mirrors bundle)."
                .to_string(),
            spec_ref: "cmd/export_wallet.rs clap-derive conflicts_with".to_string(),
            when: Predicate::FlagPresent {
                flag: "--template".to_string(),
            },
            effect: Effect {
                flag: "--descriptor".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        ConditionalRule {
            rationale: "--template and --descriptor are mutually exclusive \
                        (symmetric direction)."
                .to_string(),
            spec_ref: "cmd/export_wallet.rs clap-derive conflicts_with".to_string(),
            when: Predicate::FlagPresent {
                flag: "--descriptor".to_string(),
            },
            effect: Effect {
                flag: "--template".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --taproot-internal-key required when template ∈ tr-{multi-a, sortedmulti-a}
        // (NEW v0.16.0).
        ConditionalRule {
            rationale: "--taproot-internal-key is required for taproot multi-leaf \
                        templates (tr-multi-a / tr-sortedmulti-a)."
                .to_string(),
            spec_ref: "cmd/export_wallet.rs clap-derive required_if_eq_any".to_string(),
            when: Predicate::DropdownValueIn {
                flag: "--template".to_string(),
                values: tr_internal_key.clone(),
            },
            effect: Effect {
                flag: "--taproot-internal-key".to_string(),
                visibility: VisibilityProjection::Required,
            },
        },
        // --taproot-internal-key disabled when template ∉ tr-{multi-a, sortedmulti-a}
        // (NEW v0.16.0).
        ConditionalRule {
            rationale: "--taproot-internal-key is meaningful only for taproot \
                        multi-leaf templates."
                .to_string(),
            spec_ref: "cmd/export_wallet.rs clap-derive".to_string(),
            when: Predicate::Not {
                predicate: Box::new(Predicate::DropdownValueIn {
                    flag: "--template".to_string(),
                    values: tr_internal_key,
                }),
            },
            effect: Effect {
                flag: "--taproot-internal-key".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --threshold disabled when template is single-sig (NEW v0.16.0).
        ConditionalRule {
            rationale: "--threshold is meaningful only with a multisig --template."
                .to_string(),
            spec_ref: "SPEC §6.6 row T1 (mirror for export-wallet)".to_string(),
            when: Predicate::DropdownValueIn {
                flag: "--template".to_string(),
                values: single_sig.clone(),
            },
            effect: Effect {
                flag: "--threshold".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --multisig-path-family disabled when template is single-sig (NEW v0.16.0).
        ConditionalRule {
            rationale: "--multisig-path-family is meaningful only with a multisig \
                        --template."
                .to_string(),
            spec_ref: "SPEC §6.6 row T2 (mirror for export-wallet)".to_string(),
            when: Predicate::DropdownValueIn {
                flag: "--template".to_string(),
                values: single_sig,
            },
            effect: Effect {
                flag: "--multisig-path-family".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
    ]
}

fn convert_conditional_rules() -> Vec<ConditionalRule> {
    vec![
        // --passphrase ↔ --passphrase-stdin mutex (existing).
        ConditionalRule {
            rationale: "--passphrase and --passphrase-stdin are mutually exclusive."
                .to_string(),
            spec_ref: "cmd/convert.rs clap-derive conflicts_with".to_string(),
            when: Predicate::FlagPresent {
                flag: "--passphrase".to_string(),
            },
            effect: Effect {
                flag: "--passphrase-stdin".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        ConditionalRule {
            rationale: "--passphrase and --passphrase-stdin are mutually exclusive \
                        (symmetric direction)."
                .to_string(),
            spec_ref: "cmd/convert.rs clap-derive conflicts_with".to_string(),
            when: Predicate::FlagPresent {
                flag: "--passphrase-stdin".to_string(),
            },
            effect: Effect {
                flag: "--passphrase".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --bip38-passphrase ↔ --bip38-passphrase-stdin mutex (existing).
        ConditionalRule {
            rationale: "--bip38-passphrase and --bip38-passphrase-stdin are \
                        mutually exclusive."
                .to_string(),
            spec_ref: "cmd/convert.rs clap-derive conflicts_with".to_string(),
            when: Predicate::FlagPresent {
                flag: "--bip38-passphrase".to_string(),
            },
            effect: Effect {
                flag: "--bip38-passphrase-stdin".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        ConditionalRule {
            rationale: "--bip38-passphrase and --bip38-passphrase-stdin are \
                        mutually exclusive (symmetric direction)."
                .to_string(),
            spec_ref: "cmd/convert.rs clap-derive conflicts_with".to_string(),
            when: Predicate::FlagPresent {
                flag: "--bip38-passphrase-stdin".to_string(),
            },
            effect: Effect {
                flag: "--bip38-passphrase".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
    ]
}

fn derive_child_conditional_rules() -> Vec<ConditionalRule> {
    vec![
        // --passphrase ↔ --passphrase-stdin mutex (existing).
        ConditionalRule {
            rationale: "--passphrase and --passphrase-stdin are mutually exclusive."
                .to_string(),
            spec_ref: "cmd/derive_child.rs clap-derive conflicts_with".to_string(),
            when: Predicate::FlagPresent {
                flag: "--passphrase".to_string(),
            },
            effect: Effect {
                flag: "--passphrase-stdin".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        ConditionalRule {
            rationale: "--passphrase and --passphrase-stdin are mutually exclusive \
                        (symmetric direction)."
                .to_string(),
            spec_ref: "cmd/derive_child.rs clap-derive conflicts_with".to_string(),
            when: Predicate::FlagPresent {
                flag: "--passphrase-stdin".to_string(),
            },
            effect: Effect {
                flag: "--passphrase".to_string(),
                visibility: VisibilityProjection::Disabled,
            },
        },
        // --dice-sides required when --application == "dice" (NEW v0.16.0).
        ConditionalRule {
            rationale: "--dice-sides is required when --application is set to dice."
                .to_string(),
            spec_ref: "cmd/derive_child.rs clap-derive required_if_eq".to_string(),
            when: Predicate::DropdownValueIn {
                flag: "--application".to_string(),
                values: vec!["dice".to_string()],
            },
            effect: Effect {
                flag: "--dice-sides".to_string(),
                visibility: VisibilityProjection::Required,
            },
        },
    ]
}

/// Build the SPEC §7 schema from a clap `Command` tree.
///
/// Walks `cmd.get_subcommands()` and, for each subcommand, partitions its
/// arguments into named flags and positionals. The `gui-schema` subcommand
/// is filtered out (self-reference suppression).
///
/// Nested-subcommand flattening (v0.13.0 P2.1): when a subcommand `S` is
/// itself a `#[command(subcommand)]` parent (i.e. its own
/// `get_subcommands()` returns non-empty entries after filtering the
/// auto-generated `help`), its nested sub-subcommands are emitted as
/// hyphenated entries (`S-sub_sub`) IN PLACE OF `S`. This repairs the
/// pre-existing v0.12.0 seed-xor empty-flags rendering (where the
/// per-sub-sub flag tables were invisible to `mnemonic-gui`) and
/// generalizes to v0.13.0 slip39 + any future nested-subcommand parent.
/// Schema `version` stays at 1 — the change is additive at the name set.
fn build_schema(cmd: &Command) -> Schema {
    // v0.24.0 Tranche B.1 — collect parent-Command global args (clap-derive
    // `global = true`) once at the root, then propagate them into every
    // subcommand's flag set with `global: true`. clap's own
    // `_propagate_global_args` populates subcommand args at build-time, but
    // we walk that explicit list here so the emitter does not depend on
    // clap's lazy-build state and so nested-subcommand parents (seed-xor /
    // slip39) consistently see the global flag at the flattened
    // `S-sub_sub` level. Help (`--help`) is excluded as before.
    let global_args: Vec<&clap::Arg> = cmd
        .get_arguments()
        .filter(|a| a.is_global_set() && a.get_id().as_str() != "help")
        .collect();

    let mut subs: Vec<Subcommand> = Vec::new();
    for s in cmd
        .get_subcommands()
        .filter(|s| s.get_name() != "gui-schema" && s.get_name() != "help")
    {
        let nested: Vec<&Command> = s
            .get_subcommands()
            .filter(|ss| ss.get_name() != "help")
            .collect();
        if nested.is_empty() {
            subs.push(build_subcommand(s, &global_args));
        } else {
            for ss in nested {
                let flat = build_subcommand(ss, &global_args);
                let flat_name = format!("{}-{}", s.get_name(), ss.get_name());
                let conditional_rules = build_subcommand_conditional_rules(&flat_name);
                let meta = build_subcommand_meta(&flat_name);
                subs.push(Subcommand {
                    name: flat_name,
                    flags: flat.flags,
                    positionals: flat.positionals,
                    conditional_rules,
                    meta,
                });
            }
        }
    }

    // Deterministic ordering by subcommand name (stable across clap versions).
    subs.sort_by(|a, b| a.name.cmp(&b.name));

    Schema {
        // v0.24.0 Tranche B.1 — bump v4 → v5 for the additive Flag fields
        // {default_value, global, secret}. v4 consumers will still parse v5
        // documents (additive fields tolerate unknown-field-skipping serde),
        // but cannot consume the new fields; the version bump signals the
        // schema change so v4-only consumers can refuse the upgrade
        // explicitly. Earlier bumps: v1→v2 v0.16.0 (conditional_rules);
        // v2→v3 v0.17.0 (slot_count_* + pin_value + meta.template_groups);
        // v3→v4 v0.18.0 (disable_options Visibility variant).
        version: 5,
        cli: "mnemonic".to_string(),
        subcommands: subs,
    }
}

fn build_subcommand(sub: &Command, global_args: &[&clap::Arg]) -> Subcommand {
    let mut flags: Vec<Flag> = Vec::new();
    let mut positionals: Vec<Positional> = Vec::new();
    // IDs of the global args we'll propagate explicitly. Used to recognize
    // clap-propagated copies in `sub.get_arguments()` and re-route them
    // through the same emit_flag helper (so they pick up `global: true`).
    let global_ids: std::collections::BTreeSet<String> = global_args
        .iter()
        .map(|a| a.get_id().as_str().to_string())
        .collect();

    // v0.25.0 Phase 3 — collect the IDs of args clap reports as LOCAL on this
    // subcommand (excluding positionals, the auto-generated --help flag, and
    // anything clap has copied in via `_propagate_global_args` — those are
    // recognized via `is_global_set()`). The disjointness check below ensures
    // no future global-flag addition silently shadows a subcommand-local
    // flag with the same clap-derive identifier. Crucially we do NOT filter
    // local_ids by `global_ids` here: doing so would make the assertion
    // vacuously true. See `assert_global_local_id_disjointness`.
    let local_ids: std::collections::BTreeSet<String> = sub
        .get_arguments()
        .filter(|a| !a.is_positional())
        .filter(|a| a.get_id().as_str() != "help")
        .filter(|a| !a.is_global_set())
        .map(|a| a.get_id().as_str().to_string())
        .collect();
    assert_global_local_id_disjointness(&global_ids, &local_ids, sub.get_name());

    for arg in sub.get_arguments() {
        if arg.is_positional() {
            positionals.push(Positional {
                name: arg.get_id().to_string(),
                required: arg.is_required_set(),
                repeating: matches!(
                    arg.get_action(),
                    clap::ArgAction::Append | clap::ArgAction::Count
                ) || arg.get_num_args().is_some_and(|n| n.max_values() > 1),
            });
        } else {
            // Skip the auto-generated --help flag; it's not user surface.
            if arg.get_id().as_str() == "help" {
                continue;
            }
            // If clap already propagated a global into this subcommand,
            // skip here and let the global-args pass below emit it once
            // with `global: true`. Avoids double-listing when both code
            // paths see the same arg.
            if global_ids.contains(arg.get_id().as_str()) {
                continue;
            }
            let flag = emit_flag(arg, /*is_global=*/ false);
            flags.push(flag);
        }
    }

    // v0.24.0 Tranche B.1 — append parent-Command globals into this
    // subcommand's flag set. Each is marked `global: true` so GUI
    // consumers can render it under a top-level action-bar or repeat
    // it per-subcommand as they prefer. v0.25.0 Phase 3 — the
    // `assert_global_local_id_disjointness` invariant above guarantees
    // these IDs do not collide with the local emissions, so we can append
    // unconditionally (the pre-v0.25.0 `seen_flag_names` long-name dedup
    // was dead-defense per v0.24.0 B.1 architect review).
    for ga in global_args {
        flags.push(emit_flag(ga, /*is_global=*/ true));
    }

    // Deterministic ordering: flags by long name, positionals by declaration order.
    flags.sort_by(|a, b| a.name.cmp(&b.name));

    let conditional_rules = build_subcommand_conditional_rules(sub.get_name());
    let meta = build_subcommand_meta(sub.get_name());

    Subcommand {
        name: sub.get_name().to_string(),
        flags,
        positionals,
        conditional_rules,
        meta,
    }
}

/// v0.25.0 Phase 3 — debug-only invariant guard for the gui-schema
/// global-vs-local flag-ID partition.
///
/// `global_ids` are the clap-derive identifiers of every arg the root
/// `Cli` declares with `global = true` (currently just `--no-auto-repair`).
/// `local_ids` are the identifiers of args declared directly on a
/// subcommand AFTER filtering out positionals, the auto-generated `help`
/// flag, and anything clap copied in via `_propagate_global_args`
/// (recognized via `is_global_set()`). `local_ids` is intentionally NOT
/// pre-filtered against `global_ids` — doing so would make this
/// disjointness check vacuously true.
///
/// The invariant: the two sets are disjoint. If a future global addition
/// re-uses a subcommand-local identifier, the emitter would silently double-
/// emit (one entry with `global: true`, one without) — the GUI's schema
/// consumer would then surface a flag-duplication bug somewhere downstream.
/// The debug-assert catches this at toolkit-build time before the schema
/// ever reaches a GUI consumer.
///
/// This is `pub(crate)` for direct unit testability (see
/// `cli_gui_schema_v5_extensions.rs`'s negative-control cell) — driving the
/// invariant through a synthetic `clap::Command` tree would require leaking
/// emitter internals into tests; calling the helper directly is cleaner.
///
/// Resolves FOLLOWUP `gui-schema-global-flag-id-disjointness-debug-assert`
/// (filed v0.24.0 Tranche B.1 architect review; the pre-v0.25.0
/// `seen_flag_names` long-name dedup in `build_subcommand` was a dead-
/// defense that this debug-assert promotes to a load-bearing invariant).
pub(crate) fn assert_global_local_id_disjointness(
    global_ids: &std::collections::BTreeSet<String>,
    local_ids: &std::collections::BTreeSet<String>,
    subcmd_name: &str,
) {
    debug_assert!(
        global_ids.is_disjoint(local_ids),
        "global flag id collides with local id in subcommand `{}`: shared ids = {:?}",
        subcmd_name,
        global_ids.intersection(local_ids).collect::<Vec<_>>()
    );
}

/// v0.24.0 Tranche B.1 — assemble a `Flag` entry from a clap `Arg`, populating
/// the schema v5 `default_value` / `global` / `secret` fields.
///
/// `is_global` is supplied by the caller because the same arg may be emitted
/// from two paths (the subcommand's own arg list, or the propagated root-args
/// list); the caller knows which path applies.
fn emit_flag(arg: &clap::Arg, is_global: bool) -> Flag {
    let name = arg
        .get_long()
        .map(|l| format!("--{l}"))
        .unwrap_or_else(|| arg.get_id().to_string());
    let (kind, choices) = classify_kind(arg);
    let default_value = extract_default_value(arg, &kind);
    let secret = mnemonic_toolkit::secrets::flag_is_secret(&name);
    Flag {
        name,
        required: arg.is_required_set(),
        kind,
        choices,
        default_value,
        global: is_global,
        secret,
    }
}

/// v0.24.0 Tranche B.1 — extract a clap-derive `default_value` into a JSON
/// value whose shape matches the flag's `kind`.
///
/// Per the SPEC §7 v5 contract:
/// - `boolean` → no default emitted (a bool flag's absence is its default).
/// - `number`  → `Value::Number(i64)` parsed from the default string. Non-
///               parseable defaults fall through to string preservation.
/// - `text` / `path` / `dropdown` → `Value::String(s)`.
/// - empty / absent defaults → `None` (field omitted from JSON).
///
/// Per the kickoff hard constraint: if `get_default_values()` returns a
/// shape we cannot serialize cleanly (multi-value default for a single-value
/// flag, non-UTF-8 OsStr), we fall back to the first UTF-8 entry stringified;
/// the field's purpose is GUI seeding, not roundtrip-faithful re-emission.
fn extract_default_value(arg: &clap::Arg, kind: &str) -> Option<serde_json::Value> {
    if kind == "boolean" {
        // Booleans default to false; no need to emit the trivial default.
        return None;
    }
    let defaults = arg.get_default_values();
    if defaults.is_empty() {
        return None;
    }
    // Single-value flags: take the first default. Multi-value defaults
    // (`default_values(["a", "b"])`) are uncommon in this codebase; we
    // collapse to the first entry to keep the wire shape predictable.
    let first = defaults.first()?.to_str()?;
    match kind {
        "number" => {
            // Try i64 first (covers all signed/unsigned defaults in current
            // clap-derive sites — `--account = "0"`, `--iteration-exponent`
            // = "0"`, `--bitcoin-core-version = "25"`). Fall through to
            // string preservation if the parse fails (defensive — current
            // codebase has no float defaults).
            if let Ok(n) = first.parse::<i64>() {
                Some(serde_json::Value::Number(n.into()))
            } else {
                Some(serde_json::Value::String(first.to_string()))
            }
        }
        _ => Some(serde_json::Value::String(first.to_string())),
    }
}

/// Map a clap `Arg` to the SPEC §7 `kind` enum.
///
/// Order matters:
/// 1. boolean (clap `SetTrue` / `SetFalse`) wins before value-parser inspection
///    because flag args have a hidden bool value_parser.
/// 2. `PossibleValuesParser` (or any value_parser exposing `possible_values()`)
///    → dropdown with the enumerated choices.
/// 3. numeric `ValueParser::type_id()` match → `number`.
/// 4. `PathBuf` parser → `path`.
/// 5. fallthrough → `text`.
fn classify_kind(arg: &clap::Arg) -> (String, Option<Vec<String>>) {
    use std::any::TypeId;

    // (1) boolean flag — clap encodes these as ArgAction::SetTrue / SetFalse.
    if matches!(
        arg.get_action(),
        clap::ArgAction::SetTrue | clap::ArgAction::SetFalse
    ) {
        return ("boolean".to_string(), None);
    }

    // (2) dropdown via PossibleValuesParser (used by `#[arg(value_enum)]` and
    // by hand-built PossibleValuesParser arms). `possible_values()` returns
    // `Some(_)` iff the parser is enumeration-bounded.
    let parser = arg.get_value_parser();
    if let Some(pvs) = parser.possible_values() {
        let choices: Vec<String> = pvs.map(|v| v.get_name().to_string()).collect();
        if !choices.is_empty() {
            return ("dropdown".to_string(), Some(choices));
        }
    }

    // (3) numeric: `ValueParser::type_id()` returns an `AnyValueId` that
    // implements `PartialEq<std::any::TypeId>`, so we can match against
    // the std numeric primitives directly.
    let tid = parser.type_id();
    let is_numeric = tid == TypeId::of::<u8>()
        || tid == TypeId::of::<u16>()
        || tid == TypeId::of::<u32>()
        || tid == TypeId::of::<u64>()
        || tid == TypeId::of::<u128>()
        || tid == TypeId::of::<i8>()
        || tid == TypeId::of::<i16>()
        || tid == TypeId::of::<i32>()
        || tid == TypeId::of::<i64>()
        || tid == TypeId::of::<i128>()
        || tid == TypeId::of::<usize>()
        || tid == TypeId::of::<isize>()
        || tid == TypeId::of::<f32>()
        || tid == TypeId::of::<f64>();
    if is_numeric {
        return ("number".to_string(), None);
    }

    // (4) path-like — `PathBuf` is one of the four built-in ValueParserInner
    // variants. We match on type_id rather than the Debug string for stability.
    if tid == TypeId::of::<std::path::PathBuf>() {
        return ("path".to_string(), None);
    }

    // (5) fallthrough — String / custom value_parsers (FromInput, ToInput,
    // SlotInput, XpubPrefix, ...) / complex GUI variants. The GUI re-parses
    // these client-side per the SPEC §7 lossy-mapping contract.
    ("text".to_string(), None)
}

/// Emit the SPEC §7 schema for the supplied clap `Command` tree to `stdout`
/// as a single JSON line (no trailing newline, matching `--json` envelope
/// conventions elsewhere in the toolkit).
pub fn run<W: Write>(
    args: &GuiSchemaArgs,
    root: &Command,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    // v0.20.0 F2 — `--classify-descriptor` diagnostic surface. Per
    // `bundle.rs:931-933` precedent: probe-parse with empty keys/fingerprints
    // (only the tree is consulted), then test `canonical_origin(&tree).is_some()`.
    if let Some(input) = args.classify_descriptor.as_deref() {
        let desc = crate::parse_descriptor::parse_descriptor(input, &[], &[])?;
        let verdict = if md_codec::canonical_origin::canonical_origin(&desc.tree).is_some() {
            "canonical"
        } else {
            "non-canonical"
        };
        writeln!(stdout, "{verdict}").ok();
        return Ok(());
    }

    let schema = build_schema(root);
    // Schema is a closed type tree with no untrusted input; serialization is
    // infallible in practice. Match the `.ok()` pattern used by `bundle --json`
    // / `verify-bundle --json` / `convert --json`.
    serde_json::to_writer(&mut *stdout, &schema).ok();
    writeln!(stdout).ok();
    Ok(())
}

// v0.25.0 Phase 3 — unit-test side of the `assert_global_local_id_disjointness`
// invariant. The companion positive-invariant cell
// `global_local_id_disjointness_invariant_holds_in_current_schema` lives in
// the integration suite at
// `crates/mnemonic-toolkit/tests/cli_gui_schema_v5_extensions.rs` (it asserts
// the end-to-end wire shape). This negative-control cell calls the
// `pub(crate)` helper directly, which is not reachable from the integration
// test crate.
#[cfg(test)]
mod tests {
    #[cfg(debug_assertions)]
    use super::assert_global_local_id_disjointness;

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "collides with local id")]
    fn global_local_collision_triggers_debug_assert() {
        // Construct a synthetic global-vs-local ID partition with a deliberate
        // collision (`my_test_flag` declared as both global and local) and
        // assert that `assert_global_local_id_disjointness`'s `debug_assert!`
        // fires with the expected message substring.
        //
        // Gated to `#[cfg(debug_assertions)]` builds: `debug_assert!()`
        // compiles out in release builds, so the helper becomes a no-op there
        // and `#[should_panic]` would record a false-positive failure
        // (`test did not panic`) on `cargo test --release`. The positive-
        // invariant integration cell runs in both debug and release; this
        // unit cell is debug-only by design.
        //
        // Driving the invariant through a synthetic `clap::Command` tree
        // would require leaking emitter internals; calling the `pub(crate)`
        // helper directly is the cleaner pattern (architect-review-approved
        // alternative path per the Phase 3 kickoff).
        let mut global_ids: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        global_ids.insert("my_test_flag".to_string());
        let mut local_ids: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        local_ids.insert("my_test_flag".to_string());
        assert_global_local_id_disjointness(&global_ids, &local_ids, "synthetic-subcmd");
    }
}
