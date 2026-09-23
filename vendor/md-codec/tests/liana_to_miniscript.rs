//! Stage 1b task 5 (SPEC §4 row 1 + the `_with_network` entry points):
//! `to_miniscript.rs`'s half of G-1's four gating `NumsPoint |
//! LianaUnspendable` or-patterns (`to_miniscript.rs:341`) — before this task
//! a wire-kind-1 internal key silently rendered as the NUMS H-point, which
//! is a WRONG taproot output key, not merely a display defect.
//!
//! `kind1_from_vector` (from `common/liana.rs`) draws its vector names from
//! the STAGE-1A corpus (`tests/vectors/*.phrase.txt`, e.g.
//! `keyed_compose_tr_nums_three_leaves`) — a DIFFERENT namespace from
//! `cases.json`'s Liana-evidence `name` field (e.g. `preset-kofn-recovery-tr`).
//! The two corpora share no keys, so a `kind1_from_vector`-built descriptor's
//! derived xpub cannot be checked against a same-named `cases.json` entry —
//! this file instead builds descriptors directly from `Case::leaf_tlv_hex`
//! (the real Liana evidence's own 65-byte `chain_code‖pubkey` TLV entries)
//! when it needs to cross-check against `expected_xpub`, and uses
//! `kind1_from_vector` only where no cross-check against real evidence is
//! needed (the network-less refusal, which is a wire-kind check that does
//! not touch key material at all).

#![cfg(feature = "derive")]

use bitcoin::Network;
use md_codec::error::Error;
use md_codec::tree::Node;
use md_codec::use_site_path::UseSitePath;
use md_codec::{OriginPath, PathDecl, PathDeclPaths, Tag, TlvSection};

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/common/vendored.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/common/liana.rs"
));
// `Descriptor` (from `vendored.rs`) and `Body`/`InternalKey` (from
// `liana.rs`) are already in scope from the two splices above — importing
// them again here collides (E0252), exactly as each file's own header
// comment warns.

fn keyarg(index: u8) -> Node {
    Node {
        tag: Tag::PkK,
        body: Body::KeyArg { index },
    }
}

fn taptree2(l: Node, r: Node) -> Node {
    Node {
        tag: Tag::TapTree,
        body: Body::Children(vec![l, r]),
    }
}

/// Divergent per-`@N` origin path (`crates/md-codec/tests/common/mod.rs`'s
/// own `divergent_path`, redefined here narrowly for the same E0252 reason
/// noted above): resolves `expand_per_at_n` for ANY tree shape without
/// relying on `canonical_origin`'s BIP48-type-1 default matching, which a
/// bare `tr(...)` tree (no `wsh`/`sh` wrapper) does not.
fn divergent_path(n: u8) -> PathDecl {
    let paths = (0..n)
        .map(|c| OriginPath {
            components: vec![md_codec::PathComponent {
                hardened: true,
                value: (c as u32) * 100 + 1,
            }],
        })
        .collect();
    PathDecl {
        n,
        paths: PathDeclPaths::Divergent(paths),
    }
}

/// A wallet-policy-mode `tr(LianaUnspendable, {{pk(@0),pk(@1)},{pk(@2),pk(@3)}})`
/// built directly from `Case::leaf_tlv_hex` (real Liana evidence's own
/// 65-byte `chain_code‖pubkey` TLV entries, in wire order) — NOT from
/// `kind1_from_vector`, which draws from an unrelated fixture namespace (see
/// the file header). Any tree SHAPE over the same 4 keys in the same order
/// yields the same derived xpub (Task 4's
/// `the_chain_code_depends_on_the_keys_not_the_tree`), so a flat 4-leaf tree
/// is a faithful, simpler stand-in for `preset-kofn-recovery-tr`'s real
/// `{multi_a(2,K0,K1,K2),and_v(v:pk(K3),older(26280))}` shape for THIS
/// purpose — this file is checking the internal-key derivation end to end,
/// not re-deriving `preset-kofn-recovery-tr`'s own script tree.
fn descriptor_from_case(c: &Case) -> Descriptor {
    let tlv_bytes: Vec<[u8; 65]> = c
        .leaf_tlv_hex
        .iter()
        .map(|h| <[u8; 65]>::try_from(hex::decode(h).expect("hex").as_slice()).expect("65 bytes"))
        .collect();
    assert_eq!(tlv_bytes.len(), 4, "{}: expected 4 leaf keys", c.name);
    let tree = Node {
        tag: Tag::Tr,
        body: Body::Tr {
            internal_key: InternalKey::LianaUnspendable,
            tree: Some(Box::new(taptree2(
                taptree2(keyarg(0), keyarg(1)),
                taptree2(keyarg(2), keyarg(3)),
            ))),
        },
    };
    let mut tlv = TlvSection::new_empty();
    tlv.pubkeys = Some((0u8..4).map(|i| (i, tlv_bytes[i as usize])).collect());
    Descriptor {
        n: 4,
        path_decl: divergent_path(4),
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv,
    }
}

