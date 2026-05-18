//! `mnemonic repair` subcommand — BCH error-correction for m-format cards.
//!
//! Realizes `design/IMPLEMENTATION_PLAN_repair_v0_22.md` §2.2. Wraps the
//! library at `crate::repair::repair_card` with:
//!   - flag-repeated input (`--ms1 <s>` / `--mk1 <s> --mk1 <s> …` /
//!     `--md1 <s> --md1 <s> …`); exactly one HRP group per invocation
//!   - special value `-` on any single flag reads chunks from stdin
//!     (one chunk per line; blank lines skipped)
//!   - text-form or JSON-form output (D14)
//!   - D9 sensitive-secret stderr warning when corrected ms1 hits stdout
//!
//! Exit codes:
//!   - 0 — all chunks already valid (no repair applied)
//!   - 5 — at least one correction applied (REPAIR_APPLIED)
//!   - non-zero ToolkitError exit per `error.rs::exit_code()` on failure

use crate::error::ToolkitError;
use crate::repair::{self, CardKind, RepairOutcome, classify_hrp_prefix, validate_flag_hrp};
use crate::secret_advisory::secret_on_stdout_warning;
use clap::{ArgGroup, Args};
use std::io::{Read, Write};

#[derive(Args, Debug)]
#[command(group(
    // v0.24.0 §2.C.1 (D35 fold) — drop cross-HRP `conflicts_with_all` on
    // the three flag args. Cards self-identify by HRP; mixed-HRP invocations
    // are valid (`mnemonic repair ms1xxx mk1yyy md1zzz`). The ArgGroup
    // continues to require at-least-one of {--ms1, --mk1, --md1}; the new
    // positional `extra_strings` carries `required_unless_present_any` of
    // the three flags so an invocation with only positionals still parses.
    ArgGroup::new("kind")
        .args(["ms1", "mk1", "md1"])
        .required(false)
        .multiple(true),
))]
pub struct RepairArgs {
    /// Single ms1 chunk to repair. Use `-` to read one chunk from stdin.
    /// May be combined with --mk1 / --md1 (per-HRP cards can repair in
    /// the same invocation per D35).
    #[arg(long, value_name = "MS1")]
    pub ms1: Option<String>,

    /// One or more mk1 chunks to repair (repeating flag). Use `-` on a
    /// single occurrence to read chunks from stdin (one per line).
    /// May be combined with --ms1 / --md1 per D35.
    #[arg(long, value_name = "MK1")]
    pub mk1: Vec<String>,

    /// One or more md1 chunks to repair (repeating flag). Use `-` on a
    /// single occurrence to read chunks from stdin (one per line).
    /// May be combined with --ms1 / --mk1 per D35.
    #[arg(long, value_name = "MD1")]
    pub md1: Vec<String>,

    /// Emit a single JSON envelope on stdout instead of the text-form
    /// report (D14 — auto-fire short-circuit ALWAYS emits text-form
    /// regardless of this flag; this affects ONLY the standalone subcommand).
    #[arg(long)]
    pub json: bool,

    /// v0.24.0 §2.C.1 — positional `<STRING>...` intake. Each value
    /// self-identifies by HRP prefix (`ms1` / `mk1` / `md1`) and is routed
    /// to the same internal storage as the matching typed flag. Unknown
    /// HRPs are rejected with `ToolkitError::UnknownHrp`. At least one of
    /// {--ms1, --mk1, --md1, positional} is required.
    #[arg(
        value_name = "STRING",
        num_args = 0..,
        required_unless_present_any = ["ms1", "mk1", "md1"],
    )]
    pub extra_strings: Vec<String>,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &RepairArgs,
    stdin: &mut R,
    stdout: &mut W,
    _stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let groups = resolve_groups(args, stdin)?;

    let mut total_repairs = 0usize;
    let mut any_ms1 = false;

    // Emit per-kind reports in fixed (ms1, mk1, md1) order for deterministic
    // output regardless of CLI arg ordering.
    for (kind, chunks) in &groups {
        let outcome = repair::repair_card(*kind, chunks)?;
        total_repairs += outcome.repairs.len();
        if matches!(kind, CardKind::Ms1) {
            any_ms1 = true;
        }
        if args.json {
            emit_repair_json(&outcome, stdout)?;
        } else {
            emit_repair_text(&outcome, stdout)?;
        }
    }

    // D9: emit sensitive-secret stderr warning when ms1 was repaired (regardless
    // of whether corrections fired — even pass-through of a valid ms1 to
    // stdout is sensitive material on stdout).
    if any_ms1 {
        secret_on_stdout_warning(CardKind::Ms1, _stderr);
    }

    Ok(if total_repairs == 0 { 0 } else { 5 })
}

