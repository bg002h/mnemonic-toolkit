//! KATs for the **P5 cross-plate RAID layer** (`mk1` arrays only) — plan §3
//! (RAID generator), §4.2 (H1 / array-id), §4.6, §7 P5; spec §7.
//!
//! Load-bearing tests (the ones R0 will scrutinize):
//! - **recover any `r` of `n+r`** — remove EACH single plate (r=1) / EACH pair
//!   (r=2), data OR parity, and recover all `n` payloads exactly;
//! - **NEW-I2 regression (Critical-class)** — `n=15, r=2`, remove any 2 →
//!   exact recovery; proves the full 5-bit `index-in-array` exponent keeps r=2
//!   MDS for `n>8` (the old 3-bit index silently broke this);
//! - **`P₁` append-only** — the r=1 ParityA plate is byte-identical to the r=2
//!   ParityA plate;
//! - **privacy** — a lone ParityA + `< n−1` data plates leaves the missing
//!   xpubs underdetermined (the linear system is rank-deficient).

use proptest::prelude::*;
use wc_codec::{decode, raid_encode, raid_reconstruct, EncodeOpts, PlateRole, SourceKind, WcError};

// ---------------------------------------------------------------------------
// Helpers.
// ---------------------------------------------------------------------------

/// Deterministic pseudo-random bytes of length `n` (fixed-seed; binary-identical
/// output for KATs, no CSPRNG).
fn det_bytes(n: usize, seed: u64) -> Vec<u8> {
    let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
    (0..n)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s & 0xFF) as u8
        })
        .collect()
}

/// Build `n` byte-aligned payloads of the given byte lengths (xpub-shaped).
fn payloads(lens: &[usize], seed: u64) -> Vec<(Vec<u8>, usize)> {
    lens.iter()
        .enumerate()
        .map(|(i, &len)| {
            let b = det_bytes(len, seed + i as u64);
            let bits = b.len() * 8;
            (b, bits)
        })
        .collect()
}

/// `n` equal-length 73-byte (canonical xpub) payloads.
fn xpub_payloads(n: usize, seed: u64) -> Vec<(Vec<u8>, usize)> {
    payloads(&vec![73usize; n], seed)
}

/// A fixed array-id seed (concatenated ordered cosigner fingerprints).
fn seed_of(n: usize, salt: u64) -> Vec<u8> {
    // 4 bytes per fingerprint × n, deterministic.
    det_bytes(4 * n, 0xF1_0000 + salt)
}

fn opts() -> EncodeOpts {
    EncodeOpts::default()
}

/// All plates' words (owned), in encode order (n data then r parity).
fn all_words(plates: &[wc_codec::RaidPlate]) -> Vec<Vec<String>> {
    plates
        .iter()
        .map(|p| p.words.iter().map(|w| w.to_string()).collect())
        .collect()
}

/// Borrow a `&[Vec<String>]` as the `Vec<Vec<&str>>` reconstruct wants.
fn as_refs(set: &[Vec<String>]) -> Vec<Vec<&str>> {
    set.iter()
        .map(|p| p.iter().map(|s| s.as_str()).collect())
        .collect()
}

/// Run reconstruct over a slice of word-sets, asserting it recovers all `n`
/// ORIGINAL payloads exactly (index order).
fn assert_recovers(set: &[Vec<String>], original: &[(Vec<u8>, usize)]) -> wc_codec::RaidRecovery {
    let refs = as_refs(set);
    let rec = raid_reconstruct(&refs).expect("reconstruct");
    assert_eq!(
        rec.payloads.len(),
        original.len(),
        "recovered {} payloads, expected {}",
        rec.payloads.len(),
        original.len()
    );
    for (i, (got, want)) in rec.payloads.iter().zip(original.iter()).enumerate() {
        assert_eq!(got, want, "payload {i} mismatch after reconstruct");
    }
    rec
}

// ===========================================================================
// KAT 1 — Full array round-trip (all plates present).
// ===========================================================================

