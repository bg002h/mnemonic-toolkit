//! Bug hunt 2026-11-25 toolkit#3 — the STATED `--parity-words` budget.
//!
//! The CLI and manual used to say `m` parity words "correct `⌊m/2⌋`
//! substitutions". The decoder works on whole checkpoint blocks: a block whose
//! CRC-5 fails is erased whole (the 5-bit CRC cannot say WHICH word is wrong),
//! a damaged checkpoint word erases its block plus itself, and the header words
//! are CRC-checked before RS and are not repairable at all. These tests pin the
//! budget as it is now documented, and that the old `⌊m/2⌋` claim is false.
//! The decoder and the wire format are unchanged; every refusal below is a
//! refusal, never a wrong payload.

use wc_codec::sync::checkpoint_layout;
use wc_codec::wordmap::{symbol_to_word, word_to_symbol};
use wc_codec::{decode, encode, Decoded, EncodeOpts, SourceKind, WcError};

const HEADER_WORDS: usize = 5; // H0 + GEOM(4), solo card
const T: u8 = 44;
const U: u8 = 3;

fn payload() -> Vec<u8> {
    // Deterministic 73-byte (mk1-sized) payload.
    let mut s: u64 = 0x2026_1125;
    (0..73)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s & 0xFF) as u8
        })
        .collect()
}

fn card(m: usize) -> Vec<String> {
    let p = payload();
    let opts = EncodeOpts {
        parity_words: m,
        integrity_bits: T,
        u_slots: U,
    };
    encode(SourceKind::Mk1Xpub, &p, p.len() * 8, &opts)
        .expect("encode")
        .into_iter()
        .map(|w| w.to_string())
        .collect()
}

fn k() -> usize {
    (payload().len() * 8 + T as usize).div_ceil(11)
}

/// Engraved index of the first interleave word.
fn interleave_start() -> usize {
    HEADER_WORDS + 2 * U as usize
}

fn substitute(words: &mut [String], idx: usize) {
    let s = word_to_symbol(&words[idx]).unwrap();
    words[idx] = symbol_to_word((s + 1029) % 2048).unwrap().to_string();
}

fn dec(words: &[String]) -> Result<Decoded, WcError> {
    let refs: Vec<&str> = words.iter().map(|s| s.as_str()).collect();
    decode(&refs)
}

#[test]
fn geometry_is_what_the_test_assumes() {
    let layout = checkpoint_layout(k());
    let m = 8;
    let c = card(m);
    assert_eq!(
        c.len(),
        interleave_start() + k() + layout.checkpoint_count + m + 2,
        "header + ledger + interleave + parity + stop"
    );
    assert_eq!(layout.stride, 8, "K={} gives blocks of about sqrt(K)", k());
}

/// A header word is not repairable, whatever `m` is.
#[test]
fn header_substitution_refuses_at_any_budget() {
    for m in [8usize, 32] {
        for idx in 0..HEADER_WORDS {
            let mut c = card(m);
            substitute(&mut c, idx);
            assert!(dec(&c).is_err(), "m={m} header word {idx} must refuse");
        }
    }
}

/// One mis-stamped DATA word costs its whole block: with `m` = block size it
/// repairs, filling exactly the block.
#[test]
fn data_word_substitution_costs_its_whole_block() {
    let layout = checkpoint_layout(k());
    let base = interleave_start();
    let mut off = 0usize;
    for &sz in &layout.block_sizes {
        let m = sz;
        let mut c = card(m);
        substitute(&mut c, base + off);
        let d = dec(&c).expect("one data word, m = block size: repairs");
        assert_eq!(d.payload, payload());
        assert!(
            d.repair.erasures_filled == sz || d.repair.erasures_filled == 0,
            "block erased whole (or, on a CRC-5 collision, corrected as one error)"
        );
        off += sz + 1;
    }
}

/// A mis-stamped CHECKPOINT word costs its block plus itself: `m` = block
/// size is not enough, `m` = block size + 1 is.
#[test]
fn checkpoint_substitution_costs_block_plus_one() {
    let layout = checkpoint_layout(k());
    let sz = layout.block_sizes[0];
    let cp0 = interleave_start() + sz;

    let mut c = card(sz);
    substitute(&mut c, cp0);
    assert!(
        dec(&c).is_err(),
        "m = block size cannot repair a checkpoint"
    );

    let mut c = card(sz + 1);
    substitute(&mut c, cp0);
    assert_eq!(dec(&c).expect("m = block size + 1").payload, payload());
}

/// The documented rule of thumb: with `m` ≥ block size + 1, ONE mis-stamped
/// word anywhere outside the header repairs.
#[test]
fn block_size_plus_one_survives_any_single_non_header_word() {
    let m = checkpoint_layout(k()).stride + 1;
    let n = card(m).len();
    for idx in HEADER_WORDS..n {
        let mut c = card(m);
        substitute(&mut c, idx);
        let d = dec(&c).unwrap_or_else(|e| panic!("m={m} word {idx}: {e:?}"));
        assert_eq!(d.payload, payload(), "word {idx}");
    }
}

/// The OLD claim was `⌊m/2⌋` substitutions: m = 8 would correct 4. Two
/// mis-stamped data words in different blocks already exceed it.
#[test]
fn old_floor_m_over_2_claim_is_false() {
    let layout = checkpoint_layout(k());
    let base = interleave_start();
    let mut c = card(8);
    substitute(&mut c, base); // block 0
    substitute(&mut c, base + layout.block_sizes[0] + 1); // block 1
    assert!(
        dec(&c).is_err(),
        "two data words in two blocks exceed m = 8 (never a wrong payload)"
    );
}

/// Parity-region substitutions are ordinary RS errors: `⌊m/2⌋` of them repair.
#[test]
fn parity_word_substitutions_cost_two_each() {
    let m = 8usize;
    let layout = checkpoint_layout(k());
    let parity_start = interleave_start() + k() + layout.checkpoint_count;
    let mut c = card(m);
    for i in 0..m / 2 {
        substitute(&mut c, parity_start + i);
    }
    assert_eq!(dec(&c).expect("m/2 parity errors").payload, payload());
}
