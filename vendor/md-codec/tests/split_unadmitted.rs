//! `chunk::split_unadmitted`: the chunked form of re-emitting a card that
//! already exists, with no mint-time admission policy.
//!
//! Refugium SPEC §9 Q17: an `mr1` card that already exists must always be
//! readable and printable, including the md1 card it carries, even when md's
//! mint policy would refuse that md1 today. `split` admits (it calls
//! `encode_payload`); `split_unadmitted` does not. These rows pin that the
//! two agree byte-for-byte on everything `split` accepts, and that
//! `split_unadmitted` still emits (and round-trips) the shapes `split`
//! refuses, one per rule family (F-217, SPEC §6).

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/common/vendored.rs"
));

// NOT `use md_codec::encode::Descriptor;`: `vendored.rs` above already
// imports `md_codec::Descriptor` (E0252 otherwise).
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, InternalKey, Node};
use md_codec::use_site_path::UseSitePath;

/// Copied from `tests/mint_policy_does_not_reach_decode.rs`: two slots, one
/// REAL shared `(fingerprint, origin)`, two different keys. `split` refuses
/// it (F-217, `OriginKeyContradiction`); decode reads it.
fn contradictory_card() -> Descriptor {
    let mut a = [0u8; 65];
    a[..32].copy_from_slice(&[9u8; 32]);
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let point = |n: u8| {
        let mut sk = [0u8; 32];
        sk[31] = n;
        let sk = bitcoin::secp256k1::SecretKey::from_slice(&sk).unwrap();
        bitcoin::secp256k1::PublicKey::from_secret_key(&secp, &sk).serialize()
    };
    let mut b = a;
    a[32..].copy_from_slice(&point(1));
    b[32..].copy_from_slice(&point(2));

    let mut tlv = TlvSection::new_empty();
    // A REAL fingerprint: the all-zero sentinel is exempt by design.
    tlv.fingerprints = Some(vec![
        (0u8, [0x54, 0x36, 0xd7, 0x24]),
        (1u8, [0x54, 0x36, 0xd7, 0x24]),
    ]);
    tlv.pubkeys = Some(vec![(0u8, a), (1u8, b)]);
    Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Shared(OriginPath {
                components: vec![
                    PathComponent {
                        hardened: true,
                        value: 48,
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
                tag: Tag::SortedMulti,
                body: Body::MultiKeys {
                    k: 2,
                    indices: vec![0, 1],
                },
            }]),
        },
        tlv,
    }
}

/// Copied from `tests/mint_policy_does_not_reach_decode.rs`: a kind-1 (Liana
/// unspendable) `tr()` with a `sortedmulti_a` leaf. `split` refuses it
/// (SPEC §6 row 1, `UnspendableWithSortedMultiA`); decode reads it.
fn kind_1_card_with_a_sortedmulti_a_leaf() -> Descriptor {
    let leaf = Node {
        tag: Tag::SortedMultiA,
        body: Body::MultiKeys {
            k: 2,
            indices: vec![0, 1, 2],
        },
    };
    let tree = Node {
        tag: Tag::Tr,
        body: Body::Tr {
            internal_key: InternalKey::LianaUnspendable,
            tree: Some(Box::new(leaf)),
        },
    };
    Descriptor {
        n: 3,
        path_decl: PathDecl {
            n: 3,
            paths: PathDeclPaths::Shared(OriginPath {
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
                        value: 3,
                    },
                ],
            }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection::new_empty(),
    }
}

fn reassemble_strings(chunks: &[String]) -> Descriptor {
    let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
    md_codec::chunk::reassemble(&refs).expect("split_unadmitted output must reassemble")
}

/// On everything `split` accepts, `split_unadmitted` emits identical strings.
/// The floor keeps this from passing on an empty corpus.
#[test]
fn split_unadmitted_matches_split_on_corpus() {
    let names = all_vendored_vector_names();
    let mut compared = 0usize;
    for name in &names {
        let d = decode_vendored(&load_vendored_phrase(name))
            .unwrap_or_else(|e| panic!("{name}: vendored vector must decode: {e:?}"));
        let admitted = md_codec::split(&d).unwrap_or_else(|e| panic!("{name}: split: {e:?}"));
        let unadmitted = md_codec::split_unadmitted(&d)
            .unwrap_or_else(|e| panic!("{name}: split_unadmitted: {e:?}"));
        assert_eq!(admitted, unadmitted, "{name}: strings differ");
        compared += 1;
    }
    assert!(
        compared >= 60,
        "only {compared} vendored vectors compared; the corpus holds 68"
    );
}

/// F-217 shape: `split` refuses it, `split_unadmitted` emits it.
#[test]
fn split_unadmitted_accepts_f217_shape() {
    let card = contradictory_card();
    let err = md_codec::split(&card).expect_err("split must still refuse the F-217 shape");
    assert!(
        matches!(err, md_codec::Error::OriginKeyContradiction { .. }),
        "expected OriginKeyContradiction, got {err:?}"
    );
    let chunks = md_codec::split_unadmitted(&card)
        .expect("an existing card must re-emit even when minting it is refused");
    assert!(!chunks.is_empty());
}

/// The F-217 shape's unadmitted chunks reassemble to the input.
#[test]
fn split_unadmitted_roundtrip_f217() {
    let card = contradictory_card();
    let chunks = md_codec::split_unadmitted(&card).expect("split_unadmitted");
    // The Refugium re-emit this exists for is a chunked card: pin that the
    // refused shape takes more than one chunk and still round-trips.
    assert!(
        chunks.len() > 1,
        "expected a multi-chunk set, got {}",
        chunks.len()
    );
    assert_eq!(reassemble_strings(&chunks), card);
}

/// SPEC §6 kind-1 `sortedmulti_a` shape: `split` refuses it,
/// `split_unadmitted` emits it and it round-trips.
#[test]
fn split_unadmitted_accepts_sortedmulti_a_unspendable() {
    let card = kind_1_card_with_a_sortedmulti_a_leaf();
    let err = md_codec::split(&card).expect_err("split must still refuse the SPEC §6 row-1 shape");
    assert!(
        matches!(err, md_codec::Error::UnspendableWithSortedMultiA),
        "expected UnspendableWithSortedMultiA, got {err:?}"
    );
    let chunks = md_codec::split_unadmitted(&card)
        .expect("an existing card must re-emit even when minting it is refused");
    assert_eq!(reassemble_strings(&chunks), card);
}

/// On the refused F-217 shape, every chunk header carries the chunk-set id
/// derived from the (non-admitting) md1 encoding id. `reassemble` already
/// enforces this via `ChunkSetIdMismatch`; this row states it directly.
#[test]
fn split_unadmitted_chunk_set_id_of_refused_shape() {
    let card = contradictory_card();
    let expected = md_codec::derive_chunk_set_id(
        &md_codec::compute_md1_encoding_id(&card).expect("encoding id of an existing card"),
    );
    let chunks = md_codec::split_unadmitted(&card).expect("split_unadmitted");
    assert!(!chunks.is_empty(), "no chunks to check");
    for (i, s) in chunks.iter().enumerate() {
        let (bytes, _bits) = md_codec::codex32::unwrap_string(s).expect("unwrap chunk");
        let mut r = md_codec::bitstream::BitReader::new(&bytes);
        let header = md_codec::ChunkHeader::read(&mut r).expect("chunk header");
        assert_eq!(header.chunk_set_id, expected, "chunk {i}");
    }
}
