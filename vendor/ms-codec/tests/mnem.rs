//! Phase 1 mnem round-trip integration tests + wire-correctness golden vector.

use ms_codec::{decode, decode_with_correction, encode, Payload, PayloadKind, Tag};

/// Encode a Mnem payload and verify:
/// - the output ms1 string has the correct length (51 for 16-byte entropy)
/// - decode returns Payload::Mnem with the correct language and entropy
#[test]
fn mnem_encode_decode_round_trip_16b_japanese() {
    let entropy: Vec<u8> = (0u8..16).collect();
    let p = Payload::Mnem {
        language: 1,
        entropy: entropy.clone(),
    };
    let s = encode(Tag::ENTR, &p).expect("encode Mnem should succeed");
    // 16-byte entropy → mnem str len 51
    assert_eq!(
        s.len(),
        51,
        "mnem 16-byte entropy -> ms1 len 51, got {}",
        s.len()
    );

    let (tag, recovered) = decode(&s).expect("decode mnem should succeed");
    assert_eq!(tag, Tag::ENTR);
    assert!(
        matches!(recovered, Payload::Mnem { language: 1, .. }),
        "expected Payload::Mnem{{language:1, ..}}, got {:?}",
        recovered
    );
    assert_eq!(recovered.as_bytes(), entropy.as_slice());
}

/// A v0.1 entr string still decodes to Payload::Entr after the seam change.
/// This is the entr byte-identity guard: the 0x00 path must be UNCHANGED.
#[test]
fn entr_still_decodes_to_entr_payload() {
    let entropy = vec![0xAAu8; 16];
    let p = Payload::Entr(entropy.clone());
    let s = encode(Tag::ENTR, &p).expect("encode Entr should succeed");
    let (tag, recovered) = decode(&s).expect("decode Entr should succeed");
    assert_eq!(tag, Tag::ENTR);
    assert_eq!(recovered.kind(), PayloadKind::Entr);
    assert_eq!(recovered, Payload::Entr(entropy));
}

/// decode_with_correction on a clean mnem string returns the correct payload
/// (union length gate does not falsely reject a mnem string through this path).
#[test]
fn mnem_decode_with_correction_clean_passes() {
    let entropy: Vec<u8> = vec![0x55u8; 16];
    let p = Payload::Mnem {
        language: 0,
        entropy: entropy.clone(),
    };
    let s = encode(Tag::ENTR, &p).expect("encode mnem");
    let (tag, recovered, corrections) =
        ms_codec::decode_with_correction(&s).expect("decode_with_correction on clean mnem");
    assert_eq!(tag, Tag::ENTR);
    assert!(
        corrections.is_empty(),
        "no corrections expected for clean input"
    );
    assert_eq!(recovered.as_bytes(), entropy.as_slice());
    assert!(
        matches!(recovered, Payload::Mnem { language: 0, .. }),
        "expected Mnem language=0"
    );
}

// ── BCH correction helpers (mirror of bch_all_lengths.rs) ────────────────────

const ABC: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

fn sym(c: char) -> u8 {
    ABC.iter().position(|&b| b == c as u8).unwrap() as u8
}

/// Flip the data-part symbol at `pos` (0-indexed into post-`ms1` region)
/// by XOR-ing with `mask`, staying within the codex32 alphabet.
fn corrupt_at(s: &str, pos: usize, mask: u8) -> String {
    let mut c: Vec<char> = s.chars().collect();
    let i = 3 + pos;
    let v = sym(c[i].to_ascii_lowercase());
    c[i] = ABC[((v ^ (mask & 0x1F)) & 0x1F) as usize] as char;
    c.into_iter().collect()
}

/// Number of data-part symbols (post-`ms1`) for a given ms1 string.
fn data_part_len(s: &str) -> usize {
    s.len() - 3 // subtract 3-char HRP "ms1"
}

// ── Task 1.3 Step 5b: corrupted-mnem BCH correction, all 5 lengths ───────────

