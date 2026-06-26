//! Structural **sync / checkpoint layer** (plan §4.3, P3).
//!
//! This is the *structural* sync layer ONLY. It builds the checkpoint word
//! codec (marker + block-index mod 8 + CRC-5 local-check), interleaves
//! checkpoints into the data stream (`interleave`), and classifies a
//! possibly-corrupted message-region symbol stream (`sync_classify`) into one
//! of three outcomes: a realigned grid with known-position erasures, a *bounded*
//! set of single-deletion candidate positions, or a refusal.
//!
//! # Boundary with P4 (do NOT cross)
//!
//! P3 does **not** select among the deletion candidates and does **not** carry
//! the integrity tag — those require the P4 non-linear tag as an oracle. P3 only
//! proves the **truth is among the candidates and the candidate set is bounded
//! `≤ b`** (plan §4.3 C1: "the local reinsert-test can't pinpoint a deletion —
//! the missing value is a free unknown"). The whole-block-erasure path *is*
//! fully resolvable here: known-position erasures feed P2's
//! [`crate::rs::rs_decode`] with no tag needed.
//!
//! # Frozen constants (plan §3 / §4.3) — KAT-locked
//!
//! - **Checkpoint word (11 bits):** `marker(3) = 0b101 │ block-index(3, value =
//!   block_index mod 8) │ local-check(5) = CRC-5`.
//! - **CRC-5 generator:** `x⁵ + x² + 1` (primitive) — uniform single-substitution
//!   miss `≤ 2⁻⁵`.
//! - **Stride `b = floor(√K + 0.5)`** (DERIVED from `K`, frozen rounding). Small-K:
//!   `K < 16 ⇒` a single degenerate checkpoint (no interspersing).
//! - **Recognition / realignment:** marker + **≥2-checkpoint mod-8 index
//!   continuity** + CRC. Ambiguity ⇒ refuse-and-report.

/// The 3-bit checkpoint marker, `0b101` (plan §3 / §4.3). Distinct from the
/// stop-sign marker (`0b1111`) and the ledger marker (`0b1110`) so the word
/// classes never alias.
pub const CHECKPOINT_MARKER: u16 = 0b101;

/// The `K < 16` small-K threshold (plan §4.3, Q7): a single degenerate
/// checkpoint, no interspersing.
pub const SMALL_K_THRESHOLD: usize = 16;

/// CRC-5 generator polynomial `x⁵ + x² + 1` as a 6-bit value (bits 5, 2, 0 set)
/// — `0b100101 = 0x25` (plan §3 / §4.3, C2). Primitive ⇒ all single-bit errors
/// detected (period 31), uniform single-substitution miss `≤ 2⁻⁵`.
const CRC5_POLY: u16 = 0b10_0101;

/// Errors from the sync layer. Variants are **alphabetical** (plan / `CLAUDE.md`
/// convention) — kept so from the very first commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncError {
    /// Two (or more) realignments are equally consistent with the marker +
    /// mod-8 index-continuity + CRC evidence; accepting either could silently
    /// mis-align, so we refuse-and-report (plan §4.3).
    AmbiguousRealignment,
    /// The candidate-alignment search budget (number of single-deletion
    /// candidate positions to validate downstream) would be exceeded — refuse
    /// rather than emit an unbounded candidate set (plan §4.3 bound).
    CandidateBudgetExceeded,
    /// A run of missing / unrecognizable words opened a gap between checkpoints
    /// that cannot be bridged by the bounded offset search — the next checkpoint
    /// could not be re-anchored (plan §4.3 / §6.1 fallback).
    CheckpointGap,
    /// A block carries ≥ 2 indels (or a mix this layer cannot localize to a
    /// single bounded candidate set) — it must be handled as a whole-block
    /// erasure, but the surrounding structure also failed to realign cleanly so
    /// we cannot proceed structurally (plan §4.3).
    MultiIndelBlock,
}

impl core::fmt::Display for SyncError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SyncError::AmbiguousRealignment => {
                write!(f, "sync: two equally-consistent realignments — refuse")
            }
            SyncError::CandidateBudgetExceeded => {
                write!(f, "sync: single-deletion candidate set exceeds the bound")
            }
            SyncError::CheckpointGap => {
                write!(f, "sync: checkpoint gap could not be re-anchored")
            }
            SyncError::MultiIndelBlock => {
                write!(f, "sync: >=2 indels in a block could not be localized")
            }
        }
    }
}

