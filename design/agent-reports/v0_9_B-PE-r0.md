# Cycle B PE R0 — release-rollup review (Opus)

**Date:** 2026-05-13.
**Reviewer:** Opus (per `feedback_opus_primary_review_agent`).
**Scope:** PE (release rollup) deliverables across both repos.
**Verdict:** CLEAR. 0 critical / 0 important / 3 nit (all folded inline).
**Predecessor reports:** Phase 3a R1 CLEAR (`v0_9_B-phase-3a-r1.md`),
Phase 3b R1 CLEAR (`v0_9_B-phase-3b-r1.md`).

## PE batch under review

### Toolkit
- `design/agent-reports/v0_9_B-secret-memory-hygiene-matrix.md` (NEW) — PE.T1
- `design/SPEC_secret_memory_hygiene_v0_9_B.md` (`parse.rs:45` → `parse.rs:65`) — PE.T7
- `crates/mnemonic-toolkit/tests/mlock_g6_invariant.rs` (NEW) — PE.T3
- `crates/mnemonic-toolkit/tests/mlock_unit.rs` (Phase 2 G6 stub removed; cross-ref comment) — PE.T3
- `.github/workflows/rust.yml` (new `g6-invariant` job) — PE.T3
- `crates/mnemonic-toolkit/Cargo.toml` (0.9.2 → 0.10.0) + `Cargo.lock` — PE.T4
- `CHANGELOG.md` (`mnemonic-toolkit [0.10.0]` section) — PE.T6

### mnemonic-secret
- `.github/workflows/rust.yml` (NEW; first Rust CI for ms-cli) — PE.T2 + PE.T3 G6 job
- `crates/ms-cli/tests/mlock_g6_invariant.rs` (NEW) — PE.T3
- `crates/ms-cli/Cargo.toml` (0.2.2 → 0.3.0) + `Cargo.lock` — PE.T4
- `CHANGELOG.md` (`ms-cli [0.3.0]` section) — PE.T6

### NOT in this review (deferred to post-tag per Cycle A precedent `a87ffdb`)
- PE.T5 FOLLOWUPS closures (`secret-memory-hygiene-cycle-b` parent in both
  repos) — defers so the close-commit can cite actual tag-commit SHAs.
- PE.T9 tag push — happens after this review CLEARs.

## Source-of-truth verification (`feedback_r0_must_read_source_off_by_n`)

Reviewer verified by direct grep against source (not plan narrative):

| Claim | Source | Verified |
|---|---|---|
| 12 `ResolvedSlot` ctor sites populate `_entropy_pin:` | `synthesize.rs:{1059,1213}`, `parse_descriptor.rs:{1176,1741,1755}`, `cmd/bundle.rs:{371,441,475,518,1049,1099}`, `cmd/verify_bundle.rs:496` | ✓ |
| Of those, 6 populate `Some(Rc::new(pin_pages_for(...)))` and 6 populate `None` | identical breakdown to matrix §1 Site 2 row | ✓ |
| 1 `DerivedAccount` ctor populates `_entropy_pin:` | `derive_slot.rs:89` | ✓ |
| 7 bip85 `format_*` functions add Site 4 pins | `bip85.rs:{84,110,138,170,188,203,241}` | ✓ |
| Site 5 pin at `parse.rs:65` (post Cycle A `Zeroizing<String>` shift) | `mnemonic-secret/crates/ms-cli/src/parse.rs:65` | ✓ |
| `mnemonic` binary wires `report_at_exit()` at `main.rs:101` | `mnemonic-toolkit/crates/mnemonic-toolkit/src/main.rs:101` | ✓ |
| `ms` binary wires `report_at_exit()` at `main.rs:130` | `mnemonic-secret/crates/ms-cli/src/main.rs:130` | ✓ |
| 14-item G6 MANIFEST exactly matches top-level names in both `mlock.rs` files | `extract_top_level_names()` over both sources | ✓ |
| 11 tests in `src/mlock.rs::mod tests` | `grep -c '^    #\[test\]'` | ✓ |
| 4 tests in `tests/mlock_unit.rs` after stub removal | direct read | ✓ |
| `MlockedZeroizing<T>` retired (Fix B) — 0 references in any source | `grep -c MlockedZeroizing` | ✓ (0) |

