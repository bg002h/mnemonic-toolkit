# cycle-prep recon — 2026-05-24 — full documentation audit

**Origin/master SHA at recon time:** `b2806d6`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** `.claude/`

Free-form audit (not a FOLLOWUP slug): the end-user/front-door documentation surface vs the live v0.36.2 CLI. **The end-user MANUAL (`docs/manual/`) is in good shape** (all 20 mnemonic subcommands + all sibling CLIs have chapters; lint 6/6 GREEN; the v0.28.1 round-trip-recipe breakage was fixed). **The README is severely stale, and 3 manual-lint/navigation gaps exist.** One systemic gap (no prose-command execution gate) → FOLLOWUP.

---

## Gaps found (by tier)

### TIER 1 — definite, high-value, autonomously fixable
- **G1 — `README.md` stale at v0.8.0 (repo is v0.36.2 — 28 minor versions behind). BIGGEST gap.**
  - `README.md:13` `Status: **v0.8.0 shipped** (2026-05-07 …)` — STRUCTURALLY-WRONG (28 versions stale). Feature narrative stops at v0.8 (omits v0.9→v0.36: import-wallet, xpub-search, compare-cost, repair, inspect, seedqr, slip39, seed-xor, final-word, nostr, silent-payment, decode-address, verify-message, electrum-decrypt, BSMS/BIP-129, cross-format conversion, BIP-352, BIP-322, …).
  - `README.md:35` install example pins `mnemonic-toolkit-v0.13.0` — STALE (current tag v0.36.2; the canonical `scripts/install.sh:32` IS current, but the README's hand-written example lags).
  - `README.md:40-44` subcommand bullets enumerate only **5 of 20** (bundle, verify-bundle, convert, export-wallet, derive-child) — STRUCTURALLY-INCOMPLETE (15 missing).
  - **Action:** refresh the status line (→ v0.36.2), the feature narrative (summarize v0.9→v0.36), the subcommand bullet list (5 → 20), and the install-example tag (→ v0.36.2). Cite SHA `b2806d6`.

- **G2 — `docs/manual/tests/cli-subcommands.list` omits `electrum-decrypt` + `seedqr` → flag-coverage lint is BLIND to those 2 chapters.**
  - Live `mnemonic --help` has `electrum-decrypt` + `seedqr` (encode/decode); `cli-subcommands.list` lacks them (verified via `comm`). The flag-coverage gate therefore never validates those chapters' flags vs `--help`. **DRY-RUN VERIFIED:** adding `mnemonic electrum-decrypt` + `mnemonic seedqr encode` + `mnemonic seedqr decode` → lint flag-coverage still GREEN (0 errors) ⇒ both chapters ARE flag-complete; this is a clean GATE-EXTENSION (no drift to fix, just wire the gate). (seedqr is `encode`/`decode` sub-subcommands → 2 entries, mirroring `seed-xor split/combine`.)
  - **Action:** add the 3 lines to `cli-subcommands.list`. Cite SHA `b2806d6`.

- **G3 — manual intro (`41-mnemonic.md:3`) "Fourteen subcommands" omits 6 LIVE subcommands.** The intro link-list enumerates 14 (bundle…verify-message + gui-schema) but OMITS `electrum-decrypt`, `seedqr`, `repair`, `inspect`, `compare-cost`, `xpub-search` — all live, all with chapters. Misleading completeness claim + lost navigation. **Action:** list all 20 (or reword) + bump the count. Cite SHA `b2806d6`.

- **G4 — stale version stamps.**
  - `41-mnemonic.md:14` "this chapter mirrors v0.13.0" — STALE (chapter documents v0.36.2 surface). **Action:** → current, or make version-agnostic.
  - `60-appendices/68-release-history.md:66` "as of v0.1's tag." — STALE. **Action:** refresh or generalize.

### TIER 2 — systemic / architectural → FOLLOWUP (own cycle + R0)
- **G5 — no prose-command EXECUTION gate.** `docs/manual/tests/lint.sh` validates flag NAMES (stage 4), spelling, links, glossary, index — but NEVER executes documented commands. The v0.28.1 round-trip breakage (all 6 chapter-45 recipes failed at the `export-wallet` step; `design/AUDIT_FINDINGS_manual_v0_28_0_content.md`) shipped silently for exactly this reason and was fixed reactively. The systemic fix is a lint stage / integration test that EXECUTES the documented round-trip recipes (or a curated subset) against the pinned binary. Meatier (a recipe-extraction + run harness); deserves its own cycle + R0. **File FOLLOWUP `manual-prose-command-execution-gate`.** (`feedback_architect_must_run_prose_commands` already records the discipline; this would automate it.)

---

## Verified NON-gaps (audit-confirmed clean)
- All 20 `mnemonic` subcommands have a `## ` chapter in `41-mnemonic.md` (slip39 was a false-flag of a digit-excluding grep — chapter @:1211).
- Sibling CLIs fully documented: md (9 chapters; gui-schema intentionally omitted), ms (6), mk (6). gui-schema omitted across all per the "introspection only" convention.
- electrum-decrypt + seedqr chapters are flag-complete (G2 dry-run).
- Chapter-45 round-trip recipes now carry `--template`/`--wallet-name` (the v0.28.1 breakage is FIXED): sparrow `--template bip84`, specter `--wallet-name`, coldcard `--template bip84`, coldcard-multisig `--template wsh-sortedmulti --threshold 2`.
- Manual lint 6/6 GREEN at `b2806d6`.

---

## Cross-cutting observations
1. **The MANUAL is healthy; the README is the rot.** The manual has a 6-stage lint + lockstep invariants that kept it current through 28 versions; the README has NO gate and silently decayed to v0.8.0. The asymmetry IS the finding — the README needs the refresh AND (long-term) a freshness check.
2. **G2 is a latent recurrence of the exact `cli-subcommands.list` omission** flagged at v0.35.0 (silent-payment) + v0.36.0 (the reviewer noted electrum-decrypt/seedqr as pre-existing). Closing it now also de-risks future chapter drift for those 2.
3. **No DRIFTED-by-N positional drift** — the gaps are content-staleness (README) + gate-coverage (G2) + completeness (G3) + stamps (G4), not moved line numbers.
4. `docs/manual/FOLLOWUPS.md` is its own registry — open items are minor v0.2 candidates (release-history-auto-extract, npm-package-pinning, bch-string-length-empirical-sweep); not in scope for this audit.

---

## Recommended brainstorm-session scope
- **ONE documentation-refresh cycle (docs + lint-input only; NO toolkit code change).** Fix G1 (README refresh) + G2 (cli-subcommands.list += electrum-decrypt/seedqr) + G3 (intro lists all 20) + G4 (2 version stamps). File FOLLOWUP for G5.
- **SemVer:** docs/test-only. **R0 to confirm tagging model** — either a toolkit PATCH `v0.36.3` (consistent with the v0.28.5 docs-PATCH precedent; requires install.sh/Cargo.toml/lock bump in lockstep) OR a manual-namespace tag with no crate bump. Lean toward the toolkit PATCH for a single coherent tag, since README + cli-subcommands.list are toolkit-repo artifacts.
- **Locksteps:** NONE for GUI (no clap surface change). Manual workflow fires on the `docs/manual/**` change (validates the 6 stages incl. the newly-wired electrum-decrypt/seedqr flag-coverage). install-pin-check fires IFF a toolkit tag is cut.
- **Sizing:** small-medium. G1 (README) is the bulk — a careful rewrite of ~3 sections (status, feature narrative, subcommand bullets) + 1 install-pin line. G2/G3/G4 are a few lines each. ~1 doc file heavily + 3 lightly + 1 test-input.
- **R0 must resolve:** (a) tagging model (PATCH v0.36.3 vs manual-namespace); (b) README rewrite scope — full per-subcommand bullets (20) vs a condensed "see the manual" pointer + the headline subcommands (avoid re-creating a second drift-prone surface); (c) whether to also add a README freshness guard (a test asserting the README status version == Cargo.toml version) to prevent re-decay — arguably the highest-leverage fix, but verify feasibility; (d) confirm G2's seedqr entries are `seedqr encode`/`seedqr decode` (not bare `seedqr`).