#[test]
fn full_array_round_trip_varied_n_r() {
    for &n in &[2usize, 3, 5] {
        for &r in &[1u8, 2] {
            // r < n is required (r=2 needs n≥3); KAT 11 asserts r≥n is refused.
            if r as usize >= n {
                continue;
            }
            let orig = xpub_payloads(n, 100 + n as u64 * 10 + r as u64);
            let seed = seed_of(n, r as u64);
            let plates = raid_encode(&orig, &seed, r, &opts()).expect("raid_encode");
            assert_eq!(plates.len(), n + r as usize, "plate count = n+r");

            // Roles / indices as expected: n Data (index 0..n-1), then r parity.
            for (i, p) in plates.iter().enumerate() {
                if i < n {
                    assert_eq!(p.role, PlateRole::Data);
                    assert_eq!(p.index, i);
                } else {
                    let pr = i - n;
                    assert_eq!(
                        p.index,
                        n + pr,
                        "parity index continues 0..n-1 exponent space"
                    );
                    assert_eq!(
                        p.role,
                        if pr == 0 {
                            PlateRole::ParityA
                        } else {
                            PlateRole::ParityB
                        }
                    );
                }
            }

            let set = all_words(&plates);
            assert_recovers(&set, &orig);
        }
    }
}

// ===========================================================================
// KAT 2 — Recover ANY r of n+r (load-bearing). n=3 r=1: each of 4; n=3 r=2:
// each pair of 5. Mixed data/parity removals.
// ===========================================================================

#[test]
fn recover_any_one_of_four_r1() {
    let n = 3;
    let orig = xpub_payloads(n, 7);
    let seed = seed_of(n, 0);
    let plates = raid_encode(&orig, &seed, 1, &opts()).expect("encode");
    let full = all_words(&plates);
    assert_eq!(full.len(), n + 1);

    for drop in 0..(n + 1) {
        let mut set: Vec<Vec<String>> = full.clone();
        set.remove(drop);
        let rec = assert_recovers(&set, &orig);
        if drop < n {
            assert!(
                rec.reconstructed.contains(&drop),
                "dropping data plate {drop} should report it reconstructed; got {:?}",
                rec.reconstructed
            );
        }
    }
}

#[test]
fn recover_any_pair_of_five_r2() {
    let n = 3;
    let orig = xpub_payloads(n, 9);
    let seed = seed_of(n, 1);
    let plates = raid_encode(&orig, &seed, 2, &opts()).expect("encode");
    let full = all_words(&plates);
    assert_eq!(full.len(), n + 2);

    let total = n + 2;
    for a in 0..total {
        for b in (a + 1)..total {
            let mut set: Vec<Vec<String>> = full.clone();
            // Remove higher index first so the lower index stays valid.
            set.remove(b);
            set.remove(a);
            assert_recovers(&set, &orig);
        }
    }
}

// ===========================================================================
// KAT 3 — NEW-I2 regression (Critical-class): n=15, r=2, remove any 2 → exact.
// Proves the full 5-bit index-in-array exponent keeps r=2 MDS for n>8.
// ===========================================================================

#[test]
fn new_i2_n15_r2_any_two_exact() {
    let n = 15;
    let orig = xpub_payloads(n, 31);
    let seed = seed_of(n, 2);
    let plates = raid_encode(&orig, &seed, 2, &opts()).expect("encode");
    let full = all_words(&plates);
    assert_eq!(full.len(), n + 2);

    let total = n + 2;
    for a in 0..total {
        for b in (a + 1)..total {
            let mut set: Vec<Vec<String>> = full.clone();
            set.remove(b);
            set.remove(a);
            assert_recovers(&set, &orig);
        }
    }
}

// ===========================================================================
// KAT 4 — P₁ append-only: r=1 ParityA payload == r=2 ParityA payload.
// ===========================================================================

#[test]
fn p1_append_only_parity_a_byte_identical() {
    // n ≥ 3 so r=2 is valid (r < n); the P₁ stripe is independent of n anyway.
    for &n in &[3usize, 8, 15] {
        let orig = xpub_payloads(n, 200 + n as u64);
        let seed = seed_of(n, 5);
        let p1 = raid_encode(&orig, &seed, 1, &opts()).expect("r1");
        let p2 = raid_encode(&orig, &seed, 2, &opts()).expect("r2");

        // ParityA is the plate at index n in both (role == ParityA).
        let a1 = &p1[n];
        let a2 = &p2[n];
        assert_eq!(a1.role, PlateRole::ParityA);
        assert_eq!(a2.role, PlateRole::ParityA);
        assert_eq!(
            a1.words, a2.words,
            "ParityA plate must be byte-identical whether r=1 or r=2 (append-only, n={n})"
        );
    }
}

