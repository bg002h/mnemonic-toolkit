//! Task 4: `Skeleton` and `SkeletonKey` — membership is exhaustive and
//! fixed by the design (root + inner_wsh + template + fp_partition +
//! key_partition + key_path_kind), `keys_present` is deliberately not in
//! the key, and an incomplete walk yields no key at all.
mod common;
use md_codec::skeleton::{skeleton, skeleton_key};

#[test]
fn seated_and_unseated_do_not_share_a_key() {
    let unseated = common::kofn_recovery();
    let seated = common::seated(&[
        (0, [0xaa; 4]),
        (1, [0xbb; 4]),
        (2, [0xcc; 4]),
        (3, [0xdd; 4]),
    ]);
    // Distinct fingerprints => every slot its own group => same partition
    // shape as template-only? NO: template-only partitions to nothing. The
    // key differs, and that is correct -- identity is part of the key.
    assert_ne!(
        skeleton_key(&skeleton(&unseated).unwrap()),
        skeleton_key(&skeleton(&seated).unwrap())
    );
}

#[test]
fn a_nums_key_path_and_an_unspendable_xpub_do_not_share_a_key() {
    let nums = common::tr_nums_two_leaves();
    let xpub = common::tr_unspendable_xpub_two_leaves();
    assert_ne!(
        skeleton_key(&skeleton(&nums).unwrap()),
        skeleton_key(&skeleton(&xpub).unwrap()),
        "Nunchuk treats these as different wallets (F-449)"
    );
    // CORRECTED RATIONALE (fix round 1, I-3): this assertion passes, but not
    // because `key_path_kind` is present in the serialized string. `nums`
    // and `xpub` differ in `is_nums`, which the renderer ALWAYS reflects in
    // `template` too -- the internal key renders as either the literal
    // NUMS_H_POINT_X_ONLY_HEX constant or `@{index}/...`, two token shapes
    // that can never collide (`crate::render::render_node`'s `Tag::Tr` arm)
    // -- so `skeleton(&nums)` and `skeleton(&xpub)` differ by `template`
    // alone, with or without `key_path_kind` in the key. This test pins
    // F-449's OUTCOME (the two do not share a key); it does not exercise
    // `key_path_kind`'s presence in the string independently of `template`.
    // See `key_path_kind_is_part_of_the_key_independent_of_template` below
    // for a test that does.
}

/// Stage 1b task 5 (the brief's own Step 1 item: "`skeleton_key` differing
/// between the kinds"): wire kind 0 (`NumsPoint`) and wire kind 1
/// (`LianaUnspendable`) over the SAME two-leaf shape must not share a
/// `SkeletonKey`. Differs by `template` (the marker vs. the NUMS hex,
/// `liana_render.rs` pins that half directly) AND by `key_path_kind`
/// (`Nums` vs. `LianaUnspendable`, both now DISTINCT `KeyPathKind` values —
/// see `key_path_kind_is_part_of_the_key_independent_of_template` below for
/// the case that isolates the latter).
#[test]
fn a_nums_key_path_and_a_liana_unspendable_key_path_do_not_share_a_key() {
    let nums = common::tr_nums_two_leaves();
    let liana = common::tr_liana_unspendable_two_leaves();
    assert_ne!(
        skeleton_key(&skeleton(&nums).unwrap()),
        skeleton_key(&skeleton(&liana).unwrap()),
        "wire kind 0 and wire kind 1 must not collide in the SkeletonKey"
    );
}

#[test]
fn sh_wsh_and_bare_sh_do_not_share_a_key() {
    // sh(wsh) is NOT a ScriptKind value; it is root=Sh plus inner_wsh=true.
    assert_ne!(
        skeleton_key(&skeleton(&common::sh_wsh_2of3()).unwrap()),
        skeleton_key(&skeleton(&common::bare_sh_2of3()).unwrap())
    );
}

#[test]
fn the_serialization_is_stable_and_documented() {
    let k = skeleton_key(&skeleton(&common::kofn_recovery()).unwrap());
    let s = k.as_str();
    assert!(
        s.contains('\u{001F}'),
        "template and partitions are separated: {s}"
    );
    assert_eq!(
        skeleton_key(&skeleton(&common::kofn_recovery()).unwrap()).as_str(),
        s,
        "the key is a pure function of the descriptor"
    );
}

#[test]
fn an_incomplete_walk_yields_no_key() {
    let d = common::unclassifiable();
    assert!(
        skeleton(&d).is_err(),
        "PolicyShape.complete=false must not produce a key -- a partial \
         decomposition keyed as if whole is a false evidence match"
    );
}

