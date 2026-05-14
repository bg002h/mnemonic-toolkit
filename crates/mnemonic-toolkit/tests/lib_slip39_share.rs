//! v0.13.0 P1c-D — library tests for SLIP-39 share parse + render.
//!
//! Per SLIP-0039 §3.1 ("Format of the share mnemonic") + the python
//! reference at `python-shamir-mnemonic/shamir_mnemonic/share.py`
//! (commit `17fcce14`):
//!
//!   Wire bit-field layout (most-significant bit first; words = 10 bits each):
//!     - id_exp:       2 words = 20 bits — identifier(15) | extendable(1)
//!                                       | iteration_exponent(4)
//!     - share_params: 2 words = 20 bits — group_index(4)
//!                                       | (group_threshold − 1)(4)
//!                                       | (group_count − 1)(4)
//!                                       | member_index(4)
//!                                       | (member_threshold − 1)(4)
//!     - padded share value: variable — left-padded with 0..=8 zero
//!                          bits so that the total padded-value bit
//!                          length is a multiple of 10
//!     - checksum:     3 words = 30 bits — RS1024 over
//!                                       `cs || (id_exp .. value)`
//!
//!   Thresholds stored as `T − 1` (4-bit field 0..=15 ↔ threshold
//!   1..=16). Group/member INDICES are stored as-is (already 0..=15).
//!   Customization-string routing derives from the decoded `ext` bit:
//!     ext = 0 ⇒ cs = b"shamir"
//!     ext = 1 ⇒ cs = b"shamir_extendable"
//!
//! Parse-error ordering (per python `Share.from_mnemonic` @ 17fcce14
//! and SPEC §2.5 refusal rows 9 / 10 / 16 / 23):
//!   1. word-validity (any unknown word ⇒ `UnknownWord`)
//!   2. word-count gate (`< MIN_MNEMONIC_LENGTH_WORDS` ⇒ `InvalidPadding`)
//!   3. pre-checksum padding sanity (`padding_bits > 8` ⇒ `InvalidPadding`)
//!   4. RS1024 checksum (`InvalidChecksum`)
//!   5. `group_count < group_threshold` ⇒ `GroupThresholdExceedsCount`
//!      (P1c-E.1 expansion; mirrors python `share.py:216-219`)
//!   6. non-zero leading padding bits in value field ⇒ `InvalidPadding`
//!
//! `parse_slip39_share` parses a SINGLE share; `share_idx` carried by
//! `InvalidChecksum` / `UnknownWord` / `InvalidPadding` is therefore
//! always `0`. The `slip39_combine` caller remaps `share_idx` to the
//! position within the combine input.
//!
//! Coverage matrix:
//!   - Vector #1 (non-extendable, 128-bit, 1-of-1) — positive parse;
//!     pins identifier-bit extraction + all metadata fields.
//!   - Vector #42 (extendable, 128-bit, 1-of-1) — positive parse with
//!     `extendable == true`; pins ext-bit decoding + RS1024 cs routing.
//!     iteration_exponent extracted via the same bit-pattern as the
//!     identifier check — the upstream fixture happens to use
//!     `iter_exp = 3` here (NOT 0 by symmetry with vector #1).
//!   - Render round-trip for both vectors: `render(parse(s)) == s`.
//!   - Vector #2 (vector #1 last word flipped) ⇒
//!     `Err(InvalidChecksum { share_idx: 0 })`.
//!   - Vector #3 (invalid leading padding) ⇒
//!     `Err(InvalidPadding { share_idx: 0 })`.
//!   - Synthetic: vector #1 with a non-wordlist token substituted at a
//!     known position ⇒ `Err(UnknownWord { share_idx: 0, word_idx })`.
//!   - Synthetic too-short share (19 words, below
//!     `MIN_MNEMONIC_LENGTH_WORDS = 20` per Python `share.py`) ⇒
//!     `Err(InvalidPadding { share_idx: 0 })`. SPEC §2.5 elides this
//!     row into the InvalidPadding fold (the variant's "encoding
//!     violation" semantics per `error.rs:91-94`).
//!   - Synthetic disallowed-length share (21 words: `(10 * (21-7)) %
//!     16 = 12 > 8`) ⇒ same InvalidPadding fold.
//!
//! SPEC anchors:
//!   - `design/SPEC_slip39_v0_13_0.md` §2.1 (Share struct + public surface).
//!   - `design/SPEC_slip39_v0_13_0.md` §2.5 rows 9 / 10 / 16 (parse
//!     refusal mapping).
//!   - python-shamir-mnemonic `share.py` `_encode_id_exp`,
//!     `_encode_share_params`, `Share.from_mnemonic` (commit `17fcce14`).
//!   - `tests/fixtures/slip39_vectors.json` vectors #1, #2, #3, #42.

