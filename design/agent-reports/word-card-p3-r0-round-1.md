# Word-Card P3 — R0 review, round 1

- **Phase:** P3 — structural sync / checkpoint layer (`crates/wc-codec/src/sync.rs`, `tests/sync.rs`).
- **Branch/commit:** `feat/wc-p3-sync` @ `8ac8e811` (parent `master@5b8f1b77`, NOT merged).
- **Reviewer:** opus architect, independent adversarial R0 (own fuzz harnesses, discarded after).
- **Gate:** 0 Critical / 0 Important to merge P3 and start P4.
- **Spec:** Plan §4.3 (`design/IMPLEMENTATION_PLAN_word_card_encoding.md`).

---

## Verdict

**RED — 1 Critical / 0 Important / 3 Minor-Nit.**

The phase is *mostly* excellent: panic-safety, CRC-5 correctness, stride math, the
no-silent-misalignment guarantee, and the block-erasure→RS end-to-end tie to P2 all hold
under heavy independent fuzzing. But the **load-bearing candidate-truth guarantee is
violated**: for a non-negligible fraction of single *data* deletions, `sync_classify`
returns a `SingleDeletionCandidates` set that does **not contain the true gap** and from
which **no candidate reconstructs the original word** — i.e. a single benign deletion can
render the real word unrecoverable. This is exactly the Critical class the prompt's
verification item #1 defines. **Implementation MUST NOT advance to P4 until this is fixed and
re-reviewed to GREEN.**

---

## Critical

### C1 — `classify_single_deletion` H0-before-H1 greedy walk yields a candidate set that EXCLUDES the true gap (truth-absence ⇒ unrecoverable word)

**File:** `crates/wc-codec/src/sync.rs:400-450` (`classify_single_deletion`), specifically the
H0 test+accept at **lines 410-427** evaluated *before* H1 (lines 417-421).

