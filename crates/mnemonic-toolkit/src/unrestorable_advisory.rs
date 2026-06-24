//! Non-blocking advisory for descriptor shapes `restore --md1` cannot reconstruct
//! (FOLLOWUP `bundle-unrestorable-shape-advisory`, C1).
//!
//! `bundle` and `import-wallet` engrave a wire-faithful md1 card for the
//! descriptor shapes that `restore --md1` then REFUSES (loudly — the card stays a
//! faithful backup, but mechanical watch-only reconstruction is unavailable):
//!   1. `sortedmulti()` inside a combinator (not the sole child of `wsh`/`sh`) —
//!      md-codec's pinned miniscript 13.0.0 has no `Terminal::SortedMulti` leaf
//!      (`to_miniscript.rs`), so restore refuses ("sole child of wsh/sh").
//!   2. a HARDENED use-site path anywhere — baseline OR a per-cosigner override,
//!      `/*h` wildcard OR a hardened multipath alt — from which watch-only
//!      addresses cannot be derived (`md_codec::has_hardened_use_site`; restore
//!      refuses via the same predicate).
//!   3. a TAPROOT (`tr`) root carrying per-cosigner use-site overrides OUTSIDE the
//!      restorable subset — a `sortedmulti_a` tap leaf (md-codec render gap) or a
//!      non-NUMS internal/trunk key (D7) — for which the taproot reconstruction
//!      arm routes around the faithful per-`@N` path, so restore refuses
//!      (`taproot_override_card && !restorable_taproot_override_card`, FOLLOWUP
//!      `restore-md1-taproot-use-site-override-arm`). NON-hardened
//!      `tr(NUMS, multi_a(...))` override cards are NOW restorable (#26) — no
//!      advisory for them.
//!
//! NOTE (P2.4, #26): non-taproot non-hardened overrides AND non-hardened
//! `tr(NUMS, multi_a)` taproot overrides are now RESTORABLE (faithful per-`@N`
//! reconstruction) — so the old blanket `PerKeyUseSiteOverrides` advisory was
//! DROPPED and the taproot advisory was NARROWED. The residual override refusals
//! (hardened, sortedmulti_a/non-NUMS taproot) reuse the EXACT predicates the
//! restore guard uses (`has_hardened_use_site` / `taproot_override_card &&
//! !restorable_taproot_override_card`) — single source ⇒ advisory fires IFF
//! restore refuses.
//!
//! This module mirrors `timelock_advisory` (the v0.55.2 `older()` advisory): a
//! pure predicate over `md_codec::Descriptor` + a best-effort stderr emitter. The
//! governing property is PARITY — the advisory fires IFF restore would refuse, so
//! the shape-1 walk mirrors md-codec's `to_miniscript` acceptance set exactly (the
//! THREE restorable SortedMulti positions: `wsh(sortedmulti)`,
//! `sh(wsh(sortedmulti))`, and bare-P2SH `sh(sortedmulti)`).

use std::io::Write;

use md_codec::tag::Tag;
use md_codec::tree::{Body, Node};

/// Which unrestorable shape a descriptor carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnrestorableShape {
    /// A hardened use-site path anywhere (baseline or override; `/*h` wildcard
    /// or a hardened multipath alt). Watch-only cannot derive hardened.
    HardenedWildcard,
    /// `sortedmulti()` nested inside a combinator (shape 1).
    SortedMultiInCombinator,
    /// A taproot (`tr`) override card OUTSIDE the restorable subset — a
    /// `sortedmulti_a` tap leaf or a non-NUMS internal/trunk key (non-hardened
    /// `tr(NUMS, multi_a)` overrides are restorable and fire NO advisory).
    TaprootUseSiteOverride,
}

/// A collected unrestorable-shape advisory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnrestorableAdvisory {
    pub shape: UnrestorableShape,
}