/// v0.24.0 §2.C.1 — gather all input strings into per-kind groups,
/// merging the typed flag form (`--ms1` / `--mk1` / `--md1`) with the
/// positional `<STRING>...` form (HRP-autodetect routed). Returns groups
/// in fixed `(Ms1, Mk1, Md1)` order; empty groups are omitted from the
/// returned vector.
///
/// Mismatched-HRP flag values (`--ms1 mk1xxx`) return `ToolkitError::HrpMismatch`
/// per D34/I5 (toolkit-internal validation, not a clap parser callback).
/// Unknown-HRP positional values return `ToolkitError::UnknownHrp`.
///
/// Storage merge order: flag-form first, then positional (per plan).
fn resolve_groups<R: Read>(
    args: &RepairArgs,
    stdin: &mut R,
) -> Result<Vec<(CardKind, Vec<String>)>, ToolkitError> {
    // D34/I5 — strict per-flag HRP validation. `--ms1 mk1xxx` rejects with
    // `ToolkitError::HrpMismatch { flag: "--ms1", expected: "ms", got: "mk" }`.
    // `-` (stdin sentinel) is exempt; expanded after this check.
    if let Some(v) = &args.ms1 {
        validate_flag_hrp("--ms1", "ms", v)?;
    }
    for v in &args.mk1 {
        validate_flag_hrp("--mk1", "mk", v)?;
    }
    for v in &args.md1 {
        validate_flag_hrp("--md1", "md", v)?;
    }

    // Seed per-kind buckets from flag-form values (flag-form first per plan).
    let mut ms1_vec: Vec<String> = args.ms1.clone().map(|s| vec![s]).unwrap_or_default();
    let mut mk1_vec: Vec<String> = args.mk1.clone();
    let mut md1_vec: Vec<String> = args.md1.clone();

    // Route positional `extra_strings` by HRP prefix.
    for s in &args.extra_strings {
        match classify_hrp_prefix(s)? {
            CardKind::Ms1 => ms1_vec.push(s.clone()),
            CardKind::Mk1 => mk1_vec.push(s.clone()),
            CardKind::Md1 => md1_vec.push(s.clone()),
        }
    }

    if ms1_vec.is_empty() && mk1_vec.is_empty() && md1_vec.is_empty() {
        return Err(ToolkitError::BadInput(
            "repair: at least one of --ms1 / --mk1 / --md1 (or positional STRING) is required".into(),
        ));
    }

    // Per-kind stdin (`-`) expansion. At most one `-` across the whole
    // invocation (across both flag-form and positional combined; stdin is
    // a single non-replayable stream).
    let total_dashes = count_dashes(&ms1_vec) + count_dashes(&mk1_vec) + count_dashes(&md1_vec);
    if total_dashes > 1 {
        return Err(ToolkitError::BadInput(
            "repair: at most one `-` (stdin) value across all repair inputs".into(),
        ));
    }
    if total_dashes == 1 {
        let mut buf = String::new();
        stdin.read_to_string(&mut buf).map_err(ToolkitError::Io)?;
        let stdin_chunks: Vec<String> = buf
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        if stdin_chunks.is_empty() {
            return Err(ToolkitError::BadInput(
                "repair: stdin (`-`) yielded no non-blank chunks".into(),
            ));
        }
        ms1_vec = expand_dashes(&ms1_vec, &stdin_chunks);
        mk1_vec = expand_dashes(&mk1_vec, &stdin_chunks);
        md1_vec = expand_dashes(&md1_vec, &stdin_chunks);
    }

    let mut out: Vec<(CardKind, Vec<String>)> = Vec::with_capacity(3);
    if !ms1_vec.is_empty() {
        out.push((CardKind::Ms1, ms1_vec));
    }
    if !mk1_vec.is_empty() {
        out.push((CardKind::Mk1, mk1_vec));
    }
    if !md1_vec.is_empty() {
        out.push((CardKind::Md1, md1_vec));
    }
    Ok(out)
}

