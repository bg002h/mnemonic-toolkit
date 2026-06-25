//! BIP-39 **English** symbol ↔ word map (plan §3).
//!
//! A Word-Card *symbol* is a `GF(2^11)` element `0..=2047`; its integer value is
//! exactly the **BIP-39 English wordlist index** (bit₁₀…bit₀, MSB-first, matching
//! the bech32/codec convention — plan §3). We source the wordlist from the
//! already-pinned `bip39` crate so there is exactly **one** source of truth: the
//! word's symbol value IS its position in `bip39::Language::English.word_list()`.
//!
//! `tests/wordmap.rs` asserts an all-2048 round-trip and equality against the
//! `bip39` English list directly.

use bip39::Language;

/// Number of BIP-39 words (one per 11-bit symbol).
pub const WORD_COUNT: usize = 2048;

/// Map a BIP-39 English word to its symbol value (its 11-bit list index,
/// `0..=2047`). Returns `None` if `word` is not in the English wordlist.
///
/// Matching is exact (case-sensitive); BIP-39 English words are all lowercase.
pub fn word_to_symbol(word: &str) -> Option<u16> {
    // `find_word` for English uses binary search and returns the list index,
    // which is precisely the symbol value (plan §3).
    Language::English.find_word(word)
}

/// Map a symbol value (`0..=2047`) to its BIP-39 English word. Returns `None` if
/// `symbol >= 2048` (not a valid 11-bit symbol).
pub fn symbol_to_word(symbol: u16) -> Option<&'static str> {
    let list = Language::English.word_list();
    list.get(symbol as usize).copied()
}