impl UnrestorableAdvisory {
    /// The stderr advisory line. Every message shares the prefix
    /// `advisory: restore --md1 cannot reconstruct this descriptor` and names the
    /// shape + the slug, mirroring restore's own refusal wording.
    pub fn message(&self) -> String {
        match self.shape {
            UnrestorableShape::HardenedWildcard => "advisory: restore --md1 cannot reconstruct \
                this descriptor — it uses a hardened use-site path (`/*h` wildcard or a hardened \
                multipath alternative, baseline or per-cosigner), from which watch-only addresses \
                cannot be derived and which would silently render an unhardened path. The engraved \
                card is a faithful backup; keep the full descriptor to restore. Tracked: \
                restore-md1-per-key-use-site-and-hardened-wildcard"
                .to_string(),
            UnrestorableShape::SortedMultiInCombinator => "advisory: restore --md1 cannot \
                reconstruct this descriptor — it places sortedmulti() inside a combinator \
                (sortedmulti must be the sole child of wsh/sh). The engraved card is a faithful \
                backup; keep the full descriptor to restore. Tracked: \
                bundle-accepts-sortedmulti-in-combinator-restore-cannot"
                .to_string(),
            UnrestorableShape::TaprootUseSiteOverride => "advisory: restore --md1 cannot \
                reconstruct this descriptor — it is a taproot policy carrying per-cosigner \
                use-site path overrides in a shape not yet restorable (a sortedmulti_a tap leaf, \
                or a non-NUMS internal/trunk key; non-hardened tr(NUMS, multi_a(...)) override \
                cards ARE restorable). The engraved card is a faithful backup; keep the full \
                descriptor to restore. Tracked: restore-md1-taproot-use-site-override-arm"
                .to_string(),
        }
    }
}

/// Collect the unrestorable-shape advisories for a descriptor. At most ONE entry
/// per shape (each is a single structural fact), so the result holds 0..=3 items.
pub fn unrestorable_advisories(desc: &md_codec::Descriptor) -> Vec<UnrestorableAdvisory> {
    let mut out = Vec::new();
    // P2.4 parity: the hardened + taproot-override predicates are the SAME ones
    // the restore guard uses (`restore.rs`), so the advisory fires IFF restore
    // refuses. (Non-taproot, non-hardened use-site overrides are now restorable
    // — no advisory for them.)
    if md_codec::to_miniscript::has_hardened_use_site(desc) {
        out.push(UnrestorableAdvisory {
            shape: UnrestorableShape::HardenedWildcard,
        });
    }
    if tree_has_sortedmulti_in_combinator(&desc.tree) {
        out.push(UnrestorableAdvisory {
            shape: UnrestorableShape::SortedMultiInCombinator,
        });
    }
    // P2.4 (#26): fire IFF restore REFUSES — i.e. a taproot override card OUTSIDE
    // the restorable subset (`sortedmulti_a` leaf / non-NUMS trunk / hardened).
    // SAME expression as the narrowed restore guard (`restore.rs`) → exact
    // parity: a restorable `tr(NUMS, multi_a)` override is now SILENT here.
    if crate::taproot_override_classify::taproot_override_card(desc)
        && !crate::taproot_override_classify::restorable_taproot_override_card(desc)
    {
        out.push(UnrestorableAdvisory {
            shape: UnrestorableShape::TaprootUseSiteOverride,
        });
    }
    out
}

/// Write each advisory's message to `stderr` (best-effort; mirrors `emit_advisories`
/// in `timelock_advisory`).
pub fn emit_advisories<E: Write>(advisories: &[UnrestorableAdvisory], stderr: &mut E) {
    for a in advisories {
        let _ = writeln!(stderr, "{}", a.message());
    }
}

// ── Cycle Y (v0.73.3): LOUD funds-safety advisory ────────────────────────────
//
// A SEPARATE advisory register from `UnrestorableAdvisory` above. The
// `UnrestorableShape` messages are CALM ("the engraved card is a faithful
// backup; keep the full descriptor to restore") because those shapes loudly
// REFUSE at restore — the user is protected by the refusal. The funds-safety
// advisory below is for a shape that RESTORES SUCCESSFULLY but that no known
// wallet produces, so a misconfigured user would silently get non-matching
// addresses and risk PERMANENT LOSS OF FUNDS. It is deliberately kept a distinct
// enum + struct + collector + prefix (`WARNING (funds-safety):`, not
// `advisory:`) so a future editor cannot soften the loud text to match the calm
// siblings. (Cycle Y, FOLLOWUP `restore-md1-taproot-use-site-override-arm`
// PARTIALLY-RESOLVED; this closes the missing-warning gap, NOT a reconstruction
// gap.)

/// Which funds-safety shape a descriptor carries (a RESTORABLE-but-no-precedent
/// shape that warrants a LOUD warning rather than a calm keep-the-backup note).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FundsSafetyShape {
    /// A `tr(NUMS, multi_a)` multisig with CUSTOM per-cosigner use-site
    /// derivation paths (divergent suffixes per `@N`). Restores faithfully (#26)
    /// but has no known wallet precedent — every standard wallet uses one
    /// uniform `<0;1>/*` suffix across all cosigners.
    CustomUseSiteNumsTaproot,
}

/// A collected funds-safety advisory (LOUD; the operation still proceeds).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FundsSafetyAdvisory {
    pub shape: FundsSafetyShape,
}

