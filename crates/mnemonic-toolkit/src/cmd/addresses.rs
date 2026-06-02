//! `mnemonic addresses` — batch watch-only address derivation.
//!
//! Lists a wallet's receive/change addresses from an account xpub (direct) or a
//! seed source (phrase/entropy/seedqr → `--address-type`-implied account xpub).
//! Read-only public derivation: no private keys reach stdout, no signing.

use std::io::{Read, Write};
use std::str::FromStr;

use bitcoin::bip32::{ChildNumber, DerivationPath, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bip39::Mnemonic;
use clap::Args;
use serde_json::json;

use crate::address_render::{network_from_xpub, render_address_from_xpub};
use crate::cmd::convert::{NodeType, ScriptType, parse_from_input, parse_script_type_arg};
use crate::cmd::convert::{read_stdin_passphrase, read_stdin_to_string};
use crate::derive_slot::derive_bip32_from_entropy;
use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::template::CliTemplate;

/// `mnemonic addresses` arguments.
#[derive(Args, Debug)]
pub struct AddressesArgs {
    /// Source: `xpub=<v>` | `phrase=<v>` | `entropy=<hex>` | `seedqr=<digits>`.
    /// Secret values support `@env:VAR` and `-` (stdin).
    #[arg(long)]
    pub from: String,

    /// Address type (required): p2pkh | p2sh-p2wpkh | p2wpkh | p2tr. For seed
    /// sources it also selects the BIP-44/49/84/86 account path.
    #[arg(long, value_parser = parse_script_type_arg)]
    pub address_type: ScriptType,

    /// Account index (seed sources only). Default 0.
    #[arg(long, default_value_t = 0)]
    pub account: u32,

    /// Number of addresses per chain, from index 0 (default 10). Conflicts with `--range`.
    #[arg(long, conflicts_with = "range")]
    pub count: Option<u32>,

    /// Inclusive index range `A,B` (alternative to `--count`).
    #[arg(long, conflicts_with = "count")]
    pub range: Option<String>,

    /// Which chain(s): `receive` (0, default), `change` (1), or `both`.
    #[arg(long, value_enum, default_value = "receive")]
    pub chain: ChainSel,

    /// Network override. Defaults to the xpub's version bytes (xpub source) or
    /// mainnet (seed source); must agree with an xpub's network kind.
    #[arg(long, value_enum)]
    pub network: Option<CliNetwork>,

    /// BIP-39 passphrase (seed sources). `@env:VAR` supported; or `--passphrase-stdin`.
    #[arg(long)]
    pub passphrase: Option<String>,

    /// Read the BIP-39 passphrase from stdin (conflicts with `--passphrase`).
    #[arg(long, conflicts_with = "passphrase")]
    pub passphrase_stdin: bool,

    /// BIP-39 wordlist language for `phrase=`/`seedqr=` (default english).
    #[arg(long, value_enum)]
    pub language: Option<CliLanguage>,

    /// Emit a structured JSON object on stdout instead of multi-line text.
    #[arg(long)]
    pub json: bool,
}

/// Chain selector.
#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "lower")]
pub enum ChainSel {
    Receive,
    Change,
    Both,
}

impl ChainSel {
    fn chains(self) -> &'static [u32] {
        match self {
            ChainSel::Receive => &[0],
            ChainSel::Change => &[1],
            ChainSel::Both => &[0, 1],
        }
    }
}

fn template_for(st: ScriptType) -> CliTemplate {
    match st {
        ScriptType::P2pkh => CliTemplate::Bip44,
        ScriptType::P2wpkh => CliTemplate::Bip84,
        ScriptType::P2shP2wpkh => CliTemplate::Bip49,
        ScriptType::P2tr => CliTemplate::Bip86,
    }
}

fn bad(s: impl Into<String>) -> ToolkitError {
    ToolkitError::BadInput(s.into())
}

