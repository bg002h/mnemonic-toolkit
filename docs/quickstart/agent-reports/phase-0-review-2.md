# Phase 0 — code-quality review, round 1

**Date:** 2026-05-08
**Branch:** `quickstart/v0_1` at commit `0fb4941`
**Reviewer:** feature-dev:code-reviewer (code-quality focus)
**Verdict:** SUGGESTIONS_ONLY

## Strengths confirmed

- `tests/lint.sh` opens with `set -euo pipefail` and uses positional `$1` dispatch consistent with `docs/manual/tests/lint.sh`.
- `.github/workflows/quickstart.yml` is hardened: `permissions: contents: write` is scoped, no untrusted-input interpolation lands inside `run:` blocks (`REF_NAME` is exposed via `env:` then used as a quoted shell variable), `working-directory: docs/quickstart` is set on every `make` step so the Makefile runs against the correct tree.
- `pandoc/preamble.tex` is correct for QuickStart: no `\usepackage{makeidx}` / `\makeindex`, lockstep with the no-index decision (D-3 in spec).
- `pandoc/metadata.yaml` carries no `header-includes:` block — avoids the pandoc 3.x raw-LaTeX bug observed during the manual cycle.
- `.cspell.json` uses the correct `import:` array key (validated through Phase 0 r1).
- `.gitignore` is well-targeted: `build/`, `src/99-build-banner.md`, `mermaid-filter.err` mirror `docs/manual/.gitignore` — no over-broad globs.
- `Makefile` PDF rule uses two `xelatex` passes with no `makeindex` invocation, correct for QuickStart.

## Suggestions (non-blocking)

- **S-1: dead lint.sh args from Makefile.** `Makefile`'s `lint` target invokes `tests/lint.sh markdownlint`, `cspell`, `lychee` and additionally passes `MNEMONIC_BIN=true MD_BIN=true MS_BIN=true` from CI. The QuickStart `lint.sh` only handles the 3 cases (markdownlint/cspell/lychee) and silently ignores the 3 binary-presence env vars. Either drop the env-var passes from the CI step, or add no-op `case` arms in `lint.sh` for `glossary`, `flag-coverage`, `index-bidirectional` so a future "I'll just copy the manual's CI step" doesn't fail-open. ~5 lines of change.
- **S-2: `ALL_SRC` dead variable in Makefile.** Inherited from `docs/manual/Makefile`. The QuickStart Makefile defines `ALL_SRC := $(wildcard src/*.md)` but never references it. Cosmetic; not a regression.
- **S-3: `fontsize` redundancy in metadata.yaml.** Inherited pattern from `docs/manual/pandoc/metadata.yaml`. Cosmetic.

## Carry-over from review-1

The PUPPETEER `env:` concern flagged in `phase-0-review-1.md` was determined to be a false positive: mermaid-filter discovers `.puppeteer.json` from CWD per `node_modules/mermaid-filter/index.js:44`, and the manual's CI works without those env vars. No action.

## Verdict

**SUGGESTIONS_ONLY.** No blocking code-quality issues. S-1 is the highest-value cleanup at ~5 lines; recommend filing in `FOLLOWUPS.md` for Phase 1. S-2 and S-3 are inherited from the manual and are not regressions introduced by Phase 0.
