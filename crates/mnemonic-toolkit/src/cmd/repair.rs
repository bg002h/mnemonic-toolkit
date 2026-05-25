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
use crate::indel::{IndelCandidate, IndelOutcome, IndelRegion, IndelDirection};
use crate::repair::{self, CardArgs, CardKind, RepairError, RepairOutcome};
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

    /// Maximum insert/delete (indel) distance to search when a chunk fails
    /// normal repair — recovers a single transcribed character that was added
    /// (too long) or dropped (too short). 0 disables (default). ms1/mk1 only.
    #[arg(long, value_name = "N", default_value_t = 0, value_parser = clap::value_parser!(u8).range(0..=4))]
    pub max_indel: u8,

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
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // When indel search is active, relax the strict typed-flag HRP pre-gate so
    // a prefix-region indel (`--ms1 s10…` = `ms1…` minus 'm') reaches
    // `repair_card` and engages the indel trigger via `RepairError::HrpMismatch`
    // (§1.7). At `--max-indel 0` the gate stays strict — today's behavior.
    let groups = repair::resolve_groups(args, "repair", stdin, args.max_indel >= 1)?;

    let mut total_repairs = 0usize;
    let mut any_ms1 = false;
    let mut ambiguous_seen = false;

    // Runtime notice for the slower search budgets (combinatorial blow-up at
    // j ≥ 3 over a long-code data-part).
    if args.max_indel >= 3 {
        writeln!(
            stderr,
            "repair: searching up to {} indels; this may take a few seconds",
            args.max_indel
        )
        .ok();
    }

    // Emit per-kind reports in fixed (ms1, mk1, md1) order for deterministic
    // output regardless of CLI arg ordering. Multi-group (e.g. `--ms1 X
    // --mk1 Y`) is supported: the indel branch must NOT early-return on the
    // non-fatal (Ambiguous / recovered) outcomes, so every group still emits
    // (R0 I1). Only Unrecoverable short-circuits (exit 2).
    for (kind, chunks) in &groups {
        match repair::repair_card(*kind, chunks) {
            Ok(outcome) => {
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
            Err(e) if args.max_indel >= 1 && repair::is_indel_trigger(&e) => {
                match repair::recover_indel_card(*kind, chunks, args.max_indel as usize, 0)? {
                    IndelOutcome::Unique(c) => {
                        if matches!(kind, CardKind::Ms1) {
                            any_ms1 = true;
                        }
                        if args.json {
                            emit_indel_json("unique", &[&c], stdout)?;
                        } else {
                            emit_indel_text(&[&c], stdout)?;
                        }
                        total_repairs += 1;
                    }
                    IndelOutcome::Ambiguous(v) => {
                        if matches!(kind, CardKind::Ms1) {
                            any_ms1 = true;
                        }
                        let refs: Vec<&IndelCandidate> = v.iter().collect();
                        if args.json {
                            emit_indel_json("ambiguous", &refs, stdout)?;
                        } else {
                            emit_indel_text(&refs, stdout)?;
                        }
                        writeln!(
                            stderr,
                            "repair: ambiguous — {} candidates within --max-indel {}; choose manually",
                            v.len(),
                            args.max_indel
                        )
                        .ok();
                        ambiguous_seen = true;
                    }
                    IndelOutcome::Unrecoverable => {
                        return Err(ToolkitError::Repair(RepairError::IndelUnrecoverable {
                            hrp: kind.hrp(),
                            max_indel: args.max_indel as usize,
                        }));
                    }
                }
            }
            Err(e) => return Err(e.into()),
        }
    }

    // D9: emit sensitive-secret stderr warning when ms1 hit stdout (regardless
    // of whether corrections fired — even pass-through of a valid ms1, or a
    // recovered/ambiguous ms1 candidate, is sensitive material on stdout).
    if any_ms1 {
        secret_on_stdout_warning(CardKind::Ms1, stderr);
    }

    Ok(repair::indel_exit_code(ambiguous_seen, total_repairs))
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

// ============================================================================
// Phase 5 — indel (`--max-indel`) recovery output.
// ============================================================================

/// JSON envelope for an indel recovery emission. `status` is one of
/// `"unique"` / `"ambiguous"`; the `Unrecoverable` outcome surfaces via the
/// `Err`/exit-2 path and emits NO JSON (so `status` has no
/// `"unrecoverable"` value). Wire-shape is NOT schema_mirror-gated — GUI
/// consumers self-update per the paired-PR rule.
#[derive(serde::Serialize)]
struct IndelJson<'a> {
    schema_version: &'static str,
    status: &'static str,
    candidates: Vec<IndelCandidateJson<'a>>,
}

