# v0.36.2 — Plan-doc R0 architect review (opus) — MANDATORY pre-implementation gate

**Date:** 2026-05-24
**Cycle:** v0.36.2 rebuild argv-leakage audit as a clap-derived closure (test-only)
**Branch:** `v0.36.2-argv-audit-closure`
**Reviewer:** opus (feature-dev:code-reviewer), R0 (agentId ab3b11fc7634bcaf0)
**Target:** `design/IMPLEMENTATION_PLAN_v0_36_2_argv_audit_closure.md`

---

## Critical
None. No secret-argv flag has zero non-argv channel; closure inputs all exist + lib-public.

## Important

**I1 — Axes 2+3 are NOT a derivable cartesian; per-subcommand node/subkey acceptance is invisible to both closure inputs, so `SECRET_NODE_TYPES_ARGV × {--from subcommands}` over-generates.** Accepted node/subkey sets are per-subcommand + non-uniform: convert `--from` = all 9 nodes (`convert.rs:94-119`); derive-child = phrase/xprv only (`derive_child.rs:200-205`); final-word = phrase (`final_word.rs:56-60`); seed-xor-split = phrase (`seed_xor.rs:42-52`); slip39-split = phrase/entropy (`slip39.rs:82`); seedqr-decode = seedqr only (`seedqr.rs:62-67`); seedqr-encode = phrase (`seedqr.rs:81`). `--slot`: bundle/verify-bundle = full secret subkeys; **export-wallet REFUSES secret subkeys** (`export_wallet.rs:92-94`); import-wallet = phrase only (`import_wallet.rs:179-181`). Blanket cartesian invents non-existent routes (e.g. 9×derive-child, 5×export-wallet-all-refused). Must pick (a) hand-declare per-subcommand sets (reintroduces decay) or (b) over-complete superset with stated residual.

**I2 — No `NodeValueComposite` gui-schema kind; `--from`/`--slot` emit `kind="text"`.** `classify_kind` (`gui_schema.rs:1236-1289`) maps custom value-parsers to `"text"` fallthrough; `NodeValueComposite` is comment-only (`gui_schema.rs:43`). Phase-2 Step-1's "NodeValueComposite" discriminator does not exist; membership detection is `flag.name=="--from"`/`"--slot"` name-match only, and node-acceptance is NOT recoverable from gui-schema (reinforces I1).

**I3 — `import-wallet --ms1` + `verify-bundle --ms1` have NO `*-stdin`/`-` route — only `@env:VAR`.** `verify-bundle`: `needs_stdin_substitution` (`verify_bundle.rs:910-912`) + advisory (`:889-908`) cover only `--slot`/`--passphrase`, NOT `--ms1`; only `@env:` (`:943-945`). `import-wallet --ms1` (`import_wallet.rs:171-172`): no `--ms1-stdin`, no `-`, NO `secret_in_argv_warning` (grep: 0 hits); only `@env:` (`:1311-1312`). `--ms1-stdin` exists ONLY in xpub-search seed modes. NOT a true leak (`@env:` is a real non-argv channel — `env_sentinel.rs:1-16` lists `--ms1`), but the plan's Step-4 anchor list would match NONE → falsely flag a leak. Plan MUST add `@env:`-class anchors (`resolve_env_sentinels`/`@env:`). (NOTE: import-wallet `--ms1` has NO argv advisory at all — pre-existing hygiene gap, candidate separate FOLLOWUP, out of scope for test-only.)

## Minor
- **M1** — axis-1 live count = 25 (confirmed); OLD flag-NAME-axis rows = **9** (not "6" as Phase-1 Step-1 seed says); 25−9=16 missing (exact, matches recon). Fix the seed count.
- **M2** — `--share` is secret on BOTH slip39-combine (`--share -`, slip39.rs:143-153) + seed-xor-combine (`--share phrase=-`, seed_xor.rs:74-82) — different evidence anchors; route-keyed `evidence` handles it but call out the collision.
- **M3** — evidence-grep is NOT redundant with the closure (closure proves route EXISTS in clap; grep proves stdin route WIRED in source — orthogonal, both needed). It's a staleness vector but fails LOUD (self-heals). State in the Phase-3 doc rewrite.

