//! `mnemonic compare-cost` engine — wsh-vs-tr per-spending-condition cost
//! comparison. SPEC: `design/SPEC_compare_cost_v0_26_0.md` (R3, canonical
//! record copy of the plan-mode artifact).
//!
//! Module decomposition (SPEC §Phase 1):
//! - [`dummy_keys`]: deterministic `DefiniteDescriptorKey` substitution for
//!   abstract labels `pk(A)`, `pk(B)`, … (SPEC §2.2).
//! - [`translate`]: label → DefiniteDescriptorKey substitution that consumes
//!   the user's miniscript string and emits parsed `Miniscript<DDK, Ctx>` for
//!   both Segwitv0 and Tap contexts, with multi↔multi_a rewriting (SPEC §2.1).
//! - [`enumerate`]: minimal-satisfying-configuration enumeration over the
//!   parsed AST (SPEC §3).
//! - [`format`]: plaintext-table renderer + JSON envelope serializer (SPEC §5).
//!
//! Entry point: [`run_compare_cost`] (called from `cmd::compare_cost::run`).

use crate::error::ToolkitError;

pub mod dummy_keys;
pub mod enumerate;
pub mod format;
pub mod strip;
pub mod translate;

/// SPEC §2.3 — BIP-341 H-point NUMS x-only key.
///
/// Phase 1 source-checked existing toolkit constants (`MS_NUMS_TARGET`,
/// `MD_NUMS_TARGET`, `NUMS_XONLY_HEX`); the literal already appears at
/// `wallet_export/bip388.rs::NUMS_XONLY_HEX`. We re-declare here to avoid
/// a tight coupling to wallet_export internals.
pub const NUMS_XONLY_HEX: &str =
    "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// Subcommand-local errors. Wrapped by [`ToolkitError::CompareCost`] for the
/// crate-wide error dispatch.
#[derive(Debug)]
pub enum CompareCostError {
    /// Input miniscript / descriptor parse failed in at least one of the
    /// script contexts. Exit 2.
    Parse(String),
    /// Input miniscript valid in one context but not the other after
    /// `multi ↔ multi_a` rewriting. Exit 3.
    ContextIncompat {
        valid_in: &'static str,
        invalid_in: &'static str,
        detail: String,
    },
    /// Descriptor wrapper not in {`wsh`, `sh(wsh)`, single-leaf `tr`}. Exit 3.
    UnsupportedWrapper(String),
    /// Multi-leaf `tr(IK, {M1, M2, …})` descriptor input. Exit 3.
    MultiLeafTr,
    /// Spending-condition power-set pre-check would exceed `--max-conditions`
    /// hard cap. Exit 3.
    ConditionsTooMany { raw: usize, cap: usize },
    /// Miniscript has zero satisfying conditions (degenerate). Exit 3.
    NoSatisfyingConditions,
}

impl std::fmt::Display for CompareCostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompareCostError::Parse(m) => write!(f, "compare-cost: parse error: {m}"),
            CompareCostError::ContextIncompat {
                valid_in,
                invalid_in,
                detail,
            } => write!(
                f,
                "compare-cost: miniscript valid in {valid_in} only; cannot wrap as {invalid_in}: {detail}"
            ),
            CompareCostError::UnsupportedWrapper(w) => write!(
                f,
                "compare-cost: unsupported wrapper '{w}'; supported wrappers: wsh(..), sh(wsh(..)), single-leaf tr(IK,{{M}})."
            ),
            CompareCostError::MultiLeafTr => write!(
                f,
                "compare-cost: multi-leaf tr() input; supply one leaf at a time via --miniscript"
            ),
            CompareCostError::ConditionsTooMany { raw, cap } => write!(
                f,
                "compare-cost: spending conditions exceed --max-conditions cap ({raw} > {cap}); raise the cap or simplify the policy"
            ),
            CompareCostError::NoSatisfyingConditions => {
                write!(f, "compare-cost: no satisfying conditions for this miniscript")
            }
        }
    }
}

impl CompareCostError {
    /// SPEC §9 exit-code mapping.
    pub fn exit_code(&self) -> u8 {
        match self {
            CompareCostError::Parse(_) => 2,
            CompareCostError::ContextIncompat { .. }
            | CompareCostError::UnsupportedWrapper(_)
            | CompareCostError::MultiLeafTr
            | CompareCostError::ConditionsTooMany { .. }
            | CompareCostError::NoSatisfyingConditions => 3,
        }
    }
}

/// Input form supplied by the CLI dispatcher.
pub enum InputForm {
    /// Bare miniscript string with abstract labels or concrete hex keys.
    Miniscript(String),
    /// Full descriptor string; wrapper is stripped to recover M before
    /// comparison (SPEC §2 + Phase 2 `strip` module).
    Descriptor(String),
}

/// Args to drive [`run_compare_cost`].
pub struct CompareCostArgs {
    pub input: InputForm,
    pub feerate_sat_per_vb: f64,
    pub max_conditions: usize,
    /// `true` → emit JSON envelope; `false` → plaintext table.
    pub json: bool,
}

