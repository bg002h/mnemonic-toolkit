//! Task 1: port of the branch-decomposition walk (`md/policy_shape.go`),
//! pinning the slot-set extension and the honesty contract's refusal.
mod common;
use common::{keyarg, multikeys, node2, node3, timelock, wrap};
use md_codec::compose::{HashKind, HashLock, Lock as ComposeLock};
use md_codec::policy_shape::{KeyPathKind, Lock, LockKind, policy_shape};
use md_codec::tag::Tag;

/// `wsh(or_d(multi(2,@0,@1,@2), and_v(v:pkh(@3), older(26280))))`
fn kofn_recovery() -> md_codec::encode::Descriptor {
    let primary = multikeys(Tag::Multi, 2, vec![0, 1, 2]);
    let recovery = node2(
        Tag::AndV,
        wrap(Tag::Verify, keyarg(Tag::Pkh, 3)),
        timelock(Tag::Older, 26280),
    );
    let tree = wrap(Tag::Wsh, node2(Tag::OrD, primary, recovery));
    common::descriptor_of(tree, 4)
}

#[test]
fn branch_retains_its_slot_set_not_just_a_count() {
    let shape = policy_shape(&kofn_recovery());
    assert!(
        shape.complete,
        "the walk must classify every node of a shipped preset"
    );
    assert_eq!(shape.key_path, KeyPathKind::NotTaproot);
    assert_eq!(shape.branches.len(), 2, "or_d yields two spend paths");

    // THE EXTENSION: which slots, not how many.
    assert_eq!(shape.branches[0].slots, vec![0, 1, 2]);
    assert_eq!(shape.branches[1].slots, vec![3]);
    // The count the Go original kept is still derivable, and must agree.
    assert_eq!(shape.branches[0].slots.len(), 3);
}

/// If the walk meets something it cannot classify it must report
/// `complete = false`, so the caller shows the honest-minimal screen instead
/// of a partial (and therefore misleading) decomposition.
///
/// DEVIATION FROM THE BRIEF'S DRAFT: the brief proposed a bare `thresh` over
/// keys at the wsh root. Traced against the Go original (`md/policy_shape.go`
/// `collect`'s `tagThresh` arm, `splitBranches`'s `default` arm calling
/// `branchOf`), that shape does NOT refuse -- `thresh(2,pk_k(@0),pk_k(@1),
/// pk_k(@2))` is exactly the "combinatorial-but-still-one-branch" case the
/// Go doc comment on `splitBranches` describes, and it decodes to one
/// Complete=true branch with Keys=3, K=0/N=0. The brief's own step 6 text
/// anticipated this ("If the Go original DOES classify this shape, pick
/// whatever node it refuses").
///
/// The fork's OWN test, `TestPolicyShapeRefusesAnUnknownTag`
/// (md/policy_shape_test.go:175-189), pins the actual refusing shape: a
/// `tagTr` node nested inside a `wsh` script. `collect` has no case for
/// `Tag::Tr` (or `Tag::TapTree`) -- its doc comment says so explicitly:
/// "tagTr and tagTapTree cannot appear inside a branch ... Refuse rather
/// than guess." That is what this test pins, ported 1:1 from the Go test.
#[test]
fn an_unclassifiable_node_sets_complete_false_and_yields_no_branches() {
    let tr_leaf = common::tr_node(true, 0, None);
    let tree = wrap(Tag::Wsh, tr_leaf);
    let shape = policy_shape(&common::descriptor_of(tree, 1));
    assert!(!shape.complete);
    assert!(
        shape.branches.is_empty(),
        "an incomplete walk must not hand back a partial decomposition"
    );
}

// ---------------------------------------------------------------------
// Fix round 1, I-2: the k-of-n logic (`plain_multi`/`sole_multi` and the
// `nkeys == br.slots.len()` guard) had zero coverage. The four tests below
// port the Go original's own anti-invention suite
// (`TestPolicyShapeNeverClaimsAPlainThresholdItCannotSee`,
// md/policy_shape_test.go:84-167) plus one positive case. The second test,
// `mixed_branch_with_extra_key_beside_a_multi_reports_zero_threshold`, is
// the one that must fail if the guard's equality is mutated to `true` —
// see the mutation result recorded in the task report.
// ---------------------------------------------------------------------

