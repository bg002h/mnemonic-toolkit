//! Decoder-side validation per spec §7.

use crate::canonical_origin::canonical_origin;
use crate::encode::Descriptor;
use crate::error::Error;
use crate::origin_path::PathDeclPaths;
use crate::tag::Tag;
use crate::tree::{Body, Node};
use crate::use_site_path::UseSitePath;

/// Validate the BIP 388 well-formedness of placeholder usage in the tree.
///
/// Enforces two invariants:
/// 1. Every `@i` for `0 ≤ i < n` appears at least once in the tree.
/// 2. The first occurrences (in pre-order traversal) of distinct placeholder
///    indices appear in canonical ascending order: `@0` before `@1` before `@2`, etc.
pub fn validate_placeholder_usage(root: &Node, n: u8) -> Result<(), Error> {
    let mut seen = vec![false; n as usize];
    let mut first_occurrences: Vec<u8> = Vec::new();
    walk_for_placeholders(root, &mut seen, &mut first_occurrences)?;
    // Each @i for 0 ≤ i < n must appear at least once.
    for (i, was_seen) in seen.iter().enumerate() {
        if !was_seen {
            return Err(Error::PlaceholderNotReferenced { idx: i as u8, n });
        }
    }
    // First occurrences must be in canonical ascending order.
    for (pos, idx) in first_occurrences.iter().enumerate() {
        if *idx as usize != pos {
            return Err(Error::PlaceholderFirstOccurrenceOutOfOrder {
                expected_first: pos as u8,
                got_first: *idx,
            });
        }
    }
    Ok(())
}

fn walk_for_placeholders(
    node: &Node,
    seen: &mut [bool],
    first_occurrences: &mut Vec<u8>,
) -> Result<(), Error> {
    match &node.body {
        Body::KeyArg { index } => {
            if (*index as usize) >= seen.len() {
                return Err(Error::PlaceholderIndexOutOfRange {
                    idx: *index,
                    n: seen.len() as u8,
                });
            }
            if !seen[*index as usize] {
                seen[*index as usize] = true;
                first_occurrences.push(*index);
            }
        }
        Body::Children(children) => {
            for c in children {
                walk_for_placeholders(c, seen, first_occurrences)?;
            }
        }
        Body::Variable { children, .. } => {
            for c in children {
                walk_for_placeholders(c, seen, first_occurrences)?;
            }
        }
        Body::MultiKeys { indices, .. } => {
            // v0.30 Phase C: multi-family bodies carry raw key indices instead
            // of child Nodes. Same placeholder-usage semantics as KeyArg, per
            // index.
            for index in indices {
                if (*index as usize) >= seen.len() {
                    return Err(Error::PlaceholderIndexOutOfRange {
                        idx: *index,
                        n: seen.len() as u8,
                    });
                }
                if !seen[*index as usize] {
                    seen[*index as usize] = true;
                    first_occurrences.push(*index);
                }
            }
        }
        Body::Tr {
            is_nums,
            key_index,
            tree,
        } => {
            // SPEC v0.30 §7 + §11: when `is_nums = true` the internal key is
            // the BIP-341 NUMS H-point (not a placeholder reference); skip
            // registration. Otherwise `key_index` must be in `0..n`; out-of-
            // range raises `NUMSSentinelConflict` per SPEC §11 (Phase G
            // finalizes the variant's full doc-comment).
            if !*is_nums {
                if (*key_index as usize) >= seen.len() {
                    return Err(Error::NUMSSentinelConflict);
                }
                if !seen[*key_index as usize] {
                    seen[*key_index as usize] = true;
                    first_occurrences.push(*key_index);
                }
            }
            if let Some(t) = tree {
                walk_for_placeholders(t, seen, first_occurrences)?;
            }
        }
        Body::Hash256Body(_) | Body::Hash160Body(_) | Body::Timelock(_) | Body::Empty => {}
    }
    Ok(())
}

