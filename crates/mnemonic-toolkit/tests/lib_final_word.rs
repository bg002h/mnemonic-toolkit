//! v0.11.0 P1 — library tests for the BIP-39 final-word completer.
//!
//! Test coverage scaffold (RED at first commit; GREEN after the impl
//! at `crates/mnemonic-toolkit/src/final_word.rs` lands):
//!
//! 1. Two user-locked named anchor vectors (`abandon × 11 about` 12-word
//!    target + `beef × 12` target): size, membership, SHA-pin.
//! 2. Per-N happy paths for N ∈ {15, 21} (vectors generated at test time
//!    via `bip39::Mnemonic::from_entropy_in`); for N=12, 18, 24 the
//!    Trezor canonical zero-entropy vectors from `tests/bip39_trezor_vectors.json`.
//! 3. Refusals: empty partial, wrong-count partial, unknown-word partial.
//! 4. Determinism: same input twice → identical output.
//! 5. Cross-language: spanish partial yields spanish candidates.
//!
//! See `design/SPEC_final_word_v0_11_0.md` §4 G1 + plan §"Phase 1 Test
//! coverage". SHA pins act as the regression backstop against algorithm
//! drift; the user-locked anchor vectors are the contract for v0.11.0
//! ship.

use bip39::{Language, Mnemonic};
use bitcoin::hashes::sha256;
use bitcoin::hashes::Hash as _;
use mnemonic_toolkit::final_word::{final_word_candidates, FinalWordLanguage};

// ============================================================================
// SHA-pin helper
// ============================================================================

/// SHA-256 hex of the sorted candidate list joined by `\n` (no trailing
/// newline). This is the canonical regression-backstop hash for an anchor.
fn sha_of_candidates(candidates: &[&str]) -> String {
    let joined = candidates.join("\n");
    let h = sha256::Hash::hash(joined.as_bytes());
    format!("{}", h)
}

// ============================================================================
// Anchor vector 1 — abandon × 11 about (Trezor canonical 12-word zero-entropy)
// ============================================================================

const ABANDON_11_PARTIAL: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";

/// User-locked anchor: full target phrase is `abandon × 11 about`
/// (Trezor's canonical zero-entropy vector at `tests/bip39_trezor_vectors.json:5`).
/// Pinned at GREEN time after running the algorithm.
const ABANDON_11_EXPECTED_SHA: &str =
    "8de70f5b9e3dbac3592961468da23645f8fe84f90bf0e81a6ee0251f6a14ee32";

#[test]
fn anchor_abandon_11_size_is_128() {
    let candidates = final_word_candidates(ABANDON_11_PARTIAL, FinalWordLanguage::English)
        .expect("abandon×11 should produce candidates");
    assert_eq!(
        candidates.len(),
        128,
        "N=12 must yield 2^(11-4) = 128 candidates regardless of input"
    );
}

#[test]
fn anchor_abandon_11_includes_about() {
    let candidates = final_word_candidates(ABANDON_11_PARTIAL, FinalWordLanguage::English).unwrap();
    assert!(
        candidates.contains(&"about"),
        "canonical Trezor zero-entropy 12-word phrase ends in 'about'; \
         candidate set: {:?}",
        candidates,
    );
}

#[test]
fn anchor_abandon_11_sha_pin() {
    let candidates = final_word_candidates(ABANDON_11_PARTIAL, FinalWordLanguage::English).unwrap();
    let actual = sha_of_candidates(&candidates);
    assert_eq!(
        actual, ABANDON_11_EXPECTED_SHA,
        "SHA-pin drift detected for abandon×11 anchor; if algorithm changed \
         intentionally, update ABANDON_11_EXPECTED_SHA",
    );
}

// ============================================================================
// Anchor vector 2 — beef × 12 target (uniform-word pathological)
// ============================================================================

const BEEF_11_PARTIAL: &str =
    "beef beef beef beef beef beef beef beef beef beef beef";

/// User-locked anchor: full target phrase is `beef × 12` (uniform-word
/// pathological vector). Pinned at GREEN time.
const BEEF_11_EXPECTED_SHA: &str =
    "0ced39634ee741a5235116886ba81ce6b232ad97ad2bac332de02da772d1331d";

/// Whether `beef` itself appears in the candidate set is computed and
/// pinned by the algorithm (not asserted a priori). The test captures
/// the verdict at GREEN time as a regression backstop.
/// Captured at GREEN-1 run 2026-05-13: `beef × 11` partial yields 128 candidates
/// including `"beef"` itself. (This is the SHA-256 prefix of the partial's
/// implied entropy bits matching the 4 checksum bits encoded by `beef`'s
/// position in the wordlist; the algorithm is deterministic, so this membership
/// is a stable regression backstop.)
const BEEF_11_CANDIDATE_INCLUDES_BEEF: bool = true;

#[test]
fn anchor_beef_11_size_is_128() {
    let candidates = final_word_candidates(BEEF_11_PARTIAL, FinalWordLanguage::English).unwrap();
    assert_eq!(
        candidates.len(),
        128,
        "N=12 always yields 128 candidates regardless of input entropy content"
    );
}

#[test]
fn anchor_beef_11_sha_pin() {
    let candidates = final_word_candidates(BEEF_11_PARTIAL, FinalWordLanguage::English).unwrap();
    let actual = sha_of_candidates(&candidates);
    assert_eq!(
        actual, BEEF_11_EXPECTED_SHA,
        "SHA-pin drift detected for beef×11 anchor",
    );
}

