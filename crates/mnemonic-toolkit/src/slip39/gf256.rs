//! GF(2^8) Rijndael field arithmetic for SLIP-39 Shamir Secret Sharing.
//!
//! Per SLIP-0039 §"Design Rationale": uses the irreducible polynomial
//! `x^8 + x^4 + x^3 + x + 1` (= `0x11b`, the same Rijndael/AES
//! polynomial). Generator element is 3 (the SLIP-39 reference impl
//! convention).
//!
//! Add is byte-XOR (characteristic 2). Mul / inv go via log/exp tables
//! precomputed at module load time (`OnceLock`).
//!
//! Constant-time discipline: all operations either go through the
//! precomputed tables (data-dependent timing observable only at the
//! table-lookup level, not branch-dependent) or fall through a single
//! early-return on the zero operand. Acceptable for SLIP-39's threat
//! model (recovery operations are not on hot paths).

// Phase 1a RED stub: type signatures + module structure only.
// Bodies + log/exp tables land in P1a GREEN.

/// Rijndael reduction polynomial = x^8 + x^4 + x^3 + x + 1.
pub const REDUCTION_POLY: u16 = 0x11b;

/// Generator element used for the log/exp tables. SLIP-39 reference
/// impl convention.
pub const GENERATOR: u8 = 3;

/// Field addition in GF(256). Equivalent to XOR (characteristic 2).
pub fn add(a: u8, b: u8) -> u8 {
    todo!("P1a GREEN — implement byte-XOR")
}

/// Field multiplication in GF(256) via log/exp table lookup.
/// Returns 0 if either operand is 0.
pub fn mul(_a: u8, _b: u8) -> u8 {
    todo!("P1a GREEN — implement table-based mul")
}

/// Field multiplicative inverse in GF(256). PANICS if `a == 0`
/// (matches python-shamir-mnemonic's reference behavior;
/// callers must validate non-zero before invoking).
pub fn inv(_a: u8) -> u8 {
    todo!("P1a GREEN — implement table-based inv")
}

/// Field division: `a / b = a * inv(b)`. PANICS if `b == 0`.
pub fn div(_a: u8, _b: u8) -> u8 {
    todo!("P1a GREEN — implement via mul + inv")
}
