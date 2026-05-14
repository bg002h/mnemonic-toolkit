//! Embedded SLIP-0039 1024-word English wordlist.
//!
//! Vendored from `python-shamir-mnemonic/shamir_mnemonic/wordlist.txt` at
//! upstream commit `17fcce14` (P1c data-files commit `310687c`,
//! 2026-05-14). Author date of the upstream file: 2024-05-16.
//!
//! Spec invariants (SLIP-0039 §3.4):
//!   - exactly 1024 words (so each share-word encodes 10 bits)
//!   - ASCII lowercase
//!   - lexicographically sorted
//!   - each word's first 4 characters are unique (not enforced at
//!     this layer; relied on by Trezor hardware-keyboard input)
//!
//! API: word-to-index lookup uses a `HashMap<&'static str, u16>`
//! lazily populated via `OnceLock` (the `src/wordlists/mod.rs`
//! Electrum precedent). Index-to-word is direct slice access into the
//! statically split wordlist (also `OnceLock`-gated to avoid splitting
//! the embedded blob more than once).

use std::collections::HashMap;
use std::sync::OnceLock;

const RAW_WORDLIST: &str = include_str!("slip39_english.txt");

static WORDS: OnceLock<Vec<&'static str>> = OnceLock::new();
static INDEX: OnceLock<HashMap<&'static str, u16>> = OnceLock::new();

fn words() -> &'static [&'static str] {
    WORDS.get_or_init(|| {
        let parsed: Vec<&'static str> = RAW_WORDLIST.lines().collect();
        debug_assert_eq!(
            parsed.len(),
            1024,
            "SLIP-39 wordlist must contain exactly 1024 words; got {}",
            parsed.len()
        );
        parsed
    })
}

fn index() -> &'static HashMap<&'static str, u16> {
    INDEX.get_or_init(|| {
        let ws = words();
        let mut map = HashMap::with_capacity(ws.len());
        for (i, w) in ws.iter().enumerate() {
            map.insert(*w, i as u16);
        }
        map
    })
}

/// Look up the 0-based wordlist index for `word`. Case-sensitive
/// (ASCII lowercase per spec).
///
/// Returns `Some(0..=1023)` on hit, `None` if `word` is not in the
/// wordlist.
pub fn word_to_index(word: &str) -> Option<u16> {
    index().get(word).copied()
}

/// Look up the word at the 0-based wordlist index `idx`.
///
/// Returns `Some(&'static str)` for `idx < 1024`, `None` otherwise.
pub fn index_to_word(idx: u16) -> Option<&'static str> {
    words().get(idx as usize).copied()
}
