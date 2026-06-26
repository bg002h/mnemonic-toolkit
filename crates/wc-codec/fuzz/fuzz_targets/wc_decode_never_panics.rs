//! Fuzz target: `wc-codec::decode` never panics on arbitrary / corrupt input
//! (P6).
//!
//! Oracle (the never-panic + never-wrong-payload charter): `decode` over ANY
//! word list — random words, a valid card with arbitrary bytes flipped, a
//! truncated stream — MUST return `Ok` or `Err`, NEVER panic/abort. This is the
//! funds-safety net: a hostile or chipped card must refuse cleanly, never crash
//! and never silently return a wrong payload. Any panic is a real finding.
#![no_main]

use libfuzzer_sys::fuzz_target;
use wc_codec::decode;

/// The BIP-39 English wordlist, indexed so we can map fuzz bytes → valid words
/// (a random ASCII blob almost never hits the wordlist, so we synthesize a word
/// stream the decoder will actually try to parse).
fn wordlist() -> &'static [&'static str] {
    bip39::Language::English.word_list()
}

fuzz_target!(|data: &[u8]| {
    let wl = wordlist();
    // Map each input byte-pair to an 11-bit index → a valid BIP-39 word, so the
    // word→symbol layer succeeds and the decoder exercises the geometry / RS /
    // sync / integrity layers (where the interesting refuse paths live). Cap the
    // length at the field's codeword bound.
    let mut words: Vec<&str> = Vec::new();
    let mut i = 0;
    while i + 1 < data.len() && words.len() < 2047 {
        let idx = (((data[i] as usize) << 8) | data[i + 1] as usize) % 2048;
        words.push(wl[idx]);
        i += 2;
    }
    if words.is_empty() {
        return;
    }
    // MUST NOT panic; the result is irrelevant (Ok or Err are both fine).
    let _ = decode(&words);

    // Also feed the raw bytes lossily as whitespace-separated tokens — exercises
    // the UnknownWord refuse path and the case-folding intake.
    let s = String::from_utf8_lossy(data);
    let tokens: Vec<&str> = s.split_whitespace().take(2047).collect();
    if !tokens.is_empty() {
        let _ = decode(&tokens);
    }
});
