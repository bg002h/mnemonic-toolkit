//! End-to-end round-trip smoke tests for v0.11.

use md_codec::decode::decode_payload;
use md_codec::encode::{Descriptor, encode_payload};
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;

fn bip84_path() -> OriginPath {
    OriginPath {
        components: vec![
            PathComponent {
                hardened: true,
                value: 84,
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

#[test]
fn bip84_single_sig_round_trip() {
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(bip84_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: TlvSection::new_empty(),
    };
    let (bytes, total_bits) = encode_payload(&d).unwrap();
    let d2 = decode_payload(&bytes, total_bits).unwrap();
    assert_eq!(d, d2);
}

#[test]
fn bip84_single_sig_payload_bit_count() {
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(bip84_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: TlvSection::new_empty(),
    };
    let (_bytes, total_bits) = encode_payload(&d).unwrap();
    // v0.30: header(5) + path-decl(5+26=31) + use-site(16) + tree(Tag::Wpkh
    // 6-bit + kiw=0 at n=1) + TLV(0) = 58 bits. kiw drops to 0 (v0.30 §7
    // formula ⌈log₂(n)⌉ at n=1 is 0); Wpkh tag widened to 6 bits in Phase A.
    // Net unchanged vs the v0.18 pin of 58.
    assert_eq!(total_bits, 58);
}

fn bip48_path() -> OriginPath {
    OriginPath {
        components: vec![
            PathComponent {
                hardened: true,
                value: 48,
            },
            PathComponent {
                hardened: true,
                value: 0,
            },
            PathComponent {
                hardened: true,
                value: 0,
            },
            PathComponent {
                hardened: true,
                value: 2,
            },
        ],
    }
}

#[test]
fn bip48_2of3_sortedmulti_round_trip() {
    let d = Descriptor {
        n: 3,
        path_decl: PathDecl {
            n: 3,
            paths: PathDeclPaths::Shared(bip48_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::SortedMulti,
                body: Body::MultiKeys {
                    k: 2,
                    indices: (0..3).collect(),
                },
            }]),
        },
        tlv: TlvSection::new_empty(),
    };
    let (bytes, total_bits) = encode_payload(&d).unwrap();
    let d2 = decode_payload(&bytes, total_bits).unwrap();
    assert_eq!(d, d2);
}

#[test]
fn bip84_emit_md1_string() {
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(bip84_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: TlvSection::new_empty(),
    };
    let s = md_codec::encode::encode_md1_string(&d).unwrap();
    assert!(s.starts_with("md1"));
}

#[test]
fn bip84_md1_string_round_trip() {
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(bip84_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: TlvSection::new_empty(),
    };
    let s = md_codec::encode::encode_md1_string(&d).unwrap();
    let d2 = md_codec::decode::decode_md1_string(&s).unwrap();
    assert_eq!(d, d2);
}

#[test]
fn bip48_2of3_md1_string_round_trip() {
    let d = Descriptor {
        n: 3,
        path_decl: PathDecl {
            n: 3,
            paths: PathDeclPaths::Shared(bip48_path()),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::SortedMulti,
                body: Body::MultiKeys {
                    k: 2,
                    indices: (0..3).collect(),
                },
            }]),
        },
        tlv: TlvSection::new_empty(),
    };
    let s = md_codec::encode::encode_md1_string(&d).unwrap();
    let d2 = md_codec::decode::decode_md1_string(&s).unwrap();
    assert_eq!(d, d2);
}

#[test]
fn bip86_taproot_md1_string_round_trip() {
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(OriginPath {
                components: vec![
                    PathComponent {
                        hardened: true,
                        value: 86,
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
            }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: None,
            },
        },
        tlv: TlvSection::new_empty(),
    };
    let s = md_codec::encode::encode_md1_string(&d).unwrap();
    let d2 = md_codec::decode::decode_md1_string(&s).unwrap();
    assert_eq!(d, d2);
}

#[test]
fn vault_or_d_pk_older_md1_string_round_trip() {
    // wsh(or_d(pk(@0), and_v(v:older(144), pk(@1))))
    let d = Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Shared(OriginPath {
                components: vec![
                    PathComponent {
                        hardened: true,
                        value: 84,
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
            }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::OrD,
                body: Body::Children(vec![
                    Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg { index: 0 },
                    },
                    Node {
                        tag: Tag::AndV,
                        body: Body::Children(vec![
                            Node {
                                tag: Tag::Verify,
                                body: Body::Children(vec![Node {
                                    tag: Tag::Older,
                                    body: Body::Timelock(144),
                                }]),
                            },
                            Node {
                                tag: Tag::PkK,
                                body: Body::KeyArg { index: 1 },
                            },
                        ]),
                    },
                ]),
            }]),
        },
        tlv: TlvSection::new_empty(),
    };
    let s = md_codec::encode::encode_md1_string(&d).unwrap();
    let d2 = md_codec::decode::decode_md1_string(&s).unwrap();
    assert_eq!(d, d2);
}
