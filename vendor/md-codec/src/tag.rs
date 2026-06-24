//! v0.30 Tag enum per SPEC §3.
//!
//! 36 operators in primary 6-bit space (0x00..=0x23). Primary range
//! 0x24..=0x3E is reserved for future operators per SPEC §3.2's semantic
//! ranges. Primary value 0x3F is the extension prefix; 4-bit subcodes
//! 0x00..=0x0F are all reserved in v0.30 (no extension variants allocated).
//! TLV section tag space is SEPARATE and stays at 5-bit width per the Q13
//! split — decoder dispatches tag-width by context (bytecode vs TLV).

use crate::bitstream::{BitReader, BitWriter};
use crate::error::Error;

/// Operator tag identifying a descriptor/Miniscript fragment kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tag {
    /// `wpkh` — P2WPKH descriptor.
    Wpkh,
    /// `tr` — Taproot descriptor.
    Tr,
    /// `wsh` — P2WSH descriptor.
    Wsh,
    /// `sh` — P2SH descriptor.
    Sh,
    /// `pkh` — P2PKH descriptor.
    Pkh,
    /// Taproot tree node.
    TapTree,
    /// `multi` — k-of-n multisig.
    Multi,
    /// `sortedmulti` — sorted-key multisig.
    SortedMulti,
    /// `multi_a` — Tapscript multisig with `OP_CHECKSIGADD`.
    MultiA,
    /// `sortedmulti_a` — sorted-key Tapscript multisig.
    SortedMultiA,
    /// Miniscript `pk_k` — bare public key check.
    PkK,
    /// Miniscript `pk_h` — public-key-hash check.
    PkH,
    /// Miniscript `c:` wrapper (CHECKSIG).
    Check,
    /// Miniscript `v:` wrapper (VERIFY).
    Verify,
    /// Miniscript `s:` wrapper (SWAP).
    Swap,
    /// Miniscript `a:` wrapper (TOALTSTACK).
    Alt,
    /// Miniscript `d:` wrapper (DUPIF).
    DupIf,
    /// Miniscript `j:` wrapper (NONZERO).
    NonZero,
    /// Miniscript `n:` wrapper (ZERONOTEQUAL).
    ZeroNotEqual,
    /// Miniscript `and_v`.
    AndV,
    /// Miniscript `and_b`.
    AndB,
    /// Miniscript `andor`.
    AndOr,
    /// Miniscript `or_b`.
    OrB,
    /// Miniscript `or_c`.
    OrC,
    /// Miniscript `or_d`.
    OrD,
    /// Miniscript `or_i`.
    OrI,
    /// Miniscript `thresh`.
    Thresh,
    /// Miniscript `after` — absolute timelock.
    After,
    /// Miniscript `older` — relative timelock.
    Older,
    /// Miniscript `sha256`.
    Sha256,
    /// Miniscript `hash160`.
    Hash160,

    /// Miniscript `hash256` (primary 0x1F in v0.30; promoted from v0.x extension).
    Hash256,
    /// Miniscript `ripemd160` (primary 0x20 in v0.30; promoted from v0.x extension).
    Ripemd160,
    /// Raw public-key hash variant (primary 0x21 in v0.30; promoted from v0.x extension).
    RawPkH,
    /// Miniscript `0` literal (primary 0x22 in v0.30; promoted from v0.x extension).
    False,
    /// Miniscript `1` literal (primary 0x23 in v0.30; promoted from v0.x extension).
    True,
}

const EXTENSION_PREFIX_6BIT: u8 = 0x3F;

impl Tag {
    /// Returns `(primary_code, extension_code_opt)`. In v0.30 every allocated
    /// variant returns `(primary, None)` — the entire 4-bit extension subspace
    /// (behind primary prefix `0x3F`) is reserved for future operators. The
    /// `Option<u8>` shape is preserved for forward compatibility.
    pub(crate) fn codes(&self) -> (u8, Option<u8>) {
        match self {
            Tag::Wpkh => (0x00, None),
            Tag::Tr => (0x01, None),
            Tag::Wsh => (0x02, None),
            Tag::Sh => (0x03, None),
            Tag::Pkh => (0x04, None),
            Tag::TapTree => (0x05, None),
            Tag::Multi => (0x06, None),
            Tag::SortedMulti => (0x07, None),
            Tag::MultiA => (0x08, None),
            Tag::SortedMultiA => (0x09, None),
            Tag::PkK => (0x0A, None),
            Tag::PkH => (0x0B, None),
            Tag::Check => (0x0C, None),
            Tag::Verify => (0x0D, None),
            Tag::Swap => (0x0E, None),
            Tag::Alt => (0x0F, None),
            Tag::DupIf => (0x10, None),
            Tag::NonZero => (0x11, None),
            Tag::ZeroNotEqual => (0x12, None),
            Tag::AndV => (0x13, None),
            Tag::AndB => (0x14, None),
            Tag::AndOr => (0x15, None),
            Tag::OrB => (0x16, None),
            Tag::OrC => (0x17, None),
            Tag::OrD => (0x18, None),
            Tag::OrI => (0x19, None),
            Tag::Thresh => (0x1A, None),
            Tag::After => (0x1B, None),
            Tag::Older => (0x1C, None),
            Tag::Sha256 => (0x1D, None),
            Tag::Hash160 => (0x1E, None),
            Tag::Hash256 => (0x1F, None),
            Tag::Ripemd160 => (0x20, None),
            Tag::RawPkH => (0x21, None),
            Tag::False => (0x22, None),
            Tag::True => (0x23, None),
        }
    }

