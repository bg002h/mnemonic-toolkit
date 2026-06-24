//! Bit-aligned reader and writer.
//!
//! Per spec §4.6: bits are packed MSB-first into bytes. The first bit of the
//! payload occupies the most-significant bit of the first byte. The final byte
//! is zero-padded if needed.

use crate::error::Error;

/// MSB-first bit packer.
#[derive(Default)]
pub struct BitWriter {
    /// Backing byte buffer; the last byte is the in-progress byte.
    bytes: Vec<u8>,
    /// Bit offset within the last byte, in `0..8`. Zero means no in-progress byte.
    bit_position: usize,
}

impl BitWriter {
    /// Create an empty `BitWriter`.
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            bit_position: 0,
        }
    }

    /// Write `count` bits from `value` (LSB-aligned in `value`) into the
    /// stream MSB-first. Bits beyond `count` in `value` are ignored.
    pub fn write_bits(&mut self, value: u64, count: usize) {
        if count == 0 {
            return;
        }
        debug_assert!(count <= 64, "write_bits count must be ≤ 64");

        // Mask `value` to the requested bit count.
        let masked = if count == 64 {
            value
        } else {
            value & ((1u64 << count) - 1)
        };

        // Iterate from MSB to LSB of the requested value.
        let mut remaining = count;
        while remaining > 0 {
            // Ensure there's a current byte to write into.
            if self.bit_position == 0 {
                self.bytes.push(0);
            }
            let last = self.bytes.last_mut().unwrap();

            // How many bits free in the current byte (from bit_position MSB-side)?
            let free_in_byte = 8 - self.bit_position;
            let chunk = remaining.min(free_in_byte);

            // Pull `chunk` bits from the top of the masked value.
            let shift = (remaining - chunk) as u32;
            let bits = ((masked >> shift) & ((1u64 << chunk) - 1)) as u8;

            // Place bits into the byte at the correct offset (MSB-first).
            let byte_shift = (free_in_byte - chunk) as u32;
            *last |= bits << byte_shift;

            self.bit_position += chunk;
            if self.bit_position == 8 {
                self.bit_position = 0;
            }
            remaining -= chunk;
        }
    }

    /// Total number of bits written.
    pub fn bit_len(&self) -> usize {
        if self.bit_position == 0 {
            self.bytes.len() * 8
        } else {
            (self.bytes.len() - 1) * 8 + self.bit_position
        }
    }

    /// Consume self and produce the byte stream (final byte zero-padded).
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

// --- BitReader ---

/// MSB-first bit unpacker over a borrowed byte slice.
pub struct BitReader<'a> {
    /// Backing byte slice.
    bytes: &'a [u8],
    /// Total bits consumed so far (counted from the MSB of `bytes[0]`).
    bit_position: usize,
    /// Total bits available; defaults to `bytes.len() * 8`.
    bit_limit: usize,
}

impl<'a> BitReader<'a> {
    /// Reader that consumes exactly `bytes.len() * 8` bits (used by tests
    /// where the bit count is byte-aligned).
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bit_position: 0,
            bit_limit: bytes.len() * 8,
        }
    }

    /// Reader that consumes at most `bit_limit` bits — required when the
    /// payload's exact bit length is shorter than the byte buffer (zero-padding).
    /// Per spec §3.7, the TLV section ends when total bits are exhausted; the
    /// decoder must know `bit_limit` to avoid reading padding bits as TLV data.
    pub fn with_bit_limit(bytes: &'a [u8], bit_limit: usize) -> Self {
        debug_assert!(bit_limit <= bytes.len() * 8);
        Self {
            bytes,
            bit_position: 0,
            bit_limit,
        }
    }

    /// Read `count` bits MSB-first; returns the value LSB-aligned.
    pub fn read_bits(&mut self, count: usize) -> Result<u64, Error> {
        if count == 0 {
            return Ok(0);
        }
        debug_assert!(count <= 64, "read_bits count must be ≤ 64");
        if self.remaining_bits() < count {
            return Err(Error::BitStreamTruncated {
                requested: count,
                available: self.remaining_bits(),
            });
        }

        let mut result: u64 = 0;
        let mut remaining = count;
        while remaining > 0 {
            let byte_idx = self.bit_position / 8;
            let bit_in_byte = self.bit_position % 8; // 0 = MSB
            let free_in_byte = 8 - bit_in_byte;
            let chunk = remaining.min(free_in_byte);

            // Extract `chunk` bits starting at bit_in_byte from the MSB side.
            let byte = self.bytes[byte_idx];
            let shift = (free_in_byte - chunk) as u32;
            // Note: `1u8 << 8` overflows; guard explicitly.
            let mask: u8 = if chunk == 8 { 0xff } else { (1u8 << chunk) - 1 };
            let bits = (byte >> shift) & mask;

            result = (result << chunk) | bits as u64;
            self.bit_position += chunk;
            remaining -= chunk;
        }
        Ok(result)
    }

    /// Returns the current bit position within the stream. Used by the TLV
    /// decoder to measure consumed bits within a length-delimited region.
    pub(crate) fn bit_position(&self) -> usize {
        self.bit_position
    }

    /// Bits remaining unread (within the configured bit limit).
    pub fn remaining_bits(&self) -> usize {
        self.bit_limit.saturating_sub(self.bit_position)
    }

    /// Whether the stream is exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.remaining_bits() == 0
    }

    /// Snapshot the current bit position for rollback. Used by the TLV
    /// decoder loop to handle graceful end-of-stream when trailing
    /// codex32-padding bits look like a partial TLV.
    pub fn save_position(&self) -> usize {
        self.bit_position
    }

    /// Restore a previously saved bit position.
    pub fn restore_position(&mut self, saved: usize) {
        debug_assert!(saved <= self.bit_limit);
        self.bit_position = saved;
    }

    /// Snapshot the current bit_limit for later restoration. Paired with
    /// [`Self::set_bit_limit_for_scope`] when reading a length-delimited
    /// sub-region (e.g., a TLV body).
    pub(crate) fn save_bit_limit(&self) -> usize {
        self.bit_limit
    }

    /// Tighten the bit_limit to bound the next read operations. The new
    /// limit MUST be ≥ `bit_position` (callers already past the new
    /// limit would see truncation immediately) and ≤ the previous
    /// limit (cannot widen). Use [`Self::save_bit_limit`] to capture the
    /// prior limit and [`Self::restore_bit_limit`] to restore.
    pub(crate) fn set_bit_limit_for_scope(&mut self, new_limit: usize) {
        debug_assert!(new_limit >= self.bit_position);
        debug_assert!(new_limit <= self.bit_limit);
        self.bit_limit = new_limit;
    }

    /// Restore a previously saved bit_limit.
    pub(crate) fn restore_bit_limit(&mut self, saved: usize) {
        debug_assert!(self.bit_position <= saved);
        self.bit_limit = saved;
    }
}

