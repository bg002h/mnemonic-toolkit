# cycle-prep recon — 2026-06-07 — ci-actions-catch-up-to-latest-majors (+ embedded self-trigger sub-item)

**Origin/master SHA at recon time:** `beab477`
**Local branch:** `master`
**Sync state:** up-to-date (0 ahead / 0 behind)
**Untracked:** only recon scaffolding.

Slug verified: `ci-actions-catch-up-to-latest-majors`. **Citations ACCURATE** (filed this session at `beab477`). "Double duty" — the FOLLOWUP bundles TWO very different things: (B) the big/risky v6/v7/v8 catch-up, and (A) a small/safe `manual`+`quickstart` self-trigger improvement. **Headline: SPLIT them.** (A) is safe + independent + self-validating → do NOW; (B) is gated on "v5 stable in the wild" which is **NOT yet met** → stays deferred.

---

## Per-slug verification
### `ci-actions-catch-up-to-latest-majors`
- **WHAT:** bump JS-actions `@v5 → v6/v6/v7/v8` (latest), AFTER v5 proves stable; + (embedded) add self-trigger `.github` paths to `manual`/`quickstart`; + evaluate SHA-pinning.
- **Citations:**
  - "all 7 workflows now at `@v5`" — **ACCURATE**: 15 checkout + 4 setup-node + 4 upload + 2 download = 25 `@v5`, 0 `@v4`.
  - "Live latest: checkout v6.0.3 / setup-node v6.4.0 / upload-artifact v7.0.1 / download-artifact v8.0.1" — **ACCURATE** (re-confirmed via `gh api releases/latest` at recon time; unchanged from the v5-bump cycle).
  - **download-artifact v8** "stops auto-decompressing + hash-mismatch errors" — **ACCURATE** (upstream release-notes, verified this session; the v8 breaking change). Risk in `manual-gui.yml` release job (`:192,:198` by-name download → release-asset + gh-pages). Tag-gated → un-PR-testable → rehearsal required. Confirmed.
  - **checkout v6** "creds → `$RUNNER_TEMP`, runner ≥ v2.329.0" — **ACCURATE** (upstream v6 note). manual-gui release does a raw gh-pages `git push` (`:187` checkout precedes it) on persisted creds. Confirmed risk.
  - upload-artifact v7 (archive/ESM) + setup-node v6 (npm-only cache) "benign" — **ACCURATE** (inapplicable to current `name:`/`path:` usage + no `package.json`).
  - **Precondition "do ONLY after v5 confirmed stable via a clean release-tag run"** — **STILL UNMET**: `git tag --contains beab477` = empty (no release tag pushed since the v5 bump); the tag-gated v5 paths (`install-pin-check`, `manual-gui`/`manual`/`quickstart` release jobs) have NOT run on v5. So Cycle B's gate is open. Confirmed correct to keep deferred.
- **Action for brainstorm spec:** **SPLIT (see scope).** Cycle B (v6/v7/v8) stays deferred (precondition unmet); refresh nothing now. Extract the self-trigger sub-item → Cycle A (below). Cite SHA `beab477`.

### Embedded sub-item — `manual`/`quickstart` self-trigger paths (the "double duty" / "we will tackle it too")
- **WHAT:** add each workflow's own `.github/workflows/<self>.yml` to its push+PR `paths`, so a workflow-file edit self-fires it (mirrors `technical-manual`/`rust`/`manual-gui`, which each carry 2 self-path refs — push + PR).
- **Citations / current state (verified):**
  - `manual.yml` push.paths (`:4-6` = `docs/manual/**`, `render-mermaid`) + PR.paths (`:10-12` same) — **no self-path** → does NOT self-fire. Confirmed.
  - `quickstart.yml` push.paths (`:4-6`) + PR.paths (`:10-14` = `docs/quickstart/**`, `docs/manual/.markdownlint-cli2.jsonc`, `docs/manual/pandoc/filters/**`, `render-mermaid`) — **no self-path** → does NOT self-fire. Confirmed.
  - Mirror target: `technical-manual.yml`/`rust.yml`/`manual-gui.yml` each list their own `.github/workflows/<self>.yml` in BOTH push.paths and pull_request.paths (2 refs each). Confirmed.
