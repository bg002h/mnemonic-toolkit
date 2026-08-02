//! `mnemonic verify-message` core — VERIFY-ONLY (no signing) message-signature
//! verification. PUBLIC operation: no key material, no secrets, no mlock.
//! Binary-private (returns `crate::error::ToolkitError`).
//!
//! **Address-type dispatch:** `bitcoin 0.32`'s `is_signed_by_address` supports
//! only P2PKH; the `bip322` crate covers the segwit/taproot forms.
//!   - `legacy`  → P2PKH ("Bitcoin Signed Message" / `signmessage` RPC format).
//!   - `bip322`  → BIP-322 *simple* encoding.
//!   - `auto`    → P2PKH ⇒ legacy, else ⇒ bip322.
//!
//! **The two are NO LONGER exact complements** (they were under `bip322`
//! 0.0.10, which covered exactly P2WPKH / P2SH-P2WPKH / P2TR and refused
//! P2PKH). 0.0.11 widened its verify surface with `p2wsh`, `p2sh_multisig` and
//! `p2pkh` arms. Observable effect here: a genuine **P2WSH multisig** proof now
//! verifies, where 0.0.10 returned `UnsupportedAddress` — see
//! `p2wsh_multisig_genuine_proof_verifies`. `auto` still routes P2PKH to the
//! legacy path, so that dispatch is unchanged.
//!
//! The P2SH-multisig and P2PKH arms are entered but can never succeed here:
//! both need a **scriptSig**, and the *simple* encoding carries none
//! (`create_to_sign` hard-codes `script_sig: ScriptBuf::new()`,
//! `vendor/bip322/src/util.rs:60`). `--format bip322` on a P2PKH address
//! reaches the crate's P2PKH arm and always returns `Invalid witness`.

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
/// Returns `None` for address types the crate binds correctly by construction,
/// which flow to it unchanged: **P2TR** (x-only key taken from the address's own
/// witness program) and **P2WSH** (bound by witness-script hash), plus P2PKH and
/// anything unsupported, which the crate rejects on its own.
fn bip322_key_binding_holds(addr: &Address, signature: &str) -> Option<bool> {
    use base64::Engine as _;
    use bitcoin::address::AddressData;
    use bitcoin::consensus::Decodable;
    use bitcoin::{CompressedPublicKey, ScriptBuf, Witness};

    let is_p2sh = match addr.to_address_data() {
        AddressData::P2sh { .. } => true,
        AddressData::Segwit { witness_program } => {
            if witness_program.version().to_num() != 0 || witness_program.program().len() != 20 {
                // v1-32 P2TR and v0-32 P2WSH, plus any future witness version.
                // The crate binds these itself — P2TR from the address's own
                // x-only key, P2WSH by the witness-script hash — and this gate
                // has no key to check against, so it abstains.
                return None;
            }
            false
        }
        _ => return None, // P2PKH / unsupported — not this gate's business.
    };

    // Recover the claimed key from the BIP-322 key-path witness shape.
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
        // compressed form, so this rejects nothing a real wallet can produce.
        //
        // It is also the gate's own guard against the P2SH-only
        // `wpubkey_hash().unwrap()` panic that 0.0.10 carried (0.0.10
        // `verify.rs:168`, inside its `if is_p2sh` sighash branch — that line
        // number does NOT refer to the vendored 0.0.11 tree). The binding alone
        // would not have stopped it: `wpubkey_hash()` always hashes the
        // compressed serialization, so the owner's own key in uncompressed form
        // still matched. The pinned 0.0.11 returns a clean
        // `.context(UncompressedPublicKey)` error there instead, so today this
        // is belt-and-braces — it holds the line if the crate ever regresses.
        if raw_key.len() != 33 {
            return None;
        }
        CompressedPublicKey::from_slice(raw_key).ok()
    })();
    let Some(pk) = claimed else {
        // Not a `[signature, 33-byte compressed key]` key-path spend → fail
        // closed, for BOTH address types.
        //
        // P2WPKH has exactly one valid witness shape, so that case is trivial.
        // P2SH looks ambiguous — 0.0.11 added P2SH-multisig and P2SH-P2WSH arms
        // — but rejecting here denies no genuine owner anything, because those
        // arms are unreachable-to-success on this call path: both require a
        // NON-EMPTY **scriptSig** — P2SH-multisig parses its redeem script and
        // signatures out of it (`vendor/bip322/src/verify.rs:535`), and
        // P2SH-P2WSH requires the scriptSig to equal `push_only_script(program)`
        // (`:457-460`), taking the witness script itself from the witness
        // (`:448`) — and the BIP-322 *simple* encoding cannot carry one.
        // `create_to_sign` hard-codes
        // `script_sig: ScriptBuf::new()` (`vendor/bip322/src/util.rs:60`) and
        // populates only the witness, so a simple-encoded P2SH-multisig or
        // P2SH-P2WSH proof always errors inside the crate regardless of this
        // gate. Staying closed also keeps the 33-byte check below load-bearing.
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

    // ── P2WSH: accept-surface widening introduced by the 0.0.11 bump ─────────
    // 0.0.10 refused P2WSH with `UnsupportedAddress`; 0.0.11 verifies it and
    // binds it by witness-script hash. The key-binding gate ABSTAINS for v0-32
    // programs (it has no single key to check), so these exercise the crate's
    // own binding through the toolkit's dispatch. 1-of-1 CHECKMULTISIG scripts;
    // victim key [0x77; 32], attacker key [0x42; 32].
    const P2WSH_ADDRESS: &str = "bc1ql84tvucmpz7wa0ccdmjjk7twhzgwm8xltf5ryffme053t0vvfeys9vlryg";
    const P2WSH_GENUINE_SIG: &str = "AwBHMEQCIGLPidfWp4H8TLeHn6wDJZOvVjED3X4GkpIt7UcdfMbwAiBMVCJO/UnGGG/Ftc0nl8xV6uQSNKz7U7HYiORDsHhiDAElUSEDeWLUWzjovPgvqO+oQyoB8gyaU+JMfT8R3xl8uOcJJtpRrg==";
    /// A structurally valid P2WSH proof the ATTACKER made for their OWN P2WSH
    /// address, presented against the victim's — the script hash cannot match.
    ///
    /// NOTE (honest scope): this constant is *over-determined*. It was signed
    /// over the attacker's own sighash, so the ECDSA check rejects it even if
    /// the witness-script-hash binding were removed. It is a genuine
    /// foreign-proof-rejection oracle, NOT an isolating oracle for the binding
    /// itself — `P2WSH_BINDING_FORGERY` below is that.
    const P2WSH_FORGED_SIG: &str = "AwBHMEQCID7T4RcHbWzOYsx4TTTB7JoTDuAKebWAqCa+nU/BURTqAiBme6Cv4HfhJovCz3uve2sKZaJYLGk5Kdxz+9mvWbfxDgElUSEDJGU+rENEiAAswGu/t/EP4YmR41+f5DAtvqbSNT3AqxxRrg==";

    /// ISOLATING oracle for the witness-script-hash binding (`verify.rs:453-463`).
    ///
    /// The attacker's own 1-of-1 script, but signed over the **VICTIM's**
    /// `to_sign` sighash — so the ECDSA check would PASS and only the
    /// script-hash binding stands between it and a VALID verdict. Mutation-
    /// proven: against the pinned 0.0.11 it is rejected with `ToSignInvalid`,
    /// and with that binding block stripped from a local copy of the crate the
    /// exact same bytes verify `Ok(())`. Unlike `P2WSH_FORGED_SIG`, this cell
    /// goes red if the binding is ever weakened.
    const P2WSH_BINDING_FORGERY: &str = "AwBHMEQCIANP62M0w/nJQTG8J4AGpOVzz7atHlSZAfeTtnanBfP0AiBZ0H8bNnOZA1ezz0Yees8lmXpGWbwDQggz2qB9b9cvbgElUSEDJGU+rENEiAAswGu/t/EP4YmR41+f5DAtvqbSNT3AqxxRrg==";

    #[test]
    fn p2wsh_script_hash_binding_is_load_bearing() {
        // Burndown of `bip322-0011-widened-accept-surface-untested-classes`.
        //
        // `p2wsh_foreign_script_forgery_rejected` is over-determined — its
        // constant fails the ECDSA check too, so it would stay green even if
        // the witness-script-hash binding were deleted. This cell closes that
        // gap: P2WSH_BINDING_FORGERY carries the ATTACKER's script signed over
        // the VICTIM's sighash, so the signature check PASSES and only the
        // binding (`vendor/bip322/src/verify.rs:453-463`) can reject it.
        //
        // Mutation-proven when written: stripping that binding block from a
        // local copy of 0.0.11 flips this exact input from `ToSignInvalid` to
        // `Ok(())`. If the binding is ever weakened, this cell goes red.
        for f in [SigFormat::Bip322, SigFormat::Auto] {
            let r = verify_message(P2WSH_ADDRESS, "Hello World", P2WSH_BINDING_FORGERY, f).unwrap();
            assert!(
                !r.valid,
                "FORGERY ACCEPTED: P2WSH witness-script-hash binding is not enforced"
            );
        }
    }

    #[test]
    fn p2wsh_multisig_genuine_proof_verifies() {
        let o = verify_message(
            P2WSH_ADDRESS,
            "Hello World",
            P2WSH_GENUINE_SIG,
            SigFormat::Bip322,
        )
        .unwrap();
        assert!(o.valid, "genuine P2WSH multisig proof was rejected");
        // Non-tautology: the same proof must fail on a different message.
        assert!(
            !verify_message(
                P2WSH_ADDRESS,
                "Goodbye World",
                P2WSH_GENUINE_SIG,
                SigFormat::Bip322
            )
            .unwrap()
            .valid
        );
    }

    #[test]
    fn p2wsh_foreign_script_forgery_rejected() {
        for f in [SigFormat::Bip322, SigFormat::Auto] {
            let r = verify_message(P2WSH_ADDRESS, "Hello World", P2WSH_FORGED_SIG, f).unwrap();
            assert!(
                !r.valid,
                "FORGERY ACCEPTED: another party's P2WSH proof verified against this address"
            );
        }
    }

    #[test]
    fn p2sh_non_keypath_witness_rejected_fail_closed() {
        // Direct gate test. A non-key-path P2SH witness is rejected outright.
        // This denies no genuine owner anything: 0.0.11's P2SH-multisig and
        // P2SH-P2WSH arms both read the redeem/witness script out of the
        // scriptSig, which the BIP-322 *simple* encoding cannot carry, so those
        // proofs can never succeed on this call path anyway.
        use base64::Engine;
        use bitcoin::consensus::encode::serialize;
        use bitcoin::Witness;
        let mut w = Witness::new();
        w.push(Vec::new()); // CHECKMULTISIG dummy
        w.push(vec![0x30; 71]); // signature
        w.push(vec![0x51; 37]); // witness script
        let sig = base64::engine::general_purpose::STANDARD.encode(serialize(&w));
        let addr = parse_addr("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy").unwrap();
        assert_eq!(
            bip322_key_binding_holds(&addr, &sig),
            Some(false),
            "gate must stay fail-closed on a non-key-path P2SH witness"
        );
        // ...while the P2WPKH-shaped forgery is still a hard rejection.
        let p2wpkh = parse_addr(SEGWIT_ADDRESS).unwrap();
        assert_eq!(
            bip322_key_binding_holds(&p2wpkh, FORGED_P2WPKH),
            Some(false)
        );
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