impl FundsSafetyAdvisory {
    /// The LOUD stderr advisory line. Prefix `WARNING (funds-safety):` (distinct
    /// from the calm `advisory:`), uppercase `LOSS OF FUNDS`/`NOT`; conveys
    /// no-precedent + addresses-will-not-match + LOSS OF FUNDS + verify.
    pub fn message(&self) -> String {
        match self.shape {
            FundsSafetyShape::CustomUseSiteNumsTaproot => "WARNING (funds-safety): this card is a \
                tr(NUMS, multi_a) multisig with CUSTOM per-cosigner use-site derivation paths \
                (divergent suffixes per cosigner). No known wallet produces this shape — every \
                standard wallet uses one uniform <0;1>/* suffix across all cosigners. If you did \
                NOT deliberately intend divergent per-cosigner derivation paths, the addresses \
                reconstructed from this card will NOT match your wallet software and you risk \
                PERMANENT LOSS OF FUNDS. Verify the descriptor against your wallet before relying \
                on this card."
                .to_string(),
        }
    }
}

/// Collect the funds-safety advisories for a descriptor (single-sourced via the
/// `taproot_override_classify` predicate, so engrave and restore cannot drift).
pub fn funds_safety_advisories(desc: &md_codec::Descriptor) -> Vec<FundsSafetyAdvisory> {
    let mut out = Vec::new();
    if crate::taproot_override_classify::custom_use_site_nums_taproot_card(desc) {
        out.push(FundsSafetyAdvisory {
            shape: FundsSafetyShape::CustomUseSiteNumsTaproot,
        });
    }
    out
}

/// Write each funds-safety advisory's message to `stderr` (best-effort; mirrors
/// `emit_advisories`).
pub fn emit_funds_safety_advisories<E: Write>(advisories: &[FundsSafetyAdvisory], stderr: &mut E) {
    for a in advisories {
        let _ = writeln!(stderr, "{}", a.message());
    }
}

/// True iff `Tag::SortedMulti` appears in a position restore cannot reconstruct —
/// i.e. anywhere other than one of md-codec's three accepted sole-child positions.
/// Unit-testable on a bare `Node` (no full `md_codec::Descriptor` literal needed).
pub(crate) fn tree_has_sortedmulti_in_combinator(root: &Node) -> bool {
    // The three RESTORABLE sole-child positions (mirror md-codec's
    // `wsh_inner_to_descriptor`/`sh_inner_to_descriptor` dispatchers): if the root
    // is exactly one of them the SortedMulti is accepted and there is no other
    // subtree to check → not a combinator use.
    if is_accepted_sole_child_sortedmulti(root) {
        return false;
    }
    // Otherwise any SortedMulti anywhere in the tree is a combinator-leaf, which
    // md-codec's renderer rejects → restore refuses.
    subtree_contains_sortedmulti(root)
}

/// The single child of a `Body::Children` node, if it has exactly one.
fn sole_child(node: &Node) -> Option<&Node> {
    match &node.body {
        Body::Children(c) if c.len() == 1 => Some(&c[0]),
        _ => None,
    }
}

/// Is `root` exactly one of the three restorable sole-child SortedMulti shapes:
/// `wsh(sortedmulti)`, `sh(wsh(sortedmulti))`, or bare-P2SH `sh(sortedmulti)`?
fn is_accepted_sole_child_sortedmulti(root: &Node) -> bool {
    match root.tag {
        Tag::Wsh => sole_child(root).is_some_and(|c| c.tag == Tag::SortedMulti),
        Tag::Sh => match sole_child(root) {
            // sh(sortedmulti) — bare legacy P2SH (md-codec `new_sh_sortedmulti`).
            Some(inner) if inner.tag == Tag::SortedMulti => true,
            // sh(wsh(sortedmulti)) — nested segwit (md-codec `new_sh_wsh_sortedmulti`).
            Some(inner) if inner.tag == Tag::Wsh => {
                sole_child(inner).is_some_and(|g| g.tag == Tag::SortedMulti)
            }
            _ => false,
        },
        _ => false,
    }
}

