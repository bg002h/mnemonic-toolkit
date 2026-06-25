//! KATs for the BIP-39 English symbol ↔ word map (plan §3).

use bip39::Language;
use wc_codec::wordmap::{symbol_to_word, word_to_symbol, WORD_COUNT};

#[test]
fn count_is_2048() {
    assert_eq!(WORD_COUNT, 2048);
}

/// All-2048 round-trip: symbol → word → symbol is the identity over 0..=2047.
#[test]
fn all_symbols_round_trip() {
    for s in 0..2048u16 {
        let word = symbol_to_word(s).unwrap_or_else(|| panic!("symbol {s} mapped to no word"));
        let back =
            word_to_symbol(word).unwrap_or_else(|| panic!("word {word:?} mapped to no symbol"));
        assert_eq!(back, s, "round-trip failed for symbol {s} (word {word:?})");
    }
}

/// The map equals `bip39`'s canonical English list, position-for-position — the
/// single-source-of-truth invariant (plan §3).
#[test]
fn equals_bip39_english_list() {
    let list = Language::English.word_list();
    for (i, &w) in list.iter().enumerate() {
        assert_eq!(
            symbol_to_word(i as u16),
            Some(w),
            "symbol_to_word({i}) must equal bip39 English[{i}]"
        );
        assert_eq!(
            word_to_symbol(w),
            Some(i as u16),
            "word_to_symbol({w:?}) must equal index {i}"
        );
    }
}

/// Symbols out of the 11-bit range and non-words are rejected.
#[test]
fn rejects_out_of_range_and_non_words() {
    assert_eq!(symbol_to_word(2048), None, "2048 is out of range");
    assert_eq!(symbol_to_word(u16::MAX), None, "u16::MAX is out of range");
    assert_eq!(word_to_symbol("notawordatall"), None);
    assert_eq!(word_to_symbol(""), None);
    assert_eq!(
        word_to_symbol("Abandon"),
        None,
        "case-sensitive: capitalized rejected"
    );
}

/// A couple of known anchors at the ends of the list.
#[test]
fn known_anchors() {
    assert_eq!(symbol_to_word(0), Some("abandon"));
    assert_eq!(symbol_to_word(2047), Some("zoo"));
    assert_eq!(word_to_symbol("abandon"), Some(0));
    assert_eq!(word_to_symbol("zoo"), Some(2047));
}
