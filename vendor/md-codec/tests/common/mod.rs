//! Shared generators + helpers for the md-codec test-hardening suite.
//! Consumed by proptest_roundtrip.rs and bch_adversarial.rs via `mod common;`.
#![allow(dead_code, unused_imports)]

use md_codec::canonicalize::canonicalize_placeholder_indices;
use md_codec::encode::Descriptor;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, InternalKey, Node};
use md_codec::use_site_path::UseSitePath;
use proptest::prelude::*;

fn divergent_path(n: u8, depth: u8) -> PathDecl {
    let paths = (0..n)
        .map(|c| OriginPath {
            components: (0..depth)
                .map(|i| PathComponent {
                    hardened: true,
                    value: (c as u32) * 100 + (i as u32) + 1,
                })
                .collect(),
        })
        .collect();
    PathDecl {
        n,
        paths: PathDeclPaths::Divergent(paths),
    }
}

pub fn wrap(tag: Tag, inner: Node) -> Node {
    Node {
        tag,
        body: Body::Children(vec![inner]),
    }
}
pub fn keyarg(tag: Tag, index: u8) -> Node {
    Node {
        tag,
        body: Body::KeyArg { index },
    }
}
pub fn multikeys(tag: Tag, k: u8, indices: Vec<u8>) -> Node {
    Node {
        tag,
        body: Body::MultiKeys { k, indices },
    }
}
pub fn node2(tag: Tag, a: Node, b: Node) -> Node {
    Node {
        tag,
        body: Body::Children(vec![a, b]),
    }
}
pub fn node3(tag: Tag, a: Node, b: Node, c: Node) -> Node {
    Node {
        tag,
        body: Body::Children(vec![a, b, c]),
    }
}
pub fn thresh_node(k: u8, children: Vec<Node>) -> Node {
    Node {
        tag: Tag::Thresh,
        body: Body::Variable { k, children },
    }
}
pub fn timelock(tag: Tag, v: u32) -> Node {
    Node {
        tag,
        body: Body::Timelock(v),
    }
}
pub fn hash32(tag: Tag, h: [u8; 32]) -> Node {
    Node {
        tag,
        body: Body::Hash256Body(h),
    }
}
pub fn hash20(tag: Tag, h: [u8; 20]) -> Node {
    Node {
        tag,
        body: Body::Hash160Body(h),
    }
}
pub fn tr_node(is_nums: bool, key_index: u8, tree: Option<Node>) -> Node {
    Node {
        tag: Tag::Tr,
        body: Body::Tr {
            internal_key: if is_nums {
                InternalKey::NumsPoint
            } else {
                InternalKey::Slot(key_index)
            },
            tree: tree.map(Box::new),
        },
    }
}
pub fn taptree2(l: Node, r: Node) -> Node {
    Node {
        tag: Tag::TapTree,
        body: Body::Children(vec![l, r]),
    }
}
/// Build a template-only `Descriptor` around `tree` with `n` placeholders.
/// (Tests are a separate crate, so `md_codec::` is correct HERE.)
/// Template-only is deliberate: Task 1 tests structure, not key identity.
///
/// None of `PathDecl`, `UseSitePath` or `TlvSection` derives `Default`, so
/// this uses the spelling `rg 'Descriptor \{' crates/md-codec/tests` already
/// establishes (`crates/md-codec/tests/sh_wpkh_canonical.rs`'s
/// `sh_wpkh_descriptor`): an empty shared origin path, the standard
/// `<0;1>/*` use-site path, and `TlvSection::new_empty()`.
pub fn descriptor_of(tree: Node, n: u8) -> md_codec::encode::Descriptor {
    md_codec::encode::Descriptor {
        n,
        path_decl: PathDecl {
            n,
            paths: PathDeclPaths::Shared(OriginPath { components: vec![] }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection::new_empty(),
    }
}

/// `wsh(or_i(and_v(v:pkh(@0),older(a)), or_i(and_v(v:pkh(@1),older(b)),
/// and_v(v:pkh(@2),older(c)))))` — three `older` branches, so the abstract
/// renderer's per-kind class numbering (equal values share a class,
/// different ones don't) has something to distinguish.
pub fn three_older_descriptor(a: u32, b: u32, c: u32) -> md_codec::encode::Descriptor {
    let branch = |i: u8, v: u32| {
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::Pkh, i)),
            timelock(Tag::Older, v),
        )
    };
    let tree = wrap(
        Tag::Wsh,
        node2(
            Tag::OrI,
            branch(0, a),
            node2(Tag::OrI, branch(1, b), branch(2, c)),
        ),
    );
    descriptor_of(tree, 3)
}

/// `wsh(or_i(and_v(v:pkh(@0),after(height)),and_v(v:pkh(@1),after(time))))`
/// — one `after` branch below `LOCKTIME_THRESHOLD` (a height) and one at or
/// above it (a time), so the abstract renderer's height/time band split has
/// a case in each band, straddling the boundary a mutation of
/// `LOCKTIME_THRESHOLD` would move.
pub fn two_after_descriptor(height: u32, time: u32) -> md_codec::encode::Descriptor {
    let branch = |i: u8, v: u32| {
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::Pkh, i)),
            timelock(Tag::After, v),
        )
    };
    let tree = wrap(
        Tag::Wsh,
        node2(Tag::OrI, branch(0, height), branch(1, time)),
    );
    descriptor_of(tree, 2)
}

/// `wsh(or_i(and_v(v:pkh(@0),older(blocks)),and_v(v:pkh(@1),older(units))))`
/// — final whole-branch review, I-2: the `older` equivalent of
/// [`two_after_descriptor`]. One `older` branch below `SEQUENCE_TYPE_FLAG`
/// (blocks) and one with it set (512-second units), so the abstract
/// renderer's blocks/units band split has a case in each band — Task 2
/// added this coverage for `after`'s height/time bands and never the
/// `older` equivalent.
pub fn two_older_descriptor(blocks: u32, units: u32) -> md_codec::encode::Descriptor {
    let branch = |i: u8, v: u32| {
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::Pkh, i)),
            timelock(Tag::Older, v),
        )
    };
    let tree = wrap(
        Tag::Wsh,
        node2(Tag::OrI, branch(0, blocks), branch(1, units)),
    );
    descriptor_of(tree, 2)
}

/// `wsh(or_i(and_v(v:pkh(@0),sha256(a)),and_v(v:pkh(@1),sha256(b))))` — two
/// `sha256` branches, symmetric with [`three_older_descriptor`] but for
/// digests rather than lock values.
pub fn two_sha256_descriptor(a: [u8; 32], b: [u8; 32]) -> md_codec::encode::Descriptor {
    let branch = |i: u8, h: [u8; 32]| {
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::Pkh, i)),
            hash32(Tag::Sha256, h),
        )
    };
    let tree = wrap(Tag::Wsh, node2(Tag::OrI, branch(0, a), branch(1, b)));
    descriptor_of(tree, 2)
}

/// `wsh(or_d(multi(2,@0,@1,@2), and_v(v:pkh(@3), older(26280))))` — same
/// shape as `tests/policy_shape.rs`'s private `kofn_recovery()`, reused here
/// to pin that abstracting a lock leaves each key's use-site path
/// (`descriptor_of`'s standard `/<0;1>/*` multipath) untouched.
pub fn kofn_recovery_with_use_site() -> md_codec::encode::Descriptor {
    let primary = multikeys(Tag::Multi, 2, vec![0, 1, 2]);
    let recovery = node2(
        Tag::AndV,
        wrap(Tag::Verify, keyarg(Tag::Pkh, 3)),
        timelock(Tag::Older, 26280),
    );
    let tree = wrap(Tag::Wsh, node2(Tag::OrD, primary, recovery));
    descriptor_of(tree, 4)
}

// ─────────────────────────────────────────────────────────────────────────
// Task 3 (fp_partition / key_partition) fixtures.
// Shape: `wsh(or_d(multi(2,@0,@1,@2), and_v(v:pkh(@3), older(26280))))` —
// the SAME shape as `tests/policy_shape.rs`'s private `kofn_recovery()`
// (Task 1), reused here so `policy_shape`'s branch decomposition is a fact
// already pinned by Task 1's suite: branch 0 (the `multi`) references slots
// [0,1,2], branch 1 (the recovery leg) references slot [3].
// ─────────────────────────────────────────────────────────────────────────

/// Build the `kofn_recovery` tree with an EXPLICIT non-empty shared origin
/// path (`m/48'`), so `expand_per_at_n` succeeds instead of raising
/// `MissingExplicitOrigin` — `wsh(or_d(...))` is not one of
/// `canonical_origin`'s recognized shapes, and `descriptor_of`'s default
/// shared path is empty.
fn kofn_recovery_tree() -> Node {
    let primary = multikeys(Tag::Multi, 2, vec![0, 1, 2]);
    let recovery = node2(
        Tag::AndV,
        wrap(Tag::Verify, keyarg(Tag::Pkh, 3)),
        timelock(Tag::Older, 26280),
    );
    wrap(Tag::Wsh, node2(Tag::OrD, primary, recovery))
}

