//! `mnemonic decode-address <ADDRESS>` — decode a Bitcoin address string into
//! its network(s), script type, witness version, validity, and scriptPubKey.
//! PUBLIC-DATA utility: no secrets, no key material.

use crate::decode_address::decode_address;
use crate::error::ToolkitError;
use clap::Args;
use std::io::{Read, Write};

#[derive(Args, Debug)]
pub struct DecodeAddressArgs {
    /// The Bitcoin address to decode (mainnet/testnet/testnet4/signet/regtest;
    /// P2PKH / P2SH / P2WPKH / P2WSH / P2TR).
    pub address: String,

    /// Emit JSON instead of the human-readable block.
    #[arg(long)]
    pub json: bool,
}

#[derive(serde::Serialize)]
struct DecodeAddressJson {
    address: String,
    valid: bool,
    networks: Vec<String>,
    script_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    witness_version: Option<u8>,
    script_pubkey: String,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &DecodeAddressArgs,
    _stdin: &mut R,
    stdout: &mut W,
    _stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let d = decode_address(&args.address)?;

    if args.json {
        let envelope = DecodeAddressJson {
            address: d.address_normalized.clone(),
            valid: true,
            networks: d.networks.iter().map(|s| s.to_string()).collect(),
            script_type: d.script_type.clone(),
            witness_version: d.witness_version,
            script_pubkey: d.script_pubkey_hex.clone(),
        };
        serde_json::to_writer_pretty(&mut *stdout, &envelope)
            .map_err(|e| ToolkitError::DecodeAddress(format!("json serialize: {e}")))?;
        writeln!(stdout).map_err(ToolkitError::Io)?;
    } else {
        writeln!(stdout, "address:         {}", d.address_normalized).map_err(ToolkitError::Io)?;
        writeln!(stdout, "  networks:      {}", d.networks.join(", ")).map_err(ToolkitError::Io)?;
        writeln!(stdout, "  script_type:   {}", d.script_type).map_err(ToolkitError::Io)?;
        match d.witness_version {
            Some(v) => writeln!(stdout, "  witness_ver:   {v}").map_err(ToolkitError::Io)?,
            None => writeln!(stdout, "  witness_ver:   (none; legacy)").map_err(ToolkitError::Io)?,
        }
        writeln!(stdout, "  script_pubkey: {}", d.script_pubkey_hex).map_err(ToolkitError::Io)?;
    }
    Ok(0)
}
