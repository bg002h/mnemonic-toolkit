# R0 Review (round 3, TERMINAL) — IMPLEMENTATION_PLAN_verify_bundle_nonchunked_canonicalization.md

**Reviewer:** Fable architect (`model:"fable"`), 2026-07-12. Focused terminal convergence on the round-2 I-A/M-A folds. **Source SHA:** `de140a08`. **Usage:** 19 tool-uses, ~251s, 62907 tokens.

## Verdict: GREEN — 0 Critical / 0 Important (1 Minor doc-housekeeping, M-i)

The plan-R0 has converged. The I-A fold is correct and the re-fixtured cell is GREEN on master.

### I-A fold — CORRECT, cell GREEN on master
1. Folded cell has both fixes: `--template bip84` added to verify args + assert `contains("result: ok")`. All four in-cell cites verified accurate (verify_bundle.rs:435-443 ModeViolation-without-template; :558-567 general-path lowercase `result: ok`; :824 template-path-only "OK" string; cli_verify_bundle_full.rs:31-56 proven keyed precedent).
2. No remaining delta vs the precedent that would keep it RED: `--account 0` matches the default (verify_bundle.rs:63-64) and the bundle's emit account; expected-vs-supplied byte-identity holds (`run_full` synthesize_unified Md1Form::Policy :1095-1103, bundle md1-form default Policy bundle.rs:169-170); `--group-size 0` renders unbroken strings (display_grouping.rs:20-23,:66); keyed_cards parser byte-identical to proven template_cards.
3. **`--ms1` is REQUIRED (not optional) for keyed verify, and IS non-empty here.** `watch_only` decided by expected bundle (`expected.ms1.first().is_empty()`, :2116); a phrase slot → non-empty expected ms1 (synthesize.rs:391/417/571) → non-watch-only branch runs `ms_codec::decode(supplied_ms1)` (:2135-2137) which would FAIL if ms1 unsupplied. The cell's `filter(|m| !m.is_empty())` loop passes exactly one non-empty ms1, mirroring the precedent + the file's `verify_args`. Cell gets this right.

### M-A — CORRECT: plan cites `…_multisig.rs:293-361` (test spans :293-361, verified via `#[test]`/fn/brace). The remaining `:293-321` is only in the fold-log description.

### Fold log — ACCURATE (records round-2 I-A/M-A exactly).

### Minor
- **M-i (doc housekeeping, zero impl impact):** the header Status line + R0-trail were stale (said "awaiting round-2 GREEN", omitted planr0-round-2/round-3). [Folded post-review by the main loop.]

### Coherence scan — CLEAN (no dangling refs; Step-4 "GREEN before AND after" now true; Step-5 PASS expectation consistent; I-1 fold-log entry matches the edited cell).

## Proof of work
IMPLEMENTATION_PLAN (full :1-499); planr0-round-2 report (full); verify_bundle.rs :24-143/:380-574/:820-828/:1071-1132/:2037-2166; cli_verify_bundle_full.rs :1-130 (keyed precedent :31-56); cli_verify_bundle_md1_template.rs :1-100; cli_verify_bundle_md1_template_multisig.rs :280-300/:355-365; bundle.rs :75-95/:169-170/:990-1059; display_grouping.rs :20-32/:66; synthesize.rs ms1-emptiness (:391/:417/:454/:571-575); git HEAD = de140a08.
