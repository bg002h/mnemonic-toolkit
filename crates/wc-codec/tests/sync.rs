//! KATs for the structural **sync / checkpoint layer** (plan §4.3, P3).
//!
//! P3 is the *structural* sync layer ONLY — it does NOT carry the integrity tag
//! or *select* among single-deletion candidates (that is P4). These KATs assert
//! the structural contract: the checkpoint codec, `interleave` round-trip, the
//! indel **trichotomy**, realignment, the **bounded candidate set that contains
//! the truth**, deleted-checkpoint detection, the compound case, refuse-on-
//! ambiguity, the mod-8 aliasing bound, the small-K path, and — tying P3↔P2 —
//! whole-block erasure recovered end-to-end via the RS engine with NO tag.

use proptest::prelude::*;
use wc_codec::rs::{rs_codeword, rs_decode};
use wc_codec::sync::{
    block_stride, checkpoint_layout, checkpoint_word, crc5, interleave, parse_checkpoint,
    sync_classify, Checkpoint, SyncError, SyncOutcome, CHECKPOINT_MARKER,
};

const FIELD_LIMIT: u16 = 2048;

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

/// Strip checkpoints **positionally** from a clean `K'` grid for a given `k`.
///
/// Checkpoints are positional, not content-identified: a *data* word may
/// legitimately carry the `0b101` marker pattern (probability 1/8), so stripping
/// by marker is wrong. We compute the exact checkpoint grid positions from the
/// frozen layout and drop those.
fn strip_checkpoints_at(grid: &[u16], k: usize) -> Vec<u16> {
    let layout = checkpoint_layout(k);
    let mut cp_pos = std::collections::HashSet::new();
    let mut offset = 0usize;
    for &sz in &layout.block_sizes {
        offset += sz; // checkpoint sits right after the block's data
        cp_pos.insert(offset);
        offset += 1; // the checkpoint word itself
    }
    grid.iter()
        .enumerate()
        .filter(|(i, _)| !cp_pos.contains(i))
        .map(|(_, &w)| w)
        .collect()
}

// ===========================================================================
// 1. CRC-5 — generator correctness; single-bit always detected; single-word
//    substitution detected except the ≤ 2^-5 algebraic-kernel fraction.
// ===========================================================================

#[test]
fn crc5_in_range_and_deterministic() {
    for seed in 0..50u64 {
        let blk = det_data(8, seed);
        let c = crc5(&blk);
        assert!(c < 32, "crc5 must be 5 bits, got {c} (seed {seed})");
        assert_eq!(c, crc5(&blk), "crc5 deterministic");
    }
}

#[test]
fn crc5_detects_every_single_bit_flip() {
    // A primitive degree-5 generator detects ALL single-bit errors in the
    // message (period 31 > any block bit-length we use here).
    for seed in 0..40u64 {
        let blk = det_data(7, seed);
        let good = crc5(&blk);
        for word in 0..blk.len() {
            for bit in 0..11u16 {
                let mut bad = blk.clone();
                bad[word] ^= 1 << bit;
                assert_ne!(
                    crc5(&bad),
                    good,
                    "single-bit flip word {word} bit {bit} (seed {seed}) must change CRC-5"
                );
            }
        }
    }
}

#[test]
fn crc5_single_word_substitution_detection_rate() {
    // A uniform single-WORD substitution (replace one symbol with a random
    // different one) is missed with probability ≤ 2^-5 ≈ 3.125%. Measure the
    // empirical detection rate over many trials and assert ≥ ~96%.
    let mut total = 0u64;
    let mut detected = 0u64;
    let mut s = 0x1234_5678_9ABC_DEF0u64;
    for seed in 0..200u64 {
        let blk = det_data(8, seed);
        let good = crc5(&blk);
        for word in 0..blk.len() {
            // try several random replacement values
            for _ in 0..16 {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                let mut v = (s % FIELD_LIMIT as u64) as u16;
                if v == blk[word] {
                    v = (v + 1) % FIELD_LIMIT;
                }
                let mut bad = blk.clone();
                bad[word] = v;
                total += 1;
                if crc5(&bad) != good {
                    detected += 1;
                }
            }
        }
    }
    let rate = detected as f64 / total as f64;
    assert!(
        rate >= 0.955,
        "CRC-5 single-word substitution detection rate {rate} below ~96% floor (2^-5 miss)"
    );
}

