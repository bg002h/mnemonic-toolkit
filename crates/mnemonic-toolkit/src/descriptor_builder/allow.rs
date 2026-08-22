//! The shared `--allow` surface: the five-rule vocabulary, the never-silent
//! stderr printer, and the [`AllowSet`] → `ExtParams` mapping.
//!
//! Lifted here from `cmd/build_descriptor.rs` (where `CliAllow`, `allow_set`
//! and `emit_allow_notes` were all private) and from `gate.rs` (which owned
//! `AllowSet::to_ext_params`) **before** a second command is wired to them.
//! Doing it in that order is the point: "make it `pub`" applied twice is how
//! two subcommands end up with two rule vocabularies that drift apart
//! (PLAN Phase 1, round-1 finding N1).
//!
//! This module is a pure lift — no behaviour changes to `build-descriptor`.

use std::io::Write;

use clap::ValueEnum;
use miniscript::miniscript::analyzable::ExtParams;

use super::gate::{AllowSet, DiagnosticKind};
use crate::error::ToolkitError;

/// The 5 allowable sanity rules (allow SPEC §1) — kebab values aligned 1:1
/// with the step-3 `DiagnosticKind::as_str` names (drift self-test in
/// `cmd::build_descriptor`'s test module).
/// miniscript's 6th `ExtParams` field, `raw_pkh`, is deliberately not
/// exposed (unreachable from IR-rendered miniscript).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum CliAllow {
    /// A malleable satisfaction.
    Malleable,
    /// An unspendable mixed height/time timelock path.
    MixedTimelock,
    /// A key used more than once.
    RepeatedKeys,
    /// Exceeds script resource limits.
    ResourceLimit,
    /// An anyone-can-spend path.
    SiglessBranch,
}

impl CliAllow {
    pub(crate) fn kind(self) -> DiagnosticKind {
        match self {
            CliAllow::Malleable => DiagnosticKind::Malleable,
            CliAllow::MixedTimelock => DiagnosticKind::MixedTimelock,
            CliAllow::RepeatedKeys => DiagnosticKind::RepeatedKeys,
            CliAllow::ResourceLimit => DiagnosticKind::ResourceLimit,
            CliAllow::SiglessBranch => DiagnosticKind::SiglessBranch,
        }
    }

    pub(crate) fn kebab(self) -> &'static str {
        match self {
            CliAllow::Malleable => "malleable",
            CliAllow::MixedTimelock => "mixed-timelock",
            CliAllow::RepeatedKeys => "repeated-keys",
            CliAllow::ResourceLimit => "resource-limit",
            CliAllow::SiglessBranch => "sigless-branch",
        }
    }
}

pub(crate) fn allow_set(requested: &[CliAllow]) -> AllowSet {
    let mut set = AllowSet::default();
    for a in requested {
        match a {
            CliAllow::Malleable => set.malleable = true,
            CliAllow::MixedTimelock => set.mixed_timelock = true,
            CliAllow::RepeatedKeys => set.repeated_keys = true,
            CliAllow::ResourceLimit => set.resource_limit = true,
            CliAllow::SiglessBranch => set.sigless_branch = true,
        }
    }
    set
}

impl AllowSet {
    pub(crate) fn to_ext_params(self) -> ExtParams {
        let mut p = ExtParams::new();
        p.top_unsafe = self.sigless_branch;
        p.malleability = self.malleable;
        p.resource_limitations = self.resource_limit;
        p.repeated_pk = self.repeated_keys;
        p.timelock_mixing = self.mixed_timelock;
        p
    }
}

/// The never-silent surface (allow SPEC §3), **`build-descriptor` wording**:
/// an unmissable stderr warning for every allowed rule that FIRED (all output
/// modes, `--json` included), plus a note for each requested allowance that did
/// not fire.
///
/// `build-descriptor` runs all five rules on every artifact, so its
/// did-not-fire note may legitimately say "the policy passes that rule without
/// it" — the rule really ran.
pub(crate) fn emit_allow_notes<E: Write>(
    requested: &[CliAllow],
    fired: &[DiagnosticKind],
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    if !fired.is_empty() {
        let names: Vec<String> = fired.iter().map(|k| k.as_str().replace('_', "-")).collect();
        writeln!(
            stderr,
            "WARNING: sanity rules OVERRIDDEN by --allow and FIRED: {}. This \
             descriptor failed miniscript's funds-safety analysis; you have \
             accepted that risk after review.",
            names.join(", ")
        )
        .map_err(ToolkitError::Io)?;
    }
    let mut seen: Vec<DiagnosticKind> = Vec::new();
    for a in requested {
        let kind = a.kind();
        if seen.contains(&kind) {
            continue;
        }
        seen.push(kind);
        if !fired.contains(&kind) {
            writeln!(
                stderr,
                "note: --allow {} was requested but did not fire (the policy \
                 passes that rule without it)",
                a.kebab()
            )
            .map_err(ToolkitError::Io)?;
        }
    }
    Ok(())
}
