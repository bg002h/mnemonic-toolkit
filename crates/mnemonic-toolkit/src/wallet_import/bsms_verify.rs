//! BIP-129 Round-1 signature verification.
//!
//! Per BIP-129 §Round 1 Signer (line 81): the SIG on line 5 is a BIP-322
//! legacy-format ECDSA recoverable signature over the four-line body
//! (lines 1-4 joined by `\n`, no trailing newline), under the standard
//! "Bitcoin Signed Message" double-SHA256 digest, base64-encoded.
//!
//! Implementation uses:
//! - `bitcoin::sign_message::signed_msg_hash` for the BIP-129 digest. This
//!   function matches the Coinkite Python reference impl's `bitcoin_msg`
//!   byte-for-byte.
//! - `bitcoin::sign_message::MessageSignature::from_base64` to decode the
//!   65-byte recoverable signature on line 5 (1-byte header + 32-byte r +
//!   32-byte s); header byte conveys recovery-id + compressed-pubkey flag.
//! - `MessageSignature::recover_pubkey` to recover the signing pubkey from
//!   (digest, sig); we then assert recovered_pubkey == signer_pubkey
//!   extracted from line 3.
//!
//! Recovery is sufficient — if `recover_pubkey` returns a pubkey, the
//! signature IS valid against that pubkey by construction; comparing
//! against the declared `signer_pubkey` is the load-bearing security gate.

use super::bsms_round1::{signed_body, signer_pubkey, BsmsRound1Record};
use crate::error::ToolkitError;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::sign_message::{signed_msg_hash, MessageSignature};

/// Verify a BIP-129 Round-1 key-record signature.
///
/// `record_index` is the 0-based index into the user's `--bsms-round1`
/// invocations (for error reporting). Returns `Ok(())` on verified-and-match;
/// `Err(BsmsSignatureMismatch)` if recovery succeeds with a different
/// pubkey or fails entirely.
pub(crate) fn verify_round1_signature(
    record: &BsmsRound1Record,
    record_index: usize,
) -> Result<(), ToolkitError> {
    let body = signed_body(record);
    let digest = signed_msg_hash(&body);

    let sig = MessageSignature::from_base64(&record.signature_b64).map_err(|e| {
        ToolkitError::BsmsSignatureMismatch {
            record_index,
            signer_pubkey: pubkey_hex(record),
            reason: format!("line 5 SIG base64 / structure invalid: {}", e),
        }
    })?;

    let secp = Secp256k1::verification_only();
    let recovered =
        sig.recover_pubkey(&secp, digest)
            .map_err(|e| ToolkitError::BsmsSignatureMismatch {
                record_index,
                signer_pubkey: pubkey_hex(record),
                reason: format!("ECDSA recovery failed: {}", e),
            })?;

    let declared = signer_pubkey(record);
    if recovered.inner != declared {
        return Err(ToolkitError::BsmsSignatureMismatch {
            record_index,
            signer_pubkey: pubkey_hex(record),
            reason: format!(
                "ECDSA recovery succeeded but pubkey mismatch — recovered {} \
                 vs declared {}",
                hex::encode(recovered.inner.serialize()),
                hex::encode(declared.serialize()),
            ),
        });
    }

    Ok(())
}

fn pubkey_hex(record: &BsmsRound1Record) -> String {
    hex::encode(signer_pubkey(record).serialize())
}

#[cfg(test)]
mod tests {
    //! BIP-129 in-spec test vectors. Verbatim per recon doc
    //! `design/agent-reports/v0_27_0-phase-2-bip129-recon.md` §3a.

    use super::*;
    use crate::wallet_import::bsms_round1::parse_round1;

    // TV-1 — NO_ENCRYPTION / raw pubkey, Signer 1 (BIP-129 lines 220-230).
    const TV1: &str = "BSMS 1.0\n00\n[59865f44/48'/0'/0'/2']026d15412460ba0d881c21837bb999233896085a9ed4e5445bd637c10e579768ba\nSigner 1 key\nH6DXgqkCb353BDPkzppMFpOcdJZlpur0WRetQhIBqSn6DFzoQWBtm+ibP5wERDRNi0bxxev9B+FIvyQWq0s6im4=\n";