/// `wsh(and_v(v:pk(@0), older(144)))` — a single key behind a timelock. K/N
/// must stay 0/0: this is not a plain k-of-N over keys (round-1 review
/// pin, ported from the Go test's first case, md/policy_shape_test.go:84-108).
#[test]
fn and_v_pk_with_timelock_reports_zero_threshold_not_one_of_one() {
    let tree = wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
            timelock(Tag::Older, 144),
        ),
    );
    let shape = policy_shape(&common::descriptor_of(tree, 1));
    assert!(shape.complete, "and_v(v:pk,older) should be understood");
    assert_eq!(shape.branches.len(), 1);
    let b = &shape.branches[0];
    assert_eq!((b.k, b.n), (0, 0), "not a plain k-of-N");
    assert_eq!(b.slots, vec![0]);
    assert_eq!(
        b.locks,
        vec![Lock {
            kind: LockKind::OlderBlocks,
            value: 144
        }]
    );
}

/// THE GUARD ITSELF: `wsh(and_v(v:older(10), or_d(multi(2,@0,@1,@2),
/// c:pk_k(@3))))`. `split_branches` does not split an `or` sitting under
/// `and_v`, so this is ONE branch holding `multi(2,@0,@1,@2)` PLUS a key
/// outside it — `@3` spends this branch alone after 10 blocks. Reporting
/// the multi's 2-of-3 for the whole branch would tell an operator two of
/// three signatures are needed where one other key spends alone: the exact
/// misreading round-1 review I-2 found, and the reason `branch_of` requires
/// `nkeys == br.slots.len()` before trusting `sole_multi`'s answer. Ported
/// from the Go test's `mixed` case (md/policy_shape_test.go:121-148).
///
/// MUTATION RESULT (recorded in the task report): with the guard's
/// `nkeys as usize == br.slots.len()` temporarily replaced by a bare
/// `true`, this test fails — `(b.k, b.n)` comes back `(2, 3)` instead of
/// `(0, 0)`. So does
/// `mixed_branch_without_a_lock_still_reports_zero_threshold` below, the
/// same way. Both restored before this commit.
#[test]
fn mixed_branch_with_extra_key_beside_a_multi_reports_zero_threshold() {
    let tree = wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, timelock(Tag::Older, 10)),
            node2(
                Tag::OrD,
                multikeys(Tag::Multi, 2, vec![0, 1, 2]),
                wrap(Tag::Check, keyarg(Tag::PkK, 3)),
            ),
        ),
    );
    let shape = policy_shape(&common::descriptor_of(tree, 4));
    assert!(shape.complete, "the mixed branch should be understood");
    assert_eq!(
        shape.branches.len(),
        1,
        "split_branches does not split an or under and_v, which is what makes this branch mixed"
    );
    let b = &shape.branches[0];
    assert_eq!(
        b.slots,
        vec![0, 1, 2, 3],
        "INCONCLUSIVE fixture check: want all four keys"
    );
    assert_eq!(
        (b.k, b.n),
        (0, 0),
        "a branch holding multi(2,@0,@1,@2) AND @3 alone has no single k-of-n"
    );
}

/// The same shape with the timelock removed, so the guard is pinned as
/// being about KEYS, not locks (Go test's `mixedNoLock`,
/// md/policy_shape_test.go:151-166).
#[test]
fn mixed_branch_without_a_lock_still_reports_zero_threshold() {
    let tree = wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, multikeys(Tag::Multi, 2, vec![0, 1, 2])),
            wrap(Tag::Check, keyarg(Tag::PkK, 3)),
        ),
    );
    let shape = policy_shape(&common::descriptor_of(tree, 4));
    assert!(shape.complete);
    assert_eq!(shape.branches.len(), 1);
    let b = &shape.branches[0];
    assert_eq!((b.k, b.n), (0, 0));
    assert_eq!(b.slots, vec![0, 1, 2, 3]);
}

/// The POSITIVE case the guard must still allow through `sole_multi`: a
/// multi that accounts for EVERY key the branch references, sitting behind
/// a lock (not at the branch's own root, so `plain_multi` does not apply
/// and only `sole_multi` can find it), still reports its threshold — "a
/// threshold behind a lock or a hash is still a threshold" (fable review r0
/// I-3, quoted in `branch_of`'s doc comment).
#[test]
fn multi_behind_a_lock_still_reports_its_threshold() {
    let tree = wrap(
        Tag::Wsh,
        node2(
            Tag::AndV,
            wrap(Tag::Verify, timelock(Tag::Older, 10)),
            multikeys(Tag::Multi, 2, vec![0, 1, 2]),
        ),
    );
    let shape = policy_shape(&common::descriptor_of(tree, 3));
    assert!(shape.complete);
    assert_eq!(shape.branches.len(), 1);
    let b = &shape.branches[0];
    assert_eq!((b.k, b.n, b.sorted), (2, 3, false));
    assert_eq!(b.slots, vec![0, 1, 2]);
}

