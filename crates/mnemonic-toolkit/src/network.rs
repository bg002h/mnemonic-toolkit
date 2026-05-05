//! `--network` clap enum + NetworkKind mapping + xpub-version cross-check.
//!
//! Realizes SPEC §2.1.4 (4 networks + coin-type table) + §4.3 (network/
//! xpub cross-check via Xpub::network field).

use bitcoin::NetworkKind;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "lower")]
pub enum CliNetwork {
    Mainnet,
    Testnet,
    Signet,
    Regtest,
}

impl CliNetwork {
    /// BIP-32 coin-type for this network (SPEC §2.1.4).
    /// Mainnet: 0; testnet/signet/regtest: 1.
    pub fn coin_type(&self) -> u32 {
        match self {
            CliNetwork::Mainnet => 0,
            CliNetwork::Testnet | CliNetwork::Signet | CliNetwork::Regtest => 1,
        }
    }

    /// `bitcoin::NetworkKind` for derivation. Mainnet: Main; others: Test.
    pub fn network_kind(&self) -> NetworkKind {
        match self {
            CliNetwork::Mainnet => NetworkKind::Main,
            _ => NetworkKind::Test,
        }
    }

    /// Human-readable name for stderr engraving card and error messages.
    pub fn human_name(&self) -> &'static str {
        match self {
            CliNetwork::Mainnet => "mainnet",
            CliNetwork::Testnet => "testnet",
            CliNetwork::Signet => "signet",
            CliNetwork::Regtest => "regtest",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coin_type_table() {
        assert_eq!(CliNetwork::Mainnet.coin_type(), 0);
        assert_eq!(CliNetwork::Testnet.coin_type(), 1);
        assert_eq!(CliNetwork::Signet.coin_type(), 1);
        assert_eq!(CliNetwork::Regtest.coin_type(), 1);
    }

    #[test]
    fn network_kind_mainnet_vs_test() {
        assert_eq!(CliNetwork::Mainnet.network_kind(), NetworkKind::Main);
        assert_eq!(CliNetwork::Testnet.network_kind(), NetworkKind::Test);
        assert_eq!(CliNetwork::Signet.network_kind(), NetworkKind::Test);
        assert_eq!(CliNetwork::Regtest.network_kind(), NetworkKind::Test);
    }
}
