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
    /// DEPRECATED (v0.31.6): use `--from seedqr=<VALUE|->` instead.
    /// SeedQR numeric digit string (48/60/72/84/96 ASCII digits). `-` reads
    /// from stdin. Emits a stderr deprecation warning when used; will be
    /// removed in a future release.
    #[arg(long = "digits", value_name = "VALUE|-", conflicts_with = "from")]
    pub digits: Option<String>,

    /// Canonical input form (v0.31.6): `--from seedqr=<VALUE|->`. Only the
    /// `seedqr` node type is accepted on `seedqr decode`. `-` reads from stdin.
    #[arg(long = "from", value_name = "seedqr=<VALUE|->", value_parser = parse_from_input)]
    pub from: Option<FromInput>,

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
    // v0.31.6 — resolve the input source. Clap `conflicts_with` already
    // guarantees `--digits` and `--from` are not BOTH set; here we handle
    // the (a) `--digits` deprecated path, (b) `--from seedqr=` canonical
    // path, (c) neither-supplied required-input refusal.
    let raw_value: String = match (&args.digits, &args.from) {
        (Some(_), Some(_)) => unreachable!("clap conflicts_with prevents --digits + --from"),
        (None, None) => {
            return Err(ToolkitError::BadInput(
                "seedqr decode requires an input: --from seedqr=<VALUE|-> (canonical) or --digits <VALUE|-> (deprecated)".into(),
            ));
        }
        (Some(d), None) => {
            // Deprecated `--digits` path. Emit a stderr deprecation notice.
            let _ = writeln!(
                stderr,
                "notice: --digits is deprecated; use --from seedqr=<VALUE|-> instead (--digits will be removed in a future release)"
            );
            if d != "-" {
                secret_in_argv_warning(stderr, "--digits ", "--digits -");
            }
            d.clone()
        }
        (None, Some(fi)) => {
            // Canonical `--from seedqr=` path. Reject non-seedqr node types.
            if fi.node != NodeType::Seedqr {
                return Err(ToolkitError::BadInput(format!(
                    "seedqr decode --from accepts only the `seedqr` node type; got `{}`",
                    fi.node.as_str()
                )));
            }
            if fi.value != "-" {
                secret_in_argv_warning(stderr, "--from seedqr=", "--from seedqr=-");
            }
            fi.value.clone()
        }
    };

    // Resolve value (inline or stdin); wrap in Zeroizing.
    let digits: zeroize::Zeroizing<String> = if raw_value == "-" {
        zeroize::Zeroizing::new(read_stdin_to_string(stdin)?)
    } else {
        zeroize::Zeroizing::new(raw_value)
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
