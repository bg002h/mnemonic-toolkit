//! Reed-Solomon-1024 BCH checksum (SLIP-0039 §3.2 / §3.5).
//!
//! Per the SLIP-0039 specification: a Reed-Solomon code over GF(1024)
//! with generator polynomial `(x − a)(x − a²)(x − a³)`, yielding a
//! 30-bit redundancy field encoded as 3 × 10-bit words. The state
//! register is 30 bits wide; ten precomputed `GEN[]` constants drive
//! the LFSR update once per input symbol.
//!
//! **Generator constants** (from SLIP-0039 §3.5 reference Python):
//!
//! ```text
//! GEN = [
//!     0x0E0E040,  // 0xe0e040
//!     0x1C1C080,  // 0x1c1c080
//!     0x3838100,
//!     0x7070200,
//!     0xE0E0009,
//!     0x1C0C2412,
//!     0x38086C24,
//!     0x3090FC48,
//!     0x21B1F890,
//!     0x3F3F120,
//! ]
//! ```
//!
//! **This is NOT the same generator as BIP-93 / codex32 RS1024.** The
//! two checksums share the RS1024 *name* (10-bit symbols, 3-word
//! parity) but use different reduction polynomials and customization
//! strings.
//!
//! **Customization string** is fed BEFORE the data, character by
//! character (US-ASCII byte values, each promoted to a 10-bit
//! symbol):
//!   - `b"shamir"` for non-extendable shares (ext bit = 0)
//!   - `b"shamir_extendable"` for extendable shares (ext bit = 1)
//!
//! The `ext` discriminator is share-encoding metadata, decoded at the
//! share-parse layer (`share.rs`, P1c-D); this module is `ext`-blind
//! and accepts the cs as a `&[u8]` parameter.
//!
//! **Verification contract:** `polymod(cs || data || checksum) == 1`
//! iff the share's checksum is valid. The factor-of-1 instead of 0 is
//! a SLIP-39 convention that makes purely-zero data fail verification
//! (per SLIP-0039 §3.5 commentary).

const GEN: [u32; 10] = [
    0xe0e040, 0x1c1c080, 0x3838100, 0x7070200, 0xe0e0009, 0x1c0c2412, 0x38086c24, 0x3090fc48,
    0x21b1f890, 0x3f3f120,
];

/// Reed-Solomon-1024 polymod step. Feeds each `values[i]` symbol
/// (interpreted as a 10-bit integer — high bits ignored) through the
/// LFSR initialized to 1. Returns the 30-bit residual register.
///
/// Equivalent to SLIP-0039 §3.5 `rs1024_polymod` Python reference.
pub fn polymod(values: &[u16]) -> u32 {
    let mut chk: u32 = 1;
    for &v in values {
        let b = chk >> 20;
        // Defensive 10-bit mask: spec contract is `v in 0..1024` but
        // Rust's `u16` allows up to 0xFFFF. The Python reference omits
        // this since Python ints have no fixed width; for Rust we
        // truncate to keep the LFSR state well-formed on malformed
        // input rather than silently corrupting subsequent rounds.
        chk = ((chk & 0xf_ffff) << 10) ^ u32::from(v & 0x3ff);
        for (i, gen) in GEN.iter().enumerate() {
            if (b >> i) & 1 != 0 {
                chk ^= gen;
            }
        }
    }
    chk
}

/// Compute the 3-word RS1024 checksum for `data` under customization
/// string `cs`. Each returned `u16` is in `0..1024`.
///
/// Per SLIP-0039 §3.5: append three zero placeholders, run polymod
/// over `cs_bytes || data || [0,0,0]`, XOR with 1, then unpack the
/// resulting 30-bit value into three 10-bit checksum words.
pub fn create_checksum(cs: &[u8], data: &[u16]) -> [u16; 3] {
    let mut values: Vec<u16> = Vec::with_capacity(cs.len() + data.len() + 3);
    values.extend(cs.iter().map(|b| u16::from(*b)));
    values.extend_from_slice(data);
    values.extend_from_slice(&[0u16; 3]);
    let polymod_result = polymod(&values) ^ 1;
    [
        ((polymod_result >> 20) & 0x3ff) as u16,
        ((polymod_result >> 10) & 0x3ff) as u16,
        (polymod_result & 0x3ff) as u16,
    ]
}

/// Verify the RS1024 checksum embedded as the final 3 words of
/// `data_with_checksum`, under customization string `cs`.
///
/// Returns `true` iff `polymod(cs_bytes || data_with_checksum) == 1`
/// (the SLIP-39 valid-codeword sentinel).
pub fn verify_checksum(cs: &[u8], data_with_checksum: &[u16]) -> bool {
    let mut values: Vec<u16> = Vec::with_capacity(cs.len() + data_with_checksum.len());
    values.extend(cs.iter().map(|b| u16::from(*b)));
    values.extend_from_slice(data_with_checksum);
    polymod(&values) == 1
}