impl std::error::Error for SyncError {}

/// The outcome of [`sync_classify`] over a (possibly corrupted) message-region
/// symbol stream (plan §4.3). Exactly one of three.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncOutcome {
    /// The stream realigned to the clean `K'` grid (data interspersed with
    /// checkpoints). `grid` is the realigned `K'`-length symbol stream
    /// (checkpoints included); `erasures` are the **grid positions to erase**
    /// before feeding the RS pass: CRC-flagged corrupt blocks and known gaps
    /// (multi-indel / smudged runs). With these erasures the truth is fully
    /// recoverable by P2's `rs_decode(.., erasures)` (no tag needed).
    Aligned {
        /// The realigned `K'`-length grid (data + checkpoints).
        grid: Vec<u16>,
        /// Grid positions to erase (strictly ascending, distinct).
        erasures: Vec<usize>,
    },
    /// A single in-block deletion was localized to a **bounded** set of candidate
    /// grid positions (`≤ b`). P3 cannot *select* the true one (the missing value
    /// is a free unknown — plan §4.3 C1); it only guarantees the true gap is among
    /// `gap_positions` and that the set is bounded. P4 selects via the global tag.
    SingleDeletionCandidates {
        /// Candidate grid positions where the lone deletion may sit (`len ≤ b`),
        /// strictly ascending; **contains** the true gap position.
        gap_positions: Vec<usize>,
    },
    /// The stream could not be safely realigned (ambiguous / unbounded /
    /// multi-indel beyond this layer) — refuse-and-report (custody-safe).
    Refuse(SyncError),
}

/// Integer nearest-square-root: returns `floor(√k + 0.5)` computed with a pure
/// integer Newton `isqrt` (no float rounding ambiguity, plan §3 frozen rule).
///
/// `b = round(√k)`. Tie-free because `4k = (2m+1)²` is even=odd (impossible), so
/// `√k` is never exactly `m + 0.5`.
fn round_sqrt(k: usize) -> usize {
    if k == 0 {
        return 0;
    }
    // floor(sqrt(k)) via integer Newton's method.
    let mut x = k;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + k / x) / 2;
    }
    let fl = x; // floor(sqrt(k))
                // round: choose fl or fl+1 by comparing k to (fl + 0.5)^2 = fl² + fl + 0.25
                // i.e. round up iff k - fl² > fl  ⇔  k > fl² + fl.
    if k - fl * fl > fl {
        fl + 1
    } else {
        fl
    }
}

/// Checkpoint stride `b = floor(√K + 0.5)`, DERIVED from `K` (plan §3, NEW-I3;
/// frozen rounding). Small-K (`K < 16`) returns `K` itself so there is a single
/// degenerate trailing checkpoint and no interspersing.
pub fn block_stride(k: usize) -> usize {
    if k < SMALL_K_THRESHOLD {
        k
    } else {
        round_sqrt(k)
    }
}

/// CRC-5 over a block's payload-word bits, generator `x⁵ + x² + 1` (plan §4.3,
/// C2). Returns a 5-bit value (`0..=31`). Bit-serial, MSB-first over each word's
/// low 11 bits. Uniform single-substitution miss `≤ 2⁻⁵`; all single-bit errors
/// detected (primitive generator, period 31).
pub fn crc5(block_words: &[u16]) -> u16 {
    // Canonical bit-serial polynomial division by `x⁵ + x² + 1`. The 5-bit
    // register holds the running remainder; each message bit is shifted in
    // (MSB-first) and the polynomial subtracted whenever bit 5 overflows.
    let mut reg: u16 = 0;
    for &w in block_words {
        for bit in (0..11).rev() {
            let in_bit = (w >> bit) & 1;
            reg = (reg << 1) | in_bit; // shift in the next message bit (≤ 6 bits)
            if reg & 0b10_0000 != 0 {
                // bit 5 set ⇒ subtract the generator (x⁵ + x² + 1)
                reg ^= CRC5_POLY;
            }
            reg &= 0b1_1111; // keep the 5-bit remainder
        }
    }
    reg & 0b1_1111
}

/// Build the 11-bit checkpoint word for `block_index` over `block_words`:
/// `marker(3)=0b101 │ (block_index mod 8)(3) │ crc5(block_words)(5)` (plan §4.3).
pub fn checkpoint_word(block_index: usize, block_words: &[u16]) -> u16 {
    let idx = (block_index % 8) as u16;
    let crc = crc5(block_words) & 0b1_1111;
    (CHECKPOINT_MARKER << 8) | (idx << 5) | crc
}

