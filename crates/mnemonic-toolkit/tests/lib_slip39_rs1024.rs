//! v0.13.0 P1c — library tests for SLIP-39 RS1024 BCH checksum.
//!
//! Per SLIP-0039 §3.2 + §3.5:
//!   - Reed-Solomon code over GF(1024); generator polynomial degree 3
//!     ((x − a)(x − a²)(x − a³)).
//!   - 10 generator constants (one per parity-check bit) drive a
//!     30-bit LFSR register.
//!   - Customization string fed character-by-character (US-ASCII)
//!     BEFORE the data; cs = "shamir" (ext=0) or "shamir_extendable"
//!     (ext=1).
//!   - Checksum is 3 × 10-bit words appended to the share.
//!   - `polymod(cs || data || checksum) == 1` iff the checksum is
//!     valid.
//!
//! Coverage matrix:
//!   - polymod degenerate cases (empty input ⇒ initial register = 1).
//!   - create_checksum produces a [u16; 3] whose 10-bit elements are
//!     in `0..1024`.
//!   - create → verify round-trip for arbitrary data + cs.
//!   - verify catches single-symbol flips (BCH minimum distance ≥ 4
//!     guarantees this).
//!   - Spec-anchor (positive): vector #1 (`"duckling enlarge ..."`)
//!     verifies under cs=b"shamir".
//!   - Spec-anchor (negative): vector #2 (vector #1 with the last
//!     word flipped) does NOT verify under cs=b"shamir".
//!   - Spec-anchor (negative): vector #21 (256-bit invalid checksum)
//!     does NOT verify under cs=b"shamir".
//!   - Spec-anchor (extendable): vector #42 (`"testify swimming ..."`)
//!     verifies under cs=b"shamir_extendable" but NOT under
//!     cs=b"shamir".
//!   - Customization-string differentiation: same data, different cs
//!     ⇒ different checksum.

use mnemonic_toolkit::slip39::{rs1024, wordlist};

fn mnemonic_to_indices(m: &str) -> Vec<u16> {
    m.split_whitespace()
        .map(|w| wordlist::word_to_index(w).expect("test mnemonic word not in wordlist"))
        .collect()
}

// ============================================================================
// polymod LFSR invariants
// ============================================================================

#[test]
fn polymod_empty_input_equals_one() {
    // The LFSR initializes to chk=1; with no input symbols it stays at 1.
    assert_eq!(rs1024::polymod(&[]), 1);
}

#[test]
fn polymod_is_deterministic() {
    let data: Vec<u16> = (0u16..50).collect();
    let a = rs1024::polymod(&data);
    let b = rs1024::polymod(&data);
    assert_eq!(a, b);
}

// ============================================================================
// create_checksum / verify_checksum round-trip
// ============================================================================

#[test]
fn create_checksum_returns_three_10bit_words() {
    let data: Vec<u16> = vec![100, 200, 300, 400, 500];
    let cs = rs1024::create_checksum(b"shamir", &data);
    assert_eq!(cs.len(), 3);
    for w in cs {
        assert!(w < 1024, "checksum word {w} not in 0..1024");
    }
}

#[test]
fn create_then_verify_round_trip_shamir() {
    let data: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let chk = rs1024::create_checksum(b"shamir", &data);
    let mut full = data.clone();
    full.extend_from_slice(&chk);
    assert!(rs1024::verify_checksum(b"shamir", &full));
}

#[test]
fn create_then_verify_round_trip_shamir_extendable() {
    let data: Vec<u16> = vec![999, 1023, 0, 512, 256];
    let chk = rs1024::create_checksum(b"shamir_extendable", &data);
    let mut full = data.clone();
    full.extend_from_slice(&chk);
    assert!(rs1024::verify_checksum(b"shamir_extendable", &full));
}

// ============================================================================
// Tampering detection (BCH guarantee: single-symbol flip is caught)
// ============================================================================

