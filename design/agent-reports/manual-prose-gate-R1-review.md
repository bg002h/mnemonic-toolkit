# R1 re-review — SPEC_manual_prose_execution_gate.md (verbatim, post-fold)

Reviewer: feature-dev:code-reviewer (opus). R0 was RED 1C/4I/3M (`manual-prose-gate-R0-review.md`); folded; this is the re-dispatch.

## VERDICT: GREEN — 0 Critical / 0 Important
All R0 folds correctly applied. Verified against current source. R0 gate satisfied (RED 1C/4I/3M → fold → R1 GREEN). Cleared to enter Phase 1.

## Fold verification
- **C1 — precedent `9294723`**: VERIFIED. Commit `9294723` exists in reflog; message `test+ci: fix mislabeled descriptor-mode xpub fixture + add Cargo.lock --locked CI guard` — pure CI/test/docs, no version bump; sits between v0.37.5 and v0.37.6 tags. Class match exact.
- **I1 — cells mirror prose**: VERIFIED all 6 cells in `45-foreign-formats.md`:
  - #1 Sparrow :305-320 ends in `diff` :318-319 ✓
  - #2 Specter :404-411 ends at export-wallet (no diff) ✓
  - #3 Coldcard-SS :480-487 ends at export-wallet (no diff) ✓
  - #4 Coldcard-MS :563-572 ends in `diff` :571 ✓
  - #5 Jade :640-647 ends at export-wallet (no diff) ✓
  - #6 Electrum :752-759 ends at export-wallet (no diff) ✓
- **I2 — addendum placement**: Sparrow `:321` is blank (between fence-close :320 and next H3 :322) ✓; Coldcard-MS placement "between close-fence and next H3" unambiguous ✓.
- **I3 — lychee anchor-only default**: VERIFIED (v0.24+ enum `{none|anchor-only|text-only|full}`; bare flag → anchor-only).
- **I4 — pre-flight mandatory**: VERIFIED, 3-step ordering explicit in spec :58-61.
- **M1/M2/M3**: all applied.

## No new drift
Completeness (6 cells / 6 subsections), non-redundancy vs cross-format-recipes (A→A vs A→B), risk flags, Piece 2 Makefile-defaults check (all 4 `?=` at `Makefile:42-45`), sibling-pin assertions — all UNCHANGED + still valid.

## Minor cosmetic note (below report threshold)
Spec :33 header "Cells with expected non-empty `.out` diffs" followed by bullet :34 describing Sparrow as "round-trips clean" — internal framing nit. Capture-never-author rule means `.out` is empirical anyway; implementer won't be misled. Confidence ~40 it needs a fold; not blocking.

**Cleared to implement.** Phase 1 begins with mandatory lychee pre-flight per spec :58-61.
