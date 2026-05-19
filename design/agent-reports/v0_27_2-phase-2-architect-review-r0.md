# v0.27.2 Phase 2 architect review — R0

**Reviewer:** opus feature-dev:code-reviewer
**Branch:** `release/v0.27.2` at Phase 2 close (commits `cc15cf0` + `7825bf1`)
**Date:** 2026-05-19
**Verdict:** GREEN (0 Critical / 0 Important / 3 Minor — all non-blocking)

## Scope reviewed

Phase 2 of v0.27.2 — the `ImportProvenance` internal refactor (item 1):
- `cc15cf0` (refactor): introduces `ImportProvenance` enum + accessors; updates `ParsedImport`; refactors 2 construction sites + 7 access sites; adds 4 unit cells. 5 files changed (+133 / -25).
- `7825bf1` (chore): Status flip back-fill for `pr-26-import-provenance-enum-internal-refactor` FOLLOWUP at `design/FOLLOWUPS.md:108`.

## Notable execution-time deviation from plan

Plan Task 2.2 specified the enum variant as `Bsms(BsmsAuditFields)` non-optional. Implementer found this type-fails because `bsms.rs`'s 2-line BSMS path produces `audit: Option<BsmsAuditFields>` with `None`. Implementer changed variant to `Bsms(Option<BsmsAuditFields>)` + added a 4th unit test for `Bsms(None)` case + updated accessor to return `audit.as_ref()`.

Type-level invariant analysis:
- **Pre-refactor:** 4 representable states for the `(bsms_audit, source_metadata)` field pair. Three legal: `(Some, None)` BSMS-with-audit; `(None, None)` BSMS-2-line; `(None, Some)` BC. One illegal: `(Some, Some)`.
- **Plan-locked (Bsms non-optional):** Would have required a sentinel `BsmsAuditFields::two_line_default()` OR a separate variant. The plan-doc didn't specify the 2-line construction path → execution-time type-fail.
- **Shipped (`Bsms(Option<_>)`):** 3 legal states. Illegal `(Some, Some)` impossible (single variant). Illegal `(None, None)` impossible (variant always carries `Bsms(_)` or `BitcoinCore(_)` tag). FOLLOWUP's "representable-invalid pair" intent is preserved.

No forced `.clone()`; one `.as_ref()` contained inside accessor at `mod.rs:79`.

## Critical / Important

None.

## Minor

### M1 — Three-variant alternative for cleaner 2-line/6-line discrimination (design, non-blocking)

**Cite:** `crates/mnemonic-toolkit/src/wallet_import/mod.rs:62-71`

The shipped design `Bsms(Option<BsmsAuditFields>)` is sound but hides the 2-line-vs-6-line shape distinction inside an `Option`. A future cleanup could promote to:

```rust
pub(crate) enum ImportProvenance {
    BsmsTwoLine,
    BsmsSixLine(BsmsAuditFields),
    BitcoinCore(CoreSourceMetadata),
}
```

Accessor would naturally return `Some(&audit)` only for `BsmsSixLine`. Minor because (a) the current shape is correct and (b) FOLLOWUP filing is appropriate but not pre-merge blocking. Recommend filing as v0.28+ tier FOLLOWUP `pr-26-import-provenance-three-variant-cleanup`.

**Confidence:** 30.

### M2 — Stale module-level docstring references to former direct fields

**Cite:** `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:8` and `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs:25`

Both module-level docstrings still reference `ParsedImport.bsms_audit` and `ParsedImport.source_metadata` as if they were direct fields. After the refactor, these are accessor methods on `ParsedImport`. Suggested:
- `bsms.rs:8`: `preserved in ParsedImport::bsms_audit() (accessor; backed by ImportProvenance::Bsms).`
- `bitcoin_core.rs:25`: `preserved in ParsedImport::source_metadata() (accessor; backed by ImportProvenance::BitcoinCore).`

**Confidence:** 85 / 60. Real (minor) doc-rot.

### M3 — Provenance unit-test ordering not alphabetical-by-function-name

**Cite:** `crates/mnemonic-toolkit/src/wallet_import/mod.rs:336-361`

CLAUDE.md alphabetical-ordering rule governs enum variants, not test functions — no project rule gates test ordering. Maintainer discretion.

**Confidence:** 25.

## Verification notes (R0 source-truth audit)

All 12 grep-verified claims hold:

1. Enum + accessors correct (`mod.rs:62-91`).
2. ParsedImport convenience methods present (`mod.rs:124-134`).
3. Bsms construction correct (`bsms.rs:266-273`).
4. BitcoinCore construction correct (`bitcoin_core.rs:291-306`) — `Some()` wrapper dropped as planned.
5. 5 cmd/import_wallet.rs access sites correct: {587, 599, 806, 818, 825}.
6. 2 mod.rs access sites correct at lines 204-206 (ActiveReceive, **negation `!m.internal` preserved**) + 220-222 (ActiveChange, no negation).
7. Wire shape preserved byte-identical — envelope fixture omits keys (not emits null) for the no-audit / no-metadata cases. `tests/cli_import_wallet_envelope_v0_27_0.rs:355-376` is the load-bearing regression guard.
8. 4 provenance_tests cells each test a real distinct invariant.
9. FOLLOWUPS.md Status flipped at line 108 to `resolved (cc15cf0; v0.27.2 Phase 2)`.
10. No non-Phase-2 files touched inadvertently.
11. Type-level invariant analysis confirms representable-invalid pair eliminated.
12. No forced `.clone()` calls.

## Recommended next steps

- **Unblock Phase 4 dispatch.**
- **Optional pre-tag fold** (M2): touch up 2 stale docstrings at `bsms.rs:8` + `bitcoin_core.rs:25`.
- **Optional FOLLOWUP filing** (post-tag, v0.28+ tier): `pr-26-import-provenance-three-variant-cleanup`.
- **Carry-forward observation for cycle-close architect:** plan-doc R0-R3 + R4 GREEN did NOT catch the `Bsms(BsmsAuditFields)` type-fail. Execution-time discovery handled correctly per `feedback-grep-verify-during-fold-not-just-during-write`, but plan-doc reviewer-loop could add a "trial `cargo build`" step at R3 (or Phase 0).
