# v0.9.0 Cycle B Phase 2 R1 architect review (mlock module, Fix B)

**Reviewer:** Opus 4.7 (1M context), invoked as architect-review on Cycle B Phase 2 post-implementation (commits `8193e22` → `a49386f` → `dc41d54` → `c9eeecb` → `f181c8e` → `9765310` → `3cc0627`, atop Phase 1 close `eae66c6`).
**Date:** 2026-05-13.
**SPEC:** `design/SPEC_secret_memory_hygiene_v0_9_B.md` (commit `a49386f`, post-Fix-B fold).
**Plan:** `~/.claude/plans/v0_9_B-secret-memory-hygiene-cycle-b.md` (post-Fix-B + I-1 fold; not in git).
**R0 v1 report:** `design/agent-reports/v0_9_B-phase-2-mlock-module-r0.md` (commit `8193e22`, RE-DRAFT).
**R0 Fix-B verify:** `design/agent-reports/v0_9_B-phase-2-mlock-module-r0-fixb-verify.md` (commit `dc41d54`, LOCK).
**Scope of review:** all 12 R1 checklist items per plan §"Phase 2" T5.
**Verdict:** **FOLD — 0 Critical / 1 Important / 0 Nit** at confidence ≥ 80. One missing test (G2.3-debug). Re-dispatch R2 after fold.

---

## Summary

Total findings at confidence ≥ 80: **0 Critical / 1 Important / 0 Nit**.

The GREEN commit (`9765310`) lands Fix B verbatim: no `MlockedZeroizing<T>` type, no `new_with_drop_probe<F>`, no manual `alloc/ptr::write/dealloc` Drop body. Module surface is exactly `pin_pages_for` + `PinnedPageRange { start, page_count }` (with Drop) + `report_at_exit` + three pub test helpers (per R0 v1 I-R0-4 cfg(test) reachability fold). RED commit (`c9eeecb`) added the predicted test surface except for `g2_3_einval_debug_panics`. CI workflow (`3cc0627`) lands rust.yml from scratch with three jobs and the `--include-ignored g2_1` separate-process invocation. Unsafe discipline is clean: 7 `unsafe {` openers, all with SAFETY comments within ±5 lines; `lint_safety_first_party_mlock.rs` enforces. Cycle A lints (`lint_argv_secret_flags`, `lint_safety_third_party_blocked`, `lint_zeroize_discipline`) untouched and still green.

The one Important finding is a missing test for G2.3-debug — the gate is documented as retained in Phase 2 (SPEC §2 row 7, §6 G2.3, plan P2.T2 step 4) but `mlock_unit.rs`'s deferral comment only lists G2.2/G2.3-release/G2.5. The debug_assert is in the production source but never exercised.

---

## §1. Fix B fidelity

All R1 checklist Fix-B items PASS. `MlockedZeroizing<T>` absent; `new_with_drop_probe<F>` absent; manual `alloc/ptr::write/dealloc` Drop body absent. Module surface is exactly the locked Fix B set. Test helpers pub per R0 v1 I-R0-4.

Confidence: 100.

---

## §2. Acceptance gate verification

| Gate | Phase | Status | Evidence |
|---|---|---|---|
| G1.1 (single page) | Retained P2 | PASS | `g1_1_single_page_pin_has_page_count_one` at `tests/mlock_unit.rs:14-21` |
| G1.2 (multi page) | Retained P2 | PASS | `g1_2_multi_page_pin_has_page_count_at_least_two` at `tests/mlock_unit.rs:23-34`; uses `page_size_for_test()` |
| G1.3 (zero length) | Retained P2 | PASS | `g1_3_zero_length_is_no_op_no_syscall_no_panic` at `tests/mlock_unit.rs:36-43` |
| G1.4 (page-aligned exactly one page) | Retained P2 | PASS-with-relaxed-assert | `g1_4_page_aligned_exactly_one_page_count_one` at `tests/mlock_unit.rs:45-64`; accepts page_count ∈ {1,2} since `vec![]` isn't guaranteed page-aligned |
| G2.1 (eperm) | Retained P2, #[ignore]-gated | PASS | `g2_1_eperm_increments_failure_count` at `tests/mlock_unit.rs:79-96`; CI invokes via `--include-ignored g2_1` |
| G2.3-debug (debug_assert on EINVAL) | Retained P2 | **MISSING** | See I-1 finding |
| G2.4 (off control) | Retained P2 | PASS | `g2_4_off_control_no_failure_when_ulimit_sufficient` at `tests/mlock_unit.rs:102-114` |
| G2.2 (enomem) | Deferred to P3a | DEFERRED-DOCUMENTED | SPEC §2 row 7; plan P2.T2 |
| G2.3-release | Deferred to P3a | DEFERRED-DOCUMENTED | Same |
| G2.5 (stderr summary) | Deferred to P3a | DEFERRED-DOCUMENTED | Same |
| G3 (platform matrix) | Retained P2 | PASS | `.github/workflows/rust.yml` Ubuntu+macOS matrix; `ulimit -l 65536` on Linux |
| G4.a (Zeroize-on-drop composes) | Retained P2 | PASS-narrow | `g4_a_pin_and_zeroize_compose_without_panic` at `src/mlock.rs:409-419` |
| G4.b (Miri green) | Retained P2 | DOC-PASS | `.github/workflows/rust.yml` miri job; local deferred to CI |

---

## §3. Unsafe discipline + SAFETY-comment lint compliance

