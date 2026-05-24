# Phase 5 Review — repair --max-indel CLI surface + multi-group exit aggregation

**Round:** Phase 5 (per-phase gate). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Commit:** `1787647` (branch `m-format-incorrect-length-recovery`). Files: `cmd/repair.rs`, `repair.rs`, `indel.rs` (allow-removals only), `cmd/inspect.rs` (1-line), `tests/cli_indel.rs` (new, 11 tests).
**Controller verification:** full default `cargo test -p mnemonic-toolkit` green (128 ok-results, 0 failed); `cli_indel` 11/11; clippy `-D warnings` clean; `--max-indel 0` byte-identical to today.

## Verdict: GREEN (0 Critical / 0 Important)

### The beyond-brief `resolve_groups` HRP-relaxation is CORRECT & SAFE (explicit ruling)
The implementer found my §1.7 amendment insufficient: `resolve_groups`' strict `validate_flag_hrp` pre-gate rejected a prefix-dropped `--ms1 s10…` (exit 2) BEFORE the trigger. Fix: `relax_hrp_for_indel: bool` param; `repair` passes `max_indel>=1`; positional stays strict; `inspect` passes `false`. Verified:
1. **Scoped to typed-flag gate only** (`repair.rs:275-285`); positional `classify_hrp_prefix` (`:294`) unconditional/strict.
2. **No false recovery:** `--ms1 mk1xxx` → ms1 bucket → `HrpMismatch` trigger → only `collect_prefix` runs (data producers inert: `data_part_bounds` None), validated through `Ms1IndelOracle` ⊆-gate → ~2⁻⁶⁵ collision → `Unrecoverable`/exit 2. No silent mis-decode. mk1 oracle equally gated.
3. **All callers correct:** only `repair`(`max_indel>=1`) + `inspect`(`false`) call `resolve_groups`; **`verify_bundle` calls `validate_flag_hrp`/`classify_hrp_prefix` DIRECTLY** (`verify_bundle.rs:185-203`), independent of the param → auto-fire keeps strict gate.
4. **No new secret exposure** (value flows into existing repair path; D9 advisory fires on the ms1 arms).
5. **Default byte-identical at N=0** (Test 7 locks it: positive "too many errors" + negative "within --max-indel".not()).

### Standard Phase-5 confirmed
- Multi-group aggregation (R0 I1): no early-return on Ambiguous/recovered; per-group emit; only Unrecoverable short-circuits; `Ok(indel_exit_code(...))`; advisory reached post-loop; `_stderr`→`stderr`. Test 11 proves no group skipped.
- `is_indel_trigger` exhaustive match (HrpMismatch incl.); `indel_exit_code` 4/0/5. `--json` IndelJson (status unique/ambiguous only). Notice at N>=3. md1 refusal → BadInput exit 1 (Test 9 genuine).
- Test 3 (prefix-drop) genuinely exercises the relaxation (the load-bearing proof prefix recovery is CLI-reachable). Unit tests load-bearing (the 2-candidate emit test is the only Ambiguous-emit coverage).
- Concern #2: Test 6 asserts "within --max-indel", present in IndelUnrecoverable Display (`repair.rs:503-508`) — non-vacuous.
- Scope/hygiene: `indel.rs` only lost dead_code allows (engine now reachable); no engine/oracle logic changed; clippy clean.

### Minor (sub-threshold, recorded — fold opportunistically)
- **m-adv (conf ~30):** Plan §4.3 #8 listed "ms1 recovery fires the secret advisory on stderr" as a cli_indel cell; it's absent (advisory IS wired via `any_ms1`, covered indirectly + by cli_repair). → **fold into Phase 6** (add the explicit stderr-advisory assertion).
- **m-hrp (conf ~40):** opt-in `--max-indel>=1` loses the HrpMismatch "did you mean" suggestion on genuine typos — documented §1.7 tradeoff, FOLLOWUP candidate. Not a defect.

Phase 5 clears the gate. Cleared to advance to Phase 6 (lockstep + release-prep). NOTE for Phase 6: plan §8's blanket `cargo test -- --include-ignored` trips 2 pre-existing `mlock::tests` G2 fault-injection failures (require `MNEMONIC_TEST_MLOCK_FAIL_MODE` env; mlock.rs byte-identical to origin/master) — run the j=4 runtime-sanity test targeted, not blanket.
