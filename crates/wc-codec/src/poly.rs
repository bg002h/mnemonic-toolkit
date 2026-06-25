//! Crate-internal polynomial arithmetic over `GF(2^11)` (plan §3 / §5).
//!
//! Polynomials are dense coefficient vectors, **little-endian by degree**:
//! `coeffs[i]` is the coefficient of `x^i`. The zero polynomial is the empty
//! vector (canonical), and every public helper keeps polynomials *trimmed*
//! (no trailing zero coefficients except the canonical-empty zero poly).
//!
//! All coefficients are field elements in `0..2047` — the field ops live in
//! [`crate::field`] and are NOT reimplemented here (plan: reuse P1's field).
//!
//! This module supports the systematic evaluation-form RS engine in
//! [`crate::rs`]: Lagrange interpolation through `(βⱼ, valueⱼ)` points, Horner
//! evaluation, multiply / `divmod` / extended-Euclid (partial GCD) for the Gao
//! decoder.

use crate::field;

/// A dense polynomial over `GF(2^11)`, little-endian by degree.
///
/// Invariant maintained by the constructors / arithmetic here: no trailing
/// zero coefficients (so `degree()` and equality are canonical), with the zero
/// polynomial represented as an empty coefficient vector.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Poly {
    /// `coeffs[i]` = coefficient of `x^i`; trimmed of trailing zeros.
    coeffs: Vec<u16>,
}

impl Poly {
    /// The zero polynomial (empty coefficient vector).
    pub(crate) fn zero() -> Self {
        Poly { coeffs: Vec::new() }
    }

    /// Build from raw little-endian coefficients, trimming trailing zeros.
    pub(crate) fn from_coeffs(mut coeffs: Vec<u16>) -> Self {
        while matches!(coeffs.last(), Some(&0)) {
            coeffs.pop();
        }
        Poly { coeffs }
    }

    /// The constant polynomial `c`.
    pub(crate) fn constant(c: u16) -> Self {
        Poly::from_coeffs(vec![c])
    }

    /// Is this the zero polynomial?
    pub(crate) fn is_zero(&self) -> bool {
        self.coeffs.is_empty()
    }

    /// Degree, or `None` for the zero polynomial.
    pub(crate) fn degree(&self) -> Option<usize> {
        if self.coeffs.is_empty() {
            None
        } else {
            Some(self.coeffs.len() - 1)
        }
    }

    /// Leading coefficient, or `0` for the zero polynomial.
    fn leading(&self) -> u16 {
        self.coeffs.last().copied().unwrap_or(0)
    }

    /// Coefficient of `x^i` (0 beyond the stored range).
    fn coeff(&self, i: usize) -> u16 {
        self.coeffs.get(i).copied().unwrap_or(0)
    }

    /// Evaluate at `x` via Horner's rule.
    pub(crate) fn eval(&self, x: u16) -> u16 {
        let mut acc = 0u16;
        for &c in self.coeffs.iter().rev() {
            acc = field::add(field::mul(acc, x), c);
        }
        acc
    }

    /// Polynomial addition (= subtraction, characteristic 2).
    pub(crate) fn add(&self, other: &Poly) -> Poly {
        let n = self.coeffs.len().max(other.coeffs.len());
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            out.push(field::add(self.coeff(i), other.coeff(i)));
        }
        Poly::from_coeffs(out)
    }

    /// Polynomial multiplication (schoolbook; degrees here are ≤ ~2047).
    pub(crate) fn mul(&self, other: &Poly) -> Poly {
        if self.is_zero() || other.is_zero() {
            return Poly::zero();
        }
        let mut out = vec![0u16; self.coeffs.len() + other.coeffs.len() - 1];
        for (i, &a) in self.coeffs.iter().enumerate() {
            if a == 0 {
                continue;
            }
            for (j, &b) in other.coeffs.iter().enumerate() {
                out[i + j] = field::add(out[i + j], field::mul(a, b));
            }
        }
        Poly::from_coeffs(out)
    }

    /// Divide `self` by `divisor`, returning `(quotient, remainder)` with
    /// `self = quotient*divisor + remainder` and `deg(remainder) < deg(divisor)`.
    ///
    /// `divisor` MUST be non-zero (the only caller, the partial-GCD loop and
    /// the exact-division check, never passes a zero divisor — guarded there).
    pub(crate) fn divmod(&self, divisor: &Poly) -> (Poly, Poly) {
        debug_assert!(!divisor.is_zero(), "division by zero polynomial");
        if self.degree().unwrap_or(0) < divisor.degree().unwrap_or(0) || self.is_zero() {
            return (Poly::zero(), self.clone());
        }
        let d_deg = divisor.degree().unwrap();
        let d_lead_inv = field::inv(divisor.leading()).expect("non-zero leading coeff");

        let mut rem = self.coeffs.clone();
        let mut quot = vec![0u16; self.coeffs.len() - d_deg];

        let mut r_deg = self.coeffs.len() - 1;
        loop {
            // Trim leading zeros of the working remainder.
            while r_deg + 1 > 0 && rem.get(r_deg).copied().unwrap_or(0) == 0 {
                if r_deg == 0 {
                    break;
                }
                r_deg -= 1;
            }
            if rem.get(r_deg).copied().unwrap_or(0) == 0 || r_deg < d_deg {
                break;
            }
            // factor = rem[r_deg] / divisor.leading() at degree (r_deg - d_deg).
            let shift = r_deg - d_deg;
            let factor = field::mul(rem[r_deg], d_lead_inv);
            quot[shift] = factor;
            // rem -= factor * x^shift * divisor
            for j in 0..=d_deg {
                let sub = field::mul(factor, divisor.coeff(j));
                rem[shift + j] = field::add(rem[shift + j], sub);
            }
            if r_deg == 0 {
                break;
            }
            r_deg -= 1;
        }

        (Poly::from_coeffs(quot), Poly::from_coeffs(rem))
    }
}

