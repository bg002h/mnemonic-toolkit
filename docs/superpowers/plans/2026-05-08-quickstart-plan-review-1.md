# QuickStart plan review — round 1

**Date:** 2026-05-08
**Plan reviewed:** `docs/superpowers/plans/2026-05-08-quickstart.md` (branch `quickstart/spec`, commit `c1b7907`)
**Reviewer:** feature-dev:code-architect
**Verdict:** Not converged at start of r1. 3 critical / 3 important / 4 minor. All fixed inline.

## Critical

### C-1 — Task 0.6 `metadata.yaml` `header-includes:` duplicates preamble.tex content; fatal LaTeX double-definition

The plan's Task 0.6 metadata.yaml had `header-includes:` with `\usepackage{fvextra}` and `\DefineVerbatimEnvironment{Highlighting}{...}`. Task 0.5 preamble.tex carries the same. Both land in pandoc's preamble; LaTeX errors on the duplicate `\DefineVerbatimEnvironment`.

Manual's `pandoc/metadata.yaml:38-45` documents the rationale: "per-package `\usepackage{...}` and `\makeindex` directives are loaded from `pandoc/preamble.tex` via `-H preamble.tex` — pandoc's `header-includes:` mechanism does not reliably round-trip raw LaTeX in 3.x. preamble.tex is the single load site."

**Fix applied:** Removed the entire `header-includes:` block from Task 0.6's metadata.yaml template.

### C-2 — Task 0.10 CI workflow missing puppeteer config / MERMAID_FILTER / ensure-release-exists steps; copy-paste leaves `manual-v` in tag if-condition

Task 0.10 Step 2 listed differences from `manual.yml` but did not provide explicit step scaffolds for the puppeteer-config write, the MERMAID_FILTER=on flag, or the `gh release create` fallback. Engineer must read manual.yml and infer correctly. Also: the asset-upload `if:` condition would carry `manual-v` from the clone if not explicitly fixed.

**Fix applied:** Added three explicit YAML step scaffolds to Task 0.10 Step 2 (puppeteer config write, build PDF with MERMAID_FILTER=on, ensure-release-exists). Added explicit instruction: "the asset-upload `if:` condition becomes `startsWith(github.ref, 'refs/tags/quickstart-v')`."

### C-3 — Task 0.8 stub body uses literal "Phase N" — placeholder in every file

Task 0.8 Step 1 said each stub body should be `(Phase N — to be authored.)\n`. "N" is a fill-in-the-blank for 18 files. Violates "no placeholders" criterion.

**Fix applied:** Added explicit "Phase" column to the stub-mapping table; per-file body uses literal digit. Build banner row carries `(Makefile-managed — do not author.)` per minor M-4.

## Important

### I-1 — Task 0.3 Step 3 cspell pass criterion incompatible with `--no-summary`

`--no-summary` suppresses the `Issues found:` line. The expected output `Issues found: 0 in 0 files.` is unreachable; same with the failure-detection string.

**Fix applied:** Changed Task 0.3 Step 3 to drop `--no-summary` and use exit code as the pass criterion: `echo "exit $? — expected 0 (mdframed inherited from manual word list)"`.

### I-2 — Task 5.3 Step 2 failure recovery is vague forward-reference

The plan said "debug per manual cycle's Phase 8 lessons" without enumerating them. Manual cycle hit specific failure modes; an executor shouldn't re-discover.

**Fix applied:** Replaced the hint with a 4-item enumerated failure-mode checklist (lychee 404 with `--strip-components=1`; mermaid Chromium → puppeteer config; PDF build → reproduce locally; release upload → ensure-release-exists step).

### I-3 — Task 0.11 Step 3 hardcodes "5 transcripts" — stale if manual adds more

The expected output `OK (5 transcripts pass)` is brittle; if the manual gains a 6th transcript in a future revision, the QuickStart's Phase 0 smoke test will fail with a stale-expectation false negative.

**Fix applied:** Added a pre-step `ls docs/quickstart/transcripts/*.cmd | wc -l` to read the actual count; expected output becomes `OK (N transcripts pass)` where N matches the count.

## Minor

### M-1 — `DOCKER_IMAGE` not exercised in CI workflow (note added)

Added one-line note to Task 0.10: `DOCKER_IMAGE` is Makefile-local; CI uses host `make pdf`, not `make pdf-docker`. No CI env var needed.

### M-2 — Tasks 3.4 and 4.5 underspecified

Phase 1/2 templates have full lint/commit/reviewer detail; Phases 3/4 short-circuited "per Task 1.5/2.7." Engineer must mentally replicate.

**Fix applied:** Tasks 3.4 and 4.5 now carry: explicit `git add` paths, commit message shape, and reviewer-prompt focus areas (one line each).

### M-3 — Task 5.1 Step 2 no command shown

"Run make lint + make verify-examples with real binaries" without echoing the full invocation.

**Fix applied:** Task 5.1 Step 2 carries the full invocations with `MNEMONIC_BIN=/scratch/...` paths, mirroring Phase 0 Step 3.

### M-4 — 99-build-banner.md stub Phase assignment ambiguous

**Fix applied:** Build-banner stub body now reads `(Makefile-managed — do not author.)\n` to prevent inadvertent authoring.

## Convergence

After 3C + 3I + 4M fixes (all mechanical), plan is at 0C/0I. No round-2 dispatch needed; fixes are self-verifying.