**What the contract requires (plan §4.3 / prompt #1).** For a single in-block data deletion,
the emitted `SingleDeletionCandidates.gap_positions` must **contain the true gap position**
and be bounded `≤ b`. P4 selects the truth from this set via the global tag; if the truth is
absent, the word is unrecoverable.

**The defect.** The left-to-right anchored walk tests two hypotheses per block:
- H0 (block intact): `recv[cp0]` with `cp0 = start+sz` parses as a checkpoint with the right
  `index_mod8` **and** `crc5(recv[start..start+sz]) == cp.crc5`.
- H1 (this block lost a word): `recv[cp1]` with `cp1 = start+sz-1` parses as a checkpoint with
  the right `index_mod8`.

H0 is checked first and short-circuits (`if h0_ok { start = cp0+1; continue; }`, line 423).
When the deletion is in block `j`, the real (shifted-left) checkpoint `Cj` sits at
`recv[start+sz-1]` (= `cp1`, where H1 would correctly fire). But a **data word that
coincidentally bears the `0b101` marker** can sit at `cp0 = start+sz`; if that stray word's
`index_mod8` equals `j mod 8` **and** its low-5 CRC field equals `crc5(recv[start..start+sz])`,
**H0 passes falsely**, the walk declares block `j` intact, advances past the real `Cj`, and
fires H1 one block too late — emitting candidates for the **wrong block**, disjoint from the
true gap.

**Reproduced, RS-verified, and hand-cross-checked** (`k=58`, my-harness seed `9`, delete grid
position `9` = first data word of block 1):
- `recv[17] = 1315`: bears marker `0b101`, `index_mod8 = 1` (= `want_idx`), CRC field `= 3`,
  and `crc5(recv[9..17]) = 3` — **H0 false-positive** (verified independently in Python).
- The real shifted `C1` is at `recv[16]` (where H1 would have fired).
- `sync_classify` returns `SingleDeletionCandidates { gap_positions: [18,19,20,21,22,23,24,25] }`
  — block 2's region.
- **`candidate set CONTAIN true gap p=9? false`**; **`ANY candidate reconstructs the true
  grid via reinsert+RS? false`**; yet **reinserting at the true position 9 reconstructs the
  grid (`true`)** — so the data *is* recoverable, but P3 hands P4 a set that excludes it.

**Frequency.** Independent fast classifier over **164,400** single-data-deletions
(`k ∈ {16,17,24,37,58,81,100,130,160,199}`, 200 seeds each) found **74** disjoint-window
cases (~0.045%, ≈1/2200). The RS-backed harness (`fuzz_candidate_truth_every_position`,
25,840 cases) FAILED on the same class. The per-deletion probability is roughly
`P(stray marker) · P(index match) · P(CRC match) ≈ 1/8 · 1/8 · 1/32 = 2⁻¹¹`, accumulated over
the blocks the walk traverses and adversarially reachable on demand. For a recoverability
guarantee on a *single* deletion this is far too frequent.

**Why the implementer's KATs missed it.** `candidate_set_contains_true_gap_all_data_positions`
(tests/sync.rs:361) tests **only `k=58` with the single seed `0xC0FFEE`** and asserts a *loose*
window-overlap (`p_recv + b >= lo && p_recv <= hi + b`) rather than reinsert+RS truth
membership. That seed happens not to place a colliding stray-marker word at a checkpoint slot,
so the bug is seed-masked. (Note: even the loose assertion would have *failed* for the
`seed=9,p=9` case — `9 + 8 = 17 ≥ 18` is false — confirming it is in-scope of the intended
contract; only the single fixed seed hid it.)

**Direction (architect, not prescriptive code).** The greedy H0-first accept is unsound when
`cp0` may be a stray marker. The walk must disambiguate the H0/H1 conflict rather than trust a
single marker+index+CRC coincidence. Options, in rough order of preference:
1. **Conflict-aware anchoring:** when, at a block where a length deficit is in play, *both*
   `cp1` and `cp0` parse as checkpoints with the right index, the engine cannot locally tell
   which is the true `Cj` (the deleted value is a free unknown — CRC can't break the tie).
   Emit the **union** of both candidate windows (and refuse if that exceeds the `b` budget),
   or refuse — never silently pick H0.
2. **Global anchor-count / final-index consistency:** validate the *total* recognized-checkpoint
   count and the trailing checkpoint index against the layout before trusting any intermediate
   H0; a stray-marker H0 acceptance desynchronizes the downstream index chain, which a global
   continuity check would catch.
3. At minimum, when H1's anchor at `cp1` is a recognized checkpoint with the right index, do
   **not** let a *same-index* H0 marker at `cp0` override it — prefer (or union) the H1
   alignment, since H0's CRC validation over a window that includes a shifted foreign word is
   not trustworthy evidence of "intact".

Whatever the chosen mechanism, the post-fix KAT must assert **reinsert+RS truth-membership**
(not loose window overlap) across **many seeds and `k` values** (incl. small-K and the large-K
derived-`b` regime), at every data position — i.e. fold the independent harness's criterion
into the suite.

---

## Important

None.

---

## Minor / Nit

- **N1 — single-seed / loose-assertion candidate-truth KAT (tests/sync.rs:361).** Even after
  C1 is fixed, the existing `candidate_set_contains_true_gap_all_data_positions` is too weak to
  guard the guarantee: one seed, and window-overlap rather than reinsert+RS membership. Replace
  its truth check with the operational `reinsert-placeholder → rs_decode([c]) == grid` predicate
  and sweep multiple seeds × multiple `k`. (This is the regression test for C1.)
- **N2 — `trichotomy_substitution_flags_its_block_as_erasure` (tests/sync.rs:241) and
  `block_erasure_recovered_end_to_end_via_rs` (:639) only "work" by construction-loops that
  search for a CRC-tripping corruption.** That is correct and deliberate, but it quietly hides
  the inherent `~2⁻⁵` CRC-5 collision rate for whole-block scrambles (independently confirmed:
  full-block multi-word corruption preserves the 5-bit CRC ~1/32 of the time, so the *local*
  check alone does not flag it). This is *expected* per plan §3/§4.3 (the block-erasure flag is
  best-effort; RS + the P4 global tag are the real guarantee), but a one-line KAT or doc note
  asserting "whole-block scramble may pass the local CRC at ≤2⁻⁵; correctness rests on RS+tag,
  not the checkpoint CRC" would make the boundary explicit and prevent a future reader from
  over-trusting the checkpoint flag. (No code defect.)
- **N3 — `SyncOutcome::Aligned` doc (sync.rs:99) says "the truth is fully recoverable by
  rs_decode".** True only when enough parity is provisioned for `|erasures|` (an RS budget
  precondition the P2 layer owns). The sentence reads as an unconditional guarantee; a clause
  "given parity `m ≥ |erasures|`" would be precise. (Doc-only.)

---

## Suite results

- `cargo test -p wc-codec` (master baseline of the branch): **61 passed, 0 failed** across all
  targets (lib 10, field/poly/regroup/pad/rs unit + integration, `sync.rs` 21, `wordmap.rs` 5,
  doctests 0). The implementer's own P3 KATs are GREEN — but they do **not** exercise the C1
  failure (single seed + loose assertion; see C1/N1).
- `cargo clippy -p wc-codec --all-targets -- -D warnings`: **clean (exit 0).**
- `cargo fmt -p wc-codec --check`: **clean (exit 0).**
- **Diff hygiene:** `git diff --name-only master..HEAD` = `crates/wc-codec/{src/lib.rs,
  src/sync.rs, tests/sync.rs, tests/sync.proptest-regressions}` only. **No `mlock.rs`,
  no other-crate reformat, no `cargo fmt --all` collateral.** Clean.
- **`SyncError` variants alphabetical:** `AmbiguousRealignment, CandidateBudgetExceeded,
  CheckpointGap, MultiIndelBlock` — correct, and the `Display` match arms follow the same order.
- **No `unwrap`/`expect`/`panic!`/`unreachable!`/`unsafe` on any code line in `sync.rs`** (only
  in comments). Panic-safety is structural.

---

## Independent fuzz verification

All harnesses were written by the reviewer in the worktree (independent PRNG + independent
RS-backed oracles), run, and then **removed** (`tests/zz_r0_*.rs` deleted; branch left clean).
Independent cross-checks of CRC-5 and `round_sqrt` were done in Python.

1. **CRC-5 correctness (PASS — impl correct).** Cross-checked `crc5` three ways: the
   implementation, a (buggy) LFSR oracle, and a canonical non-augmented polynomial-division
   reference. The implementation matches the polynomial-division reference **exactly (0
   mismatches / 100,000 random blocks)**; generator `x⁵+x²+1` = `0b100101` confirmed. (My first
   LFSR oracle was wrong — withdrawn; the implementation is right.) Hand vectors: `crc5([])=0`,
   `crc5([0])=0`. The implementer's `crc5_detects_every_single_bit_flip` and
   `crc5_single_word_substitution_detection_rate (≥95.5%)` are sound.

2. **`round_sqrt` / `block_stride` (PASS).** Integer Newton `round_sqrt` matches the frozen
   `floor(√k + 0.5)` float rule with **0 mismatches over k∈0..5000**; tie-free claim verified
   (no integer `k` has `√k = m+0.5`). Matches the implementer's KAT to 2100.

3. **NO-SILENT-MISALIGNMENT (PASS — the safety guarantee holds).** `fuzz_no_silent_misalignment`
   — **24,300 cases** of single substitutions at every position (incl. forced `0b101`-marker
   words and substitutions exactly on checkpoint control words), `k∈{16,30,58,100,160}`. **0
   HARD failures:** the engine never returned a *confidently-wrong* `Aligned` grid that
   RS-decodes (with the reported erasures) to a wrong codeword, and never fabricated a grid
   (`g != recv`). The 633 NOTE-class events are CRC-5 blind-spot *missed detections* (an
   undetected substitution reported as a clean `Aligned` with empty erasures) — tolerated
   because they are within the documented `~2⁻⁵` CRC miss and are caught downstream by RS
   (1-error correction) + the P4 tag, **not** silent miscorrections. `adv_same_length_no_wrong_clean_align`
   and `adv_substitution_on_checkpoint` corroborate: no fabricated grid, no substitution
   misreported as a deletion, no wrong-RS recovery.

4. **CANDIDATE-TRUTH (FAIL — C1).** `fuzz_candidate_truth_every_position` — **25,840 cases**,
   single deletion at every position (data + checkpoint), `k∈{1,2,5,10,15,16,17,23,30,47,58,89,
   100,160}`, each candidate **operationally verified** by reinsert-placeholder + `rs_decode`.
   **FAILED.** The fast classifier localized the cause to **74 / 164,400 (~0.045%)** single
   *data* deletions where the reported candidate window is one full block displaced from the
   true gap; the RS-backed harness confirms **no candidate reconstructs the truth** in those
   cases. One case fully traced + Python-verified (k=58, p=9 → candidates [18..25], true gap 9
   absent; see C1). Checkpoint-deletion cases were never truth-absent (they route to
   `classify_deleted_checkpoint`'s merged-span erasure or refuse, both custody-safe).

5. **BLOCK-ERASURE END-TO-END (PASS, with the CRC-collision caveat of N2).** When a whole block's
   corruption trips the local CRC, `sync_classify` returns `Aligned{erasures}` and feeding
   `grid ‖ true-parity` through `rs_decode(.., erasures)` recovers the exact `K'` grid, which
   strips back to the original data — verified for `k∈{58,100,160}`. The Aligned path produced
   **0 wrong-grid recoveries**. (My naive block-corruption harness flagged 3 "no erasures"
   cases; all three were confirmed CRC-5 *collisions* — every word in the block changed but the
   5-bit CRC landed identically, prob ~1/32 — i.e. a harness-methodology artifact, **not** a
   P3 bug, and exactly the `≤2⁻⁵` local-check limit of N2. The implementer's KAT correctly
   sidesteps this by searching for a CRC-tripping corruption.)

6. **PANIC-SAFETY (PASS).** `fuzz_panic_safety` — **200,000** random `(words, k)` with `k∈0..180`,
   `len∈0..220`, plus explicit corners (`k=0`, `k=1`, empty, `[0]`), a 2000-word input, and every
   length in `K'±5` for `k∈{16,58,160}`. **No panic.** Corroborates the implementer's
   `sync_classify_never_panics` / `crc5_and_checkpoint_never_panic` proptests.

7. **mod-8 aliasing / refuse-on-ambiguity (PASS).** The implementer's `mod8_aliasing_requires_8b_destroyed`,
   `refuse_on_ambiguous_realignment`, and `refuse_explicit_two_way_ambiguity` are sound; my
   destroyed-run probes never produced a silent clean alignment (always erase-or-refuse).

---

## Bottom line

One **Critical** (C1: candidate-truth violated — single data deletion can yield a candidate set
that excludes the true gap, making the real word unrecoverable). The safety floor
(no-silent-misalignment, panic-safety, RS tie) is solid; the defect is isolated to the
`classify_single_deletion` H0/H1 greedy walk. **Gate is RED. Fix C1, fold N1-N3, re-run the
full suite + the reinsert+RS candidate-truth sweep, persist the round-2 review, and re-dispatch
to GREEN before merging P3 / starting P4.**
