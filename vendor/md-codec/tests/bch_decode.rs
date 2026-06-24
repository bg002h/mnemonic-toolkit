//! Integration cells for `decode_with_correction` per plan §4.B.1.
//!
//! Covers the 6 cases:
//!  1. `zero_error_passthrough` — clean md1, no corrections.
//!  2. `one_error_at_position_0` — corrupt 1 char at position 0.
//!  3. `one_error_at_last_data_symbol` — corrupt 1 char at the last
//!     data-part position (just before the 13-symbol BCH checksum).
//!  4. `four_error_t_boundary` — BCH t=4 boundary.
//!  5. `five_error_too_many` — exceeds capacity → `TooManyErrors`.
//!  6. `multi_chunk_one_corrupted` — 1 chunk of a 3-chunk set corrupted.
//!
//! v0.35.0 (plan §2.D.1) adds 5 cells exercising single-string non-chunked
//! md1 auto-dispatch in `decode_with_correction` per SPEC v0.30 §2.3:
//!  7. `non_chunked_zero_error_passthrough` — valid non-chunked md1
//!     decodes successfully.
//!  8. `non_chunked_one_to_four_errors_corrected` — 1..=4 errors correct.
//!  9. `non_chunked_five_errors_too_many` — 5+ errors → `TooManyErrors`.
//! 10. `non_chunked_chunked_flag_corruption_yields_chunk_set_incomplete` —
//!     post-correction chunked-flag bit set with only 1 string supplied.
//! 11. `non_chunked_round_trip_parity_via_encode_md1_string` — full
//!     encode_md1_string ↔ decode_with_correction round-trip.

use md_codec::chunk::split;
use md_codec::encode::{Descriptor, encode_md1_string};
use md_codec::error::Error;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;
use md_codec::{CorrectionDetail, decode_with_correction};

/// Codex32 alphabet, mirroring `src/chunk.rs::CODEX32_ALPHABET`. Tests need
/// a known character → 5-bit-value mapping to construct deterministic
/// corruption patterns.
const CODEX32_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// Helper: small single-chunk descriptor (bip84 wpkh). Round-trips through
/// `encode_md1_string` to produce one md1 chunk well under the 64-symbol
/// regular-form chunking threshold.
fn small_descriptor() -> Descriptor {
    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(OriginPath {
                components: vec![
                    PathComponent {
                        hardened: true,
                        value: 84,
                    },
                    PathComponent {
                        hardened: true,
                        value: 0,
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
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: TlvSection::new_empty(),
    }
}

/// Helper: 4-cosigner divergent-path wsh sortedmulti template that's large
/// enough to force chunked encoding. Mirrors `tests/chunking.rs`'s
/// `multi_chunk_descriptor` — per-cosigner path body ~180 bits × 4 cosigners
/// ~720 bits, well above the 320-bit single-string limit so chunking is
/// guaranteed.
fn multi_chunk_descriptor() -> Descriptor {
    let mut paths = Vec::new();
    for cosigner in 0..4u32 {
        let mut components = Vec::new();
        for i in 0..15u32 {
            components.push(PathComponent {
                hardened: true,
                value: cosigner * 100 + i + 1,
            });
        }
        paths.push(OriginPath { components });
    }
    Descriptor {
        n: 4,
        path_decl: PathDecl {
            n: 4,
            paths: PathDeclPaths::Divergent(paths),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::SortedMulti,
                body: Body::MultiKeys {
                    k: 2,
                    indices: (0..4).collect(),
                },
            }]),
        },
        tlv: TlvSection::new_empty(),
    }
}

/// Flip one character of an md1 chunk at the data-part position `pos`
/// (0-indexed, post-`md1` HRP). The flipped char is `original ^ mask`
/// in the 5-bit codex32 alphabet space.
fn corrupt_chunk_at(chunk: &str, pos: usize, xor_mask: u8) -> String {
    let hrp_len = 3; // "md1"
    let mut chars: Vec<char> = chunk.chars().collect();
    let abs_idx = hrp_len + pos;
    let original_char = chars[abs_idx];
    let original_sym = CODEX32_ALPHABET
        .iter()
        .position(|&b| b == original_char.to_ascii_lowercase() as u8)
        .expect("char in codex32 alphabet") as u8;
    let new_sym = (original_sym ^ (xor_mask & 0x1F)) & 0x1F;
    chars[abs_idx] = CODEX32_ALPHABET[new_sym as usize] as char;
    chars.iter().collect()
}

