//! Electrum native seed format (SPEC §14).
//!
//! HMAC-SHA512 prefix dispatch + base-2048 wordlist mapping.
//! Wordlist is byte-identical to BIP-39 English (Electrum reuses BIP-39's
//! words but applies its own validation rule); reused via `bip39::Language`.
//! Source: `electrum/wordlist/english.txt` SHA-256
//! `2f5eed53a4727b4bf8880d8f3f199efc90e58503646d9ff8eff3a2ed3b24dbda`
//! (retrieved 2026-05-06; see `design/agent-reports/v0_7-phase-3-electrum-corpus-spike.md`).

use bitcoin::hashes::{sha512, Hash, HashEngine, Hmac, HmacEngine};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SeedVersion {
    Standard,    // hex prefix "01"
    Segwit,      // hex prefix "100"
    Standard2FA, // hex prefix "101" — REFUSED at convert layer
    Segwit2FA,   // hex prefix "102" — REFUSED at convert layer
}

impl SeedVersion {
    /// Numeric label per Electrum's `version.py` (`01` / `100` / `101` / `102`).
    /// Reserved for future user-facing diagnostic surfacing.
    #[allow(dead_code)]
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Standard => "01",
            Self::Segwit => "100",
            Self::Standard2FA => "101",
            Self::Segwit2FA => "102",
        }
    }

    pub(crate) fn is_2fa(self) -> bool {
        matches!(self, Self::Standard2FA | Self::Segwit2FA)
    }
}