/// `wsh(or_d(multi(2,@0,@1,@2), and_v(v:pkh(@3), older(26280))))`,
/// template-only (no TLVs) — the same shape `tests/policy_shape.rs`'s
/// private `kofn_recovery()` pins, reused here as a card with no key
/// material at all: `fp_partition`/`key_partition` have nothing to group.
///
/// Carries the same EXPLICIT non-empty shared origin path every sibling
/// built on `kofn_recovery_tree()` (`seated`, `seated_pubkeys`,
/// `seated_same_key`) already sets, for the same reason stated on
/// `shared_origin_48`: `wsh(or_d(...))` is not one of `canonical_origin`'s
/// recognized shapes, so `descriptor_of`'s default empty shared path leaves
/// `@0..@3`'s origin unresolved. Added for Task 4 (`skeleton_key.rs`'s
/// `seated_and_unseated_do_not_share_a_key` / `the_serialization_is_stable_
/// and_documented`, both call `skeleton(&kofn_recovery()).unwrap()`):
/// `skeleton()` gates on `expand_per_at_n` succeeding BEFORE building a
/// `Skeleton` at all (design §1A (a3)), so a `kofn_recovery()` with an
/// unresolved origin would refuse there regardless of holding zero TLV
/// data. Does not change `partitions.rs`'s
/// `a_template_only_card_partitions_to_nothing`: `fp_partition`/
/// `key_partition` still come back empty with a resolvable origin -- every
/// slot's `expanded.get(slot).and_then(|e| e.fingerprint / .xpub)` is
/// `None` regardless, since no TLV is attached, so `group_ascending` skips
/// every slot exactly as before, just via per-slot absence instead of a
/// whole-descriptor expand failure.
pub fn kofn_recovery() -> md_codec::encode::Descriptor {
    let mut d = descriptor_of(kofn_recovery_tree(), 4);
    d.path_decl = shared_origin_48(4);
    d
}

/// The explicit non-empty shared origin path (`m/48'`) every Task 3 fixture
/// below uses, so `expand_per_at_n` resolves instead of raising
/// `MissingExplicitOrigin` — `wsh(or_d(...))` is not one of
/// `canonical_origin`'s recognized shapes, and `descriptor_of`'s default
/// shared path is empty. Factored out (fix round 1, I-1/I-2) once a third
/// and fourth fixture needed the identical block.
fn shared_origin_48(n: u8) -> PathDecl {
    PathDecl {
        n,
        paths: PathDeclPaths::Shared(OriginPath {
            components: vec![PathComponent {
                hardened: true,
                value: 48,
            }],
        }),
    }
}

/// The same `kofn_recovery` shape, with a per-`@N` fingerprint TLV built
/// from `fps` (ascending `(idx, fingerprint)` pairs — callers pass every
/// slot's fingerprint including the ABSENT `[0u8; 4]` sentinel where wanted)
/// and an explicit non-empty shared origin path so `expand_per_at_n`
/// resolves instead of refusing on `MissingExplicitOrigin`.
pub fn seated(fps: &[(u8, [u8; 4])]) -> md_codec::encode::Descriptor {
    let mut d = descriptor_of(kofn_recovery_tree(), 4);
    d.path_decl = shared_origin_48(4);
    d.tlv.fingerprints = Some(fps.to_vec());
    d
}

/// The same `kofn_recovery` shape, with a per-`@N` xpub (`Pubkeys`) TLV
/// built from `pks` (ascending `(idx, xpub bytes)` pairs — callers pass the
/// ABSENT `[0u8; 65]` sentinel for any slot that should carry no key) and
/// the shared explicit origin path, so `expand_per_at_n` resolves. Mirrors
/// `seated`, but exercises `key_partition`'s xpub half rather than
/// `fp_partition`'s fingerprint half — added fix round 1, I-1: the absent-
/// xpub singleton rule had zero coverage crate-wide before this.
pub fn seated_pubkeys(pks: &[(u8, [u8; 65])]) -> md_codec::encode::Descriptor {
    let mut d = descriptor_of(kofn_recovery_tree(), 4);
    d.path_decl = shared_origin_48(4);
    d.tlv.pubkeys = Some(pks.to_vec());
    d
}

/// The same `kofn_recovery` shape (slots @0..@3, split across the two
/// branches Task 1 pins), with a `Pubkeys` TLV where every slot named in
/// `same_key_idxs` carries the IDENTICAL 65-byte xpub, every other slot
/// carries a distinct one, and every slot resolves to the SAME shared
/// origin path — so the only thing that can make two slots share a
/// `key_partition` group is `same_key_idxs` itself. Exercises the
/// WHOLE-POLICY relation across branches: @0 lives in the primary `multi`
/// branch, @3 in the recovery branch, and `key_partition` — unlike
/// `fp_partition` — must still see them as one key.
pub fn seated_same_key(same_key_idxs: &[u8]) -> md_codec::encode::Descriptor {
    let mut d = descriptor_of(kofn_recovery_tree(), 4);
    d.path_decl = shared_origin_48(4);
    let shared_xpub = {
        let mut x = [0x11u8; 65];
        x[32] = 0x02; // a distinct, valid-looking compressed-pubkey prefix
        x
    };
    let pubkeys = (0..4u8)
        .map(|i| {
            if same_key_idxs.contains(&i) {
                (i, shared_xpub)
            } else {
                // Distinct per-slot filler, never colliding with
                // `shared_xpub` or another filler slot.
                let mut x = [0x22u8 + i; 65];
                x[32] = 0x03;
                (i, x)
            }
        })
        .collect();
    d.tlv.pubkeys = Some(pubkeys);
    d
}

/// `seated_same_key`, with a per-`@N` `OriginPathOverrides` TLV layered on
/// top — so a caller can move one of `same_key_idxs`' slots to a DIFFERENT
/// resolved origin than the shared `m/48'` baseline while its xpub bytes
/// stay identical to the others'. `origin_overrides` empty is byte-for-byte
/// `seated_same_key`. Added fix round 1, I-2: `key_partition`'s
/// `origin_path` half of its `(xpub, origin_path)` key had zero coverage —
/// every prior fixture put every slot at the one shared baseline path, so
/// nothing could tell "grouped because same key" apart from "grouped
/// because the origin check was never applied at all".
pub fn seated_same_key_with_overrides(
    same_key_idxs: &[u8],
    origin_overrides: &[(u8, OriginPath)],
) -> md_codec::encode::Descriptor {
    let mut d = seated_same_key(same_key_idxs);
    if !origin_overrides.is_empty() {
        d.tlv.origin_path_overrides = Some(origin_overrides.to_vec());
    }
    d
}

// ─────────────────────────────────────────────────────────────────────────
// Task 4 (Skeleton / SkeletonKey) fixtures.
// ─────────────────────────────────────────────────────────────────────────

/// `wsh(or_d(and_v(v:pkh(@3), older(26280)), multi(2,@0,@1,@2)))` — the
/// `kofn_recovery` shape with its two `or_d` operands SWAPPED (recovery leg
/// first, the multi second), so branch 0 is the HIGH-numbered slot [3] and
/// branch 1 is the LOW-numbered slots [0,1,2]. This is deliberate, not a
/// typo: with the un-swapped order, branch 0's rendered group list always
/// starts with a lower digit than branch 1's (branch 0 always holds the
/// smaller placeholder indices), so it is *already* in ascending
/// lexicographic order — a mutation that `sort()`s the per-path list before
/// emitting it is then a no-op and cannot be caught. Swapping the operands
/// makes branch 0's group list ("[[3]]"-shaped) sort lexicographically AFTER
/// branch 1's ("[[0,1][2]]"-shaped, a comma-bearing group) — the string a
/// `sort()` mutation produces then differs from the branch (template
/// traversal) order this crate's grammar requires.
fn kofn_recovery_first_tree() -> Node {
    let recovery = node2(
        Tag::AndV,
        wrap(Tag::Verify, keyarg(Tag::Pkh, 3)),
        timelock(Tag::Older, 26280),
    );
    let primary = multikeys(Tag::Multi, 2, vec![0, 1, 2]);
    wrap(Tag::Wsh, node2(Tag::OrD, recovery, primary))
}

/// The [`kofn_recovery_first_tree`] shape with BOTH a `Fingerprints` TLV
/// (asymmetric: branch 0 -- the recovery leg, slot 3 -- gets one group;
/// branch 1 -- the multi, slots 0/1/2 -- has slots 0 and 1 sharing a
/// fingerprint and slot 2 standing alone, a 2-member group) AND a `Pubkeys`
/// TLV (slots 0 and 3 -- in DIFFERENT branches -- share a key; slots 1 and 2
/// are distinct). Built for the final whole-branch review's I-3: one
/// fixture whose `fp_partition` (multi-path, asymmetric, with a 2-member
/// group, and in an order that `sort()` would actually change — see
/// [`kofn_recovery_first_tree`]) and `key_partition` (non-empty, and
/// DIFFERENT from `fp_partition`) are both non-trivial, so a single golden
/// `SkeletonKey` test can pin all five of the review's serialization
/// mutations: reversing OR sorting the per-path order changes this
/// deliberately-unsorted `fp_partition`; dropping the `,` inside a group
/// changes the `{0,1}` pair; dropping the outer `[`/`]` unbalances a
/// non-empty group list; and swapping the `key_partition`/`fp_partition`
/// emission order changes the string because the two render to different
/// text.
pub fn seated_and_keyed_asymmetric() -> md_codec::encode::Descriptor {
    let mut d = descriptor_of(kofn_recovery_first_tree(), 4);
    d.path_decl = shared_origin_48(4);
    d.tlv.fingerprints = Some(vec![
        (0, [0xaa; 4]),
        (1, [0xaa; 4]),
        (2, [0xbb; 4]),
        (3, [0xcc; 4]),
    ]);
    let shared_xpub = {
        let mut x = [0x11u8; 65];
        x[32] = 0x02;
        x
    };
    let distinct = |seed: u8| {
        let mut x = [seed; 65];
        x[32] = 0x03;
        x
    };
    d.tlv.pubkeys = Some(vec![
        (0, shared_xpub),
        (1, distinct(0x22)),
        (2, distinct(0x33)),
        (3, shared_xpub),
    ]);
    d
}

