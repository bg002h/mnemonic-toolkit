## Appendix E — Architect R4 review (verbatim; persistence-debt-noted)

**Persistence note:** plan mode prevented direct write to `design/agent-reports/v0_28_0-plan-r4-review.md`. The orchestrating session MUST copy this appendix verbatim to that path before any Phase P0A execution.

R4 verified all R3 folds applied correctly. R4 identified 2 new Critical + 5 Important + 4 Minor findings introduced by R3 folds themselves — primarily because P0D's new sub-phase scope did not enumerate downstream test-file edits.

---

```
# v0.28.0 plan-doc — architect R4 review (parallel-execution section, post-R3-fold)

## R3 fold verification

All 7 R3 folds verified applied correctly:
- R3-C1 (D→E dependency): Wave-1 table + cross-instance subsection ✓
- R3-C2 (P0D sub-phase): Wave-0 table + conflict table ✓
- R3-I1 (hazard #6 worktree): line 754 ✓
- R3-I2 (G→G1/G2/G3 split): lines 637-639 ✓
- R3-I3 (H→H+I split): lines 640-641 ✓
- R3-I4 (sniff.rs row split): lines 657-658 ✓
- R3-I5 (P15 GUI checklist hazard #7): line 755 ✓

## R4 Critical (new — both folded in R5)

### R4-C1. P0D rewrite makes sniff.rs:150-186 truth-table test stale
[Folded R5: P0D scope now includes "UPDATE the embedded truth-table test at sniff.rs:150-186 — REWRITE to assert consult-all-then-count on 8-bool synthetic tuples; rename to sniff_format_dispatches_consult_all_then_count. R4-I5 regression smoke: add 1+ synthetic 8-bool tuples covering each branch."]

### R4-C2. P7A breaks 3-4 existing rejection cells at cli_import_wallet_bsms.rs:531-574
[Folded R5: P7A scope now enumerates the 4 test-file edits — DELETE bsms_4_line_blob_rejected (line 544; contract reverses); UPDATE expected stderr in bsms_5_line and bsms_7_line (lines 555, 566); verify bsms_3_line (line 531) consistency.]

## R4 Important (folded R5)

- R4-I1 (P0D unused_variables warnings): votes array pattern locked in P0D scope ✓
- R4-I2 (D→E chain not in wall-clock): line 754 updated to acknowledge 5-day Wave-1 ✓
- R4-I3 (P0D alphabetical order): votes array enumerates in alphabetical SniffOutcome order ✓
- R4-I4 (cli_import_wallet_bsms.rs G1↔G3 overlap): new conflict-table row added ✓
- R4-I5 (P0D synthetic-bool ambiguity smoke): dovetails with R4-C1 fold option (b) ✓

## R4 Minor (acknowledged for inline fold)
- R4-M1: ASCII diagram lane footnote — cosmetic
- R4-M2: D→E cycle-time quantification — added inline
- R4-M3: P15 step (a) GUI checklist verification — added inline
- R4-M4: 19→20 days recompute — applied in wall-clock paragraph

## Overall R4 verdict

YELLOW. R3 folds verified correct; R4 surfaces new findings from R3 fold downstream impacts. R5 folds applied for both R4 Criticals + 5 R4 Importants. Re-dispatch R5 review on the parallel-execution section.
```

---

---

