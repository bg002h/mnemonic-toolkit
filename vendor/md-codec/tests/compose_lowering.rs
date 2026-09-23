//! Lowering tests for `md_codec::compose` (SPEC_wallet_policy_composer.md §4, §5).
//!
//! Every expected template string below is the FIXED spelling the Go port must
//! reproduce byte for byte; a change here is a normative change.

use md_codec::canonicalize::canonicalize_placeholder_indices;
use md_codec::chunk::{reassemble, split};
use md_codec::compose::UnspendableKind;
use md_codec::compose::{
    ComposeError, Composed, Experimental, HashKind, HashLock, KeySet, Lock, MAX_PATHS, MAX_SLOTS,
    PathList, SlotOrigin, SpendPath, Wrapper, compose, compose_with, template_with_origins,
};
use md_codec::encode::{encode_md1_string, encode_payload};
use md_codec::origin_path::{OriginPath, PathComponent, PathDeclPaths};
use md_codec::render::descriptor_to_template;

const H1: HashLock = HashLock::new(HashKind::Sha256, [0xa8; 32]);

fn keys(k: u8, n: u8) -> SpendPath {
    SpendPath {
        keys: Some(KeySet { k, n, sorted: true }),
        hash: None,
        lock: None,
    }
}

fn unsorted(k: u8, n: u8) -> SpendPath {
    SpendPath {
        keys: Some(KeySet {
            k,
            n,
            sorted: false,
        }),
        hash: None,
        lock: None,
    }
}

fn with_lock(mut p: SpendPath, lock: Lock) -> SpendPath {
    p.lock = Some(lock);
    p
}

fn with_hash(mut p: SpendPath, h: HashLock) -> SpendPath {
    p.hash = Some(h);
    p
}

fn keyless(h: HashLock, lock: Option<Lock>) -> SpendPath {
    SpendPath {
        keys: None,
        hash: Some(h),
        lock,
    }
}

fn list(wrapper: Wrapper, paths: Vec<SpendPath>) -> PathList {
    PathList { wrapper, paths }
}

fn hardened(values: &[u32]) -> OriginPath {
    OriginPath {
        components: values
            .iter()
            .map(|v| PathComponent {
                hardened: true,
                value: *v,
            })
            .collect(),
    }
}

// ---- §4e structural refusals -------------------------------------------------

#[test]
fn compose_refuses_an_empty_path_list() {
    let err = compose(&list(Wrapper::Wsh, vec![]), UnspendableKind::Nums).unwrap_err();
    assert_eq!(err, ComposeError::NoPaths);
}

#[test]
fn compose_refuses_more_than_max_paths() {
    let paths: Vec<SpendPath> = (0..(MAX_PATHS + 1)).map(|_| keys(1, 1)).collect();
    let err = compose(&list(Wrapper::Wsh, paths), UnspendableKind::Nums).unwrap_err();
    assert_eq!(err, ComposeError::TooManyPaths { got: MAX_PATHS + 1 });
}

#[test]
fn compose_refuses_a_policy_with_no_keyed_path() {
    let err = compose(
        &list(Wrapper::Wsh, vec![keyless(H1, None)]),
        UnspendableKind::Nums,
    )
    .unwrap_err();
    assert_eq!(err, ComposeError::NoKeyedPath);
}

#[test]
fn compose_refuses_a_lock_only_path() {
    let lock_only = SpendPath {
        keys: None,
        hash: None,
        lock: Some(Lock::OlderBlocks(100)),
    };
    let err = compose(
        &list(Wrapper::Wsh, vec![keys(1, 1), lock_only]),
        UnspendableKind::Nums,
    )
    .unwrap_err();
    assert_eq!(err, ComposeError::LockOnlyPath { path: 1 });
}

#[test]
fn compose_refuses_a_keyless_path_under_tr() {
    let err = compose(
        &list(Wrapper::Tr, vec![keys(2, 3), keyless(H1, None)]),
        UnspendableKind::Nums,
    )
    .unwrap_err();
    assert_eq!(err, ComposeError::KeylessUnderTr { path: 1 });
}

#[test]
fn compose_refuses_bad_thresholds() {
    assert_eq!(
        compose(&list(Wrapper::Wsh, vec![keys(0, 2)]), UnspendableKind::Nums).unwrap_err(),
        ComposeError::BadThreshold {
            path: 0,
            k: 0,
            n: 2
        }
    );
    assert_eq!(
        compose(&list(Wrapper::Wsh, vec![keys(3, 2)]), UnspendableKind::Nums).unwrap_err(),
        ComposeError::BadThreshold {
            path: 0,
            k: 3,
            n: 2
        }
    );
    assert_eq!(
        compose(
            &list(Wrapper::Wsh, vec![keys(1, 10)]),
            UnspendableKind::Nums
        )
        .unwrap_err(),
        ComposeError::BadThreshold {
            path: 0,
            k: 1,
            n: 10
        }
    );
}

