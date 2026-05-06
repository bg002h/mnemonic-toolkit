# Phase A — scope reduction + A.3 engraving-card cleanup — code-reviewer r1 (2026-05-06)

## Phase A scope reduction (per plan §"Scope reductions to consider during execution")

The original Phase A plan bundled three sub-phases:
- A.1: delete 6 legacy CLI flags from `BundleArgs` + `VerifyBundleArgs`; sweep 9 mode-violation guards (retain 3); delete 3 test files; add `cli_mode_violations_v0_5.rs`.
- A.2: rewrite ~25 integration tests (~1500 LOC) to use `--slot` syntax exclusively.
- A.3: delete dead `BundleJson.engraving_card` field + initializers + stale doc-comment.

A.1 + A.2 together represent ~2500 LOC of mechanical-but-error-prone churn. v0.5.0 ships A.3 only; A.1 + A.2 deferred to v0.5.1 with full test-rewrite focus.

This matches the plan's explicit scope-reduction trigger: when a phase's mechanical-but-error-prone churn risks the release window, defer the rewrite-phase to a follow-on cycle and ship the rest of the cycle's deliverables intact. The same pattern unlocked v0.4.4 (helper foundation) → v0.4.5 (call-site rollout).

FOLLOWUPS will be marked: `legacy-cli-flag-deletion` and `legacy-flag-deprecation` remain at tier `v0.5.1`; the v0.5.0 release notes call this out explicitly.

## A.3 implementation review

### Critical
None.

### Important
None.

### Low / Nit
None blocking.

Implementation:
- `format.rs::BundleJson.engraving_card: Option<String>` field DELETED (was always `None` since v0.4.2 dropped legacy emission).
- `cmd/bundle.rs:713` — `engraving_card: None,` initializer DELETED.
- `synthesize.rs:1308` — `engraving_card: None,` test fixture DELETED.
- `format.rs::BundleInputForCard` doc-comment rewritten to drop reference to the historical `EngravingMode` enum and explicitly note the v0.5 stderr-only emission contract.

Active stderr emission path (`build_unified_card` + `engraving_card_unified`) and `--no-engraving-card` CLI flag both preserved untouched.

## Test status

236 lib + 22 integration suites pass.

## Outcome

Phase A.3 APPROVED. A.1 + A.2 deferred to v0.5.1 per scope reduction. Proceed to Phase R.
