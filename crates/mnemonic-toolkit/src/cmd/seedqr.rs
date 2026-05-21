//! `mnemonic seedqr` subcommand (v0.30.0 / Cycle 5).
//!
//! Wraps the `seedqr` library module's `decode` / `encode` primitives
//! in a clap-derive CLI surface. Library-local `SeedqrError` is mapped
//! to `ToolkitError::BadInput` at the boundary via `map_seedqr_error`,
//! mirroring `cmd/seed_xor.rs` / `cmd/slip39.rs` / `cmd/final_word.rs`
//! per `lib.rs:14-28` documented pattern.

use crate::cmd::convert::{parse_from_input, read_stdin_to_string, FromInput, NodeType};
use crate::error::ToolkitError;
use crate::secret_advisory::secret_in_argv_warning;
use clap::{Args, Subcommand};
use mnemonic_toolkit::seedqr::{decode as seedqr_decode, encode as seedqr_encode, SeedqrError};
use std::io::{Read, Write};

#[derive(Args, Debug)]
pub struct SeedqrArgs {
    #[command(subcommand)]
    pub action: SeedqrAction,
}

#[derive(Subcommand, Debug)]
pub enum SeedqrAction {
    /// decode a SeedQR numeric string into a BIP-39 phrase
    Decode(SeedqrDecodeArgs),
    /// encode a BIP-39 phrase into a SeedQR numeric string
    Encode(SeedqrEncodeArgs),
}

#[derive(Args, Debug, Clone)]
pub struct SeedqrDecodeArgs {
    /// SeedQR numeric digit string (48 or 96 ASCII digits). `-` reads from stdin.
    #[arg(long = "digits", value_name = "VALUE|-")]
    pub digits: String,

    /// Write JSON envelope to PATH (stdout empty when set).
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct SeedqrEncodeArgs {
    /// Phrase input as `phrase=<value>` (inline) or `phrase=-` (stdin).
    #[arg(
        long = "from",
        value_name = "phrase=VALUE|-",
        value_parser = parse_from_input,
        required = true,
    )]
    pub from: FromInput,

    /// Write JSON envelope to PATH (stdout empty when set).
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,
}

/// Maps a library-local `SeedqrError` to a CLI-boundary `ToolkitError`.
/// `pub(crate)` since v0.31.3 so the `--slot @N.seedqr=` consumer
/// branches in `cmd/bundle.rs`, `cmd/verify_bundle.rs`, and
/// `cmd/export_wallet.rs` can reuse the canonical mapping (avoids
/// error-text drift across three call sites).
pub(crate) fn map_seedqr_error(e: SeedqrError, action: &str) -> ToolkitError {
    ToolkitError::BadInput(format!("seedqr: {action}: {e}"))
}

/// JSON envelope (mirrors XpubSearchEnvelope / InspectEnvelope /
/// RepairJson precedent: schema_version first; operation discriminator second).
#[derive(serde::Serialize)]
struct SeedqrEnvelope<'a> {
    schema_version: &'a str,
    operation: &'a str,
    variant: &'a str,
    word_count: usize,
    phrase: &'a str,
    digits: &'a str,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &SeedqrArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    match &args.action {
        SeedqrAction::Decode(a) => run_decode(a, stdin, stdout, stderr),
        SeedqrAction::Encode(a) => run_encode(a, stdin, stdout, stderr),
    }
}

fn run_decode<R: Read, W: Write, E: Write>(
    args: &SeedqrDecodeArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // Argv-leakage advisory for inline form.
    if args.digits != "-" {
        secret_in_argv_warning(stderr, "--digits ", "--digits -");
    }

    // Resolve --digits value (inline or stdin); wrap in Zeroizing.
    let digits: zeroize::Zeroizing<String> = if args.digits == "-" {
        zeroize::Zeroizing::new(read_stdin_to_string(stdin)?)
    } else {
        zeroize::Zeroizing::new(args.digits.clone())
    };
    let _pin_digits = mnemonic_toolkit::mlock::pin_pages_for(digits.as_bytes());

    // Decode via library primitive.
    let phrase_plain = seedqr_decode(digits.as_str()).map_err(|e| map_seedqr_error(e, "decode"))?;
    let phrase: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(phrase_plain);
    let _pin_phrase = mnemonic_toolkit::mlock::pin_pages_for(phrase.as_bytes());

    // Canonical 48/96-digit form for JSON envelope echo.
    let canonical_digits: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(
        digits
            .chars()
            .filter(|c| !c.is_ascii_whitespace())
            .collect(),
    );
    let word_count = phrase.split_whitespace().count();

    emit_decode_output(
        args,
        phrase.as_str(),
        canonical_digits.as_str(),
        word_count,
        stdout,
    )
}

