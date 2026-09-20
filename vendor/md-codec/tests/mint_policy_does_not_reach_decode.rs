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
use md_codec::tree::{Body, Node};
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
