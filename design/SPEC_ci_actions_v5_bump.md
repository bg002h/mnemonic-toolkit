# SPEC — bump GitHub JS-action majors `@v4→@v5` (clear the Node-20 runtime deprecation)

**Cycle:** CI housekeeping (deprecation-driven, time-bound).
**Date:** 2026-06-07.
**Source SHA:** `origin/master` == local `HEAD` == `15236ee`.
**Disposition:** CI only — **no version bump, no tag** (binary byte-identical).
**Rollout:** **on a PR** (not direct-to-master) — validates the self-firing subset before merge.
**Recon:** `cycle-prep-recon-ci-actions-node20-runtime-major-bump.md` (+ its post-research ADDENDUM).
**Resolves:** `ci-actions-node20-runtime-major-bump-deferred`. **Files:** `ci-actions-catch-up-to-latest-majors` (investigate v6/v6/v7/v8 AFTER v5 is confirmed stable — user direction).
**Locksteps:** none.

---

## 0. Decision + why v5 (not latest)

GitHub forces `@v4` JS-actions onto Node 24 on **2026-06-16** ("may not work as expected") and removes Node 20 from runners **2026-09-16**. The fix is to bump the action majors to one that natively targets Node 24.

Live majors (June 2026, `gh api`): **checkout v6.0.3, setup-node v6.4.0, upload-artifact v7.0.1, download-artifact v8.0.1.** Going to *latest* is a multi-major jump with real, **un-PR-testable** risk concentrated in `manual-gui`'s tag-gated release job:
- **download-artifact v8:** stops auto-decompressing downloads + hash-mismatch now errors (was warn).
- **checkout v6:** persisted git credentials move to `$RUNNER_TEMP` (manual-gui does a gh-pages `git push`).

**`@v5` is the minimal Node-24 major.** Every v5 breaking change is verified inapplicable to this repo (table below). It clears the deprecation with the smallest, fully-vetted jump. The floating `v5` tag exists on all 4 actions (confirmed). **Decision (user): pin `@v5` now; investigate v6/v7/v8 in a follow-up only after v5 is confirmed OK in the wild.**

### v5 behavioral-change table (all inapplicable here)
| action | v5 delta | applies? |
|---|---|---|
| checkout | Node 24; min runner v2.327.1; no behavioral change | No |
| setup-node | Node 24; auto-cache IF `package.json` has `packageManager` | No — repo has **zero `package.json`** |
| upload-artifact | Node 24; "not a breaking change per-se" (no artifact-semantics change) | No |
| download-artifact | Node 24; **by-ID** download path change | No — `manual-gui` downloads **by `name:`** (`:192,:198`) |

**Min-runner caveat:** v5 needs Actions Runner ≥ v2.327.1 — GitHub-hosted `ubuntu-latest` auto-updates well past this (only a self-hosted concern; none here).

---

## Item 1 — The bump: `@v4 → @v5` across 25 sites / 7 workflows