## Verification summary (confirmed correct)
- **A:** gui-schema `secret` = `flag_is_secret(name)` (`gui_schema.rs:1170`); `flag_is_secret` (`secrets.rs:49-64`) EXCLUDES `--from`/`--to`/`--slot` (`secrets.rs:37-40`) → axes 2+3 genuinely can't come from the flag bit. `flag_is_secret` not separately needed (gui-schema bit suffices for axis 1).
- **secret_taxonomy:** `SECRET_NODE_TYPES_ARGV` (`:95-105`, 9 = SECRET_NODE_TYPES + minikey) + `SECRET_SLOT_SUBKEYS` (`:111`, 5) are `pub const`, re-exported (`lib.rs:82`), already imported in `cli_gui_schema.rs:357`. `_ARGV` is the correct wider argv set (vs narrower persistence set) per `is_argv_secret_bearing` (`convert.rs:117-119`).
- **B:** 16 missing independently re-derived; exact set matches recon (10 single-subcommand + 6 xpub-search); no misclassification.
- **C:** exhaustively verified. WIRED via `*-stdin`/`-`: nostr/silent-payment `--secret`/`--passphrase`, electrum-decrypt/import-wallet `--decrypt-password`, seedqr-decode `--digits`, inspect/repair `--ms1` (`-`), xpub-search×3 `--ms1`/`--passphrase`, all `--from` (`=-`), secret `--slot` (slot-stdin). Only import-wallet+verify-bundle `--ms1` lack `*-stdin`/`-` → `@env:` (I3).
- **D:** gui-schema JSON exposes name/kind/secret (`gui_schema.rs:238-263`); test via `assert_cmd cargo_bin`. Membership = name-match (I2).
- **E:** bidirectional set-equality removal-detection sound; use set (gui-schema sorted @1102); low false-positive.
- **G:** test-only PATCH v0.36.2; Cargo.toml 0.36.1; install.sh:32 self-pin v0.36.1; FOLLOWUP slug exists (FOLLOWUPS.md:3146); no new lib export needed; deleting count-test + "20" prose safe; NO GUI/manual lockstep.
- **F:** new secret flag fails BOTH gates — coherent.

VERDICT: RED (0C/3I)

---

## Fold disposition (controller) — R0 → R1
- **I1+I2 (combined fold):** COLLAPSE axes 2+3 from per-(node/subkey) cartesian to per-**(subcommand, `--from`/`--slot`)** routes detected by gui-schema NAME-match. Rationale: the `=-`/slot-stdin non-argv route is UNIFORM per-subcommand (covers every node/subkey that subcommand accepts), so per-node granularity adds nothing to the stdin-route invariant. Completeness: declared `SECRET_FROM_SUBCOMMANDS`/`SECRET_SLOT_SUBCOMMANDS` set-equal the gui-schema `--from`/`--slot`-bearing subcommand sets (catches a NEW --from/--slot consumer; a widened node-acceptance within an existing one is safe because the `=-` route already covers all nodes). export-wallet `--slot` exception: its evidence anchor is the secret-subkey REFUSAL (`export_wallet.rs:92-94`), not a stdin route. No per-node decay surface remains.
- **I3:** add `@env:`-class anchors (`@env:`, `resolve_env_sentinels`, `needs_env_sentinel_resolution`) to the recognized evidence set; `import-wallet --ms1` + `verify-bundle --ms1` anchor on `@env:`. File a separate FOLLOWUP for import-wallet `--ms1` missing argv-advisory (pre-existing; out of scope).
- **M1** seed = 9 (not 6). **M2** note --share collision (route-keyed). **M3** state evidence-vs-closure orthogonality in the doc.
Re-dispatch R1 after fold.

---

## R1 (round 1) — VERDICT: RED (0C/1I)
Reviewer agentId a2c6f5f5dedf0f7bc. I1+I2 collapse VERIFIED sound (--from set = exactly 7 {convert, derive-child, final-word, seed-xor-split, slip39-split, seedqr-decode, seedqr-encode}; --slot set = exactly 4 {bundle, verify-bundle, export-wallet, import-wallet}; `=-` route value-uniform → no coverage lost). I3 @env: anchors load-bearing in verify_bundle.rs:937/943 + import_wallet.rs:1308/1311 for --ms1. M1 (9 rows, 16 missing), M2 (--share collision), M3 (orthogonality) all confirmed. secret_taxonomy+flag_is_secret lib-public; xpub-search ×3 correct (address-of-xpub carries neither --ms1 nor --passphrase).

