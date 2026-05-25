# R1 Architect Review (post-fold) — IMPLEMENTATION_PLAN_indel_cross_region_subst_fallback.md (v0.37.3)

**Round:** R1. **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24. **Branch:** `indel-v2-cross-region-subst-fallback` (base `a6987f4`). **Persisted verbatim before fold.**

## Verdict: RED — 0 Critical / 1 Important

Design sound; 4 of 5 folds clean. One fold (I1) still misses a compile-break class.

## Fold verification
- **I1 — NOT fully RESOLVED.** The call-arity/tuple-return inventory is complete + line-accurate (re-grepped: `recover_indel(` 13 repair + 2 indel sites; `recover_indel_card(` cmd:142 + 5 repair tests; `md1_chunk_solve(` repair:2188,2190; 5 oracle `validate` impls + 2 mocks — all exact). **Missed class: `IndelCandidate` struct-literal field-completeness.** Adding `subst_count` (no `#[non_exhaustive]`/`Default` on the struct) makes every 4-field literal a hard E0063. Plan covers the 3 producer literals (`indel.rs:112,176,215`) but MISSES 4 test literals: `indel.rs:292,298` (`dedup_collapses_same_recovered_with_differing_metadata`) + `cmd/repair.rs:368,374` (`emit_indel_two_candidate_text_and_json`) — each needs `subst_count: 0`. The inventory claims exhaustiveness; this named class is the same gap I1 filed. **Confidence 90.** Fix: add those 4 sites + a line noting the field-addition breaks all `IndelCandidate {...}` literals.
- **I2 — RESOLVED.** `is_indel_trigger` doc-comment (`repair.rs:1021-1037`, esp. 1028-1031) documents the OLD wrong-HRP→IndelUnrecoverable behavior; Phase 4 reverses it; plan's update task + `~1028-1031` anchor accurate. Manual exit-2 row unchanged (both HrpMismatch + IndelUnrecoverable → exit 2 via `error.rs:507`).
- **M1 — RESOLVED.** No-op notice gate `max_subst>=1 && max_indel==0` is the exact complement of the run-loop gate (`cmd/repair.rs:141`); stderr-only.
- **M2 — RESOLVED.** Existing `--max-indel` GUI comment IS stale ("ms1/mk1 only", `mnemonic-gui/.../mnemonic.rs:1562/1572`); plan's "ms1/mk1/md1" instruction correct; fixing the existing one out of scope.
- **M3 — RESOLVED.** Pinned `prefix_restorations` contract matches `collect_prefix` (`indel.rs:88-120`) exactly (window `[3-j..3+j]`, `levenshtein==j`, Inserted if p<3 else Deleted) + `data_part_bounds` intact case.

## New issues from folds
None. `indel_exit_code` "Phase 3 not here" note consistent (only `cmd/repair.rs:192` + `repair.rs:2089-2092`); M3 pin consistent with the Phase-2 skeleton; M1 stderr-only.

## Regression check — all hold
Phase-4 ownership (`e` by-value `:141`, `e.into()` `error.rs:336-340` → exit 2 + suggestion `repair.rs:503-511`); substitution gate (ms1 `.position`/mk-md `positions` Vec; decoder caps t=4); Phase-2 subsumes single-region + `region_str` exhaustive match (`cmd/repair.rs:308-313`) correctly flagged for `CrossRegion` in Phase 2 Step 3; exit invariant at E=0; clap range; SemVer PATCH; both READMEs + install.sh:32 + GUI post-tag.

## Required → GREEN
Fold the I1 gap: enumerate the `IndelCandidate` struct-literal class — 4 missed test sites (`indel.rs:292,298`; `cmd/repair.rs:368,374`) gain `subst_count: 0`. Re-dispatch R2.
