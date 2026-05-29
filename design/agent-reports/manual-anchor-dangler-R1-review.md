# R1 re-review — SPEC_manual_anchor_dangler_cleanup.md (verbatim, post-R0-fold)

Reviewer: feature-dev:code-reviewer (opus). R0 was RED 3C/4I/3M (`manual-anchor-dangler-R0-review.md`); 7 folds applied; this is the re-dispatch per CLAUDE.md "Reviewer-loop continues after every fold".

## VERDICT: RED (0C / 2I / 2M)

The seven R0 folds (C1-C3 + I1-I4) are all materially applied and verified against source ground truth. **No Critical drift remains.** Two new Important findings surface from internal-consistency cross-checking the folded spec: a self-contradiction between §4 Piece 3 (I4-hardened to `exit 1 + ::error::`) and §7 test-plan step 6 (still says `exit 0 + ::warning::`); and an off-by-N in Class B ref counts across §3 / §4 / sed-table (7 vs 8 vs 9 for the same population). Both are mechanical to fold and do not block R2.

## Per-fold confirmation

- **C1 — Class B sed targets literal-space (not `%20`):** APPLIED. Spec lines 71-86 enumerate the actual src/ hosts + literal-space sed recipe; `grep -rn '(#welcome-to-the-m-format constellation)' src/` → exactly 6 hits in `69-index-table.md` at lines 15, 26, 28, 29, 30, 32.
- **C2 — Class B slug-2 drops "constellation":** APPLIED. Real heading at `src/50-comparing/54-mformat-vs-others.md:1` verified; host files `src/10-foundations/11-welcome.md:94` + `src/50-comparing/51-format-decision.md:59` verified by grep (one hit each).
- **C3 — Class C deleted; reclassified as Class A; no companion FOLLOWUP:** APPLIED. §3 line 46 explicit C3-fold narrative; §3 table lists only A/B/D; §4 Piece 2 line 67 explicit deletion narrative. Grep `manual-worked-example-anchor-targets-author` → 1 hit (line 139, deletion narrative only).
- **I1 — Piece 3 baseline-capture order-explicit:** APPLIED. §4 Piece 3 line 92 explicit ordering ("Pieces 1+2 land FIRST").
- **I2 — 174 locked as canonical:** APPLIED. §3 line 37 locks 174; grep `169` → exactly 1 hit (line 37 historical-narrative only).
- **I3 — html target uses `$(MD_FILTER_ARGS)` explicitly:** APPLIED. §4 Piece 1 line 55 explicit; verified against `docs/manual/Makefile:78-79`.
- **I4 — baseline-shrunk is exit-1 ERROR:** APPLIED in §4 but INCONSISTENT WITH §7 — see I-1 below.
- **M1+M2+M3:** All applied (script `set -euo pipefail`; quickstart.yml interaction; cycle-prep deletion).

## Anti-drift checks

1. "169" residual count — 1 hit (line 37 historical only). PASS.
2. "Class C" — 1 hit (narrative-of-deletion). PASS.
3. `manual-worked-example-anchor-targets-author` — 1 hit (narrative-of-deletion). PASS.
4. `worked-example-*` — 3 hits; 1 residual drift at line 147 → M-a below.
5. §6 file list internal-consistency — PASS.

## New findings

### I-1 — §7 step 6 contradicts §4 Piece 3 I4-hardened behavior

**Where:** SPEC line 150 (Test plan step 6).
**Conflict:** §7 step 6 says "exits 0 with `::warning::`" but §4 Piece 3 line 106 (the I4 fold itself) hardens this to "exit 1 with `::error::`" as a CI-blocking error.
**Impact:** Implementer would encode the wrong exit-code expectation in the synthetic-recovery integration test.
**Fix:** Rewrite §7 step 6 to "exits 1 with the `::error::` annotation naming the shrunk slug; same PR must commit the ratcheted baseline file to clear the gate."
**Confidence: 95.**

### I-2 — Class B reference-count off-by-N across §3 / §4 sed-table / §4 line 100

**Numbers cited for same population:**
- §3 table line 42 Class B refs = 7.
- §4 sed table lines 73-75 = 6+1+1 = 8 refs.
- §4 line 100 "Piece 2 removes 8 of 9 known Class B refs".

**Empirical (grep against src/):** 6 + 1 + 1 = **8 refs / 2 unique slugs**.

**Fix:** §3 table Class B refs `7` → `8`. §4 line 100 "8 of 9" → "all 8 known Class B refs".
**Confidence: 90.**

## Sub-threshold (Minor)

### M-a — §7 step 3 carries stale C3-pre-fold wording

**Where:** line 147: "After Piece 2: `%20` references resolve; `worked-example-*` references removed."
**Fix:** "After Piece 2: literal-space references resolve (post-sed, `grep` returns zero for the two literal-space patterns)."

### M-b — §7 step 2 says "the 15 architectural anchors" — pre-C3 count

**Where:** §7 line 146. Class A is now ~24 (15 + ~9 worked-example-*).
**Fix:** "the ~24 architectural anchors (15 explicit `{#id}` losses + ~9 worked-example-* TOC slug-rule mismatches)".

## Reviewer-loop expectation

R1 finds 0C/2I/2M → RED. Apply I-1, I-2, optionally M-a + M-b (same §7 cleanup pass), then re-dispatch for R2. Expected convergence: 1 more round to 0C/0I GREEN.
