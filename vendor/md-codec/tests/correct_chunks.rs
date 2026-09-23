//! F-449 stage 2 Task 2c (SPEC §8.9's stage-2 row): `correct_chunks` is the
//! BCH-correction half of `decode_with_correction`, exposed so `md repair`
//! can keep a correction on a card whose wire version this build cannot
//! decode. It corrects and decodes NOTHING, so an unsupported version is not
//! its concern.
//!
//! FIXTURES, committed so nobody re-derives the recipe (plan Task 2c Step 1).
//! Each was built from a real v4 md1 string by `codex32::unwrap_string`,
//! rewriting the version in the first 5-bit symbol, and `codex32::wrap_payload`
//! -- so the BCH checksum is valid -- then (for the `*_ONE_ERROR` strings)
//! corrupting data position 0.

#![allow(missing_docs)]

use md_codec::{CorrectionDetail, Error, correct_chunks, decode_with_correction};

/// Wire version 12 (outside the accepted set {4, 8}), one correctable error
/// at data position 0.
pub const V12_ONE_ERROR: &str = "md1qzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg";
/// The same card corrected: BCH-valid, zero errors, still version 12.
pub const V12_CLEAN: &str = "md1uzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg";
/// The v4 control: same payload at version 4, the same one error.
pub const V4_ONE_ERROR: &str = "md1qzfdsssjjtvyyw2fdssj54qqxppcgsc276kwwfnzntuh";

#[test]
fn correct_chunks_refuses_an_empty_set() {
    assert!(matches!(correct_chunks(&[]), Err(Error::ChunkSetEmpty)));
}

#[test]
fn correct_chunks_keeps_a_correction_whatever_the_wire_version() {
    let (strings, details) = correct_chunks(&[V12_ONE_ERROR]).expect("one error is correctable");
    assert_eq!(strings, vec![V12_CLEAN.to_string()]);
    assert_eq!(
        details,
        vec![CorrectionDetail {
            chunk_index: 0,
            position: 0,
            was: 'q',
            now: 'u'
        }]
    );
    // ...while the decoding entry point still refuses the version, unchanged.
    assert!(matches!(
        decode_with_correction(&[V12_ONE_ERROR]),
        Err(Error::WireVersionMismatch { got: 12 })
    ));
}

#[test]
fn the_clean_v12_fixture_is_bch_valid_and_version_12() {
    let (strings, details) = correct_chunks(&[V12_CLEAN]).expect("clean");
    assert!(details.is_empty(), "{details:?}");
    assert_eq!(strings, vec![V12_CLEAN.to_string()]);
    assert!(matches!(
        md_codec::decode_md1_string(V12_CLEAN),
        Err(Error::WireVersionMismatch { got: 12 })
    ));
}

/// `decode_with_correction` is now `correct_chunks` + decode, and must return
/// exactly what it returned before for a supported card.
#[test]
fn decode_with_correction_is_unchanged_on_a_supported_card() {
    let (_, details) = decode_with_correction(&[V4_ONE_ERROR]).expect("v4 decodes");
    assert_eq!(details.len(), 1);
    assert_eq!(
        (details[0].position, details[0].was, details[0].now),
        (0, 'q', '5')
    );
}

#[test]
fn correct_chunks_is_atomic_on_an_uncorrectable_chunk() {
    // Five substitutions exceed t = 4.
    let mut s: Vec<char> = V4_ONE_ERROR.chars().collect();
    for (i, pos) in [5usize, 9, 14, 20, 27].iter().enumerate() {
        s[*pos] = if s[*pos] == 'q' {
            'p'
        } else {
            ['q', 'z', 'r', 'y', 'x'][i]
        };
    }
    let bad: String = s.into_iter().collect();
    assert!(matches!(
        correct_chunks(&[&bad]),
        Err(Error::TooManyErrors { chunk_index: 0, .. })
    ));
}

/// A CHUNKED card whose 4-bit chunk-header version field reads 9 -- odd and
/// above 8 -- with one correctable error at data position 0 (whole-branch
/// review M-1: pins the PARITY half of `md repair`'s advice).
///
/// Why chunked: a single-string card cannot carry an odd version at all. Its
/// first symbol is `[divergent][v3][v2][v1][v0]` and bit 0 doubles as the
/// chunked flag, so an odd single-string version reads as a chunk header
/// (measured: a single-string "v9" reports version 12). A chunk header's
/// first symbol is `[v3][v2][v1][v0][chunked=1]`, so 9 is representable.
/// Built from `md encode --force-chunked
/// "wsh(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))"`, a
/// chunked-of-1 v4 card, by `with_chunk_version` below.
pub const CHUNKED_V4_CLEAN: &str = "md1f4frpqq9q2tvyyy5jmpprj5qqcyxppgqaudyc7r5fys4a";

/// `CHUNKED_V4_CLEAN` at chunk-header version 9, BCH-valid.
pub const CHUNKED_V9_CLEAN: &str = "md1n4frpqq9q2tvyyy5jmpprj5qqcyxppgqwcudeey7atgd5";
/// `CHUNKED_V9_CLEAN` with one substitution at data position 0 (`n` -> `q`).
pub const CHUNKED_V9_ONE_ERROR: &str = "md1q4frpqq9q2tvyyy5jmpprj5qqcyxppgqwcudeey7atgd5";

/// The fixture recipe for the chunked card: rewrite the top four bits of the
/// first symbol (the chunk-header version) and re-wrap.
fn with_chunk_version(clean: &str, version: u8) -> String {
    let (mut bytes, bits) = md_codec::codex32::unwrap_string(clean).expect("chunked card");
    bytes[0] = ((version & 0x0f) << 4) | (bytes[0] & 0x0f);
    md_codec::codex32::wrap_payload(&bytes, bits).expect("re-wrap")
}

/// The fixture recipe, executable: unwrap the clean v4 card, rewrite the
/// version in the first 5-bit symbol (`[divergent][v3][v2][v1][v0]`), and
/// re-wrap so the BCH checksum is valid.
fn with_version(clean_v4: &str, version: u8) -> String {
    let (mut bytes, bits) = md_codec::codex32::unwrap_string(clean_v4).expect("v4 card");
    let divergent = bytes[0] & 0x80;
    bytes[0] = divergent | ((version & 0x0f) << 3) | (bytes[0] & 0x07);
    md_codec::codex32::wrap_payload(&bytes, bits).expect("re-wrap")
}

#[test]
fn the_version_fixtures_follow_the_recipe() {
    let clean_v4 = correct_chunks(&[V4_ONE_ERROR]).unwrap().0.remove(0);
    assert_eq!(with_version(&clean_v4, 12), V12_CLEAN);
    assert_eq!(with_chunk_version(CHUNKED_V4_CLEAN, 9), CHUNKED_V9_CLEAN);
    assert!(matches!(
        decode_with_correction(&[CHUNKED_V9_CLEAN]),
        Err(Error::WireVersionMismatch { got: 9 })
    ));
    let (strings, details) = correct_chunks(&[CHUNKED_V9_ONE_ERROR]).unwrap();
    assert_eq!((strings[0].as_str(), details.len()), (CHUNKED_V9_CLEAN, 1));
    // The recipe is the identity at the card's own version.
    assert_eq!(with_chunk_version(CHUNKED_V4_CLEAN, 4), CHUNKED_V4_CLEAN);
}