/// `sortedmulti` sets `sorted = true`; the bare `multi` above sets it
/// `false`. Both are reached through `plain_multi` (the multi sits at the
/// branch's own root here, not behind a wrapper).
#[test]
fn sortedmulti_sets_sorted_true_plain_multi_does_not() {
    let sorted_tree = wrap(Tag::Wsh, multikeys(Tag::SortedMulti, 2, vec![0, 1, 2]));
    let sorted_shape = policy_shape(&common::descriptor_of(sorted_tree, 3));
    assert!(sorted_shape.complete);
    let sb = &sorted_shape.branches[0];
    assert_eq!((sb.k, sb.n, sb.sorted), (2, 3, true));
    assert_eq!(sb.slots, vec![0, 1, 2]);

    let plain_tree = wrap(Tag::Wsh, multikeys(Tag::Multi, 2, vec![0, 1, 2]));
    let plain_shape = policy_shape(&common::descriptor_of(plain_tree, 3));
    assert!(plain_shape.complete);
    let pb = &plain_shape.branches[0];
    assert_eq!((pb.k, pb.n, pb.sorted), (2, 3, false));
}

// ---------------------------------------------------------------------
// Fix round 1, I-2: lock-band coverage. `compose::Lock::operand()` is
// `lock_from_wire`'s exact inverse, so round-tripping through it is nearly
// free and exercises all four bands `lock_from_wire` itself has no direct
// test for (it is a private function; this is how its behavior is pinned
// from the public surface).
// ---------------------------------------------------------------------

#[test]
fn lock_bands_round_trip_through_compose_lock_operand() {
    let cases: [(Tag, u32, LockKind, u32); 4] = [
        // older, blocks: no units flag set.
        (Tag::Older, 144, LockKind::OlderBlocks, 144),
        // older, 512-second units: bit 22 set; the *reported* value is the
        // unit count with that bit cleared.
        (Tag::Older, (1u32 << 22) + 5, LockKind::OlderUnits, 5),
        // after, height: strictly below the locktime threshold.
        (Tag::After, 700_000, LockKind::AfterHeight, 700_000),
        // after, time: at/above the locktime threshold.
        (
            Tag::After,
            1_600_000_000,
            LockKind::AfterTime,
            1_600_000_000,
        ),
    ];
    for (wire_tag, wire_operand, want_kind, want_value) in cases {
        let tree = wrap(
            Tag::Wsh,
            node2(
                Tag::AndV,
                wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
                timelock(wire_tag, wire_operand),
            ),
        );
        let shape = policy_shape(&common::descriptor_of(tree, 1));
        assert!(shape.complete);
        let lock = shape.branches[0].locks[0];
        assert_eq!(lock.kind, want_kind, "band: {wire_tag:?}/{wire_operand}");
        assert_eq!(lock.value, want_value);

        // The inverse: rebuild a compose::Lock from what policy_shape read
        // back and check .operand() reproduces the exact wire (tag, operand).
        let composed = match lock.kind {
            LockKind::OlderBlocks => ComposeLock::OlderBlocks(lock.value as u16),
            LockKind::OlderUnits => ComposeLock::OlderUnits(lock.value as u16),
            LockKind::AfterHeight => ComposeLock::AfterHeight(lock.value),
            LockKind::AfterTime => ComposeLock::AfterTime(lock.value),
        };
        assert_eq!(composed.operand().unwrap(), (wire_tag, wire_operand));
    }
}

// ---------------------------------------------------------------------
// Fix round 1, I-1 + I-2: hashlocks now come from `crate::compose`, and had
// zero coverage of their own.
// ---------------------------------------------------------------------

