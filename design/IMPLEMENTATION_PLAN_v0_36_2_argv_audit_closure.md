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

## The secret-argv route axes (recon + R0-corrected)

> **R0 I1+I2 fold:** axes 2+3 are NOT a `nodes × subcommands` cartesian — each `--from`/`--slot` subcommand accepts a DIFFERENT, per-subcommand node/subkey subset (convert: all 9; derive-child: phrase/xprv; seedqr-decode: seedqr; export-wallet: REFUSES secret subkeys; import-wallet: phrase only; …), and that acceptance is invisible to BOTH gui-schema (`--from`/`--slot` emit `kind="text"` — there is NO `NodeValueComposite` kind) AND `secret_taxonomy`. A cartesian over-generates non-existent routes. **The fix: collapse axes 2+3 to per-`(subcommand, --from|--slot)` routes, detected by gui-schema flag-NAME match.** The non-argv route (`=-` / slot-stdin) is UNIFORM per-subcommand — it covers every node/subkey that subcommand accepts — so per-node granularity adds nothing to the stdin-route invariant.

1. **Flag-NAME axis** — value-bearing flags whose NAME is intrinsically secret: from `gui-schema`, every `(subcommand, flag)` with `secret == true && kind != "boolean"`. (The `secret && boolean` flags are the `*-stdin` toggles — EVIDENCE, not routes.) Live count: **25** (recon + R0-confirmed); OLD table covered **9** → **16 missing**.
2. **`--from` axis** — subcommands whose gui-schema flags include a flag NAMED `--from` (name-match; gui-schema does NOT mark `--from` secret — its name is generic). Each such subcommand's source must wire the `=-`/`value == "-"` non-argv route (covers every node it accepts). Membership = the `--from`-bearing subcommand set (set-equality catches a NEW `--from` consumer; a widened node-acceptance within an existing one is safe — `=-` already covers all nodes).
3. **`--slot` axis** — subcommands whose gui-schema flags include `--slot`. Each must wire a non-argv channel: `slot-stdin` (bundle/verify-bundle) OR `@env:` (R1 I-A: **import-wallet `--slot @N.phrase=` is `@env:`-only** — `import_wallet.rs:1314-1317`; no slot-stdin) OR (export-wallet) REFUSE secret subkeys (the refusal IS the safety anchor). Membership = the `--slot`-bearing subcommand set = {bundle, verify-bundle, export-wallet, import-wallet}.

The OLD `CANONICAL_FLAG_ROWS` hand-mixed all axes into 28 frozen prose rows. The rebuild derives axis 1 from gui-schema (per-flag) + axes 2,3 from gui-schema `--from`/`--slot` membership (per-subcommand) so new entries on ANY axis can't be silently omitted — without a per-node decay surface.

## Per-route stdin-coverage (recon spot-check — R0 to complete exhaustively)
R0 EXHAUSTIVELY verified every secret-argv route has a non-argv channel:
- `*-stdin`/`-` routes: nostr/silent-payment `--secret`/`--passphrase`, electrum-decrypt/import-wallet `--decrypt-password` (→`*-stdin`); seedqr-decode `--digits`, inspect/repair `--ms1` (→`-`); xpub-search×3 `--ms1` (→`--ms1-stdin`)/`--passphrase` (→`--passphrase-stdin`); all `--from` (→`=-`); secret `--slot` (→slot-stdin).
- **`@env:`-only routes (R0 I3):** `import-wallet --ms1` + `verify-bundle --ms1` have NO `*-stdin`/`-` — their non-argv channel is the `@env:VAR` sentinel (`env_sentinel.rs:1-16` lists `--ms1`; `verify_bundle.rs:943-945`, `import_wallet.rs:1311-1312`). So the evidence-anchor set MUST recognize `@env:`-class anchors (`@env:`, `resolve_env_sentinels`, `needs_env_sentinel_resolution`) — else the closure falsely flags these two as leaks.

**No true unmitigated leak exists** (R0 confirmed). The closure makes this exhaustive going forward: if it EVER surfaces a secret-argv route with NO non-argv channel (`*-stdin`/`-`/`@env:`), that IS a real leak → escalate. (Separate pre-existing hygiene gap, out of scope: `import-wallet --ms1` fires NO argv advisory at all — FOLLOWUP candidate, NOT a missing-route.)

---