/// `tr(NUMS,{pk(@0),pk(@1)})` — a taproot policy with a provably
/// unspendable internal key (NUMS) and two tapscript leaves. Needs an
/// EXPLICIT origin (`shared_origin_48`): `canonical_origin` returns `None`
/// for any `tr(...)` carrying a TapTree, `is_nums` notwithstanding
/// (`canonical_origin.rs`'s `(Tag::Tr, Body::Tr { tree: Some(_), .. }) =>
/// None` arm does not consult `is_nums` at all), so `descriptor_of`'s
/// default empty shared path would leave `expand_per_at_n` unable to
/// resolve `@0`/`@1`.
pub fn tr_nums_two_leaves() -> md_codec::encode::Descriptor {
    let tree = tr_node(
        true,
        0,
        Some(taptree2(keyarg(Tag::PkK, 0), keyarg(Tag::PkK, 1))),
    );
    let mut d = descriptor_of(tree, 2);
    d.path_decl = shared_origin_48(2);
    d
}

/// The same shape, with a REAL (non-NUMS) internal key at `@0` — the
/// Nunchuk shape F-449 records: an internal key that is a real xpub but
/// treated as unspendable by a coordinator's own convention, which this
/// walk cannot tell apart from a genuinely spendable one
/// (`KeyPathKind::Xpub`'s own doc comment in `policy_shape.rs`). Same
/// explicit-origin need as `tr_nums_two_leaves` (TapTree present).
pub fn tr_unspendable_xpub_two_leaves() -> md_codec::encode::Descriptor {
    let tree = tr_node(
        false,
        0,
        Some(taptree2(keyarg(Tag::PkK, 1), keyarg(Tag::PkK, 2))),
    );
    let mut d = descriptor_of(tree, 3);
    d.path_decl = shared_origin_48(3);
    d
}

/// `tr(LianaUnspendable, {pk(@0),pk(@1)})` — stage 1b's wire kind 1
/// (SPEC §3d), a THIRD internal-key shape distinct from both
/// `tr_nums_two_leaves` (kind 0, the literal NUMS point) and
/// `tr_unspendable_xpub_two_leaves` (a pre-stage-1b `Slot`-encoded
/// unspendable-by-convention xpub — `KeyPathKind::Xpub`'s own doc comment).
/// Template-only (no `Pubkeys` TLV): for tests that need only the AST shape
/// (render.rs, policy_shape.rs), not real key material.
pub fn tr_liana_unspendable_two_leaves() -> md_codec::encode::Descriptor {
    let tree = Node {
        tag: Tag::Tr,
        body: Body::Tr {
            internal_key: InternalKey::LianaUnspendable,
            tree: Some(Box::new(taptree2(keyarg(Tag::PkK, 0), keyarg(Tag::PkK, 1)))),
        },
    };
    let mut d = descriptor_of(tree, 2);
    d.path_decl = shared_origin_48(2);
    d
}

/// The same shape as [`tr_liana_unspendable_two_leaves`], with a real
/// `Pubkeys` TLV (`test_xpubs()` slots 0 and 1) so `to_miniscript`'s
/// derivation path (`expand_per_at_n`, then the leaf-pubkey walk feeding
/// `nums::liana_unspendable_xpub`) has real key material to run over.
pub fn tr_liana_unspendable_two_leaves_with_pubkeys() -> md_codec::encode::Descriptor {
    let mut d = tr_liana_unspendable_two_leaves();
    d.tlv.pubkeys = Some(vec![(0, test_xpubs()[0]), (1, test_xpubs()[1])]);
    d
}

/// `sh(wsh(multi(2,@0,@1,@2)))` — root=Sh, inner_wsh=true. Canonical
/// (`canonical_origin`'s BIP48-type-1 row), so `descriptor_of`'s default
/// empty shared path resolves without an override.
pub fn sh_wsh_2of3() -> md_codec::encode::Descriptor {
    let tree = wrap(
        Tag::Sh,
        wrap(Tag::Wsh, multikeys(Tag::Multi, 2, vec![0, 1, 2])),
    );
    descriptor_of(tree, 3)
}

/// `sh(wsh(or_d(sortedmulti(2,@0,@1,@2), and_v(v:pkh(@3),older(26280)))))` —
/// final whole-branch review, I-1: `sh_wsh_2of3`'s single-`multi` shape
/// cannot tell whether the `sh(wsh(...))` unwrap in `policy_shape` actually
/// runs (a bare threshold decomposes identically either way). This shape is
/// MULTI-branch, so the unwrap's absence is observable: without it, the two
/// `or_d` alternatives collapse into one branch and every slot set changes.
/// Wire-reachable (reviewer-measured: 18 bytes / 139 bits, strict-decodes
/// byte-identical). NOT canonical: `canonical_origin`'s `sh(wsh(...))` row
/// only recognizes a bare `multi`/`sortedmulti` directly inside the `wsh`
/// (`is_wsh_inner_multi`); this fixture's inner is `or_d(...)`, the same
/// non-canonical shape `kofn_recovery_tree`'s bare `wsh(or_d(...))` is, so it
/// needs the same explicit non-empty shared origin path.
pub fn sh_wsh_or_d_sortedmulti_and_recovery() -> md_codec::encode::Descriptor {
    let primary = multikeys(Tag::SortedMulti, 2, vec![0, 1, 2]);
    let recovery = node2(
        Tag::AndV,
        wrap(Tag::Verify, keyarg(Tag::Pkh, 3)),
        timelock(Tag::Older, 26280),
    );
    let inner = node2(Tag::OrD, primary, recovery);
    let tree = wrap(Tag::Sh, wrap(Tag::Wsh, inner));
    let mut d = descriptor_of(tree, 4);
    d.path_decl = shared_origin_48(4);
    d
}

/// `sh(multi(2,@0,@1,@2))` — root=Sh, inner_wsh=false: the bare legacy
/// P2SH multi `sh_wsh_2of3` is NOT a wrapper spelling of. Legacy
/// `sh(multi)` has no canonical-origin row (`canonical_origin.rs`'s
/// "sh(sortedmulti) legacy … => None" applies identically to `sh(multi)`:
/// neither `is_wsh_inner_multi`-wrapped nor BIP49), so needs an explicit
/// origin.
pub fn bare_sh_2of3() -> md_codec::encode::Descriptor {
    let tree = wrap(Tag::Sh, multikeys(Tag::Multi, 2, vec![0, 1, 2]));
    let mut d = descriptor_of(tree, 3);
    d.path_decl = shared_origin_48(3);
    d
}

/// `wsh(tr(NUMS))` — structurally nonsensical: `tr`/`TapTree` cannot appear
/// inside a branch (`policy_shape::collect`'s own `Tag::Tr | Tag::TapTree
/// => false` arm is the mechanism that refuses it). `n = 0`: the inner `tr`
/// is NUMS-only with no TapTree, so no placeholder is referenced anywhere
/// in this tree — `expand_per_at_n`'s `for idx in 0..d.n` loop is trivially
/// satisfied, and the ONLY reason `skeleton()` refuses this descriptor is
/// `PolicyShape::complete == false`.
pub fn unclassifiable() -> md_codec::encode::Descriptor {
    let tree = wrap(Tag::Wsh, tr_node(true, 0, None));
    descriptor_of(tree, 0)
}

/// A "dead card" (fix round 1, I-1): the SAME `kofn_recovery` shape, with a
/// REAL `Fingerprints` TLV attached, but WITHOUT an explicit origin
/// (`descriptor_of`'s default empty shared path is left as-is — the point
/// of this fixture). This is exactly `DecodeOpts::partial()`'s dead-card
/// decode mode (`md decode`/`md inspect` on a card whose `@N` origin never
/// resolved): real key TLVs intact, origin unresolved. Per
/// `policy_shape::fp_partition`'s own doc comment, WITHOUT `skeleton()`'s
/// `expand_per_at_n` gate this card's `fp_partition`/`key_partition` would
/// collapse to the exact same empty shape as a genuinely template-only card
/// sharing this tree — and since the tree is `kofn_recovery`'s, with the
/// SAME template, the resulting `SkeletonKey` would be byte-identical to
/// `kofn_recovery()`'s. That collision is what the gate exists to prevent.
pub fn dead_card_real_fingerprints_unresolved_origin() -> md_codec::encode::Descriptor {
    let mut d = descriptor_of(kofn_recovery_tree(), 4);
    d.tlv.fingerprints = Some(vec![
        (0, [0xaa; 4]),
        (1, [0xbb; 4]),
        (2, [0xcc; 4]),
        (3, [0xdd; 4]),
    ]);
    d
}

