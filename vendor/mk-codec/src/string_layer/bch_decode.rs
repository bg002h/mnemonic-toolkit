//! Syndrome-based BCH decoder for the MK regular and long codes.
//!
//! Forked from `md-codec` v0.4.x (`crates/md-codec/src/encoding/bch_decode.rs`)
//! at the start of the mk1 v0.1 implementation per `design/DECISIONS.md` D-13.
//! The algorithm is shared with the sibling md1 format because both formats
//! share BIP 93's BCH polynomials; only the target residue constants
//! ([`crate::consts::MK_REGULAR_CONST`] / [`crate::consts::MK_LONG_CONST`])
//! and the HRP differ. The fork copy is expected to be retired once the
//! `mc-codex32` shared-crate extraction lands (closure Q-9 trigger: both
//! formats v1.0 with cross-validated conformance vectors).
//!
//! Implements the textbook decoder pipeline:
//!
//! 1. **Syndrome computation**: compute eight syndromes
//!    `S_m = E(α^{j_start - 1 + m})` for `m = 1, …, 8` where `α` is the
//!    primitive element of the BCH-defining field and `j_start` is the
//!    smallest integer in the 8-consecutive-roots window of the
//!    generator polynomial. For the regular code `α = β = G·ζ` (order 93)
//!    with `j_start = 77`; for the long code `α = γ = E + X·ζ` (order
//!    1023) with `j_start = 1019`.
//! 2. **Berlekamp–Massey**: derive the error-locator polynomial `Λ(x)`
//!    from the eight syndromes. Runs in `O(t²)` for `t = 4`.
//! 3. **Chien search**: enumerate `Λ` over every codeword position to
//!    locate error positions.
//! 4. **Forney's algorithm** (shifted form): derive each error magnitude
//!    `e_k = X_k^{1 - j_start} · Ω(X_k^{-1}) / Λ'(X_k^{-1})` from the
//!    syndrome polynomial `S(x)`, the error-evaluator polynomial
//!    `Ω(x) ≡ S(x)·Λ(x) mod x⁸`, and the formal derivative `Λ'(x)`.
//!    The `X_k^{1 - j_start}` factor accounts for syndromes starting at
//!    `α^{j_start}` rather than `α^1`; cf. Lin & Costello §6.3 eq. (6.21)
//!    with the substitution `S_j → S_{j_start + j - 1}`.
//! 5. **Apply corrections**: XOR the error magnitudes into the received
//!    word at the recovered positions.
//! 6. **Verify** (caller's responsibility): defensive re-check via the
//!    polymod primitive guards against pathological inputs (≥ 5 errors
//!    that happen to produce a degree-≤ 4 `Λ` with 4 valid roots).
//!
//! # Field and root structure (BIP 93 §"Generation of valid checksum")
//!
//! `GF(32)` uses the codex32/BIP 93 primitive polynomial `x⁵ + x³ + 1`,
//! with the multiplicative generator being the bit value `0b00010 = 2`
//! (the bech32 `"z"` character). This matches the `bech32` crate's
//! `Fe32` representation.
//!
//! `GF(1024) = GF(32)[ζ] / (ζ² - ζ - P)` where `P = 1` (so `ζ² = ζ + 1`).
//! `ζ` is a primitive cube root of unity. For the **regular code**:
//!
//! ```text
//! β = G·ζ                 (G = 8, so β = (0, 8) in our (lo, hi) form)
//! ord(β) = 93
//! roots of g_regular(x) are { β^17, β^20, β^46, β^49, β^52,
//!                             β^77, β^78, β^79, β^80, β^81,
//!                             β^82, β^83, β^84 }
//! 8-consecutive window: { β^77, …, β^84 } ⇒ j_start = 77
//! ```
//!
//! For the **long code**:
//!
//! ```text
//! γ = E + X·ζ             (E = 25, X = 6, so γ = (25, 6))
//! ord(γ) = 1023
//! roots of g_long(x) are { γ^32, γ^64, γ^96,
//!                          γ^895, γ^927, γ^959, γ^991,
//!                          γ^1019, γ^1020, γ^1021, γ^1022,
//!                          γ^1023, γ^1024, γ^1025, γ^1026 }
//! 8-consecutive window: { γ^1019, …, γ^1026 } ⇒ j_start = 1019
//! ```
//!
//! Both windows are 8 consecutive integer powers of the chosen primitive
//! element, satisfying the BCH bound and giving `t = 4` correction.
//!
//! # Position indexing
//!
//! The polymod consumes symbols in the order
//! `hrp_expand(hrp) || data || checksum`. If `n` is the total number of
//! symbols fed, then symbol `i` (in feed order) is the coefficient of
//! `x^{n-1-i}` in the input polynomial. Errors are constrained to the
//! `data_with_checksum` segment (the HRP prefix is fixed-and-known).
//! For `data_with_checksum.len() = L` (`L ≤ 93` regular, `96 ≤ L ≤ 108`
//! long), an error at index `k` of `data_with_checksum` lies at
//! polynomial degree `d = L - 1 - k`. The Chien search returns degrees
//! `d` and we translate to indices via `k = (L - 1) - d`.