/// Extract the codex32 data-part (post-HRP) length.
fn data_part_len(chunk: &str) -> usize {
    chunk.len() - 3 // strip "md1"
}

// ---------------------------------------------------------------------------
// Cell 1: zero-error pass-through
// ---------------------------------------------------------------------------

#[test]
fn zero_error_passthrough() {
    let d = small_descriptor();
    let chunks = split(&d).unwrap();
    let refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
    let (decoded, details) = decode_with_correction(&refs).expect("clean decode");
    assert_eq!(decoded, d, "round-trip preserves descriptor");
    assert!(
        details.is_empty(),
        "no corrections expected for clean input"
    );
}

// ---------------------------------------------------------------------------
// Cell 2: 1 error at position 0
// ---------------------------------------------------------------------------

#[test]
fn one_error_at_position_0() {
    let d = small_descriptor();
    let chunks = split(&d).unwrap();
    let bad = corrupt_chunk_at(&chunks[0], 0, 0b10101);
    let (decoded, details) = decode_with_correction(&[bad.as_str()]).expect("1-error decode");
    assert_eq!(decoded, d, "corrected decode matches original");
    assert_eq!(details.len(), 1, "exactly 1 correction reported");
    assert_eq!(details[0].chunk_index, 0);
    assert_eq!(details[0].position, 0);
    // The original char at position 0 should be what we computed.
    let original_char = chunks[0].chars().nth(3).unwrap();
    assert_eq!(
        details[0].now, original_char,
        "correction restores the original char"
    );
    assert_ne!(details[0].was, details[0].now);
}

// ---------------------------------------------------------------------------
// Cell 3: 1 error at the last data-part position
// ---------------------------------------------------------------------------

#[test]
fn one_error_at_last_data_symbol() {
    let d = small_descriptor();
    let chunks = split(&d).unwrap();
    let last_pos = data_part_len(&chunks[0]) - 1;
    let bad = corrupt_chunk_at(&chunks[0], last_pos, 0b01110);
    let (decoded, details) =
        decode_with_correction(&[bad.as_str()]).expect("1-error at last position decodes");
    assert_eq!(decoded, d);
    assert_eq!(details.len(), 1);
    assert_eq!(details[0].position, last_pos);
    let original_char = chunks[0].chars().nth(3 + last_pos).unwrap();
    assert_eq!(details[0].now, original_char);
}

// ---------------------------------------------------------------------------
// Cell 4: 4-error t-boundary
// ---------------------------------------------------------------------------

#[test]
fn four_error_t_boundary() {
    let d = small_descriptor();
    let chunks = split(&d).unwrap();
    let dp_len = data_part_len(&chunks[0]);
    // 4 distinct, well-spaced positions across the data-part.
    let positions: [usize; 4] = [0, dp_len / 4, dp_len / 2, dp_len - 1];
    let masks: [u8; 4] = [0b00001, 0b10000, 0b11111, 0b01010];
    let mut bad = chunks[0].clone();
    for (&p, &m) in positions.iter().zip(&masks) {
        bad = corrupt_chunk_at(&bad, p, m);
    }
    let (decoded, details) =
        decode_with_correction(&[bad.as_str()]).expect("4-error t-boundary decodes");
    assert_eq!(decoded, d, "corrected decode matches original");
    assert_eq!(details.len(), 4, "exactly 4 corrections reported");
    // Positions should be reported in ascending order per decode_regular_errors's sort.
    let reported_positions: Vec<usize> = details.iter().map(|c| c.position).collect();
    let mut expected_positions: Vec<usize> = positions.to_vec();
    expected_positions.sort();
    assert_eq!(reported_positions, expected_positions);
    for det in &details {
        assert_eq!(det.chunk_index, 0);
        assert_ne!(det.was, det.now, "correction changes the character");
    }
}

// ---------------------------------------------------------------------------
// Cell 5: 5 errors — exceeds BCH t = 4 capacity → TooManyErrors
// ---------------------------------------------------------------------------

