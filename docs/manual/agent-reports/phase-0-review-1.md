# Phase 0 — feature-dev:code-architect review, round 1

**Date:** 2026-05-07
**Branch:** `manual/v0_1` (commit prior to fixes: see `git log --oneline manual/v0_1`)
**Verdict:** Not yet ready for Phase 1 — 2 critical + 3 important findings + 4 nits.

## Critical

- **C1 — `makeidx` loaded twice (`preamble.tex` line 11 + `metadata.yaml`
  header-includes).** Soft duplicate in most texlive versions but fragile.
  Remedy: drop from `preamble.tex`; keep solely in `metadata.yaml`.
- **C3-final — `primer-box.lua` md-mode bold prefix bug.** The filter fed
  `pandoc.Str("**Background.**")` to `pandoc.Strong`, which escapes the
  literal asterisks for GFM, producing `\*\*Background.\*\*` rather than
  bold "Background." Remedy: use plain text in `pandoc.Str`; let
  `pandoc.Strong` handle bold rendering.

(C2 and C3-Docker-uid were initially proposed as critical but reclassified
to Important during the review — see below.)

## Important

- **C2 (reclassified) — `fvextra` + `\DefineVerbatimEnvironment{Highlighting}`
  loads after pandoc's `$highlighting-macros$` block.** Phase-0 template
  emits `$highlighting-macros$` before `$for(include-before)$`, so the
  preamble's `Highlighting` redef silently runs late. Remedy: move
  `fvextra` + Highlighting redef into `metadata.yaml` header-includes
  (which the template emits earlier).
- **C3-Docker-uid (reclassified) — `pdf-docker` recipe writes to a volume-
  mounted `build/` directory with host non-root uid.** Edge case (only
  fails if host parent dir isn't host-uid-writable). No action this round
  — revisit if CI fails.
- **I3 — flag-coverage maps multiple `md` subcommands to a single chapter
  file `42-md.md`.** Same-binary flag false-pass possible; per-pair iteration
  works but per-subcommand-section granularity does not. Phase-5 author
  awareness only; no Phase-0 change.
- **I5 / I7 — `makeindex` / `\printindex` on missing `.idx` / `.ind` files.**
  In Phase 0 with stub-only sources containing no `\index{}` markers, the
  build aborts. Remedy: guard `makeindex` invocation with `[ -f *.idx ]`;
  `\printindex` on missing `.ind` is only a warning under `-halt-on-error`,
  so no template change.
- **I6 — `metadata.yaml` `header-includes: |` literal-block scalar.**
  Pandoc treats it as a single-element list; works in practice but
  non-idiomatic. Remedy: convert to a YAML list.

## Minor / nits

- **N1 — `Dockerfile.build` uses `debian:trixie-slim` (testing).** Bookworm
  (stable) is safer for reproducibility. Filed as a v0.2 follow-up.
- **N2 — `PUPPETEER_CONFIG_FILE` env var is non-standard for `mermaid-cli` ≥ 10.**
  Should validate against pinned `@mermaid-js/mermaid-cli@^11`. Filed
  for v0.2.
- **N3 — `make clean`** doesn't remove LaTeX byproducts directly; covered
  transitively by `rm -rf $(BUILD_DIR)`. Fine in practice.
- **N4 — `filter-smoke.sh`** xelatex relative paths fine.
- **N5 — `help` target regex** matches comment lines as intended.

## Convergence assessment (round 1)

Phase 0 is not ready for Phase 1 as-is. Two blockers (C1, C3-final) plus
three important issues (C2, I5/I7, I6) need targeted edits before Phase 0
verification can be called clean. Round 2 dispatch will verify the fixes.
