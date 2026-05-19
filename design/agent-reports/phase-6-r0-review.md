# Phase 6 R0 review — wallet-import v0.26.0 (manual + GUI lockstep)

**Date:** 2026-05-18
**Reviewer:** opus architect (R0)
**Toolkit commit:** `f94441e` (HEAD; parent `72575e2` = Phase 5 R0 folds GREEN); worktree `.claude/worktrees/wallet-import-export-multiformat-brainstorm/`, branch `worktree-wallet-import-export-multiformat-brainstorm`
**GUI commit:** `b2e281a` (single commit on branch `feat/import-wallet-v0_11_0`; parent `0e40237` = master HEAD); repo `/scratch/code/shibboleth/mnemonic-gui`

**Verdict:** **YELLOW — 1 Critical, 4 Important, 5 Minor.** The two-repo Phase 6 shipment delivers a coherent manual + schema surface for `mnemonic import-wallet` and the kittest harness pins the contracts that the implementation actually keeps. The Critical and the worst two Importants all orbit the same SPEC §9.3 / Phase 6 plan §6.5-§6.6 divergence: the env-var seed channel was specified as load-bearing argv-leak protection and was promised by the kickoff per `[[feedback-run-confirm-modal-renders-argv-verbatim]]`. In v0.11.0 it is NOT implemented; `--ms1` literal seeds flow verbatim through `assemble_argv` to argv AND to the run-confirm modal monospace display. The manual prose presented to the user under `4c-import-wallet.md` §9.3 and §"Refusals + advisories" actively describes the **aspirational** behavior as if it were the shipped behavior. Folding this is the cycle-close gate; the toolkit-side manual is otherwise clean.

## Critical

### C1 — manual-gui `4c-import-wallet.md` describes argv-leak protection that v0.11.0 GUI does NOT ship

**Sites:**
- `docs/manual-gui/src/40-mnemonic/4c-import-wallet.md:125-141` (§"Env-var seed channel"): claims the GUI "collects per-cosigner-index secret values into the subprocess env-var bag, sets `MNEMONIC_MS1_<i>=<value>` per secret, replaces the argv flag values with `@env:MNEMONIC_MS1_<i>` sentinels" etc.
- Same file `:197-200` (worked-example BSMS step 7): describes the run-confirm modal as displaying `--ms1 @env:MNEMONIC_MS1_0`.
- Same file `:225-227` (§"Refusals + advisories"): "the GUI does this automatically".
- Toolkit CLI manual `docs/manual/src/40-cli-reference/41-mnemonic.md:723-728` + `:850-853`: cross-references the (non-shipped) GUI behavior.

**Verified ground truth (the GUI does NOT do any of this):**

1. `mnemonic-gui/src/runner.rs:74-114` — spawn flow is `Command::new(argv[0]).env("MNEMONIC_FORCE_TTY", "1").args(&argv[1..])`. **No secret collection. No per-cosigner env-var injection. No argv rewrite. No env-var cleanup after subprocess exit.** The only env-var set is `MNEMONIC_FORCE_TTY=1` for the auto-fire TTY contract.
2. `mnemonic-gui/src/form/invocation.rs:236-251` — the v0.3-era repeating-secret branch comment: "Repeating secrets currently route through `state.values` like non-secret repeating … the in-memory share strings are plain heap allocations during emission."
3. `mnemonic-gui/src/main.rs:683-688` — the run-confirm modal renders argv verbatim with no redaction. This is the `[[feedback-run-confirm-modal-renders-argv-verbatim]]` behavior verified live 2026-05-18.
4. `mnemonic-gui/tests/kittest_import_wallet_form.rs:154-213` (`cell_import_wallet_repeating_ms1_argv`) — asserts the literal-seed pass-through. The cell pins the broken contract.