/// Validate that all multipaths in shared default + overrides share the same alt-count.
///
/// Per spec §7, when multiple `UseSitePath` entries (the shared default plus any
/// per-`@N` overrides) carry a multipath group, all groups MUST have the same
/// number of alternatives.
///
/// D5(b): a `Some`-multipath baseline mixed with a `None`-multipath override
/// (or vice-versa) is a **legal divergent STRUCTURE** (e.g. `@0/<0;1>/*` +
/// `@1/*`), NOT a reject — a `None` entry simply carries no multipath group,
/// so it is skipped by the alt-count check below (the `if let Some(alts)`
/// guard). The C2 faithful reconstruction
/// (`crate::to_miniscript::to_miniscript_descriptor_multipath`) handles the
/// `None`-override by emitting a single-path `XPub` for that key while sibling
/// keys stay `MultiXPub`. Only two multipath groups with DIFFERENT alt-counts
/// are rejected.
pub fn validate_multipath_consistency(
    shared: &UseSitePath,
    overrides: &[(u8, UseSitePath)],
) -> Result<(), Error> {
    let mut seen_alt_count: Option<usize> = None;
    let candidates = std::iter::once(shared).chain(overrides.iter().map(|(_, p)| p));
    for path in candidates {
        if let Some(alts) = &path.multipath {
            match seen_alt_count {
                None => seen_alt_count = Some(alts.len()),
                Some(prev) if prev == alts.len() => {}
                Some(prev) => {
                    return Err(Error::MultipathAltCountMismatch {
                        expected: prev,
                        got: alts.len(),
                    });
                }
            }
        }
    }
    Ok(())
}

/// D5(a) decode canonical-form check for `use_site_path_overrides`.
///
/// Our encoders only push an override entry for `i ≥ 1` and only when it
/// DIFFERS from the resolved baseline (`Descriptor::use_site_path`). Two
/// non-canonical / adversarial wire shapes are therefore rejected at decode
/// (defense in depth — they are never emitted, only hand-crafted):
///
/// 1. An entry keyed on `@0` — the baseline cannot be overridden →
///    [`Error::BaselineUseSiteOverride`].
/// 2. An entry whose `UseSitePath` equals `baseline` — a redundant
///    (non-canonical) override → [`Error::RedundantUseSiteOverride`].
///
/// The `@0` check runs first so an adversarial `@0` entry that ALSO happens
/// to equal the baseline surfaces as the more-specific `BaselineUseSiteOverride`.
pub fn validate_use_site_overrides_canonical(
    baseline: &UseSitePath,
    overrides: &[(u8, UseSitePath)],
) -> Result<(), Error> {
    for (idx, usp) in overrides {
        if *idx == 0 {
            return Err(Error::BaselineUseSiteOverride { idx: *idx });
        }
        if usp == baseline {
            return Err(Error::RedundantUseSiteOverride { idx: *idx });
        }
    }
    Ok(())
}

/// Validate that all leaves in a tap-script-tree are permitted-leaf tags per §6.3.1.
pub fn validate_tap_script_tree(node: &Node) -> Result<(), Error> {
    walk_tap_tree_leaves(node)
}

fn walk_tap_tree_leaves(node: &Node) -> Result<(), Error> {
    if matches!(node.tag, Tag::TapTree) {
        if let Body::Children(children) = &node.body {
            for c in children {
                walk_tap_tree_leaves(c)?;
            }
        }
        Ok(())
    } else {
        // This is a leaf — validate per §6.3.1.
        if is_forbidden_leaf_tag(node.tag) {
            return Err(Error::ForbiddenTapTreeLeaf {
                tag: node.tag.codes().0,
            });
        }
        Ok(())
    }
}

fn is_forbidden_leaf_tag(tag: Tag) -> bool {
    matches!(
        tag,
        Tag::Wpkh | Tag::Tr | Tag::Wsh | Tag::Sh | Tag::Pkh | Tag::Multi | Tag::SortedMulti
    )
}

/// Validate that every `@N` in a non-canonical wrapper has an explicit
/// origin path on the wire — either via `OriginPathOverrides[idx]` or
/// via a non-empty entry in the `path_decl` (shared or divergent).
///
/// Per spec v0.13 §6.3: when `canonical_origin(&d.tree)` is `None`, the
/// wrapper is "non-canonical" and the encoder must emit an explicit
/// origin for every `@N`. The decoder enforces the same as defense in
/// depth: failure → `Error::MissingExplicitOrigin { idx }`.
///
/// If `canonical_origin(&d.tree)` is `Some(_)`, this validator is a
/// no-op — any origin spec (elided or explicit) is allowed.
pub fn validate_explicit_origin_required(d: &Descriptor) -> Result<(), Error> {
    if canonical_origin(&d.tree).is_some() {
        return Ok(());
    }
    let overrides = d.tlv.origin_path_overrides.as_deref().unwrap_or(&[]);
    for idx in 0..d.n {
        // Override path takes precedence — if present and non-empty, OK.
        if let Some((_, op)) = overrides.iter().find(|(i, _)| *i == idx) {
            if !op.components.is_empty() {
                continue;
            }
        }
        // Otherwise consult the path_decl for this idx.
        let decl_components_empty = match &d.path_decl.paths {
            PathDeclPaths::Shared(p) => p.components.is_empty(),
            PathDeclPaths::Divergent(v) => v
                .get(idx as usize)
                .map(|p| p.components.is_empty())
                .unwrap_or(true),
        };
        if decl_components_empty {
            return Err(Error::MissingExplicitOrigin { idx });
        }
    }
    Ok(())
}

