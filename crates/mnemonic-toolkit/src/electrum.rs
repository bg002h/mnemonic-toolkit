//! Electrum native seed format (SPEC §14).
//!
//! HMAC-SHA512 prefix dispatch + per-wordlist base-N mapping.
//! v0.7 supported English only; v0.8 §14 adds 4 non-English wordlists
//! (chinese_simplified, japanese, portuguese, spanish) via
//! `crate::wordlists::ElectrumWordlist`. Portuguese has 1626 words (not
//! 2048); base-N arithmetic is parameterized on `wordlist.base()`.
//!
//! Reference impl: `electrum/electrum/mnemonic.py` at commit
//! `e1099925e30d91dd033815b512f00582a8795d25`.

use crate::wordlists::{normalize_electrum, ElectrumWordlist};
use bitcoin::hashes::{sha512, Hash, HashEngine, Hmac, HmacEngine};
use hmac::Hmac as Pbkdf2Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha512;
use unicode_normalization::char::canonical_combining_class;
use unicode_normalization::UnicodeNormalization;
use zeroize::Zeroizing;

/// SPEC v0.8 §14 — encode iteration bound. Electrum's `make_seed` increments
/// the entropy integer until `validate_seed_version` matches the requested
/// version; with HMAC-SHA512 random behavior the per-iteration probability
/// of matching is ~1/256 (Standard, 8-bit prefix `01`) or ~1/4096 (Segwit,
/// 12-bit prefix `100`). 2^20 iterations is a generous cap that should never
/// fire under normal use; if it does, surface a refusal rather than spinning.
pub(crate) const MAX_ENCODE_ITERATIONS: u64 = 1 << 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SeedVersion {
    Standard,    // hex prefix "01"
    Segwit,      // hex prefix "100"
    Standard2FA, // hex prefix "101" — REFUSED at convert layer
    Segwit2FA,   // hex prefix "102" — REFUSED at convert layer
}

impl SeedVersion {
    /// Numeric label per Electrum's `version.py` (`01` / `100` / `101` / `102`).
    /// Used by SPEC v0.8 §14 stderr info-line.
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
    /// SPEC v0.8 §14 — encode iteration bound exceeded.
    EncodeIterationBoundExceeded,
}

/// Electrum's `mnemonic.py::normalize_text` EXACTLY (e1099925:80-91), used for
/// the PBKDF2 seed + passphrase: NFKD → lower → drop chars with non-zero
/// canonical combining class → collapse whitespace → strip CJK-internal
/// whitespace.
///
/// Two subtleties that the `UNICODE_HORROR` test vector pins (and that
/// `normalize_electrum` gets wrong for arbitrary input — it is fine for valid
/// wordlist words, so this is a SEPARATE function, NOT a reuse):
/// 1. Order is lower-BEFORE-strip (vs `normalize_electrum`'s strip-before-lower).
/// 2. Electrum's `unicodedata.combining(c) != 0` is the **canonical combining
///    class**, NOT the Mark general category: marks like U+034F (CGJ) and
///    U+0489 have ccc=0, so Python KEEPS them — `is_combining_mark` (category
///    Mark) would wrongly strip them. We use `canonical_combining_class(c) != 0`
///    to match Python byte-for-byte.
// cycle-15t — the normalized phrase/passphrase are secret copies alive across
// PBKDF2; return `Zeroizing<String>` so they scrub on drop. Private helper,
// consumed-by-move into `norm_phrase`/`norm_pp`, never `{:?}`-printed → bare
// `Zeroizing` (not `SecretString`) is the correct choice (rule Z-DEBUG).
fn normalize_text_electrum(s: &str) -> Zeroizing<String> {
    let nfkd: Zeroizing<String> = Zeroizing::new(s.nfkd().collect());
    let lowered: Zeroizing<String> =
        Zeroizing::new(nfkd.chars().flat_map(|c| c.to_lowercase()).collect());
    let stripped: Zeroizing<String> = Zeroizing::new(
        lowered
            .chars()
            .filter(|c| canonical_combining_class(*c) == 0)
            .collect(),
    );
    let collapsed = stripped.split_whitespace().collect::<Vec<_>>().join(" ");
    Zeroizing::new(strip_cjk_internal_whitespace(&collapsed))
}