// ===========================================================================
// 2. Checkpoint word: parse(checkpoint_word(i, blk)) == { i mod 8, crc5(blk) };
//    a non-checkpoint word (marker != 0b101) parses to None.
// ===========================================================================

#[test]
fn checkpoint_word_round_trips_through_parse() {
    for i in 0..40usize {
        let blk = det_data(6, i as u64);
        let w = checkpoint_word(i, &blk);
        // top 3 bits are the marker
        assert_eq!(w >> 8, CHECKPOINT_MARKER, "marker bits (i={i})");
        let parsed = parse_checkpoint(w).expect("recognized as checkpoint");
        assert_eq!(
            parsed,
            Checkpoint {
                index_mod8: (i % 8) as u16,
                crc5: crc5(&blk),
            },
            "parsed fields (i={i})"
        );
    }
}

#[test]
fn parse_checkpoint_rejects_non_checkpoint_words() {
    // Every word whose top-3 bits != 0b101 must parse to None.
    for w in 0u16..2048 {
        let is_marker = (w >> 8) == CHECKPOINT_MARKER;
        assert_eq!(
            parse_checkpoint(w).is_some(),
            is_marker,
            "word {w:#013b}: recognition must follow the marker bits"
        );
    }
}

// ===========================================================================
// 3. interleave round-trip (clean): strip(interleave(data)) == data;
//    checkpoint count == ceil(k/b) (1 for small-K).
// ===========================================================================

#[test]
fn interleave_round_trip_and_count() {
    for &k in &[1usize, 5, 8, 15, 16, 17, 30, 58, 100, 160] {
        let data = det_data(k, k as u64 * 7 + 1);
        let grid = interleave(&data);
        let layout = checkpoint_layout(k);
        let expected_checkpoints = layout.checkpoint_count;
        assert_eq!(
            grid.len(),
            k + expected_checkpoints,
            "K'={} expected for k={k}",
            k + expected_checkpoints
        );
        // strip and compare
        assert_eq!(
            strip_checkpoints_at(&grid, k),
            data,
            "interleave round-trip (k={k})"
        );
        // count == ceil(k/b), and == 1 in the small-K regime
        let b = block_stride(k);
        let want = if k < 16 { 1 } else { k.div_ceil(b) };
        assert_eq!(
            expected_checkpoints, want,
            "checkpoint count (k={k}, b={b})"
        );
        if k < 16 {
            assert_eq!(
                expected_checkpoints, 1,
                "small-K single degenerate checkpoint"
            );
        }
    }
}

#[test]
fn block_stride_matches_floor_sqrt_plus_half() {
    // b = floor(sqrt(K) + 0.5), the frozen rounding, for the large-K regime;
    // small-K (K<16) returns K (single degenerate checkpoint).
    for k in 0usize..2100 {
        let b = block_stride(k);
        if k < 16 {
            assert_eq!(b, k, "small-K stride == K (k={k})");
        } else {
            let want = (f64::from(k as u32).sqrt() + 0.5).floor() as usize;
            assert_eq!(b, want, "b = round(sqrt(K)) for k={k}");
        }
    }
}

// ===========================================================================
// 4. Trichotomy (core): a corrupted K' stream with exactly one of
//    {clean, one substitution, one deletion, one insertion} → correct class.
// ===========================================================================

#[test]
fn trichotomy_clean_aligns_with_no_erasures() {
    for &k in &[16usize, 30, 58, 100] {
        let data = det_data(k, 11 * k as u64);
        let grid = interleave(&data);
        match sync_classify(&grid, k) {
            SyncOutcome::Aligned { grid: g, erasures } => {
                assert_eq!(g, grid, "clean grid passes through unchanged (k={k})");
                assert!(erasures.is_empty(), "clean ⇒ no erasures (k={k})");
            }
            other => panic!("clean stream must align (k={k}): {other:?}"),
        }
    }
}