/// n biased to the kiw-width boundaries (exercises kiw 0..5).
fn n_strategy() -> impl Strategy<Value = u8> {
    prop_oneof![
        Just(1u8),
        Just(2),
        Just(3),
        Just(4),
        Just(5),
        Just(8),
        Just(9),
        Just(15),
        Just(16),
        Just(17),
        Just(31),
        Just(32),
        2u8..=32,
    ]
}

/// Bounded-recursion tr() taptree: internal TapTree{Children(2)}; leaves from the
/// permitted allow-list. Leaves reference indices in 1..=max (keypath is @0);
/// descriptor_from_tree renumbers to contiguous 0..n.
fn taptree_strategy(max_key_index: u8) -> impl Strategy<Value = Node> {
    let leaf = prop_oneof![
        (1u8..=max_key_index).prop_map(|i| keyarg(Tag::PkK, i)),
        (1u8..=max_key_index).prop_map(|i| keyarg(Tag::PkH, i)),
        (1u8..=max_key_index).prop_map(|i| multikeys(Tag::MultiA, 1, vec![i])),
        (1u32..=65535).prop_map(|t| Node {
            tag: Tag::Older,
            body: Body::Timelock(t)
        }),
    ];
    leaf.prop_recursive(3, 8, 2, |inner| {
        (inner.clone(), inner).prop_map(|(l, r)| Node {
            tag: Tag::TapTree,
            body: Body::Children(vec![l, r]),
        })
    })
}

/// Distinct placeholder indices referenced by a tree (KeyArg + MultiKeys +
/// non-NUMS Tr.key_index), so n can be derived.
fn referenced_indices(node: &Node, out: &mut std::collections::BTreeSet<u8>) {
    match &node.body {
        Body::KeyArg { index } => {
            out.insert(*index);
        }
        Body::MultiKeys { indices, .. } => {
            out.extend(indices.iter().copied());
        }
        Body::Tr { internal_key, tree } => {
            if let InternalKey::Slot(i) = internal_key {
                out.insert(*i);
            }
            if let Some(t) = tree {
                referenced_indices(t, out);
            }
        }
        Body::Children(cs) => {
            for c in cs {
                referenced_indices(c, out);
            }
        }
        Body::Variable { children, .. } => {
            for c in children {
                referenced_indices(c, out);
            }
        }
        _ => {}
    }
}

/// Rewrite every placeholder index through `perm` (old->new). NUMS Tr.key_index
/// is left untouched (no wire repr), matching referenced_indices.
fn renumber_tree(node: &mut Node, perm: &std::collections::BTreeMap<u8, u8>) {
    match &mut node.body {
        Body::KeyArg { index } => {
            *index = perm[&*index];
        }
        Body::MultiKeys { indices, .. } => {
            for i in indices.iter_mut() {
                *i = perm[&*i];
            }
        }
        Body::Tr { internal_key, tree } => {
            if let InternalKey::Slot(i) = internal_key {
                *internal_key = InternalKey::Slot(perm[&*i]);
            }
            if let Some(t) = tree {
                renumber_tree(t, perm);
            }
        }
        Body::Children(cs) => {
            for c in cs.iter_mut() {
                renumber_tree(c, perm);
            }
        }
        Body::Variable { children, .. } => {
            for c in children.iter_mut() {
                renumber_tree(c, perm);
            }
        }
        _ => {}
    }
}

/// Collect referenced indices and RENUMBER the tree to contiguous 0..n.
/// This is descriptor_from_tree's renumber logic, extracted so every root
/// builder (existing strategy, W tier, T tier, P7/P8 cells) goes through
/// the same path — which is what makes `canon()`'s `.expect` safe.
pub fn renumbered(mut tree: Node) -> (Node, u8) {
    let mut set = std::collections::BTreeSet::new();
    referenced_indices(&tree, &mut set);
    let perm: std::collections::BTreeMap<u8, u8> = set
        .iter()
        .enumerate()
        .map(|(rank, &old)| (old, rank as u8))
        .collect();
    renumber_tree(&mut tree, &perm);
    (tree, set.len() as u8)
}

/// Build a Descriptor: collect referenced indices, RENUMBER the tree to contiguous
/// 0..n, then derive n + path-decl. Explicit-origin shapes get a Divergent path.
pub fn descriptor_from_tree(tree: Node, explicit_origin: bool) -> Descriptor {
    let (tree, n) = renumbered(tree);
    let path_decl = if explicit_origin {
        divergent_path(n, 3)
    } else {
        PathDecl {
            n,
            paths: PathDeclPaths::Shared(OriginPath {
                components: vec![PathComponent {
                    hardened: true,
                    value: 84,
                }],
            }),
        }
    };
    Descriptor {
        n,
        path_decl,
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection::new_empty(),
    }
}

pub fn descriptor_strategy() -> BoxedStrategy<Descriptor> {
    let single_sig = prop_oneof![
        Just(keyarg(Tag::Wpkh, 0)),
        Just(keyarg(Tag::Pkh, 0)),
        Just(Node {
            tag: Tag::Tr,
            body: Body::Tr {
                internal_key: InternalKey::Slot(0),
                tree: None
            }
        }),
    ]
    .prop_map(|t| descriptor_from_tree(t, false));

    let sh_wpkh =
        Just(wrap(Tag::Sh, keyarg(Tag::Wpkh, 0))).prop_map(|t| descriptor_from_tree(t, false));

    let multisig = (
        n_strategy(),
        1u8..=32u8,
        prop::sample::select(vec![Tag::Multi, Tag::SortedMulti]),
    )
        .prop_filter("k<=n", |(n, k, _)| k <= n)
        .prop_map(|(n, k, mtag)| {
            let inner = multikeys(mtag, k, (0..n).collect());
            descriptor_from_tree(wrap(Tag::Wsh, inner), true)
        });

    let sh_wsh = (n_strategy(), 1u8..=32u8)
        .prop_filter("k<=n", |(n, k)| k <= n)
        .prop_map(|(n, k)| {
            let inner = wrap(Tag::Wsh, multikeys(Tag::SortedMulti, k, (0..n).collect()));
            descriptor_from_tree(wrap(Tag::Sh, inner), true)
        });

    let sh_sortedmulti = (n_strategy(), 1u8..=32u8)
        .prop_filter("k<=n", |(n, k)| k <= n)
        .prop_map(|(n, k)| {
            let inner = multikeys(Tag::SortedMulti, k, (0..n).collect());
            descriptor_from_tree(wrap(Tag::Sh, inner), true)
        });

    let tr_multi_a = (2u8..=16u8, 1u8..=16u8)
        .prop_filter("k<=n-1", |(n, k)| *k < *n)
        .prop_map(|(n, k)| {
            let leaf = multikeys(Tag::MultiA, k, (1..n).collect());
            let tree = Node {
                tag: Tag::Tr,
                body: Body::Tr {
                    internal_key: InternalKey::Slot(0),
                    tree: Some(Box::new(leaf)),
                },
            };
            descriptor_from_tree(tree, true)
        });

    let tr_taptree = (2u8..=8u8).prop_flat_map(|max| {
        taptree_strategy(max).prop_map(move |tt| {
            let tree = Node {
                tag: Tag::Tr,
                body: Body::Tr {
                    internal_key: InternalKey::Slot(0),
                    tree: Some(Box::new(tt)),
                },
            };
            descriptor_from_tree(tree, true)
        })
    });

    prop_oneof![
        single_sig,
        sh_wpkh,
        multisig,
        sh_wsh,
        sh_sortedmulti,
        tr_multi_a,
        tr_taptree
    ]
    .boxed()
}

/// canonicalize a descriptor (the fixpoint helper).
pub fn canon(d: &Descriptor) -> Descriptor {
    let mut c = d.clone();
    canonicalize_placeholder_indices(&mut c).expect("strategy descriptors are canonicalizable");
    c
}

// ─────────────────────────────────────────────────────────────────────────
// Cycle B (stress program) — shared key material + W/T strategies.
// Spec: design/BRAINSTORM_proptest_fragment_domain_expansion.md (R4 GREEN).
// ─────────────────────────────────────────────────────────────────────────

/// The BIP-84/86 published-test-vector mnemonic (address_derivation.rs pattern).
const ABANDON_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// 32 account-level xpubs (`m/86'/0'/{i}'`) derived ONCE from the abandon
/// mnemonic via `OnceLock`, each packed as the v0.13 `Pubkeys` TLV payload
/// `(chain_code || compressed_pubkey)`.
pub fn test_xpubs() -> &'static [[u8; 65]; 32] {
    static XPUBS: std::sync::OnceLock<[[u8; 65]; 32]> = std::sync::OnceLock::new();
    XPUBS.get_or_init(|| {
        use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
        use bitcoin::secp256k1::Secp256k1;
        use std::str::FromStr;
        let mn = bip39::Mnemonic::parse(ABANDON_MNEMONIC).expect("known-good mnemonic");
        let seed = mn.to_seed("");
        let secp = Secp256k1::new();
        let master =
            Xpriv::new_master(bitcoin::Network::Bitcoin, &seed).expect("seed gives master");
        let mut out = [[0u8; 65]; 32];
        for (i, slot) in out.iter_mut().enumerate() {
            let path = DerivationPath::from_str(&format!("m/86'/0'/{i}'")).expect("valid path");
            let xpriv = master.derive_priv(&secp, &path).expect("derive priv");
            let xpub = Xpub::from_priv(&secp, &xpriv);
            slot[..32].copy_from_slice(xpub.chain_code.as_ref());
            slot[32..].copy_from_slice(&xpub.public_key.serialize());
        }
        out
    })
}

