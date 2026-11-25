//! The rule/evidence table BUILD (design §2): where rules and measurements
//! meet, once, at build time — never at runtime.
//!
//! Every vendored evidence row is keyed through the ONE key implementation
//! ([`crate::descriptor_route::descriptor_from_text`] then
//! [`crate::skeleton::skeleton`]), checked against its coordinator's rules,
//! and either stored as a measured [`super::Cell`] or reported as a
//! disagreement. Any disagreement fails the build; a person resolves it and
//! commits the resolution (a narrower or wider rule, a re-attributed class,
//! or a dropped row).
//!
//! | class | rule says | evidence says |
//! | --- | --- | --- |
//! | D1 false-refusal | refuses | imported |
//! | D2 missed-refusal | admits | refused (only where the rule set does not admit measured refusals) |
//! | D3 reason-drift | refuses for X | refused with a message naming Y |
//! | D4 orphan-evidence | no rule set spans the version | any |
//! | D5 evidence-conflict | — | two rows, one key/coordinator/version/form, different outcomes |
//!
//! D5 is not in the design's first four (recon §1c): two measurements that
//! share a key and disagree cannot be expressed by the table, and the build
//! must say so rather than keep whichever row came last.

use super::{Form, REGISTRY, first_refusal};
use crate::descriptor_route::descriptor_from_text;
use crate::skeleton::{Skeleton, skeleton, skeleton_key};

/// One vendored evidence row, as the harness recorded it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRow {
    /// `<source file>:<line>` — where the row came from.
    pub id: String,
    /// A registry coordinator id.
    pub coordinator: String,
    /// The application version measured.
    pub version: String,
    /// The library/commit the binary self-reported (provenance).
    pub library_rev: String,
    /// The tool that rendered `descriptor`.
    pub renderer_tool: String,
    /// Its version.
    pub renderer_version: String,
    /// The spelling the coordinator saw.
    pub form: Form,
    /// ISO date.
    pub measured_at: String,
    /// The MULTIPATH descriptor text the key is computed from.
    pub descriptor: String,
    /// What the coordinator did.
    pub outcome: MeasuredOutcome,
    /// `Some(reason)` when the vendoring script declares this row cannot be
    /// keyed (a form md1 cannot carry). The build asserts it indeed cannot.
    pub unkeyable: Option<String>,
}

/// What a harness recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeasuredOutcome {
    /// Imported; the coordinator's own reading, where the harness records one.
    Imported(Option<OwnedDescription>),
    /// Refused, with the coordinator's message.
    Refused(String),
}

/// An owned [`super::Description`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedDescription {
    /// See [`super::Description::wallet_kind`].
    pub wallet_kind: String,
    /// See [`super::Description::threshold`].
    pub threshold: Option<(u8, u8)>,
}

/// A stored outcome, owned.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuiltOutcome {
    /// Imported as built.
    Imported,
    /// Imported, read as something else.
    ImportedAltered {
        /// Coordinator's wallet kind.
        wallet_kind: String,
        /// Coordinator's `k`-of-`n`.
        threshold: Option<(u8, u8)>,
    },
    /// Refused (measured, admitted).
    Refused(String),
}

/// One cell of the table, owned; [`render_table`] turns a list of these
/// into `table.rs`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BuiltCell {
    /// SkeletonKey string.
    pub key: String,
    /// Coordinator id.
    pub coordinator: String,
    /// Version.
    pub version: String,
    /// Form.
    pub form: Form,
    /// Outcome.
    pub outcome: BuiltOutcome,
    /// Provenance.
    pub renderer_tool: String,
    /// Provenance.
    pub renderer_version: String,
    /// Provenance.
    pub measured_at: String,
    /// Evidence row id.
    pub source: String,
}

