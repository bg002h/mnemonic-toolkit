//! Lagrange interpolation over GF(2^8).
//!
//! Per SLIP-0039 §"Polynomial Interpolation": given a set of points
//! `(x_i, y_i)` representing a polynomial of degree `threshold − 1`,
//! the value at any point `x*` is reconstructed via the Lagrange basis:
//!
//! ```text
//! f(x*) = Σ_i y_i * Π_{j≠i} (x* − x_j) / (x_i − x_j)
//! ```
//!
//! In GF(256) with characteristic 2, `−x = x` and subtraction collapses
//! to XOR:
//!
//! ```text
//! f(x*) = Σ_i y_i * Π_{j≠i} (x* XOR x_j) / (x_i XOR x_j)
//! ```
//!
//! SLIP-39 evaluates at three distinct `x*` values:
//!   - `x* = 255` (`SECRET_INDEX`) — reconstruct the master secret.
//!   - `x* = 254` (`DIGEST_INDEX`) — reconstruct the digest payload.
//!   - `x* = i` (member share index, 0..=15) — synthesize a new share
//!     during split from the (threshold) base shares at `SECRET_INDEX`,
//!     `DIGEST_INDEX`, and the random shares.
//!
//! For SLIP-39 each share has multiple bytes; interpolation is per-byte
//! position independently (each byte slot is its own Shamir polynomial).
//! See [`interpolate_secret_at`] for the multi-byte entry point.

use crate::slip39::gf256;

/// Interpolate a single byte's polynomial at evaluation point `x` given
/// a set of `(x_i, y_i)` points.
///
/// PANICS if any two `x_i` are equal (caller's responsibility to dedup
/// share indices; matches SLIP-39's "duplicate member index" refusal
/// class — caught at the share-validation layer, not here).
pub fn interpolate_at(points: &[(u8, u8)], x: u8) -> u8 {
    let mut result: u8 = 0;
    for (i, &(xi, yi)) in points.iter().enumerate() {
        // Compute Lagrange basis L_i(x) = Π_{j≠i} (x XOR x_j) / (x_i XOR x_j).
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
            num = gf256::mul(num, gf256::add(x, xj));
            den = gf256::mul(den, gf256::add(xi, xj));
        }
        let basis = gf256::div(num, den);
        // Accumulate: result += y_i * L_i(x)
        result = gf256::add(result, gf256::mul(yi, basis));
    }
    result
}

/// Interpolate a multi-byte secret at evaluation point `x`. Each point's
/// `y` is a byte slice; the per-byte interpolation runs `y.len()` times
/// independently.
///
/// All `y` slices must have the same length (caller validates).
///
/// Returns `Vec<u8>` of length `points[0].1.len()`. Caller wraps in
/// `Zeroizing<Vec<u8>>` at the boundary.
pub fn interpolate_secret_at(points: &[(u8, &[u8])], x: u8) -> Vec<u8> {
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
            points.iter().map(|&(xc, y)| (xc, y[byte_idx])).collect();
        secret.push(interpolate_at(&per_byte_points, x));
    }
    secret
}
