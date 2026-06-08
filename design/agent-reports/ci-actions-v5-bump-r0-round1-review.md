# R0 Architect Review — SPEC_ci_actions_v5_bump.md — Round 1

> Reviewer had Read/Glob/Grep/Bash/gh; parent persists. Source basis: `HEAD == 15236ee`; live upstream action versions confirmed.

**Verdict: GREEN (0 Critical / 0 Important).** 2 Minor.

## Critical
None.
## Important
None.

## Minor

**M1 — Item-2 matrix understates upload-artifact@v5's PR validation (manual-gui build job is NOT tag-gated).**
SPEC `:55` says manual-gui's "artifact pair is tag-gated → not on PR," and `:62` lists "manual-gui release-job artifacts" in the un-validated tail. Half-wrong from source: `manual-gui.yml:115` `build:` job has **no `if:` guard** and its comment (`:112-113`) states "Required on PR (regression detection) and on tag push." Since manual-gui PR paths include `.github/workflows/manual-gui.yml` (`:20`), the build job **runs on the v5-bump PR** → **upload-artifact@v5 (`:151,:158`) IS PR-validated.** Only the **download** half (`:192,:198`) is in the tag-gated `release:` job (`:176`). Strengthens the rollout (3 of 4 majors — checkout/setup-node/upload-artifact — PR-validated; only download-artifact rests on research-benign). *Fix:* matrix → manual-gui validated-set = "checkout@v5, setup-node@v5, **upload-artifact@v5** (download pair tag-gated → not on PR)"; un-validated tail → "manual-gui **download**-artifact pair."

**M2 — Floating-vs-SHA pin trade-off not stated.** Live `uses: actions/` grep returns ONLY `@v4` repo-wide (zero SHAs, zero stragglers) → repo uniformly floats majors → floating `@v5` is consistent (SPEC `:38` correct). Trade-off omitted: SHA-pinning is the supply-chain-hardening best practice but inconsistent with all 25 existing sites + out of scope here. Note, not blocker (repo does not SHA-pin). *Optional:* one line acknowledging SHA-pinning as a deferred hardening option (fold into `ci-actions-catch-up-to-latest-majors`).

## Verified-correct (from source + upstream)

- **25-count + breakdown EXACT.** Fresh grep over 7 workflows = exactly 25 `actions/(checkout|setup-node|upload-artifact|download-artifact)@v4`; broader `@v4` grep = same 25. Breakdown matches line-for-line: checkout×15 (rust 37,116,130,147,172,212,216; manual-gui 36,119,187; manual 30; quickstart 32; technical-manual 48; install-pin-check 37; sibling-pin-check 39); setup-node×4 (manual 52, manual-gui 83, quickstart 53, technical-manual 56); upload-artifact×4 (manual 120, manual-gui 151,158, quickstart 86); download-artifact×2 (manual-gui 192,198). All line numbers live-accurate.
- **`dtolnay/rust-toolchain` is the only other `uses:` action** (rust.yml:133 @nightly, :177 @1.85.0) — composite/shell, no Node JS → correctly out of scope. No other JS action (releases use `gh release` CLI).
- **Self-fire matrix verified per-workflow:** rust ✅ (PR path `:26` own file), manual-gui ✅ (PR path `:20`; lint+build both no-`if:` → run on PR; release tag-gated), technical-manual ✅ (PR path `:32`; independent setup-node@v5 validation), sibling-pin-check ✅ (bare `push:` + PR branches:master), manual ❌ (paths docs/manual only), quickstart ❌ (paths docs/quickstart only), install-pin-check ❌ (tag-only).
- **manual-gui download is by-NAME** (`:192` name: …-pdf, `:198` name: …-html) → download-artifact v5's by-ID change inapplicable (by-name unchanged since v4).
- **No `package.json` anywhere** → setup-node v5 auto-cache inapplicable; all 4 setup-node steps byte-identical (node-version '22', no cache key) → the 2 PR-validated (manual-gui, technical-manual) genuinely prove the 2 un-validated (manual, quickstart).
- **Decision sound:** v5 = Node 24 for all 4 (clears 2026-06-16 forced-Node-24 + 2026-09-16 Node-20-removal); floating v5 tags exist. Latest is real + risky (download-artifact v8.0.1 no-auto-decompress + hash-error; checkout v6.0.3 $RUNNER_TEMP creds) concentrated in the un-PR-testable manual-gui release job → defer to follow-up = correct.
- **checkout v5 safe for manual-gui gh-pages push (v5-not-v6 crux):** checkout v5 has no behavioral change beyond Node 24 — credentials stay in workspace; only v6 relocates to `$RUNNER_TEMP`. manual-gui release does a raw `git push` on persisted creds → v5 leaves it untouched.
- **Breaks-master-PR-won't-catch — all benign:** manual/quickstart upload-artifact@v5 (Node-24-only, no semantics change, no download consumer; release asset via `gh release upload`); manual-gui download-artifact@v5 (by-name, v5-unchanged). 
- **Disposition:** CI-only → byte-identical → no bump/tag; no schema_mirror/manual-mirror/sibling/GUI lockstep. FOLLOWUP flip + catch-up tracker (with v8/v6 throwaway-tag rehearsal scope) appropriate.

## Path
GREEN — clear to implement. Fold the 2 Minors (accuracy). sed-replace 25 `@v4`→`@v5`, leave dtolnay, actionlint + grep-proof, PR rollout, squash-merge, git-revert-ready.
