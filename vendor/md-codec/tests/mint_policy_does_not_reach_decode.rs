//! A card that would be REFUSED at mint must still READ.
//!
//! The encode-side refusals were added on purpose rather than on decode, and
//! `validate_origin_key_consistency` states why in its own doc: "Enforced on
//! encode rather than on decode, deliberately: this stops new impossible cards
//! without making already-written ones unreadable, and a card that cannot be
//! read is a backup that cannot be restored."
//!
//! The implementation leaked. `chunk::reassemble` -- a DECODE path -- verifies
//! a chunk set by recomputing the md1 encoding id; `compute_md1_encoding_id`
//! called `encode_payload`; `encode_payload` applies admission policy. So each
//! rule added to the mint path retroactively made older cards of that shape
//! undecodable.
//!
//! MEASURED before the fix: a 2-of-2 emitted by the shipped toolkit
//! (`--descriptor` with `--slot @N.path=`, whose paths it silently dropped)
//! stopped reading, with `verify-bundle` reporting
//! `md1_decode: fail OriginKeyContradiction` over a backup whose keys were
//! perfectly intact.
//!
//! This pins the CLASS, not the instance: mint refuses, decode still reads.
//! It is the test that would have caught the WIF regression, the one this
//! wallet shape hit, and whichever rule gets added next.

use md_codec::encode::Descriptor;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, InternalKey, Node};
use md_codec::use_site_path::UseSitePath;

/// Two slots, one REAL shared `(fingerprint, origin)`, two different keys --
/// the exact shape `bundle --descriptor` used to mint before it learned to
/// carry per-slot paths.
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
    // A REAL fingerprint: the all-zero sentinel is exempt by design (it names
    // no master), so using it here would test the exemption, not this class.
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

/// The control: minting it is still refused. Without this the test below could
/// pass because the rule stopped firing anywhere.
#[test]
fn minting_it_is_still_refused() {
    let err = md_codec::encode::encode_payload(&contradictory_card())
        .expect_err("one real (fingerprint, path) over two keys must not be MINTED");
    assert!(
        matches!(err, md_codec::Error::OriginKeyContradiction { .. }),
        "expected OriginKeyContradiction, got {err:?}"
    );
}

/// The point: its ID still computes, so a card in that shape still READS.
#[test]
fn hashing_it_is_not_minting_it() {
    md_codec::identity::compute_md1_encoding_id(&contradictory_card()).expect(
        "hashing an EXISTING card must not apply mint-time admission policy -- \
         a backup that cannot be read is worse than one that should not have \
         been written",
    );
}

/// Stage 1b task 7, fix round 2 (M2): this file's header says it pins
/// "the CLASS, not the instance ... and whichever rule gets added next" --
/// task 7 added FOUR rules next (SPEC §6), and this file was untouched.
/// Review measured the gap directly: with the §6 checks moved out of the
/// `Admission::Enforce` guard in `encode_payload_inner`, this file's two
/// existing tests (above) stayed GREEN -- because they exercise the
/// F-217/F-218 shape, not a §6 one -- while the NEW `encode.rs` unit test
/// (`a_refused_shape_still_decodes_so_existing_cards_never_stop_reading`)
/// went RED. So the file claiming to be the class gate did not cover the
/// class it claims to.
///
/// A kind-1 (Liana unspendable) `tr()` with a `sortedmulti_a` leaf -- SPEC
/// §6 row 1, `Error::UnspendableWithSortedMultiA`. Same recipe as
/// `crates/md-codec/src/encode.rs`'s `in_crate_tr_liana_with_sortedmulti_a_leaf`
/// unit-test fixture (private to that crate's own `#[cfg(test)]` module, so
/// unreachable from here) and `tests/liana_unspendable.rs`'s vendored one
/// (`tr_liana_with_sortedmulti_a_leaf`, via `kind1_from_vector` -- also
/// unreachable, `tests/common/` is a different crate root) -- rebuilt
/// in-crate a third time rather than shared, same as those two already are.
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

/// The control, mirroring `minting_it_is_still_refused` above: without this,
/// the test below could pass because the §6 row-1 rule stopped firing
/// anywhere, not because it correctly stays out of the read path.
#[test]
fn minting_a_kind_1_sortedmulti_a_card_is_still_refused() {
    let err = md_codec::encode::encode_payload(&kind_1_card_with_a_sortedmulti_a_leaf())
        .expect_err("kind 1 with a sortedmulti_a leaf must not be MINTED (SPEC §6 row 1)");
    assert!(
        matches!(err, md_codec::Error::UnspendableWithSortedMultiA),
        "expected UnspendableWithSortedMultiA, got {err:?}"
    );
}

/// The point, routed through the REAL leak path. The 2026-09-19 regression
/// this file exists to pin travelled `chunk::reassemble` ->
/// `compute_md1_encoding_id` -> `encode_payload` -- NOT a direct call to
/// `encode_payload_inner(SkipPolicy)`. `encode.rs`'s own unit test
/// (`a_refused_shape_still_decodes_so_existing_cards_never_stop_reading`)
/// calls `encode_payload_inner` directly, which is a parallel path with the
/// same Admission plumbing but is not the path a real caller travels -- it
/// pins the MECHANISM, this test pins the PRODUCT that actually leaked.
#[test]
fn hashing_a_refused_kind_1_card_through_compute_md1_encoding_id_still_reads() {
    md_codec::identity::compute_md1_encoding_id(&kind_1_card_with_a_sortedmulti_a_leaf()).expect(
        "hashing an EXISTING kind-1 card must not apply mint-time admission policy -- \
         a backup that cannot be read is worse than one that should not have been written",
    );
}

/// F-639 (F-449 stage 2): `encode_payload_unadmitted` is the public
/// comparison/hashing serialiser `md verify` uses. It must serialise every
/// mint-refused shape this file pins -- the F-217 contradiction and the §6
/// kind-1 `sortedmulti_a` card -- and its bytes must decode back to the same
/// card, so a comparison over them compares the card, not an error.
#[test]
fn the_unadmitted_serialiser_reads_every_mint_refused_shape() {
    for (label, card) in [
        ("F-217 origin/key contradiction", contradictory_card()),
        (
            "SPEC §6 kind-1 sortedmulti_a",
            kind_1_card_with_a_sortedmulti_a_leaf(),
        ),
    ] {
        assert!(
            md_codec::encode::encode_payload(&card).is_err(),
            "{label}: the control -- minting must still refuse"
        );
        let (bytes, bits) = md_codec::encode_payload_unadmitted(&card)
            .unwrap_or_else(|e| panic!("{label}: comparison must not apply mint policy: {e:?}"));
        let back = md_codec::decode::decode_payload(&bytes, bits)
            .unwrap_or_else(|e| panic!("{label}: its bytes must decode: {e:?}"));
        assert_eq!(back, card, "{label}: round trip");
    }
}

/// Emits the kind-1 card as an md1 string for md-cli's F-639 verify test,
/// so the fixture there is re-derivable: `cargo test ... -- --ignored
/// --nocapture print_the_kind_1_card`.
#[test]
#[ignore]
fn print_the_kind_1_card() {
    let (bytes, bits) =
        md_codec::encode_payload_unadmitted(&kind_1_card_with_a_sortedmulti_a_leaf()).unwrap();
    println!("{}", md_codec::codex32::wrap_payload(&bytes, bits).unwrap());
}
