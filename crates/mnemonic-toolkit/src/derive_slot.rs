//! BIP-39 + BIP-32 derivation helper shared by `cmd::bundle::resolve_slots`
//! (phrase / entropy slot branches) and `cmd::convert` (BIP-39-rooted edges).
//!
//! Locked v0.5.2: extracted from `bundle::resolve_slots` to remove the
//! duplicated derivation spine between phrase and entropy branches.

use crate::derive::DerivedAccount;
use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::template::CliTemplate;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use zeroize::Zeroizing;

/// SPEC v0.9.0 §1 item 2 — consolidated BIP-39 → BIP-32 seed step.
/// Wraps the 64-byte PBKDF2-HMAC-SHA512 output in `Zeroizing` so it
/// scrubs on drop at every call site. Seven production BIP-39 →
/// BIP-32 spines in this crate share this helper:
///
/// - `derive_slot::derive_bip32_from_entropy` (this file)
/// - `derive_slot::derive_bip32_at_path` (this file)
/// - `synthesize::synthesize_multisig_full`
/// - `parse_descriptor::bind_full_mode`
/// - `cmd::bundle::bundle_run_unified_descriptor` (Phrase + Entropy arms)
/// - `cmd::derive_child::run` (Phrase master)
///
/// Per-site code remains site-specific (input type, network
/// handling, derivation path source, return shape); only the
/// `to_seed` step is consolidated here.
pub fn derive_master_seed(mnemonic: &Mnemonic, passphrase: &str) -> Zeroizing<[u8; 64]> {
    Zeroizing::new(mnemonic.to_seed(passphrase))
}

/// entropy → mnemonic-in-language → seed → master xpriv → derive at template
/// path → (entropy, master_fingerprint, account_xpub, account_path).
///
/// `entropy.len()` must be a BIP-39-valid length (16/20/24/28/32 bytes); the
/// caller is responsible for validation. `Mnemonic::from_entropy_in` rejects
/// invalid lengths with `ToolkitError::Bip39`.
pub(crate) fn derive_bip32_from_entropy(
    entropy: &[u8],
    passphrase: &str,
    language: CliLanguage,
    network: CliNetwork,
    template: CliTemplate,
    account: u32,
) -> Result<DerivedAccount, ToolkitError> {
    // SAFETY: third-party-blocked — `bip39::Mnemonic` + `bitcoin::bip32::Xpriv`
    // have no Drop+Zeroize. FOLLOWUPS: `rust-bip39-mnemonic-zeroize-upstream`,
    // `rust-bitcoin-xpriv-zeroize-upstream`. Per-function lifetime is bounded
    // and the seed buffer is `Zeroizing<[u8; 64]>` via `derive_master_seed`.
    let mnemonic =
        Mnemonic::from_entropy_in(language.into(), entropy).map_err(ToolkitError::Bip39)?;
    let seed = derive_master_seed(&mnemonic, passphrase);

    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network.network_kind(), &seed[..])
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
    let master_fingerprint = master.fingerprint(&secp);

    let path = template.derivation_path(network, account);
    let account_xpriv = master
        .derive_priv(&secp, &path)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
    let account_xpub = Xpub::from_priv(&secp, &account_xpriv);

    if account_xpub.network != network.network_kind() {
        return Err(ToolkitError::BadInput(format!(
            "derived-xpub network {:?} does not match --network {}; this is a toolkit bug",
            account_xpub.network,
            network.human_name(),
        )));
    }

    Ok(DerivedAccount {
        entropy: entropy.to_vec(),
        master_fingerprint,
        account_xpub,
        account_xpriv,
        account_path: path,
    })
}

/// SPEC-A v0.6.1: path-driven sibling of `derive_bip32_from_entropy`.
///
/// entropy → mnemonic-in-language → seed → master xpriv → derive at the
/// supplied `--path` → return the leaf xpriv. Used by `cmd::convert`'s
/// `phrase`/`entropy` → `wif` edge (SPEC `§2`).
///
/// `path` may be at any BIP-32 depth; no normative depth assertion is made
/// (the caller is responsible for supplying a path that produces a leaf
/// privkey suitable for the downstream emission). Network-mismatch checks
/// in the parent `derive_bip32_from_entropy` are intentionally NOT
/// duplicated here — that helper guards the BIP-39 → account flow against a
/// toolkit-bug class; the path-driven flow inherits the same guarantees
/// from `Xpriv::new_master`.
pub(crate) fn derive_bip32_at_path(
    entropy: &[u8],
    passphrase: &str,
    language: CliLanguage,
    network: CliNetwork,
    path: &DerivationPath,
) -> Result<Xpriv, ToolkitError> {
    // SAFETY: third-party-blocked — `bip39::Mnemonic` + `bitcoin::bip32::Xpriv`
    // have no Drop+Zeroize. FOLLOWUPS: `rust-bip39-mnemonic-zeroize-upstream`,
    // `rust-bitcoin-xpriv-zeroize-upstream`. Per-function lifetime is bounded.
    let mnemonic =
        Mnemonic::from_entropy_in(language.into(), entropy).map_err(ToolkitError::Bip39)?;
    let seed = derive_master_seed(&mnemonic, passphrase);

    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network.network_kind(), &seed[..])
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
    master
        .derive_priv(&secp, path)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))
}