#[cfg(test)]
use super::bch::{GEN_LONG, GEN_REGULAR};

// ---------------------------------------------------------------------------
// GF(32) — same field as `crate::string_layer::bch::ALPHABET` codes
// ---------------------------------------------------------------------------

/// One element of `GF(32) = GF(2)[α] / (α⁵ + α³ + 1)`, encoded as a
/// 5-bit integer `0..32` whose binary digits are the polynomial
/// coefficients (low bit = constant term).
type Gf32 = u8;

/// Primitive polynomial reduction mask for `GF(32)`: when a `GF(32)`
/// multiplication overflows into bit 5, XOR with `0b00_1001 = 9` to fold
/// `α⁵ ≡ α³ + 1` back into the residue.
const GF32_REDUCE: u8 = 0b0_1001;

/// Multiply two `GF(32)` elements (carryless multiply with reduction).
const fn gf32_mul(a: Gf32, b: Gf32) -> Gf32 {
    let mut result: u8 = 0;
    let mut a = a;
    let mut i = 0;
    while i < 5 {
        if (b >> i) & 1 != 0 {
            result ^= a;
        }
        // Multiply a by α; reduce if it leaves the 5-bit window.
        let carry = (a >> 4) & 1;
        a = (a << 1) & 0x1F;
        if carry != 0 {
            a ^= GF32_REDUCE;
        }
        i += 1;
    }
    result
}

// ---------------------------------------------------------------------------
// GF(1024) — built as GF(32²) via ζ² = ζ + 1
// ---------------------------------------------------------------------------

/// One element of `GF(1024)` as a pair `(lo, hi)` of `GF(32)` elements
/// representing `lo + hi·ζ` where `ζ² = ζ + 1` (i.e., `ζ` is a
/// primitive cube root of unity in `GF(1024)*`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Gf1024 {
    lo: Gf32,
    hi: Gf32,
}

impl Gf1024 {
    const ZERO: Gf1024 = Gf1024 { lo: 0, hi: 0 };
    const ONE: Gf1024 = Gf1024 { lo: 1, hi: 0 };

    /// Embed a `GF(32)` element as the constant term.
    const fn from_gf32(v: Gf32) -> Self {
        Gf1024 { lo: v, hi: 0 }
    }

    fn add(self, other: Self) -> Self {
        Gf1024 {
            lo: self.lo ^ other.lo,
            hi: self.hi ^ other.hi,
        }
    }

    fn is_zero(self) -> bool {
        self.lo == 0 && self.hi == 0
    }

    /// Multiply two `GF(1024)` elements using the field relation
    /// `ζ² = ζ + 1`. Concretely:
    ///
    /// ```text
    /// (lo + hi·ζ) · (lo' + hi'·ζ)
    ///   = lo·lo' + (lo·hi' + hi·lo')·ζ + hi·hi'·ζ²
    ///   = lo·lo' + (lo·hi' + hi·lo')·ζ + hi·hi'·(ζ + 1)
    ///   = (lo·lo' + hi·hi') + (lo·hi' + hi·lo' + hi·hi')·ζ
    /// ```
    fn mul(self, other: Self) -> Self {
        let ll = gf32_mul(self.lo, other.lo);
        let lh = gf32_mul(self.lo, other.hi);
        let hl = gf32_mul(self.hi, other.lo);
        let hh = gf32_mul(self.hi, other.hi);
        Gf1024 {
            lo: ll ^ hh,
            hi: lh ^ hl ^ hh,
        }
    }

    fn pow(self, mut exp: u32) -> Self {
        let mut base = self;
        let mut result = Gf1024::ONE;
        while exp > 0 {
            if exp & 1 == 1 {
                result = result.mul(base);
            }
            base = base.mul(base);
            exp >>= 1;
        }
        result
    }

