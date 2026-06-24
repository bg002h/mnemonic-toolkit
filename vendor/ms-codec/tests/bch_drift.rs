//! Drift gate for `ms_codec::bch` public-surface BCH primitives (Phase B.3).
//!
//! Asserts that [`bch_verify_regular`] accepts a known-valid ms1 vector
//! pulled from `tests/vectors/v0.1.json` (the canonical 12-word abandon
//! entropy). Catches:
//!
//! - accidental drift in `MS_REGULAR_CONST` (would yield `false`),
//! - accidental drift in `GEN_REGULAR` / `polymod_run` / `hrp_expand`,
//! - accidental privatization of any of the imported symbols
//!   (the file stops compiling).
//!
//! See plan §2.B.2 for the rationale; Phase B.0 (e) for the byte-exact
//! cross-check against the toolkit's vendored constant.

use ms_codec::bch::bch_verify_regular;

/// Bech32 alphabet (BIP 173 / BIP 93). Index = 5-bit value.
const BECH32_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// Decode the data part of a bech32-family string (everything after the
/// final `'1'` separator) into 5-bit values. Panics on invalid char —
/// fine for a test-only helper exercising a hard-coded fixture.
fn data_part_to_u5(s: &str) -> Vec<u8> {
    let sep = s.rfind('1').expect("missing '1' separator");
    let mut inv = [0xFFu8; 128];
    for (i, &c) in BECH32_ALPHABET.iter().enumerate() {
        inv[c as usize] = i as u8;
    }
    s[sep + 1..]
        .chars()
        .map(|c| {
            let b = c as usize;
            assert!(b < 128, "non-ASCII char in data part: {c:?}");
            let v = inv[b];
            assert!(v != 0xFF, "char {c:?} not in bech32 alphabet");
            v
        })
        .collect()
}

#[test]
fn bch_verify_regular_holds_for_known_valid_ms1() {
    // Canonical 12-word abandon entropy vector from
    // `tests/vectors/v0.1.json` (first entry; mirrored verbatim).
    const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

    let data_with_checksum = data_part_to_u5(VALID_MS1);
    assert!(
        bch_verify_regular("ms", &data_with_checksum),
        "bch_verify_regular rejected canonical 12-word abandon ms1 vector — \
         either MS_REGULAR_CONST drifted or a polymod primitive regressed"
    );
}

#[test]
fn bch_verify_regular_rejects_corrupted_ms1() {
    // Same vector with a single char swapped in the data part (q→p at
    // index 5 of the data: position 8 of the full string, well clear of
    // the checksum tail). The single substitution moves the polymod
    // residue off the target by exactly one generator delta.
    const CORRUPTED_MS1: &str = "ms10entrspqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

    let data_with_checksum = data_part_to_u5(CORRUPTED_MS1);
    assert!(
        !bch_verify_regular("ms", &data_with_checksum),
        "bch_verify_regular accepted a corrupted ms1 vector — verify routine \
         is silently passing all inputs (false-positive regression)"
    );
}