#[test]
fn anchor_beef_11_beef_membership_pinned() {
    let candidates = final_word_candidates(BEEF_11_PARTIAL, FinalWordLanguage::English).unwrap();
    assert_eq!(
        candidates.contains(&"beef"),
        BEEF_11_CANDIDATE_INCLUDES_BEEF,
        "beef-in-candidate-set membership is a regression-backstop; update \
         BEEF_11_CANDIDATE_INCLUDES_BEEF if algorithm change is intentional",
    );
}

// ============================================================================
// Per-N happy paths (vectors constructed via bip39::Mnemonic::from_entropy_in)
// ============================================================================

/// Helper: given a target N and entropy bytes (length N*4/3), construct
/// the full BIP-39 mnemonic, drop the last word, return (partial, last_word).
fn partial_and_last_word(entropy: &[u8]) -> (String, String) {
    let m = Mnemonic::from_entropy_in(Language::English, entropy).unwrap();
    let words: Vec<&str> = m.words().collect();
    let last = words.last().copied().unwrap().to_string();
    let partial = words[..words.len() - 1].join(" ");
    (partial, last)
}

fn assert_completer_round_trip(entropy: &[u8], expected_size: usize) {
    let (partial, original_last_word) = partial_and_last_word(entropy);
    let candidates = final_word_candidates(&partial, FinalWordLanguage::English).unwrap();
    assert_eq!(
        candidates.len(),
        expected_size,
        "N implied by partial len: expected {expected_size} candidates; got {}",
        candidates.len(),
    );
    assert!(
        candidates.contains(&original_last_word.as_str()),
        "round-trip: original last word '{original_last_word}' must appear in candidate set",
    );
    // Sortedness check.
    let mut sorted = candidates.clone();
    sorted.sort_unstable();
    assert_eq!(candidates, sorted, "candidates must be sorted ascending");
}

#[test]
fn happy_n12_zero_entropy_trezor_canonical() {
    assert_completer_round_trip(&[0u8; 16], 128);
}

#[test]
fn happy_n15_constructed() {
    assert_completer_round_trip(&[0u8; 20], 64);
}

#[test]
fn happy_n18_zero_entropy_trezor_canonical() {
    assert_completer_round_trip(&[0u8; 24], 32);
}

#[test]
fn happy_n21_constructed() {
    assert_completer_round_trip(&[0u8; 28], 16);
}

#[test]
fn happy_n24_zero_entropy_trezor_canonical() {
    assert_completer_round_trip(&[0u8; 32], 8);
}

// ============================================================================
// Refusals
// ============================================================================

#[test]
fn refusal_empty_partial() {
    let r = final_word_candidates("", FinalWordLanguage::English);
    assert!(r.is_err(), "empty partial must refuse");
    let msg = r.unwrap_err().to_string();
    assert!(
        msg.contains("11") && msg.contains("14") && msg.contains("17") && msg.contains("20") && msg.contains("23"),
        "refusal message must enumerate accepted partial-word counts; got: {msg}",
    );
}

#[test]
fn refusal_wrong_word_count_too_few() {
    let r = final_word_candidates("abandon abandon", FinalWordLanguage::English);
    assert!(r.is_err(), "2-word partial must refuse");
}

#[test]
fn refusal_wrong_word_count_too_many() {
    // 25 words — exceeds the largest valid partial (23 → N=24).
    let twenty_five = "abandon ".repeat(25);
    let r = final_word_candidates(twenty_five.trim(), FinalWordLanguage::English);
    assert!(r.is_err(), "25-word partial must refuse");
}

#[test]
fn refusal_unknown_word_in_partial() {
    // 11 words but one is not in the English wordlist.
    let r = final_word_candidates(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon xyzzy",
        FinalWordLanguage::English,
    );
    assert!(r.is_err(), "unknown-word partial must refuse");
    let msg = r.unwrap_err().to_string();
    assert!(
        msg.to_lowercase().contains("unknown") || msg.to_lowercase().contains("not in"),
        "refusal must mention the unknown-word condition; got: {msg}",
    );
}

// ============================================================================
// Determinism
// ============================================================================

#[test]
fn determinism_same_input_same_output_twice() {
    let a = final_word_candidates(ABANDON_11_PARTIAL, FinalWordLanguage::English).unwrap();
    let b = final_word_candidates(ABANDON_11_PARTIAL, FinalWordLanguage::English).unwrap();
    assert_eq!(a, b, "same input must yield identical output");
}

// ============================================================================
// Cross-language
// ============================================================================

#[test]
fn cross_language_spanish_partial_yields_spanish_candidates() {
    // Construct a valid 12-word Spanish phrase from zero entropy.
    let m = Mnemonic::from_entropy_in(Language::Spanish, &[0u8; 16]).unwrap();
    let words: Vec<&str> = m.words().collect();
    let partial = words[..11].join(" ");
    let last = words[11];
    let candidates = final_word_candidates(&partial, FinalWordLanguage::Spanish).unwrap();
    assert_eq!(candidates.len(), 128);
    assert!(
        candidates.contains(&last),
        "round-trip: spanish original last word must appear",
    );
    // Verify the candidate words are in the Spanish wordlist (sanity).
    let spanish_set: std::collections::BTreeSet<&'static str> =
        Language::Spanish.word_list().iter().copied().collect();
    for w in &candidates {
        assert!(spanish_set.contains(w), "candidate '{w}' must be in Spanish wordlist");
    }
}
