//! Coordinator-compatibility plan 1b: verdicts, the rule/evidence build, and
//! the conformance gate over EVIDENCE rows.
//!
//! Every test names the mutation that reds it.

mod common;

use md_codec::chunk::{reassemble, split};
use md_codec::coordinator::build::{Disagreement, EvidenceRow, MeasuredOutcome, build};
use md_codec::coordinator::{
    CoordinatorVerdict, Form, REGISTRY, UnprovenReason, Verdict, none_imports, verdicts,
};
use md_codec::descriptor_route::descriptor_from_text;
use md_codec::encode::Descriptor;
use md_codec::skeleton::{Skeleton, skeleton, skeleton_key};

fn evidence() -> Vec<serde_json::Value> {
    let p = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/coordinator/evidence.jsonl"
    );
    std::fs::read_to_string(p)
        .unwrap_or_else(|e| panic!("{p}: {e}"))
        .lines()
        .map(|l| serde_json::from_str(l).expect("evidence row parses"))
        .collect()
}

/// The descriptor of the first evidence row named `name` from `coordinator`.
fn descriptor_named(coordinator: &str, name: &str) -> String {
    evidence()
        .into_iter()
        .find(|r| r["coordinator"] == coordinator && r["name"] == name)
        .unwrap_or_else(|| panic!("no {coordinator} evidence row named {name}"))["descriptor"]
        .as_str()
        .unwrap()
        .to_string()
}

fn keyed(text: &str) -> (Descriptor, Skeleton) {
    let d = descriptor_from_text(text).unwrap_or_else(|e| panic!("{e}: {text}"));
    let s = skeleton(&d).unwrap_or_else(|e| panic!("{e}"));
    (d, s)
}

/// The same policy with its key material removed: a template-only card.
fn template_of(d: &Descriptor) -> Skeleton {
    let mut t = d.clone();
    t.tlv.pubkeys = None;
    t.tlv.fingerprints = None;
    skeleton(&t).unwrap_or_else(|e| panic!("template: {e}"))
}

fn of<'a>(vs: &'a [CoordinatorVerdict], id: &str) -> Vec<&'a Verdict> {
    vs.iter()
        .filter(|v| v.coordinator.id.0 == id)
        .map(|v| &v.verdict)
        .collect()
}

fn span_of(v: &Verdict) -> String {
    match v {
        Verdict::Refuses { span, .. }
        | Verdict::ImportsAltered { span, .. }
        | Verdict::Imports { span, .. } => span.to_string(),
        Verdict::Unproven { span, .. } => span.map_or(String::new(), |s| s.to_string()),
    }
}

/// Design §2's gate, over EVIDENCE rows rather than the vector corpus
/// (the recon found 0 of 11 Liana live-gate rows and 1 of 56 matrix shapes in
/// that corpus): every keyable row keys the same through md1 chunks as
/// through its descriptor text. Mutation: make `is_liana_unspendable_key`
/// return false -> every kind-1 row fails to key and this reds.
#[test]
fn every_evidence_row_keys_identically_through_chunks_and_text() {
    let mut checked = 0;
    for r in evidence() {
        if r.get("unkeyable").is_some() {
            continue;
        }
        let id = r["id"].as_str().unwrap();
        let (d, s) = keyed(r["descriptor"].as_str().unwrap());
        let chunks = split(&d).unwrap_or_else(|e| panic!("{id}: split: {e}"));
        let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
        let back = reassemble(&refs).unwrap_or_else(|e| panic!("{id}: reassemble: {e}"));
        let s2 = skeleton(&back).unwrap_or_else(|e| panic!("{id}: {e}"));
        assert_eq!(
            skeleton_key(&s),
            skeleton_key(&s2),
            "{id}: the key knows its route"
        );
        checked += 1;
    }
    assert!(checked >= 524, "only {checked} evidence rows keyed");
}