/// A recognized checkpoint word's parsed fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Checkpoint {
    /// The 3-bit block index mod 8 carried in the word.
    pub index_mod8: u16,
    /// The 5-bit CRC-5 local-check carried in the word.
    pub crc5: u16,
}

/// Recognize a checkpoint word by its marker (`0b101` in the top 3 bits). Returns
/// `Some` with the parsed `index_mod8` + `crc5`, or `None` if the marker does not
/// match (it is an ordinary data / parity / stop-sign / ledger word).
pub fn parse_checkpoint(word: u16) -> Option<Checkpoint> {
    if (word >> 8) != CHECKPOINT_MARKER {
        return None;
    }
    Some(Checkpoint {
        index_mod8: (word >> 5) & 0b111,
        crc5: word & 0b1_1111,
    })
}

/// The checkpoint layout for a `K`-data-word stream: the number of checkpoints
/// and the per-block data-word counts (plan §4.3). For `K < 16` this is a single
/// degenerate checkpoint covering all `K` words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointLayout {
    /// The stride `b` (= `K` in the small-K degenerate case).
    pub stride: usize,
    /// The number of checkpoints (= number of blocks).
    pub checkpoint_count: usize,
    /// The number of data words in each block, in order (sums to `K`).
    pub block_sizes: Vec<usize>,
}

/// Compute the [`CheckpointLayout`] for a `K`-data-word stream (plan §4.3).
pub fn checkpoint_layout(k: usize) -> CheckpointLayout {
    if k == 0 {
        // Degenerate: an empty data stream still gets a single (empty-block)
        // checkpoint so the structure is uniform.
        return CheckpointLayout {
            stride: 0,
            checkpoint_count: 1,
            block_sizes: vec![0],
        };
    }
    if k < SMALL_K_THRESHOLD {
        // Single degenerate checkpoint over all K words (no interspersing).
        return CheckpointLayout {
            stride: k,
            checkpoint_count: 1,
            block_sizes: vec![k],
        };
    }
    let b = round_sqrt(k);
    let nblocks = k.div_ceil(b);
    let mut block_sizes = Vec::with_capacity(nblocks);
    let mut remaining = k;
    for _ in 0..nblocks {
        let take = remaining.min(b);
        block_sizes.push(take);
        remaining -= take;
    }
    debug_assert_eq!(remaining, 0);
    CheckpointLayout {
        stride: b,
        checkpoint_count: nblocks,
        block_sizes,
    }
}

/// Interleave checkpoints into `data`: emit `data` with one [`checkpoint_word`]
/// after each block of `b` data words, producing the `K'` stream (plan §4.3).
/// `K` is implied by `data.len()`. Round-trips with checkpoint-stripping.
pub fn interleave(data: &[u16]) -> Vec<u16> {
    let layout = checkpoint_layout(data.len());
    let mut out = Vec::with_capacity(data.len() + layout.checkpoint_count);
    let mut offset = 0usize;
    for (block_index, &sz) in layout.block_sizes.iter().enumerate() {
        let block = &data[offset..offset + sz];
        out.extend_from_slice(block);
        out.push(checkpoint_word(block_index, block));
        offset += sz;
    }
    out
}

// ---------------------------------------------------------------------------
// sync_classify — the structural classifier (plan §4.3).
//
// Model. The clean grid is `block₀ ‖ C₀ ‖ block₁ ‖ C₁ ‖ …` where `Cᵢ` is the
// checkpoint for block `i` (covering that block's data words). We classify a
// received stream by walking it left→right, anchoring on the *recognized*
// checkpoints (marker `0b101`) and comparing the observed inter-checkpoint data
// gaps + mod-8 indices + per-block CRCs against the expected layout.
//
// The walk is **conservative and custody-safe**: any structure it cannot
// reduce to either a clean realignment, a bounded single-deletion candidate
// set, or a whole-block erasure is **refused**, never silently mis-aligned.
//
// Checkpoint recognition is POSITIONAL, not content-only: a *data* word may
// legitimately carry the `0b101` marker pattern (probability 1/8). Every anchor
// test therefore checks both the marker AND that the word sits at a plausible
// layout position with the correct mod-8 index — never trusting a stray marker.
// ---------------------------------------------------------------------------

