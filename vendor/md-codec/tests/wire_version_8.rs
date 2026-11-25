//! Stage 1b task 2: wire version 8 (the header, the accepted-version set, and
//! `Descriptor::wire_version()` derived from the tree — not assumed).
//!
//! Version-4 bytes are unchanged in this task (stage 1a's byte-equality gate
//! proves it over all 65 vendored vectors). This gate proves the plumbing:
//! the dispatch-safety arithmetic, the accepted version set, the mismatch
//! message naming both accepted versions, and that `wire_version()` reads
//! the tree (kind 1 ⇒ 8, kind 0 ⇒ 4) rather than assuming `Slot(0)`/kind 0
//! everywhere.
//!
//! Task 2 itself deliberately did NOT round-trip a `LianaUnspendable`
//! descriptor through `encode_payload`/`decode_payload` — `wire_version()`
//! already returned 8 for one, but `Body::Tr`'s wire encoding didn't carry
//! a kind bit yet, so such a round trip mis-decoded as `NumsPoint` (and a
//! chunked one failed to reassemble). Stage 1b task 3 closed that: the two
//! tests below, appended by task 3, DO round-trip a `LianaUnspendable`
//! descriptor now that `write_node`/`read_node` carry the kind bit at
//! version 8. See `Descriptor::wire_version`'s doc comment for the
//! now-historical detail of the state task 3 closed.

use md_codec::error::Error;
use md_codec::header::Header;
use md_codec::{decode_payload, encode_payload};

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/common/vendored.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/common/liana.rs"
));

#[test]
fn version_8_is_even_so_the_dispatch_routes_it_as_single_payload() {
    // decode.rs:191-193 reads bit 0 of the FIRST SYMBOL as the chunked flag.
    // For a single payload that bit IS v0, so every usable version must be
    // EVEN. Version 5 would route a single-string plate into the chunk
    // reassembler, which then reports WireVersionMismatch{got:2}.
    for divergent in [false, true] {
        let sym = (u16::from(divergent) << 4) | u16::from(Header::WF_UNSPENDABLE_VERSION);
        assert_eq!(
            (((sym << 3) as u8) >> 3) & 1,
            0,
            "version {} dispatches as CHUNKED",
            Header::WF_UNSPENDABLE_VERSION
        );
    }
}

#[test]
fn the_decoder_accepts_4_and_8_and_refuses_everything_else() {
    for v in 0u8..16 {
        assert_eq!(
            Header::is_supported_version(v),
            matches!(v, 4 | 8),
            "version {v}"
        );
    }
}

#[test]
fn wire_version_is_derived_from_the_tree_not_assumed() {
    // G-2: do NOT assume Slot(0). A non-zero slot is constructible.
    assert_eq!(
        kind1_from_vector("keyed_compose_tr_nums_three_leaves").wire_version(),
        8
    );
    // Review round 1, M1: this pre-existing loop had no non-zero-count
    // guard, unlike task 8's new loops over the same corpus -- if
    // `all_root_tr_vectors()`'s filter ever started matching nothing, this
    // test would report `ok` having proven nothing about the corpus at all.
    let vectors = all_root_tr_vectors();
    assert!(!vectors.is_empty(), "no root tr vectors in the corpus");
    // F-449 stage 3 prerequisite: the corpus now carries wire-kind-1
    // vectors (`keyed_tr_liana_*`, `liana_taproot`), so the expected
    // version is DERIVED from the root internal key, never assumed to be 4.
    // Both populations must be non-empty, or one half proves nothing.
    let (mut v4, mut v8) = (0usize, 0usize);
    for name in &vectors {
        let d = decode_vendored(&load_vendored_phrase(name)).unwrap();
        let kind1 = matches!(
            d.tree.body,
            Body::Tr {
                internal_key: InternalKey::LianaUnspendable,
                ..
            }
        );
        let want = if kind1 { 8 } else { 4 };
        assert_eq!(d.wire_version(), want, "{name}");
        if kind1 {
            v8 += 1;
        } else {
            v4 += 1;
        }
    }
    assert!(v4 > 0, "no version-4 root tr vector in the corpus");
    assert!(v8 > 0, "no version-8 (kind-1) root tr vector in the corpus");
}

#[test]
fn the_mismatch_message_names_the_accepted_set_not_a_single_version() {
    let s = Error::WireVersionMismatch { got: 9 }.to_string();
    assert!(s.contains('9'), "must name what it got: {s}");
    assert!(!s.contains("expected 4"), "stale single-version claim: {s}");
    assert!(
        s.contains('4') && s.contains('8'),
        "must name {{4, 8}}: {s}"
    );
}

// Stage 1b task 3: the kind bit itself.

#[test]
fn kind_0_and_kind_1_over_the_same_tree_encode_to_different_bytes() {
    let k0 = decode_vendored(&load_vendored_phrase("keyed_compose_tr_nums_three_leaves")).unwrap();
    let k1 = kind1_from_vector("keyed_compose_tr_nums_three_leaves");
    assert_ne!(
        encode_payload(&k0).unwrap().0,
        encode_payload(&k1).unwrap().0
    );
}

#[test]
fn a_kind_1_tree_round_trips_at_v8() {
    let d = kind1_from_vector("keyed_compose_tr_nums_three_leaves");
    let (bytes, bits) = encode_payload(&d).unwrap();
    assert_eq!(decode_payload(&bytes, bits).unwrap(), d);
}