/// Every verified version has exactly one rule set. Mutation: shrink Core's
/// 29.4-31.1 span to 29.4-30.3 -> 31.1 has none and this reds.
#[test]
fn every_verified_version_has_exactly_one_rule_set() {
    for c in REGISTRY {
        for v in c.verified {
            let at = c.position(v).unwrap();
            let n = c
                .rules
                .iter()
                .filter(|r| {
                    let (a, b) = (c.position(r.span.since.0), c.position(r.span.until.0));
                    matches!((a, b), (Some(a), Some(b)) if a <= at && at <= b)
                })
                .count();
            assert_eq!(n, 1, "{} {v}: {n} rule sets", c.name);
        }
    }
}

/// Ruling R-1, end to end. The recon measured two kind-1 cards whose
/// SkeletonKeys are byte-identical and whose Nunchuk verdicts are opposite;
/// the key stays coordinator-independent and the rule reads key material.
/// Mutation: flip the Nunchuk R-1 clause to `== Some(true)` -> the sorted
/// card is refused and the unsorted one is silent; this reds (and so does
/// the xtask's `table_is_fresh`, on a D1).
#[test]
fn r1_leaf_order_splits_one_key_into_two_nunchuk_verdicts() {
    let (_, unsorted) = keyed(&descriptor_named("nunchuk", "k1-liana-unsorted"));
    let (d, sorted) = keyed(&descriptor_named("nunchuk", "k1-liana-sorted"));
    assert_eq!(
        skeleton_key(&unsorted),
        skeleton_key(&sorted),
        "the recon's collision"
    );
    assert_eq!(unsorted.leaf_keys_ascending, Some(false));
    assert_eq!(sorted.leaf_keys_ascending, Some(true));

    let u = verdicts(&unsorted, Some(Form::Multipath));
    assert!(
        matches!(
            of(&u, "nunchuk")[..],
            [Verdict::Refuses { renderer: None, .. }]
        ),
        "unsorted: a rule-derived refusal, {:?}",
        of(&u, "nunchuk")
    );
    let s = verdicts(&sorted, Some(Form::Multipath));
    assert!(
        matches!(of(&s, "nunchuk")[..], [Verdict::Imports { .. }]),
        "sorted: the measured positive, {:?}",
        of(&s, "nunchuk")
    );
    let t = verdicts(&template_of(&d), None);
    assert!(
        matches!(
            of(&t, "nunchuk")[..],
            [Verdict::Unproven {
                reason: UnprovenReason::KeysAbsent,
                ..
            }]
        ),
        "a template: {:?}",
        of(&t, "nunchuk")
    );
}

/// The Core boundary, MEASURED, prints as closed runs of verified versions.
/// Per-chain spelling: refused through 25.2, imported 26.0-31.1. Multipath:
/// refused through 28.4 for the spelling. After that the boundary probe only
/// PARSED it, but this control shares its SkeletonKey with the e2e run's
/// `kofn-liana-distinct` wallet (F-673), which Core 29.4 and 31.1 IMPORTED in
/// the multipath form -- so those two read `Imports` and 30.3, never
/// measured, stays silent. Mutation: drop `CORE_MULTIPATH` from the 26.0-28.4
/// rule set -> the multipath runs change and this reds.
#[test]
fn core_verdicts_follow_the_measured_boundary() {
    let (_, s) = keyed(&descriptor_named(
        "core",
        "CONTROL-accept-kofn-recovery-flat",
    ));
    let c0 = verdicts(&s, Some(Form::Chain0));
    let core: Vec<_> = of(&c0, "core");
    assert_eq!(core.len(), 2, "{core:?}");
    assert!(
        matches!(core[0], Verdict::Refuses { reason, renderer: None, .. } if reason.class == "miniscript under tr")
    );
    assert_eq!(span_of(core[0]), "24.2-25.2");
    assert!(matches!(core[1], Verdict::Imports { .. }));
    assert_eq!(span_of(core[1]), "26.0-31.1");

    let mp = verdicts(&s, Some(Form::Multipath));
    let core: Vec<_> = of(&mp, "core");
    let spans: Vec<String> = core.iter().map(|v| span_of(v)).collect();
    assert_eq!(spans, ["24.2-28.4", "29.4", "30.3", "31.1"], "{core:?}");
    assert!(matches!(core[0], Verdict::Refuses { reason, .. } if reason.class.contains("<0;1>")));
    assert!(matches!(core[1], Verdict::Imports { .. }));
    assert!(matches!(
        core[2],
        Verdict::Unproven {
            reason: UnprovenReason::NoEvidence,
            ..
        }
    ));
    assert!(matches!(core[3], Verdict::Imports { .. }));
}