#[test]
fn verify_catches_single_data_word_flip() {
    let data: Vec<u16> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let chk = rs1024::create_checksum(b"shamir", &data);
    let mut full = data.clone();
    full.extend_from_slice(&chk);
    assert!(rs1024::verify_checksum(b"shamir", &full));
    // Flip position 3 to any other value.
    full[3] ^= 1;
    assert!(!rs1024::verify_checksum(b"shamir", &full));
}

#[test]
fn verify_catches_single_checksum_word_flip() {
    let data: Vec<u16> = vec![1, 2, 3, 4, 5];
    let chk = rs1024::create_checksum(b"shamir", &data);
    let mut full = data.clone();
    full.extend_from_slice(&chk);
    // Flip the last checksum word.
    let last = full.len() - 1;
    full[last] ^= 1;
    assert!(!rs1024::verify_checksum(b"shamir", &full));
}

#[test]
fn verify_catches_two_symbol_flip() {
    // BCH(3) is guaranteed to catch any ≤ 3-symbol error; flipping 2
    // is well within that bound.
    let data: Vec<u16> = vec![100, 200, 300, 400, 500];
    let chk = rs1024::create_checksum(b"shamir", &data);
    let mut full = data.clone();
    full.extend_from_slice(&chk);
    full[0] ^= 5;
    full[4] ^= 7;
    assert!(!rs1024::verify_checksum(b"shamir", &full));
}

// ============================================================================
// Customization-string differentiation
// ============================================================================

#[test]
fn different_cs_yields_different_checksum() {
    let data: Vec<u16> = vec![1, 2, 3, 4, 5];
    let chk_a = rs1024::create_checksum(b"shamir", &data);
    let chk_b = rs1024::create_checksum(b"shamir_extendable", &data);
    assert_ne!(chk_a, chk_b);
}

#[test]
fn shamir_checksum_does_not_verify_under_extendable_cs() {
    let data: Vec<u16> = vec![1, 2, 3, 4, 5];
    let chk = rs1024::create_checksum(b"shamir", &data);
    let mut full = data.clone();
    full.extend_from_slice(&chk);
    // Valid under "shamir" — but should fail under the other cs.
    assert!(rs1024::verify_checksum(b"shamir", &full));
    assert!(!rs1024::verify_checksum(b"shamir_extendable", &full));
}

// ============================================================================
// Spec-anchor: vectors.json positive + negative samples
// ============================================================================

const VECTOR_1: &str = "duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision keyboard";
const VECTOR_2_INVALID_CHECKSUM: &str = "duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision kidney";
const VECTOR_21_INVALID_CHECKSUM_256: &str = "theory painting academic academic armed sweater year military elder discuss acne wildlife boring employer fused large satoshi bundle carbon diagnose anatomy hamster leaves tracks paces beyond phantom capital marvel lips brave detect lunar";
const VECTOR_42_EXTENDABLE: &str = "testify swimming academic academic column loyalty smear include exotic bedroom exotic wrist lobe cover grief golden smart junior estimate learn";

#[test]
fn vector_1_verifies_under_shamir() {
    let idx = mnemonic_to_indices(VECTOR_1);
    assert!(rs1024::verify_checksum(b"shamir", &idx));
}

#[test]
fn vector_2_fails_verify_under_shamir() {
    let idx = mnemonic_to_indices(VECTOR_2_INVALID_CHECKSUM);
    assert!(!rs1024::verify_checksum(b"shamir", &idx));
}

#[test]
fn vector_21_fails_verify_under_shamir() {
    let idx = mnemonic_to_indices(VECTOR_21_INVALID_CHECKSUM_256);
    assert!(!rs1024::verify_checksum(b"shamir", &idx));
}

#[test]
fn vector_42_extendable_verifies_under_extendable_cs() {
    let idx = mnemonic_to_indices(VECTOR_42_EXTENDABLE);
    assert!(rs1024::verify_checksum(b"shamir_extendable", &idx));
}

#[test]
fn vector_42_extendable_fails_verify_under_shamir_cs() {
    let idx = mnemonic_to_indices(VECTOR_42_EXTENDABLE);
    assert!(!rs1024::verify_checksum(b"shamir", &idx));
}
