# Phase 4 Root + Glue Review — r2

**Date:** 2026-05-04
**Commits under review:** `ffd72bd` (r1 fixup), parent `05909c4` (Phase 4 feature)
**Reviewer:** opus phase-review

## Verdict

0 critical / 0 important / 3 low / 2 nits

✅ **Phase 4 r2 terminator reached — cleared to proceed to Phase 5.**

## Critical / Important

(none)

## Low / Nit (unchanged from r1; deferred to design/FOLLOWUPS.md)

- **L-1:** SPEC §2.2.2 prose says "four checks" but §5.4 schema requires 9 with `skipped`. Internal SPEC inconsistency.
- **L-2:** `BundleMismatch.card: &'static str` constrains future runtime-id callers.
- **L-3:** verify-bundle text-mode line `"{}: {} {}"` trails a space when `detail` is empty.
- **N-1:** `error::Result<T>` allow-comment says "in-crate" but `pub type` is exported.
- **N-2:** `BundleMismatch` doc-comment refers to Phase 5; staleness risk.

## Confirmed

- `run<W: Write, E: Write>` signature carries `stderr`; threaded to `run_watch_only<E: Write>`. `run_full` unchanged (no SPEC §2.2.1 stderr requirement).
- §2.2.2 3-line warning emitted at top of `run_watch_only` (line 426-439), before any parse — text byte-exactly matches SPEC §2.2.2 lines 156-158.
- New unit test `watch_only_emits_spec_2_2_2_warning_to_stderr` exercises the warning emission via Vec<u8> stderr buffer + invalid xpub forcing parse error; asserts all 3 lines.
- 5 prior `watch_only_checks` tests unaffected (they call the pure helper directly).

## Smoke checks

- `cargo test -p mnemonic-toolkit`: 53 passed (52 prior + 1 new).
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`: clean.
- `cargo fmt --check -p mnemonic-toolkit`: clean.
