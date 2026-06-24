//! All-length BCH regression + hardening (v0.2.1 fix lock).
//!
//! The pre-v0.2.1 code paired a wrong `POLYMOD_INIT` (0x23181b3) with an
//! empirically-lifted `MS_REGULAR_CONST`, so `decode_with_correction` rejected
//! CLEAN 20/24/28/32-byte ms1 strings (`TooManyErrors`) — only 16-byte seeds
//! worked. The 12-word-only test monoculture (`bch_decode.rs`) hid it even though
//! the 0.1.1 corpus already carried 15/18/21-word vectors. These cells sweep ALL
//! five entropy lengths. See `design/BUG_decode_with_correction_length_divergence.md`.

use ms_codec::bch::{bch_create_checksum_regular, hrp_expand, polymod_run, MS_REGULAR_CONST};
use ms_codec::{decode_with_correction, encode, Error, Payload, Tag};

const ABC: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
const LENGTHS: [usize; 5] = [16, 20, 24, 28, 32];

fn sym(c: char) -> u8 {
    ABC.iter().position(|&b| b == c as u8).unwrap() as u8
}
/// data-part symbols (post-`ms1`), including the 13-symbol BCH checksum tail.
fn symbols(s: &str) -> Vec<u8> {
    s.chars().skip(3).map(sym).collect()
}
fn corrupt_at(s: &str, pos: usize, mask: u8) -> String {
    let mut c: Vec<char> = s.chars().collect();
    let i = 3 + pos;
    let v = sym(c[i].to_ascii_lowercase());
    c[i] = ABC[((v ^ (mask & 0x1F)) & 0x1F) as usize] as char;
    c.into_iter().collect()
}
fn clean(len: usize) -> (Vec<u8>, String) {
    let e: Vec<u8> = (0..len as u8).collect();
    let s = encode(Tag::ENTR, &Payload::Entr(e.clone())).unwrap();
    (e, s)
}

// ── Gate 5a: constant-derivation + all-length single-target residue ──────────
// THE gate that would have caught the original bug: a correct BCH verify yields
// ONE fixed target for every valid codeword regardless of length.
#[test]
fn ms_regular_const_is_secretshare32_packed() {
    // codex32 short target "SECRETSHARE32" Fe values, big-endian packed.
    let fe: [u128; 13] = [16, 25, 24, 3, 25, 11, 16, 23, 29, 3, 25, 17, 10];
    let packed = fe
        .iter()
        .enumerate()
        .fold(0u128, |a, (i, &v)| a | (v << (5 * (12 - i))));
    assert_eq!(
        MS_REGULAR_CONST, packed,
        "MS_REGULAR_CONST must be SECRETSHARE32 big-endian packed"
    );
    assert_eq!(MS_REGULAR_CONST, 0x10ce0795c2fd1e62a);
    assert_eq!(
        (MS_REGULAR_CONST >> 64) & 1,
        1,
        "the codex32 target is a 65-bit value (bit 64 set)"
    );
}

#[test]
fn polymod_lands_on_single_target_for_every_length() {
    for len in LENGTHS {
        let (_, s) = clean(len);
        let mut input = hrp_expand("ms");
        input.extend_from_slice(&symbols(&s));
        assert_eq!(
            polymod_run(&input),
            MS_REGULAR_CONST,
            "len={len}: a valid codeword's polymod residue must equal the single fixed target"
        );
    }
}

#[test]
fn handrolled_checksum_matches_codex32_encoded_tail_every_length() {
    for len in LENGTHS {
        let (_, s) = clean(len);
        let syms = symbols(&s);
        let dp = syms.len();
        let recomputed = bch_create_checksum_regular("ms", &syms[..dp - 13]);
        assert_eq!(
            &recomputed[..],
            &syms[dp - 13..],
            "len={len}: hand-rolled checksum must equal the codex32-encoded checksum tail"
        );
    }
}

// ── Gate 5b: clean passthrough → 0 corrections, every length ─────────────────
#[test]
fn clean_decode_with_correction_zero_corrections_every_length() {
    for len in LENGTHS {
        let (e, s) = clean(len);
        let (t, p, corr) = decode_with_correction(&s)
            .unwrap_or_else(|err| panic!("len={len} clean must decode: {err:?}"));
        assert_eq!(t, Tag::ENTR);
        assert_eq!(p, Payload::Entr(e));
        assert!(
            corr.is_empty(),
            "len={len}: a clean string must report 0 corrections"
        );
    }
}

