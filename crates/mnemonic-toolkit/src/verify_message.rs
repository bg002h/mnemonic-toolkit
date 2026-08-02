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

/// Does the public key carried in the witness actually control `addr`?
///
/// **Security gate (responsible disclosure, 2026-08-02).** `bip322` 0.0.6–0.0.10
/// lifted the ECDSA public key straight out of `witness[1]` and never checked it
/// against the challenged address: the one equality check (0.0.10
/// `verify.rs:134`) compared that key with *itself*, and the P2SH arm discarded
/// `script_hash` entirely (`verify.rs:87`), deriving the sighash script from the
/// attacker's own key (`verify.rs:168`). Any keypair therefore produced a proof
/// that verified against anyone's P2WPKH or P2SH address. We re-derive the
/// binding here and refuse to call the crate unless it holds.
///
/// **Retained after the upstream fix, deliberately.** The pin is now 0.0.11,
/// which binds the key itself, so this gate is no longer the only thing standing
/// between a forgery and a VALID verdict. It stays because it is independent of
/// the dependency: it keeps the invariant enforced in code we own and test, so a
/// future bump, a `[patch]`, or a vendor-tree drift cannot silently reintroduce
/// the flaw. Its regression cells are the oracle for exactly that. The cost is
/// one hash and one script comparison per verify.
///
/// Returns `None` for address types the crate binds correctly by construction —
/// P2TR takes its x-only key from the address's own witness program — so those
/// keep flowing to the crate unchanged.
fn bip322_key_binding_holds(addr: &Address, signature: &str) -> Option<bool> {
    use base64::Engine as _;
    use bitcoin::address::AddressData;
    use bitcoin::consensus::Decodable;
    use bitcoin::{CompressedPublicKey, ScriptBuf, Witness};

    let is_p2sh = match addr.to_address_data() {
        AddressData::P2sh { .. } => true,
        AddressData::Segwit { witness_program } => {
            if witness_program.version().to_num() != 0 || witness_program.program().len() != 20 {
                return None; // P2TR / future witness versions — already bound.
            }
            false
        }
        _ => return None, // P2PKH / unsupported — the crate rejects these.
    };

    // Recover the claimed key. Any structural problem is a failed binding, not
    // a pass: this gate is fail-closed.
    let claimed = (|| {
        let raw = base64::engine::general_purpose::STANDARD
            .decode(signature.trim())
            .ok()?;
        let witness =
            Witness::consensus_decode_from_finite_reader(&mut bitcoin::io::Cursor::new(raw))
                .ok()?;
        // BIP-322 simple key-path spend is exactly [signature, public key].
        if witness.len() != 2 {
            return None;
        }
        let raw_key = witness.nth(1)?;
        // Require the COMPRESSED encoding EXPLICITLY. `CompressedPublicKey`
        // names its *output* serialization, not its accepted input: `from_slice`
        // is a bare delegation to `secp256k1::PublicKey::from_slice`
        // (`vendor/bitcoin/src/crypto/key.rs:323-325`) and accepts 65-byte
        // uncompressed and hybrid keys too. Segwit v0 spends require the
        // compressed form, so this rejects nothing a real wallet can produce —
        // and it is what actually forecloses the crate's
        // `wpubkey_hash().unwrap()` panic (`vendor/bip322/src/verify.rs:168`),
        // which fires when the witness carries the address owner's OWN key in
        // uncompressed form (the binding matches, since `wpubkey_hash()` always
        // hashes the compressed serialization).
        if raw_key.len() != 33 {
            return None;
        }
        CompressedPublicKey::from_slice(raw_key).ok()
    })();
    let Some(pk) = claimed else {
        return Some(false);
    };

    // The key must reproduce the exact scriptPubKey the address commits to.
    let wpkh = pk.wpubkey_hash();
    let expected = if is_p2sh {
        ScriptBuf::new_p2sh(&ScriptBuf::new_p2wpkh(&wpkh).script_hash())
    } else {
        ScriptBuf::new_p2wpkh(&wpkh)
    };
    Some(expected == addr.script_pubkey())
}

