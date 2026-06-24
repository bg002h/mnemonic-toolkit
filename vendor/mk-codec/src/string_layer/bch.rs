//! BCH primitives for the mk1 string layer: bech32 alphabet conversion and
//! syndrome-based error correction.
//!
//! Forked from `md-codec` v0.4.x (`crates/md-codec/src/encoding.rs`) at the
//! start of the mk1 v0.1 implementation per `design/DECISIONS.md` D-13. The
//! BCH polynomials and field arithmetic are shared with the sibling md1
//! format (both reuse BIP 93's `BCH(93,80,8)` regular code and
//! `BCH(108,93,8)` long code); the only mk1-specific knobs are the HRP
//! (`"mk"`) and the NUMS-derived target residues ([`crate::consts::MK_REGULAR_CONST`]
//! / [`crate::consts::MK_LONG_CONST`]).
//!
//! Unlike md-codec's encoding module, this file does **not** expose a
//! top-level `encode_string` / `decode_string`: mk1's string-layer header
//! lives at the 5-bit symbol layer (per closure Q-5 — 2 symbols for
//! `SingleString`, 8 symbols for `Chunked`) rather than the byte-aligned
//! layer md1 uses. The mk1 `string_layer/mod.rs` builds string-level
//! encode/decode on top of the BCH primitives here.

use super::bch_decode;
use crate::consts::{HRP, MK_LONG_CONST, MK_REGULAR_CONST};

/// Which BCH code variant a string uses.
///
/// Determined by the total data-part length: regular for ≤93 chars,
/// long for 96–108 chars. Lengths 94–95 are reserved-invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BchCode {
    /// Regular code: BCH(93,80,8). 13-char checksum.
    Regular,
    /// Long code: BCH(108,93,8). 15-char checksum.
    Long,
}

/// The bech32 32-character alphabet, in 5-bit-value order.
///
/// `q=0, p=1, z=2, r=3, y=4, 9=5, x=6, 8=7, g=8, f=9, 2=10, t=11, v=12,
///  d=13, w=14, 0=15, s=16, 3=17, j=18, n=19, 5=20, 4=21, k=22, h=23,
///  c=24, e=25, 6=26, m=27, u=28, a=29, 7=30, l=31`.
pub const ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// Inverse lookup: char (lowercase ASCII) -> 5-bit value, or 0xFF if not in alphabet.
const ALPHABET_INV: [u8; 128] = build_alphabet_inv();

const fn build_alphabet_inv() -> [u8; 128] {
    let mut inv = [0xFFu8; 128];
    let mut i = 0;
    while i < 32 {
        inv[ALPHABET[i] as usize] = i as u8;
        i += 1;
    }
    inv
}

/// Convert a sequence of 8-bit bytes to a sequence of 5-bit values
/// (padded with zero bits at the end if the bit count is not a multiple of 5).
pub fn bytes_to_5bit(bytes: &[u8]) -> Vec<u8> {
    let mut acc: u32 = 0;
    let mut bits = 0u32;
    let mut out = Vec::with_capacity((bytes.len() * 8).div_ceil(5));
    for &b in bytes {
        acc = (acc << 8) | b as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(((acc >> bits) & 0x1F) as u8);
        }
    }
    if bits > 0 {
        out.push(((acc << (5 - bits)) & 0x1F) as u8);
    }
    out
}

/// Convert a sequence of 5-bit values back to 8-bit bytes.
///
/// Returns `None` if any value in `values` is ≥ 32 (out of 5-bit range),
/// or if the trailing padding bits are non-zero.
pub fn five_bit_to_bytes(values: &[u8]) -> Option<Vec<u8>> {
    let mut acc: u32 = 0;
    let mut bits = 0u32;
    let mut out = Vec::with_capacity(values.len() * 5 / 8);
    for &v in values {
        if v >= 32 {
            return None;
        }
        acc = (acc << 5) | v as u32;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((acc >> bits) & 0xFF) as u8);
        }
    }
    // Any remaining bits must be zero (padding).
    if bits >= 5 {
        return None;
    }
    if (acc & ((1 << bits) - 1)) != 0 {
        return None;
    }
    Some(out)
}

/// The bech32 separator character between HRP and data-part (BIP 173 §3).
///
/// Re-exported by [`crate::consts::HRP`] is `"mk"`; this module's
/// BCH-checksum helpers consume the HRP through their `hrp` parameter so
/// that the same primitives can verify any single-HRP codex32-derived
/// string. Production callers MUST pass [`crate::consts::HRP`].
pub const SEPARATOR: char = '1';

/// Determine the BchCode variant from a total data-part length.
///
/// Boundaries are from BIP 93 (codex32): regular code `BCH(93,80,8)` caps at 93,
/// long code `BCH(108,93,8)` runs 96–108, and lengths 94–95 are explicitly
/// reserved-invalid to prevent ambiguity in code-variant selection. Lengths
/// below 14 or above 108 are also rejected.
pub fn bch_code_for_length(data_part_len: usize) -> Option<BchCode> {
    match data_part_len {
        14..=93 => Some(BchCode::Regular),
        94..=95 => None,
        96..=108 => Some(BchCode::Long),
        _ => None,
    }
}

/// Check whether a string is all-lowercase, all-uppercase, or mixed.
///
/// Only ASCII letters are considered; non-ASCII characters (digits, punctuation,
/// Unicode letters) are treated as neither case. This is appropriate for MD
/// strings, whose alphabet is a subset of ASCII. An empty string or one with
/// no ASCII letters returns [`CaseStatus::Lower`].
pub fn case_check(s: &str) -> CaseStatus {
    let mut has_lower = false;
    let mut has_upper = false;
    for c in s.chars() {
        if c.is_ascii_lowercase() {
            has_lower = true;
        } else if c.is_ascii_uppercase() {
            has_upper = true;
        }
        if has_lower && has_upper {
            break;
        }
    }
    match (has_lower, has_upper) {
        (true, true) => CaseStatus::Mixed,
        (true, false) => CaseStatus::Lower,
        (false, true) => CaseStatus::Upper,
        (false, false) => CaseStatus::Lower, // empty / no letters; treat as lower
    }
}

/// Result of a case check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseStatus {
    /// All-lowercase or no letters.
    Lower,
    /// All-uppercase.
    Upper,
    /// Both lowercase and uppercase letters present (invalid).
    Mixed,
}

/// BCH polymod constants for the regular checksum (BCH(93,80,8)).
///
/// Source: BIP 93 (codex32) reference implementation, `ms32_polymod` function.
/// These five values are XORed into the running residue based on the top 5 bits
/// of the residue at each step. The polymod operation uses a 65-bit residue
/// (top 5 bits = current `b`, bottom 60 bits = masked state).
///
/// Verified against the canonical reference at
/// <https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki>.
pub const GEN_REGULAR: [u128; 5] = [
    0x19dc500ce73fde210,
    0x1bfae00def77fe529,
    0x1fbd920fffe7bee52,
    0x1739640bdeee3fdad,
    0x07729a039cfc75f5a,
];

