//! Deterministic `DefiniteDescriptorKey` substitution for abstract miniscript
//! labels (`pk(A)`, `pk(B)`, …). SPEC §2.2.
//!
//! Cost is key-agnostic in miniscript (signature size is constant per scheme
//! and per script context — 73 bytes ECDSA, 64 bytes Schnorr SIGHASH_DEFAULT),
//! so the choice of dummy key does not affect the output vbytes. The
//! substitution exists only so `Descriptor::plan(...)` has concrete keys to
//! work with.

use miniscript::bitcoin::secp256k1::{Secp256k1, SecretKey};
use miniscript::bitcoin::{PublicKey, XOnlyPublicKey};
use miniscript::descriptor::{
    DefiniteDescriptorKey, DescriptorPublicKey, SinglePub, SinglePubKey,
};
use sha2::{Digest, Sha256};

/// Construct a deterministic 33-byte compressed-secp DefiniteDescriptorKey
/// from the user-supplied abstract label (e.g., "A", "B", "Alice").
pub fn dummy_compressed(label: &str) -> DefiniteDescriptorKey {
    let secp = Secp256k1::new();
    let mut counter: u8 = 0;
    loop {
        let domain = format!("compare-cost-dummy-key:{label}:{counter}");
        let scalar = Sha256::digest(domain.as_bytes());
        if let Ok(sk) = SecretKey::from_slice(scalar.as_slice()) {
            let pk = PublicKey::new(sk.public_key(&secp));
            let dpk = DescriptorPublicKey::Single(SinglePub {
                key: SinglePubKey::FullKey(pk),
                origin: None,
            });
            // `Single` carries no wildcards; `new` only errs on wildcards.
            return DefiniteDescriptorKey::new(dpk).expect("Single has no wildcards");
        }
        counter = counter
            .checked_add(1)
            .expect("hash-to-scalar counter exhaustion is measure-zero");
    }
}

/// Construct a deterministic 32-byte x-only DefiniteDescriptorKey from the
/// user-supplied abstract label.
pub fn dummy_xonly(label: &str) -> DefiniteDescriptorKey {
    let secp = Secp256k1::new();
    let mut counter: u8 = 0;
    loop {
        let domain = format!("compare-cost-dummy-key:{label}:{counter}");
        let scalar = Sha256::digest(domain.as_bytes());
        if let Ok(sk) = SecretKey::from_slice(scalar.as_slice()) {
            let xpk = XOnlyPublicKey::from(sk.public_key(&secp));
            let dpk = DescriptorPublicKey::Single(SinglePub {
                key: SinglePubKey::XOnly(xpk),
                origin: None,
            });
            return DefiniteDescriptorKey::new(dpk).expect("Single has no wildcards");
        }
        counter = counter
            .checked_add(1)
            .expect("hash-to-scalar counter exhaustion is measure-zero");
    }
}

/// Derive the BIP-341 NUMS H-point as a `DefiniteDescriptorKey` x-only,
/// suitable as the internal key for `tr(NUMS, {M})`.
pub fn nums_xonly_definite() -> DefiniteDescriptorKey {
    use std::str::FromStr;
    let xpk = XOnlyPublicKey::from_str(super::NUMS_XONLY_HEX).expect("BIP-341 NUMS x-only is valid");
    let dpk = DescriptorPublicKey::Single(SinglePub {
        key: SinglePubKey::XOnly(xpk),
        origin: None,
    });
    DefiniteDescriptorKey::new(dpk).expect("Single has no wildcards")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_keys_are_deterministic() {
        let a1 = dummy_compressed("A");
        let a2 = dummy_compressed("A");
        assert_eq!(a1.to_string(), a2.to_string(), "same label → same key");
    }

    #[test]
    fn distinct_labels_yield_distinct_keys() {
        let a = dummy_compressed("A");
        let b = dummy_compressed("B");
        assert_ne!(a.to_string(), b.to_string());
    }

    #[test]
    fn compressed_and_xonly_forms_serialize_differently() {
        let a_comp = dummy_compressed("A").to_string();
        let a_xonly = dummy_xonly("A").to_string();
        // compressed is 33 bytes (66 hex chars); xonly is 32 bytes (64 hex).
        assert_eq!(a_comp.len(), 66, "compressed pubkey is 33B / 66 hex chars");
        assert_eq!(a_xonly.len(), 64, "x-only pubkey is 32B / 64 hex chars");
    }

    #[test]
    fn nums_x_only_is_bip341_h_point() {
        let nums = nums_xonly_definite().to_string();
        assert_eq!(nums, super::super::NUMS_XONLY_HEX);
    }
}
