//! BIP-85 deterministic entropy + per-app dispatchers.
//!
//! Realizes `design/SPEC_derive_child_v0_7.md` §3 (path + HMAC primitive)
//! and §4 (in-scope applications). The 6 in-scope apps:
//! BIP-39 / HD-Seed WIF / XPRV / HEX / PWD BASE64 / PWD BASE85.
//!
//! Reference: <https://github.com/bitcoin/bips/blob/master/bip-0085.mediawiki>.

use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::secret_string::SecretString;
use bip39::Mnemonic;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpriv};
use bitcoin::hashes::{sha512, Hash, HashEngine, Hmac, HmacEngine};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::NetworkKind;
use sha3::digest::{ExtendableOutput, Update, XofReader};
use sha3::Shake256;
use zeroize::Zeroizing;

/// BIP-85 §"Specification" — derive 64 bytes of entropy from a master xprv
/// at path `m/83696968'/<app_code>'/<app_params...>'/<index>'`, then
/// HMAC-SHA512(key=`b"bip-entropy-from-k"`, msg=child.private_key).
///
/// Cycle B Phase 1: the 64-byte buffer is heap-resident (`Vec<u8>`) so that
/// Phase 3's `MlockedZeroizing<Vec<u8>>` wrapper can pin its pages — mlock
/// requires heap memory. Length is invariantly 64 (HMAC-SHA512 output);
/// asserted via `debug_assert_eq!` before return.
pub(crate) fn derive_entropy(
    master: &Xpriv,
    app_code: u32,
    app_params: &[u32],
    index: u32,
) -> Result<Zeroizing<Vec<u8>>, ToolkitError> {
    let secp = Secp256k1::new();
    let mut components: Vec<ChildNumber> = Vec::with_capacity(3 + app_params.len());
    components.push(hardened(83_696_968)?);
    components.push(hardened(app_code)?);
    for &p in app_params {
        components.push(hardened(p)?);
    }
    components.push(hardened(index)?);
    let path = DerivationPath::from(components);
    // SAFETY: third-party-blocked — `Xpriv::derive_priv` returns a `Copy`
    // type with no Drop hook; tracked by FOLLOWUP
    // `rust-bitcoin-xpriv-zeroize-upstream`.
    let child = master
        .derive_priv(&secp, &path)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;

    let mut engine = HmacEngine::<sha512::Hash>::new(b"bip-entropy-from-k");
    engine.input(&child.private_key.secret_bytes());
    let mac = Hmac::<sha512::Hash>::from_engine(engine);
    let mut out = Zeroizing::new(vec![0u8; 64]);
    out.copy_from_slice(mac.as_byte_array());
    debug_assert_eq!(out.len(), 64, "BIP-85 entropy is 64-byte invariant");
    Ok(out)
}

