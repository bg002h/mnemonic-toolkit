# Phase 7 end-of-cycle holistic R0 review — wallet-import v0.26.0

**Date:** 2026-05-18
**Reviewer:** opus architect (holistic)
**Toolkit branch:** worktree-wallet-import-export-multiformat-brainstorm tip e773636
**GUI branch:** feat/import-wallet-v0_11_0 tip 616de38

**Verdict:** **GREEN — 0 Critical, 1 Important, 3 Minor.** The cycle is cycle-close-ready. All per-phase R0 folds verifiably landed in source; the manual prose was correctly diluted post Phase 6 R0 C1 to describe shipped GUI behavior accurately; the v0.5 SPEC carries §5.11 + §6.11 + §6.11.a per plan §7.1; all 13 cycle-close FOLLOWUPs are filed with `Status: open` (no `[[feedback-per-phase-agents-forget-followup-status-flip]]` debt). One Important: `pinned-upstream.toml` + GUI `Cargo.toml` toolkit-tag pin still references v0.24.0 (Phase 7 close MUST bump). Three Minor: one missed FOLLOWUP filing (Phase 1 R0 m1), the `Cargo.toml` toolkit version still 0.25.1 (Phase 7 close), and two acceptable manual-gui Screenshot TODOs per project convention.

## Critical

None.

## Important

### I1 — `pinned-upstream.toml:22` + `mnemonic-gui/Cargo.toml:42` still pin `mnemonic-toolkit-v0.24.0`

**Sites:**
- `/scratch/code/shibboleth/mnemonic-gui/pinned-upstream.toml:22` — `tag = "mnemonic-toolkit-v0.24.0"`
- `/scratch/code/shibboleth/mnemonic-gui/Cargo.toml:42` — `mnemonic-toolkit = { git = "...", tag = "mnemonic-toolkit-v0.24.0" }`

Phase 6 R0 I4 flagged this; toolkit-side cycle is shipping new error variants (`EnvVarMissing`, `ImportWalletParse`, `ImportWalletAmbiguousFormat`, `ImportWalletFormatMismatch`, `ImportWalletXprvForbidden`, `ImportWalletWatchOnlyViolation`, `ImportWalletSeedMismatch`) and new public surface (`crate::env_sentinel::resolve_env_var_sentinel`) that the GUI's schema-mirror drift gate will start consuming as soon as the GUI Cargo.toml bumps. The bump is a Phase 7 sequencing item, NOT a per-phase fold.

**Required Phase 7 sequencing (per plan §7.6):**
1. Toolkit PR/branch merge → `mnemonic-toolkit-v0.26.0` tag created.
2. GUI `pinned-upstream.toml:22` + `Cargo.toml:42` bumped → commit on `feat/import-wallet-v0_11_0`.
3. GUI PR merge → `mnemonic-gui-v0.11.0` tag.

Also note the toolkit's own `crates/mnemonic-toolkit/Cargo.toml:3` is still `version = "0.25.1"` — bump to `0.26.0` is Phase 7 close.

**Confidence: 95.**

## Minor

### M1 — Phase 1 R0 m1 (`phase-1-cell-1-13-restrengthen`) was never filed as FOLLOWUP

Grep against `design/FOLLOWUPS.md` returns no entry. Phase 7 may file it as a cycle-close cleanup OR accept as cycle-close-dropped.

**Confidence: 75.**

### M2 — Screenshot TODOs left in `4c-import-wallet.md:215,229`

Acceptable per `[[project-manual-gui-v1-0-closed]]` precedent.

**Confidence: 50.**

### M3 — Manual lint not empirically run in this read-only holistic review

Phase 6 R0 deferred `make -C docs/manual lint MNEMONIC_BIN=...` to cycle-close; Phase 7 MUST run it pre-tag per `[[feedback-architect-must-run-prose-commands]]`.

**Confidence: 60.**

## FOLLOWUP Status-flip audit

All 13 wallet-import FOLLOWUPs carry `Status: open`. **No Status-flip debt.** GUI-side companion `gui-import-wallet-env-var-secret-channel` is also `open` with correct cross-cite.

| Slug | Tier | Status |
|---|---|---|
| `bsms-first-address-verify` | v0.27 | open |
| `wallet-import-signet-regtest-disambiguation` | v0.27 | open |
| `wallet-import-bsms-checksum-delegation-note` | v0.26.0-cycle-close | open (FLIP THIS in Phase 7 close commit) |
| `bsms-verify-signatures` | v0.27 | open |
| `wallet-export-bsms-emitter` | v0.27 | open |
| `wallet-import-json-envelope-full-bundle` | v0.27 | open |
| `wallet-import-fixture-corpus-expansion` | v0.27 | open |
| `gui-import-wallet-env-var-secret-channel` | v0.12.0 | open |
| `gui-import-wallet-cell-coverage-gap` | v0.12.0 | open |
| `wallet-import-{sparrow, specter, electrum, coldcard, coldcard-multisig, jade, bsms-round-1, bsms-encrypted}` | v0.27 | open |

The `wallet-import-bsms-checksum-delegation-note` is tier `v0.26.0-cycle-close`, meaning Phase 7's SPEC-amend commit closes it. Flip `Status: open` → `Status: resolved <commit-sha>` in the same commit.

## Cross-repo readiness

| Item | Status | Phase 7 action |
|---|---|---|
| Toolkit `Cargo.toml` version `0.25.1 → 0.26.0` | not yet bumped | Phase 7 close |
| Toolkit CHANGELOG `[0.26.0]` entry | DRAFTED | Phase 7 close |
| GUI `pinned-upstream.toml:22` tag bump | still v0.24.0 | Phase 7 §7.6 step 2 |
| GUI `Cargo.toml:42` tag bump | still v0.24.0 | Phase 7 §7.6 step 2 |
| GUI `Cargo.toml:3` version `0.10.0 → 0.11.0` | not yet bumped | Phase 7 §7.6 step 2 |
| GUI CHANGELOG `[0.11.0]` entry | not yet written | Phase 7 §7.4 |
| SPEC §5.11 / §6.11 / §6.11.a in v0.5 SPEC | LANDED | done |
| `wallet-import-bsms-checksum-delegation-note` SPEC §4.4 amendment | not yet applied | Phase 7 close (this commit) |
| Manual lint `make -C docs/manual lint MNEMONIC_BIN=...` empirical run | PASSED post Phase 6 fold | re-run pre-PR |

## Cell-count tally

Baseline at v0.25.1: 1153. Post-cycle: ~1313 toolkit + 8 GUI = 168 new cells. Over budget (70-94 plan target) driven by Phase 4 canonicalize-discipline test density.

## CHANGELOG scope (recommended)

Already drafted; verified scope coverage. Add `### Resolved (FOLLOWUPS)` section pointing at `wallet-import-bsms-checksum-delegation-note`.

## Verdict reasoning

The cycle is structurally complete. The Phase 0-6 R0 reviewer loop converged on every load-bearing surface. All `pub(crate)` exports in `wallet_import/` are reachable from `cmd::import_wallet::run` (no orphaned helpers). No `TODO|FIXME|Phase N:` markers in shipped code. Single narrowly-scoped `#[allow(dead_code)]` remains, citing a documented v0.27+ FOLLOWUP. Manual prose accurately describes shipped GUI behavior. All 13 wallet-import FOLLOWUPs carry `Status: open` (no Status-flip debt). Cycle is GREEN-after-Phase-7-sequencing-completes → tag.