/// Electrum native-seed → BIP-32 seed bytes (`mnemonic.py::mnemonic_to_seed`
/// @ e1099925): `PBKDF2-HMAC-SHA512(normalize_text(phrase), b"electrum" +
/// normalize_text(passphrase), 2048)` → 64 bytes. `normalize_phrase_for_hmac`
/// IS Electrum's `normalize_text` (NFKD → strip-combining → lower → collapse
/// whitespace → strip CJK-internal whitespace; the SAME normalization applies
/// to both seed and passphrase). Returned `Zeroizing` — the 64-byte seed is
/// master-secret-equivalent. FOLLOWUP `electrum-native-seed-address-derivation`.
pub(crate) fn electrum_seed_to_bip32_seed(phrase: &str, passphrase: &str) -> Zeroizing<[u8; 64]> {
    let norm_phrase = normalize_text_electrum(phrase);
    let norm_pp = normalize_text_electrum(passphrase);
    let mut salt = Vec::with_capacity(b"electrum".len() + norm_pp.len());
    salt.extend_from_slice(b"electrum");
    salt.extend_from_slice(norm_pp.as_bytes());
    let mut out = Zeroizing::new([0u8; 64]);
    pbkdf2::<Pbkdf2Hmac<Sha512>>(norm_phrase.as_bytes(), &salt, 2048, out.as_mut_slice())
        .expect("pbkdf2 fill must succeed (dkLen 64 + 2048 iters in supported range)");
    out
}

