//! `mnemonic verify-message` — VERIFY-ONLY (no signing) Bitcoin message
//! signature verification. legacy "Bitcoin Signed Message" (P2PKH) + BIP-322
//! simple (P2WPKH/P2SH-P2WPKH/P2TR). PUBLIC operation — no secrets.

use crate::error::ToolkitError;
use crate::verify_message::{verify_message, SigFormat};
use clap::{ArgGroup, Args, ValueEnum};
use std::io::{Read, Write};

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum VerifyFormat {
    /// Auto-detect: P2PKH → legacy; segwit/taproot → BIP-322.
    Auto,
    /// legacy "Bitcoin Signed Message" (P2PKH only).
    Legacy,
    /// BIP-322 simple (P2WPKH / P2SH-P2WPKH / P2TR).
    Bip322,
}

impl From<VerifyFormat> for SigFormat {
    fn from(f: VerifyFormat) -> Self {
        match f {
            VerifyFormat::Auto => SigFormat::Auto,
            VerifyFormat::Legacy => SigFormat::Legacy,
            VerifyFormat::Bip322 => SigFormat::Bip322,
        }
    }
}

#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("message_src")
        .required(true)
        .multiple(false)
        .args(["message", "message_file", "message_stdin"]),
))]
pub struct VerifyMessageArgs {
    /// The address the message was signed by.
    #[arg(long)]
    pub address: String,

    /// The signed message (inline, exact bytes). One of --message /
    /// --message-file / --message-stdin is required.
    #[arg(long)]
    pub message: Option<String>,

    /// Read the signed message from a file (a single trailing newline is
    /// stripped — pipes/editors commonly append one).
    #[arg(long = "message-file")]
    pub message_file: Option<std::path::PathBuf>,

    /// Read the signed message from stdin (a single trailing newline is stripped).
    #[arg(long = "message-stdin")]
    pub message_stdin: bool,

    /// The signature (base64): a 65-byte recoverable sig (legacy) or a BIP-322
    /// witness-stack encoding.
    #[arg(long)]
    pub signature: String,

    /// Signature format. `auto` picks legacy for P2PKH, BIP-322 otherwise.
    #[arg(long, value_enum, default_value_t = VerifyFormat::Auto)]
    pub format: VerifyFormat,

    /// Emit JSON instead of the human-readable line.
    #[arg(long)]
    pub json: bool,
}

#[derive(serde::Serialize)]
struct VerifyMessageJson {
    address: String,
    format_requested: String,
    format_matched: String,
    valid: bool,
}

/// Strip a single trailing newline (`\n` or `\r\n`) — common when a message is
/// piped or read from a file. Inline `--message` is left exact.
fn strip_one_trailing_newline(s: &str) -> &str {
    if let Some(stripped) = s.strip_suffix("\r\n") {
        stripped
    } else if let Some(stripped) = s.strip_suffix('\n') {
        stripped
    } else {
        s
    }
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &VerifyMessageArgs,
    stdin: &mut R,
    stdout: &mut W,
    _stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let message: String = if let Some(m) = &args.message {
        m.clone()
    } else if let Some(path) = &args.message_file {
        let raw = std::fs::read_to_string(path).map_err(ToolkitError::Io)?;
        strip_one_trailing_newline(&raw).to_string()
    } else if args.message_stdin {
        let mut buf = String::new();
        stdin.read_to_string(&mut buf).map_err(ToolkitError::Io)?;
        strip_one_trailing_newline(&buf).to_string()
    } else {
        return Err(ToolkitError::VerifyMessage(
            "exactly one of --message / --message-file / --message-stdin is required".into(),
        ));
    };

    let outcome = verify_message(&args.address, &message, &args.signature, args.format.into())?;
    let requested = format!("{:?}", args.format).to_lowercase();

    if args.json {
        let envelope = VerifyMessageJson {
            address: args.address.trim().to_string(),
            format_requested: requested,
            format_matched: outcome.format_matched.to_string(),
            valid: outcome.valid,
        };
        serde_json::to_writer_pretty(&mut *stdout, &envelope)
            .map_err(|e| ToolkitError::VerifyMessage(format!("json serialize: {e}")))?;
        writeln!(stdout).map_err(ToolkitError::Io)?;
    } else {
        writeln!(
            stdout,
            "{}  (format: {})",
            if outcome.valid { "VALID" } else { "INVALID" },
            outcome.format_matched,
        )
        .map_err(ToolkitError::Io)?;
    }

    // Disposition (d): malformed input already returned Err (exit 1, stderr).
    // A cleanly-decoded signature that simply does not verify → exit 1 with the
    // structured result on stdout (no error).
    Ok(if outcome.valid { 0 } else { 1 })
}
