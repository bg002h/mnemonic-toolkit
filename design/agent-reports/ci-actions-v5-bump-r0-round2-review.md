# R0 Architect Review — SPEC_ci_actions_v5_bump.md — Round 2 (fold-confirmation)

> Round 1 = GREEN + 2 Minor (M1, M2); both folded. Reviewer had Read/Grep; parent persists. Source re-verified vs `manual-gui.yml` at `15236ee`.

**Verdict: GREEN (0C/0I).** 2 Minor (1 new from the M2 fold, 1 carry-over).

## Critical
None.
## Important
None.

## Minor

- **M-new-1 (introduced by the M2 fold) — dangling forward-reference.** Item-1's M2 text defers SHA-pinning "as an option in the catch-up follow-up," but the follow-up it points to (`ci-actions-catch-up-to-latest-majors`, Item 3) enumerates only v6/v7/v8 vetting + the optional self-trigger-path add — it does NOT list SHA-pinning. The tracker exists; its body just doesn't carry the referenced item. Fix: add a clause to Item 3, e.g. "evaluate SHA-pinning vs floating-tag style." One clause; non-blocking.

- **M-carry-1 (pre-existing, NOT fold-introduced) — un-validated-tail enumeration axis.** The tail lists `install-pin-check (checkout)` and `manual/quickstart (setup-node + upload-artifact)` but omits `manual.yml:30`/`quickstart.yml:32` checkout sites, equally un-run on this PR. Axis = "un-validated invocations, per workflow"; under it the clause is incomplete. M1 only reworded the manual-gui clause, so not a fold regression; benign-for-v5 safety unaffected. Optional tidy.

## Internal consistency
- Matrix (manual-gui row) vs summary ("3 of the 4 majors") vs un-validated tail (manual-gui **download**-artifact pair) all agree.
- No new inconsistency beyond M-new-1.
- Round-1 substance intact: 25-count (15+4+4+2), pin-@v5 decision, PR rollout/rollback, FOLLOWUP flip+new tracker, CI-only/no-bump disposition all untouched.

## Fold confirmation
- **M1 — clean, source-accurate.** `manual-gui.yml` `build:` (115-116) has no `if:`; `release:` (176-179) is `needs: build` + `if: startsWith(github.ref,'refs/tags/manual-gui-v')`. PR paths include the own file (:20) → on PR `lint`+`build` run, `release` skips → `upload-artifact@v5` (151,158) PR-validated; only `download-artifact` (192,198) not. Matrix + summary + tail match.
- **M2 — landed, but created M-new-1** (the deferral target doesn't enumerate SHA-pinning). Close with one clause in Item 3.

**Disposition:** GREEN. Fold M-new-1 (+ optional M-carry-1 tidy) and confirm.