/// The `to_miniscript.rs:341` gating site, positive control, cross-checked
/// against REAL Liana evidence: a keyed MULTIPATH descriptor embeds the
/// derived literal xpub, `<0;1>/*`, no origin — SPEC §4 row 1, exactly, and
/// byte-identical to what Liana itself computed for these 4 keys. Reverting
/// the split at `to_miniscript.rs:341` back to
/// `NumsPoint | LianaUnspendable => build_nums_internal_key()?` makes this
/// assert the NUMS H-point instead of an xpub and fails immediately.
#[test]
fn the_keyed_multipath_descriptor_embeds_the_derived_literal_xpub_matching_liana_evidence() {
    for name in [
        "preset-kofn-recovery-tr",
        "preset-tiered-recovery-tr",
        "same-seed-two-paths-tr",
        "X19-tr-kofn-nums-older5",
    ] {
        let c = case(name);
        let d = descriptor_from_case(&c);
        let desc = md_codec::to_miniscript_descriptor_multipath_with_network(&d, Network::Bitcoin)
            .unwrap_or_else(|e| {
                panic!("{name}: to_miniscript_descriptor_multipath_with_network: {e}")
            });
        let internal = desc
            .internal_key()
            .unwrap_or_else(|| panic!("{name}: tr() must have an internal key"))
            .to_string();
        assert_eq!(
            internal,
            format!("{}/<0;1>/*", c.expected_xpub),
            "{name}: the rendered internal key must be the derived xpub with <0;1>/* and no origin"
        );
    }
}

/// The single-path entry point collapses the internal key's own `<0;1>` to
/// whichever `chain` was asked for, mirroring every other key in the
/// descriptor (`build_descriptor_public_key`'s single-path collapse) rather
/// than always rendering the full multipath group.
#[test]
fn the_keyed_single_path_descriptor_collapses_the_internal_key_to_the_chosen_chain() {
    let c = case("preset-kofn-recovery-tr");
    let d = descriptor_from_case(&c);
    for (chain, suffix) in [(0u32, "/0/*"), (1u32, "/1/*")] {
        let desc = md_codec::to_miniscript_descriptor_with_network(&d, chain, Network::Bitcoin)
            .unwrap_or_else(|e| panic!("chain {chain}: {e}"));
        let internal = desc.internal_key().unwrap().to_string();
        assert_eq!(
            internal,
            format!("{}{suffix}", c.expected_xpub),
            "chain {chain}: internal key must collapse to the single alt, not stay <0;1>"
        );
    }
}

/// SPEC §2 step 5's testnet branch, exercised through the FULL pipeline (not
/// just `nums::liana_unspendable_xpub` directly, which Task 4 already
/// covers): the rendered internal key's base58 prefix is `tpub` under
/// `Network::Testnet`, and matches the recipe computed independently over
/// the same evidence's leaf pubkeys.
#[test]
fn a_tpub_wallet_renders_a_tpub_internal_key() {
    let c = case("preset-kofn-recovery-tr");
    let d = descriptor_from_case(&c);
    let desc =
        md_codec::to_miniscript_descriptor_multipath_with_network(&d, Network::Testnet).unwrap();
    let internal = desc.internal_key().unwrap().to_string();
    assert!(
        internal.starts_with("tpub"),
        "must render tpub under Network::Testnet, got {internal}"
    );
    let want = md_codec::nums::liana_unspendable_xpub(&c.leaf_pubkeys(), Network::Testnet);
    assert_eq!(internal, format!("{want}/<0;1>/*"));
}

/// SPEC §4 / Step 3: the network-less entry points refuse a wire-kind-1
/// descriptor rather than silently rendering it under an assumed mainnet —
/// the brief's own verbatim test (vector name from the STAGE-1A corpus,
/// since this test only cares about the wire-kind refusal, not key
/// material).
#[test]
fn a_kind_1_descriptor_is_refused_by_the_network_less_entry_point() {
    let d = kind1_from_vector("keyed_compose_tr_nums_three_leaves");
    assert!(matches!(
        md_codec::to_miniscript_descriptor_multipath(&d),
        Err(Error::NetworkRequiredForUnspendable)
    ));
}