/// F-673: the e2e live-site run (engrave `coord-compat-e2e/`) imported all 8
/// composer `tr` wallets -- kofn- and tiered-recovery, NUMS and Liana key,
/// demo and distinct seating -- on Core 29.4 and 31.1 in the MULTIPATH form,
/// with wallet addresses equal to the device's. Each now reads `Imports` at
/// exactly those two releases. 30.3 sits between them and was never measured
/// on these shapes, so it stays `Unproven`: a run never spans a version that
/// was not measured. The template of each is still `KeysAbsent` (a2).
/// Mutation: drop the vendoring script's `coord-compat-e2e` block and
/// re-vendor -> the 29.4 and 31.1 cells vanish and this reds.
#[test]
fn core_imports_the_composer_tr_wallets_only_where_measured() {
    for name in [
        "kofn-nums",
        "kofn-liana",
        "tiered-nums",
        "tiered-liana",
        "kofn-nums-distinct",
        "kofn-liana-distinct",
        "tiered-nums-distinct",
        "tiered-liana-distinct",
    ] {
        let (d, s) = keyed(&descriptor_named("core", name));
        let mp = verdicts(&s, Some(Form::Multipath));
        let core: Vec<_> = of(&mp, "core");
        let spans: Vec<String> = core.iter().map(|v| span_of(v)).collect();
        assert_eq!(spans, ["24.2-28.4", "29.4", "30.3", "31.1"], "{name}");
        assert!(
            matches!(core[0], Verdict::Refuses { reason, .. } if reason.class.contains("<0;1>")),
            "{name}: {:?}",
            core[0]
        );
        for i in [1, 3] {
            assert!(
                matches!(core[i], Verdict::Imports { renderer, .. }
                    if renderer.tool == "md" && renderer.version == "0.20.0"
                        && renderer.form == Form::Multipath),
                "{name}: {:?}",
                core[i]
            );
        }
        assert!(
            matches!(
                core[2],
                Verdict::Unproven {
                    reason: UnprovenReason::NoEvidence,
                    ..
                }
            ),
            "{name}: {:?}",
            core[2]
        );
        let t = verdicts(&template_of(&d), None);
        assert!(
            of(&t, "core")
                .iter()
                .all(|v| !matches!(v, Verdict::Imports { .. } | Verdict::ImportsAltered { .. })),
            "{name}: a template claimed an import"
        );
    }
}