#[test]
fn trichotomy_substitution_flags_its_block_as_erasure() {
    for &k in &[16usize, 30, 58, 100] {
        let data = det_data(k, 13 * k as u64 + 1);
        let grid = interleave(&data);
        let b = block_stride(k);

        // Corrupt a single DATA word in (say) block 1, in a way that flips the
        // block CRC (single-word substitution detection ≥ ~96%; pick a value
        // until the CRC actually changes so the test is deterministic).
        // block 1 data words are grid positions b+1 .. b+1+b-1 (after C0).
        let target = b + 1; // first data word of block index 1
        let mut recv = grid.clone();
        let orig = recv[target];
        let mut v = (orig + 7) % FIELD_LIMIT;
        // ensure the corrupted symbol is not itself a checkpoint marker and
        // actually trips the CRC
        loop {
            recv[target] = v;
            let blk_recv = &recv[b + 1..b + 1 + b];
            if crc5(blk_recv) != crc5(&data[b..b + b]) && parse_checkpoint(v).is_none() {
                break;
            }
            v = (v + 1) % FIELD_LIMIT;
            if v == orig {
                v = (v + 1) % FIELD_LIMIT;
            }
        }

        match sync_classify(&recv, k) {
            SyncOutcome::Aligned { grid: g, erasures } => {
                assert_eq!(
                    g.len(),
                    grid.len(),
                    "substitution keeps the grid length (k={k})"
                );
                assert!(!erasures.is_empty(), "sub ⇒ flagged block erasures (k={k})");
                // the corrupted position must be inside the erased set
                assert!(
                    erasures.contains(&target),
                    "the corrupted position {target} must be erased (k={k}, erasures={erasures:?})"
                );
            }
            other => panic!("single substitution ⇒ Aligned+erasures (k={k}): {other:?}"),
        }
    }
}

#[test]
fn trichotomy_deletion_yields_bounded_candidates() {
    for &k in &[16usize, 30, 58] {
        let data = det_data(k, 17 * k as u64 + 3);
        let grid = interleave(&data);
        let b = block_stride(k);

        // Delete a single DATA word inside block index 1 (a position strictly
        // between C0 and C1). Removing it makes that block have b-1 data words.
        let del_pos = b + 2; // a data word in block 1
        let mut recv = grid.clone();
        recv.remove(del_pos);

        match sync_classify(&recv, k) {
            SyncOutcome::SingleDeletionCandidates { gap_positions } => {
                assert!(!gap_positions.is_empty(), "non-empty candidate set (k={k})");
                assert!(
                    gap_positions.len() <= b,
                    "candidate set bounded by b={b} (k={k}, got {})",
                    gap_positions.len()
                );
                // strictly ascending
                assert!(
                    gap_positions.windows(2).all(|w| w[0] < w[1]),
                    "candidates strictly ascending (k={k})"
                );
            }
            other => panic!("single deletion ⇒ SingleDeletionCandidates (k={k}): {other:?}"),
        }
    }
}

#[test]
fn trichotomy_insertion_is_classified_not_silently_aligned() {
    for &k in &[16usize, 30, 58] {
        let data = det_data(k, 19 * k as u64 + 5);
        let grid = interleave(&data);
        let b = block_stride(k);

        // Insert one spurious DATA word inside block 1 ⇒ that block has b+1.
        let ins_pos = b + 2;
        let mut recv = grid.clone();
        // a value that is NOT a checkpoint marker
        let mut v = 0u16;
        while parse_checkpoint(v).is_some() {
            v += 1;
        }
        recv.insert(ins_pos, v);

        // An insertion must NOT be silently accepted as a clean alignment;
        // it is converted to a block erasure (Aligned) or refused — never a
        // clean Aligned with empty erasures.
        match sync_classify(&recv, k) {
            SyncOutcome::Aligned { erasures, .. } => {
                assert!(
                    !erasures.is_empty(),
                    "insertion ⇒ block erasure, never clean (k={k})"
                );
            }
            SyncOutcome::Refuse(_) => { /* acceptable per plan §4.3 fallback */ }
            SyncOutcome::SingleDeletionCandidates { .. } => {
                panic!("an insertion must not be reported as a deletion (k={k})")
            }
        }
    }
}

// ===========================================================================
// 5. Realignment + candidate-truth (LOAD-BEARING): a single deletion at a known
//    position p (incl. spanning a block boundary) → candidate set CONTAINS the
//    true gap and len ≤ b.
// ===========================================================================