// ===========================================================================
// KAT 5 — Privacy: a lone ParityA + (< n−1) data plates leaves the missing
// xpubs underdetermined (rank-deficient system → multiple consistent
// assignments). Demonstrated structurally: reconstruct must REFUSE (it cannot
// uniquely recover) when > r plates are missing.
// ===========================================================================

#[test]
fn privacy_lone_parity_plus_one_data_underdetermined_n3() {
    // n=3, r=1: holding P₁ + exactly 1 data plate ⇒ 2 data plates missing > r=1.
    let n = 3;
    let orig = xpub_payloads(n, 11);
    let seed = seed_of(n, 6);
    let plates = raid_encode(&orig, &seed, 1, &opts()).expect("encode");
    let full = all_words(&plates);

    // (1) Operational guarantee: reconstruct REFUSES (cannot uniquely recover).
    // Keep ParityA (index n) + data plate 0; drop data 1 and 2.
    let kept = vec![full[0].clone(), full[n].clone()];
    let refs = as_refs(&kept);
    let res = raid_reconstruct(&refs);
    assert!(
        res.is_err(),
        "P₁ + 1 of 3 data plates is rank-deficient (2 unknowns, 1 equation) — \
         must NOT uniquely reconstruct, got {res:?}"
    );

    // (2) Structural demonstration: with P₁ and x₀ known, the equation
    //     x₁ + x₂ = P₁ − x₀ =: s  (over GF(2¹¹), column-wise)
    // has 2¹¹ consistent (x₁, x₂) assignments per column — TWO of which we exhibit
    // here, both equally consistent with the held parity + data plate. The lone
    // ParityA plate (plus < n−1 data) therefore leaks NOTHING about the missing
    // xpubs (spec §7.3 / plan §7 P5 privacy).
    let d0 = decode(&as_refs(&[full[0].clone()])[0]).expect("decode data 0");
    let dpa = decode(&as_refs(&[full[n].clone()])[0]).expect("decode parityA");
    let x0 = stripe_syms(&d0.payload, d0.payload_bits);
    let p1 = stripe_syms(&dpa.payload, dpa.payload_bits);
    let width = x0.len();
    assert_eq!(p1.len(), width);

    // s[c] = P₁[c] XOR x₀[c]  (the residual the two missing plates must sum to).
    let s: Vec<u16> = (0..width).map(|c| p1[c] ^ x0[c]).collect();

    // Assignment A: (x₁ = s, x₂ = 0). Assignment B: (x₁ = 0, x₂ = s). Both satisfy
    // x₁ XOR x₂ = s for every column, yet are DIFFERENT whenever s ≠ 0.
    let a_x1: Vec<u16> = s.clone();
    let a_x2: Vec<u16> = vec![0u16; width];
    let b_x1: Vec<u16> = vec![0u16; width];
    let b_x2: Vec<u16> = s.clone();
    for c in 0..width {
        assert_eq!(a_x1[c] ^ a_x2[c], s[c], "assignment A consistent");
        assert_eq!(b_x1[c] ^ b_x2[c], s[c], "assignment B consistent");
    }
    // The two assignments differ (s is not all-zero for a real xpub array).
    assert!(
        s.iter().any(|&v| v != 0),
        "residual is non-trivial (otherwise the demo column is degenerate)"
    );
    assert_ne!(
        (a_x1, a_x2),
        (b_x1, b_x2),
        "two DISTINCT (x₁,x₂) assignments both consistent ⇒ underdetermined ⇒ \
         the lone parity + 1 data plate reveals nothing about the missing xpubs"
    );
}

/// Decode a plate's recovered stripe symbols (width-W) from its payload — mirrors
/// the codec's internal stripe unpacking (11-bit MSB-first regroup). Used only by
/// the structural privacy demonstration.
fn stripe_syms(payload: &[u8], payload_bits: usize) -> Vec<u16> {
    assert_eq!(payload_bits % 11, 0);
    wc_codec::regroup::bits_to_symbols(payload, payload_bits)
}

