//! Theme 1 — md-codec property harness.
mod common;
use common::{canon, descriptor_strategy, wire_descriptor_strategy};
use md_codec::chunk::{reassemble, split};
use md_codec::decode::{decode_md1_string, decode_payload};
use md_codec::encode::{encode_md1_string, encode_payload};
use proptest::prelude::*;

proptest! {
    // P1 — canonical-fixpoint payload bijection.
    #[test]
    fn p1_canonical_fixpoint(d in descriptor_strategy()) {
        let c = canon(&d);
        let (bytes, total_bits) = encode_payload(&c).expect("canonical encodes");
        let back = decode_payload(&bytes, total_bits).expect("canonical decodes");
        prop_assert_eq!(back, c.clone());
        let (b2, t2) = encode_payload(&d).expect("encodes");
        prop_assert_eq!((b2, t2), (bytes, total_bits));
    }

    // P2 — canonicalize-is-normalizer.
    #[test]
    fn p2_normalizer(d in descriptor_strategy()) {
        let c = canon(&d);
        let (bd, td) = encode_payload(&d).expect("encodes");
        let (bc, tc) = encode_payload(&c).expect("encodes");
        prop_assert_eq!((&bd, td), (&bc, tc));
        let back = decode_payload(&bd, td).expect("decodes");
        prop_assert_eq!(back, c);
    }

    // P3 — decode panic-freedom (decode_payload arm pins total_bits = bytes*8).
    #[test]
    fn p3_decode_payload_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..64)) {
        let total_bits = bytes.len() * 8;
        let _ = decode_payload(&bytes, total_bits);
    }
    #[test]
    fn p3_decode_str_never_panics(s in "\\PC*") {
        let _ = decode_md1_string(&s);
        let _ = reassemble(&[s.as_str()]);
    }

    // P4 — string-level round-trip. cycle-4 H6: a descriptor whose payload
    // exceeds the 80-data-symbol single-string cap legitimately rejects the
    // single-string encode (the chunked round-trip is covered by P5); for any
    // descriptor that DOES fit a single string the round-trip must hold.
    #[test]
    fn p4_string_round_trip(d in descriptor_strategy()) {
        let c = canon(&d);
        match encode_md1_string(&c) {
            Ok(s) => {
                let back = decode_md1_string(&s).expect("string decodes");
                prop_assert_eq!(back, c);
            }
            Err(md_codec::Error::PayloadTooLongForSingleString { .. }) => { /* chunked-only; see P5 */ }
            Err(e) => prop_assert!(false, "unexpected string-encode error: {e:?}"),
        }
    }

    // P5 — chunk round-trip.
    #[test]
    fn p5_chunk_round_trip(d in descriptor_strategy()) {
        let c = canon(&d);
        let chunks = split(&c).expect("splits");
        let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
        prop_assert_eq!(reassemble(&refs).expect("reassembles"), c);
    }

    // ── Cycle B (stress program): the same four properties over the
    // W tier — full-domain leaves (all 6 less-common fragments, full
    // u32/[u8;32]/[u8;20] domains) + arbitrary combinator nesting +
    // randomized TLVs. Hash-literal leaves are 166–262 bits each, so
    // nested W trees exercise multi-chunk split/reassemble under
    // property coverage for the first time. ──

    // P1(W) — canonical-fixpoint payload bijection.
    #[test]
    fn p1_w_canonical_fixpoint(d in wire_descriptor_strategy()) {
        let c = canon(&d);
        let (bytes, total_bits) = encode_payload(&c).expect("canonical encodes");
        let back = decode_payload(&bytes, total_bits).expect("canonical decodes");
        prop_assert_eq!(back, c.clone());
        let (b2, t2) = encode_payload(&d).expect("encodes");
        prop_assert_eq!((b2, t2), (bytes, total_bits));
    }

    // P2(W) — canonicalize-is-normalizer.
    #[test]
    fn p2_w_normalizer(d in wire_descriptor_strategy()) {
        let c = canon(&d);
        let (bd, td) = encode_payload(&d).expect("encodes");
        let (bc, tc) = encode_payload(&c).expect("encodes");
        prop_assert_eq!((&bd, td), (&bc, tc));
        let back = decode_payload(&bd, td).expect("decodes");
        prop_assert_eq!(back, c);
    }

    // P4(W) — string-level round-trip. cycle-4 H6: oversize descriptors reject
    // the single-string encode (chunked round-trip covered by P5(W)).
    #[test]
    fn p4_w_string_round_trip(d in wire_descriptor_strategy()) {
        let c = canon(&d);
        match encode_md1_string(&c) {
            Ok(s) => {
                let back = decode_md1_string(&s).expect("string decodes");
                prop_assert_eq!(back, c);
            }
            Err(md_codec::Error::PayloadTooLongForSingleString { .. }) => { /* chunked-only; see P5(W) */ }
            Err(e) => prop_assert!(false, "unexpected string-encode error: {e:?}"),
        }
    }

    // P5(W) — chunk round-trip.
    #[test]
    fn p5_w_chunk_round_trip(d in wire_descriptor_strategy()) {
        let c = canon(&d);
        let chunks = split(&c).expect("splits");
        let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
        prop_assert_eq!(reassemble(&refs).expect("reassembles"), c);
    }
}
