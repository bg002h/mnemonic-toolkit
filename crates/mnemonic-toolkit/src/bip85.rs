//! BIP-85 deterministic entropy + per-app dispatchers.
//!
//! Realizes `design/SPEC_derive_child_v0_7.md` §3 (path + HMAC primitive)
//! and §4 (in-scope applications). The 6 in-scope apps:
//! BIP-39 / HD-Seed WIF / XPRV / HEX / PWD BASE64 / PWD BASE85.
//!
//! Reference: <https://github.com/bitcoin/bips/blob/master/bip-0085.mediawiki>.

use crate::error::{BitcoinErrorKind, ToolkitError};
use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::hashes::{sha512, Hash, HashEngine, Hmac, HmacEngine};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::NetworkKind;

/// BIP-85 §"Specification" — derive 64 bytes of entropy from a master xprv
/// at path `m/83696968'/<app_code>'/<app_params...>'/<index>'`, then
/// HMAC-SHA512(key=`b"bip-entropy-from-k"`, msg=child.private_key).
pub(crate) fn derive_entropy(
    master: &Xpriv,
    app_code: u32,
    app_params: &[u32],
    index: u32,
) -> Result<[u8; 64], ToolkitError> {
    let secp = Secp256k1::new();
    let mut components: Vec<ChildNumber> = Vec::with_capacity(3 + app_params.len());
    components.push(hardened(83_696_968)?);
    components.push(hardened(app_code)?);
    for &p in app_params {
        components.push(hardened(p)?);
    }
    components.push(hardened(index)?);
    let path = DerivationPath::from(components);
    let child = master
        .derive_priv(&secp, &path)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;

    let mut engine = HmacEngine::<sha512::Hash>::new(b"bip-entropy-from-k");
    engine.input(&child.private_key.secret_bytes());
    let mac = Hmac::<sha512::Hash>::from_engine(engine);
    let mut out = [0u8; 64];
    out.copy_from_slice(mac.as_byte_array());
    Ok(out)
}

fn hardened(n: u32) -> Result<ChildNumber, ToolkitError> {
    ChildNumber::from_hardened_idx(n)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))
}

// ============================================================================
// BIP-39 (app 39')
// ============================================================================

/// SPEC §4 — BIP-39 entropy → mnemonic. `words` must be one of 12/15/18/21/24.
/// Path m/83696968'/39'/<language_code>'/<words>'/<index>'.
/// `language_code` indexes the BIP-85 path tree (per BIP-85 §"Language Codes");
/// `language` is the matching `bip39::Language` for wordlist selection. Caller
/// is responsible for keeping the two consistent (see
/// `crate::cmd::derive_child::resolve_bip85_language`).
pub(crate) fn format_bip39_phrase(
    master: &Xpriv,
    language_code: u32,
    language: bip39::Language,
    words: u32,
    index: u32,
) -> Result<String, ToolkitError> {
    let entropy = derive_entropy(master, 39, &[language_code, words], index)?;
    // BIP-39 entropy bytes = words * 4 / 3 (12→16, 15→20, 18→24, 21→28, 24→32).
    let bytes: usize = (words as usize) * 4 / 3;
    let mnemonic =
        Mnemonic::from_entropy_in(language, &entropy[..bytes]).map_err(ToolkitError::Bip39)?;
    Ok(mnemonic.to_string())
}

// ============================================================================
// HD-Seed WIF (app 2')
// ============================================================================

/// SPEC §4 — HD-Seed WIF. Path m/83696968'/2'/<index>'.
/// Output is a WIF-encoded 32-byte privkey from the FIRST 32 bytes of the
/// 64-byte entropy (BIP-85 reference impl convention; verified against
/// BIP-85 §"Test Vectors" for index 0 + mainnet producing
/// `Kzyv4uF39d4Jrw2W7UryTHwZr1zQVNk4dAFyqE6BuMrMh1Za7uhp`).
/// `network` selects the WIF prefix (`K…`/`L…` mainnet vs `c…` testnet).
pub(crate) fn format_hd_seed_wif(
    master: &Xpriv,
    index: u32,
    network: NetworkKind,
) -> Result<String, ToolkitError> {
    let entropy = derive_entropy(master, 2, &[], index)?;
    let inner = bitcoin::secp256k1::SecretKey::from_slice(&entropy[..32])
        .map_err(|e| ToolkitError::BadInput(format!("BIP-85 hd-seed scalar parse: {e}")))?;
    let pk = bitcoin::PrivateKey {
        compressed: true,
        network,
        inner,
    };
    Ok(pk.to_wif())
}

