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
//! ## Scope
//!
//! - **Format A** (v0.31.0) — field-level encrypted strings inside an
//!   otherwise plaintext JSON wallet (where `use_encryption: true`). Each
//!   sensitive field's value is independently encrypted + base64-encoded.
//!   See [`decrypt_field`] / [`encrypt_field`] / [`derive_key`].
//! - **Whole-file storage encryption — BIE1 / user-password (Cycle 19)** —
//!   the entire wallet file is a single base64 blob whose decoded magic is
//!   `BIE1`. The scheme is **ECIES** (NOT the "version-byte + 4-byte MAC"
//!   that this module's earlier doc-comment claimed — that was a crypto
//!   misidentification corrected at Cycle 19 P0 recon; see the FOLLOWUP).
//!   See [`ecies_decrypt_storage`] / [`ecies_decrypt_message`] /
//!   [`derive_storage_eckey`] + [`EciesDecryptError`]. Verified against
//!   `spesmilo/electrum` @ `2e640c83` (`crypto.py` / `storage.py`); KATs use
//!   Electrum's own committed `test_decrypt_message` vectors.
//! - **`BIE2` / hardware-device (xpub) storage encryption** is detected
//!   (`EciesDecryptError::Bie2Unsupported`) but never decryptable from a
//!   password (the key is the device's master key).
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

use aes::{Aes128, Aes256};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

type Aes256CbcDec = cbc::Decryptor<Aes256>;
type Aes256CbcEnc = cbc::Encryptor<Aes256>;
type Aes128CbcDec = cbc::Decryptor<Aes128>;

// ── ECIES BIE1 (whole-file storage decrypt) imports ───────────────────────
use bitcoin::secp256k1::{PublicKey, Scalar, Secp256k1};
use crypto_bigint::{Encoding, NonZero, U512};
use flate2::read::ZlibDecoder;
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2;
use sha2::Sha512;
use std::io::Read;

type HmacSha256 = Hmac<Sha256>;

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

// ════════════════════════════════════════════════════════════════════════
// Whole-file storage encryption — ECIES BIE1 (user-password). Cycle 19.
//
// Verified against spesmilo/electrum @ 2e640c83 (`crypto.py` /
// `storage.py`). `WalletStorage.decrypt(password)` =
//   zlib.decompress(ecies_decrypt_message(get_eckey_from_password(pw), raw))
// where:
//   get_eckey_from_password(pw) = PBKDF2-HMAC-SHA512(pw, salt=b"", 1024, 64)
//                                 reduced mod secp256k1 order n.
//   BIE1 blob = base64( b"BIE1"(4) || ephemeral_pubkey_compressed(33)
//                       || aes128cbc(plaintext+PKCS7) || hmac_sha256(32) ).
//   ecdh = compressed(ephemeral_pubkey × privkey); key = sha512(ecdh);
//   iv,key_e,key_m = key[0:16],[16:32],[32:64]; Encrypt-then-MAC over
//   blob[:-32]; AES-128-CBC.
// ════════════════════════════════════════════════════════════════════════

/// secp256k1 group order `n`, zero-LEFT-padded to 64 bytes for `U512`
/// reduction (128 hex chars: 64 zeros || 64 chars of n).
const SECP256K1_ORDER_U512_BE_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000000\
     FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141";

/// Library-local error for the ECIES BIE1 storage-decrypt path. Distinct from
/// [`ElectrumDecryptError`] (Format A): the failure modes and the
/// AES-128/ECDH vs AES-256/sha256d semantics are disjoint. Variants are
/// alphabetical (CLAUDE.md convention). The Phase-B CLI boundary unifies
/// `HmacMismatch | AesDecryptFailure` into one non-leaky message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EciesDecryptError {
    /// AES-128-CBC decrypt + PKCS7 unpad refused (post-MAC; rare — implies a
    /// malformed ciphertext body that nonetheless passed the HMAC).
    AesDecryptFailure,
    /// Input string is not valid base64.
    Base64DecodeFailure(String),
    /// Magic bytes are `BIE2` — hardware-device (xpub) storage encryption,
    /// which cannot be decrypted from a password.
    Bie2Unsupported,
    /// HMAC-SHA256 over `blob[:-32]` did not match the trailing 32-byte tag.
    /// This is the wrong-password (or tampered-ciphertext) detector.
    HmacMismatch,
    /// The ephemeral public key (bytes `[4..37]`) is not a valid secp256k1
    /// point.
    InvalidEphemeralPubkey,
    /// Magic bytes are neither `BIE1` nor `BIE2`.
    InvalidMagic([u8; 4]),
    /// The derived private scalar reduced to zero mod the group order
    /// (unreachable in practice for any real password).
    InvalidScalar,
    /// Decoded blob is shorter than the 85-byte minimum
    /// (4 magic + 33 ephemeral + ≥16 ciphertext + 32 mac).
    TooShort { got: usize },
    /// zlib decompression of the recovered plaintext failed.
    ZlibDecompressFailure(String),
}

