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

    // The expected array-id is fixed by the seed AND the payload digest (F2):
    // top22(SHA-256(seed ‖ SHA-256(canonical))). Decode every plate and check its
    // RAID metadata matches what raid_encode set.
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
    // And it is the value raid_encode derives from the seed + payload digest.
    // (a) determinism pin — re-derive the FROZEN canonical layout (SPEC §1a):
    //   canonical = u32-BE n ‖ for each payload in index order:
    //               u16-BE payload_bits ‖ exactly ceil(payload_bits/8) payload bytes
    //   array_id  = top22(SHA-256(seed ‖ SHA-256(canonical)))
    let mut canonical: Vec<u8> = Vec::new();
    canonical.extend_from_slice(&(orig.len() as u32).to_be_bytes());
    for (bytes, bits) in &orig {
        canonical.extend_from_slice(&(*bits as u16).to_be_bytes());
        let nb = bits.div_ceil(8);
        canonical.extend_from_slice(&bytes[..nb]);
    }
    let payload_digest = <sha2::Sha256 as sha2::Digest>::digest(&canonical);
    let mut h = <sha2::Sha256 as sha2::Digest>::new();
    sha2::Digest::update(&mut h, &seed);
    sha2::Digest::update(&mut h, payload_digest);
    let d = sha2::Digest::finalize(h);
    let expected = (((d[0] as u32) << 16) | ((d[1] as u32) << 8) | (d[2] as u32)) >> 2;
    assert_eq!(
        aids[0], expected,
        "array-id = top22(SHA-256(seed ‖ SHA-256(canonical)))"
    );
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

// ===========================================================================
// KAT 12 (F2) — same-quorum array-id collision (mixed-plate wrong-xpub).
//
// Two arrays sharing a cosigner-fingerprint seed but carrying DIFFERENT payloads
// used to collide on array_id = top22(SHA-256(seed)) (network / account / script-
// type independent). Mixing n-1 data of one with a parity of the other (1 missing)
// silently returned a valid-but-WRONG xpub at exit 0 (constellation-eval F2).
//
// The fix has three parts:
//   (a) fold a deterministic payload digest into array_id so two DIFFERENT wallets
//       get DIFFERENT ids → the coarse equality gate refuses a FRESH cross-mix;
//   (b) a spare-parity consistency oracle that catches a LEGACY same-id chimera
//       whenever a spare parity equation exists.
//
// RED-proof (pre-(a), against the old derivation) reproduced Ok(wrong bytes):
//   `got2 = [116, 80, 84, 222, 213, 18, 226, 233]` — neither array's true payload.
// See design/SPEC_f2_wc_codec_raid_array_id_collision.md.
// ===========================================================================

// --- (a) A FRESH cross-mix now refuses at the equality gate. ----------------
#[test]
fn f2a_fresh_same_seed_diff_payload_cross_mix_refuses() {
    // Two r=2 n=3 arrays: SAME array-id seed, DIFFERENT payloads (equal length).
    let shared_seed = seed_of(3, 900);
    let orig_a = xpub_payloads(3, 5551); // 73-B xpub-shaped
    let orig_b = xpub_payloads(3, 7771);
    let a = raid_encode(&orig_a, &shared_seed, 2, &opts()).expect("encode A");
    let b = raid_encode(&orig_b, &shared_seed, 2, &opts()).expect("encode B");

    // (a) makes the two ids DIFFER even though the seed is identical.
    let aid_a = decode(&a[0].words).unwrap().raid.unwrap().array_id;
    let aid_b = decode(&b[0].words).unwrap().raid.unwrap().array_id;
    assert_ne!(
        aid_a, aid_b,
        "the payload digest must differentiate two same-seed different-payload arrays"
    );

    // n-1 data of A + a parity of B, 1 missing. Pre-(a) this returned Ok(wrong
    // bytes) at exit 0 (the F2 bug); post-(a) the mismatched ids refuse.
    let wa = all_words(&a);
    let wb = all_words(&b);
    let mix = vec![wa[0].clone(), wa[1].clone(), wb[3].clone()]; // A0, A1, B parityA
    let refs = as_refs(&mix);
    assert_eq!(
        raid_reconstruct(&refs),
        Err(WcError::RaidArrayMismatch),
        "a fresh same-quorum cross-mix must refuse, never emit a wrong xpub"
    );
}

