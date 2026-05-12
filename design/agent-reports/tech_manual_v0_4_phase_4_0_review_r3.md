# Phase 4.0 harvest review — r3

Date: 2026-05-11
Reviewer: feature-dev:code-reviewer (r3)

## Summary

- md-codec: 0C / 0I / 0L / 0N
- mk-codec: 0C / 0I / 0L / 0N
- ms-codec: 0C / 0I / 0L / 0N
- mnemonic-toolkit: 0C / 0I / 0L / 0N
- Cross-cutting: 0C / 0I / 0L / 0N

Total: 0C / 0I / 0L / 0N

## r2 fix-verification

- **md-codec C1 (r2)**: CONFIRMED. Line 579 of `api-harvest-md-codec.md` reads `(Variant count = 43 from the pub enum Error { ... } block, lines 20-392, excluding test-only references.)`. The remaining "36" occurrences in the file (lines 361 and 365) both refer to the `Tag` enum, which genuinely has 36 variants — confirmed against `crates/md-codec/src/tag.rs:14-89`. Zero "36" references remain for `pub enum Error`.

## r1 fix regression check

All r1 fixes confirmed still in place, no regressions:

- **md-codec C1 (r1)** — lines 259 and 531: both read "43 variants". ✓
- **md-codec L1 (r1)** — line 42: reads "20 public modules (19 unconditional; `to_miniscript` requires `derive` feature) + one private (`mod bch`)". ✓
- **mk-codec C1 (r1)** — line 93: "22 variants"; line 324: "22 variants on `Error`". ✓
- **mk-codec L1 (r1)** — line 373: Notes item flagging stale `"md1"` strings in `bch.rs:575` and `bch.rs:603` present. ✓
- **mnemonic-toolkit I1 (r1)** — line 469: `synthesize::check_key_vector_distinctness` attribution removed; sole `pub` function correctly attributed to `parse_descriptor::check_key_vector_distinctness` at `parse_descriptor.rs:1104`; explicit verification sentence present. ✓

## New findings

None.

### Public-surface cross-check

`grep '^pub ' crates/<crate>/src/` file counts vs harvest coverage:

- md-codec: 20 files with `pub` items (19 module source files + lib.rs with `pub mod` declarations) — all 20 modules present in harvest table. ✓
- mk-codec: 12 files with `pub (fn|struct|enum|type|const)` items — all covered under the 6 `pub mod` groups in harvest. ✓
- ms-codec: 7 files with `pub` items — all covered (7 public modules in harvest; 8th is private `mod envelope`). ✓
- mnemonic-toolkit: no `[lib]` — harvest correctly scopes to internal module surface. ✓

No significant public items missing from any harvest.

## Final cross-source count audit

| Claim | Ground-truth source | Result |
|---|---|---|
| md-codec `Error`: 43 variants | `error.rs:20-392` (r1 direct count; r2 confirmed) | ✓ |
| md-codec `Tag`: 36 variants | `tag.rs:14-89` (r2 confirmed) | ✓ |
| md-codec public modules: 20 | `lib.rs:15-37` (r1+r2 confirmed; r3 file-count corroborates) | ✓ |
| mk-codec `Error`: 22 variants | `error.rs:20-162` (r1+r2 confirmed) | ✓ |
| ms-codec `Error`: 10 variants | `error.rs:9-64` (r1 confirmed; unchanged) | ✓ |
| mnemonic-toolkit `ToolkitError`: 25 variants | taxonomy table rows (25 counted) | ✓ |

## Verdict

- [x] 0 C / 0 I — Phase 4.0 ready to close (move to Phase 4.1)
- [ ] Findings present — iterate r4
