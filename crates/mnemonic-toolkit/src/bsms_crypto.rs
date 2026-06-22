//! BIP-129 encryption-envelope crypto primitives (v0.31.0 / Cycle 7a).
//!
//! Implements PBKDF2-SHA512 key derivation + AES-256-CTR + HMAC-SHA256
//! per BIP-129 §Encryption (verified vs `bitcoin/bips/bip-0129.mediawiki`
//! 2026-05-21):
//!
//! - **Key derivation:** `ENCRYPTION_KEY = PBKDF2-SHA512(password=b"No SPOF",
//!   salt=TOKEN_raw_bytes, c=2048, dkLen=32)`. NOTE: salt is RAW BYTES of
//!   TOKEN (the hex-decoded line-2 value), NOT the ASCII-hex string.
//! - **HMAC key:** `HMAC_KEY = SHA256(ENCRYPTION_KEY)` (single SHA-256;
//!   NOT sha256d).
//! - **MAC:** `MAC = HMAC-SHA256(HMAC_KEY, hex_ascii(TOKEN) || data)`.
//!   NOTE: HMAC input prefix is ASCII-HEX of TOKEN (literally the line-2
//!   hex string bytes), NOT the raw bytes. Same TOKEN, two byte
//!   representations across this module's surface. See R0 §I2 + Cycle 7
//!   P0 recon §A2 for the foot-gun analysis.
//! - **IV:** `IV = first 16 bytes of MAC`.
//! - **Encryption:** AES-256-CTR with `Ctr128BE<Aes256>` (full 16-byte IV
//!   as 128-bit big-endian counter). Locked per Coinkite Python ref
//!   `bsms/encryption.py:34` (`pyaes.Counter(int(iv.hex(), 16))`); per
//!   R0 C1 fold.
//! - **Wire:** `hex(MAC || ciphertext)`.
//! - **AE ordering:** Encrypt-and-MAC (MAC over PLAINTEXT) per BIP-129
//!   line 165.
//!
//! ## Scope (v0.31.0 / Cycle 7a)
//!
//! Pure crypto primitives. CLI integration (Cycle 7b) reads the TOKEN,
//! calls these primitives, verifies MAC, dispatches plaintext to the
//! existing `wallet_import/bsms.rs` parser.
//!
//! ## Error pattern
//!
//! Library-local `BsmsCryptoError` with hand-rolled `impl Display`
//! (mirrors `seed_xor.rs:31-67`). CLI boundary in `cmd/import_wallet.rs`
//! (Cycle 7b) will convert via a boundary mapper to `ToolkitError::BadInput`
//! at orchestrator pre-decrypt time.
//!
//! ## Symmetric `encrypt` helper
//!
//! `encrypt` is exposed as a `pub` helper (NOT `#[cfg(test)]`-gated)
//! mirroring `electrum_crypto.rs::encrypt_field` (Cycle 6a precedent).
//! Production callers use it for fixture generation; the wire-format
//! ciphertext is non-secret and `Vec<u8>` (not `Zeroizing`-wrapped) is
//! correct.

use aes::Aes256;
use ctr::cipher::{KeyIvInit, StreamCipher};
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2;
use sha2::{Digest, Sha256, Sha512};
use zeroize::Zeroizing;

/// AES-256 in CTR mode with full 16-byte IV interpreted as 128-bit
/// big-endian counter, per Coinkite Python `bsms/encryption.py:34`
/// (`pyaes.Counter(int(iv.hex(), 16))`) and BIP-129 §Encryption
/// `IV = First 16 bytes of MAC` (line 154).
type Aes256Ctr = ctr::Ctr128BE<Aes256>;

/// Library-local error. Mapped to `ToolkitError::BadInput` at the CLI
/// boundary in Cycle 7b.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BsmsCryptoError {
    /// Wire-format defect: bad hex, length mismatch, MAC too short, etc.
    InvalidWireFormat { reason: String },
    /// MAC verification failed. Indicates wrong TOKEN OR tampered ciphertext.
    /// (AES-CTR has no padding-oracle exposure; this is a clean authentication
    /// failure under BIP-129's Encrypt-and-MAC ordering.)
    MacMismatch,
}