/// Recursively true if any node in the subtree is a `Tag::SortedMulti`.
fn subtree_contains_sortedmulti(node: &Node) -> bool {
    if node.tag == Tag::SortedMulti {
        return true;
    }
    match &node.body {
        Body::Children(children) => children.iter().any(subtree_contains_sortedmulti),
        Body::Variable { children, .. } => children.iter().any(subtree_contains_sortedmulti),
        Body::Tr { tree: Some(t), .. } => subtree_contains_sortedmulti(t),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use md_codec::tag::Tag;
    use md_codec::tree::{Body, Node};

    fn leaf(tag: Tag) -> Node {
        Node {
            tag,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1],
            },
        }
    }
    fn wrap(tag: Tag, child: Node) -> Node {
        Node {
            tag,
            body: Body::Children(vec![child]),
        }
    }

    #[test]
    fn sole_child_sortedmulti_all_three_positions_not_combinator() {
        // (a) wsh(sortedmulti) — sole wsh child.
        assert!(!tree_has_sortedmulti_in_combinator(&wrap(
            Tag::Wsh,
            leaf(Tag::SortedMulti)
        )));
        // (b) sh(wsh(sortedmulti)) — sole sh→wsh grandchild.
        assert!(!tree_has_sortedmulti_in_combinator(&wrap(
            Tag::Sh,
            wrap(Tag::Wsh, leaf(Tag::SortedMulti))
        )));
        // (c) sh(sortedmulti) — bare-P2SH sole sh child (the R0-r1 I1 omission).
        assert!(!tree_has_sortedmulti_in_combinator(&wrap(
            Tag::Sh,
            leaf(Tag::SortedMulti)
        )));
    }

    #[test]
    fn sortedmulti_in_combinator_fires() {
        // wsh(or_d(pk, sortedmulti)) — SortedMulti is a combinator leaf.
        let or_d = Node {
            tag: Tag::OrD,
            body: Body::Children(vec![leaf(Tag::PkK), leaf(Tag::SortedMulti)]),
        };
        assert!(tree_has_sortedmulti_in_combinator(&wrap(Tag::Wsh, or_d)));
        // sh(wsh(or_d(pk, sortedmulti))) — nested under sh-wsh, still a combinator.
        let or_d2 = Node {
            tag: Tag::OrD,
            body: Body::Children(vec![leaf(Tag::PkK), leaf(Tag::SortedMulti)]),
        };
        assert!(tree_has_sortedmulti_in_combinator(&wrap(
            Tag::Sh,
            wrap(Tag::Wsh, or_d2)
        )));
    }

    #[test]
    fn multi_in_combinator_and_no_sortedmulti_do_not_fire() {
        // multi (a real Terminal) in a combinator restores fine → must NOT fire.
        let or_d = Node {
            tag: Tag::OrD,
            body: Body::Children(vec![leaf(Tag::PkK), leaf(Tag::Multi)]),
        };
        assert!(!tree_has_sortedmulti_in_combinator(&wrap(Tag::Wsh, or_d)));
        // A descriptor with no SortedMulti anywhere → must NOT fire.
        assert!(!tree_has_sortedmulti_in_combinator(&wrap(
            Tag::Wsh,
            leaf(Tag::Multi)
        )));
    }

    #[test]
    fn message_forms_carry_prefix_and_shape() {
        let m1 = UnrestorableAdvisory {
            shape: UnrestorableShape::SortedMultiInCombinator,
        }
        .message();
        assert!(
            m1.contains("advisory: restore --md1 cannot reconstruct") && m1.contains("sortedmulti")
        );
        let m2 = UnrestorableAdvisory {
            shape: UnrestorableShape::TaprootUseSiteOverride,
        }
        .message();
        assert!(
            m2.contains("advisory: restore --md1 cannot reconstruct")
                && m2.contains("taproot")
                && m2.contains("restore-md1-taproot-use-site-override-arm")
        );
        let m3 = UnrestorableAdvisory {
            shape: UnrestorableShape::HardenedWildcard,
        }
        .message();
        assert!(
            m3.contains("advisory: restore --md1 cannot reconstruct")
                && m3.contains("hardened use-site path")
        );
    }

    #[test]
    fn funds_safety_message_is_loud_and_funds_framed() {
        let m = FundsSafetyAdvisory {
            shape: FundsSafetyShape::CustomUseSiteNumsTaproot,
        }
        .message();
        // LOUD prefix — textually distinct from the calm `advisory:` siblings.
        assert!(
            m.starts_with("WARNING (funds-safety):"),
            "loud prefix required; got: {m}"
        );
        // Funds-safety framing: no-precedent + addresses-will-not-match + LOSS + verify.
        assert!(m.contains("tr(NUMS, multi_a)"), "names the shape; got: {m}");
        assert!(
            m.contains("No known wallet produces this shape"),
            "no-precedent phrasing; got: {m}"
        );
        assert!(
            m.contains("PERMANENT LOSS OF FUNDS"),
            "loss-of-funds phrasing; got: {m}"
        );
        assert!(
            m.contains("will NOT match your wallet"),
            "addresses-will-not-match phrasing; got: {m}"
        );
        assert!(
            m.contains("Verify the descriptor against your wallet"),
            "verify phrasing; got: {m}"
        );
        // NOT softened to the calm register.
        assert!(
            !m.contains("advisory: restore --md1 cannot reconstruct"),
            "must NOT borrow the calm prefix; got: {m}"
        );
    }
}
