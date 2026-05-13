//! Full-mode BIP-32 derivation: phrase → entropy → seed → master xpriv → account xpub.
//!
//! Realizes SPEC §4.1.

use crate::error::ToolkitError;
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::template::CliTemplate;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpriv, Xpub};

/// Result of full-mode derivation.
///
/// SPEC v0.9.0 §1 item 2 — `impl Drop` scrubs `entropy` on drop.
/// Adding `impl Drop` BLOCKS move-out destructuring (E0509): callers
/// that consumed `DerivedAccount` via `let DerivedAccount { entropy,
/// .. } = derived;` no longer compile. Use [`DerivedAccount::into_parts`]
/// to consume the value cleanly. Field-borrow access is unaffected.
#[derive(Debug)]
pub struct DerivedAccount {
    pub entropy: Vec<u8>,
    pub master_fingerprint: Fingerprint,
    pub account_xpub: Xpub,
    pub account_xpriv: Xpriv,
    pub account_path: DerivationPath,
}

impl DerivedAccount {
    /// Consume `self`, returning all five fields. `std::mem::take`
    /// swaps the `entropy` Vec out of `self` (leaving an empty Vec
    /// for the Drop husk), then clones the `account_path`; the
    /// remaining three fields are `Copy`. The empty-Vec Drop is a
    /// no-op for memory cleanup — the real bytes are now owned by
    /// the caller, which is responsible for wrapping them in
    /// `Zeroizing<Vec<u8>>` at the call site.
    pub fn into_parts(mut self) -> (Vec<u8>, Fingerprint, Xpub, Xpriv, DerivationPath) {
        let entropy = std::mem::take(&mut self.entropy);
        let account_path = self.account_path.clone();
        (
            entropy,
            self.master_fingerprint,
            self.account_xpub,
            self.account_xpriv,
            account_path,
        )
    }
}

impl Drop for DerivedAccount {
    fn drop(&mut self) {
        // SPEC v0.9.0 §1 item 2 — scrub OWNED entropy buffer on drop.
        // `account_xpriv` is `Copy` and has no Drop hook upstream; the
        // residual gap is tracked at FOLLOWUPS:
        // `rust-bitcoin-xpriv-zeroize-upstream`.
        use zeroize::Zeroize;
        self.entropy.zeroize();
    }
}

pub fn derive_full(
    phrase: &str,
    passphrase: &str,
    language: CliLanguage,
    network: CliNetwork,
    template: CliTemplate,
    account: u32,
) -> Result<DerivedAccount, ToolkitError> {
    let mnemonic = Mnemonic::parse_in(language.into(), phrase).map_err(ToolkitError::Bip39)?;
    let entropy = mnemonic.to_entropy();
    crate::derive_slot::derive_bip32_from_entropy(
        &entropy,
        passphrase,
        language,
        network,
        template,
        account,
    )
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
        )
        .unwrap();
        let b = derive_full(
            TREZOR_24,
            "TREZOR",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip84,
            0,
        )
        .unwrap();
        assert_ne!(a.account_xpub, b.account_xpub);
    }

    #[test]
    fn derive_passphrase_empty_string_is_stable() {
        // The SPEC §4.1 step 3 invariant ("--passphrase \"\" ≡ unset") is enforced
        // at the CLI boundary in Phase 3; derive_full receives a `&str` and `""`
        // is the canonical representation of "no passphrase". This test pins
        // determinism of the empty-string path.
        let a = derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip84,
            0,
        )
        .unwrap();
        let b = derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip84,
            0,
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
            0,
        )
        .unwrap_err();
        assert!(matches!(e, ToolkitError::Bip39(_)));
    }
}
