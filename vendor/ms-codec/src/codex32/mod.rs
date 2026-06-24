// Vendored from `codex32` v0.1.0 (crates.io checksum
// d230935faa4d0521349d228f39aba4ff489cf2a8bcab4d84e31f4cbd6fe918e9), CC0-1.0,
// by Andrew Poelstra. Inlined into ms-codec (Cycle-B, shape A) to own the
// Zeroize/Drop/redacting-Debug secret-hygiene fixes (FOLLOWUP
// codex32-upstream-dormant-vendor-vs-accept-decision).
//
// Copied from the upstream runtime modules (lib.rs / field.rs / checksum.rs).
// The ONLY substantive edits are: (1) Zeroize/ZeroizeOnDrop + a redacting Debug
// on `Codex32String` (Phase 2); (2) module-routing of `use` paths to fit the
// inlined submodule (`crate::field` -> `super::field` in checksum.rs, since the
// codex32 crate root is now the `codex32` submodule); (3) crate-local lint
// `#![allow(..)]`s (the crate denies missing_docs / runs -D-warnings clippy; the
// upstream copy predates both). rustfmt also normalized a few cosmetic spots
// (import order, one array literal, one fn-signature wrap) — NONE touch encoding
// logic. The ENCODING (from_seed/from_string/interpolate_at/checksum/field) is
// behaviorally UNCHANGED; the wire-byte-identity invariant is proven by
// tests/codex32_vendor_parity.rs. The upstream CC0 LICENSE is retained verbatim
// alongside as src/codex32/LICENSE.
//
// Rust Codex32 Library and Reference Implementation
// Written in 2023 by
//   Andrew Poelstra <apoelstra@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! codex32 Reference Implementation
//!
//! This project is a reference implementation of BIP-XXX "codex32", a project
//! by Leon Olson Curr and Pearlwort Snead to produce checksummed and secret-shared
//! BIP32 master seeds.
//!
//! References:
//!   * BIP-XXX <https://github.com/apoelstra/bips/blob/2023-02--volvelles/bip-0000.mediawiki>
//!   * The codex32 website <https://www.secretcodex32.com>
//!   * BIP-0173 "bech32" <https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki>
//!   * BIP-0032 "BIP 32" <https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki>
//!

// This is the shittiest lint ever and has literally never been correct when
// it has fired, and somehow in rust-bitcoin managed NOT to fire in the one
// case where it might've been useful.
// https://github.com/rust-bitcoin/rust-bitcoin/pull/1701
#![allow(clippy::suspicious_arithmetic_impl)]
// Vendored-module lint allowances (see the header note, edit class 3): the
// crate denies missing_docs (the upstream `Fe` constants + a few helpers are
// undocumented) and runs clippy under -D warnings (upstream predates two style
// lints firing on the byte-identical body — `precedence` in `from_seed`'s base32
// packing, `needless_lifetimes` on `Parts`). Scope the allows to this module so
// the runtime body stays verbatim and the crate-wide gates are unaffected.
#![allow(missing_docs)]
#![allow(clippy::precedence)]
#![allow(clippy::needless_lifetimes)]

mod checksum;
mod field;

pub use checksum::Engine as ChecksumEngine;
pub use field::Fe;
use std::{cmp, fmt};

#[derive(Debug)]
pub enum Error {
    /// Error related to a single bech32 character
    Field(field::Error),
    /// Identifier had wrong length when creating a share
    IdNotLength4(usize),
    /// When translating from u5 to u8, there was an incomplete group of
    /// size greater than 4 bits, meaning an entirely extraneous character.
    IncompleteGroup(usize),
    /// Tried a codex32 string of an illegal length
    InvalidLength(usize),
    /// Tried to decode a character which was not part of the bech32 alphabet,
    /// or, if in the HRP, was not ASCII.
    InvalidChar(char),
    /// Tried to decode a character but its case did not match the expected case
    InvalidCase(Case, char),
    /// String had an invalid checksum
    InvalidChecksum {
        /// Checksum we used, "long" or "short"
        checksum: &'static str,
        /// The string with the bad checksum
        string: String,
    },
    /// Threshold was not an allowed value (2 through 9, or 0)
    InvalidThreshold(char),
    /// Threshold was not an allowed value (2 through 9, or 0)
    InvalidThresholdN(usize),
    /// Share index was not an allowed value (only S if the threshold is 0,
    /// otherwise anything goes)
    InvalidShareIndex(Fe),
    /// A set of shares to be interpolated did not all have the same length
    MismatchedLength(usize, usize),
    /// A set of shares to be interpolated did not all have the same HRP
    MismatchedHrp(String, String),
    /// A set of shares to be interpolated did not all have the same threshold
    MismatchedThreshold(usize, usize),
    /// A set of shares to be interpolated did not all have the same ID
    MismatchedId(String, String),
    /// A share index was repeated in the set of shares to interpolate.
    RepeatedIndex(Fe),
    /// A set of shares to be interpolated did not have enough shares
    ThresholdNotPassed { threshold: usize, n_shares: usize },
}