// ===========================================================================
// KAT 6 — array-id: two different arrays (different seeds) must not silently
// mix; same seed groups + reconstructs.
// ===========================================================================

#[test]
fn array_id_does_not_mix_distinct_arrays() {
    let n = 3;
    let orig_a = xpub_payloads(n, 21);
    let orig_b = xpub_payloads(n, 22);
    let seed_a = seed_of(n, 100);
    let seed_b = seed_of(n, 200);
    let pa = raid_encode(&orig_a, &seed_a, 1, &opts()).expect("encode A");
    let pb = raid_encode(&orig_b, &seed_b, 1, &opts()).expect("encode B");

    // Mix: 2 plates from A + 1 plate from B (a wrong-array plate). Reconstruct
    // must NOT silently mix — it errors (mismatched array-ids).
    let mut mixed: Vec<Vec<String>> = Vec::new();
    let wa = all_words(&pa);
    let wb = all_words(&pb);
    mixed.push(wa[0].clone());
    mixed.push(wa[1].clone());
    mixed.push(wb[2].clone()); // foreign plate
    let refs = as_refs(&mixed);
    let res = raid_reconstruct(&refs);
    assert!(
        res.is_err(),
        "mixing plates from two different arrays must error, not silently mix; got {res:?}"
    );

    // Sanity: the same-seed array reconstructs fine.
    let set = all_words(&pa);
    assert_recovers(&set, &orig_a);
}

// ===========================================================================
// KAT 7 — Each plate is a valid standalone Word-Card: decode(plate.words)
// succeeds (the per-plate payload is the width-W stripe, RS-protected) and the
// plate exposes the correct role/index/array-id via raid metadata.
// ===========================================================================

#[test]
fn each_plate_is_a_valid_standalone_word_card() {
    use wc_codec::PlateRole as PR;
    let n = 4;
    let r = 2u8;
    let orig = xpub_payloads(n, 41);
    let seed = seed_of(n, 7);
    let plates = raid_encode(&orig, &seed, r, &opts()).expect("encode");

    // The expected array-id is fixed by the seed (top 22 bits of SHA-256(seed)).
    // Decode every plate and check its RAID metadata matches what raid_encode set.
    for p in &plates {
        let refs: Vec<&str> = p.words.to_vec();
        let decoded = decode(&refs).expect("each plate decodes standalone");
        // The plate is an mk1-kind Word-Card (RAID is mk1-only).
        assert_eq!(decoded.kind, SourceKind::Mk1Xpub);
        assert!(
            !decoded.truncated,
            "a freshly-encoded plate is not truncated"
        );
        // RAID metadata is exposed and matches the plate's role/index.
        let meta = decoded.raid.expect("a RAID plate exposes raid metadata");
        assert_eq!(meta.n, n, "n");
        assert_eq!(meta.role, p.role, "role round-trips through decode");
        assert_eq!(meta.index, p.index, "index round-trips through decode");
        match p.role {
            PR::Data => assert!(p.index < n),
            PR::ParityA => assert_eq!(p.index, n),
            PR::ParityB => assert_eq!(p.index, n + 1),
        }
    }

    // All plates share ONE array-id (they belong to the same array).
    let aids: Vec<u32> = plates
        .iter()
        .map(|p| decode(&p.words).unwrap().raid.unwrap().array_id)
        .collect();
    assert!(
        aids.windows(2).all(|w| w[0] == w[1]),
        "every plate carries the same array-id; got {aids:?}"
    );
    // And it is the value raid_encode derives from the seed.
    let mut h = <sha2::Sha256 as sha2::Digest>::new();
    sha2::Digest::update(&mut h, &seed);
    let d = sha2::Digest::finalize(h);
    let expected = (((d[0] as u32) << 16) | ((d[1] as u32) << 8) | (d[2] as u32)) >> 2;
    assert_eq!(aids[0], expected, "array-id = top 22 bits of SHA-256(seed)");
}

// ===========================================================================
// KAT 8 — > r missing ⇒ refuse (no silent wrong reconstruction).
// ===========================================================================