// ============================================================================
// XPRV (app 32')
// ============================================================================

/// SPEC §4 — Child xprv. Path m/83696968'/32'/<index>'.
/// First 32 bytes = chain code, last 32 bytes = privkey (BIP-85 §"XPRV").
/// `network` selects the prefix (`xprv…` mainnet vs `tprv…` testnet).
pub(crate) fn format_xprv_child(
    master: &Xpriv,
    index: u32,
    network: NetworkKind,
) -> Result<String, ToolkitError> {
    let entropy = derive_entropy(master, 32, &[], index)?;
    let chain_code = bitcoin::bip32::ChainCode::from(<[u8; 32]>::try_from(&entropy[..32]).unwrap());
    let inner = bitcoin::secp256k1::SecretKey::from_slice(&entropy[32..])
        .map_err(|e| ToolkitError::BadInput(format!("BIP-85 xprv scalar parse: {e}")))?;
    let xprv = Xpriv {
        network,
        depth: 0,
        parent_fingerprint: bitcoin::bip32::Fingerprint::default(),
        child_number: ChildNumber::Normal { index: 0 },
        private_key: inner,
        chain_code,
    };
    Ok(xprv.to_string())
}

// ============================================================================
// HEX (app 128169')
// ============================================================================

/// SPEC §4 — N raw hex bytes (16 ≤ N ≤ 64).
/// Path m/83696968'/128169'/<num_bytes>'/<index>'.
pub(crate) fn format_hex_bytes(
    master: &Xpriv,
    num_bytes: u32,
    index: u32,
) -> Result<String, ToolkitError> {
    let entropy = derive_entropy(master, 128_169, &[num_bytes], index)?;
    Ok(hex::encode(&entropy[..num_bytes as usize]))
}

// ============================================================================
// PWD BASE64 (app 707764') / PWD BASE85 (app 707785')
// ============================================================================

/// SPEC §4 — Base64 password (20 ≤ length ≤ 86).
/// Path m/83696968'/707764'/<length>'/<index>'. Encode 64 bytes via standard
/// Base64 (RFC 4648 §4 alphabet, `+/`, no padding stripping needed since
/// 64 → 88 chars w/ padding) and truncate to `length` chars.
pub(crate) fn format_password_base64(
    master: &Xpriv,
    length: u32,
    index: u32,
) -> Result<String, ToolkitError> {
    let entropy = derive_entropy(master, 707_764, &[length], index)?;
    let encoded = base64_standard(&entropy);
    Ok(encoded[..length as usize].to_string())
}

/// SPEC §4 — Base85 password (10 ≤ length ≤ 80).
/// Path m/83696968'/707785'/<length>'/<index>'. Encode 64 bytes via
/// RFC 1924 / btcpayserver-friendly Ascii85 (`Z85`-adjacent — see helper).
/// Truncate to `length` chars.
pub(crate) fn format_password_base85(
    master: &Xpriv,
    length: u32,
    index: u32,
) -> Result<String, ToolkitError> {
    let entropy = derive_entropy(master, 707_785, &[length], index)?;
    let encoded = base85_btc(&entropy);
    Ok(encoded[..length as usize].to_string())
}

// ============================================================================
// Encoders (hand-rolled — neither base64 nor base85 are toolkit deps).
// ============================================================================