#[test]
fn compose_refuses_a_thirty_third_slot() {
    // 3 × 9 + 6 = 33 slots.
    let paths = vec![keys(9, 9), keys(9, 9), keys(9, 9), keys(6, 6)];
    let err = compose(&list(Wrapper::Wsh, paths), UnspendableKind::Nums).unwrap_err();
    assert_eq!(
        err,
        ComposeError::TooManySlots {
            got: 33,
            max: MAX_SLOTS
        }
    );
}

#[test]
fn compose_admits_exactly_thirty_two_slots() {
    // 3 × 9 + 5 = 32 slots. Passes only once the lowering exists (Task 2).
    let paths = vec![keys(9, 9), keys(9, 9), keys(9, 9), keys(5, 5)];
    assert!(compose(&list(Wrapper::Wsh, paths), UnspendableKind::Nums).is_ok());
}

#[test]
fn compose_refuses_legacy_wrappers_outside_the_single_sorted_multi_shape() {
    for w in [Wrapper::Sh, Wrapper::ShWsh] {
        assert_eq!(
            compose(&list(w, vec![keys(1, 1)]), UnspendableKind::Nums).unwrap_err(),
            ComposeError::LegacyWrapperShape
        );
        assert_eq!(
            compose(
                &list(w, vec![keys(2, 3), keys(1, 1)]),
                UnspendableKind::Nums
            )
            .unwrap_err(),
            ComposeError::LegacyWrapperShape
        );
        assert_eq!(
            compose(
                &list(w, vec![with_lock(keys(2, 3), Lock::OlderBlocks(10))]),
                UnspendableKind::Nums
            )
            .unwrap_err(),
            ComposeError::LegacyWrapperShape
        );
        assert_eq!(
            compose(&list(w, vec![unsorted(2, 3)]), UnspendableKind::Nums).unwrap_err(),
            ComposeError::LegacyWrapperShape
        );
    }
}

#[test]
fn compose_refuses_lock_operands_outside_the_consensus_bands() {
    // Each case pins the BAND NAMED in the refusal, not only that a refusal fired.
    let cases: &[(Lock, &str)] = &[
        (Lock::OlderBlocks(0), "older in blocks needs 1..=65535"),
        (
            Lock::OlderUnits(0),
            "older in 512-second units needs 1..=65535",
        ),
        (Lock::AfterHeight(0), "after height needs 1..=499999999"),
        (
            Lock::AfterHeight(500_000_000),
            "after height needs 1..=499999999",
        ),
        (
            Lock::AfterTime(499_999_999),
            "after time needs 500000000..=2147483647",
        ),
        (
            Lock::AfterTime(2_147_483_648),
            "after time needs 500000000..=2147483647",
        ),
    ];
    for (lock, why) in cases {
        let err = compose(
            &list(Wrapper::Wsh, vec![with_lock(keys(1, 1), *lock)]),
            UnspendableKind::Nums,
        )
        .unwrap_err();
        assert_eq!(
            err,
            ComposeError::LockOutOfRange { path: 0, why },
            "{lock:?}"
        );
    }
}

#[test]
fn lock_operand_bands_are_inclusive_at_both_ends() {
    // BIP-68 / BIP-65 / BIP-379 boundaries, straight from `Lock::operand`; no
    // lowering involved, so this passes from Task 1.
    use md_codec::tag::Tag;
    assert_eq!(Lock::OlderBlocks(1).operand(), Ok((Tag::Older, 1)));
    assert_eq!(Lock::OlderBlocks(65535).operand(), Ok((Tag::Older, 65535)));
    assert_eq!(Lock::OlderUnits(1).operand(), Ok((Tag::Older, 0x0040_0001)));
    assert_eq!(
        Lock::OlderUnits(65535).operand(),
        Ok((Tag::Older, 0x0040_ffff))
    );
    assert_eq!(Lock::AfterHeight(1).operand(), Ok((Tag::After, 1)));
    assert_eq!(
        Lock::AfterHeight(499_999_999).operand(),
        Ok((Tag::After, 499_999_999))
    );
    assert_eq!(
        Lock::AfterTime(500_000_000).operand(),
        Ok((Tag::After, 500_000_000))
    );
    assert_eq!(
        Lock::AfterTime(2_147_483_647).operand(),
        Ok((Tag::After, 2_147_483_647))
    );
    assert!(
        Lock::OlderUnits(0).operand().is_err(),
        "0x400000 alone is a lock of ZERO units, i.e. none (filed md-older-zero-time-units-not-refused)"
    );
}