fn hardened(n: u32) -> Result<ChildNumber, ToolkitError> {
    ChildNumber::from_hardened_idx(n).map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))
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
) -> Result<SecretString, ToolkitError> {
    let entropy = derive_entropy(master, 39, &[language_code, words], index)?;
    // Cycle B Phase 3a Site 4 — pin the bip85-derived entropy heap pages
    // for the function-body lifetime. Drop order: _entropy_pin munlocks
    // first, then `entropy: Zeroizing<Vec<u8>>` zeroizes (Phase 1).
    let _entropy_pin = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);
    // BIP-39 entropy bytes = words * 4 / 3 (12→16, 15→20, 18→24, 21→28, 24→32).
    let bytes: usize = (words as usize) * 4 / 3;
    // SAFETY: third-party-blocked — `bip39::Mnemonic` has no Drop+Zeroize;
    // tracked by FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`.
    let mnemonic =
        Mnemonic::from_entropy_in(language, &entropy[..bytes]).map_err(ToolkitError::Bip39)?;
    Ok(SecretString::new(mnemonic.to_string()))
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
) -> Result<SecretString, ToolkitError> {
    let entropy = derive_entropy(master, 2, &[], index)?;
    let _entropy_pin = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);
    // SAFETY: third-party-blocked — `secp256k1::SecretKey` is stack-bound,
    // has `non_secure_erase` but no Drop+Zeroize; tracked by FOLLOWUP
    // `rust-secp256k1-secretkey-zeroize-upstream`. The 32-byte scalar
    // lives in stack memory until function exit.
    let inner = bitcoin::secp256k1::SecretKey::from_slice(&entropy[..32])
        .map_err(|e| ToolkitError::BadInput(format!("BIP-85 hd-seed scalar parse: {e}")))?;
    let pk = bitcoin::PrivateKey {
        compressed: true,
        network,
        inner,
    };
    Ok(SecretString::new(pk.to_wif()))
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
) -> Result<SecretString, ToolkitError> {
    let entropy = derive_entropy(master, 32, &[], index)?;
    let _entropy_pin = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);
    let chain_code = bitcoin::bip32::ChainCode::from(<[u8; 32]>::try_from(&entropy[..32]).unwrap());
    // SAFETY: third-party-blocked — `secp256k1::SecretKey` is stack-bound,
    // no Drop+Zeroize; FOLLOWUP `rust-secp256k1-secretkey-zeroize-upstream`.
    // Plus `bitcoin::bip32::Xpriv` (already tracked at
    // `rust-bitcoin-xpriv-zeroize-upstream`). Both stack-bound until
    // function exit.
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
    Ok(SecretString::new(xprv.to_string()))
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
) -> Result<SecretString, ToolkitError> {
    let entropy = derive_entropy(master, 128_169, &[num_bytes], index)?;
    let _entropy_pin = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);
    Ok(SecretString::new(hex::encode(
        &entropy[..num_bytes as usize],
    )))
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
) -> Result<SecretString, ToolkitError> {
    let entropy = derive_entropy(master, 707_764, &[length], index)?;
    let _entropy_pin = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);
    let encoded = Zeroizing::new(base64_standard(&entropy[..]));
    Ok(SecretString::new(encoded[..length as usize].to_string()))
}

/// SPEC §4 — Base85 password (10 ≤ length ≤ 80).
/// Path m/83696968'/707785'/<length>'/<index>'. Encode 64 bytes via
/// RFC 1924 / btcpayserver-friendly Ascii85 (`Z85`-adjacent — see helper).
/// Truncate to `length` chars.
pub(crate) fn format_password_base85(
    master: &Xpriv,
    length: u32,
    index: u32,
) -> Result<SecretString, ToolkitError> {
    let entropy = derive_entropy(master, 707_785, &[length], index)?;
    let _entropy_pin = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);
    let encoded = Zeroizing::new(base85_btc(&entropy[..]));
    Ok(SecretString::new(encoded[..length as usize].to_string()))
}

// ============================================================================
// DICE (app 89101')
// ============================================================================