## Phase 1 — flag-NAME axis closure + backfill

**Files:** `crates/mnemonic-toolkit/tests/lint_argv_secret_flags.rs` (rebuild)

- [ ] **Step 1 — RED:** add a test `secret_flag_routes_match_live_gui_schema` that runs `mnemonic gui-schema` (subprocess, per `cli_gui_schema.rs` pattern), collects every `(subcommand, flag.name)` with `secret==true && kind!="boolean"` into a `BTreeSet`, and asserts set-equality with a declared `SECRET_ARGV_FLAG_ROUTES: &[(&str,&str)]`. Seed `SECRET_ARGV_FLAG_ROUTES` with ONLY the **9** flag-NAME routes the old table covered (R0 M1: bundle `--passphrase`; verify-bundle `--passphrase`; convert `--passphrase` + `--bip38-passphrase`; derive-child `--passphrase`; slip39-split `--passphrase`; slip39-combine `--passphrase` + `--share`; seed-xor-combine `--share`) so the test goes RED against the **16** unlisted (nostr `--secret`; silent-payment `--secret`/`--passphrase`; electrum-decrypt/import-wallet `--decrypt-password`; import-wallet/inspect/repair/verify-bundle/xpub-search×3 `--ms1`; xpub-search×3 `--passphrase`; seedqr-decode `--digits`).
- [ ] **Step 2** — run; expect FAIL listing the ~16 missing routes (proves the closure catches the decay).
- [ ] **Step 3** — backfill all missing flag-NAME routes into `SECRET_ARGV_FLAG_ROUTES` until set-equality holds (the enumeration IS the spec — copy the failing-set the test prints, verified against the gui-schema dump).
- [ ] **Step 4 — evidence per flag route:** for each `(subcommand, --X)`, assert the implementing source (`src/cmd/<subcommand>.rs` or its module dir) contains a NON-ARGV-channel anchor — any of: `"--X-stdin"` / `"X_stdin"` / (`secret_in_argv_warning` naming `"--X"`) / (`"--X -"` / `value == "-"` carve-out) / **(R0 I3) `@env:`-class: `"@env:"` / `"resolve_env_sentinels"` / `"needs_env_sentinel_resolution"`**. Encode as a per-route `evidence: &[&str]` keyed by `(subcommand, flag)` — the route SET is now closure-checked (Step 1), so a new flag can't be added without BOTH a route entry AND its evidence. **R0 I3:** `import-wallet --ms1` + `verify-bundle --ms1` anchor on `@env:` (they have no `*-stdin`/`-`). **R0 M2:** `--share` appears on BOTH slip39-combine (`"--share -"`) and seed-xor-combine (`"--share phrase=-"`/`secret_in_argv_warning`) — distinct anchors, correctly handled by the route-keyed map (do NOT share one anchor). **If any route has NO non-argv anchor → real leak (escalate; recon+R0 found none).**
- [ ] **Step 5** — run both tests GREEN; commit.

## Phase 2 — `--from` + `--slot` axes closure (per-subcommand, R0 I1+I2 collapsed)

**Files:** same test file. **NOTE:** `secret_taxonomy::{SECRET_NODE_TYPES_ARGV, SECRET_SLOT_SUBKEYS}` are NOT enumerated per-route (R0 I1 — node-acceptance is per-subcommand + invisible to closure inputs). They serve only as documentation that `--from`/`--slot` CAN carry a secret; the closure keys on subcommand membership.