impl std::fmt::Display for BsmsCryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BsmsCryptoError::InvalidWireFormat { reason } => {
                write!(f, "bsms-crypto: invalid wire format: {reason}")
            }
            BsmsCryptoError::MacMismatch => write!(
                f,
                "bsms-crypto: MAC verification failed (wrong token or tampered ciphertext)"
            ),
        }
    }
}

impl std::error::Error for BsmsCryptoError {}

/// Derive the AES-256-CTR ENCRYPTION_KEY from a TOKEN per BIP-129
/// §Encryption (line 142-148): `PBKDF2-SHA512(password=b"No SPOF",
/// salt=token_raw, c=2048, dkLen=32)`.
///
/// **`token_raw` MUST be the RAW BYTES of the TOKEN** (hex-decoded from
/// the line-2 hex string). For TV-3 example: `[0xa5, 0x40, 0x44, 0x30,
/// 0x8c, 0xea, 0xc9, 0xb7]` (8 bytes), NOT the 16 ASCII chars `a54044...`.
/// Confusion between the two representations is a documented foot-gun
/// (P0 recon §A2). See also [`compute_mac`] which uses the OPPOSITE
/// representation.
pub fn derive_encryption_key(token_raw: &[u8]) -> Zeroizing<[u8; 32]> {
    let mut out = Zeroizing::new([0u8; 32]);
    pbkdf2::<Hmac<Sha512>>(b"No SPOF", token_raw, 2048, out.as_mut_slice())
        .expect("pbkdf2 fill must succeed (dkLen + iters in supported range)");
    out
}

/// Derive the HMAC_KEY from the ENCRYPTION_KEY per BIP-129 line 162:
/// `HMAC_KEY = SHA256(ENCRYPTION_KEY)`.
///
/// Returns `Zeroizing<[u8; 32]>` — the HMAC_KEY is secret-class (derived
/// from the `Zeroizing` ENCRYPTION_KEY) and is scrubbed on drop. The scrub
/// obligation lives in the return type so no caller can leak it by
/// forgetting to wrap (cycle-15 Group A, first-class secret-hygiene).
pub fn derive_hmac_key(encryption_key: &[u8; 32]) -> Zeroizing<[u8; 32]> {
    let mut hasher = Sha256::new();
    hasher.update(encryption_key);
    let mut out = Zeroizing::new([0u8; 32]);
    out.copy_from_slice(&hasher.finalize());
    out
}

