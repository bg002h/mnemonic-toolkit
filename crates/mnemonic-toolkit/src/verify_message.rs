//! `mnemonic verify-message` core — VERIFY-ONLY (no signing) message-signature
//! verification. PUBLIC operation: no key material, no secrets, no mlock.
//! Binary-private (returns `crate::error::ToolkitError`).
//!
//! **Address-type partition (R0 C2):** `bitcoin 0.32`'s `is_signed_by_address`
//! supports only P2PKH; the `bip322` crate covers P2WPKH/P2SH-P2WPKH/P2TR and
//! refuses P2PKH. The two are exact complements:
//!   - `legacy`  → P2PKH ("Bitcoin Signed Message" / `signmessage` RPC format).
//!   - `bip322`  → P2WPKH / P2SH-P2WPKH / P2TR (BIP-322 *simple* encoding).
//!   - `auto`    → P2PKH ⇒ legacy, else ⇒ bip322.

use crate::error::ToolkitError;
use bitcoin::address::{Address, AddressType, NetworkUnchecked};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::sign_message::{signed_msg_hash, MessageSignature};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SigFormat {
    Legacy,
    Bip322,
    Auto,
}

#[derive(Debug)]
pub(crate) struct VerifyOutcome {
    pub valid: bool,
    /// Which verification path produced the result: "legacy" | "bip322".
    pub format_matched: &'static str,
}

fn parse_addr(address: &str) -> Result<Address, ToolkitError> {
    let a: Address<NetworkUnchecked> = address
        .trim()
        .parse()
        .map_err(|e| ToolkitError::VerifyMessage(format!("invalid address: {e}")))?;
    Ok(a.assume_checked())
}

/// Legacy "Bitcoin Signed Message" verify — P2PKH ONLY (bitcoin 0.32 limit).
fn verify_legacy(address: &str, message: &str, signature: &str) -> Result<bool, ToolkitError> {
    let addr = parse_addr(address)?;
    if addr.address_type() != Some(AddressType::P2pkh) {
        return Err(ToolkitError::VerifyMessage(
            "legacy signmessage verification is P2PKH-only; use --format bip322 (or auto) \
             for segwit/taproot addresses"
                .into(),
        ));
    }
    let sig = MessageSignature::from_base64(signature).map_err(|e| {
        ToolkitError::VerifyMessage(format!(
            "signature is not a valid base64 recoverable (65-byte) signature: {e}"
        ))
    })?;
    let digest = signed_msg_hash(message);
    let secp = Secp256k1::verification_only();
    sig.is_signed_by_address(&secp, &addr, digest)
        .map_err(|e| ToolkitError::VerifyMessage(format!("legacy verify failed: {e}")))
}

/// BIP-322 *simple* verify — P2WPKH / P2SH-P2WPKH / P2TR (crate refuses P2PKH).
///
/// The pinned `bip322 0.0.10` crate can **panic** on adversarial input — e.g. a
/// P2SH address whose witness carries a valid *uncompressed* pubkey reaches
/// `wpubkey_hash().unwrap()` (`bip322/src/verify.rs:168`). We take untrusted
/// public input, so we isolate any panic with `catch_unwind` and surface it as a
/// clean error instead of crashing (exit 101). The default panic hook is
/// silenced only around the call so the crate's internal panic text does not
/// leak to stderr; the hook is restored immediately after. (In the CLI this runs
/// single-threaded; the catch is also exercised by a unit regression test.)
fn verify_bip322(address: &str, message: &str, signature: &str) -> Result<bool, ToolkitError> {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        bip322::verify_simple_encoded(address.trim(), message, signature).is_ok()
    }));
    std::panic::set_hook(prev_hook);
    outcome.map_err(|_| {
        ToolkitError::VerifyMessage(
            "BIP-322 verification could not be performed for this address/signature \
             (malformed or unsupported witness)"
                .into(),
        )
    })
}

