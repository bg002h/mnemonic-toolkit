# Phase 4.3 review — r2

Date: 2026-05-11
Reviewer: feature-dev:code-reviewer (r2)

## Summary

- Chapter: 0C / 0I / 0L / 0N
- Cross-cutting: 0C / 0I / 0L / 0N

Total: 0C / 0I / 0L / 0N

## r1 fix-verification

**C-1 — line 67 decode string:** CONFIRMED. `53-ms-codec-api.md:67` now reads:

```rust
let (tag, payload) = decode("ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f")?;
```

Byte-identical to `transcripts/ms-codec-api-roundtrip.out:1` substring after `encoded: `.

**I-1 — line 218 discriminate rule set:** CONFIRMED. Line 218 reads:

> 3. Calls `envelope::discriminate` to enforce HRP, threshold, share-index, tag-alphabet, and reserved-prefix byte invariants (SPEC §4 rules 2–5, 8).

Source confirmation from `mnemonic-secret/crates/ms-codec/src/envelope.rs`:
- Lines 94-109: rules 2, 3, 4 (HRP, threshold, share-index).
- Lines 111-115: rule 5 (tag-alphabet via `Tag::try_new`).
- Lines 124-129: rule 8 (reserved-prefix byte).

Both four-item English list and rule-number range are correct.

## Sweep for other transposed wire strings

Grep for `ms10` across `53-ms-codec-api.md` returns exactly one hit (line 67, now corrected). All other `decode()` / `encode()` call sites in the chapter use variable names (`card_str`, `suspect_str`) rather than inline literal strings. No further transposition exposure.

## New findings

None.

## Verdict

- [x] 0 C / 0 I — Phase 4.3 ready to close (move to Phase 4.4)
- [ ] Findings present — iterate r3

Both r1 fixes byte-exact against source. Single occurrence of `ms10` literal in chapter (now correct). No new issues.