/// Not one of the plan's five prescribed tests -- added to make the hard
/// constraint machine-checkable rather than merely stated: `keys_present`
/// MUST be read from raw TLV presence (`Descriptor::is_wallet_policy`),
/// never inferred from partition emptiness. `seated()` is exactly the
/// fixture that tells the two derivations apart: it sets ONLY the
/// Fingerprints TLV, never Pubkeys, so `is_wallet_policy()` -- which checks
/// Pubkeys alone -- reports `false`, while `fp_partition` is very much
/// non-empty (real fingerprint data was measured). A `keys_present` wrongly
/// inferred from partition emptiness would report `true` here instead.
#[test]
fn keys_present_is_read_from_tlv_presence_not_partition_emptiness() {
    let d = common::seated(&[
        (0, [0xaa; 4]),
        (1, [0xbb; 4]),
        (2, [0xcc; 4]),
        (3, [0xdd; 4]),
    ]);
    let s = skeleton(&d).unwrap();
    assert!(
        s.fp_partition.iter().any(|p| !p.is_empty()),
        "fixture sanity: fp_partition must be non-empty for this to be a real test: {:?}",
        s.fp_partition
    );
    assert!(
        !s.keys_present,
        "seated() carries only Fingerprints, never Pubkeys -- is_wallet_policy() is false"
    );
}

/// Not one of the plan's five prescribed tests (fix round 1, I-1): the gate
/// on `expand_per_at_n` was unguarded -- deleting it reds 0 of 574 tests,
/// because none of the five prescribed tests builds a descriptor with real
/// key TLVs AND an unresolved origin. `dead_card_real_fingerprints_
/// unresolved_origin` is exactly that: the reviewer's counterexample was
/// that such a card produces a key byte-identical to a template-only card
/// sharing its tree, WITHOUT the gate -- the false-evidence-match this
/// whole task exists to prevent.
#[test]
fn a_dead_card_with_real_keys_and_unresolved_origin_yields_no_skeleton() {
    let dead = common::dead_card_real_fingerprints_unresolved_origin();
    let err = skeleton(&dead).expect_err(
        "a card whose origin cannot resolve must refuse, even though it carries real key TLVs",
    );
    assert!(
        matches!(err, md_codec::skeleton::SkeletonError::KeysDoNotExpand(_)),
        "wrong refusal reason: {err:?}"
    );
}

/// Not one of the plan's five prescribed tests (fix round 1, I-2):
/// `key_partition` IS part of the key per design §1A, but every prescribed
/// fixture's `key_partition` comes back empty, so dropping it from the
/// serialized string reds 0 tests. `seated_same_key` is the fixture that
/// tells the two cases apart: `seated_same_key(&[0, 3])` and
/// `seated_same_key(&[])` share the same template and the same (empty --
/// neither sets Fingerprints) `fp_partition`, and differ ONLY in
/// `key_partition` (one group `{0,3}` plus two singletons, vs four
/// singletons).
#[test]
fn key_partition_is_part_of_the_key() {
    let shared = skeleton(&common::seated_same_key(&[0, 3])).unwrap();
    let distinct = skeleton(&common::seated_same_key(&[])).unwrap();
    assert_eq!(
        shared.template, distinct.template,
        "fixture sanity: only key_partition should differ"
    );
    assert_eq!(
        shared.fp_partition, distinct.fp_partition,
        "fixture sanity: only key_partition should differ"
    );
    assert_ne!(
        shared.key_partition, distinct.key_partition,
        "fixture sanity: key_partition must actually differ"
    );
    assert_ne!(
        skeleton_key(&shared),
        skeleton_key(&distinct),
        "key_partition must be part of the serialized key"
    );
}

