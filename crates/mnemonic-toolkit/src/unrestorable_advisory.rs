//! Non-blocking advisory for descriptor shapes `restore --md1` cannot reconstruct
//! (FOLLOWUP `bundle-unrestorable-shape-advisory`, C1).
//!
//! `bundle` and `import-wallet` engrave a wire-faithful md1 card for three
//! descriptor shapes that `restore --md1` then REFUSES (loudly — the card stays a
//! faithful backup, but mechanical watch-only reconstruction is unavailable):
//!   1. `sortedmulti()` inside a combinator (not the sole child of `wsh`/`sh`) —
//!      md-codec's pinned miniscript 13.0.0 has no `Terminal::SortedMulti` leaf
//!      (`to_miniscript.rs`), so restore refuses ("sole child of wsh/sh").
//!   2. per-cosigner use-site path overrides (`tlv.use_site_path_overrides`) —
//!      restore would silently render one shared suffix (`restore.rs:1247`).
//!   3. a hardened wildcard (`use_site_path.wildcard_hardened`, `/*h`) — restore
//!      would silently render `/*` (`restore.rs:1254`).
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
    /// `sortedmulti()` nested inside a combinator (shape 1).
    SortedMultiInCombinator,
    /// Per-cosigner use-site path overrides (shape 2).
    PerKeyUseSiteOverrides,
    /// A hardened wildcard `/*h` (shape 3).
    HardenedWildcard,
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
            UnrestorableShape::SortedMultiInCombinator => "advisory: restore --md1 cannot \
                reconstruct this descriptor — it places sortedmulti() inside a combinator \
                (sortedmulti must be the sole child of wsh/sh). The engraved card is a faithful \
                backup; keep the full descriptor to restore. Tracked: \
                bundle-accepts-sortedmulti-in-combinator-restore-cannot"
                .to_string(),
            UnrestorableShape::PerKeyUseSiteOverrides => "advisory: restore --md1 cannot \
                reconstruct this descriptor — it carries per-cosigner use-site path overrides (the \
                cosigners do not share one derivation suffix). The engraved card is a faithful \
                backup; keep the full descriptor to restore. Tracked: \
                restore-md1-per-key-use-site-and-hardened-wildcard"
                .to_string(),
            UnrestorableShape::HardenedWildcard => "advisory: restore --md1 cannot reconstruct \
                this descriptor — it uses a hardened wildcard (`/*h`), from which watch-only \
                addresses cannot be derived and which would silently render `/*`. The engraved \
                card is a faithful backup; keep the full descriptor to restore. Tracked: \
                restore-md1-per-key-use-site-and-hardened-wildcard"
                .to_string(),
        }
    }
}

/// Collect the unrestorable-shape advisories for a descriptor. At most ONE entry
/// per shape (each is a single structural fact), so the result holds 0..=3 items.
pub fn unrestorable_advisories(desc: &md_codec::Descriptor) -> Vec<UnrestorableAdvisory> {
    let mut out = Vec::new();
    if tree_has_sortedmulti_in_combinator(&desc.tree) {
        out.push(UnrestorableAdvisory {
            shape: UnrestorableShape::SortedMultiInCombinator,
        });
    }
    if desc.tlv.use_site_path_overrides.is_some() {
        out.push(UnrestorableAdvisory {
            shape: UnrestorableShape::PerKeyUseSiteOverrides,
        });
    }
    if desc.use_site_path.wildcard_hardened {
        out.push(UnrestorableAdvisory {
            shape: UnrestorableShape::HardenedWildcard,
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
            shape: UnrestorableShape::PerKeyUseSiteOverrides,
        }
        .message();
        assert!(
            m2.contains("advisory: restore --md1 cannot reconstruct") && m2.contains("use-site")
        );
        let m3 = UnrestorableAdvisory {
            shape: UnrestorableShape::HardenedWildcard,
        }
        .message();
        assert!(
            m3.contains("advisory: restore --md1 cannot reconstruct")
                && m3.contains("hardened wildcard")
        );
    }
}
