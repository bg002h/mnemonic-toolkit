# R0 Architect Review — IMPLEMENTATION_PLAN_indel_cross_region_subst_fallback.md (v0.37.3)

**Round:** R0 (plan-doc gate; mandatory pre-code). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Branch:** `indel-v2-cross-region-subst-fallback` (base `master a6987f4`). **Persisted verbatim before fold.**

## Verdict: RED — 0 Critical / 2 Important / 3 Minor

Design is sound — the three features genuinely compose (the user's core concern checks out). No Critical. Two Important, both "missed-call-site / doc-drift" class.

## CRITICAL (0) — confirmed sound
- **Phase 4 ownership:** `e` is bound by-value in the `Err(e) if … is_indel_trigger(&e)` arm (`cmd/repair.rs:141`); `recover_indel_card(*kind, chunks, …)?` (`:142`) doesn't move it; `match e { HrpMismatch{..} => Err(e.into()), … }` valid (`{..}` binds nothing); `e.into()` via `From<RepairError>` (`error.rs:336-340`) → exit 2 carrying the suggestion. Correct.
- **Substitution gate:** decoder returns ≤t=4 positions ⇒ `placeholders + off ≤ 4` auto-enforced; ms1 `corrections[].position` + mk1/md1 `positions` Vec gates both correct.
- **Phase 2 fidelity:** two-level restructure subsumes single-region (j_prefix=0→data, j_data=0→prefix); `if j_prefix==0 && j_data==0 {continue}` correct; dedup-on-`recovered` prevents spurious Ambiguous.
- **Exit invariant:** `--max-subst 0` ⇒ `substitution_seen` never set ⇒ `{0,5,4-ambiguous,2}` byte-identical.
- clap `value_parser!(u8).range(0..=4)` valid; SemVer PATCH correct; GUI post-tag correct; 2 README markers confirmed.

## IMPORTANT (2)

### I1 — Phase 1 Step 3 omits the `recover_indel_card` + `md1_chunk_solve` test call sites (compile-break)
Phase 1 changes `validate -> Option<(String,usize)>` (+`e_subst`), `recover_indel_card` (+`e_subst`), `mk1/md1_chunk_solve` (+`e_subst`, tuple return). Step 3 only lists the `validate` mocks + `recover_indel(...)` calls. MISSING:
- **5 `recover_indel_card(…,&chunks,1)` test calls** → `repair.rs:2054,2072,2152,2163,2177` (need 4th arg `0`).
- **2 `md1_chunk_solve(…,&allowed)` direct test calls** → `repair.rs:2188,2190` (`indel_md1_chunk_solve_rejects_out_of_set_substitution`): break twice — missing `e_subst` AND tuple return invalidates `.as_deref()`/`== None`.
**Fix:** embed exhaustive call-site list in Phase 1 Step 3. Full inventory: `recover_indel(...)` = `repair.rs:1082,1102,1123,1844,1863,1885,1907,1925,1939,1955,1972,2019,2036` + `indel.rs:280,320`; `recover_indel_card(...)` = `cmd/repair.rs:142` + `repair.rs:2054,2072,2152,2163,2177`; `md1_chunk_solve(...)` direct = `repair.rs:2188,2190`; `indel_exit_code(...)` = `cmd/repair.rs:192` + `repair.rs:2088-2092` (the last is Phase 3's 3-arg change, not Phase 1).

### I2 — Phase 4 invalidates the `is_indel_trigger` doc-comment (+ CHANGELOG narrative)
`repair.rs:1028-1031` documents the v0.37.1/.2 tradeoff: "wrong-HRP … returns `IndelUnrecoverable` instead of the 'did you mean' suggestion." Phase 4 REVERSES this. Leaving the doc-comment asserting the opposite = code-vs-doc drift. **Fix:** Phase 4 Step 3 (or Phase 5) adds a task to update `repair.rs:1028-1031` to the new fallback behavior + audit the v0.37.x CHANGELOG narrative. (Manual exit-2 row stays accurate — same exit code; only prose/doc-comment narrative needs the touch-up.)

## MINOR (3)
- **M1:** `--max-subst E≥1` with `--max-indel 0` is a silent no-op (run-loop gate `max_indel>=1`, `cmd/repair.rs:141`). Spec-intended, but consider a stderr notice when `max_subst>=1 && max_indel==0` (silent-default-with-notice convention).
- **M2:** the existing `--max-indel` GUI schema comment (`mnemonic-gui/src/schema/mnemonic.rs:1562`) is stale ("ms1/mk1 only" — md1 un-refused v0.37.2). Ensure Phase 5 Step 6's `--max-subst` addition does NOT copy that stale phrasing (fixing the existing one is out of scope).
- **M3:** `prefix_restorations` yield contract under-specified — pin it: one entry per split `p ∈ [3-j..3+j]` with `j_prefix = levenshtein(input[..p], k)`, filtered `1..=max_indel` + the `j_prefix=0` intact-prefix case. (Gated by regression cells; pin for the implementer.)

## Phasing — sound
substitution-gate → cross-region → CLI → fallback → release. Phase 1-before-2 correct (establishes the `Option<(String,usize)>` contract Phase 2 threads). §3 matrix proves composition (cells 3/4 cross×subst, 5 regression, 8/9 fallback). 

## Required: fold I1+I2 (+ cheap M1/M3), re-dispatch R1.
