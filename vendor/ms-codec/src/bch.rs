//! BIP 93 codex32 BCH primitives for HRP `"ms"` (regular code only).
//!
//! Vendored from md-codec's structure at the v0.34.0 promotion (descriptor-mnemonic
//! commit `94069ea`) per plan §2.B.2 / D22. ms1 strings are all regular-code length
//! per `consts::VALID_STR_LENGTHS`, so the long-code primitives are intentionally
//! absent (mk-codec carries the long-code variants).
//!
//! All public per plan D22 (no `pub(crate)` half-private items in ms-codec): the
//! downstream `bch_decode` module (B.4) re-declares the 3 internal consts locally
//! per the Q3 lock — they stay bare-private here.
//!
//! `MS_REGULAR_CONST` is the BIP-93 codex32 short-code target residue
//! ("SECRETSHARE32"); ms-codec is its single source of truth. (The toolkit's
//! former vendored copy was deleted in its v0.23.0 migration, which now
//! delegates to this crate. That copy held the pre-v0.2.1 WRONG value paired
//! with a wrong `POLYMOD_INIT` — see
//! `design/BUG_decode_with_correction_length_divergence.md`.)

/// BCH(93,80,8) generator polynomial coefficients (5 × 65-bit).
///
/// Identical across mk/ms/md (the polynomial is BIP-93's; only the per-HRP
/// target residue differs).
pub const GEN_REGULAR: [u128; 5] = [
    0x19dc500ce73fde210,
    0x1bfae00def77fe529,
    0x1fbd920fffe7bee52,
    0x1739640bdeee3fdad,
    0x07729a039cfc75f5a,
];

/// MS-domain target residue: codex32's "SECRETSHARE32" Fe-vec packed
/// big-endian in 5-bit chunks — the value [`polymod_run`] (started from the
/// codex32 initial residue [`POLYMOD_INIT`]) produces for ANY valid ms1
/// input, independent of entropy length.
///
/// `SECRETSHARE32 = [s,e,c,r,e,t,s,h,a,r,e,3,2] = [16,25,24,3,25,11,16,23,29,3,25,17,10]`
/// packed as `Σ vᵢ << (5·(12−i))` → `0x10ce0795c2fd1e62a` (bit 64 set). This is
/// the BIP-93 codex32 short-code target the `rust-codex32` engine (which
/// `envelope.rs` uses to encode) checks against, so the hand-rolled path here
/// is byte-equivalent to codex32 for all ms1 lengths.
///
/// NOTE (v0.2.1 fix): the previous value `0x962958058f2c192a`, paired with a
/// wrong `POLYMOD_INIT`, was empirically lifted from a single 12-word vector
/// and only validated 16-byte seeds — see
/// `design/BUG_decode_with_correction_length_divergence.md`.
pub const MS_REGULAR_CONST: u128 = 0x10ce0795c2fd1e62a;

/// codex32 initial polymod residue: the field element `1` (BIP-173/BIP-93
/// bech32-style start state), processed against `hrp_expand("ms") || data`.
/// (Was wrongly `0x23181b3`, which made `polymod_run` length-variant for valid
/// codewords — the root cause of the 20/24/28/32-byte correction bug.)
const POLYMOD_INIT: u128 = 0x1;
const REGULAR_SHIFT: u32 = 60;
const REGULAR_MASK: u128 = 0x0fffffffffffffff;

fn polymod_step(residue: u128, value: u128) -> u128 {
    let b = residue >> REGULAR_SHIFT;
    let mut new_residue = ((residue & REGULAR_MASK) << 5) ^ value;
    for (i, &g) in GEN_REGULAR.iter().enumerate() {
        if (b >> i) & 1 != 0 {
            new_residue ^= g;
        }
    }
    new_residue
}

/// Run the BCH polymod over `values` starting from the BIP-93 initial residue.
///
/// Returns the final residue; callers XOR against the per-HRP target
/// constant ([`MS_REGULAR_CONST`]) to produce a checksum or to verify
/// one. Inputs are 5-bit symbols (`u8` in `0..32`); larger values are
/// reduced modulo 32 by the underlying step.
pub fn polymod_run(values: &[u8]) -> u128 {
    let mut residue = POLYMOD_INIT;
    for &v in values {
        residue = polymod_step(residue, v as u128);
    }
    residue
}

/// BIP 173-style HRP expansion: `[c >> 5 for c in hrp] ++ [0] ++ [c & 31 for c in hrp]`.
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

/// 13-symbol regular-code BCH checksum over `hrp_expand(hrp) || data || [0; 13]`.
pub fn bch_create_checksum_regular(hrp: &str, data: &[u8]) -> [u8; 13] {
    let mut input = hrp_expand(hrp);
    input.extend_from_slice(data);
    input.extend(std::iter::repeat_n(0, 13));
    let polymod = polymod_run(&input) ^ MS_REGULAR_CONST;
    let mut out = [0u8; 13];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = ((polymod >> (5 * (12 - i))) & 0x1F) as u8;
    }
    out
}

/// Verify a regular-code BCH checksum over the data-part-with-checksum.
pub fn bch_verify_regular(hrp: &str, data_with_checksum: &[u8]) -> bool {
    if data_with_checksum.len() < 13 {
        return false;
    }
    let mut input = hrp_expand(hrp);
    input.extend_from_slice(data_with_checksum);
    polymod_run(&input) == MS_REGULAR_CONST
}