#[test]
fn hashlocks_report_kind_and_digest_for_all_four_kinds() {
    let digest32 = [0xABu8; 32];
    let digest20 = [0xCDu8; 20];

    for (tag, kind) in [
        (Tag::Sha256, HashKind::Sha256),
        (Tag::Hash256, HashKind::Hash256),
    ] {
        let tree = wrap(
            Tag::Wsh,
            node2(
                Tag::AndV,
                wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
                common::hash32(tag, digest32),
            ),
        );
        let shape = policy_shape(&common::descriptor_of(tree, 1));
        assert!(shape.complete);
        assert_eq!(
            shape.branches[0].hashlocks,
            vec![HashLock::new(kind, digest32)]
        );
    }

    for (tag, kind) in [
        (Tag::Ripemd160, HashKind::Ripemd160),
        (Tag::Hash160, HashKind::Hash160),
    ] {
        let tree = wrap(
            Tag::Wsh,
            node2(
                Tag::AndV,
                wrap(Tag::Verify, keyarg(Tag::PkK, 0)),
                common::hash20(tag, digest20),
            ),
        );
        let shape = policy_shape(&common::descriptor_of(tree, 1));
        assert!(shape.complete);
        let mut padded = [0u8; 32];
        padded[..20].copy_from_slice(&digest20);
        assert_eq!(
            shape.branches[0].hashlocks,
            vec![HashLock::new(kind, padded)]
        );
    }
}

/// THE FIX I-1 CLOSES: two `HashLock`s with the same kind and the same
/// VISIBLE digest but different alloc-gate padding still compare equal.
/// Before reusing `crate::compose::HashLock`, this module defined its own
/// `HashLock` with a *derived* `PartialEq` over the raw `[u8; 32]`, which
/// would have made this exact case fail.
#[test]
fn hashlock_equality_ignores_alloc_gate_padding() {
    let mut a = [0u8; 32];
    a[..20].copy_from_slice(&[0x11; 20]);
    let mut b = a;
    b[20..].fill(0xFF); // different padding, identical visible 20 bytes
    assert_ne!(a, b, "fixture check: the arrays must actually differ");
    assert_eq!(
        HashLock::new(HashKind::Ripemd160, a),
        HashLock::new(HashKind::Ripemd160, b)
    );
}

// ---------------------------------------------------------------------
// Fix round 1, I-2: taproot coverage — Nums/Xpub, the taptree walk,
// tap_depth/depth, and every-leaf-visited.
// ---------------------------------------------------------------------

/// A key-path-only NUMS `tr` has NO leaves: reporting a branch would invent
/// a spend path that does not exist.
#[test]
fn bare_nums_taproot_keypath_only_reports_no_branches() {
    let tree = common::tr_node(true, 0, None);
    let shape = policy_shape(&common::descriptor_of(tree, 1));
    assert!(shape.complete);
    assert_eq!(shape.key_path, KeyPathKind::Nums);
    assert!(shape.branches.is_empty());
    assert_eq!(shape.tap_depth, 0);
}

/// A key-path-only real-key `tr` has NO tapscript leaves, but the key path
/// ITSELF is a spend path (design §1A, fix round 1 I-5: "a 'path' for `tr`
/// is a taptree leaf, plus the key path as path 0 when the internal key is
/// spendable"): one Schnorr signature against a real (non-NUMS) internal
/// key satisfies this output with no script and no taptree proof at all, so
/// treating this policy as having NO spend path would be wrong, not merely
/// incomplete. Renamed from `bare_xpub_taproot_keypath_only_reports_no_
/// branches` (fix round 1): that name and assertion predate I-5 and were
/// simply incorrect once the key path counts.
#[test]
fn bare_xpub_taproot_keypath_only_reports_the_key_path_as_its_one_branch() {
    let tree = common::tr_node(false, 0, None);
    let shape = policy_shape(&common::descriptor_of(tree, 1));
    assert!(shape.complete);
    assert_eq!(shape.key_path, KeyPathKind::Xpub);
    assert_eq!(
        shape.branches.len(),
        1,
        "the key path is its own spend path, not zero"
    );
    assert_eq!(shape.branches[0].slots, vec![0], "the internal key's slot");
    assert_eq!(
        shape.branches[0].k, 0,
        "no threshold node -- a single Schnorr signature, not a multi"
    );
    assert_eq!(shape.branches[0].depth, 0);
    assert!(shape.branches[0].locks.is_empty());
    assert!(shape.branches[0].hashlocks.is_empty());
    assert_eq!(shape.tap_depth, 0);
}

