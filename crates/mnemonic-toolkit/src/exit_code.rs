//! The constellation exit codes (`SPEC_constellation_exit_codes.md` §3, in
//! mnemonic-engrave's `design/`).
//!
//! **A local copy.** It mirrors the `exit` module of mnemonic-io-lib (the
//! spec's decision 4: the same constant names, `Bits` with `|`, `contains`,
//! `is_ok` and `to_exit_code`, and the R11 legend) until that crate is
//! published with it; then `mnemonic` takes the shared one and this file goes.
//! The numbers are pinned to the spec by the tests below.
//!
//! Codes 1 to 63 are sums of the six bits, and each sum is unique. 64, 101 and
//! 130 never combine. Which failure is which class stays the toolkit's own
//! decision: `ToolkitError::exit_code` makes it.

// A mirror of the shared module: not every item has a user in `mnemonic`.
#![allow(dead_code)]

use std::ops::{BitOr, BitOrAssign};

/// 0: everything requested was done and every check passed.
pub const OK: u8 = 0;
/// Bit 1: a well-formed input was compared against something and came out no.
pub const NO: u8 = 1;
/// Bit 2: an input is wrong for this verb whatever the flags.
pub const INVALID: u8 = 2;
/// Bit 4: the tool checked part of the result and could not confirm it.
pub const UNVERIFIED: u8 = 4;
/// Bit 8: refused under a safety or policy rule that a flag, network, channel
/// or different destination would waive; also a network mismatch.
pub const REFUSED: u8 = 8;
/// Bit 16: a checksum correction was applied and the corrected value used.
pub const REPAIRED: u8 = 16;
/// Bit 32: something needed from the system failed or is missing.
pub const IO: u8 = 32;
/// 64, alone: the command line alone is wrong. Nothing ran.
pub const USAGE: u8 = 64;
/// 101, alone: a bug, or any failure no class covers (Rust's panic code).
pub const INTERNAL: u8 = 101;
/// 130, alone: the operator stopped it, at a prompt or with Ctrl-C.
pub const STOPPED: u8 = 130;

/// The root `--help` legend every binary ends with (spec R11), word for word.
pub const HELP_LEGEND: &str = "\
Exit status: 0 = done, every check passed. 64 = bad command line, nothing
ran. 101 = internal error. 130 = stopped by the operator. Other codes from
128 up = killed by a signal. Otherwise the sum (1 to 63) of: 1 a check said
no; 2 an input is invalid; 4 verify the result before trusting it; 8 refused
(network mismatch, secret on the command line, unsafe output); 16 a
correction was applied; 32 a file, device or helper failed.";

/// A set of the six combinable bits (1, 2, 4, 8, 16, 32). Its exit code is
/// their sum, so every combination is a unique code from 0 to 63.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Bits(u8);

impl Bits {
    /// No bit set: exit 0.
    pub const OK: Bits = Bits(OK);
    pub const NO: Bits = Bits(NO);
    pub const INVALID: Bits = Bits(INVALID);
    pub const UNVERIFIED: Bits = Bits(UNVERIFIED);
    pub const REFUSED: Bits = Bits(REFUSED);
    pub const REPAIRED: Bits = Bits(REPAIRED);
    pub const IO: Bits = Bits(IO);

    /// True when every bit of `other` is set here.
    pub const fn contains(self, other: Bits) -> bool {
        self.0 & other.0 == other.0
    }

    /// True when no bit is set (exit 0).
    pub const fn is_ok(self) -> bool {
        self.0 == 0
    }

    /// The process exit status: the sum of the bits, 0 to 63.
    pub const fn to_exit_code(self) -> u8 {
        self.0
    }
}

impl BitOr for Bits {
    type Output = Bits;
    fn bitor(self, rhs: Bits) -> Bits {
        Bits(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bits {
    fn bitor_assign(&mut self, rhs: Bits) {
        self.0 |= rhs.0;
    }
}

/// R8: a stdout write failure ADDS 32 to a combinable code (0 to 63) and does
/// not replace it. 64, 101 and 130 never combine and are kept.
pub fn with_io_failure(code: u8) -> u8 {
    if code < USAGE {
        code | IO
    } else {
        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The numbers are the spec's (§3); a change here is a constellation change.
    #[test]
    fn numbers_match_the_spec() {
        assert_eq!(
            [OK, NO, INVALID, UNVERIFIED, REFUSED, REPAIRED, IO, USAGE, INTERNAL, STOPPED],
            [0, 1, 2, 4, 8, 16, 32, 64, 101, 130]
        );
        let all = [NO, INVALID, UNVERIFIED, REFUSED, REPAIRED, IO];
        assert_eq!(all.iter().fold(0u8, |a, b| a | b), 63);
        assert_eq!(all.iter().map(|b| b.count_ones()).sum::<u32>(), 6);
    }

    #[test]
    fn bits_sum_and_contain() {
        let b = Bits::REPAIRED | Bits::UNVERIFIED;
        assert_eq!(b.to_exit_code(), 20);
        assert!(b.contains(Bits::REPAIRED));
        assert!(b.contains(Bits::UNVERIFIED));
        assert!(!b.contains(Bits::INVALID));
        let mut c = b;
        c |= Bits::INVALID;
        assert_eq!(c.to_exit_code(), 22);
        assert!(Bits::OK.is_ok());
        assert_eq!(Bits::default(), Bits::OK);
    }

    #[test]
    fn io_failure_adds_and_never_combines_with_usage_or_internal() {
        assert_eq!(with_io_failure(OK), 32);
        assert_eq!(with_io_failure(20), 52);
        assert_eq!(with_io_failure(IO), 32);
        assert_eq!(with_io_failure(USAGE), 64);
        assert_eq!(with_io_failure(INTERNAL), 101);
    }

    #[test]
    fn legend_is_the_spec_text() {
        let flat = HELP_LEGEND.split_whitespace().collect::<Vec<_>>().join(" ");
        assert_eq!(
            flat,
            "Exit status: 0 = done, every check passed. 64 = bad command line, nothing ran. \
             101 = internal error. 130 = stopped by the operator. Other codes from 128 up = \
             killed by a signal. Otherwise the sum (1 to 63) of: 1 a check said no; 2 an input \
             is invalid; 4 verify the result before trusting it; 8 refused (network mismatch, \
             secret on the command line, unsafe output); 16 a correction was applied; 32 a \
             file, device or helper failed."
        );
    }
}