/// Validate that every `Pubkeys` TLV entry's 33-byte compressed pubkey
/// field (bytes 32..65 of the 65-byte payload) parses as a valid
/// secp256k1 point. The 32-byte chain code prefix is unvalidated (any
/// 32 bytes are a structurally valid BIP 32 chain code).
///
/// Per spec v0.13 §6.4: failure → `Error::InvalidXpubBytes { idx }`.
/// When `d.tlv.pubkeys` is `None` (template-only mode), this is a no-op.
pub fn validate_xpub_bytes(d: &Descriptor) -> Result<(), Error> {
    let Some(entries) = d.tlv.pubkeys.as_deref() else {
        return Ok(());
    };
    for (idx, xpub) in entries {
        if bitcoin::secp256k1::PublicKey::from_slice(&xpub[32..65]).is_err() {
            return Err(Error::InvalidXpubBytes { idx: *idx });
        }
    }
    Ok(())
}

/// Validate that no `OriginPathOverrides[idx]` entry is present-but-empty
/// (zero path components). Per spec v0.13 §6.3 (I-1 hardening, P0
/// pathless/dead-card partial-decode).
///
/// Runs UNCONDITIONALLY — regardless of `canonical_origin(&d.tree)` — so
/// a CANONICAL-shape wire (e.g. `wpkh(@0)`) carrying an empty override is
/// ALSO rejected (I-1a). This is a DISTINCT error variant from
/// `Error::MissingExplicitOrigin` so partial-allowing decode (P0.2, which
/// swallows ONLY `MissingExplicitOrigin`) never swallows this: a
/// present-but-empty override is a MALFORMED wire, not a dead card, and
/// must not partial-render (I-1b, fatal-in-partial).
///
/// Converges with [`crate::canonicalize::expand_per_at_n`], which runs
/// the same check independently (defense in depth for a hand-built
/// `Descriptor` that bypasses decode).
pub fn validate_no_empty_origin_overrides(d: &Descriptor) -> Result<(), Error> {
    let overrides = d.tlv.origin_path_overrides.as_deref().unwrap_or(&[]);
    for (idx, op) in overrides {
        if op.components.is_empty() {
            return Err(Error::EmptyOriginOverride { idx: *idx });
        }
    }
    Ok(())
}

