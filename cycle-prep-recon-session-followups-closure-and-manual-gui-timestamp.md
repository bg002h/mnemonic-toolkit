# cycle-prep recon — 2026-06-06 — gui-timestamp-default-value-drift-v0.47.3 + cli-help-golden-broad-staleness-not-gated + manual-gui-export-wallet-timestamp-default-now-stale

**Origin/master SHA at recon time:** toolkit `6eb175a`, mnemonic-gui `c440e91`
**Local branch:** `master` (both repos)
**Sync state:** `up-to-date (0 ahead / 0 behind)` in both repos
**Untracked:** toolkit: `.claude/`, `CONTINUITY.md`, `cycle-prep-recon-*.md`, `feature-coverage-survey-*.md` (working artifacts) — no tracked-tree modifications.

Slug(s) verified: the 3 FOLLOWUPs filed across this session's ships. **Slugs 1 & 2 = RESOLVED (closure confirmed, no cycle). Slug 3 = OPEN and its citations have a STRUCTURAL ERROR** (understates scope: 3 stale sites, not 2; one cited-as-"wrong" line is actually a real stale site).

---

## Per-slug verification

### `gui-timestamp-default-value-drift-v0.47.3` — RESOLVED (closure confirmed)
- **WHAT:** GUI `export-wallet --timestamp` `default_value:"now"` silently suppressed an explicit `Now` after toolkit v0.47.3 flipped the default to `0`.
- **Citations (verified against `mnemonic-gui` `origin/master` `c440e91`):**
  - `mnemonic-gui/src/schema/mnemonic.rs` `--timestamp` `default_value` — **ACCURATE/resolved**: `git show origin/master:src/schema/mnemonic.rs` line 1044 = `default_value: Some("0")` + help reworded to "`0` (default; rescan from genesis), `now`, or unix seconds." ✓
  - inverted regression guard — **ACCURATE/resolved**: `d33_timestamp_now_is_emitted_when_default_is_zero` present at `tests/argv_assembler.rs:492`; the old `d33_timestamp_now_at_default_suppresses` is absent. ✓
- **Action:** NONE — genuinely resolved (mnemonic-gui v0.28.0, `c440e91`). No cycle.

### `cli-help-golden-broad-staleness-not-gated` — RESOLVED (closure confirmed)
- **WHAT:** 21 unrendered v0.8.0-era `cli-help/*.txt` `--help` snapshots drifted silently.
- **Citations (verified against toolkit `origin/master` `6eb175a`):**
  - `docs/manual/transcripts/cli-help/*.txt` — **ACCURATE/resolved**: `git show origin/master:docs/manual/transcripts/cli-help/mnemonic.txt` → `fatal: path … does not exist` (the whole dir is gone). ✓
- **Action:** NONE — genuinely resolved (cli-help-cleanup cycle, `a83dc75`). No cycle.

