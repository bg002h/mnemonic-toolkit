# v0.9.0 Cycle B Phase 2 R2 verification report (mlock fold)

**Reviewer:** Opus 4.7 (1M context), invoked as architect-review on Cycle B Phase 2 R1 I-1 fold.
**Date:** 2026-05-13.
**Fold commit:** `e53cca8`.
**R1 report under review:** `design/agent-reports/v0_9_B-phase-2-mlock-module-r1.md` (commit `cd32ad9`, FOLD verdict).
**Scope:** verify the fold resolves I-1 (G2.3-debug missing) and the additional cfg(test) reachability fix (G2.x moved from integration tests to library unit tests) introduces no regressions.
**Verdict:** **CLEAR — 0 Critical / 0 Important / 0 Nit at confidence ≥ 80.**

---

## Summary

Total findings at confidence ≥ 80: **0 Critical / 0 Important / 0 Nit.**

The fold cleanly resolves R1's I-1. G2.3-debug now exists as a library unit test (`g2_3_einval_debug_panics`) with the correct `#[should_panic(expected = "EINVAL")]` attribute, and the production `debug_assert!` message at `src/mlock.rs:115-120` literally contains the substring "EINVAL" so the matcher succeeds. The additional cfg(test) reachability fix (R0 v1 I-R0-4 framework applied to G2 by analogy) is sound: G2.x require `cfg(test)`-gated FAIL_MODE injection which is unreachable from `tests/mlock_unit.rs` per RFC 1604 — they correctly migrated to `src/mlock.rs`'s `#[cfg(test)] mod tests` block.

Phase 2 is ready to ship (P2.T6 push).

---

## §1. G2.3-debug verification (the original I-1)

`crates/mnemonic-toolkit/src/mlock.rs:469-476`:

