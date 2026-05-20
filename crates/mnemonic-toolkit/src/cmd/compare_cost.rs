//! CLI dispatch for `mnemonic compare-cost`. SPEC §1.
//!
//! Phase 3 surface: `--miniscript` and `--descriptor` (mutually exclusive),
//! plus stdin fallback when neither flag is given and stdin is non-TTY.

use std::io::{IsTerminal, Read, Write};

use clap::Args;

use crate::cost::{self, CompareCostArgs as EngineArgs, InputForm};
use crate::error::ToolkitError;

#[derive(Args, Debug)]
pub struct CompareCostArgs {
    /// Bare miniscript fragment with abstract key labels (e.g.,
    /// `or_b(pk(A), pk(B))`) OR concrete hex pubkeys. Cost is key-agnostic;
    /// abstract labels are auto-substituted with deterministic dummy keys.
    /// Mutually exclusive with `--descriptor`.
    #[arg(long, conflicts_with = "descriptor")]
    pub miniscript: Option<String>,

    /// Full descriptor — `wsh(M)`, `sh(wsh(M))`, or single-leaf `tr(IK, {M})`
    /// (v0.28.0). The wrapper is stripped to recover the inner miniscript M
    /// before the comparison. Multi-leaf `tr(IK, {M1, M2, ...})` and keypath-
    /// only `tr(IK)` inputs are refused with exit 3. Mutually exclusive with
    /// `--miniscript`.
    #[arg(long)]
    pub descriptor: Option<String>,

    /// Feerate in sats per virtual byte for the sats columns. Default 1.0
    /// (so sats == vbytes when unspecified). Decimal values accepted.
    #[arg(long, default_value_t = 1.0, value_parser = parse_feerate)]
    pub feerate: f64,

    /// Hard cap on the raw enumeration size `n_abs × n_rel × 2^(|signers| +
    /// |preimages|)` — refuses pre-enumeration if exceeded (per SPEC §3.3
    /// step 1). Default 4096. When > 256, a soft warn-trail entry appears
    /// in `notes[]` at 256 rows produced.
    #[arg(long, default_value_t = 4096, value_parser = parse_max_conditions)]
    pub max_conditions: usize,

    /// Emit a JSON envelope on stdout instead of the plaintext table.
    #[arg(long)]
    pub json: bool,
}

fn parse_feerate(s: &str) -> Result<f64, String> {
    let v: f64 = s
        .parse()
        .map_err(|e: std::num::ParseFloatError| format!("invalid feerate '{s}': {e}"))?;
    if !v.is_finite() || !(0.0..=10_000.0).contains(&v) {
        return Err(format!("feerate must be in [0.0, 10000.0]; got {v}"));
    }
    Ok(v)
}

fn parse_max_conditions(s: &str) -> Result<usize, String> {
    let v: usize = s
        .parse()
        .map_err(|e: std::num::ParseIntError| format!("invalid --max-conditions '{s}': {e}"))?;
    if v == 0 {
        return Err("--max-conditions must be >= 1".to_string());
    }
    Ok(v)
}

pub fn run<R: Read, W: Write>(
    args: &CompareCostArgs,
    stdin: &mut R,
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    let input = match (&args.miniscript, &args.descriptor) {
        (Some(m), None) => InputForm::Miniscript(m.clone()),
        (None, Some(d)) => InputForm::Descriptor(d.clone()),
        (Some(_), Some(_)) => {
            // Should be unreachable: clap's `conflicts_with` rejects this at parse-time.
            return Err(ToolkitError::BadInput(
                "supply exactly one of --miniscript or --descriptor".to_string(),
            ));
        }
        (None, None) => {
            // Phase 3: stdin fallback when piped. TTY case → clear error.
            if std::io::stdin().is_terminal() {
                return Err(ToolkitError::BadInput(
                    "compare-cost: no input; supply --miniscript <STR> or --descriptor <STR>"
                        .to_string(),
                ));
            }
            classify_stdin_input(stdin)?
        }
    };
    let engine_args = EngineArgs {
        input,
        feerate_sat_per_vb: args.feerate,
        max_conditions: args.max_conditions,
        json: args.json,
    };
    cost::run_compare_cost(&engine_args, stdout)
}

/// Read the first non-blank line from stdin and classify it as either a
/// bare miniscript (no top-level wrapper) or a full descriptor (starts
/// with `wsh(`, `sh(`, `tr(`, etc.). Heuristic: if the trimmed first line
/// has a top-level identifier prefix in the set `{wsh, sh, tr, wpkh, pkh,
/// combo, addr, rawtr, raw}`, treat as a descriptor; otherwise treat as a
/// bare miniscript. Note: `pk` is INTENTIONALLY OMITTED from this set —
/// `pk(K)` is valid both as a descriptor (BIP-380 `Descriptor::Bare`) and
/// as a miniscript fragment; routing it through the miniscript path
/// handles both interpretations correctly.
fn classify_stdin_input<R: Read>(stdin: &mut R) -> Result<InputForm, ToolkitError> {
    let mut buf = String::new();
    stdin
        .read_to_string(&mut buf)
        .map_err(|e| ToolkitError::BadInput(format!("compare-cost: stdin read: {e}")))?;
    let trimmed = buf.trim();
    let first_line = trimmed.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let first_line = first_line.trim();
    if first_line.is_empty() {
        return Err(ToolkitError::BadInput(
            "compare-cost: stdin is empty; supply a miniscript or descriptor".to_string(),
        ));
    }
    // Top-level identifier extraction: the run of identifier chars before
    // the first `(`. Compare against the descriptor-only prefix set.
    let head: String = first_line
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    const DESCRIPTOR_PREFIXES: &[&str] = &[
        "wsh", "sh", "tr", "wpkh", "pkh", "combo", "addr", "rawtr", "raw",
    ];
    if DESCRIPTOR_PREFIXES.contains(&head.as_str()) {
        Ok(InputForm::Descriptor(first_line.to_string()))
    } else {
        Ok(InputForm::Miniscript(first_line.to_string()))
    }
}
