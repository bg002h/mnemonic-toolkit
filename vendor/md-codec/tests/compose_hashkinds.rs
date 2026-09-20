//! Phase 1 of SPEC_hashlock_kinds: composing a hashlock of any of the four
//! miniscript hash kinds, not just `sha256`.
//!
//! WHY THIS FILE EXISTS. `md-codec` could already DECODE, render and lower all
//! four kinds — tags `0x1D`–`0x20`, proptested — while `compose` hardcoded
//! `Tag::Sha256` and `SpendPath.hash` was a bare `[u8; 32]`. So an `md1`
//! carrying a `ripemd160` hashlock read back correctly and could not be
//! authored. These tests are the authoring side.
//!
//! The 20-byte kinds are the ones with something to get wrong: the digest is
//! stored in a fixed `[u8; 32]` for the alloc gate (spec §5), so a lowering that
//! forgets `digest_len()` emits twelve zero bytes of padding into the script.

use md_codec::chunk::{reassemble, split};
use md_codec::compose::{HashKind, HashLock, KeySet, PathList, SpendPath, Wrapper, compose};
use md_codec::render::descriptor_to_template;
use md_codec::tag::Tag;

fn lock20(kind: HashKind) -> HashLock {
    let mut d = [0u8; 32];
    for (i, b) in d.iter_mut().take(20).enumerate() {
        *b = 0xA0 | (i as u8 & 0x0F);
    }
    HashLock::new(kind, d)
}

fn lock32(kind: HashKind) -> HashLock {
    HashLock::new(kind, [0xCD; 32])
}

/// Spec §5: `digest_len()` is the ONE place a length is written, and the
/// accessor hands back exactly that many bytes — never the zero padding.
#[test]
fn digest_len_is_the_only_length_and_the_accessor_honours_it() {
    for (kind, want) in [
        (HashKind::Sha256, 32usize),
        (HashKind::Hash256, 32),
        (HashKind::Ripemd160, 20),
        (HashKind::Hash160, 20),
    ] {
        assert_eq!(kind.digest_len(), want, "{kind:?}");
        let hl = if want == 32 {
            lock32(kind)
        } else {
            lock20(kind)
        };
        assert_eq!(
            hl.digest().len(),
            want,
            "{kind:?}: the accessor must not expose the alloc-gate padding"
        );
        assert!(
            hl.digest().iter().any(|b| *b != 0),
            "{kind:?}: fixture is degenerate"
        );
    }
}

/// Spec §5: the `HashKind -> Tag` mapping is a TOTAL function, and it is not
/// the wire `Tag` set — a hashlock field typed as `Tag` could hold `Wpkh`.
#[test]
fn every_kind_maps_to_its_own_wire_tag() {
    let pairs = [
        (HashKind::Sha256, Tag::Sha256),
        (HashKind::Hash256, Tag::Hash256),
        (HashKind::Ripemd160, Tag::Ripemd160),
        (HashKind::Hash160, Tag::Hash160),
    ];
    for (kind, tag) in pairs {
        assert_eq!(kind.tag(), tag, "{kind:?}");
    }
    let tags: Vec<Tag> = pairs.iter().map(|(_, t)| *t).collect();
    let mut uniq = tags.clone();
    uniq.sort_by_key(|t| format!("{t:?}"));
    uniq.dedup();
    assert_eq!(uniq.len(), tags.len(), "two kinds share a tag");
}

/// The lowering emits the kind's own tag, and — for the 20-byte kinds — a
/// 20-byte body. A lowering that ignored `digest_len()` would still compile and
/// still round-trip; it would just commit to twelve bytes of padding.
#[test]
fn lowering_emits_the_kinds_tag_and_its_own_width() {
    for kind in [
        HashKind::Sha256,
        HashKind::Hash256,
        HashKind::Ripemd160,
        HashKind::Hash160,
    ] {
        let hl = if kind.digest_len() == 32 {
            lock32(kind)
        } else {
            lock20(kind)
        };
        let out = compose(&PathList {
            wrapper: Wrapper::Wsh,
            paths: vec![SpendPath {
                keys: Some(KeySet {
                    k: 1,
                    n: 1,
                    sorted: true,
                }),
                hash: Some(hl),
                lock: None,
            }],
        })
        .unwrap_or_else(|e| panic!("{kind:?}: compose refused: {e:?}"));

        let rendered = descriptor_to_template(&out.descriptor).expect("render");
        assert!(
            rendered.contains(&format!("{}(", kind.token())),
            "{kind:?}: lowered to {rendered}"
        );
        // The digest in the script is the accessor's bytes, hex-encoded — so a
        // 20-byte kind must show 40 hex characters, not 64.
        // `fold`, not `map(format!).collect()` -- the latter trips clippy's
        // `format_collect` under `-D warnings`, a required CI context.
        let hex: String = hl.digest().iter().fold(String::new(), |mut acc, b| {
            use std::fmt::Write as _;
            let _ = write!(acc, "{b:02x}");
            acc
        });
        assert!(
            rendered.contains(&hex),
            "{kind:?}: script does not carry the digest at its own width.\n\
             want {hex}\ngot  {rendered}"
        );
    }
}