/// Design §1 (a2): a template can be refused, never claimed to import.
/// X24 is imported by Liana and by Core when keyed. Two templates of it must
/// claim nothing: the real one (whose partitions render EMPTY, so its key
/// differs from the seated card's -- measured here, see the design fold) and
/// the seated Skeleton with only `keys_present` cleared, which keeps the
/// seated key and so would find the measured cells if the guard were gone.
/// Mutation: delete `at_version`'s `if !s.keys_present` return -> the second
/// template reads `Imports` and this reds.
#[test]
fn a_template_is_never_claimed_to_import() {
    let (d, keyed_s) = keyed(&descriptor_named(
        "liana",
        "X24-wsh-2of3-1of1-unlocked-plus-rec",
    ));
    let real = template_of(&d);
    assert_ne!(
        skeleton_key(&real),
        skeleton_key(&keyed_s),
        "a template's partitions are empty"
    );
    let mut cleared = keyed_s.clone();
    cleared.keys_present = false;
    assert_eq!(
        skeleton_key(&cleared),
        skeleton_key(&keyed_s),
        "keys_present is not in the key"
    );
    for t in [real, cleared] {
        for form in [None, Some(Form::Multipath), Some(Form::Chain0)] {
            for v in verdicts(&t, form) {
                assert!(
                    !matches!(
                        v.verdict,
                        Verdict::Imports { .. } | Verdict::ImportsAltered { .. }
                    ),
                    "{form:?}: {} claimed a positive for a template",
                    v.coordinator.name
                );
            }
        }
    }
    // A structure-only refusal still stands without keys.
    let (d, _) = keyed(&descriptor_named("liana", "preset-hashlock-gated-wsh"));
    let t = verdicts(&template_of(&d), None);
    assert!(matches!(
        of(&t, "liana")[..],
        [Verdict::Refuses { reason, .. }] if reason.class == "a hash lock"
    ));
}

/// Design §1 (c): measured, "accepts, but not as built". X24 (Liana reads
/// 2-of-3 plus 1-of-1 as 2-of-4) and F-626 (Nunchuk reads an unsorted 2-of-3
/// as MINISCRIPT). The runtime reads the GENERATED table, so a mutation of
/// `liana_reads_as_built` (return true) reds the xtask's `table_is_fresh`
/// first; after `cargo xtask verdicts` regenerates, X24 is a plain `Imports`
/// and this reds.
#[test]
fn imports_altered_is_derived_from_the_coordinators_own_reading() {
    let (_, x24) = keyed(&descriptor_named(
        "liana",
        "X24-wsh-2of3-1of1-unlocked-plus-rec",
    ));
    let v = verdicts(&x24, Some(Form::Multipath));
    // Two runs, not one "8.0-15.0": a run carries ONE measurement date, and
    // Liana 8.0 was measured 2026-09-19, 15.0 on 2026-09-20 (F-674 item 3).
    let liana = of(&v, "liana");
    assert_eq!(liana.len(), 2, "{liana:?}");
    for (w, (span, date)) in liana
        .iter()
        .zip([("8.0", "2026-09-19"), ("15.0", "2026-09-20")])
    {
        assert!(
            matches!(w, Verdict::ImportsAltered { as_read, measured_at, .. }
                if as_read.threshold == Some((2, 4)) && measured_at.0 == date),
            "{w:?}"
        );
        assert_eq!(span_of(w), span);
    }

    let (_, unsorted) = keyed(&descriptor_named("nunchuk", "plain-2of3-wsh-UNSORTED"));
    let v = verdicts(&unsorted, Some(Form::Multipath));
    assert!(
        matches!(of(&v, "nunchuk")[..], [Verdict::ImportsAltered { as_read, .. }] if as_read.wallet_kind == "MINISCRIPT"),
        "{:?}",
        of(&v, "nunchuk")
    );
}