// ---- §5 wsh lowering, by rendered text -----------------------------------------

fn text(list: &PathList) -> String {
    descriptor_to_template(&compose(list, UnspendableKind::Nums).unwrap().descriptor).unwrap()
}

/// Every slot's origin, in slot order, read from `path_decl` (the rendered
/// text never carries origins: descriptor-mnemonic F-219).
fn origins(c: &Composed) -> Vec<OriginPath> {
    match &c.descriptor.path_decl.paths {
        PathDeclPaths::Shared(o) => vec![o.clone(); c.descriptor.n as usize],
        PathDeclPaths::Divergent(v) => v.clone(),
    }
}

#[test]
fn unseated_slots_take_ascending_default_accounts_under_the_wrapper_script_type() {
    let c = compose(&list(Wrapper::Wsh, vec![keys(2, 3)]), UnspendableKind::Nums).unwrap();
    assert_eq!(
        origins(&c),
        vec![
            hardened(&[48, 0, 0, 2]),
            hardened(&[48, 0, 1, 2]),
            hardened(&[48, 0, 2, 2])
        ]
    );
    assert_eq!(
        template_with_origins(&c).unwrap(),
        "wsh(sortedmulti(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*))"
    );
    // One slot: a shared declaration, not a one-element divergent list.
    let c = compose(&list(Wrapper::Wsh, vec![keys(1, 1)]), UnspendableKind::Nums).unwrap();
    assert!(matches!(
        c.descriptor.path_decl.paths,
        PathDeclPaths::Shared(_)
    ));
    assert_eq!(origins(&c), vec![hardened(&[48, 0, 0, 2])]);
    // Script types: sh(wsh) is 1', sh is 2', tr is 3'.
    let c = compose(
        &list(Wrapper::ShWsh, vec![keys(2, 2)]),
        UnspendableKind::Nums,
    )
    .unwrap();
    assert_eq!(
        origins(&c),
        vec![hardened(&[48, 0, 0, 1]), hardened(&[48, 0, 1, 1])]
    );
    let c = compose(&list(Wrapper::Sh, vec![keys(2, 2)]), UnspendableKind::Nums).unwrap();
    assert_eq!(
        origins(&c),
        vec![hardened(&[48, 0, 0, 2]), hardened(&[48, 0, 1, 2])]
    );
    // tr's 3' is asserted in Task 3, once the taproot lowering exists.
}

#[test]
fn template_with_origins_inlines_two_digit_slots_without_touching_their_prefixes() {
    // Hand-written, NOT printer-generated: `@1/` must not be rewritten inside
    // `@10/` or `@11/`. Twelve slots: a 9-of-9 head and a 3-of-3 tail.
    let c = compose(
        &list(Wrapper::Wsh, vec![keys(9, 9), keys(3, 3)]),
        UnspendableKind::Nums,
    )
    .unwrap();
    assert_eq!(
        template_with_origins(&c).unwrap(),
        "wsh(or_d(multi(9,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*,@3/48'/0'/3'/2'/<0;1>/*,@4/48'/0'/4'/2'/<0;1>/*,@5/48'/0'/5'/2'/<0;1>/*,@6/48'/0'/6'/2'/<0;1>/*,@7/48'/0'/7'/2'/<0;1>/*,@8/48'/0'/8'/2'/<0;1>/*),multi(3,@9/48'/0'/9'/2'/<0;1>/*,@10/48'/0'/10'/2'/<0;1>/*,@11/48'/0'/11'/2'/<0;1>/*)))"
    );
}

#[test]
fn sole_unlocked_multi_path_under_wsh_is_sortedmulti() {
    assert_eq!(
        text(&list(Wrapper::Wsh, vec![keys(2, 3)])),
        "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))"
    );
}

#[test]
fn sole_unsorted_multi_path_under_wsh_is_multi_and_experimental() {
    let c = compose(
        &list(Wrapper::Wsh, vec![unsorted(2, 3)]),
        UnspendableKind::Nums,
    )
    .unwrap();
    assert_eq!(
        descriptor_to_template(&c.descriptor).unwrap(),
        "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))"
    );
    assert_eq!(c.experimental, vec![Experimental::UnsortedKeys(0)]);
}