/// BCH correction recovers a mnem string from ≤4 corrupted data-part symbols,
/// for all five mnem entropy lengths {16,20,24,28,32} bytes.
///
/// This guards the mnem string-length set {51,58,64,70,77} — a brand-new,
/// previously-unexercised length set on the correction path. The entr set
/// {50,56,62,69,75} is already covered by bch_all_lengths.rs::corrects_1_to_4_errors.
///
/// Positions chosen deterministically via the same formula as bch_all_lengths.rs:
///   positions[j] = 1 + j * max(dp / (k + 1), 1)
/// so each k-error group is evenly spaced, non-overlapping, and well within the
/// data-part bounds.
#[test]
fn mnem_decode_with_correction_recovers_from_corruption() {
    for &n in &[16usize, 20, 24, 28, 32] {
        let entropy = vec![0xABu8; n];
        let p = Payload::Mnem {
            language: 2,
            entropy: entropy.clone(),
        };
        let s = encode(Tag::ENTR, &p).unwrap_or_else(|e| panic!("encode n={n} failed: {e:?}"));
        let dp = data_part_len(&s);

        for k in 1..=4usize {
            let positions: Vec<usize> = (0..k).map(|j| 1 + j * (dp / (k + 1)).max(1)).collect();
            let mut bad = s.clone();
            for &pos in &positions {
                bad = corrupt_at(&bad, pos, 0x1F);
            }

            let (tag, recovered, corr) = decode_with_correction(&bad)
                .unwrap_or_else(|e| panic!("n={n} k={k} correction failed: {e:?}"));
            assert_eq!(tag, Tag::ENTR, "n={n} k={k}: tag must be ENTR");
            assert_eq!(
                recovered,
                Payload::Mnem { language: 2, entropy: entropy.clone() },
                "n={n} k={k}: recovered payload must match original (language=2, entropy=[0xAB;{n}])"
            );

            let got_positions: std::collections::BTreeSet<usize> =
                corr.iter().map(|d| d.position).collect();
            let want_positions: std::collections::BTreeSet<usize> =
                positions.iter().copied().collect();
            assert_eq!(
                got_positions, want_positions,
                "n={n} k={k}: reported correction positions must equal injected positions"
            );
        }
    }
}

/// Wire-correctness golden vector: English (language=0) + fixed 16-byte entropy.
///
/// Pinned by running the encoder ONCE and recording the output. This guards
/// against a self-consistent-but-wrong packing regression: if the wire layout
/// changes (e.g. prefix order, language-byte position), this test fails
/// loudly even if every internal round-trip still passes.
///
/// Captured on branch ms-v0.2-kofn-mnem at commit c66ca2e (Phase 1 seam change).
/// Entropy (hex): 0c1e24e5917544d666c342992acfda1b
/// Language byte: 0x00 (English)
/// On-wire payload: [0x02][0x00][entropy_16_bytes] = 18 bytes
/// Expected ms1 string length: 51 (per VALID_MNEM_STR_LENGTHS[0])
#[test]
fn golden_mnem_english_16b_wire_vector() {
    let entropy: Vec<u8> = vec![
        0x0c, 0x1e, 0x24, 0xe5, 0x91, 0x75, 0x44, 0xd6, 0x66, 0xc3, 0x42, 0x99, 0x2a, 0xcf, 0xda,
        0x1b,
    ];
    let p = Payload::Mnem {
        language: 0,
        entropy: entropy.clone(),
    };
    let s = encode(Tag::ENTR, &p).expect("encode mnem golden");

    // Pin the exact wire string byte-for-byte.
    assert_eq!(
        s, "ms10entrsqgqqc83yukgh23xkvmp59xf2eldpk4cdrq2y4h82yz",
        "mnem wire encoding drifted from golden vector"
    );
    assert_eq!(s.len(), 51);

    // Also verify it decodes back correctly.
    let (tag, recovered) = decode(&s).expect("decode golden");
    assert_eq!(tag, Tag::ENTR);
    assert_eq!(
        recovered,
        Payload::Mnem {
            language: 0,
            entropy
        }
    );
}
