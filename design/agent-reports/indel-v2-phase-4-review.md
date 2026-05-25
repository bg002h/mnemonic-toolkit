# indel-v2 Phase 4 Review — HrpMismatch suggestion-fallback

**Round:** Phase 4 (per-phase gate). **Reviewer:** feature-dev:code-reviewer (opus). **Date:** 2026-05-24.
**Commit:** `e9e202c` (branch `indel-v2-cross-region-subst-fallback`). Files: `cmd/repair.rs`, `repair.rs`, `tests/cli_indel.rs`.
**Controller verification:** cli_indel 19/19; full default suite 128 ok (0 failed); clippy clean; version still 0.37.2; FOLLOWUP still open (no Phase-5 leak).

## Verdict: GREEN (0 Critical / 0 Important / 0 Minor)

### Checks out
- **`Unrecoverable` arm** (cmd/repair.rs:191-205): `match e { HrpMismatch{..} => Err(e.into()), _ => IndelUnrecoverable }`. `e` owned (bound by `Err(e) if … is_indel_trigger(&e)`; `recover_indel_card(*kind, chunks, …)?` borrows chunks/copies kind, never moves e); `{..}` binds nothing (no partial move); `e.into()` via `From<RepairError>` (error.rs:336-339) → `Repair(e)` → exit 2 + HrpMismatch Display.
- **Composition + exit invariance:** recoverable prefix-drop reaches Unique/Ambiguous before Unrecoverable (fallback doesn't fire); genuine wrong-HRP fails → HrpMismatch branch → suggestion. Both HrpMismatch + IndelUnrecoverable → exit 2 (error.rs:507), so only the stderr message differs.
- **Doc-comment fix (R0 I2)** (repair.rs:~1057-1063): `is_indel_trigger` doc now describes the NEW fallback ("falls back to the original HrpMismatch … rather than the generic IndelUnrecoverable; Phase 4 v0.37.3"); no leftover stale sentence.
- **Test** `genuine_wrong_hrp_falls_back_to_suggestion_not_indel_unrecoverable`: MK1_C0 to `--ms1 --max-indel 1` → failure + stderr "HRP mismatch" (the always-present Display token, repair.rs:509) + NOT "could not be recovered within --max-indel" (IndelUnrecoverable Display). "mk" is Lev1-equidistant from ms/md → `suggest_hrp` None → no "did you mean" suffix, so the stable "HRP mismatch" token is the correct assertion. Regression test confirms prefix-drop still recovers (exit 5).
- **No scope creep:** Cargo.toml 0.37.2; FOLLOWUP open; no manual/install/CHANGELOG; engine/oracles untouched (only the doc-comment in repair.rs).

Phase 4 clears the gate. Cleared to advance to Phase 5 (release-prep).
