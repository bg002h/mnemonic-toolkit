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

use crate::cmd::convert::ScriptType;
use crate::error::ToolkitError;
use crate::network::CliNetwork;
use bitcoin::secp256k1::{Parity, PublicKey, Secp256k1, SecretKey, Signing, Verification, XOnlyPublicKey};
use bitcoin::{Address, CompressedPublicKey};
use std::str::FromStr;
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

/// Even-y compressed pubkey (`02‖x`) from an x-only key.
pub fn even_y_compressed(xonly: XOnlyPublicKey) -> CompressedPublicKey {
    CompressedPublicKey(PublicKey::from_x_only_public_key(xonly, Parity::Even))
}

/// Render the Bitcoin address for an x-only nostr key under `script_type`.
pub fn address_for<C: Verification>(
    secp: &Secp256k1<C>,
    xonly: XOnlyPublicKey,
    script_type: ScriptType,
    network: CliNetwork,
) -> String {
    let compressed = even_y_compressed(xonly);
    match script_type {
        ScriptType::P2pkh => Address::p2pkh(compressed, network.network_kind()).to_string(),
        ScriptType::P2wpkh => Address::p2wpkh(&compressed, network.known_hrp()).to_string(),
        ScriptType::P2shP2wpkh => Address::p2shwpkh(&compressed, network.network_kind()).to_string(),
        ScriptType::P2tr => Address::p2tr(secp, xonly, None, network.known_hrp()).to_string(),
    }
}

/// Build the checksummed Bitcoin descriptor wrapping the nostr key.
pub fn descriptor_for(xonly: XOnlyPublicKey, script_type: ScriptType) -> Result<String, ToolkitError> {
    let body = match script_type {
        ScriptType::P2tr => format!("tr({xonly})"),
        ScriptType::P2wpkh => format!("wpkh({})", even_y_compressed(xonly)),
        ScriptType::P2pkh => format!("pkh({})", even_y_compressed(xonly)),
        ScriptType::P2shP2wpkh => format!("sh(wpkh({}))", even_y_compressed(xonly)),
    };
    let desc = miniscript::Descriptor::<miniscript::DescriptorPublicKey>::from_str(&body)
        .map_err(|e| ToolkitError::NostrKeyParse(format!("descriptor build failed: {e}")))?;
    Ok(desc.to_string()) // Display appends the BIP-380 `#checksum`
}

/// Plain compressed WIF for the (already even-y-normalized) secret.
pub fn wif_for(secret: &SecretKey, network: CliNetwork) -> String {
    bitcoin::PrivateKey { compressed: true, network: network.network_kind(), inner: *secret }.to_wif()
}

/// Electrum imported-key script-type prefix, per Electrum's `WIF_SCRIPT_TYPES`
/// (`bitcoin.py`): nested-segwit is `p2wpkh-p2sh`; taproot is `p2tr` (newer Electrum).
pub fn electrum_prefix(script_type: ScriptType) -> &'static str {
    match script_type {
        ScriptType::P2pkh => "p2pkh:",
        ScriptType::P2wpkh => "p2wpkh:",
        ScriptType::P2shP2wpkh => "p2wpkh-p2sh:",
        ScriptType::P2tr => "p2tr:",
    }
}

#[cfg(test)]
mod derive_tests {
    use super::*;
    use crate::cmd::convert::ScriptType;
    use crate::network::CliNetwork;
    use std::str::FromStr;

    fn secp() -> Secp256k1<bitcoin::secp256k1::All> { Secp256k1::new() }