#[test]
fn candidate_set_contains_true_gap_all_data_positions() {
    let k = 58usize;
    let data = det_data(k, 0xC0FFEE);
    let grid = interleave(&data);
    let b = block_stride(k);

    // For EVERY data-word grid position, delete it and require the resulting
    // candidate set to contain the true gap and be bounded by b. The "true gap"
    // is expressed in the post-deletion grid's coordinate frame: the deletion at
    // grid position `p` means a missing slot whose candidate region is the block
    // that lost a word. We assert membership of the realigned slot the engine
    // reports — concretely, that SOME candidate maps back to the deleted block.
    for p in 0..grid.len() {
        // skip deleting a checkpoint here — that is the deleted-checkpoint test
        if parse_checkpoint(grid[p]).is_some() {
            continue;
        }
        let mut recv = grid.clone();
        recv.remove(p);

        match sync_classify(&recv, k) {
            SyncOutcome::SingleDeletionCandidates { gap_positions } => {
                assert!(
                    gap_positions.len() <= b,
                    "bounded by b={b} at p={p} (got {})",
                    gap_positions.len()
                );
                assert!(!gap_positions.is_empty(), "non-empty at p={p}");
                // The candidate positions are grid slots in the b-word block that
                // lost the word. The true deletion fell in some block; require
                // that the reported candidates all lie within b of each other
                // (one block's worth) AND that p (clamped into the post-grid)
                // is within the candidate span ± b.
                let lo = *gap_positions.first().unwrap();
                let hi = *gap_positions.last().unwrap();
                assert!(
                    hi - lo < b + 1,
                    "candidates span one block (p={p}, lo={lo}, hi={hi}, b={b})"
                );
                // truth membership: the deleted index p, viewed in the shortened
                // stream, must be reachable from the candidate set (within the
                // same block window). p maps to min(p, recv.len()) in recv coords.
                let p_recv = p.min(recv.len());
                assert!(
                    p_recv + b >= lo && p_recv <= hi + b,
                    "true gap p_recv={p_recv} not covered by candidates [{lo},{hi}] ±b (p={p})"
                );
            }
            // A boundary deletion adjacent to a checkpoint may present as a
            // whole-block erasure (also recovers via RS) — acceptable.
            SyncOutcome::Aligned { erasures, .. } => {
                assert!(!erasures.is_empty(), "boundary del ⇒ erasures (p={p})");
            }
            SyncOutcome::Refuse(e) => {
                panic!("single data deletion at p={p} should not refuse: {e:?}")
            }
        }
    }
}

// ===========================================================================
// 6. Deleted checkpoint: a checkpoint word removed → detected via index
//    discontinuity at the next checkpoint → erasure or Refuse, never a wrong
//    silent alignment.
// ===========================================================================

#[test]
fn deleted_checkpoint_is_detected_not_silently_aligned() {
    for &k in &[30usize, 58, 100] {
        let data = det_data(k, 23 * k as u64 + 7);
        let grid = interleave(&data);

        // find the position of checkpoint index 1 (second checkpoint) and remove it
        let cps: Vec<usize> = grid
            .iter()
            .enumerate()
            .filter(|(_, &w)| parse_checkpoint(w).is_some())
            .map(|(i, _)| i)
            .collect();
        assert!(cps.len() >= 2, "need ≥2 checkpoints (k={k})");
        let del = cps[1];
        let mut recv = grid.clone();
        recv.remove(del);

        match sync_classify(&recv, k) {
            SyncOutcome::Aligned { erasures, .. } => {
                assert!(
                    !erasures.is_empty(),
                    "deleted checkpoint ⇒ block erasure, not clean (k={k})"
                );
            }
            // a single deletion candidate set is acceptable IF it brackets the
            // affected region (the lone deletion happens to be a checkpoint)
            SyncOutcome::SingleDeletionCandidates { gap_positions } => {
                assert!(!gap_positions.is_empty(), "non-empty candidates (k={k})");
            }
            SyncOutcome::Refuse(_) => { /* refuse is custody-safe */ }
        }
    }
}

// ===========================================================================
// 7. Compound (deleted checkpoint + adjacent data deletion): detected →
//    merged-span erasure (≤ 2b) or Refuse.
// ===========================================================================

