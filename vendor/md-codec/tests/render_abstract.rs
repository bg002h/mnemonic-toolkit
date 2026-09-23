//! Task 2: the abstracting render mode — `descriptor_to_abstract_template`
//! replaces lock values and hash digests with per-kind equality classes
//! (`kind#class`) so two policies differing only in which concrete lock
//! value or digest they use can be compared.

mod common;
use md_codec::render::descriptor_to_abstract_template;

#[test]
fn equal_lock_values_share_a_class_and_different_ones_do_not() {
    // wsh(or_i(and_v(v:pkh(@0),older(26280)),
    //          or_i(and_v(v:pkh(@1),older(1000)),
    //               and_v(v:pkh(@2),older(26280)))))
    let d = common::three_older_descriptor(26280, 1000, 26280);
    let t = descriptor_to_abstract_template(&d).unwrap();
    assert!(t.contains("older(older-blocks#1)"), "got {t}");
    assert!(t.contains("older(older-blocks#2)"), "got {t}");
    assert_eq!(
        t.matches("older-blocks#1").count(),
        2,
        "the two equal values share class 1: {t}"
    );
    assert!(
        !t.contains("26280"),
        "no literal lock value may survive: {t}"
    );
}

#[test]
fn after_locks_carry_height_and_time_bands() {
    // wsh(or_i(and_v(v:pkh(@0),after(499999999)),
    //          and_v(v:pkh(@1),after(500000000))))
    //
    // 499_999_999 and 500_000_000 straddle LOCKTIME_THRESHOLD by exactly
    // one, so this pins the height/time split at its boundary: a mutation
    // that moves the threshold (e.g. to 800_000_000) reclassifies
    // 500_000_000 from after-time into after-height, which this test
    // catches (nothing asserts on a literal `after(...)` today without it —
    // R0 fix round 1, I-1).
    let d = common::two_after_descriptor(499_999_999, 500_000_000);
    let t = descriptor_to_abstract_template(&d).unwrap();
    assert!(t.contains("after(after-height#1)"), "got {t}");
    assert!(t.contains("after(after-time#1)"), "got {t}");
    assert!(
        !t.contains("499999999") && !t.contains("500000000"),
        "no literal lock value may survive: {t}"
    );
}

#[test]
fn older_locks_carry_blocks_and_units_bands() {
    // wsh(or_i(and_v(v:pkh(@0),older(10)),and_v(v:pkh(@1),older((1<<22)|10))))
    //
    // Final whole-branch review, I-2: the `older-blocks`/`older-units` band
    // label was unguarded -- collapsing the two labels merges the
    // SkeletonKey of two wire-reachable, semantically different policies (10
    // blocks vs. 10 units of 512 seconds) into one, and 581 tests stayed
    // green. This is the `older` equivalent of
    // `after_locks_carry_height_and_time_bands` above: Task 2 guarded
    // `after`'s two bands and never added the `older` equivalent.
    let units = (1u32 << 22) | 10;
    let d = common::two_older_descriptor(10, units);
    let t = descriptor_to_abstract_template(&d).unwrap();
    assert!(t.contains("older(older-blocks#1)"), "got {t}");
    assert!(t.contains("older(older-units#1)"), "got {t}");
    assert!(
        !t.contains("older(10)") && !t.contains(&format!("older({units})")),
        "no literal lock value may survive: {t}"
    );
}

#[test]
fn digests_carry_a_class_too_symmetric_with_locks() {
    let d = common::two_sha256_descriptor([0x11; 32], [0x11; 32]);
    let t = descriptor_to_abstract_template(&d).unwrap();
    assert_eq!(
        t.matches("sha256(#1)").count(),
        2,
        "two branches committing to the SAME digest share a class: {t}"
    );
    assert!(!t.contains("1111"), "no literal digest may survive: {t}");
}

#[test]
fn the_use_site_is_kept() {
    let d = common::kofn_recovery_with_use_site();
    let t = descriptor_to_abstract_template(&d).unwrap();
    assert!(
        t.contains("/<0;1>/*"),
        "the key is looked up against evidence whose template field keeps it: {t}"
    );
}