#[derive(serde::Serialize)]
struct IndelCandidateJson<'a> {
    recovered: &'a str,
    indel_count: usize,
    region: &'static str,
    direction: &'static str,
}

fn region_str(r: IndelRegion) -> &'static str {
    match r {
        IndelRegion::Prefix => "prefix",
        IndelRegion::DataPart => "data-part",
        IndelRegion::CrossRegion => "cross-region",
    }
}

fn direction_str(d: IndelDirection) -> &'static str {
    match d {
        IndelDirection::Inserted => "inserted",
        IndelDirection::Deleted => "deleted",
    }
}

fn candidate_json(c: &IndelCandidate) -> IndelCandidateJson<'_> {
    IndelCandidateJson {
        recovered: &c.recovered,
        indel_count: c.indel_count,
        region: region_str(c.region),
        direction: direction_str(c.direction),
    }
}

/// Text-form indel emission — each recovered string on its own line (mirrors
/// the corrected-chunks tail of `emit_repair_text`).
fn emit_indel_text<W: Write>(cands: &[&IndelCandidate], stdout: &mut W) -> Result<(), ToolkitError> {
    for c in cands {
        writeln!(stdout, "{}", c.recovered).map_err(ToolkitError::Io)?;
    }
    Ok(())
}

/// JSON-form indel emission — single `IndelJson` envelope on stdout.
fn emit_indel_json<W: Write>(
    status: &'static str,
    cands: &[&IndelCandidate],
    stdout: &mut W,
) -> Result<(), ToolkitError> {
    let envelope = IndelJson {
        schema_version: "1",
        status,
        candidates: cands.iter().map(|c| candidate_json(c)).collect(),
    };
    let body = serde_json::to_string(&envelope)
        .map_err(|e| ToolkitError::BadInput(format!("indel JSON serialize: {e}")))?;
    writeln!(stdout, "{body}").map_err(ToolkitError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Ambiguous CLI exit path is cryptographically unreachable with real
    /// vectors (~2⁻⁶⁵ checksum collision), so this unit test is the canonical
    /// coverage for the multi-candidate emit helpers. Hand-build a 2-candidate
    /// Vec (one Prefix/Inserted, one DataPart/Deleted) and assert both the
    /// text-form (one recovered per line) and the JSON envelope shape.
    #[test]
    fn emit_indel_two_candidate_text_and_json() {
        let a = IndelCandidate {
            recovered: "ms1aaa".to_string(),
            indel_count: 1,
            region: IndelRegion::Prefix,
            direction: IndelDirection::Inserted,
            subst_count: 0,
        };
        let b = IndelCandidate {
            recovered: "ms1bbb".to_string(),
            indel_count: 1,
            region: IndelRegion::DataPart,
            direction: IndelDirection::Deleted,
            subst_count: 0,
        };
        let refs = vec![&a, &b];

        let mut text = Vec::new();
        emit_indel_text(&refs, &mut text).unwrap();
        assert_eq!(String::from_utf8(text).unwrap(), "ms1aaa\nms1bbb\n");

        let mut json = Vec::new();
        emit_indel_json("ambiguous", &refs, &mut json).unwrap();
        let v: serde_json::Value =
            serde_json::from_slice(&json).expect("valid JSON envelope");
        assert_eq!(v["schema_version"], "1");
        assert_eq!(v["status"], "ambiguous");
        assert_eq!(v["candidates"][0]["recovered"], "ms1aaa");
        assert_eq!(v["candidates"][0]["region"], "prefix");
        assert_eq!(v["candidates"][0]["direction"], "inserted");
        assert_eq!(v["candidates"][1]["recovered"], "ms1bbb");
        assert_eq!(v["candidates"][1]["region"], "data-part");
        assert_eq!(v["candidates"][1]["direction"], "deleted");
    }
}
