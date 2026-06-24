//! Origin-path-decl block per spec §3.4.
//!
//! Block format:
//! - shared mode (bit 4 = 0): `[n: 5 bits, encoded n-1][origin-path-encoding]`
//! - divergent mode (bit 4 = 1): `[n: 5 bits, encoded n-1][origin-path-encoding × n]`
//!
//! origin-path-encoding (explicit-only per D19′):
//!   `[depth: 4 bits][component × depth]`
//!
//! component:
//!   `[hardened: 1 bit][value: LP4-ext varint]`

use crate::bitstream::{BitReader, BitWriter};
use crate::error::Error;
use crate::varint::{read_varint, write_varint};

/// A single BIP-32 path component (e.g. `84'` or `0`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathComponent {
    /// Whether this component is hardened (apostrophe in BIP-32 notation).
    pub hardened: bool,
    /// Index value (u31 effective range, encoded as LP4-ext varint).
    pub value: u32,
}

impl PathComponent {
    /// Encode this component into `w`: 1 hardened bit + LP4-ext varint value.
    pub fn write(&self, w: &mut BitWriter) -> Result<(), Error> {
        w.write_bits(u64::from(self.hardened), 1);
        write_varint(w, self.value)?;
        Ok(())
    }

    /// Decode a `PathComponent` from `r`.
    pub fn read(r: &mut BitReader) -> Result<Self, Error> {
        let hardened = r.read_bits(1)? != 0;
        let value = read_varint(r)?;
        Ok(Self { hardened, value })
    }
}

/// Maximum number of components in a single origin path (4-bit depth field).
pub const MAX_PATH_COMPONENTS: usize = 15;

/// An explicit BIP-32 origin path (a sequence of `PathComponent`s).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OriginPath {
    /// Ordered components from root toward leaf.
    pub components: Vec<PathComponent>,
}

impl OriginPath {
    /// Encode the path: 4-bit depth followed by each component.
    pub fn write(&self, w: &mut BitWriter) -> Result<(), Error> {
        if self.components.len() > MAX_PATH_COMPONENTS {
            return Err(Error::PathDepthExceeded {
                got: self.components.len(),
                max: MAX_PATH_COMPONENTS,
            });
        }
        w.write_bits(self.components.len() as u64, 4);
        for c in &self.components {
            c.write(w)?;
        }
        Ok(())
    }

    /// Decode an `OriginPath` from `r`.
    pub fn read(r: &mut BitReader) -> Result<Self, Error> {
        let depth = r.read_bits(4)? as usize;
        let mut components = Vec::with_capacity(depth);
        for _ in 0..depth {
            components.push(PathComponent::read(r)?);
        }
        Ok(Self { components })
    }
}

/// A path declaration: key count `n` plus either a shared origin path or
/// `n` divergent origin paths (mode selected by header bit 4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathDecl {
    /// Key count, 1..=32 (encoded on the wire as `n - 1` in 5 bits).
    pub n: u8,
    /// Path payload — shared (single path) or divergent (one per key).
    pub paths: PathDeclPaths,
}

/// Path payload for a [`PathDecl`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathDeclPaths {
    /// Single origin path shared by all `n` keys (header bit 4 = 0).
    Shared(OriginPath),
    /// `n` distinct origin paths, one per key (header bit 4 = 1).
    Divergent(Vec<OriginPath>),
}

impl PathDecl {
    /// Encode this `PathDecl` into `w`. The mode (shared vs divergent) is
    /// determined by `self.paths`; the caller is responsible for setting
    /// header bit 4 to match.
    ///
    /// # Errors
    ///
    /// - [`Error::KeyCountOutOfRange`] if `self.n` is outside `1..=32`.
    /// - [`Error::DivergentPathCountMismatch`] if `self.paths` is `Divergent`
    ///   and the vector length does not equal `self.n`.
    /// - [`Error::PathDepthExceeded`] propagated from a component's path encoder
    ///   if any contained path exceeds [`MAX_PATH_COMPONENTS`].
    pub fn write(&self, w: &mut BitWriter) -> Result<(), Error> {
        if !(1..=32).contains(&(self.n as u32)) {
            return Err(Error::KeyCountOutOfRange { n: self.n });
        }
        // Encode n-1 in 5 bits per spec §4.2 (count - 1 offset).
        w.write_bits((self.n - 1) as u64, 5);
        match &self.paths {
            PathDeclPaths::Shared(p) => p.write(w)?,
            PathDeclPaths::Divergent(paths) => {
                if paths.len() != self.n as usize {
                    return Err(Error::DivergentPathCountMismatch {
                        n: self.n,
                        got: paths.len(),
                    });
                }
                for p in paths {
                    p.write(w)?;
                }
            }
        }
        Ok(())
    }

