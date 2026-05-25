# R2 Architect Review (post-fold) — IMPLEMENTATION_PLAN_indel_cross_region_subst_fallback.md (v0.37.3)

**Round:** R2 (focused). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24. **Branch:** `indel-v2-cross-region-subst-fallback`.

## Verdict: GREEN — 0 Critical / 0 Important

The I1 struct-literal field-completeness fold is complete and correct. Plan cleared for Phases 1-5.

### I1 — RESOLVED
`grep "IndelCandidate {"` returns exactly **7 source literals** (+ the struct def `indel.rs:30`, not a literal): producers `indel.rs:112,176,215` + test literals `indel.rs:292,298` + `cmd/repair.rs:368,374`. The plan's Phase-1 Step-3 bullet lists ALL 7; NO 8th site exists. Instruction correct: struct has only `#[derive(Debug,Clone,PartialEq,Eq)]` (no `#[non_exhaustive]`/`Default`) ⇒ field addition is hard E0063 on every literal; 4 test literals add `subst_count: 0`, 3 producers thread the oracle's returned `sc` (they sit inside `if let Some(rec)=oracle.validate(...)` → `Some((rec, sc))`).

### No new drift
New bullet consistent with the Phase-1 signature block (`pub subst_count: usize`), the Phase-2 skeleton (`IndelCandidate{…, subst_count: sc}`), and the §6 fold-log.

### Re-confirm (unchanged by this fold)
I2 (`is_indel_trigger` doc-comment `repair.rs:1028-1031` OLD-behavior, Phase-4 task anchored), M1 (no-op-notice gate complement of `cmd/repair.rs:141`), M2 (GUI "ms1/mk1/md1"), M3 (`prefix_restorations` pin) — all hold. Regression set (Phase-4 ownership, substitution gate, Phase-2 subsumption, exit invariant at E=0, clap range, SemVer/lockstep) untouched by a struct-field addition.

**Gate met: 0C/0I. Cleared for implementation.**
