# C5 (import --format descriptor) plan-R0 round 3 — architect confirmation (verbatim)

> Reviewer: opus architect (general-purpose). Final confirmation of the round-2 residue folds on
> `design/PLAN_C5_import_format_descriptor_2026-06-17.md` @ toolkit `b15f5e6`.

---

**Verdict: GREEN (0C/0I)**

All three round-2 residues resolved; no new contradiction introduced.

1. **Residue #1 (`DescriptorParse` error-variant):** ZERO occurrences. All `DescriptorParse` hits are the legitimate `DescriptorParser` STRUCT name (trailing `r`). Impl §line 99-100 correctly uses `ImportWalletParse`. Resolved.
2. **Residue #2 ("decision 5"):** Execution checklist correctly reads "checksum-tolerant-vs-required (decision 6)". Resolved.
3. **Residue #3 (citations "thin glue"):** table reads "the import-parser glue" + the NOTE that `descriptor_concrete_to_resolved_slots` is NOT the path. Resolved.

**Internal-consistency sweep — uniform:**
- Toolkit MINOR v0.57.1→v0.58.0; GUI MINOR v0.41.0→v0.42.0; GUI pin v0.56.0→v0.58.0 (TWO behind, framing consistent).
- Checksum consistently TOLERANT (Decision 6 + impl step 2); the "WRONG" reference is the corrected historical note.
- Explicit specter glue path consistently chosen (`concrete_keys_to_placeholders` + `extract_origin_components`); `descriptor_concrete_to_resolved_slots` consistently NOT-the-path.
- No stale decision-number pointers; no "bundle ONLY"/"one behind"/stale "PATCH" leftovers (the two `PATCH` hits are the R0-vet question framing, resolved to MINOR).

This is the final R0 gate and it is GREEN — implementation may begin.