/// Not one of the plan's five prescribed tests (fix round 1, I-3): no REAL
/// descriptor pair can pin `key_path_kind`'s presence in the serialized
/// string independently of `template` -- `is_nums` always changes
/// `template` too (see the corrected rationale on
/// `a_nums_key_path_and_an_unspendable_xpub_do_not_share_a_key` above), so
/// dropping the `key_path_kind` token reds 0 of the prescribed tests. This
/// test instead exercises `skeleton_key` directly against its ACTUAL
/// signature (`&Skeleton`, not `&Descriptor`): three hand-built `Skeleton`
/// values, identical in every field except `shape.key_path` -- a
/// combination `skeleton()` itself could never produce from a real
/// `Descriptor` (`policy_shape` ties `key_path` to `is_nums`/the kind bit,
/// which the renderer always reflects in `template`), but one
/// `skeleton_key`'s own contract must still serialize correctly, since
/// nothing in its signature restricts it to `skeleton()`-produced inputs.
///
/// Stage 1b task 5 extends the original `nums`/`xpub` pair to a THIRD value,
/// `LianaUnspendable` (`KeyPathKind` grew a fourth variant this task) — all
/// three now pairwise distinct in the serialized key, independent of
/// `template`.
#[test]
fn key_path_kind_is_part_of_the_key_independent_of_template() {
    use md_codec::policy_shape::{KeyPathKind, PolicyShape};
    let mut nums = skeleton(&common::kofn_recovery()).unwrap();
    nums.shape = PolicyShape {
        complete: true,
        key_path: KeyPathKind::Nums,
        branches: Vec::new(),
        tap_depth: 0,
    };
    let mut xpub = nums.clone();
    xpub.shape.key_path = KeyPathKind::Xpub;
    let mut liana = nums.clone();
    liana.shape.key_path = KeyPathKind::LianaUnspendable;
    assert_eq!(
        nums.template, xpub.template,
        "fixture sanity: only shape.key_path differs"
    );
    assert_eq!(
        nums.template, liana.template,
        "fixture sanity, same as above"
    );
    assert_eq!(nums.fp_partition, xpub.fp_partition);
    assert_eq!(nums.key_partition, xpub.key_partition);
    assert_ne!(
        skeleton_key(&nums),
        skeleton_key(&xpub),
        "key_path_kind must be part of the serialized key even though no real \
         descriptor can exercise this independently of the template"
    );
    assert_ne!(
        skeleton_key(&nums),
        skeleton_key(&liana),
        "Nums and LianaUnspendable must not collide either"
    );
    assert_ne!(
        skeleton_key(&xpub),
        skeleton_key(&liana),
        "Xpub and LianaUnspendable must not collide"
    );
}

/// Not one of the plan's five prescribed tests (fix round 1, I-4):
/// `Skeleton::root`/`Skeleton::inner_wsh` had zero assertions anywhere --
/// mutating `sh(wsh)` to report `inner_wsh = false` reds 0 of the prior six
/// tests (the redundancy with `template`'s literal wrapper spelling is real,
/// but it is an argument, not a gate). A six-`RootKind`-value table (all six
/// values, both `Sh` spellings), assigning the fields directly rather than
/// going through `skeleton_key` -- exactly what the redundancy argument
/// says must not be needed to tell these apart, but a gate must check
/// anyway.
#[test]
fn root_and_inner_wsh_are_read_correctly() {
    use md_codec::policy_shape::RootKind;
    use md_codec::tag::Tag;

    let wpkh = skeleton(&common::descriptor_of(common::keyarg(Tag::Wpkh, 0), 1)).unwrap();
    assert_eq!((wpkh.root, wpkh.inner_wsh), (RootKind::Wpkh, false));

    let pkh = skeleton(&common::descriptor_of(common::keyarg(Tag::Pkh, 0), 1)).unwrap();
    assert_eq!((pkh.root, pkh.inner_wsh), (RootKind::Pkh, false));

    let wsh = skeleton(&common::kofn_recovery()).unwrap();
    assert_eq!((wsh.root, wsh.inner_wsh), (RootKind::Wsh, false));

    let bare_sh = skeleton(&common::bare_sh_2of3()).unwrap();
    assert_eq!(
        (bare_sh.root, bare_sh.inner_wsh),
        (RootKind::Sh, false),
        "bare sh(...) must report inner_wsh = false"
    );

    let sh_wsh = skeleton(&common::sh_wsh_2of3()).unwrap();
    assert_eq!(
        (sh_wsh.root, sh_wsh.inner_wsh),
        (RootKind::Sh, true),
        "sh(wsh(...)) must report inner_wsh = true"
    );

    let sh_wpkh = skeleton(&common::descriptor_of(
        common::wrap(Tag::Sh, common::keyarg(Tag::Wpkh, 0)),
        1,
    ))
    .unwrap();
    assert_eq!((sh_wpkh.root, sh_wpkh.inner_wsh), (RootKind::ShWpkh, false));

    let tr = skeleton(&common::tr_nums_two_leaves()).unwrap();
    assert_eq!((tr.root, tr.inner_wsh), (RootKind::Tr, false));
}

// ─────────────────────────────────────────────────────────────────────────
// Final whole-branch review, I-3: golden literal-key tests.
//
// Every key assertion above compares two COMPUTED keys (`assert_ne!`
// between two `skeleton_key()` calls). That is invisible to any mutation
// that changes serialization UNIFORMLY on both sides -- the reviewer applied
// five, separately, and 581/581 stayed green each time:
//
//   1. `for path in fp.iter().rev()`             (render_fp_partition)
//   2. paths rendered then `sort()`ed              (render_fp_partition)
//   3. drop the `,` between slots                  (render_slots)
//   4. drop the outer `[`/`]`                       (render_group_list)
//   5. emit key_partition before fp_partition       (skeleton_key)
//
// These two tests pin LITERAL expected strings instead, computed by running
// the shipped code once (not hand-derived) and pasted here as byte-for-byte
// literals. `\u{1f}` marks the same U+001F separator `skeleton_key`'s own
// doc comment names.
// ─────────────────────────────────────────────────────────────────────────