/// Same refusal, single-path entry point — the brief names both
/// `to_miniscript_descriptor_with_network` and
/// `to_miniscript_descriptor_multipath_with_network` as the pair of
/// interfaces this task produces, so both network-less counterparts must
/// refuse identically.
#[test]
fn a_kind_1_descriptor_is_refused_by_the_single_path_network_less_entry_point() {
    let d = kind1_from_vector("keyed_compose_tr_nums_three_leaves");
    assert!(matches!(
        md_codec::to_miniscript_descriptor(&d, 0),
        Err(Error::NetworkRequiredForUnspendable)
    ));
}

/// Positive control: a kind-0 (ordinary NUMS) descriptor is completely
/// unaffected by this task — the network-less entry points still work
/// exactly as before, refusing NOTHING. Guards against an over-eager
/// refusal that fires on `wire_version() == 4` too.
#[test]
fn a_kind_0_descriptor_still_works_on_the_network_less_entry_point() {
    let name = "keyed_compose_tr_nums_three_leaves";
    let d = decode_vendored(&load_vendored_phrase(name)).unwrap();
    assert_eq!(
        d.wire_version(),
        4,
        "{name} must still be a kind-0 (v4) vector"
    );
    assert!(md_codec::to_miniscript_descriptor_multipath(&d).is_ok());
    assert!(md_codec::to_miniscript_descriptor(&d, 0).is_ok());
}

/// A leaf carrying MULTIPLE keys (`multi_a`), not just one `pk()` per leaf:
/// proves `collect_leaf_pubkeys` walks every key OCCURRENCE within a leaf
/// (`Miniscript::iter_pk()`), not merely one representative key per leaf —
/// the shape `preset-kofn-recovery-tr`'s real evidence needs
/// (`multi_a(2,K0,K1,K2)` is one leaf with three keys) but the flat-4-leaf
/// stand-in above never exercises. Compares the full pipeline's answer
/// against `nums::liana_unspendable_xpub` called directly over the SAME
/// three-key list in the SAME order — an independent computation using the
/// primitive Task 4 already proved correct, not a tautology.
#[test]
fn a_multi_key_leaf_contributes_every_key_in_order() {
    let pks = common::test_xpubs();
    let tree = Node {
        tag: Tag::Tr,
        body: Body::Tr {
            internal_key: InternalKey::LianaUnspendable,
            tree: Some(Box::new(Node {
                tag: Tag::MultiA,
                body: Body::MultiKeys {
                    k: 2,
                    indices: vec![0, 1, 2],
                },
            })),
        },
    };
    let mut tlv = TlvSection::new_empty();
    tlv.pubkeys = Some((0u8..3).map(|i| (i, pks[i as usize])).collect());
    let d = Descriptor {
        n: 3,
        path_decl: divergent_path(3),
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv,
    };
    let desc =
        md_codec::to_miniscript_descriptor_multipath_with_network(&d, Network::Bitcoin).unwrap();
    let internal = desc.internal_key().unwrap().to_string();
    let leaf_pubkeys: Vec<[u8; 33]> = (0..3)
        .map(|i| <[u8; 33]>::try_from(&pks[i][32..65]).unwrap())
        .collect();
    let want = md_codec::nums::liana_unspendable_xpub(&leaf_pubkeys, Network::Bitcoin);
    assert_eq!(internal, format!("{want}/<0;1>/*"));
}

mod common {
    // `test_xpubs` needs no more than this file already pulled in; re-declared
    // narrowly here rather than pulling in the whole `tests/common/mod.rs`
    // (which would collide with `vendored.rs`'s `use md_codec::Descriptor;`
    // already spliced above — see that file's own header comment on why
    // `tests/` crate roots cannot share code via plain `use`).
    pub fn test_xpubs() -> &'static [[u8; 65]; 32] {
        static XPUBS: std::sync::OnceLock<[[u8; 65]; 32]> = std::sync::OnceLock::new();
        XPUBS.get_or_init(|| {
            use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
            use bitcoin::secp256k1::Secp256k1;
            use std::str::FromStr;
            const ABANDON_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
            let mn = bip39::Mnemonic::parse(ABANDON_MNEMONIC).expect("known-good mnemonic");
            let seed = mn.to_seed("");
            let secp = Secp256k1::new();
            let master = Xpriv::new_master(bitcoin::Network::Bitcoin, &seed)
                .expect("seed gives master");
            let mut out = [[0u8; 65]; 32];
            for (i, slot) in out.iter_mut().enumerate() {
                let path =
                    DerivationPath::from_str(&format!("m/86'/0'/{i}'")).expect("valid path");
                let xpriv = master.derive_priv(&secp, &path).expect("derive priv");
                let xpub = Xpub::from_priv(&secp, &xpriv);
                slot[..32].copy_from_slice(xpub.chain_code.as_ref());
                slot[32..].copy_from_slice(&xpub.public_key.serialize());
            }
            out
        })
    }
}
