# v0.31.3 plan-doc R1 review (fold-verification)

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** R1
**Plan under review:** `design/PLAN_mnemonic_toolkit_v0_31_3.md` (renamed from `v0_32_0.md` post-R0 SemVer fold)
**Date:** 2026-05-21
**Source SHA:** `7e50902` (master HEAD; no new commits since R0)

## Verdict

**GREEN.** All 7 R0 findings (3 Critical + 2 Important + 2 Minor) folded correctly. No new Critical/Important issues introduced.

## Fold verification

| Finding | Status | Fold location in plan-doc |
|---|---|---|
| **C1** Seedqr at position 1 (after Phrase, before Entropy) | ✓ | Code block §Task 1 Step 1; recon at "P0 STRICT-GATE recon"; risk register first bullet |
| **C2** Branch placement AFTER Phrase, BEFORE Xpub | ✓ | §Task 2 Step 3 + risk register |
| **C3** Promote `map_seedqr_error` to `pub(crate)` | ✓ | §Task 2 Step 1 + Tech Stack + risk register |
| **I1** SemVer → PATCH v0.31.3 | ✓ | Goal + Tech Stack + version-bump table; rationale rewritten coherently |
| **I2** `bundle_seedqr_slot_double_stdin_refused` cell | ✓ | §Test files modified |
| **M1** Byte-equal on both 12-word AND 24-word happy-paths | ✓ | §Test files modified |
| **M2** File `bundle-slot-help-text-master-xpub-drift` FOLLOWUP | ✓ | §Release tooling + Phase 6 Step 2 |

## Source citation re-verification

- `slot_input.rs:17-32` (enum declaration) — verified at HEAD.
- `slot_input.rs:60-62` (is_secret_bearing) — verified.
- `slot_input.rs:145-149` (from_token error msg) — verified.
- `slot_input.rs:274-278` (exempted_v0_19_0 matcher) — verified.
- `slot_input.rs:313-330` (is_legal_set) — verified.
- Position-math proof: `[Seedqr(1), Path(6)]` and `[Seedqr(1), Fingerprint(5), Path(6)]` both ascending-sorted with Seedqr at position 1. ✓

## Phase 6 FOLLOWUP filings list

Complete: closes 1 (`seedqr-bundle-slot-integration`) + files 2 (`bundle-slot-help-text-master-xpub-drift`, `gui-seedqr-slot-subkey-help-mirror`).

## Recommendation

**Proceed to Phase 2 dispatch.**
