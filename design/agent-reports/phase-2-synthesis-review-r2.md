# Phase 2 Synthesis Review — r2

**Date:** 2026-05-04
**Commits under review:** `baa1cf9` (r1 fixup), parent `38165fe` (Phase 2 feature)
**Reviewer:** opus phase-review

## Verdict

0 critical / 0 important / 6 low / nits

✅ **Phase 2 r2 terminator reached — cleared to proceed to Phase 3.**

## Critical

(none)

## Important

(none)

## Low / Nit (unchanged from r1; deferred to design/FOLLOWUPS.md)

- **L-1:** SPEC §5.5 omits `NetworkMismatch` / `FutureFormat` from `kind` table.
- **L-2:** `chunk_mk1` fallback — corroborated by Phase 2 chunk counts (mk1=2, md1=3 for BIP-84 single-sig).
- **L-3:** md1 hyphens vs mk1 spaces in chunked form.
- **L-4:** `debug_assert_eq!(&card.policy_id_stubs[0], &stub)` is tautological; release elides it.
- **L-5:** Plan source stale 24-word fingerprint `73c5da0a` (correct: `5436d724`); patched in handoff during Task 2.1.
- **L-6:** ms1 not round-tripped in 16-cell test (Phase 5 fixtures cover this).

## Confirmed

- I-1 fix landed at `baa1cf9`: test renamed to `derive_passphrase_empty_string_is_stable` with comment clarifying CLI-layer enforcement is Phase 3.
- No other code changes in fixup.
- 44 tests passing; clippy + fmt clean.
- r1 report committed alongside.
