//! Exhaustive small-error correctness sweep + a beyond-t acceptance-rate
//! tripwire for the md1 regular `BCH(93, 80, 8)` decoder
//! ([`decode_regular_errors`]).
//!
//! # Oracle — the injected pattern is ground truth by construction
//!
//! Every correctness cell builds a valid codeword with the *encoder*
//! ([`bch_create_checksum_regular`]), XORs a KNOWN ≤2-error pattern into it,
//! runs the *decoder*, and asserts the recovered codeword is byte-identical to
//! the pre-corruption original. The code has minimum distance ≥ 9, so a word
//! within Hamming distance ≤ 2 of a codeword is within ≤ 2 of a UNIQUE
//! codeword (the original) — the decoder therefore has exactly one correct
//! answer, and we never trust the decoder's own internals to check it.
//!
//! # Coverage
//!
//!  * `exhaustive_single_error_sweep` — all 93 positions × 31 non-zero
//!    magnitudes = 2 883 decodes (< 1 s debug).
//!  * `bounded_two_error_sweep` — all 4 278 position-pairs × a bounded seeded
//!    magnitude subset (8 per pair ≈ 34 k decodes ≈ 9 s debug).
//!  * `full_two_error_sweep` — the FULL all-pairs × all 31×31 magnitude
//!    matrix (≈ 4.1 M decodes ≈ 18 min debug); `#[ignore]`-gated (release-only).
//!  * `beyond_t_acceptance_rate_tripwire` — ≥ 1e5 seeded e≥5 injections;
//!    counts *any* `Some` return (miscorrection acceptance) and asserts it
//!    stays under a re-measured, pinned bound (≈ 24 s debug).
//!
//! # RED-under-mutation (per-cell, verified during authoring)
//!
//!  * Correctness sweeps — RED when the decoder's position map
//!    `k = data_with_checksum_len - 1 - d` (`bch_decode.rs`) is mutated to
//!    `k = d`: corrections land at mirrored positions, so `recovered !=
//!    original`.
//!  * Acceptance tripwire — RED when the **L≤4 root-count consistency cap**
//!    is relaxed (the `error_degrees.len() != deg` guard inside `chien_search`
//!    together with its defensive re-check in `decode_regular_errors`): the
//!    any-`Some`-at-e≥5 count spikes from 10 to ~91 575 / 1e5. (The literal
//!    `deg > 4` degree cap does NOT spike this t=4 code — chien-search's
//!    root-count consistency is the operative capacity gate — so the L≤4 cap
//!    is the faithful "capacity relax" mutation here.)

use md_codec::bch::{MD_REGULAR_CONST, bch_create_checksum_regular, hrp_expand, polymod_run};
use md_codec::bch_decode::decode_regular_errors;

/// Number of data symbols in a maximal single-string regular codeword
/// (`BCH(93, 80, 8)`): 80 data + 13 checksum = 93 total symbols.
const DATA_SYMBOLS: usize = 80;
const CODEWORD_LEN: usize = 93;

/// A fixed, arbitrary 80-symbol data pattern. The exact pattern is immaterial
/// to correctness (BCH correction is data-agnostic) but pinning it keeps the
/// sweep and the acceptance-rate measurement fully deterministic.
fn base_data() -> Vec<u8> {
    (0..DATA_SYMBOLS as u8)
        .map(|i| i.wrapping_mul(7).wrapping_add(3) & 0x1F)
        .collect()
}

/// Build the 93-symbol valid regular codeword for `data` (via the encoder).
fn valid_codeword(data: &[u8]) -> Vec<u8> {
    let checksum = bch_create_checksum_regular("md", data);
    let mut cw = data.to_vec();
    cw.extend_from_slice(&checksum);
    cw
}

/// The BCH residue a decoder caller passes to [`decode_regular_errors`]:
/// `polymod(hrp_expand("md") || data_with_checksum) ⊕ MD_REGULAR_CONST`
/// (mirrors `chunk::decode_with_correction`).
fn residue_of(dwc: &[u8]) -> u128 {
    let mut input = hrp_expand("md");
    input.extend_from_slice(dwc);
    polymod_run(&input) ^ MD_REGULAR_CONST
}

/// Correct `corrupted` with the md decoder and return the repaired symbol
/// vector, or `None` if the decoder declined to correct.
fn bch_correct(corrupted: &[u8]) -> Option<Vec<u8>> {
    let (positions, magnitudes) = decode_regular_errors(residue_of(corrupted), corrupted.len())?;
    let mut fixed = corrupted.to_vec();
    for (&p, &m) in positions.iter().zip(&magnitudes) {
        fixed[p] ^= m;
    }
    Some(fixed)
}

/// Deterministic xorshift64 PRNG (no `rand` dev-dep; mirrors the seeded style
/// already used in `tests/bch_adversarial.rs`).
struct Rng(u64);
impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    /// A value in `0..n`.
    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
    /// A non-zero 5-bit magnitude in `1..=31`.
    fn magnitude(&mut self) -> u8 {
        ((self.next_u64() as u8) & 0x1F).max(1)
    }
}

// ---------------------------------------------------------------------------
// Cell 1 — exhaustive single-error sweep (all 93 positions × 31 magnitudes)
// ---------------------------------------------------------------------------

