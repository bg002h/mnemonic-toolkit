//! Theme 3 (SPEC_mk_codec_test_hardening.md §5) — indel reject-contract. BCH
//! is substitution-only; an inserted/deleted symbol (length change) must fail
//! closed. This is the contract the toolkit's `repair --max-indel` oracle
//! relies on: `mnemonic-toolkit/crates/mnemonic-toolkit/src/repair.rs:1001`
//! (`Mk1IndelOracle`) + the comment at `:997-1000` ("mk_codec::decode
//! self-corrects t≤4 UNGUARDED, which would defeat the pure-indel rule").
//!
//! Assertion strength (SPEC §5): T3a/T3b pin a FIXED indel verified to error,
//! so `is_err()`/variant-pin is safe. The weaker `!= Ok(original)` is reserved
//! for the randomized T2c sweep (a ≈2⁻³² cross-chunk-hash collision could make
//! an in-band indel return a DIFFERENT valid card) — do NOT randomize T3a.

use std::str::FromStr;

use bitcoin::NetworkKind;
use bitcoin::bip32::{ChainCode, ChildNumber, DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use mk_codec::{Error, KeyCard, decode, encode_with_chunk_set_id};

fn fixture_card() -> KeyCard {
    let path = DerivationPath::from_str("48'/0'/0'/2'").unwrap();
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(&[0x42u8; 32]).unwrap();
    let pk = PublicKey::from_secret_key(&secp, &sk);
    let comps: Vec<ChildNumber> = path.as_ref().to_vec();
    let xpub = Xpub {
        network: NetworkKind::Main,
        depth: comps.len() as u8,
        parent_fingerprint: Fingerprint::from([0x10, 0x20, 0x30, 0x40]),
        child_number: *comps.last().unwrap(),
        public_key: pk,
        chain_code: ChainCode::from([0xCCu8; 32]),
    };
    // a few stubs → multi-chunk, so the indel lands in a chunk fragment.
    KeyCard::new(
        (0u8..6).map(|i| [i, i, i, i]).collect(),
        Some(Fingerprint::from([0xAA, 0xBB, 0xCC, 0xDD])),
        path,
        xpub,
    )
}

// T3a — a single-symbol indel (insert one symbol, then delete one) must fail
// closed (`Err`), never self-correct into a different valid card. Covers BOTH
// rejection paths: the DELETE keeps the chunk in-band (data-part 108→107, the
// hard case → BCH/cross-chunk-hash rejection), while the INSERT pushes it
// out-of-band (108→109 > the long-code 108 cap → `InvalidStringLength`). Both
// assert `is_err()` (variant not pinned — an indel can surface
// `BchUncorrectable`/`CrossChunkHashMismatch`/`MalformedPayloadPadding`).
#[test]
fn t3a_single_indel_fails_closed() {
    let card = fixture_card();
    let strings = encode_with_chunk_set_id(&card, 0).unwrap();
    let s0 = &strings[0];

    // INSERT one symbol mid-data-part (char-index 15) → length+1 (out-of-band).
    let mut chars: Vec<char> = s0.chars().collect();
    chars.insert(15, 'p');
    let inserted: String = chars.into_iter().collect();
    let mut v_ins = strings.clone();
    v_ins[0] = inserted;
    let parts: Vec<&str> = v_ins.iter().map(String::as_str).collect();
    assert!(
        decode(&parts).is_err(),
        "an inserted symbol must fail closed (never Ok); got {:?}",
        decode(&parts)
    );

    // DELETE one symbol mid-data-part (char-index 15) → length-1.
    let mut chars: Vec<char> = s0.chars().collect();
    chars.remove(15);
    let deleted: String = chars.into_iter().collect();
    let mut v_del = strings.clone();
    v_del[0] = deleted;
    let parts: Vec<&str> = v_del.iter().map(String::as_str).collect();
    assert!(
        decode(&parts).is_err(),
        "a deleted symbol must fail closed (never Ok); got {:?}",
        decode(&parts)
    );
}

// T3b — a delete that pushes a chunk's data-part length into the reserved
// 94/95 gap (or otherwise out of band) is a DETERMINISTIC InvalidStringLength.
#[test]
fn t3b_out_of_band_length_is_invalid_string_length() {
    // Construct a single string whose data-part length, after a delete, lands
    // outside any BCH band. The cleanest deterministic case: take a real chunk
    // and truncate it to a length the band table rejects (94 or 95 data-part
    // symbols → None in bch_code_for_length). We force this by trimming the
    // string to total length = 3 (HRP) + 94 (data-part, reserved) and feeding
    // it; decode must surface InvalidStringLength.
    let card = fixture_card();
    let strings = encode_with_chunk_set_id(&card, 0).unwrap();
    // Find a chunk long enough to trim into the reserved gap.
    let long = strings.iter().max_by_key(|s| s.chars().count()).unwrap();
    let chars: Vec<char> = long.chars().collect();
    // Target total length = 97 → data-part = 97 - 3 = 94 (reserved-invalid gap).
    // The mapping is `data_part = total - 3` (HRP `mk` + separator `1`); the
    // 8-symbol chunked header is INSIDE the data part (R0 I2; verified at
    // `src/string_layer/bch.rs:662,669` — `data_part = &rest[1..]`,
    // `bch_code_for_length(data_part.len())`). So 94 → `None` →
    // `InvalidStringLength(94)`, deterministically. Pin the value.
    assert!(
        chars.len() > 97,
        "fixture chunk too short to trim into the reserved gap; enlarge fixture_card()"
    );
    let trimmed: String = chars[..97].iter().collect();
    assert!(
        matches!(
            decode(&[trimmed.as_str()]),
            Err(Error::InvalidStringLength(94))
        ),
        "reserved-gap length (data-part 94) must be InvalidStringLength(94); got {:?}",
        decode(&[trimmed.as_str()])
    );
}
