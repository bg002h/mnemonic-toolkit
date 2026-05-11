# tech-manual v0.1 Phase 1.0 review r1 — pipeline scaffold (commit 445845c)

Reviewed: 45 files in `docs/technical-manual/` — Makefile, Dockerfile.build, tests/lint.sh, tests/api-surface-coverage.sh, tests/verify-examples.sh, tests/filter-smoke.sh, pandoc/{metadata.yaml,preamble.tex,templates/manual.latex,filters/}, 25 stub src/ chapters, README.md, AUTHORING.md, FOLLOWUPS.md, .cspell.json, .markdownlint-cli2.jsonc, .gitignore.

## Critical: 0

## Important: 2 (both folded inline)

### I1 — `SOURCE_DATE_EPOCH` not forwarded into `pdf-docker` container

**File:** `docs/technical-manual/Makefile` lines 195–201. The `pdf-docker` recipe did not pass `SOURCE_DATE_EPOCH` / `FORCE_SOURCE_DATE` into the container, so SPEC §7 A11 same-host byte-identity gate was not satisfied via the docker path.

**Fix applied:** added conditional `-e SOURCE_DATE_EPOCH=... -e FORCE_SOURCE_DATE=1` to the docker run line when `SOURCE_DATE_EPOCH` is set in the outer environment.

### I2 — Stale `flag-coverage` in `make help` output and AUTHORING.md

**Files:** `Makefile` lines 13, 18; `AUTHORING.md` line 182.

**Fix applied:** updated Makefile `make help` listing to say `api-surface-coverage (hint, warning-only)`; updated AUTHORING.md "Six checks" enumeration to match.

## Low: 3 (all folded inline)

### L1 — AUTHORING.md references wrong index/glossary paths

`AUTHORING.md` line 68 referenced `src/60-appendices/69-index-table.md`; line 120 referenced `src/60-appendices/61-glossary.md`. Tech-manual paths are `src/60-back-matter/{61-glossary.md,62-index-table.md}`.

**Fix applied:** both path strings updated.

### L2 — Stale "user manual" header in `pandoc/templates/manual.latex` and `pandoc/preamble.tex`

**Fix applied:** updated to "technical manual" and corrected the preamble.tex header comment path.

### L3 — `figures/cache/`, `transcripts/`, `examples/` missing from committed tree

**Fix applied:** added `.gitkeep` to each so the layout matches the README/SPEC documentation.

## Nit: 1 (folded inline)

### N1 — `filter-smoke.sh` comment said "docs/manual/"

**Fix applied:** clarified the comment to note tech-manual scope while preserving the variable name `MANUAL_DIR` for Makefile-pattern compatibility.

## Post-fix verification

- `make pdf` green (30pp stub, SOURCE_DATE_EPOCH=1746921600 byte-identical across runs).
- `make lint` 6/6 green.

## Disposition

0C / 0I / 0L / 0N at Phase 1.0 close.