7 `unsafe {` openers (cfg-split syscall wrappers + Rust 2024 `unsafe_op_in_unsafe_fn` requirement). All carry SAFETY comments within ±5 lines per `tests/lint_safety_first_party_mlock.rs`:

| # | Line | Context | Δ |
|---|---|---|---|
| 1 | 42 | `libc::sysconf(_SC_PAGESIZE)` in `page_size()` | 4 |
| 2 | 81 | `sys_munlock` in `PinnedPageRange::drop` | 4 |
| 3 | 104 | `sys_mlock_attempt` in `pin_pages_for` | 5 |
| 4 | 256 | `libc::mlock` in prod `sys_mlock_attempt` | 2 |
| 5 | 267 | `libc::munlock` in prod `sys_munlock` | 1 |
| 6 | 276 | `libc::mlock` in cfg(test) `sys_mlock_attempt` FailMode::Off | 2 |
| 7 | 292 | `libc::munlock` in cfg(test) `sys_munlock` | 1 |

Lint passing confirmed in local test run. Confidence: 100.

---

## §4. Soundness checks

Page-rounding formula correct (4 unit tests assert single/multi/zero/aligned cases). MlockState thread-safety correct (Atomic + OnceLock primitives + idempotent first_errno set). EINVAL handling source matches SPEC §2 row 6 (debug_assert + record_failure fall-through). Drop-order discipline: PinnedPageRange::drop is munlock-only (no zeroize; caller's Zeroizing<Vec<u8>> handles zeroize per Fix B). Confidence: 95+ across the board.

---

## §5. No public-API drift

`src/lib.rs` contains exactly `pub mod mlock;`. `mlock.rs` exports exactly 6 pub items (PinnedPageRange struct, pin_pages_for, report_at_exit, 3 test helpers). `main.rs` mod declarations unchanged. PASS. Confidence: 100.

---

## §6. Cycle A discipline preserved

All three Cycle A lints (`lint_argv_secret_flags`, `lint_safety_third_party_blocked`, `lint_zeroize_discipline`) untouched and still green. No new `Zeroizing<T>` sites in Phase 2 (Sites 2/3 add `_entropy_pin` sibling fields in Phase 3a). PASS. Confidence: 100.

---

## §7. G7 wire-format SHA pins

Phase 2 touched zero fixture corpus files. Pins continue to hold transitively from Phase 1 R1's verification. Confidence: 92.

---

## §8. CI workflow correctness

`.github/workflows/rust.yml` (91 lines): three jobs (test matrix Ubuntu+macOS, miri Ubuntu nightly, clippy Ubuntu); `ulimit -l 65536` on Linux; `--include-ignored g2_1` separate cargo-test invocation; Miri scoped to `--lib mlock::`; trigger paths correct; security clean (no untrusted user input). PASS. Confidence: 95.

---

## §9. Mass-balance LOC sanity

Total Phase 2 footprint: ~731 lines (~652 non-blank) vs plan estimate ~500 LOC. ~30% over budget, driven by in-module unit tests + cfg-split syscall wrappers. Structural overage (architecture artifacts), not feature bloat. Not actionable.

---

## §10. Plan/SPEC consistency

All other Phase 2 items consistent. The one gap is G2.3-debug — SPEC + plan retain it; implementation lacks the test. See I-1.

---

## §11. No surprise additions

Cargo.toml: `libc = "0.2"` only. Cargo.lock regenerated. FOLLOWUPS untouched. Manual untouched. No drive-by refactors. PASS. Confidence: 100.

---

## §12. Phase 1 R0 Nit 2 (open, not P2 finding)

`bip85.rs:4-5` doc-comment header still reads "The 6 in-scope apps" — pre-v0.8 inaccuracy (DICE is the 7th). Pending for opportunistic future cleanup. Not Phase-2-blocking.

---

## §13. Verdict + exit-gate decision

### I-1 (Important, conf 85): G2.3-debug test missing

**File:** `crates/mnemonic-toolkit/tests/mlock_unit.rs`
**Source debug_assert:** `crates/mnemonic-toolkit/src/mlock.rs:115-120`

The G2.3-debug gate is retained in Phase 2 per SPEC §2 row 7 + SPEC §6 G2.3 + plan §"Phase 2" T2 step 4. The `mlock_unit.rs` header comment defers G2.2/G2.3-release/G2.5 to Phase 3a, implicitly retaining G2.3-debug — but no `g2_3_*` test exists in the file. The `debug_assert!` at `src/mlock.rs:115-120` is never exercised.

**Suggested fix (Option A — preferred):** Add `g2_3_einval_debug_panics` (`#[ignore]`-gated, `#[should_panic(expected = "EINVAL")]`) to `tests/mlock_unit.rs` + a fourth `--include-ignored g2_3` CI step in `.github/workflows/rust.yml`. Mirrors the G2.1 pattern.

**Alternative fix (Option B):** Amend SPEC §2 row 7 + §6 G2.3 to defer G2.3-debug to Phase 3a alongside G2.3-release (cleaner pairing since both want subprocess isolation).

Confidence 85. Real gap with clear evidence; below 95 only because the underlying debug_assert is in production source and indirectly covered by reasoning (record_failure plumbing is exercised by G2.1).

---

### Final verdict

**FOLD — 0 Critical / 1 Important / 0 Nit at confidence ≥ 80.**

Fix B is implemented faithfully; unsafe discipline is clean; CI infrastructure is correctly structured; Cycle A discipline is preserved; no surprise additions. The single Important finding is a documented-but-not-exercised acceptance gate (G2.3-debug). Fold I-1 (add the test + CI step OR amend SPEC), then re-dispatch R2 for verification.