#[test]
fn single_key_under_wsh_is_pkh() {
    assert_eq!(
        text(&list(Wrapper::Wsh, vec![keys(1, 1)])),
        "wsh(pkh(@0/<0;1>/*))"
    );
}

#[test]
fn a_locked_multi_path_is_unsorted_multi_without_the_experimental_mark() {
    // Sorted forms cannot nest inside a fragment (BIP-383/388; md refuses), so
    // the lowering forces `multi` and does NOT report it as chosen-unsorted.
    let c = compose(
        &list(
            Wrapper::Wsh,
            vec![with_lock(keys(2, 3), Lock::OlderBlocks(26280)), keys(1, 1)],
        ),
        UnspendableKind::Nums,
    )
    .unwrap();
    assert_eq!(
        descriptor_to_template(&c.descriptor).unwrap(),
        "wsh(or_i(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),older(26280)),pkh(@3/<0;1>/*)))"
    );
    assert!(c.experimental.is_empty());
}

#[test]
fn two_path_wsh_with_a_bare_multi_head_uses_or_d() {
    // The reference two-path wallet, wsh form (spec §5, C21/C23).
    let l = list(
        Wrapper::Wsh,
        vec![keys(2, 3), with_lock(keys(1, 1), Lock::OlderBlocks(26280))],
    );
    assert_eq!(
        text(&l),
        "wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pkh(@3/<0;1>/*),older(26280))))"
    );
}

#[test]
fn a_single_key_head_uses_or_i_not_or_d() {
    // I1/C21: or_d(pkh(P1), R) is dominated and publishes P1's key.
    let l = list(
        Wrapper::Wsh,
        vec![keys(1, 1), with_lock(keys(1, 1), Lock::OlderBlocks(100))],
    );
    assert_eq!(
        text(&l),
        "wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(100))))"
    );
}

#[test]
fn conjunct_order_is_keys_hash_lock() {
    let p = with_lock(with_hash(keys(2, 3), H1), Lock::AfterHeight(1_000_000));
    let l = list(Wrapper::Wsh, vec![p, keys(1, 1)]);
    let h = "a8".repeat(32);
    assert_eq!(
        text(&l),
        format!(
            "wsh(or_i(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:sha256({h}),after(1000000))),pkh(@3/<0;1>/*)))"
        )
    );
}

#[test]
fn a_keyless_wsh_path_is_admitted_and_marked_experimental() {
    let l = list(
        Wrapper::Wsh,
        vec![keys(2, 3), keyless(H1, Some(Lock::AfterHeight(1_383_520)))],
    );
    let c = compose(&l, UnspendableKind::Nums).unwrap();
    let h = "a8".repeat(32);
    assert_eq!(
        descriptor_to_template(&c.descriptor).unwrap(),
        format!(
            "wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:sha256({h}),after(1383520))))"
        )
    );
    assert_eq!(c.experimental, vec![Experimental::KeylessPath(1)]);
}

#[test]
fn eight_paths_chain_right_associatively_and_the_last_stands_alone() {
    let paths: Vec<SpendPath> = (0..8)
        .map(|i| with_lock(keys(1, 1), Lock::OlderBlocks(100 + i)))
        .collect();
    let t = text(&list(Wrapper::Wsh, paths));
    assert_eq!(t.matches("or_i(").count(), 7, "{t}");
    assert!(t.ends_with(",older(107))))))))))"), "{t}");
}

#[test]
fn legacy_wrappers_wrap_the_single_sorted_multi() {
    assert_eq!(
        text(&list(Wrapper::ShWsh, vec![keys(2, 3)])),
        "sh(wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*)))"
    );
    assert_eq!(
        text(&list(Wrapper::Sh, vec![keys(2, 3)])),
        "sh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))"
    );
}

#[test]
fn a_time_lock_of_one_unit_encodes_as_0x400001() {
    let c = compose(
        &list(
            Wrapper::Wsh,
            vec![with_lock(keys(1, 1), Lock::OlderUnits(1))],
        ),
        UnspendableKind::Nums,
    )
    .unwrap();
    let text = descriptor_to_template(&c.descriptor).unwrap();
    assert!(text.contains("older(4194305)"), "{text}");
}

