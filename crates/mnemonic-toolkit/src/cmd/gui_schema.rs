//! `mnemonic gui-schema` subcommand — emit SPEC §7 GUI-overlay schema JSON.
//!
//! Companion to the `mnemonic-gui` v0.2 Phase C.2 contract
//! (`bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `mnemonic-gui-schema-mirror`).
//!
//! Walks the clap `Command` tree via `clap::CommandFactory` and serializes a
//! machine-readable schema of every existing subcommand's flag surface to
//! stdout as JSON. The GUI consumes this schema to render forms; on
//! `version != 1` the GUI refuses to launch.
//!
//! ## SPEC §7 contract
//!
//! ```json
//! {
//!   "version": 1,
//!   "cli": "mnemonic",
//!   "subcommands": [
//!     {
//!       "name": "bundle",
//!       "flags":       [ { "name", "required", "kind", "choices": [..] | null } ],
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
pub struct GuiSchemaArgs {}

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
        // --template disable single-sig options when slot_count >= 2 (NEW
        // v0.18.0; SPEC §6.6 row 10 — SINGLE_SIG_TEMPLATE_WITH_MULTISIG_SLOTS).
        // Uses the v4-cycle disable_options Effect: GUI greys out single-sig
        // values in the --template Dropdown so the user can't select e.g.
        // bip84 with 3 slots. Schema-time only (no argv-emission impact per
        // §6.10.4); CLI row 10 is the residual surface. Closes v2-cycle
        // DEFERRED entry in §6.10.7 (Effect side of the rows-9/10/11 trio).
        ConditionalRule {
            rationale: "Single-sig --template values (bip44/49/84/86) require \
                        exactly one slot. When the user has supplied 2+ slots, \
                        single-sig template options are invalid — the GUI greys \
                        them out in the Dropdown so the user can't select one. \
                        Schema-time only: argv emission is unaffected (stale \
                        pre-disabled values still emit; CLI row 10 catches them \
                        at run time). See SPEC §6.10.4 emission-mapping table \
                        for disable_options semantics."
                .to_string(),
            spec_ref: "SPEC §6.6 row 10; bundle.rs::mode_text::\
                       SINGLE_SIG_TEMPLATE_WITH_MULTISIG_SLOTS"
                .to_string(),
            when: Predicate::SlotCountGte { value: 2 },
            effect: Effect {
                flag: "--template".to_string(),
                visibility: VisibilityProjection::DisableOptions {
                    values: single_sig_template_values(),
                },
            },
        },
        // --template disable multisig options when slot_count == 1 (NEW
        // v0.18.0; SPEC §6.6 row 11 — MULTISIG_TEMPLATE_WITH_SINGLE_SLOT).
        // Mirror of row 10; greys out multisig values when only 1 slot is
        // present. Schema-time only per §6.10.4.
        ConditionalRule {
            rationale: "Multisig --template values (wsh-multi/sortedmulti, \
                        sh-wsh-multi/sortedmulti, tr-multi-a/sortedmulti-a) \
                        require 2+ slots. When the user has supplied exactly 1 \
                        slot, multisig template options are invalid — the GUI \
                        greys them out in the Dropdown so the user can't \
                        select one. Schema-time only: argv emission is \
                        unaffected (stale pre-disabled values still emit; CLI \
                        row 11 catches them at run time). See SPEC §6.10.4 \
                        emission-mapping table for disable_options semantics."
                .to_string(),
            spec_ref: "SPEC §6.6 row 11; bundle.rs::mode_text::\
                       MULTISIG_TEMPLATE_WITH_SINGLE_SLOT"
                .to_string(),
            when: Predicate::SlotCountEq { value: 1 },
            effect: Effect {
                flag: "--template".to_string(),
                visibility: VisibilityProjection::DisableOptions {
                    values: multisig_template_values(),
                },
            },
        },
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
            subs.push(build_subcommand(s));
        } else {
            for ss in nested {
                let flat = build_subcommand(ss);
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
        version: 4,
        cli: "mnemonic".to_string(),
        subcommands: subs,
    }
}

fn build_subcommand(sub: &Command) -> Subcommand {
    let mut flags: Vec<Flag> = Vec::new();
    let mut positionals: Vec<Positional> = Vec::new();

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
            let name = arg
                .get_long()
                .map(|l| format!("--{l}"))
                .unwrap_or_else(|| arg.get_id().to_string());
            let (kind, choices) = classify_kind(arg);
            flags.push(Flag {
                name,
                required: arg.is_required_set(),
                kind,
                choices,
            });
        }
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
    _args: &GuiSchemaArgs,
    root: &Command,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    let schema = build_schema(root);
    // Schema is a closed type tree with no untrusted input; serialization is
    // infallible in practice. Match the `.ok()` pattern used by `bundle --json`
    // / `verify-bundle --json` / `convert --json`.
    serde_json::to_writer(&mut *stdout, &schema).ok();
    writeln!(stdout).ok();
    Ok(())
}