/// Liana's class 9 boundary, `k >= 2`, pinned in BOTH directions (plan 1b
/// r0 I-1). Liana's normaliser flattens a second unlocked 1-of-n into bare
/// keys and folds them into the primary (measured: `cx-multi-then-1of2multi`
/// imported as 2-of-4); a second unlocked 2-of-2 stays a threshold and is
/// refused (`cx-single-then-multi`). Checked keyed AND as a template, because
/// `md compose` reaches it keylessly. Mutations (both measured): `b.k >= 1`
/// -> the 1-of-2 row is refused and this reds (the table build reds too, on a
/// D1); `b.k >= 3` -> the 2-of-2 row is not refused and this reds.
#[test]
fn liana_class_9_refuses_a_second_threshold_only_from_k_2() {
    const CLASS: &str = "a second unlocked k-of-n path with k >= 2";
    let refuses_9 = |vs: &[CoordinatorVerdict]| matches!(of(vs, "liana")[..], [Verdict::Refuses { reason, .. }] if reason.class == CLASS);
    let (d, folded) = keyed(&descriptor_named("liana", "cx-multi-then-1of2multi"));
    assert_eq!(
        folded.shape.branches[1].k, 1,
        "precondition: the second path is 1-of-2"
    );
    // Measured at 15.0 only, so 8.0 is silent; no run refuses.
    let v = verdicts(&folded, Some(Form::Multipath));
    let liana = of(&v, "liana");
    assert!(
        liana.iter().any(|x| matches!(x, Verdict::ImportsAltered { as_read, .. } if as_read.threshold == Some((2, 4))))
            && !liana.iter().any(|x| matches!(x, Verdict::Refuses { .. })),
        "{liana:?}"
    );
    assert!(!refuses_9(&verdicts(&template_of(&d), None)));

    let (d, refused) = keyed(&descriptor_named("liana", "cx-single-then-multi"));
    assert_eq!(
        refused.shape.branches[1].k, 2,
        "precondition: the second path is 2-of-2"
    );
    assert!(refuses_9(&verdicts(&refused, Some(Form::Multipath))));
    assert!(refuses_9(&verdicts(&template_of(&d), None)));
}

/// Ruling 2's none case: every coordinator refuses at every verified
/// version. A keyless `wsh` path is refused by all three from source; a
/// template that some coordinators refuse and others are merely silent on is
/// not "none". Mutation: `none_imports` using `any` -> the kofn `tr` template
/// reads as none and this reds.
#[test]
fn none_imports_only_when_every_run_refuses() {
    let (d, _) = keyed(&descriptor_named("liana", "keyless-hash-path-wsh"));
    let t = verdicts(&template_of(&d), None);
    assert!(none_imports(&t), "{t:?}");

    // Some runs refuse (Liana: NUMS key path; Core 24.2-25.2: miniscript
    // under tr), others are silent: not "none".
    let (d, _) = keyed(&descriptor_named("liana", "preset-kofn-recovery-tr"));
    let t = verdicts(&template_of(&d), None);
    assert!(
        t.iter()
            .any(|v| matches!(v.verdict, Verdict::Refuses { .. })),
        "precondition: {t:?}"
    );
    assert!(!none_imports(&t), "{t:?}");
}

fn row(name: &str, coordinator: &str, version: &str, outcome: MeasuredOutcome) -> EvidenceRow {
    EvidenceRow {
        id: format!("synthetic:{name}"),
        coordinator: coordinator.into(),
        version: version.into(),
        library_rev: "test".into(),
        renderer_tool: "test".into(),
        renderer_version: "0".into(),
        form: Form::Multipath,
        measured_at: "2026-09-23".into(),
        descriptor: descriptor_named("liana", name),
        outcome,
        unkeyable: None,
    }
}