/// Classify a (possibly corrupted / indel'd) message-region symbol stream into a
/// [`SyncOutcome`] (plan §4.3). `k` is the *expected* data-word count (read from
/// the header in P4; supplied directly here). NEVER panics on any input.
pub fn sync_classify(received_message_region: &[u16], k: usize) -> SyncOutcome {
    let recv = received_message_region;
    let layout = checkpoint_layout(k);
    let b = layout.stride.max(1); // bound for candidate sets / spans
    let kprime = k + layout.checkpoint_count;

    // -------------------------------------------------------------------
    // Length-trichotomy gate. The clean K' is known from the header (`k`).
    // -------------------------------------------------------------------
    let len = recv.len();

    // Helper: validate the FULL clean grid **positionally** (length == K').
    //
    // Checkpoints are POSITIONAL, not content-identified: a data word may
    // legitimately carry the `0b101` marker pattern (probability 1/8), so we must
    // NOT reject a clean grid merely because a data word looks like a checkpoint.
    // We anchor at each expected checkpoint position from the layout and require
    // (a) the word there parses as a checkpoint with the right index, and (b) its
    // CRC matches the block. A mismatching CRC ⇒ erase the whole data block. A
    // checkpoint word whose *own* marker/index is wrong (corrupted control word)
    // ⇒ we cannot trust positional anchoring ⇒ `None` (fall to realignment).
    //
    // Returns `Some(erasures)` (empty == perfectly clean), or `None` if the
    // positional anchors do not hold (mis-structured ⇒ try realignment).
    fn validate_clean_grid(recv: &[u16], layout: &CheckpointLayout) -> Option<Vec<usize>> {
        let mut erasures = Vec::new();
        let mut offset = 0usize;
        // Count how many checkpoint anchors are *intact* (right marker+index). A
        // corrupted checkpoint word is part of the RS message and can be erased,
        // but if too many anchors are broken we cannot trust positional layout.
        let mut broken_anchors = 0usize;
        for (block_index, &sz) in layout.block_sizes.iter().enumerate() {
            let cp_pos = offset + sz;
            if cp_pos >= recv.len() {
                return None;
            }
            let block = &recv[offset..offset + sz];
            match parse_checkpoint(recv[cp_pos]) {
                Some(cp) if cp.index_mod8 == (block_index % 8) as u16 => {
                    // Intact anchor: validate the block CRC.
                    if crc5(block) != cp.crc5 {
                        erasures.extend(offset..offset + sz);
                    }
                }
                _ => {
                    // The checkpoint control word itself is corrupted (wrong
                    // marker or wrong index). Erase the checkpoint position AND
                    // its block (we cannot CRC-check a block whose checkpoint is
                    // unreadable). Count it as a broken anchor.
                    broken_anchors += 1;
                    erasures.extend(offset..=cp_pos);
                }
            }
            offset = cp_pos + 1;
        }
        // The whole grid must be consumed exactly.
        if offset != recv.len() {
            return None;
        }
        // If MOST anchors are broken we have lost positional trust ⇒ realign.
        // One broken anchor in a multi-checkpoint grid is fine (erased + RS).
        let total = layout.checkpoint_count;
        if total >= 2 && broken_anchors * 2 > total {
            return None;
        }
        Some(erasures)
    }

    // === Case A: length == K' — clean or substitution(s), no indel. =====
    if len == kprime {
        if let Some(erasures) = validate_clean_grid(recv, &layout) {
            return SyncOutcome::Aligned {
                grid: recv.to_vec(),
                erasures,
            };
        }
        // length matches but the grid is misaligned (a corrupted checkpoint, or
        // a swap that destroyed a checkpoint and produced a stray marker). Fall
        // through to the realignment walk below.
    }

    // === Case B: length == K' - 1 — a single deletion (data word OR a
    //            checkpoint). Localize to a bounded candidate set, or erase. ===
    if len + 1 == kprime {
        return classify_single_deletion(recv, &layout, b);
    }

    // === Case C: length == K' + 1 — a single insertion. Convert the affected
    //            block to a whole-block erasure on the realigned grid. ========
    if len == kprime + 1 {
        return classify_single_insertion(recv, &layout, b);
    }

    // === Case D: any other length delta — multi-indel / heavy damage.
    //            Attempt a conservative block-erasure realignment, else refuse. =
    classify_heavy(recv, &layout, b)
}

