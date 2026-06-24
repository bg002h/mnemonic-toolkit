//! Integration cells for `decode_with_correction` per plan §4.B.1.
//!
//! Covers the 6 ms-codec-specific cases (ms1 is single-chunk, so the
//! md-codec's `multi_chunk_one_corrupted` cell is replaced by
//! `corrupt_checksum_region` — a correction landing in the 13-symbol BCH
//! checksum tail rather than the data portion):
//!
//!  1. `zero_error_passthrough` — clean ms1, no corrections.
//!  2. `one_error_at_position_0` — corrupt 1 char at position 0.
//!  3. `one_error_at_last_data_symbol` — corrupt 1 char at the last
//!     data-part position (just before the 13-symbol BCH checksum).
//!  4. `four_error_t_boundary` — BCH t=4 boundary.
//!  5. `five_error_too_many` — exceeds capacity → `TooManyErrors`.
//!  6. `corrupt_checksum_region` — single-error in the checksum tail
//!     (still within t=4; verifies the decoder treats checksum + data
//!     symbols symmetrically).
//!
//! These cells use the canonical 12-word abandon ms1 vector from
//! `tests/vectors/v0.1.json` (entry 0; same fixture as `tests/bch_drift.rs`).
//! NOTE (v0.2.1): the regular-code polymod invariant
//! `polymod == MS_REGULAR_CONST` now holds for ALL ms1 entropy lengths
//! (16/20/24/28/32 B), not just the 12-word anchor — the pre-v0.2.1 code's
//! wrong `POLYMOD_INIT` made `decode_with_correction` reject clean 20+-byte
//! seeds. The all-length sweep lives in `tests/bch_all_lengths.rs`; see
//! `design/BUG_decode_with_correction_length_divergence.md`.

use ms_codec::{decode_with_correction, CorrectionDetail, Error, Tag};

/// Codex32 alphabet — needed for deterministic single-character
/// corruption masks in the 5-bit symbol space.
const CODEX32_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// Canonical 12-word abandon ms1 from `tests/vectors/v0.1.json` (entry 0).
/// 50 chars total; data-part length = 47 (= 50 - 3 HRP).
const VALID_MS1_12W: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