/// Run `mnemonic addresses`.
pub fn run<R: Read, W: Write, E: Write>(
    args: &AddressesArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let from = parse_from_input(&args.from).map_err(bad)?;
    let from_uses_stdin = from.value == "-";

    // Single-stdin-per-invocation guard.
    if args.passphrase_stdin && from_uses_stdin {
        return Err(bad(
            "--passphrase-stdin cannot coexist with --from <node>=- (a single stdin cannot serve both)",
        ));
    }

    // argv-leak advisories for inline secret-bearing values (mirror convert scope).
    if from.node.is_argv_secret_bearing() && !from_uses_stdin && !from.value.starts_with("@env:") {
        let node = args.from.split('=').next().unwrap_or("");
        crate::secret_advisory::secret_in_argv_warning(
            stderr,
            &format!("--from {node}="),
            &format!("--from {node}=-"),
        );
    }
    // `--passphrase` only applies to seed sources (xpub rejects it below), so
    // don't fire the advisory for an xpub source that's about to be refused (M2).
    if from.node != NodeType::Xpub {
        if let Some(pp) = args.passphrase.as_deref() {
            if !pp.starts_with("@env:") {
                crate::secret_advisory::secret_in_argv_warning(
                    stderr,
                    "--passphrase",
                    "--passphrase-stdin",
                );
            }
        }
    }

    // Effective BIP-39 passphrase (stdin / @env: / inline).
    let passphrase: String = if args.passphrase_stdin {
        read_stdin_passphrase(stdin)?
    } else {
        match args.passphrase.as_deref() {
            Some(p) => crate::env_sentinel::resolve_env_var_sentinel(p, "--passphrase")?,
            None => String::new(),
        }
    };

    // Resolved `--from` value (stdin / @env: / literal).
    let from_value: String = if from_uses_stdin {
        read_stdin_to_string(stdin)?
    } else {
        crate::env_sentinel::resolve_env_var_sentinel(&from.value, "--from")?
    };

    // Resolve the account xpub + effective network (+ the JSON `account` field).
    let (account_xpub, network, account_field): (Xpub, CliNetwork, Option<u32>) = match from.node {
        NodeType::Xpub => {
            if args.account != 0 {
                return Err(bad(
                    "--account does not apply to --from xpub= (the xpub is already an account key)",
                ));
            }
            if args.passphrase.is_some() || args.passphrase_stdin {
                return Err(bad(
                    "--passphrase / --passphrase-stdin do not apply to --from xpub= (no BIP-39 seed)",
                ));
            }
            let xpub = Xpub::from_str(&from_value)
                .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
            let net = match args.network {
                None => network_from_xpub(&xpub),
                Some(n) => {
                    if n.network_kind() != xpub.network {
                        return Err(bad(format!(
                            "--network {} disagrees with the xpub's network kind; refusing to render wrong-network addresses",
                            n.human_name()
                        )));
                    }
                    n
                }
            };
            (xpub, net, None)
        }
        NodeType::Phrase | NodeType::Entropy | NodeType::Seedqr => {
            let language = args.language.unwrap_or_default();
            let network = args.network.unwrap_or(CliNetwork::Mainnet);
            // I1: scrub the intermediate entropy (master secret) on drop, matching
            // convert's `Zeroizing<Vec<u8>>` convention (convert.rs:1147).
            let entropy: zeroize::Zeroizing<Vec<u8>> = zeroize::Zeroizing::new(match from.node {
                NodeType::Phrase => Mnemonic::parse_in(language.into(), &from_value)
                    .map_err(ToolkitError::Bip39)?
                    .to_entropy(),
                NodeType::Entropy => hex::decode(from_value.trim())
                    .map_err(|e| bad(format!("--from entropy= hex-decode: {e}")))?,
                NodeType::Seedqr => {
                    let phrase = mnemonic_toolkit::seedqr::decode(&from_value)
                        .map_err(|e| crate::cmd::seedqr::map_seedqr_error(e, "addresses"))?;
                    Mnemonic::parse_in(language.into(), &phrase)
                        .map_err(ToolkitError::Bip39)?
                        .to_entropy()
                }
                _ => unreachable!(),
            });
            let acct = derive_bip32_from_entropy(
                &entropy,
                &passphrase,
                language.into(),
                network,
                template_for(args.address_type),
                args.account,
            )?;
            (acct.account_xpub, network, Some(args.account))
        }
        other => {
            return Err(bad(format!(
                "--from {other:?} is not supported by `addresses` (use xpub/phrase/entropy/seedqr)"
            )));
        }
    };

    let indices = resolve_indices(args.count, args.range.as_deref())?;
    let secp = Secp256k1::verification_only();

    // (chain, index, address), chain-major (receive before change).
    let mut rows: Vec<(u32, u32, String)> = Vec::new();
    for &chain in args.chain.chains() {
        for &index in &indices {
            let leaf = ChildNumber::from_normal_idx(index).map_err(|_| {
                bad(format!("index {index} out of BIP-32 normal range (0..2147483647)"))
            })?;
            let dp: DerivationPath = vec![ChildNumber::from_normal_idx(chain).unwrap(), leaf].into();
            let child = account_xpub
                .derive_pub(&secp, &dp)
                .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
            rows.push((
                chain,
                index,
                render_address_from_xpub(&secp, &child, args.address_type, network),
            ));
        }
    }

    if args.json {
        emit_json(stdout, from.node, args.address_type, network, account_field, &rows)?;
    } else {
        emit_text(stdout, args.chain, &rows)?;
    }
    crate::secret_advisory::emit_output_class_advisory(
        crate::secret_advisory::OutputClass::WatchOnly,
        stderr,
    );
    Ok(0)
}

