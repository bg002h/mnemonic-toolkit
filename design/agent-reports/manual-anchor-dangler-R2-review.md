# R2 fold-verify — SPEC_manual_anchor_dangler_cleanup.md (verbatim)

Reviewer: Sonnet (trivial fold-verify per `feedback_opus_primary_review_agent.md`). R1 was RED 0C/2I/2M; four folds applied.

## VERDICT: GREEN (0C / 0I)

All four folds verified against source ground truth. All six anti-drift checks pass.

## Per-fold confirmation

**I-1 — §7 step 6 exit-code + ratchet language:** APPLIED. Line 150 now reads "exits 1 with the `::error::` annotation" and "The same PR MUST commit the ratcheted baseline to clear the gate". The historical attribution "(was previously `exit 0 + ::warning::` — pre-I4-hardening wording)" is retrospective prose, not a specification of current behavior; the operative text is correct.

**I-2 — Class B refs count:** APPLIED. §3 table line 42 now shows `8` (was `7`). Line 100 reads "Piece 2 removes all 8 known Class B refs" (was "8 of 9"). Both match the empirical 6+1+1=8 count from the sed table.

**M-a — §7 step 3 stale wording:** APPLIED. Line 147 now reads "literal-space references resolve (post-sed, `grep -rn` returns zero hits for the two literal-space patterns)" — no "worked-example-* references removed" wording remains in that step.

**M-b — §7 step 2 count:** APPLIED. Line 146 now reads "the ~24 architectural anchors (15 explicit `{#id}` losses + ~9 worked-example-* TOC slug-rule mismatches)". No unqualified "15 architectural anchors" text remains anywhere in the spec.

## Anti-drift check results

1. `::warning::` in §7 step 6 — appears only in historical-attribution clause, not as specified current behavior. PASS.
2. "exits 0 with" / "exit 0 with" — zero matches in the spec. Step 4 uses "exits 0 silently"; step 6 uses "exits 1". PASS.
3. Stray `7` or `9` in Class B count context — no matches. Only `## 7 —` and `## 9 —` section headings remain. PASS.
4. Unqualified "15 architectural" — zero matches. PASS.
5. `%20` in §7 step 3 — zero matches in that step; `worked-example-` in step 3 — zero matches. PASS.
6. §3 Class B row shows `8` — confirmed at line 42. PASS.

## Residual sub-threshold items

None. All R1 M-items were folded cleanly. No new sub-threshold findings.

## Cleared to implement.