impl Descriptor {
    /// The ascending `@N` indices whose origin cannot be resolved: a pure
    /// query mirroring [`validate_explicit_origin_required`]'s SEMANTICS
    /// (P0.1, pathless/dead-card partial-decode).
    ///
    /// Returns the ascending indices where `canonical_origin(&self.tree)`
    /// is `None` AND the per-idx origin (override-or-`path_decl`) is
    /// empty. Returns `[]` when `canonical_origin` is `Some` OR every idx
    /// has a non-empty origin (i.e. exactly the set of shapes that decode
    /// cleanly today under the strict default). Does NOT call
    /// [`crate::canonicalize::expand_per_at_n`] — this is a
    /// non-erroring, side-effect-free query, not an expansion; callers
    /// needing per-`@N` origin/use-site/fp/xpub records still use
    /// `expand_per_at_n` (which stays strict and fail-closed
    /// unconditionally — see its doc comment).
    pub fn unresolved_origin_indices(&self) -> Vec<u8> {
        if canonical_origin(&self.tree).is_some() {
            return Vec::new();
        }
        let overrides = self.tlv.origin_path_overrides.as_deref().unwrap_or(&[]);
        let mut out = Vec::new();
        for idx in 0..self.n {
            // Override path takes precedence — if present and non-empty, resolved.
            if let Some((_, op)) = overrides.iter().find(|(i, _)| *i == idx) {
                if !op.components.is_empty() {
                    continue;
                }
            }
            // Otherwise consult the path_decl for this idx.
            let decl_components_empty = match &self.path_decl.paths {
                PathDeclPaths::Shared(p) => p.components.is_empty(),
                PathDeclPaths::Divergent(v) => v
                    .get(idx as usize)
                    .map(|p| p.components.is_empty())
                    .unwrap_or(true),
            };
            if decl_components_empty {
                out.push(idx);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::Tag;
    use crate::tree::{Body, Node};

    #[test]
    fn placeholder_usage_ok_for_2_of_3() {
        let root = Node {
            tag: Tag::SortedMulti,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1, 2],
            },
        };
        validate_placeholder_usage(&root, 3).unwrap();
    }

    #[test]
    fn placeholder_usage_rejects_unreferenced() {
        let root = Node {
            tag: Tag::SortedMulti,
            body: Body::MultiKeys {
                k: 1,
                indices: vec![0, 1],
            },
        };
        assert!(matches!(
            validate_placeholder_usage(&root, 3),
            Err(Error::PlaceholderNotReferenced { idx: 2, n: 3 })
        ));
    }

    #[test]
    fn placeholder_usage_rejects_out_of_order_first_occurrences() {
        let root = Node {
            tag: Tag::SortedMulti,
            body: Body::MultiKeys {
                k: 1,
                indices: vec![1, 0],
            },
        };
        assert!(matches!(
            validate_placeholder_usage(&root, 2),
            Err(Error::PlaceholderFirstOccurrenceOutOfOrder { .. })
        ));
    }

    #[test]
    fn multipath_consistency_ok_when_all_match() {
        let shared = UseSitePath::standard_multipath();
        let overrides = vec![(1u8, UseSitePath::standard_multipath())];
        validate_multipath_consistency(&shared, &overrides).unwrap();
    }

    #[test]
    fn multipath_consistency_rejects_mismatched_alt_counts() {
        use crate::use_site_path::Alternative;
        let shared = UseSitePath::standard_multipath();
        let overrides = vec![(
            1u8,
            UseSitePath {
                multipath: Some(vec![
                    Alternative {
                        hardened: false,
                        value: 0,
                    },
                    Alternative {
                        hardened: false,
                        value: 1,
                    },
                    Alternative {
                        hardened: false,
                        value: 2,
                    },
                ]),
                wildcard_hardened: false,
            },
        )];
        assert!(matches!(
            validate_multipath_consistency(&shared, &overrides),
            Err(Error::MultipathAltCountMismatch {
                expected: 2,
                got: 3
            })
        ));
    }

    #[test]
    fn tap_tree_leaf_rejects_wsh() {
        let leaf = Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![]),
        };
        assert!(matches!(
            validate_tap_script_tree(&leaf),
            Err(Error::ForbiddenTapTreeLeaf { .. })
        ));
    }

    #[test]
    fn tap_tree_leaf_accepts_pk_k() {
        let leaf = Node {
            tag: Tag::PkK,
            body: Body::KeyArg { index: 0 },
        };
        validate_tap_script_tree(&leaf).unwrap();
    }

    #[test]
    fn placeholder_usage_rejects_index_out_of_range_n3() {
        // n=3 → key_index_width=2 admits 0..=3 structurally. @3 is out of range.
        let root = Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 3 },
        };
        let err = validate_placeholder_usage(&root, 3).unwrap_err();
        assert!(matches!(
            err,
            Error::PlaceholderIndexOutOfRange { idx: 3, n: 3 }
        ));
    }

    #[test]
    fn placeholder_usage_rejects_index_out_of_range_n5() {
        // n=5 → key_index_width=3 admits 0..=7. @5..=7 are out of range.
        let root = Node {
            tag: Tag::SortedMulti,
            body: Body::MultiKeys {
                k: 1,
                indices: vec![5],
            },
        };
        let err = validate_placeholder_usage(&root, 5).unwrap_err();
        assert!(matches!(
            err,
            Error::PlaceholderIndexOutOfRange { idx: 5, n: 5 }
        ));
    }

    #[test]
    fn placeholder_usage_rejects_index_out_of_range_n15() {
        // n=15 → key_index_width=4 admits 0..=15. @15 just out of range.
        let root = Node {
            tag: Tag::SortedMulti,
            body: Body::MultiKeys {
                k: 1,
                indices: vec![15],
            },
        };
        let err = validate_placeholder_usage(&root, 15).unwrap_err();
        assert!(matches!(
            err,
            Error::PlaceholderIndexOutOfRange { idx: 15, n: 15 }
        ));
    }

    #[test]
    fn placeholder_usage_rejects_out_of_range_in_tr_key_index() {
        // SPEC v0.30 §7 + §11: `is_nums = false` with `key_index >= n` is a
        // `NUMSSentinelConflict` (distinct from KeyArg's
        // `PlaceholderIndexOutOfRange`; NUMS is signalled by `is_nums = true`
        // with `key_index` unused on wire).
        let root = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 3,
                tree: None,
            },
        };
        let err = validate_placeholder_usage(&root, 3).unwrap_err();
        assert!(matches!(err, Error::NUMSSentinelConflict));
    }

    #[test]
    fn placeholder_usage_accepts_nums_flag_in_tr() {
        // SPEC v0.30 §7: `is_nums = true` is the NUMS-H-point signal and
        // MUST pass validation. validate_placeholder_usage requires every
        // @i in 0..n to be referenced; the @0 reference here satisfies that
        // for n=1.
        let root = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: true,
                key_index: 0,
                tree: Some(Box::new(Node {
                    tag: Tag::PkK,
                    body: Body::KeyArg { index: 0 },
                })),
            },
        };
        validate_placeholder_usage(&root, 1)
            .expect("is_nums flag + @0 reference must validate under v0.30");
    }
}