/// Initial residue value for both the regular and long polymod algorithms (BIP 93).
///
/// Both `ms32_polymod` and `ms32_long_polymod` start with this residue before
/// processing any input characters.
pub const POLYMOD_INIT: u128 = 0x23181b3;

/// Right-shift amount to extract the top 5 bits from a 65-bit regular-code residue.
///
/// Usage: `b = residue >> REGULAR_SHIFT` gives the 5-bit feedback selector
/// for the polymod algorithm.
pub const REGULAR_SHIFT: u32 = 60;

/// Mask preserving the low 60 bits of a 65-bit regular-code residue.
pub const REGULAR_MASK: u128 = 0x0fffffffffffffff;

/// BCH polymod constants for the long checksum (BCH(108,93,8)).
///
/// Source: BIP 93 (codex32) reference implementation, `ms32_long_polymod` function.
/// The long polymod uses a 75-bit residue (top 5 bits = `b`, bottom 70 bits = masked state).
///
/// Verified against the canonical reference at
/// <https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki>.
pub const GEN_LONG: [u128; 5] = [
    0x3d59d273535ea62d897,
    0x7a9becb6361c6c51507,
    0x543f9b7e6c38d8a2a0e,
    0x0c577eaeccf1990d13c,
    0x1887f74f8dc71b10651,
];

/// Right-shift amount to extract the top 5 bits from a 75-bit long-code residue.
///
/// Usage: `b = residue >> LONG_SHIFT` gives the 5-bit feedback selector
/// for the polymod algorithm.
pub const LONG_SHIFT: u32 = 70;

/// Mask preserving the low 70 bits of a 75-bit long-code residue.
pub const LONG_MASK: u128 = 0x3fffffffffffffffff;

/// One step of the BCH polymod algorithm from BIP 93.
///
/// Updates the running `residue` to incorporate the next 5-bit input `value`
/// using the polynomial defined by `gen`, shift width `shift`, and mask `mask`.
/// The same function is used for both the regular and long codes; pass
/// `(GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK)` for the regular code and
/// `(GEN_LONG, LONG_SHIFT, LONG_MASK)` for the long code.
///
/// Returns the updated residue after incorporating `value`. The top 5 bits of
/// the returned residue feed the next iteration's `b` selector.
///
/// This is a direct port of BIP 93's `ms32_polymod` / `ms32_long_polymod` inner
/// loop. See <https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki> .
fn polymod_step(residue: u128, value: u128, r#gen: &[u128; 5], shift: u32, mask: u128) -> u128 {
    let b = residue >> shift;
    let mut new_residue = ((residue & mask) << 5) ^ value;
    for (i, &g) in r#gen.iter().enumerate() {
        if (b >> i) & 1 != 0 {
            new_residue ^= g;
        }
    }
    new_residue
}

/// BIP 173-style HRP-expansion: produces the 5-bit-symbol prelude that gets
/// prepended to the data part before running the BCH polymod.
///
/// For each HRP character `c`, emits `c >> 5` (high 3 bits); then emits a
/// single 0 separator; then emits each character's `c & 31` (low 5 bits).
/// The result has length `2 * hrp.len() + 1` for ASCII HRPs.
///
/// For `hrp_expand("md")` this returns `[3, 3, 0, 13, 4]`.
pub fn hrp_expand(hrp: &str) -> Vec<u8> {
    let bytes = hrp.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() * 2 + 1);
    for &c in bytes {
        out.push(c >> 5);
    }
    out.push(0);
    for &c in bytes {
        out.push(c & 31);
    }
    out
}

/// Run polymod over a sequence of 5-bit values using the parameters for
/// either the regular or long BCH code, starting from POLYMOD_INIT.
///
/// v0.3.1: promoted from `pub(in crate::string_layer)` to `pub` so
/// downstream consumers (toolkit `repair` feature) can compute polymod
/// residues against ms / md / mk target constants (all 3 share the
/// BIP-93 BCH(93,80,8) generator). Test-helper-drift concern remains
/// resolved by the sibling `bch_decode` module using THIS function
/// directly rather than re-implementing.
pub fn polymod_run(values: &[u8], r#gen: &[u128; 5], shift: u32, mask: u128) -> u128 {
    let mut residue = POLYMOD_INIT;
    for &v in values {
        residue = polymod_step(residue, v as u128, r#gen, shift, mask);
    }
    residue
}

/// Compute the 13-character BCH checksum for the regular code over the
/// HRP-expanded preamble plus the data part.
///
/// `data` is the sequence of 5-bit values for the data part (header + payload),
/// not including the checksum. Returns the 13-element checksum array, ready
/// to append to `data` to form the full data-part-plus-checksum.
///
/// The algorithm runs polymod over `hrp_expand(hrp) || data || [0; 13]`,
/// then XORs the result with [`MK_REGULAR_CONST`] to extract the checksum.
pub fn bch_create_checksum_regular(hrp: &str, data: &[u8]) -> [u8; 13] {
    // Regular code: 13-symbol checksum (0..=12), pad/array/extraction all use 13.
    let mut input = hrp_expand(hrp);
    input.extend_from_slice(data);
    input.extend(std::iter::repeat_n(0, 13));
    let polymod = polymod_run(&input, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK) ^ MK_REGULAR_CONST;
    let mut out = [0u8; 13];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = ((polymod >> (5 * (12 - i))) & 0x1F) as u8;
    }
    out
}

/// Verify a regular-code BCH checksum.
///
/// `data_with_checksum` is the full data part including the trailing 13
/// checksum characters. Returns `true` iff the polymod over
/// `hrp_expand(hrp) || data_with_checksum` equals [`MK_REGULAR_CONST`].
pub fn bch_verify_regular(hrp: &str, data_with_checksum: &[u8]) -> bool {
    if data_with_checksum.len() < 13 {
        return false;
    }
    let mut input = hrp_expand(hrp);
    input.extend_from_slice(data_with_checksum);
    polymod_run(&input, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK) == MK_REGULAR_CONST
}

/// Compute the 15-character BCH checksum for the long code.
///
/// Same algorithm as [`bch_create_checksum_regular`] but uses the long-code
/// polymod parameters (`GEN_LONG`, `LONG_SHIFT`, `LONG_MASK`) and target
/// constant ([`MK_LONG_CONST`]). Produces a 15-element checksum array.
pub fn bch_create_checksum_long(hrp: &str, data: &[u8]) -> [u8; 15] {
    // Long code: 15-symbol checksum (0..=14), pad/array/extraction all use 15.
    let mut input = hrp_expand(hrp);
    input.extend_from_slice(data);
    input.extend(std::iter::repeat_n(0, 15));
    let polymod = polymod_run(&input, &GEN_LONG, LONG_SHIFT, LONG_MASK) ^ MK_LONG_CONST;
    let mut out = [0u8; 15];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = ((polymod >> (5 * (14 - i))) & 0x1F) as u8;
    }
    out
}