#[test]
fn slots_are_numbered_by_first_appearance_and_canonicalisation_is_identity() {
    let l = list(
        Wrapper::Wsh,
        vec![keys(2, 3), with_lock(keys(1, 1), Lock::OlderBlocks(26280))],
    );
    let c = compose(&l, UnspendableKind::Nums).unwrap();
    let indices: Vec<u8> = c.slots.iter().map(|s| s.index).collect();
    assert_eq!(indices, vec![0, 1, 2, 3]);
    assert_eq!(c.slots[3].path, 1);
    let mut d = c.descriptor.clone();
    canonicalize_placeholder_indices(&mut d).unwrap();
    assert_eq!(
        d, c.descriptor,
        "compose must emit canonical numbering itself"
    );
}

#[test]
fn composed_templates_encode_and_round_trip_through_the_wire() {
    let l = list(
        Wrapper::Wsh,
        vec![keys(2, 3), with_lock(keys(1, 1), Lock::OlderBlocks(26280))],
    );
    let c = compose(&l, UnspendableKind::Nums).unwrap();
    let (_bytes, bits) = encode_payload(&c.descriptor).unwrap();
    assert!(bits > 0);
    let chunks = split(&c.descriptor).unwrap();
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    let back = reassemble(&refs).unwrap();
    assert_eq!(back, c.descriptor);
    // Not `if let Ok`: a guard around an assertion is a PASS that stops running
    // the day the shape outgrows one string (exec review M-2).
    let s = encode_md1_string(&c.descriptor).expect("a two-path wsh fits one md1 string");
    assert!(s.starts_with("md1"));
}

// ---- §4f declared origins and the invariant (wsh half) --------------------------

#[test]
fn compose_with_refuses_two_slots_at_one_origin_unless_both_fingerprints_differ() {
    let l = list(Wrapper::Wsh, vec![keys(2, 2)]);
    let same = hardened(&[48, 0, 0, 2]);
    // Neither fingerprinted: refused.
    let d = vec![
        Some(SlotOrigin {
            origin: same.clone(),
            fingerprint: None,
        }),
        Some(SlotOrigin {
            origin: same.clone(),
            fingerprint: None,
        }),
    ];
    assert_eq!(
        compose_with(&l, &d, UnspendableKind::Nums).unwrap_err(),
        ComposeError::IndistinguishableSlots { a: 0, b: 1 }
    );
    // One fingerprinted: still refused (the one-card-fills-two-slots case).
    let d = vec![
        Some(SlotOrigin {
            origin: same.clone(),
            fingerprint: Some([9, 9, 9, 9]),
        }),
        Some(SlotOrigin {
            origin: same.clone(),
            fingerprint: None,
        }),
    ];
    assert_eq!(
        compose_with(&l, &d, UnspendableKind::Nums).unwrap_err(),
        ComposeError::IndistinguishableSlots { a: 0, b: 1 }
    );
    // Both fingerprinted and distinct: admitted, as a shared origin.
    let d = vec![
        Some(SlotOrigin {
            origin: same.clone(),
            fingerprint: Some([9, 9, 9, 9]),
        }),
        Some(SlotOrigin {
            origin: same,
            fingerprint: Some([8, 8, 8, 8]),
        }),
    ];
    assert!(compose_with(&l, &d, UnspendableKind::Nums).is_ok());
}

#[test]
fn compose_with_refuses_a_declaration_slice_of_the_wrong_length() {
    let l = list(Wrapper::Wsh, vec![keys(2, 2)]);
    assert_eq!(
        compose_with(&l, &[None], UnspendableKind::Nums).unwrap_err(),
        ComposeError::WrongSlotCount { got: 1, want: 2 }
    );
}

// ---- §5 taproot lowering ---------------------------------------------------------

const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

#[test]
fn two_path_taproot_with_no_single_key_uses_nums_and_two_leaves() {
    // The reference two-path wallet, tr form (brainstorm §3.4), all slots unseated.
    // With two leaves the unlocked multi is NOT sole, so it is multi_a.
    let l = list(
        Wrapper::Tr,
        vec![keys(2, 3), with_lock(keys(1, 1), Lock::OlderBlocks(26280))],
    );
    let c = compose(&l, UnspendableKind::Nums).unwrap();
    assert_eq!(
        descriptor_to_template(&c.descriptor).unwrap(),
        format!(
            "tr({NUMS},{{multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(26280))}})"
        )
    );
    assert_eq!(c.internal_key_path, None);
}

