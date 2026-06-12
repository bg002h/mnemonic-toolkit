//! BIP-39 final-word completer.
//!
//! Given an N-1-word partial mnemonic and a language, return the
//! lexicographically sorted set of wordlist entries that, when appended
//! as the Nth word, yield a phrase with a valid BIP-39 checksum.
//!
//! See `design/SPEC_final_word_v0_11_0.md` §2.1 for the algorithm
//! contract. Output set size is a function of N only:
//!
//! | N | Set size = 2^(11 − CS) |
//! |---|------------------------|
//! | 12 | 128 |
//! | 15 | 64  |
//! | 18 | 32  |
//! | 21 | 16  |
//! | 24 | 8   |
//!
//! Algorithm: naïve enumeration over the 2048-entry wordlist with
//! `bip39::Mnemonic::parse_in` as the correctness oracle. Costs 2048
//! SHA-256 ops per query (~milliseconds). Correctness is delegated to
//! the well-tested bip39 crate; no hand-rolled checksum logic.
//!
//! This module returns a dedicated `FinalWordError`. The CLI handler at
//! `src/cmd/final_word.rs` (P2) wraps `FinalWordError` into
//! `ToolkitError` at the binary boundary — keeping the library surface
//! self-contained.

/// The language argument for `final_word_candidates`. Mirror of the
/// binary-private `CliLanguage` enum at `src/language.rs`, scoped to
/// the lib surface so external tests don't need access to the binary's
/// `mod language;`.
///
/// CLI converts `crate::language::CliLanguage` → `FinalWordLanguage` at
/// the call boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalWordLanguage {
    English,
    SimplifiedChinese,
    TraditionalChinese,
    Czech,
    French,
    Italian,
    Japanese,
    Korean,
    Portuguese,
    Spanish,
}

impl From<FinalWordLanguage> for bip39::Language {
    fn from(l: FinalWordLanguage) -> bip39::Language {
        match l {
            FinalWordLanguage::English => bip39::Language::English,
            FinalWordLanguage::SimplifiedChinese => bip39::Language::SimplifiedChinese,
            FinalWordLanguage::TraditionalChinese => bip39::Language::TraditionalChinese,
            FinalWordLanguage::Czech => bip39::Language::Czech,
            FinalWordLanguage::French => bip39::Language::French,
            FinalWordLanguage::Italian => bip39::Language::Italian,
            FinalWordLanguage::Japanese => bip39::Language::Japanese,
            FinalWordLanguage::Korean => bip39::Language::Korean,
            FinalWordLanguage::Portuguese => bip39::Language::Portuguese,
            FinalWordLanguage::Spanish => bip39::Language::Spanish,
        }
    }
}

/// Errors returned by `final_word_candidates`. Library-local; the CLI
/// handler wraps each variant into `ToolkitError::BadInput` /
/// `ToolkitError::Bip39` at the binary boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalWordError {
    /// Partial word count is not in `{11, 14, 17, 20, 23}`. The carried
    /// value is the actual word count seen.
    BadWordCount(usize),
    /// One of the partial words is not in the selected language's
    /// BIP-39 wordlist. The carried value is the 0-based position.
    UnknownWord { position: usize },
}

impl std::fmt::Display for FinalWordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinalWordError::BadWordCount(got) => write!(
                f,
                "final-word: got {} words; expected one of [11, 14, 17, 20, 23] \
                 (target = K+1 must be in {{12,15,18,21,24}})",
                got,
            ),
            FinalWordError::UnknownWord { position } => write!(
                f,
                "final-word: unknown BIP-39 word at position {} (not in selected wordlist; \
                 did you pick the right --language?)",
                position,
            ),
        }
    }
}

impl std::error::Error for FinalWordError {}

/// Accepted partial word counts (= N-1 for each valid N).
const VALID_PARTIAL_COUNTS: &[usize] = &[11, 14, 17, 20, 23];

/// Compute the set of valid Nth-word completions for the given partial.
///
/// Returns lexicographically sorted `Vec<&'static str>` borrowed from
/// `bip39::Language::word_list()`. Set size is deterministic by partial
/// length (see module-doc table).
///
/// Errors:
/// - [`FinalWordError::BadWordCount`] if the partial's word count is not in `{11, 14, 17, 20, 23}`.
/// - [`FinalWordError::UnknownWord`] if any partial word is not in the
///   selected language's BIP-39 wordlist.
///
/// Note: the working buffer is bare `String`, not `Zeroizing<String>`.
/// The partial is secret material, but the per-iteration local drops
/// at the bottom of each loop body; caller-side wrap of the partial
/// is the canonical scrub site (CLI handler at `cmd::final_word::run`).
pub fn final_word_candidates(
    partial_phrase: &str,
    language: FinalWordLanguage,
) -> Result<Vec<&'static str>, FinalWordError> {
    let words: Vec<&str> = partial_phrase.split_whitespace().collect();

    if !VALID_PARTIAL_COUNTS.contains(&words.len()) {
        return Err(FinalWordError::BadWordCount(words.len()));
    }

    let lang: bip39::Language = language.into();
    let wordlist: &'static [&'static str; 2048] = lang.word_list();

    // Upfront unknown-word check so error attribution is precise.
    // Without this the first parse_in probe would fail with InvalidChecksum
    // for nearly every random partial, masking the unknown-word condition.
    for (i, w) in words.iter().enumerate() {
        if !wordlist.contains(w) {
            return Err(FinalWordError::UnknownWord { position: i });
        }
    }

    let mut candidates: Vec<&'static str> = Vec::with_capacity(128);
    for &candidate in wordlist {
        let mut full = String::with_capacity(partial_phrase.len() + 1 + candidate.len());
        full.push_str(partial_phrase);
        full.push(' ');
        full.push_str(candidate);
        if bip39::Mnemonic::parse_in(lang, &full).is_ok() {
            candidates.push(candidate);
        }
    }
    candidates.sort_unstable();
    Ok(candidates)
}