/// Verify a long-code BCH checksum.
///
/// Same algorithm as [`bch_verify_regular`] with long-code parameters.
/// Returns false if `data_with_checksum` is shorter than 15 symbols.
pub fn bch_verify_long(hrp: &str, data_with_checksum: &[u8]) -> bool {
    if data_with_checksum.len() < 15 {
        return false;
    }
    let mut input = hrp_expand(hrp);
    input.extend_from_slice(data_with_checksum);
    polymod_run(&input, &GEN_LONG, LONG_SHIFT, LONG_MASK) == MK_LONG_CONST
}

/// Result of a successful BCH decode + correct attempt.
///
/// Returned by [`bch_correct_regular`] / [`bch_correct_long`] when correction
/// succeeds. `corrections_applied == 0` means the input was already valid;
/// `> 0` means substitutions were applied at the indicated positions.
///
/// Marked `#[non_exhaustive]` to allow future fields (e.g., confidence
/// score, syndrome metadata) without breaking downstream struct-literal
/// construction. Construct via the [`bch_correct_regular`] /
/// [`bch_correct_long`] APIs.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrectionResult {
    /// The corrected `data_with_checksum` slice (input may have been modified).
    pub data: Vec<u8>,
    /// Number of substitutions applied (0 = clean input).
    pub corrections_applied: usize,
    /// Indices into `data` of the substituted positions.
    pub corrected_positions: Vec<usize>,
}

/// Attempt to correct a regular-code BCH-checksummed string with up to four
/// substitutions, the full t = 4 capacity of the BCH(93, 80, 8) code.
///
/// Implements the standard syndrome-based BCH decoder pipeline: syndrome
/// computation in `GF(1024) = GF(32²)`, Berlekamp–Massey for the
/// error-locator polynomial, Chien search for error positions, Forney's
/// algorithm for error magnitudes. After applying the proposed corrections,
/// the result is re-verified via [`bch_verify_regular`]; the decoder rejects
/// any output that does not produce a valid codeword (defensive guard
/// against pathological 5+-error inputs whose syndromes happen to factor as
/// a degree-≤ 4 locator).
///
/// Returns `Ok(CorrectionResult)` if the input is clean or up to four
/// substitutions repair it. Returns `Err(Error::BchUncorrectable)` otherwise.
///
/// # Algorithm details
///
/// See the private `bch_decode` submodule for the algorithm and the
/// `GF(1024)` field representation.
pub fn bch_correct_regular(
    hrp: &str,
    data_with_checksum: &[u8],
) -> Result<CorrectionResult, crate::Error> {
    if bch_verify_regular(hrp, data_with_checksum) {
        return Ok(CorrectionResult {
            data: data_with_checksum.to_vec(),
            corrections_applied: 0,
            corrected_positions: vec![],
        });
    }
    // Compute polymod over hrp_expand(hrp) || data_with_checksum, XOR with
    // the MD target constant. The result is congruent to the error
    // polynomial E(x) modulo g_regular(x).
    let mut input = hrp_expand(hrp);
    input.extend_from_slice(data_with_checksum);
    let residue = polymod_run(&input, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK) ^ MK_REGULAR_CONST;

    if let Some((positions, magnitudes)) =
        bch_decode::decode_regular_errors(residue, data_with_checksum.len())
    {
        if positions.is_empty() {
            // Should be unreachable (caller already verified); guard anyway.
            return Ok(CorrectionResult {
                data: data_with_checksum.to_vec(),
                corrections_applied: 0,
                corrected_positions: vec![],
            });
        }
        let mut corrected = data_with_checksum.to_vec();
        for (&p, &m) in positions.iter().zip(&magnitudes) {
            if p >= corrected.len() {
                return Err(crate::Error::BchUncorrectable(format!(
                    "decoder reported error position {p} outside data ({} symbols)",
                    corrected.len()
                )));
            }
            corrected[p] ^= m;
        }
        // Defensive: re-verify. Catches the 5+-error edge case.
        if bch_verify_regular(hrp, &corrected) {
            return Ok(CorrectionResult {
                corrections_applied: positions.len(),
                corrected_positions: positions,
                data: corrected,
            });
        }
    }
    Err(crate::Error::BchUncorrectable(
        "regular code: more than 4 substitutions or pathological pattern".into(),
    ))
}

/// Long-code analog of [`bch_correct_regular`].
///
/// Implements the same BM/Chien/Forney pipeline against the long-code
/// generator polynomial, reaching the full t = 4 capacity of
/// `BCH(108, 93, 8)`.
pub fn bch_correct_long(
    hrp: &str,
    data_with_checksum: &[u8],
) -> Result<CorrectionResult, crate::Error> {
    if bch_verify_long(hrp, data_with_checksum) {
        return Ok(CorrectionResult {
            data: data_with_checksum.to_vec(),
            corrections_applied: 0,
            corrected_positions: vec![],
        });
    }
    let mut input = hrp_expand(hrp);
    input.extend_from_slice(data_with_checksum);
    let residue = polymod_run(&input, &GEN_LONG, LONG_SHIFT, LONG_MASK) ^ MK_LONG_CONST;

    if let Some((positions, magnitudes)) =
        bch_decode::decode_long_errors(residue, data_with_checksum.len())
    {
        if positions.is_empty() {
            return Ok(CorrectionResult {
                data: data_with_checksum.to_vec(),
                corrections_applied: 0,
                corrected_positions: vec![],
            });
        }
        let mut corrected = data_with_checksum.to_vec();
        for (&p, &m) in positions.iter().zip(&magnitudes) {
            if p >= corrected.len() {
                return Err(crate::Error::BchUncorrectable(format!(
                    "decoder reported error position {p} outside data ({} symbols)",
                    corrected.len()
                )));
            }
            corrected[p] ^= m;
        }
        if bch_verify_long(hrp, &corrected) {
            return Ok(CorrectionResult {
                corrections_applied: positions.len(),
                corrected_positions: positions,
                data: corrected,
            });
        }
    }
    Err(crate::Error::BchUncorrectable(
        "long code: more than 4 substitutions or pathological pattern".into(),
    ))
}

