//! Kills for the md-codec baseline GAP-B mutation survivors (tests only).
//! Each test names the survivor it kills in its doc comment.

use md_codec::bch::{bch_create_checksum_regular, bch_verify_regular};
use md_codec::bch_decode::decode_regular_errors;
use md_codec::bitstream::BitWriter;
use md_codec::canonical_origin::canonical_origin;
use md_codec::tree::{Body, InternalKey, Node};
use md_codec::use_site_path::{Alternative, UseSitePath};
use md_codec::validate::{validate_explicit_origin_required, validate_unspendable_shape};
use md_codec::{
    Descriptor, Error, OriginPath, PathComponent, PathDecl, PathDeclPaths, Tag, TlvSection,
};

fn node(tag: Tag, body: Body) -> Node {
    Node { tag, body }
}

fn pk(i: u8) -> Node {
    node(Tag::PkK, Body::KeyArg { index: i })
}

fn desc(n: u8, tree: Node) -> Descriptor {
    Descriptor {
        n,
        path_decl: PathDecl {
            n,
            paths: PathDeclPaths::Shared(OriginPath { components: vec![] }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection::new_empty(),
    }
}

/// Kills bch.rs:100:5 `bch_verify_regular -> true`.
#[test]
fn bch_verify_regular_rejects_every_single_symbol_corruption() {
    let data: Vec<u8> = (0..20u8).map(|i| (i * 7 + 3) & 31).collect();
    let mut word = data.clone();
    word.extend_from_slice(&bch_create_checksum_regular("md", &data));
    assert!(bch_verify_regular("md", &word));
    for pos in 0..word.len() {
        for flip in 1..32u8 {
            let mut bad = word.clone();
            bad[pos] ^= flip;
            assert!(!bch_verify_regular("md", &bad), "pos {pos} flip {flip}");
        }
    }
}

/// Kills bch_decode.rs:268:21 (`>`->`==` and `>`->`<`): a 3-error word must
/// be corrected (BCH t = 4), which needs the locator's trailing zeros trimmed.
#[test]
fn decode_regular_errors_corrects_a_three_error_word() {
    let (pos, mag) = decode_regular_errors(0x1ac326caeb37621e7, 66).expect("3 errors correctable");
    assert_eq!(pos.len(), 3);
    assert_eq!(mag.len(), 3);
}

/// Kills bch_decode.rs:432:17 `||`->`&&` (t = 4 capacity guard): a word with
/// locator degree above 4 must be refused, not "repaired".
#[test]
fn decode_regular_errors_refuses_a_word_past_capacity() {
    assert!(decode_regular_errors(0x11bb43d931313c9b, 74).is_none());
}

/// Kills canonical_origin.rs:75:41 `&&`->`||`.
#[test]
fn sh_wsh_non_multi_has_no_canonical_origin() {
    let t = node(
        Tag::Sh,
        Body::Children(vec![node(Tag::Wsh, Body::Children(vec![pk(0)]))]),
    );
    assert!(canonical_origin(&t).is_none());
    let multi = node(
        Tag::Sh,
        Body::Children(vec![node(
            Tag::Wsh,
            Body::Children(vec![node(
                Tag::Multi,
                Body::MultiKeys {
                    k: 1,
                    indices: vec![0, 1],
                },
            )]),
        )]),
    );
    let o = canonical_origin(&multi).expect("sh(wsh(multi)) canonical");
    let v: Vec<(bool, u32)> = o.components.iter().map(|c| (c.hardened, c.value)).collect();
    assert_eq!(v, vec![(true, 48), (true, 0), (true, 0), (true, 1)]);
}

fn usp(a: u32, b: u32) -> UseSitePath {
    UseSitePath {
        multipath: Some(vec![
            Alternative {
                hardened: false,
                value: a,
            },
            Alternative {
                hardened: false,
                value: b,
            },
        ]),
        wildcard_hardened: false,
    }
}

fn tlv_with_overrides(v: Vec<(u8, UseSitePath)>) -> TlvSection {
    let mut t = TlvSection::new_empty();
    t.use_site_path_overrides = Some(v);
    t
}

/// Kills tlv.rs:122:29 `<=`->`>`.
#[test]
fn use_site_override_indices_must_be_strictly_ascending() {
    let mut w = BitWriter::new();
    tlv_with_overrides(vec![(0, usp(2, 3)), (1, usp(4, 5))])
        .write(&mut w, 2)
        .expect("ascending list writes");
    let mut w = BitWriter::new();
    let e = tlv_with_overrides(vec![(1, usp(2, 3)), (0, usp(4, 5))])
        .write(&mut w, 2)
        .unwrap_err();
    assert!(matches!(
        e,
        Error::OverrideOrderViolation {
            prev: 1,
            current: 0
        }
    ));
    let mut w = BitWriter::new();
    let e = tlv_with_overrides(vec![(1, usp(2, 3)), (1, usp(4, 5))])
        .write(&mut w, 2)
        .unwrap_err();
    assert!(matches!(e, Error::OverrideOrderViolation { .. }));
}

/// Kills validate.rs:301:66 `==`->`!=`.
#[test]
fn missing_explicit_origin_names_the_key_without_an_override() {
    let mut d = desc(2, node(Tag::Wsh, Body::Children(vec![pk(0)])));
    let one = OriginPath {
        components: vec![PathComponent {
            hardened: true,
            value: 48,
        }],
    };
    d.tlv.origin_path_overrides = Some(vec![(0, one.clone())]);
    assert!(matches!(
        validate_explicit_origin_required(&d),
        Err(Error::MissingExplicitOrigin { idx: 1 })
    ));
    d.tlv.origin_path_overrides = Some(vec![(0, one.clone()), (1, one)]);
    assert!(validate_explicit_origin_required(&d).is_ok());
}

/// Kills validate.rs:637:9 (delete `Body::Variable` arm in reject_nested_unspendable).
#[test]
fn nested_unspendable_under_a_variable_node_is_refused() {
    let liana = node(
        Tag::Tr,
        Body::Tr {
            internal_key: InternalKey::LianaUnspendable,
            tree: None,
        },
    );
    let d = desc(
        1,
        node(
            Tag::Thresh,
            Body::Variable {
                k: 1,
                children: vec![liana],
            },
        ),
    );
    assert!(matches!(
        validate_unspendable_shape(&d),
        Err(Error::UnspendableNotRootTr)
    ));
}

fn liana_root(tree: Node) -> Descriptor {
    desc(
        3,
        node(
            Tag::Tr,
            Body::Tr {
                internal_key: InternalKey::LianaUnspendable,
                tree: Some(Box::new(tree)),
            },
        ),
    )
}

fn sma() -> Node {
    node(
        Tag::SortedMultiA,
        Body::MultiKeys {
            k: 2,
            indices: vec![0, 1, 2],
        },
    )
}

/// Kills validate.rs:661:9 (delete `Body::Children` arm in contains_sortedmulti_a).
#[test]
fn sortedmulti_a_under_a_children_node_is_refused_under_liana_root() {
    let d = liana_root(node(Tag::TapTree, Body::Children(vec![sma(), pk(0)])));
    assert!(matches!(
        validate_unspendable_shape(&d),
        Err(Error::UnspendableWithSortedMultiA)
    ));
}

/// Kills validate.rs:662:9 (delete `Body::Variable` arm in contains_sortedmulti_a).
#[test]
fn sortedmulti_a_under_a_variable_node_is_refused_under_liana_root() {
    let d = liana_root(node(
        Tag::Thresh,
        Body::Variable {
            k: 1,
            children: vec![sma()],
        },
    ));
    assert!(matches!(
        validate_unspendable_shape(&d),
        Err(Error::UnspendableWithSortedMultiA)
    ));
}