impl From<field::Error> for Error {
    fn from(e: field::Error) -> Error {
        Error::Field(e)
    }
}

/// Lowercase or uppercase (as applied to the bech32 alphabet)
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub enum Case {
    /// qpzr...
    Lower,
    /// QPZR...
    Upper,
}

/// A codex32 string, containing a valid checksum
///
/// Cycle-B P2 (the ONLY behavioral change vs upstream): the inner secret
/// `String` is scrubbed on drop via `zeroize::ZeroizeOnDrop`, and `Debug` is
/// hand-rolled length-only (the upstream derived `Debug` echoed the full secret
/// — the L22-class footgun). `Clone`/`PartialEq`/`Eq`/`Hash` are RETAINED
/// (load-bearing: `interpolate_at`'s self-return clone, `combine_shares`'s
/// `derived != parsed[j]` compare, source-compat). The encoding bodies are
/// UNTOUCHED.
#[derive(Clone, PartialEq, Eq, Hash, zeroize::ZeroizeOnDrop)]
pub struct Codex32String(String);

impl fmt::Display for Codex32String {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for Codex32String {
    /// Redacting: NEVER echoes the secret string (the upstream derived `Debug`
    /// leaked it). Length-only — enough to debug a length/shape bug, nothing of
    /// the payload. The char-count is non-sensitive (ms1 lengths are a small
    /// public set).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Codex32String([REDACTED; {} chars])",
            self.0.chars().count()
        )
    }
}

impl Codex32String {
    fn sanity_check(&self) -> Result<(), Error> {
        let parts = self.parts_inner()?;
        let incomplete_group = (parts.payload.len() * 5) % 8;
        if incomplete_group > 4 {
            return Err(Error::IncompleteGroup(incomplete_group));
        }
        Ok(())
    }

    /// Construct a codex32 string from a not-yet-checksummed string
    pub fn from_unchecksummed_string(mut s: String) -> Result<Self, Error> {
        // Determine what checksum to use and extend the string
        let (len, mut checksum) = if s.len() < 81 {
            (13, checksum::Engine::new_codex32_short())
        } else {
            (15, checksum::Engine::new_codex32_long())
        };
        s.reserve_exact(len);

        // Split out the HRP
        let (hrp, real_string) = match s.rsplit_once('1') {
            Some((s1, s2)) => (s1, s2),
            None => ("", &s[..]),
        };
        // Compute the checksum
        checksum.input_hrp(hrp)?;
        checksum.input_data_str(real_string)?;
        for ch in checksum.into_residue() {
            s.push(ch.to_char());
        }

        let ret = Codex32String(s);
        ret.sanity_check()?;
        Ok(ret)
    }

    /// Construct a codex32 string from an already-checksummed string
    pub fn from_string(s: String) -> Result<Self, Error> {
        let (name, mut checksum) = if s.len() >= 48 && s.len() < 94 {
            ("short", checksum::Engine::new_codex32_short())
        } else if s.len() >= 125 && s.len() < 128 {
            ("long", checksum::Engine::new_codex32_long())
        } else {
            return Err(Error::InvalidLength(s.len()));
        };

        // Split out the HRP
        let (hrp, real_string) = match s.rsplit_once('1') {
            Some((s1, s2)) => (s1, s2),
            None => ("", &s[..]),
        };
        checksum.input_hrp(hrp)?;
        checksum.input_data_str(real_string)?;
        if !checksum.is_valid() {
            return Err(Error::InvalidChecksum {
                checksum: name,
                string: s,
            });
        }
        // Looks good, return
        let ret = Codex32String(s);
        ret.sanity_check()?;
        Ok(ret)
    }

