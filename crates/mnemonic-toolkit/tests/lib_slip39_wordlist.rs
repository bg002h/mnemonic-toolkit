//! v0.13.0 P1c — library tests for SLIP-39 wordlist (1024 words).
//!
//! The wordlist is the canonical Trezor `python-shamir-mnemonic`
//! `wordlists/wordlist.txt` (vendored at the P1c data-files commit).
//! Per SLIP-0039 §3.4: 1024 ASCII lowercase words, lexicographically
//! sorted; each word's first 4 characters are unique (a property the
//! spec leans on for typo recovery but which we do not test at this
//! layer — see P1c rs1024.rs harness for BCH guarantees).
//!
//! API:
//!   - `word_to_index(word: &str) -> Option<u16>` — `Some(0..=1023)` on
//!     hit, `None` on miss. Case-sensitive (ASCII lower per spec).
//!   - `index_to_word(idx: u16) -> Option<&'static str>` — `Some` for
//!     `idx < 1024`, `None` otherwise.

use mnemonic_toolkit::slip39::wordlist;

// ============================================================================
// Cardinality + structural invariants
// ============================================================================

#[test]
fn has_exactly_1024_words() {
    // The defining property of the SLIP-39 wordlist: exactly 2^10 words
    // so each share-word encodes 10 bits.
    let mut count = 0u16;
    for i in 0u16..1024 {
        assert!(wordlist::index_to_word(i).is_some(), "missing word at index {i}");
        count += 1;
    }
    assert_eq!(count, 1024);
    assert!(wordlist::index_to_word(1024).is_none());
    assert!(wordlist::index_to_word(u16::MAX).is_none());
}

#[test]
fn wordlist_is_lexicographically_sorted() {
    let mut prev = wordlist::index_to_word(0).expect("word 0 must exist");
    for i in 1u16..1024 {
        let curr = wordlist::index_to_word(i).expect("word i must exist");
        assert!(
            prev < curr,
            "wordlist not sorted at index {i}: {prev:?} >= {curr:?}"
        );
        prev = curr;
    }
}

#[test]
fn words_are_ascii_lowercase() {
    for i in 0u16..1024 {
        let w = wordlist::index_to_word(i).unwrap();
        assert!(!w.is_empty(), "word at {i} is empty");
        assert!(
            w.bytes().all(|b| b.is_ascii_lowercase()),
            "word at {i} is not ASCII lowercase: {w:?}"
        );
    }
}

// ============================================================================
// Spec-pinned anchor words (first + last)
// ============================================================================

#[test]
fn word_zero_is_academic() {
    assert_eq!(wordlist::index_to_word(0), Some("academic"));
}

#[test]
fn word_1023_is_zero() {
    assert_eq!(wordlist::index_to_word(1023), Some("zero"));
}

// ============================================================================
// word_to_index lookups
// ============================================================================

#[test]
fn word_to_index_academic() {
    assert_eq!(wordlist::word_to_index("academic"), Some(0));
}

#[test]
fn word_to_index_zero() {
    assert_eq!(wordlist::word_to_index("zero"), Some(1023));
}

#[test]
fn word_to_index_unknown_returns_none() {
    assert_eq!(wordlist::word_to_index("notaslip39word"), None);
    assert_eq!(wordlist::word_to_index(""), None);
    // 'aardvark' < 'academic' lexicographically so it tests the head
    // of a binary search / hashmap miss without trailing-prefix issues.
    assert_eq!(wordlist::word_to_index("aardvark"), None);
}

#[test]
fn word_to_index_is_case_sensitive() {
    // SLIP-0039 §3.4 specifies ASCII lowercase. Mixed case must miss.
    assert_eq!(wordlist::word_to_index("Academic"), None);
    assert_eq!(wordlist::word_to_index("ACADEMIC"), None);
}

// ============================================================================
// Round-trip property
// ============================================================================

#[test]
fn round_trip_all_indices() {
    for i in 0u16..1024 {
        let w = wordlist::index_to_word(i).unwrap();
        assert_eq!(
            wordlist::word_to_index(w),
            Some(i),
            "round-trip failed for index {i} ({w:?})"
        );
    }
}

// ============================================================================
// BIP-39 vs SLIP-39: disjoint by spec (sanity smoke)
// ============================================================================

#[test]
fn wordlist_is_not_bip39_wordlist() {
    // Sanity: BIP-39 has "abandon" at index 0; SLIP-39 has "academic".
    // If someone accidentally embedded the BIP-39 wordlist this catches
    // it instantly.
    assert_ne!(wordlist::index_to_word(0), Some("abandon"));
    assert_eq!(wordlist::word_to_index("abandon"), None);
}
