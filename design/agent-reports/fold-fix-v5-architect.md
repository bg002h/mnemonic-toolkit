# FIX-V5 transcription fold: architect STOP-1/2/5/7 texts

Ruling: `refugium-coordinator/coordinator/agent-reports/architect-2026-11-04-fix-v5-stops.md`.
Target: `design/FIX_v5_wsh_cost.md`, branch `fix/v5-wsh-cost`, base head 835cba39.

Gate: `bash design/agent-reports/fix-doc-tk-v5-evidence/doc-gate.sh`
- before: PASS (`/scratch/cache/tk-fix-v5-logs/fold-gate-before.log`)
- after: PASS, exit 0 (`/scratch/cache/tk-fix-v5-logs/fold-gate-after.log`)
- gate's own "not covered" line: figure correctness, Core agreement, build/tests of the patched tree, GUI byte gate, manual job, runtime output, prose

Fold commit: 92c93bf6 `FIX-V5 fold: architect STOP-1/2/5/7 texts` (89 lines changed in the FIX doc; gate tail in the message).

## Edit table

Apply log: `/scratch/cache/tk-fix-v5-logs/fold-apply.log` (25 applied, 0 STOP).

| Ruling edit | Anchor in FIX doc | Result |
|---|---|---|
| S7-header | header NEW_BASE and status lines | applied |
| S7-6first | §6 first sentence (base on origin/master) | applied |
| S1-3-R5 | §0.1 row R5 | applied |
| S1-3-R6 | §0.1 row R6 | applied |
| S1-1-A2 | §1 A2 heading and paragraph | applied (A2 moved to F3) |
| S1-2-33h | §3.3 heading | applied |
| S1-2-33s | §3.3 body | applied |
| S1-3-S5b | §5 bullet | applied |
| S1-3-S8cast | §8 bullet (cast_dupif) | applied |
| S1-3-S8thr | §8 bullet (threshold) | applied |
| S1-4-6s3 | §6 step 3 | applied |
| S5-6s3b | §6 step 3 (STOP-5 revert text) | applied |
| S5-4.2row | §4.2 row | applied |
| S5-4.2png | §4.2 PNG paragraph | applied (held out to F4) |
| S5-5count | §5 file count "Eight files change" | applied |
| S5-5row7 | §5 row 7 | applied |
| S5-5row8 | §5 row 8 | applied |
| S5-7item12 | §7 item 12 | applied |
| S7-7pre | §7 preamble | applied |
| S7-7item1 | §7 item 1 | applied |
| S7-7item3 | §7 item 3 | applied |
| S7-7item14 | §7 item 14 | applied |
| S5-F2b3 | §8 F2 build impact (3) | applied |
| S1-5-F2b4 | §8 F2 build impact (4) | applied |
| S1-6F3F4 | §8 new FOLLOWUPS F3 `compare-cost-wsh-cost-self-check` and F4 `manual-gui-j4-20-compare-cost-v5-figures` | applied |

Checks run after the edits: no literal code fences around F3/F4, no residual "record the deferral" text, F3 at line 487, F4 at line 494, "## 10" at line 500. The §6 bullet used an inclusive span; the A2 span stayed exclusive so the "## 2." heading survives.

## STOPs (not applied)

- STOP-2: the ruling's "Ruled" line has no exact wording. Not applied.
- STOP-3, STOP-4, STOP-6: not ruled in the ruling. Only the text the ruling gives was applied; nothing else.
- STOP-7 §3.1 and §3.2: the measured-line quotes need a build measurement. Not applied.
- STOP-7 "Order:" paragraph and the FIX-V7 coordinator note: no anchor in the doc. Coordinator action.

## Residue for FIX review (not fixed; the ruling gives no replacement text)

- Header Status line: "Part A (V5) waits on STOP-1."
- §0/§5 self-check rows: "written by the builder once STOP-1 is ruled." (~line 72)
- Line ~89: `fix-v5-production.patch` "Do not apply (STOP-1)."
- §3.3 rows: "written by the builder once STOP-1 (and STOP-6...) are ruled." (~lines 243-244)
- Line ~252: "STOP-4 decides what the human view may already have printed."
- Line ~427: §7 item 12 "STOP-5 decides what passes here."
- Line ~639: "Held by STOP-1 (a) until the ruling."
- Line ~997: "STOP-1 holds A2."
- §6 step 3: a dangling "Under any other ruling, the ruling's text replaces this step..." sentence after the ruled first two sentences.
- §3.3: "These rows move to F3 unchanged; no code exists for them." is followed by the original "No code exists for them...", which repeats the point.
- NEW_BASE drops the "worktree not under /tmp" rule from §6, as the ruling wrote it. Flagged for the reviewer to confirm.

## Not changed

`design/agent-reports/fix-doc-tk-v5-evidence/doc-gate.sh` was not modified; the ruling does not change it and the gate passed without it.
