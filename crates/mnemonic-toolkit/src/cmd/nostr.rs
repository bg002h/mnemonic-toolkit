//! `mnemonic nostr` — wrap an existing nostr key (`npub`/`nsec`) as Bitcoin
//! addresses, descriptors, and (for `nsec`) a WIF. See
//! `design/BRAINSTORM_v0_34_0_nostr_key_wrappers.md`.

use crate::cmd::convert::ScriptType;
use crate::error::ToolkitError;
use crate::network::CliNetwork;
use clap::Args;
use std::io::{Read, Write};

#[derive(Args, Debug)]
pub struct NostrArgs {
    /// Public key: `npub1…` (NIP-19) or 64-hex x-only. Watch-only outputs.
    #[arg(long, group = "key")]
    pub pubkey: Option<String>,

    /// Secret key: `nsec1…` (NIP-19) or 64-hex scalar. Adds WIF. SECRET — leaks via argv.
    #[arg(long, group = "key")]
    pub secret: Option<String>,

    /// Read the secret key from a file (avoids argv exposure).
    #[arg(long = "secret-file", group = "key")]
    pub secret_file: Option<std::path::PathBuf>,

    /// Read the secret key from stdin.
    #[arg(long = "secret-stdin", group = "key")]
    pub secret_stdin: bool,

    /// Address/descriptor script type. Defaults to `p2tr` when neither this nor
    /// `--all-script-types` is given.
    #[arg(long = "script-type", value_parser = crate::cmd::convert::parse_script_type_arg, conflicts_with = "all_script_types")]
    pub script_type: Option<ScriptType>,

    /// Emit descriptor + address for all four script types.
    #[arg(long = "all-script-types")]
    pub all_script_types: bool,

    /// Bitcoin network (affects address HRP + WIF version byte).
    // Do NOT add a Default/#[default] derive to CliNetwork; default_value_t renders via ValueEnum.
    #[arg(long, value_enum, default_value_t = CliNetwork::Mainnet)]
    pub network: CliNetwork,

    /// Emit JSON instead of the human-readable block.
    #[arg(long)]
    pub json: bool,
}

// Signature MUST match the sibling pattern (by-ref args, Result<u8>); the
// dispatch is `match &cli.command`. Verify against cmd/electrum_decrypt.rs.
pub fn run<R: Read, W: Write, E: Write>(
    args: &NostrArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let types: Vec<ScriptType> = if args.all_script_types {
        vec![ScriptType::P2tr, ScriptType::P2wpkh, ScriptType::P2shP2wpkh, ScriptType::P2pkh]
    } else {
        vec![args.script_type.unwrap_or(ScriptType::P2tr)]
    };

    if let Some(p) = args.pubkey.as_deref() {
        let xonly = crate::nostr::decode_npub(p)?;
        writeln!(stdout, "nostr key (public)").map_err(ToolkitError::Io)?;
        writeln!(stdout, "  x-only:      {xonly}").map_err(ToolkitError::Io)?;
        for st in &types {
            writeln!(stdout, "  script-type: {}", st.as_str()).map_err(ToolkitError::Io)?;
            writeln!(stdout, "  descriptor:  {}", crate::nostr::descriptor_for(xonly, *st)?).map_err(ToolkitError::Io)?;
            writeln!(stdout, "  address:     {}", crate::nostr::address_for(&secp, xonly, *st, args.network)).map_err(ToolkitError::Io)?;
        }
        return Ok(0);
    }

    // --secret* path is implemented in B3; B5 makes the key group required.
    let _ = (stdin, stderr);
    Err(ToolkitError::NostrKeyParse(
        "exactly one of --pubkey / --secret / --secret-file / --secret-stdin is required".into(),
    ))
}
