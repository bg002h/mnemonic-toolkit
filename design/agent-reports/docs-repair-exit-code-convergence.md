# Convergence R0 — docs repair exit-code fold (I1/I2) — Fable, adversarial

**Persisted per CLAUDE.md.** Scoped to the I1/I2 folds + optional Minor; rest verified GREEN in the post-impl pass.

## I1 (79-repair.md mk-cli v0.12.0) — CLOSED
(a) :90-97 now states v0.12.0 added BOTH the incomplete-group UNVERIFIED advisory AND the complete-group reassembly-reject (exit 2, SetReassemblyMismatch); ≤v0.11.2 blessed that case at exit 5. Tag-reconfirmed: `SetReassemblyMismatch=>2` present at mk-cli-v0.12.0 (error.rs:109), absent at v0.11.2; advisory byte-verbatim (repair.rs:143-146). (b) :66-73 "any NON-rejected correction" + carves out reassembly-fail→exit 2. (c) pinned note :104-109 scopes BOTH cases at the v0.11.0 pin. Table coherent, no residual contradiction.

## I2 (RepairJson version-scoping) — CLOSED
79:12-17 + :38-42 + 5A:10-17 + :49-53 → shared-field/toolkit-superset framing (verdict added after `kind` since v0.81.0). Source-verified: RepairJson.verdict present at mnemonic-toolkit-v0.81.0 (repair.rs:289-302), absent at v0.80.0; live mk/md envelopes = {schema_version,kind,corrected_chunks,repairs} (no verdict). 6a:8-9/:39-40 LEFT UNCHANGED + still TRUE (ms-cli v0.14.0 has verdict byte-parity, its own doc says so). Grep: byte-exact/byte-match only in 6a.

## Optional Minor (4i:98) — FOLDED
Exit-2 row now enumerates the toolkit mk1 complete-group reassembly-reject (SetReassemblyMismatch, since toolkit v0.80.0; correction NOT applied). Tag-verified present at v0.80.0, absent v0.79.0.

## No drift + gates — GREEN
markdownlint 0 / cspell 0 (.cspell.json untouched); `make html` OK; `make lint` 12/12 GREEN against pinned gui-v0.57.0 (lychee 2071 OK, gui-schema-coverage 984 anchors, figures 61/61, tutorial 50+98 byte-identical). Anchors byte-identical. 5 files, no code/schema/non-gui change, NO-BUMP.

## Findings
Critical/Important: none. Minor (non-blocking, NOT folded — optional cosmetics): (1) FOLLOWUPS.md:74 resolution note doesn't mention the reassembly-reject scoping (not false, could be fuller); (2) 79:72 "row 2 above" cosmetic; (3) 4i:121 "chunk-set-id" prose style.

## VERDICT: GREEN (0C/0I) — docs ready to ship.

---
**SHIP (opus, 2026-07-11):** GREEN, tag-source-verified. Shipping docs NO-BUMP direct to master. 3 Minors left as-is (optional cosmetics).
