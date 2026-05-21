# v0.28.7 plan-doc R1 architect review — opus

**Reviewer:** opus
**Plan-doc:** `design/PLAN_mnemonic_toolkit_v0_28_7.md` (post-R0-fold state)
**Date:** 2026-05-20

## Verdict: YELLOW (4 stale-narrative residuals from R0 folds; no architectural rework)

## R0 fold landing summary

- **C1 (fixture paths in Task 3 matrix):** LANDED. All 6 fixtures verified to exist at `crates/mnemonic-toolkit/tests/fixtures/wallet_import/`. However, the same fixture-correction was NOT propagated to Task 2 Step 3 (separate site; see I-R1-1 below).
- **I1 (Task 1 Step 6 regex narrative):** LANDED. Prose at L120-143 is internally coherent. False-start variants dropped.
- **I2 (Slug 4 parse-side detection):** LANDED. Task 4 Step 2 uses `matches!(script_type, WalletScriptType::P2tr | WalletScriptType::P2trMulti)`.
- **I3 (alphabetical insertion direction):** LANDED. All Steps 1-4 say "Insert BEFORE `BsmsTaprootRefused`" correctly.

## Important (R1 fold targets — all mechanical)

1. **I-R1-1 — Task 2 Step 3 fixture path stale.** `coldcard-multisig-mainnet.json` → use `coldcard-ms-2of3-p2wsh-with-xfp.txt`.
2. **I-R1-2 — Risk-flag #1 stale regex narrative.** Delete/replace ("regex crate v1.x does NOT support lookarounds").
3. **I-R1-3 — Risk-flag #4 stale string-sniff narrative.** Replace with `WalletScriptType` scope check.
4. **I-R1-4 — Risk-flag #2 stale fixture-naming pattern.** Restate against actual fixture mix.

## R2 disposition

R1 reviewer's recommendation: "Fold the 4 Important findings inline (~10 minutes: 1 path correction in code block + 3 risk-flag rewrites/deletions), then proceed to execution. No re-dispatch needed unless additional folds surface during the prose cleanup. All 4 R0 folds landed semantically correctly; the residual issues are stale-narrative cleanup, not architectural rework."

All 4 R1 folds applied in controller-direct edits at `/scratch/code/shibboleth/mnemonic-toolkit/design/PLAN_mnemonic_toolkit_v0_28_7.md`. Proceeding to execution per R1's stated pre-authorization for trivial post-fold cleanup.