**Fix:**
1. Rewrite `4c-import-wallet.md` §"Env-var seed channel" to describe the **shipped** behavior: the GUI emits the user's typed value verbatim to argv; toolkit-side resolves `@env:VAR` if the user typed it. NOTICE that to avoid argv-leak in v0.11.0 GUI the user MUST type `@env:MY_VAR` themselves with `MY_VAR` exported in the calling shell.
2. Same patch to `4c-import-wallet.md` §"Refusals + advisories" — strip "the GUI does this automatically".
3. Toolkit CLI manual `41-mnemonic.md:723-728` + `:850-853` — same dilution.
4. File `gui-import-wallet-env-var-secret-channel` FOLLOWUP in `mnemonic-gui/FOLLOWUPS.md` (the implementer's report claims they did — verified by `Grep`: NO such entry exists in either repo). Tier `v0.12.0`. Cross-repo companion in toolkit FOLLOWUPS.md.

**Confidence: 95.**

## Important

### I1 — `gui-import-wallet-env-var-secret-channel` FOLLOWUP claimed-filed-but-not-filed

**Sites:** `mnemonic-gui/tests/kittest_import_wallet_form.rs:44-46` module-doc cites the FOLLOWUP slug; `Grep` against both FOLLOWUPS.md files returns **NO matches**. Dangling reference.

**Fix:** File the FOLLOWUP. Captured under C1 fold.

### I2 — Phase 6 plan §6.3-§6.9 cells: 3 of 9 specified items not covered

| Plan § | Cell | Status |
|---|---|---|
| §6.3 | form_renders_import_wallet_in_combobox | `cell_import_wallet_in_subcommands_set` (covered) |
| §6.4 | file_picker_blob_argv | `cell_import_wallet_blob_path_argv` + `cell_import_wallet_blob_stdio_sentinel_argv` (covered) |
| §6.4.a | file_picker_extension_filter | **NOT covered** |
| §6.4.b | blob_paste_textarea_routes_to_stdin | **NOT covered** |
| §6.5 | repeating_ms1_text_inputs_sentinel_argv | LITERAL emission, not sentinel (covered-with-changed-semantics per C1) |
| §6.6 | run_confirm_modal_shows_sentinels | **NOT covered** (blocked by C1) |
| §6.7 | select_descriptor_dropdown_argv | `cell_import_wallet_format_dropdown_argv` + `cell_import_wallet_select_descriptor_default_suppressed` (covered) |
| §6.8 | env_var_unset_subprocess_exits_1 | **NOT covered** (deferred with C1) |
| §6.9 | env_var_no_parent_leak | **NOT covered** (deferred with C1) |

**Fix:** EITHER add cells for §6.4.a and §6.4.b in the fold pass, OR file FOLLOWUP `gui-import-wallet-cell-coverage-gap` recording the deferral. §6.8/§6.9 ride the C1 FOLLOWUP.

**Confidence: 85.**

### I3 — Schema `--select-descriptor` is FlagKind::Text but manual claims TaggedOrIndexed widget

**Sites:**
- `mnemonic-gui/src/schema/mnemonic.rs:1432-1443` — `FlagKind::Text`, default `Some("all")`.
- `4c-import-wallet.md:108-110`: "The GUI renders this flag as a TaggedOrIndexed widget — a dropdown with the three named tags plus a Number spinner for the integer form."

**Fix:** EITHER bump the schema `--select-descriptor` to `FlagKind::TaggedOrIndexed(&["all", "active-receive", "active-change"])` OR walk back the manual prose to match `FlagKind::Text`. Manual walkback is faster and lower-risk for v0.26.0; the schema is functionally correct (free-form text validates by toolkit at run time).

**Confidence: 85.**

### I4 — `pinned-upstream.toml [mnemonic].tag = "mnemonic-toolkit-v0.24.0"` blocks the schema-mirror drift gate from greening for v0.11.0 GUI

**Site:** `mnemonic-gui/pinned-upstream.toml:22`

Phase 7 cycle-close MUST bump:
- `pinned-upstream.toml:22` from `mnemonic-toolkit-v0.24.0` → `mnemonic-toolkit-v0.26.0`
- `Cargo.toml` `[dependencies] mnemonic-toolkit` tag — same bump.

Phase 7 sequence: toolkit PR merge → toolkit tag created → GUI PR's bumps committed → GUI PR merge.

**Fix:** Coordination item only — capture in Phase 7 cycle-close checklist.

**Confidence: 90.**

## Minor

### M1 — `4c-import-wallet.md:151-154` slot-editor refusal hint reads ambiguously

Says "the watch-only validator refuses non-`phrase` subkeys at run time" — implies GUI-side validation; actual rejection is toolkit-side at parse-time. Rewrite to: "The GUI's slot editor renders all subkeys; the toolkit (not the GUI) rejects non-`phrase` subkeys at parse time."

**Confidence: 70.**

### M2 — `41-mnemonic.md:687-688` doesn't state fewer-than-N `--ms1` fallback

The flag-table cell doesn't say what happens when the user supplies FEWER `--ms1` than the blob has cosigners. Add one sentence: "Cosigners not addressed by any `--ms1[N]` flag remain watch-only (no entropy attached)."

**Confidence: 60.**

### M3 — Envelope key spelling agreement (positive)

`bsms_audit` / `source_metadata` / `status` field-name agreement across all three manuals + source-of-truth `cmd/import_wallet.rs`. Mirror clean. Including v0.26.0 `status` extension key (Phase 5 R0 fold) surfaced at `45-foreign-formats.md:242-245`.

**Confidence: 100** (positive verification).

### M4 — Screenshot TODO anchors

`4c-import-wallet.md:204,218` "Screenshot: TODO post-v0.11.0-GUI tag". Acceptable per project convention (v1.0 manual-gui used same pattern). File a Phase 7 cycle-close FOLLOWUP if not covered.

**Confidence: 60.**

### M5 — Cspell additions are tight (positive)

`docs/manual/.cspell.json:87-89` adds `listdescriptors`, `VARNAME`; `docs/manual-gui/.cspell.json:91-92` same. No misspellings slipped in.

**Confidence: 100** (positive).

## Out-of-scope but worth recording

- **Manual lint empirical verification.** Did NOT run `make -C docs/manual lint` in this review (read-only convention). Phase 7 cycle-close MUST.
- **BIP-129 / BIP-380 / BIP-389 citations** in `45-foreign-formats.md` verified normative.
- **FOLLOWUP slug pre-existence.** Verified `bsms-first-address-verify`, `bsms-verify-signatures`, `wallet-export-bsms-emitter`, `wallet-import-fixture-corpus-expansion`, `wallet-import-signet-regtest-disambiguation` exist. **NOT verified to exist** (cited in `45-foreign-formats.md:262-271`): `wallet-import-sparrow`, `wallet-import-specter`, `wallet-import-electrum`, `wallet-import-coldcard`, `wallet-import-coldcard-multisig`, `wallet-import-jade`, `wallet-import-bsms-round-1`, `wallet-import-bsms-encrypted`. Either file as placeholders or strip slug-form references.

## Cross-repo readiness for Phase 7

1. `pinned-upstream.toml [mnemonic].tag` bump (I4).
2. `mnemonic-gui/Cargo.toml` `mnemonic-toolkit` dep tag bump.
3. **C1 fold:** dilute or rewrite manual prose to match shipped GUI behavior; file `gui-import-wallet-env-var-secret-channel` FOLLOWUP cross-repo.
4. **I1 fold:** file the FOLLOWUP the kittest cell already names.
5. **I2 fold:** either add §6.4.a + §6.4.b cells or amend the plan-doc to record the deferral.
6. **I3 fold:** schema `--select-descriptor` widget vs manual prose — pick manual walkback.
7. 8 missing-FOLLOWUP slugs cited in `45-foreign-formats.md:262-271` — file as placeholders.
8. Phase 7 cycle-close architect must run `make -C docs/manual lint MNEMONIC_BIN=...` empirically.
9. Re-grep both FOLLOWUPS.md files to verify the Status-flip discipline (`[[feedback-per-phase-agents-forget-followup-status-flip]]`).

## Verdict reasoning

Phase 6 deliverable is structurally sound: GUI schema entry mirrors the toolkit clap surface, kittest cells pin contracts that actually ship, toolkit manual is byte-comparable against `--help`, `cli-subcommands.list` mirror gate has the right entry, foreign-formats chapter is normatively accurate.

The single Critical (and its dependent Importants) is the manual-prose vs runner-source mismatch on the load-bearing security claim. Shipping a manual that claims protection the GUI does not provide is a regression worse than shipping no manual update at all. Fix is small (prose rewrite + FOLLOWUPs filed cross-repo); once C1 + I1 + I2 + I3 land the cycle closes cleanly.