## Workflow safety (`feedback_r2_blocking_vs_cosmetic_gate`)

Both repos' `.github/workflows/rust.yml` use only trusted GitHub-provided
context (`${{ matrix.os }}`, `${{ runner.os }}`, `${{ github.workspace }}`).
No untrusted-input interpolation (issue/PR titles, commit messages, etc.).
`MNEMONIC_TEST_MLOCK_FAIL_MODE: "off"` is quoted defensively (the YAML 1.1
`off` boolean-false trap previously observed at toolkit run `25834458333`
on 2026-05-14 is pre-empted on the ms-cli mirror by quoting from the
start).

ms-cli test invocations use `--bin ms` (not `--lib`) because ms-cli is a
binary-only crate (`[[bin]] name = "ms"`); `mod mlock;` is binary-private.
Confirmed by reading `mnemonic-secret/crates/ms-cli/src/main.rs:20` and
the absence of `[[lib]]` in `crates/ms-cli/Cargo.toml`.

## Nits folded inline (3 total)

**N-1 (matrix Site 1 narrative).** `v0_9_B-secret-memory-hygiene-matrix.md`
§1 Site 1 rows for `bundle` + `verify-bundle` claimed the pin pattern was
`pin_pages_for(&synthetic_args.as_bytes())`. Actual: per-field
`pin_pages_for(p.as_bytes())` for the passphrase (`bundle.rs:129`) plus
per-slot `pin_pages_for(s.value.as_bytes())` (`bundle.rs:133`), each
immediately after `apply_stdin_substitutions()` returns. Narrative
corrected.

**N-2 (matrix §5 G6 manifest enumeration).** §5 G6 narrative enumerated
the manifest as `{... FailMode (cfg(test)) + FailMode::parse +
FailMode::current ...}`, but the actual `MANIFEST` const in the G6 test
sources does NOT include those names — they live inside `mod fail_mode`
(column-indented) and the test's `extract_top_level_names` filter
excludes them by the leading-whitespace check. Including them would
break the test. The nested `mod fail_mode` items are still covered by
the normalized-source byte-equality check (test 1), just not by the
name-export manifest check (test 2). Narrative corrected to enumerate
the actual 14-item MANIFEST and add a note about the nested `fail_mode`
coverage.

**N-3 (toolkit CHANGELOG test count).** The v0.10.0 CHANGELOG "Tests"
section claimed "~25 new mlock-module unit + subprocess tests". Actual
count: 11 tests in `src/mlock.rs::mod tests` + 4 in `tests/mlock_unit.rs`
+ 2 in `tests/mlock_g6_invariant.rs` = 17 total. Section rewritten to
enumerate each component precisely.

## Cycle-close gate disposition (mirrors SPEC §6)

| Gate | Status | Evidence |
|---|---|---|
| G1 — Functional correctness | ✓ | 4 G1.1-G1.4 in `tests/mlock_unit.rs`; ubuntu+macos matrix |
| G2 — Soft-fail coverage | ✓ | G2.1/G2.3-debug/G2.3-release/G2.4 in `src/mlock.rs::mod tests`; CI workflow steps with subprocess-fresh env |
| G3 — Platform coverage | ✓ | Ubuntu + macOS matrix in `.github/workflows/rust.yml` |
| G4.a — Zeroize-on-Drop preserved | ✓ | Sites 2/3/4 declaration-order discipline; Cycle A `impl Drop for DerivedAccount` PRESERVED |
| G4.b — Miri on unsafe blocks | ✓ | `cargo +nightly miri test mlock::` CI job |
| G5 — Cross-repo lockstep | ⏳ | tag pair `mnemonic-toolkit-v0.10.0` + `ms-cli-v0.3.0` to push within single PE session (PE.T9) |
| G6 — Inline-copy equivalence | ✓ | `tests/mlock_g6_invariant.rs` in both repos; CI `g6-invariant` jobs check out sibling at master |
| G7 — No wire-format regression | ✓ | mlock is functionally transparent; v0.1 + v0.2 fixture-corpus SHA pins still hold |

## Verdict

**CLEAR for tag push.** All 7 SPEC §6 gates met save G5 (which is the
tag-push action itself; PE.T9 satisfies it). The 3 nits were doc-accuracy
items folded inline. No critical or important findings; no rework needed.