/// Encode a 5-bit-symbol data stream as a complete mk1 string.
///
/// The data stream is the concatenation `header_symbols || bytes_to_5bit(payload_bytes)`
/// where `header_symbols` is the 2-symbol single-string header or the
/// 8-symbol chunked header (closure Q-5). The BCH code variant (regular or
/// long) is auto-selected from the resulting data-part length per BIP 93:
/// regular for ≤93-symbol data parts, long for 96–108-symbol data parts.
/// Lengths in the reserved-invalid 94–95 gap or outside the BIP 93 valid
/// range return [`Error::InvalidStringLength`].
///
/// Per the v0.1 emit policy described in `design/IMPLEMENTATION_PLAN_mk_v0_1.md`
/// §5.4, callers control fragment sizing so that each chunked fragment lands
/// within long-code territory. Single-string mk1 may pick regular or long
/// based on bytecode size.
///
/// Returns the full string starting with [`crate::consts::HRP`] and the
/// BIP 173 separator (`"mk1"`).
pub fn encode_5bit_to_string(data_5bit: &[u8]) -> Result<String, crate::Error> {
    use crate::Error;

    // Auto-determine code from the eventual data-part length (data_5bit + checksum).
    let regular_total = data_5bit.len() + 13;
    let long_total = data_5bit.len() + 15;
    let code = match (
        bch_code_for_length(regular_total),
        bch_code_for_length(long_total),
    ) {
        (Some(BchCode::Regular), _) => BchCode::Regular,
        (_, Some(BchCode::Long)) => BchCode::Long,
        // Neither code variant accepts this data-part length: too short, in
        // the 94–95 reserved-invalid gap, or too long for v0.1.
        _ => {
            // Pick the closest length to report — long_total is always larger,
            // so report that as the "actual length you tried to produce".
            return Err(Error::InvalidStringLength(long_total));
        }
    };

    let checksum: Vec<u8> = match code {
        BchCode::Regular => bch_create_checksum_regular(HRP, data_5bit).to_vec(),
        BchCode::Long => bch_create_checksum_long(HRP, data_5bit).to_vec(),
    };

    let mut full = String::with_capacity(HRP.len() + 1 + data_5bit.len() + checksum.len());
    full.push_str(HRP);
    full.push(SEPARATOR);
    for &v in data_5bit {
        full.push(ALPHABET[v as usize] as char);
    }
    for v in checksum {
        full.push(ALPHABET[v as usize] as char);
    }
    Ok(full)
}

/// Result of a successful mk1 string decode at the BCH layer.
///
/// Use [`Self::data`] to access the data part as 5-bit values (header
/// symbols + payload, checksum stripped); the string-layer reassembler
/// in `crate::string_layer` splits header symbols off and feeds the
/// remaining payload through [`five_bit_to_bytes`] to recover the original
/// fragment bytes.
///
/// The full post-correction 5-bit symbol sequence (data **plus** the trailing
/// 13- or 15-char checksum) is retained internally as [`Self::data_with_checksum`]
/// and can be queried by [`Self::corrected_char_at`] for any position in
/// the data part — including positions that fall inside the checksum region.
/// The decoder-report layer uses this to surface the real corrected
/// character when BCH ECC repairs a substitution inside the checksum
/// (parallels md-codec's `Correction.corrected` field).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedString {
    /// Detected BCH code variant.
    pub code: BchCode,
    /// Number of substitution errors corrected (0 = clean input, 1 = recovered).
    pub corrections_applied: usize,
    /// Indices into the data-part (chars after `"md1"`) of any corrected positions.
    pub corrected_positions: Vec<usize>,
    /// Full post-correction 5-bit symbol sequence (data part + checksum), in
    /// the same coordinate system as [`Self::corrected_positions`].
    ///
    /// Length is `data().len() + 13` (regular code) or `data().len() + 15`
    /// (long code). Indices `0..data().len()` mirror [`Self::data`] symbol-for-symbol;
    /// indices `data().len()..` are the corrected checksum symbols. Use
    /// [`Self::corrected_char_at`] for the human-readable bech32 character at
    /// any position.
    pub data_with_checksum: Vec<u8>,
}

impl DecodedString {
    /// Data part as 5-bit values, with the trailing checksum stripped.
    ///
    /// Returns a slice into [`Self::data_with_checksum`] — the data part is
    /// `data_with_checksum[..len - checksum_len]`, where `checksum_len` is 13
    /// for [`BchCode::Regular`] and 15 for [`BchCode::Long`].
    pub fn data(&self) -> &[u8] {
        let checksum_len = match self.code {
            BchCode::Regular => 13,
            BchCode::Long => 15,
        };
        &self.data_with_checksum[..self.data_with_checksum.len() - checksum_len]
    }

    /// Look up the corrected bech32 character at the given position in the
    /// data part (chars after the `"md1"` HRP+separator).
    ///
    /// `char_position` is 0-indexed. Positions `0..data().len()` are in the
    /// data region; positions `data().len()..data().len() + checksum_len` are
    /// inside the BCH checksum (13 chars for [`BchCode::Regular`], 15 for
    /// [`BchCode::Long`]). All positions return the post-correction
    /// character — i.e., what the symbol *should* be after BCH repair, which
    /// is exactly what [`Correction.corrected`][crate::Correction::corrected]
    /// is documented to report.
    ///
    /// # Panics
    ///
    /// Panics if `char_position >= data_with_checksum.len()`. Callers are
    /// responsible for clamping the position to a valid range; in the decode
    /// pipeline this is guaranteed by the BCH layer (it never reports a
    /// `corrected_position` outside `data_with_checksum`). Note that
    /// `data_with_checksum` includes the checksum region; "outside the data
    /// part" elsewhere in this crate excludes the checksum and is a tighter
    /// bound than what this method requires.
    pub fn corrected_char_at(&self, char_position: usize) -> char {
        let v = self.data_with_checksum[char_position];
        ALPHABET[v as usize] as char
    }
}

