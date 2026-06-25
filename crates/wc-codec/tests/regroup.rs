//! KATs for the bit-precise MSB-first 8 ↔ 11 regroup (plan §4.1).

use wc_codec::regroup::{bits_to_symbols, symbols_to_bits, RegroupError};

/// Hand-computed MSB-first vector. bytes = 0xB5 0x2A = bits 1011010100101010.
/// total_bits = 16 ⇒ 2 symbols:
///   sym0 = top 11 bits 10110101001 = 1449
///   sym1 = next 5 bits 01010, low-bit padded → 01010000000 = 640
#[test]
fn known_small_vector_msb_first() {
    let bytes = [0xB5u8, 0x2A];
    let syms = bits_to_symbols(&bytes, 16);
    assert_eq!(syms, vec![1449u16, 640], "MSB-first packing mismatch");

    // Inverse recovers the bytes; the 6 pad bits at the end are zero.
    let back = symbols_to_bits(&syms, 16).expect("decode");
    assert_eq!(back, bytes.to_vec());
}

/// Byte-aligned round-trip over a spread of buffers (the mk1 case,
/// total_bits = 8 * len).
#[test]
fn byte_aligned_round_trip() {
    for len in 0..40usize {
        // Deterministic pseudo-random bytes.
        let bytes: Vec<u8> = (0..len)
            .map(|i| (i.wrapping_mul(37).wrapping_add(11)) as u8)
            .collect();
        let total_bits = len * 8;
        let syms = bits_to_symbols(&bytes, total_bits);
        assert_eq!(
            syms.len(),
            total_bits.div_ceil(11),
            "symbol count (len={len})"
        );
        let back = symbols_to_bits(&syms, total_bits).expect("decode");
        assert_eq!(back, bytes, "byte-aligned round-trip (len={len})");
    }
}

/// Bit-precise round-trip for `total_bits` that is NEITHER a multiple of 8 NOR of
/// 11 — simulating an md1 payload whose exact bit length is carried through.
#[test]
fn bit_precise_round_trip_non_multiples() {
    // 0xB5 = 10110101. Use enough bytes to hold the requested bits.
    let bytes: Vec<u8> = (0..16u8)
        .map(|i| i.wrapping_mul(53).wrapping_add(7))
        .collect();
    // 91 is not a multiple of 8 (8*11=88, 91=88+3) and not a multiple of 11
    // (11*8=88, 91=88+3). Available bits = 16*8 = 128 ≥ 91.
    for &total_bits in &[91usize, 93, 17, 19, 100, 23, 45] {
        assert!(
            total_bits % 8 != 0,
            "{total_bits} must not be a multiple of 8"
        );
        assert!(
            total_bits % 11 != 0,
            "{total_bits} must not be a multiple of 11"
        );
        let syms = bits_to_symbols(&bytes, total_bits);
        assert_eq!(syms.len(), total_bits.div_ceil(11));
        let back = symbols_to_bits(&syms, total_bits).expect("decode");
        // `back` is ceil(total_bits/8) bytes; the carried bits must match the
        // source's top `total_bits` bits. Re-encode `back` and compare symbols.
        let re = bits_to_symbols(&back, total_bits);
        assert_eq!(re, syms, "bit-precise round-trip (total_bits={total_bits})");
    }
}

/// Decode rejects a non-zero trailing pad bit (plan §4.1: pad MUST be zero).
#[test]
fn rejects_nonzero_trailing_pad() {
    // total_bits = 16 ⇒ 2 symbols, 22 - 16 = 6 pad bits in the final symbol.
    let bytes = [0xB5u8, 0x2A];
    let mut syms = bits_to_symbols(&bytes, 16);
    // Flip the lowest pad bit of the final symbol (bit 0, which is a pad bit).
    syms[1] |= 1;
    assert_eq!(symbols_to_bits(&syms, 16), Err(RegroupError::NonZeroPad));

    // Flipping a non-pad bit is NOT a pad violation (it just changes the data).
    let mut syms2 = bits_to_symbols(&bytes, 16);
    syms2[0] ^= 1; // a data bit
    assert!(symbols_to_bits(&syms2, 16).is_ok());
}

/// Empty input is the identity (0 bits → 0 symbols → 0 bytes).
#[test]
fn empty_input() {
    let syms = bits_to_symbols(&[], 0);
    assert!(syms.is_empty());
    assert_eq!(symbols_to_bits(&[], 0), Ok(Vec::new()));
}

/// A single symbol carrying fewer than 11 meaningful bits round-trips, and an
/// all-zero pad survives.
#[test]
fn single_partial_symbol() {
    // 3 bits from 0xA0 = 101 (top 3 bits of 10100000).
    let syms = bits_to_symbols(&[0xA0], 3);
    assert_eq!(syms.len(), 1);
    // 101 padded to 11 bits low-zero → 10100000000 = 1280.
    assert_eq!(syms[0], 0b101_0000_0000);
    let back = symbols_to_bits(&syms, 3).expect("decode");
    // ceil(3/8) = 1 byte; top 3 bits = 101, rest zero → 0xA0.
    assert_eq!(back, vec![0xA0]);
}

/// Requesting more bits than the symbol stream carries is rejected.
#[test]
fn rejects_not_enough_bits() {
    let syms = vec![0u16; 1]; // carries 11 bits
    match symbols_to_bits(&syms, 12) {
        Err(RegroupError::NotEnoughBits {
            requested,
            available,
        }) => {
            assert_eq!(requested, 12);
            assert_eq!(available, 11);
        }
        other => panic!("expected NotEnoughBits, got {other:?}"),
    }
}

/// An out-of-range symbol (>= 2048) is rejected.
#[test]
fn rejects_symbol_out_of_range() {
    let syms = vec![2048u16];
    match symbols_to_bits(&syms, 11) {
        Err(RegroupError::SymbolOutOfRange { index, value }) => {
            assert_eq!(index, 0);
            assert_eq!(value, 2048);
        }
        other => panic!("expected SymbolOutOfRange, got {other:?}"),
    }
}
