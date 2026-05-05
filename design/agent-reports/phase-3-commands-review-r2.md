# Phase 3 Commands Review — r2

**Date:** 2026-05-04
**Commits under review:** `0f4943c` (r1 fixup), parents `e92b3a9` (Phase 3 feature) + `fbf401a` (r1 report)
**Reviewer:** opus phase-review

## Verdict

0 critical / 0 important / 3 low / 0 nits

✅ **Phase 3 r2 terminator reached — cleared to proceed to Phase 4.**

## Critical

(none)

## Important

(none)

## Low / Nit (unchanged from r1; deferred to design/FOLLOWUPS.md)

- **L-1:** `friendly_mk_codec` `MixedCase` message `"mk1 mixed case in input string"` vs SPEC §6.4.4 `"mixed case in mk1 input string"`. Phase 5 fixtures will pin.
- **L-2:** `bundle.rs` calls `chunk_5char` directly for mk1; `chunk_mk1` alias unused but functionally identical.
- **L-3:** Depth advisory emitted before watch-only account-index hazard warning in `bundle_watch_only`. Sub-order unspecified by SPEC.

## Confirmed

- C-1 fully resolved: `run_watch_only` delegates to testable `watch_only_checks` helper that emits exactly 9 checks in SPEC §5.4 order.
- `ms1_entropy_match` and `mk1_path_match` always skipped per SPEC §2.2.2.
- mk1_*_match cascade-skip on mk1 decode fail; md1_*_match + stub_linkage cascade-skip on md1 decode fail.
- `md1_xpub_match` correctly uses `xpub_to_65(supplied_xpub)` against `md1.tlv.pubkeys[0].1`.
- 5 new unit tests with `assert_spec_order` helper enforcing count=9 + name order.
- `run_full` order unchanged from r1 (9 checks via mk1-decode-success or mk1-decode-fail branches + helpers).
- No regressions; carry-over Lows unchanged.

## Smoke checks

- `cargo test -p mnemonic-toolkit`: 52 passed (47 prior + 5 new watch-only checks tests)
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`: clean
- `cargo fmt --check -p mnemonic-toolkit`: clean
