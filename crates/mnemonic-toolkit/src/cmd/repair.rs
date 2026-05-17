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
use crate::repair::{self, CardKind, RepairOutcome};
use crate::secret_advisory::secret_on_stdout_warning;
use clap::{ArgGroup, Args};
use std::io::{Read, Write};

#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("kind")
        .args(["ms1", "mk1", "md1"])
        .required(true)
        .multiple(true), // multiple within ONE group allowed (mk1+mk1+…); cross-group rejected below
))]
pub struct RepairArgs {
    /// Single ms1 chunk to repair. Use `-` to read one chunk from stdin.
    /// Mutually exclusive with --mk1 / --md1.
    #[arg(long, value_name = "MS1", conflicts_with_all = ["mk1", "md1"])]
    pub ms1: Option<String>,

    /// One or more mk1 chunks to repair (repeating flag). Use `-` on a
    /// single occurrence to read chunks from stdin (one per line).
    /// Mutually exclusive with --ms1 / --md1.
    #[arg(long, value_name = "MK1", conflicts_with_all = ["ms1", "md1"])]
    pub mk1: Vec<String>,

    /// One or more md1 chunks to repair (repeating flag). Use `-` on a
    /// single occurrence to read chunks from stdin (one per line).
    /// Mutually exclusive with --ms1 / --mk1.
    #[arg(long, value_name = "MD1", conflicts_with_all = ["ms1", "mk1"])]
    pub md1: Vec<String>,

    /// Emit a single JSON envelope on stdout instead of the text-form
    /// report (D14 — auto-fire short-circuit ALWAYS emits text-form
    /// regardless of this flag; this affects ONLY the standalone subcommand).
    #[arg(long)]
    pub json: bool,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &RepairArgs,
    stdin: &mut R,
    stdout: &mut W,
    _stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let (kind, chunks) = resolve_kind_and_chunks(args, stdin)?;
    let outcome = repair::repair_card(kind, &chunks)?;

    if args.json {
        emit_repair_json(&outcome, stdout)?;
    } else {
        emit_repair_text(&outcome, stdout)?;
    }

    // D9: emit sensitive-secret stderr warning when kind is Ms1 (regardless
    // of whether corrections fired — even pass-through of a valid ms1 to
    // stdout is sensitive material on stdout).
    if matches!(kind, CardKind::Ms1) {
        secret_on_stdout_warning(kind, _stderr);
    }

    Ok(if outcome.repairs.is_empty() { 0 } else { 5 })
}

fn resolve_kind_and_chunks<R: Read>(
    args: &RepairArgs,
    stdin: &mut R,
) -> Result<(CardKind, Vec<String>), ToolkitError> {
    let (kind, raw): (CardKind, Vec<String>) = if let Some(ms) = &args.ms1 {
        (CardKind::Ms1, vec![ms.clone()])
    } else if !args.mk1.is_empty() {
        (CardKind::Mk1, args.mk1.clone())
    } else if !args.md1.is_empty() {
        (CardKind::Md1, args.md1.clone())
    } else {
        return Err(ToolkitError::BadInput(
            "repair: exactly one of --ms1 / --mk1 / --md1 is required".into(),
        ));
    };

    // Expand any `-` values into stdin lines (one chunk per non-blank line).
    let dash_count = raw.iter().filter(|s| s.as_str() == "-").count();
    if dash_count == 0 {
        return Ok((kind, raw));
    }
    if dash_count > 1 {
        return Err(ToolkitError::BadInput(
            "repair: at most one `-` (stdin) value across all repair flags".into(),
        ));
    }

    let mut buf = String::new();
    stdin
        .read_to_string(&mut buf)
        .map_err(ToolkitError::Io)?;
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

    let mut out: Vec<String> = Vec::with_capacity(raw.len() - 1 + stdin_chunks.len());
    for c in raw {
        if c == "-" {
            out.extend(stdin_chunks.iter().cloned());
        } else {
            out.push(c);
        }
    }
    Ok((kind, out))
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
