//! Embedded Electrum wordlists (SPEC v0.8 §14).
//!
//! Source: `spesmilo/electrum/electrum/wordlist/` at commit
//! `e1099925e30d91dd033815b512f00582a8795d25` (2026-05-07). Per-file blob SHAs:
//!
//! | Wordlist             | Words | Blob SHA                                  |
//! |----------------------|-------|-------------------------------------------|
//! | chinese_simplified   | 2048  | b90f1ed85b2a90855a82f12cb9d83124724cff5f  |
//! | japanese             | 2048  | c4c9dca4e58694e24a0ec398f15849269aa63c7a  |
//! | portuguese           | 1626  | 420d3d12aa0b78d47b1b24d56c4c1d7e8d01fc20  |
//! | spanish              | 2048  | d0900c2c78fb441714c67df8e83736616d915d63  |
//!
//! (Portuguese is unusual: 1626 words after stripping the file's 27-line
//! Monero-project copyright header + 1 blank line. Base-N arithmetic in
//! `crate::electrum` parameterizes on `wordlist().len()`.)
//!
//! English re-uses `bip39::Language::English.word_list()` (byte-identical to
//! Electrum's English wordlist; established at v0.7 Phase 3).
//!
//! Note: portuguese has 1626 words (not 2048); base-N arithmetic in
//! `crate::electrum` is parameterized on `wordlist().len()` rather than
//! hardcoded 2048.

use std::sync::OnceLock;
use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

/// Electrum wordlist selector for `--electrum-language`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ElectrumWordlist {
    English,
    Spanish,
    Japanese,
    Portuguese,
    ChineseSimplified,
}

impl ElectrumWordlist {
    /// Stable label for stderr / SPEC §14 info-lines. Reserved for the
    /// follow-up wordlist-scoped info-line emission (currently the
    /// `--electrum-language` flag selects silently).
    #[allow(dead_code)]
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::English => "english",
            Self::Spanish => "spanish",
            Self::Japanese => "japanese",
            Self::Portuguese => "portuguese",
            Self::ChineseSimplified => "chinese-simplified",
        }
    }

    /// Wordlist as a slice of normalized words (Electrum's `normalize_text`
    /// pre-applied: NFKD + lowercase + combining-mark strip + whitespace
    /// collapse). Word lookup against this slice expects a normalized input.
    pub(crate) fn words(&self) -> &'static [String] {
        match self {
            Self::English => english_words(),
            Self::Spanish => spanish_words(),
            Self::Japanese => japanese_words(),
            Self::Portuguese => portuguese_words(),
            Self::ChineseSimplified => chinese_simplified_words(),
        }
    }

    /// Base for the multiply/divide arithmetic: equals `wordlist.len()`.
    pub(crate) fn base(&self) -> u32 {
        self.words().len() as u32
    }
}

// ============================================================================
// Wordlist initialization. Each file is normalized once at first access.
// ============================================================================

static ENGLISH: OnceLock<Vec<String>> = OnceLock::new();
static SPANISH: OnceLock<Vec<String>> = OnceLock::new();
static JAPANESE: OnceLock<Vec<String>> = OnceLock::new();
static PORTUGUESE: OnceLock<Vec<String>> = OnceLock::new();
static CHINESE_SIMPLIFIED: OnceLock<Vec<String>> = OnceLock::new();

fn english_words() -> &'static [String] {
    ENGLISH
        .get_or_init(|| {
            bip39::Language::English
                .word_list()
                .iter()
                .map(|w| normalize_electrum(w))
                .collect()
        })
        .as_slice()
}

fn spanish_words() -> &'static [String] {
    SPANISH
        .get_or_init(|| parse_wordlist(include_str!("electrum_spanish.txt")))
        .as_slice()
}

fn japanese_words() -> &'static [String] {
    JAPANESE
        .get_or_init(|| parse_wordlist(include_str!("electrum_japanese.txt")))
        .as_slice()
}

fn portuguese_words() -> &'static [String] {
    PORTUGUESE
        .get_or_init(|| parse_wordlist(include_str!("electrum_portuguese.txt")))
        .as_slice()
}

fn chinese_simplified_words() -> &'static [String] {
    CHINESE_SIMPLIFIED
        .get_or_init(|| parse_wordlist(include_str!("electrum_chinese_simplified.txt")))
        .as_slice()
}

fn parse_wordlist(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(normalize_electrum)
        .collect()
}

/// Electrum `mnemonic.py::normalize_text` (excluding the CJK-whitespace step,
/// which only affects multi-word phrases — not individual words).
///
/// Steps: NFKD → lowercase → strip Unicode combining marks → trim. The full
/// CJK-whitespace handling is done at the phrase level in
/// `crate::electrum::normalize_phrase`.
pub(crate) fn normalize_electrum(s: &str) -> String {
    // Use the Unicode Canonical-Combining-Class table via
    // `unicode_normalization::char::is_combining_mark` to match Python's
    // `unicodedata.combining(c) != 0` behavior across all scripts (Latin
    // diacriticals, Japanese voiced/semi-voiced sound marks, Hebrew points,
    // Arabic harakat, etc.). A hand-rolled range table would miss Japanese
    // U+3099 / U+309A and silently break decode round-trips for `ぶ`/`ぷ`.
    s.nfkd()
        .filter(|c| !is_combining_mark(*c))
        .flat_map(|c| c.to_lowercase())
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_byte_identical_to_bip39() {
        let bip39_en = bip39::Language::English.word_list();
        let our_en = ElectrumWordlist::English.words();
        assert_eq!(our_en.len(), bip39_en.len());
        for (a, b) in our_en.iter().zip(bip39_en.iter()) {
            // bip39's English wordlist is plain ASCII; normalize is a no-op.
            assert_eq!(a, *b);
        }
    }

    #[test]
    fn spanish_loads_2048_words() {
        let words = ElectrumWordlist::Spanish.words();
        assert_eq!(words.len(), 2048);
    }

    #[test]
    fn portuguese_loads_1626_words() {
        // Electrum's Portuguese wordlist is unusual — 1626 words after stripping
        // the file's Monero copyright header. Base-N arithmetic must respect this.
        let words = ElectrumWordlist::Portuguese.words();
        assert_eq!(words.len(), 1626);
    }

    #[test]
    fn japanese_loads_2048_words() {
        let words = ElectrumWordlist::Japanese.words();
        assert_eq!(words.len(), 2048);
    }

    #[test]
    fn chinese_simplified_loads_2048_words() {
        let words = ElectrumWordlist::ChineseSimplified.words();
        assert_eq!(words.len(), 2048);
    }

    #[test]
    fn normalize_strips_combining_diacritics() {
        // `almíbar` (NFC) → `almi` + combining-acute + `bar` → strip → `almibar`.
        assert_eq!(normalize_electrum("almíbar"), "almibar");
        // Lowercase applies.
        assert_eq!(normalize_electrum("ALMÍBAR"), "almibar");
    }

    #[test]
    fn base_matches_word_count() {
        assert_eq!(ElectrumWordlist::English.base(), 2048);
        assert_eq!(ElectrumWordlist::Spanish.base(), 2048);
        assert_eq!(ElectrumWordlist::Japanese.base(), 2048);
        assert_eq!(ElectrumWordlist::Portuguese.base(), 1626);
        assert_eq!(ElectrumWordlist::ChineseSimplified.base(), 2048);
    }
}