impl std::fmt::Display for EciesDecryptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EciesDecryptError::AesDecryptFailure => {
                write!(f, "AES-128-CBC decrypt + PKCS7 unpad refused")
            }
            EciesDecryptError::Base64DecodeFailure(msg) => {
                write!(f, "storage blob is not valid base64: {msg}")
            }
            EciesDecryptError::Bie2Unsupported => write!(
                f,
                "wallet is encrypted with a hardware-device key (BIE2 / XPUB_PASSWORD); \
                 it cannot be decrypted from a password"
            ),
            EciesDecryptError::HmacMismatch => {
                write!(f, "HMAC mismatch (wrong password or corrupted ciphertext)")
            }
            EciesDecryptError::InvalidEphemeralPubkey => {
                write!(f, "ephemeral public key is not a valid secp256k1 point")
            }
            EciesDecryptError::InvalidMagic(m) => {
                write!(f, "unrecognized storage magic bytes: {m:02x?}")
            }
            EciesDecryptError::InvalidScalar => {
                write!(f, "derived private scalar is zero mod the group order")
            }
            EciesDecryptError::TooShort { got } => {
                write!(f, "storage blob length {got} < 85-byte minimum")
            }
            EciesDecryptError::ZlibDecompressFailure(msg) => {
                write!(f, "zlib decompression failed: {msg}")
            }
        }
    }
}

impl std::error::Error for EciesDecryptError {}

/// Derive the secp256k1 private scalar from a wallet password via Electrum's
/// `WalletStorage.get_eckey_from_password`:
/// `int(PBKDF2-HMAC-SHA512(password, salt=b"", iterations=1024, dklen=64))
/// mod n` (the secp256k1 group order). The salt is empty (an Electrum design
/// choice — no per-wallet salting). Returns the 32-byte big-endian scalar.
pub fn derive_storage_eckey(password: &[u8]) -> Result<Zeroizing<[u8; 32]>, EciesDecryptError> {
    let mut secret = Zeroizing::new([0u8; 64]);
    pbkdf2::<Hmac<Sha512>>(password, b"", 1024, secret.as_mut_slice())
        .expect("pbkdf2 fill must succeed (dkLen + iters in supported range)");
    // scalar = int.from_bytes(secret, 'big') mod n.
    let n = Option::from(NonZero::new(U512::from_be_hex(SECP256K1_ORDER_U512_BE_HEX)))
        .expect("secp256k1 group order is nonzero");
    let reduced = U512::from_be_slice(secret.as_slice()).rem(&n);
    let reduced_bytes = Zeroizing::new(reduced.to_be_bytes()); // [u8; 64]
    let mut scalar = Zeroizing::new([0u8; 32]);
    scalar.copy_from_slice(&reduced_bytes[32..64]);
    if scalar.iter().all(|&b| b == 0) {
        return Err(EciesDecryptError::InvalidScalar);
    }
    Ok(scalar)
}

