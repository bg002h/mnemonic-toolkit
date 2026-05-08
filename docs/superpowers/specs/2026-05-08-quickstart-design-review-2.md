# QuickStart design review — round 2

**Date:** 2026-05-08
**Spec reviewed:** `docs/superpowers/specs/2026-05-08-quickstart-design.md` (branch `quickstart/spec`, commit `fdff3ba`)
**Round-1 report:** `2026-05-08-quickstart-design-review-1.md`
**Reviewer:** feature-dev:code-architect
**Verdict:** Not converged at start of r2. 1 critical (introduced by r1) / 1 important (new) / 0 nits. Both fixed inline.

## Verification of round-1 fixes

| Item | Verdict |
|---|---|
| C-1 (cspell extends) | PASS in §2 D3, §4 file-tree, §4 rationale, §6.3 (no `.cspell.json` in cross-paths). **Caveat: §6.2 table cell still stale — see C-R1.** |
| C-2 (Docker tag) | PASS — `DOCKER_IMAGE ?= mnemonic-quickstart-build:latest` in §6.1 with rationale |
| I-1 (template) | PASS — `--template` dropped; rationale in §6.1 |
| I-2 (cross-paths) | PASS — `quickstart.yml` cross-paths correct; rationale stated |
| I-3 (Docker mount lesson) | PASS — §9 carries the host-`make pdf` choice + `$GITHUB_WORKSPACE` rule |
| N-1 (paths-tag-push comment) | PASS |
| N-2 (DANGER box re-author) | PASS |
| N-3 (rc-tag cleanup) | PASS (minor nit: missing space before `&&`; fixed inline) |
| N-4 (phase merge) | PASS — 6 phases in §8 |

## New issues introduced by r1 fixes

### C-R1 — §6.2 lint table still says "symlinked `.cspell.json`"

**Location:** §6.2 line 153.

Stale copy-paste from pre-C-1. Contradicts D3, §4 file-tree, §4 rationale, and §6.3 (which all correctly describe `.cspell.json` as local-with-extends). An implementer following §6.2 would create a symlink and defeat the entire C-1 fix.

**Fix applied:** Changed cell to "local `.cspell.json` (extends manual's via cspell `extends` key)".

### I-R1 — cspell `extends` path resolution not grounded

**Context:** Local `.cspell.json` contains `"extends": "../manual/.cspell.json"`. Whether `../` resolves to **config-file location** or **invocation CWD** is non-obvious. cspell docs state "relative paths are relative to the config file" for `dictionaryDefinitions`; the same semantics almost certainly apply to `extends` but the spec did not record this assumption.

**Fix applied:** Added one sentence to §4 rationale paragraph: "cspell resolves `extends` relative to the config file's location (not the invocation CWD), so `../manual/.cspell.json` is stable regardless of where cspell is invoked." Phase-0 fallback noted (absolute path or repo-root `cspell.config.yaml`).

## Convergence assessment

After C-R1 and I-R1 fixes (one cell + one sentence), spec is at **0C/0I**. No round-3 dispatch needed — fixes are mechanical and self-verifying. Spec is ready for writing-plans.
