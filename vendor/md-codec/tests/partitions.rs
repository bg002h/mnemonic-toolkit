//! Task 3: the two partitions over key identity — `fp_partition` (per spend
//! path, for Liana's `DuplicateOriginSamePath`) and `key_partition`
//! (whole-policy, for its `DuplicateKey`) — plus the absence rule that an
//! all-zero fingerprint/xpub sentinel never groups with another occurrence
//! of itself.
mod common;
use md_codec::origin_path::{OriginPath, PathComponent};
use md_codec::policy_shape::{fp_partition, key_partition, policy_shape};

#[test]
fn slots_sharing_a_fingerprint_group_within_a_path() {
    // Path 0 seats @0 and @1 from ONE seed, @2 from another.
    let d = common::seated(&[
        (0, [0xaa; 4]),
        (1, [0xaa; 4]),
        (2, [0xbb; 4]),
        (3, [0xcc; 4]),
    ]);
    let p = fp_partition(&d, &policy_shape(&d));
    assert_eq!(p[0], vec![vec![0u8, 1], vec![2]], "path 0: {p:?}");
    assert_eq!(p[1], vec![vec![3u8]], "path 1: {p:?}");
}

#[test]
fn an_absent_fingerprint_is_its_own_singleton() {
    // md-codec's ABSENT sentinel is all-zero; two absent slots are NOT known
    // to share a signer, and grouping them would assert a relation nobody
    // measured.
    let d = common::seated(&[
        (0, [0x00; 4]),
        (1, [0x00; 4]),
        (2, [0xbb; 4]),
        (3, [0xcc; 4]),
    ]);
    let p = fp_partition(&d, &policy_shape(&d));
    assert_eq!(
        p[0],
        vec![vec![0u8], vec![1], vec![2]],
        "absent fingerprints never join: {p:?}"
    );
}

#[test]
fn key_partition_groups_by_xpub_and_origin_path_across_the_whole_policy() {
    // @0 and @3 are the SAME key at the same origin, in different paths.
    let d = common::seated_same_key(&[0, 3]);
    let kp = key_partition(&d);
    assert!(kp.contains(&vec![0u8, 3]), "whole-policy grouping: {kp:?}");
}

#[test]
fn a_template_only_card_partitions_to_nothing() {
    let d = common::kofn_recovery(); // template-only, no Pubkeys TLV
    assert!(
        fp_partition(&d, &policy_shape(&d))
            .iter()
            .all(|p| p.is_empty())
    );
    assert!(key_partition(&d).is_empty());
}

/// Task 4 fix round 1: `common::kofn_recovery()` gained an explicit
/// resolvable origin path so `skeleton()`'s stricter gate could accept it
/// (see `crates/md-codec/src/skeleton.rs`'s module doc and
/// `common::kofn_recovery`'s own doc comment). That fix made this crate's
/// only exercise of `fp_partition`/`key_partition`'s `expand_per_at_n`
/// EARLY-RETURN branches (`let Ok(expanded) = ... else { return ... }`,
/// `policy_shape.rs` lines ~799/~845) go dead crate-wide: nothing else
/// calls either function directly on a descriptor whose origin fails to
/// resolve. `skeleton()` cannot revive them either — its own Gate 1 refuses
/// exactly that input before either function is ever called. This test
/// calls `fp_partition`/`key_partition` DIRECTLY (bypassing `skeleton()`)
/// on `common::dead_card_real_fingerprints_unresolved_origin()` — real
/// `Fingerprints` TLV data, unresolved origin — to pin the ORIGINAL claim
/// those branches' doc comment makes: a card carrying real key data still
/// collapses to the SAME empty output as a genuinely template-only card
/// when called this way, which is precisely why `skeleton()` must never be
/// allowed to reach this path with real key data present.
#[test]
fn expand_failure_collapses_real_key_data_to_the_same_empty_shape_as_no_keys() {
    let dead = common::dead_card_real_fingerprints_unresolved_origin();
    assert!(
        md_codec::canonicalize::expand_per_at_n(&dead).is_err(),
        "fixture sanity: origin must genuinely fail to resolve"
    );
    let shape = policy_shape(&dead);
    let dead_fp = fp_partition(&dead, &shape);
    let dead_kp = key_partition(&dead);
    assert!(
        dead_fp.iter().all(|p| p.is_empty()),
        "real fingerprints TLV data is present, but expand failure hides it: {dead_fp:?}"
    );
    assert!(dead_kp.is_empty());

    // Byte-identical to the template-only card sharing this tree.
    let template_only = common::kofn_recovery();
    assert_eq!(
        dead_fp,
        fp_partition(&template_only, &policy_shape(&template_only))
    );
    assert_eq!(dead_kp, key_partition(&template_only));
}

// ─────────────────────────────────────────────────────────────────────────
// Fix round 1, I-1 / I-2: two mutation-confirmed coverage gaps in
// `key_partition` (see task-3-report.md's "Fix round 1" appendix for the
// mutation each of these was proven against).
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn an_absent_xpub_is_its_own_singleton() {
    // md-codec's ABSENT-xpub sentinel is the same all-zero convention as the
    // fingerprint one; two absent-keyed slots are NOT known to be the same
    // key, and grouping them would assert an identity nobody measured — the
    // same argument `an_absent_fingerprint_is_its_own_singleton` pins for
    // `fp_partition`, now pinned for `key_partition`'s xpub half.
    let d = common::seated_pubkeys(&[
        (0, [0x00; 65]),
        (1, [0x00; 65]),
        (2, [0xaa; 65]),
        (3, [0xbb; 65]),
    ]);
    let kp = key_partition(&d);
    assert_eq!(
        kp,
        vec![vec![0u8], vec![1], vec![2], vec![3]],
        "absent xpubs never join, and neither do the two genuinely distinct \
         real ones: {kp:?}"
    );
}

#[test]
fn key_partition_does_not_group_same_xpub_at_different_origins() {
    // @0 and @3 carry the IDENTICAL xpub bytes, but @3's origin is
    // overridden away from the shared `m/48'` baseline every other fixture
    // in this file uses. `key_partition` groups on `(xpub, origin_path)`
    // per the brief — same bytes at a different origin is NOT the same key
    // record, and must not merge.
    let different_origin = OriginPath {
        components: vec![PathComponent {
            hardened: true,
            value: 99,
        }],
    };
    let d = common::seated_same_key_with_overrides(&[0, 3], &[(3, different_origin)]);
    let kp = key_partition(&d);
    assert!(
        !kp.contains(&vec![0u8, 3]),
        "same xpub bytes at different origins must not merge: {kp:?}"
    );
    assert!(kp.contains(&vec![0u8]), "@0 stays its own group: {kp:?}");
    assert!(kp.contains(&vec![3u8]), "@3 stays its own group: {kp:?}");
}

#[test]
fn key_partition_groups_same_xpub_at_the_same_origin() {
    // The positive control for the test above: identical construction path,
    // no origin override, so @0 and @3 share both xpub bytes AND origin —
    // they must merge.
    let d = common::seated_same_key_with_overrides(&[0, 3], &[]);
    let kp = key_partition(&d);
    assert!(
        kp.contains(&vec![0u8, 3]),
        "same xpub bytes at the same origin must merge: {kp:?}"
    );
}