/// `len == K' − 1`: exactly one symbol is missing. We anchor checkpoints
/// **positionally** left-to-right (collision-resistant: a data word matching the
/// marker is ignored unless it sits at a plausible anchor with the right index +
/// CRC). A data-word deletion makes exactly one block short by one — emit its
/// bounded candidate set. A *checkpoint* deletion shifts the anchor chain — fall
/// to the deleted-checkpoint path (erase the merged span) or refuse.
fn classify_single_deletion(recv: &[u16], layout: &CheckpointLayout, b: usize) -> SyncOutcome {
    // Left-to-right anchored walk. `start` is the current block's first data
    // position in `recv`. For each expected block we test two hypotheses:
    //   H0: the checkpoint is at `start + sz` (block intact, deletion is LATER)
    //   H1: the checkpoint is at `start + sz - 1` (THIS block lost a data word)
    // validated by the right index AND a CRC match over the candidate block.
    let mut start = 0usize;
    for (block_index, &sz) in layout.block_sizes.iter().enumerate() {
        let want_idx = (block_index % 8) as u16;

        // H0 — intact block.
        let cp0 = start + sz;
        let h0_ok = cp0 < recv.len()
            && parse_checkpoint(recv[cp0]).is_some_and(|cp| {
                cp.index_mod8 == want_idx && crc5(&recv[start..start + sz]) == cp.crc5
            });

        // H1 — this block is short by one (the deletion happened here).
        let cp1 = if sz >= 1 { start + sz - 1 } else { usize::MAX };
        let h1_ok = sz >= 1
            && cp1 < recv.len()
            && parse_checkpoint(recv[cp1]).is_some_and(|cp| cp.index_mod8 == want_idx);

        if h0_ok {
            // Block intact; continue past its checkpoint.
            start = cp0 + 1;
            continue;
        }

        if h1_ok {
            // The deletion is in THIS block. The shortened block occupies grid
            // positions `start..cp1` (sz-1 data words). The true missing slot is
            // any of the `sz` interior positions — bounded by `b` (block width).
            // We emit grid candidate positions `start..=cp1` (the sz slots where
            // a word could have been removed: before each surviving data word and
            // after the last).
            let cands: Vec<usize> = (start..=cp1).collect();
            if cands.len() > b {
                // Should not happen (sz ≤ b), but stay safe.
                return SyncOutcome::Refuse(SyncError::CandidateBudgetExceeded);
            }
            return SyncOutcome::SingleDeletionCandidates {
                gap_positions: cands,
            };
        }

        // Neither hypothesis holds at this block ⇒ the anomaly is not a plain
        // single-data-deletion (likely a deleted checkpoint, or heavier damage).
        // Hand off to the deleted-checkpoint detector.
        return classify_deleted_checkpoint(recv, layout, b);
    }

    // Walked every block without finding the short one — but the length is K'−1,
    // so a symbol IS missing. It must be a checkpoint (the final one, or one we
    // could not anchor). Defer to the deleted-checkpoint path.
    classify_deleted_checkpoint(recv, layout, b)
}