#[cfg(test)]
mod explicit_origin_required_tests {
    use super::*;
    use crate::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
    use crate::tag::Tag;
    use crate::tlv::TlvSection;
    use crate::tree::{Body, Node};
    use crate::use_site_path::UseSitePath;

    fn empty_path() -> OriginPath {
        OriginPath { components: vec![] }
    }

    fn bip84_path() -> OriginPath {
        OriginPath {
            components: vec![
                PathComponent {
                    hardened: true,
                    value: 84,
                },
                PathComponent {
                    hardened: true,
                    value: 0,
                },
                PathComponent {
                    hardened: true,
                    value: 0,
                },
            ],
        }
    }

    /// Build a single-key descriptor with `n=1`, the given tree root, an
    /// empty shared path_decl (origin elided on wire), and an empty TLV
    /// section.
    fn single_key_descriptor(tree: Node) -> Descriptor {
        Descriptor {
            n: 1,
            path_decl: PathDecl {
                n: 1,
                paths: PathDeclPaths::Shared(empty_path()),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree,
            tlv: TlvSection::new_empty(),
        }
    }

    #[test]
    fn validate_explicit_origin_required_passes_canonical_wpkh() {
        // wpkh(@0) has canonical BIP-84 origin → empty path_decl OK.
        let d = single_key_descriptor(Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        });
        validate_explicit_origin_required(&d).unwrap();
    }

