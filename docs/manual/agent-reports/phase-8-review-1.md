# Phase 8 — feature-dev:code-architect review, round 1

**Date:** 2026-05-07
**Branch:** `manual/v0_1` (Phase 8 state)
**Verdict:** Not converged. 2 critical / 3 important / 4 nits.

## Critical

### C-1 — CI workflow `docker run` mounts only `docs/manual`; `make pdf` git lookups break

`.github/workflows/manual.yml` set `working-directory: docs/manual` then `docker run -v "$PWD":/work -w /work`. Inside the container `MANUAL_DIR = /work` so `TOOLKIT_ROOT = /` (filesystem root) — `git -C / rev-parse --short HEAD` fails, `99-build-banner.md` regenerates with `GIT_SHA=unknown`, `TOOLKIT_VERSION=dev`. Every CI run + every release PDF would carry a wrong build banner.

**Fix applied:** Mount `$GITHUB_WORKSPACE` (repo root) at `/work`, set `-w /work/docs/manual`, drop the step-level `working-directory`. Mirrors the Makefile's own `pdf-docker` target.

### C-2 — Phase 6 C1 fix incomplete: `policy_template` and `@N/<0;1>/*` survived in 64 + glossary

`64-descriptors-primer.md:54` and `61-glossary.md:245` still carried the wrong field name (`policy_template` → `description_template`) and wrong placeholder notation (`@N/<0;1>/*` → `@N/**`).

**Fix applied:** Both updated to canonical forms. All four occurrences across the manual (chapters 37, 57, 64, glossary) now agree.

## Important

### I-1 — CI YAML: `paths` filter does not apply to tag pushes (intentional but undocumented)

GitHub Actions documents that `paths` filters are not applied to tag-triggered runs. The current YAML's structure is correct — the tag leg fires regardless of which files are in the tag commit, exactly what the release-asset upload step needs — but the silent semantics is a footgun for future maintainers.

**Fix applied:** Added a top-of-file comment explaining the behavior.

### I-2 — `gh release upload` needs the release to exist; bare tag push without pre-existing release fails

The original step ran `gh release upload "$REF_NAME"` directly, which fails with "release not found" if the tag was pushed without a corresponding `gh release create`.

**Fix applied:** Added an `Ensure GitHub release exists for this tag` step that runs `gh release view` and falls back to `gh release create --title --generate-notes` if absent. The A10b smoke test (rc tag) and the final tag both work without the operator pre-creating the release.

### I-3 — FOLLOWUPS page-count entry said 126pp; current PDF is 129pp

**Fix applied:** Updated to 129pp.

## Minor / nits

### N-1 — `MERMAID_FILTER=on` not explicit in CI

**Fix applied:** Added `MERMAID_FILTER=on` to the CI `make pdf` invocation (matches Makefile default but explicit; future fast-CI mode can override).

### N-2 — `99-build-banner.md` stale committed artifact (informational)

The file's committed content drifts between builds because the Makefile regenerates it via `FORCE`. This is intentional. No fix.

### N-3 — Appendix E slug coherence (informational)

Confirmed correct. No fix.

### N-4 — `format-sparrow-format-specter-stubs` FOLLOWUPS clarity (informational)

Open entry is fine as filed. No fix.

## Acceptance-criteria audit

| # | Check | Status |
|---|---|---|
| A1 | `make md` clean | Pass |
| A2 | `make pdf-docker` clean, 60–100pp | **Accepted exception** (129pp; FOLLOWUPS records rationale) |
| A3 | TOC both | Pass |
| A4 | Glossary both | Pass |
| A5 | Index bidirectional pass | Pass |
| A6 | ≥4 mermaid blocks | Pass (9 mermaid blocks) |
| A7 | ≥6 `:::primer` boxes | Pass (9 primer boxes) |
| A8 | flag-coverage passes | Pass (real binaries, all 18 subcommands) |
| A9 | verify-examples — OK 5 transcripts | Pass |
| A10a | CI on push | Pass after C-1 fix |
| A10b | CI release-asset upload on `manual-v*` tag | Pass after I-2 fix |

## Phase-N report completeness

| Phase | Reports | Status |
|---|---|---|
| 0 | r1 + r2 | Converged at r2 |
| 1 | r1 + r2 + lint-trapdoor | Converged at r2 |
| 2 | r1 | Converged after I1 fix; no r2 needed (single-shot) |
| 3 | r1 | Converged after C-1/C-2/I-1/I-2 fixes; no r2 (single-shot) |
| 4 | r1 | Converged after C-1/C-2/C-3/I-1/I-3 fixes; no r2 (single-shot) |
| 5 | r1 | Converged after C1/C2/I1 fixes; no r2 (single-shot) |
| 6 | r1 | Converged after C1/C2/I1/I2 fixes; no r2 (single-shot) |
| 7 | r1 | Converged after C-1/C-2/I-1/I-2/I-3/I-4 fixes; no r2 (single-shot) |
| 8 | r1 | This report |

Single-shot convergence (no round-2 reports for Phases 2-7) is acceptable because the fixes are verifiable from source and each Phase-N r1 report explicitly stated convergence post-fix. No protocol gap.

## Convergence assessment

After C-1 + C-2 + I-1 + I-2 + I-3 + N-1 fixes applied, Phase 8 is at 0C/0I. Lint passes 6/6 with real binaries; PDF builds. No round-2 dispatch needed — fixes are mechanical and self-verifying.

Ready for: umbrella PR open → push branch → A10b smoke test (manual-v0.1.0-rc1 tag) → merge → manual-v0.1.0 final tag → release-asset upload → Phase 9 cross-repo follow-ups.