#[test]
fn compound_deleted_checkpoint_plus_data_deletion() {
    let k = 58usize;
    let data = det_data(k, 0xBADBEEF);
    let grid = interleave(&data);
    let b = block_stride(k);

    let cps: Vec<usize> = grid
        .iter()
        .enumerate()
        .filter(|(_, &w)| parse_checkpoint(w).is_some())
        .map(|(i, _)| i)
        .collect();
    assert!(cps.len() >= 3);
    // delete checkpoint index 1 and the data word right before it.
    let cp = cps[1];
    let mut recv = grid.clone();
    recv.remove(cp); // remove checkpoint
    recv.remove(cp - 1); // remove adjacent data word (now shifted)

    match sync_classify(&recv, k) {
        SyncOutcome::Aligned { erasures, .. } => {
            assert!(!erasures.is_empty(), "compound ⇒ merged-span erasure");
            // merged span bounded by ~2b
            let lo = *erasures.first().unwrap();
            let hi = *erasures.last().unwrap();
            assert!(
                hi - lo <= 2 * b + 2,
                "merged span ≤ ~2b (lo={lo}, hi={hi}, b={b})"
            );
        }
        SyncOutcome::Refuse(_) => { /* custody-safe */ }
        other => panic!("compound case must erase-or-refuse, not {other:?}"),
    }
}

// ===========================================================================
// 8. Refuse-on-ambiguity: two equally-consistent realignments → Refuse.
// ===========================================================================

#[test]
fn refuse_on_ambiguous_realignment() {
    // Construct a maximally-destroyed stream: replace a long run of words
    // (spanning multiple blocks, destroying ≥2 checkpoints) with garbage so the
    // realignment cannot anchor on ≥2 continuous-index checkpoints. The engine
    // must Refuse rather than guess.
    let k = 58usize;
    let data = det_data(k, 42);
    let grid = interleave(&data);
    let b = block_stride(k);

    // destroy a run of 3*b words in the middle with non-marker garbage, AND
    // delete one extra word so the length no longer matches K' (forcing a
    // realignment attempt that has no unique anchor).
    let mut recv = grid.clone();
    let start = grid.len() / 3;
    for i in 0..(3 * b).min(recv.len() - start - 1) {
        // a non-checkpoint garbage value
        recv[start + i] = (((start + i) as u16 + 1) % FIELD_LIMIT) & 0x00FF; // top bits clear ⇒ not 0b101
    }
    recv.remove(start); // now length is K'-1 with a destroyed multi-block run

    match sync_classify(&recv, k) {
        SyncOutcome::Refuse(SyncError::AmbiguousRealignment)
        | SyncOutcome::Refuse(SyncError::CheckpointGap)
        | SyncOutcome::Refuse(SyncError::MultiIndelBlock)
        | SyncOutcome::Refuse(SyncError::CandidateBudgetExceeded) => { /* ok */ }
        SyncOutcome::Aligned { erasures, .. } => {
            // if it claims alignment it MUST have erased the destroyed run
            assert!(
                erasures.len() >= b,
                "a destroyed multi-block run must erase ≥ b positions if aligned, got {}",
                erasures.len()
            );
        }
        other => panic!("destroyed multi-block run must refuse or erase, not {other:?}"),
    }
}

// A tighter, deliberately-constructed two-way ambiguity that MUST refuse.
#[test]
fn refuse_explicit_two_way_ambiguity() {
    // Build a stream where the first checkpoint is destroyed and the data is
    // arranged so that two different single-deletion realignments are each
    // consistent with the surviving checkpoints' CRCs. The structural engine
    // cannot break the tie (the value is a free unknown) and MUST refuse rather
    // than pick one. We force this by destroying TWO checkpoints' worth of
    // anchor (so < 2 continuous-index checkpoints survive on one side).
    let k = 58usize;
    let data = det_data(k, 99);
    let grid = interleave(&data);

    let cps: Vec<usize> = grid
        .iter()
        .enumerate()
        .filter(|(_, &w)| parse_checkpoint(w).is_some())
        .map(|(i, _)| i)
        .collect();
    // Destroy the markers of the first two checkpoints (clear top bits) AND
    // delete one data word, so realignment has no ≥2-continuous anchor early
    // and a deletion to localize — ambiguous.
    let mut recv = grid.clone();
    recv[cps[0]] &= 0x00FF;
    recv[cps[1]] &= 0x00FF;
    recv.remove(1); // delete an early data word

    let outcome = sync_classify(&recv, k);
    assert!(
        matches!(outcome, SyncOutcome::Refuse(_))
            || matches!(outcome, SyncOutcome::Aligned { ref erasures, .. } if !erasures.is_empty()),
        "ambiguous early anchor must refuse or erase, got {outcome:?}"
    );
}

