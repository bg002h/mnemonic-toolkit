//! Systematic **evaluation-form** Reed–Solomon over `GF(2^11)` (plan §3 / §4.1).
//!
//! # Frozen wire construction (plan §3) — MUST match exactly
//!
//! - Evaluation points `βⱼ = α^j` for `j = 0, 1, 2, …` (α = [`field::ALPHA`]),
//!   distinct for `j < 2047`.
//! - **Systematic, evaluation form:** a message of `k` data symbols is placed
//!   verbatim at positions `0..k-1`, i.e. `dataⱼ = P(βⱼ)` for `j < k`, where
//!   `P` is the unique degree-`<k` polynomial interpolating `(βⱼ, dataⱼ)`.
//!   **Parity symbol `i` = `P(β_{k+i})`** for `i = 0..m-1`. Codeword position
//!   `j` carries `P(βⱼ)`.
//! - **Append-only / prefix-extensible:** the `β`-sequence is a fixed prefix,
//!   so the first `m` parity symbols are IDENTICAL whether you generate `m` or
//!   `m' > m` of them. (KAT `append_only_prefix`.)
//! - Length cap: `n = k + m ≤ 2047` (else [`RsError::LengthExceedsField`]).
//!
//! Only the *codeword* is wire-frozen — the decode **algorithm** is not. This
//! implementation decodes with **Gao's algorithm** (partial GCD over the
//! interpolant), with **erasures handled by puncturing** (the erased
//! coordinates are excluded from the interpolation point-set). The error +
//! erasure budget (RS distance `d = n - k + 1`) is: correct `t` errors and
//! `s` erasures iff **`2t + s ≤ m`**. Beyond budget the decoder returns
//! [`RsError::Uncorrectable`] (or, exactly at `2t+s = m+1`, MAY return a
//! wrong-but-valid codeword — that residual is caught by the P4 integrity tag,
//! not here). It **never panics**.

use crate::field;
use crate::poly::{self, Poly};

/// The multiplicative-group order — the largest number of distinct `βⱼ = α^j`
/// evaluation points (`j = 0..2046`), hence the codeword length cap.
const MAX_N: usize = field::MULTIPLICATIVE_ORDER as usize; // 2047

/// Errors from the RS encode / decode paths. Variants are **alphabetical**
/// (plan / `CLAUDE.md` convention).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RsError {
    /// An erasure index was `>= codeword.len()` (out of range), or the erasure
    /// list was not strictly sorted-ascending / not distinct.
    ErasureOutOfRange {
        /// The offending erasure index.
        index: usize,
        /// The codeword length it must be below.
        codeword_len: usize,
    },
    /// The codeword length `n = k + m` would exceed the field's distinct
    /// evaluation points (`n > 2047`), so the construction is undefined.
    LengthExceedsField {
        /// The requested `n = k + m` (encode) or `codeword.len()` (decode).
        n: usize,
        /// The hard cap (`2047`).
        max: usize,
    },
    /// A symbol (data, parity, or received codeword symbol) was outside the
    /// `GF(2^11)` range `0..=2047`.
    SymbolOutOfRange {
        /// Index of the offending symbol within its slice.
        index: usize,
        /// The offending value (`>= 2048`).
        value: u16,
    },
    /// The error + erasure weight exceeds the `2t + s ≤ m` budget, or the
    /// punctured system is underdetermined — the message cannot be recovered.
    Uncorrectable,
    /// The requested `data_len` is inconsistent with the codeword (e.g.
    /// `data_len > codeword.len()`), making the geometry undefined.
    Underdetermined {
        /// The requested number of data symbols.
        data_len: usize,
        /// The codeword length available.
        codeword_len: usize,
    },
}

impl core::fmt::Display for RsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RsError::ErasureOutOfRange {
                index,
                codeword_len,
            } => write!(
                f,
                "erasure index {index} out of range / unsorted (codeword length {codeword_len})"
            ),
            RsError::LengthExceedsField { n, max } => {
                write!(
                    f,
                    "codeword length n={n} exceeds the field cap of {max} symbols"
                )
            }
            RsError::SymbolOutOfRange { index, value } => {
                write!(
                    f,
                    "symbol {value} at index {index} is outside GF(2^11) (0..=2047)"
                )
            }
            RsError::Uncorrectable => {
                write!(f, "error/erasure weight exceeds the RS budget (2t + s ≤ m)")
            }
            RsError::Underdetermined {
                data_len,
                codeword_len,
            } => write!(
                f,
                "data_len {data_len} is inconsistent with codeword length {codeword_len}"
            ),
        }
    }
}

impl std::error::Error for RsError {}

/// `βⱼ = α^j`, the `j`-th evaluation point. Distinct for `j < 2047`.
#[inline]
fn beta(j: usize) -> u16 {
    field::pow(field::ALPHA, j as u32)
}

/// Validate that every symbol is a field element (`< 2048`).
fn check_symbols(syms: &[u16]) -> Result<(), RsError> {
    for (index, &value) in syms.iter().enumerate() {
        if value >= field::ORDER {
            return Err(RsError::SymbolOutOfRange { index, value });
        }
    }
    Ok(())
}