**I-A (NEW, fold-completeness gap):** the I3 `@env:` extension was applied to the Phase-1 flag-NAME axis but NOT propagated to the Phase-2 `--slot` axis. `import-wallet --slot @N.phrase=` is secret (`import_wallet.rs:1044-1051` accepts only `phrase`) and its ONLY non-argv channel is `@env:` (`import_wallet.rs:1314-1317` `resolve_env_var_sentinel`) — NOT slot-stdin (grep: zero `slot_stdin`/`apply_slot_stdin` in import_wallet.rs) and NOT the export-wallet refusal. So Phase-2 Step-3's `--slot` allowlist (slot-stdin OR refusal) is UNSATISFIABLE for import-wallet → a RED test the implementer can't satisfy without un-reviewed deviation. FIX: add `@env:`-class anchors to the `--slot` allowlist (plan lines 25 + 54).

**M-A (Minor):** export-wallet refusal anchor tokens `"secret subkeys"`/`"REFUSED"` exist ONLY in the doc comment (`export_wallet.rs:92`); the load-bearing runtime refusal is `wallet_export/mod.rs:60,108` (`validate_watch_only` / "watch-only by definition"). Satisfiable as-is (grep hits the doc comment) but anchoring on the runtime token is more robust. Optional.
**M-B (Minor):** the Phase-2 Step-2 `--from` list appends "…" but the set is exactly 7 (closed); drop the ellipsis.

VERDICT: RED (0C/1I)

## Fold disposition (controller) — R1 → R2
- **I-A:** add `@env:`-class anchors (`@env:`/`resolve_env_sentinels`/`needs_env_sentinel_resolution`/`resolve_env_var_sentinel`) to the Phase-2 `--slot` evidence allowlist (lines 25 + 54); import-wallet `--slot` anchors on `@env:`.
- **M-A:** export-wallet `--slot` anchor = prefer the runtime token (`validate_watch_only` call-site, verify it's in `cmd/export_wallet.rs` at impl; else `wallet_export/mod.rs` "watch-only by definition") over the doc-comment tokens.
- **M-B:** drop the trailing "…" — `--from` set is exactly the 7 listed.
Re-dispatch R2.

---

## R2 (round 2) — VERDICT: RED (0C/2I)
Reviewer agentId a70cde428f158b63e. I-A/M-A/M-B VERIFIED landed (import-wallet --slot @env: anchors present @import_wallet.rs:1297-1317; export-wallet `validate_watch_only` @export_wallet.rs:13,256,334; --from closed set of 7). Two NEW Important, one root cause:
- **I-1:** flattened nested-subcommand `source_file` mis-resolution — gui-schema emits `xpub-search-path-of-xpub` etc. (`gui_schema.rs:1002` hyphen-flatten); files are `src/cmd/xpub_search/<mode>.rs` + `seed_intake.rs`, NOT `src/cmd/xpub-search-path-of-xpub.rs`. Also `seedqr-decode`→`src/cmd/seedqr.rs`. Generic `<subcommand>.rs` rule → missing-file panic.
- **I-2:** inspect/repair `--ms1` stdin route is in SHARED `src/repair.rs` (`value == "-"` @:145, `resolve_groups`/`expand_dashes` @:223/:251), NOT `cmd/inspect.rs`/`cmd/repair.rs` (which carry only the doc-phrase) → no anchor in the named file.
- **M-1 (root cause):** plan never commits a per-route `source_file` table for the 16 backfilled routes; the generic rule is wrong for flattened subcommands + shared-library routes.

VERDICT: RED (0C/2I)

## Fold disposition (controller) — R2 → R3
Folded M-1 (closes I-1+I-2): Phase-1 Step-4 now carries an EXPLICIT per-route `source_file` requirement + a verified non-obvious-mappings table (xpub-search×3 → `xpub_search/seed_intake.rs` for --ms1 + `<mode>.rs` for --passphrase; seedqr-decode → `seedqr.rs`; inspect/repair --ms1 → `src/repair.rs`; seed-xor/slip39 → their cmd files; rest → own `cmd/<name>.rs`). Phase-2 --from notes seedqr-decode/-encode → `seedqr.rs`. Re-dispatch R3.