/// Lagrange interpolation: the unique polynomial of degree `< points.len()`
/// passing through `(xᵢ, yᵢ)`. The `xᵢ` MUST be distinct (guaranteed by the
/// RS caller — the `βⱼ = α^j` are distinct for `j < 2047`).
///
/// **O(n²)** field-op cost: the master vanishing polynomial
/// `M(x) = ∏ⱼ (x - xⱼ)` is built once in O(n²); each basis numerator
/// `∏_{j≠i}(x - xⱼ) = M(x) / (x - xᵢ)` is recovered by a single linear
/// synthetic division in O(n); the denominator `∏_{j≠i}(xᵢ - xⱼ)` is the
/// derivative-free product `M'(xᵢ)` computed as the running product while
/// dividing. `n` here is ≤ 2047, well within the Word-Card sizes.
pub(crate) fn interpolate(xs: &[u16], ys: &[u16]) -> Poly {
    debug_assert_eq!(xs.len(), ys.len());
    let n = xs.len();
    if n == 0 {
        return Poly::zero();
    }

    // Master vanishing polynomial M(x) = ∏_j (x - x_j), degree n. In char 2,
    // (x - x_j) = (x + x_j) has coefficients [x_j, 1].
    let mut master = Poly::constant(1);
    for &xj in xs {
        master = master.mul(&Poly::from_coeffs(vec![xj, 1]));
    }

    let mut acc_coeffs = vec![0u16; n];
    for i in 0..n {
        // Basis numerator B_i(x) = M(x) / (x - x_i), via synthetic division by
        // the monic linear (x + x_i) [char 2]. master has degree n, so B_i has
        // degree n-1 (n coefficients). Synthetic division for monic (x - root):
        //   b_{deg-1} = m_deg; b_{t-1} = m_t + root·b_t  (downward).
        // In char 2 "+ root·b_t" is field::add(.., field::mul(root, b_t)).
        let m_coeffs = &master.coeffs; // length n+1
        let mut basis = vec![0u16; n]; // degrees 0..n-1
        let root = xs[i];
        // highest basis coeff = leading master coeff (== 1)
        basis[n - 1] = *m_coeffs.last().unwrap();
        for t in (0..n - 1).rev() {
            // m_{t+1} + root * basis[t+1]
            let m_next = m_coeffs.get(t + 1).copied().unwrap_or(0);
            basis[t] = field::add(m_next, field::mul(root, basis[t + 1]));
        }

        // Denominator ∏_{j≠i} (x_i - x_j) = B_i(x_i) (the synthetic-division
        // quotient evaluated at the removed root — exactly M'(x_i)).
        let mut den = 0u16;
        for &c in basis.iter().rev() {
            den = field::add(field::mul(den, root), c);
        }
        let den_inv = field::inv(den).expect("distinct interpolation nodes ⇒ den ≠ 0");
        let weight = field::mul(ys[i], den_inv);

        // acc += weight * B_i
        if weight != 0 {
            for (t, &b) in basis.iter().enumerate() {
                acc_coeffs[t] = field::add(acc_coeffs[t], field::mul(weight, b));
            }
        }
    }
    Poly::from_coeffs(acc_coeffs)
}

/// One step of the extended Euclidean algorithm on `(a, b)`, run until the
/// remainder degree drops **below** `deg_stop`. Returns `(r, v)` with the
/// invariant `u·a + v·b = r` for the returned remainder `r` and its `v`
/// multiplier (Gao's decoder only needs `r` and `v`).
///
/// Used by [`crate::rs`]'s Gao decoder: `a = g0 = ∏(x-βⱼ)`, `b = g1` (the
/// interpolant through the received points). Stopping at the first remainder
/// with `deg < deg_stop = (n_used + k)/2` yields `(r, v)` whose exact quotient
/// `r / v` is the corrected message polynomial.
pub(crate) fn partial_gcd(a: &Poly, b: &Poly, deg_stop: usize) -> (Poly, Poly) {
    // (r_prev, r_cur) with their v-multipliers (the coefficient on `b`).
    let mut r_prev = a.clone();
    let mut r_cur = b.clone();
    let mut v_prev = Poly::zero(); // v for a: u0·a + 0·b = a
    let mut v_cur = Poly::constant(1); // v for b: 0·a + 1·b = b

    // Stop as soon as the *current* remainder is already below the threshold
    // (covers the no-error case where g1 itself is the answer).
    while r_cur.degree().map(|d| d >= deg_stop).unwrap_or(false) {
        let (q, rem) = r_prev.divmod(&r_cur);
        // v_next = v_prev - q·v_cur  (= v_prev + q·v_cur in char 2)
        let v_next = v_prev.add(&q.mul(&v_cur));
        r_prev = r_cur;
        r_cur = rem;
        v_prev = v_cur;
        v_cur = v_next;
        // If the remainder became zero before crossing the threshold, stop.
        if r_cur.is_zero() {
            break;
        }
    }
    (r_cur, v_cur)
}