/// Decode an mk1 string, validating HRP, case, length, and checksum.
///
/// Performs full BCH error correction up to four substitutions
/// (`t = 4` capacity of the BCH(93, 80, 8) regular code and the
/// BCH(108, 93, 8) long code), via syndrome-based Berlekamp–Massey +
/// Forney decoding (implemented in the sibling `bch_decode` module).
///
/// Errors:
/// - [`Error::MixedCase`] if the string mixes upper and lower case.
/// - [`Error::InvalidHrp`] if the HRP is missing or not [`crate::consts::HRP`].
/// - [`Error::InvalidStringLength`] if the data-part length isn't a valid mk1 length.
/// - [`Error::InvalidChar`] if the data part contains a non-bech32 character.
/// - [`Error::BchUncorrectable`] if the checksum can't be repaired within
///   the BCH `t = 4` correction radius.
///
/// [`Error::MixedCase`]: crate::Error::MixedCase
/// [`Error::InvalidHrp`]: crate::Error::InvalidHrp
/// [`Error::InvalidStringLength`]: crate::Error::InvalidStringLength
/// [`Error::InvalidChar`]: crate::Error::InvalidChar
/// [`Error::BchUncorrectable`]: crate::Error::BchUncorrectable
pub fn decode_string(s: &str) -> Result<DecodedString, crate::Error> {
    use crate::Error;

    if matches!(case_check(s), CaseStatus::Mixed) {
        return Err(Error::MixedCase);
    }
    let s_lower = s.to_lowercase();

    let sep_pos = s_lower
        .rfind(SEPARATOR)
        .ok_or_else(|| Error::InvalidHrp(s_lower.clone()))?;
    let (hrp, rest) = s_lower.split_at(sep_pos);
    let data_part = &rest[1..]; // skip the '1' separator

    if hrp != HRP {
        return Err(Error::InvalidHrp(hrp.to_string()));
    }

    let code =
        bch_code_for_length(data_part.len()).ok_or(Error::InvalidStringLength(data_part.len()))?;

    let mut values: Vec<u8> = Vec::with_capacity(data_part.len());
    for (i, c) in data_part.chars().enumerate() {
        if !c.is_ascii() {
            return Err(Error::InvalidChar { ch: c, position: i });
        }
        let v = ALPHABET_INV[c as usize];
        if v == 0xFF {
            return Err(Error::InvalidChar { ch: c, position: i });
        }
        values.push(v);
    }

    let correction = match code {
        BchCode::Regular => bch_correct_regular(hrp, &values),
        BchCode::Long => bch_correct_long(hrp, &values),
    };
    let result = correction?;

    Ok(DecodedString {
        code,
        corrections_applied: result.corrections_applied,
        corrected_positions: result.corrected_positions,
        data_with_checksum: result.data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bch_code_equality() {
        assert_eq!(BchCode::Regular, BchCode::Regular);
        assert_ne!(BchCode::Regular, BchCode::Long);
    }

    #[test]
    fn bch_code_can_be_hashed() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(BchCode::Regular);
        set.insert(BchCode::Long);
        set.insert(BchCode::Regular);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn alphabet_is_32_unique_chars() {
        let mut seen = std::collections::HashSet::new();
        for &c in ALPHABET {
            assert!(seen.insert(c), "duplicate char in alphabet: {}", c as char);
        }
        assert_eq!(seen.len(), 32);
    }

    #[test]
    fn bytes_to_5bit_round_trip_zero() {
        let bytes = vec![0x00];
        let fives = bytes_to_5bit(&bytes);
        assert_eq!(fives, vec![0, 0]);
        let back = five_bit_to_bytes(&fives).unwrap();
        assert_eq!(back, bytes);
    }

    #[test]
    fn bytes_to_5bit_round_trip_known_value() {
        // 0xFF = binary 11111111. Splits as 11111 (=31) and 111 (padded with 00 to 11100=28).
        let bytes = vec![0xFF];
        let fives = bytes_to_5bit(&bytes);
        assert_eq!(fives, vec![31, 28]);
    }

    #[test]
    fn bytes_to_5bit_round_trip_multibyte() {
        // 3 bytes = 24 bits → 5 five-bit groups (25 bits, 1 pad bit).
        let bytes = vec![0xDE, 0xAD, 0xBE];
        let back = five_bit_to_bytes(&bytes_to_5bit(&bytes)).unwrap();
        assert_eq!(back, bytes);
    }

    #[test]
    fn five_bit_to_bytes_rejects_nonzero_padding() {
        // Two 5-bit values = 10 bits, of which 8 form a byte and 2 are padding.
        // If padding bits are nonzero, decode must fail.
        // 31 = 11111, 1 = 00001. Last 2 bits (= 01) are nonzero padding.
        assert!(five_bit_to_bytes(&[31, 1]).is_none());
    }

    #[test]
    fn five_bit_to_bytes_rejects_value_out_of_range() {
        assert!(five_bit_to_bytes(&[32]).is_none());
    }

    #[test]
    fn bch_code_for_length_regular() {
        assert_eq!(bch_code_for_length(14), Some(BchCode::Regular));
        assert_eq!(bch_code_for_length(93), Some(BchCode::Regular));
    }

    #[test]
    fn bch_code_for_length_long() {
        assert_eq!(bch_code_for_length(96), Some(BchCode::Long));
        assert_eq!(bch_code_for_length(108), Some(BchCode::Long));
    }

    #[test]
    fn bch_code_for_length_rejects_94_and_95() {
        assert_eq!(bch_code_for_length(94), None);
        assert_eq!(bch_code_for_length(95), None);
    }

    #[test]
    fn bch_code_for_length_rejects_extremes() {
        assert_eq!(bch_code_for_length(0), None);
        assert_eq!(bch_code_for_length(13), None);
        assert_eq!(bch_code_for_length(109), None);
        assert_eq!(bch_code_for_length(1000), None);
    }

    #[test]
    fn case_check_lowercase() {
        assert_eq!(case_check("md1qq"), CaseStatus::Lower);
    }

    #[test]
    fn case_check_uppercase() {
        assert_eq!(case_check("MD1QQ"), CaseStatus::Upper);
    }

    #[test]
    fn case_check_mixed() {
        assert_eq!(case_check("mD1qq"), CaseStatus::Mixed);
    }

    #[test]
    fn case_check_empty_string_is_lower() {
        assert_eq!(case_check(""), CaseStatus::Lower);
    }

    #[test]
    fn case_check_digits_only_is_lower() {
        // Digits have no case; result must be Lower (BIP 173: no-letter strings are lower).
        assert_eq!(case_check("1234"), CaseStatus::Lower);
    }

    #[test]
    fn gen_regular_has_5_entries() {
        assert_eq!(GEN_REGULAR.len(), 5);
    }

    #[test]
    fn gen_long_has_5_entries() {
        assert_eq!(GEN_LONG.len(), 5);
    }

    #[test]
    fn gen_regular_matches_bip93_canonical_values() {
        // Cross-checked against https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki
        // ms32_polymod GEN array. If this fails, the constants drifted from the BIP.
        assert_eq!(GEN_REGULAR[0], 0x19dc500ce73fde210);
        assert_eq!(GEN_REGULAR[1], 0x1bfae00def77fe529);
        assert_eq!(GEN_REGULAR[2], 0x1fbd920fffe7bee52);
        assert_eq!(GEN_REGULAR[3], 0x1739640bdeee3fdad);
        assert_eq!(GEN_REGULAR[4], 0x07729a039cfc75f5a);
    }

    #[test]
    fn gen_long_matches_bip93_canonical_values() {
        // Cross-checked against https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki
        // ms32_long_polymod GEN array.
        assert_eq!(GEN_LONG[0], 0x3d59d273535ea62d897);
        assert_eq!(GEN_LONG[1], 0x7a9becb6361c6c51507);
        assert_eq!(GEN_LONG[2], 0x543f9b7e6c38d8a2a0e);
        assert_eq!(GEN_LONG[3], 0x0c577eaeccf1990d13c);
        assert_eq!(GEN_LONG[4], 0x1887f74f8dc71b10651);
    }

    #[test]
    fn polymod_init_matches_bip93() {
        // POLYMOD_INIT is unchanged from BIP 93; the GEN_REGULAR / GEN_LONG
        // constants have their own value-equality tests.
        assert_eq!(POLYMOD_INIT, 0x23181b3);
    }

    // (NUMS-derivation reproducer for `MK_REGULAR_CONST` / `MK_LONG_CONST`
    // lives in `crate::consts::tests::nums_constants_reproduce_from_domain`,
    // which uses the mk1-specific domain `b"shibbolethnumskey"`. Duplicating
    // it here would risk drift if either side were updated in isolation.)

    #[test]
    fn polymod_masks_are_consistent_with_shifts() {
        // The mask must be (1 << shift) - 1 so that masking preserves bits below
        // the shift boundary, exactly matching the BIP 93 algorithm.
        assert_eq!(REGULAR_MASK, (1u128 << REGULAR_SHIFT) - 1);
        assert_eq!(LONG_MASK, (1u128 << LONG_SHIFT) - 1);
        assert_eq!(REGULAR_SHIFT, 60);
        assert_eq!(LONG_SHIFT, 70);
    }

    #[test]
    fn polymod_step_zero_residue_zero_value() {
        // Both residue and value zero, no GEN XORs since b = 0.
        assert_eq!(
            polymod_step(0, 0, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK),
            0
        );
    }

    #[test]
    fn polymod_step_value_only_xor_when_residue_zero() {
        // Residue 0, value 7 → result is 7 (XORed into the shifted-zero residue).
        assert_eq!(
            polymod_step(0, 7, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK),
            7
        );
    }

    #[test]
    fn polymod_step_isolates_each_gen_entry() {
        // Setting just bit `shift+i` in the residue → b = 1<<i → only GEN[i] is XORed.
        for i in 0..5 {
            let r = 1u128 << (REGULAR_SHIFT + i);
            assert_eq!(
                polymod_step(r, 0, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK),
                GEN_REGULAR[i as usize],
                "bit {} of b should isolate GEN_REGULAR[{}]",
                i,
                i
            );
        }
    }

    #[test]
    fn polymod_step_xors_multiple_gens_when_multiple_b_bits_set() {
        // b = 0b00011 → XOR GEN[0] and GEN[1].
        let r = 0b00011u128 << REGULAR_SHIFT;
        assert_eq!(
            polymod_step(r, 0, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK),
            GEN_REGULAR[0] ^ GEN_REGULAR[1]
        );
        // b = 0b11111 → XOR all 5.
        let r = 0b11111u128 << REGULAR_SHIFT;
        let expected =
            GEN_REGULAR[0] ^ GEN_REGULAR[1] ^ GEN_REGULAR[2] ^ GEN_REGULAR[3] ^ GEN_REGULAR[4];
        assert_eq!(
            polymod_step(r, 0, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK),
            expected
        );
    }

    #[test]
    fn polymod_step_works_for_long_code() {
        // Same parameterization works for the long code (shift=70, mask=LONG_MASK).
        let r = 1u128 << LONG_SHIFT;
        assert_eq!(
            polymod_step(r, 0, &GEN_LONG, LONG_SHIFT, LONG_MASK),
            GEN_LONG[0]
        );
        // b = 0b11111 → XOR all 5 long GENs.
        let r = 0b11111u128 << LONG_SHIFT;
        let expected = GEN_LONG[0] ^ GEN_LONG[1] ^ GEN_LONG[2] ^ GEN_LONG[3] ^ GEN_LONG[4];
        assert_eq!(
            polymod_step(r, 0, &GEN_LONG, LONG_SHIFT, LONG_MASK),
            expected
        );
    }

    #[test]
    fn polymod_step_init_residue_first_iteration() {
        // POLYMOD_INIT < 2^60 so b = 0 in the first iteration; only the shift+xor happens.
        // Verify: polymod_step(POLYMOD_INIT, 0) = POLYMOD_INIT << 5.
        assert_eq!(
            polymod_step(POLYMOD_INIT, 0, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK),
            POLYMOD_INIT << 5
        );
        // And with value=v: polymod_step(POLYMOD_INIT, v) = (POLYMOD_INIT << 5) ^ v.
        assert_eq!(
            polymod_step(POLYMOD_INIT, 31, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK),
            (POLYMOD_INIT << 5) ^ 31
        );
    }

    #[test]
    fn polymod_step_value_and_gen_xor_combined() {
        // Both effects active: b = 1 (bit 0 of b set) AND value = 5.
        // Expected: ((residue & mask) << 5) ^ value ^ GEN[0]
        //         = (0 << 5) ^ 5 ^ GEN[0]
        //         = GEN_REGULAR[0] ^ 5
        let r = 1u128 << REGULAR_SHIFT;
        assert_eq!(
            polymod_step(r, 5, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK),
            GEN_REGULAR[0] ^ 5
        );
    }

    #[test]
    fn hrp_expand_mk_matches_spec() {
        // BIP 173 hrp_expand for the MK HRP. Each ASCII byte contributes
        // its high 3 bits then (after the [0] separator) its low 5 bits.
        // 'm' = 0x6D → high 3 bits = 3, low 5 bits = 13.
        // 'k' = 0x6B → high 3 bits = 3, low 5 bits = 11.
        // Result: [3, 3, 0, 13, 11]. Documented in the BIP draft §"Checksum".
        assert_eq!(hrp_expand(crate::consts::HRP), vec![3, 3, 0, 13, 11]);
    }

    #[test]
    fn hrp_expand_empty_returns_just_separator() {
        // Edge case: empty HRP yields just the [0] separator.
        assert_eq!(hrp_expand(""), vec![0]);
    }

    #[test]
    fn bch_round_trip_regular() {
        // Encode then verify a small data part. The verify call sees the
        // full data + checksum, so polymod returns MK_REGULAR_CONST exactly.
        let hrp = "mk";
        let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let checksum = bch_create_checksum_regular(hrp, &data);
        assert_eq!(checksum.len(), 13);

        let mut full = data.clone();
        full.extend_from_slice(&checksum);
        assert!(bch_verify_regular(hrp, &full));
    }

    #[test]
    fn bch_verify_rejects_single_char_tampering_regular() {
        // Flipping one bit in one symbol breaks verification.
        // (Spot check; BCH detects all single-symbol errors by construction.)
        let hrp = "mk";
        let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let checksum = bch_create_checksum_regular(hrp, &data);
        let mut full = data.clone();
        full.extend_from_slice(&checksum);
        full[5] ^= 0x01;
        assert!(!bch_verify_regular(hrp, &full));
    }

    #[test]
    fn bch_verify_rejects_too_short_input_regular() {
        // Less than 13 symbols cannot hold a checksum.
        assert!(!bch_verify_regular("mk", &[0, 1, 2]));
        assert!(!bch_verify_regular("mk", &[]));
    }

    // (mk1-specific pinned-checksum vectors are deferred to Phase 6 vector
    // corpus generation, which writes both regular- and long-code conformance
    // points to disk under `crates/mk-codec/src/test_vectors/v0.1.json`.
    // Forking md-codec's pinned vectors verbatim would record the wrong
    // values: mk1's HRP and target constants both differ.)

    #[test]
    fn bch_zero_data_does_not_self_validate_regular() {
        // The all-zeros data + all-zeros checksum must NOT validate, because
        // MK_REGULAR_CONST was chosen NUMS-style to avoid this trivial case.
        // Data length 8 is arbitrary; any non-empty zero-fill exhibits the same
        // negative result. 8 echoes the regular-code known-vector data length.
        let mut zero = vec![0u8; 8];
        zero.extend(std::iter::repeat_n(0, 13));
        assert!(!bch_verify_regular("mk", &zero));
    }

    #[test]
    fn bch_round_trip_empty_data_regular() {
        // Empty data part is a degenerate but valid input: the checksum
        // covers only the HRP preamble. encode → verify must round-trip.
        let checksum = bch_create_checksum_regular("mk", &[]);
        assert!(bch_verify_regular("mk", &checksum));
    }

    #[test]
    fn bch_round_trip_long() {
        let hrp = "mk";
        let data: Vec<u8> = (0..16).collect();
        let checksum = bch_create_checksum_long(hrp, &data);
        assert_eq!(checksum.len(), 15);
        let mut full = data.clone();
        full.extend_from_slice(&checksum);
        assert!(bch_verify_long(hrp, &full));
    }

    #[test]
    fn bch_verify_rejects_single_char_tampering_long() {
        // Flipping one bit in one symbol breaks verification.
        // (Spot check; BCH detects all single-symbol errors by construction.)
        let hrp = "mk";
        let data: Vec<u8> = (0..16).collect();
        let checksum = bch_create_checksum_long(hrp, &data);
        let mut full = data.clone();
        full.extend_from_slice(&checksum);
        full[7] ^= 0x01;
        assert!(!bch_verify_long(hrp, &full));
    }

    #[test]
    fn bch_verify_rejects_too_short_input_long() {
        // Less than 15 symbols cannot hold a long-code checksum.
        assert!(!bch_verify_long("mk", &[0; 14]));
        assert!(!bch_verify_long("mk", &[]));
    }

    #[test]
    fn bch_zero_data_does_not_self_validate_long() {
        // All-zeros must not validate, by NUMS construction of MK_LONG_CONST.
        // Data length 16 is arbitrary; any non-empty zero-fill exhibits the same
        // negative result. 16 echoes the long-code known-vector data length.
        let mut zero = vec![0u8; 16];
        zero.extend(std::iter::repeat_n(0, 15));
        assert!(!bch_verify_long("mk", &zero));
    }

    #[test]
    fn bch_round_trip_empty_data_long() {
        // Degenerate but valid: checksum covers only the HRP preamble.
        let checksum = bch_create_checksum_long("mk", &[]);
        assert!(bch_verify_long("mk", &checksum));
    }

    #[test]
    fn bch_correct_regular_clean_input() {
        // Clean input → 0 corrections, identity result.
        let hrp = "mk";
        let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let checksum = bch_create_checksum_regular(hrp, &data);
        let mut full = data.clone();
        full.extend_from_slice(&checksum);
        let r = bch_correct_regular(hrp, &full).unwrap();
        assert_eq!(r.corrections_applied, 0);
        assert!(r.corrected_positions.is_empty());
        assert_eq!(r.data, full);
    }

    #[test]
    fn bch_correct_regular_one_error() {
        // Single-symbol corruption is recoverable.
        let hrp = "mk";
        let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let checksum = bch_create_checksum_regular(hrp, &data);
        let mut full = data.clone();
        full.extend_from_slice(&checksum);
        let original = full.clone();
        full[3] = (full[3] + 1) & 0x1F;
        let r = bch_correct_regular(hrp, &full).unwrap();
        assert_eq!(r.corrections_applied, 1);
        assert_eq!(r.corrected_positions, vec![3]);
        assert_eq!(r.data, original);
    }

    #[test]
    fn bch_correct_regular_two_errors_recovered_v0_2() {
        // v0.2 BM/Forney decoder reaches the BCH(93,80,8) full t = 4
        // capacity. A 2-error pattern is now recoverable. This test was
        // `..._uncorrectable_v0_1` in v0.1; flipped sign in v0.2.
        let hrp = "mk";
        let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let checksum = bch_create_checksum_regular(hrp, &data);
        let mut full = data.clone();
        full.extend_from_slice(&checksum);
        let original = full.clone();
        full[3] = (full[3] + 1) & 0x1F;
        full[7] = (full[7] + 1) & 0x1F;
        let r = bch_correct_regular(hrp, &full).unwrap();
        assert_eq!(r.corrections_applied, 2);
        assert!(r.corrected_positions.contains(&3));
        assert!(r.corrected_positions.contains(&7));
        assert_eq!(r.data, original);
    }

    #[test]
    fn bch_correct_long_clean_input() {
        let hrp = "mk";
        let data: Vec<u8> = (0..16).collect();
        let checksum = bch_create_checksum_long(hrp, &data);
        let mut full = data.clone();
        full.extend_from_slice(&checksum);
        let r = bch_correct_long(hrp, &full).unwrap();
        assert_eq!(r.corrections_applied, 0);
    }

    #[test]
    fn bch_correct_long_one_error() {
        let hrp = "mk";
        let data: Vec<u8> = (0..16).collect();
        let checksum = bch_create_checksum_long(hrp, &data);
        let mut full = data.clone();
        full.extend_from_slice(&checksum);
        let original = full.clone();
        full[5] = (full[5] + 1) & 0x1F;
        let r = bch_correct_long(hrp, &full).unwrap();
        assert_eq!(r.corrections_applied, 1);
        assert_eq!(r.corrected_positions, vec![5]);
        assert_eq!(r.data, original);
    }

    #[test]
    fn bch_correct_returns_correction_result_with_position() {
        // Verify the API contract: a successful 1-error correction reports
        // exactly the position that was changed.
        let hrp = "mk";
        let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let checksum = bch_create_checksum_regular(hrp, &data);
        let mut full = data.clone();
        full.extend_from_slice(&checksum);
        // Damage the second checksum byte (position 9 from start).
        full[9] = (full[9] + 7) & 0x1F;
        let r = bch_correct_regular(hrp, &full).unwrap();
        assert_eq!(r.corrected_positions, vec![9]);
    }

    /// Build a fake mk1 5-bit data stream for round-trip tests:
    /// `[v0, v1, ...]` are 2 bech32-symbol single-string-style header
    /// symbols; `payload_bytes` is the byte-level fragment.
    fn build_5bit_data(header_symbols: &[u8], payload_bytes: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(header_symbols.len() + payload_bytes.len() * 2);
        out.extend_from_slice(header_symbols);
        out.extend(bytes_to_5bit(payload_bytes));
        out
    }

    #[test]
    fn encode_5bit_to_string_round_trip_regular() {
        // 2-symbol single-string header + 4-byte payload → 7 5-bit symbols
        // (header [v=0, t=0] || bytes_to_5bit(4 bytes) = 2 + 7 = 9 symbols).
        // 9 + 13 regular checksum = 22-char data part — well within regular range.
        let header_symbols = [0u8, 0u8];
        let payload = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let data_5bit = build_5bit_data(&header_symbols, &payload);
        let s = encode_5bit_to_string(&data_5bit).unwrap();
        assert!(s.starts_with("mk1"), "string did not start with mk1: {}", s);

        let decoded = decode_string(&s).unwrap();
        assert_eq!(decoded.code, BchCode::Regular);
        assert_eq!(decoded.corrections_applied, 0);
        assert!(decoded.corrected_positions.is_empty());
        assert_eq!(decoded.data(), data_5bit.as_slice());

        // Recover the payload by stripping the 2-symbol header and byte-decoding.
        let payload_5bit = &decoded.data()[2..];
        let recovered = five_bit_to_bytes(payload_5bit).unwrap();
        assert_eq!(recovered, payload);
    }

    #[test]
    fn encode_5bit_to_string_round_trip_long() {
        // Force a long-code path with an 8-symbol chunked-style header +
        // a 53-byte fragment: data_5bit.len() = 8 + ceil(53*8/5) = 8 + 85 = 93,
        // + 15 long checksum = 108 — exact long-code upper bound.
        let header_symbols = [0u8; 8];
        let payload = vec![0xA5u8; 53];
        let data_5bit = build_5bit_data(&header_symbols, &payload);
        assert_eq!(
            data_5bit.len(),
            93,
            "fixture invariant: 8 header + 85 payload symbols"
        );
        let s = encode_5bit_to_string(&data_5bit).unwrap();
        assert!(s.starts_with("mk1"));
        let decoded = decode_string(&s).unwrap();
        assert_eq!(decoded.code, BchCode::Long);
        assert_eq!(decoded.data(), data_5bit.as_slice());

        let recovered = five_bit_to_bytes(&decoded.data()[8..]).unwrap();
        assert_eq!(recovered, payload);
    }

    #[test]
    fn encode_starts_with_hrp_and_separator() {
        // Minimum-shape input: 1 5-bit symbol + 13 regular checksum = 14 — the
        // tightest valid regular-code data-part length.
        let s = encode_5bit_to_string(&[1u8]).unwrap();
        assert!(s.starts_with("mk1"), "string did not start with mk1: {}", s);
    }

    #[test]
    fn decode_rejects_invalid_hrp() {
        let s = encode_5bit_to_string(&[0u8; 10]).unwrap();
        let bad = s.replacen("mk", "bt", 1);
        assert!(matches!(
            decode_string(&bad),
            Err(crate::Error::InvalidHrp(_))
        ));
    }

    #[test]
    fn decode_rejects_mixed_case() {
        let s = encode_5bit_to_string(&[0u8; 10]).unwrap();
        let bad: String = s
            .chars()
            .enumerate()
            .map(|(i, c)| if i == 5 { c.to_ascii_uppercase() } else { c })
            .collect();
        assert!(matches!(decode_string(&bad), Err(crate::Error::MixedCase)));
    }

    #[test]
    fn decode_rejects_invalid_char() {
        // 'b' is excluded from the bech32 alphabet; substitute one in the data
        // part to force a parse-time character rejection.
        let s = encode_5bit_to_string(&[0u8; 10]).unwrap();
        // s looks like "mk1...". Splice 'b' at index 5 (definitely past "mk1").
        let mut chars: Vec<char> = s.chars().collect();
        chars[5] = 'b';
        let bad: String = chars.into_iter().collect();
        assert!(matches!(
            decode_string(&bad),
            Err(crate::Error::InvalidChar { .. })
        ));
    }

    #[test]
    fn decode_rejects_missing_separator() {
        // No '1' at all in the string. rfind('1') returns None → InvalidHrp.
        let bad = "mknoseparatorhere";
        assert!(matches!(
            decode_string(bad),
            Err(crate::Error::InvalidHrp(_))
        ));
    }

    #[test]
    fn decode_recovers_one_error() {
        // Encode, corrupt one char in the data part, decode should auto-correct.
        let data_5bit = vec![0u8, 0u8, 1, 2, 3, 4, 5];
        let s = encode_5bit_to_string(&data_5bit).unwrap();

        let mut chars: Vec<char> = s.chars().collect();
        // Corrupt position 6 (past "mk1", well within the data part).
        let original_char = chars[6];
        chars[6] = if original_char == 'q' { 'p' } else { 'q' };
        let corrupted: String = chars.into_iter().collect();

        let decoded = decode_string(&corrupted).unwrap();
        assert_eq!(decoded.corrections_applied, 1);
        assert_eq!(decoded.corrected_positions.len(), 1);
        assert_eq!(decoded.data(), data_5bit.as_slice());
    }

    #[test]
    fn encode_rejects_data_part_in_reserved_invalid_length_range() {
        // For 5-bit data-part lengths 0..=12 (so data_part = 13..=25 with regular
        // checksum, or 15..=27 with long), `bch_code_for_length` rejects below 14.
        // Empty input → data_5bit.len()=0 → regular_total=13 → None; long_total=15
        // → Regular. Wait — regular range starts at 14 not 13.
        //
        // Actual invariant test: len 0 → regular_total=13 (None, below 14) and
        // long_total=15 (Regular). So it falls back to long->regular ladder ...
        // Re-checking encode_5bit_to_string: only fails when both miss [14..=93]
        // and [96..=108]. For data_5bit.len()=79, regular_total=92 → Regular ✓.
        // The provable reserved-invalid case is a length that misses both
        // ranges; the BIP 93 BCH ladder leaves no such gap below 109 because
        // [14..=93] ∪ [96..=108] only excludes {0..=13, 94..=95, ≥109}. The
        // smallest input length that produces invalid data-part lengths in
        // BOTH the regular and long branches is therefore data_5bit.len() ≥ 94
        // (regular_total ≥ 107 in invalid territory, long_total ≥ 109 too long).
        let too_long = vec![0u8; 94];
        let result = encode_5bit_to_string(&too_long);
        assert!(matches!(result, Err(crate::Error::InvalidStringLength(_))));
    }
}