/// `len == K' − 1` but a clean single-data-deletion shape was NOT found ⇒ the
/// missing symbol was (most likely) a *checkpoint*. We re-anchor **positionally**
/// from the left: blocks before the deleted checkpoint anchor at their expected
/// positions (intact). At the deleted-checkpoint block `j`, block `j`'s data and
/// block `j+1`'s data merge across the vanished checkpoint slot, so block
/// `j+1`'s checkpoint appears one position EARLY (index `(j+1) mod 8`, skipping
/// `j mod 8`). We erase that merged span (≤ ~2b) and align. If we cannot find a
/// unique re-anchor downstream ⇒ refuse.
fn classify_deleted_checkpoint(recv: &[u16], layout: &CheckpointLayout, b: usize) -> SyncOutcome {
    // Forward positional walk. `start` = current block's first data position.
    let mut start = 0usize;
    let mut block_index = 0usize;
    while block_index < layout.block_sizes.len() {
        let sz = layout.block_sizes[block_index];
        let want_idx = (block_index % 8) as u16;
        let cp_pos = start + sz;

        let intact = cp_pos < recv.len()
            && parse_checkpoint(recv[cp_pos]).is_some_and(|cp| cp.index_mod8 == want_idx);

        if intact {
            start = cp_pos + 1;
            block_index += 1;
            continue;
        }

        // Anchor failed at block `block_index`. Hypothesis: THIS block's
        // checkpoint was deleted, merging it with block `block_index+1`. Block
        // `j+1`'s checkpoint (index `(j+1) mod 8`) should now appear at
        //   start + sz_j + sz_{j+1}   (one earlier than the clean layout).
        if block_index + 1 < layout.block_sizes.len() {
            let next_idx = ((block_index + 1) % 8) as u16;
            let sz_next = layout.block_sizes[block_index + 1];
            let merged_cp = start + sz + sz_next; // shifted-left by the missing cp
            let next_ok = merged_cp < recv.len()
                && parse_checkpoint(recv[merged_cp]).is_some_and(|cp| cp.index_mod8 == next_idx);
            if next_ok {
                // The deleted checkpoint sat between block `j`'s data and block
                // `j+1`'s data. Restore the K' grid by REINSERTING a placeholder
                // at the vanished checkpoint slot (= start + sz), then erase the
                // merged span: block `j` data + the placeholder checkpoint +
                // block `j+1` data (≤ ~2b+1 slots). The value at the placeholder
                // and the merged data are free unknowns the RS pass recovers.
                let cp_slot = start + sz; // where the checkpoint should be in K'
                let mut grid = recv.to_vec();
                if cp_slot > grid.len() {
                    return SyncOutcome::Refuse(SyncError::CheckpointGap);
                }
                grid.insert(cp_slot, 0); // placeholder → grid is now K' long
                                         // Erase block-j data, the placeholder cp, block-(j+1) data.
                let lo = start;
                let hi = merged_cp; // in the reinserted grid, the next cp is at
                                    // merged_cp+1; the merged-data span ends at merged_cp.
                if hi >= grid.len() || hi < lo {
                    return SyncOutcome::Refuse(SyncError::CheckpointGap);
                }
                if hi - lo > 2 * b + 2 {
                    return SyncOutcome::Refuse(SyncError::MultiIndelBlock);
                }
                let erasures: Vec<usize> = (lo..=hi).collect();
                return SyncOutcome::Aligned { grid, erasures };
            }
            // Could not re-anchor at the next checkpoint ⇒ ambiguous/heavier.
            return SyncOutcome::Refuse(SyncError::AmbiguousRealignment);
        }

        // The FINAL block's checkpoint was deleted (no block after it). Restore
        // K' by appending a placeholder checkpoint at the end, then erase the
        // trailing block data + that placeholder (≤ b+1).
        let mut grid = recv.to_vec();
        let lo = start;
        grid.push(0); // placeholder trailing checkpoint → grid is K' long
        let hi = grid.len().saturating_sub(1);
        if hi < lo {
            return SyncOutcome::Refuse(SyncError::CheckpointGap);
        }
        if hi - lo > b + 2 {
            return SyncOutcome::Refuse(SyncError::MultiIndelBlock);
        }
        let erasures: Vec<usize> = (lo..=hi).collect();
        return SyncOutcome::Aligned { grid, erasures };
    }

    // Walked all blocks with every anchor intact but length is K'−1 — should not
    // happen; refuse rather than fabricate.
    SyncOutcome::Refuse(SyncError::AmbiguousRealignment)
}

/// `len == K' + 1`: exactly one symbol was inserted. We anchor **positionally**
/// left-to-right (H0: checkpoint at `start+sz`, block intact; H1: checkpoint at
/// `start+sz+1`, THIS block is long by one). The over-long block is erased as a
/// whole-block erasure on a K'-trimmed grid (we cannot know which inserted slot
/// is spurious without the tag; RS recovers the block). If unanchorable ⇒ refuse.
fn classify_single_insertion(recv: &[u16], layout: &CheckpointLayout, b: usize) -> SyncOutcome {
    let mut start = 0usize;
    for (block_index, &sz) in layout.block_sizes.iter().enumerate() {
        let want_idx = (block_index % 8) as u16;

        // H0 — intact block.
        let cp0 = start + sz;
        let h0_ok = cp0 < recv.len()
            && parse_checkpoint(recv[cp0]).is_some_and(|cp| {
                cp.index_mod8 == want_idx && crc5(&recv[start..start + sz]) == cp.crc5
            });
        if h0_ok {
            start = cp0 + 1;
            continue;
        }

        // H1 — this block is long by one (the insertion landed here).
        let cp1 = start + sz + 1;
        let h1_ok = cp1 < recv.len()
            && parse_checkpoint(recv[cp1]).is_some_and(|cp| cp.index_mod8 == want_idx);
        if h1_ok {
            // Trim the grid back to K' by dropping ONE word from the over-long
            // block (we cannot know which is spurious), then erase the whole
            // block's data so the RS pass recovers it.
            let mut grid = recv.to_vec();
            grid.remove(cp1 - 1); // drop the last data slot of the long block
            let erasures: Vec<usize> = (start..start + sz).collect();
            if erasures.len() > b + 2 {
                return SyncOutcome::Refuse(SyncError::MultiIndelBlock);
            }
            return SyncOutcome::Aligned { grid, erasures };
        }

        // Neither hypothesis at this block ⇒ heavier/ambiguous.
        return classify_heavy(recv, layout, b);
    }

    classify_heavy(recv, layout, b)
}

