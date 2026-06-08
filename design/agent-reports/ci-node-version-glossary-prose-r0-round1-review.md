# R0 Architect Review — SPEC_ci_node_version_and_glossary_prose.md — Round 1

> Reviewer had Read/Glob/Grep/WebFetch; parent persists. Grounded at `HEAD == origin/master == d7ca67a`.

**Verdict: 0 Critical / 1 Important / 2 Minor — NOT GREEN as written.** One cheap fold (correct the §5.4 self-trigger honesty note) plus stating the Node-22 support basis flips this to GREEN. The substance of all four items is sound; the single blocker is a factual error in the SPEC's own verification-honesty note.

## Critical
None.

## Important

### I1 — §5.4 self-trigger honesty note is factually WRONG for `manual-gui.yml` (it DOES self-fire)
**Where:** SPEC lines 97-100: *"The other 3 workflows (`manual`, `manual-gui`, `quickstart`) do NOT trigger on their own `.github` file (their `paths` are docs-only)…"*

**Source contradicts this for manual-gui.** `manual-gui.yml` lists its own workflow file in BOTH trigger path-filters:
- push `paths`: `.github/workflows/manual-gui.yml` (`manual-gui.yml:14`)
- PR `paths`: `.github/workflows/manual-gui.yml` (`manual-gui.yml:20`)

So the commit that bumps `manual-gui.yml:85` to `node-version: '22'` WILL fire the `manual-gui` workflow on push to master, run `setup-node@v4` at `'22'`, and exercise `npm install -g markdownlint-cli2@^0.13 cspell@^8` (lines 87-91). Node-22 is therefore **push-validated for manual-gui too** — not just technical-manual.

The other two are classified correctly:
- `manual.yml` paths (`:11-19`) = `docs/manual/**` + `docs/tools/render-mermaid-cache.py` only → editing `manual.yml` does NOT self-fire. ✓
- `quickstart.yml` paths (`:11-21`) = `docs/quickstart/**` + a couple `docs/manual/**` lint-config paths + render-mermaid → editing `quickstart.yml` does NOT self-fire. ✓
- `technical-manual.yml` paths include `.github/workflows/technical-manual.yml` (`:23`, `:32`) AND `docs/technical-manual/**` (`:21`, `:30`) → self-fires (and the glossary edit fires it too). ✓

**Why Important, not Minor:** the error direction is conservative (the SPEC under-claims validation coverage; the edits are harmless either way). But this exact check was explicitly elevated by the gate, the false claim is destined for the durable ship report + MEMORY.md, and accurate-record discipline is the recurring failure class for this project. R0 blessing a SPEC with a wrong CI-behavior claim defeats the gate.

**Fix:** reword §5.4 to add manual-gui to the push-validated set; keep manual + quickstart as next-natural-run.

## Minor

### M1 — Node-22 support basis asserted but unverified in SPEC (verified here)
SPEC line 54 claims *"both support Node ≥18"*. Verified against the registry:
- `cspell@^8` resolves to the highest 8.x = **8.19.4**, `engines.node ">=18"`. (cspell `latest` is 10.0.1 with `engines.node ">=22.18.0"`, and `main` requires 22.18+ — but `^8` never resolves to 10.x, so irrelevant; the `^8` pin is the load-bearing protection.)
- `markdownlint-cli2` `latest` declares `engines.node ">=20"`; `@^0.13` is older and at least as permissive.
Both already run on Node 20, so 20→22 only widens compatibility. **Item 2 is safe.** SPEC should state this basis (the gate asked for it) rather than the vaguer "≥18."

### M2 — Item 3 tracker scope omits `download-artifact@v4`
SPEC line 73 scopes the deferred tracker to *"`checkout`/`setup-node`/`upload-artifact`."* `manual-gui.yml` also uses `actions/download-artifact@v4` (`:192`, `:198`) in its release job. The `@v4→@v5` migration must include `download-artifact` too. Add it to the tracker's scope enumeration. (Also surface the 2026-06-16 deadline as near-term — 9 days out — so the tracker reads as time-urgent.)

## Verified-correct (confirmed against source)

**Item 1 — fully confirmed:**
- Source default IS `0`, not `now`: `export_wallet.rs:212` `#[arg(long, default_value = "0", value_parser = parse_timestamp)]`; doc-comment `:210-211`. Glossary `61-glossary.md:385` "default `now`" → genuinely stale.
- **TimestampArg-vs-TimestampArgValue (the CRITICAL CHECK):** `TimestampArg` IS the `pub(crate) enum` (`Now`/`Unix(i64)`) at `wallet_export/mod.rs:144` (cited file); its `to_json` (`:150-154`) renders `Now`→`"now"`, `Unix(n)`→integer — glossary behavioral description still holds. `TimestampArgValue` is a DIFFERENT type (`pub struct TimestampArgValue(pub TimestampArg)`) in a DIFFERENT file `cmd/export_wallet.rs:297` (the clap field newtype). **The SPEC is RIGHT to keep the `wallet_export/mod.rs::TimestampArg` citation and NOT rename it.** The FOLLOWUP body (`FOLLOWUPS.md:2327`) is the self-mis-cite; renaming would point at the wrong file + mislabel a struct as the enum + could self-RED G2. Reworded prose ("default `0` (rescan from genesis); `now`+unix seconds also accepted") is accurate per `parse_timestamp` (`:311-323`).
- Pure prose-claim change, not gated by symbol-ref-check — correct.

**Item 2 — confirmed:** all 4 `node-version: '20'` lines exist where stated (`manual.yml:54`, `manual-gui.yml:85`, `quickstart.yml:55`, `technical-manual.yml:58`); all 4 use Node ONLY for `npm install -g markdownlint-cli2@^0.13 cspell@^8` (no mermaid-cli/chromium; lychee is a downloaded binary). Item-2-vs-Item-3 distinction is correct and load-bearing: `node-version` governs the workflow run-steps' Node; the action-runtime is `runs.using` inside each action, advanced only by `@v4→@v5`.

**Item 3:** exactly 7 workflows exist (`rust`, `manual`, `manual-gui`, `quickstart`, `technical-manual`, `install-pin-check`, `sibling-pin-check`) — matches; all use `@v4` actions; separating runtime-deprecation from node-version-input is correct.

**Item 4:** slug-1 codec-G2 wontfix = no action — correct.

**Disposition:** docs+CI only, no bump/tag — correct. No locksteps (no clap-flag/CLI/codec surface) — confirmed.

## Required folds before GREEN
1. (I1) Correct §5.4: add `manual-gui` to the push-validated set; keep `manual` + `quickstart` as next-natural-run.
2. (M1) State the Node-22 basis explicitly (cspell@^8→8.19.4 `node>=18`; markdownlint-cli2 `node>=20`; both already on Node 20).
3. (M2) Add `download-artifact@v4` to the Item-3 tracker scope; flag the 2026-06-16 deadline as near-term.

Sources: cspell 8.19.4 manifest `engines.node ">=18"`; cspell latest 10.0.1 `">=22.18.0"` (unreachable via ^8); markdownlint-cli2 latest `">=20"`.