#[derive(Debug)]
pub(crate) enum ElectrumError {
    Empty,
    /// Wordlist miss — the inner string is the offending word; reserved for
    /// diagnostic surfacing (currently collapsed to a single refusal text).
    UnknownWord(#[allow(dead_code)] String),
    InvalidVersion,
}

/// HMAC-SHA512(key=`"Seed version"`, msg=phrase) hex-prefix dispatch
/// (Electrum `mnemonic.py::is_new_seed`).
pub(crate) fn validate_seed_version(phrase: &str) -> Result<SeedVersion, ElectrumError> {
    let normalized = normalize_phrase(phrase);
    if normalized.is_empty() {
        return Err(ElectrumError::Empty);
    }
    let hex = hmac_sha512_hex(b"Seed version", normalized.as_bytes());
    // Order matters: `101`/`102` start with `10`, so `100` must be checked
    // after them. `01` is unambiguous.
    if hex.starts_with("101") {
        Ok(SeedVersion::Standard2FA)
    } else if hex.starts_with("102") {
        Ok(SeedVersion::Segwit2FA)
    } else if hex.starts_with("100") {
        Ok(SeedVersion::Segwit)
    } else if hex.starts_with("01") {
        Ok(SeedVersion::Standard)
    } else {
        Err(ElectrumError::InvalidVersion)
    }
}

/// Decode words → entropy bytes (Electrum `mnemonic.py::mnemonic_decode`).
/// Algorithm: pop words right-to-left, accumulating `i = i*2048 + index(w)`,
/// then serialize `i` as big-endian bytes.
pub(crate) fn phrase_to_entropy(phrase: &str) -> Result<Vec<u8>, ElectrumError> {
    let normalized = normalize_phrase(phrase);
    if normalized.is_empty() {
        return Err(ElectrumError::Empty);
    }
    let words: Vec<&str> = normalized.split_whitespace().collect();
    let wl = bip39::Language::English.word_list();
    // Build big-endian byte representation of `i` directly via base-2048 multiply-add.
    let mut acc: Vec<u8> = vec![0]; // little-endian internally; reversed at end.
    for w in words.iter().rev() {
        let idx = wl
            .iter()
            .position(|x| *x == *w)
            .ok_or_else(|| ElectrumError::UnknownWord((*w).to_string()))?;
        // acc = acc * 2048 + idx (little-endian byte arithmetic).
        mul_add_le(&mut acc, 2048, idx as u32);
    }
    // Strip leading zeros (high-order); reverse to big-endian.
    while acc.len() > 1 && *acc.last().unwrap() == 0 {
        acc.pop();
    }
    acc.reverse();
    Ok(acc)
}

/// Encode entropy → phrase at `version`. Increments the integer until
/// `validate_seed_version` matches the requested version, per Electrum's
/// `mnemonic.py::Mnemonic::make_seed` algorithm.
pub(crate) fn entropy_to_phrase(
    entropy: &[u8],
    version: SeedVersion,
) -> Result<String, ElectrumError> {
    if entropy.is_empty() {
        return Err(ElectrumError::Empty);
    }
    if version.is_2fa() {
        // Caller (cmd/convert) gates this; defensive double-check.
        return Err(ElectrumError::InvalidVersion);
    }
    // Internal little-endian buffer.
    let mut acc: Vec<u8> = entropy.iter().rev().copied().collect();
    let wl = bip39::Language::English.word_list();
    loop {
        // Render acc as a phrase via base-2048 division.
        let mut buf = acc.clone();
        let mut words: Vec<&'static str> = Vec::new();
        // Loop at least once so a zero entropy still emits a single word.
        loop {
            let rem = div_assign_le(&mut buf, 2048);
            words.push(wl[rem as usize]);
            if buf.iter().all(|&b| b == 0) {
                break;
            }
        }
        let phrase = words.join(" ");
        // Loop on InvalidVersion (no prefix match yet); only the requested
        // version is a stop condition.
        if let Ok(v) = validate_seed_version(&phrase) {
            if v == version {
                return Ok(phrase);
            }
        }
        // Increment acc by 1 (little-endian).
        add_one_le(&mut acc);
    }
}

// ============================================================================
// internals
// ============================================================================

fn hmac_sha512_hex(key: &[u8], msg: &[u8]) -> String {
    let mut engine = HmacEngine::<sha512::Hash>::new(key);
    engine.input(msg);
    let mac = Hmac::<sha512::Hash>::from_engine(engine);
    hex::encode(mac.as_byte_array())
}

/// Subset of Electrum `mnemonic.py::normalize_text`: lowercase + collapse
/// whitespace. v0.7 uses the BIP-39 English wordlist (no diacritics in
/// the words); full NFKD + diacritic-stripping for non-Latin Electrum
/// wordlists is tracked as v0.8 FOLLOWUP `electrum-non-latin-wordlists`.
fn normalize_phrase(s: &str) -> String {
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Little-endian: `acc = acc * mul + add`.
fn mul_add_le(acc: &mut Vec<u8>, mul: u32, add: u32) {
    let mut carry: u64 = add as u64;
    for byte in acc.iter_mut() {
        let v = (*byte as u64) * (mul as u64) + carry;
        *byte = (v & 0xff) as u8;
        carry = v >> 8;
    }
    while carry > 0 {
        acc.push((carry & 0xff) as u8);
        carry >>= 8;
    }
}

/// Little-endian: `acc /= div`, returning remainder.
fn div_assign_le(acc: &mut [u8], div: u32) -> u32 {
    let mut rem: u64 = 0;
    for byte in acc.iter_mut().rev() {
        let v = (rem << 8) | (*byte as u64);
        *byte = (v / div as u64) as u8;
        rem = v % div as u64;
    }
    rem as u32
}

/// Little-endian increment by 1.
fn add_one_le(acc: &mut Vec<u8>) {
    for byte in acc.iter_mut() {
        let (v, carry) = byte.overflowing_add(1);
        *byte = v;
        if !carry {
            return;
        }
    }
    acc.push(1);
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Vectors verified in `design/agent-reports/v0_7-phase-3-electrum-corpus-spike.md`.
    const STANDARD: &str =
        "cram swing cover prefer miss modify ritual silly deliver chunk behind inform able";
    const SEGWIT: &str =
        "wild father tree among universe such mobile favorite target dynamic credit identify";
    const STANDARD_2FA: &str =
        "science dawn member doll dutch real can brick knife deny drive list";
    const SEGWIT_2FA: &str =
        "universe topic remind silver february ranch shine worth innocent cattle enhance wise";

    const STANDARD_HEX: &str = "2738290a29d0c8b7523ac6ea9c63370191";
    const SEGWIT_HEX: &str = "0708661136ef5411cf61f6e07fcfd4efd8";

    #[test]
    fn validate_all_four_versions() {
        assert_eq!(validate_seed_version(STANDARD).unwrap(), SeedVersion::Standard);
        assert_eq!(validate_seed_version(SEGWIT).unwrap(), SeedVersion::Segwit);
        assert_eq!(validate_seed_version(STANDARD_2FA).unwrap(), SeedVersion::Standard2FA);
        assert_eq!(validate_seed_version(SEGWIT_2FA).unwrap(), SeedVersion::Segwit2FA);
    }

    #[test]
    fn invalid_phrase_unknown_word() {
        // BIP-39 word "abandon" is in the wordlist; but a random non-word is not.
        let bogus = "notaword notaword notaword notaword notaword notaword notaword notaword notaword notaword notaword notaword";
        // First fails HMAC dispatch (random text won't have one of the 4 prefixes).
        assert!(validate_seed_version(bogus).is_err());
    }

    #[test]
    fn decode_standard_hex() {
        let bytes = phrase_to_entropy(STANDARD).unwrap();
        assert_eq!(hex::encode(&bytes), STANDARD_HEX);
    }

    #[test]
    fn decode_segwit_hex() {
        let bytes = phrase_to_entropy(SEGWIT).unwrap();
        assert_eq!(hex::encode(&bytes), SEGWIT_HEX);
    }

    #[test]
    fn round_trip_standard() {
        let bytes = phrase_to_entropy(STANDARD).unwrap();
        let phrase = entropy_to_phrase(&bytes, SeedVersion::Standard).unwrap();
        assert_eq!(phrase, STANDARD);
    }

    #[test]
    fn round_trip_segwit() {
        let bytes = phrase_to_entropy(SEGWIT).unwrap();
        let phrase = entropy_to_phrase(&bytes, SeedVersion::Segwit).unwrap();
        assert_eq!(phrase, SEGWIT);
    }

    #[test]
    fn encode_with_increment_search() {
        // entropy `0x01` is unlikely to map to either Standard or Segwit on the
        // first try; entropy_to_phrase should increment until it does.
        let p = entropy_to_phrase(&[0x01], SeedVersion::Standard).unwrap();
        assert_eq!(validate_seed_version(&p).unwrap(), SeedVersion::Standard);
        let p = entropy_to_phrase(&[0x01], SeedVersion::Segwit).unwrap();
        assert_eq!(validate_seed_version(&p).unwrap(), SeedVersion::Segwit);
    }

    #[test]
    fn refuse_2fa_encode() {
        assert!(matches!(
            entropy_to_phrase(&[0x01], SeedVersion::Standard2FA),
            Err(ElectrumError::InvalidVersion)
        ));
        assert!(matches!(
            entropy_to_phrase(&[0x01], SeedVersion::Segwit2FA),
            Err(ElectrumError::InvalidVersion)
        ));
    }
}
