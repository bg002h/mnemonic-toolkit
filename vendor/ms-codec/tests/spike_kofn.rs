//! Phase-0 K-of-N spike — the hard gate for the ms v0.2 codex32 share design.
//!
//! This proves three load-bearing claims against the PINNED `codex32 = "=0.1.0"`
//! BEFORE any real code is written (CLAUDE.md mandatory-R0 / design-correctness
//! gate). If ANY assertion here fails, the design is wrong — STOP, do not patch
//! around it. The spike is KEPT as a permanent guard so a future codex32 bump
//! that breaks these invariants fails loudly.
//!
//! Claims:
//!   (a) ZERO byte-identity — `from_seed("ms", 0, "entr", Fe::S, [0x00]++ent)`
//!       round-trips for all five entr lengths. This is the construction
//!       `encode_shares(ZERO, 1)` reuses; it MUST stay byte-identical to v0.1.
//!   (b) K-of-N round-trip — secret-at-S + (k-1) random defining shares at
//!       distinct non-`s` indices recombine (any k of the n distributed shares,
//!       interpolated at Fe::S) back to the exact secret string + bytes. For
//!       entr AND mnem, all five entropy lengths, every k in 2..=9, with n=k and
//!       a wider n=k+2 case to exercise interpolation-derived distributed shares.
//!   (c) C1 index-`s` short-circuit — `interpolate_at` returns the secret share
//!       directly when index `s` is among the inputs, WITHOUT Lagrange-validating
//!       the other inputs. This justifies `combine_shares`' pre-reject of index-`s`.
//!
//! `Fe` non-`s` index construction: `Fe::from_char(c)` over a fixed pool of
//! bech32-alphabet chars (`qpzry9x8gf2tvdw0s3jn54khce6mua7l` minus `s`), taken in
//! a fixed canonical order. `from_char` returns `Result<Fe, ms_codec::codex32::Error>`.

use ms_codec::codex32::{Codex32String, Fe};

const HRP: &str = "ms";

/// The five BIP-39 entropy byte-lengths ms1 accepts.
const ENT_LENGTHS: [usize; 5] = [16, 20, 24, 28, 32];

/// Expected total ms1 string lengths per entropy length, for an entr secret/share
/// (prefix `[0x00]`): mirrors `consts::VALID_STR_LENGTHS`.
const ENTR_STR_LENGTHS: [usize; 5] = [50, 56, 62, 69, 75];

/// Expected total ms1 string lengths per entropy length, for a mnem secret/share
/// (prefix `[0x02][lang]`): mirrors `consts::VALID_MNEM_STR_LENGTHS`.
const MNEM_STR_LENGTHS: [usize; 5] = [51, 58, 64, 70, 77];

/// Fixed canonical pool of distinct non-`s` bech32-alphabet characters, in the
/// alphabet's own order with `s` removed. We pull share indices from the front of
/// this pool; with up to 9 shares (k <= 9, n <= k+2 = 11) we never run out.
const NON_S_CHARS: &[char] = &[
    'q', 'p', 'z', 'r', 'y', '9', 'x', '8', 'g', 'f', '2', 't', 'v', 'd', 'w', '0', '3', 'j', 'n',
    '5', '4', 'k', 'h', 'c', 'e', '6', 'm', 'u', 'a', '7', 'l',
];

/// Build the `i`-th non-`s` index as an `Fe` via `Fe::from_char`.
fn non_s_index(i: usize) -> Fe {
    Fe::from_char(NON_S_CHARS[i]).expect("pool char is a valid bech32 element")
}

/// (a) ZERO byte-identity: the `from_seed("ms", 0, "entr", Fe::S, [0x00]++ent)`
/// construction round-trips for all five entr lengths — `from_string` of its
/// own `.to_string()` recovers the exact `[0x00]++entropy` wire bytes.
#[test]
fn zero_share_is_byte_identical_to_single() {
    for &n in &ENT_LENGTHS {
        let mut data = vec![0x00u8];
        data.extend(std::iter::repeat_n(0xABu8, n));

        let single = Codex32String::from_seed(HRP, 0, "entr", Fe::S, &data)
            .expect("from_seed(threshold=0, index=S) must succeed for a v0.1 single");
        let s = single.to_string();

        // Re-parse the emitted string and confirm byte-identity of the payload.
        let reparsed =
            Codex32String::from_string(s.clone()).expect("the emitted v0.1 single must re-parse");
        assert_eq!(
            reparsed.parts().data(),
            data,
            "ZERO single must round-trip its [0x00]++entropy wire bytes (n_ent={n})"
        );

        // The construction is deterministic: a second build is byte-identical.
        let single2 = Codex32String::from_seed(HRP, 0, "entr", Fe::S, &data).unwrap();
        assert_eq!(
            s,
            single2.to_string(),
            "ZERO single construction must be deterministic (n_ent={n})"
        );
    }
}