/// `seated_and_keyed_asymmetric()`: multi-path (the recovery-first `or_d`
/// shape, two branches — see [`common::kofn_recovery_first_tree`]'s doc for
/// why the operands are swapped from `kofn_recovery`'s order), with an
/// ASYMMETRIC `fp_partition` (branch 0, the recovery leg, is the singleton
/// {3}; branch 1, the multi, groups slots {0,1} together and {2} alone) and
/// a non-empty, cross-branch `key_partition` ({0,3} share a key). One
/// fixture exercises all five of the final review's mutations: 1 (reverse
/// the per-path order) and 2 (sort the per-path order) both change this
/// string because branch 0's rendering ("[[3]]") sorts AFTER branch 1's
/// ("[[0,1][2]]") — the opposite of template-traversal order, so neither
/// mutation is a no-op; 3 (drop the `,` between slots) needs a 2+-member
/// group, which {0,1} supplies; 4 (drop the outer `[`/`]`) needs a
/// non-empty group list, which every group here is; 5 (swap the
/// `key_partition`/`fp_partition` emission order) needs the two to render to
/// DIFFERENT text, which they do (asymmetric fp groups vs. one cross-branch
/// pair plus two singletons).
#[test]
fn golden_key_multi_path_asymmetric_fp_and_key_partition() {
    let s = skeleton(&common::seated_and_keyed_asymmetric()).unwrap();
    assert_eq!(
        skeleton_key(&s).as_str(),
        "wsh(or_d(and_v(v:pkh(@3/<0;1>/*),older(older-blocks#1)),multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*)))\u{1f}[[[3]][[0,1][2]]][[0,3][1][2]]\u{1f}NotTaproot"
    );
}

/// `tr_unspendable_xpub_two_leaves()`: a taproot policy with a KEY-PATH
/// branch (I-5's extension -- a spendable/`Xpub` internal key pushed as
/// `branches[0]`) plus two tapscript leaves. `fp_partition`/`key_partition`
/// are both empty here (no TLVs attached) — this golden's job is the
/// `tr(...)` template shape and the `key_path_kind` label, not the five
/// partition-serialization mutations (the first golden above covers those).
#[test]
fn golden_key_taproot_with_key_path_branch() {
    let s = skeleton(&common::tr_unspendable_xpub_two_leaves()).unwrap();
    assert_eq!(
        skeleton_key(&s).as_str(),
        "tr(@0/<0;1>/*,{pk(@1/<0;1>/*),pk(@2/<0;1>/*)})\u{1f}[[][][]][]\u{1f}Xpub"
    );
}

/// `leaf_keys_ascending` is STRICT: Nunchuk deduplicates, Liana does not, so
/// a repeated leaf key breaks agreement. Mutation: `pk <= p` -> `pk < p` in
/// `skeleton.rs` -> the duplicate case reads `Some(true)` and this reds.
#[test]
fn leaf_keys_ascending_is_strict_and_order_sensitive() {
    use common::{keyarg, taptree2, test_xpubs, tr_liana_unspendable_two_leaves_with_pubkeys};
    use md_codec::tag::Tag;
    use md_codec::tree::{Body, InternalKey, Node};

    let a = test_xpubs()[0];
    let b = test_xpubs()[1];
    let mut ab = tr_liana_unspendable_two_leaves_with_pubkeys();
    ab.tlv.pubkeys = Some(vec![(0, a), (1, b)]);
    let mut ba = ab.clone();
    ba.tlv.pubkeys = Some(vec![(0, b), (1, a)]);
    let got = [ab, ba].map(|d| skeleton(&d).unwrap().leaf_keys_ascending);
    assert!(
        got == [Some(true), Some(false)] || got == [Some(false), Some(true)],
        "exactly one order ascends: {got:?}"
    );

    let mut dup = tr_liana_unspendable_two_leaves_with_pubkeys();
    dup.tree = Node {
        tag: Tag::Tr,
        body: Body::Tr {
            internal_key: InternalKey::LianaUnspendable,
            tree: Some(Box::new(taptree2(keyarg(Tag::PkK, 0), keyarg(Tag::PkK, 0)))),
        },
    };
    assert_eq!(skeleton(&dup).unwrap().leaf_keys_ascending, Some(false));
}
