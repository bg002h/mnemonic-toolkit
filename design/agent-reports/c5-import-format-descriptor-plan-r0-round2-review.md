# C5 (import --format descriptor) plan-R0 round 2 — architect review (verbatim)

> Reviewer: opus architect (general-purpose). Confirms the round-1 folds (M1-M5 + error-variant).
> Verdict RED (3 fold residues); all folded post-review (see footer) → round-3 confirmation pending.

---

**Verdict: RED**

Three fold residues, one of which is a material internal contradiction:

1. **[Contradiction — gates] Line 100** says `NO ToolkitError variant (reuse \`BadInput\`/\`DescriptorParse\`)` — directly contradicts the folded **Decision 8** and the round-1 error-variant refinement, both of which specify **`ImportWalletParse`** for parser internal errors. Confirmed against source: `error.rs:216-219` documents `ImportWalletParse` as the variant the other import parsers (`BsmsParser`, `BitcoinCoreParser`) emit — the "specter's convention" the review cited; `error.rs:122-123` documents `DescriptorParse` as descriptor *content* failures, a different namespace. Line 100 names the wrong variant. **Fix:** `(reuse \`BadInput\`/\`ImportWalletParse\`)`.

2. **[Stale citation — fold residue] Line 156 (Execution R0 checklist)** says checksum is `(decision 5)`. After the fold, the checksum item is **Decision 6**; decision 5 is now "Comment-strip semantics." Off-by-one decision pointer. **Fix:** `(decision 6)`.

3. **[Stale framing — fold residue, minor] Line 39 (citations table)** still labels `descriptor_concrete_to_resolved_slots` as `(the thin glue)`. M2 dropped that helper as the path. **Fix:** re-annotate ("NOT the glue path; see impl step 3 / R0-r1 M2").

**On the 6 asked confirmations:** M1 ✓ (no leftover "requires"), M2 ✓ for impl step 3 body but residue #3 in the citation table, M3 ✓, M4 ✓ (two behind, MEASURE intact), M5 ✓ in the body, error-variant ✗ at line 100 (#1). Version targets otherwise consistent: toolkit v0.57.1→v0.58.0 and GUI v0.41.0→v0.42.0 uniform everywhere else.

Finding #1 is the blocker (material contradiction on which error variant to wire); #2 and #3 are stale-citation cleanups. Recommend folding all three, persisting, re-dispatching for round-3 per the post-fold reviewer-loop discipline.

---

## FOLD (post-round-2, by implementer)

- **#1 fixed:** Implementation §line 100 → "NO new ToolkitError variant — parser internal errors use `ImportWalletParse` (specter's convention), strip/arity refusals use `BadInput` (Decision 8)." Now consistent with Decision 8.
- **#2 fixed:** Execution R0 checklist → "checksum-tolerant-vs-required (decision 6)".
- **#3 fixed:** citations table → "descriptor-string → placeholder + keys (the import-parser glue) | `concrete_keys_to_placeholders` + `extract_origin_components` (the specter path — R0-r1 M2). NOTE: `descriptor_concrete_to_resolved_slots` is NOT the path …".
- **Self-sweep:** `grep DescriptorParse|decision 5)|the thin glue|likely PATCH|one behind|requires a checksum|REQUIRES one` → only false-positive hits on `DescriptorParser` (the parser STRUCT name, legitimate) and the line-72 corrected-explanation. No remaining contradiction. Version targets uniform (toolkit v0.58.0, GUI v0.42.0).