/// Heavy-damage / multi-indel fallback (plan §4.3). Custody-safe contract:
///
/// - We only ever produce an [`SyncOutcome::Aligned`] when the received stream is
///   already the exact `K'` length (so a valid RS message exists) AND we can
///   positionally anchor enough checkpoints to *account* for the damage by
///   whole-block erasures. A corrupted checkpoint word or a CRC-failing block is
///   erased; everything must reconcile to a clean `K'` grid with bounded
///   erasures.
/// - Any length mismatch we could not localize to a single bounded indel
///   (`|len − K'| > 1`, or a same-length stream we cannot positionally anchor)
///   is **refused** — P4 handles multi-indel candidate reconstruction with the
///   global tag; P3 never fabricates a wrong silent alignment.
fn classify_heavy(recv: &[u16], layout: &CheckpointLayout, b: usize) -> SyncOutcome {
    let kprime = recv_kprime_target(layout);

    // A length mismatch beyond a single localized indel is out of P3's scope.
    if recv.len() != kprime {
        return SyncOutcome::Refuse(SyncError::MultiIndelBlock);
    }

    // Same length as K': positionally anchor block-by-block. Each block's
    // checkpoint must sit at its expected slot with the right index; a block
    // whose checkpoint is unreadable OR whose CRC fails is erased. We require a
    // *majority* of intact anchors (else positional trust is lost ⇒ refuse).
    let mut erasures: Vec<usize> = Vec::new();
    let mut offset = 0usize;
    let mut broken_anchors = 0usize;
    let mut bad = false;
    for (block_index, &sz) in layout.block_sizes.iter().enumerate() {
        let cp_pos = offset + sz;
        if cp_pos >= recv.len() {
            return SyncOutcome::Refuse(SyncError::CheckpointGap);
        }
        let block = &recv[offset..offset + sz];
        match parse_checkpoint(recv[cp_pos]) {
            Some(cp) if cp.index_mod8 == (block_index % 8) as u16 => {
                if crc5(block) != cp.crc5 {
                    erasures.extend(offset..offset + sz);
                    bad = true;
                }
            }
            _ => {
                broken_anchors += 1;
                erasures.extend(offset..=cp_pos);
                bad = true;
            }
        }
        offset = cp_pos + 1;
    }
    if offset != recv.len() {
        return SyncOutcome::Refuse(SyncError::CheckpointGap);
    }
    // If too many anchors are broken, positional trust is lost — a same-length
    // stream could be a multi-indel that happens to reconcile in length. Refuse.
    let total = layout.checkpoint_count;
    if total >= 2 && broken_anchors * 2 > total {
        return SyncOutcome::Refuse(SyncError::AmbiguousRealignment);
    }
    if !bad {
        // Same length, all anchors intact, all CRCs pass — this is actually a
        // clean grid (the caller reached `heavy` only on a structural mismatch
        // it could not localize). Treat as clean.
        return SyncOutcome::Aligned {
            grid: recv.to_vec(),
            erasures: Vec::new(),
        };
    }
    let _ = b;
    erasures.sort_unstable();
    erasures.dedup();
    SyncOutcome::Aligned {
        grid: recv.to_vec(),
        erasures,
    }
}

/// The clean `K'` length from a layout (data + checkpoints).
fn recv_kprime_target(layout: &CheckpointLayout) -> usize {
    layout.block_sizes.iter().sum::<usize>() + layout.checkpoint_count
}