/// SPEC v0.8 §4 — BIP-85 DICE rolls. Path m/83696968'/89101'/<sides>'/<rolls>'/<index>'.
/// Per BIP-85 v1.3.0 §"DICE" the algorithm is:
///   1. Derive 64 bytes of BIP-85 entropy at the spec path.
///   2. Seed a SHAKE256 DRNG with the entropy (BIP85-DRNG-SHAKE256).
///   3. For each roll: read `bytes_per_roll = ceil(bits_per_roll / 8)` bytes,
///      keep the most significant `bits_per_roll = ceil(log_2(sides))` bits,
///      reject the trial if it's >= sides; otherwise accept as a roll value
///      in `[0, sides-1]`.
///   4. Output rolls separated by `,` (per BIP-85 spec example).
///
/// Constraints: `2 <= sides <= 2^32 - 1`, `1 <= rolls <= 2^32 - 1`.
pub(crate) fn format_dice_rolls(
    master: &Xpriv,
    sides: u32,
    rolls: u32,
    index: u32,
) -> Result<SecretString, ToolkitError> {
    if sides < 2 {
        return Err(ToolkitError::BadInput(format!(
            "BIP-85 dice: sides must be >= 2, got {sides}",
        )));
    }
    if rolls < 1 {
        return Err(ToolkitError::BadInput(
            "BIP-85 dice: rolls must be >= 1".into(),
        ));
    }

    let entropy = derive_entropy(master, 89_101, &[sides, rolls], index)?;
    let _entropy_pin = mnemonic_toolkit::mlock::pin_pages_for(&entropy[..]);

    // BIP85-DRNG-SHAKE256: seed a SHAKE256 stream with the 64-byte entropy.
    let mut shake = Shake256::default();
    shake.update(&entropy[..]);
    let mut reader = shake.finalize_xof();

    // bits_per_roll = ceil(log_2(sides)); bytes_per_roll = ceil(bits_per_roll / 8).
    let bits_per_roll = u32::BITS - (sides - 1).leading_zeros();
    let bytes_per_roll = bits_per_roll.div_ceil(8) as usize;

    let mut out: Zeroizing<Vec<String>> = Zeroizing::new(Vec::with_capacity(rolls as usize));
    let mut buf = vec![0u8; bytes_per_roll];
    while out.len() < rolls as usize {
        reader.read(&mut buf);
        // Big-endian assembly of bytes into a u32 trial value (sides up to
        // 2^32 - 1 fits in u32).
        let mut trial: u32 = 0;
        for &b in &buf {
            trial = (trial << 8) | (b as u32);
        }
        // Trim excess low bits — BIP-85 spec says "retain the most significant
        // `bits_per_roll` bits". `bytes_per_roll * 8` bits total, keep top
        // `bits_per_roll`, so right-shift by the difference.
        let total_bits = (bytes_per_roll as u32) * 8;
        let shift = total_bits - bits_per_roll;
        trial >>= shift;
        if trial < sides {
            out.push(trial.to_string());
        }
        // else: rejection sample — read the next chunk.
    }

    Ok(SecretString::new(out.join(",")))
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
    const ALPHA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= input.len() {
        let b0 = input[i];
        let b1 = input[i + 1];
        let b2 = input[i + 2];
        out.push(ALPHA[(b0 >> 2) as usize] as char);
        out.push(ALPHA[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(ALPHA[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
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
        out.push(ALPHA[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
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
    const ALPHA: &[u8; 85] =
        b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+-;<=>?@^_`{|}~";
    debug_assert!(
        input.len() % 4 == 0,
        "base85_btc: caller must pass 4-byte-aligned input (BIP-85 always 64 bytes)",
    );
    let mut out = String::with_capacity(input.len() / 4 * 5);
    for chunk in input.chunks_exact(4) {
        let mut n: u32 = ((chunk[0] as u32) << 24)
            | ((chunk[1] as u32) << 16)
            | ((chunk[2] as u32) << 8)
            | (chunk[3] as u32);
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
        assert_eq!(&*pwd, "dKLoepugzdVJvdL56ogNV");
    }

    /// BIP-85 §"Test Vectors" — PWD BASE85 length 12 at index 0.
    #[test]
    fn pwd_base85_matches_spec() {
        let pwd = format_password_base85(&master(), 12, 0).unwrap();
        assert_eq!(&*pwd, "_s`{TW89)i4`");
    }

    /// BIP-85 v1.3.0 §"DICE" reference vector.
    /// Path m/83696968'/89101'/6'/10'/0' → rolls 1,0,0,2,0,1,5,5,2,4.
    #[test]
    fn dice_d6_10_rolls_matches_spec() {
        let rolls = format_dice_rolls(&master(), 6, 10, 0).unwrap();
        assert_eq!(&*rolls, "1,0,0,2,0,1,5,5,2,4");
    }

    // ========================================================================
    // cycle-15t T1 — Slug-1 type-level fence: every `format_*` returns a
    // `SecretString` (length-only redacting Debug), NOT a bare `String`.
    // RED until the 7 returns flip; mirrors the bip85 lint type rows.
    // ========================================================================

    /// T1 — the 7 `format_*` fn pointers coerce to `-> Result<SecretString, _>`.
    /// A bare-`String` return makes this fail to compile (RED), proving the
    /// type-level fence rather than a runtime check.
    #[test]
    fn t1_format_fns_return_secret_string() {
        use crate::secret_string::SecretString;
        let _f1: fn(&Xpriv, u32, bip39::Language, u32, u32) -> Result<SecretString, ToolkitError> =
            format_bip39_phrase;
        let _f2: fn(&Xpriv, u32, NetworkKind) -> Result<SecretString, ToolkitError> =
            format_hd_seed_wif;
        let _f3: fn(&Xpriv, u32, NetworkKind) -> Result<SecretString, ToolkitError> =
            format_xprv_child;
        let _f4: fn(&Xpriv, u32, u32) -> Result<SecretString, ToolkitError> = format_hex_bytes;
        let _f5: fn(&Xpriv, u32, u32) -> Result<SecretString, ToolkitError> =
            format_password_base64;
        let _f6: fn(&Xpriv, u32, u32) -> Result<SecretString, ToolkitError> =
            format_password_base85;
        let _f7: fn(&Xpriv, u32, u32, u32) -> Result<SecretString, ToolkitError> =
            format_dice_rolls;
    }

    /// T2 — Debug-redaction: `{:?}` on a `format_*` output never leaks the
    /// secret substring and DOES carry the redaction marker (proves the chosen
    /// `SecretString`, not a bare `Zeroizing<String>` whose tuple Debug would
    /// print the plaintext — the cycle-14 leak class). Mirrors
    /// `secret_string.rs::debug_redacts_the_secret`.
    #[test]
    fn t2_format_outputs_debug_redacts() {
        let pwd = format_password_base64(&master(), 21, 0).unwrap();
        let dbg = format!("{pwd:?}");
        assert!(
            !dbg.contains("dKLoepugzdVJvdL56ogNV"),
            "Debug leaked the derived password: {dbg}"
        );
        assert!(
            dbg.contains("redacted"),
            "Debug should mark redaction: {dbg}"
        );

        let wif = format_hd_seed_wif(&master(), 0, NetworkKind::Main).unwrap();
        let wif_plain: String = (*wif).to_string();
        let wif_dbg = format!("{wif:?}");
        assert!(
            !wif_dbg.contains(&wif_plain),
            "Debug leaked the HD-seed WIF: {wif_dbg}"
        );
        assert!(wif_dbg.contains("redacted"));
    }

    // ========================================================================
    // Path B-lite Site 4 — bip85 function-local pin coverage.
    //
    // Tests assert `attempts_for_test() > baseline` after a production code
    // path that should pin. record_attempt fires unconditionally on every
    // pin_pages_for call (mlock.rs:97), independent of the FAIL_MODE harness
    // and cfg(test) gating. This pattern works from binary-crate tests where
    // the library's cfg(test) FAIL_MODE branch is NOT reachable (cfg(test)
    // is per-crate-not-per-build, RFC 1604).
    // ========================================================================

    /// Site 4 — `format_bip39_phrase` invokes `pin_pages_for` on the bip85-
    /// derived entropy buffer. After GREEN, the function body adds a
    /// `let _pin = pin_pages_for(&entropy[..])` immediately after the
    /// `derive_entropy(...)?` binding. R1 reviewer decides whether to
    /// expand coverage to the other 6 `format_*` functions; the slice-fn
    /// pin pattern is uniform across all 7.
    #[test]
    fn site_4_format_bip39_phrase_invokes_pin() {
        let baseline = mnemonic_toolkit::mlock::attempts_for_test();
        let _ = format_bip39_phrase(&master(), 0, bip39::Language::English, 12, 0);
        assert!(
            mnemonic_toolkit::mlock::attempts_for_test() > baseline,
            "format_bip39_phrase must invoke pin_pages_for on derived entropy; \
             attempts counter did not increment",
        );
    }

    /// Boundary: sides=2 (coin flip). bits_per_roll = 1; rolls are 0 or 1.
    #[test]
    fn dice_d2_rolls_in_range() {
        let rolls = format_dice_rolls(&master(), 2, 50, 0).unwrap();
        for r in rolls.split(',') {
            let v: u32 = r.parse().unwrap();
            assert!(v < 2, "d2 roll out of [0,1]: {v}");
        }
    }

    /// Boundary: sides=256 (no rejection sampling required). 1 byte per roll.
    #[test]
    fn dice_d256_rolls_in_range() {
        let rolls = format_dice_rolls(&master(), 256, 20, 0).unwrap();
        for r in rolls.split(',') {
            let v: u32 = r.parse().unwrap();
            assert!(v < 256, "d256 roll out of [0,255]: {v}");
        }
    }

    /// Refusal: sides < 2.
    #[test]
    fn dice_sides_too_small_refused() {
        let r = format_dice_rolls(&master(), 1, 5, 0);
        assert!(
            matches!(r, Err(ToolkitError::BadInput(ref m)) if m.contains("sides must be >= 2"))
        );
    }

    /// Cycle B Phase 1 lock — `derive_entropy` returns
    /// `Result<Zeroizing<Vec<u8>>, ToolkitError>` (heap-promoted from the prior
    /// `Zeroizing<[u8; 64]>` so Phase 3's `MlockedZeroizing<Vec<u8>>` wrapper
    /// can pin the buffer's pages — mlock requires heap-resident memory).
    /// Invariant: the inner Vec is always length 64.
    #[test]
    fn derive_entropy_returns_zeroizing_vec_of_64_bytes() {
        let e: Zeroizing<Vec<u8>> = derive_entropy(&master(), 39, &[0, 12], 0).unwrap();
        assert_eq!(e.len(), 64);
    }

    /// Cycle B Phase 1 byte-determinism guard — the return-type heap-promotion
    /// must not perturb the derivation bytes. SPEC §6 G7 wire-format
    /// invariant; mirrors `feedback_spike_before_locking_wire_format`.
    #[test]
    fn derive_entropy_is_byte_deterministic() {
        let a = derive_entropy(&master(), 39, &[0, 12], 0).unwrap();
        let b = derive_entropy(&master(), 39, &[0, 12], 0).unwrap();
        assert_eq!(&a[..], &b[..]);
    }

    /// Cycle-15t internal-scratch evidence — the three pre-return derived-secret
    /// scratch locals in the encode/dice fns are `Zeroizing`-wrapped: the
    /// `format_password_base64`/`base85` full `encoded` String (only `[..length]`
    /// is wrapped into the `SecretString` return; the full encode lingers) and the
    /// `format_dice_rolls` per-roll `out: Vec<String>` aggregate (the dice secret).
    /// The anchors are assembled at runtime via `format!` so this test's own
    /// source does NOT self-match the `src.contains(...)` checks (Lane T's T8
    /// precedent in `seedqr.rs`).
    #[test]
    fn internal_encode_dice_scratch_is_zeroizing() {
        let src = std::fs::read_to_string("src/bip85.rs").expect("read src/bip85.rs");
        let zn = format!("{}::new", "Zeroizing");
        // base64 encode scratch.
        assert!(
            src.contains(&format!("let encoded = {zn}(base64_standard(")),
            "encode scratch: base64 `encoded` must be Zeroizing::new(base64_standard(...))"
        );
        // base85 encode scratch.
        assert!(
            src.contains(&format!("let encoded = {zn}(base85_btc(")),
            "encode scratch: base85 `encoded` must be Zeroizing::new(base85_btc(...))"
        );
        // dice per-roll aggregate.
        let zv = format!("{}<Vec<String>>", "Zeroizing");
        assert!(
            src.contains(&format!("out: {zv}")),
            "dice scratch: `out` aggregate must be Zeroizing<Vec<String>>"
        );
    }
}
