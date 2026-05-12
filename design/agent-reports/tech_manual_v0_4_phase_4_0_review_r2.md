# Phase 4.0 harvest review — r2

Date: 2026-05-11
Reviewer: feature-dev:code-reviewer (r2)

## Summary

- md-codec: 1C / 0I / 0L / 0N
- mk-codec: 0C / 0I / 0L / 0N
- ms-codec: 0C / 0I / 0L / 0N
- mnemonic-toolkit: 0C / 0I / 0L / 0N
- Cross-cutting: 0C / 0I / 0L / 0N

Total: 1C / 0I / 0L / 0N

## r1 fix-verification

- **md-codec C1**: ✓ applied at lines 259 and 531 ("43 variants" in both). Tag enum count at line 365 unchanged ("36 variants" — correct; Tag genuinely has 36 variants, verified by reading `tag.rs:15-89`).
- **md-codec L1**: ✓ applied at line 42 — now reads "20 public modules (19 unconditional; `to_miniscript` requires `derive` feature) + one private (`mod bch`)".
- **mk-codec C1**: ✓ applied at lines 93 and 324 — both now read "22 variants". Verified against `mk-codec/src/error.rs`: 22 variants from `InvalidHrp` through `CardPayloadTooLarge`.
- **mk-codec L1**: ✓ Notes section now includes the stale-`"md1"`-string item at line 373, citing both `bch.rs:575` and `bch.rs:603`.
- **mnemonic-toolkit I1**: ✓ Notes item at line 469 revised. `synthesize::check_key_vector_distinctness` attribution removed. Sole `pub` function correctly attributed as `parse_descriptor::check_key_vector_distinctness` at `parse_descriptor.rs:1104`. Explicit verification sentence added: "Verified: `src/synthesize.rs` contains no function named `check_key_vector_distinctness`".

## New findings

### md-codec — Critical

#### C1 (new) — Error taxonomy closing parenthetical still says "36"; r1 missed a third occurrence

- **Location:** harvest line 579 — `(Variant count = 36 from the \`pub enum Error { ... }\` block, lines 20-392, excluding test-only references.)`
- **Issue:** r1 identified two occurrences of "36 variants" for `pub enum Error` (lines 259 and 531) and both were corrected to 43. A third occurrence exists at line 579 — a closing parenthetical immediately below the taxonomy table — and was not touched. With the r1 fix landed, line 531's preamble now reads "43 variants" and the table below has 43 rows, but the closing parenthetical at line 579 says "Variant count = 36" — directly contradictory to the preamble two screens up.
- **Evidence:** Line 531: `pub enum Error from src/error.rs — 43 variants (line 19 to 392)`. Line 579: `(Variant count = 36 from the pub enum Error { ... } block, lines 20-392, excluding test-only references.)`. The table itself (lines 533-577) has 43 data rows.
- **Recommendation:** Change "36" to "43" at line 579 so all three count citations are consistent.

## Count integrity verification

| Claim | Ground-truth source | Result |
|---|---|---|
| md-codec `Error`: 43 variants | `crates/md-codec/src/error.rs:20-392` direct read | ✓ 43 confirmed |
| md-codec `Tag`: 36 variants | `crates/md-codec/src/tag.rs:15-89` direct read | ✓ 36 confirmed |
| md-codec public modules: 20 | `crates/md-codec/src/lib.rs:17-37` direct read (19 `pub mod` + 1 `#[cfg(feature="derive")] pub mod`) | ✓ 20 confirmed |
| mk-codec `Error`: 22 variants | `crates/mk-codec/src/error.rs:20-162` direct read | ✓ 22 confirmed |
| ms-codec `Error`: 10 variants | r1 confirmed; harvest unchanged | ✓ no change |
| mnemonic-toolkit `ToolkitError`: 25 variants | harvest table enumeration | ✓ consistent with harvest |

## Verdict

- [ ] 0 C / 0 I — Phase 4.0 ready to close (move to Phase 4.1)
- [x] Findings present — iterate r3

One Critical: md-codec harvest line 579 parenthetical still reads `Variant count = 36` after the r1 fix corrected lines 259 and 531 to 43. Mechanical one-word fix. All other r1 fixes correctly applied with no regressions, and the cross-source count audit confirms every other count is accurate.