#[test]
fn the_unlocked_single_key_becomes_the_internal_key_and_slot_zero() {
    // Path 1: 2-of-2 locked; path 2: single unlocked key; path 3: single locked key.
    let l = list(
        Wrapper::Tr,
        vec![
            with_lock(keys(2, 2), Lock::OlderBlocks(100)),
            keys(1, 1),
            with_lock(keys(1, 1), Lock::AfterHeight(900_000)),
        ],
    );
    let c = compose(&l, UnspendableKind::Nums).unwrap();
    assert_eq!(c.internal_key_path, Some(1));
    assert_eq!(c.slots[0].path, 1, "the extracted key is @0");
    assert_eq!(
        descriptor_to_template(&c.descriptor).unwrap(),
        "tr(@0/<0;1>/*,{and_v(v:multi_a(2,@1/<0;1>/*,@2/<0;1>/*),older(100)),and_v(v:pk(@3/<0;1>/*),after(900000))})"
    );
    let mut d = c.descriptor.clone();
    canonicalize_placeholder_indices(&mut d).unwrap();
    assert_eq!(d, c.descriptor);
}

#[test]
fn a_single_remaining_leaf_is_written_bare() {
    let l = list(
        Wrapper::Tr,
        vec![keys(1, 1), with_lock(keys(1, 1), Lock::OlderBlocks(65535))],
    );
    assert_eq!(
        text(&l),
        "tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),older(65535)))"
    );
}

#[test]
fn a_lone_single_key_is_a_key_path_only_tr() {
    assert_eq!(text(&list(Wrapper::Tr, vec![keys(1, 1)])), "tr(@0/<0;1>/*)");
}

#[test]
fn a_sole_unlocked_multi_leaf_is_sortedmulti_a() {
    let l = list(Wrapper::Tr, vec![keys(2, 3)]);
    assert_eq!(
        text(&l),
        format!("tr({NUMS},sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))")
    );
}

#[test]
fn four_leaves_form_a_right_spine() {
    let paths: Vec<SpendPath> = (0..4)
        .map(|i| with_lock(keys(1, 1), Lock::OlderBlocks(10 + i)))
        .collect();
    let t = text(&list(Wrapper::Tr, paths));
    // {P1,{P2,{P3,P4}}}: three opening braces, and the deepest pair is P3,P4.
    assert_eq!(t.matches('{').count(), 3, "{t}");
    assert!(
        t.contains("older(12)),and_v(v:pk(@3/<0;1>/*),older(13))}}})"),
        "{t}"
    );
}

#[test]
fn only_the_first_listed_unlocked_single_key_is_extracted() {
    // Two unlocked single keys: the first is the internal key, the second stays a leaf.
    let l = list(Wrapper::Tr, vec![keys(2, 2), keys(1, 1), keys(1, 1)]);
    let c = compose(&l, UnspendableKind::Nums).unwrap();
    assert_eq!(c.internal_key_path, Some(1));
    assert_eq!(
        descriptor_to_template(&c.descriptor).unwrap(),
        "tr(@0/<0;1>/*,{multi_a(2,@1/<0;1>/*,@2/<0;1>/*),pk(@3/<0;1>/*)})"
    );
}

#[test]
fn taproot_templates_round_trip_through_the_wire() {
    let l = list(
        Wrapper::Tr,
        vec![keys(2, 3), with_lock(keys(1, 1), Lock::OlderBlocks(26280))],
    );
    let c = compose(&l, UnspendableKind::Nums).unwrap();
    let chunks = split(&c.descriptor).unwrap();
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    assert_eq!(reassemble(&refs).unwrap(), c.descriptor);
}

#[test]
fn tr_default_origins_use_script_type_three() {
    let c = compose(&list(Wrapper::Tr, vec![keys(2, 2)]), UnspendableKind::Nums).unwrap();
    assert_eq!(
        origins(&c),
        vec![hardened(&[48, 0, 0, 3]), hardened(&[48, 0, 1, 3])]
    );
}

