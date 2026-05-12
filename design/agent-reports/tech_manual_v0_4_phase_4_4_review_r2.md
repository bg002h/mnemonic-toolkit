# Phase 4.4 review — r2

Date: 2026-05-11
Reviewer: feature-dev:code-reviewer (r2)

## Summary

- Chapter: 0C / 1I / 0L / 0N
- All other deliverables: 0C / 0I / 0L / 0N

Total: 0C / 1I / 0L / 0N

## r1 fix-verification

**C-1 (`ExportWalletMissingFields`, 26-variant count):** PARTIAL.
- §V.4.4 preamble (line 145): correctly reads "26-row table" + v0.8.1 phase-0 reservation. ✓
- §V.4.4 closing count (line 176): correctly reads "(Variant count = 26; row count = 26.)". ✓
- §V.4.4 table row for `ExportWalletMissingFields` (line 171): present. ✓
- **§V.4.3.1 module table row (line 25): MISSED — still says "25 variants".**

HEAD `error.rs` variant count confirmed at **26** via direct enumeration of `:10-127`.

**I-1 (`wallet_export` row):** FULLY FIXED.
- Path: `src/wallet_export/mod.rs` correct (directory exists at that path). ✓
- `taproot_multisig_unsupported_message`: `pub fn` at `mod.rs:47`. ✓
- `build_missing_fields_refusal`: `pub(crate) fn` at `mod.rs:267` (r1's note that this is `pub` was inaccurate; reality is `pub(crate)`, but the chapter's binary-only framing at line 17 explicitly covers `pub(crate)` module-references — inclusion is appropriate). ✓
- Description updated to mention missing-field refusal builder. ✓

## New findings

### I-1r2 — §V.4.3.1 module table `ToolkitError` row still says "25 variants"

**Location:** `54-mnemonic-toolkit-api.md:25`

**Evidence.** Line 25 reads `| ToolkitError | pub | #[non_exhaustive]; 25 variants; error.rs:10 |`. The C-1 fix updated §V.4.4 (lines 145, 171, 176) but did not propagate to this §V.4.3.1 summary row.

**Fix.** Change `25 variants` to `26 variants`.

## Regression check

None. The stale count is prose embedded in a Markdown table cell; markdownlint / cspell / lychee don't check variant counts. `make lint` 6/6 unaffected.

## Verdict

- [ ] 0 C / 0 I — Phase 4.4 ready to close
- [x] Findings present — iterate r3

One residual I: line 25 needs `25` → `26`. All other r1 fixes correctly applied; HEAD-truth count (26) confirmed independently.