### `manual-gui-export-wallet-timestamp-default-now-stale` — OPEN — **citations STRUCTURALLY-WRONG (understated scope)**
- **WHAT:** toolkit repo's GUI user manual (`docs/manual-gui/`, separate pinned cadence) still documents `export-wallet --timestamp` default as `now`; toolkit v0.47.3 made it `0`.
- **Citations (verified against toolkit `origin/master` `6eb175a`):**
  - `docs/manual-gui/src/40-mnemonic/45-export-wallet.md:30` (flag-list summary "(default `now`)") — **ACCURATE**: `:30` = ``- [`--timestamp`](#…) — Bitcoin Core `timestamp` field (default `now`)``.
  - `…:340-343` (the `--timestamp` section prose) — **DRIFTED-by-2**: the prose body is at **`:342-346`** (header `## `--timestamp`` at `:340`); `:342-344` = "Two valid forms: `now` (the default; emits the literal string `"now"` in the JSON, which Core interprets at import time as the current block timestamp) or a non-negative integer Unix-seconds value." Content ACCURATE, line range off by ~2.
  - **`…:422` — STRUCTURALLY-WRONG in the FOLLOWUP body.** The filed FOLLOWUP says *"The cited `45-export-wallet.md:422` manual-gui ref was **wrong** (no such timestamp example at that line)."* This is FALSE. `:422` IS a real stale site — a worked `importdescriptors` JSON example output block (`:415-426`) containing `"timestamp": "now"` (would render `"timestamp": 0` after the v0.47.3 default flip). The error arose because the GUI v0.28.0 recon grepped the **mnemonic-gui** repo (which has no `docs/`) instead of the **toolkit** repo where `docs/manual-gui/` actually lives. → **THREE stale sites, not two.**
  - `docs/manual-gui/tests/expected_gui_schema_inventory.json` `--timestamp` entry has NO `default_value` field — **ACCURATE** (not stale, not gated).
- **Action for brainstorm spec:** Correct the FOLLOWUP (remove the false "`:422` was wrong" clause; record THREE stale sites: `:30`, `:342-344`, `:422`). Reword all three to lead with the `0` default (genesis rescan), mirroring the toolkit manual's v0.47.3 wording (`41-mnemonic.md:707` + `37-wallet-export.md` Timestamp bullet): `:30` summary → "(default `0`; rescan from genesis)"; `:342-344` prose → lead with `0` as the default, keep `now`/unix as alternatives; `:422` JSON example → `"timestamp": 0`. Cite source SHA `6eb175a`.

---

## Cross-cutting observations
1. **A recon-caught structural error in a same-session-filed FOLLOWUP.** `manual-gui-export-wallet-timestamp-default-now-stale` was filed during the GUI v0.28.0 cycle on the back of a grep run in the WRONG repo (GUI, no `docs/`), which produced the false "`:422` ref was wrong" claim and missed the JSON example. This is the canonical decayed-citation/structural-error class cycle-prep exists to catch — even for a FOLLOWUP filed hours ago.
2. **Repo-location trap:** `docs/manual-gui/` (the GUI user manual) lives in the **toolkit** repo, NOT in `mnemonic-gui`. Any grep verifying manual-gui claims must run in the toolkit repo. (Same trap bit both the v0.47.3 R0 M2 disposition and the GUI v0.28.0 recon.)
3. **Slugs 1 & 2 are genuinely resolved** — `origin/master` bytes confirm both (GUI schema `Some("0")` + inverted test; toolkit `cli-help/` dir absent). No loose ends; no re-open.
4. **manual-gui is gated** by `.github/workflows/manual-gui.yml` (fires on push/PR touching `docs/manual-gui/**`) running the 7-phase `make -C docs/manual-gui lint` (markdownlint/cspell/lychee/etc.) — the lint checks markup, NOT semantic-default accuracy, so the prose reword must keep `make -C docs/manual-gui lint` GREEN but the staleness itself was never lint-catchable.

---

## Recommended brainstorm-session scope
- **Slugs 1 & 2:** **NO cycle** — resolved this session; closure confirmed against `origin/master`. (Recon's only output for them is this confirmation.)
- **Slug 3 (`manual-gui-export-wallet-timestamp-default-now-stale`):** **ONE tiny docs-only cycle.** Reword 3 stale sites in `docs/manual-gui/src/40-mnemonic/45-export-wallet.md` (`:30`, `:342-344`, `:422`) to the `0` default + correct the FOLLOWUP's false `:422` claim. **Size: ~4 lines.** **SemVer:** docs-only — **NO toolkit crate version bump / NO tag** (anchor-dangler `dd7c228` + cli-help-cleanup `a83dc75` precedent; `docs/manual-gui/` rides its own `manual-gui-v*` release cadence, not the toolkit crate version). **Locksteps:** `make -C docs/manual-gui lint` GREEN + `manual-gui.yml` CI fires; NO GUI `schema_mirror`, NO sibling-codec, NO `docs/manual/` change. **No RED phase** (prose). Mandatory R0 still applies (small spec → likely 1 round). Resolve the FOLLOWUP on ship. No inter-slug dependency.
