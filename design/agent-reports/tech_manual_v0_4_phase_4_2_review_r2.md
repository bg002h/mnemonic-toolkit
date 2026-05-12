# Phase 4.2 review — r2

Date: 2026-05-11
Reviewer: feature-dev:code-reviewer (r2)

## Summary

- Chapter: 0C / 0I / 0L / 0N
- Cross-cutting: 0C / 0I / 0L / 0N

Total: 0C / 0I / 0L / 0N

## r1 fix-verification

**I-1 (`BchCode`, line 203):** CONFIRMED. Chapter line 203 reads:

```
| `BchCode`\index{BchCode} | `pub enum BchCode { Regular, Long }` (`Hash` + standard derives; exhaustive) | which BCH code variant a string uses | `bch.rs:27` |
```

No `#[non_exhaustive]` claim. Source `bch.rs:26-32` confirms only `#[derive(...)]`.

**I-2 (`CaseStatus`, line 204):** CONFIRMED. Chapter line 204 reads:

```
| `CaseStatus`\index{CaseStatus} | `pub enum CaseStatus { Lower, Upper, Mixed }` (exhaustive) | case-check result | `bch.rs:155` |
```

Source `bch.rs:154-162` confirms only `#[derive(...)]`.

**§V.2.7 enumeration (line 368):** CONFIRMED. Reads:

> `KeyCard`, `Error`, `StringLayerHeader`, `CorrectionResult`, `DecodedString`, `ChunkFragment` ARE marked `#[non_exhaustive]`. `BchCode`, `CaseStatus`, `BytecodeHeader`, and `XpubCompact` are NOT.

Exact match for the r1-prescribed text.

## Audit of the 6-marked + 4-unmarked claim

Comprehensive grep for `#[non_exhaustive]` across `crates/mk-codec/src/**/*.rs` yields exactly 6 hits. Each verified:

| Type | Chapter claim | Source | Match |
|---|---|---|---|
| `KeyCard` | marked | `key_card.rs:22` | ✓ |
| `Error` | marked | `error.rs:18` | ✓ |
| `StringLayerHeader` | marked | `string_layer/header.rs:33` | ✓ |
| `CorrectionResult` | marked | `string_layer/bch.rs:362` | ✓ |
| `DecodedString` | marked | `string_layer/bch.rs:568` | ✓ |
| `ChunkFragment` | marked | `string_layer/chunk.rs:25` | ✓ |
| `BchCode` | NOT | `string_layer/bch.rs:26` (no attribute) | ✓ |
| `CaseStatus` | NOT | `string_layer/bch.rs:154` (no attribute) | ✓ |
| `BytecodeHeader` | NOT | `bytecode/header.rs:29` (no attribute) | ✓ |
| `XpubCompact` | NOT | `bytecode/xpub_compact.rs:31` (no attribute) | ✓ |

10/10 correct.

## Table column-count regression check

Lines 201-208 form a 4-column table; separator row `|---|---|---|---|` matches; all rows have 4 cells. No row-count breakage from the two edits.

## `make lint` status

Both fixes were prose-only edits within existing table cells / paragraph text. No structural perturbations. Lint 6/6 green confirmed by the implementer between r1 and r2.

## New findings

None. Re-scan of the chapter with focus on `#[non_exhaustive]` annotations across the chapter (10 instances) plus the inline NOT-marked notes at lines 131 (`BytecodeHeader`) and 158 (`XpubCompact`) — all consistent with source.

## Verdict

- [x] 0 C / 0 I — Phase 4.2 ready to close (move to Phase 4.3)
- [ ] Findings present — iterate r3
