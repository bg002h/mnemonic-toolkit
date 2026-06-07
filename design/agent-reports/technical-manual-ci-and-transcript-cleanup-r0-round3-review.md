# R0 Architect Review — technical-manual CI workflow + api-harvest transcript cleanup — Round 3 (fold-confirmation, GREEN)

> SPEC: `/scratch/code/shibboleth/mnemonic-toolkit/design/SPEC_technical_manual_ci_and_transcript_cleanup.md`
> Source SHA: `6c9e629` (== `origin/master` == local `HEAD`).
> Scope: confirm Round-2 folds (I1, M1') are clean and the SPEC is internally consistent end-to-end. Round 1's full verification not re-derived. Reviewer had Read/Glob/Grep only; parent persists.

## Verdict: GREEN (0C / 0I)

0 Critical, 0 Important. One disposed Minor-note (non-blocking, logged below for audit continuity).

## Critical
None.

## Important
None.

## Minor (disposed, non-blocking)

- **M-r3.1 (register note, line 167 — accepted as-is).** The warning-reword string and the `skip:absent-sibling` status name (line 141) retain a codec/sibling-only label after I1 broadened the human-facing docs to "codec AND renamed-toolkit files." This is **consistent, not a contradiction**: I1 operates on the human-facing docs register (the YAML taxonomy comment + §2 prose + ship-plan step 5), where the codec-vs-renamed-toolkit distinction is a meta-observation. Line 167 / line 141 are the script's runtime-output register, which is necessarily coarse — at runtime the script only knows "unresolvable + `ABSENT` non-empty" and cannot disprove the basename lives in an absent sibling (the literal rationale at lines 137-139). `skip:absent-sibling` is the script's honest epistemic label. Broadening it would also overstep this task's scope, which is to confirm the gate-fix logic (incl. the warning reword) remains **untouched**. No action.

## Fold confirmation

- **I1 — CLEAN.** The reworded YAML comment block (lines 182-185) now reads "G2 for any ref whose file is unresolvable here — codec files in absent siblings, AND renamed toolkit files cited by bare basename from non-authoritative chapters — skips gracefully in bare CI." This agrees with §2 (lines 109-111: "The same skip extends to a *non-authoritative-chapter* ref to any file unresolvable in the present (toolkit) repo") and ship-plan step 5 (lines 298-302: "any bare-basename ref in a non-authoritative chapter whose file is unresolvable in the present (toolkit) repo … (i) codec-file G2 … AND (ii) a *renamed toolkit file* cited by bare basename"). All three share the "any file unresolvable in the present (toolkit) repo" framing with the two-part codec + renamed-toolkit enumeration. §2's line-106 opening ("codec-file G2 … is not enforced") is a specific→general arc, not a holdout: the "The same skip extends to …" marker at line 109 makes the codec case explicitly non-exhaustive, so §2 genuinely carries the full framing. The pre-M1 narrow "G2 for TOOLKIT refs enforced; … absent sibling skips" framing has zero occurrences. Mutually consistent.

- **M1' — CLEAN.** All three api-surface-coverage descriptions now say `lib.rs/format.rs`: line 99 (§ "make lint is binary-independent"), line 179 (YAML header comment), line 240 (`*_BIN=true` step comment). Consistent with each other and with the accurate rationale (binary-only toolkit crate reads `format.rs`, having no `lib.rs`). No "reads `lib.rs`"-only holdout remains.

## Check-by-check

1. **YAML ↔ §2 ↔ ship-plan step 5 agreement — PASS.** Quoted and reconciled above (I1 fold confirmation).
2. **All `lib.rs` mentions consistent — PASS.** Lines 99, 179, 240 all `lib.rs/format.rs`. No other occurrences.
3. **No new inconsistency / contradiction / broken ref / factual error — PASS.** The YAML's FOLLOWUP reference (line 186) matches the filed id in ship-plan step 5 (line 296). No fold-introduced drift.
4. **Gate-fix logic, proof matrix, triggers, tool-install list, disposition untouched — PASS (internal coherence).** Cannot diff against Round 2 from here, but: gate-fix logic (Item 2a, lines 132-167) is internally coherent; proof matrix reconciles (Test A 298 checked + 427 skipped = 725, matching Test B's 725/0; Test E line 276 reuses 298/427); triggers (push/PR on-paths, no tag, lines 189-200 + 254-255) coherent; tool-install list (node + markdownlint-cli2 + cspell + lychee + python3; no cargo/chromium, lines 219-235 + 280-281) coherent; disposition (docs+CI only, no bump/tag, lines 7-8) coherent. The folds were wording-only on the I1/M1' sites.
5. **New FOLLOWUP id consistent + old narrow id zero occurrences — PASS.** Enumeration of all `technical-manual-*` tokens returns exactly `{transcript-lineref-staleness (line 9), ci-workflow-source-checkout (line 10), g2-uncovered-in-bare-ci (lines 186, 296)}`. No fourth token; old narrow id absent.

## Disposition

R0 has converged. The SPEC is internally consistent end-to-end after the Round-2 folds. Per the CLAUDE.md hard gate, implementation may proceed (apply Item-2a fix → `git rm` 4 transcripts → add workflow YAML → re-prove local 725/0 + `/tmp/ciclone` Test E → flip 2 FOLLOWUPs + file the gap FOLLOWUP). No further reviewer-loop round required.