    fn inv(self) -> Self {
        // Fermat: a^(2^10 - 2) = a^1022 = a^-1 in GF(1024)*.
        debug_assert!(!self.is_zero(), "inv of zero in GF(1024)");
        self.pow(1022)
    }
}

/// `ζ`, a primitive cube root of unity. With our `Y² = Y + 1` quadratic,
/// `ζ` is `Y` itself, encoded as `(0, 1)` in `(lo, hi)` form. Used in
/// tests to verify the field relation `ζ² = ζ + 1`; the runtime
/// arithmetic uses [`BETA`] and [`GAMMA`] directly.
#[cfg(test)]
const ZETA: Gf1024 = Gf1024 { lo: 0, hi: 1 };

/// `β = G·ζ = 8·ζ`, the primitive element for the **regular code**'s
/// BCH-defining group. `β` has order 93. (BIP 93 §"Generation of valid
/// checksum".)
const BETA: Gf1024 = Gf1024 { lo: 0, hi: 8 };

/// `γ = E + X·ζ = 25 + 6·ζ`, the primitive element for the **long
/// code**'s BCH-defining group. `γ` has order 1023.
const GAMMA: Gf1024 = Gf1024 { lo: 25, hi: 6 };

/// Smallest exponent in the 8-consecutive-roots window of the regular
/// code's generator polynomial: `g_regular(β^j) = 0` for `j = 77, …, 84`.
const REGULAR_J_START: u32 = 77;

/// Smallest exponent in the 8-consecutive-roots window of the long
/// code's generator polynomial: `g_long(γ^j) = 0` for
/// `j = 1019, 1020, …, 1026`.
const LONG_J_START: u32 = 1019;

// `β` and `γ` orders (93 and 1023, respectively) are tested directly
// in the unit-test module via inline integer constants; we don't need
// run-time symbols for them.

// ---------------------------------------------------------------------------
// Generator polynomial reconstruction (used only for self-test)
// ---------------------------------------------------------------------------

/// Reconstruct `g_regular(x)`, the degree-13 BCH generator polynomial,
/// from `GEN_REGULAR[0]`. Returns coefficients with `result[i]` being
/// the coefficient of `x^i`. The leading coefficient (`result[13]`) is 1.
///
/// **Why this works**: `polymod_step` computes
/// `(residue · x + symbol) mod g(x)` in `GF(32)[x] / g(x)`. The constant
/// `GEN_REGULAR[0]` is, by construction,
/// `1 · x^13 mod g(x) = x^13 mod g(x)`, packed as 13 5-bit GF(32)
/// coefficients (high coeff = `x^12`, low coeff = `x^0`). Since
/// `g(x) = x^13 + (x^13 - g(x))`, and reduction in characteristic 2
/// gives `g_low = x^13 mod g(x)`, we have
/// `g(x) = x^13 + GEN_REGULAR[0]_packed_as_polynomial`.
#[cfg(test)]
fn generator_polynomial_regular() -> [Gf32; 14] {
    let mut g = [0u8; 14];
    g[13] = 1;
    for (i, slot) in g.iter_mut().enumerate().take(13) {
        *slot = ((GEN_REGULAR[0] >> (5 * i)) & 0x1F) as u8;
    }
    g
}

/// Reconstruct `g_long(x)`, the degree-15 BCH generator polynomial,
/// from `GEN_LONG[0]`. Same packing convention as
/// [`generator_polynomial_regular`].
#[cfg(test)]
fn generator_polynomial_long() -> [Gf32; 16] {
    let mut g = [0u8; 16];
    g[15] = 1;
    for (i, slot) in g.iter_mut().enumerate().take(15) {
        *slot = ((GEN_LONG[0] >> (5 * i)) & 0x1F) as u8;
    }
    g
}

/// Horner-form polynomial evaluation: GF(32)-coefficient polynomial at
/// a GF(1024) point. `coeffs[i]` is the coefficient of `x^i`.
fn horner(coeffs: &[Gf32], x: Gf1024) -> Gf1024 {
    let mut acc = Gf1024::ZERO;
    for &c in coeffs.iter().rev() {
        acc = acc.mul(x).add(Gf1024::from_gf32(c));
    }
    acc
}

/// Horner-form polynomial evaluation: GF(1024)-coefficient polynomial
/// at a GF(1024) point. `coeffs[i]` is the coefficient of `x^i`.
fn horner_ext(coeffs: &[Gf1024], x: Gf1024) -> Gf1024 {
    let mut acc = Gf1024::ZERO;
    for &c in coeffs.iter().rev() {
        acc = acc.mul(x).add(c);
    }
    acc
}

