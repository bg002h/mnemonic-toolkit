//! `mnemonic decode-address` core — decode a Bitcoin address string into its
//! network(s), script type, witness version, and scriptPubKey. PUBLIC-DATA
//! utility: no secrets, no key material, no mlock. Binary-private (returns
//! `crate::error::ToolkitError`, which is not in `lib.rs`).

use crate::error::ToolkitError;
use bitcoin::address::{Address, NetworkUnchecked};
use bitcoin::Network;

/// The decoded facts about an address string.
pub(crate) struct DecodedAddress {
    /// Networks whose prefix/HRP this address is valid for. The address layer
    /// cannot disambiguate testnet/testnet4/signet (shared `tb1`/base58
    /// prefixes), so we report the full set rather than a single network.
    pub networks: Vec<&'static str>,
    /// Lowercase script type via `AddressType`'s Display: p2pkh/p2sh/p2wpkh/
    /// p2wsh/p2tr/p2a (forward-compatible — `AddressType` is `#[non_exhaustive]`),
    /// or "unknown" for a future/unrecognized output type.
    pub script_type: String,
    /// Segwit witness version (Some(0) for v0, Some(1) for taproot, …); None for
    /// legacy (P2PKH/P2SH) addresses.
    pub witness_version: Option<u8>,
    /// scriptPubKey as lowercase hex.
    pub script_pubkey_hex: String,
    /// The canonical re-serialized address (round-trips the input).
    pub address_normalized: String,
}

/// Decode an address string. Errors only when the string is not a valid
/// Bitcoin address for ANY supported network.
pub(crate) fn decode_address(input: &str) -> Result<DecodedAddress, ToolkitError> {
    let trimmed = input.trim();
    let unchecked: Address<NetworkUnchecked> = trimmed
        .parse()
        .map_err(|e| ToolkitError::DecodeAddress(format!("not a valid Bitcoin address: {e}")))?;

    // `tb1` HRP (and the testnet base58 prefixes) are valid for testnet,
    // testnet4, AND signet; regtest is the distinct `bcrt1` HRP. Probe all.
    let mut networks = Vec::new();
    for (label, net) in [
        ("mainnet", Network::Bitcoin),
        ("testnet", Network::Testnet),
        ("testnet4", Network::Testnet4),
        ("signet", Network::Signet),
        ("regtest", Network::Regtest),
    ] {
        if unchecked.is_valid_for_network(net) {
            networks.push(label);
        }
    }

    // Post-parse the address is structurally valid; assume_checked is safe and
    // network-independent for script_pubkey / address_type / witness_program.
    let checked = unchecked.assume_checked();
    let spk = checked.script_pubkey();
    let script_type = checked
        .address_type()
        .map(|t| t.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let witness_version = checked.witness_program().map(|wp| wp.version().to_num());

    Ok(DecodedAddress {
        networks,
        script_type,
        witness_version,
        script_pubkey_hex: hex::encode(spk.as_bytes()),
        address_normalized: checked.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p2wpkh_mainnet_bip173_vector() {
        let d = decode_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4").unwrap();
        assert_eq!(d.script_type, "p2wpkh");
        assert_eq!(d.witness_version, Some(0));
        assert_eq!(
            d.script_pubkey_hex,
            "0014751e76e8199196d454941c45d1b3a323f1433bd6"
        );
        assert!(d.networks.contains(&"mainnet"));
        assert!(!d.networks.contains(&"testnet"));
    }

    #[test]
    fn p2pkh_mainnet_script_pubkey() {
        let d = decode_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2").unwrap();
        assert_eq!(d.script_type, "p2pkh");
        assert_eq!(d.witness_version, None);
        assert!(d.script_pubkey_hex.starts_with("76a914") && d.script_pubkey_hex.ends_with("88ac"));
        assert!(d.networks.contains(&"mainnet"));
    }

    #[test]
    fn p2sh_mainnet() {
        let d = decode_address("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy").unwrap();
        assert_eq!(d.script_type, "p2sh");
        assert_eq!(d.witness_version, None);
        // P2SH scriptPubKey: OP_HASH160 <20> OP_EQUAL → a914…87
        assert!(d.script_pubkey_hex.starts_with("a914") && d.script_pubkey_hex.ends_with("87"));
    }

    #[test]
    fn p2tr_witness_v1() {
        // BIP-350 canonical P2TR example.
        let d = decode_address("bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqzk5jj0")
            .unwrap();
        assert_eq!(d.script_type, "p2tr");
        assert_eq!(d.witness_version, Some(1));
        assert!(d.script_pubkey_hex.starts_with("5120"));
    }

    #[test]
    fn invalid_address_errors() {
        assert!(decode_address("not-an-address").is_err());
        assert!(decode_address("").is_err());
    }

    #[test]
    fn tb1_hrp_valid_for_testnet_testnet4_signet_not_regtest() {
        let d = decode_address("tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx").unwrap();
        // tb1 HRP is valid for testnet + testnet4 + signet; regtest is bcrt1.
        assert!(d.networks.contains(&"testnet"));
        assert!(d.networks.contains(&"testnet4"));
        assert!(d.networks.contains(&"signet"));
        assert!(!d.networks.contains(&"regtest"));
        assert!(!d.networks.contains(&"mainnet"));
        assert_eq!(d.script_type, "p2wpkh");
    }

    #[test]
    fn whitespace_trimmed() {
        let d = decode_address("  bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4  ").unwrap();
        assert_eq!(d.script_type, "p2wpkh");
    }
}