    /// Break the string up into its constituent parts
    fn parts_inner(&self) -> Result<Parts, Error> {
        let (hrp, s) = match self.0.rsplit_once('1') {
            Some((s1, s2)) => (s1, s2),
            None => ("", &self.0[..]),
        };
        let checksum_len = if self.0.len() > 93 { 15 } else { 13 };
        let ret = Parts {
            hrp,
            threshold: match s.as_bytes()[0] {
                b'0' => 0,
                b'2' => 2,
                b'3' => 3,
                b'4' => 4,
                b'5' => 5,
                b'6' => 6,
                b'7' => 7,
                b'8' => 8,
                b'9' => 9,
                _ => return Err(Error::InvalidThreshold(s.as_bytes()[0].into())),
            },
            id: &s[1..5],
            share_index: Fe::from_char(s.as_bytes()[5].into()).unwrap(),
            payload: &s[6..s.len() - checksum_len],
            checksum: &s[s.len() - checksum_len..],
        };
        if ret.threshold == 0 && ret.share_index != Fe::S {
            return Err(Error::InvalidShareIndex(ret.share_index));
        }
        Ok(ret)
    }

    /// Break the string up into its constituent parts
    pub fn parts(&self) -> Parts {
        // unwrap OK since we validated the input on parse
        self.parts_inner().unwrap()
    }

    /// Interpolate a set of shares to derive a share at a specific index.
    ///
    /// Using the index `Fe::S` will recover the master seed.
    pub fn interpolate_at(shares: &[Codex32String], target: Fe) -> Result<Codex32String, Error> {
        // Collect indices and sanity check
        if shares.is_empty() {
            return Err(Error::ThresholdNotPassed {
                threshold: 1,
                n_shares: 0,
            });
        }
        let mut indices = Vec::with_capacity(shares.len());
        let s0_parts = shares[0].parts();
        if s0_parts.threshold > shares.len() {
            return Err(Error::ThresholdNotPassed {
                threshold: s0_parts.threshold,
                n_shares: shares.len(),
            });
        }
        for share in shares {
            let parts = share.parts();
            if shares[0].0.len() != share.0.len() {
                return Err(Error::MismatchedLength(shares[0].0.len(), share.0.len()));
            }
            if s0_parts.hrp != parts.hrp {
                return Err(Error::MismatchedHrp(s0_parts.hrp.into(), parts.hrp.into()));
            }
            if s0_parts.threshold != parts.threshold {
                return Err(Error::MismatchedThreshold(
                    s0_parts.threshold,
                    parts.threshold,
                ));
            }
            if s0_parts.id != parts.id {
                return Err(Error::MismatchedId(s0_parts.id.into(), parts.id.into()));
            }
            indices.push(parts.share_index);
        }

        // Do lagrange interpolation
        let mut mult = Fe::P;
        for i in 0..shares.len() {
            if indices[i] == target {
                // If we're trying to output an input share, just output it directly.
                // Naive Lagrange multiplication would otherwise multiply by 0.
                return Ok(shares[i].clone());
            }

            mult *= indices[i] + target;
        }

        let payload_len = 6 + s0_parts.payload.len() + s0_parts.checksum.len();
        let hrp_len = shares[0].0.len() - payload_len;
        let mut result = vec![Fe::Q; payload_len];

        for i in 0..shares.len() {
            let mut inv = Fe::P;
            for j in 0..shares.len() {
                inv *= indices[j]
                    + if i == j {
                        target
                    } else {
                        // If there is a repeated index, just call this an error. Technically
                        // speaking, we could reject the other one and re-do the threshold
                        // check in case we had enough unique ones .. but easier to just make
                        // it the user's responsibility to provide unique indices to begin with.
                        if indices[i] == indices[j] {
                            return Err(Error::RepeatedIndex(indices[i]));
                        }
                        indices[i]
                    }
            }

            for (j, res_j) in result.iter_mut().enumerate() {
                let ch_at_i = char::from(shares[i].0.as_bytes()[hrp_len + j]);
                *res_j += mult / inv * Fe::from_char(ch_at_i).unwrap();
            }
        }

        let mut s = s0_parts.hrp.to_owned();
        s.push('1');
        if s0_parts.hrp.chars().all(char::is_uppercase) {
            s.extend(
                result
                    .into_iter()
                    .map(Fe::to_char)
                    .map(|c| c.to_ascii_uppercase()),
            );
        } else {
            s.extend(result.into_iter().map(Fe::to_char));
        }
        Ok(Codex32String(s))
    }

