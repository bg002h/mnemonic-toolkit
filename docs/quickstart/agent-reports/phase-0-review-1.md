# Phase 0 — spec-compliance review, round 1

**Date:** 2026-05-08
**Branch:** `quickstart/v0_1` at commit `c8ab0e0`
**Reviewer:** feature-dev:code-reviewer (spec-compliance focus)
**Verdict:** SPEC_COMPLIANT

## Compliant

§4 source-tree layout, §6.1 Makefile, §6.2 lint trimming, §6.3 CI workflow all match the spec. All 5 src/ subdirs, 18 .md stubs, 6 symlinks, local `.cspell.json`, local `tests/lint.sh`, local `pandoc/{preamble.tex,metadata.yaml}`, `Makefile` + `README.md` + `FOLLOWUPS.md` + `.github/workflows/quickstart.yml` present and structurally correct.

## Concern validation

1. **cspell `extends:` → `import:` deviation.** Confirmed correct. `extends` is not a valid cspell key; `import` (array form) is. Implementation uses `import:`. Verified `mdframed` resolves through the chain. **Spec correction required:** §spec C-1 should say `import:` not `extends:`.

2. **6 vs 7 symlinks.** 6 symlinks correct. The plan's "7 required" wording overcounted by treating the two lua filters inside the `pandoc/filters/` directory symlink as individual symlinks. Implementation matches the `ln -s` block exactly.

3. **Added `.gitignore`.** Consistent with `docs/manual/.gitignore` pattern (artifact classes: `build/`, `src/99-build-banner.md`, `mermaid-filter.err`). Justified.

## Missing (must fix)

None.

## Extra (must remove or justify)

- `.gitignore` — justified per Concern 3.

## Important — non-blocking for Phase 0 but must fix before Phase 1 ships mermaid content

**CI workflow lacks `PUPPETEER_CONFIG_FILE` and `PUPPETEER_EXECUTABLE_PATH` env vars.** The puppeteer-config write step writes `/etc/puppeteer-config.json` correctly, but mermaid-filter / Puppeteer needs the env vars set to pick it up. Phase 0 stubs have no mermaid blocks so this doesn't fail now; Phase 1's first mermaid block will trigger it. Add to CI workflow's `env:` block:

```yaml
PUPPETEER_CONFIG_FILE: /etc/puppeteer-config.json
PUPPETEER_EXECUTABLE_PATH: /usr/bin/chromium-browser
```

## Spec corrections (deviation justified, spec needs update)

**C-1: `extends:` → `import:` in cspell config.** Spec §C-1 should use the correct cspell key.

## Verdict

**SPEC_COMPLIANT.** Phase 0 implements all spec requirements. Two items to apply before Phase 1 starts: (a) CI puppeteer env vars, (b) spec text correction `extends:` → `import:`.
