## Appendix F — Architect R5 review (verbatim; persistence-debt-noted)

**Persistence note:** plan mode prevented direct write to `design/agent-reports/v0_28_0-plan-r5-review.md`. The orchestrating session MUST copy this appendix verbatim to that path before any Phase P0A execution.

R5 verified all R4 folds applied correctly. R5 identified 2 new Critical + 5 Important + 4 Minor — primarily because R4-C2 fold enumerated 4 occurrences of the `"expected 2 or 6 lines"` literal in the test file, but the actual grep at `fcf9e6d` shows 7 occurrences total (across source + tests + helper). R5's general lesson: string-literal sweeps must use `grep -rn` not line-range-citation.

---

```
# v0.28.0 plan-doc — architect R5 review (parallel-execution section, post-R4-fold)

## R4 fold verification

All 5 substantive R4 folds verified applied correctly:
- R4-C1 (P0D test rewrite to consult-all-then-count) ✓
- R4-C2 (P7A test-file edits 531-574) ✓ — but incomplete; see R5-C1
- R4-I1 (votes-array pattern lock) ✓
- R4-I2 (D→E 5-day wall-clock) ✓
- R4-I3 (alphabetical SniffOutcome order) ✓ — but framing wrong; see R5-I2
- R4-I4 (G1↔G3 conflict-table row) ✓
- R4-I5 (regression smoke) ✓

## R5 Critical (new — both folded)

### R5-C1. P7A enumeration missed 5th occurrence at cli_import_wallet_bsms.rs:621
[Folded R6: P7A scope now enumerates all 8 string-literal sites (after full grep: 2 source + 6 test/helper); discipline lock requires grep-pre-and-post run by executor.]

### R5-C2. P7A scope missed wallet_import/roundtrip.rs:87 canonicalize_bsms parallel surface
[Folded R6: P7A scope now includes "(2) R5-C2 mirror fix: add 4 => arm to wallet_import/roundtrip.rs::canonicalize_bsms at roundtrip.rs:82-90". Without this, P7C --json roundtrip cells fail asymmetrically.]

## R5 Important
- R5-I1 (sniff.rs:1-25 doc-comment update): deferred-minor; not folded (executor cosmetic).
- R5-I2 (alphabetical SniffOutcome wording incorrect — Ambiguous + NoMatch are not parsers): deferred-minor; the votes-array spec is correct on substance.
- R5-I3 (test docstring equivalence-class restatement): deferred-minor; spirit preserved.
- R5-I4 (4-line provenance empty-string sentinel discriminator design): noted for P7A execution; SPEC §10 should document the discriminator.
- R5-I5 (wall-clock arithmetic 20→16): folded.

## R5 Minor
- R5-M1-M4: cosmetic, executor-folded.

## Overall R5 verdict

YELLOW. R6 folds applied for R5 Criticals (test-literal completeness + canonicalize_bsms mirror) + R5-I5 arithmetic. Deferred R5-Importants are documentation-quality items that don't block correctness. R5 also surfaced a general grep-sweep-discipline that's now in the §"Verification — string-literal sweep" section.
```

---

---