fn count_dashes(v: &[String]) -> usize {
    v.iter().filter(|s| s.as_str() == "-").count()
}

fn expand_dashes(input: &[String], stdin_chunks: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(input.len());
    for c in input {
        if c == "-" {
            out.extend(stdin_chunks.iter().cloned());
        } else {
            out.push(c.clone());
        }
    }
    out
}

fn emit_repair_text<W: Write>(outcome: &RepairOutcome, stdout: &mut W) -> Result<(), ToolkitError> {
    if !outcome.repairs.is_empty() {
        writeln!(stdout, "# Repair report").map_err(ToolkitError::Io)?;
        let kind_str = kind_str(outcome.kind);
        for repair in &outcome.repairs {
            let n = repair.corrected_positions.len();
            let plural = if n == 1 { "correction" } else { "corrections" };
            write!(
                stdout,
                "#   {} chunk {}: {} {} at ",
                kind_str, repair.chunk_index, n, plural
            )
            .map_err(ToolkitError::Io)?;
            for (i, (pos, was, now)) in repair.corrected_positions.iter().enumerate() {
                if i > 0 {
                    write!(stdout, ", ").map_err(ToolkitError::Io)?;
                }
                write!(stdout, "position {pos}: '{was}' -> '{now}'").map_err(ToolkitError::Io)?;
            }
            writeln!(stdout).map_err(ToolkitError::Io)?;
        }
    }
    for chunk in &outcome.corrected_chunks {
        writeln!(stdout, "{chunk}").map_err(ToolkitError::Io)?;
    }
    Ok(())
}

#[derive(serde::Serialize)]
struct RepairJson<'a> {
    schema_version: &'static str,
    kind: &'static str,
    corrected_chunks: &'a [String],
    repairs: Vec<RepairJsonDetail<'a>>,
}

#[derive(serde::Serialize)]
struct RepairJsonDetail<'a> {
    chunk_index: usize,
    original_chunk: &'a str,
    corrected_chunk: &'a str,
    corrected_positions: Vec<RepairJsonPosition>,
}

#[derive(serde::Serialize)]
struct RepairJsonPosition {
    position: usize,
    was: String,
    now: String,
}

fn emit_repair_json<W: Write>(outcome: &RepairOutcome, stdout: &mut W) -> Result<(), ToolkitError> {
    let envelope = RepairJson {
        schema_version: "1",
        kind: kind_str(outcome.kind),
        corrected_chunks: &outcome.corrected_chunks,
        repairs: outcome
            .repairs
            .iter()
            .map(|r| RepairJsonDetail {
                chunk_index: r.chunk_index,
                original_chunk: &r.original_chunk,
                corrected_chunk: &r.corrected_chunk,
                corrected_positions: r
                    .corrected_positions
                    .iter()
                    .map(|(p, w, n)| RepairJsonPosition {
                        position: *p,
                        was: w.to_string(),
                        now: n.to_string(),
                    })
                    .collect(),
            })
            .collect(),
    };
    let body =
        serde_json::to_string(&envelope).map_err(|e| ToolkitError::BadInput(format!("repair JSON serialize: {e}")))?;
    writeln!(stdout, "{body}").map_err(ToolkitError::Io)?;
    Ok(())
}

fn kind_str(kind: CardKind) -> &'static str {
    match kind {
        CardKind::Ms1 => "ms1",
        CardKind::Mk1 => "mk1",
        CardKind::Md1 => "md1",
    }
}
