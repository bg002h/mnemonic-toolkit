# v0.36.2 Implementation Plan — rebuild the argv-leakage audit as a clap-derived closure

> **For agentic workers:** per-phase TDD; tests before impl; per-phase opus reviewer-loop until 0C/0I; persist reviews to `design/agent-reports/` before the fold. NO parallel code-gen. Steps use `- [ ]`.

**Goal:** Replace the hand-frozen `CANONICAL_FLAG_ROWS` (28 rows, frozen at v0.13.0, silently omits ~16 post-v0.13.0 secret-argv routes) with a **closure** that enumerates every secret-bearing argv route from the LIVE surface and fails when a route is added (or removed) without acknowledgment — turning a lagging hand-checklist into a leading gate. Test/lint-only.

**Architecture:** A test enumerates secret-argv routes across **three axes** from authoritative live inputs, then (a) asserts the enumerated set set-equals a small declared set (completeness + removal-detection), and (b) asserts each route's implementing source carries a stdin-alternative evidence anchor (proves the `*-stdin` / `=-` route is wired, not just named). No user-facing change.

**Tech Stack:** Rust integration test; `mnemonic gui-schema` subprocess (per-subcommand flag list + `secret` bits); lib-public `mnemonic_toolkit::secret_taxonomy::{SECRET_NODE_TYPES_ARGV, SECRET_SLOT_SUBKEYS}`.

**Source SHA:** citations grep-verified vs `origin/master` @ `5fbed42` (2026-05-24). Recon: `cycle-prep-recon-lint-argv-secret-flags-canonical-table-rebuild.md`.

---

## SemVer + lockstep
- **PATCH → v0.36.2.** Test/lint-only — NO CLI surface change ⇒ **NO GUI schema_mirror lockstep, NO manual lockstep, NO sibling-codec companions.**
- Touches `crates/mnemonic-toolkit/tests/lint_argv_secret_flags.rs` only (+ a possible tiny doc note if a `secret_taxonomy` const needs a comment). Version bump + Cargo.lock + install.sh self-pin + CHANGELOG + close FOLLOWUP.

## The three secret-argv axes (recon-confirmed)
1. **Flag-NAME axis** — value-bearing flags whose NAME is intrinsically secret: from `gui-schema`, every `(subcommand, flag)` with `secret == true && kind != "boolean"`. (The `secret && boolean` flags are the `*-stdin` toggles — they are EVIDENCE, not routes.) Live count: **25** such flags (recon).
2. **Node-value axis** — `--from <node>=` where the node is secret: `SECRET_NODE_TYPES_ARGV` (`secret_taxonomy.rs:95`) × {subcommands declaring `--from`}. (gui-schema does NOT mark `--from` secret — its name is generic — so this axis CANNOT come from the flag bit; it MUST come from `secret_taxonomy`.)
3. **Slot-subkey axis** — `--slot @N.<subkey>=` where the subkey is secret: `SECRET_SLOT_SUBKEYS` (`secret_taxonomy.rs:111` = `["phrase","seedqr","entropy","xprv","wif"]`) × {subcommands with `--slot` / `allows_slots`}.

The OLD `CANONICAL_FLAG_ROWS` hand-mixed axes 1+2+3 into 28 prose rows and froze. The rebuild derives axes 1 (from gui-schema) + 2,3 (from `secret_taxonomy`) so new entries on ANY axis can't be silently omitted.

## Per-route stdin-coverage (recon spot-check — R0 to complete exhaustively)
Every spot-checked secret-argv flag HAS a stdin route: `--secret`→`--secret-stdin`, `--passphrase`→`--passphrase-stdin`, `--decrypt-password`→`--decrypt-password-stdin`, `--ms1`→`--ms1-stdin` (xpub-search seed_intake.rs:30,104) or `--ms1 -` (inspect.rs:33), `--digits`→`--digits -` (seedqr.rs:56), `--share`→`--share -`, `--from <node>=`→`=-`, `--slot @N.x=`→`slot-stdin`. **If the closure surfaces ANY secret-argv route with no wired stdin alternative, that is a REAL leak → escalate (add the `*-stdin` flag; that sub-fix WOULD need GUI/manual lockstep).** Recon found none, but the closure makes this exhaustive.

---

## Phase 1 — flag-NAME axis closure + backfill

**Files:** `crates/mnemonic-toolkit/tests/lint_argv_secret_flags.rs` (rebuild)