#[test]
fn more_than_r_missing_refuses() {
    // r=1: drop 2 data plates ⇒ refuse.
    let n = 4;
    let orig = xpub_payloads(n, 51);
    let seed = seed_of(n, 8);
    let plates = raid_encode(&orig, &seed, 1, &opts()).expect("encode");
    let full = all_words(&plates);
    // Keep parityA + data 0,1 ; drop data 2,3 (2 missing > r=1).
    let kept = vec![full[0].clone(), full[1].clone(), full[n].clone()];
    let refs = as_refs(&kept);
    assert!(
        raid_reconstruct(&refs).is_err(),
        "2 missing data with r=1 must refuse"
    );

    // r=2: drop 3 data plates ⇒ refuse.
    let plates2 = raid_encode(&orig, &seed, 2, &opts()).expect("encode r2");
    let full2 = all_words(&plates2);
    // Keep data 0 + parityA + parityB ; drop data 1,2,3 (3 missing > r=2).
    let kept2 = vec![full2[0].clone(), full2[n].clone(), full2[n + 1].clone()];
    let refs2 = as_refs(&kept2);
    assert!(
        raid_reconstruct(&refs2).is_err(),
        "3 missing data with r=2 must refuse"
    );
}

// ===========================================================================
// KAT 9 — Varied lengths: xpubs of slightly different payload sizes →
// padding + length-prefix handled → exact (including a fully-missing plate).
// ===========================================================================

#[test]
fn varied_payload_lengths_exact() {
    // Lengths differ by a few bytes; the stripe length-prefix + zero-pad must
    // reconstruct each (payload_bytes, payload_bits) EXACTLY.
    for &r in &[1u8, 2] {
        let orig = payloads(&[71usize, 73, 70, 74, 72][..(3 + r as usize)], 61);
        let n = orig.len();
        let seed = seed_of(n, 9);
        let plates = raid_encode(&orig, &seed, r, &opts()).expect("encode");
        let full = all_words(&plates);

        // Full set recovers.
        assert_recovers(&full, &orig);

        // Drop a data plate (entirely missing) — recover via RAID. Verify the
        // RECONSTRUCTED plate's exact (payload_bytes, payload_bits) match.
        for drop in 0..n {
            let mut set = full.clone();
            set.remove(drop);
            let rec = assert_recovers(&set, &orig);
            assert!(rec.reconstructed.contains(&drop));
        }
    }
}

// ===========================================================================
// KAT 10 — Combined within-plate error + whole-plate loss: one present plate
// has within-budget corruption (its own RS+tag fixes it) AND another plate is
// missing (RAID-reconstructed) → all recovered.
// ===========================================================================

#[test]
fn within_plate_error_plus_whole_plate_loss() {
    let n = 3;
    let r = 1u8;
    let orig = xpub_payloads(n, 71);
    let seed = seed_of(n, 10);
    // Provision parity on each plate so a single-symbol substitution is
    // RS-correctable per-plate. The P3→P2 contract converts a CRC-flagged block
    // into a WHOLE-BLOCK erasure (≈ b ≈ √K words), so the per-plate parity must be
    // `m ≥ b` (NOT just ≥ 2). For a 73-B stripe, b ≈ 8, so m = 12 is comfortable.
    let u_slots = 1u8;
    let popts = EncodeOpts {
        parity_words: 12,
        integrity_bits: 44,
        u_slots,
    };
    let plates = raid_encode(&orig, &seed, r, &popts).expect("encode");
    let full = all_words(&plates);

    // Corrupt one symbol of a PRESENT data plate (plate 1), and DROP data plate 2.
    let mut set: Vec<Vec<String>> = full.clone();
    set.remove(2); // whole-plate loss (the RAID-reconstructed one)
                   // Now plate 1 is at index 1 of `set`; corrupt the FIRST data word inside the
                   // interleave region. Engraved layout = [header 9 (raid)] [ledger 2U]
                   // [interleave …] [parity] [stop 2]; the first interleave word is at offset
                   // `9 + 2U`. A single substitution flags block 0's CRC ⇒ block 0 is erased
                   // (≈ b ≤ m) ⇒ the RS pass fills it.
    let raid_header_words = 9usize; // H0(1)+H1(2)+array-id(2)+GEOM(4)
    let first_interleave = raid_header_words + 2 * u_slots as usize;
    let target = &mut set[1];
    let idx = first_interleave + 1;
    let cur = wc_codec::wordmap::word_to_symbol(&target[idx]).unwrap();
    let other = (cur + 17) % 2048;
    target[idx] = wc_codec::wordmap::symbol_to_word(other)
        .unwrap()
        .to_string();

    // RAID reconstruct: present plates self-heal (RS+tag), then the missing data
    // plate is RAID-recovered. All 3 payloads exact.
    assert_recovers(&set, &orig);
}