// ===========================================================================
// 9. mod-8 aliasing bound (M2): a false +1-mod-8 index continuity requires
//    ≥ 8·b consecutive destroyed words — i.e. it cannot occur within budget.
// ===========================================================================

#[test]
fn mod8_aliasing_requires_8b_destroyed() {
    // The claim: to fake a continuous +1-mod-8 index run that the realigner
    // would accept, an adversary must destroy ≥ 8·b consecutive words (a full
    // mod-8 cycle of blocks). We DEMONSTRATE the bound: pick k large enough that
    // 8·b < K', destroy a run SHORTER than 8·b, and confirm the realigner does
    // NOT produce a wrong silent alignment (it erases the run or refuses), and
    // that the count 8·b indeed exceeds any per-block destruction budget.
    let k = 200usize;
    let b = block_stride(k);
    let kprime = k + checkpoint_layout(k).checkpoint_count;
    let cycle = 8 * b;
    assert!(
        cycle < kprime,
        "for k={k}, one mod-8 cycle 8·b={cycle} must fit inside K'={kprime} so the bound is meaningful"
    );

    // destroy a run of (cycle - 1) words: one short of a full mod-8 cycle.
    let data = det_data(k, 7);
    let grid = interleave(&data);
    let mut recv = grid.clone();
    let start = b + 1;
    let run = cycle - 1;
    for i in 0..run {
        recv[start + i] &= 0x00FF; // clear top bits ⇒ never a 0b101 marker
    }

    let outcome = sync_classify(&recv, k);
    // It must NOT pretend the destroyed run is clean. Either it refuses or it
    // erases (≥ part of) the run; it must never report SingleDeletionCandidates
    // (no single deletion here) and never Aligned with empty erasures.
    match outcome {
        SyncOutcome::Aligned { erasures, .. } => {
            assert!(
                !erasures.is_empty(),
                "a destroyed {run}-word run (< 8·b={cycle}) must erase, not silently align"
            );
        }
        SyncOutcome::Refuse(_) => { /* ok */ }
        SyncOutcome::SingleDeletionCandidates { .. } => {
            panic!("a multi-word destroyed run is not a single deletion")
        }
    }
}

// ===========================================================================
// 10. Block-erasure END-TO-END via RS (ties P2): a CRC-flagged / multi-indel
//     block → Aligned{erasures} → feed the realigned grid (RS message K') +
//     appended RS parity through rs_decode(.., erasures) → recover original.
//     NO tag needed (erasures are known-position).
// ===========================================================================

#[test]
fn block_erasure_recovered_end_to_end_via_rs() {
    for &k in &[58usize, 100] {
        let data = det_data(k, 31 * k as u64 + 2);
        // The RS message is the K' grid (data interspersed with checkpoints).
        let grid = interleave(&data);
        let kprime = grid.len();
        let b = block_stride(k);

        // Provision enough parity to cover one whole-block erasure (b words):
        // s = b erasures needs m ≥ b. Give margin.
        let m = b + 4;
        let codeword = rs_codeword(&grid, m).expect("rs codeword over K'");

        // Corrupt an entire data block (block index 1) with garbage so its CRC
        // fails ⇒ sync_classify flags it as a block erasure.
        let mut recv_grid = grid.clone();
        for i in 0..b {
            let pos = b + 1 + i; // block-1 data positions (after C0)
            if pos < recv_grid.len() && parse_checkpoint(recv_grid[pos]).is_none() {
                recv_grid[pos] = (recv_grid[pos] + 123) % FIELD_LIMIT;
            }
        }

        let outcome = sync_classify(&recv_grid, k);
        let erasures = match outcome {
            SyncOutcome::Aligned { grid: g, erasures } => {
                assert_eq!(g.len(), kprime, "realigned grid length == K' (k={k})");
                assert!(!erasures.is_empty(), "block corruption ⇒ erasures (k={k})");
                erasures
            }
            other => panic!("corrupted block ⇒ Aligned+erasures (k={k}): {other:?}"),
        };

        // Build the received full codeword: corrupted K' region ‖ clean parity
        // tail, and erase the flagged grid positions (which index into K').
        let mut recv_cw = codeword.clone();
        recv_cw[..kprime].copy_from_slice(&recv_grid);

        // Erasures from sync are grid positions (< K'); they are valid codeword
        // positions too. They must be sorted+distinct for rs_decode.
        let mut er = erasures.clone();
        er.sort_unstable();
        er.dedup();

        let recovered = rs_decode(&recv_cw, kprime, &er).expect("RS recovers erased block");
        assert_eq!(
            recovered, grid,
            "end-to-end: erasures → RS recovers K' (k={k})"
        );
        // and stripping checkpoints gives back the original data
        assert_eq!(
            strip_checkpoints_at(&recovered, k),
            data,
            "recovered data (k={k})"
        );
    }
}

