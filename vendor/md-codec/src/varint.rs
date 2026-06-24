//! LP4-ext varint per spec §4.1.
//!
//! Encoding: [L: 4 bits][payload: L bits], with L=15 reserved as a
//! continuation marker. When L=15: [L_high: 4 bits][payload_low: 14 bits]
//! [payload_high: L_high bits], total payload bits = L_high + 14.

use crate::bitstream::{BitReader, BitWriter};
use crate::error::Error;

/// Encode `value` as an LP4-ext varint into `writer`.
///
/// Returns `Err(Error::VarintOverflow)` if `value` exceeds the single-extension
/// payload range (max `2^29 - 1`). Recursive extension is not implemented;
/// callers must ensure values fit within the supported range.
pub fn write_varint(writer: &mut BitWriter, value: u32) -> Result<(), Error> {
    // Determine number of payload bits needed.
    let bits_needed = if value == 0 {
        0
    } else {
        32 - value.leading_zeros() as usize
    };

    if bits_needed <= 14 {
        writer.write_bits(bits_needed as u64, 4);
        writer.write_bits(value as u64, bits_needed);
        Ok(())
    } else {
        // Extension form: L=15, then [L_high:4][payload_low:14][payload_high:L_high]
        let l_high = bits_needed - 14;
        if l_high > 15 {
            return Err(Error::VarintOverflow { value });
        }
        writer.write_bits(15, 4);
        writer.write_bits(l_high as u64, 4);
        let low_mask = (1u64 << 14) - 1;
        let payload_low = (value as u64) & low_mask;
        let payload_high = (value as u64) >> 14;
        writer.write_bits(payload_low, 14);
        writer.write_bits(payload_high, l_high);
        Ok(())
    }
}

/// Decode an LP4-ext varint from `reader`.
pub fn read_varint(reader: &mut BitReader) -> Result<u32, Error> {
    let l = reader.read_bits(4)? as usize;
    if l < 15 {
        let value = reader.read_bits(l)? as u32;
        Ok(value)
    } else {
        let l_high = reader.read_bits(4)? as usize;
        let payload_low = reader.read_bits(14)? as u32;
        let payload_high = reader.read_bits(l_high)? as u32;
        Ok((payload_high << 14) | payload_low)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(value: u32) {
        let mut w = BitWriter::new();
        write_varint(&mut w, value).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_varint(&mut r).unwrap(), value);
    }

    #[test]
    fn varint_zero() {
        round_trip(0);
    }

    #[test]
    fn varint_one() {
        round_trip(1);
    }

    #[test]
    fn varint_84() {
        round_trip(84);
    }

    #[test]
    fn varint_1024() {
        round_trip(1024);
    }

    #[test]
    fn varint_16383_no_extension() {
        // 14-bit boundary; should NOT trigger extension
        round_trip(16383);
    }

    #[test]
    fn varint_16384_uses_extension() {
        round_trip(16384);
    }

    #[test]
    fn varint_max_u31() {
        // Single-extension range: 14 + L_high (max 15) = 29 payload bits.
        // Spec snippet labeled this "max_u31" but the wire format (4-bit L_high)
        // caps single-extension payload at 29 bits.
        round_trip((1u32 << 29) - 1);
    }

    #[test]
    fn varint_zero_costs_4_bits() {
        let mut w = BitWriter::new();
        write_varint(&mut w, 0).unwrap();
        assert_eq!(w.bit_len(), 4);
    }

    #[test]
    fn varint_one_costs_5_bits() {
        let mut w = BitWriter::new();
        write_varint(&mut w, 1).unwrap();
        assert_eq!(w.bit_len(), 5);
    }

    #[test]
    fn varint_84_costs_11_bits() {
        let mut w = BitWriter::new();
        write_varint(&mut w, 84).unwrap();
        assert_eq!(w.bit_len(), 11);
    }

    #[test]
    fn varint_overflow_returns_error_instead_of_panicking() {
        // 1 << 30 needs 30 payload bits; single-extension caps at 29.
        let mut w = BitWriter::new();
        let result = write_varint(&mut w, 1u32 << 30);
        assert!(matches!(result, Err(Error::VarintOverflow { value }) if value == 1u32 << 30));
    }

    #[test]
    fn varint_max_single_extension_succeeds() {
        // (1 << 29) - 1 needs exactly 29 payload bits = 14 + 15. Should succeed.
        let mut w = BitWriter::new();
        write_varint(&mut w, (1u32 << 29) - 1).unwrap();
    }

    #[test]
    fn varint_l_zero_decodes_to_zero_directly() {
        // Hand-craft 4 bits of zero (L=0) directly in a byte stream;
        // verify read_varint returns 0 without consuming additional bits.
        let mut w = BitWriter::new();
        w.write_bits(0b0000, 4); // L=0
        w.write_bits(0b1010, 4); // arbitrary trailing bits to confirm we don't consume them
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_varint(&mut r).unwrap(), 0);
        // Verify exactly 4 bits were consumed.
        assert_eq!(r.read_bits(4).unwrap(), 0b1010);
    }
}
