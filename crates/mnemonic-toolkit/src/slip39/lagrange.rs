//! Lagrange interpolation over GF(2^8) at point x=0.
//!
//! Per SLIP-0039 §"Polynomial Interpolation": given a set of points
//! `(x_i, y_i)` representing a polynomial of degree `threshold - 1`,
//! the value at any point `x*` (in particular x*=0 to recover the
//! original secret) is reconstructed via the Lagrange basis:
//!
//! ```text
//! f(x*) = Σ_i y_i * Π_{j≠i} (x* - x_j) / (x_i - x_j)
//! ```
//!
//! In GF(256), `-x = x` (characteristic 2), so the formula simplifies
//! at x*=0 to:
//!
//! ```text
//! f(0) = Σ_i y_i * Π_{j≠i} x_j / (x_i XOR x_j)
//! ```
//!
//! For SLIP-39 each share has multiple bytes; interpolation is per-byte
//! position independently (each byte slot is its own Shamir polynomial).
//! See `interpolate_secret_at_zero` for the multi-byte entry point.

use crate::slip39::gf256;

/// Interpolate a single byte's polynomial at x=0 given a set of
/// `(x_i, y_i)` evaluation points.
///
/// PANICS if any two `x_i` are equal (caller's responsibility to dedup
/// share indices; matches SLIP-39's "duplicate member index" refusal
/// class — caught at the share-validation layer, not here).
pub fn interpolate_at_zero(points: &[(u8, u8)]) -> u8 {
    let mut result: u8 = 0;
    for (i, &(xi, yi)) in points.iter().enumerate() {
        // Compute Lagrange basis L_i(0) = Π_{j≠i} x_j / (x_i XOR x_j)
        let mut num: u8 = 1;
        let mut den: u8 = 1;
        for (j, &(xj, _)) in points.iter().enumerate() {
            if i == j {
                continue;
            }
            assert!(
                xi != xj,
                "lagrange: duplicate x-coordinate {} (caller must dedup)",
                xi,
            );
            num = gf256::mul(num, xj);
            den = gf256::mul(den, gf256::add(xi, xj));
        }
        let basis = gf256::div(num, den);
        // Accumulate: result += y_i * L_i(0)
        result = gf256::add(result, gf256::mul(yi, basis));
    }
    result
}

/// Interpolate a multi-byte secret at x=0. Each point's `y` is a byte
/// slice; the per-byte interpolation runs `y.len()` times independently.
///
/// All `y` slices must have the same length (caller validates).
///
/// Returns `Vec<u8>` of length `points[0].1.len()`. Caller wraps in
/// `Zeroizing<Vec<u8>>` at the boundary.
pub fn interpolate_secret_at_zero(points: &[(u8, &[u8])]) -> Vec<u8> {
    assert!(!points.is_empty(), "lagrange: empty point set");
    let secret_len = points[0].1.len();
    for (i, (_, y)) in points.iter().enumerate() {
        assert_eq!(
            y.len(),
            secret_len,
            "lagrange: share {} has length {} but expected {}",
            i,
            y.len(),
            secret_len,
        );
    }

    let mut secret = Vec::with_capacity(secret_len);
    for byte_idx in 0..secret_len {
        let per_byte_points: Vec<(u8, u8)> =
            points.iter().map(|&(x, y)| (x, y[byte_idx])).collect();
        secret.push(interpolate_at_zero(&per_byte_points));
    }
    secret
}