#[test]
fn five_error_too_many() {
    let d = small_descriptor();
    let chunks = split(&d).unwrap();
    let dp_len = data_part_len(&chunks[0]);
    let positions: [usize; 5] = [0, dp_len / 5, 2 * dp_len / 5, 3 * dp_len / 5, dp_len - 1];
    let masks: [u8; 5] = [0b00001, 0b00010, 0b00100, 0b01000, 0b10000];
    let mut bad = chunks[0].clone();
    for (&p, &m) in positions.iter().zip(&masks) {
        bad = corrupt_chunk_at(&bad, p, m);
    }
    let err = decode_with_correction(&[bad.as_str()])
        .expect_err("5-error pattern must not decode successfully");
    match err {
        Error::TooManyErrors { chunk_index, bound } => {
            assert_eq!(chunk_index, 0, "the only chunk is index 0");
            assert_eq!(bound, 8, "BCH(93,80,8) singleton bound is 8");
        }
        other => panic!("expected TooManyErrors, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Cell 6: multi-chunk set with one chunk corrupted (1 error)
// ---------------------------------------------------------------------------

#[test]
fn multi_chunk_one_corrupted() {
    let d = multi_chunk_descriptor();
    let chunks = split(&d).unwrap();
    assert!(
        chunks.len() >= 2,
        "multi-chunk descriptor must split into 2+ chunks; got {}",
        chunks.len()
    );
    // Pick the middle (or only-non-first) chunk to corrupt.
    let target_idx = chunks.len() / 2;
    let bad_chunk = corrupt_chunk_at(&chunks[target_idx], 4, 0b01101);
    let mut input: Vec<String> = chunks.to_vec();
    input[target_idx] = bad_chunk;
    let refs: Vec<&str> = input.iter().map(|s| s.as_str()).collect();
    let (decoded, details) = decode_with_correction(&refs).expect("multi-chunk decode");
    assert_eq!(decoded, d, "round-trip restores descriptor");
    assert_eq!(
        details.len(),
        1,
        "exactly 1 correction across the chunk set"
    );
    let det: &CorrectionDetail = &details[0];
    assert_eq!(
        det.chunk_index, target_idx,
        "correction reports the corrupted chunk's index"
    );
    assert_eq!(det.position, 4);
    let original_char = chunks[target_idx].chars().nth(3 + 4).unwrap();
    assert_eq!(det.now, original_char);
}

// ===========================================================================
// v0.35.0 — single-string non-chunked md1 auto-dispatch (plan §2.D.1).
// ===========================================================================

// ---------------------------------------------------------------------------
// Cell 7: non-chunked happy path — valid single-payload md1 decodes cleanly.
// ---------------------------------------------------------------------------

#[test]
fn non_chunked_zero_error_passthrough() {
    let d = small_descriptor();
    // `encode_md1_string` emits single-payload non-chunked form (no chunk
    // header). The first 5-bit symbol's bit 0 (chunked-flag per SPEC v0.30
    // §2.3) is 0 → `decode_with_correction` must auto-dispatch to the
    // non-chunked decode path instead of `reassemble`.
    let s = encode_md1_string(&d).expect("encode_md1_string");
    let (decoded, details) =
        decode_with_correction(&[s.as_str()]).expect("non-chunked decode succeeds");
    assert_eq!(decoded, d, "round-trip preserves descriptor");
    assert!(
        details.is_empty(),
        "no corrections expected for clean input"
    );
}

// ---------------------------------------------------------------------------
// Cell 8: non-chunked 1..=4 error correction across the BCH t-boundary.
// ---------------------------------------------------------------------------

#[test]
fn non_chunked_one_to_four_errors_corrected() {
    let d = small_descriptor();
    let s = encode_md1_string(&d).expect("encode_md1_string");
    let dp_len = data_part_len(&s);

    // Walk 1..=4 errors; for each error budget, corrupt that many
    // well-spaced positions, expect a clean BCH correction back to the
    // original descriptor. NOTE: position 0 is excluded so we don't flip
    // bit 0 of the first 5-bit symbol (the chunked-flag); that case is
    // covered by Cell 10 below.
    for error_count in 1..=4usize {
        let positions: Vec<usize> = (0..error_count)
            .map(|i| 1 + ((dp_len - 2) * i) / error_count.max(1))
            .collect();
        let masks: [u8; 4] = [0b00001, 0b10000, 0b11111, 0b01010];
        let mut bad = s.clone();
        for (i, &p) in positions.iter().enumerate() {
            bad = corrupt_chunk_at(&bad, p, masks[i]);
        }
        let (decoded, details) = decode_with_correction(&[bad.as_str()])
            .unwrap_or_else(|e| panic!("{error_count}-error decode must succeed, got {e:?}"));
        assert_eq!(
            decoded, d,
            "{error_count}-error corrected decode matches original"
        );
        assert_eq!(
            details.len(),
            error_count,
            "{error_count}-error correction report length"
        );
        for det in &details {
            assert_eq!(det.chunk_index, 0);
            assert_ne!(det.was, det.now);
        }
    }
}

// ---------------------------------------------------------------------------
// Cell 9: non-chunked 5-error pattern → TooManyErrors (BCH t=4 boundary).
// ---------------------------------------------------------------------------

#[test]
fn non_chunked_five_errors_too_many() {
    let d = small_descriptor();
    let s = encode_md1_string(&d).expect("encode_md1_string");
    let dp_len = data_part_len(&s);
    // 5 distinct, well-spaced positions across the data-part (mirrors
    // Cell 5's chunked-case pattern). Position 0 is included; the
    // intent is to exercise the BCH-correction capacity exhaustion, not
    // the chunked-flag-bit branch.
    let positions: [usize; 5] = [0, dp_len / 5, 2 * dp_len / 5, 3 * dp_len / 5, dp_len - 1];
    let masks: [u8; 5] = [0b00001, 0b00010, 0b00100, 0b01000, 0b10000];
    let mut bad = s.clone();
    for (&p, &m) in positions.iter().zip(&masks) {
        bad = corrupt_chunk_at(&bad, p, m);
    }
    let err = decode_with_correction(&[bad.as_str()])
        .expect_err("5-error pattern must not decode successfully");
    match err {
        Error::TooManyErrors { chunk_index, bound } => {
            assert_eq!(chunk_index, 0, "the only chunk is index 0");
            assert_eq!(bound, 8, "BCH(93,80,8) singleton bound is 8");
        }
        other => panic!("expected TooManyErrors, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Cell 10: chunked-flag-set + only 1 string supplied → ChunkSetIncomplete.
//
// This is the SPEC v0.30 §2.3 ambiguity edge: post-correction, the first
// 5-bit symbol's bit 0 is set (== 1 → chunked) but the caller provided only
// 1 input string. Per plan §2.D.1, surface as the existing
// `ChunkSetIncomplete` variant (no new error kind).
// ---------------------------------------------------------------------------

#[test]
fn non_chunked_chunked_flag_corruption_yields_chunk_set_incomplete() {
    // Take a multi-chunk descriptor's chunked-form chunk[0] and supply it
    // alone — the chunk header's chunked-flag bit is set, but only 1 string
    // is provided, so `decode_with_correction` must reject with
    // `ChunkSetIncomplete`.
    let d = multi_chunk_descriptor();
    let chunks = split(&d).expect("split multi-chunk");
    assert!(
        chunks.len() >= 2,
        "multi-chunk descriptor must split into 2+ chunks; got {}",
        chunks.len()
    );
    let single = &chunks[0];
    let err = decode_with_correction(&[single.as_str()])
        .expect_err("chunked-form with only 1 string must not decode");
    match err {
        Error::ChunkSetIncomplete { got, expected } => {
            assert_eq!(got, 1, "exactly 1 string supplied");
            assert_eq!(
                expected,
                chunks.len(),
                "expected count matches chunk-set size"
            );
        }
        other => panic!("expected ChunkSetIncomplete, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Cell 11: full round-trip parity vs encode_md1_string (zero-error).
// ---------------------------------------------------------------------------

#[test]
fn non_chunked_round_trip_parity_via_encode_md1_string() {
    // Confirms `encode_md1_string` ↔ `decode_with_correction(&[s])` is a
    // bit-identity round-trip for non-chunked-form inputs.
    let d = small_descriptor();
    let s = encode_md1_string(&d).expect("encode_md1_string");
    let (decoded, details) =
        decode_with_correction(&[s.as_str()]).expect("non-chunked decode succeeds");
    assert_eq!(decoded, d, "round-trip preserves descriptor");
    assert!(details.is_empty(), "no corrections for clean round-trip");
    // Sanity: re-encode the decoded form and compare with the original
    // string to catch any subtle decode-side normalization drift.
    let s2 = encode_md1_string(&decoded).expect("re-encode_md1_string");
    assert_eq!(
        s, s2,
        "re-encoded string matches the original byte-for-byte"
    );
}