- [ ] **Step 1 — RED:** add `secret_from_subcommands_match_gui_schema` — from gui-schema, collect subcommands whose flags include a flag NAMED `"--from"` (R0 I2: name-match, NOT a kind); assert set-equality with a declared `SECRET_FROM_SUBCOMMANDS: &[&str]`. Add `secret_slot_subcommands_match_gui_schema` likewise for `"--slot"`. Seed both EMPTY → RED listing the live `--from`/`--slot` subcommand sets.
- [ ] **Step 2** — run; FAIL listing the live members. `--from` = exactly the **7**: convert, derive-child, final-word, seed-xor-split, slip39-split, seedqr-decode, seedqr-encode (R1 M-B: closed set, no extras). `--slot` = exactly the **4**: bundle, verify-bundle, export-wallet, import-wallet. (Verify against the dump.)
- [ ] **Step 3** — populate the declared sets to match; assert each subcommand's source has its non-argv anchor:
  - `--from` → `"=-"` / `value == "-"` (all 7).
  - `--slot` → `"slot-stdin"` / `"slot_stdin"` (bundle, verify-bundle); **`@env:`-class** `"@env:"`/`"resolve_env_var_sentinel"`/`"resolve_env_sentinels"` (R1 I-A: import-wallet — `import_wallet.rs:1314-1317`); the secret-subkey REFUSAL (export-wallet — R1 M-A: anchor on the RUNTIME token `"validate_watch_only"` / `"watch-only by definition"` if present in `cmd/export_wallet.rs`, else the `wallet_export/mod.rs` source; the doc-comment `"secret subkeys"`/`"REFUSED"` @`export_wallet.rs:92` is a weaker fallback).
  - (R0 I1: the `=-`/slot-stdin/`@env:` route is uniform per-subcommand — covers every node/subkey accepted — so no per-node enumeration is needed.)
- [ ] **Step 4** — GREEN; commit.

## Phase 3 — retire stale scaffolding + release

- [ ] **Step 1** — delete the obsolete `canonical_list_has_twenty_eight_rows` hardcoded-count test (superseded by the closures) and the stale "20 flag-rows" prose (`:5`, `:44`); update the module doc to describe the closure model + removal-detection property (the closures fail on BOTH unlisted-new AND listed-but-removed routes — preserving the original `:20-24` removal-detection intent without the hand-frozen count). **(R0 M3)** state that the set-closure (route EXISTS in clap) and the per-route evidence-grep (non-argv route WIRED in source) are ORTHOGONAL — both retained; the evidence map is a staleness vector but fails LOUD (missing anchor → red), so it self-heals rather than silently decaying.
- [ ] **Step 2 — release-prep:** `Cargo.toml` 0.36.1 → 0.36.2; `Cargo.lock` regen; `scripts/install.sh:32` self-pin `mnemonic-toolkit-v0.36.2`; `CHANGELOG.md` [0.36.2] PATCH entry (test-only).
- [ ] **Step 3 — FOLLOWUPS:** close `lint-argv-secret-flags-canonical-table-rebuild-from-clap`. File `import-wallet-ms1-argv-advisory-gap` (R0 I3 NOTE: `import-wallet --ms1` fires NO `secret_in_argv_warning` — only `@env:`; a pre-existing advisory-hygiene gap, NOT a missing route, out of scope for this test-only cycle).
- [ ] **Step 4 — end-of-cycle opus review** → persist `design/agent-reports/v0_36_2-end-of-cycle-review.md` → fold → GREEN.
- [ ] **Step 5 — ship toolkit:** merge→master (ff), tag `mnemonic-toolkit-v0.36.2`, push, GH release; verify rust + manual + install-pin-check CI. **No GUI cycle** (test-only; no CLI surface change).

---

## Self-review / open R0 questions
- **Spec coverage:** flag axis (P1), `--from`+`--slot` axes (P2), retire+release (P3). ✓
- **R0+R1 dispositions folded (all RESOLVED):** I1+I2 (axes 2/3 collapsed to per-`(subcommand, --from|--slot)` name-match — no per-node cartesian; export-wallet refusal is its anchor); I3 (`@env:` anchors on the flag axis for `--ms1`); **R1 I-A (`@env:` propagated to the `--slot` axis for import-wallet)**; M1 (seed = 9); M2 (`--share` route-keyed collision); M3 (closure-vs-evidence orthogonality); **R1 M-A (export-wallet anchors on the runtime refusal token)**; **R1 M-B (`--from` = closed set of 7)**. Confirmed: gui-schema subprocess is the flag-axis source (Cli binary-private); `--from`/`--slot` membership = name-match (no `NodeValueComposite` kind); bidirectional set-equality preserves removal-detection; no new lib export needed. Live sets: `--from`=7, `--slot`=4, axis-1 flag routes=25 (9 covered + 16 missing).
- **Type consistency:** `SECRET_ARGV_FLAG_ROUTES: &[(&str,&str)]` + per-route evidence; node/slot declared sets derived from `secret_taxonomy` constants.
- **No placeholder evidence:** every backfilled route must cite a real source anchor proven to exist (Step 4 / P2 Step 3); a route with no anchor escalates, it is NOT given a fake anchor.