pub(crate) fn verify_message(
    address: &str,
    message: &str,
    signature: &str,
    fmt: SigFormat,
) -> Result<VerifyOutcome, ToolkitError> {
    let is_p2pkh = parse_addr(address)?.address_type() == Some(AddressType::P2pkh);
    match fmt {
        SigFormat::Legacy => Ok(VerifyOutcome {
            valid: verify_legacy(address, message, signature)?,
            format_matched: "legacy",
        }),
        SigFormat::Bip322 => Ok(VerifyOutcome {
            valid: verify_bip322(address, message, signature)?,
            format_matched: "bip322",
        }),
        SigFormat::Auto if is_p2pkh => Ok(VerifyOutcome {
            valid: verify_legacy(address, message, signature)?,
            format_matched: "legacy",
        }),
        SigFormat::Auto => Ok(VerifyOutcome {
            valid: verify_bip322(address, message, signature)?,
            format_matched: "bip322",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── BIP-322 simple vectors (external oracle: the bip322 crate's own
    //    SEGWIT_ADDRESS test = the BIP-322 mediawiki vectors). ──
    const SEGWIT_ADDRESS: &str = "bc1q9vza2e8x573nczrlzms0wvx3gsqjx7vavgkx0l";
    const SIG_HELLO_WORLD: &str = "AkcwRAIgZRfIY3p7/DoVTty6YZbWS71bc5Vct9p9Fia83eRmw2QCICK/ENGfwLtptFluMGs2KsqoNSk89pO7F29zJLUx9a/sASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViHI=";
    const SIG_EMPTY: &str = "AkcwRAIgM2gBAQqvZX15ZiysmKmQpDrG83avLIT492QBzLnQIxYCIBaTpOaD20qRlEylyxFSeEA2ba9YOixpX8z46TSDtS40ASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViHI=";

    #[test]
    fn bip322_hello_world_valid() {
        let o = verify_message(SEGWIT_ADDRESS, "Hello World", SIG_HELLO_WORLD, SigFormat::Bip322).unwrap();
        assert!(o.valid);
        assert_eq!(o.format_matched, "bip322");
    }

    #[test]
    fn bip322_empty_message_valid() {
        assert!(verify_message(SEGWIT_ADDRESS, "", SIG_EMPTY, SigFormat::Bip322).unwrap().valid);
    }

    #[test]
    fn bip322_wrong_message_invalid() {
        let o = verify_message(SEGWIT_ADDRESS, "Goodbye World", SIG_HELLO_WORLD, SigFormat::Bip322).unwrap();
        assert!(!o.valid);
    }

    #[test]
    fn auto_dispatches_segwit_to_bip322() {
        let o = verify_message(SEGWIT_ADDRESS, "Hello World", SIG_HELLO_WORLD, SigFormat::Auto).unwrap();
        assert!(o.valid);
        assert_eq!(o.format_matched, "bip322");
    }

    // ── Legacy P2PKH: self-generate a deterministic (RFC6979) vector via the
    //    bitcoin crate's signing primitive (signing happens ONLY in tests; the
    //    toolkit binary never signs). This exercises the legacy dispatch +
    //    P2PKH gating + base64 round-trip + digest wiring end-to-end. ──
    fn make_legacy_p2pkh_vector(message: &str) -> (String, String) {
        use bitcoin::hashes::Hash;
        use bitcoin::secp256k1::{Message, SecretKey};
        use bitcoin::{Network, PublicKey};
        let secp = Secp256k1::new();
        let sk = SecretKey::from_slice(&[0x11u8; 32]).unwrap();
        let pk = PublicKey::new(sk.public_key(&secp));
        let addr = Address::p2pkh(pk, Network::Bitcoin);
        let digest = signed_msg_hash(message);
        let secp_msg = Message::from_digest(digest.to_byte_array());
        let rec = secp.sign_ecdsa_recoverable(&secp_msg, &sk);
        let sig = MessageSignature::new(rec, true);
        (addr.to_string(), sig.to_base64())
    }

    #[test]
    fn legacy_p2pkh_valid() {
        let (addr, sig) = make_legacy_p2pkh_vector("Hello World");
        let o = verify_message(&addr, "Hello World", &sig, SigFormat::Legacy).unwrap();
        assert!(o.valid);
        assert_eq!(o.format_matched, "legacy");
    }

    #[test]
    fn legacy_p2pkh_tampered_message_invalid() {
        let (addr, sig) = make_legacy_p2pkh_vector("Hello World");
        assert!(!verify_message(&addr, "Tampered", &sig, SigFormat::Legacy).unwrap().valid);
    }

    #[test]
    fn auto_dispatches_p2pkh_to_legacy() {
        let (addr, sig) = make_legacy_p2pkh_vector("Hello World");
        let o = verify_message(&addr, "Hello World", &sig, SigFormat::Auto).unwrap();
        assert!(o.valid);
        assert_eq!(o.format_matched, "legacy");
    }

    #[test]
    fn legacy_format_on_segwit_address_errors() {
        // --format legacy on a non-P2PKH address → honest error (not a misleading
        // "bad base64"). SIG content irrelevant; the gate fires on address type.
        let err = verify_message(SEGWIT_ADDRESS, "Hello World", SIG_HELLO_WORLD, SigFormat::Legacy)
            .unwrap_err();
        assert!(err.message().contains("P2PKH-only"));
    }

    #[test]
    fn malformed_address_errors() {
        assert!(verify_message("not-an-address", "m", "AAAA", SigFormat::Auto).is_err());
    }

    // C1 regression: the pinned bip322 0.0.10 crate panics (wpubkey_hash().unwrap())
    // on a P2SH address whose BIP-322 witness carries a 65-byte UNCOMPRESSED pubkey.
    // The toolkit takes untrusted public input, so this MUST be a clean error/false,
    // never a process crash. We craft a 2-item witness [dummy-sig, uncompressed-key].
    fn craft_uncompressed_p2sh_sig() -> String {
        use base64::Engine;
        use bitcoin::consensus::encode::serialize;
        use bitcoin::secp256k1::{Message as SecpMsg, SecretKey};
        use bitcoin::Witness;
        let secp = Secp256k1::new();
        let sk = SecretKey::from_slice(&[0x22u8; 32]).unwrap();
        // A VALID uncompressed pubkey (real curve point) — passes
        // PublicKey::from_slice + the pubkey-equality guard, but wpubkey_hash()
        // errors on uncompressed → the crate's `.unwrap()` panics.
        let uncompressed = sk.public_key(&secp).serialize_uncompressed().to_vec();
        // A structurally-valid DER signature of total length 71 or 72 (so it
        // passes the crate's length + from_der + SIGHASH_ALL gates and REACHES
        // the panic at verify.rs:168). Grind the digest for the right length.
        let mut der_plus_sighash = Vec::new();
        for n in 0u32.. {
            let digest = bitcoin::hashes::sha256::Hash::hash(&n.to_le_bytes());
            use bitcoin::hashes::Hash as _;
            let sig = secp.sign_ecdsa(&SecpMsg::from_digest(digest.to_byte_array()), &sk);
            let mut der = sig.serialize_der().to_vec();
            der.push(0x01); // SIGHASH_ALL
            if matches!(der.len(), 71 | 72) {
                der_plus_sighash = der;
                break;
            }
        }
        let mut w = Witness::new();
        w.push(der_plus_sighash);
        w.push(uncompressed);
        base64::engine::general_purpose::STANDARD.encode(serialize(&w))
    }

    #[test]
    fn p2sh_uncompressed_pubkey_does_not_panic() {
        // P2SH (non-segwit) mainnet address; the address layer can't see the
        // redeem script, so it routes to bip322 under auto/bip322.
        let addr = "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy";
        let sig = craft_uncompressed_p2sh_sig();
        // Must return (Err or Ok(valid=false)) — reaching this assert at all
        // proves no panic unwound out of the crate.
        let r = verify_message(addr, "hello", &sig, SigFormat::Bip322);
        assert!(r.is_err() || !r.unwrap().valid);
        let r2 = verify_message(addr, "hello", &sig, SigFormat::Auto);
        assert!(r2.is_err() || !r2.unwrap().valid);
    }
}
