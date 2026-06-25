//! KATs for the systematic **evaluation-form** Reed–Solomon engine (plan §3 /
//! §4.1, P2). The wire-frozen contract is the *codeword* (data verbatim at
//! `β₀..β_{k-1}`, parity `P(β_{k+i})`); the decode algorithm (Gao partial-GCD)
//! is NOT wire-frozen and is exercised here only for correctness.
//!
//! Load-bearing tests:
//! - **round_trip_no_errors** — `decode(codeword(data,m), k, []) == data`;
//! - **append_only_prefix** — `parity(data,m)[..m'] == parity(data,m')` for
//!   `m' < m` (the prefix-extensibility guarantee, plan §3);
//! - **correct_floor_m_over_2_errors** (proptest) — exactly `⌊m/2⌋` random
//!   substitutions are ALWAYS corrected exactly.

use proptest::prelude::*;
use wc_codec::rs::{rs_codeword, rs_decode, rs_parity, RsError};

const FIELD_LIMIT: u16 = 2048; // valid symbols are 0..2047

// ---------------------------------------------------------------------------
// 1. Round-trip (no errors) across small/edge and realistic (k, m).
// ---------------------------------------------------------------------------

/// Deterministic pseudo-random data of length `k`, all symbols in 0..2047.
fn det_data(k: usize, seed: u64) -> Vec<u16> {
    let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
    (0..k)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s % FIELD_LIMIT as u64) as u16
        })
        .collect()
}

#[test]
fn round_trip_no_errors() {
    // (k, m) pairs: edges (k=1, m=0/1), small, and a realistic mk1-ish shape.
    let cases: &[(usize, usize)] = &[
        (1, 0),
        (1, 1),
        (1, 4),
        (2, 0),
        (2, 1),
        (3, 2),
        (5, 0),
        (5, 4),
        (16, 8),
        (58, 8),   // canonical mk1 shape (plan §4.1)
        (160, 30), // realistic large
        (200, 47),
    ];
    for &(k, m) in cases {
        let data = det_data(k, (k * 131 + m) as u64);
        let cw = rs_codeword(&data, m).expect("codeword");
        assert_eq!(cw.len(), k + m, "codeword length (k={k}, m={m})");
        let got = rs_decode(&cw, k, &[]).expect("decode clean");
        assert_eq!(got, data, "round-trip (k={k}, m={m})");
    }
}

// ---------------------------------------------------------------------------
// 2. Systematic check — data appears verbatim in the first k positions.
// ---------------------------------------------------------------------------

#[test]
fn systematic_data_verbatim() {
    for &(k, m) in &[(1usize, 4usize), (5, 3), (16, 8), (58, 8), (100, 20)] {
        let data = det_data(k, 7 * k as u64 + m as u64);
        let cw = rs_codeword(&data, m).expect("codeword");
        assert_eq!(
            &cw[..k],
            data.as_slice(),
            "systematic prefix (k={k}, m={m})"
        );
    }
}

// ---------------------------------------------------------------------------
// 3. Append-only prefix (LOAD-BEARING) — first m' parity identical for m' < m.
// ---------------------------------------------------------------------------

