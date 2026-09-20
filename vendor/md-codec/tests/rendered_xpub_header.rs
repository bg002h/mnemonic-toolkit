//! The rendered descriptor's xpub header must agree with the origin beside it.
//!
//! An md1 card stores each cosigner key as a 65-byte `(chain_code ||
//! compressed_pubkey)` Pubkeys-TLV payload — the four BIP-32 metadata fields
//! (`network`, `depth`, `parent_fingerprint`, `child_number`) are NOT on the
//! wire. `xpub_from_tlv_bytes` fills them with placeholders, which is correct
//! for ADDRESS derivation (only `chain_code` + `public_key` participate in
//! CKDpub, and `derive.rs` documents exactly that).
//!
//! It is NOT correct for RENDERING. A rendered key carries its origin — e.g.
//! `[73c5da0a/48'/0'/0'/2']xpub…` — which declares the key sits four levels
//! below the master. Serialising it with `depth = 0`, `parent_fingerprint =
//! 00000000`, `child_number = 0` emits a key that contradicts the origin
//! printed immediately to its left: a master-looking xpub under a depth-4
//! path. Two of those three fields are recoverable from the origin path the
//! card DOES carry, and this is the test that they are recovered.
//!
//! `parent_fingerprint` is the one field md1 genuinely cannot carry — it is
//! `hash160(parent_pubkey)[..4]` and the parent pubkey is not on the wire —
//! so it stays zero, asserted below so the loss is explicit rather than
//! discovered later.
//!
//! GOLDENS COME FROM OUTSIDE md-codec: the expected depth/child/chain_code/
//! public_key are read off a real `Xpub` derived with rust-bitcoin from the
//! abandon mnemonic, never from md-codec's own render.
//!
//! Feature-gated behind `derive` (default-on).

#![cfg(feature = "derive")]

use bitcoin::Network;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;
use md_codec::{Descriptor, OriginPath, PathComponent, PathDecl, PathDeclPaths, Tag, TlvSection};
use std::str::FromStr;

const ABANDON_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// The real account `Xpub` at `path_str` — the TRUTH this test measures
/// against. Derived with rust-bitcoin, not with md-codec.
fn account_xpub(path_str: &str) -> Xpub {
    let mn = bip39::Mnemonic::parse(ABANDON_MNEMONIC).expect("known good mnemonic");
    let seed = mn.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(Network::Bitcoin, &seed).expect("seed → master");
    let path = DerivationPath::from_str(path_str).expect("valid path");
    let xpriv = master.derive_priv(&secp, &path).expect("derive priv");
    Xpub::from_priv(&secp, &xpriv)
}

/// The 65-byte Pubkeys-TLV payload for that xpub — exactly what an md1 card
/// stores, with the header discarded.
fn tlv_bytes(x: &Xpub) -> [u8; 65] {
    let mut out = [0u8; 65];
    out[..32].copy_from_slice(x.chain_code.as_ref());
    out[32..].copy_from_slice(&x.public_key.serialize());
    out
}

fn origin(components: &[(bool, u32)]) -> OriginPath {
    OriginPath {
        components: components
            .iter()
            .map(|&(hardened, value)| PathComponent { hardened, value })
            .collect(),
    }
}

/// The three paths under test. DELIBERATELY divergent in BOTH depth and
/// terminal component, so a fix that hardcodes either one (say `depth = 4`,
/// or `child = 2'`) still fails here.
const PATHS: [&str; 3] = ["m/48'/0'/0'/2'", "m/48'/0'/1'/1'", "m/84'/0'/7'"];

fn origin_for(i: usize) -> OriginPath {
    match i {
        0 => origin(&[(true, 48), (true, 0), (true, 0), (true, 2)]),
        1 => origin(&[(true, 48), (true, 0), (true, 1), (true, 1)]),
        _ => origin(&[(true, 84), (true, 0), (true, 7)]),
    }
}

/// `wsh(sortedmulti(2, @0, @1, @2))` keyed with the three accounts above,
/// each under its OWN origin.
fn keyed_descriptor() -> Descriptor {
    let mut tlv = TlvSection::new_empty();
    tlv.pubkeys = Some(
        (0..3u8)
            .map(|i| (i, tlv_bytes(&account_xpub(PATHS[i as usize]))))
            .collect(),
    );
    tlv.fingerprints = Some((0..3u8).map(|i| (i, [0xaa, 0xbb, 0xcc, i])).collect());
    Descriptor {
        n: 3,
        path_decl: PathDecl {
            n: 3,
            paths: PathDeclPaths::Divergent((0..3).map(origin_for).collect()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::SortedMulti,
                body: Body::MultiKeys {
                    k: 2,
                    indices: vec![0, 1, 2],
                },
            }]),
        },
        tlv,
    }
}

/// Every `xpub…` token in `s`, in order of appearance.
fn rendered_xpubs(s: &str) -> Vec<Xpub> {
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while let Some(rel) = s[i..].find("xpub") {
        let start = i + rel;
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_alphanumeric() {
            end += 1;
        }
        out.push(
            Xpub::from_str(&s[start..end]).unwrap_or_else(|e| {
                panic!("rendered token {:?} is not an xpub: {e}", &s[start..end])
            }),
        );
        i = end;
    }
    out
}

#[test]
fn rendered_xpub_header_agrees_with_its_origin() {
    let d = keyed_descriptor();
    let rendered = md_codec::to_miniscript::to_miniscript_descriptor_multipath(&d)
        .expect("keyed wsh(sortedmulti) renders")
        .to_string();

    let got = rendered_xpubs(&rendered);
    assert_eq!(got.len(), 3, "expected 3 rendered xpubs in {rendered:?}");

    // `sortedmulti` renders in LEXICOGRAPHIC key order, not @N order, so match
    // each rendered key to its truth by public key rather than by position.
    for r in &got {
        let truth = PATHS
            .iter()
            .map(|p| account_xpub(p))
            .find(|t| t.public_key == r.public_key)
            .unwrap_or_else(|| panic!("rendered key {r} is not one of the three accounts"));

        assert_eq!(
            r.chain_code, truth.chain_code,
            "chain code must survive the 65-byte round trip"
        );
        assert_eq!(
            r.depth, truth.depth,
            "rendered depth must equal the origin's component count \
             (truth {}, origin {:?})",
            truth.depth, truth.child_number
        );
        assert_eq!(
            r.child_number, truth.child_number,
            "rendered child number must equal the origin's terminal component"
        );
        assert_eq!(
            r.parent_fingerprint,
            Default::default(),
            "parent fingerprint is NOT on the md1 wire (it is hash160 of the \
             PARENT pubkey, which the card does not carry) — it must stay zero \
             rather than be invented"
        );
    }
}

/// The depths under test are genuinely different, so the assertion above
/// cannot be satisfied by a constant. Guards against the test decaying into a
/// tautology if `PATHS` is ever edited.
#[test]
fn the_three_paths_do_not_share_a_depth_and_terminal() {
    let depths: Vec<u8> = PATHS.iter().map(|p| account_xpub(p).depth).collect();
    let children: Vec<_> = PATHS.iter().map(|p| account_xpub(p).child_number).collect();
    assert!(
        depths.iter().any(|d| *d != depths[0]),
        "PATHS must span more than one depth, got {depths:?}"
    );
    assert!(
        children.iter().any(|c| *c != children[0]),
        "PATHS must span more than one terminal component, got {children:?}"
    );
}
