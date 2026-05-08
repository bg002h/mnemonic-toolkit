# QuickStart design review — round 1

**Date:** 2026-05-08
**Spec reviewed:** `docs/superpowers/specs/2026-05-08-quickstart-design.md` (branch `quickstart/spec`)
**Reviewer:** feature-dev:code-architect
**Verdict:** Not converged. 2 critical / 3 important / 4 nits.

## Critical

### C-1 — `.cspell.json` is symlinked; QuickStart authors have no escape hatch for new words

The symlink locks the QuickStart's word list to the manual's. Any newcomer-voice term not in the manual's list fails cspell with no local fix. Editing the manual's `.cspell.json` for a QuickStart-specific word triggers `manual.yml` CI on a `docs/manual/**` touch.

**Fix:** Make `docs/quickstart/.cspell.json` a local file using cspell's `extends` key:

```json
{ "extends": "../manual/.cspell.json", "words": [] }
```

Inherits the full manual word list while giving the QuickStart its own extension point. Supported since cspell v6.

### C-2 — `pdf-docker` image tag collision: both Makefiles default to `mnemonic-manual-build:latest`

The manual's `DOCKER_IMAGE ?= mnemonic-manual-build:latest`. Cloning unchanged means concurrent or sequential `make pdf-docker` runs from both directories overwrite the same tagged image. Auditing which image built which PDF becomes impossible.

**Fix:** Set `DOCKER_IMAGE ?= mnemonic-quickstart-build:latest` in QuickStart Makefile. Layers cache-hit since Dockerfile is identical (symlinked).

## Important

### I-1 — No LaTeX template specified; `manual.latex` has `\printindex` hardcoded

`docs/manual/pandoc/templates/manual.latex:83` contains `\printindex`. Spec lists `pandoc/{preamble.tex, metadata.yaml, filters/}` for QuickStart but no `pandoc/templates/`. If the cloned Makefile retains `--template=$(TEMPLATES_DIR)/manual.latex` with `TEMPLATES_DIR = $(QUICKSTART_DIR)/pandoc/templates`, pandoc errors on the missing template. If it drops `--template`, pandoc's default works (no `\printindex`) — correct for the no-index QuickStart, but should be designed not discovered.

**Fix:** Spec says explicitly: QuickStart Makefile drops `--template` (pandoc's built-in latex template adequate for ~34pp; lacks `\printindex`). If output is unsatisfactory, add local `pandoc/templates/quickstart.latex` without `\printindex`.

### I-2 — `docs/manual/` symlink-target changes never re-run QuickStart CI

`quickstart.yml` triggers on `paths: docs/quickstart/**`. Updates to symlinked-into-QuickStart configs (`docs/manual/.markdownlint-cli2.jsonc`, `docs/manual/pandoc/filters/`) live in the QuickStart immediately but never re-validate it.

**Fix:** Add cross-paths to `quickstart.yml`:

```yaml
pull_request:
  paths:
    - 'docs/quickstart/**'
    - 'docs/manual/.markdownlint-cli2.jsonc'
    - 'docs/manual/pandoc/filters/**'
```

(`.cspell.json` removed from this list once C-1 makes it a local extension.) No need for `Dockerfile.build` (image rebuild not in CI path) or `verify-examples.sh` (logic-change caught next CI run).

### I-3 — Docker working-directory / mount lesson absent from §9

The manual's most painful CI bug (phase-8 C-1) — `working-directory: docs/manual` + `docker run -v "$PWD":/work` mounted only the manual dir, breaking git lookups in build banner — is not in §9. The spec runs host `make pdf` in CI (correct), but if a future Docker step is added, the lesson should already be on record.

**Fix:** Add to §9: "CI runs host `make pdf` (not `make pdf-docker`); the Docker target is for local reproducibility only. If a Docker `run` step is ever added, mount `$GITHUB_WORKSPACE` not `$PWD` and set `-w /work/docs/quickstart`."

## Minor / nits

### N-1 — Spec omits the paths-filter-tag-push comment

`manual.yml` carries a top-of-file comment explaining `paths` filters don't apply to tag-triggered runs. Spec doesn't mention carrying it forward.

**Fix:** Add to §6.3: "Carry the top-of-file comment from `manual.yml` explaining that `paths` filters are not applied to tag-triggered runs."

### N-2 — DANGER box re-authoring scope ambiguous

D5 locks "same canonical seed and DANGER pattern" but §5 says "re-author all prose in newcomer voice." Unclear if box body text is verbatim (inadvisable) or re-authored.

**Fix:** Add to §5 voice differences: "DANGER box body text is re-authored in newcomer voice; box trigger and severity level are identical to the manual's."

### N-3 — Q9 rc-tag cleanup not specified

Manual cycle deleted the rc tag after smoke test, leaving a dangling release. Q9 should specify cleanup.

**Fix:** Add to Q9: "After rc smoke passes: `git tag -d quickstart-v0.1.0-rc1 && git push origin :quickstart-v0.1.0-rc1 && gh release delete quickstart-v0.1.0-rc1 --yes`."

### N-4 — Phases 4 + 5 are collapsible

Each is ~4pp / 2 chapters — smaller than any manual phase. Splitting adds reviewer overhead without commensurate risk.

**Fix (optional):** Merge into a single "Part IV + V" phase. Total phase count 7 → 6.

## Verification of correct elements

| Check | Status |
|---|---|
| Symlink relative paths (transcripts, lint configs, Dockerfile, pandoc/filters) | OK; all spec paths verified |
| `find -type f` traversal of symlinked transcripts/ | Safe (transcripts/ not under SRC_DIR) |
| 5-transcript count consistency | OK (5 `.cmd` files in `docs/manual/transcripts/`) |
| CI dual-trigger on mixed PRs | Benign (each workflow runs independently) |
| Chapter page targets | Achievable with scope discipline |
| `paths`-not-applying-to-tags semantics in `on:` snippet | Correctly structured |

## Convergence assessment

After C-1 + C-2 fixes and I-1 + I-2 + I-3 additions, round 2 should be a brief spot-check. I-1 (template decision) is highest implementation risk — recommend deciding before ExitPlanMode. N-1, N-2, N-3 are one-line additions; N-4 is optional.
