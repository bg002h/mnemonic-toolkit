//! Which wallet coordinators will import a policy — coordinator-compatibility
//! plan 1b (`design/DESIGN_coordinator_compatibility.md` §1-§3 in
//! `mnemonic-engrave`).
//!
//! Three sources, one answer per coordinator per VERIFIED version:
//!
//! - **Rules** ([`RuleSet`]): refusals derived from the coordinator's source,
//!   in its own order of refusal, each citing where it is established. A
//!   rule may only ever REFUSE (ruling 3).
//! - **Evidence** (the generated table, [`cells`]): what a harness printed. The only
//!   source of a positive. Built from vendored evidence rows by `cargo xtask
//!   verdicts`, which fails on any rule/evidence disagreement it cannot
//!   resolve (the `build` module, D1-D5).
//! - **Silence**: [`Verdict::Unproven`], for everything else.
//!
//! A template (no keys) can be refused, never claimed to import (design §1
//! (a2)): every clause that reads key identity is skipped and the verdict is
//! `Unproven { KeysAbsent }`.

#[cfg(feature = "derive")]
pub mod build;
mod registry;
#[rustfmt::skip]
mod table;

pub use registry::REGISTRY;

use crate::policy_shape::{Branch, KeyPathKind, LockKind, RootKind};
use crate::skeleton::{Skeleton, skeleton_key};

/// A coordinator's stable identifier (`"liana"`, `"nunchuk"`, `"core"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CoordinatorId(pub &'static str);

/// A coordinator version as its APPLICATION displays it (design §3). Opaque:
/// ordered only by its position in [`Coordinator::verified`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Version(pub &'static str);

/// A closed run of VERIFIED versions (design §1 (e)). Both ends verified;
/// never open-ended (design §3 mechanism 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// First verified version of the run.
    pub since: Version,
    /// Last verified version of the run.
    pub until: Version,
}

/// Which spelling of a descriptor a measurement was taken on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Form {
    /// `/<0;1>/*` — `md descriptor`'s default.
    Multipath,
    /// `/0/*` — `md descriptor --chain 0`.
    Chain0,
    /// `/1/*` — `md descriptor --chain 1`.
    Chain1,
}

impl Form {
    /// The spelling used in notices and in the vendored evidence.
    pub fn as_str(self) -> &'static str {
        match self {
            Form::Multipath => "multipath",
            Form::Chain0 => "chain0",
            Form::Chain1 => "chain1",
        }
    }
}

/// Which tool rendered the descriptor a measurement was taken on (design
/// §1 (d)): provenance on every MEASURED verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RendererId {
    /// The rendering tool.
    pub tool: &'static str,
    /// Its version.
    pub version: &'static str,
    /// The spelling.
    pub form: Form,
}

/// ISO 8601 date a measurement was recorded. Printed for a human; never
/// compared to a clock (design §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasuredAt(pub &'static str);

/// A refusal class: the coordinator's own order-of-refusal name, plus the
/// source that establishes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reason {
    /// The class name, e.g. `"no locked path"`.
    pub class: &'static str,
    /// Where it is established.
    pub cite: &'static str,
}

/// The coordinator's OWN reading of an imported policy, as its harness
/// recorded it — never hand-authored (design §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Description {
    /// The wallet kind the coordinator named, e.g. `"MULTI_SIG"`, `"Liana"`.
    pub wallet_kind: &'static str,
    /// The `k`-of-`n` the coordinator read, where it reads one.
    pub threshold: Option<(u8, u8)>,
}

/// Why a verdict is silent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnprovenReason {
    /// No rule refuses and no measurement covers this policy.
    NoEvidence,
    /// A template: no key identity, so no positive, and no key-reading rule.
    KeysAbsent,
    /// A version outside every verified span.
    OutsideEveryVerifiedSpan,
}

/// One coordinator's answer over one span of verified versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Refused. `renderer: None` = rule-derived (spelling-independent);
    /// `Some` = measured, with its date.
    Refuses {
        /// Why.
        reason: Reason,
        /// Over which verified versions.
        span: Span,
        /// `Some` for a measured refusal.
        renderer: Option<RendererId>,
        /// `Some` for a measured refusal.
        measured_at: Option<MeasuredAt>,
    },
    /// Imported, but read as a different wallet (design §1 (c)).
    ImportsAltered {
        /// The coordinator's reading.
        as_read: Description,
        /// Over which verified versions.
        span: Span,
        /// Provenance.
        renderer: RendererId,
        /// Provenance.
        measured_at: MeasuredAt,
    },
    /// Imported as built. Measured only.
    Imports {
        /// Over which verified versions.
        span: Span,
        /// Provenance.
        renderer: RendererId,
        /// Provenance.
        measured_at: MeasuredAt,
    },
    /// Silent.
    Unproven {
        /// Why.
        reason: UnprovenReason,
        /// Over which verified versions, when known.
        span: Option<Span>,
    },
}

