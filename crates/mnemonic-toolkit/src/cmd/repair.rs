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
use crate::repair::{self, CardArgs, CardKind, RepairOutcome};
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

impl CardArgs for RepairArgs {
    fn ms1(&self) -> Option<&String> {
        self.ms1.as_ref()
    }
    fn mk1(&self) -> &[String] {
        &self.mk1
    }
    fn md1(&self) -> &[String] {
        &self.md1
    }
    fn extra_strings(&self) -> &[String] {
        &self.extra_strings
    }
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &RepairArgs,
    stdin: &mut R,
    stdout: &mut W,
    _stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let groups = repair::resolve_groups(args, "repair", stdin)?;

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