    /// Decode a `PathDecl` from `r` using `divergent_mode` (header bit 4).
    pub fn read(r: &mut BitReader, divergent_mode: bool) -> Result<Self, Error> {
        let n = (r.read_bits(5)? + 1) as u8;
        let paths = if divergent_mode {
            let mut paths = Vec::with_capacity(n as usize);
            for _ in 0..n {
                paths.push(OriginPath::read(r)?);
            }
            PathDeclPaths::Divergent(paths)
        } else {
            PathDeclPaths::Shared(OriginPath::read(r)?)
        };
        Ok(Self { n, paths })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bip84() -> OriginPath {
        // m/84'/0'/0'
        OriginPath {
            components: vec![
                PathComponent {
                    hardened: true,
                    value: 84,
                },
                PathComponent {
                    hardened: true,
                    value: 0,
                },
                PathComponent {
                    hardened: true,
                    value: 0,
                },
            ],
        }
    }

    #[test]
    fn origin_path_round_trip_bip84() {
        let p = bip84();
        let mut w = BitWriter::new();
        p.write(&mut w).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(OriginPath::read(&mut r).unwrap(), p);
    }

    #[test]
    fn origin_path_bit_cost_bip84() {
        // depth(4) + 84' (1+11) + 0' (1+4) + 0' (1+4) = 26 bits
        let p = bip84();
        let mut w = BitWriter::new();
        p.write(&mut w).unwrap();
        assert_eq!(w.bit_len(), 26);
    }

    #[test]
    fn origin_path_rejects_depth_too_large() {
        let p = OriginPath {
            components: (0..16)
                .map(|_| PathComponent {
                    hardened: false,
                    value: 0,
                })
                .collect(),
        };
        let mut w = BitWriter::new();
        assert!(matches!(
            p.write(&mut w),
            Err(Error::PathDepthExceeded { got: 16, max: 15 })
        ));
    }
}

#[cfg(test)]
mod path_decl_tests {
    use super::*;

    #[test]
    fn path_decl_shared_round_trip() {
        let p = PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(OriginPath {
                components: vec![
                    PathComponent {
                        hardened: true,
                        value: 84,
                    },
                    PathComponent {
                        hardened: true,
                        value: 0,
                    },
                    PathComponent {
                        hardened: true,
                        value: 0,
                    },
                ],
            }),
        };
        let mut w = BitWriter::new();
        p.write(&mut w).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(PathDecl::read(&mut r, false).unwrap(), p);
    }

    #[test]
    fn path_decl_shared_bit_cost_bip84() {
        // n(5) + depth(4) + 84' (1+11) + 0' (1+4) + 0' (1+4) = 31 bits
        let p = PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(OriginPath {
                components: vec![
                    PathComponent {
                        hardened: true,
                        value: 84,
                    },
                    PathComponent {
                        hardened: true,
                        value: 0,
                    },
                    PathComponent {
                        hardened: true,
                        value: 0,
                    },
                ],
            }),
        };
        let mut w = BitWriter::new();
        p.write(&mut w).unwrap();
        assert_eq!(w.bit_len(), 31);
    }

    #[test]
    fn path_decl_divergent_round_trip() {
        let p = PathDecl {
            n: 2,
            paths: PathDeclPaths::Divergent(vec![
                OriginPath {
                    components: vec![PathComponent {
                        hardened: true,
                        value: 84,
                    }],
                },
                OriginPath {
                    components: vec![PathComponent {
                        hardened: true,
                        value: 86,
                    }],
                },
            ]),
        };
        let mut w = BitWriter::new();
        p.write(&mut w).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(PathDecl::read(&mut r, true).unwrap(), p);
    }

    #[test]
    fn path_decl_n_zero_rejected() {
        let p = PathDecl {
            n: 0,
            paths: PathDeclPaths::Shared(OriginPath { components: vec![] }),
        };
        let mut w = BitWriter::new();
        assert!(matches!(
            p.write(&mut w),
            Err(Error::KeyCountOutOfRange { n: 0 })
        ));
    }
}