/// The build fails on every disagreement class, each for its own reason.
/// Mutation: delete the D5 comparison in `build` -> the conflicting pair is
/// silently deduplicated and this reds.
#[test]
fn the_build_names_every_disagreement_class() {
    use MeasuredOutcome::{Imported, Refused};
    let generic = || Refused("Descriptor is not compatible with a Liana spending policy.".into());
    let cases: Vec<(EvidenceRow, &str)> = vec![
        // single-wsh has no locked path: a Liana rule refuses it.
        (row("single-wsh", "liana", "15.0", Imported(None)), "D1"),
        // kofn-recovery-wsh is Liana's own shape: no rule refuses it.
        (
            row("preset-kofn-recovery-wsh", "liana", "15.0", generic()),
            "D2",
        ),
        // older-units: the rule says time units; this message names another class.
        (
            row(
                "older-units-wsh",
                "liana",
                "15.0",
                Refused("A Liana policy requires at least one recovery path.".into()),
            ),
            "D3",
        ),
        (
            row("preset-kofn-recovery-wsh", "liana", "9.9", Imported(None)),
            "D4",
        ),
    ];
    for (r, want) in cases {
        let got = build(std::slice::from_ref(&r)).err().unwrap_or_default();
        let class = match got.as_slice() {
            [Disagreement::D1FalseRefusal { .. }] => "D1",
            [Disagreement::D2MissedRefusal { .. }] => "D2",
            [Disagreement::D3ReasonDrift { .. }] => "D3",
            [Disagreement::D4Orphan { .. }] => "D4",
            other => panic!("{}: {other:?}", r.id),
        };
        assert_eq!(class, want, "{}", r.id);
    }
    // D5: Nunchuk admits measured refusals, so two rows for one cell can
    // disagree only with each other.
    let a = row(
        "preset-kofn-recovery-wsh",
        "nunchuk",
        "2.1.1",
        Imported(None),
    );
    let mut b = a.clone();
    b.outcome = Refused("ParseWalletDescriptor=REFUSE".into());
    b.id = "synthetic:b".into();
    let got = build(&[a, b]).err().unwrap_or_default();
    assert!(
        matches!(got[..], [Disagreement::D5EvidenceConflict { .. }]),
        "{got:?}"
    );
    // A row declared unkeyable must really be unkeyable.
    let mut k = row("preset-kofn-recovery-wsh", "liana", "15.0", Imported(None));
    k.unkeyable = Some("claimed".into());
    let got = build(&[k]).err().unwrap_or_default();
    assert!(matches!(got[..], [Disagreement::Keying { .. }]), "{got:?}");
}

/// F-674 item 3: every row's `measured_at` is the local date its measurement
/// was TAKEN, as each source's own report states it -- not the date of the
/// engrave commit that recorded the line, in UTC. The Core e2e run
/// (agent-reports/e2e-live-site-wallets.md:3, "Date 2026-09-23") was
/// committed at 20:09 -0700 and printed "measured 2026-09-24"; the Liana 8.0
/// and Nunchuk runs of 2026-09-19 were copied in after midnight and printed
/// 2026-09-20. The dates come from engrave's design/evidence/measured-at.json.
/// Mutation: vendor with the old git-blame rule -> the e2e, Liana 8.0 and
/// Nunchuk rows red.
#[test]
fn measured_at_is_the_date_each_source_states() {
    let stated: &[(&str, &str)] = &[
        (
            "composer-fable-r0/fable-liana-parse-out.jsonl:",
            "2026-09-19",
        ),
        (
            "composer-fable-r0/fable-nunchuk-harness-out.txt:",
            "2026-09-19",
        ),
        (
            "composer-fable-r0/fable-liana-parse-out-v15.jsonl:",
            "2026-09-20",
        ),
        ("coord-compat-1b/", "2026-09-23"),
        ("coord-compat-core-boundary/", "2026-09-23"),
        ("coord-compat-e2e/core-composer-tr.out:", "2026-09-23"),
        ("f449-stage2/", "2026-09-23"),
        ("f449-stage4/", "2026-09-23"),
    ];
    let rows = evidence();
    assert!(!rows.is_empty());
    let mut per_source = std::collections::BTreeMap::<&str, usize>::new();
    for r in &rows {
        let id = r["id"].as_str().unwrap();
        let at = r["measured_at"].as_str().unwrap();
        // F-640's record was appended to the v15.0 file three days later.
        let want = if r["name"] == "nested-2of2-two-recoveries-tr"
            && id.starts_with("composer-fable-r0/fable-liana-parse-out-v15.jsonl:")
        {
            "2026-09-23"
        } else {
            let (p, d) = stated
                .iter()
                .find(|(p, _)| id.starts_with(p))
                .unwrap_or_else(|| panic!("row {id}: no stated date for its source"));
            *per_source.entry(p).or_default() += 1;
            d
        };
        assert_eq!(at, want, "row {id}");
    }
    // Every source above is present, so no expectation is vacuous.
    assert_eq!(per_source.len(), stated.len(), "{per_source:?}");
}
