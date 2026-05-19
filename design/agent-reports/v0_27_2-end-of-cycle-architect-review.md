# v0.27.2 + v0.11.1 lockstep cycle — end-of-cycle architect review (post-merge)

**Reviewer:** opus feature-dev:code-reviewer (1M context)
**Branch / artifacts reviewed:**
- toolkit master squash `ec04a00929f62c1c7f026e65aaeb710fc8a787cf` (PR #30, tag `mnemonic-toolkit-v0.27.2`)
- toolkit follow-on `e0ad04a` (sibling FOLLOWUP Status flip)
- GUI master squash `5254b59dbd4fd7532963c608955711575d528ace` (PR #9, tag `mnemonic-gui-v0.11.1`)
**Date:** 2026-05-19
**Verdict:** **YELLOW** — cycle shipped cleanly; one cross-repo FOLLOWUP cross-cite is out-of-sync + 1 cumulative-gap memory candidate. Hotfix not required.

## Critical
None.

## Important

### I1 — Cross-repo FOLLOWUP cross-cite drift: `gui-workflow-trigger-include-release-branches`

Toolkit-side at `design/FOLLOWUPS.md:2447` says `Status: resolved (sibling tag mnemonic-gui-v0.11.1; v0.27.2 Phase 3)`. GUI-side on `mnemonic-gui/FOLLOWUPS.md` master still says `Status: open`. The bidirectional Companion-cite invariant from CLAUDE.md ("both entries update in lockstep") violated.

**Fix:** 1-line patch on mnemonic-gui master flipping Status to `resolved (mnemonic-gui-v0.11.1; commit 5254b59)`.

Generalized failure mode: per-repo Status flips are reliable; cross-repo Companion flips are not. Tracked by `[[feedback-per-phase-agents-forget-followup-status-flip]]`, extended to cross-repo scope.

## Minor

### M1 — `Bsms(Option<BsmsAuditFields>)` deviation correctly accepted

The shipped variant shape preserves the type-level "representable-invalid" invariant (impossible cross-product of Bsms-with-source-metadata or BitcoinCore-with-bsms-audit is now unrepresentable). The residual `Option` correctly models the BSMS 2-line vs 6-line shape variance — it is not a representable-invalid state. Downstream impact: zero (5 emit sites at `cmd/import_wallet.rs` are pure field→method renames preserving `Option<&_>` shape). FOLLOWUP M1 `pr-26-import-provenance-three-variant-cleanup` at `design/FOLLOWUPS.md:2524-2539` correctly captures the residual design-aesthetic gap.

**No action required.**

### M2 — CHANGELOG accuracy verified

Per-section verification against shipped content: Fixed (2 items), Changed (1 item with the Bsms(Option<_>) variant accurately described), Tests (+1 + +4 cells verified), Conventions (4 new CLAUDE.md additions verified), Closed FOLLOWUPS (6 verified), Filed FOLLOWUPS (2 verified). No drift between narrative and shipped content.

### M3 — Cumulative schema-mirror gap (recommend new toolkit FOLLOWUP)

The Phase 3 inline CI fix added 8 flags to GUI `src/schema/mnemonic.rs` that v0.27.0 + v0.27.1 cycles never paired with a GUI schema update. The gap is cumulative — not a v0.27.2 regression — but was only revealed when v0.11.1's pin bump fired the `schema_mirror` drift gate on the accumulated delta.

The CLAUDE.md "Mirror invariant" line 24 covers the manual mirror; there is no equivalent "GUI schema in lockstep" clause. A toolkit-PR-only consumer can ship surface drift indefinitely until the next GUI pin bump.

**Recommended new FOLLOWUP (toolkit-side, v0.28+):** `gui-schema-mirror-lockstep-discipline` — codify the GUI schema lockstep invariant.

## Filed-as-FOLLOWUP / memory candidates

- **F1** `feedback-trial-cargo-build-at-plan-doc-r3` — plan-doc R3 for refactor-class cycles should include a trial cargo build with proposed type changes applied. Static review missed `Bsms(BsmsAuditFields)` type-fail vs upstream `audit: Option<_>` binding.
- **F2** `feedback-schema-mirror-gui-lockstep-cumulative-gap` — toolkit CLI surface drift accumulates silently until next GUI pin bump fires schema_mirror gate; codify GUI schema lockstep invariant.
- **F3** `feedback-ci-fixture-path-portability-include-str` — `include_str!("../../sibling-repo/...")` works locally with sibling worktree but fails in CI; always copy fixtures into local `tests/fixtures/` for portability.
- **F4** (optional, lower confidence) `feedback-clippy-doc-lazy-continuation-on-docstring-folds` — architect-GREEN'd docstring folds should still run `cargo clippy --all-targets -- -D warnings`.

## Cycle thesis vs reality

7 FOLLOWUPs closed (6 toolkit + 1 GUI) + 2 new filed. Zero wire-shape change verified. Patch-tier classification valid. Nothing smuggled in/out.

## Recommended actions

1. (1-min) Flip GUI-side companion FOLLOWUP entry to close I1.
2. (filing) Add F1, F2, F3 as memory entries.
3. (optional) File new toolkit FOLLOWUP `gui-schema-mirror-lockstep-discipline` from M3.

**No hotfix required.** Cycle is properly shipped; all findings are post-ship hygiene.
