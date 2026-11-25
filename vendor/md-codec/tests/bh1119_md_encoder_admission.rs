//! FIX_bh1119_md_encoder_admission: `encode_payload` decodes its own output
//! and returns the decoder's error if the decoder refuses (rows R1-R5, R7,
//! R8, controls K1-K3). See `design/FIX_bh1119_md_encoder_admission.md`.

#![cfg(feature = "derive")]

use bitcoin::Network;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::{Alternative, UseSitePath};
use md_codec::{Descriptor, OriginPath, PathComponent, PathDecl, PathDeclPaths, Tag, TlvSection};
use std::str::FromStr;

const ABANDON_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// 65-byte `(chain_code || compressed_pubkey)` Pubkeys-TLV payload for the
/// abandon-mnemonic account xpub at `path_str`. Mirrors
/// `address_derivation.rs::account_xpub_bytes`.
fn account_xpub_bytes(path_str: &str) -> [u8; 65] {
    let mn = bip39::Mnemonic::parse(ABANDON_MNEMONIC).expect("known good mnemonic");
    let seed = mn.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(Network::Bitcoin, &seed).expect("seed → master");
    let path = DerivationPath::from_str(path_str).expect("valid path");
    let account_xpriv = master.derive_priv(&secp, &path).expect("derive priv");
    let account_xpub = Xpub::from_priv(&secp, &account_xpriv);
    let mut out = [0u8; 65];
    out[..32].copy_from_slice(account_xpub.chain_code.as_ref());
    out[32..].copy_from_slice(&account_xpub.public_key.serialize());
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

fn alt(value: u32) -> Alternative {
    Alternative {
        hardened: false,
        value,
    }
}

/// `<a;b>/*` use-site path (non-hardened wildcard).
fn mp(a: u32, b: u32) -> UseSitePath {
    UseSitePath {
        multipath: Some(vec![alt(a), alt(b)]),
        wildcard_hardened: false,
    }
}

/// Build `wsh(multi(2, @0/<base>/*, @1/<over>/*))` with the abandon-mnemonic
/// xpubs at two distinct account paths and a per-`@N` use-site override on
/// `@1` (when `over != base`). `tree_tag`/wrapper kept simple (wsh+multi).
fn divergent_wsh_multi(base: UseSitePath, over_at1: Option<UseSitePath>) -> Descriptor {
    let xpub0 = account_xpub_bytes("m/48'/0'/0'/2'");
    let xpub1 = account_xpub_bytes("m/48'/0'/1'/2'");
    let mut tlv = TlvSection::new_empty();
    tlv.pubkeys = Some(vec![(0u8, xpub0), (1u8, xpub1)]);
    if let Some(over) = over_at1 {
        tlv.use_site_path_overrides = Some(vec![(1u8, over)]);
    }
    Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Divergent(vec![
                origin(&[(true, 48), (true, 0), (true, 0), (true, 2)]),
                origin(&[(true, 48), (true, 0), (true, 1), (true, 2)]),
            ]),
        },
        use_site_path: base,
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::Multi,
                body: Body::MultiKeys {
                    k: 2,
                    indices: vec![0, 1],
                },
            }]),
        },
        tlv,
    }
}

use md_codec::{
    Error, decode_md1_string, decode_payload, encode_payload, encode_payload_unadmitted,
};

const CARD: &str = "md1yqpqqxqq8xtwhw4xwn4qh";

fn card() -> Descriptor {
    decode_md1_string(CARD).expect("decode the base card")
}

/// `wsh` over `n` `Tag::ZeroNotEqual` nodes over `Tag::PkK` key 0, shared
/// origin `m/84'`, standard `<0;1>/*` use-site: the tree `md encode
/// 'wsh(<N x n>:pk(@0/<0;1>/*))' --path bip84` builds.
fn depth_descriptor(n: usize) -> Descriptor {
    let mut node = Node {
        tag: Tag::PkK,
        body: Body::KeyArg { index: 0 },
    };
    for _ in 0..n {
        node = Node {
            tag: Tag::ZeroNotEqual,
            body: Body::Children(vec![node]),
        };
    }
    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin(&[(true, 84)])),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![node]),
        },
        tlv: TlvSection::new_empty(),
    }
}

#[test]
fn bh_md_i5_path_decl_count_mismatch_refused() {
    let mut d = card();
    d.path_decl.n = 2;
    let r = encode_payload(&d);
    assert!(
        matches!(
            r,
            Err(Error::BitStreamTruncated {
                requested: 1,
                available: 0
            })
        ),
        "got {r:?}"
    );
}

#[test]
fn bh_md_i5_non_root_tag_refused() {
    let mut d = card();
    d.tree.tag = Tag::PkK;
    let r = encode_payload(&d);
    assert!(
        matches!(
            r,
            Err(Error::OperatorContextViolation {
                tag: Tag::PkK,
                context: md_codec::error::ContextKind::TopLevel
            })
        ),
        "got {r:?}"
    );
}

#[test]
fn bh_md_i5_invalid_pubkey_bytes_refused() {
    let mut d = card();
    d.tlv.pubkeys = Some(vec![(0, [0u8; 65])]);
    let r = encode_payload(&d);
    assert!(
        matches!(r, Err(Error::InvalidXpubBytes { idx: 0 })),
        "got {r:?}"
    );
}

#[test]
fn bh_md_i5_empty_origin_override_refused() {
    let mut d = card();
    d.tlv.origin_path_overrides = Some(vec![(0, OriginPath { components: vec![] })]);
    let r = encode_payload(&d);
    assert!(
        matches!(r, Err(Error::EmptyOriginOverride { idx: 0 })),
        "got {r:?}"
    );
}

#[test]
fn bh_md_row10_depth_127_encode_refused() {
    let r = encode_payload(&depth_descriptor(127));
    assert!(
        matches!(
            r,
            Err(Error::DecodeRecursionDepthExceeded {
                depth: 128,
                max: 128
            })
        ),
        "got {r:?}"
    );
}

#[test]
fn bh_md_i5_at0_use_site_override_refused() {
    let mut d = divergent_wsh_multi(UseSitePath::standard_multipath(), None);
    d.tlv.use_site_path_overrides = Some(vec![(0u8, mp(2, 3))]);
    let r = encode_payload(&d);
    assert!(
        matches!(r, Err(Error::BaselineUseSiteOverride { idx: 0 })),
        "got {r:?}"
    );
}

#[test]
fn bh_md_i5_redundant_use_site_override_refused() {
    let mut d = divergent_wsh_multi(UseSitePath::standard_multipath(), None);
    d.tlv.use_site_path_overrides = Some(vec![(1u8, UseSitePath::standard_multipath())]);
    let r = encode_payload(&d);
    assert!(
        matches!(r, Err(Error::RedundantUseSiteOverride { idx: 1 })),
        "got {r:?}"
    );
}

#[test]
fn bh_md_i5_control_unmodified_roundtrips() {
    let (bytes, bits) = encode_payload(&card()).expect("encode");
    decode_payload(&bytes, bits).expect("decode");
}

#[test]
fn bh_md_row10_depth_126_encodes_and_decodes() {
    let (bytes, bits) = encode_payload(&depth_descriptor(126)).expect("encode");
    decode_payload(&bytes, bits).expect("decode");
}

#[test]
fn bh_md_row10_depth_127_unadmitted_still_serialises() {
    encode_payload_unadmitted(&depth_descriptor(127)).expect("unadmitted encode is not gated");
}
