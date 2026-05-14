//! `mnemonic slip39` subcommand — SLIP-39 K-of-N Shamir backup splitter.
//!
//! Realizes `design/SPEC_slip39_v0_13_0.md` §2.2. Two sub-subcommands:
//!   - `split`: master secret (BIP-39 phrase or hex entropy) → N SLIP-39
//!     shares organized in 1..=16 groups.
//!   - `combine`: ≥K SLIP-39 shares → master secret (BIP-39 phrase or hex
//!     entropy), per the share-set's recorded group/member thresholds.
//!
//! v0.13.0 P2.1 ships the Args / Subcommand surface as a stub: clap
//! enumerates the SPEC §2.2 flag tables in `--help`, but `run_split` /
//! `run_combine` return `ToolkitError::BadInput` with a stub stem (exit 1).
//! Full handler impl + 5 cli_slip39_*.rs test files + lint anchor rows +
//! 6 SPEC patches land at P2.2 GREEN.
//!
//! Cycle A/B discipline rails to be wired at P2.2 GREEN (per
//! `design/PLAN_v0_13_0_p2.md` §3.3 + §3.5):
//!   - 5 argv-leakage advisory call sites (split: `--from phrase=`,
//!     `--from entropy=`, `--passphrase`; combine: `--share`,
//!     `--passphrase`)
//!   - `Zeroizing<String>` wraps on parsed `--from`, `--share`,
//!     `--passphrase`
//!   - mlock `pin_pages_for` Site 1 pins on parsed-input heap buffers
//!   - K-of-N stdout-on-TTY parameterized advisory (extends v0.12.0
//!     seed-xor TTY advisory shape)
//!   - shared `secret_advisory::warn_if_world_readable` for `--json-out`
//!     world-readable-path advisory (extracted from
//!     `cmd/seed_xor.rs::emit_world_readable_advisory` per R0 Q5)
//!   - G9 iteration-exponent threshold advisory (E >= 5)
//!   - env-var determinism wedge: `MNEMONIC_SLIP39_TEST_RNG` (32-byte
//!     hex) + `MNEMONIC_SLIP39_TEST_IDENTIFIER` (decimal u16); always-on
//!     insecurity advisory when either is set

use crate::cmd::convert::{parse_from_input, FromInput};
use crate::error::ToolkitError;
use crate::language::CliLanguage;
use clap::{Args, Subcommand, ValueEnum};
use std::io::{Read, Write};

#[derive(Args, Debug)]
pub struct Slip39Args {
    #[command(subcommand)]
    pub command: Slip39Command,
}

#[derive(Subcommand, Debug)]
pub enum Slip39Command {
    /// Split a master secret into SLIP-39 shares (1..=16 groups × 1..=16 members).
    Split(Slip39SplitArgs),
    /// Combine ≥K SLIP-39 shares back into the master secret.
    Combine(Slip39CombineArgs),
}

#[derive(Args, Debug)]
pub struct Slip39SplitArgs {
    /// Master secret as `phrase=<value-or->` OR `entropy=<hex-or->`.
    ///
    /// Inline forms emit an argv-leakage advisory (`/proc/$PID/cmdline`
    /// exposure); prefer the `=-` (stdin) variant for sensitive input.
    #[arg(
        long = "from",
        value_name = "phrase=<value-or--> or entropy=<hex-or-->",
        value_parser = parse_from_input,
        required = true,
    )]
    pub from: FromInput,

    /// SLIP-39 passphrase (NOT BIP-39 passphrase).
    ///
    /// Inline value emits an argv-leakage advisory; prefer
    /// `--passphrase-stdin` for sensitive passphrases. The argv-leakage
    /// advisory at P2.2 GREEN fires iff this field is `Some(_)`
    /// (user supplied the flag), regardless of value — so empty
    /// passphrases (`--passphrase ""`) still trigger the advisory.
    #[arg(long = "passphrase", conflicts_with = "passphrase_stdin")]
    pub passphrase: Option<String>,

    /// Read passphrase from stdin (single-stdin-per-invocation;
    /// `conflicts_with = "passphrase"` enforced via clap).
    #[arg(long = "passphrase-stdin", default_value_t = false)]
    pub passphrase_stdin: bool,

    /// Groups required to reconstruct (1 <= group-threshold <= group_count).
    #[arg(long = "group-threshold", required = true)]
    pub group_threshold: u8,

    /// Group spec: repeating; `<member_count>,<member_threshold>` per
    /// `--group`. The flag's position in argv is the SLIP-39 `group_idx`
    /// returned in `BadGroupSpec` refusals.
    #[arg(
        long = "group",
        value_name = "N,T",
        required = true,
        action = clap::ArgAction::Append,
        value_parser = parse_group_spec,
    )]
    pub group: Vec<(u8, u8)>,

    /// PBKDF2 cost exponent; 0..=15; iterations = 10000 · 2^E. Trezor's
    /// reference uses E=1 (20000 iterations); E >= 5 emits a performance
    /// advisory.
    #[arg(long = "iteration-exponent", default_value_t = 0)]
    pub iteration_exponent: u8,

    /// BIP-39 language of input phrase; ignored for `entropy=` inputs.
    #[arg(long = "language", default_value = "english")]
    pub language: CliLanguage,

    /// Side-effect: write versioned JSON envelope to PATH (in addition
    /// to plain-stdout shares).
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,
}

