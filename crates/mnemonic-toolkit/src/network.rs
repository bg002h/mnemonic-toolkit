//! `--network` clap enum + NetworkKind mapping + xpub-version cross-check.
//!
//! Realizes SPEC §2.1.4 (4 networks + coin-type table) + §4.3 (network/
//! xpub cross-check via Xpub::network field).

use bitcoin::address::KnownHrp;
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

    /// SPEC v0.7 §10.a — `bitcoin::address::KnownHrp` for bech32/bech32m
    /// address constructors (`p2wpkh`, `p2tr`). Mainnet → `Mainnet`; testnet
    /// + signet → `Testnets` (shared `tb1...` HRP); regtest → `Regtest`.
    pub fn known_hrp(&self) -> KnownHrp {
        match self {
            CliNetwork::Mainnet => KnownHrp::Mainnet,
            CliNetwork::Testnet | CliNetwork::Signet => KnownHrp::Testnets,
            CliNetwork::Regtest => KnownHrp::Regtest,
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

    /// The `bitcoin::Network` for this CLI network (1:1 mapping).
    pub fn to_bitcoin_network(self) -> bitcoin::Network {
        match self {
            CliNetwork::Mainnet => bitcoin::Network::Bitcoin,
            CliNetwork::Testnet => bitcoin::Network::Testnet,
            CliNetwork::Signet => bitcoin::Network::Signet,
            CliNetwork::Regtest => bitcoin::Network::Regtest,
        }
    }
}

/// cycle-5 S-NET: the static name of a `NetworkKind` for error messages.
/// BIP-32 xpub version bytes distinguish only two families — `Main`
/// (`0488b21e`) and `Test` (`043587cf`, covering testnet/signet/regtest) — so
/// non-mainnet renders as `"testnet"` at this granularity.
pub(crate) const fn network_kind_name(kind: NetworkKind) -> &'static str {
    match kind {
        NetworkKind::Main => "mainnet",
        NetworkKind::Test => "testnet",
    }
}

/// cycle-5 S-NET: the shared fail-closed network-provenance invariant.
///
/// Rejects when a decoded artifact's `NetworkKind` (an xpub's `.network` OR a
/// WIF's `pk.network`) disagrees with the asserted network (coin-type-derived,
/// `--network`-derived, or envelope-declared). Granularity is `NetworkKind`
/// (Main vs Test, 2-way) — exactly the partition xpub version bytes and
/// coin-types encode (mainnet vs testnet/signet/regtest). Ports the
/// `synthesize.rs` `CosignerSpec` predicate to a reusable site.
///
/// PRECONDITION (caller-side): callers MUST skip this call entirely when there
/// is NO asserted network (originless / no-coin-type input). The helper itself
/// is unconditional — given two `NetworkKind`s it compares them; the
/// skip-when-no-asserted-network discipline lives at the call site so that an
/// originless `tpub` descriptor is NOT over-rejected.
pub(crate) fn assert_network_agrees(
    decoded: NetworkKind,
    asserted: NetworkKind,
    context: &'static str,
) -> Result<(), crate::error::ToolkitError> {
    if decoded != asserted {
        return Err(crate::error::ToolkitError::NetworkMismatch {
            decoded_network: network_kind_name(decoded),
            expected_network: network_kind_name(asserted),
            context,
        });
    }
    Ok(())
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

    // --- cycle-5 S-NET: shared network-provenance helper ---

    #[test]
    fn network_kind_name_renders_two_families() {
        assert_eq!(network_kind_name(NetworkKind::Main), "mainnet");
        assert_eq!(network_kind_name(NetworkKind::Test), "testnet");
    }

    #[test]
    fn assert_network_agrees_same_kind_is_ok() {
        assert!(assert_network_agrees(NetworkKind::Main, NetworkKind::Main, "test").is_ok());
        assert!(assert_network_agrees(NetworkKind::Test, NetworkKind::Test, "test").is_ok());
    }

    #[test]
    fn assert_network_agrees_main_vs_test_rejects() {
        let err = assert_network_agrees(NetworkKind::Main, NetworkKind::Test, "test ctx")
            .expect_err("Main vs Test must reject");
        match err {
            crate::error::ToolkitError::NetworkMismatch {
                decoded_network,
                expected_network,
                context,
            } => {
                assert_eq!(decoded_network, "mainnet");
                assert_eq!(expected_network, "testnet");
                assert_eq!(context, "test ctx");
            }
            other => panic!("expected NetworkMismatch, got {other:?}"),
        }
        assert_eq!(err_exit_code(NetworkKind::Main, NetworkKind::Test), 2);
    }

    #[test]
    fn assert_network_agrees_test_vs_main_rejects_symmetric() {
        let err = assert_network_agrees(NetworkKind::Test, NetworkKind::Main, "ctx")
            .expect_err("Test vs Main must reject");
        match err {
            crate::error::ToolkitError::NetworkMismatch {
                decoded_network,
                expected_network,
                ..
            } => {
                assert_eq!(decoded_network, "testnet");
                assert_eq!(expected_network, "mainnet");
            }
            other => panic!("expected NetworkMismatch, got {other:?}"),
        }
    }

    fn err_exit_code(decoded: NetworkKind, asserted: NetworkKind) -> u8 {
        assert_network_agrees(decoded, asserted, "x")
            .expect_err("mismatch")
            .exit_code()
    }

    #[test]
    fn network_kind_from_signet_regtest_is_test() {
        assert_eq!(
            NetworkKind::from(bitcoin::Network::Signet),
            NetworkKind::Test
        );
        assert_eq!(
            NetworkKind::from(bitcoin::Network::Regtest),
            NetworkKind::Test
        );
        assert_eq!(
            NetworkKind::from(bitcoin::Network::Testnet),
            NetworkKind::Test
        );
        assert_eq!(
            NetworkKind::from(bitcoin::Network::Bitcoin),
            NetworkKind::Main
        );
    }
}
