//! v0.13.0 P1a — library tests for SLIP-39 math primitives.
//!
//! Per SPEC §4 G1 (Lagrange behavior verified by spec test vectors at
//! P1c) + field-axiom property tests at this layer.

use mnemonic_toolkit::slip39::{gf256, lagrange};

// ============================================================================
// GF(256) field axiom property tests
// ============================================================================

#[test]
fn add_is_xor() {
    // Field add in characteristic-2 GF(256) IS XOR.
    for a in 0u8..=255 {
        for b in 0u8..=255 {
            assert_eq!(gf256::add(a, b), a ^ b);
        }
    }
}

#[test]
fn add_identity_zero() {
    for a in 0u8..=255 {
        assert_eq!(gf256::add(a, 0), a);
        assert_eq!(gf256::add(0, a), a);
    }
}

#[test]
fn add_self_inverse() {
    // a + a = 0 in characteristic 2.
    for a in 0u8..=255 {
        assert_eq!(gf256::add(a, a), 0);
    }
}

#[test]
fn add_commutative() {
    for a in 0u8..=255 {
        for b in 0u8..=255 {
            assert_eq!(gf256::add(a, b), gf256::add(b, a));
        }
    }
}

#[test]
fn add_associative_sample() {
    // Sample triples — full 256³ ≈ 16M is excessive.
    for a in (0..=255).step_by(17) {
        for b in (0..=255).step_by(19) {
            for c in (0..=255).step_by(23) {
                let lhs = gf256::add(gf256::add(a, b), c);
                let rhs = gf256::add(a, gf256::add(b, c));
                assert_eq!(lhs, rhs, "associativity at ({a},{b},{c})");
            }
        }
    }
}

// ----- mul -----

#[test]
fn mul_by_zero() {
    for a in 0u8..=255 {
        assert_eq!(gf256::mul(a, 0), 0, "a * 0 = 0 at a={a}");
        assert_eq!(gf256::mul(0, a), 0, "0 * a = 0 at a={a}");
    }
}

#[test]
fn mul_by_one() {
    for a in 0u8..=255 {
        assert_eq!(gf256::mul(a, 1), a, "a * 1 = a at a={a}");
        assert_eq!(gf256::mul(1, a), a, "1 * a = a at a={a}");
    }
}

#[test]
fn mul_commutative_sample() {
    for a in (1..=255).step_by(7) {
        for b in (1..=255).step_by(11) {
            assert_eq!(gf256::mul(a, b), gf256::mul(b, a));
        }
    }
}

#[test]
fn mul_associative_sample() {
    for a in (1..=255).step_by(13) {
        for b in (1..=255).step_by(17) {
            for c in (1..=255).step_by(19) {
                let lhs = gf256::mul(gf256::mul(a, b), c);
                let rhs = gf256::mul(a, gf256::mul(b, c));
                assert_eq!(lhs, rhs, "mul-associativity at ({a},{b},{c})");
            }
        }
    }
}

#[test]
fn distributivity_sample() {
    // a * (b + c) = a*b + a*c
    for a in (1..=255).step_by(13) {
        for b in (0..=255).step_by(17) {
            for c in (0..=255).step_by(19) {
                let lhs = gf256::mul(a, gf256::add(b, c));
                let rhs = gf256::add(gf256::mul(a, b), gf256::mul(a, c));
                assert_eq!(lhs, rhs, "distributivity at ({a},{b},{c})");
            }
        }
    }
}

// ----- inv -----

#[test]
fn inv_round_trip() {
    // inv(inv(a)) == a for all non-zero a.
    for a in 1u8..=255 {
        let ai = gf256::inv(a);
        assert_eq!(gf256::inv(ai), a, "inv-round-trip at a={a}");
    }
}

#[test]
fn mul_by_inv_yields_one() {
    for a in 1u8..=255 {
        let ai = gf256::inv(a);
        assert_eq!(gf256::mul(a, ai), 1, "a * inv(a) = 1 at a={a}");
        assert_eq!(gf256::mul(ai, a), 1, "inv(a) * a = 1 at a={a}");
    }
}

#[test]
#[should_panic]
fn inv_of_zero_panics() {
    let _ = gf256::inv(0);
}

// ----- div -----

#[test]
fn div_by_one_is_identity() {
    for a in 0u8..=255 {
        assert_eq!(gf256::div(a, 1), a);
    }
}

