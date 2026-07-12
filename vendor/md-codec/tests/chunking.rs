//! Multi-card chunking round-trip tests.

use md_codec::chunk::{derive_chunk_set_id, split};
use md_codec::encode::Descriptor;
use md_codec::identity::compute_md1_encoding_id;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;

fn small_descriptor() -> Descriptor {
    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
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
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: TlvSection::new_empty(),
    }
}

#[test]
fn small_descriptor_splits_into_one_chunk() {
    let d = small_descriptor();
    let chunks = split(&d).unwrap();
    assert_eq!(chunks.len(), 1);
    for c in &chunks {
        assert!(c.starts_with("md1"));
    }
}

#[test]
fn chunk_set_id_matches_md1_encoding_id_top_20_bits() {
    let d = small_descriptor();
    let md1_id = compute_md1_encoding_id(&d).unwrap();
    let derived = derive_chunk_set_id(&md1_id);
    let bytes = md1_id.as_bytes();
    let expected = ((bytes[0] as u32) << 12) | ((bytes[1] as u32) << 4) | ((bytes[2] as u32) >> 4);
    assert_eq!(derived, expected);
}

#[test]
fn small_descriptor_split_then_reassemble() {
    use md_codec::chunk::reassemble;
    let d = small_descriptor();
    let chunks = split(&d).unwrap();
    let chunk_refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
    let d2 = reassemble(&chunk_refs).unwrap();
    assert_eq!(d, d2);
}

#[test]
fn single_string_payload_bit_limit_matches_regular_form() {
    // Sanity-check the F2 hot-fix: 64 data symbols × 5 bits = 320 (regular-form
    // codex32). v0.11 originally set 75 × 5 = 375 (long-form), but long-form was
    // dropped in v0.12.0.
    assert_eq!(md_codec::chunk::SINGLE_STRING_PAYLOAD_BIT_LIMIT, 320);
}

fn deep_path_descriptor() -> Descriptor {
    // Build a single-sig wpkh template with a maximally deep BIP 32 path.
    // 15 hardened components, each costing 1 + 4 + 7 = 12 bits (LP4-ext for
    // value < 128 takes 4-bit L + 7-bit payload), total path body ~180 bits.
    // Plus header (5) + n (5) + path-depth (4) + use-site (16) + tree (5) + TLV (0)
    // gives roughly 215 bits — still single-string under the new 320-bit limit.
    let mut components = Vec::new();
    for i in 0..15u32 {
        components.push(PathComponent {
            hardened: true,
            value: i + 1,
        });
    }
    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(OriginPath { components }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: TlvSection::new_empty(),
    }
}

fn multi_chunk_descriptor() -> Descriptor {
    // Build a divergent-path 4-cosigner wallet with 15 hardened path components per
    // cosigner. Per-cosigner path body is ~180 bits; 4 cosigners → ~720 bits of
    // path-decl alone, plus tree and TLV — comfortably above the 320-bit
    // single-string limit, so chunking is required.
    let mut paths = Vec::new();
    for cosigner in 0..4u32 {
        let mut components = Vec::new();
        for i in 0..15u32 {
            components.push(PathComponent {
                hardened: true,
                value: cosigner * 100 + i + 1,
            });
        }
        paths.push(OriginPath { components });
    }
    Descriptor {
        n: 4,
        path_decl: PathDecl {
            n: 4,
            paths: PathDeclPaths::Divergent(paths),
        },
        use_site_path: UseSitePath::standard_multipath(),
        // v0.31: root tag must be in {Sh, Wsh, Wpkh, Pkh, Tr}; wrap SortedMulti in Wsh.
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::SortedMulti,
                body: Body::MultiKeys {
                    k: 2,
                    indices: (0..4).collect(),
                },
            }]),
        },
        tlv: TlvSection::new_empty(),
    }
}

#[test]
fn deep_path_descriptor_still_single_string() {
    // Sanity that the new 320-bit limit still accommodates a moderately-deep
    // single-sig wallet.
    let d = deep_path_descriptor();
    let chunks = split(&d).unwrap();
    assert_eq!(chunks.len(), 1, "deep single-sig should fit in one chunk");
}

#[test]
fn multi_chunk_descriptor_splits_and_reassembles() {
    use md_codec::chunk::reassemble;
    let d = multi_chunk_descriptor();
    let chunks = split(&d).unwrap();
    assert!(
        chunks.len() >= 2,
        "expected multi-chunk emission, got {}",
        chunks.len()
    );
    for c in &chunks {
        assert!(c.starts_with("md1"));
    }
    let chunk_refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
    let d2 = reassemble(&chunk_refs).unwrap();
    assert_eq!(d, d2);
}

/// F-A2: `decode_md1_string` must auto-dispatch a chunked single-string
/// (chunked-flag set on the first symbol) through the chunk-reassembly path
/// instead of the single-payload primitive. Before the fix this returned
/// `WireVersionMismatch { got: 9 }` (the version field misread across the
/// chunk-header layout); after the fix it round-trips.
#[test]
fn decode_md1_string_auto_dispatches_single_chunk() {
    let d = small_descriptor();
    let chunks = split(&d).unwrap();
    assert_eq!(chunks.len(), 1, "fixture must be a single chunk");
    let decoded =
        md_codec::decode::decode_md1_string(&chunks[0]).expect("chunked single string decodes");
    assert_eq!(decoded, d, "chunked single-string decode must round-trip");
}

