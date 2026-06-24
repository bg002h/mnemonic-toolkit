//! Taproot use-site-override classification predicates (#25/#26).
//!
//! The SINGLE source of the two predicates that partition a taproot
//! use-site-override md1 card into {faithfully-reconstructable} vs
//! {loud-refuse + engrave-advisory}. They live HERE — a tiny `md_codec`-only
//! leaf module — rather than in `cmd::restore`, so BOTH the binary's
//! `cmd::restore` (the restore guard + the classify-reroute) AND
//! `unrestorable_advisory` (the engrave advisory, which is mounted into the LIB
//! crate under `cfg(fuzzing)` where `cmd` is absent) can reach them via one
//! crate-root path. Co-locating them in `cmd::restore` previously broke
//! `cargo fuzz build` (E0433: `unrestorable_advisory` could not resolve
//! `crate::cmd::restore::…` under `cfg(fuzzing)`) — FOLLOWUP
//! `fuzz-build-broken-unrestorable-advisory-references-bin-only-cmd`.
//!
//! Both functions are PURE structural reads of the on-the-wire
//! `md_codec::Descriptor` (tree tag + body + TLV); no toolkit-internal deps.
//! Sharing one expression is load-bearing: the advisory fires IFF restore
//! refuses (`taproot_override_card && !restorable_taproot_override_card`) — a
//! single source guarantees that parity cannot drift.

/// Whether `d` is a taproot card carrying per-`@N` use-site overrides — the
/// BLANKET predicate (#25). The RESTORABLE subset of these (non-hardened
/// `tr(NUMS, multi_a)`) is carved out by `restorable_taproot_override_card`
/// (#26) and reconstructs faithfully via the per-`@N` multipath builder; the
/// REMAINDER (a `sortedmulti_a` tap leaf, a non-NUMS internal/trunk key, or a
/// hardened use-site) routes around the faithful per-`@N` path and would
/// mis-render, so it stays REFUSED (FOLLOWUP
/// `restore-md1-taproot-use-site-override-arm`). This blanket predicate is the
/// base term shared by the restore guard (P2.3) and the engrave advisory (P2.4,
/// `unrestorable_advisory.rs`), each of which subtracts `restorable_…` so the
/// advisory fires IFF restore refuses (exact parity).
pub(crate) fn taproot_override_card(d: &md_codec::Descriptor) -> bool {
    matches!(d.tree.tag, md_codec::Tag::Tr) && d.tlv.use_site_path_overrides.is_some()
}

/// The RESTORABLE subset of `taproot_override_card`: a taproot override card the
/// toolkit CAN now reconstruct faithfully per-`@N` (#26). This is the SINGLE
/// source shared VERBATIM by the restore guard (P2.3, `restore.rs`), the
/// classify-reroute (P2.2, the `Template` arm at the sole `classify_taproot_restore`
/// caller), AND the engrave advisory (P2.4, `unrestorable_advisory.rs`) — so
/// guard-admits ⟺ classify-reroutes ⟺ advisory-silent (single expression ⇒ exact
/// parity; mirrors #25's hardened/override parity). The four conjuncts:
/// 1. `taproot_override_card(d)` — `Tag::Tr` root ∧ `use_site_path_overrides`.
/// 2. NUMS internal key (D7 — a real/non-NUMS trunk key is out of scope; the
///    `@-in-both` and per-`@N` non-NUMS internal cases are not yet covered).
/// 3. the sole tap-script leaf is a PLAIN `MultiA` (NOT `SortedMultiA` — md-codec
///    0.37.0 still hard-`Err`s `SortedMultiA` as a non-root tap leaf; that leg
///    rides the `taproot-coverage-cycle-on-miniscript-gt-13-1-0` umbrella).
/// 4. NO hardened use-site anywhere (`/*h` or a hardened multipath alt) — watch-only
///    cannot derive hardened (#25 Point B, reused verbatim).
///
/// Conjuncts 2+3 are read off the wire tree using the EXACT `Body::Tr { is_nums,
/// tree: Some(inner), .. }` destructure `classify_taproot_restore` uses, so the
/// predicate's NUMS/leaf read CANNOT diverge from classify (R0 Min-B). A
/// `tree: None` (keypath-only tr) or non-`Body::Tr` body yields `false`.
pub(crate) fn restorable_taproot_override_card(d: &md_codec::Descriptor) -> bool {
    use md_codec::tree::Body;
    if !taproot_override_card(d) {
        return false;
    }
    if md_codec::to_miniscript::has_hardened_use_site(d) {
        return false;
    }
    match &d.tree.body {
        Body::Tr {
            is_nums: true,
            tree: Some(inner),
            ..
        } => inner.tag == md_codec::Tag::MultiA,
        // Non-NUMS trunk (D7 out of scope), keypath-only tr (`tree: None`), or a
        // non-`Tr` body all fall through to unrestorable.
        _ => false,
    }
}

