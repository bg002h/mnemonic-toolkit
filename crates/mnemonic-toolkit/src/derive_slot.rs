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
use bitcoin::bip32::{Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;

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
    let mnemonic =
        Mnemonic::from_entropy_in(language.into(), entropy).map_err(ToolkitError::Bip39)?;
    let seed = mnemonic.to_seed(passphrase);

    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network.network_kind(), &seed)
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
        account_path: path,
    })
}