    #[test]
    fn validate_explicit_origin_required_passes_with_overrides_for_non_canonical() {
        // sh(sortedmulti(@0,@1,@2)) — non-canonical. Must have explicit
        // origin per @N. Provide overrides for all three.
        let mut d = Descriptor {
            n: 3,
            path_decl: PathDecl {
                n: 3,
                paths: PathDeclPaths::Shared(empty_path()),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Sh,
                body: Body::Children(vec![Node {
                    tag: Tag::SortedMulti,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: vec![0, 1, 2],
                    },
                }]),
            },
            tlv: TlvSection::new_empty(),
        };
        d.tlv.origin_path_overrides = Some(vec![
            (0u8, bip84_path()),
            (1u8, bip84_path()),
            (2u8, bip84_path()),
        ]);
        validate_explicit_origin_required(&d).unwrap();
    }

    #[test]
    fn validate_explicit_origin_required_fails_sh_sortedmulti_with_empty_path_decl() {
        // sh(sortedmulti(@0,@1,@2)) — non-canonical. Empty path_decl, no
        // overrides → fails on idx=0.
        let d = Descriptor {
            n: 3,
            path_decl: PathDecl {
                n: 3,
                paths: PathDeclPaths::Shared(empty_path()),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Sh,
                body: Body::Children(vec![Node {
                    tag: Tag::SortedMulti,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: vec![0, 1, 2],
                    },
                }]),
            },
            tlv: TlvSection::new_empty(),
        };
        let err = validate_explicit_origin_required(&d).unwrap_err();
        assert!(matches!(err, Error::MissingExplicitOrigin { idx: 0 }));
    }

    #[test]
    fn validate_explicit_origin_required_fails_bare_wsh_with_empty_path_decl() {
        // bare wsh(@0) — non-canonical (no `multi`/`sortedmulti` inner).
        let d = single_key_descriptor(Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::PkK,
                body: Body::KeyArg { index: 0 },
            }]),
        });
        let err = validate_explicit_origin_required(&d).unwrap_err();
        assert!(matches!(err, Error::MissingExplicitOrigin { idx: 0 }));
    }

    #[test]
    fn validate_explicit_origin_required_passes_tr_keypath_only_with_empty_path_decl() {
        // tr(@0) key-path only → BIP 86 canonical exists → empty path_decl OK.
        let d = single_key_descriptor(Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: None,
            },
        });
        validate_explicit_origin_required(&d).unwrap();
    }

    #[test]
    fn validate_explicit_origin_required_fails_tr_with_taptree_with_empty_path_decl() {
        // tr(@0, TapTree) → no canonical → must be explicit.
        let d = single_key_descriptor(Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: Some(Box::new(Node {
                    tag: Tag::PkK,
                    body: Body::KeyArg { index: 0 },
                })),
            },
        });
        let err = validate_explicit_origin_required(&d).unwrap_err();
        assert!(matches!(err, Error::MissingExplicitOrigin { idx: 0 }));
    }

    #[test]
    fn validate_explicit_origin_required_passes_with_populated_shared_path_decl() {
        // Bare wsh(@0) with a populated shared path_decl — explicit origin
        // is on the wire via path_decl, so the validator is satisfied even
        // without an OriginPathOverrides entry.
        let mut d = single_key_descriptor(Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::PkK,
                body: Body::KeyArg { index: 0 },
            }]),
        });
        d.path_decl.paths = PathDeclPaths::Shared(bip84_path());
        validate_explicit_origin_required(&d).unwrap();
    }

    #[test]
    fn validate_explicit_origin_required_passes_divergent_when_all_populated() {
        // sh(sortedmulti(...)) with divergent path_decl, all entries populated.
        let d = Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Divergent(vec![bip84_path(), bip84_path()]),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Sh,
                body: Body::Children(vec![Node {
                    tag: Tag::SortedMulti,
                    body: Body::MultiKeys {
                        k: 1,
                        indices: vec![0, 1],
                    },
                }]),
            },
            tlv: TlvSection::new_empty(),
        };
        validate_explicit_origin_required(&d).unwrap();
    }

    #[test]
    fn validate_explicit_origin_required_fails_divergent_when_one_idx_empty() {
        // sh(sortedmulti(...)) with divergent path_decl; @1 has empty path,
        // no override → fails on idx=1.
        let d = Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Divergent(vec![bip84_path(), empty_path()]),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Sh,
                body: Body::Children(vec![Node {
                    tag: Tag::SortedMulti,
                    body: Body::MultiKeys {
                        k: 1,
                        indices: vec![0, 1],
                    },
                }]),
            },
            tlv: TlvSection::new_empty(),
        };
        let err = validate_explicit_origin_required(&d).unwrap_err();
        assert!(matches!(err, Error::MissingExplicitOrigin { idx: 1 }));
    }

    // ─── P0.1: Descriptor::unresolved_origin_indices ─────────────────────
    //
    // Pure query mirroring `validate_explicit_origin_required`'s SEMANTICS
    // (does NOT call `expand_per_at_n`). Every case below has a sibling
    // `validate_explicit_origin_required_*` test above/below asserting the
    // same shape's Ok/Err verdict; these pin the parallel non-erroring
    // query's ascending-index-vec verdict.

    #[test]
    fn unresolved_origin_indices_empty_for_canonical_wpkh() {
        // wpkh(@0) has canonical BIP-84 origin → empty path_decl still []
        let d = single_key_descriptor(Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        });
        assert_eq!(d.unresolved_origin_indices(), Vec::<u8>::new());
    }

    #[test]
    fn unresolved_origin_indices_empty_for_canonical_tr_keypath() {
        // tr(@0) key-path only → BIP-86 canonical → [].
        let d = single_key_descriptor(Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: None,
            },
        });
        assert_eq!(d.unresolved_origin_indices(), Vec::<u8>::new());
    }

    #[test]
    fn unresolved_origin_indices_empty_for_canonical_sh_wpkh() {
        // sh(wpkh(@0)) → BIP-49 canonical (F-A1) → [].
        let d = single_key_descriptor(Node {
            tag: Tag::Sh,
            body: Body::Children(vec![Node {
                tag: Tag::Wpkh,
                body: Body::KeyArg { index: 0 },
            }]),
        });
        assert_eq!(d.unresolved_origin_indices(), Vec::<u8>::new());
    }

    #[test]
    fn unresolved_origin_indices_empty_for_canonical_wsh_multi() {
        // wsh(multi(2,@0,@1)) → BIP-48 type-2 canonical → [].
        let d = Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Shared(empty_path()),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Wsh,
                body: Body::Children(vec![Node {
                    tag: Tag::Multi,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: vec![0, 1],
                    },
                }]),
            },
            tlv: TlvSection::new_empty(),
        };
        assert_eq!(d.unresolved_origin_indices(), Vec::<u8>::new());
    }

    #[test]
    fn unresolved_origin_indices_empty_for_explicit_origin_dead_shape() {
        // sh(sortedmulti(2,@0,@1)) is a dead shape (canonical_origin ==
        // None) but the shared path_decl is EXPLICITLY populated → every
        // idx resolves → [].
        let d = Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Shared(bip84_path()),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Sh,
                body: Body::Children(vec![Node {
                    tag: Tag::SortedMulti,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: vec![0, 1],
                    },
                }]),
            },
            tlv: TlvSection::new_empty(),
        };
        assert_eq!(d.unresolved_origin_indices(), Vec::<u8>::new());
    }

    #[test]
    fn unresolved_origin_indices_single_for_tr_with_taptree() {
        // tr(@0, TapTree) → no canonical default → [0].
        let d = single_key_descriptor(Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: Some(Box::new(Node {
                    tag: Tag::PkK,
                    body: Body::KeyArg { index: 0 },
                })),
            },
        });
        assert_eq!(d.unresolved_origin_indices(), vec![0u8]);
    }

    #[test]
    fn unresolved_origin_indices_both_for_tr_with_taptree_two_keys() {
        // tr(@0, pk(@1)) — key-path @0 + tap leaf pk(@1). canonical_origin
        // is None (Tr with Some(tree)), so BOTH indices are unresolved
        // with an empty shared path_decl.
        let d = Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Shared(empty_path()),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Tr,
                body: Body::Tr {
                    is_nums: false,
                    key_index: 0,
                    tree: Some(Box::new(Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg { index: 1 },
                    })),
                },
            },
            tlv: TlvSection::new_empty(),
        };
        assert_eq!(d.unresolved_origin_indices(), vec![0u8, 1u8]);
    }

    #[test]
    fn unresolved_origin_indices_both_for_sh_sortedmulti_dead() {
        // sh(sortedmulti(2,@0,@1)) — legacy P2SH multi, dead shape, empty
        // shared path_decl, no overrides → [0, 1].
        let d = Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Shared(empty_path()),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Sh,
                body: Body::Children(vec![Node {
                    tag: Tag::SortedMulti,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: vec![0, 1],
                    },
                }]),
            },
            tlv: TlvSection::new_empty(),
        };
        assert_eq!(d.unresolved_origin_indices(), vec![0u8, 1u8]);
    }

    #[test]
    fn unresolved_origin_indices_single_for_bare_wsh() {
        // bare wsh(@0) — non-canonical (no multi/sortedmulti inner) → [0].
        let d = single_key_descriptor(Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::PkK,
                body: Body::KeyArg { index: 0 },
            }]),
        });
        assert_eq!(d.unresolved_origin_indices(), vec![0u8]);
    }

    #[test]
    fn unresolved_origin_indices_both_for_raw_miniscript_body() {
        // wsh(or_d(pk_k(@0), pk_h(@1))) — raw miniscript body, dead shape
        // → [0, 1].
        let d = Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Shared(empty_path()),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Wsh,
                body: Body::Children(vec![Node {
                    tag: Tag::OrD,
                    body: Body::Children(vec![
                        Node {
                            tag: Tag::PkK,
                            body: Body::KeyArg { index: 0 },
                        },
                        Node {
                            tag: Tag::PkH,
                            body: Body::KeyArg { index: 1 },
                        },
                    ]),
                }]),
            },
            tlv: TlvSection::new_empty(),
        };
        assert_eq!(d.unresolved_origin_indices(), vec![0u8, 1u8]);
    }

    #[test]
    fn unresolved_origin_indices_partial_divergent() {
        // sh(sortedmulti(...)) with divergent path_decl; @0 populated, @1
        // empty, no overrides → [1] only.
        let d = Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Divergent(vec![bip84_path(), empty_path()]),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Sh,
                body: Body::Children(vec![Node {
                    tag: Tag::SortedMulti,
                    body: Body::MultiKeys {
                        k: 1,
                        indices: vec![0, 1],
                    },
                }]),
            },
            tlv: TlvSection::new_empty(),
        };
        assert_eq!(d.unresolved_origin_indices(), vec![1u8]);
    }

    #[test]
    fn unresolved_origin_indices_empty_when_override_resolves_dead_shape() {
        // sh(sortedmulti(2,@0,@1)) dead shape, empty shared path_decl, but
        // a NON-EMPTY override resolves @0 → only @1 unresolved.
        let d = Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Shared(empty_path()),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Sh,
                body: Body::Children(vec![Node {
                    tag: Tag::SortedMulti,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: vec![0, 1],
                    },
                }]),
            },
            tlv: {
                let mut t = TlvSection::new_empty();
                t.origin_path_overrides = Some(vec![(0u8, bip84_path())]);
                t
            },
        };
        assert_eq!(d.unresolved_origin_indices(), vec![1u8]);
    }
}

