# Phase P0A — architect R2 review

**Reviewer:** Opus 4.7 via feature-dev:code-architect
**Branch:** `v0.28.0/p0a-spec-scaffolding`
**Commit under review:** `e8d0b07` (R2 fold commit; preceded by `87cb7e6` R1 fold, `aa3a537` P0A scope, `12c248f` cycle-followups)
**Source SHA verified against:** working-tree at `e8d0b07`
**Previous reviews:**
- [`v0_28_0-P0A-r0-review.md`](v0_28_0-P0A-r0-review.md) (YELLOW, 4 Important)
- [`v0_28_0-P0A-r1-review.md`](v0_28_0-P0A-r1-review.md) (YELLOW, 2 Important: N1 citation drift recurrence, N2 count contradiction)

---

## Fold verification (R1 → R2)

| R1 finding | R2 fold applied? | Verified at source |
|---|---|---|
| **N1** `bitcoin_core.rs:74` → `:81` citation drift recurrence (SPEC §6.1.1 + §12 + doc-comment range `:59-72` → `:59-80`) | YES — all three drift sites updated | SPEC §6.1.1 line 94 cites `:81` with `:59-80`; SPEC §12 line 542 cites `:81` with `:59-80`; both annotated with audit-trail parenthetical |
| **N2** SPEC §A line 27 "10 new format markers" → "8" | YES — §A line 27 now reads "expanded with 8 new format markers (5 originals + 8 additions = 13 entries; R1 I3/I4 folds removed 2 prior candidates)" | Consistent with §6.1.1 body's "5 to 13" claim |

Both R1 Importants folded cleanly. No new line-number drift introduced by R2 SPEC-text-only edits.

## Source-grep re-verification at `e8d0b07`

| Citation | Verified? | Notes |
|---|---|---|
| `wallet_import/bitcoin_core.rs:81` `const VENDOR_MARKER_KEYS:` declaration | YES | `81:const VENDOR_MARKER_KEYS: &[&str] = &[` |
| Doc-comment range `:59-80` | YES | line 59 opens; line 80 closes; line 81 is const declaration |
| VENDOR_MARKER_KEYS count = 13 | YES | 5 originals + 8 additions |
| SPEC §A line 27 "8 new format markers" | YES | |
| SPEC §6.1.1 line 94 cites `:81` with `:59-80` | YES | |
| SPEC §12 line 542 cites `:81` with `:59-80` | YES | |
| No residual `bitcoin_core.rs:74` or `:62` in SPEC | YES | grep returns 0 matches |

## Critical: None.

## Important: None.

## Minor

- M1 (cosmetic): SPEC §12 line 542 + §6.1.1 line 94 carry verbose audit-trail parentheticals. Improve transparency but heavier on review-meta. Not a fold; audit trail is load-bearing.
- M2 (cosmetic): VENDOR_MARKER_KEYS list ordering remains source-order, not alphabetical. Tagged for optional FOLLOWUP `vendor-marker-keys-ordering-discipline`; not blocking.

## Scope-creep audit

No new scope. Pure R2 SPEC-text-edit round.

## Overall verdict

**GREEN.**

R1's N1 + N2 folds applied correctly and verified at source. P0A has now passed three reviewer-loop rounds (R0 YELLOW 4I → R1 YELLOW 2I → R2 GREEN 0C/0I).

**Recommend merging P0A PR.** The §1.4 namespace lock, §2.1 8-value `--format` set, §2.2 schema_version-stay-at-"1" rationale, §6.1 sniff semantic carry-forward, §6.1.1 VENDOR_MARKER_KEYS 13-entry lock, §6.2 alphabetical SniffOutcome final order, §10 BIP-129 `path-restrictions` line-3 nomenclature lock, and §11.x per-parser provenance schemas are all source-faithful and provide a sound normative foundation for P0B.1, P0B.2, P0C, P0D (Wave 0 remaining) and the 6 Wave-1 per-parser phases (P1-P6) downstream.

---

**Sources:**
- Working-tree `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs` at `e8d0b07`
- Working-tree `design/SPEC_wallet_import_v0_28_0.md` at `e8d0b07`
- Previous rounds: R0 + R1 review docs
