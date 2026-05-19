# v0.27.2 Phase 1 architect review — R0

**Reviewer:** opus feature-dev:code-reviewer
**Branch:** `release/v0.27.2` at Phase 1 close (post-`6c959cb`)
**Date:** 2026-05-19
**Verdict:** YELLOW (0 Critical / 1 Important / 2 Minor)

## Scope reviewed

Phase 1 batch of v0.27.2 — 5 task commits (1.1-1.5) + 4 Status SHA back-fill commits:

| Commit | Task | Closes FOLLOWUP |
|---|---|---|
| `79734f8` | 1.1 (CLAUDE.md alphabetical-ordering Convention + companion FOLLOWUP for retroactive sort) | `error-rs-canonical-ordering-doc` |
| `08cf0a9` | 1.2 (CLAUDE.md persist-architect-reviews-verbatim Convention) | `compare-cost-agent-reports-back-fill` |
| `04a3949` | (interim Status flip commit for 1.1 + 1.2) | — |
| `c9ead62` | 1.3 (mlock_unit g1_1 page-aligned fix) | `mlock-g1-1-test-page-alignment-luck` |
| `7e6c70c` | (Status SHA back-fill for 1.3) | — |
| `93bf3ff` | 1.4 (gui-schema arm-count regression test pinned at 6) | `gui-schema-arm-drop-detector` |
| `5f88ff6` | (Status SHA back-fill for 1.4) | — |
| `8304f5b` | 1.5 (xpub-search searched-count doc clarification + n_targets factor restored) | `xpub-search-address-of-xpub-searched-count-semantic` |
| `6c959cb` | (Status SHA back-fill for 1.5) | — |

Total Phase 1 diff: ~85 LOC across `CLAUDE.md`, `design/FOLLOWUPS.md`, `src/error.rs`, `src/cmd/xpub_search/address_of_xpub.rs`, `tests/mlock_unit.rs`, `tests/cli_gui_schema_conditional_rules.rs`.

## Findings

**Critical:** none.

**Important:**

### I1. (confidence 80) Test creates `&[u8]` slice from uninitialized memory — potential undefined behavior.

- **File:** `crates/mnemonic-toolkit/tests/mlock_unit.rs:33-35`
- **Detail:** The new `g1_1_single_page_pin_has_page_count_one` test calls `alloc(layout)` followed immediately by `std::slice::from_raw_parts(ptr, 64)` without initializing the buffer. Constructing a `&[u8]` reference to uninitialized memory is UB per Rust's reference-validity rules (even though `pin_pages_for` only reads `buf.as_ptr()` + `buf.len()` and never dereferences the bytes). Miri will flag this; std-test runs likely pass today.
- **Fix:** Add `std::ptr::write_bytes(ptr, 0u8, 64);` (or use `alloc_zeroed`) before the `from_raw_parts` call. Update the SAFETY comment to reference initialization.
- **Suggested patch:**
  ```rust
  let ptr = alloc(layout);
  assert!(!ptr.is_null(), "alloc failed");
  std::ptr::write_bytes(ptr, 0u8, 64);  // initialize before borrowing as &[u8]
  let slice = std::slice::from_raw_parts(ptr, 64);
  ```
  Or: use `std::alloc::alloc_zeroed(layout)`.
- **Importance rationale:** The previous test used `vec![0xAAu8; 64]` (initialized). The Phase 1.3 fix preserves the page-alignment invariant but regresses on initialization. Plan/spec did not specify zero-init, but UB-free test code is the project norm.

**Minor:**

### M1. (confidence 70) Plan Task 4.2 still says "flip 7 entries" but 5 are already flipped by Phase 1.

- **File:** `design/IMPLEMENTATION_PLAN_v0_27_2_followup_cleanup.md:1531-1580` (Task 4.2)
- **Detail:** Phase 1's eager-flip pattern (good — guards against `feedback-per-phase-agents-forget-followup-status-flip`) leaves only 2 entries for Phase 4 Task 4.2 to flip: `pr-26-import-provenance-enum-internal-refactor` (Phase 2) and `gui-workflow-trigger-include-release-branches` (Phase 3 sibling). Task 4.2 Step 1 lists all 7 slugs to update. The find-replace will be a no-op for the 5 already-flipped entries, but the agent operator may mis-read this as "5 entries are missing flips" and accidentally re-flip with the WRONG SHA.
- **Fix:** Before dispatching Phase 4, amend Task 4.2 Step 1 to list only the 2 remaining entries OR add a note: "5 of these 7 were already flipped in Phase 1 commits — verify and skip; only Phase 2/3 entries need new SHA fills."
- Not blocking Phase 2 dispatch; concerns Phase 4 only.

### M2. (confidence 60) Docstring says `scanned_external` / `scanned_internal` are "in `AddressOfXpubResult`" — strictly they are fields on the `AddressResultJson::NoMatch` per-target variant inside `AddressOfXpubResult.results`.