/// F-A2 recovery-safety: a non-chunked single-string still decodes via the
/// single-payload path (chunked-flag = 0 for the all-even usable version set
/// {4,8,12}); the dispatch never diverts a currently-decoding input.
#[test]
fn decode_md1_string_non_chunked_unchanged() {
    let d = small_descriptor();
    let s = md_codec::encode::encode_md1_string(&d).expect("single-string encodes");
    let decoded = md_codec::decode::decode_md1_string(&s).expect("non-chunked decodes");
    assert_eq!(decoded, d);
}

/// F-A2 (post-impl Minor-2): a lone chunk `i`-of-`N` (N≥2) fed to the
/// single-string entry point must dispatch and then fail closed as an
/// incomplete set — never silently decode a partial payload.
#[test]
fn decode_md1_string_lone_chunk_of_multi_is_incomplete() {
    let d = multi_chunk_descriptor();
    let chunks = split(&d).unwrap();
    assert!(chunks.len() >= 2, "fixture must be multi-chunk");
    let err = md_codec::decode::decode_md1_string(&chunks[1]).unwrap_err();
    assert!(
        matches!(err, md_codec::Error::ChunkSetIncomplete { .. }),
        "lone chunk-of-N must yield ChunkSetIncomplete, got {err:?}"
    );
}

fn near_cap_descriptor() -> Descriptor {
    // Push toward 64 chunks via a giant unknown TLV. The wire-format
    // encoder preserves unknown TLVs verbatim, so we can synthesize a
    // payload of arbitrary size by stuffing the unknown vec.
    //
    // 64 chunks × 320 bits = 20480 bits ≈ 2560 bytes. Account for the
    // chunk-header overhead and the TLV framing — aim for ~2700 bytes
    // of unknown payload to land just under the cap.
    use md_codec::tlv::TlvSection;
    let big_payload: Vec<u8> = (0..2400).map(|i| (i % 251) as u8).collect();
    let big_bit_len = big_payload.len() * 8;
    let mut tlv = TlvSection::new_empty();
    // Tag 0x10 — unknown to v0.13 (well beyond 0x00..0x03 known tags).
    tlv.unknown.push((0x10, big_payload, big_bit_len));
    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(OriginPath {
                components: vec![PathComponent {
                    hardened: true,
                    value: 84,
                }],
            }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv,
    }
}

fn over_cap_descriptor() -> Descriptor {
    // Same shape as near-cap but inflated past 64 chunks.
    // 64 × 320 = 20480 bits = 2560 bytes; add ~600 bytes to push over.
    use md_codec::tlv::TlvSection;
    let big_payload: Vec<u8> = (0..2700).map(|i| (i % 251) as u8).collect();
    let big_bit_len = big_payload.len() * 8;
    let mut tlv = TlvSection::new_empty();
    tlv.unknown.push((0x10, big_payload, big_bit_len));
    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(OriginPath {
                components: vec![PathComponent {
                    hardened: true,
                    value: 84,
                }],
            }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv,
    }
}

#[test]
fn near_cap_descriptor_splits_to_at_most_64_chunks_and_round_trips() {
    use md_codec::chunk::reassemble;
    let d = near_cap_descriptor();
    let chunks = split(&d).unwrap();
    assert!(
        chunks.len() <= 64,
        "near-cap descriptor must produce ≤64 chunks (got {})",
        chunks.len()
    );
    assert!(
        chunks.len() >= 8,
        "near-cap descriptor should produce many chunks (got {})",
        chunks.len()
    );
    let chunk_refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
    let d2 = reassemble(&chunk_refs).unwrap();
    assert_eq!(d, d2);
}

#[test]
fn over_cap_descriptor_rejected_with_chunk_count_exceeds_max() {
    use md_codec::error::Error;
    let d = over_cap_descriptor();
    let err = split(&d).unwrap_err();
    assert!(
        matches!(err, Error::ChunkCountExceedsMax { needed } if needed > 64),
        "expected ChunkCountExceedsMax with needed > 64, got {:?}",
        err
    );
}

#[test]
fn tampered_chunk_rejected_by_bch_verify() {
    use md_codec::chunk::reassemble;
    let d = multi_chunk_descriptor();
    let chunks = split(&d).unwrap();
    // Corrupt one symbol of the first chunk's body (skip past "md1" HRP+sep).
    let mut tampered = chunks[0].clone().into_bytes();
    let pos = "md1".len();
    let original = tampered[pos];
    // Swap to the next valid bech32 character (lookup-free: 'q' or 'p'); ensure
    // it changes.
    tampered[pos] = if original == b'q' { b'p' } else { b'q' };
    let tampered_str = String::from_utf8(tampered).unwrap();
    let mut chunk_refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
    chunk_refs[0] = tampered_str.as_str();
    let result = reassemble(&chunk_refs);
    assert!(result.is_err(), "tampered chunk should fail BCH verify");
}
