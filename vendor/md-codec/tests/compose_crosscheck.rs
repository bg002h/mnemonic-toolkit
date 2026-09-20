//! Spec §5b: every composed template converts to a rust-miniscript descriptor,
//! passes `sanity_check` (a keyless-wsh path fails it with exactly the
//! signature rule, and parses under `top_unsafe`), derives an address, and —
//! where every path has a key — lifts to the same semantic policy as the
//! COMPILER's output for the same spend conditions. `cross_check` is the whole
//! contract for one list; Task 5 runs it over the family, Task 7 over presets.

use std::str::FromStr;

#[path = "compose_support.rs"]
mod support;
use support::*;

use md_codec::compose::{KeySet, Lock, PathList, SpendPath, Wrapper, compose};
use md_codec::render::descriptor_to_template;
use md_codec::to_miniscript::to_miniscript_descriptor;
use miniscript::bitcoin::Network;
use miniscript::descriptor::DescriptorPublicKey;
use miniscript::policy::{Concrete, Liftable};
use miniscript::{Descriptor, ExtParams, Legacy, Miniscript, Segwitv0};

fn keys(k: u8, n: u8) -> SpendPath {
    SpendPath {
        keys: Some(KeySet { k, n, sorted: true }),
        hash: None,
        lock: None,
    }
}

fn locked(k: u8, n: u8, lock: Lock) -> SpendPath {
    SpendPath {
        keys: Some(KeySet { k, n, sorted: true }),
        hash: None,
        lock: Some(lock),
    }
}

fn two_path(wrapper: Wrapper) -> PathList {
    PathList {
        wrapper,
        paths: vec![keys(2, 3), locked(1, 1, Lock::OlderBlocks(26280))],
    }
}

/// The §5b legs for one list. Returns the index-0 address (empty for a
/// keyless-wsh list, whose leg is the documented sanity FAILURE).
pub fn cross_check(name: &str, list: &PathList, keyless_wsh: bool) -> String {
    let d = keyed(list);
    let c = compose(list).unwrap_or_else(|e| panic!("{name}: {e}"));
    let conv = to_miniscript_descriptor(&d, 0).unwrap_or_else(|e| panic!("{name}: convert: {e}"));
    if keyless_wsh {
        let e = conv
            .sanity_check()
            .expect_err("a signature-free path must fail the default sanity check");
        assert!(e.to_string().contains("require a signature"), "{name}: {e}");
        return String::new();
    }
    conv.sanity_check()
        .unwrap_or_else(|e| panic!("{name}: sanity: {e}"));
    let addr = conv
        .derive_at_index(0)
        .unwrap_or_else(|e| panic!("{name}: derive: {e}"))
        .address(Network::Bitcoin)
        .unwrap_or_else(|e| panic!("{name}: address: {e}"))
        .to_string();
    // The converter's own key strings, in traversal (= emitted) order, minus
    // the NUMS internal key; the compiler gets the SAME key values, so the
    // lifted policies differ only if the spend conditions do.
    let key_strings: Vec<String> = conv
        .iter_pk()
        .map(|k| k.to_string())
        .filter(|k| !k.starts_with(NUMS))
        .collect();
    assert_eq!(key_strings.len(), c.slots.len(), "{name}: one key per slot");
    let policy = concrete_policy(list, &c, &key_strings);
    let concrete = Concrete::<DescriptorPublicKey>::from_str(&policy)
        .unwrap_or_else(|e| panic!("{name}: {policy}: {e}"));
    let theirs: Descriptor<DescriptorPublicKey> = match list.wrapper {
        Wrapper::Wsh => Descriptor::new_wsh(concrete.compile::<Segwitv0>().unwrap()).unwrap(),
        Wrapper::ShWsh => Descriptor::new_sh_wsh(concrete.compile::<Segwitv0>().unwrap()).unwrap(),
        Wrapper::Sh => Descriptor::new_sh(concrete.compile::<Legacy>().unwrap()).unwrap(),
        Wrapper::Tr => {
            // Same internal-key decision as the lowering: NUMS when no path is
            // an unlocked single key, else let the compiler extract one (the
            // lifted OR is the same set whichever key sits on the key path).
            let nums = c
                .internal_key_path
                .is_none()
                .then(|| DescriptorPublicKey::from_str(NUMS).unwrap());
            concrete
                .compile_tr(nums)
                .unwrap_or_else(|e| panic!("{name}: compile_tr: {e}"))
        }
    };
    let ours = conv.lift().unwrap().normalized().sorted();
    let theirs = theirs.lift().unwrap().normalized().sorted();
    assert_eq!(
        ours, theirs,
        "{name}: same spend conditions, whatever the fragments"
    );
    addr
}