/// Which `Skeleton` fields a clause reads (design §1, `ReadSet`). Declared,
/// because "does this rule read key identity" cannot be asked of an opaque
/// `fn` pointer, and design §1 (a2) turns on exactly that question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadSet(u8);

impl ReadSet {
    /// Root, template, branch structure.
    pub const STRUCTURE: ReadSet = ReadSet(1);
    /// The partitions, or key material such as `leaf_keys_ascending`.
    pub const KEY_IDENTITY: ReadSet = ReadSet(2);
    /// Lock values.
    pub const LOCK_VALUES: ReadSet = ReadSet(4);
    /// Hashlock digests.
    pub const DIGESTS: ReadSet = ReadSet(8);

    /// The union of two sets.
    pub const fn with(self, other: ReadSet) -> ReadSet {
        ReadSet(self.0 | other.0)
    }

    /// Whether `other` is a subset of `self`.
    pub const fn contains(self, other: ReadSet) -> bool {
        self.0 & other.0 == other.0
    }
}

/// One refusal clause of a [`RuleSet`].
#[derive(Debug, Clone, Copy)]
pub struct Clause {
    /// The class it names, and where that is established.
    pub reason: Reason,
    /// What it reads.
    pub reads: ReadSet,
    /// Whether it refuses this policy.
    pub fires: fn(&Skeleton) -> bool,
}

/// A refusal that depends on the SPELLING, not the policy.
#[derive(Debug, Clone, Copy)]
pub struct FormRefusal {
    /// The refused spelling.
    pub form: Form,
    /// Why.
    pub reason: Reason,
}

/// A coordinator's rules over one span of verified versions.
#[derive(Debug, Clone, Copy)]
pub struct RuleSet {
    /// Where these rules hold.
    pub span: Span,
    /// The versions whose SOURCE was read to write them.
    pub source_verified_at: &'static [&'static str],
    /// In the coordinator's own order of refusal; the first that fires names
    /// the reason.
    pub clauses: &'static [Clause],
    /// Spellings refused whatever the policy.
    pub form_refusals: &'static [FormRefusal],
    /// Whether a measured refusal no clause explains is stored as a verdict
    /// (true for a coordinator whose refusals are renderer-dependent,
    /// design §1) or is a D2 build failure.
    pub measured_refusals_admitted: bool,
}

/// One wallet coordinator.
#[derive(Debug, Clone, Copy)]
pub struct Coordinator {
    /// Stable id.
    pub id: CoordinatorId,
    /// Display name.
    pub name: &'static str,
    /// Every version any measurement or source reading verified, oldest
    /// first. The ONLY versions a verdict may name.
    pub verified: &'static [&'static str],
    /// Rule sets; together they must cover every verified version.
    pub rules: &'static [RuleSet],
    /// Whether the coordinator's recorded reading matches the policy as
    /// built. `false` makes a measured import `ImportsAltered`.
    pub reads_as_built: fn(&Skeleton, &Description) -> bool,
    /// The refusal class a coordinator's own message names, when it names
    /// one — the D3 check. `None` for a generic message.
    pub class_of_message: fn(&str) -> Option<&'static str>,
}

impl Coordinator {
    /// Position of `v` in [`Coordinator::verified`].
    pub fn position(&self, v: &str) -> Option<usize> {
        self.verified.iter().position(|x| *x == v)
    }

    /// The rule set whose span holds verified version `v`.
    pub fn rule_set_at(&self, v: &str) -> Option<&RuleSet> {
        let at = self.position(v)?;
        self.rules.iter().find(|r| {
            matches!(
                (self.position(r.span.since.0), self.position(r.span.until.0)),
                (Some(a), Some(b)) if a <= at && at <= b
            )
        })
    }
}