use mnemonic_toolkit::slip39::wordlist;
use mnemonic_toolkit::slip39::{
    parse_slip39_share, render_slip39_share, Slip39Error,
};

// ============================================================================
// Spec-anchor mnemonics — copied byte-for-byte from
// `tests/fixtures/slip39_vectors.json` entries 1 / 2 / 3 / 42.
// ============================================================================

const VECTOR_1: &str = "duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision keyboard";

const VECTOR_2_INVALID_CHECKSUM: &str = "duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision kidney";

const VECTOR_3_INVALID_PADDING: &str = "duckling enlarge academic academic email result length solution fridge kidney coal piece deal husband erode duke ajar music cargo fitness";

const VECTOR_42_EXTENDABLE: &str = "testify swimming academic academic column loyalty smear include exotic bedroom exotic wrist lobe cover grief golden smart junior estimate learn";

// ============================================================================
// Positive parse — vector #1 (non-extendable, 128-bit, 1-of-1)
// ============================================================================

#[test]
fn parse_vector_1_decodes_one_of_one_metadata() {
    let s = parse_slip39_share(VECTOR_1).expect("vector #1 must parse");
    assert!(!s.extendable, "vector #1 is non-extendable (cs=b\"shamir\")");
    assert_eq!(s.iteration_exponent, 0);
    assert_eq!(s.group_threshold, 1);
    assert_eq!(s.group_count, 1);
    assert_eq!(s.member_threshold, 1);
    assert_eq!(s.group_index, 0);
    assert_eq!(s.member_index, 0);
}

#[test]
fn parse_vector_1_identifier_matches_bit_extraction() {
    // The first two share words bit-pack `id_exp_int`:
    //   id_exp_int = (identifier << 5) | (ext << 4) | iter_exp
    // For vector #1: ext=0, iter_exp=0, so identifier = id_exp_int >> 5.
    // Build id_exp_int from the first two word indices and check the
    // parser extracts the same identifier.
    let d = u32::from(wordlist::word_to_index("duckling").expect("duckling in wordlist"));
    let e = u32::from(wordlist::word_to_index("enlarge").expect("enlarge in wordlist"));
    let id_exp_int = (d << 10) | e;
    let expected_identifier = (id_exp_int >> 5) as u16;
    let s = parse_slip39_share(VECTOR_1).expect("vector #1 must parse");
    assert_eq!(s.identifier, expected_identifier);
}

// ============================================================================
// Positive parse — vector #42 (extendable, 128-bit, 1-of-1)
// ============================================================================

