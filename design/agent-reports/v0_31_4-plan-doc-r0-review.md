# v0.31.4 plan-doc R0 review

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** R0
**Plan under review:** `design/PLAN_mnemonic_toolkit_v0_31_4.md`
**Date:** 2026-05-21
**Source SHA:** `bee253f` (master HEAD)

## Verdict

**YELLOW.** 0 Critical / 2 Important / 1 Minor. All tractable as plan-doc fold.

## Important (I)

**I1 — "Project's pattern" claim for `LazyLock` is factually wrong.** Grep across `crates/mnemonic-toolkit/` returns ZERO matches for `LazyLock`, `once_cell`, or `Lazy::new`. The established pattern across `sparrow.rs:555/566/678`, `bsms.rs:501/520`, `bitcoin_core.rs:530/553/561`, `pipeline.rs:38`, `electrum.rs:920`, `specter.rs:358/467`, `coldcard.rs:507` is **`Regex::new(...).expect(...)` inside the function body** (re-compiled per call; `parse()` not in hot loop). For symmetry, drop the `LazyLock` static and inline `Regex::new(r"@\d+/\*\*").expect("at-placeholder regex is a fixed string literal")` adjacent to L338 — mirroring `sparrow.rs:566` precedent.

**I2 — Test surface under-specified.** Plan acknowledges the regression cell may need to be inline-only and leaves the resolution open. Commit to:
- **Regex-unit cell** with positive cases (`@0/**`, `@1/**`, `@10/**`) + negative cases (`@/**`, `@0/*`, `@a/**`, empty).
- **Backward-compat cell** asserting an existing `@0/**` template-mode fixture still routes to template-mode (locks the no-behavior-change claim).

Two cells, both deterministic; no hypothetical-blob construction.

## Minor (M)

**M1** — Inline comment block at L326-329 confirmed; update wording should mirror existing `sparrow.rs:566` regex precedent + cite the closed FOLLOWUP slug.

## Verified clear

- Regex literal `r"@\d+/\*\*"` correctness — `\d+` matches one+ digits (`@0/**`, `@1/**`, `@10/**` ✓; `@/**` ✗); `/` literal ✓; `\*\*` two literal asterisks ✓.
- Sparrow emit invariant at `wallet_export/sparrow.rs:230` `(0..n).map(|i| format!("@{i}/**"))` confirms `@0/**` always present in current template-mode emit; no behavior change under current invariants.
- Existing regex at `sparrow.rs:566` `Regex::new(r"@\d+(?:/\*\*)?")` already escapes `\*\*` correctly; this is the template-precedent.
- SemVer-PATCH justified (defensive hardening; no behavior change).

## Recommendation

Fold I1+I2, then proceed to Phase 2.
