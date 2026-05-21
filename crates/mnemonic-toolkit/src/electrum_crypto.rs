//! Electrum field-level encryption decrypt + encrypt primitives (v0.31.0 / Cycle 6a).
//!
//! Implements Electrum's `pw_encode_bytes` / `pw_decode_bytes` (Format A
//! field-level encryption) per `electrum/crypto.py`. The scheme:
//!
//! - Key derivation: `key = sha256d(password)` (double SHA-256; NO PBKDF2;
//!   NO salt; NO iteration count). 32 bytes → AES-256.
//! - Encryption: AES-256-CBC with PKCS7 padding. 16-byte random IV.
//! - Wire format: `base64(iv (16 bytes) || aes_cbc(plaintext + PKCS7))`.
//!
//! ## Scope (v0.31.0)
//!
//! - **Format A only** — field-level encrypted strings inside an otherwise
//!   plaintext JSON wallet (where `use_encryption: true`). Each sensitive
//!   field's value is independently encrypted + base64-encoded.
//! - **Format B deferred** — whole-file storage encryption (version-byte
//!   prefix + 4-byte SHA-256 MAC) is NOT supported. Tracked as FOLLOWUP
//!   `wallet-import-electrum-encrypted-storage-format-b`.
//!
//! ## Error pattern
//!
//! Library-local `ElectrumDecryptError` with hand-rolled `impl Display`
//! (mirrors `seed_xor.rs:31-67`). The CLI boundary in `cmd/import_wallet.rs`
//! (Cycle 6b Phase 3) converts via a boundary mapper to
//! `ToolkitError::BadInput` at orchestrator pre-decrypt time.
//!
//! ## Reference
//!
//! `electrum/crypto.py::_pw_decode_raw` + `EncodeAES_bytes` /
//! `DecodeAES_bytes`. Verified against the SeedQR-style cross-impl
//! discipline (Python-side known vectors locked in the test module).

use aes::Aes256;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

type Aes256CbcDec = cbc::Decryptor<Aes256>;
type Aes256CbcEnc = cbc::Encryptor<Aes256>;

/// Library-local error type. Mapped to `ToolkitError::BadInput` at the CLI
/// boundary in Cycle 6b Phase 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElectrumDecryptError {
    /// Input string is not valid base64 (bad chars or wrong padding).
    Base64DecodeFailure(String),
    /// Decoded ciphertext is too short to contain the 16-byte IV.
    CiphertextTooShort { got: usize },
    /// Ciphertext body (post-IV) is not a positive multiple of 16 bytes.
    CiphertextNotBlockAligned { got: usize },
    /// AES-CBC decrypt + PKCS7 unpadding refused. Most common cause: wrong
    /// password (the derived key fails PKCS7 strip after decryption).
    AesDecryptFailure,
    /// Decrypted bytes are not valid UTF-8. Indicates wrong password OR a
    /// non-text payload (which Electrum field encryption never produces).
    Utf8DecodeFailure,
}

impl std::fmt::Display for ElectrumDecryptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElectrumDecryptError::AesDecryptFailure => {
                write!(
                    f,
                    "AES-CBC decrypt + PKCS7 unpad refused (likely wrong password)"
                )
            }
            ElectrumDecryptError::Base64DecodeFailure(msg) => {
                write!(f, "ciphertext is not valid base64: {msg}")
            }
            ElectrumDecryptError::CiphertextNotBlockAligned { got } => write!(
                f,
                "ciphertext body length {got} not a positive multiple of 16 bytes",
            ),
            ElectrumDecryptError::CiphertextTooShort { got } => {
                write!(f, "ciphertext length {got} < 16 bytes (no room for IV)",)
            }
            ElectrumDecryptError::Utf8DecodeFailure => {
                write!(
                    f,
                    "decrypted bytes are not valid UTF-8 (likely wrong password)"
                )
            }
        }
    }
}

impl std::error::Error for ElectrumDecryptError {}

