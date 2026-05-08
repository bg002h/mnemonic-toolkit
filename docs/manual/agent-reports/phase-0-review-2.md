# Phase 0 — feature-dev:code-architect review, round 2

**Date:** 2026-05-07
**Branch:** `manual/v0_1` HEAD = round-1-fixes commit.
**Verdict:** Converged. 0 critical / 0 important / 2 trivial future-fragility nits.

## Verification of round-1 findings

| Finding | Status | Evidence |
|---|---|---|
| C1 | RESOLVED | `preamble.tex` carries only a comment about `makeidx`; load is exclusive to `metadata.yaml`. |
| C3-final | RESOLVED | `primer-box.lua` constructs `pandoc.Strong({ pandoc.Str("Background.") })` — bold delegated to AST writer. |
| C2 | RESOLVED | `metadata.yaml` `header-includes` list contains `fvextra` + Highlighting redef as scalar items, ahead of `$for(include-before)$` in template. |
| I5 / I7 | RESOLVED | Makefile guards `makeindex` with `[ -f *.idx ]`; `\printindex` no-op-on-missing-`.ind` confirmed safe under nonstopmode. |
| I6 | RESOLVED | `metadata.yaml` `header-includes` is a YAML list (four `- ` items). |

## New findings

- **N-new-1** — `filter-smoke.sh` PDF leg `makeindex` is unconditional;
  fixture has `\index{m-format star}` so safe in practice. No action.
- **N-new-2** — `manual.latex` `\printindex` always emitted; with no
  source-side `\index{}` markers it produces an empty index page
  (benign visual clutter). No action.

Both nits are future-fragility observations with zero impact on
Phase 0 correctness.

## Convergence assessment

Phase 0 deliverables are clean. The toolchain skeleton is ready for
Phase 1 to lay frontmatter + glossary scaffold + index plumbing on
top of it. Nothing in the round-1 fixes regressed any prior behaviour.
