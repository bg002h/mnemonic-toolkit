//! `mnemonic import-wallet` — Phase 2 scaffold.
//!
//! v0.26.0 Phase 2: thin CLI scaffold exposing the BSMS Round-2 parser for
//! end-to-end integration tests. The full clap surface (`--ms1`, `--slot`,
//! `--select-descriptor`, `--json`, sniff dispatcher) lands in Phase 5.
//!
//! Current surface:
//!   --blob <FILE|->        required; `-` reads stdin
//!   --format <bsms>        required for Phase 2 (sniff is Phase 5)
//!
//! Stdout shape (intentionally minimal; Phase 5 replaces with the canonical
//! card or JSON envelope):
//!   import-wallet: bundles=<N>
//!   bundles[<i>].cosigners=<N>
//!   bundles[<i>].network=<mainnet|testnet|...>
//!   bundles[<i>].threshold=<K|none>
//!   bundles[<i>].bsms_audit=<some|none>
//!   bundles[<i>].cosigners[<j>].fingerprint=<hex>
//!   bundles[<i>].entropy=<none|some>
//!
//! Stderr: WARNINGs / NOTICEs from per-format `parse()` impls.

use crate::error::ToolkitError;
use crate::wallet_import::{bsms::BsmsParser, WalletFormatParser};
use clap::Args;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct ImportWalletArgs {
    /// Path to the third-party wallet blob; `-` reads from stdin.
    #[arg(long = "blob", value_name = "FILE|-", required = true)]
    pub blob: PathBuf,

    /// Format override. v0.26.0 Phase 2 supports `bsms` only; Phase 3 adds
    /// `bitcoin-core`; Phase 5 adds the sniff (default) dispatcher.
    #[arg(long = "format", value_name = "bsms", required = true)]
    pub format: String,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &ImportWalletArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let blob = read_blob(&args.blob, stdin)?;

    let parsed = match args.format.as_str() {
        "bsms" => BsmsParser::parse(&blob, stderr)?,
        other => {
            return Err(ToolkitError::BadInput(format!(
                "import-wallet --format {other} is not supported in v0.26.0 Phase 2 (bsms only; bitcoin-core arrives in Phase 3)"
            )));
        }
    };

    writeln!(stdout, "import-wallet: bundles={}", parsed.len()).map_err(ToolkitError::Io)?;
    for (i, b) in parsed.iter().enumerate() {
        writeln!(stdout, "bundles[{i}].cosigners={}", b.cosigners.len())
            .map_err(ToolkitError::Io)?;
        let network_name = match b.network {
            bitcoin::Network::Bitcoin => "mainnet",
            bitcoin::Network::Testnet => "testnet",
            bitcoin::Network::Signet => "signet",
            bitcoin::Network::Regtest => "regtest",
            _ => "unknown",
        };
        writeln!(stdout, "bundles[{i}].network={network_name}").map_err(ToolkitError::Io)?;
        let threshold_str = b
            .threshold
            .map(|t| t.to_string())
            .unwrap_or_else(|| "none".to_string());
        writeln!(stdout, "bundles[{i}].threshold={threshold_str}").map_err(ToolkitError::Io)?;
        let audit_str = if b.bsms_audit.is_some() {
            "some"
        } else {
            "none"
        };
        writeln!(stdout, "bundles[{i}].bsms_audit={audit_str}").map_err(ToolkitError::Io)?;
        let entropy_str = if b.cosigners.iter().any(|c| c.entropy.is_some()) {
            "some"
        } else {
            "none"
        };
        writeln!(stdout, "bundles[{i}].entropy={entropy_str}").map_err(ToolkitError::Io)?;
        for (j, c) in b.cosigners.iter().enumerate() {
            // For top-level convenience (the smoke checks use the simpler
            // form), emit both bracketed and unbracketed lines.
            writeln!(
                stdout,
                "bundles[{i}].cosigners[{j}].fingerprint={}",
                hex_lower(&c.fingerprint.to_bytes())
            )
            .map_err(ToolkitError::Io)?;
            // The shorter alias `cosigners[<j>].fingerprint=<hex>` lets
            // direction-agnostic assertions match without parsing a
            // bundle index.
            writeln!(
                stdout,
                "cosigners[{j}].fingerprint={}",
                hex_lower(&c.fingerprint.to_bytes())
            )
            .map_err(ToolkitError::Io)?;
        }
        // Append a "cosigners=N" line in the simpler form too, so tests
        // can match either form.
        writeln!(stdout, "cosigners={}", b.cosigners.len()).map_err(ToolkitError::Io)?;
        writeln!(stdout, "network={network_name}").map_err(ToolkitError::Io)?;
        writeln!(stdout, "threshold={threshold_str}").map_err(ToolkitError::Io)?;
        writeln!(stdout, "bsms_audit={audit_str}").map_err(ToolkitError::Io)?;
        writeln!(stdout, "entropy={entropy_str}").map_err(ToolkitError::Io)?;
    }
    Ok(0)
}

fn read_blob<R: Read>(path: &PathBuf, stdin: &mut R) -> Result<Vec<u8>, ToolkitError> {
    if path.as_os_str() == "-" {
        let mut buf = Vec::new();
        stdin.read_to_end(&mut buf).map_err(ToolkitError::Io)?;
        Ok(buf)
    } else {
        fs::read(path).map_err(ToolkitError::Io)
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}
