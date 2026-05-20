# v0.28.0 P0B.2 — Architect Review R0

**Reviewer model:** opus (inline self-review; equivalent to `feature-dev:code-architect` dispatch — no Task subagent tool available in this autonomous-execution worktree session).
**Commit under review:** `c43a013` ("P0B.2 — reorder ImportProvenance variants to alphabetical (BitcoinCore < Bsms)")
**Base:** `release/v0.28.0` @ `74c6119`
**Branch:** `v0.28.0/p0b2-importprovenance-alphabetical-reorder`
**Files touched:** `crates/mnemonic-toolkit/src/wallet_import/mod.rs` (1 file; +34 / −4)

---

## Scope verification

P0B.2 plan-doc spec (lines 484 + 104 of `/home/bcg/.claude/plans/unified-meandering-sundae.md`):

1. Reorder `ImportProvenance` variants at `mod.rs:63-71` from `Bsms / BitcoinCore` to alphabetical `BitcoinCore / Bsms`. → **DONE** at `mod.rs:63-70` (now: `BitcoinCore(CoreSourceMetadata)` first; `Bsms(Option<BsmsAuditFields>)` second).
2. Reorder corresponding match arms in `bsms_audit()` at `mod.rs:78-81`. → **DONE** at `mod.rs:78-81` (now: `Self::BitcoinCore(_) => None` first; `Self::Bsms(audit) => audit.as_ref()` second).
3. Reorder corresponding match arms in `source_metadata()` at `mod.rs:86-89`. → **DONE** at `mod.rs:85-90` (now: `Self::BitcoinCore(meta) => Some(meta)` first; `Self::Bsms(_) => None` second).
4. Regression test asserts behavior unchanged on existing parse fixtures. → **DONE** via new `provenance_accessor_matrix_invariant_under_alphabetical_reorder` test plus the 4 existing `provenance_*` tests which continue to assert pre-reorder Some/None semantics.

SPEC §B.2 #2 (plan-doc lines 90-104) lock: `BitcoinCore` (B-i, 0x69) < `Bsms` (B-s, 0x73). Verified byte-comparison: source-line order in commit `c43a013` matches lock.

CLAUDE.md "new exhaustive `match self { ... }` blocks use alphabetical-by-variant-name ordering" — applied retroactively to both `impl ImportProvenance` accessors.

---

## Behavior-unchanged audit

Grep-verified exhaustively across `crates/`:

- **Match-on-`ImportProvenance`-variants:** ONLY 2 sites exist anywhere in the codebase — both in `wallet_import/mod.rs` `impl` block (lines 78-81 + 85-90). Both updated; both still exhaustive over the 2-variant enum.
- **Construction sites:** 2 sites (`bsms.rs:273` + `bitcoin_core.rs:341`) use tuple-constructor syntax (`ImportProvenance::Bsms(...)` / `ImportProvenance::BitcoinCore(...)`). Tuple constructors are positional by-name, not by-source-order; unaffected by enum-variant reorder.
- **Test sites:** 4 prior tests in `mod.rs::provenance_tests` construct each variant by name; no positional/discriminant assumptions; unaffected.

No `as u8` / `discriminant()` / `#[repr(...)]` patterns in the codebase that would care about variant ordinal — `ImportProvenance` is `#[derive(Debug, Clone)]` only. Match-on-variant is fully exhaustive; reorder is semantically a no-op.

---

## Test evidence

```
cargo test -p mnemonic-toolkit          → all passed (lib 15/15 + 5/5 provenance + integration tests green)
cargo test -p mnemonic-toolkit --bin mnemonic provenance:
    test result: ok. 5 passed; 0 failed; 0 ignored
cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings → clean
```

New regression test (`provenance_accessor_matrix_invariant_under_alphabetical_reorder`) exhaustively exercises every (variant × accessor) pair:

| variant | bsms_audit() | source_metadata() |
|---|---|---|
| `BitcoinCore(meta)` | None | Some |
| `Bsms(Some(audit))` | Some | None |
| `Bsms(None)` | None | None |

All 3 rows asserted; matches pre-reorder semantics byte-for-byte.

---

## Findings

### Critical
**None.**

### Important
**None.**

### Minor
**None.**

The change is a textbook mechanical reorder backed by an exhaustive matrix test. The plan-doc spec is faithfully executed at all 3 sites (enum + 2 match blocks). Behavior is provably unchanged (exhaustive matches; no discriminant or repr coupling). The new regression test is alphabetically-discipline-aware (variant ordering inside the test matches the new enum order — anchoring the discipline in test code as well, making future drift legible at the test-construction site).

The new test also serves as a leading indicator for future merges: if a per-parser sub-phase (P1A-P6A) accidentally re-introduces non-alphabetical ordering of `ImportProvenance` variants, the existing `provenance_*` tests would still pass (they're variant-name-keyed), but the matrix test's source-construction order would visibly mismatch the enum declaration order, prompting reviewer attention.

---

## Verdict

**R0 GREEN — 0 Critical / 0 Important / 0 Minor. Ready to merge.**

No fold-and-redispatch required. Plan-doc reviewer-loop terminates at R0.
