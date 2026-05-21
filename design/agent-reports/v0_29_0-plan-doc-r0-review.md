# v0.29.0 plan-doc R0 architect review — opus

**Reviewer:** opus
**Plan-doc:** `design/PLAN_mnemonic_toolkit_v0_29_0.md`
**Date:** 2026-05-21

## Verdict: YELLOW

0 Critical + 4 Important + 5 Minor + 2 new FOLLOWUPs.

## Important (R0 fold targets)

### I1 — GUI schema-mirror scope confusion (Task 7 Step 2 + Risk Flag)

Plan-doc says implementer updates `mnemonic-gui/src/schema/mnemonic.rs` for the xpub-search wire-shape break. **This is wrong.** The GUI's `schema_mirror.rs` test enforces **flag-name set parity** between hand-maintained `SubcommandSchema` and `gui-schema` JSON output (verified at `mnemonic-gui/tests/schema_mirror.rs:91-121` + `tests/xpub_search_schema_mirror.rs`). Slugs A and B change **internal types + runtime JSON output**, NOT clap flag definitions. **`src/schema/mnemonic.rs` needs zero edits beyond pin bump** — the v0.29.0 binary's `gui-schema` JSON will emit the same flag-name set as v0.28.4.

Fold: rewrite Task 7 Step 2 to: "Run `mnemonic gui-schema` against new binary; diff JSON; only edit `src/schema/mnemonic.rs` if a flag name/dropdown-value drifted (expected: no drift)." Risk-flag "schema-mirror exact-byte sensitivity" also wrong; rewrite.

### I2 — Slug C arm-grouping incompatible with alphabetical sort (Task 4 Step 4)

`exit_code` has multi-variant `|` arms today (`error.rs:429-481`) where same-exit variants are grouped (e.g., 18 variants `=> 2`). Alphabetical sort will INTERLEAVE different-exit variants, forcing every variant onto its own arm. **Lock: all `exit_code` match arms become single-variant post-sort.** Non-trivial mechanical work that inflates diff.

Fold: Task 4 Step 4 explicit lock + size estimate revision.

### I3 — Split Slug C into separate commit on same branch (Task 9)

Bundling 132 arm reorders with semantic refactors (Slugs A+B) hides regression risk. Sonnet verification of "zero-semantic-drift" across 132 arms is brittle.

Fold: Task 9 → 2 commits on same branch — `refactor(error): retroactive alphabetical sort` (Slug C only; sonnet diff-verify single concern) THEN `release(toolkit): v0.29.0 — Slugs A + B + version bump`. Same tag, same push, bisect-friendly.

### I4 — Task 7 ordering contradiction (Task 7 Step 1 vs Steps 2-4)

Task 7 Step 1 says "deferred until Task 9 completes" while Step 2/3/4 don't gate similarly. Implementer reading sequentially may execute Step 2 against stale binary.

Fold: Lock entire Task 7 behind Task 9 Steps 1-5 completion. Re-order if needed.

## Minor

1. Concern #2 (serde tag conflict) safe — `result` field disappears from struct (becomes discriminator only) per [serde-rs/serde#1161].
2. Concern #3 (`target_xpub_canonical` on no-match) verified — populated NOT optional; tagged-enum `NoMatch` retains correctly.
3. Concern #8 verified — one consumer file: `tests/cli_xpub_search_drift_v0_27_0.rs` (lines 80-106 + 137-144 + 180-188) needs `#[ignore]` under Slug B.
4. Concern #10 (CHANGELOG accuracy) — deferred to execution-time, acceptable.
5. Slug A alphabetical: `BsmsSixLine` < `BsmsTwoLine` (`S` < `T`) confirmed correct.

## New FOLLOWUPs surfaced

1. `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` — document in CLAUDE.md / FOLLOWUPS that GUI schema-mirror only gates flag-name parity. Wire-shape consumers of xpub-search JSON have NO automated drift gate (real gap).
2. `error-rs-exit-code-arm-fragmentation-post-sort` — record that post-sort, all `exit_code` arms become single-variant; future re-grouping is a separate readability decision.

## Path to GREEN

Fold I1 (rewrite Task 7 Step 2 + Risk Flag for schema-mirror scope) + I2 (lock `exit_code` arm fragmentation) + I3 (split Slug C into separate commit) + I4 (gate all Task 7 behind toolkit tag push). Re-dispatch reviewer for R1 short verification.