/// HMAC-SHA512(key=`"Seed version"`, msg=phrase) hex-prefix dispatch
/// (Electrum `mnemonic.py::is_new_seed`). Wordlist-agnostic: the HMAC
/// is over the normalized phrase bytes, not over wordlist indices. Uses
/// the FULL Electrum `normalize_text` (including CJK-internal-whitespace
/// stripping), matching upstream byte-for-byte.
pub(crate) fn validate_seed_version(phrase: &str) -> Result<SeedVersion, ElectrumError> {
    let normalized = normalize_phrase_for_hmac(phrase);
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
/// Algorithm: pop words right-to-left, accumulating `i = i*base + index(w)`,
/// then serialize `i` as big-endian bytes.
pub(crate) fn phrase_to_entropy(
    phrase: &str,
    wordlist: ElectrumWordlist,
) -> Result<Zeroizing<Vec<u8>>, ElectrumError> {
    // Per-word normalization (NFKD + lowercase + strip combining); explicitly
    // NOT collapsing CJK-internal whitespace, since Electrum's `mnemonic_decode`
    // splits on whitespace BEFORE looking up each word in the wordlist. Stripping
    // CJK-internal whitespace would collapse `眼 悲 叛` into a single super-word
    // `眼悲叛` that no wordlist contains.
    // SPEC v0.9.0 §1 item 2 (cycle-15t) — wrap each normalized secret word in
    // Zeroizing at the consumption boundary so it scrubs on drop. We do NOT
    // widen `wordlists::normalize_electrum` (M-4: cross-module helper with
    // other callers); the wrap lives only here.
    let words: Vec<Zeroizing<String>> = phrase
        .split_whitespace()
        .map(|w| Zeroizing::new(normalize_electrum(w)))
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() {
        return Err(ElectrumError::Empty);
    }
    let wl = wordlist.words();
    let base = wordlist.base();
    // SPEC v0.9.0 §1 item 2 — `acc` holds the secret integer during
    // base-N decode; wrap so it scrubs on drop. cycle-15t: return the
    // `Zeroizing<Vec<u8>>` BY MOVE instead of deref-cloning the inner `Vec`
    // out — the prior return cloned the secret entropy into a bare
    // un-scrubbed heap `Vec` that outlived `acc`'s scrub.
    let mut acc: zeroize::Zeroizing<Vec<u8>> = zeroize::Zeroizing::new(vec![0]);
    for w in words.iter().rev() {
        let idx = wl
            .iter()
            .position(|x| x.as_str() == w.as_str())
            .ok_or_else(|| ElectrumError::UnknownWord((**w).clone()))?;
        // acc = acc * base + idx (little-endian byte arithmetic).
        mul_add_le(&mut acc, base, idx as u32);
    }
    // Strip leading zeros (high-order); reverse to big-endian.
    while acc.len() > 1 && *acc.last().unwrap() == 0 {
        acc.pop();
    }
    acc.reverse();
    Ok(acc)
}

/// Encode entropy → phrase at `version` and `wordlist`. Increments the
/// integer until `validate_seed_version` matches the requested version,
/// per Electrum's `mnemonic.py::Mnemonic::make_seed` algorithm. Bounded
/// at `MAX_ENCODE_ITERATIONS` (SPEC v0.8 §14) to refuse pathological loops.
pub(crate) fn entropy_to_phrase(
    entropy: &[u8],
    version: SeedVersion,
    wordlist: ElectrumWordlist,
) -> Result<String, ElectrumError> {
    if entropy.is_empty() {
        return Err(ElectrumError::Empty);
    }
    if version.is_2fa() {
        // Caller (cmd/convert) gates this; defensive double-check.
        return Err(ElectrumError::InvalidVersion);
    }
    let wl = wordlist.words();
    let base = wordlist.base();
    // SPEC v0.9.0 §1 item 2 — `acc` is the secret integer + per-step
    // base-N digits; wrap in Zeroizing so it scrubs on drop.
    let mut acc: zeroize::Zeroizing<Vec<u8>> =
        zeroize::Zeroizing::new(entropy.iter().rev().copied().collect());
    let mut iterations: u64 = 0;
    loop {
        if iterations >= MAX_ENCODE_ITERATIONS {
            return Err(ElectrumError::EncodeIterationBoundExceeded);
        }
        // Render acc as a phrase via base-N division.
        let mut buf = acc.clone();
        let mut words: Vec<&str> = Vec::new();
        loop {
            let rem = div_assign_le(&mut buf, base);
            words.push(wl[rem as usize].as_str());
            if buf.iter().all(|&b| b == 0) {
                break;
            }
        }
        // cycle-15t — each rejected candidate's secret phrase scrubs on drop
        // (Zeroizing scratch). I-1: the return STAYS `String` (do NOT widen —
        // `compute_outputs` type-unifies it against bare-`String` match arms);
        // the single matched candidate moves its value out at the `Ok`.
        let phrase = Zeroizing::new(words.join(" "));
        if let Ok(v) = validate_seed_version(&phrase) {
            if v == version {
                return Ok((*phrase).clone());
            }
        }
        add_one_le(&mut acc);
        iterations += 1;
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

/// SPEC v0.8 §14 — full Electrum `normalize_text` for HMAC dispatch:
/// NFKD + lowercase + strip combining marks + collapse whitespace +
/// strip CJK-internal whitespace. Used ONLY for HMAC seed-version
/// dispatch, where Electrum hashes the fully-normalized phrase. The
/// wordlist-lookup path uses per-word normalization without CJK
/// whitespace stripping (see `phrase_to_entropy`).
// cycle-15t — HMAC-dispatch normalize intermediates are secret phrase copies;
// return `Zeroizing<String>` (private helper, consumed-by-move, never
// `{:?}`-printed). `stage1` wraps at the `normalize_electrum` consumption
// boundary (M-4: the cross-module helper itself stays `-> String`).
fn normalize_phrase_for_hmac(s: &str) -> Zeroizing<String> {
    let stage1 = Zeroizing::new(normalize_electrum(s));
    let collapsed: Zeroizing<String> =
        Zeroizing::new(stage1.split_whitespace().collect::<Vec<_>>().join(" "));
    Zeroizing::new(strip_cjk_internal_whitespace(&collapsed))
}

fn strip_cjk_internal_whitespace(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    for i in 0..chars.len() {
        if chars[i].is_whitespace()
            && i > 0
            && i + 1 < chars.len()
            && is_cjk(chars[i - 1])
            && is_cjk(chars[i + 1])
        {
            continue;
        }
        out.push(chars[i]);
    }
    out
}

fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x4E00..=0x9FFF   // CJK Unified Ideographs
        | 0x3400..=0x4DBF // CJK Unified Ideographs Extension A
        | 0x20000..=0x2A6DF // Extension B
        | 0x2A700..=0x2B73F // Extension C
        | 0x2B740..=0x2B81F // Extension D
        | 0xF900..=0xFAFF // CJK Compatibility Ideographs
        | 0x3040..=0x309F // Hiragana
        | 0x30A0..=0x30FF // Katakana
        | 0xAC00..=0xD7AF // Hangul Syllables
    )
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

    // English vectors verified in `design/agent-reports/v0_7-phase-3-electrum-corpus-spike.md`.
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
        assert_eq!(
            validate_seed_version(STANDARD).unwrap(),
            SeedVersion::Standard
        );
        assert_eq!(validate_seed_version(SEGWIT).unwrap(), SeedVersion::Segwit);
        assert_eq!(
            validate_seed_version(STANDARD_2FA).unwrap(),
            SeedVersion::Standard2FA
        );
        assert_eq!(
            validate_seed_version(SEGWIT_2FA).unwrap(),
            SeedVersion::Segwit2FA
        );
    }

    #[test]
    fn invalid_phrase_unknown_word() {
        let bogus = "notaword notaword notaword notaword notaword notaword notaword notaword notaword notaword notaword notaword";
        assert!(validate_seed_version(bogus).is_err());
    }

    #[test]
    fn decode_standard_hex() {
        let bytes = phrase_to_entropy(STANDARD, ElectrumWordlist::English).unwrap();
        assert_eq!(hex::encode(&bytes), STANDARD_HEX);
    }

    #[test]
    fn decode_segwit_hex() {
        let bytes = phrase_to_entropy(SEGWIT, ElectrumWordlist::English).unwrap();
        assert_eq!(hex::encode(&bytes), SEGWIT_HEX);
    }

    #[test]
    fn round_trip_standard() {
        let bytes = phrase_to_entropy(STANDARD, ElectrumWordlist::English).unwrap();
        let phrase =
            entropy_to_phrase(&bytes, SeedVersion::Standard, ElectrumWordlist::English).unwrap();
        assert_eq!(phrase, STANDARD);
    }

    #[test]
    fn round_trip_segwit() {
        let bytes = phrase_to_entropy(SEGWIT, ElectrumWordlist::English).unwrap();
        let phrase =
            entropy_to_phrase(&bytes, SeedVersion::Segwit, ElectrumWordlist::English).unwrap();
        assert_eq!(phrase, SEGWIT);
    }

    #[test]
    fn encode_with_increment_search() {
        let p =
            entropy_to_phrase(&[0x01], SeedVersion::Standard, ElectrumWordlist::English).unwrap();
        assert_eq!(validate_seed_version(&p).unwrap(), SeedVersion::Standard);
        let p = entropy_to_phrase(&[0x01], SeedVersion::Segwit, ElectrumWordlist::English).unwrap();
        assert_eq!(validate_seed_version(&p).unwrap(), SeedVersion::Segwit);
    }

    #[test]
    fn refuse_2fa_encode() {
        assert!(matches!(
            entropy_to_phrase(&[0x01], SeedVersion::Standard2FA, ElectrumWordlist::English),
            Err(ElectrumError::InvalidVersion)
        ));
        assert!(matches!(
            entropy_to_phrase(&[0x01], SeedVersion::Segwit2FA, ElectrumWordlist::English),
            Err(ElectrumError::InvalidVersion)
        ));
    }

    // ============================================================================
    // Electrum tests/test_mnemonic.py — non-English vectors (commit pinned in
    // src/wordlists/mod.rs).
    // ============================================================================

    /// Spanish (lang='es') vector with combining-acute diacriticals (NFD form).
    /// Expected entropy from Electrum tests/test_mnemonic.py SEED_TEST_CASES['spanish']
    /// integer 3423992296655289706780599506247192518735 = 17 bytes (132-bit entropy
    /// exceeds u128, so compared in hex).
    #[test]
    fn decode_spanish_vector() {
        let phrase =
            "almíbar tibio superar vencer hacha peatón príncipe matar consejo polen vehículo odisea";
        let bytes = phrase_to_entropy(phrase, ElectrumWordlist::Spanish).unwrap();
        assert_eq!(hex::encode(&bytes), "0a0fecede9bf8a975eb6b4ef75bb79a04f");
    }

    /// Japanese (lang='ja') vector. Entropy 1938439226660562861250521787963972783469.
    #[test]
    fn decode_japanese_vector() {
        let phrase =
            "なのか ひろい しなん まなぶ つぶす さがす おしゃれ かわく おいかける けさき かいとう さたん";
        let bytes = phrase_to_entropy(phrase, ElectrumWordlist::Japanese).unwrap();
        assert_eq!(hex::encode(&bytes), "05b251d0b0f32da46966cd6e16ca740d6d");
    }

    /// Chinese Simplified (lang='zh') vector. Entropy 3083737086352778425940060465574397809099.
    /// Note: Electrum strips whitespace BETWEEN CJK characters during normalize.
    #[test]
    fn decode_chinese_simplified_vector() {
        let phrase = "眼 悲 叛 改 节 跃 衡 响 疆 股 遂 冬";
        let bytes = phrase_to_entropy(phrase, ElectrumWordlist::ChineseSimplified).unwrap();
        assert_eq!(hex::encode(&bytes), "090ff228d676340e9ad295e25d9fef11cb");
    }

    /// Spanish encode round-trip: entropy → phrase → entropy.
    #[test]
    fn round_trip_spanish() {
        let phrase =
            "almíbar tibio superar vencer hacha peatón príncipe matar consejo polen vehículo odisea";
        let bytes = phrase_to_entropy(phrase, ElectrumWordlist::Spanish).unwrap();
        let re_phrase =
            entropy_to_phrase(&bytes, SeedVersion::Standard, ElectrumWordlist::Spanish).unwrap();
        // Re-encoded phrase normalizes to the same input phrase.
        let re_bytes = phrase_to_entropy(&re_phrase, ElectrumWordlist::Spanish).unwrap();
        assert_eq!(re_bytes, bytes);
    }

    #[test]
    fn round_trip_portuguese_base_1626() {
        // Portuguese is base-1626 (not 2048). Pick a small entropy and verify
        // the round-trip exercises the non-2048 base path correctly.
        let bytes = vec![0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let phrase =
            entropy_to_phrase(&bytes, SeedVersion::Standard, ElectrumWordlist::Portuguese).unwrap();
        let re_bytes = phrase_to_entropy(&phrase, ElectrumWordlist::Portuguese).unwrap();
        // Increment search may change the leading bytes; round-trip recovers
        // the encoded entropy (post-increment), not the input.
        let re_phrase = entropy_to_phrase(
            &re_bytes,
            SeedVersion::Standard,
            ElectrumWordlist::Portuguese,
        )
        .unwrap();
        assert_eq!(re_phrase, phrase);
    }

    /// Cross-language collision negative test — a Spanish phrase must NOT
    /// decode against the Japanese wordlist (or vice versa).
    #[test]
    fn cross_language_decode_rejected() {
        let spanish_phrase =
            "almíbar tibio superar vencer hacha peatón príncipe matar consejo polen vehículo odisea";
        let r = phrase_to_entropy(spanish_phrase, ElectrumWordlist::Japanese);
        assert!(matches!(r, Err(ElectrumError::UnknownWord(_))));
    }

    // ========================================================================
    // cycle-15t Slug-2 — electrum derived-secret zeroize.
    // ========================================================================

    /// T4 — `phrase_to_entropy` returns `Zeroizing<Vec<u8>>` (RED until the
    /// deref-clone-out-of-acc move-fix flips the return type). A bare-`Vec<u8>`
    /// return makes this fn-pointer coercion fail to compile.
    #[test]
    #[allow(clippy::type_complexity)] // the fn-pointer type IS the fence
    fn t4_phrase_to_entropy_returns_zeroizing() {
        let _f: fn(&str, ElectrumWordlist) -> Result<zeroize::Zeroizing<Vec<u8>>, ElectrumError> =
            phrase_to_entropy;
    }

    /// T4-GUARD (stays GREEN) — `entropy_to_phrase` STILL returns
    /// `Result<String, _>` (I-1: the public return is NOT widened, to keep
    /// `compute_outputs`' match-arm unification intact).
    #[test]
    fn t4_guard_entropy_to_phrase_stays_string() {
        let _f: fn(&[u8], SeedVersion, ElectrumWordlist) -> Result<String, ElectrumError> =
            entropy_to_phrase;
    }

    /// T4-GUARD (M-4, stays GREEN) — `wordlists::normalize_electrum` is NOT
    /// widened (cross-module helper with other callers): a source-grep proves
    /// its `-> String` decl is unchanged; the `Zeroizing` wrap lives only at
    /// the electrum-side call sites.
    #[test]
    fn t4_guard_wordlists_normalize_electrum_unwidened() {
        let src =
            std::fs::read_to_string("src/wordlists/mod.rs").expect("read src/wordlists/mod.rs");
        assert!(
            src.contains("pub(crate) fn normalize_electrum(s: &str) -> String"),
            "M-4: wordlists::normalize_electrum must keep `-> String` (cross-module helper)"
        );
    }

    /// T5 — no-clone-out-of-Zeroizing: `phrase_to_entropy` returns by MOVE,
    /// never clones the secret entropy out of the `Zeroizing` wrapper into a
    /// bare un-scrubbed `Vec`. The forbidden-literal anchor is assembled at
    /// runtime so this test's own source does NOT self-match.
    #[test]
    fn t5_phrase_to_entropy_no_clone_out_of_zeroizing() {
        let src = std::fs::read_to_string("src/electrum.rs").expect("read src/electrum.rs");
        // Assemble the forbidden literal at runtime (avoids self-match in this
        // test's source). The pattern is the deref-clone-out-of-acc return.
        let forbidden = format!("Ok((*{}).clone())", "acc");
        assert!(
            !src.contains(&forbidden),
            "T5: electrum must NOT clone secret entropy out of the Zeroizing wrapper"
        );
        assert!(
            src.contains("    Ok(acc)\n"),
            "T5: phrase_to_entropy must return the Zeroizing Vec by move"
        );
    }

    /// T-norm-scrub evidence — the normalize intermediates wrap in Zeroizing
    /// at the electrum consumption boundary (norm_phrase/norm_pp plus the
    /// per-word and per-candidate scratch). Anchors are wrap-survivable single
    /// tokens (R0 plan-doc m-2): the toolkit is never cargo-fmt'd.
    #[test]
    fn t_electrum_normalize_intermediates_are_zeroizing() {
        let src = std::fs::read_to_string("src/electrum.rs").expect("read src/electrum.rs");
        // electrum-LOCAL helper return widened to Zeroizing<String>.
        assert!(
            src.contains("fn normalize_text_electrum(s: &str) -> Zeroizing<String>"),
            "normalize_text_electrum must return Zeroizing<String>"
        );
        // per-word + per-candidate consumption-boundary wraps.
        assert!(
            src.contains("Zeroizing::new(normalize_electrum"),
            "per-word normalize result must wrap in Zeroizing at the call site"
        );
        assert!(
            src.contains("fn normalize_phrase_for_hmac(s: &str) -> Zeroizing<String>"),
            "normalize_phrase_for_hmac must return Zeroizing<String>"
        );
    }
}
