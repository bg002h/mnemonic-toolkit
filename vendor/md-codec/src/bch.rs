//! BIP 93 codex32 BCH primitives for HRP `"md"` (regular code only).
//!
//! Extracted from the v0.x `encoding` module; v0.11 needs only the regular-code
//! checksum + verify (long code dropped along with v0.x).

/// BCH(93,80,8) generator polynomial coefficients (5 × 65-bit).
pub const GEN_REGULAR: [u128; 5] = [
    0x19dc500ce73fde210,
    0x1bfae00def77fe529,
    0x1fbd920fffe7bee52,
    0x1739640bdeee3fdad,
    0x07729a039cfc75f5a,
];

/// MD-domain target residue (NUMS-style, top 65 bits of
/// `SHA-256("shibbolethnums")`).
pub const MD_REGULAR_CONST: u128 = 0x0815c07747a3392e7;

/// Initial polymod residue. This value (`0x23181b3`) IS codex32/BIP-93's
/// `ms32_polymod` initial residue — the reference `ms32_polymod` seeds its
/// accumulator with exactly this constant, and [`GEN_REGULAR`] above is
/// BIP-93's `ms32` generator, term for term. (It is **not** bech32/BIP-173's
/// init `1`; that init belongs to a different code. Earlier notes here
/// claiming md1 "deliberately deviates from codex32's init `1`" and that
/// "only ms1 uses `1`" were both wrong: codex32's init is `0x23181b3`, and no
/// constellation format — md1, mk1, or ms1 — uses `1`.)
///
/// All three constellation codes (md1, mk1, ms1) build on the codex32 `ms32`
/// code. md1 and mk1 seed this init literally (mk1: `mk-codec`'s
/// `string_layer::bch::POLYMOD_INIT`). ms1 (`ms-codec`) uses the mathematically
/// **equivalent** formulation — codex32's literal `1` init with an
/// `hrp_expand("ms")` prepend — because `0x23181b3` is exactly the fold of
/// `hrp_expand("ms")` from `1`; a raw constant-diff against `ms-codec`'s
/// `POLYMOD_INIT = 0x1` is therefore NOT a discrepancy. Cross-format domain
/// separation comes entirely from the per-HRP **target** residue that each
/// format XORs against the final polymod (md1 uses [`MD_REGULAR_CONST`], a
/// NUMS-derived constant, in place of codex32's own `MS32_CONST`) — never from
/// the init. Because this same value seeds both
/// [`bch_create_checksum_regular`] and [`bch_verify_regular`], its contribution
/// cancels between create and verify, so `polymod(valid codeword) ==
/// MD_REGULAR_CONST` holds at every length for any fixed init.
const POLYMOD_INIT: u128 = 0x23181b3;
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

/// Run the BCH polymod over `values` starting from `POLYMOD_INIT`.
///
/// Returns the final residue; callers XOR against the per-HRP target
/// constant (e.g. [`MD_REGULAR_CONST`]) to produce a checksum or to
/// verify one. Inputs are 5-bit symbols (`u8` in `0..32`); larger
/// values are reduced modulo 32 by the underlying step.
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
    let polymod = polymod_run(&input) ^ MD_REGULAR_CONST;
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
    polymod_run(&input) == MD_REGULAR_CONST
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::hashes::{Hash, sha256};

    /// Drift-guard: `MD_REGULAR_CONST` must reproduce from its documented
    /// NUMS rule — the top 65 bits of `SHA-256("shibbolethnums")`. Mirrors
    /// mk-codec's `consts::tests::nums_constants_reproduce_from_domain`
    /// (and ms-codec's `tests/bch_all_lengths.rs::ms_regular_const_is_secretshare32_packed`).
    /// Catches accidental drift if the domain string or the constant is
    /// edited without the other — without this, a silent edit to either would
    /// break cross-format domain separation undetected.
    #[test]
    fn md_regular_const_reproduces_from_nums_domain() {
        let digest = sha256::Hash::hash(b"shibbolethnums");
        let bytes = digest.as_byte_array();
        // Leading 128 bits of the 256-bit digest as a big-endian u128, then
        // the top 65 bits (shift right by 128 - 65 = 63).
        let hi = u128::from_be_bytes(bytes[0..16].try_into().unwrap());
        let derived = hi >> 63;
        assert_eq!(
            derived, MD_REGULAR_CONST,
            "MD_REGULAR_CONST drift from SHA-256(\"shibbolethnums\") top-65-bits",
        );
    }
}
