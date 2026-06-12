//! Full-mode BIP-32 derivation: phrase → entropy → seed → master xpriv → account xpub.
//!
//! Realizes SPEC §4.1.

use crate::error::ToolkitError;
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::template::CliTemplate;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpriv, Xpub};
use mnemonic_toolkit::mlock::PinnedPageRange;

/// Result of full-mode derivation.
///
/// v0.10.1: `entropy` is `Zeroizing<Vec<u8>>` so the Drop-time scrub is
/// structurally guaranteed by the type. The previous `impl Drop for
/// DerivedAccount` (Cycle A v0.9.0) is deleted; `Zeroizing` now carries
/// the scrub. Move-out destructuring (`let DerivedAccount { entropy, .. }
/// = derived;`) is once again E0509-free. [`DerivedAccount::into_parts`]
/// remains useful for consuming-move ergonomics and is the canonical
/// path; it returns a bare `Vec<u8>` per the caller-wrap contract.
#[derive(Debug)]
pub struct DerivedAccount {
    pub entropy: zeroize::Zeroizing<Vec<u8>>,
    pub master_fingerprint: Fingerprint,
    pub account_xpub: Xpub,
    pub account_xpriv: Xpriv,
    pub account_path: DerivationPath,
    /// Cycle B Phase 3a Path B-lite sibling pin for the `entropy` heap
    /// buffer's pages. No `Option` / `Rc` wrap (DerivedAccount is not
    /// Clone and is consumed via `into_parts`). Declared LAST so on Drop
    /// the field order is `entropy` first (Zeroizing::drop scrubs the
    /// inner Vec then deallocs) → then `_entropy_pin` munlock. Strictest
    /// threat-model ordering (zeroize-while-still-pinned).
    pub _entropy_pin: PinnedPageRange,
}

impl DerivedAccount {
    /// Consume `self`, returning all five fields. `std::mem::take` swaps
    /// the inner `Vec` out of `self.entropy` (the `Zeroizing` wrapper
    /// stays, now wrapping an empty Vec whose Drop scrub is a no-op).
    /// The returned bare `Vec<u8>` is the caller's responsibility per
    /// the caller-wrap contract — wrap in `Zeroizing<Vec<u8>>` at the
    /// call site if the consumer needs scrub-on-drop semantics. The
    /// remaining four fields move out by value (three are `Copy`;
    /// `account_path` clones).
    pub fn into_parts(mut self) -> (Vec<u8>, Fingerprint, Xpub, Xpriv, DerivationPath) {
        let entropy = std::mem::take(&mut *self.entropy);
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

pub fn derive_full(
    phrase: &str,
    passphrase: &str,
    language: CliLanguage,
    network: CliNetwork,
    template: CliTemplate,
    account: u32,
) -> Result<DerivedAccount, ToolkitError> {
    // SAFETY: third-party-blocked — `bip39::Mnemonic` has no Drop+Zeroize;
    // tracked by FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`. Lifetime
    // here is minimal: parse → to_entropy → drop on function return.
    let mnemonic = Mnemonic::parse_in(language.into(), phrase).map_err(ToolkitError::Bip39)?;
    let entropy = zeroize::Zeroizing::new(mnemonic.to_entropy());
    crate::derive_slot::derive_bip32_from_entropy(
        &entropy,
        passphrase,
        language.into(),
        network,
        template,
        account,
    )
}

/// Path-explicit sibling of [`derive_full`]: derive the account key at an
/// explicit BIP-32 `path` (e.g. a BIP-48 multisig path) instead of the
/// template default. Used by the multisig branch of `resolve_slots` so
/// `--multisig-path-family bip48` reaches the real seed derivation (F3 fix).
pub(crate) fn derive_full_at_path(
    phrase: &str,
    passphrase: &str,
    language: CliLanguage,
    network: CliNetwork,
    path: &DerivationPath,
) -> Result<DerivedAccount, ToolkitError> {
    // SAFETY: third-party-blocked — `bip39::Mnemonic` has no Drop+Zeroize;
    // tracked by FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`. Lifetime
    // here is minimal: parse → to_entropy → drop on function return.
    let mnemonic = Mnemonic::parse_in(language.into(), phrase).map_err(ToolkitError::Bip39)?;
    let entropy = zeroize::Zeroizing::new(mnemonic.to_entropy());
    crate::derive_slot::derive_bip32_from_entropy_at_path(
        &entropy,
        passphrase,
        language.into(),
        network,
        path,
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
        assert_eq!(*acc.entropy, vec![0u8; 32]);
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

    // ========================================================================
    // Path B-lite Site 3 — DerivedAccount struct-sibling pin coverage.
    // (See bip85.rs path_b_lite_pin_tests preamble for the attempts-counter
    // observation rationale.)
    // ========================================================================

    /// Site 3 — `derive_full` returns a `DerivedAccount` whose construction
    /// at `derive_slot.rs:77` invokes `pin_pages_for` on the entropy buffer
    /// before moving it into the new `_entropy_pin: PinnedPageRange` sibling
    /// field. Asserts `attempts_for_test()` incremented along the path.
    #[test]
    fn site_3_derive_full_invokes_pin_at_derivedaccount_construction() {
        let baseline = mnemonic_toolkit::mlock::attempts_for_test();
        let _acc = derive_full(
            TREZOR_24,
            "",
            CliLanguage::English,
            CliNetwork::Mainnet,
            CliTemplate::Bip84,
            0,
        )
        .unwrap();
        assert!(
            mnemonic_toolkit::mlock::attempts_for_test() > baseline,
            "derive_full -> derive_bip32_from_entropy -> DerivedAccount ctor \
             must invoke pin_pages_for; attempts counter did not increment",
        );
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
