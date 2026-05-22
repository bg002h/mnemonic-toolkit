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
    _args: &NostrArgs,
    _stdin: &mut R,
    _stdout: &mut W,
    _stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // Implemented in B2–B5. The exactly-one-key invariant becomes a required
    // ArgGroup in B5.
    todo!("implemented in B2–B5")
}