#[cfg(test)]
mod xpub_bytes_tests {
    use super::*;
    use crate::origin_path::{OriginPath, PathDecl, PathDeclPaths};
    use crate::tag::Tag;
    use crate::tlv::TlvSection;
    use crate::tree::{Body, Node};
    use crate::use_site_path::UseSitePath;

    /// G (the secp256k1 generator) compressed: 0x02 || x(G).
    /// Used for "valid pubkey" tests.
    fn valid_compressed_g() -> [u8; 33] {
        // x(G) = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798
        let mut out = [0u8; 33];
        out[0] = 0x02;
        let x: [u8; 32] = [
            0x79, 0xBE, 0x66, 0x7E, 0xF9, 0xDC, 0xBB, 0xAC, 0x55, 0xA0, 0x62, 0x95, 0xCE, 0x87,
            0x0B, 0x07, 0x02, 0x9B, 0xFC, 0xDB, 0x2D, 0xCE, 0x28, 0xD9, 0x59, 0xF2, 0x81, 0x5B,
            0x16, 0xF8, 0x17, 0x98,
        ];
        out[1..].copy_from_slice(&x);
        out
    }

    fn descriptor_with_pubkeys(pks: Option<Vec<(u8, [u8; 65])>>) -> Descriptor {
        let mut d = Descriptor {
            n: 1,
            path_decl: PathDecl {
                n: 1,
                paths: PathDeclPaths::Shared(OriginPath { components: vec![] }),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: Node {
                tag: Tag::Wpkh,
                body: Body::KeyArg { index: 0 },
            },
            tlv: TlvSection::new_empty(),
        };
        d.tlv.pubkeys = pks;
        d
    }

    #[test]
    fn validate_xpub_bytes_template_only_no_op() {
        let d = descriptor_with_pubkeys(None);
        validate_xpub_bytes(&d).unwrap();
    }

    #[test]
    fn validate_xpub_bytes_passes_for_valid_compressed_pubkey() {
        let mut xpub = [0u8; 65];
        // Chain code 0..32 — arbitrary 32 bytes are valid.
        for (i, b) in xpub[0..32].iter_mut().enumerate() {
            *b = i as u8;
        }
        // Compressed pubkey 32..65 = G.
        xpub[32..65].copy_from_slice(&valid_compressed_g());
        let d = descriptor_with_pubkeys(Some(vec![(0u8, xpub)]));
        validate_xpub_bytes(&d).unwrap();
    }

    #[test]
    fn validate_xpub_bytes_fails_for_invalid_pubkey_prefix() {
        // Prefix 0x04 is uncompressed-marker; not a valid 33-byte compressed
        // pubkey prefix (only 0x02 / 0x03 are).
        let mut xpub = [0u8; 65];
        xpub[32] = 0x04;
        let d = descriptor_with_pubkeys(Some(vec![(0u8, xpub)]));
        let err = validate_xpub_bytes(&d).unwrap_err();
        assert!(matches!(err, Error::InvalidXpubBytes { idx: 0 }));
    }

    #[test]
    fn validate_xpub_bytes_fails_for_off_curve_x_coordinate() {
        // 0x02 || all-0xFF x-coord. x = p-1 wraps to a non-curve x in
        // most cases; in particular this exact value fails to lift in
        // libsecp256k1's compressed-point parser. Verify via the same
        // routine the validator uses.
        let mut xpub = [0u8; 65];
        xpub[32] = 0x02;
        for b in xpub[33..65].iter_mut() {
            *b = 0xFF;
        }
        // Sanity: confirm bitcoin's parser actually rejects this, so the
        // test exercises the failure path in our validator.
        assert!(bitcoin::secp256k1::PublicKey::from_slice(&xpub[32..65]).is_err());
        let d = descriptor_with_pubkeys(Some(vec![(0u8, xpub)]));
        let err = validate_xpub_bytes(&d).unwrap_err();
        assert!(matches!(err, Error::InvalidXpubBytes { idx: 0 }));
    }

    #[test]
    fn validate_xpub_bytes_reports_first_failing_idx() {
        // Two entries: idx=0 valid, idx=2 invalid → error reports idx=2.
        let mut good = [0u8; 65];
        good[32..65].copy_from_slice(&valid_compressed_g());
        let mut bad = [0u8; 65];
        bad[32] = 0x04; // invalid prefix
        let d = descriptor_with_pubkeys(Some(vec![(0u8, good), (2u8, bad)]));
        let err = validate_xpub_bytes(&d).unwrap_err();
        assert!(matches!(err, Error::InvalidXpubBytes { idx: 2 }));
    }
}
