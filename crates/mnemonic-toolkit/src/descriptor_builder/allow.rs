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
//! One vocabulary, two products:
//!
//! * `build-descriptor` **authors** a descriptor and enforces all five rules on
//!   every artifact it emits. Its wording is *"don't author this"*.
//! * `export-wallet` **re-emits** a descriptor someone else already authored.
//!   It enforces exactly one rule — `sigless-branch` — uniformly across every
//!   wrapper and every arm. Its wording is *"understand what watching this
//!   means"*, and it must never claim a check that did not run.

use std::io::Write;

use clap::ValueEnum;
use miniscript::descriptor::DescriptorPublicKey;
use miniscript::miniscript::analyzable::ExtParams;
use miniscript::Descriptor as MsDescriptor;

use super::gate::{AllowSet, DiagnosticKind};
use crate::error::ToolkitError;
use crate::parse_descriptor::parse_descriptor_lenient;

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

// ======================================================================
// export-wallet side — the admission gate, its per-wrapper fired-detection,
// and its own note wordings.
// ======================================================================

/// The rules `export-wallet` actually enforces (PLAN Phase 1, round-2 ruling
/// **(b)**): `sigless-branch` and nothing else.
///
/// Round 2 deliberately did NOT take option (a) "all five, for true
/// `build-descriptor` parity": that would start refusing every currently
/// exportable `wsh`/`sh` descriptor that is malleable, repeats a key, mixes
/// timelocks or exceeds resource limits — four new refusals nobody asked for on
/// a surface shipped since v0.97.0, each deserving its own evidence and its own
/// decision.
///
/// The vocabulary stays shared (all five values parse); the other four are
/// simply never enforced here, and [`emit_export_allow_notes`] says so rather
/// than leaving a flag that silently does nothing.
pub(crate) fn export_enforces(rule: CliAllow) -> bool {
    match rule {
        CliAllow::SiglessBranch => true,
        CliAllow::Malleable
        | CliAllow::MixedTimelock
        | CliAllow::RepeatedKeys
        | CliAllow::ResourceLimit => false,
    }
}

/// Fired-detection for `sigless-branch`, **per enforced wrapper** (PLAN Phase 1,
/// round-2 finding F-3): per-leaf for `tr`, top-level for `wsh`/`sh`/`bare`.
///
/// This is new code, not reuse (round-1 finding I2). The build side's detector
/// is `Miniscript::<_, Segwitv0>::requires_sig()` on a single `wsh` inner and
/// cannot run on a tapleaf at all — `Miniscript<_, Tap>` is a different type,
/// and tapleaves are exactly the shape Phase 1 unlocks.
///
/// `pkh`/`wpkh` are a bare public key: a signature is structurally required, so
/// they can never be sigless. A `tr()` is judged on its LEAVES — we cannot tell
/// an unspendable NUMS internal key from a real one, and one keyless leaf makes
/// the whole output anyone-can-spend regardless of the key path.
pub(crate) fn descriptor_has_sigless_branch(d: &MsDescriptor<DescriptorPublicKey>) -> bool {
    use miniscript::descriptor::ShInner;
    match d {
        MsDescriptor::Pkh(_) | MsDescriptor::Wpkh(_) => false,
        MsDescriptor::Bare(bare) => !bare.as_inner().requires_sig(),
        MsDescriptor::Wsh(wsh) => !wsh.as_inner().requires_sig(),
        MsDescriptor::Sh(sh) => match sh.as_inner() {
            ShInner::Wsh(wsh) => !wsh.as_inner().requires_sig(),
            ShInner::Wpkh(_) => false,
            ShInner::Ms(ms) => !ms.requires_sig(),
        },
        MsDescriptor::Tr(tr) => tr.leaves().any(|leaf| !leaf.miniscript().requires_sig()),
    }
}