#[test]
fn the_reference_two_path_wallets_pass_the_cross_check() {
    let wsh = cross_check("two_path_wsh", &two_path(Wrapper::Wsh), false);
    assert!(wsh.starts_with("bc1q"), "{wsh}");
    let tr = cross_check("two_path_tr", &two_path(Wrapper::Tr), false);
    assert!(tr.starts_with("bc1p"), "{tr}");
}

#[test]
fn the_cross_check_notices_a_wrong_lowering() {
    // Mutation in the TEST, not the code: hand the compiler a DIFFERENT policy
    // (threshold 1 instead of 2) and the lift equality must fail — a check that
    // can fail is the only kind worth running over the family.
    let list = two_path(Wrapper::Wsh);
    let d = keyed(&list);
    let c = compose(&list).unwrap();
    let conv = to_miniscript_descriptor(&d, 0).unwrap();
    let key_strings: Vec<String> = conv.iter_pk().map(|k| k.to_string()).collect();
    let wrong = concrete_policy(&list, &c, &key_strings).replacen("thresh(2,", "thresh(1,", 1);
    let concrete = Concrete::<DescriptorPublicKey>::from_str(&wrong).unwrap();
    let theirs = Descriptor::new_wsh(concrete.compile::<Segwitv0>().unwrap())
        .unwrap()
        .lift()
        .unwrap()
        .normalized()
        .sorted();
    let ours = conv.lift().unwrap().normalized().sorted();
    assert_ne!(ours, theirs, "the lift comparison must be able to fail");
}

#[test]
fn a_keyless_wsh_path_is_admitted_with_top_unsafe_and_refused_by_the_default_sanity() {
    let list = PathList {
        wrapper: Wrapper::Wsh,
        paths: vec![
            keys(2, 3),
            SpendPath {
                keys: None,
                hash: Some(H),
                lock: Some(Lock::AfterHeight(1_383_520)),
            },
        ],
    };
    assert_eq!(cross_check("keyless_wsh", &list, true), "");
    // The inner miniscript the device would emit, parsed two ways.
    let d = keyed(&list);
    let text = descriptor_to_template(&d).unwrap();
    // Strip exactly ONE `wsh(` and ONE `)`: `trim_end_matches(')')` would eat
    // every closing paren of the inner script.
    let mut inner = text
        .strip_prefix("wsh(")
        .and_then(|t| t.strip_suffix(')'))
        .expect("a wsh template")
        .to_string();
    for (i, xpub) in XPUB.iter().enumerate().take(3) {
        inner = inner.replace(
            &format!("@{i}/<0;1>/*"),
            &format!("[73c5da0a/48'/0'/{i}'/2']{xpub}/<0;1>/*"),
        );
    }
    let sane = Miniscript::<DescriptorPublicKey, Segwitv0>::from_str(&inner);
    assert!(
        sane.is_err(),
        "the default parse must refuse a sigless spend path"
    );
    let insane = Miniscript::<DescriptorPublicKey, Segwitv0>::from_str_ext(
        &inner,
        &ExtParams::new().top_unsafe(),
    )
    .expect("top_unsafe admits the keyless path and nothing else");
    assert!(insane.lift().is_ok());
}

#[test]
fn every_family_entry_passes_the_5b_cross_check() {
    for (name, list, _, tags) in family() {
        let keyless = tags.contains(&"keyless-wsh");
        let addr = cross_check(name, &list, keyless);
        if !keyless {
            assert!(
                addr.starts_with("bc1") || addr.starts_with('3'),
                "{name}: {addr}"
            );
        }
    }
}

#[test]
fn every_preset_passes_the_5b_cross_check() {
    use md_codec::compose::presets;
    let cases: Vec<(&str, PathList)> = vec![
        (
            "plain_multisig",
            presets::plain_multisig(Wrapper::Wsh, 2, 3).unwrap(),
        ),
        (
            "simple_timelocked_inheritance",
            presets::simple_timelocked_inheritance(Wrapper::Wsh, 65535).unwrap(),
        ),
        (
            "kofn_recovery_tr",
            presets::kofn_recovery(Wrapper::Tr, 2, 3, 52560).unwrap(),
        ),
        (
            "kofn_recovery_wsh",
            presets::kofn_recovery(Wrapper::Wsh, 2, 3, 52560).unwrap(),
        ),
        (
            "tiered_recovery",
            presets::tiered_recovery(Wrapper::Wsh, 2, 2, 2, 3, 4032).unwrap(),
        ),
        (
            "hashlock_gated",
            presets::hashlock_gated(Wrapper::Wsh, H, 144).unwrap(),
        ),
        (
            "decaying_multisig",
            presets::decaying_multisig(Wrapper::Wsh, 2, 3, 1, 2, 1000, 2000, 4_000_000).unwrap(),
        ),
    ];
    for (name, list) in cases {
        let addr = cross_check(name, &list, false);
        assert!(addr.starts_with("bc1"), "{name}: {addr}");
    }
}