#[test]
fn append_only_prefix() {
    for &k in &[1usize, 2, 5, 16, 58, 160] {
        let data = det_data(k, 0xABCD ^ k as u64);
        let m_big = 40usize;
        let big = rs_parity(&data, m_big).expect("big parity");
        // Several small m' < m_big.
        for &m_small in &[0usize, 1, 2, 7, 13, 30, 39] {
            let small = rs_parity(&data, m_small).expect("small parity");
            assert_eq!(small.len(), m_small, "parity len (k={k}, m'={m_small})");
            assert_eq!(
                &big[..m_small],
                small.as_slice(),
                "append-only prefix (k={k}, m'={m_small} vs m={m_big})"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Cross-check vs P1 field — every parity symbol is a valid field element.
// ---------------------------------------------------------------------------

#[test]
fn parity_symbols_in_field() {
    for &(k, m) in &[(1usize, 5usize), (16, 8), (58, 30), (160, 47)] {
        let data = det_data(k, 3 + k as u64);
        let parity = rs_parity(&data, m).expect("parity");
        for (i, &p) in parity.iter().enumerate() {
            assert!(
                p < FIELD_LIMIT,
                "parity[{i}] = {p} out of field (k={k}, m={m})"
            );
        }
        let cw = rs_codeword(&data, m).expect("codeword");
        for (i, &c) in cw.iter().enumerate() {
            assert!(
                c < FIELD_LIMIT,
                "codeword[{i}] = {c} out of field (k={k}, m={m})"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Refuse / no-panic — bad inputs, out-of-range, oversize n, erasure errors.
// ---------------------------------------------------------------------------

#[test]
fn reject_symbol_out_of_range_encode() {
    let mut data = det_data(5, 1);
    data[2] = 2048; // out of GF(2^11)
    assert!(matches!(
        rs_parity(&data, 4),
        Err(RsError::SymbolOutOfRange { .. })
    ));
    assert!(matches!(
        rs_codeword(&data, 4),
        Err(RsError::SymbolOutOfRange { .. })
    ));
}

#[test]
fn reject_oversize_codeword() {
    // n = k + m > 2047 must be rejected at encode time.
    let data = vec![0u16; 2040];
    assert!(matches!(
        rs_parity(&data, 8),
        Err(RsError::LengthExceedsField { .. })
    ));
    // exactly 2047 is OK length-wise.
    assert!(rs_parity(&vec![1u16; 2000], 47).is_ok());
}

#[test]
fn decode_rejects_bad_shapes() {
    let data = det_data(10, 9);
    let cw = rs_codeword(&data, 6).expect("cw");

    // data_len longer than codeword.
    assert!(rs_decode(&cw, cw.len() + 1, &[]).is_err());
    // data_len == 0 with a non-empty codeword is degenerate — must not panic.
    let _ = rs_decode(&cw, 0, &[]);

    // out-of-range symbol in the codeword.
    let mut bad = cw.clone();
    bad[3] = 5000;
    assert!(matches!(
        rs_decode(&bad, data.len(), &[]),
        Err(RsError::SymbolOutOfRange { .. })
    ));

    // erasure index out of range.
    assert!(matches!(
        rs_decode(&cw, data.len(), &[cw.len()]),
        Err(RsError::ErasureOutOfRange { .. })
    ));
    // erasures not sorted / not distinct.
    assert!(rs_decode(&cw, data.len(), &[2, 2]).is_err());
    assert!(rs_decode(&cw, data.len(), &[5, 1]).is_err());
}

#[test]
fn decode_refuses_when_all_erased_underdetermined() {
    // Erase MORE than m positions with zero correctable budget -> Err, no panic.
    let data = det_data(8, 4);
    let m = 4;
    let cw = rs_codeword(&data, m).expect("cw");
    let erasures: Vec<usize> = (0..=m).collect(); // m+1 erasures
    assert!(matches!(
        rs_decode(&cw, data.len(), &erasures),
        Err(RsError::Uncorrectable)
    ));
}

// ---------------------------------------------------------------------------
// 4. Correct ⌊m/2⌋ errors (proptest) — exact, every time.
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    #[test]
    fn correct_floor_m_over_2_errors(
        k in 1usize..40,
        m in 1usize..30,
        seed in any::<u64>(),
    ) {
        let data = det_data(k, seed);
        let cw = rs_codeword(&data, m).unwrap();
        let n = cw.len();
        let t = m / 2; // ⌊m/2⌋ errors, the full error-only budget
        prop_assume!(t >= 1);

        // Choose t distinct positions deterministically from the seed.
        let mut positions: Vec<usize> = (0..n).collect();
        // Fisher-Yates with a seed-derived PRNG.
        let mut s = seed ^ 0xDEAD_BEEF_CAFE_F00D;
        for i in (1..n).rev() {
            s ^= s << 13; s ^= s >> 7; s ^= s << 17;
            let j = (s % (i as u64 + 1)) as usize;
            positions.swap(i, j);
        }
        let err_pos = &positions[..t];

        let mut recv = cw.clone();
        for &p in err_pos {
            // a wrong value distinct from the true symbol
            s ^= s << 13; s ^= s >> 7; s ^= s << 17;
            let mut v = (s % FIELD_LIMIT as u64) as u16;
            if v == cw[p] { v = (v + 1) % FIELD_LIMIT; }
            recv[p] = v;
        }

        let got = rs_decode(&recv, k, &[]).expect("decode within ⌊m/2⌋ budget");
        prop_assert_eq!(got, data, "k={}, m={}, t={}", k, m, t);
    }
}

// ---------------------------------------------------------------------------
// 5. Recover m erasures (proptest) — up to m exact; m+1 -> Err.
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn recover_up_to_m_erasures(
        k in 1usize..40,
        m in 1usize..30,
        seed in any::<u64>(),
    ) {
        let data = det_data(k, seed);
        let cw = rs_codeword(&data, m).unwrap();
        let n = cw.len();

        // erase exactly s = min(m, n) positions (sorted distinct).
        let s = m.min(n);
        let mut positions: Vec<usize> = (0..n).collect();
        let mut st = seed ^ 0x1234_5678_9ABC_DEF0;
        for i in (1..n).rev() {
            st ^= st << 13; st ^= st >> 7; st ^= st << 17;
            let j = (st % (i as u64 + 1)) as usize;
            positions.swap(i, j);
        }
        let mut er: Vec<usize> = positions[..s].to_vec();
        er.sort_unstable();

        let mut recv = cw.clone();
        for &p in &er {
            st ^= st << 13; st ^= st >> 7; st ^= st << 17;
            recv[p] = (st % FIELD_LIMIT as u64) as u16; // arbitrary garbage
        }

        let got = rs_decode(&recv, k, &er).expect("decode up to m erasures");
        prop_assert_eq!(got, data, "k={}, m={}, s={}", k, m, s);

        // m+1 erasures (when there is room) must refuse.
        if n > m {
            let mut er2: Vec<usize> = positions[..m + 1].to_vec();
            er2.sort_unstable();
            let mut recv2 = cw.clone();
            for &p in &er2 {
                st ^= st << 13; st ^= st >> 7; st ^= st << 17;
                recv2[p] = (st % FIELD_LIMIT as u64) as u16;
            }
            prop_assert!(
                matches!(rs_decode(&recv2, k, &er2), Err(RsError::Uncorrectable)),
                "m+1 erasures must be Uncorrectable (k={}, m={})", k, m
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Mixed budget (proptest) — 2t + s ≤ m exact; 2t + s = m+1 never panics.
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    #[test]
    fn mixed_errors_and_erasures(
        k in 2usize..36,
        m in 2usize..28,
        seed in any::<u64>(),
        t_raw in 0usize..14,
    ) {
        let data = det_data(k, seed);
        let cw = rs_codeword(&data, m).unwrap();
        let n = cw.len();

        // Pick t in 0..=⌊m/2⌋ and fill the rest of the budget with erasures so
        // 2t + s == m exactly (the boundary of the budget).
        let t = t_raw.min(m / 2);
        let s = m - 2 * t;
        prop_assume!(t + s <= n); // need that many distinct positions

        // shuffle a position pool
        let mut pool: Vec<usize> = (0..n).collect();
        let mut st = seed ^ 0xA5A5_5A5A_F0F0_0F0F;
        for i in (1..n).rev() {
            st ^= st << 13; st ^= st >> 7; st ^= st << 17;
            let j = (st % (i as u64 + 1)) as usize;
            pool.swap(i, j);
        }
        let err_pos: Vec<usize> = pool[..t].to_vec();
        let mut er: Vec<usize> = pool[t..t + s].to_vec();
        er.sort_unstable();

        let mut recv = cw.clone();
        // errors: wrong values, NOT in the erasure set
        for &p in &err_pos {
            st ^= st << 13; st ^= st >> 7; st ^= st << 17;
            let mut v = (st % FIELD_LIMIT as u64) as u16;
            if v == cw[p] { v = (v + 1) % FIELD_LIMIT; }
            recv[p] = v;
        }
        // erasures: arbitrary garbage, flagged as known-bad
        for &p in &er {
            st ^= st << 13; st ^= st >> 7; st ^= st << 17;
            recv[p] = (st % FIELD_LIMIT as u64) as u16;
        }

        // At the budget boundary 2t + s == m: MUST be exact.
        let got = rs_decode(&recv, k, &er).expect("within budget 2t+s==m");
        prop_assert_eq!(got, data, "boundary 2t+s==m (k={}, m={}, t={}, s={})", k, m, t, s);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// Beyond budget: 2t + s = m + 1 must NEVER panic (Err or a wrong-but-valid
    /// answer are both acceptable; correctness is NOT asserted past budget).
    #[test]
    fn beyond_budget_never_panics(
        k in 2usize..30,
        m in 2usize..24,
        seed in any::<u64>(),
        t_raw in 0usize..12,
    ) {
        let data = det_data(k, seed);
        let cw = rs_codeword(&data, m).unwrap();
        let n = cw.len();

        // 2t + s = m + 1
        let t = t_raw.min((m + 1) / 2);
        if 2 * t > m + 1 { return Ok(()); }
        let s = (m + 1) - 2 * t;
        prop_assume!(t + s <= n);

        let mut pool: Vec<usize> = (0..n).collect();
        let mut st = seed ^ 0x0F0F_F0F0_5A5A_A5A5;
        for i in (1..n).rev() {
            st ^= st << 13; st ^= st >> 7; st ^= st << 17;
            let j = (st % (i as u64 + 1)) as usize;
            pool.swap(i, j);
        }
        let err_pos: Vec<usize> = pool[..t].to_vec();
        let mut er: Vec<usize> = pool[t..t + s].to_vec();
        er.sort_unstable();

        let mut recv = cw.clone();
        for &p in &err_pos {
            st ^= st << 13; st ^= st >> 7; st ^= st << 17;
            let mut v = (st % FIELD_LIMIT as u64) as u16;
            if v == cw[p] { v = (v + 1) % FIELD_LIMIT; }
            recv[p] = v;
        }
        for &p in &er {
            st ^= st << 13; st ^= st >> 7; st ^= st << 17;
            recv[p] = (st % FIELD_LIMIT as u64) as u16;
        }

        // Only requirement: does not panic, returns a typed result.
        let _ = rs_decode(&recv, k, &er);
    }
}