/// Build a wallet-policy-mode Descriptor: renumber via `renumbered` (the
/// descriptor_from_tree logic), divergent 3-deep origin paths, and attach
/// `tlv.pubkeys` for every `@i` from the OnceLock pool. Usable for any
/// n in 1..=32 (P7 oversize-multi cells go above the T-tier 16 cap).
pub fn descriptor_with_pubkeys(tree: Node) -> Descriptor {
    let (tree, n) = renumbered(tree);
    assert!(
        (1..=32).contains(&n),
        "descriptor must reference 1..=32 keys, got {n}"
    );
    let mut tlv = TlvSection::new_empty();
    tlv.pubkeys = Some((0..n).map(|i| (i, test_xpubs()[i as usize])).collect());
    Descriptor {
        n,
        path_decl: divergent_path(n, 3),
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv,
    }
}

/// Pre-order sequential key-index assignment: every key slot in the tree
/// (KeyArg, MultiKeys element, non-NUMS Tr internal key) gets the next
/// fresh index. Guarantees (a) all-distinct `@i` per descriptor — the
/// T-tier tap rule (b) [each `@i` at most once per tr descriptor incl.
/// internal key] — and (b) first occurrences ascending in pre-order.
pub fn assign_sequential_indices(node: &mut Node, next: &mut u8) {
    match &mut node.body {
        Body::KeyArg { index } => {
            *index = *next;
            *next += 1;
        }
        Body::MultiKeys { indices, .. } => {
            for i in indices.iter_mut() {
                *i = *next;
                *next += 1;
            }
        }
        Body::Tr { internal_key, tree } => {
            if let InternalKey::Slot(i) = internal_key {
                *i = *next;
                *next += 1;
            }
            if let Some(t) = tree {
                assign_sequential_indices(t, next);
            }
        }
        Body::Children(cs) => {
            for c in cs.iter_mut() {
                assign_sequential_indices(c, next);
            }
        }
        Body::Variable { children, .. } => {
            for c in children.iter_mut() {
                assign_sequential_indices(c, next);
            }
        }
        _ => {}
    }
}

// ─── Tier 1 — wire-domain strategy (W): full domains, arbitrary nesting ──

const W_WRAPPERS: [Tag; 7] = [
    Tag::Check,
    Tag::Verify,
    Tag::Swap,
    Tag::Alt,
    Tag::DupIf,
    Tag::NonZero,
    Tag::ZeroNotEqual,
];
const W_ARITY2: [Tag; 6] = [Tag::AndV, Tag::AndB, Tag::OrB, Tag::OrC, Tag::OrD, Tag::OrI];

/// Full-u32 timelock domain, biased to the spec's boundary constants.
pub const W_BOUNDARY_TIMELOCKS: [u32; 11] = [
    0,
    1,
    0xFFFF,
    0x0001_0000,
    0x0040_FFFF,
    0x0041_0000,
    499_999_999,
    500_000_000,
    0x7FFF_FFFF,
    0x8000_0000,
    u32::MAX,
];

fn w_timelock_node() -> BoxedStrategy<Node> {
    let v = prop_oneof![
        3 => prop::sample::select(W_BOUNDARY_TIMELOCKS.to_vec()),
        1 => any::<u32>(),
    ];
    (prop::sample::select(vec![Tag::After, Tag::Older]), v)
        .prop_map(|(t, v)| timelock(t, v))
        .boxed()
}

fn w_keyless_leaf() -> BoxedStrategy<Node> {
    prop_oneof![
        4 => w_timelock_node(),
        1 => any::<[u8; 32]>().prop_map(|h| hash32(Tag::Sha256, h)),
        1 => any::<[u8; 32]>().prop_map(|h| hash32(Tag::Hash256, h)),
        1 => any::<[u8; 20]>().prop_map(|h| hash20(Tag::Ripemd160, h)),
        1 => any::<[u8; 20]>().prop_map(|h| hash20(Tag::Hash160, h)),
        1 => any::<[u8; 20]>().prop_map(|h| hash20(Tag::RawPkH, h)),
        1 => Just(Node { tag: Tag::True, body: Body::Empty }),
        1 => Just(Node { tag: Tag::False, body: Body::Empty }),
    ]
    .boxed()
}

/// Multi-family node over the full wire domain: k ≤ len, duplicate indices
/// permitted (the wire layer doesn't forbid them).
fn w_multikeys(tags: Vec<Tag>, max_idx: u8, max_len: usize) -> BoxedStrategy<Node> {
    (
        prop::sample::select(tags),
        prop::collection::vec(0..=max_idx, 1..=max_len),
    )
        .prop_flat_map(|(tag, idxs)| {
            let len = idxs.len() as u8;
            (1..=len).prop_map(move |k| multikeys(tag, k, idxs.clone()))
        })
        .boxed()
}

fn w_keyed_leaf(max_idx: u8, max_len: usize) -> BoxedStrategy<Node> {
    prop_oneof![
        2 => (0..=max_idx).prop_map(|i| keyarg(Tag::PkK, i)),
        2 => (0..=max_idx).prop_map(|i| keyarg(Tag::PkH, i)),
        3 => w_multikeys(
            vec![Tag::Multi, Tag::SortedMulti, Tag::MultiA, Tag::SortedMultiA],
            max_idx,
            max_len
        ),
    ]
    .boxed()
}

/// One combinator layer over `child`: unary wrappers × arity-2 × AndOr(3)
/// × Thresh(k ≤ children ≤ 4). Worst-case node count = 1 + 4·|child|.
fn w_level(child: BoxedStrategy<Node>) -> BoxedStrategy<Node> {
    prop_oneof![
        2 => (prop::sample::select(W_WRAPPERS.to_vec()), child.clone())
            .prop_map(|(t, c)| wrap(t, c)),
        3 => (prop::sample::select(W_ARITY2.to_vec()), child.clone(), child.clone())
            .prop_map(|(t, a, b)| node2(t, a, b)),
        1 => (child.clone(), child.clone(), child.clone())
            .prop_map(|(a, b, c)| node3(Tag::AndOr, a, b, c)),
        2 => prop::collection::vec(child, 2..=4).prop_flat_map(|cs| {
            let len = cs.len() as u8;
            (1..=len).prop_map(move |k| thresh_node(k, cs.clone()))
        }),
    ]
    .boxed()
}

/// wsh/sh/sh(wsh) inner subtree with the ≥1-key guarantee carried by
/// construction (a designated key-bearing leaf in every arm). Layered
/// depth ≤ 4, fan-out ≤ 4, ≤ 25 nodes incl. root wrappers (w2 worst case
/// 21; pairing 23; sh(wsh(·)) adds 2). Size enforcement is the encoded
/// ≤ 18,000-bit assert in the final map, not this node arithmetic.
fn w_inner(max_idx: u8, max_len: usize) -> BoxedStrategy<Node> {
    let leaf = prop_oneof![w_keyed_leaf(max_idx, max_len), w_keyless_leaf()].boxed();
    let w1 = w_level(leaf.clone());
    let w01 = prop_oneof![leaf.clone(), w1.clone()].boxed();
    let w2 = w_level(w01);
    let sub = prop_oneof![1 => leaf, 2 => w1.clone(), 2 => w2].boxed();
    let key = w_keyed_leaf(max_idx, max_len);
    prop_oneof![
        1 => key.clone(),
        1 => (prop::sample::select(W_WRAPPERS.to_vec()), key.clone())
            .prop_map(|(t, k)| wrap(t, k)),
        4 => (
            prop::sample::select(W_ARITY2.to_vec()),
            sub,
            key.clone(),
            any::<bool>()
        )
            .prop_map(|(t, s, k, flip)| if flip { node2(t, k, s) } else { node2(t, s, k) }),
        1 => (w1.clone(), key.clone(), w1.clone())
            .prop_map(|(a, k, b)| node3(Tag::AndOr, a, k, b)),
        1 => (key, prop::collection::vec(w1, 1..=3)).prop_flat_map(|(k0, rest)| {
            let mut cs = vec![k0];
            cs.extend(rest);
            let len = cs.len() as u8;
            (1..=len).prop_map(move |k| thresh_node(k, cs.clone()))
        }),
    ]
    .boxed()
}