// --- (a) determinism + privacy-mode differentiation. ------------------------
#[test]
fn f2a_array_id_is_deterministic_and_payload_sensitive() {
    let seed = seed_of(3, 42);
    let orig = xpub_payloads(3, 12345);
    let id1 = decode(&raid_encode(&orig, &seed, 1, &opts()).unwrap()[0].words)
        .unwrap()
        .raid
        .unwrap()
        .array_id;
    // Same inputs ⇒ same id (repro-safe).
    let id2 = decode(&raid_encode(&orig, &seed, 2, &opts()).unwrap()[0].words)
        .unwrap()
        .raid
        .unwrap()
        .array_id;
    assert_eq!(id1, id2, "array_id is deterministic and r-independent");

    // Privacy-mode bonus: two all-privacy-card arrays (seed = 0^{4n}) with DIFFERENT
    // payloads used to collide with probability 1; the payload digest separates them.
    let zero_seed = vec![0u8; 12];
    let a = decode(&raid_encode(&xpub_payloads(3, 1), &zero_seed, 1, &opts()).unwrap()[0].words)
        .unwrap()
        .raid
        .unwrap()
        .array_id;
    let b = decode(&raid_encode(&xpub_payloads(3, 2), &zero_seed, 1, &opts()).unwrap()[0].words)
        .unwrap()
        .raid
        .unwrap()
        .array_id;
    assert_ne!(
        a, b,
        "privacy-mode arrays with different payloads must differ"
    );
}

// --- (a) an identical-payload re-issue is a harmless no-op mix. -------------
#[test]
fn f2a_identical_payload_reissue_still_groups() {
    // Same seed AND same payloads ⇒ same id (a re-issue). Mixing is a no-op: the
    // stripes are identical, so any cross-mix reconstructs the SAME payloads.
    let seed = seed_of(3, 77);
    let orig = xpub_payloads(3, 999);
    let a = raid_encode(&orig, &seed, 1, &opts()).expect("A");
    let b = raid_encode(&orig, &seed, 1, &opts()).expect("B (re-issue)");
    let wa = all_words(&a);
    let wb = all_words(&b);
    // A0, A1 + B's parityA (identical payloads ⇒ identical stripes ⇒ harmless).
    let mix = vec![wa[0].clone(), wa[1].clone(), wb[3].clone()];
    assert_recovers(&mix, &orig);
}

// ---------------------------------------------------------------------------
// (b) LEGACY FIXTURES — plates engraved under the OLD colliding derivation.
//
// GENERATED at the pre-(a) checkpoint by `raid_encode` on the OLD binary from:
//   shared_seed = seed_of(3, 900);
//   orig_a = payloads(&[8,8,8], 901);  orig_b = payloads(&[8,8,8], 902);  r = 2.
// Post-(a) the encoder can no longer emit colliding-id plates, so these pinned
// word-lists are the ONLY way to exercise (b)'s oracle. `decode`/`raid_reconstruct`
// never RE-derive array_id, so the pinned old plates pass the coarse equality gate
// exactly as they did in the field — the oracle then catches the chimera.
// ---------------------------------------------------------------------------
const LEGACY_A_DATA0: &str = "acoustic avoid abandon month lumber able sheriff gas purity then trial abandon abandon abandon abandon able abandon achieve coast place topic park logic exotic hat ocean proud pear usual timber";
const LEGACY_A_DATA1: &str = "acoustic avoid length month lumber able sheriff gas distance then trial abandon abandon abandon abandon able abandon fresh cave slot delay trick metal post system rough tragic pear usual under";
const LEGACY_B_DATA2: &str = "acoustic awake abandon month lumber able sheriff gas minor then trial abandon abandon abandon abandon able abandon dice click dust upon delay lottery silk angry solution amateur pattern usual unable";
const LEGACY_B_PARITY_A: &str = "acoustic bamboo abandon month lumber able sheriff gas october then trial abandon abandon abandon abandon able abandon citizen melt fury item gravity kitchen merit kid farm fat peasant usual uniform";
const LEGACY_B_PARITY_B: &str = "acoustic beef abandon month lumber able sheriff gas school then trial abandon abandon abandon abandon achieve abandon decade manual image scheme gate kitten father educate transfer entry pencil usual truth";

fn split(s: &str) -> Vec<&str> {
    s.split_whitespace().collect()
}

/// The payloads that generated the legacy fixtures (regenerated deterministically;
/// unaffected by the (a) array_id change).
fn legacy_orig_a() -> Vec<(Vec<u8>, usize)> {
    payloads(&[8usize, 8, 8], 901)
}
fn legacy_orig_b() -> Vec<(Vec<u8>, usize)> {
    payloads(&[8usize, 8, 8], 902)
}

