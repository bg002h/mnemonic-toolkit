# R0 Architect Gate — Round 2 (fold-confirmation) — SPEC_ci_node_version_and_glossary_prose.md

> Round 1 = 0C/1I/2M; all folded. Reviewer had Read/Glob/Grep/WebFetch; parent persists.

**Verdict: GREEN (0C / 0I / 1m).** All three Round-1 folds (I1, M1, M2) landed correctly and are primary-source accurate. No new Critical or Important findings; one optional Minor. Implementation may proceed.

## Critical
None.
## Important
None.

## Minor (new, non-blocking)
- **m1 — §5.4 enumerates only the trigger-relevant touched files.** The sentence names the two self-firing workflows (`technical-manual`, `manual-gui`) but not `manual.yml`/`quickstart.yml`, which are also edited. They are correctly handled in the next sentence ("validates on next natural run"), so the section is internally consistent on a full read. Optional: add a half-clause acknowledging all 4 `.yml` are edited but only 2 self-trigger. Pure wording.
- **(advisory)** M1 asserts `cspell@^8` resolves to highest 8.x = `8.19.4`; confirmed `8.19.4 → engines.node ">=18"` directly but not that 8.19.4 is the absolute highest 8.x. Not gate-relevant: no `^8` resolution demands node `>22`; the `>=22.18` floor is 10.x-only (unreachable via `^8`).

## Fold confirmation
- **I1 — CLEAN.** `manual-gui.yml` lists its own file at push `:14` / PR `:20`; `technical-manual.yml` at push `:23` / PR `:32` (+ `docs/technical-manual/**` `:21`). Both self-fire → node-22 push-validated for both. `manual.yml` (paths = `docs/manual/**`, render-mermaid) and `quickstart.yml` (paths = `docs/quickstart/**`, render-mermaid) do NOT list their own files → no self-trigger. "2 of 4 observed-green" is exactly correct.
- **M1 — CLEAN, not overstated.** `cspell@8.19.4 → engines.node ">=18"`; `markdownlint-cli2@^0.13` → `0.13.x`, `0.13.0 → ">=18"` (SPEC's "≤ node>=20" is a conservative upper bound). Node 22 satisfies both. cspell `>=22.18` floor is 10.x-only, unreachable via `^8`. Only these two Node tools are consumed.
- **M2 — CLEAN.** `download-artifact@v4` confirmed at `manual-gui.yml:192,:198` (release job). Tracker scope spans checkout/setup-node/upload-artifact/download-artifact across all 7 workflows. Deadline math correct (2026-06-07 → 06-16 = 9 days).

## Round-1 substance re-confirmation (intact)
- Item 1: `61-glossary.md:385` stale `default now` (SPEC is the plan); `wallet_export/mod.rs::TimestampArg` citation accurate (`pub(crate) enum :144`; `Now`→`"now"`/`Unix(i64)`→int `:152-153`); "DO NOT rename" correct.
- Four node-version lines exact: `manual.yml:54`, `manual-gui.yml:85`, `quickstart.yml:55`, `technical-manual.yml:58`.
- Items 3/4 + disposition (docs+CI, no-bump/no-tag, no locksteps) untouched + consistent.

**No further reviewer round required — GREEN.**