/// Run one (kind, n_ent, k, n) round-trip case. Returns Err(msg) on the first
/// failed claim so the caller can STOP/BLOCK with the exact failing tuple.
fn run_case(prefix: &[u8], n_ent: usize, k: u8, n: usize) -> Result<(), String> {
    // 1. secret bytes = prefix ++ [0xCD; n_ent]; id = "tst7".
    let mut secret_bytes = prefix.to_vec();
    secret_bytes.extend(std::iter::repeat_n(0xCDu8, n_ent));
    let id = "tst7";

    // 2. secret-at-S.
    let secret_s = Codex32String::from_seed(HRP, k as usize, id, Fe::S, &secret_bytes)
        .map_err(|e| format!("from_seed(secret S) failed: {e:?}"))?;

    // 3. k-1 defining shares at distinct non-`s` indices, with filler payloads
    //    distinct from the secret (filler byte 0x10+j) but the SAME byte length.
    let mut defining: Vec<Codex32String> = vec![secret_s.clone()];
    for j in 0..(k as usize - 1) {
        let mut filler = prefix.to_vec();
        filler.extend(std::iter::repeat_n(0x10u8 + j as u8, n_ent));
        let share = Codex32String::from_seed(HRP, k as usize, id, non_s_index(j), &filler)
            .map_err(|e| format!("from_seed(defining {j}) failed: {e:?}"))?;
        defining.push(share);
    }
    // defining is now k points: [secret_s, def_1 .. def_{k-1}].

    // 4. Distributed shares = def_1..def_{k-1} PLUS interpolation-derived shares
    //    at fresh distinct non-`s` indices, until we have n distributed shares.
    //    (We never distribute secret_s itself.)
    let mut distributed: Vec<(Fe, Codex32String)> = Vec::with_capacity(n);
    for j in 0..(k as usize - 1) {
        distributed.push((non_s_index(j), defining[j + 1].clone()));
    }
    // Next free pool slot is index (k-1).
    let mut next_pool = k as usize - 1;
    while distributed.len() < n {
        let idx = non_s_index(next_pool);
        next_pool += 1;
        let derived = Codex32String::interpolate_at(&defining, idx)
            .map_err(|e| format!("interpolate_at(derive distributed @ {idx:?}) failed: {e:?}"))?;
        distributed.push((idx, derived));
    }
    if distributed.len() != n {
        return Err(format!(
            "expected {n} distributed shares, got {}",
            distributed.len()
        ));
    }

    // 6 (length): assert every distributed share string is the expected length.
    let len_set = if prefix[0] == 0x00 {
        ENTR_STR_LENGTHS
    } else {
        MNEM_STR_LENGTHS
    };
    let pos = ENT_LENGTHS.iter().position(|&e| e == n_ent).unwrap();
    let expected_len = len_set[pos];
    for (idx, share) in &distributed {
        let got = share.to_string().len();
        if got != expected_len {
            return Err(format!(
                "share @ {idx:?} length {got} != expected {expected_len} (n_ent={n_ent}, k={k})"
            ));
        }
    }
    // Also assert secret_s string length matches (same construction shape).
    if secret_s.to_string().len() != expected_len {
        return Err(format!(
            "secret_s length {} != expected {expected_len} (n_ent={n_ent}, k={k})",
            secret_s.to_string().len()
        ));
    }

    // 5 (recover): take k-subsets of the distributed shares, interpolate at S,
    //    and assert the recovered string AND wire bytes match the secret.
    //    Test at least two different k-subsets when n > k.
    let recover_and_check = |subset: &[Codex32String], label: &str| -> Result<(), String> {
        let recovered = Codex32String::interpolate_at(subset, Fe::S)
            .map_err(|e| format!("interpolate_at(recover S, {label}) failed: {e:?}"))?;
        if recovered.to_string() != secret_s.to_string() {
            return Err(format!(
                "recovered string != secret_s ({label}, n_ent={n_ent}, k={k}, n={n})"
            ));
        }
        if recovered.parts().data() != secret_bytes {
            return Err(format!(
                "recovered bytes != secret_bytes ({label}, n_ent={n_ent}, k={k}, n={n})"
            ));
        }
        Ok(())
    };

    let strings: Vec<Codex32String> = distributed.iter().map(|(_, s)| s.clone()).collect();

    // Subset A: the first k distributed shares.
    let subset_a: Vec<Codex32String> = strings[..k as usize].to_vec();
    recover_and_check(&subset_a, "first-k")?;

    // Subset B (only when n > k): the LAST k distributed shares — a different set.
    if n > k as usize {
        let subset_b: Vec<Codex32String> = strings[(n - k as usize)..].to_vec();
        recover_and_check(&subset_b, "last-k")?;
    }

    Ok(())
}