#[test]
fn parse_vector_42_decodes_extendable_bit() {
    // Vector #42's checksum verifies ONLY under cs=b"shamir_extendable"
    // (anchored at `lib_slip39_rs1024.rs::vector_42_extendable_*`);
    // therefore a parse that succeeds proves the parser decodes the
    // ext bit and routes the cs correctly.
    //
    // iteration_exponent is NOT 0 here (unlike vector #1): the upstream
    // fixture-generator chose word indices that yield iter_exp = 3.
    // Extract via the same bit pattern as the identifier-extraction
    // anchor below — never via a hardcoded literal, because there is
    // no semantic reason for any particular iter_exp for an extendable
    // 1-of-1 vector.
    let t = u32::from(wordlist::word_to_index("testify").expect("testify in wordlist"));
    let sw = u32::from(wordlist::word_to_index("swimming").expect("swimming in wordlist"));
    let id_exp_int = (t << 10) | sw;
    let expected_iter_exp = (id_exp_int & 0xF) as u8;
    let expected_identifier = (id_exp_int >> 5) as u16;
    let s = parse_slip39_share(VECTOR_42_EXTENDABLE).expect("vector #42 must parse");
    assert!(s.extendable, "vector #42 is extendable (cs=b\"shamir_extendable\")");
    assert_eq!(s.iteration_exponent, expected_iter_exp);
    assert_eq!(s.identifier, expected_identifier);
    assert_eq!(s.group_threshold, 1);
    assert_eq!(s.group_count, 1);
    assert_eq!(s.member_threshold, 1);
    assert_eq!(s.group_index, 0);
    assert_eq!(s.member_index, 0);
}

// ============================================================================
// Render round-trip
//
// Vectors-on-disk are produced by `python-shamir-mnemonic` (an
// independent reference impl), so a `parse → render → string-equal`
// round-trip is a strong external anchor: a symmetric pack/unpack bug
// in our impl would have to produce the *same wrong* wire bytes as the
// correct-by-construction Python reference, which is implausible.
//
// The narrower symmetric-bug class — bugs that affect rendering of
// metadata configurations NOT covered by the vectors set — is left
// un-pinned here. Closing this gap requires a direct render test that
// constructs a `Share` from explicit field values; that depends on a
// GREEN design decision for P1c-D about `Share` constructibility
// (since SPEC §2.1 marks the `value` field private). Re-visit at the
// post-GREEN R0 review once the `Share` surface is concrete.
// ============================================================================

#[test]
fn render_round_trip_vector_1() {
    let s = parse_slip39_share(VECTOR_1).expect("vector #1 must parse");
    let rendered = render_slip39_share(&s);
    assert_eq!(rendered, VECTOR_1);
}

#[test]
fn render_round_trip_vector_42() {
    let s = parse_slip39_share(VECTOR_42_EXTENDABLE).expect("vector #42 must parse");
    let rendered = render_slip39_share(&s);
    assert_eq!(rendered, VECTOR_42_EXTENDABLE);
}

// ============================================================================
// Negative — RS1024 checksum failure (vectors.json #2)
// ============================================================================

#[test]
fn parse_vector_2_returns_invalid_checksum() {
    let err = parse_slip39_share(VECTOR_2_INVALID_CHECKSUM)
        .expect_err("vector #2 must refuse (RS1024 mismatch)");
    assert_eq!(err, Slip39Error::InvalidChecksum { share_idx: 0 });
}

// ============================================================================
// Negative — invalid padding bits (vectors.json #3)
// ============================================================================

#[test]
fn parse_vector_3_returns_invalid_padding() {
    // Vector #3 is constructed with a valid RS1024 checksum but the
    // leading padding bits of the share-value field are non-zero.
    // Parser must surface this after the checksum check passes.
    let err = parse_slip39_share(VECTOR_3_INVALID_PADDING)
        .expect_err("vector #3 must refuse (non-zero padding)");
    assert_eq!(err, Slip39Error::InvalidPadding { share_idx: 0 });
}

// ============================================================================
// Negative — unknown word at a known position
// ============================================================================

#[test]
fn parse_unknown_word_reports_position() {
    // Replace the 6th word (0-based index 5, "result" in vector #1) with
    // a non-wordlist token. Per the parse-error ordering above, word
    // validity is checked at entry — so this case wins over checksum /
    // padding even though both would also fail.
    let mut words: Vec<&str> = VECTOR_1.split_whitespace().collect();
    words[5] = "notawordatall";
    let tampered = words.join(" ");
    let err = parse_slip39_share(&tampered)
        .expect_err("tampered share with unknown word must refuse");
    assert_eq!(err, Slip39Error::UnknownWord { share_idx: 0, word_idx: 5 });
}