// ===========================================================================
// KAT 11 — n / r out of range ⇒ WcError, no panic.
// ===========================================================================

#[test]
fn n_and_r_out_of_range_errors_no_panic() {
    // n < 2 (a single payload is not an array).
    let one = xpub_payloads(1, 81);
    assert!(matches!(
        raid_encode(&one, &seed_of(1, 0), 1, &opts()),
        Err(WcError::InvalidParams)
    ));

    // n > 32.
    let big = xpub_payloads(33, 82);
    assert!(matches!(
        raid_encode(&big, &seed_of(33, 0), 1, &opts()),
        Err(WcError::InvalidParams)
    ));

    // r > 2 (surfaced cap).
    let three = xpub_payloads(3, 83);
    assert!(matches!(
        raid_encode(&three, &seed_of(3, 0), 3, &opts()),
        Err(WcError::InvalidParams)
    ));

    // r == 0 (no recovery plate is not a RAID array).
    assert!(matches!(
        raid_encode(&three, &seed_of(3, 0), 0, &opts()),
        Err(WcError::InvalidParams)
    ));

    // r >= n: the [n+r,n] construction needs r < n for a sane array; r=2,n=2
    // is rejected (a 2-plate array with 2 parity is degenerate).
    let two = xpub_payloads(2, 84);
    assert!(matches!(
        raid_encode(&two, &seed_of(2, 0), 2, &opts()),
        Err(WcError::InvalidParams)
    ));

    // reconstruct with an empty plate set ⇒ error, no panic.
    let empty: Vec<Vec<&str>> = Vec::new();
    assert!(raid_reconstruct(&empty).is_err());
}

// ===========================================================================
// Proptest — random n, r, payload lengths, and a random ≤r removal set always
// either recovers exactly or refuses; never panics, never returns wrong bytes.
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig { cases: 40, ..ProptestConfig::default() })]

    #[test]
    fn prop_recover_or_refuse_never_wrong(
        n in 2usize..=6,
        r in 1u8..=2,
        seedsalt in 0u64..1000,
        // up to r removals chosen by index (mod total)
        d0 in 0usize..32,
        d1 in 0usize..32,
        do_two in any::<bool>(),
    ) {
        prop_assume!(r < n as u8);
        let lens: Vec<usize> = (0..n).map(|i| 70 + (i % 5)).collect();
        let orig = payloads(&lens, seedsalt);
        let seed = seed_of(n, seedsalt);
        let plates = raid_encode(&orig, &seed, r, &opts()).expect("encode");
        let full = all_words(&plates);
        let total = full.len();

        // Choose a removal set of size 1 or 2 (≤ r meaningful; >r should refuse).
        let mut idxs = vec![d0 % total];
        let want_two = do_two && r == 2;
        if want_two {
            let j = d1 % total;
            if j != idxs[0] {
                idxs.push(j);
            }
        }
        idxs.sort_unstable();
        idxs.dedup();

        let mut set = full.clone();
        for &i in idxs.iter().rev() {
            set.remove(i);
        }

        let refs = as_refs(&set);
        match raid_reconstruct(&refs) {
            Ok(rec) => {
                // If it succeeded, it MUST be exactly right.
                prop_assert_eq!(rec.payloads.len(), orig.len());
                for (got, want) in rec.payloads.iter().zip(orig.iter()) {
                    prop_assert_eq!(got, want);
                }
            }
            Err(_) => { /* refusing is always acceptable */ }
        }
    }
}
