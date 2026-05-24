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