/// Standard Base64 (RFC 4648) encoder. Handles arbitrary input length and
/// emits `=` padding. The BIP-85 PWD BASE64 application slices the first
/// `length` chars; padding chars never appear in the slice for the SPEC
/// `length ≤ 86` range when input is 64 bytes (output is 88 chars including
/// 2 trailing `=`; `length` cap of 86 excludes both pads).
fn base64_standard(input: &[u8]) -> String {
    const ALPHA: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= input.len() {
        let b0 = input[i];
        let b1 = input[i + 1];
        let b2 = input[i + 2];
        out.push(ALPHA[(b0 >> 2) as usize] as char);
        out.push(ALPHA[((b0 & 0x03) << 4 | b1 >> 4) as usize] as char);
        out.push(ALPHA[((b1 & 0x0f) << 2 | b2 >> 6) as usize] as char);
        out.push(ALPHA[(b2 & 0x3f) as usize] as char);
        i += 3;
    }
    let rem = input.len() - i;
    if rem == 1 {
        let b0 = input[i];
        out.push(ALPHA[(b0 >> 2) as usize] as char);
        out.push(ALPHA[((b0 & 0x03) << 4) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let b0 = input[i];
        let b1 = input[i + 1];
        out.push(ALPHA[(b0 >> 2) as usize] as char);
        out.push(ALPHA[((b0 & 0x03) << 4 | b1 >> 4) as usize] as char);
        out.push(ALPHA[((b1 & 0x0f) << 2) as usize] as char);
        out.push('=');
    }
    out
}

/// "BTC base85" — the BIP-85 reference implementation
/// (<https://github.com/ethankosakovsky/bip85/blob/master/bip85.py>) uses the
/// `base64.b85encode` flavor from Python's stdlib, alphabet:
/// `0-9`, `A-Z`, `a-z`, `!#$%&()*+-;<=>?@^_\`{|}~`
/// (RFC 1924 alphabet, also called `b85`). Encodes 4 bytes → 5 chars
/// per chunk; trailing partial chunk pads with NUL on encode then truncates
/// the output. For BIP-85 PWD BASE85 the input is always 64 bytes (a clean
/// multiple of 4) so no trailing-padding logic is needed.
fn base85_btc(input: &[u8]) -> String {
    const ALPHA: &[u8; 85] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+-;<=>?@^_`{|}~";
    debug_assert!(
        input.len() % 4 == 0,
        "base85_btc: caller must pass 4-byte-aligned input (BIP-85 always 64 bytes)",
    );
    let mut out = String::with_capacity(input.len() / 4 * 5);
    for chunk in input.chunks_exact(4) {
        let mut n: u32 =
            (chunk[0] as u32) << 24 | (chunk[1] as u32) << 16 | (chunk[2] as u32) << 8 | (chunk[3] as u32);
        let mut group = [0u8; 5];
        for slot in group.iter_mut().rev() {
            *slot = ALPHA[(n % 85) as usize];
            n /= 85;
        }
        out.push_str(std::str::from_utf8(&group).expect("ALPHA is ASCII"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const MASTER_XPRV: &str =
        "xprv9s21ZrQH143K2LBWUUQRFXhucrQqBpKdRRxNVq2zBqsx8HVqFk2uYo8kmbaLLHRdqtQpUm98uKfu3vca1LqdGhUtyoFnCNkfmXRyPXLjbKb";

    fn master() -> Xpriv {
        Xpriv::from_str(MASTER_XPRV).unwrap()
    }

    /// BIP-85 §"Test Vectors" — BIP-39 12 English words at index 0 emits
    /// the spec's pinned 16-byte entropy.
    #[test]
    fn bip39_12_words_entropy_matches_spec() {
        let e = derive_entropy(&master(), 39, &[0, 12], 0).unwrap();
        assert_eq!(hex::encode(&e[..16]), "6250b68daf746d12a24d58b4787a714b");
    }

    /// BIP-85 §"Test Vectors" — HEX 64 bytes at index 0 emits the spec's
    /// pinned 64-byte entropy verbatim.
    #[test]
    fn hex_64_bytes_entropy_matches_spec() {
        let e = derive_entropy(&master(), 128_169, &[64], 0).unwrap();
        assert_eq!(
            hex::encode(e),
            "492db4698cf3b73a5a24998aa3e9d7fa96275d85724a91e71aa2d645442f878555d078fd1f1f67e368976f04137b1f7a0d19232136ca50c44614af72b5582a5c",
        );
    }

    /// BIP-85 §"Test Vectors" — PWD BASE64 length 21 at index 0.
    #[test]
    fn pwd_base64_matches_spec() {
        let pwd = format_password_base64(&master(), 21, 0).unwrap();
        assert_eq!(pwd, "dKLoepugzdVJvdL56ogNV");
    }

    /// BIP-85 §"Test Vectors" — PWD BASE85 length 12 at index 0.
    #[test]
    fn pwd_base85_matches_spec() {
        let pwd = format_password_base85(&master(), 12, 0).unwrap();
        assert_eq!(pwd, "_s`{TW89)i4`");
    }
}
