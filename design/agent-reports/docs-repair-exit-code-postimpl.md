# Post-impl R0 — GUI-manual repair exit-code lockstep (docs-only) — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Prose review of the 5-file docs-only diff vs live source in all 4 repos + git tags. Tree left as-is.

## Verified GREEN (probes 1-7)
- **Exit-code prose (load-bearing):** all 4 chapters' tables + worked examples match live source — `ms repair`→4 (ms-cli repair.rs:153), `mk repair` standalone→5 (mk-cli:165) / reassembly-fail→2 (error.rs:109), `mnemonic repair` ms1→4/mk1 5-or-4/md1→5 (toolkit repair.rs:1163-1172,:1049-1050,:451-453,:1491-1503), advisories byte-verbatim.
- **Per-binary version citations:** tag-diff verified (ms-cli-v0.14.0 demote, mk-cli-v0.12.0 advisory+reassembly-reject, toolkit v0.80/v0.81 legs, pins via gui-v0.57.0). No toolkit version in 6a (I-R3-1 satisfied); 4i toolkit-only; 79 mk-cli-only; 5A superset.
- Worked examples fixed (6a/4i→exit 4+advisory; 5A/79 stay 5); grep clean; anchors byte-identical (lychee 0); `make html` OK + `make lint` 12/12 GREEN against pinned gui-v0.57.0 (3 unpinned-sibling failures = pre-existing on the unrelated xpub-search chapter, FOLLOWUPS ~:3975-3981); NO-BUMP; FOLLOWUP flip cross-checks.

## Findings
Critical: none.
**Important:**
1. **79-repair.md:82-85 — false claim about mk-cli v0.12.0 + under-scoped pinned note.** "(only added the stderr advisory, not a new exit tier)" is WRONG: v0.12.0 ALSO moved a complete-`chunk_set_id`-group correction that fails cross-chunk reassembly from exit 5 (blessed at ≤v0.11.2) → exit 2 (`CliError::SetReassemblyMismatch`, the funds fix row 2 at :56 documents). (a) contradicts the chapter's own table; (b) :60-61 "reports exit 5 for ANY correction" overstates (rejected→2); (c) pinned note :91-93 under-scopes row 2. Fix: reword the parenthetical (v0.12.0 = advisory + complete-group reassembly reject; only incomplete kept 5) + scope row 2 at the pin.
2. **Stale `RepairJson` "byte-match" claims contradict the new superset intro.** 5A:49-51 (§--json "byte-matches RepairJson shape") + 79:12 ("byte-exact") + 79:33-34 are false on current binaries (toolkit envelope carries `verdict` for all kinds; md/mk lack it) AND contradict the superset intro this diff added at 5A:11-15. The FOLLOWUP scope included "the toolkit RepairJson superset note" — it landed only in 5A's intro. Fix: version-scope each (true at the pinned tier; current toolkit is a SUPERSET). 6a's byte-exact at :8-9/:39-40 are STILL TRUE (ms-cli v0.14.0 has `verdict` byte-parity) — leave.
**Minor:** 3. 4i:98 row 2 omits the toolkit mk1 complete-group reassembly-reject (exit 2, v0.80.0) — incomplete, SPEC-inherited. 4. md1 parenthetical wrote "content-id check" vs SPEC's "sibling delegate returns Ok only on full decode success" — factually accurate, avoids the prohibited "chunk-set-hash", deviates from the acceptance letter. 5. R0 audit: docs-SPEC rounds 3-4 persisted only as trailer summaries (not verbatim).

## VERDICT: OPEN (0C / 2I)
Both Importants = small localized prose fixes in 79-repair.md (+ one clause in 5A §--json). Otherwise fully verified. Fold + re-dispatch a scoped convergence review.

---
**FOLD STATUS (opus, 2026-07-11):** I1 + I2 sent back to the docs implementer (has file context; R0 gave exact loci). Scoped convergence R0 after the fold.