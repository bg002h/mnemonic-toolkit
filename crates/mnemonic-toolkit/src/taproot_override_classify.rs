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