/// tr() root for the W tier. Taptree LEAF top-tags avoid the §6.3.1
/// forbidden set (bare Multi/SortedMulti never appear as a leaf root;
/// combinator leaves may still carry them INSIDE — decode permits that).
/// Key guarantee: non-NUMS internal key, or a designated key-bearing leaf.
fn w_tr(max_idx: u8, max_len: usize) -> BoxedStrategy<Node> {
    let leaf_full = prop_oneof![w_keyed_leaf(max_idx, max_len), w_keyless_leaf()].boxed();
    let tap_keyed = prop_oneof![
        1 => (0..=max_idx).prop_map(|i| keyarg(Tag::PkK, i)),
        1 => (0..=max_idx).prop_map(|i| keyarg(Tag::PkH, i)),
        2 => w_multikeys(vec![Tag::MultiA, Tag::SortedMultiA], max_idx, max_len),
    ]
    .boxed();
    let tap_atom = prop_oneof![2 => tap_keyed.clone(), 1 => w_keyless_leaf()].boxed();
    let w1 = w_level(leaf_full.clone());
    let w2 = w_level(prop_oneof![leaf_full, w1.clone()].boxed());
    let tap_leaf1 = prop_oneof![2 => tap_atom.clone(), 2 => w1].boxed();
    let single_leaf = prop_oneof![2 => tap_atom, 2 => tap_leaf1.clone(), 1 => w2].boxed();
    prop_oneof![
        3 => (0..=max_idx, single_leaf).prop_map(|(i, l)| tr_node(false, i, Some(l))),
        1 => (0..=max_idx).prop_map(|i| tr_node(false, i, None)),
        2 => tap_keyed.clone().prop_map(|l| tr_node(true, 0, Some(l))),
        2 => (tap_keyed, tap_leaf1.clone(), any::<bool>()).prop_map(|(k, o, flip)| {
            let tt = if flip { taptree2(k, o) } else { taptree2(o, k) };
            tr_node(true, 0, Some(tt))
        }),
        1 => (0..=max_idx, tap_leaf1.clone(), tap_leaf1.clone(), tap_leaf1)
            .prop_map(|(i, a, b, c)| tr_node(false, i, Some(taptree2(taptree2(a, b), c)))),
    ]
    .boxed()
}

/// Which TLVs the W tier attaches this case. Pubkeys/fingerprints cap the
/// key universe at n ≤ 8 (payload budget belt-and-braces, spec [I1′]).
#[derive(Clone, Copy, Debug)]
pub struct WTlvMode {
    pubkeys: bool,
    fingerprints: bool,
    origin_overrides: bool,
}

fn w_tlv_mode() -> impl Strategy<Value = WTlvMode> {
    (
        prop::bool::weighted(0.35),
        prop::bool::weighted(0.35),
        prop::bool::weighted(0.35),
    )
        .prop_map(|(pubkeys, fingerprints, origin_overrides)| WTlvMode {
            pubkeys,
            fingerprints,
            origin_overrides,
        })
}

fn w_origin_override_path() -> impl Strategy<Value = OriginPath> {
    prop::collection::vec((any::<bool>(), 0u32..=10_000), 1..=3).prop_map(|cs| OriginPath {
        components: cs
            .into_iter()
            .map(|(hardened, value)| PathComponent { hardened, value })
            .collect(),
    })
}

/// Sparse per-`@N` TLV entry vector (the strategy-output shape).
type SparseTlv<T> = Option<Vec<(u8, T)>>;

/// TLV entries for a renumbered tree with `n` keys. Pubkeys attach the
/// full 0..n map (valid curve points from the T-tier pool); fingerprints
/// and origin overrides attach non-empty ascending subsets.
fn w_tlv_entries(mode: WTlvMode, n: u8) -> BoxedStrategy<TlvSection> {
    let idxs: Vec<u8> = (0..n).collect();
    let pubkeys_s: BoxedStrategy<SparseTlv<[u8; 65]>> = if mode.pubkeys {
        Just(Some(
            (0..n)
                .map(|i| (i, test_xpubs()[i as usize]))
                .collect::<Vec<_>>(),
        ))
        .boxed()
    } else {
        Just(None).boxed()
    };
    let fps_s: BoxedStrategy<SparseTlv<[u8; 4]>> = if mode.fingerprints {
        prop::sample::subsequence(idxs.clone(), 1..=n as usize)
            .prop_flat_map(|sel| {
                prop::collection::vec(any::<[u8; 4]>(), sel.len())
                    .prop_map(move |bytes| Some(sel.iter().copied().zip(bytes).collect::<Vec<_>>()))
            })
            .boxed()
    } else {
        Just(None).boxed()
    };
    let origin_s: BoxedStrategy<SparseTlv<OriginPath>> = if mode.origin_overrides {
        prop::sample::subsequence(idxs, 1..=n as usize)
            .prop_flat_map(|sel| {
                prop::collection::vec(w_origin_override_path(), sel.len())
                    .prop_map(move |paths| Some(sel.iter().copied().zip(paths).collect::<Vec<_>>()))
            })
            .boxed()
    } else {
        Just(None).boxed()
    };
    (pubkeys_s, fps_s, origin_s)
        .prop_map(|(pubkeys, fingerprints, origin_path_overrides)| {
            let mut t = TlvSection::new_empty();
            t.pubkeys = pubkeys;
            t.fingerprints = fingerprints;
            t.origin_path_overrides = origin_path_overrides;
            t
        })
        .boxed()
}

/// Tier 1 (W): decode-valid but not necessarily type-valid descriptors over
/// the FULL wire domains, with randomized Shared/Divergent path-decls and
/// occasional origin/fingerprint/pubkey TLVs. The final map enforces the
/// payload budget on the ACTUAL ENCODING (≤ 18,000 bits, margin under the
/// 20,480-bit chunk cliff) — a loud panic here is the intended drift signal.
pub fn wire_descriptor_strategy() -> BoxedStrategy<Descriptor> {
    w_tlv_mode()
        .prop_flat_map(|mode| {
            let (max_idx, max_len): (u8, usize) = if mode.pubkeys || mode.fingerprints {
                (7, 8)
            } else {
                (31, 16)
            };
            let inner = w_inner(max_idx, max_len);
            let tree = prop_oneof![
                3 => inner.clone().prop_map(|i| wrap(Tag::Wsh, i)),
                2 => inner.clone().prop_map(|i| wrap(Tag::Sh, i)),
                2 => inner.prop_map(|i| wrap(Tag::Sh, wrap(Tag::Wsh, i))),
                3 => w_tr(max_idx, max_len),
            ];
            (tree, any::<bool>()).prop_flat_map(move |(tree, divergent)| {
                let (tree, n) = renumbered(tree);
                w_tlv_entries(mode, n).prop_map(move |tlv| Descriptor {
                    n,
                    path_decl: if divergent {
                        divergent_path(n, 3)
                    } else {
                        PathDecl {
                            n,
                            paths: PathDeclPaths::Shared(OriginPath {
                                components: vec![PathComponent {
                                    hardened: true,
                                    value: 84,
                                }],
                            }),
                        }
                    },
                    use_site_path: UseSitePath::standard_multipath(),
                    tree: tree.clone(),
                    tlv,
                })
            })
        })
        .prop_map(|d| {
            let c = canon(&d);
            let (_bytes, total_bits) =
                md_codec::encode::encode_payload(&c).expect("W-tier descriptor must encode");
            assert!(
                total_bits <= 18_000,
                "W-tier payload budget exceeded: {total_bits} bits > 18,000"
            );
            d
        })
        .boxed()
}

// ─── Tier 2 — typed strategy (T): type-correct-by-construction + xpub TLVs ──

/// Consensus-valid `after` values for the chosen absolute-lock class.
/// Boundary-biased; valid domain is 1..=0x7FFF_FFFF.
fn t_after_value(abs_time: bool) -> BoxedStrategy<u32> {
    if abs_time {
        prop_oneof![
            3 => prop::sample::select(vec![500_000_000u32, 0x7FFF_FFFF]),
            2 => 500_000_000u32..=0x7FFF_FFFF,
        ]
        .boxed()
    } else {
        prop_oneof![
            3 => prop::sample::select(vec![1u32, 144, 0xFFFF, 0x0001_0000, 499_999_999]),
            2 => 1u32..=499_999_999,
        ]
        .boxed()
    }
}

/// Consensus-valid `older` values for the chosen relative-lock class
/// (non-zero, bit 31 clear; class = bit 22). INCLUDES the out-of-BIP-68-mask
/// values 0x10000 (height class) / 0x00410000 (time class) — miniscript's
/// known leniency, pinned Ok in P6 (toolkit v0.53.9 context).
fn t_older_value(rel_time: bool) -> BoxedStrategy<u32> {
    if rel_time {
        prop_oneof![
            3 => prop::sample::select(vec![0x0040_0001u32, 0x0040_FFFF, 0x0041_0000]),
            2 => (1u32..=0xFFFF).prop_map(|v| 0x0040_0000 | v),
        ]
        .boxed()
    } else {
        prop_oneof![
            3 => prop::sample::select(vec![1u32, 144, 0xFFFF, 0x0001_0000]),
            2 => 1u32..=0xFFFF,
        ]
        .boxed()
    }
}

fn t_lock_node(rel_time: bool, abs_time: bool) -> BoxedStrategy<Node> {
    prop_oneof![
        t_older_value(rel_time).prop_map(|v| timelock(Tag::Older, v)),
        t_after_value(abs_time).prop_map(|v| timelock(Tag::After, v)),
    ]
    .boxed()
}