/// A spendable internal key PLUS one tapscript leaf: two spend paths, key
/// path first (design §1A "path 0"), the leaf second. Renamed from
/// `taproot_with_one_leaf_reports_xpub_keypath_and_depth_zero` (fix round 1,
/// I-5): that name/assertion (`branches.len() == 1`) predates the key-path
/// branch and undercounted this policy's spend paths by one.
#[test]
fn taproot_with_one_leaf_reports_the_key_path_then_the_leaf() {
    let tree = common::tr_node(false, 0, Some(keyarg(Tag::PkK, 1)));
    let shape = policy_shape(&common::descriptor_of(tree, 2));
    assert!(shape.complete);
    assert_eq!(shape.key_path, KeyPathKind::Xpub);
    assert_eq!(
        shape.branches.len(),
        2,
        "the key path (@0) plus the one tapscript leaf (@1)"
    );
    assert_eq!(
        shape.branches[0].slots,
        vec![0],
        "key path is path 0, per design §1A"
    );
    assert_eq!(shape.branches[0].depth, 0);
    assert_eq!(shape.branches[1].slots, vec![1], "the leaf");
    assert_eq!(shape.branches[1].depth, 0);
    assert_eq!(shape.tap_depth, 0);
}

/// `{{A,B},{C,{D,E}}}` — five leaves, unbalanced, depths 2,2,2,3,3, PLUS the
/// spendable key path as branch 0 at depth 0 (fix round 1, I-5). Ported
/// from `TestPolicyShapeReportsEveryLeafOfADeepTree`
/// (md/policy_shape_test.go:193-218): pins the objection SPEC §4.2/C3
/// raises — a summary must not describe one leaf and stay silent about the
/// others. `tap_depth` is unaffected by the key-path branch: it is computed
/// only inside `walk_tap_tree`'s taptree descent, and the key path is not
/// part of the taptree.
#[test]
fn deep_taptree_reports_every_leaf_and_the_correct_max_depth() {
    let leaf = |i: u8| keyarg(Tag::PkK, i);
    let branch = |l, r| common::taptree2(l, r);
    let tree_body = branch(
        branch(leaf(1), leaf(2)),
        branch(leaf(3), branch(leaf(4), leaf(5))),
    );
    let tree = common::tr_node(false, 0, Some(tree_body));
    let shape = policy_shape(&common::descriptor_of(tree, 6));
    assert!(
        shape.complete,
        "a well-formed deep taptree must be understood"
    );
    assert_eq!(
        shape.branches.len(),
        6,
        "branches = {}, want 6 (key path + 5 leaves) -- a path was dropped",
        shape.branches.len()
    );
    assert_eq!(shape.tap_depth, 3);
    assert_eq!(shape.key_path, KeyPathKind::Xpub);
    assert_eq!(
        shape.branches[0].slots,
        vec![0],
        "key path is path 0, per design §1A"
    );
    let depths: Vec<u8> = shape.branches.iter().map(|b| b.depth).collect();
    assert_eq!(depths, vec![0, 2, 2, 2, 3, 3]);
}

// ---------------------------------------------------------------------
// Stage 1b task 5: wire kind 1 (LianaUnspendable) is a FOURTH
// `KeyPathKind`, distinct from `Nums` even though the wire's `is_nums` flag
// is `true` for both. G-1 gating site (`policy_shape.rs:251`): reverting
// the split back to `NumsPoint | LianaUnspendable => KeyPathKind::Nums`
// makes `liana_unspendable_taproot_is_its_own_key_path_kind` fail
// immediately (`shape.key_path` would read `Nums`, not `LianaUnspendable`).
// ---------------------------------------------------------------------

#[test]
fn liana_unspendable_taproot_is_its_own_key_path_kind() {
    let shape = policy_shape(&common::tr_liana_unspendable_two_leaves());
    assert!(shape.complete);
    assert_eq!(shape.key_path, KeyPathKind::LianaUnspendable);
    assert_ne!(
        shape.key_path,
        KeyPathKind::Nums,
        "wire kind 1 must not collapse onto kind 0's classification"
    );
}

/// A `LianaUnspendable` internal key is provably unspendable, like `Nums`
/// (SPEC §2/§4b) — same invariant as a NUMS key path: no key-path `Branch`
/// is pushed for it, only the tapscript leaves.
#[test]
fn liana_unspendable_taproot_pushes_no_key_path_branch() {
    let shape = policy_shape(&common::tr_liana_unspendable_two_leaves());
    assert!(shape.complete);
    assert_eq!(
        shape.branches.len(),
        2,
        "two tapscript leaves, no key-path branch (unspendable, like NUMS)"
    );
    assert_eq!(shape.branches[0].slots, vec![0]);
    assert_eq!(shape.branches[1].slots, vec![1]);
}

