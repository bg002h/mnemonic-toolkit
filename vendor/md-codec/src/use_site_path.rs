//! Use-site-path-decl block per spec §3.5.
//!
//! Block format:
//!   [has-multipath: 1 bit]
//!   [if has-multipath:
//!     [alt-count: 3 bits, encoded count-2; range 2..9]
//!     [alternative × count]
//!   ]
//!   [wildcard-hardened: 1 bit]
//!
//! alternative: [hardened: 1 bit][value: LP4-ext varint]

use crate::bitstream::{BitReader, BitWriter};
use crate::error::Error;
use crate::varint::{read_varint, write_varint};

/// One alternative within a multipath substitution group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alternative {
    /// Whether this alternative is a hardened child index.
    pub hardened: bool,
    /// Child-number value of this alternative.
    pub value: u32,
}

impl Alternative {
    /// Encode this alternative onto the bit stream `w`.
    pub fn write(&self, w: &mut BitWriter) -> Result<(), Error> {
        w.write_bits(u64::from(self.hardened), 1);
        write_varint(w, self.value)?;
        Ok(())
    }

    /// Decode a single alternative from the bit stream `r`.
    pub fn read(r: &mut BitReader) -> Result<Self, Error> {
        let hardened = r.read_bits(1)? != 0;
        let value = read_varint(r)?;
        Ok(Self { hardened, value })
    }
}

/// Minimum number of alternatives in a multipath group.
pub const MIN_ALT_COUNT: usize = 2;
/// Maximum number of alternatives in a multipath group (3-bit field + 2).
pub const MAX_ALT_COUNT: usize = 9;

/// Use-site path declaration: an optional multipath group plus a wildcard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseSitePath {
    /// Optional multipath alternatives (`Some` ⇒ has-multipath bit set).
    pub multipath: Option<Vec<Alternative>>,
    /// Whether the trailing wildcard is hardened (`*h`).
    pub wildcard_hardened: bool,
}

impl UseSitePath {
    /// The dominant `<0;1>/*` form.
    pub fn standard_multipath() -> Self {
        Self {
            multipath: Some(vec![
                Alternative {
                    hardened: false,
                    value: 0,
                },
                Alternative {
                    hardened: false,
                    value: 1,
                },
            ]),
            wildcard_hardened: false,
        }
    }

    /// Encode this use-site path onto the bit stream `w`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::AltCountOutOfRange`] if `self.multipath` contains fewer
    /// than [`MIN_ALT_COUNT`] or more than [`MAX_ALT_COUNT`] alternatives.
    pub fn write(&self, w: &mut BitWriter) -> Result<(), Error> {
        if let Some(alts) = &self.multipath {
            if !(MIN_ALT_COUNT..=MAX_ALT_COUNT).contains(&alts.len()) {
                return Err(Error::AltCountOutOfRange { got: alts.len() });
            }
            w.write_bits(1, 1);
            // Encode alt-count - 2 in 3 bits per spec §4.2.
            w.write_bits((alts.len() - MIN_ALT_COUNT) as u64, 3);
            for a in alts {
                a.write(w)?;
            }
        } else {
            w.write_bits(0, 1);
        }
        w.write_bits(u64::from(self.wildcard_hardened), 1);
        Ok(())
    }

    /// Decode a use-site path from the bit stream `r`.
    pub fn read(r: &mut BitReader) -> Result<Self, Error> {
        let has_multipath = r.read_bits(1)? != 0;
        let multipath = if has_multipath {
            let alt_count = (r.read_bits(3)? as usize) + MIN_ALT_COUNT;
            let mut alts = Vec::with_capacity(alt_count);
            for _ in 0..alt_count {
                alts.push(Alternative::read(r)?);
            }
            Some(alts)
        } else {
            None
        };
        let wildcard_hardened = r.read_bits(1)? != 0;
        Ok(Self {
            multipath,
            wildcard_hardened,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_site_path_standard_round_trip() {
        let p = UseSitePath::standard_multipath();
        let mut w = BitWriter::new();
        p.write(&mut w).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(UseSitePath::read(&mut r).unwrap(), p);
    }

    #[test]
    fn use_site_path_standard_bit_cost() {
        // has-mp(1) + count=2(3) + alt0 (1+4) + alt1 (1+5) + wildcard(1) = 16 bits
        let p = UseSitePath::standard_multipath();
        let mut w = BitWriter::new();
        p.write(&mut w).unwrap();
        assert_eq!(w.bit_len(), 16);
    }

    #[test]
    fn use_site_path_bare_star_round_trip() {
        let p = UseSitePath {
            multipath: None,
            wildcard_hardened: false,
        };
        let mut w = BitWriter::new();
        p.write(&mut w).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(UseSitePath::read(&mut r).unwrap(), p);
    }

    #[test]
    fn use_site_path_bare_star_bit_cost() {
        // has-mp(0) + wildcard(0) = 2 bits
        let p = UseSitePath {
            multipath: None,
            wildcard_hardened: false,
        };
        let mut w = BitWriter::new();
        p.write(&mut w).unwrap();
        assert_eq!(w.bit_len(), 2);
    }

    #[test]
    fn use_site_path_hardened_wildcard_round_trip() {
        let p = UseSitePath {
            multipath: None,
            wildcard_hardened: true,
        };
        let mut w = BitWriter::new();
        p.write(&mut w).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(UseSitePath::read(&mut r).unwrap(), p);
    }

    #[test]
    fn use_site_path_alt_count_too_small_rejected() {
        let p = UseSitePath {
            multipath: Some(vec![Alternative {
                hardened: false,
                value: 0,
            }]),
            wildcard_hardened: false,
        };
        let mut w = BitWriter::new();
        assert!(matches!(
            p.write(&mut w),
            Err(Error::AltCountOutOfRange { got: 1 })
        ));
    }

    #[test]
    fn use_site_path_alt_count_too_large_rejected() {
        let p = UseSitePath {
            multipath: Some(
                (0..10)
                    .map(|i| Alternative {
                        hardened: false,
                        value: i,
                    })
                    .collect(),
            ),
            wildcard_hardened: false,
        };
        let mut w = BitWriter::new();
        assert!(matches!(
            p.write(&mut w),
            Err(Error::AltCountOutOfRange { got: 10 })
        ));
    }
}
