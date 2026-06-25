//! `GF(2^11)` finite-field arithmetic — the value-layer field for the Word-Card
//! engine.
//!
//! # Frozen constants (plan §3 / spec §9.5)
//!
//! All values here are **frozen for recoverability**; the KATs in
//! `tests/field.rs` assert them and they MUST NOT change without a format
//! version bump.
//!
//! - **Field:** `GF(2^11)`.
//! - **Primitive polynomial:** `p(x) = x^11 + x^2 + 1` ⇒ the reduction constant
//!   `0x805` (bits 11, 2, 0 set). The "modulus" `0x805` is the 12-bit value whose
//!   bits mark the polynomial coefficients `x^11 .. x^0`.
//! - **Primitive element:** `α = x` ⇒ `0x002`. `ord(α) = 2047 = 23 · 89`, so the
//!   only proper divisors of the order are `23` and `89`; the primitivity KAT
//!   asserts `α^2047 = 1`, `α^23 ≠ 1`, `α^89 ≠ 1` (plan §3).
//!
//! Elements are represented as a `u16` carrying the low 11 bits (`0..=2047`).
//! There are `2048` elements and `2047` non-zero elements (the multiplicative
//! group `GF(2^11)^×`).

/// Reduction constant for the primitive polynomial `p(x) = x^11 + x^2 + 1`
/// (plan §3). Bit 11 (`x^11`), bit 2 (`x^2`) and bit 0 (`1`) are set.
pub const MODULUS: u16 = 0x805;

/// The primitive element `α = x` (plan §3). Its multiplicative order is the full
/// group order `2047`.
pub const ALPHA: u16 = 0x002;

/// Number of elements in the field, `2^11 = 2048`.
pub const ORDER: u16 = 2048;

/// Order of the multiplicative group `GF(2^11)^× = ORDER - 1 = 2047 = 23 · 89`.
pub const MULTIPLICATIVE_ORDER: u16 = 2047;

/// Mask isolating the low 11 bits — the canonical element representation.
const ELEM_MASK: u16 = 0x07FF;

/// Field addition. In characteristic-2 fields addition is bitwise XOR (and is its
/// own inverse: `a + a = 0`).
#[inline]
pub fn add(a: u16, b: u16) -> u16 {
    (a ^ b) & ELEM_MASK
}

/// Field subtraction — identical to [`add`] in characteristic 2 (XOR is its own
/// inverse). Provided as a named alias for call-site clarity.
#[inline]
pub fn sub(a: u16, b: u16) -> u16 {
    add(a, b)
}

/// Field multiplication via carry-less (Russian-peasant) multiplication reduced
/// modulo the primitive polynomial `0x805`.
///
/// On each step we accumulate `a` into the result if the current low bit of `b`
/// is set, then shift `a` left by one. When that shift carries out of bit 10
/// (i.e. the pre-shift value had bit 10 set), the product now has an `x^11` term,
/// which we reduce by XOR-ing the polynomial `0x805` (drops bit 11, folds in the
/// `x^2 + 1` tail).
#[inline]
pub fn mul(a: u16, b: u16) -> u16 {
    let mut a = a & ELEM_MASK;
    let mut b = b & ELEM_MASK;
    let mut product: u16 = 0;
    while b != 0 {
        if b & 1 != 0 {
            product ^= a;
        }
        b >>= 1;
        // Will the upcoming `<<1` carry out of bit 10 into bit 11?
        let carry = a & 0x0400; // bit 10
        a <<= 1;
        if carry != 0 {
            a ^= MODULUS; // reduce: clears bit 11, folds x^2 + 1
        }
        a &= ELEM_MASK;
    }
    product & ELEM_MASK
}

/// Exponentiation by squaring: `base^exp` in `GF(2^11)`.
///
/// `pow(_, 0) == 1` (the multiplicative identity), including `pow(0, 0) == 1` by
/// the standard convention.
pub fn pow(base: u16, mut exp: u32) -> u16 {
    let mut result: u16 = 1;
    let mut b = base & ELEM_MASK;
    while exp != 0 {
        if exp & 1 != 0 {
            result = mul(result, b);
        }
        b = mul(b, b);
        exp >>= 1;
    }
    result
}

/// Multiplicative inverse of a non-zero element, via Fermat's little theorem:
/// `a^(2^11 - 2) = a^2046 = a^(-1)` for `a ≠ 0` (since `a^2047 = 1`).
///
/// Returns `None` for `0` (which has no inverse).
pub fn inv(a: u16) -> Option<u16> {
    if a & ELEM_MASK == 0 {
        return None;
    }
    // a^(q-2) where q = 2047 (group order): a^2045 * a = a^2046, and
    // a^2046 * a = a^2047 = 1, so a^2046 is the inverse.
    Some(pow(a, (MULTIPLICATIVE_ORDER - 1) as u32))
}