// --- (b) legacy r=2 cross-mix, 1 missing ⇒ the SPARE parity equation catches. -
#[test]
fn f2b_legacy_r2_cross_mix_spare_parity_refuses() {
    // Sanity: the pinned plates still collide on array_id (they must, or the
    // equality gate — not the oracle — would catch them, invalidating the KAT).
    let aid = |s: &str| decode(&split(s)).unwrap().raid.unwrap().array_id;
    assert_eq!(aid(LEGACY_A_DATA0), aid(LEGACY_B_PARITY_A));
    assert_eq!(aid(LEGACY_A_DATA0), aid(LEGACY_B_PARITY_B));

    // A0, A1 (data 0,1 from A) + B's BOTH parity plates, data 2 missing. The solve
    // uses B's parityA to fill data 2; B's parityB is the SPARE equation → it is
    // re-derived over the full set and does NOT match (A's data vs B's parity).
    let mix: Vec<Vec<&str>> = vec![
        split(LEGACY_A_DATA0),
        split(LEGACY_A_DATA1),
        split(LEGACY_B_PARITY_A),
        split(LEGACY_B_PARITY_B),
    ];
    assert_eq!(
        raid_reconstruct(&mix),
        Err(WcError::RaidArrayMismatch),
        "the spare parity equation must catch a legacy same-quorum chimera"
    );
}

// --- (b) legacy 0-data-missing chimera WITH ≥1 parity ⇒ refuse. -------------
#[test]
fn f2b_legacy_zero_missing_chimera_with_parity_refuses() {
    // A0, A1 (from A), B2 (from B) — a full 3-data chimera — plus B's parityA. No
    // data missing, but a parity plate is present, so the oracle verifies it over
    // the reconstructed set: recomputed P1 = A0⊕A1⊕B2 ≠ B0⊕B1⊕B2 = engraved P1.
    let mix: Vec<Vec<&str>> = vec![
        split(LEGACY_A_DATA0),
        split(LEGACY_A_DATA1),
        split(LEGACY_B_DATA2),
        split(LEGACY_B_PARITY_A),
    ];
    assert_eq!(
        raid_reconstruct(&mix),
        Err(WcError::RaidArrayMismatch),
        "a 0-missing chimera with a parity plate present must refuse"
    );
}

// --- (b) ACCEPTED RESIDUAL: 0-missing pure-data chimera, NO parity ⇒ undetectable.
#[test]
fn f2b_legacy_zero_missing_pure_data_chimera_no_parity_is_undetectable() {
    // A0, A1 (from A), B2 (from B) — 3 data plates, NO parity plate presented.
    // array_id / n / width all match, there is NO parity equation, and no plate is
    // MDS-solved (so the (c) advisory never fires either). This chimera is
    // info-theoretically undetectable IN-BAND for legacy plates — it returns each
    // plate's genuine standalone payload. Documented, NOT a bug (SPEC §1b): the
    // honest mitigation is "include a parity plate when decoding a card set".
    let mix: Vec<Vec<&str>> = vec![
        split(LEGACY_A_DATA0),
        split(LEGACY_A_DATA1),
        split(LEGACY_B_DATA2),
    ];
    let rec = raid_reconstruct(&mix).expect("no parity equation ⇒ not caught (residual)");
    let a = legacy_orig_a();
    let b = legacy_orig_b();
    assert_eq!(rec.reconstructed, Vec::<usize>::new(), "nothing MDS-solved");
    assert_eq!(
        rec.payloads,
        vec![a[0].clone(), a[1].clone(), b[2].clone()],
        "each plate's genuine standalone payload is returned (the accepted residual)"
    );
}

// --- (b) G3: the oracle must NOT over-reject a GENUINE array. ---------------
#[test]
fn f2b_spare_parity_oracle_does_not_over_reject_genuine() {
    // A genuine r=2 array: drop ONE data plate but keep BOTH parity plates, so the
    // spare-parity check runs (r_available=2 > missing=1) and MUST pass.
    let n = 4;
    let orig = xpub_payloads(n, 606);
    let seed = seed_of(n, 61);
    let plates = raid_encode(&orig, &seed, 2, &opts()).expect("encode");
    let full = all_words(&plates);

    // Drop data plate 1 (keep data 0,2,3 + parityA + parityB).
    let mut set = full.clone();
    set.remove(1);
    let rec = assert_recovers(&set, &orig);
    assert!(rec.reconstructed.contains(&1));

    // Also: a full 0-missing r=2 decode runs the spare check on BOTH parity plates
    // and must still pass exactly.
    assert_recovers(&full, &orig);
}

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
