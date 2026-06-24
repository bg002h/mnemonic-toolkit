//! Theme 1 (SPEC_mk_codec_test_hardening.md §3) — property tests for the
//! `KeyCard` encode↔decode bijection (P1) and decode panic-freedom (P2).

mod common;

use common::{csid_strategy, keycard_strategy};
use mk_codec::{decode, encode_with_chunk_set_id};
use proptest::prelude::*;

proptest! {
    // P1 — bijection. `decode(encode_with_chunk_set_id(card, csid)) == card`
    // for any card over the full strategy space and any 20-bit csid.
    #[test]
    fn keycard_roundtrip(card in keycard_strategy(), csid in csid_strategy()) {
        let strings = encode_with_chunk_set_id(&card, csid)
            .expect("strategy produces only encodable cards");
        let parts: Vec<&str> = strings.iter().map(String::as_str).collect();
        let recovered = decode(&parts).expect("a freshly-encoded card must decode");
        prop_assert_eq!(recovered, card);
    }

    // P2a — decode never panics on an arbitrary single string.
    #[test]
    fn decode_never_panics_on_arbitrary_string(s in "\\PC*") {
        let _ = decode(&[s.as_str()]); // must return Ok/Err, never panic
    }

    // P2b — decode never panics on an arbitrary list of strings.
    #[test]
    fn decode_never_panics_on_arbitrary_string_list(
        v in prop::collection::vec("\\PC*", 0..6usize)
    ) {
        let parts: Vec<&str> = v.iter().map(String::as_str).collect();
        let _ = decode(&parts); // must not panic
    }

    // P2c — decode never panics on a corrupted-but-real encoding.
    #[test]
    fn decode_never_panics_on_corrupted_encoding(
        card in keycard_strategy(),
        csid in csid_strategy(),
        n_flips in 0usize..30usize,
        seed in any::<u64>(),
    ) {
        let strings = encode_with_chunk_set_id(&card, csid).unwrap();
        // Deterministic pseudo-random flips across the joined first string.
        let mut s: Vec<char> = strings[0].chars().collect();
        let mut x = seed | 1;
        for _ in 0..n_flips.min(s.len().saturating_sub(3)) {
            x ^= x << 13; x ^= x >> 7; x ^= x << 17; // xorshift64
            let idx = 3 + (x as usize % s.len().saturating_sub(3).max(1));
            s[idx] = if s[idx] == 'q' { 'p' } else { 'q' };
        }
        let corrupted: String = s.into_iter().collect();
        let mut parts_owned = strings.clone();
        parts_owned[0] = corrupted;
        let parts: Vec<&str> = parts_owned.iter().map(String::as_str).collect();
        let _ = decode(&parts); // must not panic
    }
}
