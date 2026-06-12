//! GF(2^8) Rijndael field arithmetic for SLIP-39 Shamir Secret Sharing.
//!
//! Per SLIP-0039 §"Design Rationale": uses the irreducible polynomial
//! `x^8 + x^4 + x^3 + x + 1` (= `0x11b`, the same Rijndael/AES
//! polynomial). Generator element is 3 (the SLIP-39 reference impl
//! convention; primitive over the polynomial).
//!
//! Add is byte-XOR (characteristic 2). Mul / inv go via log/exp tables
//! computed once at module load time (`OnceLock`).

use std::sync::OnceLock;

/// Rijndael reduction polynomial = x^8 + x^4 + x^3 + x + 1.
pub const REDUCTION_POLY: u16 = 0x11b;

/// Generator element used for the log/exp tables. SLIP-39 reference
/// impl convention.
pub const GENERATOR: u8 = 3;

struct Tables {
    /// `exp[i]` = `GENERATOR^i` in GF(256), for `i` in `[0, 255)`.
    /// (`exp[255]` is identical to `exp[0]` = 1; the table is cyclic.)
    exp: [u8; 256],
    /// `log[a]` = discrete log of `a` base `GENERATOR`, for `a` in
    /// `[1, 256)`. `log[0]` is unused (kept as 0 sentinel).
    log: [u8; 256],
}

fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut exp = [0u8; 256];
        let mut log = [0u8; 256];
        let mut x: u16 = 1;
        for (i, slot) in exp.iter_mut().enumerate().take(255) {
            *slot = x as u8;
            log[x as usize] = i as u8;
            // Multiply by GENERATOR = 3 = polynomial (x + 1) in GF(2^8):
            //   x_new = (x << 1) XOR x      // = x * 2 + x in characteristic 2
            // then reduce mod REDUCTION_POLY if bit 8 is set.
            x = (x << 1) ^ x;
            if x & 0x100 != 0 {
                x ^= REDUCTION_POLY;
            }
        }
        // exp[255] = exp[0] = 1 (cyclic). Set explicitly so wrap-around
        // access works without modulo.
        exp[255] = 1;
        Tables { exp, log }
    })
}

/// Field addition in GF(256). Equivalent to XOR (characteristic 2).
pub fn add(a: u8, b: u8) -> u8 {
    a ^ b
}

/// Field multiplication in GF(256) via log/exp table lookup.
/// Returns 0 if either operand is 0.
pub fn mul(a: u8, b: u8) -> u8 {
    if a == 0 || b == 0 {
        return 0;
    }
    let t = tables();
    let log_sum = (t.log[a as usize] as u16) + (t.log[b as usize] as u16);
    let idx = if log_sum >= 255 {
        log_sum - 255
    } else {
        log_sum
    };
    t.exp[idx as usize]
}

/// Field multiplicative inverse in GF(256). PANICS if `a == 0`
/// (matches python-shamir-mnemonic's reference behavior; callers must
/// validate non-zero before invoking).
pub fn inv(a: u8) -> u8 {
    assert!(a != 0, "gf256::inv(0) is undefined");
    let t = tables();
    let neg_log = (255 - (t.log[a as usize] as u16)) % 255;
    t.exp[neg_log as usize]
}

/// Field division: `a / b = a * inv(b)`. PANICS if `b == 0`.
pub fn div(a: u8, b: u8) -> u8 {
    assert!(b != 0, "gf256::div by zero");
    mul(a, inv(b))
}