// ── Theme 1 / Gate 5c: 1..=4-error correction + accurate positions, all lengths ─
#[test]
fn corrects_1_to_4_errors_every_length() {
    for len in LENGTHS {
        let (e, s) = clean(len);
        let dp = symbols(&s).len();
        for k in 1..=4usize {
            let positions: Vec<usize> = (0..k).map(|j| 1 + j * (dp / (k + 1)).max(1)).collect();
            let mut bad = s.clone();
            for &p in &positions {
                bad = corrupt_at(&bad, p, 0x1F);
            }
            let (t, p2, corr) = decode_with_correction(&bad)
                .unwrap_or_else(|err| panic!("len={len} k={k} must correct: {err:?}"));
            assert_eq!(t, Tag::ENTR);
            assert_eq!(p2, Payload::Entr(e.clone()), "len={len} k={k} payload");
            let got: std::collections::BTreeSet<usize> = corr.iter().map(|c| c.position).collect();
            let want: std::collections::BTreeSet<usize> = positions.iter().copied().collect();
            assert_eq!(
                got, want,
                "len={len} k={k}: reported positions must equal injected"
            );
        }
    }
}

// ── Theme 2: 5-8-error miscorrection sweep → != Ok(original), every length ────
// The BCH(93,80,8) code is non-perfect, so 5-8 errors can legitimately miscorrect
// to a DIFFERENT valid codeword (Ok(different)); what must NEVER happen is a false
// claim of recovering the ORIGINAL from beyond-t errors. `is_err()` would be flaky.
#[test]
fn five_to_eight_errors_never_return_original_every_length() {
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    for len in LENGTHS {
        let (e, s) = clean(len);
        let original = Payload::Entr(e);
        let dp = symbols(&s).len();
        for _ in 0..80u32 {
            for n_err in 5..=8usize {
                let mut pos = std::collections::BTreeSet::new();
                while pos.len() < n_err {
                    x ^= x << 13;
                    x ^= x >> 7;
                    x ^= x << 17;
                    pos.insert((x as usize) % dp);
                }
                let mut bad = s.clone();
                let mask = ((x as u8) | 1) & 0x1F;
                for &p in &pos {
                    bad = corrupt_at(&bad, p, mask);
                }
                if let Ok((_, p2, _)) = decode_with_correction(&bad) {
                    assert_ne!(
                        p2, original,
                        "len={len} n_err={n_err}: 5-8 errors silently returned the original"
                    );
                }
            }
        }
    }
}

// ── Theme 3: indel reject-contract (the toolkit Ms1IndelOracle relies on this) ─
// T3-ms-1: a raw wrong-length (indel) string fails closed. NOT a specific
// variant — `decode_with_correction` computes the BCH residue over the wrong-
// length symbol vector BEFORE the rule-9 length gate, so the error is almost
// always `TooManyErrors`, occasionally `UnexpectedStringLength`. The oracle never
// feeds a wrong-length string (`indel.rs` length-restores candidates first); this
// pins the CODEC contract the oracle's soundness rests on. The "never self-correct
// to the original" half is covered by the Theme-2 sweep above.
#[test]
fn raw_wrong_length_fails_closed_every_length() {
    for len in LENGTHS {
        let (_, s) = clean(len);
        let mut ins: Vec<char> = s.chars().collect();
        ins.insert(3 + 5, 'p');
        let inserted: String = ins.into_iter().collect();
        assert!(
            decode_with_correction(&inserted).is_err(),
            "len={len}: an inserted symbol must fail closed"
        );

        let mut del: Vec<char> = s.chars().collect();
        del.remove(3 + 5);
        let deleted: String = del.into_iter().collect();
        let r = decode_with_correction(&deleted);
        assert!(
            matches!(
                &r,
                Err(Error::TooManyErrors { .. }) | Err(Error::UnexpectedStringLength { .. })
            ),
            "len={len}: a deleted symbol must fail closed (got {r:?})"
        );
    }
}