/// Compute the BIP-129 MAC over a key/descriptor record per line 152:
/// `MAC = HMAC-SHA256(HMAC_KEY, hex_ascii(TOKEN) || data)`.
///
/// **`token_hex` MUST be the LOWERCASE ASCII HEX representation of TOKEN,
/// NOT the raw bytes.** Per BIP-129 §Encryption and Coinkite Python ref
/// `bsms/encryption.py::m_a_c` (which uses `(token + data).encode()`
/// where `token` is the line-2 hex string). This is the OPPOSITE byte
/// representation from the PBKDF2 salt used in [`derive_encryption_key`].
/// Same TOKEN, two byte forms across this module's surface — the
/// asymmetry is intentional per BIP-129; see [P0 recon §A2 / foot-gun
/// analysis](../../design/cycle-7-p0-recon.md).
///
/// Returns the 32-byte HMAC-SHA256 output as a bare `[u8; 32]`: the MAC is
/// a published BIP-129 authentication tag (its first 16 bytes become the
/// on-wire IV; it is compared against the untrusted `mac_recv`) — NOT
/// secret-class, so it is deliberately un-wrapped.
#[must_use]
pub fn compute_mac(hmac_key: &[u8; 32], token_hex: &str, data: &[u8]) -> [u8; 32] {
    let mut mac =
        <Hmac<Sha256> as Mac>::new_from_slice(hmac_key).expect("HMAC accepts any key length");
    mac.update(token_hex.as_bytes());
    mac.update(data);
    let result = mac.finalize().into_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// AES-256-CTR decrypt per BIP-129 §Encryption (line 158):
/// `Plaintext = AES-256-CTR-Decrypt(Ciphertext, DKey, IV)`.
///
/// Uses `Ctr128BE<Aes256>` (full 16-byte IV as 128-bit big-endian counter
/// initial value) per Coinkite Python `bsms/encryption.py:34`.
///
/// Returns plaintext bytes wrapped in `Zeroizing` (the plaintext is a
/// BIP-129 Round-1 key record or Round-2 descriptor record — both contain
/// secret material like xpubs / signatures / token values).
///
/// The wire format does NOT carry padding; AES-CTR is a stream cipher
/// (plaintext length = ciphertext length). No `MacMismatch` check here;
/// the caller (Cycle 7b CLI orchestrator) does MAC verify after decrypt.
pub fn decrypt(
    ciphertext: &[u8],
    encryption_key: &[u8; 32],
    iv: &[u8; 16],
) -> Result<Zeroizing<Vec<u8>>, BsmsCryptoError> {
    let mut buf = Zeroizing::new(ciphertext.to_vec());
    let mut cipher = Aes256Ctr::new(encryption_key.into(), iv.into());
    cipher.apply_keystream(&mut buf);
    Ok(buf)
}

/// AES-256-CTR encrypt per BIP-129 §Encryption (line 156):
/// `Ciphertext = AES-256-CTR-Encrypt(Plaintext, DKey, IV)`.
///
/// Symmetric inverse of [`decrypt`] (AES-CTR is self-inverse — same
/// operation in both directions). Exposed as `pub` for fixture generation
/// per `electrum_crypto.rs::encrypt_field` (Cycle 6a) precedent. Returns
/// `Vec<u8>` unwrapped because ciphertext is non-secret wire material.
pub fn encrypt(plaintext: &[u8], encryption_key: &[u8; 32], iv: &[u8; 16]) -> Vec<u8> {
    let mut buf = plaintext.to_vec();
    let mut cipher = Aes256Ctr::new(encryption_key.into(), iv.into());
    cipher.apply_keystream(&mut buf);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_hmac_key_returns_zeroizing() {
        // Fn-pointer fence: return types are invariant, so this fails to COMPILE
        // on the bare-[u8;32] signature and compiles only once derive_hmac_key
        // returns Zeroizing<[u8;32]> (cycle-15 Group A first-class-hygiene).
        let _f: fn(&[u8; 32]) -> zeroize::Zeroizing<[u8; 32]> = derive_hmac_key;
    }

    // BIP-129 TV-3 (STANDARD encryption mode, Signer 1).
    // Source: github.com/bitcoin/bips/blob/master/bip-0129.mediawiki §Test Vectors
    // (verified 2026-05-21). Coinkite Python ref cross-validated.
    //
    // TOKEN hex string (line 2 of the Round-1 record).
    const TV3_TOKEN_HEX: &str = "a54044308ceac9b7";
    // TOKEN raw bytes (hex-decoded; used as PBKDF2 salt).
    const TV3_TOKEN_RAW: [u8; 8] = [0xa5, 0x40, 0x44, 0x30, 0x8c, 0xea, 0xc9, 0xb7];
    // ENCRYPTION_KEY = PBKDF2-SHA512(b"No SPOF", TOKEN_RAW, 2048, 32).
    const TV3_ENCRYPTION_KEY_HEX: &str =
        "7673ffd9efd70336a5442eda0b31457f7b6cdf7b42fe17f274434df55efa9839";
    // HMAC_KEY = SHA256(ENCRYPTION_KEY).
    const TV3_HMAC_KEY_HEX: &str =
        "3d4c422806ba8964c9ee45070cd675c024d96648a0ddb4001325818c84951de2";
    // MAC = HMAC-SHA256(HMAC_KEY, ASCII_HEX(TOKEN) || plaintext).
    const TV3_MAC_HEX: &str = "fbdbdb64e6a8231c342131d9f13dcd5a954b4c5021658fa5afcb3fc74dc82706";
    // IV = first 16 bytes of MAC.
    const TV3_IV_HEX: &str = "fbdbdb64e6a8231c342131d9f13dcd5a";
    // CIPHERTEXT (full ciphertext from BIP-129 TV-3 .dat file; 304 hex chars / 152 bytes).
    const TV3_CIPHERTEXT_HEX: &str = "53f491cfd1431c292d922ea5a5dec3eb8ddaa6ed38ae109e7b040f0f23013e89a89b4d27476761a01197a3277850b2bc1621ae626efe65f2081eec6eb571c4f787bf1c49d061b43f70fd73cb3f37fa591d2400973ac0644c8941a83f1d4155e98f01fa2fdeb9f86c2e2413154fd18566a28fb0d9d8bd6172efabcfa6dab09ee7029bf3dd43376df52c118a6d291ec168f4ec7f7df951dfc6135fd8cb4b234da62eaea6017dfe5ca418f083e02e3aba2962ba313ba17b6468c7672fb218329a9f3fe4e4887fb87dac57c63ebff0e715a44498d18de8afc10e1cfeb46a1fc65ce871fef8a43b289305433a90c342d025aa4c19454fcfbcf911e9e2f928d5affd0536a6ddc2e816";
    // Plaintext: Signer 1's 5-line Round-1 record (newline-separated, no trailing newline).
    // Per BIP-129 §Test Vectors STANDARD mode Signer 1.
    const TV3_PLAINTEXT: &str = "BSMS 1.0\na54044308ceac9b7\n[b7868815/48'/0'/0'/2']xpub6FA5rfxJc94K1kNtxRby1hoHwi7YDyTWwx1KUR3FwskaF6HzCbZMz3zQwGnCqdiFeMTPV3YneTGS2YQPiuNYsSvtggWWMQpEJD4jXU7ZzEh\nSigner 1 key\nH8DYht5P6ko0bQqDV6MtUxpzBSK+aVHxbvMavA5byvLrOlCEGmO1WFR7k2wu42J6dxXD8vrmDQSnGq5MTMMbZ98=";

    // ──────────────────────────────────────────────────────────────────────
    // TV-3 cross-validation (the load-bearing happy path)
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn tv3_derive_encryption_key_matches_bip129() {
        let key = derive_encryption_key(&TV3_TOKEN_RAW);
        assert_eq!(hex::encode(key.as_slice()), TV3_ENCRYPTION_KEY_HEX);
    }

    #[test]
    fn tv3_derive_hmac_key_matches_bip129() {
        let enc_key: [u8; 32] = hex::decode(TV3_ENCRYPTION_KEY_HEX)
            .unwrap()
            .try_into()
            .unwrap();
        let hmac_key = derive_hmac_key(&enc_key);
        assert_eq!(hex::encode(hmac_key), TV3_HMAC_KEY_HEX);
    }

    #[test]
    fn tv3_compute_mac_matches_bip129() {
        let hmac_key: [u8; 32] = hex::decode(TV3_HMAC_KEY_HEX).unwrap().try_into().unwrap();
        let mac = compute_mac(&hmac_key, TV3_TOKEN_HEX, TV3_PLAINTEXT.as_bytes());
        assert_eq!(hex::encode(mac), TV3_MAC_HEX);
    }

    #[test]
    fn tv3_iv_is_first_16_bytes_of_mac() {
        let mac_bytes = hex::decode(TV3_MAC_HEX).unwrap();
        let iv_bytes = hex::decode(TV3_IV_HEX).unwrap();
        assert_eq!(&mac_bytes[..16], &iv_bytes[..]);
    }

    #[test]
    fn tv3_decrypt_recovers_bip129_plaintext_under_ctr128be() {
        // Load-bearing cross-validation: Ctr128BE<Aes256> with full 16-byte
        // IV as 128-bit big-endian counter must recover the BIP-129 TV-3
        // 5-line Round-1 plaintext byte-identical.
        let enc_key: [u8; 32] = hex::decode(TV3_ENCRYPTION_KEY_HEX)
            .unwrap()
            .try_into()
            .unwrap();
        let iv: [u8; 16] = hex::decode(TV3_IV_HEX).unwrap().try_into().unwrap();
        let ciphertext = hex::decode(TV3_CIPHERTEXT_HEX).unwrap();
        let plaintext = decrypt(&ciphertext, &enc_key, &iv).unwrap();
        assert_eq!(plaintext.as_slice(), TV3_PLAINTEXT.as_bytes());
    }

    #[test]
    fn tv3_encrypt_produces_bip129_ciphertext_byte_identical() {
        // Symmetric inverse: encrypting TV-3 plaintext with the TV-3
        // ENCRYPTION_KEY + IV must produce the BIP-129 TV-3 ciphertext.
        let enc_key: [u8; 32] = hex::decode(TV3_ENCRYPTION_KEY_HEX)
            .unwrap()
            .try_into()
            .unwrap();
        let iv: [u8; 16] = hex::decode(TV3_IV_HEX).unwrap().try_into().unwrap();
        let ciphertext = encrypt(TV3_PLAINTEXT.as_bytes(), &enc_key, &iv);
        assert_eq!(hex::encode(&ciphertext), TV3_CIPHERTEXT_HEX);
    }

    #[test]
    fn tv3_end_to_end_round_trip() {
        // Full pipeline: TOKEN raw → derive_encryption_key → derive_hmac_key →
        // compute_mac → IV (first 16 bytes of MAC) → encrypt → decrypt →
        // byte-identical plaintext recovery + MAC equality.
        let enc_key = derive_encryption_key(&TV3_TOKEN_RAW);
        let hmac_key = derive_hmac_key(&enc_key);
        let mac = compute_mac(&hmac_key, TV3_TOKEN_HEX, TV3_PLAINTEXT.as_bytes());
        assert_eq!(hex::encode(mac), TV3_MAC_HEX);

        let iv: [u8; 16] = mac[..16].try_into().unwrap();
        let ciphertext = encrypt(TV3_PLAINTEXT.as_bytes(), &enc_key, &iv);
        assert_eq!(hex::encode(&ciphertext), TV3_CIPHERTEXT_HEX);

        let recovered = decrypt(&ciphertext, &enc_key, &iv).unwrap();
        assert_eq!(recovered.as_slice(), TV3_PLAINTEXT.as_bytes());
    }

    // ──────────────────────────────────────────────────────────────────────
    // Foot-gun cells: PBKDF2-salt-raw vs HMAC-input-ASCII-hex asymmetry
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn pbkdf2_salt_with_ascii_hex_does_not_match_tv3() {
        // Negative cell: deriving the key with ASCII-hex of TOKEN as salt
        // (the WRONG representation) must NOT produce the TV-3 ENCRYPTION_KEY.
        let key = derive_encryption_key(TV3_TOKEN_HEX.as_bytes());
        assert_ne!(hex::encode(key.as_slice()), TV3_ENCRYPTION_KEY_HEX);
    }

    #[test]
    fn compute_mac_with_uppercase_token_hex_does_not_match_tv3() {
        // Negative cell documenting the lowercase-ASCII-hex contract: the
        // exact uppercase-cased TOKEN hex (the WRONG representation per
        // BIP-129 + Coinkite Python which both use lowercase) must NOT
        // produce the TV-3 MAC.
        let hmac_key: [u8; 32] = hex::decode(TV3_HMAC_KEY_HEX).unwrap().try_into().unwrap();
        let uppercase_token_hex = TV3_TOKEN_HEX.to_uppercase();
        let mac = compute_mac(&hmac_key, &uppercase_token_hex, TV3_PLAINTEXT.as_bytes());
        assert_ne!(hex::encode(mac), TV3_MAC_HEX);
    }

    // ──────────────────────────────────────────────────────────────────────
    // NO_ENCRYPTION TOKEN sanity (BIP-129 TV-1/TV-2 framing)
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn no_encryption_token_derives_deterministic_key() {
        // NO_ENCRYPTION mode TOKEN = 0x00 (one byte). PBKDF2 deterministic
        // → reproducible across runs.
        let token_raw = [0x00u8];
        let key_a = derive_encryption_key(&token_raw);
        let key_b = derive_encryption_key(&token_raw);
        assert_eq!(key_a.as_slice(), key_b.as_slice());
        // Non-zero output.
        assert_ne!(key_a.as_slice(), &[0u8; 32]);
    }

    // ──────────────────────────────────────────────────────────────────────
    // Round-trip cells with synthetic inputs
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn encrypt_decrypt_round_trip_short_plaintext() {
        let enc_key: [u8; 32] = [0x42; 32];
        let iv: [u8; 16] = [0x77; 16];
        let plaintext = b"short message";
        let ciphertext = encrypt(plaintext, &enc_key, &iv);
        let recovered = decrypt(&ciphertext, &enc_key, &iv).unwrap();
        assert_eq!(recovered.as_slice(), plaintext);
    }

    #[test]
    fn encrypt_decrypt_round_trip_block_boundary_16_bytes() {
        let enc_key: [u8; 32] = [0x42; 32];
        let iv: [u8; 16] = [0x77; 16];
        let plaintext = b"sixteen_byte_msg"; // exactly 16 bytes
        let ciphertext = encrypt(plaintext, &enc_key, &iv);
        let recovered = decrypt(&ciphertext, &enc_key, &iv).unwrap();
        assert_eq!(recovered.as_slice(), plaintext);
    }

    #[test]
    fn encrypt_decrypt_round_trip_block_boundary_17_bytes() {
        let enc_key: [u8; 32] = [0x42; 32];
        let iv: [u8; 16] = [0x77; 16];
        let plaintext = b"seventeen_byte_x!"; // 17 bytes (crosses AES block boundary)
        let ciphertext = encrypt(plaintext, &enc_key, &iv);
        let recovered = decrypt(&ciphertext, &enc_key, &iv).unwrap();
        assert_eq!(recovered.as_slice(), plaintext);
    }

    #[test]
    fn encrypt_decrypt_round_trip_empty_plaintext() {
        // AES-CTR is a stream cipher; empty plaintext → empty ciphertext.
        let enc_key: [u8; 32] = [0x42; 32];
        let iv: [u8; 16] = [0x77; 16];
        let ciphertext = encrypt(b"", &enc_key, &iv);
        assert_eq!(ciphertext.len(), 0);
        let recovered = decrypt(&ciphertext, &enc_key, &iv).unwrap();
        assert_eq!(recovered.as_slice(), b"");
    }

    #[test]
    fn encrypt_is_aes_ctr_self_inverse() {
        // AES-CTR is self-inverse: encrypt(encrypt(p)) = p.
        let enc_key: [u8; 32] = [0x42; 32];
        let iv: [u8; 16] = [0x77; 16];
        let plaintext = b"self-inverse test";
        let once = encrypt(plaintext, &enc_key, &iv);
        let twice = encrypt(&once, &enc_key, &iv);
        assert_eq!(twice.as_slice(), plaintext);
    }

    #[test]
    fn different_iv_produces_different_ciphertext() {
        let enc_key: [u8; 32] = [0x42; 32];
        let iv_a: [u8; 16] = [0x00; 16];
        let iv_b: [u8; 16] = [0xff; 16];
        let plaintext = b"same key, different IV";
        let ct_a = encrypt(plaintext, &enc_key, &iv_a);
        let ct_b = encrypt(plaintext, &enc_key, &iv_b);
        assert_ne!(ct_a, ct_b);
    }

    #[test]
    fn different_key_produces_different_ciphertext() {
        let enc_key_a: [u8; 32] = [0x42; 32];
        let enc_key_b: [u8; 32] = [0x43; 32];
        let iv: [u8; 16] = [0x77; 16];
        let plaintext = b"different key";
        let ct_a = encrypt(plaintext, &enc_key_a, &iv);
        let ct_b = encrypt(plaintext, &enc_key_b, &iv);
        assert_ne!(ct_a, ct_b);
    }

    // ──────────────────────────────────────────────────────────────────────
    // MAC-related cells (no MacMismatch error path at the library level —
    // that's a Cycle 7b CLI-orchestrator responsibility — but we cover the
    // primitives that feed the MAC verify.)
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn mac_changes_with_different_data() {
        let hmac_key: [u8; 32] = [0x42; 32];
        let token_hex = "deadbeef";
        let mac_a = compute_mac(&hmac_key, token_hex, b"data-A");
        let mac_b = compute_mac(&hmac_key, token_hex, b"data-B");
        assert_ne!(mac_a, mac_b);
    }

    #[test]
    fn mac_changes_with_different_token_hex() {
        let hmac_key: [u8; 32] = [0x42; 32];
        let mac_a = compute_mac(&hmac_key, "deadbeef", b"data");
        let mac_b = compute_mac(&hmac_key, "cafef00d", b"data");
        assert_ne!(mac_a, mac_b);
    }

    #[test]
    fn mac_deterministic_for_identical_inputs() {
        let hmac_key: [u8; 32] = [0x42; 32];
        let mac_a = compute_mac(&hmac_key, "deadbeef", b"data");
        let mac_b = compute_mac(&hmac_key, "deadbeef", b"data");
        assert_eq!(mac_a, mac_b);
    }
}
