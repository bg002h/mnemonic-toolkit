//! Address-search engine for `xpub-search address-of-xpub` (P3).
//!
//! Given a parent xpub + a list of target address strings + scan bounds, scan
//! child indices `chain ∈ {0, 1} × i ∈ [0, gap_limit)` and report which
//! targets match at which (chain, index). Per-target first-match wins.

use crate::cmd::convert::ScriptType;
use crate::network::CliNetwork;
use bitcoin::bip32::{DerivationPath, Xpub};
use bitcoin::secp256k1::{Secp256k1, Verification};
use bitcoin::Address;
use std::str::FromStr;

/// Per-target scan result.
pub enum AddressMatchKind {
    Match {
        chain: &'static str, // "external" | "internal"
        index: u32,
    },
    NoMatch {
        scanned_external: u32,
        scanned_internal: u32,
    },
}

pub struct AddressMatch {
    pub target: String,
    pub result: AddressMatchKind,
    pub script_type: ScriptType,
}

/// Build an address string from a child xpub, mirroring
/// `cmd/convert.rs::build_address_from_xpub`. Inlined here (not pulled via
/// `pub use`) to avoid exposing the convert.rs private fn surface.
fn render_address<C: Verification>(
    secp: &Secp256k1<C>,
    child: &Xpub,
    script_type: ScriptType,
    network: CliNetwork,
) -> String {
    match script_type {
        ScriptType::P2pkh => Address::p2pkh(child.to_pub(), network.network_kind()).to_string(),
        ScriptType::P2wpkh => Address::p2wpkh(&child.to_pub(), network.known_hrp()).to_string(),
        ScriptType::P2shP2wpkh => {
            Address::p2shwpkh(&child.to_pub(), network.network_kind()).to_string()
        }
        ScriptType::P2tr => {
            Address::p2tr(secp, child.to_x_only_pub(), None, network.known_hrp()).to_string()
        }
    }
}

/// Scan one or both chains for each target. First-match-per-target wins.
/// Returns the per-target outcomes in input order.
pub fn scan_xpub_for_addresses<C: Verification>(
    xpub: &Xpub,
    targets: &[String],
    gap_limit: u32,
    scan_internal: bool,
    script_type: ScriptType,
    network: CliNetwork,
    secp: &Secp256k1<C>,
) -> Vec<AddressMatch> {
    // Build a flat (chain, index, rendered_address) cache for the entire
    // scan window. This costs one BIP-32 derivation per (chain, index)
    // regardless of how many targets we have — far cheaper than re-scanning
    // per target.
    let chains: Vec<(&'static str, u32)> = if scan_internal {
        vec![("external", 0), ("internal", 1)]
    } else {
        vec![("external", 0)]
    };

    // (chain_name, index, address_string)
    let mut rendered: Vec<(&'static str, u32, String)> =
        Vec::with_capacity(chains.len() * gap_limit as usize);
    for (chain_name, chain_idx) in &chains {
        for i in 0..gap_limit {
            let dp = match DerivationPath::from_str(&format!("m/{chain_idx}/{i}")) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let child = match xpub.derive_pub(secp, &dp) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let addr = render_address(secp, &child, script_type, network);
            rendered.push((*chain_name, i, addr));
        }
    }

    // For each target: linear scan (first match wins).
    targets
        .iter()
        .map(|target| {
            for (chain_name, i, addr) in &rendered {
                if addr == target {
                    return AddressMatch {
                        target: target.clone(),
                        result: AddressMatchKind::Match {
                            chain: chain_name,
                            index: *i,
                        },
                        script_type,
                    };
                }
            }
            AddressMatch {
                target: target.clone(),
                result: AddressMatchKind::NoMatch {
                    scanned_external: gap_limit,
                    scanned_internal: if scan_internal { gap_limit } else { 0 },
                },
                script_type,
            }
        })
        .collect()
}