/// Reads exactly `bit_len` MSB-first bits from `src_bytes` and appends them
/// to `dst`. Bits are sourced as if `src_bytes` were the output of a
/// `BitWriter` finalized with `into_bytes()` (so the trailing partial byte
/// is in the high bits of the last source byte). The destination is
/// extended in-place — no padding inserted.
///
/// Generalizes the read-bits-then-write-bits pattern used by the TLV
/// encoder when re-emitting a sub-bitstream's bits into the outer wire
/// without 1-bit drift on non-byte-aligned boundaries.
pub fn re_emit_bits(dst: &mut BitWriter, src_bytes: &[u8], bit_len: usize) -> Result<(), Error> {
    let mut src_reader = BitReader::with_bit_limit(src_bytes, bit_len);
    let mut remaining = bit_len;
    while remaining > 0 {
        let chunk = remaining.min(8);
        let bits = src_reader.read_bits(chunk)?;
        dst.write_bits(bits, chunk);
        remaining -= chunk;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_5_bits_msb_first() {
        let mut w = BitWriter::new();
        w.write_bits(0b10110, 5);
        // 0b10110_000 = 0xb0 in MSB-first packing of just 5 bits with zero
        // padding on the low 3 bits.
        assert_eq!(w.into_bytes(), vec![0b1011_0000]);
    }

    #[test]
    fn write_two_5_bit_values_packs_into_one_and_a_bit() {
        let mut w = BitWriter::new();
        w.write_bits(0b11111, 5);
        w.write_bits(0b00001, 5);
        // first 5: 11111___, then 00001 occupies bits 5..0 of the next
        // 5-bit slot. Combined: 11111_000_01 = 11111000_01000000 = 0xf8 0x40
        assert_eq!(w.into_bytes(), vec![0b1111_1000, 0b0100_0000]);
    }

    #[test]
    fn write_8_bits_is_one_byte() {
        let mut w = BitWriter::new();
        w.write_bits(0xab, 8);
        assert_eq!(w.into_bytes(), vec![0xab]);
    }

    #[test]
    fn write_zero_bits_is_noop() {
        let mut w = BitWriter::new();
        w.write_bits(0xff, 0);
        assert_eq!(w.bit_len(), 0);
        assert_eq!(w.into_bytes(), Vec::<u8>::new());
    }

    #[test]
    fn round_trip_5_bit_values() {
        let mut w = BitWriter::new();
        w.write_bits(0b10110, 5);
        w.write_bits(0b00001, 5);
        let bytes = w.into_bytes();

        let mut r = BitReader::new(&bytes);
        assert_eq!(r.read_bits(5).unwrap(), 0b10110);
        assert_eq!(r.read_bits(5).unwrap(), 0b00001);
    }

    #[test]
    fn read_past_end_errors() {
        let bytes = vec![0xff];
        let mut r = BitReader::new(&bytes);
        assert!(r.read_bits(9).is_err());
        // State must be preserved on truncation error.
        assert_eq!(r.remaining_bits(), 8);
    }

    #[test]
    fn read_full_byte_aligned() {
        let bytes = vec![0xab, 0xcd];
        let mut r = BitReader::new(&bytes);
        assert_eq!(r.read_bits(8).unwrap(), 0xab);
        assert_eq!(r.read_bits(8).unwrap(), 0xcd);
    }

    #[test]
    fn save_and_restore_position() {
        let bytes = vec![0b1011_0010, 0b0100_0000];
        let mut r = BitReader::new(&bytes);
        let saved = r.save_position();
        let _ = r.read_bits(5).unwrap();
        assert_eq!(r.save_position(), 5);
        r.restore_position(saved);
        assert_eq!(r.read_bits(5).unwrap(), 0b10110);
    }

    #[test]
    fn with_bit_limit_excludes_padding() {
        // 5-bit payload + 3-bit zero padding = 1 byte
        let mut w = BitWriter::new();
        w.write_bits(0b10110, 5);
        let bytes = w.into_bytes(); // [0b1011_0000]; padding is the trailing 000

        let mut r = BitReader::with_bit_limit(&bytes, 5);
        assert_eq!(r.read_bits(5).unwrap(), 0b10110);
        assert!(r.is_exhausted());
        // Attempting to read further (into the padding) errors.
        assert!(r.read_bits(1).is_err());
    }

    #[test]
    fn re_emit_bits_round_trip_byte_aligned() {
        // Source bitstream: a single full byte 0xab.
        let mut src = BitWriter::new();
        src.write_bits(0xab, 8);
        let src_bit_len = src.bit_len();
        let src_bytes = src.into_bytes();

        let mut dst = BitWriter::new();
        re_emit_bits(&mut dst, &src_bytes, src_bit_len).unwrap();

        assert_eq!(dst.bit_len(), 8);
        let dst_bytes = dst.into_bytes();
        assert_eq!(dst_bytes, vec![0xab]);
    }

    #[test]
    fn re_emit_bits_round_trip_all_widths_1_through_23() {
        // Sweep every bit-width in 1..=23. For each width, write a unique
        // pattern as the source, re-emit it into a destination, then read it
        // back from the destination and assert equality.
        for width in 1..=23usize {
            let pattern: u64 = if width == 64 {
                0xffff_ffff_ffff_ffff
            } else {
                (1u64 << width) - 1
            } & 0xa5_a5_a5_a5_a5_a5_a5_a5; // checkerboard, masked to width

            let mut src = BitWriter::new();
            src.write_bits(pattern, width);
            let src_bit_len = src.bit_len();
            let src_bytes = src.into_bytes();
            assert_eq!(src_bit_len, width);

            let mut dst = BitWriter::new();
            re_emit_bits(&mut dst, &src_bytes, width).unwrap();
            assert_eq!(dst.bit_len(), width);

            let dst_bytes = dst.into_bytes();
            let mut r = BitReader::with_bit_limit(&dst_bytes, width);
            assert_eq!(r.read_bits(width).unwrap(), pattern, "width={width}");
        }
    }

    #[test]
    fn re_emit_bits_non_byte_aligned_source() {
        // Source: 5 bits then 7 bits = 12-bit non-byte-aligned bitstream.
        let mut src = BitWriter::new();
        src.write_bits(0b10110, 5);
        src.write_bits(0b1010101, 7);
        let src_bit_len = src.bit_len();
        assert_eq!(src_bit_len, 12);
        let src_bytes = src.into_bytes();

        let mut dst = BitWriter::new();
        re_emit_bits(&mut dst, &src_bytes, src_bit_len).unwrap();
        assert_eq!(dst.bit_len(), 12);

        let dst_bytes = dst.into_bytes();
        let mut r = BitReader::with_bit_limit(&dst_bytes, 12);
        assert_eq!(r.read_bits(5).unwrap(), 0b10110);
        assert_eq!(r.read_bits(7).unwrap(), 0b1010101);
    }

    #[test]
    fn re_emit_bits_appends_to_existing_dst() {
        // Pre-fill destination with 3 bits, then re-emit 9 bits from source.
        // Verify total length is 12 and the bits are positioned correctly.
        let mut dst = BitWriter::new();
        dst.write_bits(0b101, 3);

        let mut src = BitWriter::new();
        src.write_bits(0b1_1110_0001, 9);
        let src_bit_len = src.bit_len();
        let src_bytes = src.into_bytes();

        re_emit_bits(&mut dst, &src_bytes, src_bit_len).unwrap();
        assert_eq!(dst.bit_len(), 12);

        let dst_bytes = dst.into_bytes();
        let mut r = BitReader::with_bit_limit(&dst_bytes, 12);
        assert_eq!(r.read_bits(3).unwrap(), 0b101);
        assert_eq!(r.read_bits(9).unwrap(), 0b1_1110_0001);
    }
}