    /// Encode this tag (6 bits primary, plus 4 more if extension) into `w`.
    pub fn write(&self, w: &mut BitWriter) {
        let (primary, ext) = self.codes();
        w.write_bits(u64::from(primary), 6);
        if let Some(e) = ext {
            w.write_bits(u64::from(e), 4);
        }
    }

    /// Decode a tag from `r`, consuming 6 bits (or 10 for extension).
    ///
    /// Per SPEC v0.30 §3.2 + §11.1: 6-bit primary values 0x24..=0x3E are the
    /// reserved range and produce `Error::TagOutOfRange { primary }`. Primary
    /// value 0x3F is the extension prefix; the decoder consumes the following
    /// 4-bit subcode and returns `Error::TagOutOfRange { primary: 0x3F }`
    /// because no extension variants are allocated in v0.30 (the subcode is
    /// consumed but not reported in the error payload).
    pub fn read(r: &mut BitReader) -> Result<Self, Error> {
        let primary = r.read_bits(6)? as u8;
        if primary == EXTENSION_PREFIX_6BIT {
            // Consume the 4-bit subcode and reject — v0.30 allocates none.
            let _subcode = r.read_bits(4)?;
            return Err(Error::TagOutOfRange { primary });
        }
        match primary {
            0x00 => Ok(Tag::Wpkh),
            0x01 => Ok(Tag::Tr),
            0x02 => Ok(Tag::Wsh),
            0x03 => Ok(Tag::Sh),
            0x04 => Ok(Tag::Pkh),
            0x05 => Ok(Tag::TapTree),
            0x06 => Ok(Tag::Multi),
            0x07 => Ok(Tag::SortedMulti),
            0x08 => Ok(Tag::MultiA),
            0x09 => Ok(Tag::SortedMultiA),
            0x0A => Ok(Tag::PkK),
            0x0B => Ok(Tag::PkH),
            0x0C => Ok(Tag::Check),
            0x0D => Ok(Tag::Verify),
            0x0E => Ok(Tag::Swap),
            0x0F => Ok(Tag::Alt),
            0x10 => Ok(Tag::DupIf),
            0x11 => Ok(Tag::NonZero),
            0x12 => Ok(Tag::ZeroNotEqual),
            0x13 => Ok(Tag::AndV),
            0x14 => Ok(Tag::AndB),
            0x15 => Ok(Tag::AndOr),
            0x16 => Ok(Tag::OrB),
            0x17 => Ok(Tag::OrC),
            0x18 => Ok(Tag::OrD),
            0x19 => Ok(Tag::OrI),
            0x1A => Ok(Tag::Thresh),
            0x1B => Ok(Tag::After),
            0x1C => Ok(Tag::Older),
            0x1D => Ok(Tag::Sha256),
            0x1E => Ok(Tag::Hash160),
            0x1F => Ok(Tag::Hash256),
            0x20 => Ok(Tag::Ripemd160),
            0x21 => Ok(Tag::RawPkH),
            0x22 => Ok(Tag::False),
            0x23 => Ok(Tag::True),
            _ => Err(Error::TagOutOfRange { primary }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(t: Tag) {
        let mut w = BitWriter::new();
        t.write(&mut w);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(Tag::read(&mut r).unwrap(), t);
    }

    #[test]
    fn tag_wpkh() {
        round_trip(Tag::Wpkh);
    }
    #[test]
    fn tag_tr() {
        round_trip(Tag::Tr);
    }
    #[test]
    fn tag_taptree() {
        round_trip(Tag::TapTree);
    }
    #[test]
    fn tag_thresh() {
        round_trip(Tag::Thresh);
    }
    #[test]
    fn tag_hash256() {
        round_trip(Tag::Hash256);
    }
    #[test]
    fn tag_false() {
        round_trip(Tag::False);
    }
    #[test]
    fn tag_true() {
        round_trip(Tag::True);
    }

    /// SPEC v0.30 §3.2 + §11.1: the entire primary reserved range 0x24..=0x3E
    /// and the entire extension subspace 0x00..=0x0F (behind prefix 0x3F) are
    /// rejected with `Error::TagOutOfRange`. The variant's `primary` field
    /// carries the raw 6-bit value read off the wire — 0x3F for extension-
    /// subspace failures (the 4-bit subcode is consumed but not reported).
    #[test]
    fn tag_reserved_range_rejected() {
        // Arm 1: primary reserved-range low boundary 0x24
        let mut w = BitWriter::new();
        w.write_bits(0x24, 6);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert!(matches!(
            Tag::read(&mut r),
            Err(Error::TagOutOfRange { primary: 0x24 })
        ));

        // Arm 2: primary reserved-range high boundary 0x3E
        let mut w = BitWriter::new();
        w.write_bits(0x3E, 6);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert!(matches!(
            Tag::read(&mut r),
            Err(Error::TagOutOfRange { primary: 0x3E })
        ));

        // Arm 3: extension prefix 0x3F + subcode 0x00 (low boundary)
        let mut w = BitWriter::new();
        w.write_bits(0x3F, 6);
        w.write_bits(0x00, 4);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert!(matches!(
            Tag::read(&mut r),
            Err(Error::TagOutOfRange { primary: 0x3F })
        ));

        // Arm 4: extension prefix 0x3F + subcode 0x0F (high boundary)
        let mut w = BitWriter::new();
        w.write_bits(0x3F, 6);
        w.write_bits(0x0F, 4);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert!(matches!(
            Tag::read(&mut r),
            Err(Error::TagOutOfRange { primary: 0x3F })
        ));
    }
}
