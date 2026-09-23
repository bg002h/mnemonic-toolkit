//! The lowering proper (spec §5): path body, wsh chain, slot numbering, §4f
//! origins and the shared `finish`. Taproot's spine lives in `tr.rs` and uses
//! the `pub(super)` pieces here.

use super::{
    ComposeError, Composed, Experimental, HashKind, KeySet, PathList, Slot, SlotOrigin, SpendPath,
    UnspendableKind, Wrapper, default_origin,
};
use crate::encode::Descriptor;
use crate::origin_path::{OriginPath, PathDecl, PathDeclPaths};
use crate::tag::Tag;
use crate::tlv::TlvSection;
use crate::tree::{Body, Node};
use crate::use_site_path::UseSitePath;

/// A path with its slot indices already assigned.
pub(super) struct Numbered<'a> {
    pub(super) path: &'a SpendPath,
    pub(super) path_index: usize,
    pub(super) slots: Vec<u8>,
}

fn key_leaf(
    tag_single: Tag,
    tag_multi: Tag,
    tag_sorted: Tag,
    ks: KeySet,
    slots: &[u8],
    sorted_legal: bool,
) -> Node {
    if ks.n == 1 {
        return Node {
            tag: tag_single,
            body: Body::KeyArg { index: slots[0] },
        };
    }
    let tag = if sorted_legal && ks.sorted {
        tag_sorted
    } else {
        tag_multi
    };
    Node {
        tag,
        body: Body::MultiKeys {
            k: ks.k,
            indices: slots.to_vec(),
        },
    }
}

fn verify(x: Node) -> Node {
    Node {
        tag: Tag::Verify,
        body: Body::Children(vec![x]),
    }
}

fn and_v(a: Node, b: Node) -> Node {
    Node {
        tag: Tag::AndV,
        body: Body::Children(vec![verify(a), b]),
    }
}

/// `and_v(v:KEYS, and_v(v:<hashop>(H), LOCK))`, dropping absent parts (spec §5).
/// `<hashop>` is the path's `HashLock` kind -- sha256, hash256, ripemd160 or
/// hash160 -- not a literal (SPEC_hashlock_kinds §3 F1).
pub(super) fn path_body(p: &Numbered<'_>, tap: bool, sorted_legal: bool) -> Node {
    let mut parts: Vec<Node> = Vec::with_capacity(3);
    if let Some(ks) = p.path.keys {
        let (single, multi, sorted) = if tap {
            (Tag::PkK, Tag::MultiA, Tag::SortedMultiA)
        } else {
            (Tag::PkH, Tag::Multi, Tag::SortedMulti)
        };
        parts.push(key_leaf(single, multi, sorted, ks, &p.slots, sorted_legal));
    }
    if let Some(h) = p.path.hash {
        // THE BODY'S WIDTH IS THE KIND'S, never 32 by default. The digest is
        // stored in a fixed [u8; 32] for the alloc gate (spec §5), so a
        // 20-byte kind carries twelve bytes of padding that `digest()` hides;
        // writing the whole array here would commit that padding into the
        // script -- which compiles, round-trips, and cannot be spent.
        //
        // Matched on the KIND, not on digest_len(), so a fifth kind is a
        // compile error here rather than a silent fall-through.
        let body = match h.kind() {
            HashKind::Sha256 | HashKind::Hash256 => {
                let mut b = [0u8; 32];
                b.copy_from_slice(h.digest());
                Body::Hash256Body(b)
            }
            HashKind::Ripemd160 | HashKind::Hash160 => {
                let mut b = [0u8; 20];
                b.copy_from_slice(h.digest());
                Body::Hash160Body(b)
            }
        };
        parts.push(Node {
            tag: h.kind().tag(),
            body,
        });
    }
    if let Some(lock) = p.path.lock {
        let (tag, operand) = lock.operand().expect("validated by `validate`");
        parts.push(Node {
            tag,
            body: Body::Timelock(operand),
        });
    }
    let mut it = parts.into_iter().rev();
    let mut acc = it
        .next()
        .expect("a path has at least one part after validation");
    for part in it {
        acc = and_v(part, acc);
    }
    acc
}