/// BIP-322 *simple* verify — P2WPKH / P2SH-P2WPKH / P2TR (crate refuses P2PKH).
///
/// The input here is attacker-supplied by definition, so the crate call is
/// isolated with `catch_unwind` and any unwind surfaced as a clean error rather
/// than an exit-101 abort. The default panic hook is silenced only around the
/// call so crate-internal panic text does not leak to stderr; the hook is
/// restored immediately after. (In the CLI this runs single-threaded.)
///
/// The superseded `bip322 0.0.10` had two reachable panics: `wpubkey_hash()
/// .unwrap()` on an uncompressed witness key (its `verify.rs:168`), and an
/// unguarded `witness.to_vec()[0]` on the P2TR arm, which — unlike the v0 and
/// P2SH arms — carried no witness-length guard, so a 0-item witness indexed an
/// empty vec (its `verify.rs:212`). BOTH are fixed in the pinned 0.0.11, and
/// `historically_panicking_inputs_are_handled_cleanly` pins that they stay
/// fixed. The guard is retained deliberately: it costs nothing on the success
/// path and still covers panics nobody has enumerated. Do NOT retire it on the
/// strength of those two cells alone.
///
/// NOTE: the global panic-hook swap is NOT thread-safe — if a future feature
/// ever calls this from multiple threads (e.g. batch verification), the hook
/// swap races. The toolkit is single-threaded today; revisit if that changes.
fn verify_bip322(address: &str, message: &str, signature: &str) -> Result<bool, ToolkitError> {
    // MUST precede the crate call — see `bip322_key_binding_holds`. Without it
    // the crate accepts a signature from a key that does not control `address`.
    if bip322_key_binding_holds(&parse_addr(address)?, signature) == Some(false) {
        return Ok(false);
    }
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        bip322::verify_simple_encoded(address.trim(), message, signature.trim()).is_ok()
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
    fn signature_surrounding_whitespace_tolerated() {
        // A signature pasted or read from a file commonly carries a trailing
        // newline. The address has always been trimmed; the signature was not,
        // so a genuine proof reported INVALID purely over transport whitespace.
        let padded = format!("  {}\n", SIG_HELLO_WORLD);
        for f in [SigFormat::Bip322, SigFormat::Auto] {
            assert!(
                verify_message(SEGWIT_ADDRESS, "Hello World", &padded, f)
                    .unwrap()
                    .valid,
                "genuine proof rejected over surrounding whitespace"
            );
        }
    }

    #[test]
    fn bip322_hello_world_valid() {
        let o = verify_message(
            SEGWIT_ADDRESS,
            "Hello World",
            SIG_HELLO_WORLD,
            SigFormat::Bip322,
        )
        .unwrap();
        assert!(o.valid);
        assert_eq!(o.format_matched, "bip322");
    }

    #[test]
    fn bip322_empty_message_valid() {
        assert!(
            verify_message(SEGWIT_ADDRESS, "", SIG_EMPTY, SigFormat::Bip322)
                .unwrap()
                .valid
        );
    }

    #[test]
    fn bip322_wrong_message_invalid() {
        let o = verify_message(
            SEGWIT_ADDRESS,
            "Goodbye World",
            SIG_HELLO_WORLD,
            SigFormat::Bip322,
        )
        .unwrap();
        assert!(!o.valid);
    }

    #[test]
    fn auto_dispatches_segwit_to_bip322() {
        let o = verify_message(
            SEGWIT_ADDRESS,
            "Hello World",
            SIG_HELLO_WORLD,
            SigFormat::Auto,
        )
        .unwrap();
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
        assert!(
            !verify_message(&addr, "Tampered", &sig, SigFormat::Legacy)
                .unwrap()
                .valid
        );
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
        let err = verify_message(
            SEGWIT_ADDRESS,
            "Hello World",
            SIG_HELLO_WORLD,
            SigFormat::Legacy,
        )
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

    // ── Key-binding regression (responsible disclosure, 2026-08-02) ──────────
    // Pinned `bip322 0.0.10` never binds the witness public key to the
    // challenged address for P2WPKH / P2SH-P2WPKH: `verify_full` lifts the key
    // out of `witness[1]` (attacker-controlled) and the only equality check
    // (`verify.rs:134`) compares that key against itself. So ANY keypair can
    // produce a proof that verifies against SOMEONE ELSE'S address. P2TR is
    // unaffected — its x-only key is read from the address's witness program.
    //
    // The vulnerable `sign_simple_encoded` likewise never checked key↔address,
    // so producing these was just "sign the victim's address with the attacker's
    // key". They are FROZEN here as literal attack artifacts rather than
    // regenerated at test time: the signer is not the subject under test, and
    // pinning the bytes keeps these regressions independent of any future
    // `bip322` signing-API change (0.0.11 already changed that signature). All
    // four were generated with the vulnerable 0.0.10 signer, message
    // "Hello World", attacker key = [0x42; 32].

    /// Forgery against `SEGWIT_ADDRESS` (P2WPKH) by a key that does not own it.
    const FORGED_P2WPKH: &str = "AkgwRQIhAJTBOPamjqAO8JElIC5qar0xByZiH+vUa5TAeoPEwE+rAiABXk+5/GhuPg+Mc3spH+eLoSR/ln/EaNNG7+wXMIrGigEhAyRlPqxDRIgALMBrv7fxD+GJkeNfn+QwLb6m0jU9wKsc";
    /// Real mainnet P2SH address the attacker does not control, + its forgery.
    const P2SH_ADDRESS: &str = "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy";
    const FORGED_P2SH: &str = "AkgwRQIhAIqaalDGdYNZPLR61ST1AySk8LEJuYhNvMzHBCblydsgAiAV9e9FL69lt1KazEWAr6wjrDegUTDo/c0TNehBDflS1gEhAyRlPqxDRIgALMBrv7fxD+GJkeNfn+QwLb6m0jU9wKsc";
    /// P2SH-P2WPKH address of key [0x33; 32] + a GENUINE proof from that key.
    const GENUINE_P2SH_ADDRESS: &str = "3BgEEMQZk9QbuMwG5VUj2J22X8JRzzSAFu";
    const GENUINE_P2SH_SIG: &str = "AkgwRQIhAM9j0r240qaiDvf7v5w38BJvA6nVaTOMXnr1BBETyMTCAiBzZnpzi05IudsuFG9rLReAR4Furw7u4SEe+7Jk1UJNGgEhAjxyrdtP3wmvlPDJTX/pKjhqfnDPih2FkWOGuyU1x7Gx";
    /// P2TR address of key [0x11; 32] + an attempted forgery against it.
    const P2TR_ADDRESS: &str = "bc1p9fjtrm3nwhemkjek0wxtswz2glmneu33w9lcylrvd7alttk0psmq6cnwza";
    const FORGED_P2TR: &str = "AUGURunomtEqUh6lRdByf2CnBTbKAsC2fBbgoRkmUfWLayUjs4Me2O2m/5P9oNXkLDcLgRlUCOuCDqg6ZUEoieFPAQ==";

    #[test]
    fn p2wpkh_foreign_key_forgery_rejected() {
        // SEGWIT_ADDRESS is the BIP-322 vector address; the attacker does not
        // hold its key. Both dispatch paths must refuse.
        for f in [SigFormat::Bip322, SigFormat::Auto] {
            let o = verify_message(SEGWIT_ADDRESS, "Hello World", FORGED_P2WPKH, f).unwrap();
            assert!(
                !o.valid,
                "FORGERY ACCEPTED: unrelated key verified against a P2WPKH address"
            );
        }
    }

    #[test]
    fn p2sh_p2wpkh_foreign_key_forgery_rejected() {
        for f in [SigFormat::Bip322, SigFormat::Auto] {
            let r = verify_message(P2SH_ADDRESS, "Hello World", FORGED_P2SH, f).unwrap();
            assert!(
                !r.valid,
                "FORGERY ACCEPTED: unrelated key verified against a P2SH address"
            );
        }
    }

    #[test]
    fn p2sh_p2wpkh_genuine_signature_still_valid() {
        // Positive control for the gate's P2SH arm: the key that DOES control
        // the address must still verify (guards against over-rejection).
        let o = verify_message(
            GENUINE_P2SH_ADDRESS,
            "Hello World",
            GENUINE_P2SH_SIG,
            SigFormat::Bip322,
        )
        .unwrap();
        assert!(o.valid, "genuine P2SH-P2WPKH proof was rejected");
        // ...and the same address must still reject a tampered message.
        assert!(
            !verify_message(
                GENUINE_P2SH_ADDRESS,
                "Goodbye World",
                GENUINE_P2SH_SIG,
                SigFormat::Bip322
            )
            .unwrap()
            .valid
        );
    }

    #[test]
    fn p2tr_foreign_key_forgery_rejected() {
        // Control: taproot binds the key via the address itself, so the same
        // attack must already fail. Guards against a fix that regresses P2TR.
        let r = verify_message(P2TR_ADDRESS, "Hello World", FORGED_P2TR, SigFormat::Bip322);
        assert!(r.is_err() || !r.unwrap().valid, "FORGERY ACCEPTED on P2TR");
    }

    /// The P2SH-P2WPKH address the `craft_uncompressed_p2sh_sig` key actually
    /// controls. Its `wpubkey_hash` MATCHES the crafted witness key (that hash
    /// is always over the compressed form), so the binding alone does not stop
    /// this input — only the explicit 33-byte check does. Pre-fix this reached
    /// the crate and panicked.
    fn uncompressed_vector_owner_address() -> String {
        use bitcoin::secp256k1::SecretKey;
        use bitcoin::{CompressedPublicKey, Network, PrivateKey};
        let secp = Secp256k1::new();
        let pk = PrivateKey::new(
            SecretKey::from_slice(&[0x22u8; 32]).unwrap(),
            Network::Bitcoin,
        );
        let cpk = CompressedPublicKey::from_private_key(&secp, &pk).unwrap();
        Address::p2shwpkh(&cpk, Network::Bitcoin).to_string()
    }

    #[test]
    fn uncompressed_witness_key_rejected() {
        let sig = craft_uncompressed_p2sh_sig();
        // (a) A P2SH address the key does not control — binding fails outright.
        for f in [SigFormat::Bip322, SigFormat::Auto] {
            let r = verify_message("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy", "hello", &sig, f);
            assert!(r.is_err() || !r.unwrap().valid);
        }
        // (b) The address the key DOES control. Reaching these asserts at all
        // proves no panic unwound out of the crate.
        let owner = uncompressed_vector_owner_address();
        for f in [SigFormat::Bip322, SigFormat::Auto] {
            assert!(
                !verify_message(&owner, "hello", &sig, f).unwrap().valid,
                "an uncompressed witness key must never verify"
            );
        }
    }

    #[test]
    fn historically_panicking_inputs_are_handled_cleanly() {
        // Two inputs CRASHED the superseded 0.0.10 (absorbed at the time only by
        // `verify_bip322`'s catch_unwind):
        //   1. P2SH-P2WPKH whose witness carries the OWNER's key in uncompressed
        //      form → `wpubkey_hash().unwrap()` (0.0.10 verify.rs:168). Note the
        //      binding alone does NOT stop this — `wpubkey_hash()` always hashes
        //      the compressed form — which is why the gate's 33-byte check
        //      exists.
        //   2. P2TR with a 0-item witness → that arm carried no witness-length
        //      guard, so `witness.to_vec()[0]` indexed an empty vec (0.0.10
        //      verify.rs:212). The gate passes P2TR through by design, so this
        //      one reached the crate directly.
        // Both are fixed in the pinned 0.0.11. This pins that they STAY fixed:
        // an accidental downgrade or a vendor-tree drift turns it red.
        let owner = uncompressed_vector_owner_address();
        let sig = craft_uncompressed_p2sh_sig();
        for f in [SigFormat::Bip322, SigFormat::Auto] {
            assert!(
                !verify_message(&owner, "hello", &sig, f).unwrap().valid,
                "uncompressed owner key must be a clean INVALID, not a panic"
            );
            // "AA==" is base64 of a serialized 0-item witness.
            assert!(
                !verify_message(P2TR_ADDRESS, "hello", "AA==", f)
                    .unwrap()
                    .valid,
                "empty-witness P2TR must be a clean INVALID, not a panic"
            );
        }
    }
}