    // CRUX: the WIF derived from an nsec must control the key behind the npub.
    // Iterate scalars to hit both even-y and odd-y originals; an odd-y seed
    // fails the parity/x-only asserts below unless `wif_for` encoded `n−d`
    // (not raw `d`). `any_negated` guards against a seed range that never
    // exercises the negate path (which would give false confidence).
    #[test]
    fn wif_controls_the_npub_address_all_script_types() {
        let secp = secp();
        let mut any_negated = false;
        for seed in 1u8..=10 {
            let mut bytes = [0u8; 32];
            bytes[31] = seed;
            let sk = SecretKey::from_slice(&bytes).unwrap();
            let (xonly, _) = sk.x_only_public_key(&secp); // published npub key
            let (norm, negated) = normalize_to_even_y(&secp, sk);
            any_negated |= negated;
            // The WIF key, at its ACTUAL parity, must be the even-y point whose
            // x-only equals the npub — i.e. it controls the npub-derived address.
            let wif = wif_for(&norm, CliNetwork::Mainnet);
            let pk = bitcoin::PrivateKey::from_wif(&wif).unwrap();
            let (wif_xonly, wif_parity) = pk.inner.x_only_public_key(&secp);
            assert_eq!(wif_parity, Parity::Even, "seed {seed}: WIF key is odd-y");
            assert_eq!(wif_xonly, xonly, "seed {seed}: WIF x-only != npub");
            // Smoke: address_for renders a non-empty address for every type.
            for st in [ScriptType::P2pkh, ScriptType::P2wpkh, ScriptType::P2shP2wpkh, ScriptType::P2tr] {
                assert!(
                    !address_for(&secp, xonly, st, CliNetwork::Mainnet).is_empty(),
                    "seed {seed} {st:?}: empty address"
                );
            }
        }
        assert!(any_negated, "seed range never exercised the even-y negate path");
    }

    #[test]
    fn descriptor_has_checksum_and_round_trips() {
        let xonly = decode_npub("npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg").unwrap();
        let tr = descriptor_for(xonly, ScriptType::P2tr).unwrap();
        assert!(tr.starts_with("tr(") && tr.contains('#'), "got {tr}");
        let wpkh = descriptor_for(xonly, ScriptType::P2wpkh).unwrap();
        // even-y compressed form is always `02…` (never `03…`).
        assert!(wpkh.starts_with("wpkh(02"), "got {wpkh}");
        // miniscript must accept our own checksummed output (round-trip).
        assert!(miniscript::Descriptor::<miniscript::DescriptorPublicKey>::from_str(&tr).is_ok());
        assert!(miniscript::Descriptor::<miniscript::DescriptorPublicKey>::from_str(&wpkh).is_ok());
    }
}

#[cfg(test)]
mod decode_tests {
    use super::*;

    // NIP-19 spec vectors — the canonical matched keypair: the nsec scalar
    // 67dea2ed… normalized to even-y has pubkey x-only 7e7e9c42… == the npub.
    // Each bech32↔hex row is independently asserted by the decode tests.
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

#[cfg(test)]
mod cross_impl_fixture {
    use super::*;
    use crate::cmd::convert::ScriptType;
    use crate::network::CliNetwork;

    // Independent oracle: pure-Python secp256k1 + BIP-340 lift_x + BIP-341/86
    // taptweak + bech32/bech32m/base58check (NOT rust-bitcoin). Key = NIP-19
    // npub10elf… (x-only 7e7e9c42…df4e); even-y compressed = 02‖x. Regenerate
    // via tests/external/regen_nostr_vectors.md.
    const NPUB: &str = "npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg";
    const EXPECTED_P2PKH: &str = "16vqz4S2bJ8F4r1rSrGU3RxkUReZYrr7X3";
    const EXPECTED_P2WPKH: &str = "bc1qgyrepq5ukvwl7z7z5lk0066wx6vz75pn9ww6pv";
    const EXPECTED_P2SH: &str = "3546dKS2XmpDUbyQrA7zmrbE2fayRvHWyJ";
    const EXPECTED_P2TR: &str = "bc1pvvymzaajnverlq90cqupmtwep2txzarvvwqfs4p8jfvkepqaws5scnww04";

    #[test]
    fn pinned_addresses_match_independent_oracle() {
        let secp = Secp256k1::new();
        let xonly = decode_npub(NPUB).unwrap();
        assert_eq!(address_for(&secp, xonly, ScriptType::P2pkh, CliNetwork::Mainnet), EXPECTED_P2PKH);
        assert_eq!(address_for(&secp, xonly, ScriptType::P2wpkh, CliNetwork::Mainnet), EXPECTED_P2WPKH);
        assert_eq!(address_for(&secp, xonly, ScriptType::P2shP2wpkh, CliNetwork::Mainnet), EXPECTED_P2SH);
        assert_eq!(address_for(&secp, xonly, ScriptType::P2tr, CliNetwork::Mainnet), EXPECTED_P2TR);
    }
}
