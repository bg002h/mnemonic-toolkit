# Phase P.3 + P.6 helper wire-up — code-reviewer r1 (2026-05-06)

## Findings

### Critical
None.

### Important

**I-1: Two stale `#[allow(dead_code)]` annotations remain post-wiring.**
`crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` lines 1212 and 1230 — the SuppliedCards struct + emit_verify_checks function annotated as dead-code but now actively called from run_full and run_watch_only. The plan's "remove `#[allow(dead_code)]` from helper" task was deferred to Phase R but should land with the wire-up commit so rustc surfaces accidental orphaning in any future refactor.

**Status:** addressed inline before commit. Both attributes deleted; comments referring to "wired into run_full in P.3" deleted.

**I-2: `run_watch_only` --master-fingerprint guard at line 363 is unreachable from run() dispatch.**
The mode-violation pre-check at lines 221-227 of run() catches missing --master-fingerprint with a byte-exact §6.6 ModeViolation error before run_watch_only is invoked. The defensive guard inside run_watch_only emits a different error type (BadInput) and is dead code under normal dispatch.

**Status:** noted; benign. Defensive guard retained as belt-and-braces in case of future internal call paths. Not blocking.

### Low / Nit

**L-1: Watch-only `SuppliedCards.ms1 = &args.ms1` may be non-empty if user supplies --ms1 in watch-only mode.**
Old `watch_only_checks` ignored args.ms1; the helper now compares it against the synthesized watch-only Bundle's `ms1: vec![""]`. Result: `ms1_decode` may decode the user's spurious ms1, then `ms1_entropy_match` fails because expected="" ≠ supplied=non-empty. This is arguably more useful (tool flags the user's mistake) but is a behavior change.

**Status:** captured as FOLLOWUP `verify-bundle-watch-only-spurious-ms1-handling` at `v0.4-nice-to-have` for SPEC §5.7 confirmation.

**L-2: `emit_verify_checks` doc-comment "wired into run_full in P.3" stale.**
Subsumed by I-1 fix (attribute + comment deleted entirely).

**L-3: Test name `verify_bundle_json_emits_9_checks_in_spec_order` still accurate post-P.6.**
Nit only; no change.

## Deletion safety assessment

- `verify_md1_and_stub` / `verify_md1_only`: fully replaced by `emit_md1_checks`. No surviving callers.
- `watch_only_checks` + 5 unit tests: scenarios genuinely covered by helper unit tests + cli_verify_bundle_watch_only.rs end-to-end suite. No coverage regression.

## Outcome

P.3 + P.6 APPROVED with I-1 addressed inline. Proceed to P.4.