#[test]
fn div_then_mul_round_trip() {
    // (a / b) * b == a for b != 0.
    for a in (0..=255).step_by(7) {
        for b in (1..=255).step_by(11) {
            let q = gf256::div(a, b);
            assert_eq!(gf256::mul(q, b), a, "div-mul round-trip at ({a},{b})");
        }
    }
}

#[test]
#[should_panic]
fn div_by_zero_panics() {
    let _ = gf256::div(5, 0);
}

// ----- constants -----

#[test]
fn reduction_polynomial_is_rijndael() {
    assert_eq!(gf256::REDUCTION_POLY, 0x11b);
}

#[test]
fn generator_is_three() {
    assert_eq!(gf256::GENERATOR, 3);
}

// ============================================================================
// Lagrange interpolation tests
// ============================================================================

#[test]
fn lagrange_interpolate_single_point_recovers_constant() {
    // Polynomial of degree 0: f(x) = c. Any single point recovers c.
    assert_eq!(lagrange::interpolate_at_zero(&[(5, 42)]), 42);
    assert_eq!(lagrange::interpolate_at_zero(&[(99, 0)]), 0);
    assert_eq!(lagrange::interpolate_at_zero(&[(1, 255)]), 255);
}

#[test]
fn lagrange_interpolate_two_points_recovers_linear() {
    // Polynomial f(x) = a + b*x in GF(256). f(0) = a.
    // Compute f(1) + f(2) via the GF primitives, then verify interpolation
    // recovers a.
    let f_at_1 = gf256::add(17, gf256::mul(42, 1));
    let f_at_2 = gf256::add(17, gf256::mul(42, 2));
    let recovered = lagrange::interpolate_at_zero(&[(1, f_at_1), (2, f_at_2)]);
    assert_eq!(recovered, 17, "linear interp at x=0 recovers constant term");
}

#[test]
fn lagrange_interpolate_three_points_recovers_quadratic() {
    // Polynomial f(x) = a + b*x + c*x². f(0) = a.
    let a = 100u8;
    let b = 7u8;
    let c = 3u8;
    let f_at = |x: u8| -> u8 {
        let bx = gf256::mul(b, x);
        let cxx = gf256::mul(c, gf256::mul(x, x));
        gf256::add(gf256::add(a, bx), cxx)
    };
    let pts = [(1u8, f_at(1)), (2u8, f_at(2)), (3u8, f_at(3))];
    let recovered = lagrange::interpolate_at_zero(&pts);
    assert_eq!(recovered, a, "quadratic interp at x=0 recovers constant term");
}

#[test]
fn lagrange_multi_byte_recovers_secret() {
    // 16-byte secret, 2-of-N threshold (degree-1 polynomial per byte).
    // Synthesize shares manually: pick a "b" coefficient per byte slot,
    // then sample the polynomial at x=1 and x=2.
    let secret = [0xAAu8; 16];
    let b_coeffs = [0x33u8; 16];

    let make_share = |x: u8| -> Vec<u8> {
        (0..16)
            .map(|i| gf256::add(secret[i], gf256::mul(b_coeffs[i], x)))
            .collect()
    };

    let s1 = make_share(1);
    let s2 = make_share(2);
    let pts: Vec<(u8, &[u8])> = vec![(1, &s1), (2, &s2)];
    let recovered = lagrange::interpolate_secret_at_zero(&pts);
    assert_eq!(recovered.as_slice(), &secret, "2-share recovery byte-equal");
}

#[test]
fn lagrange_two_of_three_recovery_any_pair() {
    // 32-byte secret, 2-of-3 threshold; verify ANY 2 of 3 shares recovers.
    let secret = [0x5Au8; 32];
    let b_coeffs = [0xC3u8; 32];

    let make_share = |x: u8| -> Vec<u8> {
        (0..32)
            .map(|i| gf256::add(secret[i], gf256::mul(b_coeffs[i], x)))
            .collect()
    };

    let s1 = make_share(1);
    let s2 = make_share(2);
    let s3 = make_share(3);

    let pairs: Vec<Vec<(u8, &[u8])>> = vec![
        vec![(1, &s1[..]), (2, &s2[..])],
        vec![(1, &s1[..]), (3, &s3[..])],
        vec![(2, &s2[..]), (3, &s3[..])],
    ];
    for (i, pair) in pairs.iter().enumerate() {
        let recovered = lagrange::interpolate_secret_at_zero(pair);
        assert_eq!(
            recovered.as_slice(),
            &secret,
            "2-of-3 recovery via pair {i} must yield original secret",
        );
    }
}