/// The `m` **systematic parity** symbols for `data`: `P(β_{k+i})`, `i=0..m-1`,
/// where `P` is the degree-`<k` interpolant of `(βⱼ, dataⱼ)`.
///
/// Append-only: `rs_parity(data, m)[..m'] == rs_parity(data, m')` for `m' < m`,
/// because the parity positions are the fixed prefix `β_k, β_{k+1}, …`.
pub fn rs_parity(data: &[u16], parity_len: usize) -> Result<Vec<u16>, RsError> {
    check_symbols(data)?;
    let k = data.len();
    let n = k
        .checked_add(parity_len)
        .ok_or(RsError::LengthExceedsField {
            n: usize::MAX,
            max: MAX_N,
        })?;
    if n > MAX_N {
        return Err(RsError::LengthExceedsField { n, max: MAX_N });
    }
    if parity_len == 0 {
        return Ok(Vec::new());
    }
    if k == 0 {
        // The unique degree-<0 polynomial is the zero polynomial; every
        // evaluation is 0. (Empty data ⇒ all-zero parity.)
        return Ok(vec![0u16; parity_len]);
    }

    // Interpolate P through (β_j, data_j) for j = 0..k-1.
    let xs: Vec<u16> = (0..k).map(beta).collect();
    let p = poly::interpolate(&xs, data);

    // Evaluate at β_{k}..β_{k+m-1}.
    let parity = (0..parity_len).map(|i| p.eval(beta(k + i))).collect();
    Ok(parity)
}

/// Convenience: the full systematic codeword `data ‖ parity`.
pub fn rs_codeword(data: &[u16], parity_len: usize) -> Result<Vec<u16>, RsError> {
    let parity = rs_parity(data, parity_len)?;
    let mut cw = Vec::with_capacity(data.len() + parity.len());
    cw.extend_from_slice(data);
    cw.extend_from_slice(&parity);
    Ok(cw)
}

/// Validate the erasure list: each in range, strictly ascending (⇒ distinct).
fn check_erasures(erasures: &[usize], codeword_len: usize) -> Result<(), RsError> {
    let mut prev: Option<usize> = None;
    for &e in erasures {
        if e >= codeword_len {
            return Err(RsError::ErasureOutOfRange {
                index: e,
                codeword_len,
            });
        }
        if let Some(p) = prev {
            if e <= p {
                // not strictly ascending ⇒ unsorted or duplicate
                return Err(RsError::ErasureOutOfRange {
                    index: e,
                    codeword_len,
                });
            }
        }
        prev = Some(e);
    }
    Ok(())
}

/// Decode a received `codeword` back to its `data_len` data symbols, correcting
/// errors and the known-bad `erasures` (sorted, distinct, in-range) within the
/// RS budget `2t + s ≤ m` (`m = codeword.len() - data_len`).
///
/// Returns [`RsError::Uncorrectable`] when the weight exceeds the budget or the
/// punctured system is underdetermined; **never panics**. No-silent-miscorrect
/// *within* budget is guaranteed by RS distance; beyond budget a wrong-but-valid
/// codeword is acceptable (caught by the integrity tag in P4, not here).
pub fn rs_decode(
    codeword: &[u16],
    data_len: usize,
    erasures: &[usize],
) -> Result<Vec<u16>, RsError> {
    check_symbols(codeword)?;
    let n = codeword.len();
    if n > MAX_N {
        return Err(RsError::LengthExceedsField { n, max: MAX_N });
    }
    let k = data_len;
    if k > n {
        return Err(RsError::Underdetermined {
            data_len: k,
            codeword_len: n,
        });
    }
    check_erasures(erasures, n)?;

    // Degenerate but well-defined: k == 0 ⇒ message is empty.
    if k == 0 {
        return Ok(Vec::new());
    }
    // No parity: any error/erasure is uncorrectable; only a clean read decodes.
    let m = n - k;
    if !erasures.is_empty() && erasures.len() > m {
        // s > m can never satisfy 2t + s ≤ m (t ≥ 0) ⇒ refuse early.
        return Err(RsError::Uncorrectable);
    }

    // --- Gao decode with erasure puncturing ------------------------------
    // Use only the NON-erased positions. With s erasures we keep N = n - s
    // points; Gao corrects t errors iff 2t < N - k + 1 ⇔ 2t + s ≤ m.
    let erased: Vec<bool> = {
        let mut v = vec![false; n];
        for &e in erasures {
            v[e] = true;
        }
        v
    };
    let used: Vec<usize> = (0..n).filter(|&j| !erased[j]).collect();
    let n_used = used.len();
    if n_used < k {
        // Fewer surviving points than the message degree ⇒ underdetermined.
        return Err(RsError::Uncorrectable);
    }

    let xs: Vec<u16> = used.iter().map(|&j| beta(j)).collect();
    let ys: Vec<u16> = used.iter().map(|&j| codeword[j]).collect();

    // g0 = ∏_{j∈used} (x - β_j);  g1 = interpolant through (β_j, y_j).
    let mut g0 = Poly::constant(1);
    for &x in &xs {
        // (x - β_j) = (x + β_j) in char 2 ⇒ coeffs [β_j, 1].
        g0 = g0.mul(&Poly::from_coeffs(vec![x, 1]));
    }
    let g1 = poly::interpolate(&xs, &ys);

    // Stop the partial-GCD when deg(remainder) < (n_used + k)/2 (integer div
    // is the correct floor threshold for Gao). The decoded message poly is the
    // exact quotient r / v.
    let deg_stop = (n_used + k) / 2;
    let (r, v) = poly::partial_gcd(&g0, &g1, deg_stop);

    if v.is_zero() {
        return Err(RsError::Uncorrectable);
    }
    let (quot, rem) = r.divmod(&v);
    if !rem.is_zero() {
        // v does not divide r exactly ⇒ beyond the correctable budget.
        return Err(RsError::Uncorrectable);
    }
    // The message polynomial must have degree < k.
    if quot.degree().map(|d| d >= k).unwrap_or(false) {
        return Err(RsError::Uncorrectable);
    }

    // Re-evaluate the recovered polynomial at the systematic data positions
    // β_0..β_{k-1} to get the corrected data symbols.
    let recovered: Vec<u16> = (0..k).map(|j| quot.eval(beta(j))).collect();
    Ok(recovered)
}
