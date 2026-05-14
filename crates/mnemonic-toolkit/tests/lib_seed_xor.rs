//! v0.12.0 P1 — library tests for `mnemonic_toolkit::seed_xor`.
//!
//! Per SPEC §4 G1 + G2:
//! - G1 (Coldcard interop): byte-pinned vectors for `--deterministic-from-master`
//!   over 12/18/24-word entropy.
//! - G2 (algorithmic round-trip): property-style split → combine round-trip
//!   across all 5 sizes × N ∈ {2,3,4,5} shares, ≥ 100 vectors per size.
//!
//! Library-local error type tests + length-validation refusals also live here.

use mnemonic_toolkit::seed_xor::{
    seed_xor_combine, seed_xor_split, seed_xor_split_deterministic, SeedXorError,
    MIN_SHARES, VALID_ENTROPY_LENGTHS,
};
use rand_core::{RngCore, SeedableRng};

// A deterministic seedable RNG so property tests + anchors are reproducible.
// (For CLI default we use `OsRng`; for tests we use ChaCha20 with a pinned seed.)
struct DeterministicRng {
    state: rand_chacha::ChaCha20Rng,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        DeterministicRng {
            state: rand_chacha::ChaCha20Rng::seed_from_u64(seed),
        }
    }
}

impl rand_core::RngCore for DeterministicRng {
    fn next_u32(&mut self) -> u32 {
        self.state.next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.state.next_u64()
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.state.fill_bytes(dest)
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.state.try_fill_bytes(dest)
    }
}

impl rand_core::CryptoRng for DeterministicRng {}

// ============================================================================
// Length-validation refusals
// ============================================================================

#[test]
fn split_refuses_invalid_entropy_length() {
    let mut rng = DeterministicRng::new(0);
    let bad = vec![0u8; 17]; // not in {16, 20, 24, 28, 32}
    let err = seed_xor_split(&bad, 2, &mut rng).unwrap_err();
    match err {
        SeedXorError::BadEntropyLength { got, expected_one_of } => {
            assert_eq!(got, 17);
            assert_eq!(expected_one_of, &[16, 20, 24, 28, 32]);
        }
        other => panic!("expected BadEntropyLength, got {:?}", other),
    }
}

#[test]
fn split_refuses_share_count_below_min() {
    let mut rng = DeterministicRng::new(0);
    let entropy = vec![0u8; 16];
    let err = seed_xor_split(&entropy, 1, &mut rng).unwrap_err();
    match err {
        SeedXorError::TooFewShares { got, min } => {
            assert_eq!(got, 1);
            assert_eq!(min, MIN_SHARES);
            assert_eq!(min, 2);
        }
        other => panic!("expected TooFewShares, got {:?}", other),
    }
}

#[test]
fn combine_refuses_mismatched_share_lengths() {
    let s1 = vec![0u8; 16];
    let s2 = vec![0u8; 32];
    let err = seed_xor_combine(&[&s1, &s2]).unwrap_err();
    match err {
        SeedXorError::MismatchedShareLengths { lengths } => {
            assert_eq!(lengths, vec![16, 32]);
        }
        other => panic!("expected MismatchedShareLengths, got {:?}", other),
    }
}

#[test]
fn combine_refuses_invalid_uniform_length() {
    let s1 = vec![0u8; 17];
    let s2 = vec![0u8; 17];
    let err = seed_xor_combine(&[&s1, &s2]).unwrap_err();
    match err {
        SeedXorError::BadEntropyLength { got, .. } => assert_eq!(got, 17),
        other => panic!("expected BadEntropyLength, got {:?}", other),
    }
}

#[test]
fn combine_refuses_single_share() {
    let s = vec![0u8; 16];
    let err = seed_xor_combine(&[&s]).unwrap_err();
    match err {
        SeedXorError::TooFewShares { got, min } => {
            assert_eq!(got, 1);
            assert_eq!(min, 2);
        }
        other => panic!("expected TooFewShares, got {:?}", other),
    }
}

// ============================================================================
// G2 — algorithmic round-trip across all 5 sizes
// ============================================================================

fn round_trip_check(entropy_len: usize, n_shares: usize, seed: u64) {
    let mut rng = DeterministicRng::new(seed);
    let mut entropy = vec![0u8; entropy_len];
    rng.fill_bytes(&mut entropy);

    let shares = seed_xor_split(&entropy, n_shares, &mut rng).unwrap();
    assert_eq!(shares.len(), n_shares);
    for s in &shares {
        assert_eq!(s.len(), entropy_len);
    }

    let refs: Vec<&[u8]> = shares.iter().map(|s| s.as_slice()).collect();
    let recovered = seed_xor_combine(&refs).unwrap();
    assert_eq!(
        recovered.as_slice(),
        entropy.as_slice(),
        "round-trip mismatch at entropy_len={entropy_len}, n_shares={n_shares}, seed={seed}",
    );
}

#[test]
fn round_trip_12_word_size_2_to_5_shares() {
    for n_shares in 2..=5 {
        for seed in 0..100 {
            round_trip_check(16, n_shares, seed);
        }
    }
}

#[test]
fn round_trip_15_word_size_2_to_5_shares() {
    for n_shares in 2..=5 {
        for seed in 0..100 {
            round_trip_check(20, n_shares, seed);
        }
    }
}