- **File:** `crates/mnemonic-toolkit/src/error.rs:242-244`
- **Detail:** The docstring on `XpubSearchNoMatch` says "The per-target JSON envelope fields `scanned_external` / `scanned_internal` (in `AddressOfXpubResult`) report unique child-addresses derived per-target". The fields are declared on `AddressResultJson::NoMatch` at `crates/mnemonic-toolkit/src/cmd/xpub_search/address_of_xpub.rs:85-91`. `AddressOfXpubResult` carries a `results: Vec<AddressResultJson>` field at `address_of_xpub.rs:96`. The phrasing "in `AddressOfXpubResult`" is loose; "in `AddressOfXpubResult.results[].NoMatch`" or "in the per-target `AddressResultJson::NoMatch` entries of `AddressOfXpubResult.results`" would be more precise.
- **Fix:** minor wording tweak; not blocking.

## Per-commit verification

- **79734f8 (Task 1.1):** `CLAUDE.md:34` correctly forward-looking ("New `enum ToolkitError` variants + new exhaustive `match self` blocks use alphabetical-by-variant-name ordering."); explicitly notes "Pre-v0.27.2 variants ... are not yet sorted — retroactive sort tracked as `error-rs-retroactive-alphabetical-sort`". Plan-amendment correctly applied; not misleading. New FOLLOWUP entry at `FOLLOWUPS.md:2515-2522` well-formed (`Status: open`, `Tier: v0.28+`).
- **08cf0a9 (Task 1.2):** `CLAUDE.md:35` correctly persists the convention with path template `design/agent-reports/<cycle>-phase-N-<round>-review.md`.
- **c9ead62 (Task 1.3):** Test rewrite matches plan Step 3 verbatim, modulo the I1 UB issue above.
- **93bf3ff (Task 1.4):** New cell `dispatcher_arm_count_matches_pinned_constant` at `cli_gui_schema_conditional_rules.rs:525-545` correctly uses `EXPECTED_ARM_COUNT = 6` matching the 6 arms at `gui_schema.rs:338-343` (`bundle`, `verify-bundle`, `export-wallet`, `convert`, `derive-child`, `compare-cost`). Regex `r#"(?m)^\s+"[a-z-]+" => [a-z_]+_conditional_rules\(\),$"#` matches those 6 lines and excludes the `_ =>` fallback arm + the standalone fn definitions.
- **8304f5b (Task 1.5):** Docstring extension at `error.rs:230-244` adds the missing `n_targets` factor for address mode + the `candidate-comparisons performed` framing; inline comment at `address_of_xpub.rs:288-292` matches plan Step 3 verbatim. The `total_scanned` computation (`address_of_xpub.rs:293-296`) is byte-for-byte identical to pre-Phase-1.5 — DOC-ONLY change confirmed; no semantic delta. Existing JSON envelope shape (`AddressResultJson::NoMatch.scanned_external/internal`) unchanged.

## FOLLOWUPS Status flip discipline

All 5 entries show `Status: resolved (<sha>; v0.27.2 Phase 1.X)`:
- `error-rs-canonical-ordering-doc` → resolved (79734f8; v0.27.2 Phase 1.1) ✓
- `compare-cost-agent-reports-back-fill` → resolved (08cf0a9; v0.27.2 Phase 1.2) ✓
- `mlock-g1-1-test-page-alignment-luck` → resolved (c9ead62; v0.27.2 Phase 1.3) ✓
- `gui-schema-arm-drop-detector` → resolved (93bf3ff; v0.27.2 Phase 1.4) ✓
- `xpub-search-address-of-xpub-searched-count-semantic` → resolved (`8304f5b`; v0.27.2 Phase 1.5) ✓

SHAs match the brief's commit table. No collateral damage observed on other entries. New entry `error-rs-retroactive-alphabetical-sort` is well-formed (Status: open, Tier: v0.28+).

## Production code regression check

- `error.rs`: doc-only (lines 230-244 docstring extension; struct definition lines 245-248 unchanged; no new variants).
- `address_of_xpub.rs`: doc-only inline comment at lines 288-292; `total_scanned` computation byte-identical.
- `mlock_unit.rs`: test-only; new cell body uses public APIs `mlock::page_size_for_test()` + `mlock::pin_pages_for()`.
- `cli_gui_schema_conditional_rules.rs`: test-only addition.
- `CLAUDE.md`: doc-only additions to Conventions section.
- `FOLLOWUPS.md`: Status-line edits + 1 new entry append.

No `pub` API changes. No CLI flag delta. Confirmed.

## Cross-task interaction

The 5 Phase 1 commits are disjoint (different files except FOLLOWUPS.md which gets append-only edits). The Status flip pattern is the documented anti-flake guard per `feedback-per-phase-agents-forget-followup-status-flip` and does not conflict with Phase 4 Task 4.2 (no-op re-flips). Phase 4 Task 4.2 wording could be tightened (see M1) but is not blocking.

## Verdict

**YELLOW** — 1 Important (I1: test UB on uninitialized buffer slice). 0 Critical.

Phase 2 dispatch is **conditionally unblocked**: recommend folding I1 (one-line `write_bytes` add) before dispatching Phase 2.