#[test]
fn compose_with_uses_declared_origins_and_fills_unseated_slots_with_the_lowest_free_account() {
    let l = list(
        Wrapper::Tr,
        vec![keys(2, 2), with_lock(keys(1, 1), Lock::OlderBlocks(100))],
    );
    let fp_a = [0x73, 0xc5, 0xda, 0x0a];
    // Slot @0 seated at account 1, slot @2 seated at account 0; slot @1 unseated.
    let declared = vec![
        Some(SlotOrigin {
            origin: hardened(&[48, 0, 1, 3]),
            fingerprint: Some(fp_a),
        }),
        None,
        Some(SlotOrigin {
            origin: hardened(&[48, 0, 0, 3]),
            fingerprint: Some([1, 2, 3, 4]),
        }),
    ];
    let c = compose_with(&l, &declared, UnspendableKind::Nums).unwrap();
    // Accounts 0 and 1 are taken, so the unseated slot @1 gets account 2.
    assert_eq!(
        origins(&c),
        vec![
            hardened(&[48, 0, 1, 3]),
            hardened(&[48, 0, 2, 3]),
            hardened(&[48, 0, 0, 3])
        ]
    );
    assert_eq!(
        c.descriptor.tlv.fingerprints,
        Some(vec![(0, fp_a), (2, [1, 2, 3, 4])])
    );
    // No path is an unlocked single key (the 1-of-1 is locked), so NUMS and two leaves.
    assert_eq!(
        template_with_origins(&c).unwrap(),
        format!(
            "tr({NUMS},{{multi_a(2,@0/48'/0'/1'/3'/<0;1>/*,@1/48'/0'/2'/3'/<0;1>/*),and_v(v:pk(@2/48'/0'/0'/3'/<0;1>/*),older(100))}})"
        )
    );
}

// ---- §4d presets --------------------------------------------------------------------

use md_codec::compose::presets;

#[test]
fn presets_compose_and_carry_the_documented_shapes() {
    let p = presets::plain_multisig(Wrapper::Wsh, 2, 3).unwrap();
    assert_eq!(text(&p), text(&list(Wrapper::Wsh, vec![keys(2, 3)])));

    let p = presets::simple_timelocked_inheritance(Wrapper::Wsh, 65535).unwrap();
    assert_eq!(p.paths.len(), 2);
    assert_eq!(p.paths[1].lock, Some(Lock::OlderBlocks(65535)));

    let p = presets::kofn_recovery(Wrapper::Tr, 2, 3, 52560).unwrap();
    assert_eq!(
        p.paths[0].keys,
        Some(KeySet {
            k: 2,
            n: 3,
            sorted: true
        })
    );
    assert_eq!(p.paths[1].lock, Some(Lock::OlderBlocks(52560)));

    let p = presets::tiered_recovery(Wrapper::Wsh, 2, 2, 2, 3, 4032).unwrap();
    assert_eq!(p.paths.len(), 2);

    let p = presets::hashlock_gated(Wrapper::Wsh, H1, 144).unwrap();
    assert!(p.paths[0].hash.is_some());
    assert_eq!(p.paths[1].lock, Some(Lock::OlderBlocks(144)));

    let p = presets::decaying_multisig(Wrapper::Wsh, 2, 3, 1, 2, 1000, 2000, 4_000_000).unwrap();
    assert_eq!(p.paths.len(), 3);
    assert_eq!(
        p.paths[1].keys,
        Some(KeySet {
            k: 1,
            n: 2,
            sorted: true
        }),
        "the recovery quorum is no harder than the primary: that is the decay"
    );
    // Same threshold over MORE keys is admitted: it is no harder to satisfy.
    assert!(presets::decaying_multisig(Wrapper::Wsh, 2, 3, 2, 5, 1000, 2000, 4_000_000).is_ok());
    assert_eq!(p.paths[2].lock, Some(Lock::AfterHeight(4_000_000)));
    for l in [
        presets::plain_multisig(Wrapper::Wsh, 2, 3).unwrap(),
        presets::simple_timelocked_inheritance(Wrapper::Wsh, 65535).unwrap(),
        presets::kofn_recovery(Wrapper::Tr, 2, 3, 52560).unwrap(),
        presets::tiered_recovery(Wrapper::Wsh, 2, 2, 2, 3, 4032).unwrap(),
        presets::hashlock_gated(Wrapper::Wsh, H1, 144).unwrap(),
        presets::decaying_multisig(Wrapper::Wsh, 2, 3, 1, 2, 1000, 2000, 4_000_000).unwrap(),
    ] {
        compose(&l, UnspendableKind::Nums).unwrap_or_else(|e| panic!("{l:?}: {e}"));
    }
}