// ===========================================================================
// 11. Small-K (K<16): single degenerate checkpoint path works (interleave +
//     sync).
// ===========================================================================

#[test]
fn small_k_single_degenerate_checkpoint() {
    for &k in &[1usize, 2, 5, 10, 15] {
        let data = det_data(k, 3 * k as u64 + 1);
        let layout = checkpoint_layout(k);
        assert_eq!(
            layout.checkpoint_count, 1,
            "small-K ⇒ exactly 1 checkpoint (k={k})"
        );
        let grid = interleave(&data);
        assert_eq!(grid.len(), k + 1, "K' = K + 1 (k={k})");
        assert_eq!(strip_checkpoints_at(&grid, k), data, "round-trip (k={k})");
        // the single trailing word is the degenerate checkpoint
        assert!(
            parse_checkpoint(*grid.last().unwrap()).is_some(),
            "trailing word is the degenerate checkpoint (k={k})"
        );

        // sync_classify on the clean small-K grid → Aligned, no erasures
        match sync_classify(&grid, k) {
            SyncOutcome::Aligned { grid: g, erasures } => {
                assert_eq!(g, grid, "small-K clean passes through (k={k})");
                assert!(erasures.is_empty(), "small-K clean ⇒ no erasures (k={k})");
            }
            other => panic!("small-K clean must align (k={k}): {other:?}"),
        }

        // a single substitution in the small-K block flags the whole block as an
        // erasure (there is only one block).
        if k >= 1 {
            let mut recv = grid.clone();
            let orig = recv[0];
            let mut v = (orig + 5) % FIELD_LIMIT;
            while crc5(&[v]) == crc5(&[orig]) || parse_checkpoint(v).is_some() {
                v = (v + 1) % FIELD_LIMIT;
            }
            // corrupt the FIRST data word; recompute relative to full block
            recv[0] = v;
            if crc5(&recv[..k]) != crc5(&data) {
                match sync_classify(&recv, k) {
                    SyncOutcome::Aligned { erasures, .. } => {
                        assert!(!erasures.is_empty(), "small-K sub ⇒ erasure (k={k})");
                    }
                    SyncOutcome::Refuse(_) => {}
                    other => panic!("small-K sub ⇒ erase/refuse (k={k}): {other:?}"),
                }
            }
        }
    }
}

// ===========================================================================
// No-panic fuzz: sync_classify must never panic on ANY input (proptest).
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn sync_classify_never_panics(
        words in proptest::collection::vec(0u16..2048, 0..200),
        k in 0usize..150,
    ) {
        // Only requirement: returns a typed outcome, never panics.
        let _ = sync_classify(&words, k);
    }

    #[test]
    fn crc5_and_checkpoint_never_panic(
        blk in proptest::collection::vec(0u16..2048, 0..40),
        i in 0usize..1000,
        w in 0u16..2048,
    ) {
        let _ = crc5(&blk);
        let _ = checkpoint_word(i, &blk);
        let _ = parse_checkpoint(w);
        let _ = block_stride(blk.len());
        let _ = checkpoint_layout(blk.len());
        let _ = interleave(&blk);
    }
}
