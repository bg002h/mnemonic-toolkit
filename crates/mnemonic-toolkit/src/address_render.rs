//! Shared address rendering + network inference for `convert`, `xpub-search`,
//! and `addresses`. De-duplicates the previously-private copies that lived
//! verbatim in each (`convert.rs::build_address_from_xpub` /
//! `xpub_search/address_search.rs::render_address`; `convert.rs::network_from_xpub`
//! / `xpub_search/address_of_xpub.rs::network_from_xpub`).
//!
//! Bin module (imports `cmd::convert::ScriptType` + `network::CliNetwork`, which
//! are bin-only).

use bitcoin::bip32::Xpub;
use bitcoin::secp256k1::{Secp256k1, Verification};
use bitcoin::{Address, NetworkKind};

use crate::cmd::convert::ScriptType;
use crate::network::CliNetwork;

/// Render an address string from a (derived) child xpub.
pub(crate) fn render_address_from_xpub<C: Verification>(
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

/// Infer `CliNetwork` from an xpub's version bytes. `NetworkKind::Test` collapses
/// testnet / signet / regtest into `Testnet` (the bech32 HRP `tb1...` is shared;
/// signet/regtest disambiguation is not encoded in the version-byte prefix).
pub(crate) fn network_from_xpub(xpub: &Xpub) -> CliNetwork {
    match xpub.network {
        NetworkKind::Main => CliNetwork::Mainnet,
        NetworkKind::Test => CliNetwork::Testnet,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // A known mainnet bip84 account xpub (m/84'/0'/0').
    const ACCT_84: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";

    #[test]
    fn render_all_four_types_and_network_infer() {
        let secp = Secp256k1::verification_only();
        let xpub = Xpub::from_str(ACCT_84).unwrap();
        let child = xpub
            .derive_pub(&secp, &bitcoin::bip32::DerivationPath::from_str("m/0/0").unwrap())
            .unwrap();
        assert!(render_address_from_xpub(&secp, &child, ScriptType::P2wpkh, CliNetwork::Mainnet)
            .starts_with("bc1q"));
        assert!(render_address_from_xpub(&secp, &child, ScriptType::P2tr, CliNetwork::Mainnet)
            .starts_with("bc1p"));
        assert!(render_address_from_xpub(&secp, &child, ScriptType::P2pkh, CliNetwork::Mainnet)
            .starts_with('1'));
        assert!(render_address_from_xpub(&secp, &child, ScriptType::P2shP2wpkh, CliNetwork::Mainnet)
            .starts_with('3'));
        assert_eq!(network_from_xpub(&xpub), CliNetwork::Mainnet);
    }
}
