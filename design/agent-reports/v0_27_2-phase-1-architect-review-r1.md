# v0.27.2 Phase 1 architect review — R1

**Reviewer:** opus feature-dev:code-reviewer
**Branch:** `release/v0.27.2` at fold commit `9376d41`
**Date:** 2026-05-19
**Verdict:** GREEN (0 Critical / 0 Important / 0 Minor)

## Scope reviewed

R0 fold-pass: verify the 3 R0 findings (I1 + M1 + M2) were correctly applied and surface any regressions introduced by the folds.

## Fold verification

### I1 — `alloc(layout)` → `alloc_zeroed(layout)` (CONFIRMED clean)

- **File:** `crates/mnemonic-toolkit/tests/mlock_unit.rs:25-44`
- **Verification:**
  - Line 26 import: `use std::alloc::{alloc_zeroed, dealloc, Layout};` — `alloc` removed, `alloc_zeroed` added, `dealloc` + `Layout` retained.
  - Line 35: `let ptr = alloc_zeroed(layout);` — call site updated.
  - Lines 29-32 SAFETY comment: now states "alloc_zeroed returns initialized memory (zero-filled), so the &[u8] slice is well-formed" — addresses the UB-on-uninitialized-borrow concern precisely.
  - `dealloc(ptr, layout)` at line 42 unchanged (correct — alloc_zeroed allocations are deallocated via `dealloc`, not `dealloc_zeroed`).
  - Grep for `\balloc\b` in file: only the import line + the string literal `"alloc failed"` at line 36 — no stray `alloc(layout)` calls remain.
- **Status:** RESOLVED.

### M2 — Docstring precision for per-target JSON envelope fields (CONFIRMED clean)

- **File:** `crates/mnemonic-toolkit/src/error.rs:242-245`
- **New phrasing:** "The per-target JSON envelope fields `scanned_external` / `scanned_internal` (on `AddressResultJson::NoMatch` entries inside `AddressOfXpubResult.results`) report unique child-addresses derived per-target".
- **Cross-check against source ground truth:**
  - `address_of_xpub.rs:77-91`: `AddressResultJson` is an untagged enum with `NoMatch { target, result, scanned_external: u32, scanned_internal: u32 }` — fields confirmed.
  - `address_of_xpub.rs:95-102`: `AddressOfXpubResult { pub results: Vec<AddressResultJson>, ... }` — top-level shape confirmed.
  - Phrasing reads cleanly in surrounding docstring context (lines 230-249).
- **Status:** RESOLVED.

### M1 — Plan Task 4.2 blockquote note (CONFIRMED clean + math verified)

- **File:** `design/IMPLEMENTATION_PLAN_v0_27_2_followup_cleanup.md:1538`
- **Blockquote:** "Phase 1 + 2 eagerly flipped 6 of these 7 entries per-task during execution (per the `feedback-per-phase-agents-forget-followup-status-flip` anti-flake guard). Only 1 entry — `gui-workflow-trigger-include-release-branches` — should still show `Status: open` at Phase 4 entry. Verify each entry's current Status BEFORE editing; do NOT re-flip already-resolved entries (the existing SHA citations are correct)."
- **Positioning:** Above the 7-slug list at line 1540, below the Step 1 header at line 1536. Correct placement — agent reads the warning before the slug enumeration.
- **Count math verified against live FOLLOWUPS.md:**
  - Phase 1.1 → `error-rs-canonical-ordering-doc` resolved 79734f8 (FOLLOWUPS.md:2423)
  - Phase 1.2 → `compare-cost-agent-reports-back-fill` resolved 08cf0a9 (FOLLOWUPS.md:2435)
  - Phase 1.3 → `mlock-g1-1-test-page-alignment-luck` resolved c9ead62 (FOLLOWUPS.md:153)
  - Phase 1.4 → `gui-schema-arm-drop-detector` resolved 93bf3ff (FOLLOWUPS.md:2408)
  - Phase 1.5 → `xpub-search-address-of-xpub-searched-count-semantic` resolved 8304f5b (FOLLOWUPS.md:201)
  - Phase 2 → `pr-26-import-provenance-enum-internal-refactor` (FOLLOWUPS.md:108 currently `open`; Phase 2 will flip)
  - Phase 4 → `gui-workflow-trigger-include-release-branches` (FOLLOWUPS.md:2447 `open`; Phase 4 flips after Phase 3 ships the sibling workflow edit)
  - 5 + 1 = 6 flipped by Phase 4 entry; 1 remaining — matches blockquote claim exactly.
- **Status:** RESOLVED.

## Fresh-eyes scan

No new issues introduced by the folds.

- `alloc_zeroed` import replacement: clean substitution. No other callers of `alloc` in the file. `Layout` + `dealloc` retained correctly.
- M2 phrasing reads naturally in the surrounding docstring; no awkward sentence boundary.
- M1 blockquote's "Phase 1 + 2" attribution is precise (Phase 1 ships 5 flips at R0 time; Phase 2 will ship 1 more; Phase 4 handles the 7th after Phase 3's sibling-repo fix). The blockquote correctly does NOT attribute the Phase 4 flip to Phase 1+2.

## Verdict

**GREEN** — all 3 R0 findings cleanly folded. No new issues surfaced. Phase 2 dispatch UNBLOCKED.
