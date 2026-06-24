//! Single-payload header (5 bits) per SPEC v0.30 §2.1.
//!
//!   bit 4: divergent-paths flag (0 = shared origin path, 1 = divergent)
//!   bits 3..0: 4-bit version field (v0.30 = 4; usable WF-redesign set {4, 8, 12} per §2.4)
//!
//! The chunk header (`chunk.rs`, SPEC v0.30 §2.2) is a separate 37-bit form
//! with a different first-symbol layout — chunked-flag relocated to bit 0 of
//! the first 5-bit symbol enabling in-band auto-dispatch per SPEC §2.3. v0.x
//! single-payload (version=0) and v0.x chunked-misread-as-version=2 are both
//! rejected with `Error::WireVersionMismatch` per the SPEC §2.5 trace.

use crate::bitstream::{BitReader, BitWriter};
use crate::error::Error;

/// 5-bit single-payload header per SPEC v0.30 §2.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// Wire-format generation (4 bits). v0.30 = 4.
    pub version: u8,
    /// Bit 4: false = shared origin path, true = divergent per-`@N` paths.
    pub divergent_paths: bool,
}

impl Header {
    /// Wire-format version constant for v0.30 (the redesign cycle).
    /// Usable WF-redesign version set per SPEC §2.4: {4, 8, 12}.
    pub const WF_REDESIGN_VERSION: u8 = 4;

    /// Encode the 5-bit header into the bit stream.
    pub fn write(&self, w: &mut BitWriter) {
        let bits = (u64::from(self.divergent_paths) << 4) | u64::from(self.version & 0b1111);
        w.write_bits(bits, 5);
    }

    /// Decode the 5-bit header from the bit stream. Rejects inputs whose
    /// version field ≠ `WF_REDESIGN_VERSION` (4 in this release) with
    /// `Error::WireVersionMismatch` per SPEC §2.5.
    pub fn read(r: &mut BitReader) -> Result<Self, Error> {
        let bits = r.read_bits(5)?;
        let divergent_paths = (bits >> 4) & 1 != 0;
        let version = (bits & 0b1111) as u8;
        if version != Self::WF_REDESIGN_VERSION {
            return Err(Error::WireVersionMismatch { got: version });
        }
        Ok(Self {
            version,
            divergent_paths,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_round_trip_shared() {
        let h = Header {
            version: Header::WF_REDESIGN_VERSION,
            divergent_paths: false,
        };
        let mut w = BitWriter::new();
        h.write(&mut w);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(Header::read(&mut r).unwrap(), h);
    }

    #[test]
    fn header_round_trip_divergent() {
        let h = Header {
            version: Header::WF_REDESIGN_VERSION,
            divergent_paths: true,
        };
        let mut w = BitWriter::new();
        h.write(&mut w);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(Header::read(&mut r).unwrap(), h);
    }

    /// SPEC v0.30 §2.5 v0.x rejection trace. Two arms cover (a) v0.x
    /// single-payload (version=0) and (b) v0.x chunked read as single-
    /// payload (auto-dispatch read by `decode.rs` per §2.3 routes to
    /// `Header::read` and the embedded chunked-flag bit is parsed as
    /// version bit v0, yielding got=2). Both arms reject cleanly with
    /// `Error::WireVersionMismatch`.
    #[test]
    fn header_rejects_version_mismatch() {
        // Arm 1: v0.x single-payload (paths=0, version=0)
        //   first 5 bits MSB-first = [0][0][0][0][0] = 0b00000
        //   packed MSB-aligned byte = 0b00000_000 = 0x00
        let bytes = vec![0x00];
        let mut r = BitReader::new(&bytes);
        assert!(matches!(
            Header::read(&mut r),
            Err(Error::WireVersionMismatch { got: 0 })
        ));

        // Arm 2: v0.x chunked-misread-as-single-payload per SPEC §2.5
        //   v0.x chunked first 5 bits = [v_msb=0][v=0][v_lsb=0][chunked=1][reserved=0]
        //   = 0b00010 (numeric value 2). Read as v0.30 single-payload:
        //   bit 4 = paths = 0; bits 3..0 = version = 0b0010 = 2.
        //   packed MSB-aligned byte = 0b00010_000 = 0x10
        let bytes = vec![0x10];
        let mut r = BitReader::new(&bytes);
        assert!(matches!(
            Header::read(&mut r),
            Err(Error::WireVersionMismatch { got: 2 })
        ));
    }

    #[test]
    fn header_common_case_byte_value() {
        // Common case: version=4 (v0.30), divergent_paths=false ⇒
        //   first 5 bits = [paths=0][v3=0][v2=1][v1=0][v0=0] = 0b00100 = 0x04
        //   packed MSB-aligned byte = 0b00100_000 = 0x20
        let h = Header {
            version: Header::WF_REDESIGN_VERSION,
            divergent_paths: false,
        };
        let mut w = BitWriter::new();
        h.write(&mut w);
        assert_eq!(w.into_bytes(), vec![0x20]);
    }
}
