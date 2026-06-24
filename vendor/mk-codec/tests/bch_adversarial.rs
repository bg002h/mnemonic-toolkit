//! Theme 2 (SPEC_mk_codec_test_hardening.md §4) — BCH adversarial coverage:
//! 3/4-error correction THROUGH the public `decode()` (T2a/T2b) and a
//! randomized 5–8-error miscorrection sweep (T2c). mk's guard model: per-chunk
//! `bch_correct_*` re-verify + the 4-byte cross-chunk hash at reassembly
//! (the residual is ~2⁻³² — see T2c). Both BCH codes are t=4.

mod common;

use std::str::FromStr;

use bitcoin::bip32::{DerivationPath, Fingerprint};
use common::{csid_strategy, flip_chars, keycard_strategy};
use mk_codec::{Error, KeyCard, decode, encode_with_chunk_set_id};
use proptest::prelude::*;

/// Build a deterministic multi-chunk card large enough that `strings[0]` is a
/// long-code (non-last, full-size) chunk and `strings.last()` is a regular-code
/// chunk. ~6 stubs ⇒ bytecode well over the single-string capacity ⇒ ≥2 chunks.
fn multi_chunk_card() -> KeyCard {
    let path = DerivationPath::from_str("48'/0'/0'/2'").unwrap();
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let sk = bitcoin::secp256k1::SecretKey::from_slice(&[0x42u8; 32]).unwrap();
    let pk = bitcoin::secp256k1::PublicKey::from_secret_key(&secp, &sk);
    let comps: Vec<bitcoin::bip32::ChildNumber> = path.as_ref().to_vec();
    let xpub = bitcoin::bip32::Xpub {
        network: bitcoin::NetworkKind::Main,
        depth: comps.len() as u8,
        parent_fingerprint: Fingerprint::from([0x10, 0x20, 0x30, 0x40]),
        child_number: *comps.last().unwrap(),
        public_key: pk,
        chain_code: bitcoin::bip32::ChainCode::from([0xCCu8; 32]),
    };
    KeyCard::new(
        (0u8..6).map(|i| [i, i, i, i]).collect(),
        Some(Fingerprint::from([0xAA, 0xBB, 0xCC, 0xDD])),
        path,
        xpub,
    )
}

/// data-part length (symbols) AS SEEN BY `bch_code_for_length` — the full
/// post-`mk1` data part (chunked header + payload + BCH checksum), i.e. total
/// chars minus the 3-char `mk1` HRP+separator. The 8-symbol chunked header is
/// PART of the band-table input — do NOT subtract it (R0 I1; verified at
/// `src/string_layer/bch.rs:662,669`: `data_part = &rest[1..]`,
/// `bch_code_for_length(data_part.len())`). Used to assert both BCH code
/// variants are exercised. For the 6-stub `multi_chunk_card`, `strings[0]`
/// data-part = 108 (Long), `strings.last()` = 25 (Regular).
fn data_part_len(s: &str) -> usize {
    s.chars().count().saturating_sub(3)
}

#[test]
fn t2a_three_and_four_error_correction_through_public_decode() {
    let card = multi_chunk_card();
    let strings = encode_with_chunk_set_id(&card, 0).unwrap();
    assert!(
        strings.len() >= 2,
        "fixture must be multi-chunk; got {}",
        strings.len()
    );

    // strings[0] is a non-last (full-size, long-code) chunk; strings.last()
    // is the regular-code chunk (mirrors the structure documented in
    // src/string_layer/pipeline.rs's 5-burst test).
    let long_dl = data_part_len(&strings[0]);
    let reg_dl = data_part_len(strings.last().unwrap());
    assert!(
        (96..=108).contains(&long_dl),
        "strings[0] must be a long-code chunk (data-part 96..=108); got {long_dl}. \
         Increase the stub count in multi_chunk_card() if this fails."
    );
    assert!(
        (14..=93).contains(&reg_dl),
        "last chunk must be a regular-code chunk (data-part 14..=93); got {reg_dl}"
    );

    // Corrupt 3, then 4, data-part symbols (past the 3-char HRP + 8-symbol
    // header → char-index ≥ 11) in EACH band; BCH t=4 must recover the original.
    for &n in &[3usize, 4usize] {
        let positions: Vec<usize> = (11..11 + n).collect();

        // long-code chunk (strings[0])
        let mut s_long = strings.clone();
        s_long[0] = flip_chars(&strings[0], &positions);
        let parts: Vec<&str> = s_long.iter().map(String::as_str).collect();
        assert_eq!(
            decode(&parts).expect("BCH t=4 corrects the long-code chunk"),
            card,
            "{n}-error correction failed for the long-code chunk"
        );

        // regular-code chunk (strings.last())
        let li = strings.len() - 1;
        let mut s_reg = strings.clone();
        s_reg[li] = flip_chars(&strings[li], &positions);
        let parts: Vec<&str> = s_reg.iter().map(String::as_str).collect();
        assert_eq!(
            decode(&parts).expect("BCH t=4 corrects the regular-code chunk"),
            card,
            "{n}-error correction failed for the regular-code chunk"
        );
    }
}