- Test name: `g2_3_einval_debug_panics` — matches R1 suggested-fix Option A naming.
- `#[test]` + `#[ignore = "subprocess: requires MNEMONIC_TEST_MLOCK_FAIL_MODE=einval in env"]` + `#[should_panic(expected = "EINVAL")]` — all three attributes present.
- Test body: `let buf = vec![0u8; 64]; let _pin = pin_pages_for(&buf);` — invokes the production codepath that hits the `debug_assert!` via the EInval injection branch.
- Production debug_assert message at `src/mlock.rs:115-120` contains the literal substring `EINVAL` so `#[should_panic(expected = "EINVAL")]` matches at the message level (Rust's should_panic does substring matching against the formatted panic message).

Confidence: 100. I-1 fully resolved.

---

## §2. G2.1 + G2.4 placement verification

`g2_1_eperm_increments_failure_count` (`src/mlock.rs:448-462`):
- `#[ignore]`-gated with EPerm-env subprocess hint.
- Asserts `failure_count_for_test() > 0` and `first_errno_for_test() == Some(libc::EPERM)`.

`g2_4_off_no_synthesized_failures` (`src/mlock.rs:482-492`):
- `#[ignore]`-gated with off-env + ulimit hint.
- Asserts `failure_count_for_test() == 0`.

Both correctly use `pin_pages_for(&buf)` to drive the production codepath and rely on the pub test-helper accessors for state inspection.

Confidence: 100.

---

## §3. Integration-test cleanup verification

`crates/mnemonic-toolkit/tests/mlock_unit.rs` (90 lines):
- G1.1-G1.4 retained — page-count contract tests that need no cfg(test) hook.
- G2.x removed; module-level header explicitly explains the move with a back-reference to RFC 1604 and the R0 v1 I-R0-4 cfg(test) reachability framework.
- G6 placeholder retained.

Confidence: 100.

---

## §4. CI workflow verification

`.github/workflows/rust.yml` (119 lines):

- Three G2.x cargo-test steps, each with step-scoped `env:` block setting `MNEMONIC_TEST_MLOCK_FAIL_MODE` to the appropriate value (`eperm`, `einval`, `off`).
- Each step filters to the specific test name: `cargo test -p mnemonic-toolkit --lib mlock::tests::g2_N -- --include-ignored`.
- `ulimit -l 65536` set on Linux runners in all four test steps (build + default-tests + G2.1/G2.3/G2.4).
- No `${{ github.event.* }}` / `inputs.*` / `secrets.*` interpolation — only trusted `matrix.os` and `runner.os`.
- Miri job scoped to `--lib mlock::`.
- Clippy job uses `--all-targets -- -D warnings`.

Minor observation (not a finding — below confidence gate): one step has `MNEMONIC_TEST_MLOCK_FAIL_MODE: off` (unquoted). YAML 1.1 treats bare `off` as boolean false, which GitHub Actions may coerce to the string `"false"`. However, `fail_mode::parse("false")` returns `None`, falling back to `unwrap_or(FailMode::Off)` — so the test's observed behavior is identical regardless of which way the YAML parser jumps. Functionally robust; cosmetic clarity at most. (Confidence ~50 that quoting `'off'` would be marginally better; below the 80 reporting gate.)

Confidence: 95 overall.

---

## §5. cfg(test) reachability reasoning verification

The fold commit message correctly identifies the underlying constraint: `cfg(test)` is per-crate-not-per-build (RFC 1604). The library compiled for integration-test linking has `cfg(test)=false`, so the `fail_mode` module and the cfg(test) `sys_mlock_attempt` variant are both invisible to `tests/mlock_unit.rs`.

This is the same constraint R0 v1 I-R0-4 surfaced for the (now-moot) drop-probe. R1 inherited the framework but didn't explicitly flag it for FAIL_MODE; the fold caught the analogy and applied the fix proactively. The reasoning is sound and the application is correct.

Confidence: 100.

---

## §6. Local-verification claim spot-check

Static-verification basis:
- `g2_3_einval_debug_panics` body produces a debug_assert panic message containing "EINVAL"; `#[should_panic(expected = "EINVAL")]` uses substring match per Rust stdlib — PASS by construction.
- `g2_1_eperm_increments_failure_count` asserts EPerm injection's record_failure side effects, which directly correspond to the FailMode::EPerm branch returning `Err(libc::EPERM)` → record_failure → failure_count+=1, first_errno=EPERM — PASS by construction.
- `g2_4_off_no_synthesized_failures` under FAIL_MODE=off + `ulimit -l 65536` invokes the real `libc::mlock` via FailMode::Off; a 64-byte buf within `ulimit -l 65536` should succeed and not increment failure_count — PASS by construction (modulo CI env).
- The three `#[ignore]` attributes ensure default `cargo test -p mnemonic-toolkit` shows G2.x as ignored without setting any env var.
- No clippy-triggering patterns introduced.

Confidence: 92.

---

## §7. No regressions / surprise additions

- Fold-commit scope: `src/mlock.rs`, `tests/mlock_unit.rs`, `.github/workflows/rust.yml` only. No other files touched.
- G2.x unit tests do NOT call `std::env::set_var` inside test bodies — only the existing OnceLock initializer reads env. Correct: relies on env-before-process-start invariant.
- Unsafe-block count unchanged: 7 `unsafe {` openers in `src/mlock.rs`. SAFETY-comment lint discipline preserved.
- No `Cargo.toml` / `Cargo.lock` changes.
- FOLLOWUPS.md untouched.
- Manual untouched.

Confidence: 100.

---

## §8. Verdict + exit-gate decision

**CLEAR — 0 Critical / 0 Important / 0 Nit at confidence ≥ 80.**

The fold resolves R1's I-1 cleanly: G2.3-debug exists as `g2_3_einval_debug_panics` with `#[should_panic(expected = "EINVAL")]`; the production debug_assert message contains "EINVAL" so the matcher succeeds. The additional cfg(test) reachability fix (moving G2.1/G2.3-debug/G2.4 from integration tests to library unit tests, with per-test CI subprocess invocations) is the correct application of R0 v1 I-R0-4's framework. No regressions or surprise additions.

**Phase 2 exit-gate: GREEN.** Cleared for P2.T6 push.