/// Derive the AES-256 key from a password via Electrum's `sha256d` scheme.
///
/// Equivalent to Electrum's `_hash_password(password, version=1)` which
/// computes `SHA-256(SHA-256(password))`.
pub fn derive_key(password: &[u8]) -> Zeroizing<[u8; 32]> {
    let mut hasher = Sha256::new();
    hasher.update(password);
    let first = hasher.finalize_reset();
    hasher.update(first);
    let second = hasher.finalize();
    let mut out = Zeroizing::new([0u8; 32]);
    out.copy_from_slice(&second);
    out
}

/// Decrypt a base64-encoded Electrum-encrypted field.
///
/// Wire format: `base64(iv (16 bytes) || aes-cbc(plaintext + PKCS7))`.
///
/// Returns the decrypted UTF-8 string wrapped in `Zeroizing` for in-memory
/// hygiene (the plaintext is secret material — typically a BIP-39 mnemonic
/// or BIP-32 extended private key).
pub fn decrypt_field(
    b64_ciphertext: &str,
    password: &[u8],
) -> Result<Zeroizing<String>, ElectrumDecryptError> {
    let key = derive_key(password);
    let wire = BASE64
        .decode(b64_ciphertext.as_bytes())
        .map_err(|e| ElectrumDecryptError::Base64DecodeFailure(e.to_string()))?;

    if wire.len() < 16 {
        return Err(ElectrumDecryptError::CiphertextTooShort { got: wire.len() });
    }

    let (iv, ciphertext) = wire.split_at(16);
    if ciphertext.is_empty() || ciphertext.len() % 16 != 0 {
        return Err(ElectrumDecryptError::CiphertextNotBlockAligned {
            got: ciphertext.len(),
        });
    }

    let mut buf = Zeroizing::new(ciphertext.to_vec());
    let decryptor = Aes256CbcDec::new_from_slices(key.as_slice(), iv)
        .expect("AES-256-CBC accepts 32-byte key + 16-byte IV (verified by prior length checks)");
    let plaintext_bytes_len = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|_| ElectrumDecryptError::AesDecryptFailure)?
        .len();
    buf.truncate(plaintext_bytes_len);

    let plaintext_string =
        String::from_utf8(buf.to_vec()).map_err(|_| ElectrumDecryptError::Utf8DecodeFailure)?;
    Ok(Zeroizing::new(plaintext_string))
}

