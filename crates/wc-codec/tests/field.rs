//! KATs for the `GF(2^11)` field (plan §3). The **primitivity** and **full-orbit**
//! tests are load-bearing: they prove `α` generates the whole multiplicative
//! group, which every downstream RS/RAID guarantee relies on.

use wc_codec::field::{add, inv, mul, pow, sub, ALPHA, MODULUS, MULTIPLICATIVE_ORDER, ORDER};

const ELEM_MASK: u16 = 0x07FF;

#[test]
fn frozen_constants() {
    // Plan §3: p(x) = x^11 + x^2 + 1 = 0x805; α = x = 0x002.
    assert_eq!(MODULUS, 0x805, "primitive polynomial constant");
    assert_eq!(ALPHA, 0x002, "primitive element");
    assert_eq!(ORDER, 2048, "field order 2^11");
    assert_eq!(MULTIPLICATIVE_ORDER, 2047, "group order 2047 = 23·89");
}

/// Load-bearing primitivity KAT (plan §3): ord(α) = 2047 = 23·89, whose only
/// proper divisors are 23 and 89. α^2047 = 1 AND α^23 ≠ 1 AND α^89 ≠ 1 proves the
/// order is exactly 2047 (a primitive element).
#[test]
fn alpha_is_primitive() {
    assert_eq!(pow(ALPHA, 2047), 1, "α^2047 must be 1");
    assert_ne!(pow(ALPHA, 23), 1, "α^23 must NOT be 1");
    assert_ne!(pow(ALPHA, 89), 1, "α^89 must NOT be 1");
}

/// Full-orbit KAT: the powers α^0 .. α^2046 hit every one of the 2047 non-zero
/// elements exactly once (a permutation of GF(2^11)^×), and α^2047 wraps to α^0.
#[test]
fn alpha_full_orbit() {
    let mut seen = vec![false; ORDER as usize];
    let mut x: u16 = 1; // α^0
    for i in 0..MULTIPLICATIVE_ORDER as usize {
        assert!(x != 0, "orbit hit zero at step {i} (α not primitive)");
        assert!(!seen[x as usize], "orbit repeated value {x} at step {i}");
        seen[x as usize] = true;
        x = mul(x, ALPHA);
    }
    // After 2047 multiplications we are back to α^0 = 1.
    assert_eq!(x, 1, "α^2047 must wrap to 1");
    // Every non-zero element was visited exactly once; zero never was.
    assert!(!seen[0], "zero must not appear in the orbit");
    for (v, &hit) in seen.iter().enumerate().skip(1) {
        assert!(hit, "non-zero element {v} was never visited");
    }
}

/// `inv` is correct for ALL 2047 non-zero elements: a · inv(a) = 1; and inv(0) is
/// None.
#[test]
fn inverse_all_nonzero() {
    assert_eq!(inv(0), None, "zero has no inverse");
    for a in 1..ORDER {
        let ai = inv(a).unwrap_or_else(|| panic!("inv({a}) returned None"));
        assert_eq!(mul(a, ai), 1, "a·inv(a) must be 1 for a={a}");
    }
}

#[test]
fn mul_identity_and_zero() {
    for a in 0..ORDER {
        assert_eq!(mul(a, 1), a, "1 is the multiplicative identity (a={a})");
        assert_eq!(mul(1, a), a, "1 is the multiplicative identity (a={a})");
        assert_eq!(mul(a, 0), 0, "mul(a,0) must be 0 (a={a})");
        assert_eq!(mul(0, a), 0, "mul(0,a) must be 0 (a={a})");
    }
}

#[test]
fn mul_commutative_sample() {
    // A deterministic spread of sample pairs.
    let samples: [u16; 8] = [0, 1, 2, 7, 100, 1023, 1500, 2047];
    for &a in &samples {
        for &b in &samples {
            assert_eq!(mul(a, b), mul(b, a), "mul commutative (a={a}, b={b})");
        }
    }
}

#[test]
fn add_is_xor_and_self_inverse() {
    let samples: [u16; 8] = [0, 1, 2, 7, 100, 1023, 1500, 2047];
    for &a in &samples {
        for &b in &samples {
            assert_eq!(add(a, b), a ^ b, "add is XOR (a={a}, b={b})");
            assert_eq!(sub(a, b), a ^ b, "sub is XOR (a={a}, b={b})");
        }
        // self-inverse: a + a = 0
        assert_eq!(add(a, a), 0, "a+a must be 0 (a={a})");
    }
}

/// Distributivity on a sample: a·(b+c) = a·b + a·c.
#[test]
fn distributivity_sample() {
    let samples: [u16; 6] = [1, 2, 7, 100, 1023, 2047];
    for &a in &samples {
        for &b in &samples {
            for &c in &samples {
                let lhs = mul(a, add(b, c));
                let rhs = add(mul(a, b), mul(a, c));
                assert_eq!(lhs, rhs, "distributivity (a={a}, b={b}, c={c})");
            }
        }
    }
}

/// `pow` agrees with iterated `mul`, and pow(_, 0) = 1.
#[test]
fn pow_matches_iterated_mul() {
    for &base in &[2u16, 3, 7, 100, 1023, 2047] {
        assert_eq!(pow(base, 0), 1, "pow(base,0) must be 1");
        let mut acc: u16 = 1;
        for e in 0..50u32 {
            assert_eq!(pow(base, e), acc, "pow({base},{e}) mismatch");
            acc = mul(acc, base);
        }
    }
}

/// All field elements stay within the 11-bit range (no stray high bits leak).
#[test]
fn results_stay_in_range() {
    let samples: [u16; 6] = [1, 2, 7, 100, 1023, 2047];
    for &a in &samples {
        for &b in &samples {
            assert!(mul(a, b) <= ELEM_MASK, "mul out of range");
            assert!(add(a, b) <= ELEM_MASK, "add out of range");
        }
    }
}