/// `export-wallet`'s single admission gate (PLAN Phase 1, round-3 topology
/// **(B)**, re-stated as a mechanism by round 4).
///
/// Invoked at each of `export-wallet`'s two `EmitInputs` construction sites
/// (`run` and `run_from_import_json`), on that arm's canonical descriptor,
/// honouring the [`AllowSet`]. Uniform: the rule runs on every wrapper and every
/// arm. On the `--template`/`--slot` arm it therefore RUNS and cannot fire — a
/// consequence of the uniform gate, not an exemption from it.
///
/// Returns the allowed rules that actually FIRED, for the never-silent warning.
///
/// **`cmd/restore.rs`'s two `EmitInputs` builders are explicitly out of scope.**
/// The shared pre-`EmitInputs` boundary serves them too, and gating there would
/// silently break a shipped, waiver-less surface: `restore --md1 --format
/// bitcoin-core` on a sigless `wsh` emits flagless at exit 0 and must keep
/// doing so. If that door should be ruled on, it is its own decision with its
/// own release note.
pub(crate) fn export_admission_gate(
    canonical: &str,
    allow: &AllowSet,
) -> Result<Vec<DiagnosticKind>, ToolkitError> {
    let parsed = parse_descriptor_lenient(canonical)
        .map_err(|e| ToolkitError::DescriptorParse(format!("export-wallet admission: {e}")))?;
    let mut fired = Vec::new();
    if descriptor_has_sigless_branch(&parsed) {
        if !allow.sigless_branch {
            return Err(ToolkitError::DescriptorParse(
                "export-wallet: this wallet has a spend path that requires no signature \
                 (anyone-can-spend); rerun with --allow sigless-branch after review. \
                 The flag permits EMISSION of the wallet file — it does not make any \
                 wallet application accept it."
                    .to_string(),
            ));
        }
        fired.push(DiagnosticKind::SiglessBranch);
    }
    Ok(fired)
}