/// One measured cell of the generated table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    /// The [`crate::skeleton::SkeletonKey`] string.
    pub key: &'static str,
    /// [`CoordinatorId`]'s string.
    pub coordinator: &'static str,
    /// A verified version.
    pub version: &'static str,
    /// The spelling measured.
    pub form: Form,
    /// What the harness recorded.
    pub outcome: CellOutcome,
    /// Provenance.
    pub renderer_tool: &'static str,
    /// Provenance.
    pub renderer_version: &'static str,
    /// Provenance.
    pub measured_at: &'static str,
    /// The evidence row it came from.
    pub source: &'static str,
}

/// A measured outcome, as stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellOutcome {
    /// Imported as built.
    Imported,
    /// Imported, read as something else.
    ImportedAltered(Description),
    /// Refused, with the coordinator's message.
    Refused(&'static str),
}

/// The measured cells, generated by the `build` module.
pub fn cells() -> &'static [Cell] {
    table::CELLS
}

/// A verdict about one coordinator.
#[derive(Debug, Clone, Copy)]
pub struct CoordinatorVerdict {
    /// Which coordinator.
    pub coordinator: &'static Coordinator,
    /// Its answer over one span.
    pub verdict: Verdict,
}

/// The per-version answer before runs are merged into spans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum At {
    Refuses(Reason, Option<(RendererId, MeasuredAt)>),
    Altered(Description, RendererId, MeasuredAt),
    Imports(RendererId, MeasuredAt),
    Unproven(UnprovenReason),
}

/// The first clause of `rs` that fires on `s`. With no keys present, a
/// clause that reads key identity is skipped: it cannot be evaluated, and a
/// structure-only refusal is still sound (design §1 (a2)).
pub fn first_refusal(rs: &RuleSet, s: &Skeleton) -> Option<Reason> {
    rs.clauses
        .iter()
        .filter(|c| s.keys_present || !c.reads.contains(ReadSet::KEY_IDENTITY))
        .find(|c| (c.fires)(s))
        .map(|c| c.reason)
}

fn at_version(c: &Coordinator, v: &'static str, s: &Skeleton, key: &str, form: Option<Form>) -> At {
    let Some(rs) = c.rule_set_at(v) else {
        return At::Unproven(UnprovenReason::OutsideEveryVerifiedSpan);
    };
    // The spelling first: a coordinator parses key expressions before it
    // reads the script (the order `build` measured on Core 24.2-25.2).
    if let Some(fr) = form.and_then(|f| rs.form_refusals.iter().find(|fr| fr.form == f)) {
        return At::Refuses(fr.reason, None);
    }
    if let Some(reason) = first_refusal(rs, s) {
        return At::Refuses(reason, None);
    }
    if !s.keys_present {
        return At::Unproven(UnprovenReason::KeysAbsent);
    }
    let Some(form) = form else {
        return At::Unproven(UnprovenReason::NoEvidence);
    };
    let Some(cell) = table::CELLS
        .iter()
        .find(|x| x.key == key && x.coordinator == c.id.0 && x.version == v && x.form == form)
    else {
        return At::Unproven(UnprovenReason::NoEvidence);
    };
    let renderer = RendererId {
        tool: cell.renderer_tool,
        version: cell.renderer_version,
        form: cell.form,
    };
    let at = MeasuredAt(cell.measured_at);
    match cell.outcome {
        CellOutcome::Imported => At::Imports(renderer, at),
        CellOutcome::ImportedAltered(d) => At::Altered(d, renderer, at),
        CellOutcome::Refused(msg) => At::Refuses(
            Reason {
                class: "measured",
                cite: msg,
            },
            Some((renderer, at)),
        ),
    }
}

fn verdict_of(at: At, span: Span) -> Verdict {
    match at {
        At::Refuses(reason, measured) => Verdict::Refuses {
            reason,
            span,
            renderer: measured.map(|m| m.0),
            measured_at: measured.map(|m| m.1),
        },
        At::Altered(as_read, renderer, measured_at) => Verdict::ImportsAltered {
            as_read,
            span,
            renderer,
            measured_at,
        },
        At::Imports(renderer, measured_at) => Verdict::Imports {
            span,
            renderer,
            measured_at,
        },
        At::Unproven(reason) => Verdict::Unproven {
            reason,
            span: Some(span),
        },
    }
}