/// (b) K-of-N round-trip — the load-bearing gate. entr AND mnem, all five entropy
/// lengths, every k in 2..=9, n=k and (where n<=31) a wider n=k+2 case.
#[test]
fn kofn_round_trip_entr_and_mnem() {
    // entr prefix = [0x00]; mnem prefix = [0x02][lang=ja=1].
    let kinds: [(&str, Vec<u8>); 2] = [("entr", vec![0x00u8]), ("mnem(ja)", vec![0x02u8, 0x01u8])];

    let mut failures: Vec<String> = Vec::new();

    for (kind_name, prefix) in &kinds {
        for &n_ent in &ENT_LENGTHS {
            for k in 2u8..=9 {
                // n = k (the tight case).
                if let Err(e) = run_case(prefix, n_ent, k, k as usize) {
                    failures.push(format!("[{kind_name}] n=k case: {e}"));
                }
                // n = k+2 (interpolation-derived distributed shares), n <= 31.
                let wide_n = k as usize + 2;
                if wide_n <= 31 {
                    if let Err(e) = run_case(prefix, n_ent, k, wide_n) {
                        failures.push(format!("[{kind_name}] n=k+2 case: {e}"));
                    }
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "K-of-N round-trip FAILED — design is wrong, STOP/BLOCK. First failures:\n{}",
        failures
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// (c) C1 index-`s` short-circuit — `interpolate_at([secret_s, d], Fe::S)` returns
/// `secret_s` directly without Lagrange-validating `d`; and `[secret_s, secret_s]`
/// returns Ok (no RepeatedIndex). Justifies `combine_shares`' pre-reject of index-`s`.
#[test]
fn interpolate_short_circuits_on_index_s() {
    // entr secret, threshold k=2, index S.
    let mut secret_bytes = vec![0x00u8];
    secret_bytes.extend(std::iter::repeat_n(0xCDu8, 16));
    let id = "tst7";
    let secret_s = Codex32String::from_seed(HRP, 2, id, Fe::S, &secret_bytes)
        .expect("from_seed(secret S, k=2) must succeed");

    // One distributed share at a non-`s` index, threshold 2, same id/length/hrp.
    // Its PAYLOAD is arbitrary filler — the short-circuit must NOT depend on it
    // being a valid Lagrange point of any particular secret.
    let mut filler = vec![0x00u8];
    filler.extend(std::iter::repeat_n(0x10u8, 16));
    let d = Codex32String::from_seed(HRP, 2, id, non_s_index(0), &filler)
        .expect("from_seed(distributed, k=2) must succeed");

    // The short-circuit MUST fire: result == secret_s, regardless of d's payload.
    let short = Codex32String::interpolate_at(&[secret_s.clone(), d.clone()], Fe::S)
        .expect("interpolate_at([secret_s, d], S) must short-circuit Ok (C1) — if it errors, C1 is WRONG, STOP/BLOCK");
    assert_eq!(
        short.to_string(),
        secret_s.to_string(),
        "C1 short-circuit must return secret_s WITHOUT validating d — if not, the C1 reasoning is wrong, STOP/BLOCK"
    );

    // [secret_s, secret_s] must also short-circuit Ok (no RepeatedIndex), since
    // the index-`s` match returns before the Lagrange repeated-index check.
    let dup = Codex32String::interpolate_at(&[secret_s.clone(), secret_s.clone()], Fe::S)
        .expect("interpolate_at([secret_s, secret_s], S) must be Ok (no RepeatedIndex) — C1");
    assert_eq!(
        dup.to_string(),
        secret_s.to_string(),
        "duplicate-secret_s short-circuit must return secret_s"
    );
}