/// The CUSTOM (divergent per-cosigner) use-site on a NUMS-taproot card — the
/// RESTORABLE `tr(NUMS, multi_a)` override subset that #26/v0.59.1 reconstructs
/// faithfully BUT that no known wallet produces (every standard wallet uses ONE
/// uniform `<0;1>/*` suffix across all cosigners). Cycle Y (v0.73.3) fires a LOUD
/// funds-safety advisory for exactly this shape at engrave AND restore (the
/// reconstruction is UNCHANGED — proceed-and-warn, not refuse).
///
/// This is the EXACT complement of the un-restorable taproot advisory
/// (`taproot_override_card && !restorable_taproot_override_card`,
/// `unrestorable_advisory.rs`): the two are MUTUALLY EXCLUSIVE for any taproot
/// override card, and BASELINE `tr(NUMS, multi_a)` (no `use_site_path_overrides`
/// → `None` → `taproot_override_card == false`) fires NEITHER.
///
/// Single-sourced here so the engrave-surface advisory (`unrestorable_advisory`,
/// reachable under `cfg(fuzzing)`) and the restore-surface advisory
/// (`cmd::restore::run_multisig`) share ONE expression and cannot drift.
pub(crate) fn custom_use_site_nums_taproot_card(d: &md_codec::Descriptor) -> bool {
    taproot_override_card(d) && restorable_taproot_override_card(d)
}

#[cfg(test)]
mod custom_use_site_predicate_tests {
    //! Cycle Y (v0.73.3) truth table for `custom_use_site_nums_taproot_card` —
    //! the LOUD funds-safety advisory trigger. Each `Descriptor` is reassembled
    //! from a REAL md1 card (generated offline via `mnemonic bundle` over the
    //! fixed C0/C1/C2 phrases) through `md_codec::chunk`, the identical wire path
    //! `restore --md1` walks — so the predicate sees exactly the on-the-wire
    //! tree/TLV shape, not a hand-forged literal. The fixtures mirror the
    //! `restorable_taproot_override_card` truth table in `cmd::restore`.
    use super::*;

    fn desc(cards: &[&str]) -> md_codec::Descriptor {
        md_codec::chunk::reassemble(cards).expect("reassemble md1 cards")
    }