- **Cost flag (must surface to R0):** `manual.yml` + `quickstart.yml` are **HEAVY** workflows (pandoc + full `texlive-*` install + `cargo install` sibling CLIs + `cargo build` + PDF build / `make audit`), unlike the lint-only `technical-manual`. Self-triggering them fires a multi-minute heavy build on every workflow-file edit. Workflow edits are RARE, so the cost is bounded — but it's a real tradeoff (not the near-free self-trigger that `technical-manual` has). R0 should confirm "accept the heavy self-fire for the validation benefit."
- **Self-validating bonus:** the PR that ADDS the self-path to `manual.yml`/`quickstart.yml` will — by that very edit — fire `manual`/`quickstart` on the PR, giving their `setup-node@v5` + `upload-artifact@v5` a **live PR run** — retroactively closing the "rode in on research-benign, never a live run" gap from the v5-bump cycle. The fix proves itself on its own PR.
- **Action for brainstorm spec:** add `- '.github/workflows/manual.yml'` to manual.yml push.paths + PR.paths; `- '.github/workflows/quickstart.yml'` to quickstart.yml push.paths + PR.paths (4 inserted lines). Cite SHA `beab477`.

---

## Cross-cutting observations
1. **The FOLLOWUP conflates a do-now-safe item with a do-later-risky one.** (A) self-trigger = a trigger-paths edit, zero action-version risk, not gated on v5-stability. (B) v6/v7/v8 = real breaking changes (download-artifact v8 decompress, checkout v6 creds) in an un-PR-testable tag-gated job, gated on v5 proving stable. Bundling would either (wrongly) hold the safe fix behind the v5-stability gate, or (wrongly) drag the risky bump forward. **Split.**
2. **Cycle B precondition is genuinely unmet** — no release tag has run on v5; do not start B until one does (and then rehearse v8 on a throwaway `manual-gui-vTEST` tag).
3. **No locksteps** for either: CI-only `.github/workflows/*.yml` edits, no clap-flag/CLI/codec surface → no `schema_mirror`/manual-mirror/sibling/GUI coupling.
4. SHA-pinning eval stays parked in Cycle B (out of scope for the self-trigger fix).

---

## Recommended brainstorm-session scope
**SPLIT into two cycles; do Cycle A now, keep Cycle B deferred.**

- **Cycle A (NOW) — `manual`/`quickstart` self-trigger paths.** Tiny (4 inserted lines), CI-only, **no version bump/tag**. SemVer N/A. No locksteps. Mandatory R0 (must confirm: the heavy-self-fire cost tradeoff; that adding the self-path doesn't change checkout-nesting/WS-derivation for any gate; that the PR self-validates manual/quickstart's v5 actions). Rollout: a PR (which self-fires manual+quickstart → live v5 validation + proves the self-trigger works). Resolves the embedded sub-item; UPDATE the catch-up FOLLOWUP to drop the self-trigger bullet (absorbed) and keep B's v6/v7/v8 + SHA-pin scope. ~1 commit.
- **Cycle B (DEFERRED) — `@v5 → v6/v7/v8` catch-up.** Precondition: a clean release-tag run proves v5 stable. Then per-major vet + **rehearse download-artifact v8 on a throwaway `manual-gui-vTEST` tag** + checkout v6 creds-vs-gh-pages-push + SHA-pin eval. Keep as the `ci-actions-catch-up-to-latest-majors` FOLLOWUP (minus the extracted self-trigger sub-item). Bigger, risk-managed, own cycle.

Ordering: A independent of B; A can ship immediately. B waits on the v5-stability signal (a future release tag).