    // TV-2 — NO_ENCRYPTION / xpub, Signer 1 (BIP-129 lines 259-269).
    const TV2: &str = "BSMS 1.0\n00\n[1cf0bf7e/48'/0'/0'/2']xpub6FL8FhxNNUVnG64YurPd16AfGyvFLhh7S2uSsDqR3Qfcm6o9jtcMYwh6DvmcBF9qozxNQmTCVvWtxLpKTnhVLN3Pgnu2D3pAoXYFgVyd8Yz\nSigner 1 key\nIB7v+qi1b+Xrwm/3bF+Rjl8QbIJ/FMQ40kUsOOQo1SqUWn5QlFWbBD8BKPRetfo1L1N7DmYjVscZNsmMrqRJGWw=\n";

    // TV-3 — STANDARD encryption / xpub, Signer 1 (BIP-129 lines 301-311).
    const TV3: &str = "BSMS 1.0\na54044308ceac9b7\n[b7868815/48'/0'/0'/2']xpub6FA5rfxJc94K1kNtxRby1hoHwi7YDyTWwx1KUR3FwskaF6HzCbZMz3zQwGnCqdiFeMTPV3YneTGS2YQPiuNYsSvtggWWMQpEJD4jXU7ZzEh\nSigner 1 key\nH8DYht5P6ko0bQqDV6MtUxpzBSK+aVHxbvMavA5byvLrOlCEGmO1WFR7k2wu42J6dxXD8vrmDQSnGq5MTMMbZ98=\n";

    #[test]
    fn tv1_no_encryption_raw_pubkey_signer1_verifies() {
        let r = parse_round1(TV1).expect("parse");
        verify_round1_signature(&r, 0).expect("TV-1 must verify");
    }

    #[test]
    fn tv2_no_encryption_xpub_signer1_verifies() {
        let r = parse_round1(TV2).expect("parse");
        verify_round1_signature(&r, 0).expect("TV-2 must verify");
    }

    #[test]
    fn tv3_standard_encryption_xpub_signer1_verifies() {
        let r = parse_round1(TV3).expect("parse");
        verify_round1_signature(&r, 0).expect("TV-3 must verify");
    }

    #[test]
    fn tv1_with_flipped_signature_byte_rejects_with_signature_mismatch() {
        // Flip one base64 char in the signature so it decodes to a different
        // 65-byte signature. Choose a payload-region char (skip the leading
        // header byte position).
        let bad_sig = "H6DXgqkCb353BDPkzppMFpOcdJZlpur0WRetQhIBqSn6DFzoQWBtm+ibP5wERDRNi0bxxev9B+FIvyQWq0s6im5=";
        let bad = TV1.replace(
            "H6DXgqkCb353BDPkzppMFpOcdJZlpur0WRetQhIBqSn6DFzoQWBtm+ibP5wERDRNi0bxxev9B+FIvyQWq0s6im4=",
            bad_sig,
        );
        let r = parse_round1(&bad).expect("parse");
        let err = verify_round1_signature(&r, 0).expect_err("must reject");
        assert!(matches!(err, ToolkitError::BsmsSignatureMismatch { .. }));
    }

    #[test]
    fn tv1_with_flipped_token_byte_rejects_with_signature_mismatch() {
        // TOKEN is part of the signed body; flipping it changes the digest
        // → recovery yields a different pubkey → mismatch.
        // Use NO_ENCRYPTION-compatible 2-hex-char form so the parser still
        // accepts it as a valid token shape; the verify-time mismatch is
        // the failure mode under test.
        let bad = TV1.replace("\n00\n", "\nff\n");
        let r = parse_round1(&bad).expect("parse");
        let err = verify_round1_signature(&r, 0).expect_err("must reject");
        assert!(matches!(err, ToolkitError::BsmsSignatureMismatch { .. }));
    }

    #[test]
    fn record_index_propagates_into_signature_mismatch_error() {
        let bad = TV1.replace("\n00\n", "\nff\n");
        let r = parse_round1(&bad).expect("parse");
        let err = verify_round1_signature(&r, 42).expect_err("must reject");
        if let ToolkitError::BsmsSignatureMismatch { record_index, .. } = err {
            assert_eq!(record_index, 42);
        } else {
            panic!("expected BsmsSignatureMismatch");
        }
    }
}