Pin **floating `@v5`** (matches the repo's existing floating-`@v4` style — a live grep shows the repo uniformly floats majors, zero SHA pins; floating picks up v5 patches). Trade-off (R0-r1 M2): SHA-pinning is the supply-chain-hardening best practice (immune to tag-repoint), but adopting it for these 25 sites would be inconsistent with the established style and is out of scope for a deprecation-clearing bump — deferred as an option in the catch-up follow-up. The 25 sites:
- `checkout@v4 → @v5` ×15: `rust.yml:37,116,130,147,172,212,216`; `manual-gui.yml:36,119,187`; `manual.yml:30`; `quickstart.yml:32`; `technical-manual.yml:48`; `install-pin-check.yml:37`; `sibling-pin-check.yml:39`.
- `setup-node@v4 → @v5` ×4: `manual.yml:52`, `manual-gui.yml:83`, `quickstart.yml:53`, `technical-manual.yml:56`.
- `upload-artifact@v4 → @v5` ×4: `manual.yml:120`, `manual-gui.yml:151,158`, `quickstart.yml:86`.
- `download-artifact@v4 → @v5` ×2: `manual-gui.yml:192,198`.

**Out of scope (NOT bumped):** `dtolnay/rust-toolchain@nightly` + `@1.85.0` (rust.yml) — composite/shell actions, not Node-JS → the Node-20 deprecation does not apply. (Line numbers are at-write snapshots; the implementer greps `uses: actions/<name>@v4` and replaces all — `sed` is acceptable here because, unlike a code change, every `actions/*@v4` occurrence is in-scope and the diff is reviewed.)

---

## Item 2 — PR-based rollout + validatability (the rollback strategy)

Open a **PR** from a branch (e.g. `ci/actions-v5`). On the PR, a workflow runs its NEW (v5) version **iff** the PR's changed paths match its `pull_request.paths`. Since the PR edits `.github/workflows/*.yml`:

| workflow | self-fires on the PR? | v5 actions validated on the PR |
|---|---|---|
| `rust` | ✅ PR paths include `.github/workflows/rust.yml` | checkout@v5 |
| `manual-gui` | ✅ PR paths include `.github/workflows/manual-gui.yml` | checkout@v5, setup-node@v5, **upload-artifact@v5** (the `build:` job has NO `if:` guard → runs on PR; only the `download`-artifact pair in the tag-gated `release:` job is not on PR) |
| `technical-manual` | ✅ PR paths include own file | checkout@v5, setup-node@v5 |
| `sibling-pin-check` | ✅ bare `push:` fires on the branch push | checkout@v5 |
| `manual` | ❌ PR paths = `docs/manual/**` + render-mermaid (no self-path) | — (validates on next `docs/manual/**` change) |
| `quickstart` | ❌ PR paths = `docs/quickstart/**` + … (no self-path) | — (validates on next `docs/quickstart/**` change) |
| `install-pin-check` | ❌ tag-only (`mnemonic-toolkit-v*`) | — (validates on next release tag) |

**So the PR push-validates `checkout@v5` (4 workflows incl. all 7 of rust's checkouts) + `setup-node@v5` (2 workflows) + `upload-artifact@v5` (manual-gui build job) on real runners before merge — 3 of the 4 majors.** The un-validated tail — `manual`/`quickstart` (checkout + setup-node + upload-artifact — their jobs don't self-fire on a workflow-only edit), `install-pin-check` (checkout, tag-only), and the `manual-gui` **download**-artifact pair (tag-gated `release:` job) — is **verified-benign for v5** (identical setup-node to the validated ones; upload-artifact v5 = Node-24-only no-semantics-change; download-artifact v5 by-ID change inapplicable to by-name). Only `download-artifact@v5` rests purely on research, not a live run. Self-trigger-path gaps for `manual`/`quickstart` are NOT fixed here (keep the bump atomic; out of scope) — noted as a possible improvement in the catch-up follow-up.

**Rollback:** the merge is **squash → one atomic commit**; if any post-merge run (or the next tag) breaks, `git revert <sha>` restores `@v4` in one commit. CI-only files → no build artifacts / released versions / downstream consumers depend on them. **No long-lived rollback branch needed** — the PR is the pre-merge gate, `git revert` is the post-merge undo.

---

## Item 3 — FOLLOWUPs

- Flip `ci-actions-node20-runtime-major-bump-deferred` → **resolved** (the `@v5` bump clears the Node-20 deprecation; record the v5-vs-latest decision + the validation scope).
- File `ci-actions-catch-up-to-latest-majors` (**open**): investigate `@v5 → v6/v6/v7/v8` **after v5 is confirmed stable in the wild** (user direction). Must vet: **download-artifact v8** (no-auto-decompress + hash-error — rehearse `manual-gui`'s tag-gated release job via a throwaway `manual-gui-vTEST` tag), **checkout v6** (`$RUNNER_TEMP` credentials vs the gh-pages `git push`), upload-artifact v7 (archive/ESM — benign), setup-node v6 (npm-only caching — benign). Also in scope for that follow-up: **evaluate SHA-pinning vs the current floating-major style** (R0-r1 M2 — supply-chain hardening) and optionally add self-trigger `.github` paths to `manual`/`quickstart` so future workflow edits self-validate.

---

## 4. Verification

1. **actionlint** on all 7 workflows post-edit.
2. **Grep proof:** zero `actions/(checkout|setup-node|upload-artifact|download-artifact)@v4` remain; exactly 25 `@v5`; `dtolnay/rust-toolchain` untouched.
3. **PR CI (pre-merge gate):** open the PR; confirm `rust` + `manual-gui` + `technical-manual` + `sibling-pin-check` runs go **green** with the v5 actions (checkout@v5 + setup-node@v5 exercised). Do NOT merge until green.
4. **Post-merge:** the `push: branches:[master]` triggers re-run on master — confirm green. The tag-gated tail (`install-pin-check`, `manual-gui` release) validates on its next natural tag; `git revert`-ready.
5. **No bump/tag**; no lockstep.

---

## 5. Ship plan
1. Branch `ci/actions-v5`. Apply Item 1 (25 edits). Apply Item 3 (FOLLOWUP flip + new tracker). Add this SPEC + the R0 review(s).
2. Verify §4.1–4.2 locally (actionlint + grep).
3. Push branch; open PR; watch §4.3 (self-firing subset green).
4. **Squash-merge** when green. Verify §4.4 (master runs green).
5. Memory + MEMORY.md index.

### Out of scope
- v6/v6/v7/v8 (the catch-up follow-up).
- `dtolnay/rust-toolchain` versions; `manual`/`quickstart` self-trigger-path fix.
- Any crate/CLI/docs-content change.
