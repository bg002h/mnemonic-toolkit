# v0.30.0 plan-doc R1 review

**Reviewer:** opus
**Round:** R1 (verify R0 fold)
**Plan under review:** design/PLAN_mnemonic_toolkit_v0_30_0.md (v2)
**R0 review:** design/agent-reports/v0_30_0-plan-doc-r0-review.md
**Date:** 2026-05-21

**Tooling note:** R1 reviewer's tools (Read/Grep/Glob/WebFetch/WebSearch) did not include Write — review returned verbatim; orchestrator persisted.

## R0 fold verification

| R0 finding | Claimed fold | Verified? | Note |
|---|---|---|---|
| C1 (thiserror) | hand-rolled impl Display; no thiserror dep | yes | Plan-doc shows `#[derive(Debug, Clone, PartialEq, Eq)]` + hand-rolled `impl Display` + `impl Error`. No `thiserror::Error` derive; no `#[error(...)]` attrs; no Cargo.toml dep add. Risk-register calls out dep-escape as REJECT. |
| C2 (exit 1 not 2) | exit 1 everywhere | yes | P0 STRICT-GATE locks block at top says `1`; manual chapter "Exit codes" says `1`; CHANGELOG omits any "2" claim; all 15 test-cell `.code(1)` assertions confirmed. |
| C3 (value_parser) | value_parser = parse_from_input + import | yes | `value_parser = parse_from_input` inside the `#[arg]` block; `use crate::cmd::convert::{parse_from_input, read_stdin_to_string, FromInput, NodeType};`. Matches seed_xor.rs:49 precedent. |
| C4 (DIGITS_24 index 102) | all sites updated to `...0102` | yes | All 4 `DIGITS_24` definitions and all manual/example digit strings end in `0102`. No `0099` remains. |
| I1 (NodeType::Phrase validation) | explicit BadInput rejection + test cell | yes | `seedqr encode only accepts phrase=<value> or phrase=-`. Test cell `encode_rejects_non_phrase_node_xpub`. |
| I2 (Zeroizing/mlock/argv-warning) | full hygiene applied + import | yes | Import of `secret_in_argv_warning`; advisory emit; `Zeroizing` wrap; `mlock::pin_pages_for` page pins; advisory presence/absence test cells. |
| I3 (--digits in flag_is_secret) | match arm + test extended | yes | Task 3 Step 4 adds `"--digits"` to match arm in alphabetical position + extends test list. |
| I4 (GUI placement names) | between seed-xor-combine (L2367) and slip39-split (L2375); encode before decode | yes | Plan cites correct neighbors; verb-ordering rationale (create-side before recover-side) makes seedqr-encode first. Verified against gui repo source. |
| I5 (.map removal) | dispatch arm without .map(|_| 0) | yes | `Command::Seedqr(args) => cmd::seedqr::run(args, stdin, stdout, stderr),` matches FinalWord/SeedXor/Slip39 pattern. |
| I6 (≥30 cells) | 31 test cells | yes | Test file ships 31 cells; covers (a) non-phrase rejection (I1), (b) JSON-mode encode rejection, (c) `.code(1)` numeric assertions on every failure path, (d) 13/15/18/21/25-word rejection, (e) stdin-form-encode tests, (f) round-trip-through-JSON envelope. |

## New Critical (introduced by v2)

NONE.

## New Important (introduced by v2)

### NEW-I1 — `decode_no_argv_advisory_on_stdin_form` test uses vacuous substring assertion

**Plan-doc location:** Task 4 Step 1 (`tests/cli_seedqr.rs`).
**Citation drift:** the test asserts `!stderr_str.contains("supplied in argv")`, but `secret_advisory.rs:36-38` emits `"warning: secret material on argv ({flag}) — pipe via {alternative} to avoid /proc/$PID/cmdline exposure"`. The substring `"supplied in argv"` is NEVER written; the negation is vacuously true for inline AND stdin code paths.

**Class:** same regression-gate latency bug as v0.5.1's `ci_workflow_snapshot` substring vacuity ([[feedback-ci-snapshot-test-substring-vacuity]]). A future regression that drops the conditional in `cmd/seedqr.rs::run_decode` would still pass the negative test.

**Fix:** assert on the load-bearing template substring `"secret material on argv"`. Same correction applies symmetrically to the positive cells `decode_emits_argv_advisory_on_inline_form` + `encode_emits_argv_advisory_on_inline_form` — both currently assert on the flag substring alone (e.g. `"--digits"`), which is too loose; tighten with `"secret material on argv"` AND the flag substring as paired assertions.

**Applied inline:** all three test cells updated in plan-doc to assert on `"secret material on argv"` (load-bearing) + flag substring (specificity).

## Verdict

**GREEN** after the one-line NEW-I1 fold inline. All 10 R0 findings folded correctly; no new Criticals; one Important regression-gate vacuity caught + applied inline.

## Summary

The v2 plan-doc folds all four Critical and all six Important R0 findings cleanly. Source citations (lib.rs L62/L63, cmd/mod.rs L13/L14, GUI schema L2367/L2375, error.rs:429 exit-code mapping, secret_advisory imports) are accurate against current source. Hand-rolled `impl Display` matches `seed_xor.rs:31-67` precedent exactly; `value_parser = parse_from_input` matches `seed_xor.rs:49`; `mnemonic_toolkit::mlock::pin_pages_for(...)` matches the 30+ existing call sites. Test cell count (31) clears the brainstorm-locked floor of 30. One new Important issue caught: the `decode_no_argv_advisory_on_stdin_form` negative test asserts on a substring (`"supplied in argv"`) that does not exist in the real `secret_in_argv_warning` template (which writes `"secret material on argv"`). The test would pass for every code path including a regression that drops the conditional, defeating the negative gate. Same too-loose pattern in two paired positive cells. Applied tighter `"secret material on argv"` substring + flag-name paired assertions inline. Plan-doc is GREEN, ready to commit.
