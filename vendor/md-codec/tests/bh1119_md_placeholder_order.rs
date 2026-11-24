//! bughunt-1119 row #5 (`design/FIX_bh1119_md_placeholder_order.md`): library rows e12, e14, e15, e18.
//! Placeholder order (C2, I12): canonicalisation recomputes the use-site baseline and is atomic.

use md_codec::encode::Descriptor;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::{Alternative, UseSitePath};
use md_codec::{
    Error, canonicalize_placeholder_indices, compute_md1_encoding_id,
    compute_wallet_descriptor_template_id, compute_wallet_policy_id, encode_payload,
};

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

fn comp(value: u32) -> PathComponent {
    PathComponent {
        hardened: true,
        value,
    }
}

fn shared_decl() -> PathDecl {
    PathDecl {
        n: 2,
        paths: PathDeclPaths::Shared(OriginPath {
            components: vec![comp(48), comp(0), comp(0), comp(2)],
        }),
    }
}

fn wsh_multi(indices: Vec<u8>) -> Node {
    Node {
        tag: Tag::Wsh,
        body: Body::Children(vec![Node {
            tag: Tag::Multi,
            body: Body::MultiKeys { k: 2, indices },
        }]),
    }
}

#[test]
fn e12_c2_identities_follow_the_canonical_spelling() {
    let d = md_codec::decode::decode_md1_string("md1yppq2zcyxppgzcuqqcr8qpnypdll2zu").unwrap();
    // N: D relabelled as e1's template parses.
    let mut n = d.clone();
    n.tree = wsh_multi(vec![1, 0]);
    n.use_site_path = usp(0, 1);
    n.tlv.use_site_path_overrides = Some(vec![(1, usp(2, 3))]);

    let mut canon = n.clone();
    canonicalize_placeholder_indices(&mut canon).unwrap();
    assert_eq!(canon, d, "canonicalised N must equal D");

    assert_eq!(
        compute_wallet_descriptor_template_id(&n).unwrap(),
        compute_wallet_descriptor_template_id(&d).unwrap()
    );
    assert_eq!(
        compute_md1_encoding_id(&n).unwrap(),
        compute_md1_encoding_id(&d).unwrap()
    );
    assert_eq!(
        compute_wallet_policy_id(&n).unwrap(),
        compute_wallet_policy_id(&d).unwrap()
    );
    let hex = |b: &[u8]| -> String {
        use std::fmt::Write;
        b.iter().fold(String::new(), |mut acc, x| {
            write!(acc, "{x:02x}").unwrap();
            acc
        })
    };
    assert_eq!(
        hex(compute_wallet_descriptor_template_id(&n)
            .unwrap()
            .as_bytes()),
        "ee4e423a9b62fec656e822b2022eafb8"
    );
    assert_eq!(
        hex(compute_md1_encoding_id(&n).unwrap().as_bytes()),
        "a343d85c7e3fdc1a76a2b1adee39e2c0"
    );
    assert_eq!(
        hex(compute_wallet_policy_id(&n).unwrap().as_bytes()),
        "2f10ddf928ef27bbdbf8a32c5007d61f"
    );
}

fn divergent_error_fixture() -> Descriptor {
    Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Divergent(vec![]),
        },
        use_site_path: usp(0, 1),
        tree: wsh_multi(vec![1, 0]),
        tlv: TlvSection::new_empty(),
    }
}

#[test]
fn e14_i12_error_leaves_descriptor_unchanged() {
    let mut d = divergent_error_fixture();
    let pre = d.clone();
    let err = canonicalize_placeholder_indices(&mut d).unwrap_err();
    assert!(
        matches!(err, Error::DivergentPathCountMismatch { n: 2, got: 0 }),
        "got {err:?}"
    );
    assert_eq!(d, pre, "descriptor mutated despite returned error");
}

#[test]
fn e15_i12_repair_and_retry_matches_one_shot() {
    let paths = vec![
        OriginPath {
            components: vec![comp(84)],
        },
        OriginPath {
            components: vec![comp(48)],
        },
    ];
    let mut control = divergent_error_fixture();
    control.path_decl.paths = PathDeclPaths::Divergent(paths.clone());
    canonicalize_placeholder_indices(&mut control).unwrap();

    let mut d = divergent_error_fixture();
    assert!(canonicalize_placeholder_indices(&mut d).is_err());
    d.path_decl.paths = PathDeclPaths::Divergent(paths);
    canonicalize_placeholder_indices(&mut d).unwrap();
    assert_eq!(
        d, control,
        "retry after repair diverges from one-shot control"
    );
}

#[test]
fn e18_c2_duplicate_override_refused() {
    let mut d = Descriptor {
        n: 2,
        path_decl: shared_decl(),
        use_site_path: usp(0, 1),
        tree: wsh_multi(vec![1, 0]),
        tlv: {
            let mut t = TlvSection::new_empty();
            t.use_site_path_overrides = Some(vec![(1, usp(2, 3)), (1, usp(4, 5))]);
            t
        },
    };
    let pre = d.clone();
    assert_eq!(
        canonicalize_placeholder_indices(&mut d),
        Err(Error::OverrideOrderViolation {
            prev: 1,
            current: 1
        })
    );
    assert_eq!(d, pre);
    assert!(encode_payload(&pre).is_err());
}