#[test]
fn t2b_checksum_region_and_mixed_correction() {
    let card = multi_chunk_card();
    let strings = encode_with_chunk_set_id(&card, 0).unwrap();
    let li = strings.len() - 1;
    let last = &strings[li];
    let total = last.chars().count();

    // The BCH checksum is the trailing 13 symbols (regular code). Corrupt
    // inside the checksum tail (NOT the data part) — exercises the
    // position-translation `k = L-1-d` (src/string_layer/bch_decode.rs:587)
    // that the existing corpus never reaches.
    let checksum_positions: Vec<usize> = (total - 4..total).collect(); // 4 tail symbols
    let mut s_csum = strings.clone();
    s_csum[li] = flip_chars(last, &checksum_positions);
    let parts: Vec<&str> = s_csum.iter().map(String::as_str).collect();
    assert_eq!(
        decode(&parts).expect("BCH corrects checksum-region errors"),
        card,
        "checksum-region 4-error correction failed"
    );

    // Mixed: 2 in the data part + 2 in the checksum tail (total 4 = t-boundary).
    let mixed: Vec<usize> = vec![11, 12, total - 2, total - 1];
    let mut s_mix = strings.clone();
    s_mix[li] = flip_chars(last, &mixed);
    let parts: Vec<&str> = s_mix.iter().map(String::as_str).collect();
    assert_eq!(
        decode(&parts).expect("BCH corrects mixed data+checksum at the t=4 boundary"),
        card,
        "mixed data+checksum 4-error correction failed"
    );
}

proptest! {
    // T2c — randomized miscorrection sweep. Corrupt 5–8 distinct symbols in
    // ONE chunk's data part. The robust, non-flaky property is
    // `decode(perturbed) != Ok(original)`: three outcomes are all legal —
    // Err(BchUncorrectable), Err(CrossChunkHashMismatch), or (≈2⁻³², the
    // accepted 4-byte cross-chunk-hash residual) Ok(a DIFFERENT card). The
    // contract under test is "a ≥5-error corruption never SILENTLY returns the
    // original as if clean." Asserting `.is_err()` would flake ~1-in-4.3e9.
    #[test]
    fn t2c_five_to_eight_error_corruption_never_returns_original(
        card in keycard_strategy(),
        csid in csid_strategy(),
        n_errors in 5usize..=8usize,
        seed in any::<u64>(),
    ) {
        let strings = encode_with_chunk_set_id(&card, csid).unwrap();
        // (encode always yields ≥1 string — no assume needed; R0 M1.)
        // Target chunk 0; corrupt n distinct data-part positions (char-index ≥ 11
        // for chunked, ≥ 5 for single-chunk — use ≥ 11 and require enough length).
        let s0 = &strings[0];
        let len = s0.chars().count();
        prop_assume!(len > 11 + n_errors);
        let mut positions = Vec::new();
        let mut x = seed | 1;
        while positions.len() < n_errors {
            x ^= x << 13; x ^= x >> 7; x ^= x << 17;
            let idx = 11 + (x as usize % (len - 11));
            if !positions.contains(&idx) { positions.push(idx); }
        }
        let mut perturbed = strings.clone();
        perturbed[0] = flip_chars(s0, &positions);
        let parts: Vec<&str> = perturbed.iter().map(String::as_str).collect();

        if let Ok(recovered) = decode(&parts) {
            prop_assert_ne!(
                recovered,
                card.clone(),
                "≥5-error corruption silently returned the original card"
            );
        }
        // Err(_) => BchUncorrectable / CrossChunkHashMismatch — both legal
    }
}

#[test]
fn t4_stub_count_boundary_255_roundtrip_256_reject() {
    let path = DerivationPath::from_str("48'/0'/0'/2'").unwrap();
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let sk = bitcoin::secp256k1::SecretKey::from_slice(&[0x07u8; 32]).unwrap();
    let pk = bitcoin::secp256k1::PublicKey::from_secret_key(&secp, &sk);
    let comps: Vec<bitcoin::bip32::ChildNumber> = path.as_ref().to_vec();
    let xpub = bitcoin::bip32::Xpub {
        network: bitcoin::NetworkKind::Main,
        depth: comps.len() as u8,
        parent_fingerprint: Fingerprint::from([0x10, 0x20, 0x30, 0x40]),
        child_number: *comps.last().unwrap(),
        public_key: pk,
        chain_code: bitcoin::bip32::ChainCode::from([0xCCu8; 32]),
    };

    // 255 stubs (the encoder's 1-byte stub_count max) — ~1100-byte bytecode ⇒
    // a many-chunk (>2) real-card round-trip. 255 distinct 4-byte stubs.
    let stubs_255: Vec<[u8; 4]> = (0..255u16)
        .map(|i| [i as u8, (i >> 8) as u8, 0xAB, 0xCD])
        .collect();
    let card_255 = KeyCard::new(stubs_255, None, path.clone(), xpub);
    let strings = encode_with_chunk_set_id(&card_255, 1).expect("255 stubs encodes");
    assert!(
        strings.len() > 2,
        "255 stubs must produce a >2-chunk card; got {}",
        strings.len()
    );
    let parts: Vec<&str> = strings.iter().map(String::as_str).collect();
    assert_eq!(decode(&parts).expect("255-stub card decodes"), card_255);

    // 256 stubs — over the 1-byte cap ⇒ encoder rejects.
    let stubs_256: Vec<[u8; 4]> = (0..256u16)
        .map(|i| [i as u8, (i >> 8) as u8, 0xAB, 0xCD])
        .collect();
    let card_256 = KeyCard::new(stubs_256, None, path, xpub);
    assert!(
        matches!(
            encode_with_chunk_set_id(&card_256, 1),
            Err(Error::InvalidPolicyIdStubCount)
        ),
        "256 stubs must be rejected with InvalidPolicyIdStubCount"
    );
}