- [ ] **Step 1 — RED:** add a test `secret_flag_routes_match_live_gui_schema` that runs `mnemonic gui-schema` (subprocess, per `cli_gui_schema.rs` pattern), collects every `(subcommand, flag.name)` with `secret==true && kind!="boolean"`, and asserts set-equality with a declared `SECRET_ARGV_FLAG_ROUTES: &[(&str,&str)]`. Seed `SECRET_ARGV_FLAG_ROUTES` with ONLY the current 6 flag-NAME routes the old table happened to cover (bundle/verify-bundle/convert/derive-child/slip39 `--passphrase` + `--bip38-passphrase` + `--share`) so the test goes RED against the ~16 unlisted (nostr `--secret`, silent-payment `--secret`/`--passphrase`, electrum-decrypt/import-wallet `--decrypt-password`, import-wallet/inspect/repair/verify-bundle/xpub-search×3 `--ms1`, xpub-search×3 `--passphrase`, seedqr-decode `--digits`).
- [ ] **Step 2** — run; expect FAIL listing the ~16 missing routes (proves the closure catches the decay).
- [ ] **Step 3** — backfill all missing flag-NAME routes into `SECRET_ARGV_FLAG_ROUTES` until set-equality holds (the enumeration IS the spec — copy the failing-set the test prints, verified against the gui-schema dump).
- [ ] **Step 4 — evidence per flag route:** for each `(subcommand, --X)`, assert the implementing source (`src/cmd/<subcommand>.rs` or its module dir) contains a stdin-alternative anchor: `"--X-stdin"` OR `"X_stdin"` OR (`secret_in_argv_warning` naming `"--X"`) OR (`"--X -"` / `value == "-"` carve-out). Encode this as a per-route `evidence: &[&str]` (keyed by route, like today) — BUT the route SET is now closure-checked (Step 1), so a new flag can't be added without BOTH a route entry AND its evidence. **If any backfilled route has NO wireable evidence → that is a real leak (escalate per the severity watch).**
- [ ] **Step 5** — run both tests GREEN; commit.

## Phase 2 — node-value + slot-subkey axes closure

**Files:** same test file.

- [ ] **Step 1 — RED:** add `secret_node_routes_match_taxonomy` — for each subcommand whose gui-schema declares a `--from`-style flag (NodeValueComposite), assert the declared `SECRET_FROM_NODE_ROUTES` set-equals `{(subcommand, node) : node ∈ SECRET_NODE_TYPES_ARGV}`; similarly `secret_slot_routes_match_taxonomy` for `--slot`/`allows_slots` × `SECRET_SLOT_SUBKEYS`. Use `mnemonic_toolkit::secret_taxonomy::{SECRET_NODE_TYPES_ARGV, SECRET_SLOT_SUBKEYS}` directly (lib-public).
- [ ] **Step 2** — run; FAIL until declared sets match the taxonomy×subcommand product.
- [ ] **Step 3** — populate the declared node/slot route sets to match; assert each has its `=-`/`slot-stdin` evidence anchor (the existing convert/bundle/derive-child/final-word/seed-xor/slip39 `--from`/`--slot` evidence patterns).
- [ ] **Step 4** — GREEN; commit.

## Phase 3 — retire stale scaffolding + release

- [ ] **Step 1** — delete the obsolete `canonical_list_has_twenty_eight_rows` hardcoded-count test (superseded by the closures) and the stale "20 flag-rows" prose (`:5`, `:44`); update the module doc to describe the closure model + removal-detection property (the closures fail on BOTH unlisted-new AND listed-but-removed routes — preserving the original `:20-24` removal-detection intent without the hand-frozen count).
- [ ] **Step 2 — release-prep:** `Cargo.toml` 0.36.1 → 0.36.2; `Cargo.lock` regen; `scripts/install.sh:32` self-pin `mnemonic-toolkit-v0.36.2`; `CHANGELOG.md` [0.36.2] PATCH entry (test-only).
- [ ] **Step 3 — FOLLOWUPS:** close `lint-argv-secret-flags-canonical-table-rebuild-from-clap`.
- [ ] **Step 4 — end-of-cycle opus review** → persist `design/agent-reports/v0_36_2-end-of-cycle-review.md` → fold → GREEN.
- [ ] **Step 5 — ship toolkit:** merge→master (ff), tag `mnemonic-toolkit-v0.36.2`, push, GH release; verify rust + manual + install-pin-check CI. **No GUI cycle** (test-only; no CLI surface change).

---

## Self-review / open R0 questions
- **Spec coverage:** flag axis (P1), node+slot axes (P2), retire+release (P3). ✓
- **Design decisions for R0:** (a) is the 3-axis closure the right model, or is a simpler "flag-axis set-equality + keep the hand node/slot rows" sufficient? (b) the evidence-anchor model under the closure — per-route `evidence` keyed by route identity (kept) vs a uniform naming-convention rule; (c) `gui-schema` is the flag-axis source (subprocess) — is parsing it in this test acceptable, or should the flag axis use a lib-public clap walk (the Cli is binary-private, so gui-schema subprocess is likely the only route — confirm); (d) does `gui-schema` mark `--from`/`--slot` in a way that lets the test detect "subcommand has a --from / has slots" (NodeValueComposite kind / a slots flag), or must that membership be declared; (e) the **severity escalation** path if any route lacks a wired stdin alternative (recon found none, but the closure must define what FAILS vs what merely WARNS); (f) removal-detection: set-equality (both directions) is the cleanest way to preserve the original intent — confirm.
- **Type consistency:** `SECRET_ARGV_FLAG_ROUTES: &[(&str,&str)]` + per-route evidence; node/slot declared sets derived from `secret_taxonomy` constants.
- **No placeholder evidence:** every backfilled route must cite a real source anchor proven to exist (Step 4 / P2 Step 3); a route with no anchor escalates, it is NOT given a fake anchor.
