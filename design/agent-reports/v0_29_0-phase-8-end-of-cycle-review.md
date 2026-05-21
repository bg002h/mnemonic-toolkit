# v0.29.0 Phase 8 end-of-cycle opus review

**Reviewer:** opus
**Date:** 2026-05-21
**Cycle:** mnemonic-toolkit-v0.29.0 (Wave 3 SemVer-minor cliff + paired GUI)

## Verdict: GREEN

0 Critical / 0 Important / 0 Minor / 0 new FOLLOWUPs.

Recommendation: **PROCEED-TO-COMMIT-AS-2-COMMITS-PER-I3.**

## Slug-by-slug substantiation

### Slug A — ImportProvenance 2-variant split
- Enum at `wallet_import/mod.rs:70-140`: `BsmsSixLine(BsmsAuditFields)` (L75) + `BsmsTwoLine` (L77) alphabetically between `BitcoinCore` (L72) and `Coldcard` (L86). `BsmsSixLine` < `BsmsTwoLine` (`S` < `T`).
- All 7 accessor `match self {}` blocks at `mod.rs:147-266` exhaustively handle both variants.
- Construction site at `wallet_import/bsms.rs:342-345`: `match audit { Some(a) => BsmsSixLine(a), None => BsmsTwoLine }`.
- Test cells at `mod.rs:513, 527, 534, 554, 561` use new variant names; regression-guard `provenance_accessor_matrix_invariant_under_alphabetical_reorder` (L546) exhaustively exercises 3 variants × 2 accessors.
- No leftover `ImportProvenance::Bsms(_)` in `crates/` (grep clean).

### Slug B — xpub-search tagged-enum conversion
- All 3 result types `#[serde(tag = "result", rename_all = "snake_case")]` with `Match` + `NoMatch` variants.
- `NoMatch` variants omit `path` / `template` / `account` (path+passphrase) / `matched_cosigners` (account); confirmed by serde unit test at `cmd/xpub_search/mod.rs:165-198`.
- v0.27.0 drift cells `#[ignore]`-gated at `tests/cli_xpub_search_drift_v0_27_0.rs:80, 142, 189`.
- `XpubSearchJson::AddressOfXpub(AddressOfXpubResult)` deliberately untouched (uses inner per-target tagged enum already).

### Slug C — error.rs retroactive alphabetical sort
- All 44 variants alphabetical in: enum (`L11-287`) + `exit_code` match (`L428-473`, single-variant arms per R0-I2 lock) + `kind` match (`L480-527`) + `message` match (`L533-700`).
- Partial-match `details` (`L707-731`): 7 named arms alphabetical (Bip388Distinctness / BundleMismatch / CosignerSpec / FutureFormat / ModeViolation / NetworkMismatch / SlotInputViolation).
- Spot-checked exit codes + Display strings against master: all preserved byte-exact.

## Cross-cutting

- `Cargo.toml` v0.28.7, `CHANGELOG.md` no v0.29.0 entry, `install.sh:32` unbumped, FOLLOWUP statuses unflipped — all expected per plan-doc Task 7+9 ordering (gated behind toolkit-tag push).
- `gui-schema` JSON byte-identical between v0.28.7 and working tree (confirms no clap surface drift). GUI schema-mirror needs NO edit at Task 7.

## Commit ordering

Per plan-doc Task 9 R0-I3 lock:
1. **Commit 1:** Slug C sort-only (error.rs).
2. **Commit 2:** Slug A + Slug B + version bump + CHANGELOG + install.sh + FOLLOWUPS. Tag lands on Commit 2.
