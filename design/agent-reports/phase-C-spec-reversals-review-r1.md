# Phase C — SPEC reversals (3 items) — code-reviewer r1 (2026-05-06)

## Findings

### Critical
None.

### Important
None.

### Low / Nit
None blocking. Implementation review:

**C.1 — typed-DerivationPath equality shipped.** `parse_descriptor.rs::check_key_vector_distinctness` now compares `cs[i].path == cs[j].path` (typed `DerivationPath`, folds `h` → `'`) instead of `path_raw` raw-string equality. Doc-comment rewritten to reflect SPEC §4.11.b reversal. Existing v0.4.1 test `bip388_h_vs_apostrophe_paths_distinct_under_raw_string` migrated to `bip388_h_vs_apostrophe_paths_collide_under_typed_equality_v0_5` (asserts collision instead of distinctness; matches v0.5 SPEC).

**C.2 — SPEC-only codification.** No code change needed (the helper already short-circuits watch-only via `expected.ms1[0].is_empty()` regardless of supplied). New integration test `verify_bundle_watch_only_spurious_ms1_silently_absorbed_v0_5` in `cli_verify_bundle_watch_only.rs` confirms `--xpub --ms1 X` (watch-only with spurious ms1) still produces `result: ok`.

**C.3 + C.4 — `detect_removed_subcommand` trap deleted.**
- Function + `REMOVED_SUBCOMMAND_ERR` const removed from `bundle_unified.rs` (~25 lines).
- 5 inline `#[test]` items removed from `bundle_unified.rs` (~50 lines).
- `main.rs` pre-clap call site removed (~6 lines).
- Two integration tests in `cli_bip388_distinctness.rs` migrated from byte-exact stderr pinning to clap-fallback exit-64 assertion (toolkit's format-violation override of clap's default 2).

## Test status

236 lib + 22 integration suites pass (was 241+22 in Phase B; -5 from C.3+C.4 inline tests deleted; +0 net for parse_descriptor distinctness migration; +1 for C.2 spurious-ms1 integration test).

## Outcome

Phase C APPROVED. Proceed to Phase D.