// ============================================================================
// Negative — share too short / disallowed word count
//
// Python `Share.from_mnemonic` enforces two pre-checksum length gates:
//   1. `len(words) < MIN_MNEMONIC_LENGTH_WORDS` (=20 for SLIP-39).
//   2. `padding_len = (RADIX_BITS * (len - METADATA_LENGTH_WORDS)) % 16
//       > 8` — fires for word counts like 21 (`(10*14)%16 = 12`).
// Both refuse with `MnemonicError("Invalid mnemonic length.")` upstream.
//
// SPEC §2.5 elides these rows; the toolkit's `Slip39Error` has no
// dedicated `BadShareWordCount` variant, so the natural fold is into
// `InvalidPadding { share_idx }` per `error.rs:91-94`'s
// "encoding violation" semantics. If GREEN chooses a different fold,
// flip both these assertions to match — and update SPEC §2.5.
// ============================================================================

#[test]
fn parse_too_short_mnemonic_returns_invalid_padding() {
    // 19 words — below MIN_MNEMONIC_LENGTH_WORDS = 20.
    let words: Vec<&str> = VECTOR_1.split_whitespace().take(19).collect();
    let short = words.join(" ");
    let err = parse_slip39_share(&short).expect_err("19-word share must refuse");
    assert_eq!(err, Slip39Error::InvalidPadding { share_idx: 0 });
}

#[test]
fn parse_disallowed_word_count_returns_invalid_padding() {
    // 21 words: `(10 * (21 - 7)) % 16 = 12`, which is > 8.
    let words: Vec<&str> = VECTOR_1
        .split_whitespace()
        .chain(std::iter::once("academic"))
        .collect();
    let long = words.join(" ");
    let err = parse_slip39_share(&long).expect_err("21-word share must refuse");
    assert_eq!(err, Slip39Error::InvalidPadding { share_idx: 0 });
}

// ============================================================================
// Negative — group_threshold > group_count (parse-time refusal).
//
// Per python-shamir-mnemonic `share.py` @ commit 17fcce14 lines 216-219:
//
//     if self.group_count < self.group_threshold:
//         raise MnemonicError(...)
//
// The toolkit's parser must mirror this; the P1c-E.1 expansion folds in
// a dedicated `Slip39Error::GroupThresholdExceedsCount` variant (plan
// §2.3 + §8). Pins vectors.json entry #10 (the 128-bit case).
//
// Vector #10 share 1's share_params words are "acrobat acid" (wordlist
// indices 4 and 1):
//     share_params_int = (4 << 10) | 1 = 0x1001
//       member_threshold = (0x1001 & 0xF) + 1 = 2
//       member_index     = (0x1001 >> 4) & 0xF = 0
//       group_count      = ((0x1001 >> 8) & 0xF) + 1 = 1
//       group_threshold  = ((0x1001 >> 12) & 0xF) + 1 = 2
//       group_index      = (0x1001 >> 16) & 0xF = 0
//
// So group_threshold(2) > group_count(1) — the parse must refuse with
// the new variant carrying those values. The refusal fires AFTER the
// RS1024 checksum check (the fixture's share has a valid checksum) and
// AFTER share_params decoding, but BEFORE the value-decode step.
// ============================================================================

const VECTOR_10_SHARE_1: &str = "music husband acrobat acid artist finance center either graduate swimming object bike medical clothes station aspect spider maiden bulb welcome";

#[test]
fn parse_too_high_group_threshold_returns_group_threshold_exceeds_count() {
    let err = parse_slip39_share(VECTOR_10_SHARE_1)
        .expect_err("vector #10 share 1 must refuse (group_threshold > group_count)");
    assert_eq!(
        err,
        Slip39Error::GroupThresholdExceedsCount {
            share_idx: 0,
            threshold: 2,
            count: 1,
        },
    );
}
