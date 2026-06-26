//! 8 ↔ 11 bit regrouping, **MSB-first** (plan §4.1).
//!
//! The Word-Card payload is a byte string that must be packed into `GF(2^11)`
//! symbols (11 bits each) and back. The packing is **bit-precise**: an exact
//! `total_bits` is carried so the byte boundary need not be a multiple of 11
//! (the mk1 case is byte-aligned, `total_bits = 8 * len`, but md1 carries an
//! exact `total_bits` that is generally NOT a multiple of 8 — plan §4.1).
//!
//! Convention (plan §4.1): bits are consumed **most-significant-first** from each
//! byte, and packed most-significant-first into each 11-bit symbol. The final
//! partial symbol is **low-bit-padded with zeros**; the inverse asserts those
//! trailing pad bits are zero and rejects any non-zero pad.

/// Errors from the regroup decode path. Variants are **alphabetical** (plan /
/// `CLAUDE.md` convention; P4 folded the deferred P1-N1 reorder).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegroupError {
    /// The trailing pad bits (between `total_bits` and `11 * symbols.len()`) were
    /// not all zero, violating the frozen low-bit-zero-pad rule (plan §4.1).
    NonZeroPad,
    /// `symbols_to_bits` was asked for more bits than the symbol stream carries
    /// (`11 * symbols.len() < total_bits`).
    NotEnoughBits {
        /// Bits requested.
        requested: usize,
        /// Bits available in the symbol stream.
        available: usize,
    },
    /// A symbol value exceeded the 11-bit range (`>= 2048`).
    SymbolOutOfRange {
        /// Index of the offending symbol.
        index: usize,
        /// The offending value.
        value: u16,
    },
}

impl core::fmt::Display for RegroupError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RegroupError::NonZeroPad => {
                write!(f, "regroup: trailing pad bits are non-zero (must be zero)")
            }
            RegroupError::NotEnoughBits {
                requested,
                available,
            } => write!(
                f,
                "regroup: requested {requested} bits but symbol stream carries only {available}"
            ),
            RegroupError::SymbolOutOfRange { index, value } => {
                write!(f, "regroup: symbol[{index}] = {value} exceeds 11-bit range")
            }
        }
    }
}

impl std::error::Error for RegroupError {}

/// Number of 11-bit symbols needed to carry `total_bits` (ceil division).
#[inline]
fn symbol_count(total_bits: usize) -> usize {
    total_bits.div_ceil(11)
}

/// Pack the first `total_bits` bits of `bytes` (MSB-first) into 11-bit symbols.
///
/// The final partial symbol is **low-bit-padded with zeros**. Returns
/// `ceil(total_bits / 11)` symbols. `total_bits` must not exceed `8 * bytes.len()`
/// (the caller is responsible — debug-asserted).
///
/// Byte-aligned packing is just `total_bits = 8 * bytes.len()` (the mk1 case).
pub fn bits_to_symbols(bytes: &[u8], total_bits: usize) -> Vec<u16> {
    debug_assert!(
        total_bits <= bytes.len() * 8,
        "total_bits ({total_bits}) exceeds available bits ({})",
        bytes.len() * 8
    );
    let n_symbols = symbol_count(total_bits);
    let mut out = Vec::with_capacity(n_symbols);

    // A sliding bit accumulator: `acc` holds `acc_bits` pending bits, packed
    // MSB-first in its low `acc_bits` positions.
    let mut acc: u32 = 0;
    let mut acc_bits: u32 = 0;
    let mut bits_consumed = 0usize;

    let mut byte_idx = 0usize;
    while out.len() < n_symbols {
        // Refill the accumulator until we have >= 11 bits or have consumed all
        // requested bits.
        while acc_bits < 11 && bits_consumed < total_bits {
            let byte = bytes[byte_idx];
            // How many bits of this byte are still in range?
            let bits_left_in_byte = 8 - (bits_consumed % 8);
            let take = bits_left_in_byte.min(total_bits - bits_consumed);
            // Extract the top `take` bits starting at offset `bits_consumed % 8`.
            let offset = bits_consumed % 8;
            let chunk = (byte >> (8 - offset - take)) & ((1u16 << take) as u8).wrapping_sub(1);
            acc = (acc << take) | chunk as u32;
            acc_bits += take as u32;
            bits_consumed += take;
            if bits_consumed % 8 == 0 {
                byte_idx += 1;
            }
        }

        if acc_bits >= 11 {
            // Emit the top 11 bits.
            let shift = acc_bits - 11;
            let sym = ((acc >> shift) & 0x07FF) as u16;
            out.push(sym);
            acc_bits -= 11;
            acc &= (1u32 << acc_bits) - 1; // keep only the remaining low bits
        } else {
            // Final partial symbol: low-bit pad with zeros up to 11 bits.
            let pad = 11 - acc_bits;
            let sym = ((acc << pad) & 0x07FF) as u16;
            out.push(sym);
            acc = 0;
            acc_bits = 0;
        }
    }

    out
}

/// Inverse of [`bits_to_symbols`]: unpack `total_bits` bits (MSB-first) from the
/// 11-bit `symbols` into bytes.
///
/// The trailing pad bits — positions `total_bits .. 11 * symbols.len()` — are
/// **asserted to be zero**; a non-zero pad is rejected with
/// [`RegroupError::NonZeroPad`] (plan §4.1). The output byte length is
/// `ceil(total_bits / 8)`; the final byte is low-bit-zero-padded if `total_bits`
/// is not a multiple of 8.
pub fn symbols_to_bits(symbols: &[u16], total_bits: usize) -> Result<Vec<u8>, RegroupError> {
    let available = symbols.len() * 11;
    if total_bits > available {
        return Err(RegroupError::NotEnoughBits {
            requested: total_bits,
            available,
        });
    }
    for (index, &value) in symbols.iter().enumerate() {
        if value >= 2048 {
            return Err(RegroupError::SymbolOutOfRange { index, value });
        }
    }

    let n_bytes = total_bits.div_ceil(8);
    let mut out = vec![0u8; n_bytes];

    // Stream bits MSB-first from each symbol; place the first `total_bits` into
    // the output, then assert the remainder are zero.
    let mut bit_pos = 0usize; // absolute bit index across the symbol stream
    for &sym in symbols {
        for k in (0..11).rev() {
            let bit = ((sym >> k) & 1) as u8;
            if bit_pos < total_bits {
                if bit != 0 {
                    let byte = bit_pos / 8;
                    let off = bit_pos % 8;
                    out[byte] |= bit << (7 - off);
                }
            } else {
                // Trailing pad bit — must be zero.
                if bit != 0 {
                    return Err(RegroupError::NonZeroPad);
                }
            }
            bit_pos += 1;
        }
    }

    Ok(out)
}
