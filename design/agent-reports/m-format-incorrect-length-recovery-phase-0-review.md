# Phase 0 Review — indel scaffolding

**Round:** Phase 0 (per-phase gate). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Diff:** `47cb988..75a989e` (branch `m-format-incorrect-length-recovery`). Files: `src/indel.rs` (new), `src/main.rs`, `src/repair.rs`.

## Verdict: GREEN (0 Critical / 0 Important)

Phase 0 scaffolding holds — clean scope-faithful insertion, no behavior, no CLI changes, no premature recovery logic.

### Checks out
- **§2.1 types** exact shapes: `IndelRegion`/`IndelDirection`/`IndelCandidate{recovered,indel_count,region,direction}`/`IndelOutcome::{Unique,Ambiguous,Unrecoverable}`/`IndelOracle::validate(&self,&str,&BTreeSet<usize>)->Option<String>`/`recover_indel(input,hrp,max_indel,oracle)` (indel.rs:17-91).
- `collect_*` are EMPTY stubs (no premature logic). `dedup_by_recovered` (129-132) keyed on `recovered` ONLY (`sort_by` + `dedup_by(|a,b| a.recovered==b.recovered)`) — NOT derived `PartialEq` (R0 I2 satisfied). `data_part_bounds` (137-143) and `levenshtein` (148-170, correct DP) fully implemented. `PLACEHOLDER_CHAR='q'` (67).
- **§2.3** `RepairError::IndelUnrecoverable{hrp,max_indel}` (repair.rs:397-401) inserted after `HrpMismatch`/before `TooManyErrors` (pure insertion, no reorder); Display arm (488-493); exit 2 via unchanged `Repair(_)=>2`.
- **§2.5** `mod indel;` between `friendly` and `language` (main.rs:16).
- **No scope creep** — cmd/repair.rs untouched; no oracles/CLI/flags yet.
- **Test real** — `recover_indel_empty_budget_is_unrecoverable` drives mock `NoOracle`, asserts `Unrecoverable` at N=0 via the real entry point.
- **`#[allow]` hygiene** — all narrow per-item with "used from Phase N+" comments; no blanket module allow.

### Advisory (forward-looking, not a Phase-0 defect)
- The `#[allow(unused_imports)]` on `use std::collections::BTreeSet;` (repair.rs:32) is the correct Phase-0 bridge (BTreeSet unused in repair.rs yet), but MUST be removed when `Ms1IndelOracle`/`Mk1IndelOracle` first reference BTreeSet in Phase 2 (else redundant-allow). → carried into the Phase 2 brief.

Phase 0 clears the gate. Cleared to advance to Phase 1.