/// Drive the comparison end-to-end and write either the plaintext table or
/// the JSON envelope to `stdout`. Errors propagate as `ToolkitError`.
pub fn run_compare_cost<W: std::io::Write>(
    args: &CompareCostArgs,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    // 1. Translate user input → Segwitv0 + Tap miniscripts.
    let (translated, advisory) = match &args.input {
        InputForm::Miniscript(s) => {
            let t = translate::translate_miniscript(s).map_err(ToolkitError::CompareCost)?;
            (t, None)
        }
        InputForm::Descriptor(s) => {
            strip::translate_descriptor(s).map_err(ToolkitError::CompareCost)?
        }
    };
    let original_input = match &args.input {
        InputForm::Miniscript(s) => s.as_str(),
        InputForm::Descriptor(s) => s.as_str(),
    };

    // 2. Build wsh + tr descriptors.
    let wsh_desc = translate::build_wsh_descriptor(translated.segv0.clone())
        .map_err(ToolkitError::CompareCost)?;
    let tr_desc = translate::build_tr_descriptor(translated.tap.clone())
        .map_err(ToolkitError::CompareCost)?;

    // 3. Walk AST to collect signers, preimages, timelocks; eager
    //    combinatorial precheck against --max-conditions; enumerate minimal
    //    satisfying configurations.
    let report = enumerate::enumerate_minimal_conditions(
        &translated,
        &wsh_desc,
        &tr_desc,
        args.max_conditions,
    )
    .map_err(ToolkitError::CompareCost)?;

    // 4. Render.
    let mut notes: Vec<String> = Vec::new();
    if let Some(ad) = advisory {
        notes.push(ad);
    }
    if args.feerate_sat_per_vb == 0.0 {
        notes.push("feerate is 0; sats columns will be 0".to_string());
    }
    notes.push("per-condition vbytes are rounded individually; absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct".to_string());
    if report.soft_cap_reached {
        notes.push(format!(
            "enumeration reached soft threshold; {} conditions shown",
            report.rows.len()
        ));
    }
    if translated.concrete_keys {
        notes.push("input had concrete keys; cost is identical to the abstract case".to_string());
    }
    if report.has_hash_fragments {
        notes.push(
            "input contains hash-preimage fragments; preimage-known rows are enumerated assuming the user can supply each preimage (cost only — no preimage knowledge is implied)".to_string(),
        );
    }
    // SPEC §11 (v0.28.0) + §2.3 — advisory when `tr(IK, {M})` is supplied
    // with a non-NUMS internal key. The per-condition rows still compare
    // wsh(M) vs tr(NUMS,{M}) on the script-path; the keypath-spend cost
    // appears separately (annotation line in plaintext; `keypath_spend`
    // field in JSON).
    if let Some(ik_hex) = translated.tr_non_nums_internal_key_xonly_hex.as_deref() {
        notes.push(format!(
            "input had a non-NUMS internal key IK ({ik_hex}); this report compares script-path-only cost (tr modeled as tr(NUMS, {{M}})). Keyspend-via-IK costs ~58 vB total (under SIGHASH_DEFAULT) and is the cheapest spend if signing with IK is acceptable."
        ));
    }

    // SPEC §11 (v0.28.0) — when input is `tr(IK, {M})` with non-NUMS IK,
    // surface the keypath-spend cost. P2TR keyspend witness under
    // SIGHASH_DEFAULT = 1B stack-count + 1B sig-len + 64B Schnorr = 66B;
    // total vbytes = `(164 + 66 + 3) / 4 = 58`. Cost is fixed; the IK
    // hex is the user-supplied non-NUMS internal key.
    let keypath_spend = translated
        .tr_non_nums_internal_key_xonly_hex
        .as_deref()
        .map(|ik_hex| KeypathSpend {
            internal_key_xonly_hex: ik_hex.to_string(),
            vbytes: format::witness_bytes_to_vbytes(KEYPATH_SPEND_WITNESS_BYTES),
        });

    let input_form_label = match &args.input {
        InputForm::Miniscript(_) => "miniscript",
        InputForm::Descriptor(_) => "descriptor",
    };
    if args.json {
        format::render_json(
            original_input,
            input_form_label,
            &translated.extracted,
            args.feerate_sat_per_vb,
            &report.rows,
            &notes,
            keypath_spend.as_ref(),
            stdout,
        )
        .map_err(ToolkitError::Io)?;
    } else {
        format::render_table(
            original_input,
            &translated.extracted,
            args.feerate_sat_per_vb,
            &report.rows,
            &notes,
            keypath_spend.as_ref(),
            stdout,
        )
        .map_err(ToolkitError::Io)?;
    }

    Ok(())
}

/// SPEC §11 (v0.28.0) — keypath-spend cost surfaced when input is
/// `tr(IK, {M})` with a non-NUMS internal key. The wire-shape carries
/// the IK hex (for user-visible attribution) and the computed vbytes.
pub struct KeypathSpend {
    pub internal_key_xonly_hex: String,
    pub vbytes: i64,
}

/// SPEC §11 — P2TR keyspend witness bytes under SIGHASH_DEFAULT.
/// `1 (stack-count varint) + 1 (sig-length-prefix) + 64 (Schnorr sig) = 66`.
const KEYPATH_SPEND_WITNESS_BYTES: usize = 66;