fn run_encode<R: Read, W: Write, E: Write>(
    args: &SeedqrEncodeArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // Validate that --from carries a phrase= node (NOT xpub=, ms1=, etc.).
    // Mirrors cmd/seed_xor.rs:163-167.
    if args.from.node != NodeType::Phrase {
        return Err(ToolkitError::BadInput(
            "seedqr encode only accepts phrase=<value> or phrase=-".into(),
        ));
    }

    // Argv-leakage advisory for inline form.
    if args.from.value != "-" {
        secret_in_argv_warning(stderr, "--from phrase=", "--from phrase=-");
    }

    // Resolve phrase input (inline or stdin); wrap in Zeroizing.
    let phrase: zeroize::Zeroizing<String> = if args.from.value == "-" {
        zeroize::Zeroizing::new(read_stdin_to_string(stdin)?)
    } else {
        zeroize::Zeroizing::new(args.from.value.clone())
    };
    let _pin_phrase = mnemonic_toolkit::mlock::pin_pages_for(phrase.as_bytes());

    // Encode via library primitive.
    let digits_plain = seedqr_encode(phrase.as_str()).map_err(|e| map_seedqr_error(e, "encode"))?;
    let digits: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(digits_plain);
    let _pin_digits = mnemonic_toolkit::mlock::pin_pages_for(digits.as_bytes());

    let canonical_phrase: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(
        phrase
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join(" "),
    );
    let word_count = canonical_phrase.split_whitespace().count();

    emit_encode_output(
        args,
        canonical_phrase.as_str(),
        digits.as_str(),
        word_count,
        stdout,
    )
}

fn emit_decode_output<W: Write>(
    args: &SeedqrDecodeArgs,
    phrase: &str,
    digits: &str,
    word_count: usize,
    stdout: &mut W,
) -> Result<u8, ToolkitError> {
    if let Some(path) = &args.json_out {
        let envelope = SeedqrEnvelope {
            schema_version: "1",
            operation: "decode",
            variant: "standard",
            word_count,
            phrase,
            digits,
        };
        let json = serde_json::to_string_pretty(&envelope)
            .map_err(|e| ToolkitError::BadInput(format!("seedqr: decode: json serialize: {e}")))?;
        std::fs::write(path, json).map_err(|e| {
            ToolkitError::BadInput(format!("seedqr: decode: json-out write to {path:?}: {e}"))
        })?;
    } else {
        writeln!(stdout, "{phrase}")
            .map_err(|e| ToolkitError::BadInput(format!("seedqr: decode: stdout write: {e}")))?;
    }
    Ok(0)
}

fn emit_encode_output<W: Write>(
    args: &SeedqrEncodeArgs,
    phrase: &str,
    digits: &str,
    word_count: usize,
    stdout: &mut W,
) -> Result<u8, ToolkitError> {
    if let Some(path) = &args.json_out {
        let envelope = SeedqrEnvelope {
            schema_version: "1",
            operation: "encode",
            variant: "standard",
            word_count,
            phrase,
            digits,
        };
        let json = serde_json::to_string_pretty(&envelope)
            .map_err(|e| ToolkitError::BadInput(format!("seedqr: encode: json serialize: {e}")))?;
        std::fs::write(path, json).map_err(|e| {
            ToolkitError::BadInput(format!("seedqr: encode: json-out write to {path:?}: {e}"))
        })?;
    } else {
        writeln!(stdout, "{digits}")
            .map_err(|e| ToolkitError::BadInput(format!("seedqr: encode: stdout write: {e}")))?;
    }
    Ok(0)
}
