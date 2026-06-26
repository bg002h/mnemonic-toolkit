# Word-Card P3 — R0 review, round 2

- **Phase:** P3 — structural sync / checkpoint layer (`crates/wc-codec/src/sync.rs`, `tests/sync.rs`).
- **Branch/commit:** `feat/wc-p3-sync` @ `82f1dc6a` (fold of `8ac8e811`; parent `master@5b8f1b77`, NOT merged).
- **Round-1 review:** `design/agent-reports/word-card-p3-r0-round-1.md` (RED — 1C/0I/3 Minor; the C1 candidate-truth violation).
- **Reviewer:** opus architect, independent adversarial R0 round 2 (own PRNG + own RS-backed truth oracle, throwaway harness `tests/zz_r0_round2.rs`, deleted after; branch left clean).
- **Gate:** 0 Critical / 0 Important to merge P3 and start P4.
- **Mandate:** verify C1 is GENUINELY closed; re-run the killer fuzz independently; do not trust the implementer's report.

---

## Verdict

**GREEN — 0 Critical / 0 Important / 0 new Minor.**

C1 is genuinely closed. The fold replaced the unsound greedy H0-before-H1 walk with a
**global per-block validator** (`deletion_hypothesis_valid`) that is both provably sound
(the true block ALWAYS validates; truth can never be excluded) and empirically clean under
my own independent re-fuzz: **zero truth misses across 248,681 RS-backed single-deletion
cases + 152,413 targeted stray-marker-trigger cases + 103,793 structural cases + 5,000
seeds of the exact round-1 repro position**, including the precise C1 trigger family
(a data word bearing marker `0b101` + matching index + colliding CRC at a block boundary).
The fold introduced **no new defect**: panic-safety, CRC-5 correctness, and all round-1
PASSes still hold; diff hygiene is clean (`wc-codec`-only, no `mlock.rs`/cross-crate/`fmt
--all` collateral); `SyncError` stays alphabetical; full suite + clippy + fmt are green.
The three Minor folds (N1 reinsert+RS KAT, N2 scramble-rate KAT + CRC doc, N3 `Aligned` doc)
are correct and non-vacuous.

**Gate is GREEN. P3 may merge and P4 may start.**

---

## C1 closure — independent re-fuzz (own harness, RS-backed oracle, discarded after)

I wrote my own throwaway harness (`tests/zz_r0_round2.rs`) with an **independent xorshift
PRNG** (different constants), an **independent positional checkpoint-strip**, and an
**operational RS truth oracle** (reinsert placeholder at a candidate → `rs_decode(.., [c])`
→ `strip_checkpoints` → compare to the original data — NOT a window-overlap heuristic).
The contract asserted for every single *data* deletion: the outcome is either
(a) `SingleDeletionCandidates` from which the **true gap reinserts+RS-reconstructs the
original**, (b) a correct `Aligned{erasures}` that RS-recovers, or (c) `Refuse` — but
**NEVER** a candidate set from which no reinsert+RS reconstructs the truth. A single
truth-miss = still Critical.

Results (all in release for the heavy RS sweeps; structural/repro confirmed in both
debug and release):

- **Broad RS-backed sweep (≥100k required):** `total = 248,681` single-data-deletions,
  `k ∈ {1,2,5,10,15,16,17,23,24,30,37,47,58,81,100,130,160,199}`, **two** PRNG families
  (the impl-style xorshift that masked C1 in round 1, **and** my own), every data
  position, each candidate verified by reinsert + `rs_decode`. **classified=248,500,
  refused=181 (~0.073%, genuine adjacent-marker ambiguity — custody-safe), erasure-path=0,
  MISSES = 0.** Zero truth misses.

- **Targeted stray-marker-boundary search (the exact C1 mechanism):** `k=58`, 3,000 grids,
  every data position → `total = 152,413`. Diagnostic: **all 3,000 grids contained a
  stray-marker data word** (a data word bearing `0b101` somewhere — the precise collision
  class C1 exploited), so the trigger condition was present throughout. **MISSES = 0.**

- **Reproduce the round-1 repro specifically (`k=58`, delete grid position 9):** I swept
  5,000 seeds of the impl's xorshift family at `k=58`, exercising `p=9` (the first data
  word of block 1 — exactly the round-1 failing position), with the RS oracle.
  **"ROUND-1 REPRO CLASS (k=58, p=9, 5000 seeds): all recover."** The round-1 word `1315`
  (independently confirmed in Python: marker `0b101`, index_mod8 `1`, crc5 `3` — the exact
  H0 false-positive) can no longer exclude the truth: the global validator guarantees the
  true block validates, and if the stray collision also makes a second block validate, the
  classifier **refuses** rather than silently picking the wrong block.

