# SPEC — CI node-version 20→22 + glossary TimestampArg prose fix (+ defer-tracker for the action-major bump)

**Cycle:** CI/docs housekeeping (small).
**Date:** 2026-06-07.
**Source SHA:** `origin/master` == local `HEAD` == `d7ca67a`.
**Disposition:** docs + CI only — **no version bump, no tag** (binary byte-identical).
**Recon:** `cycle-prep-recon-technical-manual-codec-g2-wontfix-and-timestamparg-prose.md` (slug-2 staleness CONFIRMED; the FOLLOWUP's own `TimestampArgValue` claim is STRUCTURALLY-WRONG — corrected below).
**Resolves:** `technical-manual-glossary-timestamparg-default-prose-stale`. **Files:** `ci-actions-node20-runtime-major-bump-deferred`.
**User decision (this session):** defer the action-major `@v4→@v5` runtime migration; do `node-version 20→22` now + the glossary prose fix.
**Locksteps:** none (no clap-flag / CLI / codec surface).

---

## Item 1 — Glossary `TimestampArg` prose fix (resolves the slug-2 FOLLOWUP)

`docs/technical-manual/src/60-back-matter/61-glossary.md:385`. The entry says the
`--timestamp` flag default is `now`; the actual default is `0` (genesis rescan)
since v0.47.3 (`crates/mnemonic-toolkit/src/cmd/export_wallet.rs:212`
`#[arg(long, default_value = "0", …)]`; doc-comment `:210-211` "`0` (default;
rescan from genesis…), `now`, or unix seconds").

**Change (narrow):** in the parenthetical "…`ExportWalletArgs::timestamp`, default
`now`)", replace **`default now` → `default 0` (rescan from genesis; `now` and
unix seconds also accepted)**.

**DO NOT** rename the `TimestampArg` citation. The recon found the FOLLOWUP's
body claim ("the enum is `TimestampArgValue`, not `TimestampArg`") is
**STRUCTURALLY-WRONG**:
- `TimestampArg` IS still the `pub(crate) enum` (`Now`/`Unix(i64)`) at
  `crates/mnemonic-toolkit/src/wallet_export/mod.rs:144` — the glossary
  documents it correctly (both render-behaviors still hold via
  `TimestampArg::to_json`). The qualified citation
  `crates/mnemonic-toolkit/src/wallet_export/mod.rs::TimestampArg` is ACCURATE
  and stays (renaming it would point at the wrong file — `TimestampArgValue`
  lives in `cmd/export_wallet.rs:297`, not `wallet_export/mod.rs` — and mislabel
  a newtype struct as the enum, and would self-RED G2).
- `TimestampArgValue` is the clap **field** newtype `pub struct
  TimestampArgValue(pub TimestampArg)` (`cmd/export_wallet.rs:297`). Mentioning
  it is optional/additive (not a correction); this cycle keeps the fix minimal —
  the entry is *about the enum* — and changes only the stale default phrase.

This is a **pure prose-claim** fix → NOT gated by `symbol-ref-check` (it pins
locations, not claims). Hand-verified.

---

## Item 2 — `node-version: '20' → '22'` (4 workflows)

Node 20 LTS reached EOL in April 2026. Bump the `setup-node` input in:
`.github/workflows/manual.yml:54`, `manual-gui.yml:85`, `quickstart.yml:55`,
`technical-manual.yml:58`.

Safe + uniform: all 4 use Node identically — ONLY `npm install -g
markdownlint-cli2@^0.13 cspell@^8`; `lychee` is a downloaded binary, not Node.
No other Node consumer (mermaid-cli/chromium are intentionally absent — caches
are checked in). **Node-22 support basis (R0-r1 M1, npm-registry-verified):**
`cspell@^8` resolves to the highest 8.x = `8.19.4`, `engines.node ">=18"`;
`markdownlint-cli2@^0.13` ≤ `node>=20`. Both already run on the current Node 20,
so 20→22 only widens compatibility. (cspell's `node>=22.18` floor is on the 10.x
major / `main` only — unreachable via `^8`; the `^8` pin is the protection.)