/// Fix round 1 I-2. A round-trip test CANNOT catch a polarity inversion:
/// if `write_node` and `read_node` both flip 0<->1 for the kind bit, every
/// round-trip test in this file (including `a_kind_1_tree_round_trips_at_v8`
/// above) stays green, because encode and decode still agree with EACH
/// OTHER -- just not with the spec's chosen polarity (0 = NumsPoint,
/// 1 = LianaUnspendable). This was measured by mutation: inverting the bit
/// on both sides passed 1445/1445. It matters beyond elegance -- stage 3
/// ports this bit to Go, and if the two implementations disagree on
/// polarity, a plate written by one is silently misread by the other, as
/// the other kind.
///
/// So this test reads the RAW bit off the wire with a `BitReader`, never
/// through `read_node`'s `InternalKey` decode -- a symmetric inversion of
/// both `write_node` and `read_node` cannot hide from it, because it never
/// calls `read_node` at all. DO NOT "simplify" this back to a round trip.
#[test]
fn the_kind_bit_polarity_is_pinned_on_the_wire_not_just_round_tripped() {
    use md_codec::bitstream::{BitReader, BitWriter};
    use md_codec::tag::Tag;
    use md_codec::tree::{Body, Node, write_node};

    // The raw kind-bit value `write_node` puts on the wire for `internal_key`
    // at version 8, read directly off the bytes -- bypassing `read_node`
    // entirely. `key_index_width = 0` because neither NumsPoint nor
    // LianaUnspendable ever writes a key_index field.
    let raw_kind_bit = |internal_key: InternalKey| -> u64 {
        let node = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                internal_key,
                tree: None,
            },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &node, 0, Header::WF_UNSPENDABLE_VERSION).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        Tag::read(&mut r).unwrap(); // consume the tag (6 bits, Tr = 0x01)
        let is_nums = r.read_bits(1).unwrap();
        assert_eq!(is_nums, 1, "NumsPoint/LianaUnspendable must set is_nums=1");
        r.read_bits(1).unwrap() // the kind bit itself, per SPEC §3d
    };

    assert_eq!(
        raw_kind_bit(InternalKey::NumsPoint),
        0,
        "SPEC §3d: kind 0 (NumsPoint) must be bit value 0 on the wire"
    );
    assert_eq!(
        raw_kind_bit(InternalKey::LianaUnspendable),
        1,
        "SPEC §3d: kind 1 (LianaUnspendable) must be bit value 1 on the wire"
    );

    // Golden hex, the strongest pin: `Tag::Tr` (0b000001) | is_nums(1) |
    // kind(1) | has_tree(0) = 0b000001_1_1_0 = 9 bits, MSB-aligned and
    // zero-padded to 2 bytes = [0x07, 0x00] for LianaUnspendable at v8.
    let node = Node {
        tag: Tag::Tr,
        body: Body::Tr {
            internal_key: InternalKey::LianaUnspendable,
            tree: None,
        },
    };
    let mut w = BitWriter::new();
    write_node(&mut w, &node, 0, Header::WF_UNSPENDABLE_VERSION).unwrap();
    assert_eq!(w.bit_len(), 9);
    assert_eq!(
        w.into_bytes(),
        vec![0x07, 0x00],
        "golden bytes for kind 1 at v8"
    );

    // F-449 stage 3 R0 m1: pin the READ side too. A reader that always
    // yields LianaUnspendable at v8, or that reads a kind bit for a Slot
    // key, round-trips every minimal vector and so passes every other test
    // in the suite; only reading GOLDEN bytes back separates it. The Slot
    // golden is `Tag::Tr` | is_nums 0 | key_index `01` (kiw 2) | has_tree 0
    // = 0b000001_0_01_0, 10 bits = [0x04, 0x80]: no kind bit at any
    // version, so a reader that consumes one mis-reads the slot index.
    use md_codec::tree::read_node;
    for (bytes, kiw, version, want, what) in [
        (
            vec![0x06, 0x00],
            0u8,
            Header::WF_UNSPENDABLE_VERSION,
            InternalKey::NumsPoint,
            "nums at v8",
        ),
        (
            vec![0x07, 0x00],
            0,
            Header::WF_UNSPENDABLE_VERSION,
            InternalKey::LianaUnspendable,
            "liana at v8",
        ),
        (
            vec![0x06],
            0,
            Header::WF_REDESIGN_VERSION,
            InternalKey::NumsPoint,
            "nums at v4 (no kind bit)",
        ),
        (
            vec![0x04, 0x80],
            2,
            Header::WF_UNSPENDABLE_VERSION,
            InternalKey::Slot(1),
            "slot 1 at v8 (no kind bit)",
        ),
    ] {
        let mut r = BitReader::new(&bytes);
        let n = read_node(&mut r, kiw, version).unwrap_or_else(|e| panic!("{what}: {e}"));
        let expected = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                internal_key: want,
                tree: None,
            },
        };
        assert_eq!(n, expected, "{what}: golden {bytes:02x?} read back wrong");
        // And the write side of the Slot golden, which the rows above lack.
        let mut w = BitWriter::new();
        write_node(&mut w, &expected, kiw, version).unwrap();
        assert_eq!(w.into_bytes(), bytes, "{what}: write side");
    }
}