/// Decrypt an Electrum BIE1 ECIES message given the recipient private scalar.
///
/// Returns the raw recovered plaintext (NOT zlib-decompressed — see
/// [`ecies_decrypt_storage`] for the whole-file storage pipeline). Validated
/// directly by Electrum's own `tests/test_bitcoin.py::test_decrypt_message`
/// vectors.
pub fn ecies_decrypt_message(
    blob_b64: &str,
    privkey: &[u8; 32],
) -> Result<Zeroizing<Vec<u8>>, EciesDecryptError> {
    let raw = BASE64
        .decode(blob_b64.as_bytes())
        .map_err(|e| EciesDecryptError::Base64DecodeFailure(e.to_string()))?;
    if raw.len() < 85 {
        return Err(EciesDecryptError::TooShort { got: raw.len() });
    }
    let magic: [u8; 4] = raw[0..4].try_into().expect("len >= 85 guarantees 4 bytes");
    match &magic {
        b"BIE1" => {}
        b"BIE2" => return Err(EciesDecryptError::Bie2Unsupported),
        _ => return Err(EciesDecryptError::InvalidMagic(magic)),
    }

    let ephemeral_pubkey = PublicKey::from_slice(&raw[4..37])
        .map_err(|_| EciesDecryptError::InvalidEphemeralPubkey)?;
    let ciphertext = &raw[37..raw.len() - 32];
    let mac = &raw[raw.len() - 32..];

    // ECDH: ecdh_point = ephemeral_pubkey × privkey_scalar; compressed (33B).
    let secp = Secp256k1::new();
    let scalar = Scalar::from_be_bytes(*privkey).map_err(|_| EciesDecryptError::InvalidScalar)?;
    // mul_tweak cannot fail here: the scalar is in [1, n-1] (derive_storage_eckey
    // rejects zero; reduction guarantees < n), and a valid order-n point times
    // a nonzero scalar < n is never the identity (prime-order group).
    let shared = ephemeral_pubkey
        .mul_tweak(&secp, &scalar)
        .expect("valid point × nonzero scalar in [1,n-1] is never the identity");
    let shared_compressed = Zeroizing::new(shared.serialize()); // [u8; 33]

    // key = sha512(shared_compressed); iv,key_e,key_m = key[0:16],[16:32],[32:64].
    let mut key = Zeroizing::new([0u8; 64]);
    key.copy_from_slice(&Sha512::digest(shared_compressed.as_slice()));
    let iv = &key[0..16];
    let key_e = &key[16..32];
    let key_m = &key[32..64];

    // Encrypt-then-MAC: verify HMAC over blob[:-32] (magic || ephemeral ||
    // ciphertext) BEFORE decrypting, to avoid a PKCS7 padding oracle.
    let mut h = <HmacSha256 as Mac>::new_from_slice(key_m).expect("HMAC accepts any key length");
    h.update(&raw[..raw.len() - 32]);
    h.verify_slice(mac)
        .map_err(|_| EciesDecryptError::HmacMismatch)?;

    if ciphertext.is_empty() || ciphertext.len() % 16 != 0 {
        return Err(EciesDecryptError::AesDecryptFailure);
    }
    let mut buf = Zeroizing::new(ciphertext.to_vec());
    let dec = Aes128CbcDec::new_from_slices(key_e, iv)
        .expect("AES-128-CBC accepts 16-byte key + 16-byte IV");
    let pt_len = dec
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|_| EciesDecryptError::AesDecryptFailure)?
        .len();
    buf.truncate(pt_len);
    Ok(buf)
}

/// Decrypt an Electrum BIE1 user-password-encrypted whole wallet file.
///
/// Pipeline: `derive_storage_eckey(password)` → [`ecies_decrypt_message`] →
/// `zlib.decompress`. Returns the recovered wallet JSON bytes (secret —
/// the whole-file plaintext can carry seed/xprv material). `BIE2`
/// (hardware-device) blobs return [`EciesDecryptError::Bie2Unsupported`].
pub fn ecies_decrypt_storage(
    blob_b64: &str,
    password: &[u8],
) -> Result<Zeroizing<Vec<u8>>, EciesDecryptError> {
    let privkey = derive_storage_eckey(password)?;
    let compressed = ecies_decrypt_message(blob_b64, &privkey)?;
    let mut out = Zeroizing::new(Vec::new());
    ZlibDecoder::new(compressed.as_slice())
        .read_to_end(&mut out)
        .map_err(|e| EciesDecryptError::ZlibDecompressFailure(e.to_string()))?;
    Ok(out)
}

/// Magic-byte discriminator for an Electrum storage-encrypted wallet file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElectrumStorageMagic {
    /// `BIE1` — user-password ECIES (decryptable via [`ecies_decrypt_storage`]).
    Bie1,
    /// `BIE2` — hardware-device (xpub) ECIES (NOT password-decryptable).
    Bie2,
}

