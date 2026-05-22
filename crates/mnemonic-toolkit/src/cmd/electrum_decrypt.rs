//! `mnemonic electrum-decrypt` subcommand (v0.33.0 / Cycle 18).
//!
//! Surfaces the `electrum_crypto::decrypt_field` primitive (shipped Cycle 6a,
//! previously unused-by-CLI) as a standalone subcommand: decrypt an Electrum
//! field-encrypted secret (`base64(iv || aes-cbc(plaintext + PKCS7))`,
//! key = `sha256d(password)`) and emit the recovered plaintext — an
//! Electrum-native seed phrase or a BIP-32 xprv (the wallet's keystore type
//! determines which; the wire carries no discriminator, so the output is
//! emitted opaquely).
//!
//! Architecture: dedicated subcommand (NOT a `convert` source) per the
//! Cycle-18 architecture decision — the decrypted node-type is unknowable
//! before decryption, which `convert`'s commit-types-up-front model cannot
//! express. Mirrors the `cmd/seed_xor.rs` / `cmd/slip39.rs` secret-handling
//! template (Zeroizing + mlock + argv/stdout/world-readable advisories).

use crate::cmd::convert::{read_stdin_passphrase, read_stdin_to_string};
use crate::error::ToolkitError;
use crate::secret_advisory::{
    secret_in_argv_warning, secret_on_stdout_warning_unconditional, warn_if_world_readable,
};
use clap::{ArgGroup, Args};
use mnemonic_toolkit::electrum_crypto::{decrypt_field, ElectrumDecryptError};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Args, Debug)]
#[command(group(
    // Exactly one password source is required (mirrors the repair.rs /
    // inspect.rs struct-level ArgGroup form). `--decrypt-password-stdin` is a
    // bool: a `false` does not count as present in the group.
    ArgGroup::new("decrypt_password_source")
        .args(["decrypt_password", "decrypt_password_file", "decrypt_password_stdin"])
        .required(true)
        .multiple(false),
))]
pub struct ElectrumDecryptArgs {
    /// Electrum field-encrypted secret as base64 (`iv || aes-cbc(...)`).
    /// `-` reads the base64 from stdin. NOT secret (it is ciphertext).
    #[arg(long = "ciphertext", value_name = "VALUE|-")]
    pub ciphertext: String,

    /// Decryption password (inline). Emits an argv-leakage advisory —
    /// prefer `--decrypt-password-file` or `--decrypt-password-stdin`.
    #[arg(long = "decrypt-password", value_name = "VALUE")]
    pub decrypt_password: Option<String>,

    /// Read the decryption password from a file (trailing newline stripped).
    #[arg(long = "decrypt-password-file", value_name = "PATH")]
    pub decrypt_password_file: Option<PathBuf>,

    /// Read the decryption password from stdin (raw, NULL-byte preserving).
    #[arg(long = "decrypt-password-stdin")]
    pub decrypt_password_stdin: bool,

    /// Write a JSON envelope to PATH instead of plain text on stdout.
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<PathBuf>,
}

#[derive(serde::Serialize)]
struct ElectrumDecryptEnvelope<'a> {
    schema_version: &'a str,
    operation: &'a str,
    plaintext: &'a str,
}

/// Map a library-local `ElectrumDecryptError` to a CLI-boundary
/// `ToolkitError`. The two wrong-password / corruption failure modes
/// (`AesDecryptFailure` from PKCS7-unpad refusal, `Utf8DecodeFailure` from
/// a non-UTF-8 result) are UNIFIED into one message so the CLI does not leak
/// which mode occurred.
fn map_electrum_decrypt_error(e: ElectrumDecryptError) -> ToolkitError {
    match e {
        ElectrumDecryptError::AesDecryptFailure | ElectrumDecryptError::Utf8DecodeFailure => {
            ToolkitError::BadInput(
                "electrum-decrypt: decryption failed (wrong password or corrupted ciphertext)"
                    .to_string(),
            )
        }
        other => ToolkitError::BadInput(format!("electrum-decrypt: {other}")),
    }
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &ElectrumDecryptArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // Single-stdin-per-invocation guard: `--ciphertext -` and
    // `--decrypt-password-stdin` both consume stdin.
    let ciphertext_uses_stdin = args.ciphertext == "-";
    if ciphertext_uses_stdin && args.decrypt_password_stdin {
        return Err(ToolkitError::BadInput(
            "--ciphertext=- and --decrypt-password-stdin cannot both read from stdin".to_string(),
        ));
    }

    // Resolve the password (exactly one form per the ArgGroup). Inline form
    // emits the argv-leakage advisory.
    let password: zeroize::Zeroizing<String> = if let Some(pw) = &args.decrypt_password {
        secret_in_argv_warning(stderr, "--decrypt-password ", "--decrypt-password-stdin");
        zeroize::Zeroizing::new(pw.clone())
    } else if let Some(path) = &args.decrypt_password_file {
        let raw = std::fs::read_to_string(path).map_err(|e| {
            ToolkitError::BadInput(format!(
                "--decrypt-password-file: cannot read {}: {e}",
                path.display()
            ))
        })?;
        // Strip a single trailing newline (mirrors token/file conventions).
        zeroize::Zeroizing::new(raw.strip_suffix('\n').unwrap_or(&raw).to_string())
    } else {
        // `--decrypt-password-stdin` (ArgGroup guarantees this is the
        // remaining case). NULL-byte-preserving stdin read.
        zeroize::Zeroizing::new(read_stdin_passphrase(stdin)?)
    };
    let _pin_pw = mnemonic_toolkit::mlock::pin_pages_for(password.as_bytes());

    // Resolve the ciphertext (inline or stdin). Not secret → no advisory.
    let ciphertext: String = if ciphertext_uses_stdin {
        read_stdin_to_string(stdin)?
    } else {
        args.ciphertext.clone()
    };

    // Decrypt via the library primitive.
    let plaintext = decrypt_field(ciphertext.trim(), password.as_bytes())
        .map_err(map_electrum_decrypt_error)?;
    let _pin_pt = mnemonic_toolkit::mlock::pin_pages_for(plaintext.as_bytes());

    if let Some(path) = &args.json_out {
        let envelope = ElectrumDecryptEnvelope {
            schema_version: "1",
            operation: "electrum-decrypt",
            plaintext: plaintext.as_str(),
        };
        let json = serde_json::to_string_pretty(&envelope).map_err(|e| {
            ToolkitError::BadInput(format!("electrum-decrypt: json serialize: {e}"))
        })?;
        std::fs::write(path, json).map_err(|e| {
            ToolkitError::BadInput(format!("electrum-decrypt: json-out write to {path:?}: {e}"))
        })?;
        warn_if_world_readable(path, stderr);
    } else {
        writeln!(stdout, "{}", plaintext.as_str())
            .map_err(|e| ToolkitError::BadInput(format!("electrum-decrypt: stdout write: {e}")))?;
        secret_on_stdout_warning_unconditional(stderr);
    }
    Ok(0)
}