/// Listed order, recursive, last path alone: `or_d` iff the head is a bare
/// multi-key set, else `or_i` (spec §5, C21, C23).
fn wsh_chain(paths: &[Numbered<'_>]) -> Node {
    let sole = paths.len() == 1;
    let mut nodes: Vec<Node> = paths
        .iter()
        .map(|p| path_body(p, false, sole && p.path.is_bare_multi()))
        .collect();
    let mut acc = nodes.pop().expect("at least one path");
    let heads = &paths[..paths.len() - 1];
    for (p, node) in heads.iter().zip(nodes).rev() {
        let tag = if p.path.is_bare_multi() {
            Tag::OrD
        } else {
            Tag::OrI
        };
        acc = Node {
            tag,
            body: Body::Children(vec![node, acc]),
        };
    }
    acc
}

/// Slot numbering by first appearance in the EMITTED text (spec §5). For
/// wsh that is listed order; for tr the extracted internal key comes first.
/// The returned `Numbered` list is in LISTED order regardless.
pub(super) fn number(list: &PathList, first: Option<usize>) -> (Vec<Numbered<'_>>, Vec<Slot>) {
    let mut order: Vec<usize> = Vec::with_capacity(list.paths.len());
    if let Some(f) = first {
        order.push(f);
    }
    order.extend((0..list.paths.len()).filter(|i| Some(*i) != first));
    let mut next: u8 = 0;
    let mut slots = Vec::new();
    let mut by_path: Vec<Option<Numbered<'_>>> = (0..list.paths.len()).map(|_| None).collect();
    for pi in order {
        let p = &list.paths[pi];
        let mut mine = Vec::new();
        if let Some(ks) = p.keys {
            for ordinal in 0..ks.n {
                slots.push(Slot {
                    index: next,
                    path: pi,
                    ordinal,
                });
                mine.push(next);
                next += 1;
            }
        }
        by_path[pi] = Some(Numbered {
            path: p,
            path_index: pi,
            slots: mine,
        });
    }
    let numbered: Vec<Numbered<'_>> = by_path.into_iter().flatten().collect();
    (numbered, slots)
}

/// The EXPERIMENTAL marks: every keyless path; every path that asked for
/// unsorted keys at a position where sorted was legal.
pub(super) fn experimental(
    list: &PathList,
    sole_sorted_legal: impl Fn(usize) -> bool,
) -> Vec<Experimental> {
    let mut out = Vec::new();
    for (i, p) in list.paths.iter().enumerate() {
        match p.keys {
            None => out.push(Experimental::KeylessPath(i)),
            Some(ks) if ks.n >= 2 && !ks.sorted && sole_sorted_legal(i) => {
                out.push(Experimental::UnsortedKeys(i))
            }
            _ => {}
        }
    }
    out
}

/// §4f: declared origins for seated slots; the lowest free default account
/// for unseated ones; the pairwise-distinguishability invariant.
#[allow(clippy::type_complexity)]
fn origins(
    list: &PathList,
    declared: &[Option<SlotOrigin>],
) -> Result<(PathDecl, Option<Vec<(u8, [u8; 4])>>), ComposeError> {
    let n = declared.len();
    let mut per_slot: Vec<Option<SlotOrigin>> = declared.to_vec();
    let mut taken: Vec<OriginPath> = per_slot
        .iter()
        .flatten()
        .map(|s| s.origin.clone())
        .collect();
    for slot in per_slot.iter_mut() {
        if slot.is_none() {
            let mut account: u32 = 0;
            loop {
                let candidate = default_origin(list.wrapper, account);
                if !taken.contains(&candidate) {
                    taken.push(candidate.clone());
                    *slot = Some(SlotOrigin {
                        origin: candidate,
                        fingerprint: None,
                    });
                    break;
                }
                account += 1;
            }
        }
    }
    let resolved: Vec<SlotOrigin> = per_slot
        .into_iter()
        .map(|s| s.expect("filled above"))
        .collect();
    for a in 0..n {
        for b in (a + 1)..n {
            if resolved[a].origin == resolved[b].origin {
                let distinct = match (resolved[a].fingerprint, resolved[b].fingerprint) {
                    (Some(x), Some(y)) => x != y,
                    _ => false,
                };
                if !distinct {
                    return Err(ComposeError::IndistinguishableSlots {
                        a: a as u8,
                        b: b as u8,
                    });
                }
            }
        }
    }
    let all_same = resolved.windows(2).all(|w| w[0].origin == w[1].origin);
    let paths = if all_same {
        PathDeclPaths::Shared(resolved[0].origin.clone())
    } else {
        PathDeclPaths::Divergent(resolved.iter().map(|s| s.origin.clone()).collect())
    };
    let fps: Vec<(u8, [u8; 4])> = resolved
        .iter()
        .enumerate()
        .filter_map(|(i, s)| s.fingerprint.map(|fp| (i as u8, fp)))
        .collect();
    let fingerprints = if fps.is_empty() { None } else { Some(fps) };
    Ok((PathDecl { n: n as u8, paths }, fingerprints))
}

/// Assemble the `Descriptor` around a finished tree.
pub(super) fn finish(
    list: &PathList,
    declared: &[Option<SlotOrigin>],
    tree: Node,
    slots: Vec<Slot>,
    internal_key_path: Option<usize>,
    experimental: Vec<Experimental>,
) -> Result<Composed, ComposeError> {
    let (path_decl, fingerprints) = origins(list, declared)?;
    let mut tlv = TlvSection::new_empty();
    tlv.fingerprints = fingerprints;
    let descriptor = Descriptor {
        n: declared.len() as u8,
        path_decl,
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv,
    };
    Ok(Composed {
        descriptor,
        slots,
        internal_key_path,
        experimental,
        // Set by `compose_with`, the one entry point that knows the request.
        unspendable_request_unmet: false,
    })
}

pub(super) fn lower(
    list: &PathList,
    declared: &[Option<SlotOrigin>],
    unspendable: UnspendableKind,
) -> Result<Composed, ComposeError> {
    match list.wrapper {
        Wrapper::Tr => super::tr::lower_tr(list, declared, unspendable),
        Wrapper::Wsh | Wrapper::Sh | Wrapper::ShWsh => {
            let (numbered, slots) = number(list, None);
            let sole = list.paths.len() == 1;
            let inner = wsh_chain(&numbered);
            let tree = match list.wrapper {
                Wrapper::Sh => Node {
                    tag: Tag::Sh,
                    body: Body::Children(vec![inner]),
                },
                Wrapper::ShWsh => Node {
                    tag: Tag::Sh,
                    body: Body::Children(vec![Node {
                        tag: Tag::Wsh,
                        body: Body::Children(vec![inner]),
                    }]),
                },
                _ => Node {
                    tag: Tag::Wsh,
                    body: Body::Children(vec![inner]),
                },
            };
            let exp = experimental(list, |i| sole && list.paths[i].is_bare_multi());
            finish(list, declared, tree, slots, None, exp)
        }
    }
}