// ---------------------------------------------------------------------
// Fix round 1, I-2: the sh(wsh) unwrap and andor.
// ---------------------------------------------------------------------

/// `sh(wsh(multi(2,@0,@1,@2)))` must decompose identically to bare
/// `wsh(multi(2,@0,@1,@2))` — the unwrap exists so the branch describes the
/// script that actually runs, not the wrapper around it.
#[test]
fn sh_wsh_unwraps_to_the_same_shape_as_bare_wsh() {
    let inner = multikeys(Tag::Multi, 2, vec![0, 1, 2]);
    let sh_wsh_tree = wrap(Tag::Sh, wrap(Tag::Wsh, inner.clone()));
    let wsh_tree = wrap(Tag::Wsh, inner);

    let sh_wsh_shape = policy_shape(&common::descriptor_of(sh_wsh_tree, 3));
    let wsh_shape = policy_shape(&common::descriptor_of(wsh_tree, 3));

    assert!(sh_wsh_shape.complete);
    assert_eq!(sh_wsh_shape, wsh_shape);
    assert_eq!(sh_wsh_shape.branches[0].slots, vec![0, 1, 2]);
    assert_eq!(
        (sh_wsh_shape.branches[0].k, sh_wsh_shape.branches[0].n),
        (2, 3)
    );
}

/// Final whole-branch review, I-1: the single-branch fixture above cannot
/// tell whether the `sh(wsh(...))` unwrap actually runs, because a
/// single-`multi` script decomposes identically with or without it --
/// `plain_multi` fails on `Tag::Wsh` either way, then `sole_multi` finds the
/// same one multi under the (still-present, one-level-shallower) `Wsh`
/// wrapper regardless. `sh(wsh(or_d(sortedmulti(2,@0,@1,@2),
/// and_v(v:pkh(@3),older(26280)))))` is wire-reachable (18 bytes / 139 bits,
/// strict-decodes byte-identical -- reviewer-measured) and MULTI-branch: with
/// the unwrap disabled, `inner` stays the still-wrapped `Wsh` node, and
/// `split_branches`'s catch-all arm (`_ => branch_of(n, depth)`) has no
/// special case for `Tag::Wsh` -- it hands the whole node to `branch_of`,
/// whose `collect` walk treats `Tag::Wsh`'s child as plain conjunction and
/// recurses straight through the `or_d(...)` inside it (`collect` has no
/// alternative-splitting logic; only `split_branches` does), merging what
/// must be two independently satisfiable spend paths into one branch and
/// changing every slot set involved.
#[test]
fn sh_wsh_with_multiple_branches_unwraps_correctly() {
    let d = common::sh_wsh_or_d_sortedmulti_and_recovery();
    let shape = policy_shape(&d);
    assert!(shape.complete);
    assert_eq!(
        shape.branches.len(),
        2,
        "the unwrap must not merge the two independently satisfiable spend paths: {:?}",
        shape.branches
    );
    assert_eq!(shape.branches[0].slots, vec![0, 1, 2]);
    assert_eq!(shape.branches[1].slots, vec![3]);
    assert_eq!(
        (
            shape.branches[0].k,
            shape.branches[0].n,
            shape.branches[0].sorted
        ),
        (2, 3, true),
        "the primary leg is sortedmulti(2,@0,@1,@2)"
    );
}

/// `andor(X, Y, Z)` is `(X and Y) or Z`: two branches, the first a
/// conjunction of X and Y (walked, never emitted, as a synthetic `and_v`).
#[test]
fn andor_splits_into_x_and_y_or_z() {
    let tree = wrap(
        Tag::Wsh,
        node3(
            Tag::AndOr,
            keyarg(Tag::PkK, 0),
            keyarg(Tag::PkK, 1),
            keyarg(Tag::PkK, 2),
        ),
    );
    let shape = policy_shape(&common::descriptor_of(tree, 3));
    assert!(shape.complete);
    assert_eq!(shape.branches.len(), 2);
    assert_eq!(shape.branches[0].slots, vec![0, 1]);
    assert_eq!(shape.branches[1].slots, vec![2]);
}
