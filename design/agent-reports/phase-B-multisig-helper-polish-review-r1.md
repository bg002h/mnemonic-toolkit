# Phase B — multisig helper polish (5 items) — code-reviewer r1 (2026-05-06)

## Findings

### Critical
None.

### Important
None.

### Low / Nit
None blocking. Implementation review:

**B.1 — full-mode helper unit test added:** `helper_multisig_full_emits_3plus6n_checks_in_spec_order` constructs a multisig Bundle via watch-only synthesis (mk1+md1) then manually populates `expected.ms1` with non-empty strings derived from `synthesize_full(seed_a/seed_b)`. Asserts case 2 (full-supplied-decodes-Ok) on both slots.

**B.2 — positional-fallback condition refactored** to a clean `match` (lines ~1057-1060). Identical semantics; reads cleanly.

**B.3 — multiset md1_xpub_match shipped.** `exp_sorted == act_sorted` after `sort()`; forensic fields use unsorted comma-joined hex on mismatch so user sees slot ordering.

**B.4 — MappingFailure enum shipped** with precedence `XpubNotInPolicy > DecodeFailed > NotSupplied`. Replaces `Vec<Option<&KeyCard>>` with `Vec<Result<&KeyCard, MappingFailure>>`. Two-pass mapping loop: first pass places successful decodes; xpub-mismatch promotes a NotSupplied slot to XpubNotInPolicy. Second pass places DecodeFailed.

**B.5 — four-case ms1 emission shipped.** Case 1 (watch-only) unchanged; case 2 (substantive) unchanged; case 3 (decode-Err) cascade-skip preserved; case 4 (full-mode supplied-absent) NEW — both `ms1_decode[i]` and `ms1_entropy_match[i]` now `passed: false`. Byte-exact decode_error per SPEC §5.7. New `helper_multisig_missing_ms1_emits_passed_false_per_spec_5_7_case_4` unit test pins the byte-exact decode_error string for the new failure path.

## Test status

241 lib + 22 integration suites pass (was 239+22; +2 from B.1+B.5 unit tests).

## Outcome

Phase B APPROVED. Proceed to Phase C.
