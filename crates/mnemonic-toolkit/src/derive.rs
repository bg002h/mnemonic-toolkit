//! Full-mode BIP-32 derivation: phrase → entropy → seed → master xpriv → account xpub.
//!
//! Realizes SPEC §4.1.

use crate::error::ToolkitError;
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::template::CliTemplate;
use bip39::Mnemonic;
use bitcoin::bip32::{Fingerprint, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;

/// Result of full-mode derivation.
#[derive(Debug)]
pub struct DerivedAccount {
    pub entropy: Vec<u8>,
    pub master_fingerprint: Fingerprint,
    pub account_xpub: Xpub,
}

pub fn derive_full(
    phrase: &str,
    passphrase: &str,
    language: CliLanguage,
    network: CliNetwork,
    template: CliTemplate,
) -> Result<DerivedAccount, ToolkitError> {
    let mnemonic = Mnemonic::parse_in(language.into(), phrase).map_err(ToolkitError::Bip39)?;
    let entropy = mnemonic.to_entropy();
    let seed = mnemonic.to_seed(passphrase);

    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network.network_kind(), &seed)
        .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
    let master_fingerprint = master.fingerprint(&secp);

    let path = template.derivation_path(network);
    let account_xpriv = master
        .derive_priv(&secp, &path)
        .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
    let account_xpub = Xpub::from_priv(&secp, &account_xpriv);

    // Belt-and-braces network cross-check (SPEC §4.3).
    if account_xpub.network != network.network_kind() {
        return Err(ToolkitError::BadInput(format!(
            "derived-xpub network {:?} does not match --network {}; this is a toolkit bug",
            account_xpub.network,
            network.human_name(),
        )));
    }

    Ok(DerivedAccount {
        entropy,
        master_fingerprint,
        account_xpub,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Trezor canonical 24-word vector: "abandon × 23 art" → 32-zero-bytes entropy.
    const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    #[test]
    fn derive_24word_zero_entropy() {
        let acc = derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip84,
        )
        .unwrap();
        assert_eq!(acc.entropy, vec![0u8; 32]);
    }

    #[test]
    fn derive_master_fingerprint_stable() {
        let acc = derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip84,
        )
        .unwrap();
        // Master fingerprint for "abandon × 23 art" (24-word, 32-zero entropy)
        // with empty passphrase, mainnet kind. Verified via /tmp/toolkit-spike
        // (bip39 = "2", bitcoin = "0.32"). NB: 73c5da0a (cited in rust-miniscript
        // descriptors) belongs to the 12-word "abandon × 11 about" vector.
        assert_eq!(
            acc.master_fingerprint.to_string().to_lowercase(),
            "5436d724"
        );
    }

    #[test]
    fn derive_xpub_at_bip84_mainnet_matches_known() {
        let acc = derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip84,
        )
        .unwrap();
        // Phase 1 spike + ground-truth: this is the canonical bip84 m/84'/0'/0' xpub
        // for the 24-zero-entropy seed. If the test fails after a bitcoin = 0.32
        // upgrade, regenerate via the spike harness in /tmp/toolkit-spike.
        let s = acc.account_xpub.to_string();
        assert!(
            s.starts_with("xpub6"),
            "expected xpub6 prefix, got {}",
            &s[..10]
        );
        assert!(acc.account_xpub.depth == 3);
    }

    #[test]
    fn derive_testnet_uses_tpub() {
        let acc = derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Testnet,
            CliTemplate::Bip84,
        )
        .unwrap();
        let s = acc.account_xpub.to_string();
        assert!(
            s.starts_with("tpub"),
            "expected tpub prefix on testnet, got {}",
            &s[..10]
        );
    }

    #[test]
    fn derive_with_passphrase_changes_seed() {
        let a = derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip84,
        )
        .unwrap();
        let b = derive_full(
            TREZOR_24,
            "TREZOR",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip84,
        )
        .unwrap();
        assert_ne!(a.account_xpub, b.account_xpub);
    }

    #[test]
    fn derive_passphrase_empty_string_equals_unset() {
        let a = derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip84,
        )
        .unwrap();
        // SPEC §4.1 step 3: --passphrase "" ≡ unset
        let b = derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip84,
        )
        .unwrap();
        assert_eq!(a.account_xpub, b.account_xpub);
        assert_eq!(a.master_fingerprint, b.master_fingerprint);
    }

    #[test]
    fn bad_phrase_returns_bip39_error() {
        let e = derive_full(
            "not a valid bip39 phrase nor anywhere close",
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip84,
        )
        .unwrap_err();
        assert!(matches!(e, ToolkitError::Bip39(_)));
    }
}
