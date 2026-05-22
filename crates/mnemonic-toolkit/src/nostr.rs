//! Nostr-key wrappers — NIP-19 (`npub`/`nsec`) decode, BIP-340 even-y
//! normalization, and Bitcoin address/descriptor/WIF derivation for the
//! `mnemonic nostr` subcommand.
//!
//! A nostr key is a BIP-340 x-only secp256k1 key. Taproot (`p2tr`) is the
//! native mapping — the x-only key IS the taproot internal key, no parity
//! fabrication. Non-taproot (`p2pkh`/`p2wpkh`/`p2sh-p2wpkh`) uses the BIP-340
//! even-y compressed form `02‖x` (mirrors `cost/strip.rs` §11). For `nsec`,
//! the secret is normalized to even-y so the emitted WIF controls the emitted
//! address (see `normalize_to_even_y`).

#![allow(unused_imports)] // skeleton — imports consumed by Tasks A1/A2/A3

use crate::error::ToolkitError;
use bitcoin::secp256k1::{Parity, PublicKey, Secp256k1, SecretKey, Signing, Verification, XOnlyPublicKey};
use bitcoin::CompressedPublicKey;
use zeroize::Zeroizing;

/// Normalize a secret to BIP-340 even-y form. If `d·G` has odd y, returns
/// `n−d` (so the key matches the even-y `02‖x` address and the taproot
/// internal key); else returns `d` unchanged. Returns `(normalized, negated?)`.
/// The x-only pubkey is parity-independent, so it is unchanged either way.
pub fn normalize_to_even_y<C: Signing>(secp: &Secp256k1<C>, secret: SecretKey) -> (SecretKey, bool) {
    let (_xonly, parity) = secret.x_only_public_key(secp);
    match parity {
        Parity::Odd => (secret.negate(), true),
        Parity::Even => (secret, false),
    }
}

/// Decode an `npub1…` (NIP-19 bech32) or 64-hex string into an x-only key.
pub fn decode_npub(input: &str) -> Result<XOnlyPublicKey, ToolkitError> {
    let bytes = decode_nostr_key(input, "npub")?;
    XOnlyPublicKey::from_slice(&bytes)
        .map_err(|_| ToolkitError::NostrKeyParse("not a valid secp256k1 x-only public key".into()))
}

/// Decode an `nsec1…` (NIP-19 bech32) or 64-hex string into a secret key.
pub fn decode_nsec(input: &str) -> Result<SecretKey, ToolkitError> {
    let bytes = decode_nostr_key(input, "nsec")?;
    SecretKey::from_slice(&bytes)
        .map_err(|_| ToolkitError::NostrKeyParse("not a valid secp256k1 secret key".into()))
}

/// Shared decode: 64-hex OR NIP-19 bech32 (HRP-checked) → 32 zeroizing bytes.
fn decode_nostr_key(input: &str, expected_hrp: &str) -> Result<Zeroizing<Vec<u8>>, ToolkitError> {
    let trimmed = input.trim();
    if trimmed.len() == 64 && trimmed.bytes().all(|b| b.is_ascii_hexdigit()) {
        let v = hex::decode(trimmed)
            .map_err(|e| ToolkitError::NostrKeyParse(format!("invalid hex key: {e}")))?;
        return Ok(Zeroizing::new(v));
    }
    let (hrp, data) = bitcoin::bech32::decode(trimmed)
        .map_err(|e| ToolkitError::NostrKeyParse(format!("invalid bech32 nostr key: {e}")))?;
    let expected = bitcoin::bech32::Hrp::parse(expected_hrp).expect("static nostr HRP is valid");
    if hrp != expected {
        return Err(ToolkitError::NostrKeyParse(format!(
            "expected an '{expected_hrp}' key but got HRP '{hrp}'"
        )));
    }
    if data.len() != 32 {
        return Err(ToolkitError::NostrKeyParse(format!(
            "{expected_hrp} key must decode to 32 bytes; got {}",
            data.len()
        )));
    }
    Ok(Zeroizing::new(data))
}

#[cfg(test)]
mod normalize_tests {
    use super::*;

    fn secp() -> Secp256k1<bitcoin::secp256k1::All> { Secp256k1::new() }

    #[test]
    fn normalized_secret_always_has_even_y_pubkey() {
        for seed in 1u8..=20 {
            let mut bytes = [0u8; 32];
            bytes[31] = seed;
            let sk = SecretKey::from_slice(&bytes).unwrap();
            let (xonly_before, parity_before) = sk.x_only_public_key(&secp());
            let (norm, negated) = normalize_to_even_y(&secp(), sk);
            let (xonly_after, parity_after) = norm.x_only_public_key(&secp());
            assert_eq!(parity_after, Parity::Even, "seed {seed}: not even-y after normalize");
            assert_eq!(xonly_before, xonly_after, "seed {seed}: x-only changed");
            assert_eq!(negated, parity_before == Parity::Odd, "seed {seed}: negate flag wrong");
        }
    }
}

#[cfg(test)]
mod decode_tests {
    use super::*;

    // NIP-19 spec vectors. NOTE: the npub and nsec below are DISTINCT keys
    // (not a keypair); each bech32↔hex row is internally consistent, which is
    // all these decode tests assert.
    const NPUB: &str = "npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg";
    const PUB_HEX: &str = "7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e";
    const NSEC: &str = "nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5";
    const SEC_HEX: &str = "67dea2ed018072d675f5415ecfaed7d2597555e202d85b3d65ea4e58d2d92ffa";

    #[test]
    fn npub_bech32_decodes_to_expected_xonly() {
        assert_eq!(decode_npub(NPUB).unwrap().to_string(), PUB_HEX);
    }
    #[test]
    fn npub_hex_decodes_equal_to_bech32() {
        assert_eq!(decode_npub(PUB_HEX).unwrap(), decode_npub(NPUB).unwrap());
    }
    #[test]
    fn nsec_bech32_decodes_to_expected_scalar() {
        assert_eq!(hex::encode(decode_nsec(NSEC).unwrap().secret_bytes()), SEC_HEX);
    }
    #[test]
    fn wrong_hrp_is_refused() {
        assert!(matches!(decode_nsec(NPUB), Err(ToolkitError::NostrKeyParse(_))));
    }
    #[test]
    fn bad_bech32_is_refused() {
        assert!(matches!(decode_npub("npub1notvalid"), Err(ToolkitError::NostrKeyParse(_))));
    }
}
