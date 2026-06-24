//! Shared generators + corruption helpers for the mk-codec test-hardening
//! suite. Consumed by `proptest_roundtrip.rs` and `bch_adversarial.rs` via
//! `mod common;`. Cargo does not treat `common/mod.rs` as its own test binary.
#![allow(dead_code)] // each test file uses a subset of these helpers

use std::str::FromStr;

use bitcoin::NetworkKind;
use bitcoin::bip32::{ChainCode, ChildNumber, DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use mk_codec::KeyCard;
use proptest::prelude::*;

/// The 14 standard-path dictionary entries (mirror of
/// `crates/mk-codec/src/bytecode/path.rs::STANDARD_PATHS`). Generating these
/// exercises the 1-byte standard-path indicator encode mode.
pub const STANDARD_PATHS: &[&str] = &[
    "m/44'/0'/0'",
    "m/49'/0'/0'",
    "m/84'/0'/0'",
    "m/86'/0'/0'",
    "m/48'/0'/0'/2'",
    "m/48'/0'/0'/1'",
    "m/87'/0'/0'",
    "m/44'/1'/0'",
    "m/49'/1'/0'",
    "m/84'/1'/0'",
    "m/86'/1'/0'",
    "m/48'/1'/0'/2'",
    "m/48'/1'/0'/1'",
    "m/87'/1'/0'",
];

/// A derivation path: either a standard dictionary entry (1-byte indicator
/// encode mode) OR a random explicit path of 1..=10 components with random
/// hardened bits (the `0xFE` escape mode). Both round-trip; an explicit path
/// that happens to match a dictionary entry will encode via the indicator
/// (`lookup_path`) — that is correct and not asserted against.
pub fn path_strategy() -> impl Strategy<Value = DerivationPath> {
    let standard = (0..STANDARD_PATHS.len())
        .prop_map(|i| DerivationPath::from_str(STANDARD_PATHS[i]).expect("valid standard path"));

    let explicit = prop::collection::vec(
        (0u32..0x8000_0000u32, any::<bool>()).prop_map(|(idx, hardened)| {
            if hardened {
                ChildNumber::from_hardened_idx(idx).expect("idx < 2^31")
            } else {
                ChildNumber::from_normal_idx(idx).expect("idx < 2^31")
            }
        }),
        1..=10usize,
    )
    .prop_map(DerivationPath::from);

    // The empty path (no-path / depth-0 key, e.g. a WIF) — encodes as `0xFE 0x00`,
    // decodes to depth-0 / child Normal{0}. v0.4.0+ (mk1-no-path-depth0-support).
    let empty = Just(DerivationPath::from_str("m").expect("empty path parses"));

    prop_oneof![standard, explicit, empty].boxed()
}

/// An `Xpub` built by DIRECT struct construction (precedent:
/// `tests/round_trip.rs::synthetic_xpub`). `depth`/`child_number` are derived
/// from `path` so they are consistent by construction (sidesteps the
/// depth/child "lossless by construction" seam — SPEC §1.1). `public_key`,
/// `chain_code`, `parent_fingerprint`, and `network` are strategy-varied.
pub fn xpub_strategy(path: DerivationPath) -> impl Strategy<Value = Xpub> {
    // `path.as_ref()` → `&[ChildNumber]` (avoids `clippy::into_iter_on_ref`
    // under CI's `-D warnings`; R0 M3). `path` is moved into the closure below.
    let components: Vec<ChildNumber> = path.as_ref().to_vec();
    let depth = components.len() as u8;
    // Normal{0} for the empty (no-path / depth-0) case; terminal component
    // otherwise. Mirrors reconstruct_xpub / synthetic_xpub.
    let child_number = components
        .last()
        .copied()
        .unwrap_or(ChildNumber::Normal { index: 0 });

    (
        any::<[u8; 32]>().prop_filter("valid secp256k1 scalar", |b| {
            SecretKey::from_slice(b).is_ok()
        }),
        any::<[u8; 32]>(),
        any::<[u8; 4]>(),
        any::<bool>(),
    )
        .prop_map(move |(sk_bytes, cc, pfp, mainnet)| {
            let secp = Secp256k1::new();
            let sk = SecretKey::from_slice(&sk_bytes).expect("filtered to valid scalar");
            let pk = PublicKey::from_secret_key(&secp, &sk);
            Xpub {
                network: if mainnet {
                    NetworkKind::Main
                } else {
                    NetworkKind::Test
                },
                depth,
                parent_fingerprint: Fingerprint::from(pfp),
                child_number,
                public_key: pk,
                chain_code: ChainCode::from(cc),
            }
        })
}

/// A valid, encodable, depth/child-consistent `KeyCard`. `policy_id_stubs`
/// length 1..=8 (the 255-stub boundary is a separate deterministic cell, T4).
pub fn keycard_strategy() -> impl Strategy<Value = KeyCard> {
    (
        prop::collection::vec(any::<[u8; 4]>(), 1..=8usize),
        prop::option::of(any::<[u8; 4]>().prop_map(Fingerprint::from)),
        path_strategy(),
    )
        .prop_flat_map(|(stubs, fp, path)| {
            let p = path.clone();
            xpub_strategy(path)
                .prop_map(move |xpub| KeyCard::new(stubs.clone(), fp, p.clone(), xpub))
        })
        .boxed()
}

/// A chunk-set-id within the 20-bit wire cap (`> MAX_CHUNK_SET_ID` →
/// `ChunkedHeaderMalformed`).
pub fn csid_strategy() -> impl Strategy<Value = u32> {
    0u32..=0x000F_FFFFu32
}

/// Flip the bech32 symbol at each `position` (char index) to a guaranteed-
/// different symbol — 'q' (value 0) ↔ 'p' (value 1) — preserving string
/// length (so the BCH length band is unchanged; the flips are pure
/// substitutions). Mirrors the corruption idiom in
/// `src/string_layer/pipeline.rs`'s 5-burst test.
pub fn flip_chars(s: &str, positions: &[usize]) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    for &p in positions {
        chars[p] = if chars[p] == 'q' { 'p' } else { 'q' };
    }
    chars.into_iter().collect()
}
