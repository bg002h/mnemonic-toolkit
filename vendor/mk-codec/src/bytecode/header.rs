//! 1-byte bytecode header — version + reserved + fingerprint flag.
//!
//! Per `design/SPEC_mk_v0_1.md` §3.1 (closure Q-8 lock):
//!
//! ```text
//! bit 7-4: version (4 bits)        — 0x0 in v0.1
//! bit 3:   reserved                — MUST be 0 in v0.1
//! bit 2:   fingerprint flag        — 1 if origin_fingerprint is present
//! bit 1:   reserved                — MUST be 0 in v0.1
//! bit 0:   reserved                — MUST be 0 in v0.1
//! ```
//!
//! Valid v0.1 header bytes: `0x00` and `0x04`. mk1's bit-allocation
//! shape mirrors md1's; bit-2 fingerprint-flag semantics align at the
//! cross-format pattern level (D-14).

use crate::error::{Error, Result};

/// Version field width (bits).
const VERSION_SHIFT: u8 = 4;

/// Bit-2 fingerprint flag mask.
const FINGERPRINT_FLAG_MASK: u8 = 0b0000_0100;

/// Reserved-bit mask in v0.1: bits 0, 1, 3.
const RESERVED_MASK: u8 = 0b0000_1011;

/// Parsed mk1 bytecode header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BytecodeHeader {
    /// Version field, range 0..=15.
    pub version: u8,
    /// Bit 2: when `true`, `origin_fingerprint` is present in the payload.
    pub fingerprint_flag: bool,
}

impl BytecodeHeader {
    /// Parse a single byte into a `BytecodeHeader`. Rejects unknown
    /// versions and any reserved bits set.
    pub fn parse(byte: u8) -> Result<Self> {
        let version = byte >> VERSION_SHIFT;
        if version != 0 {
            return Err(Error::UnsupportedVersion(version));
        }
        if byte & RESERVED_MASK != 0 {
            return Err(Error::ReservedBitsSet);
        }
        Ok(BytecodeHeader {
            version,
            fingerprint_flag: byte & FINGERPRINT_FLAG_MASK != 0,
        })
    }

    /// Serialize the header to its single-byte wire form.
    pub fn to_byte(self) -> u8 {
        let mut byte = (self.version & 0x0F) << VERSION_SHIFT;
        if self.fingerprint_flag {
            byte |= FINGERPRINT_FLAG_MASK;
        }
        byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_no_fingerprint() {
        let h = BytecodeHeader::parse(0x00).unwrap();
        assert_eq!(h.version, 0);
        assert!(!h.fingerprint_flag);
        assert_eq!(h.to_byte(), 0x00);
    }

    #[test]
    fn round_trip_with_fingerprint() {
        let h = BytecodeHeader::parse(0x04).unwrap();
        assert_eq!(h.version, 0);
        assert!(h.fingerprint_flag);
        assert_eq!(h.to_byte(), 0x04);
    }

    #[test]
    fn rejects_unsupported_version() {
        // version=1 in bits 7-4
        assert!(matches!(
            BytecodeHeader::parse(0x10),
            Err(Error::UnsupportedVersion(1)),
        ));
        // version=15 in bits 7-4
        assert!(matches!(
            BytecodeHeader::parse(0xF0),
            Err(Error::UnsupportedVersion(15)),
        ));
    }

    #[test]
    fn rejects_reserved_bit_0() {
        assert!(matches!(
            BytecodeHeader::parse(0b0000_0001),
            Err(Error::ReservedBitsSet),
        ));
    }

    #[test]
    fn rejects_reserved_bit_1() {
        assert!(matches!(
            BytecodeHeader::parse(0b0000_0010),
            Err(Error::ReservedBitsSet),
        ));
    }

    #[test]
    fn rejects_reserved_bit_3() {
        assert!(matches!(
            BytecodeHeader::parse(0b0000_1000),
            Err(Error::ReservedBitsSet),
        ));
    }

    #[test]
    fn rejects_combined_reserved_bits() {
        // Bit 2 set (fp flag, allowed) + bit 0 set (reserved, rejected)
        assert!(matches!(
            BytecodeHeader::parse(0b0000_0101),
            Err(Error::ReservedBitsSet),
        ));
    }
}