/// The never-silent surface, **`export-wallet` wording** (PLAN Phase 1,
/// round-1 finding M2 and round-3 finding R3-2).
///
/// Two things this must get right, both of which the build-side printer gets
/// wrong on this surface:
///
/// 1. **The act is different.** Build's warning says *don't author this*;
///    export's says *understand what watching this means*.
/// 2. **A note may never claim a check that did not run.** Under ruling (b)
///    four of the five rules never execute here, so the build-side
///    *"(the policy passes that rule without it)"* parenthetical would be a
///    lie for them — the descriptor was not checked at all. Those four get
///    their own wording; only `sigless-branch`, which really runs, keeps a
///    fire-verdict and the "passes that rule" claim.
pub(crate) fn emit_export_allow_notes<E: Write>(
    requested: &[CliAllow],
    fired: &[DiagnosticKind],
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    if !fired.is_empty() {
        let names: Vec<String> = fired.iter().map(|k| k.as_str().replace('_', "-")).collect();
        writeln!(
            stderr,
            "WARNING: sanity rules OVERRIDDEN by --allow and FIRED: {}. This wallet has a \
             spend path that needs no signature — anyone who learns the descriptor can \
             move the funds. You asked to emit a watch-only file for it anyway, after \
             review.",
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
        if !export_enforces(*a) {
            // The rule NEVER RAN. Say only that, and say it the same way on
            // every arm — row 2 of the note matrix has no arm dimension.
            writeln!(
                stderr,
                "note: --allow {rule} has no effect on export-wallet \u{2014} only \
                 sigless-branch is enforced here; the descriptor was NOT checked against \
                 {rule}",
                rule = a.kebab()
            )
            .map_err(ToolkitError::Io)?;
        } else if !fired.contains(&kind) {
            // The rule ran and did not fire, so "passes" is a true claim.
            writeln!(
                stderr,
                "note: --allow {} was requested but did not fire (the descriptor passes \
                 that rule without it)",
                a.kebab()
            )
            .map_err(ToolkitError::Io)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod export_allow_tests {
    use super::*;
    use std::str::FromStr;

    const K: &str = "[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";
    const K2: &str = "[22222222/48h/0h/0h/2h]xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
    const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";
    const H: &str = "4743d7c47df21d29e3ed3dfec5d0c0a884ccc2708637dddf771c36d214056954";

    /// The enforced-rule partition IS ruling (b): exactly one rule, and it is
    /// `sigless-branch`. Driven off `value_variants()` so a sixth rule (or a
    /// quiet flip of `export_enforces`) breaks this test rather than silently
    /// widening what `export-wallet` refuses.
    #[test]
    fn export_enforces_exactly_sigless_branch() {
        let enforced: Vec<&str> = CliAllow::value_variants()
            .iter()
            .filter(|a| export_enforces(**a))
            .map(|a| a.kebab())
            .collect();
        assert_eq!(enforced, vec!["sigless-branch"]);
    }

    /// The leniency is scoped to the sanity check and nothing else: the strict
    /// parse refuses this exact string today, and the lenient one admits it.
    #[test]
    fn lenient_parse_admits_a_keyless_tapleaf_that_strict_parse_refuses() {
        let d = format!("tr({NUMS},and_v(v:after(1383520),sha256({H})))");
        assert!(
            MsDescriptor::<DescriptorPublicKey>::from_str(&d).is_err(),
            "strict parse must still refuse a keyless tapleaf (the pre-Phase-1 behaviour)"
        );
        let lenient = parse_descriptor_lenient(&d).expect("lenient parse admits it");
        assert!(descriptor_has_sigless_branch(&lenient));
    }

    #[test]
    fn lenient_parse_still_rejects_malformed_input() {
        assert!(parse_descriptor_lenient("wsh(").is_err());
        assert!(parse_descriptor_lenient("not-a-descriptor").is_err());
        assert!(parse_descriptor_lenient("wsh(pk(deadbeef))").is_err());
    }

    /// The BIP-380 checksum is still verified — `expression::Tree::from_str`
    /// does it before any node is built, so leniency does not open a
    /// transcription hole.
    #[test]
    fn lenient_parse_still_verifies_the_bip380_checksum() {
        let parsed = parse_descriptor_lenient(&format!("wsh(pk({K}))")).unwrap();
        let with_csum = parsed.to_string();
        assert!(parse_descriptor_lenient(&with_csum).is_ok());
        let (body, csum) = with_csum
            .rsplit_once('#')
            .expect("Display appends a checksum");
        let flipped = format!(
            "{body}#{}{}",
            if csum.starts_with('q') { 'p' } else { 'q' },
            &csum[1..]
        );
        assert!(
            parse_descriptor_lenient(&flipped).is_err(),
            "a corrupted checksum must still refuse: {flipped}"
        );
    }

    /// Per-wrapper fired-detection (F-3). The `wsh`/`sh` cells are the silent
    /// hole the plan closes; the `tr` cells are the shape the build side
    /// structurally could not reach.
    #[test]
    fn sigless_detection_is_per_wrapper() {
        let keyless = format!("and_v(v:after(1383520),sha256({H}))");
        let cases: Vec<(String, bool)> = vec![
            (format!("wsh(pk({K}))"), false),
            (format!("wsh({keyless})"), true),
            (format!("wsh(or_i(pk({K}),{keyless}))"), true),
            (format!("sh(wsh(pk({K})))"), false),
            (format!("sh(wsh({keyless}))"), true),
            (format!("sh(pk({K}))"), false),
            (format!("wpkh({K})"), false),
            (format!("pkh({K})"), false),
            (format!("tr({NUMS},pk({K}))"), false),
            (format!("tr({NUMS},{keyless})"), true),
            // One keyed leaf AND one keyless leaf: one bad leaf poisons the output.
            (format!("tr({NUMS},{{pk({K}),{keyless}}})"), true),
            (format!("tr({NUMS},{{pk({K}),pk({K2})}})"), false),
            // Key-path-only tr has no leaves, so no keyless leaf.
            (format!("tr({NUMS})"), false),
        ];
        for (desc, want) in cases {
            let d = parse_descriptor_lenient(&desc).unwrap_or_else(|e| panic!("{desc}: {e}"));
            assert_eq!(
                descriptor_has_sigless_branch(&d),
                want,
                "sigless detection for {desc}"
            );
        }
    }

    /// The gate refuses flagless and admits with the flag, on both enforced
    /// wrappers — and reports the fired rule so the warning can name it.
    #[test]
    fn gate_refuses_flagless_and_admits_with_the_flag() {
        let keyless = format!("and_v(v:after(1383520),sha256({H}))");
        for d in [
            format!("wsh({keyless})"),
            format!("sh(wsh({keyless}))"),
            format!("tr({NUMS},{keyless})"),
        ] {
            let err = export_admission_gate(&d, &AllowSet::default()).unwrap_err();
            assert!(
                err.to_string().contains("--allow sigless-branch"),
                "refusal must name the flag: {err}"
            );
            let allow = allow_set(&[CliAllow::SiglessBranch]);
            let fired = export_admission_gate(&d, &allow).expect("admitted with the flag");
            assert_eq!(fired, vec![DiagnosticKind::SiglessBranch], "{d}");
        }
    }

    /// A sane descriptor is admitted with no flag AND reports nothing fired,
    /// so the did-not-fire note is reachable.
    #[test]
    fn gate_admits_a_sane_descriptor_and_reports_nothing_fired() {
        let d = format!("wsh(multi(2,{K},{K2}))");
        assert_eq!(
            export_admission_gate(&d, &AllowSet::default()).unwrap(),
            Vec::<DiagnosticKind>::new()
        );
        let allow = allow_set(&[CliAllow::SiglessBranch]);
        assert_eq!(
            export_admission_gate(&d, &allow).unwrap(),
            Vec::<DiagnosticKind>::new()
        );
    }

    /// Over-admission guard (round-1 finding I4), at the unit level: requesting
    /// any of the other four rules must not admit a sigless descriptor.
    #[test]
    fn allowing_another_rule_does_not_admit_a_sigless_branch() {
        let d = format!("wsh(and_v(v:after(1383520),sha256({H})))");
        for other in [
            CliAllow::Malleable,
            CliAllow::MixedTimelock,
            CliAllow::RepeatedKeys,
            CliAllow::ResourceLimit,
        ] {
            let set = allow_set(&[other]);
            assert!(
                export_admission_gate(&d, &set).is_err(),
                "--allow {} must not admit a sigless branch",
                other.kebab()
            );
        }
    }

    /// R3-2 at the printer level: the "passes that rule" parenthetical, and any
    /// fire-verdict at all, belong only to a rule that ran.
    #[test]
    fn unenforced_rules_never_claim_a_verdict() {
        for other in [
            CliAllow::Malleable,
            CliAllow::MixedTimelock,
            CliAllow::RepeatedKeys,
            CliAllow::ResourceLimit,
        ] {
            let mut buf = Vec::new();
            emit_export_allow_notes(&[other], &[], &mut buf).unwrap();
            let s = String::from_utf8(buf).unwrap();
            assert!(
                !s.contains("passes that rule"),
                "{}: must not claim a check that never ran: {s}",
                other.kebab()
            );
            assert!(!s.contains("did not fire"), "{}: {s}", other.kebab());
            assert!(s.contains("has no effect on export-wallet"), "{s}");
            assert!(s.contains("was NOT checked against"), "{s}");
        }
        // The one rule that DOES run may report a verdict.
        let mut buf = Vec::new();
        emit_export_allow_notes(&[CliAllow::SiglessBranch], &[], &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("did not fire"), "{s}");
        assert!(s.contains("passes that rule"), "{s}");
    }

    /// The export warning must speak the export act. Pinned against the build
    /// wording so a future edit cannot quietly collapse the two back together.
    #[test]
    fn export_and_build_warnings_are_distinct() {
        let fired = [DiagnosticKind::SiglessBranch];
        let mut e = Vec::new();
        emit_export_allow_notes(&[CliAllow::SiglessBranch], &fired, &mut e).unwrap();
        let export = String::from_utf8(e).unwrap();
        let mut b = Vec::new();
        emit_allow_notes(&[CliAllow::SiglessBranch], &fired, &mut b).unwrap();
        let build = String::from_utf8(b).unwrap();
        assert_ne!(export, build);
        assert!(export.contains("anyone who learns the descriptor can move the funds"));
        assert!(!export.contains("failed miniscript's funds-safety analysis"));
        assert!(build.contains("failed miniscript's funds-safety analysis"));
        // Both name the rule the same way — one vocabulary.
        assert!(export.contains("sigless-branch") && build.contains("sigless-branch"));
    }
}
