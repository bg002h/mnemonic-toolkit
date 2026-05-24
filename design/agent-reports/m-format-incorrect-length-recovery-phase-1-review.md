# Phase 1 Review — too-long delete-and-validate producer + ms1 oracle

**Round:** Phase 1 (per-phase gate). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Commit:** `f59271c` (branch `m-format-incorrect-length-recovery`). Files: `src/indel.rs`, `src/repair.rs`.
**Controller verification:** `cargo test -p mnemonic-toolkit --bins` → 798 passed / 2 ignored; both Phase 1 tests pass; `cargo clippy --all-targets -- -D warnings` clean.

## Verdict: GREEN (0 Critical / 0 Important)

### Checks out
- **`collect_data_delete`** (indel.rs:134-166) matches §2.1: `data_part_bounds` guard, `data.len()<=j` early return, `combinations(data.len(),j)`, deletes combo indices, validates with EMPTY `allowed`, pushes `IndelCandidate{DataPart,Deleted,j}`.
- **`combinations`** (indel.rs:105-132) correct lexicographic k-subset generator; `combinations(4,2)`=6 pairs; `k>n`→empty; `k==0`→one empty set.
- **`Ms1IndelOracle::validate`** (repair.rs:861-875) matches §2.2: `decode_with_correction` → accept iff `corrections.all(|c| allowed.contains(&c.position))` returning `apply_ms_corrections(cand,&corrections).0`; else `None`. Pure-indel ⊆ rule correct (delete: allowed=∅ ⇒ only zero-correction exact-codeword candidates pass).
- **`recover_indel_card`** (repair.rs:882-900): ms1 arm → `recover_indel(chunk,"ms",n,&Ms1IndelOracle)`; mk1 → `Unrecoverable` (Phase 4 placeholder); md1 → `BadInput` refusal. NOT wired into cmd/repair.rs.
- **No scope creep:** `collect_prefix`/`collect_data_insert` still empty stubs; no `Mk1IndelOracle`/`mk1_chunk_solve`; no `--max-indel`/cmd change/main change.
- Recovered string is exact original (apply_ms_corrections lowercases, but candidate is already lowercase → no-op). Both tests real end-to-end, assert `Unique`. The `q`-run in VALID_MS1 makes `dedup_by_recovered` (keyed on `recovered` only) load-bearing in test 1 — collapses correctly.
- **`#[allow]` hygiene:** BTreeSet `unused_imports` allow REMOVED (now used); `dead_code` removed from `combinations`/`data_part_bounds`/`collect_data_delete` (now called); remaining allows only on still-empty stubs + Phase-5-reachable items. No blanket allow.

### Minor (non-blocking, recorded)
- `indel_ms1_too_long_by_two_recovers` (repair.rs:1632-1641) asserts only `recovered==VALID_MS1`, omitting `direction`/`region`/`indel_count` (j=1 test covers them). Cosmetic; optional to tighten in a later touch.

Phase 1 clears the gate. Cleared to advance to Phase 2.
