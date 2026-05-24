# Phase 2 Review — too-short placeholder-then-decode producer (ms1)

**Round:** Phase 2 (per-phase gate). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Commit:** `f040cf4` (branch `m-format-incorrect-length-recovery`). Files: `src/indel.rs`, `src/repair.rs`.
**Controller verification:** `cargo test -p mnemonic-toolkit --bins` → 802 passed / 2 ignored; 4 Phase-2 tests pass; clippy `-D warnings` clean.

## Verdict: GREEN (0 Critical / 0 Important) — no findings

### Checks out
- **`collect_data_insert`** (indel.rs:166-203) matches §2.1: `data_part_bounds` guard; `slots=data.len()+j`; `combinations(slots,j)` as placeholder POSITION sets; `built` interleaves `PLACEHOLDER_CHAR` at combo indices and source chars at the rest in order; `allowed=combo`; validates `format!("{hrp}1{built}")`; pushes `IndelCandidate{DataPart,Inserted,j}`. Correctly omits the delete producer's `data.len()<=j` floor (insertion needs none).
- **`built` interleaving traced correct:** combo has exactly j of slots=D+j positions ⇒ exactly D non-combo slots = source count; `src.next()` called exactly D times, never runs out; source chars land at non-combo slots in order, so hypothesizing the placeholder at the true omission position reconstructs the original. `combinations` max index = n-1 (never ≥ slots). The `built.len()!=slots` guard is defensive-only.
- **⊆ coordinate alignment:** `allowed` (combo) and `ms_codec::CorrectionDetail.position` are both data-part 0-indexed (`ms-codec decode.rs:79-90,223`). Exact.
- **Tests sound:** non-'q' drop (genuine BCH solve, 1 correction ⊆{1}); 'q' drop (zero-correction ∅⊆{8} collision path; q-run combos collapse via dedup → Unique); pure-indel rejection genuinely non-vacuous (drop+subst ⇒ correction at pos 2 ∉{1} ⇒ ⊆ rejects ⇒ Unrecoverable, not a parse/length failure); j=2 (combo {1,5} reconstructs, ⊆{1,5}). The report's "inddi_…" was a report-only typo; source identifier is valid.
- **Hygiene/scope:** `PLACEHOLDER_CHAR` + `collect_data_insert` carry no allow (now used/called); remaining allows only on Phase-5-reachable items + the still-empty `collect_prefix` stub; no mk1/CLI/main changes; `recover_indel_card` mk1 arm still the Phase-1 `Ok(Unrecoverable)` placeholder.

Phase 2 clears the gate. Cleared to advance to Phase 3 (`collect_prefix`).