/// Flip one character of an ms1 string at the data-part position `pos`
/// (0-indexed, post-`ms1` HRP). The flipped char is `original ^ mask` in
/// the 5-bit codex32 alphabet space.
fn corrupt_at(s: &str, pos: usize, xor_mask: u8) -> String {
    let hrp_len = 3; // "ms1"
    let mut chars: Vec<char> = s.chars().collect();
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

/// Data-part length (post-HRP, includes the 13-symbol BCH checksum tail).
fn data_part_len(s: &str) -> usize {
    s.len() - 3 // strip "ms1"
}

// ---------------------------------------------------------------------------
// Cell 1: zero-error pass-through
// ---------------------------------------------------------------------------

#[test]
fn zero_error_passthrough() {
    let (tag, _payload, details) =
        decode_with_correction(VALID_MS1_12W).expect("clean ms1 must decode");
    assert_eq!(tag, Tag::ENTR);
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
    // Position 0 of the data-part is the threshold character ('0'); flip
    // it to something else and confirm correction restores it.
    let bad = corrupt_at(VALID_MS1_12W, 0, 0b10101);
    assert_ne!(bad, VALID_MS1_12W, "corruption changed the string");
    let (tag, _payload, details) =
        decode_with_correction(&bad).expect("1-error decode must succeed");
    assert_eq!(tag, Tag::ENTR);
    assert_eq!(details.len(), 1, "exactly 1 correction reported");
    assert_eq!(details[0].position, 0);
    let original_char = VALID_MS1_12W.chars().nth(3).unwrap();
    assert_eq!(
        details[0].now, original_char,
        "correction restores the original char"
    );
    assert_ne!(
        details[0].was, details[0].now,
        "correction changes the character"
    );
}

// ---------------------------------------------------------------------------
// Cell 3: 1 error at the last data-part position
// ---------------------------------------------------------------------------

#[test]
fn one_error_at_last_data_symbol() {
    // "Last data-part position" = just before the 13-symbol BCH checksum
    // tail. For the 12-word ms1 (data_part_len 47, checksum 13), that's
    // data-part index 47 - 13 - 1 = 33.
    let dp_len = data_part_len(VALID_MS1_12W);
    let last_data_pos = dp_len - 13 - 1;
    let bad = corrupt_at(VALID_MS1_12W, last_data_pos, 0b01110);
    let (tag, _payload, details) =
        decode_with_correction(&bad).expect("1-error at last data position must decode");
    assert_eq!(tag, Tag::ENTR);
    assert_eq!(details.len(), 1);
    assert_eq!(details[0].position, last_data_pos);
    let original_char = VALID_MS1_12W.chars().nth(3 + last_data_pos).unwrap();
    assert_eq!(details[0].now, original_char);
}

// ---------------------------------------------------------------------------
// Cell 4: 4-error t-boundary
// ---------------------------------------------------------------------------

#[test]
fn four_error_t_boundary() {
    // 4 distinct, well-spaced positions across the 47-symbol data-part.
    let dp_len = data_part_len(VALID_MS1_12W);
    let positions: [usize; 4] = [0, dp_len / 4, dp_len / 2, dp_len - 1];
    let masks: [u8; 4] = [0b00001, 0b10000, 0b11111, 0b01010];
    let mut bad = VALID_MS1_12W.to_string();
    for (&p, &m) in positions.iter().zip(&masks) {
        bad = corrupt_at(&bad, p, m);
    }
    let (tag, _payload, details) =
        decode_with_correction(&bad).expect("4-error t-boundary must decode");
    assert_eq!(tag, Tag::ENTR);
    assert_eq!(details.len(), 4, "exactly 4 corrections reported");
    // Positions are reported in ascending order per decode_regular_errors's sort.
    let reported_positions: Vec<usize> = details.iter().map(|c| c.position).collect();
    let mut expected_positions: Vec<usize> = positions.to_vec();
    expected_positions.sort();
    assert_eq!(reported_positions, expected_positions);
    for det in &details {
        assert_ne!(det.was, det.now, "correction changes the character");
    }
}

// ---------------------------------------------------------------------------
// Cell 5: 5 errors — exceeds BCH t = 4 capacity → TooManyErrors
// ---------------------------------------------------------------------------

#[test]
fn five_error_too_many() {
    let dp_len = data_part_len(VALID_MS1_12W);
    let positions: [usize; 5] = [0, dp_len / 5, 2 * dp_len / 5, 3 * dp_len / 5, dp_len - 1];
    let masks: [u8; 5] = [0b00001, 0b00010, 0b00100, 0b01000, 0b10000];
    let mut bad = VALID_MS1_12W.to_string();
    for (&p, &m) in positions.iter().zip(&masks) {
        bad = corrupt_at(&bad, p, m);
    }
    let err =
        decode_with_correction(&bad).expect_err("5-error pattern must not decode successfully");
    match err {
        Error::TooManyErrors { bound } => {
            assert_eq!(bound, 8, "BCH(93,80,8) singleton bound is 8");
        }
        other => panic!("expected TooManyErrors, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Cell 6: corruption in the checksum tail (ms1 single-chunk replacement
// for md-codec's multi_chunk_one_corrupted cell).
// ---------------------------------------------------------------------------

#[test]
fn corrupt_checksum_region() {
    // Pick a single position INSIDE the 13-symbol BCH checksum tail. The
    // BCH decoder treats checksum + data symbols symmetrically (the
    // codeword IS data || checksum), so a 1-symbol error in the checksum
    // tail must correct the same way.
    let dp_len = data_part_len(VALID_MS1_12W);
    // First symbol of the checksum tail = dp_len - 13.
    let checksum_pos = dp_len - 13 + 5; // middle-ish of the 13-symbol tail
    let bad = corrupt_at(VALID_MS1_12W, checksum_pos, 0b11001);
    let (tag, _payload, details) =
        decode_with_correction(&bad).expect("1-error in checksum tail must decode");
    assert_eq!(tag, Tag::ENTR);
    assert_eq!(details.len(), 1, "exactly 1 correction reported");
    assert_eq!(
        details[0].position, checksum_pos,
        "correction position lies in the checksum tail"
    );
    let original_char = VALID_MS1_12W.chars().nth(3 + checksum_pos).unwrap();
    assert_eq!(details[0].now, original_char);
    // Also confirm CorrectionDetail's Eq derive works (uses `PartialEq, Eq`).
    let expected = CorrectionDetail {
        position: checksum_pos,
        was: details[0].was,
        now: details[0].now,
    };
    assert_eq!(details[0], expected);
}