    // `tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<2;3>/*))` — divergent override, NUMS
    // internal, plain MultiA leaf, non-hardened → CUSTOM-restorable → TRUE.
    const NUMS_MULTI_A_OVERRIDE: &[&str] = &[
        "md1ffnfjpspq2tvyyyhqqxquszzs95czskp0prnchdq4hp5gmug4cysnmv90d3tcduh4e8ua7fqtvnvzghtrh69g",
        "md1ffnfjps0duhh2nfa2v52y0y447v27zqh7rcvclsqukx9fn0d4jnuw7trxprw9qc4yl7vvxugj6djuy0jmqkf8",
        "md1ffnfjpssp0av5mc5nppyd7f7vmyxulga94lx5z6xnfuus80jjtjml4fkxw84uw2va3dkm04q0zdp57mpar9x8",
        "md1ffnfjps7n8u3lgfqtxzxyq8gqn55sa4xm6pve4c8f78rzhtg8cjktw7aryhcgeak4fvgq7hktsr2xwcx6c",
    ];
    // `tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))` — UNIFORM suffix across all
    // cosigners → `use_site_path_overrides` is `None` → `taproot_override_card`
    // false → CUSTOM predicate FALSE (the load-bearing baseline-silence cell).
    const NUMS_MULTI_A_BASELINE_UNIFORM: &[&str] = &[
        "md1fx4nepspq2tvyyyhqqxquszzshs3eutks2ms6yd7y2uzgfakzhkc4ux7t6un7wlynk7tm5rf06x65jszgka",
        "md1fx4nepsdxn65eg5g7fttuc4uyp0u8se3lqpevv2nx7mt98caukxvzxu2p32fluccdcszl6nfuhyp8k6q3h3",
        "md1fx4nepsn9x79ycgfr0j0nxeph868fd0e4qk35608ypmu5jukla2d3n3a0rjn8vtdkmaf5c3uggmum5zpj87",
        "md1fx4nepslu3lgfqtxzxyq8gqn55sa4xm6pve4c8f78rzhtg8cjktw7aryhcgeak4fvgqft0n9uq8g6dy7",
    ];
    // `tr(NUMS,sortedmulti_a(...))` override — md-codec render gap → un-restorable
    // → CUSTOM predicate FALSE (fires the CALM advisory, not the loud one).
    const NUMS_SORTEDMULTI_A_OVERRIDE: &[&str] = &[
        "md1ftf38pspq2tvyyy4qqxqujzzs95czskp0prnchdq4hp5gmug4cyja6p372zc9gwrh7h9q2hqlafphjqhy6vu7",
        "md1ftf38psvxs7s5yl6smtcxjz9m806prlsm794tkxqs6806lhaeh6reknylagmwyjycf8044xg7stmtpsjl5fdj",
        "md1ftf38psnt9flsdlkvt6f6cthyl98fejsahhtp2x7t365s9qhgfvt63yacv0jzrws489wwl2qv67ruv8vzywrf",
        "md1ftf38psmcse69e0qvuhzq6k5jt8ymyydynzrv4kudj9m56mcqpxckrlzeq339uepg0p7q6gzlrcel29yrh",
    ];
    // `tr(@0,multi_a(...))` override — real (non-NUMS) trunk key (D7) → CUSTOM
    // predicate FALSE.
    const NON_NUMS_MULTI_A_OVERRIDE: &[&str] = &[
        "md1f3sl6zspqjtvyyy5qgjqgtqxnkqqdgzskp0npeutks2dcdzxlrzsezsqc27rchwsv0jskq40meejhx8ptl2",
        "md1f3sl6zsdgwrh7h9q2hyxs7s5yl6smtcxjz9m806prlsm794tkxqs6806lhaeh6reknylagqkr2s9n7c2vsc",
        "md1f3sl6zsndcjgnpya7k5edv487ph7e30f8tpwunu5knn9pm0wkz5duhr4fq2pwsjch4zfmsq6dryjtwrel8g",
        "md1f3sl6zsmrussm59fetnh6s7yxw3wtcr89csx44yjeexeprfycsm9dhrv3waxk7qqfk9slcwmfzgkfetgnvw",
        "md1f3sl6z3zeq339uepg0plpz2zll50ju3dcmghtxtfv0y025ltk2vc8a3ex8yqnc896wtrlv4g04rwua8nzh8",
        "md1f3sl6z3fhqdghjmksz3ry92d3gv4ejtmu9f0zxf3clxvtlnnv86xy4qee32ay5gp9lt69yuy5m4",
    ];
    // `tr(NUMS,multi_a(...,@1/<2;3>/*h))` — hardened alt → un-restorable → CUSTOM
    // predicate FALSE.
    const NUMS_MULTI_A_HARDENED_OVERRIDE: &[&str] = &[
        "md1f36rfpspq2tvyyy4qqxquszzs95czshp0prnchdq4hp5gmug4cyja6p372zc9gwrh7h9q2hqrnqxdtcr2cxyl",
        "md1f36rfpsvxs7s5yl6smtcxjz9m806prlsm794tkxqs6806lhaeh6reknylagmwyjycf8044xgwcc03gsp4pm5n",
        "md1f36rfpsnt9flsdlkvt6f6cthyl98fejsahhtp2x7t365s9qhgfvt63yacv0jzrws489wwl2qujdhx98lg3u6g",
        "md1f36rfpsmcse69e0qvuhzq6k5jt8ymyydynzrv4kudj9m56mcqpxckrlzeq339uepg0p7qk49ams3vfwqr0",
    ];