/// Encrypt a UTF-8 string into Electrum field-encryption wire format.
///
/// Symmetric inverse of [`decrypt_field`]. The caller supplies the IV
/// explicitly (16 bytes); production code should pass cryptographically
/// random bytes (e.g., via `rand_core::OsRng::fill_bytes`). Test fixtures
/// use deterministic IVs for reproducibility.
pub fn encrypt_field(plaintext: &str, password: &[u8], iv: &[u8; 16]) -> String {
    let key = derive_key(password);
    let plaintext_bytes = plaintext.as_bytes();
    // PKCS7 padding adds at minimum 1 byte (and at most 16). Allocate the
    // ciphertext buffer at next-multiple-of-16 capacity.
    let padded_len = (plaintext_bytes.len() / 16 + 1) * 16;
    let mut buf = vec![0u8; padded_len];
    buf[..plaintext_bytes.len()].copy_from_slice(plaintext_bytes);

    let encryptor = Aes256CbcEnc::new_from_slices(key.as_slice(), iv)
        .expect("AES-256-CBC accepts 32-byte key + 16-byte IV");
    let ciphertext_len = encryptor
        .encrypt_padded_mut::<Pkcs7>(&mut buf, plaintext_bytes.len())
        .expect("buffer capacity equals padded_len; padding cannot overflow")
        .len();
    buf.truncate(ciphertext_len);

    let mut wire = Vec::with_capacity(16 + ciphertext_len);
    wire.extend_from_slice(iv);
    wire.extend_from_slice(&buf);

    BASE64.encode(&wire)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Cross-impl smoke vector generated via Python `cryptography` backend
    // (Cycle 6a P0 recon). Replication recipe:
    //
    //     python3 -c "
    //     import hashlib, base64
    //     from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
    //     from cryptography.hazmat.backends import default_backend
    //     pw = b'test-password'
    //     key = hashlib.sha256(hashlib.sha256(pw).digest()).digest()
    //     iv = bytes.fromhex('00112233445566778899aabbccddeeff')
    //     plaintext = b'hello world'
    //     padlen = 16 - (len(plaintext) % 16)
    //     padded = plaintext + bytes([padlen]) * padlen
    //     cipher = Cipher(algorithms.AES(key), modes.CBC(iv), backend=default_backend())
    //     enc = cipher.encryptor()
    //     ct = enc.update(padded) + enc.finalize()
    //     wire = iv + ct
    //     print(base64.b64encode(wire).decode())
    //     "
    //     # Expected: ABEiM0RVZneImaq7zN3u/zY0181f7qAY/NWiVQFLdHE=

    const TEST_PASSWORD: &[u8] = b"test-password";
    const TEST_PLAINTEXT: &str = "hello world";
    const TEST_IV: [u8; 16] = [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
        0xff,
    ];
    const TEST_CIPHERTEXT_B64: &str = "ABEiM0RVZneImaq7zN3u/zY0181f7qAY/NWiVQFLdHE=";
    const TEST_KEY_HEX: &str = "1bbfc37c1ad8e131b25747e9c1e0ccdffeeb1e946f1e8949d40717834cde2dc4";

    // ──────────────────────────────────────────────────────────────────────
    // derive_key
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn derive_key_known_vector_test_password() {
        let key = derive_key(TEST_PASSWORD);
        assert_eq!(hex::encode(key.as_slice()), TEST_KEY_HEX);
    }

    #[test]
    fn derive_key_empty_password_is_deterministic_nonzero() {
        // sha256d(b"") = sha256(sha256(b"")) — deterministic, nonzero.
        let key_a = derive_key(b"");
        let key_b = derive_key(b"");
        assert_eq!(key_a.as_slice(), key_b.as_slice());
        assert_ne!(key_a.as_slice(), &[0u8; 32]);
    }

    #[test]
    fn derive_key_long_password_no_truncation() {
        // Verify large inputs hash through cleanly (no internal truncation).
        let long_pw = vec![0xa5u8; 1024];
        let key_a = derive_key(&long_pw);
        let mut variant = long_pw.clone();
        variant[1023] = 0xa6; // change last byte
        let key_b = derive_key(&variant);
        assert_ne!(key_a.as_slice(), key_b.as_slice());
    }

    // ──────────────────────────────────────────────────────────────────────
    // Cross-impl smoke against Python reference
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn decrypt_field_matches_python_known_vector() {
        let plaintext = decrypt_field(TEST_CIPHERTEXT_B64, TEST_PASSWORD).unwrap();
        assert_eq!(plaintext.as_str(), TEST_PLAINTEXT);
    }

    #[test]
    fn encrypt_field_matches_python_known_vector() {
        let b64 = encrypt_field(TEST_PLAINTEXT, TEST_PASSWORD, &TEST_IV);
        assert_eq!(b64, TEST_CIPHERTEXT_B64);
    }

    // ──────────────────────────────────────────────────────────────────────
    // Round-trips
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn round_trip_12_word_phrase() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let iv = [0u8; 16];
        let b64 = encrypt_field(phrase, b"my-password", &iv);
        let recovered = decrypt_field(&b64, b"my-password").unwrap();
        assert_eq!(recovered.as_str(), phrase);
    }

    #[test]
    fn round_trip_empty_plaintext() {
        // Empty plaintext exercises the PKCS7 boundary (entire block is padding).
        let iv = [0xafu8; 16];
        let b64 = encrypt_field("", b"pw", &iv);
        let recovered = decrypt_field(&b64, b"pw").unwrap();
        assert_eq!(recovered.as_str(), "");
    }

    #[test]
    fn round_trip_block_boundary_15_bytes() {
        let iv = [0x77u8; 16];
        let pt = "aaaaaaaaaaaaaaa"; // 15 bytes → 1 byte PKCS7
        let b64 = encrypt_field(pt, b"pw", &iv);
        let recovered = decrypt_field(&b64, b"pw").unwrap();
        assert_eq!(recovered.as_str(), pt);
    }

    #[test]
    fn round_trip_block_boundary_16_bytes() {
        let iv = [0x77u8; 16];
        let pt = "aaaaaaaaaaaaaaaa"; // 16 bytes → 16 bytes PKCS7 (full block)
        let b64 = encrypt_field(pt, b"pw", &iv);
        let recovered = decrypt_field(&b64, b"pw").unwrap();
        assert_eq!(recovered.as_str(), pt);
    }

    #[test]
    fn round_trip_block_boundary_17_bytes() {
        let iv = [0x77u8; 16];
        let pt = "aaaaaaaaaaaaaaaaa"; // 17 bytes → 15 bytes PKCS7
        let b64 = encrypt_field(pt, b"pw", &iv);
        let recovered = decrypt_field(&b64, b"pw").unwrap();
        assert_eq!(recovered.as_str(), pt);
    }

    #[test]
    fn round_trip_utf8_multibyte() {
        let pt = "résumé🔑";
        let iv = [0x55u8; 16];
        let b64 = encrypt_field(pt, b"pw", &iv);
        let recovered = decrypt_field(&b64, b"pw").unwrap();
        assert_eq!(recovered.as_str(), pt);
    }

    // ──────────────────────────────────────────────────────────────────────
    // Refusal cells
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn decrypt_field_wrong_password() {
        let result = decrypt_field(TEST_CIPHERTEXT_B64, b"wrong-password");
        // PKCS7 unpadding will almost certainly fail with a wrong key; on the
        // rare (1-in-65536 ish) collision the UTF-8 check catches it.
        assert!(matches!(
            result,
            Err(ElectrumDecryptError::AesDecryptFailure | ElectrumDecryptError::Utf8DecodeFailure)
        ));
    }

    #[test]
    fn decrypt_field_invalid_base64() {
        let result = decrypt_field("not_valid_base64_!@#$%", TEST_PASSWORD);
        assert!(matches!(
            result,
            Err(ElectrumDecryptError::Base64DecodeFailure(_))
        ));
    }

    #[test]
    fn decrypt_field_ciphertext_too_short_for_iv() {
        // 16 chars of base64 → 10 bytes of binary (one trailing `=` per 8 chars
        // of unpadded data), < 16-byte IV minimum.
        let result = decrypt_field("AAECAwQFBgcICQ==", TEST_PASSWORD);
        assert!(matches!(
            result,
            Err(ElectrumDecryptError::CiphertextTooShort { got: 10 })
        ));
    }

    #[test]
    fn decrypt_field_empty_ciphertext_post_iv() {
        // Exactly 16 bytes (just the IV, no ciphertext body).
        let iv_only = BASE64.encode([0u8; 16]);
        let result = decrypt_field(&iv_only, TEST_PASSWORD);
        assert!(matches!(
            result,
            Err(ElectrumDecryptError::CiphertextNotBlockAligned { got: 0 })
        ));
    }

    #[test]
    fn decrypt_field_ciphertext_not_block_aligned() {
        // 24 bytes: 16-byte IV + 8-byte body (not multiple of 16).
        let mut wire = vec![0u8; 24];
        wire[16..].copy_from_slice(b"00000000");
        let b64 = BASE64.encode(&wire);
        let result = decrypt_field(&b64, TEST_PASSWORD);
        assert!(matches!(
            result,
            Err(ElectrumDecryptError::CiphertextNotBlockAligned { got: 8 })
        ));
    }

    // ──────────────────────────────────────────────────────────────────────
    // Encrypt determinism + IV-as-keyspace-prefix invariant
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn encrypt_field_iv_is_prefix_of_wire() {
        let iv = [0x42u8; 16];
        let b64 = encrypt_field("test", b"pw", &iv);
        let wire = BASE64.decode(&b64).unwrap();
        assert_eq!(&wire[..16], &iv);
    }

    #[test]
    fn encrypt_field_different_iv_different_ciphertext() {
        let iv_a = [0x00u8; 16];
        let iv_b = [0xffu8; 16];
        let ct_a = encrypt_field("test", b"pw", &iv_a);
        let ct_b = encrypt_field("test", b"pw", &iv_b);
        assert_ne!(ct_a, ct_b);
    }
}