// ---------------------------------------------------------------------------
// Syndromes
// ---------------------------------------------------------------------------

/// Compute the eight syndromes
/// `S_m = E(α^{j_start + m - 1})` for `m = 1, …, 8`, where `E(x)` is the
/// error polynomial (recoverable as the polymod residue minus the MD
/// target constant). The remainder is already congruent to `E(x)`
/// modulo `g(x)`, so evaluating it at the generator's roots is
/// equivalent to evaluating `E(x)` itself.
fn compute_syndromes(
    residue_xor_const: u128,
    checksum_len: u32,
    alpha: Gf1024,
    j_start: u32,
) -> [Gf1024; 8] {
    // Unpack the remainder: `checksum_len` GF(32) coefficients packed
    // with the highest-order coefficient (x^{checksum_len-1}) at bit
    // 5*(checksum_len-1) and the constant term (x^0) at bits 0..5.
    // Stack-allocate at the maximum (Long code = 15); the active slice
    // is `&coeffs[..checksum_len]`.
    let mut coeffs = [0u8; 15];
    let len = checksum_len as usize;
    for i in 0..checksum_len {
        coeffs[i as usize] = ((residue_xor_const >> (5 * i)) & 0x1F) as u8;
    }
    let coeffs = &coeffs[..len];

    let mut syndromes = [Gf1024::ZERO; 8];
    let alpha_j_start = alpha.pow(j_start);
    let mut alpha_j = alpha_j_start;
    for s in &mut syndromes {
        *s = horner(coeffs, alpha_j);
        alpha_j = alpha_j.mul(alpha);
    }
    syndromes
}

// ---------------------------------------------------------------------------
// Berlekamp–Massey
// ---------------------------------------------------------------------------

/// Berlekamp–Massey for BCH over `GF(1024)`. Returns the error-locator
/// polynomial `Λ(x)` with `Λ(0) = 1`. `Λ` has degree equal to the
/// number of errors when the received word is correctable.
fn berlekamp_massey(syndromes: &[Gf1024; 8]) -> Vec<Gf1024> {
    // Standard formulation (Massey 1969 / Lin & Costello §6.3, adapted
    // for 0-indexed syndromes where syndromes[k] = S_{j_start + k}).
    let n = syndromes.len();
    let mut lam: Vec<Gf1024> = vec![Gf1024::ONE]; // current connection poly
    let mut prev: Vec<Gf1024> = vec![Gf1024::ONE]; // last-updated connection poly
    let mut l: usize = 0; // current LFSR length
    let mut m: usize = 1; // shift since last update
    let mut b = Gf1024::ONE; // discrepancy from last update

    for k in 0..n {
        // Discrepancy: d = syndromes[k] + sum_{i=1..L} lam[i] * syndromes[k-i]
        let mut d = syndromes[k];
        for i in 1..=l {
            // i > k means k - i would underflow; skip rather than wrap.
            // i >= lam.len() means lam[i] doesn't exist yet; same skip.
            if i <= k && i < lam.len() {
                d = d.add(lam[i].mul(syndromes[k - i]));
            }
        }

        if d.is_zero() {
            m += 1;
        } else if 2 * l <= k {
            // Length increases. New lam = lam - (d/b) * x^m * prev.
            let t = lam.clone();
            let scale = d.mul(b.inv());
            let new_len = (lam.len()).max(prev.len() + m);
            lam.resize(new_len, Gf1024::ZERO);
            for (i, &p) in prev.iter().enumerate() {
                let idx = i + m;
                lam[idx] = lam[idx].add(scale.mul(p));
            }
            l = k + 1 - l;
            prev = t;
            b = d;
            m = 1;
        } else {
            // Length stays the same. lam = lam - (d/b) * x^m * prev.
            let scale = d.mul(b.inv());
            let new_len = (lam.len()).max(prev.len() + m);
            lam.resize(new_len, Gf1024::ZERO);
            for (i, &p) in prev.iter().enumerate() {
                let idx = i + m;
                lam[idx] = lam[idx].add(scale.mul(p));
            }
            m += 1;
        }
    }

    while lam.len() > 1 && lam.last().is_some_and(|x| x.is_zero()) {
        lam.pop();
    }
    lam
}

// ---------------------------------------------------------------------------
// Chien search + Forney
// ---------------------------------------------------------------------------

