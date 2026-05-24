# v0.36.2 — End-of-cycle architect review (opus) — MANDATORY pre-tag gate

**Date:** 2026-05-24
**Cycle:** v0.36.2 rebuild argv-leakage audit as a clap-derived 3-axis closure (test-only)
**Reviewer:** opus (feature-dev:code-reviewer), end-of-cycle (agentId a72bdb57f2cba0fae)
**Scope:** whole-cycle diff origin/master..HEAD (9 files; substantive = tests/lint_argv_secret_flags.rs) + live source.

## Critical
None.
## Important
None.
## Minor
- **M1 (doc, FOLDED):** axis-1 completeness is TRANSITIVE on `flag_is_secret` (gui-schema `secret`=`flag_is_secret(name)`). A future secret value flag missing from `flag_is_secret` AND not `--from`/`--slot` would escape the closure — but also escapes the runtime advisory/zeroize/GUI-mask (more visible), so flag_is_secret is the authoritative anchor by design; no separate gate over it. Pre-existing boundary, not a regression. → folded a one-line module-doc note.

## Verification summary (all 7 gate items GREEN)
1. Closure correct + leading-gate: (a) flag-axis enumerates secret&&!boolean → 25, independently re-derived (10 --passphrase + 1 --bip38-passphrase + 2 --decrypt-password + 1 --digits + 7 --ms1 + 2 --secret + 2 --share) = MATCH; (b) --from=7, --slot=4 re-derived = MATCH; (c) evidence test orthogonal (proves route WIRED, not named) + fails loud; (d) only false-GREEN class = M1 boundary. Negative tests hold under committed logic.
2. Every route's anchor verified in COMMITTED source (import-wallet/verify-bundle --ms1 @env:; inspect/repair --ms1 repair.rs:145 `value=="-"`; xpub-search×3 seed_intake.rs; per-mode --passphrase; seedqr.rs; export-wallet validate_watch_only; --share collision distinct). Matches R3.
3. Version/release: Cargo.toml 0.36.2; Cargo.lock 0.36.2; install.sh:32 v0.36.2; CHANGELOG accurate; PATCH; NO CLI change → NO GUI/manual lockstep.
4. FOLLOWUPs: rebuild slug resolved (FOLLOWUPS.md:3150); import-wallet-ms1-argv-advisory-gap filed open.
5. No real leak: import-wallet/verify-bundle --ms1 + import-wallet --slot @N.phrase= are @env:-only (working channel, reachable); --ms1 advisory absence = hygiene FOLLOWUP not missing channel. Nothing escalates.
6. Test integrity: MNEMONIC_BIN else CARGO_BIN_EXE_mnemonic (bin name `mnemonic`); serde_json; BTreeSet (deterministic); crate_root `.` correct under cargo test cwd. No flakiness.
7. Clean-tag: old CANONICAL_FLAG_ROWS/28-count-test/"20"-prose fully removed (only the historical note @:3 references it); no debug; imports used in assert messages; R0 persisted (4-round RED 3I→1I→2I→GREEN).

VERDICT: GREEN (0C/0I)

## Controller note
GREEN → gate satisfied. M1 (doc boundary) folded. Cleared to tag/ship v0.36.2 (toolkit-only; no GUI cycle).