fn t_hash_node() -> BoxedStrategy<Node> {
    prop_oneof![
        any::<[u8; 32]>().prop_map(|h| hash32(Tag::Sha256, h)),
        any::<[u8; 32]>().prop_map(|h| hash32(Tag::Hash256, h)),
        any::<[u8; 20]>().prop_map(|h| hash20(Tag::Ripemd160, h)),
        any::<[u8; 20]>().prop_map(|h| hash20(Tag::Hash160, h)),
    ]
    .boxed()
}

/// Bare key leaf (`pk(@i)` / `pk_h(@i)`). Index 0 is a placeholder —
/// `assign_sequential_indices` allocates the real fresh index.
fn t_ka() -> BoxedStrategy<Node> {
    prop_oneof![Just(keyarg(Tag::PkK, 0)), Just(keyarg(Tag::PkH, 0)),].boxed()
}

/// Multi-family leaf with `min_n..=max_n` PLACEHOLDER key slots, k ≤ n.
fn t_multi_node(tag: Tag, min_n: u8, max_n: u8) -> BoxedStrategy<Node> {
    (min_n..=max_n)
        .prop_flat_map(move |n| (1..=n).prop_map(move |k| multikeys(tag, k, vec![0; n as usize])))
        .boxed()
}

/// Segwitv0 (`wsh`) typed B-tree: spec grammar with the segwit Bdu pool
/// (keys, multi, hashlocks, recursive or_d) and W = s:pk | a:<Bdu leaf>.
/// Every arm carries ≥1 key by construction. Worst-case key slots: 15.
fn t_segwit_tree(rel_time: bool, abs_time: bool) -> BoxedStrategy<Node> {
    let ka = t_ka();
    let hash = t_hash_node();
    let lock = t_lock_node(rel_time, abs_time);
    let multi3 = t_multi_node(Tag::Multi, 1, 3);
    let leaf_key = prop_oneof![2 => ka.clone(), 1 => multi3.clone()].boxed();
    let leaf_any = prop_oneof![
        2 => leaf_key.clone(),
        1 => hash.clone(),
        1 => lock.clone(),
    ]
    .boxed();
    let bdu0 = prop_oneof![2 => ka.clone(), 1 => multi3.clone(), 1 => hash].boxed();
    let bdu_key = prop_oneof![2 => ka.clone(), 1 => multi3].boxed();
    let bdu1 = prop_oneof![
        2 => bdu0.clone(),
        1 => (bdu0.clone(), bdu0.clone()).prop_map(|(a, b)| node2(Tag::OrD, a, b)),
    ]
    .boxed();
    let w0 = prop_oneof![
        1 => Just(wrap(Tag::Swap, keyarg(Tag::PkK, 0))),
        1 => bdu0.clone().prop_map(|b| wrap(Tag::Alt, b)),
    ]
    .boxed();
    let vfirst = prop_oneof![1 => lock, 1 => leaf_any.clone()].boxed();
    let thresh1 =
        (bdu_key.clone(), prop::collection::vec(w0.clone(), 1..=2)).prop_flat_map(|(first, ws)| {
            let mut cs = vec![first];
            cs.extend(ws);
            let len = cs.len() as u8;
            (1..=len).prop_map(move |k| thresh_node(k, cs.clone()))
        });
    let b1 = prop_oneof![
        3 => leaf_key.clone(),
        2 => (vfirst.clone(), leaf_key.clone())
            .prop_map(|(x, y)| node2(Tag::AndV, wrap(Tag::Verify, x), y)),
        2 => (leaf_key.clone(), w0.clone()).prop_map(|(b, w)| node2(Tag::AndB, b, w)),
        2 => (leaf_key.clone(), leaf_any.clone()).prop_map(|(a, b)| node2(Tag::OrI, a, b)),
        2 => (bdu1.clone(), leaf_key.clone()).prop_map(|(a, b)| node2(Tag::OrD, a, b)),
        2 => (bdu0.clone(), leaf_any.clone(), leaf_key.clone())
            .prop_map(|(a, b, c)| node3(Tag::AndOr, a, b, c)),
        2 => thresh1,
    ]
    .boxed();
    let thresh2 =
        (bdu_key, prop::collection::vec(w0.clone(), 1..=2)).prop_flat_map(|(first, ws)| {
            let mut cs = vec![first];
            cs.extend(ws);
            let len = cs.len() as u8;
            (1..=len).prop_map(move |k| thresh_node(k, cs.clone()))
        });
    let b2 = prop_oneof![
        4 => b1.clone(),
        1 => (b1.clone(), leaf_any.clone())
            .prop_map(|(b, x)| node2(Tag::AndV, wrap(Tag::Verify, b), x)),
        1 => (b1.clone(), w0).prop_map(|(b, w)| node2(Tag::AndB, b, w)),
        1 => (b1.clone(), leaf_any).prop_map(|(a, b)| node2(Tag::OrI, a, b)),
        1 => (bdu1, b1.clone()).prop_map(|(a, b)| node2(Tag::OrD, a, b)),
        1 => (bdu0, b1, leaf_key).prop_map(|(a, b, c)| node3(Tag::AndOr, a, b, c)),
        1 => thresh2,
    ]
    .boxed();
    // GAP-2: the seven fragment arms T previously omitted (DupIf/NonZero/
    // ZeroNotEqual/OrB/OrC/True/False), as FIXED proven shapes. R0-I2
    // CONSTRAINT (load-bearing): children are ONLY keys + locks in these exact
    // positions — do NOT route the leaf_any/bdu*/hash pools into or_b/or_c/j:/n:
    // child slots; those compose type-invalid trees (e.g. or_b(older,s:pk),
    // j:older) that P6 step-1 panics on (no prop_filter). All seven are B-type.
    let seven = prop_oneof![
        // or_b(pk, s:pk)
        Just(node2(
            Tag::OrB,
            keyarg(Tag::PkK, 0),
            wrap(Tag::Swap, keyarg(Tag::PkK, 0)),
        )),
        // t:or_c(pk, v:pk)  ==  and_v(or_c(pk, v:pk), True)
        Just(node2(
            Tag::AndV,
            node2(
                Tag::OrC,
                keyarg(Tag::PkK, 0),
                wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            ),
            Node {
                tag: Tag::True,
                body: Body::Empty,
            },
        )),
        // or_i(pk, d:v:LOCK)
        t_lock_node(rel_time, abs_time).prop_map(|l| node2(
            Tag::OrI,
            keyarg(Tag::PkK, 0),
            wrap(Tag::DupIf, wrap(Tag::Verify, l)),
        )),
        // j:pk
        Just(wrap(Tag::NonZero, keyarg(Tag::PkK, 0))),
        // or_i(pk, n:and_v(v:pk, LOCK))
        t_lock_node(rel_time, abs_time).prop_map(|l| node2(
            Tag::OrI,
            keyarg(Tag::PkK, 0),
            wrap(
                Tag::ZeroNotEqual,
                node2(Tag::AndV, wrap(Tag::Verify, keyarg(Tag::PkK, 0)), l),
            ),
        )),
        // u:pk  ==  or_i(pk, False)
        Just(node2(
            Tag::OrI,
            keyarg(Tag::PkK, 0),
            Node {
                tag: Tag::False,
                body: Body::Empty,
            },
        )),
        // tv:pk ==  and_v(v:pk, True)
        Just(node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            Node {
                tag: Tag::True,
                body: Body::Empty,
            },
        )),
    ]
    .boxed();
    // Standalone wide multi exercises Segwitv0 multi up to the T-tier
    // n ≤ 16 cap (the miniscript limit is 20). The 16 is a deliberate T-tier
    // key-BUDGET choice, NOT an infra limit — test_xpubs() has 32 keys and
    // descriptor_with_pubkeys accepts 1..=32. The valid 17..=20 render/address
    // window is pinned deterministically by the self_test_wsh_multi_17_of_*
    // goldens; n ≥ 21 is P7 oversize-refusal territory.
    let wide_multi = t_multi_node(Tag::Multi, 2, 16);
    prop_oneof![5 => b2, 2 => seven, 1 => wide_multi].boxed()
}

/// Legacy (`sh`) typed tree: depth ≤ 2, ≤ 6 key slots, multi ≤ 6 keys
/// standalone / ≤ 2 in compound positions (pk_cost 520 headroom).
fn t_legacy_tree(rel_time: bool, abs_time: bool) -> BoxedStrategy<Node> {
    let ka = t_ka();
    let hash = t_hash_node();
    let lock = t_lock_node(rel_time, abs_time);
    let multi2 = t_multi_node(Tag::Multi, 1, 2);
    let multi6 = t_multi_node(Tag::Multi, 1, 6);
    let bdu0 = prop_oneof![2 => ka.clone(), 1 => multi2.clone(), 1 => hash.clone()].boxed();
    let bdu_key = prop_oneof![2 => ka.clone(), 1 => multi2].boxed();
    let w0 = prop_oneof![
        1 => Just(wrap(Tag::Swap, keyarg(Tag::PkK, 0))),
        1 => bdu0.clone().prop_map(|b| wrap(Tag::Alt, b)),
    ]
    .boxed();
    let vfirst = prop_oneof![1 => lock, 1 => hash, 1 => ka.clone()].boxed();
    let thresh =
        (bdu_key, prop::collection::vec(w0.clone(), 1..=2)).prop_flat_map(|(first, ws)| {
            let mut cs = vec![first];
            cs.extend(ws);
            let len = cs.len() as u8;
            (1..=len).prop_map(move |k| thresh_node(k, cs.clone()))
        });
    prop_oneof![
        2 => ka.clone(),
        2 => multi6,
        2 => (vfirst, ka.clone()).prop_map(|(x, y)| node2(Tag::AndV, wrap(Tag::Verify, x), y)),
        2 => (ka.clone(), w0).prop_map(|(b, w)| node2(Tag::AndB, b, w)),
        1 => (ka.clone(), ka.clone()).prop_map(|(a, b)| node2(Tag::OrI, a, b)),
        2 => (bdu0.clone(), ka.clone()).prop_map(|(a, b)| node2(Tag::OrD, a, b)),
        1 => (bdu0, ka.clone(), ka).prop_map(|(a, b, c)| node3(Tag::AndOr, a, b, c)),
        2 => thresh,
    ]
    .boxed()
}