#[test]
fn exhaustive_single_error_sweep() {
    let original = valid_codeword(&base_data());
    assert_eq!(original.len(), CODEWORD_LEN);
    // A clean codeword must decode to itself (empty correction set).
    assert_eq!(
        bch_correct(&original).as_deref(),
        Some(original.as_slice()),
        "a valid codeword must pass through unchanged"
    );

    let mut checked = 0usize;
    for pos in 0..CODEWORD_LEN {
        for mag in 1u8..32 {
            let mut w = original.clone();
            w[pos] ^= mag;
            let recovered = bch_correct(&w)
                .unwrap_or_else(|| panic!("1-error at pos {pos} mag {mag:05b} must decode"));
            assert_eq!(
                recovered, original,
                "1-error at pos {pos} mag {mag:05b}: recovered != injected-original"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, CODEWORD_LEN * 31, "must cover 93 × 31 patterns");
}

// ---------------------------------------------------------------------------
// Cell 2 — bounded two-error sweep (all 4 278 pairs × 8 seeded magnitudes)
// ---------------------------------------------------------------------------

const TWO_ERROR_MAG_SAMPLES: usize = 8;

#[test]
fn bounded_two_error_sweep() {
    let original = valid_codeword(&base_data());
    // Seed is fixed so the magnitude subset is identical run-to-run.
    let mut rng = Rng(0xB0BA_FE77_1234_5678);
    let mut pairs = 0usize;
    for p1 in 0..CODEWORD_LEN {
        for p2 in (p1 + 1)..CODEWORD_LEN {
            for _ in 0..TWO_ERROR_MAG_SAMPLES {
                let m1 = rng.magnitude();
                let m2 = rng.magnitude();
                let mut w = original.clone();
                w[p1] ^= m1;
                w[p2] ^= m2;
                let recovered = bch_correct(&w).unwrap_or_else(|| {
                    panic!("2-error ({p1},{m1:05b})+({p2},{m2:05b}) must decode")
                });
                assert_eq!(
                    recovered, original,
                    "2-error ({p1},{m1:05b})+({p2},{m2:05b}): recovered != injected-original"
                );
            }
            pairs += 1;
        }
    }
    assert_eq!(
        pairs,
        CODEWORD_LEN * (CODEWORD_LEN - 1) / 2,
        "must cover all 4278 pairs"
    );
}

// ---------------------------------------------------------------------------
// Cell 3 — FULL all-pairs two-error sweep (release-only, #[ignore])
// ---------------------------------------------------------------------------

#[test]
#[ignore = "exhaustive 4.1M-decode sweep (~18 min debug); run in release CI"]
fn full_two_error_sweep() {
    let original = valid_codeword(&base_data());
    for p1 in 0..CODEWORD_LEN {
        for p2 in (p1 + 1)..CODEWORD_LEN {
            for m1 in 1u8..32 {
                for m2 in 1u8..32 {
                    let mut w = original.clone();
                    w[p1] ^= m1;
                    w[p2] ^= m2;
                    let recovered = bch_correct(&w).unwrap_or_else(|| {
                        panic!("2-error ({p1},{m1:05b})+({p2},{m2:05b}) must decode")
                    });
                    assert_eq!(
                        recovered, original,
                        "2-error ({p1},{m1:05b})+({p2},{m2:05b}): recovered != injected-original"
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Cell 4 — beyond-t acceptance-rate tripwire
// ---------------------------------------------------------------------------

/// Seed pinned for the acceptance-rate measurement below.
const ACCEPTANCE_SEED: u64 = 0x1234_5678_9ABC_DEF0;
/// Number of seeded e≥5 injection trials (spec: ≥ 1e5).
const ACCEPTANCE_TRIALS: u32 = 100_000;
/// Upper bound on `Some`-returns (miscorrection acceptances) at e≥5, RE-MEASURED
/// at this cell's own (L=93, seed, e∈5..=8 mix): the measured baseline is 10;
/// this bound (5× headroom) trips loudly when the L≤4 capacity guard is
/// relaxed (measured spike: 91 575 / 1e5).
const ACCEPTANCE_BOUND: u32 = 50;

#[test]
fn beyond_t_acceptance_rate_tripwire() {
    let original = valid_codeword(&base_data());
    let l = original.len();
    let mut rng = Rng(ACCEPTANCE_SEED);
    let mut any_some = 0u32;

    for _ in 0..ACCEPTANCE_TRIALS {
        let e = 5 + rng.below(4); // e ∈ 5..=8 (beyond t = 4)
        let mut positions = std::collections::BTreeSet::new();
        while positions.len() < e {
            positions.insert(rng.below(l));
        }
        let mut w = original.clone();
        for &p in &positions {
            w[p] ^= rng.magnitude();
        }
        // Count ANY Some return: the original is never re-emitted (e ≥ 5 is a
        // real distance ≥ 5 from the codeword), so every Some is a
        // miscorrection acceptance. A capacity-guard relax makes this spike.
        if decode_regular_errors(residue_of(&w), l).is_some() {
            any_some += 1;
        }
    }

    assert!(
        any_some <= ACCEPTANCE_BOUND,
        "beyond-t miscorrection acceptance spiked: {any_some} Some-returns over \
         {ACCEPTANCE_TRIALS} trials exceeds the pinned bound {ACCEPTANCE_BOUND} \
         (baseline 10) — the L≤4 capacity guard may have been weakened"
    );
}