/// Detect whether a wallet-file blob is an Electrum storage-encrypted blob.
///
/// A storage-encrypted Electrum wallet file is a single base64 line whose
/// decoded 4-byte prefix is the magic `BIE1` (user-password) or `BIE2`
/// (hardware-device). Returns `None` for plaintext / JSON wallets: a JSON
/// `{...}` contains `{`, which is not in the base64 alphabet, so the decode
/// fails — no false-positive against the JSON / text-prefix sniff paths.
///
/// Trims ASCII whitespace (files commonly carry a trailing newline) and uses
/// the same `base64::STANDARD` engine as [`ecies_decrypt_message`], so a
/// positive detection guarantees the same trimmed input feeds the decrypt.
pub fn detect_storage_magic(blob: &[u8]) -> Option<ElectrumStorageMagic> {
    let trimmed = std::str::from_utf8(blob).ok()?.trim();
    let raw = BASE64.decode(trimmed).ok()?;
    if raw.len() < 85 {
        return None;
    }
    match &raw[0..4] {
        b"BIE1" => Some(ElectrumStorageMagic::Bie1),
        b"BIE2" => Some(ElectrumStorageMagic::Bie2),
        _ => None,
    }
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

    // ══════════════════════════════════════════════════════════════════════
    // ECIES BIE1 whole-file storage decrypt (Cycle 19)
    //
    // Authoritative oracle: Electrum's OWN committed vectors
    // (tests/test_bitcoin.py::test_decrypt_message, spesmilo/electrum @
    // 2e640c83): password "pw123" + BIE1 blob → plaintext. These validate
    // derive_storage_eckey (password→privkey, PBKDF2-SHA512 mod-n) AND the
    // full ECIES envelope (ECDH + sha512 KDF + AES-128-CBC + HMAC) in one
    // shot — a wrong mod-n reduction yields a wrong ECDH key → HMAC mismatch.
    // ══════════════════════════════════════════════════════════════════════

    const PW123: &[u8] = b"pw123";
    // get_eckey_from_password("pw123") scalar: int(PBKDF2-HMAC-SHA512(pw, b"",
    // 1024, 64)) mod n. Computed via Python stdlib (hashlib) at spike time.
    const PW123_SCALAR_HEX: &str =
        "2db55bc2121375cef4274b59c36ac703922622527a5af5e6c8df149e3b85b0df";
    const BIE1_KAT1: &str = "QklFMQMDFtgT3zWSQsa+Uie8H/WvfUjlu9UN9OJtTt3KlgKeSTi6SQfuhcg1uIz9hp3WIUOFGTLr4RNQBdjPNqzXwhkcPi2Xsbiw6UCNJncVPJ6QBg==";
    const BIE1_KAT2: &str = "QklFMQKXOXbylOQTSMGfo4MFRwivAxeEEkewWQrpdYTzjPhqjHcGBJwdIhB7DyRfRQihuXx1y0ZLLv7XxLzrILzkl/H4YUtZB4uWjuOAcmxQH4i/Og==";
    const BIE1_KAT3: &str = "QklFMQLOOsabsXtGQH8edAa6VOUa5wX8/DXmxX9NyHoAx1a5bWgllayGRVPeI2bf0ZdWK0tfal0ap0ZIVKbd2eOJybqQkILqT6E1/Syzq0Zicyb/AA1eZNkcX5y4gzloxinw00ubCA8M7gcUjJpOqbnksATcJ5y2YYXcHMGGfGurWu6uJ/UyrNobRidWppRMW5yR9/6utyNvT6OHIolCMEf7qLcmtneoXEiz51hkRdZS7weNf9mGqSbz9a2NL3sdh1A0feHIjAZgcCKcAvksNUSauf0/FnIjzTyPRpjRDMeDC8Ci3sGiuO3cvpWJwhZfbjcS26KmBv2CHWXfRRNFYOInHZNIXWNAoBB47Il5bGSMd+uXiGr+SQ9tNvcu+BiJNmFbxYqg+oQ8dGAl1DtvY2wJVY8k7vO9BIWSpyIxfGw7EDifhc5vnOmGe016p6a01C3eVGxgl23UYMrP7+fpjOcPmTSF4rk5U5ljEN3MSYqlf1QEv0OqlI9q1TwTK02VBCjMTYxDHsnt04OjNBkNO8v5uJ4NR+UUDBEp433z53I59uawZ+dbk4v4ZExcl8EGmKm3Gzbal/iJ/F7KQuX2b/ySEhLOFVYFWxK73X1nBvCSK2mC2/8fCw8oI5pmvzJwQhcCKTdEIrz3MMvAHqtPScDUOjzhXxInQOCb3+UBj1PPIdqkYLvZss1TEaBwYZjLkVnK2MBj7BaqT6Rp6+5A/fippUKHsnB6eYMEPR2YgDmCHL+4twxHJG6UWdP3ybaKiiAPy2OHNP6PTZ0HrqHOSJzBSDD+Z8YpaRg29QX3UEWlqnSKaan0VYAsV1VeaN0XFX46/TWO0L5tjhYVXJJYGqo6tIQJymxATLFRF6AZaD1Mwd27IAL04WkmoQoXfO6OFfwdp/shudY/1gBkDBvGPICBPtnqkvhGF+ZF3IRkuPwiFWeXmwBxKHsRx/3+aJu32Ml9+za41zVk2viaxcGqwTc5KMexQFLAUwqhv+aIik7U+5qk/gEVSuRoVkihoweFzKolNF+BknH2oB4rZdPixag5Zje3DvgjsSFlOl69W/67t/Gs8htfSAaHlsB8vWRQr9+v/lxTbrAw+O0E+sYGoObQ4qQMyQshNZEHbpPg63eWiHtJJnrVBvOeIbIHzoLDnMDsWVWZSMzAQ1vhX1H5QLgSEbRlKSliVY03kDkh/Nk/KOn+B2q37Ialq4JcRoIYFGJ8AoYEAD0tRuTqFddIclE75HzwaNG7NyKW1plsa72ciOPwsPJsdd5F0qdSQ3OSKtooTn7uf6dXOc4lDkfrVYRlZ0PX";

    fn pw123_privkey() -> Zeroizing<[u8; 32]> {
        derive_storage_eckey(PW123).unwrap()
    }

    fn mutate_blob(blob: &str, f: impl Fn(&mut Vec<u8>)) -> String {
        let mut raw = BASE64.decode(blob).unwrap();
        f(&mut raw);
        BASE64.encode(&raw)
    }

    // ── derive_storage_eckey ──────────────────────────────────────────────

    #[test]
    fn derive_storage_eckey_pw123_known_scalar() {
        assert_eq!(hex::encode(pw123_privkey().as_slice()), PW123_SCALAR_HEX);
    }

    #[test]
    fn derive_storage_eckey_empty_password_deterministic_nonzero() {
        let a = derive_storage_eckey(b"").unwrap();
        let b = derive_storage_eckey(b"").unwrap();
        assert_eq!(a.as_slice(), b.as_slice());
        assert_ne!(a.as_slice(), &[0u8; 32]);
    }

    // ── ecies_decrypt_message: the 3 authoritative Electrum KATs ───────────

    #[test]
    fn ecies_decrypt_message_electrum_kat_short_vectors() {
        let pk = pw123_privkey();
        assert_eq!(
            ecies_decrypt_message(BIE1_KAT1, &pk).unwrap().as_slice(),
            b"me<(s_s)>age"
        );
        assert_eq!(
            ecies_decrypt_message(BIE1_KAT2, &pk).unwrap().as_slice(),
            b"me<(s_s)>age"
        );
    }

    #[test]
    fn ecies_decrypt_message_electrum_kat_long_multiblock_vector() {
        let pk = pw123_privkey();
        let expected = "hey_there".repeat(100);
        assert_eq!(
            ecies_decrypt_message(BIE1_KAT3, &pk).unwrap().as_slice(),
            expected.as_bytes()
        );
    }

    // ── ecies_decrypt_message: negatives ──────────────────────────────────

    #[test]
    fn ecies_decrypt_message_bad_base64() {
        let pk = pw123_privkey();
        assert!(matches!(
            ecies_decrypt_message("!!!not base64!!!", &pk),
            Err(EciesDecryptError::Base64DecodeFailure(_))
        ));
    }

    #[test]
    fn ecies_decrypt_message_too_short() {
        let pk = pw123_privkey();
        let short = BASE64.encode([0u8; 50]);
        assert!(matches!(
            ecies_decrypt_message(&short, &pk),
            Err(EciesDecryptError::TooShort { got: 50 })
        ));
    }

    #[test]
    fn ecies_decrypt_message_invalid_magic() {
        let pk = pw123_privkey();
        let m = mutate_blob(BIE1_KAT1, |r| r[0] = b'X');
        assert!(matches!(
            ecies_decrypt_message(&m, &pk),
            Err(EciesDecryptError::InvalidMagic(_))
        ));
    }

    #[test]
    fn ecies_decrypt_message_bie2_unsupported() {
        let pk = pw123_privkey();
        let m = mutate_blob(BIE1_KAT1, |r| r[3] = b'2'); // BIE1 -> BIE2
        assert!(matches!(
            ecies_decrypt_message(&m, &pk),
            Err(EciesDecryptError::Bie2Unsupported)
        ));
    }

    #[test]
    fn ecies_decrypt_message_bad_ephemeral_pubkey() {
        let pk = pw123_privkey();
        let m = mutate_blob(BIE1_KAT1, |r| r[4] = 0x00); // invalid compressed-point prefix
        assert!(matches!(
            ecies_decrypt_message(&m, &pk),
            Err(EciesDecryptError::InvalidEphemeralPubkey)
        ));
    }

    #[test]
    fn ecies_decrypt_message_hmac_mismatch_on_mac_tamper() {
        let pk = pw123_privkey();
        let m = mutate_blob(BIE1_KAT1, |r| {
            let n = r.len();
            r[n - 1] ^= 0x01;
        });
        assert!(matches!(
            ecies_decrypt_message(&m, &pk),
            Err(EciesDecryptError::HmacMismatch)
        ));
    }

    #[test]
    fn ecies_decrypt_message_wrong_password_hmac_mismatch() {
        let wrong = derive_storage_eckey(b"not-pw123").unwrap();
        assert!(matches!(
            ecies_decrypt_message(BIE1_KAT1, &wrong),
            Err(EciesDecryptError::HmacMismatch)
        ));
    }

    // ── zlib leg (true cross-impl oracle: Python-stdlib zlib.compress) ─────

    #[test]
    fn zlib_kat_decompresses_python_stdlib_output() {
        // Python: zlib.compress(b'{"seed_version": 18, "use_encryption": false}')
        let comp = hex::decode(
            "789cab562a4e4d4d892f4b2d2acecccf53b25230b4d051502a2d4e8d4fcd4b2eaa2c288188a625e614a7d602006aec0ff2",
        )
        .unwrap();
        let mut out = Vec::new();
        ZlibDecoder::new(comp.as_slice())
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, b"{\"seed_version\": 18, \"use_encryption\": false}");
    }

    // ── ecies_decrypt_storage: composition (zlib ∘ ecies) ─────────────────

    /// Test-only BIE1 ECIES encrypt (caller-fixed ephemeral) for the storage
    /// wiring round-trip. The crypto AUTHORITY is the Electrum KATs + the zlib
    /// KAT above; this only proves the zlib-after-ecies wiring + ECDH symmetry.
    fn ecies_encrypt_storage_for_test(
        json: &[u8],
        password: &[u8],
        ephemeral_sk_bytes: &[u8; 32],
    ) -> String {
        use bitcoin::secp256k1::SecretKey;
        use flate2::{write::ZlibEncoder, Compression};
        use std::io::Write;

        let secp = Secp256k1::new();
        let recip_scalar = derive_storage_eckey(password).unwrap();
        let recip_sk = SecretKey::from_slice(recip_scalar.as_slice()).unwrap();
        let recip_pk = PublicKey::from_secret_key(&secp, &recip_sk);
        let eph_sk = SecretKey::from_slice(ephemeral_sk_bytes).unwrap();
        let eph_pk = PublicKey::from_secret_key(&secp, &eph_sk).serialize(); // 33B
                                                                             // ECDH symmetry: recip_pk * eph_scalar == eph_pk * recip_scalar.
        let eph_scalar = Scalar::from_be_bytes(*ephemeral_sk_bytes).unwrap();
        let ecdh = recip_pk.mul_tweak(&secp, &eph_scalar).unwrap().serialize();
        let mut key = [0u8; 64];
        key.copy_from_slice(&Sha512::digest(&ecdh[..]));
        let (iv, key_e, key_m) = (&key[0..16], &key[16..32], &key[32..64]);

        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        enc.write_all(json).unwrap();
        let compressed = enc.finish().unwrap();

        let padded_len = (compressed.len() / 16 + 1) * 16;
        let mut buf = compressed.clone();
        buf.resize(padded_len, 0);
        let ct_len = cbc::Encryptor::<Aes128>::new_from_slices(key_e, iv)
            .unwrap()
            .encrypt_padded_mut::<Pkcs7>(&mut buf, compressed.len())
            .unwrap()
            .len();
        buf.truncate(ct_len);

        let mut wire = Vec::new();
        wire.extend_from_slice(b"BIE1");
        wire.extend_from_slice(&eph_pk);
        wire.extend_from_slice(&buf);
        let mut h = <HmacSha256 as Mac>::new_from_slice(key_m).unwrap();
        h.update(&wire);
        let mac = h.finalize().into_bytes();
        wire.extend_from_slice(&mac);
        BASE64.encode(&wire)
    }

    #[test]
    fn ecies_decrypt_storage_round_trip_wiring() {
        let json = br#"{"seed_version": 18, "use_encryption": true, "wallet_type": "standard"}"#;
        let blob = ecies_encrypt_storage_for_test(json, b"hunter2", &[0x11u8; 32]);
        let recovered = ecies_decrypt_storage(&blob, b"hunter2").unwrap();
        assert_eq!(recovered.as_slice(), json);
    }

    #[test]
    fn ecies_decrypt_storage_wrong_password_hmac_mismatch() {
        let json = br#"{"seed_version": 18}"#;
        let blob = ecies_encrypt_storage_for_test(json, b"correct horse", &[0x11u8; 32]);
        assert!(matches!(
            ecies_decrypt_storage(&blob, b"battery staple"),
            Err(EciesDecryptError::HmacMismatch)
        ));
    }

    #[test]
    fn ecies_decrypt_storage_non_zlib_plaintext_is_zlib_failure() {
        // The Electrum KAT blobs decrypt to raw (non-zlib) text, so the
        // storage path's zlib.decompress must reject — proving zlib runs
        // AFTER ecies in the composition.
        assert!(matches!(
            ecies_decrypt_storage(BIE1_KAT1, PW123),
            Err(EciesDecryptError::ZlibDecompressFailure(_))
        ));
    }

    // ── error Display smoke ───────────────────────────────────────────────

    #[test]
    fn ecies_error_display_is_nonempty_and_bie2_is_clear() {
        assert!(!EciesDecryptError::HmacMismatch.to_string().is_empty());
        assert!(EciesDecryptError::Bie2Unsupported
            .to_string()
            .contains("hardware-device"));
    }

    // ── detect_storage_magic (Phase B) ────────────────────────────────────

    #[test]
    fn detect_storage_magic_bie1() {
        assert_eq!(
            detect_storage_magic(BIE1_KAT1.as_bytes()),
            Some(ElectrumStorageMagic::Bie1)
        );
    }

    #[test]
    fn detect_storage_magic_bie1_tolerates_trailing_newline() {
        let with_nl = format!("{BIE1_KAT1}\n");
        assert_eq!(
            detect_storage_magic(with_nl.as_bytes()),
            Some(ElectrumStorageMagic::Bie1)
        );
    }

    #[test]
    fn detect_storage_magic_bie2() {
        let bie2 = mutate_blob(BIE1_KAT1, |r| r[3] = b'2'); // BIE1 -> BIE2
        assert_eq!(
            detect_storage_magic(bie2.as_bytes()),
            Some(ElectrumStorageMagic::Bie2)
        );
    }

    #[test]
    fn detect_storage_magic_none_for_json_wallet() {
        // A plaintext Electrum JSON wallet — `{` is not in the base64 alphabet.
        let json = br#"{"seed_version": 18, "use_encryption": false, "wallet_type": "standard"}"#;
        assert_eq!(detect_storage_magic(json), None);
    }

    #[test]
    fn detect_storage_magic_none_for_short_base64() {
        let short = BASE64.encode([0u8; 40]);
        assert_eq!(detect_storage_magic(short.as_bytes()), None);
    }

    #[test]
    fn detect_storage_magic_none_for_non_bie_base64() {
        // Valid base64, >=85 bytes, but magic is not BIE1/BIE2.
        let other = BASE64.encode([0x41u8; 90]);
        assert_eq!(detect_storage_magic(other.as_bytes()), None);
    }
}