/// Search for the roots of `Λ(x)` among `α⁰, α⁻¹, …, α⁻⁽ᴸ⁻¹⁾`, where
/// `L = data_with_checksum_len` (we restrict the search to legitimate
/// error positions; HRP-prefix positions are not transmitted).
///
/// Returns the list of polynomial degrees `d ∈ [0, L)` such that
/// `Λ(α⁻ᵈ) = 0`. Each such `d` is the polynomial degree of an error.
/// Returns `None` if the number of distinct roots found does not equal
/// `deg(Λ)`.
fn chien_search(
    lambda: &[Gf1024],
    data_with_checksum_len: usize,
    alpha: Gf1024,
) -> Option<Vec<usize>> {
    let deg = lambda.len() - 1;
    if deg == 0 {
        return Some(Vec::new());
    }

    let mut error_degrees = Vec::with_capacity(deg);
    let alpha_inv = alpha.inv();
    let mut current = Gf1024::ONE; // α^0
    for d in 0..data_with_checksum_len {
        if horner_ext(lambda, current).is_zero() {
            error_degrees.push(d);
        }
        current = current.mul(alpha_inv);
    }

    if error_degrees.len() != deg {
        return None;
    }
    Some(error_degrees)
}

/// Shifted Forney's algorithm: given `Λ(x)`, the syndromes (at
/// `α^{j_start}, …, α^{j_start + 7}`), and the error degrees `d_k` such
/// that `α^{-d_k}` are the roots of `Λ`, compute the GF(32) error
/// magnitudes at each position.
///
/// Formula (with `j_start` shift):
///
/// ```text
/// e_k = X_k^{1 - j_start} · Ω(X_k^{-1}) / Λ'(X_k^{-1})
/// ```
///
/// where `X_k = α^{d_k}`, `Ω(x) ≡ S(x)·Λ(x) mod x^8`, and `Λ'(x)` is
/// the formal derivative.
///
/// Returns `None` if any computed magnitude does not lie in the symbol
/// field `GF(32)`.
fn forney(
    syndromes: &[Gf1024; 8],
    lambda: &[Gf1024],
    error_degrees: &[usize],
    alpha: Gf1024,
    j_start: u32,
) -> Option<Vec<Gf32>> {
    // Ω(x) = S(x) * Λ(x) mod x^8, where S(x) = sum_{m=0..7} S_{j_start + m} * x^m.
    let s_poly: Vec<Gf1024> = syndromes.to_vec();
    let mut omega = vec![Gf1024::ZERO; 8];
    for i in 0..s_poly.len().min(8) {
        for j in 0..lambda.len() {
            if i + j < 8 {
                omega[i + j] = omega[i + j].add(s_poly[i].mul(lambda[j]));
            }
        }
    }

    // Λ'(x) = formal derivative. In characteristic 2 only odd-power
    // terms survive: Λ'(x) = sum_{i odd} lambda[i] * x^{i-1}.
    let mut lambda_prime = vec![Gf1024::ZERO; lambda.len().saturating_sub(1)];
    for i in 1..lambda.len() {
        if i % 2 == 1 {
            lambda_prime[i - 1] = lambda[i];
        }
    }

    let mut magnitudes = Vec::with_capacity(error_degrees.len());
    for &d in error_degrees {
        // X_k = α^d.
        let x_k = alpha.pow(d as u32);
        let x_k_inv = x_k.inv();
        let omega_val = horner_ext(&omega, x_k_inv);
        let lam_p_val = horner_ext(&lambda_prime, x_k_inv);
        if lam_p_val.is_zero() {
            return None;
        }

        // Compute X_k^{1 - j_start}. Note `1 - j_start` is negative;
        // since X_k has order ord(α) (93 or 1023), we use
        // X_k^{1 - j_start} = X_k^{(ord - j_start + 1) mod ord}.
        // But we handle this generically via x_k_inv^{j_start - 1}.
        let shift = j_start.saturating_sub(1);
        let x_k_shift = x_k_inv.pow(shift); // = X_k^{-(j_start - 1)} = X_k^{1 - j_start}

        let mag = x_k_shift.mul(omega_val.mul(lam_p_val.inv()));

        // Magnitude must lie in GF(32) (the high coefficient must be zero).
        if mag.hi != 0 {
            return None;
        }
        if mag.lo == 0 {
            // Zero magnitude is not a real error — typically signals
            // more than 4 actual errors that fooled BM.
            return None;
        }
        magnitudes.push(mag.lo);
    }
    Some(magnitudes)
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Decode a regular-code BCH error pattern. Inputs:
///
/// - `residue_xor_const`: the value
///   `polymod(hrp_expand(hrp) || data_with_checksum) ⊕ MK_REGULAR_CONST`.
///   By the BCH syndrome property, this is congruent to the error
///   polynomial `E(x)` modulo `g_regular(x)`.
/// - `data_with_checksum_len`: the total symbol count of
///   `data_with_checksum` (in the `0..=93` range for the regular code).
///
/// Returns `Some((positions, magnitudes))` if the algorithm finds a
/// consistent error pattern of weight `≤ 4`. Each `positions[k]` is an
/// index into `data_with_checksum` (post-HRP-prefix); each
/// `magnitudes[k]` is a `GF(32)` symbol that must be XORed into
/// `data_with_checksum[positions[k]]` to repair the codeword. Returns
/// `None` if the pattern is uncorrectable.
// v0.3.1: promoted from `pub(super)` so downstream consumers (toolkit
// `repair` feature) can compute corrections for ms / md HRPs using their
// own target-residue constants (all 3 share the BIP-93 BCH(93,80,8) code).
pub fn decode_regular_errors(
    residue_xor_const: u128,
    data_with_checksum_len: usize,
) -> Option<(Vec<usize>, Vec<Gf32>)> {
    decode_errors(
        residue_xor_const,
        data_with_checksum_len,
        13,
        BETA,
        REGULAR_J_START,
    )
}

/// Long-code analog of [`decode_regular_errors`].
///
/// v0.3.1: promoted from `pub(super)` for downstream-consumer access (see
/// `decode_regular_errors` docs).
pub fn decode_long_errors(
    residue_xor_const: u128,
    data_with_checksum_len: usize,
) -> Option<(Vec<usize>, Vec<Gf32>)> {
    decode_errors(
        residue_xor_const,
        data_with_checksum_len,
        15,
        GAMMA,
        LONG_J_START,
    )
}

fn decode_errors(
    residue_xor_const: u128,
    data_with_checksum_len: usize,
    checksum_len: u32,
    alpha: Gf1024,
    j_start: u32,
) -> Option<(Vec<usize>, Vec<Gf32>)> {
    let syndromes = compute_syndromes(residue_xor_const, checksum_len, alpha, j_start);

    // All-zero syndromes ⇒ no errors (caller usually detects earlier).
    if syndromes.iter().all(|s| s.is_zero()) {
        return Some((Vec::new(), Vec::new()));
    }

    let lambda = berlekamp_massey(&syndromes);
    let deg = lambda.len() - 1;
    if deg == 0 || deg > 4 {
        // > 4 errors is above the BCH(•, •, 8) / t = 4 capacity.
        return None;
    }

    let error_degrees = chien_search(&lambda, data_with_checksum_len, alpha)?;
    if error_degrees.len() != deg {
        return None;
    }

    let magnitudes = forney(&syndromes, &lambda, &error_degrees, alpha, j_start)?;

    // Translate polynomial degrees back to data_with_checksum indices.
    // For data_with_checksum[k] (k = 0..L-1), polynomial degree d = L - 1 - k.
    // So k = L - 1 - d.
    let mut positions = Vec::with_capacity(error_degrees.len());
    for &d in &error_degrees {
        if d >= data_with_checksum_len {
            // Should not happen since chien_search bounds d to [0, L).
            return None;
        }
        let k = data_with_checksum_len - 1 - d;
        positions.push(k);
    }

    // Sort ascending by position for deterministic output. Magnitudes
    // need to be reordered along with the positions.
    let mut paired: Vec<(usize, Gf32)> = positions.into_iter().zip(magnitudes).collect();
    paired.sort_by_key(|p| p.0);
    let positions: Vec<usize> = paired.iter().map(|p| p.0).collect();
    let magnitudes: Vec<Gf32> = paired.iter().map(|p| p.1).collect();

    Some((positions, magnitudes))
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::{MK_LONG_CONST, MK_REGULAR_CONST};
    use crate::string_layer::bch::{
        GEN_LONG, GEN_REGULAR, LONG_MASK, LONG_SHIFT, REGULAR_MASK, REGULAR_SHIFT,
        bch_create_checksum_long, bch_create_checksum_regular, hrp_expand,
    };

    #[test]
    fn gf32_mul_identity() {
        for v in 0..32u8 {
            assert_eq!(gf32_mul(v, 1), v);
            assert_eq!(gf32_mul(1, v), v);
        }
    }

    #[test]
    fn gf32_mul_zero() {
        for v in 0..32u8 {
            assert_eq!(gf32_mul(v, 0), 0);
            assert_eq!(gf32_mul(0, v), 0);
        }
    }

    #[test]
    fn gf32_alpha_powers_match_bech32_log_inv_table() {
        // Cross-check: alpha = 2 (= "z"). Powers of alpha must match the
        // LOG_INV table from the bech32 crate.
        let mut a: u8 = 1;
        let expected: [u8; 31] = [
            1, 2, 4, 8, 16, 9, 18, 13, 26, 29, 19, 15, 30, 21, 3, 6, 12, 24, 25, 27, 31, 23, 7, 14,
            28, 17, 11, 22, 5, 10, 20,
        ];
        for &exp in &expected {
            assert_eq!(a, exp);
            a = gf32_mul(a, 2);
        }
        // After 31 multiplications by alpha, we should be back to 1.
        assert_eq!(a, 1);
    }

    #[test]
    fn zeta_is_primitive_cube_root_of_unity() {
        // ζ² = ζ + 1, ζ³ = ζ·(ζ + 1) = ζ² + ζ = 2ζ + 1 = 1 (in char 2).
        let zeta_sq = ZETA.mul(ZETA);
        assert_eq!(zeta_sq, ZETA.add(Gf1024::ONE), "ζ² should equal ζ + 1");
        let zeta_cu = zeta_sq.mul(ZETA);
        assert_eq!(zeta_cu, Gf1024::ONE, "ζ³ should equal 1");
    }

    #[test]
    fn beta_has_order_93_regular() {
        // β = G·ζ has order 93 (BIP 93 §"Generation of valid checksum").
        let mut p = Gf1024::ONE;
        for j in 1..=93 {
            p = p.mul(BETA);
            if p == Gf1024::ONE {
                assert_eq!(j, 93, "β prematurely returned to 1 at exponent {}", j);
            }
        }
        assert_eq!(p, Gf1024::ONE, "β^93 should equal 1");
    }

    #[test]
    fn gamma_has_order_1023_long() {
        // γ = E + X·ζ has order 1023 (BIP 93 §"Generation of valid checksum").
        // Quick-check at the 3 prime divisors of 1023 = 3·11·31.
        for &q in &[341u32, 93u32, 33u32] {
            // 1023/3, 1023/11, 1023/31
            assert_ne!(GAMMA.pow(q), Gf1024::ONE, "γ^(1023/p) = 1 for some p");
        }
        assert_eq!(GAMMA.pow(1023), Gf1024::ONE, "γ^1023 should equal 1");
    }

    #[test]
    fn generator_polynomial_evaluates_to_zero_at_specified_roots() {
        // Cross-check the BIP 93 §"Generation of valid checksum" claim
        // that g_regular(β^i) = 0 for i ∈ {17, 20, 46, 49, 52, 77..84}
        // and g_long(γ^i) = 0 for i ∈ {32, 64, 96, 895, 927, 959, 991,
        // 1019..1026}. Reconstructs g(x) from GEN_*[0] and verifies.
        let g_reg = generator_polynomial_regular();
        let g_long = generator_polynomial_long();

        let regular_roots: [u32; 13] = [17, 20, 46, 49, 52, 77, 78, 79, 80, 81, 82, 83, 84];
        for &i in &regular_roots {
            assert!(
                horner(&g_reg, BETA.pow(i)).is_zero(),
                "g_regular(β^{}) != 0",
                i
            );
        }

        let long_roots: [u32; 15] = [
            32, 64, 96, 895, 927, 959, 991, 1019, 1020, 1021, 1022, 1023, 1024, 1025, 1026,
        ];
        for &i in &long_roots {
            assert!(
                horner(&g_long, GAMMA.pow(i)).is_zero(),
                "g_long(γ^{}) != 0",
                i
            );
        }
    }

    // Re-export the production polymod_run so tests validate field arithmetic
    // against the same code path the codec actually runs. A local duplicate
    // (which used to live here) would let polymod_run bugs go undetected if
    // both copies agreed on the wrong answer.
    use crate::string_layer::bch::polymod_run;

    #[test]
    fn one_error_decodes_correctly_regular() {
        let hrp = "mk";
        let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let checksum = bch_create_checksum_regular(hrp, &data);
        let mut codeword = data.clone();
        codeword.extend_from_slice(&checksum);
        let original = codeword.clone();

        let err_pos = 5;
        let err_mag: u8 = 0b10101;
        codeword[err_pos] ^= err_mag;

        let mut input = hrp_expand(hrp);
        input.extend_from_slice(&codeword);
        let polymod = polymod_run(&input, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK);
        let residue = polymod ^ MK_REGULAR_CONST;

        let (positions, magnitudes) =
            decode_regular_errors(residue, codeword.len()).expect("1-error must decode");
        assert_eq!(positions, vec![err_pos]);
        assert_eq!(magnitudes, vec![err_mag]);

        let mut corrected = codeword.clone();
        for (p, m) in positions.iter().zip(&magnitudes) {
            corrected[*p] ^= m;
        }
        assert_eq!(corrected, original);
    }

    #[test]
    fn two_errors_decode_correctly_regular() {
        let hrp = "mk";
        let data: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let checksum = bch_create_checksum_regular(hrp, &data);
        let mut codeword = data.clone();
        codeword.extend_from_slice(&checksum);
        let original = codeword.clone();

        let positions_in: [usize; 2] = [3, 17];
        let mags_in: [u8; 2] = [0b11001, 0b00111];
        for (&p, &m) in positions_in.iter().zip(&mags_in) {
            codeword[p] ^= m;
        }

        let mut input = hrp_expand(hrp);
        input.extend_from_slice(&codeword);
        let polymod = polymod_run(&input, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK);
        let residue = polymod ^ MK_REGULAR_CONST;

        let (positions, magnitudes) =
            decode_regular_errors(residue, codeword.len()).expect("2-error must decode");
        assert_eq!(positions, vec![3, 17]);
        assert_eq!(magnitudes, vec![mags_in[0], mags_in[1]]);

        let mut corrected = codeword.clone();
        for (p, m) in positions.iter().zip(&magnitudes) {
            corrected[*p] ^= m;
        }
        assert_eq!(corrected, original);
    }

    #[test]
    fn four_errors_decode_correctly_long() {
        let hrp = "mk";
        let data: Vec<u8> = (0..16).collect();
        let checksum = bch_create_checksum_long(hrp, &data);
        let mut codeword = data.clone();
        codeword.extend_from_slice(&checksum);
        let original = codeword.clone();

        let positions_in: [usize; 4] = [0, 5, 18, 28];
        let mags_in: [u8; 4] = [0b00001, 0b10000, 0b11111, 0b01010];
        for (&p, &m) in positions_in.iter().zip(&mags_in) {
            codeword[p] ^= m;
        }

        let mut input = hrp_expand(hrp);
        input.extend_from_slice(&codeword);
        let polymod = polymod_run(&input, &GEN_LONG, LONG_SHIFT, LONG_MASK);
        let residue = polymod ^ MK_LONG_CONST;

        let (positions, magnitudes) =
            decode_long_errors(residue, codeword.len()).expect("4-error must decode");
        assert_eq!(positions, vec![0, 5, 18, 28]);
        assert_eq!(magnitudes, mags_in.to_vec());

        let mut corrected = codeword.clone();
        for (p, m) in positions.iter().zip(&magnitudes) {
            corrected[*p] ^= m;
        }
        assert_eq!(corrected, original);
    }

    #[test]
    fn five_errors_either_rejects_or_returns_bogus_recovery() {
        // The decoder doesn't detect 5+ errors directly. It may return
        // None or return Some() with bogus positions/magnitudes that
        // fail to reproduce the original. The caller's responsibility
        // is to re-verify via `bch_verify_*`.
        let hrp = "mk";
        let data: Vec<u8> = (0..16).collect();
        let checksum = bch_create_checksum_long(hrp, &data);
        let mut codeword = data.clone();
        codeword.extend_from_slice(&checksum);

        let positions_in: [usize; 5] = [0, 5, 10, 15, 20];
        let mags_in: [u8; 5] = [1, 2, 3, 4, 5];
        for (&p, &m) in positions_in.iter().zip(&mags_in) {
            codeword[p] ^= m;
        }

        let mut input = hrp_expand(hrp);
        input.extend_from_slice(&codeword);
        let polymod = polymod_run(&input, &GEN_LONG, LONG_SHIFT, LONG_MASK);
        let residue = polymod ^ MK_LONG_CONST;

        if let Some((positions, magnitudes)) = decode_long_errors(residue, codeword.len()) {
            let original = {
                let mut o = data.clone();
                o.extend_from_slice(&checksum);
                o
            };
            let mut corrected = codeword.clone();
            for (p, m) in positions.iter().zip(&magnitudes) {
                corrected[*p] ^= m;
            }
            assert_ne!(
                corrected, original,
                "5-error decode should not produce the original codeword"
            );
        }
    }
}