/// Every coordinator's verdict for `s`, one entry per RUN of consecutive
/// verified versions with the same answer. `form` is the spelling the
/// caller is about to print; `None` for a template, which has none.
pub fn verdicts(s: &Skeleton, form: Option<Form>) -> Vec<CoordinatorVerdict> {
    let key = skeleton_key(s);
    let mut out = Vec::new();
    for c in REGISTRY {
        let mut run: Option<(At, &'static str, &'static str)> = None;
        for v in c.verified {
            let at = at_version(c, v, s, key.as_str(), form);
            run = match run {
                Some((prev, since, _)) if prev == at => Some((prev, since, v)),
                Some((prev, since, until)) => {
                    out.push(CoordinatorVerdict {
                        coordinator: c,
                        verdict: verdict_of(
                            prev,
                            Span {
                                since: Version(since),
                                until: Version(until),
                            },
                        ),
                    });
                    Some((at, v, v))
                }
                None => Some((at, v, v)),
            };
        }
        if let Some((prev, since, until)) = run {
            out.push(CoordinatorVerdict {
                coordinator: c,
                verdict: verdict_of(
                    prev,
                    Span {
                        since: Version(since),
                        until: Version(until),
                    },
                ),
            });
        }
    }
    out
}

/// Ruling 2's "none" case: every coordinator refuses at every verified
/// version. `Unproven` is not a refusal, so a template whose positives are
/// merely unmeasurable is never "none".
pub fn none_imports(vs: &[CoordinatorVerdict]) -> bool {
    !vs.is_empty()
        && vs
            .iter()
            .all(|v| matches!(v.verdict, Verdict::Refuses { .. }))
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.since == self.until {
            write!(f, "{}", self.since.0)
        } else {
            write!(f, "{}-{}", self.since.0, self.until.0)
        }
    }
}

/// One line per verdict, version-qualified and dated where measured
/// (design §3 mechanism 1). Never "Liana imports"; always "Liana 15.0
/// imports (…, 2026-09-20)".
pub fn describe(v: &CoordinatorVerdict) -> String {
    let name = v.coordinator.name;
    match v.verdict {
        Verdict::Refuses {
            reason,
            span,
            renderer: None,
            ..
        } => format!("{name} {span}: refuses ({})", reason.class),
        Verdict::Refuses {
            reason,
            span,
            renderer: Some(r),
            measured_at,
        } => format!(
            "{name} {span}: refused the {} form when measured ({}, {} {}, {}): {}",
            r.form.as_str(),
            measured_at.map_or("undated", |m| m.0),
            r.tool,
            r.version,
            v.coordinator.id.0,
            reason.cite
        ),
        Verdict::ImportsAltered {
            as_read,
            span,
            renderer,
            measured_at,
        } => {
            let t = as_read
                .threshold
                .map_or(String::new(), |(k, n)| format!(" {k}-of-{n}"));
            format!(
                "{name} {span}: imports the {} form, but reads it as {}{t} -- not the wallet as built ({})",
                renderer.form.as_str(),
                as_read.wallet_kind,
                measured_at.0
            )
        }
        Verdict::Imports {
            span,
            renderer,
            measured_at,
        } => format!(
            "{name} {span}: imports the {} form (measured {})",
            renderer.form.as_str(),
            measured_at.0
        ),
        Verdict::Unproven { reason, span } => {
            let span = span.map_or(String::new(), |s| format!(" {s}"));
            let why = match reason {
                UnprovenReason::NoEvidence => "not measured for this policy",
                UnprovenReason::KeysAbsent => "a template has no keys, so no import can be claimed",
                UnprovenReason::OutsideEveryVerifiedSpan => "no verified version",
            };
            format!("{name}{span}: unproven ({why})")
        }
    }
}

// ---------------------------------------------------------------------------
// Predicates shared by the rule clauses in `registry`. Each reads only what
// its clause's `ReadSet` declares.
// ---------------------------------------------------------------------------

fn unlocked(b: &Branch) -> bool {
    b.locks.is_empty()
}

/// A spend path with no key that a lock or hash alone satisfies. A `0`
/// branch (`or_i(X,0)`) has no key and no lock and is NOT one: it is
/// unsatisfiable. A `1` branch is missed, which errs toward silence.
fn keyless_gated(b: &Branch) -> bool {
    b.slots.is_empty() && (!b.locks.is_empty() || !b.hashlocks.is_empty())
}

fn any_lock(s: &Skeleton, kinds: &[LockKind]) -> bool {
    s.shape
        .branches
        .iter()
        .any(|b| b.locks.iter().any(|l| kinds.contains(&l.kind)))
}

fn is_tr(s: &Skeleton) -> bool {
    s.root == RootKind::Tr
}

fn is_nums(s: &Skeleton) -> bool {
    s.shape.key_path == KeyPathKind::Nums
}