#[derive(Args, Debug)]
pub struct Slip39CombineArgs {
    /// SLIP-39 share mnemonic. Repeating; at most ONE may be `-` (stdin).
    ///
    /// Inline values emit a per-occurrence argv-leakage advisory at
    /// P2.2 GREEN; prefer `--share -` (stdin) for sensitive shares.
    #[arg(
        long = "share",
        value_name = "<slip39-mnemonic-or->",
        required = true,
        action = clap::ArgAction::Append,
    )]
    pub share: Vec<String>,

    /// SLIP-39 passphrase used at split time. Same shape constraints as
    /// the split flag (Option + conflicts_with).
    #[arg(long = "passphrase", conflicts_with = "passphrase_stdin")]
    pub passphrase: Option<String>,

    /// Read passphrase from stdin (incompatible with any `--share -`
    /// AND with `--passphrase`).
    #[arg(long = "passphrase-stdin", default_value_t = false)]
    pub passphrase_stdin: bool,

    /// Output shape: `entropy` (default; hex on stdout) or `phrase`
    /// (BIP-39 mnemonic).
    #[arg(long = "to", default_value = "entropy")]
    pub to: Slip39ToShape,

    /// BIP-39 language for `--to phrase`; ignored for `--to entropy`.
    #[arg(long = "language", default_value = "english")]
    pub language: CliLanguage,

    /// Side-effect: write versioned JSON envelope to PATH (in addition
    /// to plain-stdout secret).
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,
}

/// `--to` output shape selector. SPEC §2.2 combine flag table.
#[derive(Clone, Copy, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum Slip39ToShape {
    /// Hex-encoded raw master secret bytes (default).
    Entropy,
    /// BIP-39 mnemonic, language per `--language`.
    Phrase,
}

/// `--group N,T` value parser. `N` is `member_count`, `T` is
/// `member_threshold`. Both 1..=16 per SLIP-0039.
///
/// P2.1 STUB: minimum parsing to satisfy clap's value-parser invocation
/// path so the stub `run_split` is reached with a populated `group` Vec
/// when the user provides minimal args. Range validation (T <= N <= 16)
/// happens at the library boundary in `slip39_split` and surfaces via
/// the `BadGroupSpec` variant, mapped to SPEC §2.5 rows 4-5 at P2.2
/// GREEN.
pub fn parse_group_spec(s: &str) -> Result<(u8, u8), String> {
    let (n, t) = s
        .split_once(',')
        .ok_or_else(|| format!("expected `<member_count>,<member_threshold>`; got `{s}`"))?;
    let n: u8 = n
        .parse()
        .map_err(|e| format!("member_count: {e} (`{n}` is not a valid 0..=255)"))?;
    let t: u8 = t
        .parse()
        .map_err(|e| format!("member_threshold: {e} (`{t}` is not a valid 0..=255)"))?;
    Ok((n, t))
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &Slip39Args,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    match &args.command {
        Slip39Command::Split(a) => run_split(a, stdin, stdout, stderr),
        Slip39Command::Combine(a) => run_combine(a, stdin, stdout, stderr),
    }
}

fn run_split<R: Read, W: Write, E: Write>(
    _args: &Slip39SplitArgs,
    _stdin: &mut R,
    _stdout: &mut W,
    _stderr: &mut E,
) -> Result<u8, ToolkitError> {
    Err(ToolkitError::BadInput(
        "slip39 split: P2.1 stub — full impl ships at P2.2".into(),
    ))
}

fn run_combine<R: Read, W: Write, E: Write>(
    _args: &Slip39CombineArgs,
    _stdin: &mut R,
    _stdout: &mut W,
    _stderr: &mut E,
) -> Result<u8, ToolkitError> {
    Err(ToolkitError::BadInput(
        "slip39 combine: P2.1 stub — full impl ships at P2.2".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_group_spec_accepts_canonical_shape() {
        assert_eq!(parse_group_spec("3,2").unwrap(), (3, 2));
        assert_eq!(parse_group_spec("16,16").unwrap(), (16, 16));
        assert_eq!(parse_group_spec("1,1").unwrap(), (1, 1));
    }

    #[test]
    fn parse_group_spec_rejects_missing_comma() {
        let err = parse_group_spec("32").unwrap_err();
        assert!(err.contains("member_count"), "got: {err}");
    }

    #[test]
    fn parse_group_spec_rejects_non_numeric() {
        assert!(parse_group_spec("a,2").is_err());
        assert!(parse_group_spec("2,b").is_err());
    }
}