#[test]
fn presets_lower_to_their_pinned_templates() {
    // Spec §10 item 3: "the five presets as Concrete policies + expected
    // templates". The Concrete-policy half is the §5b cross-check in
    // `compose_crosscheck.rs`; this is the expected-template half, pinned as
    // literals so a preset that drifts in SHAPE (which tier carries the lock,
    // which quorum is smaller) fails here.
    let h = "a8".repeat(32);
    let cases: Vec<(&str, PathList, String)> = vec![
        ("plain_multisig", presets::plain_multisig(Wrapper::Wsh, 2, 3).unwrap(),
         "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))".to_string()),
        ("simple_timelocked_inheritance", presets::simple_timelocked_inheritance(Wrapper::Wsh, 65535).unwrap(),
         "wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(65535))))".to_string()),
        ("kofn_recovery", presets::kofn_recovery(Wrapper::Tr, 2, 3, 52560).unwrap(),
         format!("tr({NUMS},{{multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(52560))}})")),
        ("tiered_recovery", presets::tiered_recovery(Wrapper::Wsh, 2, 2, 2, 3, 4032).unwrap(),
         "wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*),and_v(v:multi(2,@2/<0;1>/*,@3/<0;1>/*,@4/<0;1>/*),older(4032))))".to_string()),
        ("hashlock_gated", presets::hashlock_gated(Wrapper::Wsh, H1, 144).unwrap(),
         format!("wsh(or_i(and_v(v:pkh(@0/<0;1>/*),sha256({h})),and_v(v:pkh(@1/<0;1>/*),older(144))))")),
        ("decaying_multisig", presets::decaying_multisig(Wrapper::Wsh, 2, 3, 1, 2, 1000, 2000, 4_000_000).unwrap(),
         "wsh(or_i(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),older(1000)),or_i(and_v(v:multi(1,@3/<0;1>/*,@4/<0;1>/*),older(2000)),and_v(v:pkh(@5/<0;1>/*),after(4000000)))))".to_string()),
    ];
    for (name, list, expected) in cases {
        assert_eq!(text(&list), expected, "{name}");
    }
}

#[test]
fn presets_refuse_parameters_the_grammar_refuses() {
    assert!(matches!(
        presets::plain_multisig(Wrapper::Wsh, 3, 2),
        Err(ComposeError::BadThreshold { .. })
    ));
    assert!(matches!(
        presets::simple_timelocked_inheritance(Wrapper::Wsh, 0),
        Err(ComposeError::LockOutOfRange { path: 1, .. })
    ));
    assert!(matches!(
        presets::kofn_recovery(Wrapper::Wsh, 2, 3, 70_000),
        Err(ComposeError::LockOutOfRange { path: 1, .. })
    ));
    // The refusal names the tier that carries the bad lock, not tier 1.
    assert_eq!(
        presets::tiered_recovery(Wrapper::Wsh, 2, 2, 2, 3, 70_000)
            .unwrap_err()
            .to_string(),
        "path 2: older in blocks needs 1..=65535"
    );
    // Decay must be a decay: later tiers unlock LATER, and the recovery quorum is not larger.
    assert!(matches!(
        presets::decaying_multisig(Wrapper::Wsh, 2, 3, 1, 2, 2000, 1000, 4_000_000),
        Err(ComposeError::PresetShape { .. })
    ));
    assert!(matches!(
        presets::decaying_multisig(Wrapper::Wsh, 1, 2, 2, 3, 1000, 2000, 4_000_000),
        Err(ComposeError::PresetShape { .. })
    ));
}

/// F-449 stage 2 Task 2 Step 4 (SPEC §6 row 3): the codec SIGNALS a `liana`
/// request it could not honour because a real internal key was extracted.
/// Never set for the default, never set when Liana's key was composed.
#[test]
fn unspendable_request_unmet_is_set_exactly_when_liana_meets_a_real_key() {
    use md_codec::tree::{Body, InternalKey};
    let real = list(
        Wrapper::Tr,
        vec![keys(1, 1), with_lock(keys(1, 1), Lock::OlderBlocks(100))],
    );
    let unspendable = list(
        Wrapper::Tr,
        vec![keys(2, 3), with_lock(keys(1, 1), Lock::OlderBlocks(100))],
    );
    let c = compose(&real, UnspendableKind::Liana).unwrap();
    assert!(
        c.unspendable_request_unmet,
        "liana over a real key must signal"
    );
    assert_eq!(c.internal_key_path, Some(0));
    assert!(
        !compose(&real, UnspendableKind::Nums)
            .unwrap()
            .unspendable_request_unmet
    );
    let c = compose(&unspendable, UnspendableKind::Liana).unwrap();
    assert!(!c.unspendable_request_unmet, "liana was composed: {c:?}");
    assert!(matches!(
        c.descriptor.tree.body,
        Body::Tr {
            internal_key: InternalKey::LianaUnspendable,
            ..
        }
    ));
    assert!(
        !compose(&unspendable, UnspendableKind::Nums)
            .unwrap()
            .unspendable_request_unmet
    );
}
