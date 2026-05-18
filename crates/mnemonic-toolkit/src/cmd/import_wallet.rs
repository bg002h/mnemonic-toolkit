//! `mnemonic import-wallet` — Phase 3 scaffold.
//!
//! v0.26.0 Phase 2 added the BSMS Round-2 parser scaffold. Phase 3 extends
//! the dispatch to `--format bitcoin-core` and adds the `--select-descriptor`
//! filter helper. The full clap surface (`--ms1`, `--slot`, `--json`, sniff
//! dispatcher) lands in Phase 5.
//!
//! Current surface:
//!   --blob <FILE|->                                              required; `-` reads stdin
//!   --format <bsms|bitcoin-core>                                 required for Phase 3 (sniff is Phase 5)
//!   --select-descriptor <N|active-receive|active-change|all>     optional; default `all`
//!
//! Stdout shape (intentionally minimal; Phase 5 replaces with the canonical
//! card or JSON envelope):
//!   import-wallet: bundles=<N>
//!   bundles[<i>].cosigners=<N>
//!   bundles[<i>].network=<mainnet|testnet|...>
//!   bundles[<i>].threshold=<K|none>
//!   bundles[<i>].bsms_audit=<some|none>
//!   bundles[<i>].entropy=<none|some>
//!   bundles[<i>].source_metadata=<some|none>
//!   bundles[<i>].cosigners[<j>].fingerprint=<hex>
//!   cosigners[<j>].fingerprint=<hex>          // top-level alias
//!   cosigners=<N>                              // top-level alias
//!   network=<mainnet|testnet|...>             // top-level alias
//!   threshold=<K|none>                         // top-level alias
//!   bsms_audit=<some|none>                     // top-level alias
//!   entropy=<none|some>                        // top-level alias
//!
//! Stderr: WARNINGs / NOTICEs from per-format `parse()` impls.

use crate::error::ToolkitError;
use crate::wallet_import::{
    apply_select_descriptor, bitcoin_core::BitcoinCoreParser, bsms::BsmsParser, ParsedImport,
    SelectDescriptor, WalletFormatParser,
};
use clap::Args;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct ImportWalletArgs {
    /// Path to the third-party wallet blob; `-` reads from stdin.
    #[arg(long = "blob", value_name = "FILE|-", required = true)]
    pub blob: PathBuf,

    /// Format override. v0.26.0 Phase 3 supports `bsms` + `bitcoin-core`;
    /// Phase 5 adds the sniff (default) dispatcher.
    #[arg(long = "format", value_name = "bsms|bitcoin-core", required = true)]
    pub format: String,

    /// Multi-descriptor selector for Bitcoin Core blobs (SPEC §5.3).
    /// Accepts an integer (`0`, `1`, ...), `active-receive`, `active-change`,
    /// or `all` (default). Has no effect on BSMS blobs (which carry a single
    /// descriptor) — Phase 5 emits a NOTICE when a non-default value is
    /// supplied alongside `--format bsms`; Phase 3 silently treats it as
    /// `all` for BSMS.
    #[arg(
        long = "select-descriptor",
        value_name = "N|active-receive|active-change|all",
        default_value = "all"
    )]
    pub select_descriptor: String,
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
        "bitcoin-core" => BitcoinCoreParser::parse(&blob, stderr)?,
        other => {
            return Err(ToolkitError::BadInput(format!(
                "import-wallet --format {other} is not supported in v0.26.0 (bsms + bitcoin-core only)"
            )));
        }
    };

    // SPEC §5.3 — `--select-descriptor` filter. For BSMS, parsed.len() == 1
    // and the only valid filter outcomes are `all` / `0` / a failing
    // active-*-filter (no source_metadata). Phase 5 adds the
    // NOTICE-and-coerce-to-all behavior for BSMS + non-default; Phase 3 just
    // applies the filter as-is so the helper's exit-code routing is testable.
    let select = parse_select(&args.select_descriptor)?;
    let parsed = match args.format.as_str() {
        // Per SPEC §5.3: BSMS coerces any non-default value to `all`.
        "bsms" => match select {
            SelectDescriptor::All => apply_select_descriptor(parsed, SelectDescriptor::All)?,
            _ => parsed,
        },
        _ => apply_select_descriptor(parsed, select)?,
    };

    emit_summary(stdout, &parsed)?;
    Ok(0)
}

fn parse_select(s: &str) -> Result<SelectDescriptor, ToolkitError> {
    match s {
        "all" => Ok(SelectDescriptor::All),
        "active-receive" => Ok(SelectDescriptor::ActiveReceive),
        "active-change" => Ok(SelectDescriptor::ActiveChange),
        other => {
            // Accept bare integer N.
            if let Ok(n) = other.parse::<usize>() {
                return Ok(SelectDescriptor::ByIndex(n));
            }
            Err(ToolkitError::BadInput(format!(
                "--select-descriptor: invalid value `{other}`; expected `N` (integer), `active-receive`, `active-change`, or `all`"
            )))
        }
    }
}

fn emit_summary<W: Write>(stdout: &mut W, parsed: &[ParsedImport]) -> Result<(), ToolkitError> {
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
        let src_meta_str = if b.source_metadata.is_some() {
            "some"
        } else {
            "none"
        };
        writeln!(stdout, "bundles[{i}].source_metadata={src_meta_str}")
            .map_err(ToolkitError::Io)?;
        if let Some(m) = &b.source_metadata {
            writeln!(stdout, "bundles[{i}].active={}", m.active).map_err(ToolkitError::Io)?;
            writeln!(stdout, "bundles[{i}].internal={}", m.internal).map_err(ToolkitError::Io)?;
        }
        for (j, c) in b.cosigners.iter().enumerate() {
            writeln!(
                stdout,
                "bundles[{i}].cosigners[{j}].fingerprint={}",
                hex_lower(&c.fingerprint.to_bytes())
            )
            .map_err(ToolkitError::Io)?;
            writeln!(
                stdout,
                "cosigners[{j}].fingerprint={}",
                hex_lower(&c.fingerprint.to_bytes())
            )
            .map_err(ToolkitError::Io)?;
        }
        writeln!(stdout, "cosigners={}", b.cosigners.len()).map_err(ToolkitError::Io)?;
        writeln!(stdout, "network={network_name}").map_err(ToolkitError::Io)?;
        writeln!(stdout, "threshold={threshold_str}").map_err(ToolkitError::Io)?;
        writeln!(stdout, "bsms_audit={audit_str}").map_err(ToolkitError::Io)?;
        writeln!(stdout, "entropy={entropy_str}").map_err(ToolkitError::Io)?;
    }
    Ok(())
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