/// `--count N` → `0..N`; `--range A,B` → `A..=B`; neither → `0..10`. Validates
/// the BIP-32 normal-index ceiling (`< 2^31`) BEFORE allocating.
fn resolve_indices(count: Option<u32>, range: Option<&str>) -> Result<Vec<u32>, ToolkitError> {
    const MAX_PLUS1: u32 = 1u32 << 31; // valid normal indices 0..=2^31-1
    match (count, range) {
        (Some(_), Some(_)) => unreachable!("clap conflicts_with"),
        (Some(c), None) => {
            if c > MAX_PLUS1 {
                return Err(bad(format!(
                    "--count {c} exceeds the BIP-32 normal-index ceiling (max 2147483648)"
                )));
            }
            Ok((0..c).collect())
        }
        (None, Some(r)) => {
            let (a, b) = r
                .split_once(',')
                .ok_or_else(|| bad(format!("--range expects `A,B`, got {r:?}")))?;
            let a: u32 = a
                .trim()
                .parse()
                .map_err(|e| bad(format!("--range start {a:?}: {e}")))?;
            let b: u32 = b
                .trim()
                .parse()
                .map_err(|e| bad(format!("--range end {b:?}: {e}")))?;
            if a > b {
                return Err(bad(format!("--range start {a} must be <= end {b}")));
            }
            if b >= MAX_PLUS1 {
                return Err(bad(format!(
                    "--range end {b} exceeds the BIP-32 normal-index ceiling (2147483647)"
                )));
            }
            Ok((a..=b).collect())
        }
        (None, None) => Ok((0..10).collect()),
    }
}

fn source_label(node: NodeType) -> &'static str {
    match node {
        NodeType::Xpub => "xpub",
        NodeType::Phrase => "phrase",
        NodeType::Entropy => "entropy",
        NodeType::Seedqr => "seedqr",
        _ => "unknown",
    }
}

fn emit_text<W: Write>(
    stdout: &mut W,
    chain: ChainSel,
    rows: &[(u32, u32, String)],
) -> Result<(), ToolkitError> {
    let grouped = matches!(chain, ChainSel::Both);
    let mut cur_chain: Option<u32> = None;
    for (c, idx, addr) in rows {
        if grouped && cur_chain != Some(*c) {
            let label = if *c == 0 { "receive" } else { "change" };
            writeln!(stdout, "{label} (m/{c}/i):").map_err(ToolkitError::Io)?;
            cur_chain = Some(*c);
        }
        writeln!(stdout, "  {idx}  {addr}").map_err(ToolkitError::Io)?;
    }
    Ok(())
}

fn emit_json<W: Write>(
    stdout: &mut W,
    source: NodeType,
    addr_type: ScriptType,
    network: CliNetwork,
    account: Option<u32>,
    rows: &[(u32, u32, String)],
) -> Result<(), ToolkitError> {
    let addresses: Vec<_> = rows
        .iter()
        .map(|(c, i, a)| json!({ "chain": c, "index": i, "address": a }))
        .collect();
    let mut envelope = json!({
        "schema_version": "1",
        "source": source_label(source),
        "address_type": addr_type.as_str(),
        "network": network.human_name(),
        "addresses": addresses,
    });
    if let Some(acct) = account {
        envelope["account"] = json!(acct);
    }
    let s = serde_json::to_string(&envelope)
        .map_err(|e| bad(format!("json serialization: {e}")))?;
    writeln!(stdout, "{s}").map_err(ToolkitError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ceiling_boundary_unit() {
        // 2^31 accepted (highest index 2^31-1); 2^31+1 rejected. Unit-only (a CLI
        // run would build an 8 GB Vec).
        assert!(resolve_indices(Some(2_147_483_648), None).is_ok());
        assert!(resolve_indices(Some(2_147_483_649), None).is_err());
        assert!(resolve_indices(None, Some("0,2147483648")).is_err());
        assert!(resolve_indices(None, Some("5,2")).is_err());
        assert_eq!(resolve_indices(None, None).unwrap().len(), 10);
    }
}
