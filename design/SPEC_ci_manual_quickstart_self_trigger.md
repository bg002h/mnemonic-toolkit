# SPEC — self-validating triggers for `manual.yml` + `quickstart.yml` (Cycle A)

**Cycle:** CI housekeeping (self-trigger parity).
**Date:** 2026-06-07.
**Source SHA:** `origin/master` == local `HEAD` == `beab477`.
**Disposition:** CI only — **no version bump, no tag** (binary byte-identical).
**Rollout:** **on a PR** (which self-fires `manual`+`quickstart` → live-validates their `@v5` actions).
**Recon:** `cycle-prep-recon-ci-actions-catch-up-and-self-trigger.md` (the "double duty" split: this is Cycle A; the v6/v7/v8 catch-up is deferred Cycle B).
**Resolves:** the embedded self-trigger sub-item of `ci-actions-catch-up-to-latest-majors` (extracted here; the FOLLOWUP is updated to drop it).
**Locksteps:** none.

---

## 0. Why

`manual.yml` and `quickstart.yml` do NOT list their own `.github/workflows/<self>.yml` in their trigger `paths`, so a workflow-file edit does not self-fire them — unlike `technical-manual.yml`, `rust.yml`, `manual-gui.yml` (each carries the self-path in push+PR). Consequence (seen in the `@v5` bump cycle `beab477`): their `setup-node@v5`/`upload-artifact@v5` "rode in on research-benign" — never exercised on a live PR run, because the bump PR (which only edited `.github/workflows/*.yml`) didn't trigger them. Adding the self-path is a small future-proofing parity fix: future workflow edits self-validate.

**Self-validating bonus:** the PR that lands this fix edits `manual.yml`/`quickstart.yml`, which — once the self-path is present — fires `manual`+`quickstart` **on that very PR**, giving their `@v5` actions the live run they missed. The fix proves itself.

---

## Item 1 — Add the self-path to both trigger blocks (4 inserted lines)

Line numbers grep-verified at `beab477` (R0-r1 I1 — the first draft mis-cited them).
`.github/workflows/manual.yml` — add `- '.github/workflows/manual.yml'` to:
- `push.paths` (after `:13` `docs/tools/render-mermaid-cache.py`)
- `pull_request.paths` (after `:19` `docs/tools/render-mermaid-cache.py`)

`.github/workflows/quickstart.yml` — add `- '.github/workflows/quickstart.yml'` to:
- `push.paths` (after `:13` `docs/tools/render-mermaid-cache.py`)
- `pull_request.paths` (after `:21` `docs/tools/render-mermaid-cache.py`)

Mirrors the exact pattern in `technical-manual.yml` (push `:23`, PR `:32`), `rust.yml` (push `:20`, PR `:26`), `manual-gui.yml` (push `:14`, PR `:20`) — each: self-path in both push.paths + pull_request.paths. No other change to either workflow.

**Cost — accepted tradeoff (recon flag):** `manual.yml` + `quickstart.yml` are HEAVY (pandoc + full `texlive-*` + `cargo install` sibling CLIs + `cargo build` + PDF build / `make audit`), unlike the lint-only `technical-manual`. After this change, a workflow-file edit fires a multi-minute heavy build. Workflow-file edits are RARE, and catching a broken build/PDF/release workflow *at edit time* (rather than on the next docs change or release tag) is the point — so the cost is accepted. (Tag-event + docs-path triggers are unchanged; this only ADDS the workflow-self path.)

**Safety:** adding a `paths` entry cannot break an existing run — it only widens *when* the workflow fires. No job/step/checkout-nesting/`working-directory` change → no effect on any gate's WS-derivation. actionlint-validated.

---

## Item 2 — Update the catch-up FOLLOWUP (drop the absorbed sub-item)

In `ci-actions-catch-up-to-latest-majors`, remove the "optionally add self-trigger `.github` paths to `manual`/`quickstart`" clause (now resolved by this cycle) and note it was extracted + shipped here. Keep the v6/v7/v8 vetting + download-artifact-v8 rehearsal + SHA-pin eval + the v5-stability precondition (still unmet — Cycle B stays deferred).

---

## 3. Verification

1. **actionlint** on `manual.yml` + `quickstart.yml` (and a full `.github/workflows/*.yml` pass).
2. **Grep verification (POST-implementation):** after the edit, grep each of `manual.yml`/`quickstart.yml` and assert exactly 2 self-path refs (push + PR), matching the technical-manual/rust/manual-gui pattern (which currently shows 2 each). (Pre-edit baseline: 0 self-path refs in each — the gap being closed.)
3. **PR (the live gate):** open the PR; confirm `manual` AND `quickstart` now **self-fire** on it (they didn't before) and go **green** with their `@v5` actions exercised (checkout@v5 + setup-node@v5 + upload-artifact@v5 on real runners) — this both proves the self-trigger works AND closes the v5-validation gap. (rust/manual-gui/technical-manual/sibling-pin-check also self-fire as before — harmless re-runs.)
4. Squash-merge when green; post-merge master push re-fires + confirms; `git revert`-ready.

---

## 4. Ship plan
1. Branch `ci/self-trigger-manual-quickstart`. Apply Item 1 + Item 2. Add this SPEC + R0 review(s).
2. Verify §3.1–3.2 locally (actionlint + grep).
3. Push; open PR; watch §3.3 — **confirm manual + quickstart self-fire and pass** (the load-bearing proof).
4. Squash-merge when green; verify master runs.
5. Memory + MEMORY.md index.

### Out of scope
- The v6/v7/v8 catch-up (deferred Cycle B; precondition unmet).
- SHA-pinning (parked in Cycle B).
- Any change to manual/quickstart jobs/steps; any other workflow.
