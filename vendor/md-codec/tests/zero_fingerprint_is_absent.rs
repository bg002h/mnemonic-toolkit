//! An all-zero master fingerprint is the ABSENT sentinel, not an identity.
//!
//! `validate_origin_key_consistency` refuses two slots that bind one
//! `(fingerprint, origin path)` pair to two different xpubs — correctly, since
//! BIP-32 makes that pair name exactly one key. Its own documented scope says
//! the check needs BOTH slots to carry a fingerprint: "Without one, the origin
//! path names no master, so no contradiction is provable and none is claimed."
//!
//! `[0u8; 4]` defeats that scope while satisfying its `Some(_)`. Zero is what a
//! producer writes when there IS no master to name — it is the same sentinel
//! BIP-32 uses for a depth-0 key's parent fingerprint — so two slots carrying
//! it are two ABSENCES, not two claims about one master. Refusing them reports
//! a contradiction that was never asserted.
//!
//! MEASURED, and this is why it matters beyond tidiness: `mnemonic bundle`
//! emits `[00000000/m]` for a WIF slot (a WIF has no master and no path), so a
//! legal 2-of-2 of two DISTINCT WIFs tripped this. Worse, the refusal reaches
//! DECODE — `chunk::reassemble` → `compute_md1_encoding_id` → `encode_payload`
//! → validators — so an already-engraved card of that shape stopped being
//! READABLE.
//!
//! The exemption cannot hide a real contradiction worth having: a genuine
//! master whose fingerprint is literally `00000000` is a 1-in-2^32 accident, and
//! all it loses there is one advisory refusal on a card that names its master
//! with the sentinel for "no master".

use md_codec::encode::Descriptor;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;

/// Two slots, two DIFFERENT 65-byte keys, one shared `(fingerprint, path)`.
fn two_slot_descriptor(fp: [u8; 4], path: Vec<PathComponent>) -> Descriptor {
    let mut a = [0u8; 65];
    a[..32].copy_from_slice(&[7u8; 32]);
    // A valid compressed point: 0x02 ‖ x, with an x that is on the curve.
    a[32] = 0x02;
    a[33..].copy_from_slice(&[0x11u8; 32]);
    let mut b = a;
    b[33..].copy_from_slice(&[0x22u8; 32]);

    let mut tlv = TlvSection::new_empty();
    tlv.fingerprints = Some(vec![(0u8, fp), (1u8, fp)]);
    tlv.pubkeys = Some(vec![(0u8, a), (1u8, b)]);
    Descriptor {
        n: 2,
        path_decl: PathDecl {
            n: 2,
            paths: PathDeclPaths::Shared(OriginPath { components: path }),
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

#[test]
fn a_zero_fingerprint_is_not_an_identity_claim() {
    let d = two_slot_descriptor([0, 0, 0, 0], Vec::new());
    assert!(
        md_codec::validate::validate_origin_key_consistency(&d).is_ok(),
        "two slots carrying the ABSENT-fingerprint sentinel assert nothing about \
         a master, so there is no contradiction to report"
    );
}

/// The control. Without it the exemption above could be satisfied by a
/// validator that never fires at all.
#[test]
fn a_real_shared_fingerprint_still_contradicts() {
    let d = two_slot_descriptor([0x73, 0xc5, 0xda, 0x0a], Vec::new());
    let err = md_codec::validate::validate_origin_key_consistency(&d)
        .expect_err("one REAL (fingerprint, path) naming two different keys is impossible");
    assert!(
        matches!(err, md_codec::Error::OriginKeyContradiction { .. }),
        "expected OriginKeyContradiction, got {err:?}"
    );
}

/// A zero fingerprint with a NON-empty path is still exempt: the fingerprint is
/// what names the master, and without one the path hangs off nothing.
#[test]
fn a_zero_fingerprint_with_a_path_is_also_exempt() {
    let path = vec![
        PathComponent {
            hardened: true,
            value: 48,
        },
        PathComponent {
            hardened: true,
            value: 0,
        },
    ];
    let d = two_slot_descriptor([0, 0, 0, 0], path);
    assert!(
        md_codec::validate::validate_origin_key_consistency(&d).is_ok(),
        "a path under no master still names no key"
    );
}
