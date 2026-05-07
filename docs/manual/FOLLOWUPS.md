# Manual FOLLOWUPS

Manual-local deferred-work tracker. Closes lockstep with toolkit
release cadence; every entry resolves into either a manual revision or
a confirmed retirement. Mirrors the FOLLOWUPS pattern used in
`mnemonic-toolkit/design/FOLLOWUPS.md` and the sibling repos.

## Open

### `release-history-auto-extract` — v0.2 candidate

`src/60-appendices/68-release-history.md` is hand-authored for v0.1.
For v0.2, replace with an auto-extraction script
(`tools/digest-changelogs.sh`) that reads each of the four sibling
repos' `CHANGELOG.md` files and emits a per-repo prose summary keyed
by tag.

**Why:** four repos × per-tag updates = manual diff drift if maintained
by hand. Auto-extraction localises the toil to a single script.

**How to apply:** stage during a v0.2 cycle when there is also a
material content edit elsewhere; do not gate v0.2 on the script alone.

### `figures-cache-implementation` — v0.2 candidate

`make figures-cache` is currently a stub. It should pre-render every
` ```mermaid ` block under `src/` to SVG, key the cache file by
SHA-256 of the source block, and write into `figures/cache/`. The
`MERMAID_FILTER=skip` mode then consumes the cache to support builds
on hosts where `mermaid-filter` (Chromium) is unavailable.

**Why:** Chromium / Puppeteer is a large dependency. Some contributors
on minimal Linux images can't render mermaid live. Pre-rendered cache
lets them still produce a PDF.

**How to apply:** v0.2; not blocking v0.1.

### `npm-package-pinning` — v0.2 candidate

`Dockerfile.build` uses `^` semver ranges on npm packages. For full
reproducibility, pin to exact versions and check in a lockfile
(`package-lock.json` in `docs/manual/`).

**Why:** semver-range installs aren't bit-reproducible across rebuilds.

**How to apply:** introduce `package.json` + `package-lock.json` in v0.2.

### `cspell-dictionary-curation`

`cspell` lint will produce many false positives until a project
dictionary is maintained. Add `docs/manual/.cspell.json` with a
curated word list of m-format-star vocabulary (`mnemonic`, `codex32`,
`mdframed`, etc.) once Phase 1 stabilises the glossary.

## Closed

(none yet)