/// Tap leaf grammar — SANE-BY-CONSTRUCTION (spec rule (a)): every leaf is
/// signature-bearing AND non-malleable. The tap Bdu pool is KEYS ONLY
/// (pk/pk_h/multi_a + recursive or_d); hashlocks/timelocks appear ONLY
/// under `v:` inside `and_v(v:<lock|hash>, <sig-bearing B>)`. Every
/// production here has an empirical sanity proof in the round-1/2/3
/// evidence logs or the golden self-test cells.
fn t_tap_leaf(rel_time: bool, abs_time: bool) -> BoxedStrategy<Node> {
    let ka = t_ka();
    let hash = t_hash_node();
    let lock = t_lock_node(rel_time, abs_time);
    let multi_a2 = t_multi_node(Tag::MultiA, 2, 2);
    let multi_a3 = t_multi_node(Tag::MultiA, 1, 3);
    let bdu0 = prop_oneof![2 => ka.clone(), 1 => multi_a2].boxed();
    let bdu1 = prop_oneof![
        2 => bdu0.clone(),
        1 => (bdu0.clone(), bdu0.clone()).prop_map(|(a, b)| node2(Tag::OrD, a, b)),
    ]
    .boxed();
    let w0 = prop_oneof![
        1 => Just(wrap(Tag::Swap, keyarg(Tag::PkK, 0))),
        1 => bdu0.clone().prop_map(|b| wrap(Tag::Alt, b)),
    ]
    .boxed();
    let vfirst = prop_oneof![1 => lock, 1 => hash, 1 => ka.clone()].boxed();
    let sb_simple = prop_oneof![
        2 => ka.clone(),
        1 => (vfirst.clone(), ka.clone())
            .prop_map(|(x, y)| node2(Tag::AndV, wrap(Tag::Verify, x), y)),
    ]
    .boxed();
    let thresh =
        (bdu0.clone(), prop::collection::vec(w0.clone(), 1..=2)).prop_flat_map(|(first, ws)| {
            let mut cs = vec![first];
            cs.extend(ws);
            let len = cs.len() as u8;
            (1..=len).prop_map(move |k| thresh_node(k, cs.clone()))
        });
    prop_oneof![
        2 => ka,
        1 => multi_a3,
        2 => (vfirst, sb_simple.clone())
            .prop_map(|(x, y)| node2(Tag::AndV, wrap(Tag::Verify, x), y)),
        2 => (sb_simple.clone(), w0).prop_map(|(b, w)| node2(Tag::AndB, b, w)),
        2 => (sb_simple.clone(), sb_simple.clone()).prop_map(|(a, b)| node2(Tag::OrI, a, b)),
        2 => (bdu1, sb_simple.clone()).prop_map(|(a, b)| node2(Tag::OrD, a, b)),
        2 => (bdu0, sb_simple.clone(), sb_simple).prop_map(|(a, b, c)| node3(Tag::AndOr, a, b, c)),
        2 => thresh,
    ]
    .boxed()
}

/// tr() roots for the T tier: NUMS-or-key internal, 1–2 sane leaves,
/// multi_a ≤ 16 keys (n-budget). Key slots ≤ 16 total by construction.
///
/// MINIMAL GENERATOR CONSTRAINT (found by P6 during bring-up): taptree
/// DEPTH ≤ 2 since the ff4732e pin (2026-08-20).
///
/// It was capped at depth ≤ 1 — at most `{a,b}` — because miniscript 13.0.0
/// had a Display/parse asymmetry on DEPTH-2 taptrees: upstream's own
/// `TapTree::combine(combine(a,b),c)` Displayed as the malformed `{{a,b,c}}`
/// instead of `{{a,b},c}`, which miniscript's OWN `Descriptor::from_str`
/// then rejected. Never an md-codec bug — the wire round-trip and address
/// derivation do not go through the string form.
///
/// PR #953 fixes it and is in the pinned rev, so the depth-2 arm is restored
/// here and `upstream_taptree_depth2_display_asymmetry` is inverted to assert
/// the fix. THAT IS THE POINT OF THE BUMP: the generator can now reach the
/// nested shapes the whole tr()/wsh() feature is about, which the cap had
/// been quietly excluding from every property test.
fn t_tr_tree(rel_time: bool, abs_time: bool) -> BoxedStrategy<Node> {
    let leaf = t_tap_leaf(rel_time, abs_time);
    prop_oneof![
        3 => leaf.clone().prop_map(|l| tr_node(false, 0, Some(l))),
        2 => leaf.clone().prop_map(|l| tr_node(true, 0, Some(l))),
        2 => (any::<bool>(), leaf.clone(), leaf.clone()).prop_map(|(nums, a, b)| {
            tr_node(nums, 0, Some(taptree2(a, b)))
        }),
        // DEPTH 2, UNBALANCED ON PURPOSE: `{{a,b},c}` has leaf depths (2,2,1),
        // a decreasing sequence, which is exactly the shape the pre-#953
        // formatter got wrong. A balanced `{{a,b},{c,d}}` round-trips even with
        // the bug and would prove nothing.
        2 => (any::<bool>(), leaf.clone(), leaf.clone(), leaf).prop_map(|(nums, a, b, c)| {
            tr_node(nums, 0, Some(taptree2(taptree2(a, b), c)))
        }),
        1 => t_multi_node(Tag::MultiA, 1, 16).prop_map(|m| tr_node(true, 0, Some(m))),
        1 => t_multi_node(Tag::MultiA, 1, 15).prop_map(|m| tr_node(false, 0, Some(m))),
    ]
    .boxed()
}

/// Tier 2 (T): type-correct-by-construction miniscript descriptors with
/// real xpub TLVs attached for every `@i`. NO prop_filter anywhere —
/// every emitted descriptor MUST pass P6's full chain. Timelock classes
/// (rule (c)) are chosen once per descriptor: relative height-XOR-time and
/// absolute height-XOR-time (rel + abs together is fine) — DELIBERATELY
/// stricter than miniscript's per-spend-path rule (see spec [M2′]).
pub fn typed_descriptor_strategy() -> BoxedStrategy<Descriptor> {
    (any::<bool>(), any::<bool>())
        .prop_flat_map(|(rel_time, abs_time)| {
            prop_oneof![
                3 => t_segwit_tree(rel_time, abs_time).prop_map(|t| wrap(Tag::Wsh, t)),
                2 => t_legacy_tree(rel_time, abs_time).prop_map(|t| wrap(Tag::Sh, t)),
                3 => t_tr_tree(rel_time, abs_time),
            ]
        })
        .prop_map(|mut tree| {
            let mut next = 0u8;
            assign_sequential_indices(&mut tree, &mut next);
            assert!(
                (1..=16).contains(&next),
                "T-tier key budget violated: {next} key slots (cap 16)"
            );
            descriptor_with_pubkeys(tree)
        })
        .boxed()
}

// ─── Anti-vacuity walkers (generator_covers_all_fragments) ──────────────

/// Collect every Tag in the tree plus all (After/Older) timelock values.
pub fn collect_tags_and_locks(
    node: &Node,
    tags: &mut std::collections::HashSet<Tag>,
    locks: &mut std::collections::HashSet<u32>,
) {
    tags.insert(node.tag);
    match &node.body {
        Body::Timelock(v) => {
            locks.insert(*v);
        }
        Body::Children(cs) => {
            for c in cs {
                collect_tags_and_locks(c, tags, locks);
            }
        }
        Body::Variable { children, .. } => {
            for c in children {
                collect_tags_and_locks(c, tags, locks);
            }
        }
        Body::Tr { tree: Some(t), .. } => {
            collect_tags_and_locks(t, tags, locks);
        }
        _ => {}
    }
}

/// flip one codex32 symbol at data-part position `pos` (post-"md1") of a chunk.
pub fn corrupt_chunk_at(chunk: &str, pos: usize, xor_mask: u8) -> String {
    const A: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let mut chars: Vec<char> = chunk.chars().collect();
    let idx = 3 + pos;
    assert!(
        idx < chars.len(),
        "corrupt position {pos} past data-part (chunk len {})",
        chars.len()
    );
    let sym = A
        .iter()
        .position(|&b| b == (chars[idx] as u8).to_ascii_lowercase())
        .unwrap() as u8;
    chars[idx] = A[((sym ^ (xor_mask & 0x1F)) & 0x1F) as usize] as char;
    chars.into_iter().collect()
}