/// Why the build failed on one row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Disagreement {
    /// Rule refuses; evidence imported.
    D1FalseRefusal {
        /// Row.
        row: String,
        /// The rule's class.
        class: &'static str,
    },
    /// Rule admits; evidence refused.
    D2MissedRefusal {
        /// Row.
        row: String,
        /// The coordinator's message.
        message: String,
    },
    /// Both refuse, for different named classes.
    D3ReasonDrift {
        /// Row.
        row: String,
        /// The rule's class.
        rule: &'static str,
        /// The class the message names.
        evidence: &'static str,
    },
    /// No rule set spans the row's version, or the coordinator is unknown.
    D4Orphan {
        /// Row.
        row: String,
    },
    /// Two rows with one key/coordinator/version/form and different outcomes.
    D5EvidenceConflict {
        /// The rows.
        rows: (String, String),
    },
    /// A row that should key did not, or a row declared unkeyable did.
    Keying {
        /// Row.
        row: String,
        /// What happened.
        detail: String,
    },
}

/// The build's result: the stored cells, or every disagreement.
pub struct Built {
    /// Cells, sorted and deduplicated.
    pub cells: Vec<BuiltCell>,
    /// Rows the vendoring script declared unkeyable, confirmed so.
    pub unkeyable: usize,
    /// Rows a rule refused and the evidence agreed (not stored).
    pub agreed_refusals: usize,
}

/// Key one row's descriptor through the ONE key implementation.
fn key_row(row: &EvidenceRow) -> Result<Skeleton, String> {
    let d = descriptor_from_text(&row.descriptor).map_err(|e| e.to_string())?;
    skeleton(&d).map_err(|e| e.to_string())
}

/// Build the table from `rows`, or report every disagreement.
///
/// # Errors
///
/// Every [`Disagreement`] found, in row order.
pub fn build(rows: &[EvidenceRow]) -> Result<Built, Vec<Disagreement>> {
    let mut bad = Vec::new();
    let mut cells: Vec<BuiltCell> = Vec::new();
    let mut unkeyable = 0;
    let mut agreed_refusals = 0;
    for row in rows {
        let keyed = key_row(row);
        let s = match (&row.unkeyable, keyed) {
            (Some(_), Err(_)) => {
                unkeyable += 1;
                continue;
            }
            (Some(why), Ok(_)) => {
                bad.push(Disagreement::Keying {
                    row: row.id.clone(),
                    detail: format!("declared unkeyable ({why}) but it keys"),
                });
                continue;
            }
            (None, Err(e)) => {
                bad.push(Disagreement::Keying {
                    row: row.id.clone(),
                    detail: e,
                });
                continue;
            }
            (None, Ok(s)) => s,
        };
        let Some(c) = REGISTRY.iter().find(|c| c.id.0 == row.coordinator) else {
            bad.push(Disagreement::D4Orphan {
                row: row.id.clone(),
            });
            continue;
        };
        let Some(rs) = c.rule_set_at(&row.version) else {
            bad.push(Disagreement::D4Orphan {
                row: row.id.clone(),
            });
            continue;
        };
        // A spelling refusal comes FIRST: Core parses key expressions before
        // it looks at the script, so on 24.2-25.2 a multipath `tr` miniscript
        // is refused for its `<0;1>`, not for its miniscript (a D3 this
        // build found; recorded in plan 1b).
        let rule = rs
            .form_refusals
            .iter()
            .find(|f| f.form == row.form)
            .map(|f| f.reason)
            .or_else(|| first_refusal(rs, &s));
        let outcome = match (&row.outcome, rule) {
            (MeasuredOutcome::Refused(msg), Some(r)) => {
                if let Some(named) = (c.class_of_message)(msg) {
                    if named != r.class {
                        bad.push(Disagreement::D3ReasonDrift {
                            row: row.id.clone(),
                            rule: r.class,
                            evidence: named,
                        });
                    }
                }
                agreed_refusals += 1;
                continue;
            }
            (MeasuredOutcome::Refused(msg), None) => {
                if !rs.measured_refusals_admitted {
                    bad.push(Disagreement::D2MissedRefusal {
                        row: row.id.clone(),
                        message: msg.clone(),
                    });
                    continue;
                }
                BuiltOutcome::Refused(msg.clone())
            }
            (MeasuredOutcome::Imported(_), Some(r)) => {
                bad.push(Disagreement::D1FalseRefusal {
                    row: row.id.clone(),
                    class: r.class,
                });
                continue;
            }
            (MeasuredOutcome::Imported(None), None) => BuiltOutcome::Imported,
            (MeasuredOutcome::Imported(Some(d)), None) => {
                // The static `Description` borrows; the check needs only a view.
                let view = super::Description {
                    wallet_kind: leak_free(&d.wallet_kind),
                    threshold: d.threshold,
                };
                if (c.reads_as_built)(&s, &view) {
                    BuiltOutcome::Imported
                } else {
                    BuiltOutcome::ImportedAltered {
                        wallet_kind: d.wallet_kind.clone(),
                        threshold: d.threshold,
                    }
                }
            }
        };
        cells.push(BuiltCell {
            key: skeleton_key(&s).as_str().to_string(),
            coordinator: row.coordinator.clone(),
            version: row.version.clone(),
            form: row.form,
            outcome,
            renderer_tool: row.renderer_tool.clone(),
            renderer_version: row.renderer_version.clone(),
            measured_at: row.measured_at.clone(),
            source: row.id.clone(),
        });
    }
    cells.sort();
    // D5, and dedup: one cell per (key, coordinator, version, form). Two rows
    // agreeing on the outcome keep the first as the cited source.
    let mut kept: Vec<BuiltCell> = Vec::new();
    for cell in cells {
        match kept.last() {
            Some(prev)
                if prev.key == cell.key
                    && prev.coordinator == cell.coordinator
                    && prev.version == cell.version
                    && prev.form == cell.form =>
            {
                if prev.outcome != cell.outcome {
                    bad.push(Disagreement::D5EvidenceConflict {
                        rows: (prev.source.clone(), cell.source.clone()),
                    });
                }
            }
            _ => kept.push(cell),
        }
    }
    if bad.is_empty() {
        Ok(Built {
            cells: kept,
            unkeyable,
            agreed_refusals,
        })
    } else {
        Err(bad)
    }
}