- **Structural no-wrong-block sweep (fast, no RS, wide k incl. 160/200/255):** `total =
  103,793`, classified=103,695, **`not_in_set = 0`** — whenever the classifier returns
  `SingleDeletionCandidates`, the true gap `p` (clean-K' frame) is in the bounded (≤ b)
  set. Never a wrong block.

- **Small-K degenerate (RS-backed):** `k ∈ 1..15`, 200 seeds each, every data position —
  **0 misses.**

**Why it is now sound (structural proof I verified by hand against the code).**
`deletion_hypothesis_valid(recv, geom, j)` reads, for a hypothesised true block `j`:
blocks `i<j` at their clean positions (gap is after them), block `j`'s checkpoint at
`cp_pos_j−1` (CRC not checked — gap is a free unknown inside), and blocks `i>j` shifted
left by 1. For a *genuine* deletion in block `j` this reproduces every other block's data
and checkpoint **exactly**, so the true block ALWAYS passes — the truth can never be
excluded. A wrong hypothesis `j'≠j` garbles the data between `j'` and the true gap, so some
checkpoint CRC/index fails. `classify_single_deletion` collects ALL passing blocks; since
the true block always passes, the passing set always contains it. One passing block ⇒ it is
the true block ⇒ candidates (its ≤ b data slots) contain the truth. Two passing blocks ⇒ a
genuine unbreakable adjacent-marker tie ⇒ `Refuse(AmbiguousRealignment)` (never a silent
pick). Zero ⇒ the missing word was a checkpoint ⇒ defer to `classify_deleted_checkpoint`.

**C1 is closed — genuinely, not papered over.**

---

## Validator soundness (verification item #2)

Read `deletion_hypothesis_valid` (sync.rs:455-494) line by line:

- **Recv-shift logic is correct.** `cp_shift = (i >= except_block) ? 1 : 0`;
  `data_shift = (i > except_block) ? 1 : 0`. The implicated block's checkpoint shifts (gap
  precedes it) but its data prefix is not CRC-checked (`i == except_block` → `continue`),
  so the data_shift asymmetry at `i == except_block` is irrelevant — exactly right. Blocks
  before the gap: unshifted. Blocks after: −1. The gap-bearing block's checkpoint: −1.
- **No off-by-one at first/last block.** First block (`i=0`): if it is the implicated
  block, `cp_shift=1` reads `recv[cp_pos_0−1]`; underflow guarded by
  `if g.cp_pos < cp_shift { return false }`. Last block: handled identically; the
  candidate set is its `sz` slots regardless of whether the deleted word was the block's
  first or last (the validator never assumes a position within the block).
- **Bounds checks are complete.** `cp_pos < cp_shift`, `cp_pos >= recv.len()`,
  `data_start < data_shift`, and `ds + sz > recv.len()` are all guarded before indexing →
  no panic on any reachable input (confirmed by 50k random panic-fuzz + corners).
- **Small-K single degenerate checkpoint:** `block_geometry` yields one block; the
  validator and the `sz==0` skip in `classify_single_deletion` handle it; small-K RS sweep
  is clean.
- **"Two blocks validate ⇒ Refuse" actually fires.** `classify_single_deletion` collects
  `passing_blocks` and refuses when `len() > 1` (sync.rs:543-545) — it does NOT silently
  pick `[0]`. The 181 refusals in the broad sweep are exactly these genuine ties; none was
  a wrong-block emission.
- **The truth-block is guaranteed to validate in ALL cases**, including deletion of the
  block's first or last word and checkpoint-adjacent words — proven above and confirmed
  empirically (deleting `p=data_start_j` and `p=cp_pos_j−1` are both covered by the
  every-position sweeps with 0 misses).

No soundness gap.

---

## New Critical / Important / Minor

**None.** No new Critical, no new Important, no new Minor introduced by the fold.

