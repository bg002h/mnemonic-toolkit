# R0 review — SPEC_manual_prose_execution_gate.md (verbatim, persisted before fold)

Reviewer: feature-dev:code-reviewer (opus). Base `badb619`. Cycle: manual-prose-execution gate full sweep.

**Verdict: RED — 1 Critical / 4 Important / 3 Minor.** Spec mostly sound; load-bearing Makefile-defaults safety check for Piece 2 PASSES (all 4 of MNEMONIC_BIN/MD_BIN/MS_BIN/MK_BIN defaulted via `?=` at Makefile:42-45, so `:?` change won't break `make audit`). But one false-precedent claim controls Phase 4 ship mode + four important inaccuracies surface during execution.

## Critical (1)
### C1 — `v0.34.3 precedent` claim is FALSE; v0.34.3 was a tagged PATCH bump
Spec :6 + :79 say "Untagged commit, no version bump (v0.34.3 precedent)." Ground truth: v0.34.3 IS labeled SemVer-PATCH in CHANGELOG (and was tagged). MEMORY confused "docs/test-only change class" with "no-bump release mode" — they're orthogonal. Phase 4 plan currently says skip the bump; if followed, the tip diverges from Cargo.toml + CHANGELOG. Fold: either (a) drop the false precedent and cite an actual untagged-docs-commit precedent (the v0.37.6 hygiene commit `9294723` — CI guard + test fixture, commit-to-master no-bump for pure CI/test/docs with zero binary change), OR (b) ship as v0.37.8 PATCH with full Phase-6.

## Important (4)
### I1 — Cells #2/#3/#5/#6 prose recipes do NOT include `diff`
Only cells #1 (sparrow `:317-319`) and #4 (coldcard-multisig `:571`) end in a `diff` invocation in chapter prose. Cells #2/#3/#5/#6 end at `export-wallet`. The spec's "Pattern" line and "Expected non-empty diffs for cells 2/3/4" sentence are inconsistent with prose. Two valid folds: (a) transcripts mirror prose literally → #2/#3/#5/#6 capture exit-0 gate only (empty `.out`); (b) add `diff` lines to chapter-45 prose first → then capture. Recommend (a) for fidelity to the "capture, never author" rule.

### I2 — Prose addendum :317 lands INSIDE the sparrow code-fence
Line 317 = `# Compare under per-format canonicalize (semantic round-trip)` inside ```sh ... ``` (fence :307-:320). Adding a "prose addendum" there pollutes the recipe. Move to after close-fence: sparrow `:321` (blank between :320 close and :322 next H3); specter `:412` (between :411 close and :413 prose start). Spec citations off by 4 (sparrow) / 1 (specter).

### I3 — `--include-fragments` at v0.24.2 is enum (default `anchor-only`)
v0.24.0 changed `include_fragments` from bool to enum (`none|anchor-only|text-only|full`). Bare `--include-fragments` defaults to `anchor-only` — exactly what we want, but worth one clarifying sentence so readers know text-fragments are NOT validated.

### I4 — Pre-flight existing-dangler enumeration is mandatory, not optional
Grep finds **124 `](#anchor)` references across 22 manual files.** Pandoc auto-anchor generation is heuristic; lychee's parse may not match. Probability all 124 resolve clean is low. If Phase 1 ships the lint edit without pre-flight, CI breaks on first push. Strengthen Phase 1: "(i) trial-run lychee with `--include-fragments` locally FIRST, (ii) triage results into ≤2-fix vs backlog FOLLOWUP, (iii) THEN edit lint.sh." Make it a precondition, not optional pre-flight.

## Minor (3)
- **M1** — Makefile defaults range `:43-45` → actually `:42-45` (all 4 binaries; MNEMONIC_BIN at :42). Load-bearing safety check PASSES.
- **M2** — `.err` files in existing transcripts are 0-byte (empty), not "1-byte placeholder." `verify-examples.sh:85` only checks `[ -f "$err_file" ]` (presence sentinel; content may be empty).
- **M3** — `:411-415` blockheight-drop citation: line 411 is fence-close, actual prose at `:413-415`. Cosmetic.

## GREEN (spec claims that DO check out)
- **Load-bearing Piece 2 safety check passes**: all 4 binaries defaulted via `?=` (Makefile:42-45) → `:?` change won't break `make audit`.
- All 6 `### Round-trip example` anchors at the cited lines (305/404/480/563/640/752); all 6 fixtures exist; v0.37.0 `template_from_descriptor` auto-derive wired for the 4 template-requiring formats.
- `verify-examples.sh:46-49` is currently `:=true` (matches diff target); `transcripts/foreign-formats/` is empty.
- lychee v0.24.2 pinned (`manual.yml:68` + `Dockerfile.build:55`); `--from-import-json conflicts_with_all = ["template","descriptor"]` at `export_wallet.rs:185`; `--wallet-name` flag at `:143-144`.
- Parent FOLLOWUP `manual-yml-and-install-sh-sibling-gui-pin-staleness` resolved v0.36.4; new FOLLOWUP `manual-yml-sibling-pin-vs-install-sh-drift-gate` aligns as defense-in-depth.
- `install.sh:35/38/41` ↔ `manual.yml:72-88` pins match; pieces 1/2/3 file-disjoint (lint.sh ↔ transcripts/foreign-formats/* ↔ verify-examples.sh).

## Required for GREEN
1. C1 — resolve ship mode (precedent citation).
2. I1 — pick cells-mirror-prose-literally OR add diff to prose first.
3. I2 — addendum line numbers (off by 4/1).
4. I3 — add `--include-fragments = anchor-only` clarifying sentence.
5. I4 — promote pre-flight to mandatory Phase 1 step (i).
Minors M1-M3 inline. Re-dispatch architect for R1 after fold (per "reviewer-loop continues after every fold").