/// `reads_as_built` takes a `Description` of `&'static str`s because the
/// runtime table is static. The build only needs to compare, so it matches
/// the known wallet kinds back to a static spelling, and anything else to a
/// sentinel no coordinator treats as "as built".
fn leak_free(kind: &str) -> &'static str {
    const KNOWN: &[&str] = &[
        "MULTI_SIG",
        "SINGLE_SIG",
        "MINISCRIPT",
        "Liana",
        "ADDRESSES_DIFFER",
    ];
    KNOWN
        .iter()
        .find(|k| **k == kind)
        .copied()
        .unwrap_or("UNRECOGNISED")
}

/// Render `cells` as the Rust source of `coordinator/table.rs`. Byte-stable:
/// the same cells always render the same text, which is what lets
/// `cargo xtask verdicts --check` and the xtask's own test detect a stale
/// table.
pub fn render_table(cells: &[BuiltCell], evidence_path: &str) -> String {
    let mut out = String::new();
    out.push_str("//! GENERATED by `cargo xtask verdicts` from\n");
    out.push_str(&format!("//! `{evidence_path}`. Do not edit.\n\n"));
    out.push_str("#[allow(unused_imports)]\n");
    out.push_str("use super::{Cell, CellOutcome, Description, Form};\n\n");
    out.push_str("pub(super) static CELLS: &[Cell] = &[\n");
    for c in cells {
        let form = match c.form {
            Form::Multipath => "Form::Multipath",
            Form::Chain0 => "Form::Chain0",
            Form::Chain1 => "Form::Chain1",
        };
        let outcome = match &c.outcome {
            BuiltOutcome::Imported => "CellOutcome::Imported".to_string(),
            BuiltOutcome::ImportedAltered {
                wallet_kind,
                threshold,
            } => format!(
                "CellOutcome::ImportedAltered(Description {{ wallet_kind: {wallet_kind:?}, threshold: {threshold:?} }})"
            ),
            BuiltOutcome::Refused(m) => format!("CellOutcome::Refused({m:?})"),
        };
        out.push_str(&format!(
            "    Cell {{ key: {:?}, coordinator: {:?}, version: {:?}, form: {form}, outcome: {outcome}, renderer_tool: {:?}, renderer_version: {:?}, measured_at: {:?}, source: {:?} }},\n",
            c.key, c.coordinator, c.version, c.renderer_tool, c.renderer_version, c.measured_at, c.source
        ));
    }
    out.push_str("];\n");
    out
}
