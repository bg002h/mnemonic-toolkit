//! Shared test fixtures for bytecode-layer round-trip tests.
//!
//! Building real BIP 32 xpubs at arbitrary derivation depths in test
//! data requires either a per-test-vector xpub string or programmatic
//! derivation. The synthetic helper here constructs an `Xpub` whose
//! `depth` and `child_number` are correctly derived from a given path
//! and whose other fields are deterministic test bytes — sufficient
//! for round-trip tests where the bytes don't need to encode a real
//! BIP 32 derivation.
//!
//! Note: the parent module (`bytecode/mod.rs`) already gates this
//! file with `#[cfg(test)]`; no `#![cfg(test)]` inner attribute is
//! needed. Newer clippy fires `clippy::duplicated_attributes` if
//! both are present.

use bitcoin::NetworkKind;
use bitcoin::bip32::{ChainCode, ChildNumber, DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};

/// Build a synthetic mainnet `Xpub` with `depth` + `child_number`
/// derived from `path`. Other fields use deterministic test values.
pub(crate) fn synthetic_xpub(path: &DerivationPath) -> Xpub {
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(&[1u8; 32]).unwrap();
    let pk = PublicKey::from_secret_key(&secp, &sk);
    let components: Vec<ChildNumber> = path.into_iter().copied().collect();
    let depth = components.len() as u8;
    let child_number = components
        .last()
        .copied()
        .unwrap_or(ChildNumber::Normal { index: 0 });
    Xpub {
        network: NetworkKind::Main,
        depth,
        parent_fingerprint: Fingerprint::from([0xAA, 0xBB, 0xCC, 0xDD]),
        child_number,
        public_key: pk,
        chain_code: ChainCode::from([0x55u8; 32]),
    }
}