    /// Creates a S share from bare seed data
    pub fn from_seed(
        hrp: &str,
        threshold: usize,
        id: &str,
        share_idx: Fe,
        data: &[u8],
    ) -> Result<Codex32String, Error> {
        if id.len() != 4 {
            return Err(Error::IdNotLength4(id.len()));
        }

        let mut ret = String::with_capacity(hrp.len() + 6 + (data.len() * 8 + 4) / 5);
        ret.push_str(hrp);
        ret.push('1');
        let k = match threshold {
            0 => Fe::_0,
            2 => Fe::_2,
            3 => Fe::_3,
            4 => Fe::_4,
            5 => Fe::_5,
            6 => Fe::_6,
            7 => Fe::_7,
            8 => Fe::_8,
            9 => Fe::_9,
            x => return Err(Error::InvalidThresholdN(x)),
        };
        // FIXME correct case to match HRP
        ret.push(k.to_char());
        ret.push_str(id);
        ret.push(share_idx.to_char());

        // Convert byte data to base 32
        let mut next_u5 = 0;
        let mut rem = 0;
        for byte in data {
            // Each byte provides at least one u5. Push that.
            let u5 = (next_u5 << (5 - rem)) | byte >> (3 + rem);
            ret.push(Fe::from_u8(u5).unwrap().to_char());
            next_u5 = byte & ((1 << (3 + rem)) - 1);
            // If there were 2 or more bits from the last iteration, then
            // this iteration will push *two* u5s.
            if rem >= 2 {
                ret.push(Fe::from_u8(next_u5 >> (rem - 2)).unwrap().to_char());
                next_u5 &= (1 << (rem - 2)) - 1;
            }
            rem = (rem + 8) % 5;
        }
        if rem > 0 {
            ret.push(Fe::from_u8(next_u5 << (5 - rem)).unwrap().to_char());
        }

        // Initialize checksum engine with HRP and header
        let mut checksum = if data.len() < 51 {
            checksum::Engine::new_codex32_short()
        } else {
            checksum::Engine::new_codex32_long()
        };
        checksum.input_hrp(hrp)?;
        checksum.input_data_str(&ret[hrp.len() + 1..])?;
        // Now, to compute the checksum, we stick the target residue onto the end
        // of the input string, the take the resulting residue as the checksum
        checksum.input_own_target();
        ret.extend(checksum.into_residue().into_iter().map(Fe::to_char));

        let mut checksum = checksum::Engine::new_codex32_short();
        checksum.input_hrp(hrp)?;
        checksum.input_data_str(&ret[hrp.len() + 1..])?;
        Ok(Codex32String(ret))
    }
}

/// A codex32 string, split into its constituent partrs
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Parts<'s> {
    hrp: &'s str,
    threshold: usize,
    id: &'s str,
    share_index: Fe,
    payload: &'s str,
    checksum: &'s str,
}

impl<'s> Parts<'s> {
    /// Extract the binary data from a checksummed string
    ///
    /// If the string does not have a multiple-of-8 number of bits, right-pad the
    /// final byte with 0s.
    pub fn data(&self) -> Vec<u8> {
        let mut ret = Vec::with_capacity((self.payload.len() * 5 + 7) / 8);

        let mut next_byte = 0;
        let mut rem = 0;
        for ch in self.payload.chars() {
            let fe = Fe::from_char(ch).unwrap(); // unwrap ok since string is valid bech32
            match rem.cmp(&3) {
                cmp::Ordering::Less => {
                    // If we are within 3 bits of the start we can fit the whole next char in
                    next_byte |= fe.to_u8() << (3 - rem);
                }
                cmp::Ordering::Equal => {
                    // If we are exactly 3 bits from the start then this char fills in the byte
                    ret.push(next_byte | fe.to_u8());
                    next_byte = 0;
                }
                cmp::Ordering::Greater => {
                    // Otherwise we have to break it in two
                    let overshoot = rem - 3;
                    assert!(overshoot > 0);
                    ret.push(next_byte | (fe.to_u8() >> overshoot));
                    next_byte = fe.to_u8() << (8 - overshoot);
                }
            }
            rem = (rem + 5) % 8;
        }
        debug_assert!(rem <= 4); // checked when parsing the string
        ret
    }
}