I specifically looked for fold-introduced regressions (per the standing lesson that "folds
themselves can introduce drift" and that mechanical fixes have hidden real bugs before):
the classifier was substantially rewritten, so I re-ran the entire round-1 PASS battery
(below) — all hold. The new helper `block_geometry` is a pure layout derivation with no new
panic surface. `deletion_hypothesis_valid` is allocation-free and fully bounds-guarded.

---

## Regression check (round-1 PASSes re-confirmed on 82f1dc6a)

The fold changed the classifier; I re-confirmed every round-1 PASS still holds:

- **No-silent-misalignment.** In-suite coverage (`trichotomy_*`, `refuse_on_ambiguous_realignment`,
  `refuse_explicit_two_way_ambiguity`, `mod8_aliasing_requires_8b_destroyed`,
  `trichotomy_insertion_is_classified_not_silently_aligned`) all GREEN; my structural sweep
  (incl. forced `0b101` markers via the stray-marker grids, and on-checkpoint deletions
  routed to the checkpoint path) produced **0** wrong-block emissions and **0** confident-
  wrong alignments.
- **Block-erasure → RS end-to-end.** `block_erasure_recovered_end_to_end_via_rs` GREEN; my
  RS oracle's `Aligned{erasures}` branch recovered the original on every boundary-erasure
  case (erasure-path count was 0 in the broad sweep because boundary deletions classified
  as candidates here, but the branch is exercised and correct in the implementer's KAT).
- **Deleted-checkpoint / compound.** `deleted_checkpoint_is_detected_not_silently_aligned`,
  `compound_deleted_checkpoint_plus_data_deletion` GREEN — the `passing_blocks.is_empty()`
  path correctly defers to `classify_deleted_checkpoint`.
- **mod-8 aliasing.** `mod8_aliasing_requires_8b_destroyed` GREEN.
- **Panic-safety.** `sync_classify_never_panics` / `crc5_and_checkpoint_never_panic`
  proptests GREEN (re-run at `PROPTEST_CASES=4000`); my 50,000-case random fuzz +
  `k=0/1`, empty, `[0]`, 2000-word, and every `K'±5` length corner — **no panic**.
- **CRC-5 unperturbed.** 20,000 random blocks cross-checked against an independent
  polynomial-long-division reference — **0 mismatches**; `crc5([])=0`, `crc5([0])=0`;
  `checkpoint_word`/`parse_checkpoint` round-trip holds. The fold did not touch `crc5`.

No regression.

---

## New KATs non-vacuous (verification item #4)

- **`candidate_truth_membership_via_reinsert_rs_exhaustive` (N1 regression):** uses the
  operational `reinsert_rs_recover` predicate (reinsert placeholder → `rs_decode(.., [c])`
  → `strip_checkpoints_at` → compare to `data`) — NOT window-overlap. Sweeps
  `k ∈ {16,30,58,100}` × 6 seeds × every data position; asserts `truth_miss == 0` AND
  `classified_del * 50 > total * 49` (so >98% of cases actually exercise the RS predicate —
  non-vacuous). Passes (also re-run standalone, 0.88s release).
- **`candidate_true_gap_in_set_structural_exhaustive`:** structural half, `k ∈
  {16,30,58,100,160,200}` × 60 seeds × every position; asserts `gap_positions.contains(&p)`
  and `len ≤ b` and strictly ascending. Non-vacuous (`classified_del*100 > total*97`).
- **N2 `crc5_whole_block_scramble_passes_at_about_one_in_32`:** measures the whole-block
  scramble CRC-collision rate and asserts it sits in `[0.005, 0.09]` (near `2⁻⁵`) — i.e.
  proves the flag is NOT a guarantee. Correct and non-vacuous. The accompanying `crc5` doc
  (sync.rs:166-175) correctly states the boundary (flag, not integrity; real guarantees =
  RS + P4 tag).
- **N3 `Aligned` doc (sync.rs:97-103):** now says recovery holds "given enough parity
  `m ≥ |erasures|`" and that `rs_decode` "correctly REFUSES (`RsError::Uncorrectable`)"
  below budget. I verified this against `rs_decode` (rs.rs:230-233: `erasures.len() > m ⇒
  Uncorrectable`) — accurate.

---

## Gates / suite results

- **`cargo test -p wc-codec` (full, debug):** **63 passed, 0 failed** across all targets —
  lib 0, field 10, pad 5, regroup 8, rs 12, **sync 23**, wordmap 5, doctests 0.
- **`PROPTEST_CASES=4000` full suite (release):** all targets GREEN, **sync 23 passed** at
  4000 proptest cases.
- **`cargo clippy -p wc-codec --all-targets -- -D warnings`:** clean, **exit 0**.
- **`cargo fmt -p wc-codec --check`:** clean, **exit 0**.
- **Diff hygiene:** fold `8ac8e811..82f1dc6a` = `crates/wc-codec/{src/sync.rs, tests/sync.rs}`
  only; full branch vs master = those plus `src/lib.rs` + `tests/sync.proptest-regressions`.
  **No `mlock.rs`, no other-crate reformat, no `cargo fmt --all` collateral.**
- **`SyncError` alphabetical:** `AmbiguousRealignment, CandidateBudgetExceeded,
  CheckpointGap, MultiIndelBlock` — enum decl + `Display` arms both ordered.
- **Panic-safety:** **0** `unwrap`/`expect`/`panic!`/`unreachable!`/`unsafe` on any code
  line in `sync.rs`.
- **Worktree left clean:** throwaway harness removed; `git status` clean at `82f1dc6a`.

---

## Bottom line

**GREEN — 0 Critical / 0 Important / 0 new Minor.** C1 is genuinely and provably closed:
the global validator structurally guarantees the true gap is always among the candidates
(or the case refuses, never picks wrong), confirmed by 248,681 RS-backed cases + 152,413
targeted-trigger cases + 103,793 structural cases + the exact round-1 repro across 5,000
seeds — **all with zero truth misses**. No fold-introduced regression; all round-1 PASSes
hold; the three Minor folds are correct and non-vacuous; suite/clippy/fmt green; diff
hygiene clean. **P3 may merge and P4 may begin.**
