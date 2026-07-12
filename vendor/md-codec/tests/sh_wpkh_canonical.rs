//! F-A1: `sh(wpkh)` elided-origin canonical-default (BIP49 nested segwit).
//!
//! Before the fix `canonical_origin(sh(wpkh(@N)))` was `None`, so an
//! origin-elided `sh(wpkh)` card self-rejected on decode
//! (`MissingExplicitOrigin`) and `compute_wallet_policy_id` errored. After
//! the fix the wrapper resolves to `m/49'/0'/0'`, so:
//!   * the elided form decodes,
//!   * its `WalletPolicyId` / 12-word phrase equals the explicit-`49'` form,
//!   * an explicit-`49'` card still decodes byte-identically.

use md_codec::canonical_origin::canonical_origin;
use md_codec::decode::decode_md1_string;
use md_codec::encode::{Descriptor, encode_md1_string};
use md_codec::identity::compute_wallet_policy_id;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;

/// `sh(wpkh(@0))` tree.
fn sh_wpkh_tree() -> Node {
    Node {
        tag: Tag::Sh,
        body: Body::Children(vec![Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        }]),
    }
}

fn bip49_path() -> OriginPath {
    OriginPath {
        components: vec![
            PathComponent {
                hardened: true,
                value: 49,
            },
            PathComponent {
                hardened: true,
                value: 0,
            },
            PathComponent {
                hardened: true,
                value: 0,
            },
        ],
    }
}

/// `sh(wpkh(@0/<0;1>/*))` with the given origin-path declaration.
fn sh_wpkh_descriptor(paths: PathDeclPaths) -> Descriptor {
    Descriptor {
        n: 1,
        path_decl: PathDecl { n: 1, paths },
        use_site_path: UseSitePath::standard_multipath(),
        tree: sh_wpkh_tree(),
        tlv: TlvSection::new_empty(),
    }
}

fn elided() -> Descriptor {
    sh_wpkh_descriptor(PathDeclPaths::Shared(OriginPath { components: vec![] }))
}

fn explicit_49() -> Descriptor {
    sh_wpkh_descriptor(PathDeclPaths::Shared(bip49_path()))
}

#[test]
fn canonical_origin_sh_wpkh_is_bip49() {
    assert_eq!(canonical_origin(&sh_wpkh_tree()), Some(bip49_path()));
}

#[test]
fn compute_wallet_policy_id_elided_sh_wpkh_succeeds() {
    // Pre-F-A1 this errored MissingExplicitOrigin; post-fix it computes.
    assert!(
        compute_wallet_policy_id(&elided()).is_ok(),
        "elided sh(wpkh) policy-id must compute after F-A1"
    );
}

#[test]
fn elided_sh_wpkh_policy_id_equals_explicit_49() {
    let id_elided = compute_wallet_policy_id(&elided()).expect("elided computes");
    let id_explicit = compute_wallet_policy_id(&explicit_49()).expect("explicit computes");
    assert_eq!(
        id_elided, id_explicit,
        "elided sh(wpkh) WalletPolicyId must equal the explicit m/49'/0'/0' form"
    );
    assert_eq!(
        id_elided.to_phrase().unwrap().to_string(),
        id_explicit.to_phrase().unwrap().to_string(),
        "12-word anchor phrases must match across origin-elision"
    );
}

#[test]
fn elided_sh_wpkh_string_round_trips() {
    // Encode with NO explicit origin, then decode — previously rejected with
    // `MissingExplicitOrigin`; now round-trips.
    let d = elided();
    let s = encode_md1_string(&d).expect("elided sh(wpkh) encodes");
    let decoded = decode_md1_string(&s).expect("elided sh(wpkh) decodes after F-A1");
    // Decoded tree matches the input wrapper shape.
    assert_eq!(decoded.tree, sh_wpkh_tree());
}

#[test]
fn explicit_49_sh_wpkh_still_round_trips_byte_identically() {
    // Recovery-safety: an explicit-origin sh(wpkh) card is unchanged.
    let d = explicit_49();
    let s = encode_md1_string(&d).expect("explicit sh(wpkh) encodes");
    let decoded = decode_md1_string(&s).expect("explicit sh(wpkh) decodes");
    // Re-encode the decoded descriptor: byte-identical wire.
    let s2 = encode_md1_string(&decoded).expect("re-encode");
    assert_eq!(
        s, s2,
        "explicit-49' sh(wpkh) wire must be stable across decode"
    );
}