#[test]
fn round_trip_18_word_size_2_to_5_shares() {
    for n_shares in 2..=5 {
        for seed in 0..100 {
            round_trip_check(24, n_shares, seed);
        }
    }
}

#[test]
fn round_trip_21_word_size_2_to_5_shares() {
    for n_shares in 2..=5 {
        for seed in 0..100 {
            round_trip_check(28, n_shares, seed);
        }
    }
}

#[test]
fn round_trip_24_word_size_2_to_5_shares() {
    for n_shares in 2..=5 {
        for seed in 0..100 {
            round_trip_check(32, n_shares, seed);
        }
    }
}

// ============================================================================
// G1 — Coldcard deterministic anchor (12/18/24-word interop)
// ============================================================================

/// `seed_xor_split_deterministic` is order-independent of the input bytes
/// for shares 0..N-2 (they're SHA256d outputs); the last share is the
/// XOR-residual. We pin the byte-exact share[0] for a canonical
/// `abandon × 12` input as a regression anchor.
#[test]
fn deterministic_split_abandon_12_share_0_byte_pin() {
    // abandon × 12 = entropy [0x00; 16]
    let entropy = [0u8; 16];
    let shares = seed_xor_split_deterministic(&entropy, 2).unwrap();
    // share[0] = sha256d(b"Batshitoshi " || [0;16] || b"0 of 2 parts")[:16]
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(b"Batshitoshi ");
    buf.extend_from_slice(&entropy);
    buf.extend_from_slice(b"0 of 2 parts");
    use bitcoin::hashes::{sha256d, Hash};
    let h = sha256d::Hash::hash(&buf);
    let expected_share_0 = &h.as_byte_array()[..16];
    assert_eq!(
        shares[0].as_slice(),
        expected_share_0,
        "share[0] byte-pin (Coldcard SHA256d-deterministic anchor); if Coldcard changes formula, update both",
    );
    // round-trip:
    let refs: Vec<&[u8]> = shares.iter().map(|s| s.as_slice()).collect();
    let recovered = seed_xor_combine(&refs).unwrap();
    assert_eq!(recovered.as_slice(), &entropy);
}

#[test]
fn deterministic_split_round_trip_12_18_24_word_sizes() {
    // Coldcard-native sizes: 16/24/32 bytes
    for &len in &[16usize, 24, 32] {
        let entropy = vec![0xa5u8; len];
        let shares = seed_xor_split_deterministic(&entropy, 3).unwrap();
        assert_eq!(shares.len(), 3);
        let refs: Vec<&[u8]> = shares.iter().map(|s| s.as_slice()).collect();
        let recovered = seed_xor_combine(&refs).unwrap();
        assert_eq!(recovered.as_slice(), entropy.as_slice());
    }
}

#[test]
fn deterministic_split_is_reproducible() {
    // Same input → same output (deterministic).
    let entropy = [0u8; 32];
    let a = seed_xor_split_deterministic(&entropy, 4).unwrap();
    let b = seed_xor_split_deterministic(&entropy, 4).unwrap();
    for (sa, sb) in a.iter().zip(b.iter()) {
        assert_eq!(sa.as_slice(), sb.as_slice(), "deterministic split must be reproducible");
    }
}

// ============================================================================
// Zeroize discipline tests
// ============================================================================

#[test]
fn returned_shares_are_zeroizing() {
    // Compile-time check: the return type implements ZeroizeOnDrop semantics
    // via the wrapping `Zeroizing<Vec<u8>>`. We can't directly observe the
    // drop-zero in safe Rust, but we CAN verify the type binds Zeroizing.
    let mut rng = DeterministicRng::new(42);
    let entropy = vec![0u8; 16];
    let shares: Vec<zeroize::Zeroizing<Vec<u8>>> =
        seed_xor_split(&entropy, 2, &mut rng).unwrap();
    // type-binding check — if we changed the return shape this would no longer compile
    let _ = shares;
}

// ============================================================================
// Determinism vs randomness — split() with same seed yields same shares
// ============================================================================

#[test]
fn random_split_uses_supplied_rng() {
    // Same RNG seed → identical share sets.
    let entropy = vec![0xffu8; 32];
    let mut rng_a = DeterministicRng::new(7);
    let mut rng_b = DeterministicRng::new(7);
    let a = seed_xor_split(&entropy, 3, &mut rng_a).unwrap();
    let b = seed_xor_split(&entropy, 3, &mut rng_b).unwrap();
    for (sa, sb) in a.iter().zip(b.iter()) {
        assert_eq!(sa.as_slice(), sb.as_slice());
    }
}

#[test]
fn random_split_differs_across_seeds() {
    let entropy = vec![0xffu8; 32];
    let mut rng_a = DeterministicRng::new(7);
    let mut rng_b = DeterministicRng::new(11);
    let a = seed_xor_split(&entropy, 3, &mut rng_a).unwrap();
    let b = seed_xor_split(&entropy, 3, &mut rng_b).unwrap();
    // share[0] should differ (mask differs)
    assert_ne!(a[0].as_slice(), b[0].as_slice());
}

#[test]
fn valid_entropy_lengths_constant() {
    assert_eq!(VALID_ENTROPY_LENGTHS, &[16, 20, 24, 28, 32]);
}