/// Spec §10's first vector requirement: **round trip per kind** — compose →
/// md1 → decode → template.
///
/// **`wsh` and `tr`, NOT `sh(wsh)`, and that is not an omission.** §10 asks for
/// `wsh` and `sh(wsh)`, but this composer refuses a hashlock under a legacy
/// wrapper *for every kind including sha256*: `ComposeError::LegacyWrapperShape`
/// — *"legacy wrappers hold one plain sorted multisig only (n >= 2, no lock, no
/// hash); use wsh or tr"* (`compose/mod.rs:480-489`). That rule predates this
/// cycle and is deliberate narrowness, so §10's requirement is unsatisfiable as
/// written and `tr` is the second wrapper that can actually carry one. Do not
/// "fix" this by adding `ShWsh` back — it refuses before any kind logic runs.
///
/// This is the test that would have caught a lowering that wrote the alloc-gate
/// padding: the wire body for a 20-byte kind is `Tag::Ripemd160` + 160 bits, so
/// a 32-byte body either fails to reassemble or comes back as a different tag.
/// Asserting the TEMPLATE after the round trip, not just that reassembly
/// succeeded, is what makes it a restore test rather than a "did it parse".
#[test]
fn every_kind_round_trips_compose_to_md1_to_template() {
    for wrapper in [Wrapper::Wsh, Wrapper::Tr] {
        for kind in [
            HashKind::Sha256,
            HashKind::Hash256,
            HashKind::Ripemd160,
            HashKind::Hash160,
        ] {
            let hl = if kind.digest_len() == 32 {
                lock32(kind)
            } else {
                lock20(kind)
            };
            let c = compose(&PathList {
                wrapper,
                paths: vec![SpendPath {
                    keys: Some(KeySet {
                        k: 1,
                        n: 1,
                        sorted: true,
                    }),
                    hash: Some(hl),
                    lock: None,
                }],
            })
            .unwrap_or_else(|e| panic!("{wrapper:?}/{kind:?}: compose refused: {e:?}"));

            let before = descriptor_to_template(&c.descriptor).expect("render");
            let chunks = split(&c.descriptor).expect("split");
            let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
            let back = reassemble(&refs).expect("reassemble");
            assert_eq!(
                back, c.descriptor,
                "{wrapper:?}/{kind:?}: the descriptor did not survive the wire"
            );

            let after = descriptor_to_template(&back).expect("render");
            assert_eq!(
                after, before,
                "{wrapper:?}/{kind:?}: the template changed across the round trip"
            );
            assert!(
                after.contains(&format!("{}(", kind.token())),
                "{wrapper:?}/{kind:?}: the kind was lost on the wire: {after}"
            );
        }
    }
}

/// Spec §5: the alloc-gate padding is **unobservable**. That is a property of
/// the TYPE, not of how carefully callers use it — §5 mandates that eleven
/// map/set/equality sites re-key on `HashLock`, so a derived `Eq` comparing all
/// 32 bytes would make the padding load-bearing at every one of them.
///
/// Derived traits shipped for one round and made it observable (R0 round 1,
/// I-1); `Eq`/`Ord`/`Hash` are hand-written over `(kind, digest())` now.
#[test]
fn the_padding_is_unobservable_through_eq_ord_and_hash() {
    use std::collections::HashSet;

    let mut a = [0u8; 32];
    for (i, b) in a.iter_mut().take(20).enumerate() {
        *b = 0xA0 | (i as u8 & 0x0F);
    }
    let mut b = a;
    b[31] = 0xFF; // padding only — beyond ripemd160's 20 bytes
    b[20] = 0x01;

    let la = HashLock::new(HashKind::Ripemd160, a);
    let lb = HashLock::new(HashKind::Ripemd160, b);

    assert_eq!(la.digest(), lb.digest(), "fixture: same visible digest");
    assert_eq!(la, lb, "padding must not make two equal locks unequal");
    assert_eq!(
        la.cmp(&lb),
        core::cmp::Ordering::Equal,
        "padding must not order two equal locks"
    );

    let mut set = HashSet::new();
    set.insert(la);
    set.insert(lb);
    assert_eq!(
        set.len(),
        1,
        "padding must not split one lock into two keys"
    );

    // ...and a DIFFERENT kind over the same bytes is still a different lock.
    assert_ne!(
        la,
        HashLock::new(HashKind::Hash160, a),
        "the kind is part of identity: the same 32 bytes mean different things"
    );
}