**This is the `node-version` INPUT (the Node the workflow uses for the lint
tools), NOT the action runtime.** The action-runtime deprecation (`@v4` actions
on Node 20) is a SEPARATE concern, deferred (Item 3).

---

## Item 3 — File the deferred action-major-bump tracker

The CI annotation (`actions/checkout@v4`, `setup-node@v4`, `upload-artifact@v4`
run on Node 20; **forced to Node 24 on 2026-06-16, Node 20 removed 2026-09-16**)
is the *action-runtime* deprecation — distinct from Item 2. Per the user's
decision it is DEFERRED. File `ci-actions-node20-runtime-major-bump-deferred`
(open) recording: scope = `@v4→@v5` for checkout / setup-node / upload-artifact /
**download-artifact** (R0-r1 M2 — `manual-gui.yml:192,:198` uses
`download-artifact@v4` in its release job) across **all 7 workflows** (`rust`,
`manual`, `manual-gui`, `quickstart`, `technical-manual`, `install-pin-check`,
`sibling-pin-check`); the verification cost (each workflow re-run green;
`upload-artifact`/`download-artifact` v4→v5 + `setup-node` v5 behavior changes
need a real read, not a blind sed); **NEAR-TERM — forced to Node 24 on
2026-06-16 (9 days out), Node 20 removed 2026-09-16; revisit before 2026-06-16.**

---

## Item 4 — slug-1 (`technical-manual-codec-g2-not-enforceable-in-single-repo-ci`): NO ACTION

Recon verdict: accepted-wontfix tracker, citations accurate, deliberately `open`
as a discoverability marker. Nothing to implement this cycle.

---

## 5. Verification

1. **Item 1 hand-check:** re-read `61-glossary.md:385` — only the default phrase
   changed; the `TimestampArg` citation untouched.
2. **Local `make lint` GREEN (siblings present):** the glossary edit is prose
   inside an existing entry; symbol-ref-check unaffected (no citation change),
   markdownlint/cspell pass. Expect 725/0, all 7 steps green.
3. **actionlint** on all 4 edited workflows.
4. **CI run after push (honest scope, R0-r1 I1):** this push edits all 4 `.yml`
   files (node-version) + the glossary, but only 2 of the 4 self-trigger. It
   touches `docs/technical-manual/**` (glossary) + `.github/workflows/technical-manual.yml`
   AND `.github/workflows/manual-gui.yml`. **Both `technical-manual`
   AND `manual-gui` self-fire** — technical-manual via its own `.github` path
   (`:23,:32`) + the glossary `docs/technical-manual/**` path; manual-gui via its
   own `.github` path (`manual-gui.yml:14,:20`). So **node-version 22 is
   push-validated for BOTH.** `manual.yml` and `quickstart.yml` do NOT trigger on
   their own workflow-file edits (docs-only `paths`), so their byte-identical bump
   validates on their next natural run — note this in the ship report + memory (do
   not claim all 4 are observed-green; claim 2 of 4). Risk is negligible
   (identical change; tools support Node 22 — see Item 2 basis).

---

## 6. Ship plan

1. Apply Items 1-2.
2. `design/FOLLOWUPS.md`: flip `technical-manual-glossary-timestamparg-default-prose-stale`
   → resolved (with the recon's correction recorded: only the default was stale;
   `TimestampArg` citation was already accurate). File
   `ci-actions-node20-runtime-major-bump-deferred` (Item 3).
3. Verify §5.
4. Stage paths explicitly (no `git add -A`). Commit (`git commit -F -`,
   Co-Authored-By trailer). Push to `master`. **No bump, no tag.** Watch the
   technical-manual CI run.
5. Memory + MEMORY.md index.

### Out of scope
- The `@v4→@v5` action-runtime migration (Item 3 tracker; deferred by decision).
- Any `TimestampArgValue` additive prose / claim re-validation beyond the default.
- slug-1 codec-G2 (Item 4; accepted wontfix).