    #[test]
    fn custom_nums_multi_a_override_is_true() {
        let d = desc(NUMS_MULTI_A_OVERRIDE);
        assert!(taproot_override_card(&d), "must be a taproot override card");
        assert!(
            custom_use_site_nums_taproot_card(&d),
            "non-hardened tr(NUMS,multi_a) divergent override is the CUSTOM-restorable case"
        );
    }

    #[test]
    fn baseline_uniform_nums_multi_a_is_false() {
        let d = desc(NUMS_MULTI_A_BASELINE_UNIFORM);
        assert!(
            !taproot_override_card(&d),
            "uniform <0;1>/* across all cosigners → no use-site overrides → not a card"
        );
        assert!(
            !custom_use_site_nums_taproot_card(&d),
            "BASELINE (uniform) tr(NUMS,multi_a) must NOT fire the loud advisory"
        );
    }

    #[test]
    fn sortedmulti_a_override_is_false() {
        let d = desc(NUMS_SORTEDMULTI_A_OVERRIDE);
        assert!(taproot_override_card(&d), "must be a taproot override card");
        assert!(
            !custom_use_site_nums_taproot_card(&d),
            "sortedmulti_a leaf is un-restorable → loud advisory must NOT fire"
        );
    }

    #[test]
    fn non_nums_trunk_override_is_false() {
        let d = desc(NON_NUMS_MULTI_A_OVERRIDE);
        assert!(taproot_override_card(&d), "must be a taproot override card");
        assert!(
            !custom_use_site_nums_taproot_card(&d),
            "non-NUMS real-trunk internal key is D7 out of scope → loud advisory must NOT fire"
        );
    }

    #[test]
    fn hardened_override_is_false() {
        let d = desc(NUMS_MULTI_A_HARDENED_OVERRIDE);
        assert!(taproot_override_card(&d), "must be a taproot override card");
        assert!(
            !custom_use_site_nums_taproot_card(&d),
            "a hardened use-site override is un-restorable → loud advisory must NOT fire"
        );
    }

    #[test]
    fn mutually_exclusive_with_unrestorable_taproot_advisory() {
        // For ANY taproot override card, the CUSTOM (loud) trigger and the
        // un-restorable (calm) trigger are MUTUALLY EXCLUSIVE; BASELINE fires
        // neither. This is the single-source parity invariant.
        for cards in [
            NUMS_MULTI_A_OVERRIDE,
            NUMS_SORTEDMULTI_A_OVERRIDE,
            NON_NUMS_MULTI_A_OVERRIDE,
            NUMS_MULTI_A_HARDENED_OVERRIDE,
        ] {
            let d = desc(cards);
            let loud = custom_use_site_nums_taproot_card(&d);
            let calm = taproot_override_card(&d) && !restorable_taproot_override_card(&d);
            assert!(
                loud ^ calm,
                "exactly one of {{loud, calm}} must fire for a taproot override card"
            );
        }
        // BASELINE (uniform) fires NEITHER.
        let d = desc(NUMS_MULTI_A_BASELINE_UNIFORM);
        let loud = custom_use_site_nums_taproot_card(&d);
        let calm = taproot_override_card(&d) && !restorable_taproot_override_card(&d);
        assert!(!loud, "baseline: not loud");
        assert!(!calm, "baseline: not calm (not an override card at all)");
    }
}
