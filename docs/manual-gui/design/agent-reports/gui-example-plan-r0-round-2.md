# Plan-R0 review (round 2, SCOPED fold-fidelity) — `gui_example_tutorial` IMPLEMENTATION PLAN

- **Reviewer:** opus-tier architect (scoped round-2 plan-R0; house bar 0C/0I before ANY implementation).
- **Date:** 2026-07-05.
- **Scope (narrow, per the round-2 brief):** verify the round-1 folds (2I / 5m + the USER release-attach-only decision) landed faithfully; the spec amendment is surgical + the supersession sentence covers the un-rewritten restatements; the I1 census table is internally consistent with the P2.1 expectations + STOP tripwire; the P2.2→P2.1 label-correction is accurate not drift; and no NEW C/I from the text folds. Round-1 verified everything else (architecture, ordering, ratification fidelity) — NOT re-litigated here.
- **Artifacts under gate:** `IMPLEMENTATION_PLAN_gui_example_tutorial.md` (status: r1 folded 2026-07-05); `SPEC_gui_example_tutorial.md` §3.2(c) surgical amendment; against the prescriptions in `agent-reports/gui-example-plan-r0-round-1.md`.
- **Ground-truth:** both files read in full; exhaustive grep of every `gui_example.pdf` / `committed` / `release-attach` / `verify-examples-gui` / `P2.1|P2.2` occurrence in both files.

---

## VERDICT: **GREEN — 0 Critical / 0 Important. BUILD MAY BEGIN (P1.1).**

Both round-1 Importants and all 5 Minors are folded faithfully; the USER release-attach-only decision (which supersedes what round-1 had asked to be a flag-to-user *checkpoint* — the user resolved it outright) is threaded consistently across all six claimed sites and the spec §3.2(c) amendment; the I1 F1 census is internally consistent A1-row-by-row with the P2.1 intake expectations and the per-surface STOP tripwire; and the deliberate P2.2→P2.1 correction is **accurate — BLESSED**. No new Critical or Important surfaced from the text folds. Per the house gate, the standing pre-code prohibition is cleared: P1.1 (pin bump v0.74.0→v0.75.0) may begin.

---

## 1. FOLD FIDELITY — I2 (release-attach-only USER decision), 6 sites

Round-1 I2 asked for a flag-to-user checkpoint; the USER instead **decided** release-attach-only, so the fold is stronger than prescribed (decision landed, not deferred). Verified across all six claimed sites — each carries the `USER DECISION 2026-07-05, plan-R0 r1 I2` provenance:

| Site | Location | Fold |
|---|---|---|
| header cite | plan L3 + L5 | "§3.2c AMENDED 2026-07-05 by USER decision — release-attach-only, pointer-in-repo"; status line records "incl. the USER release-attach-only decision". ✓ |
| P2.2 path-filter MOOT | plan L136 | "path-filter decision is MOOT ... no `docs/gui_example.pdf` is committed ... tag pushes bypass `paths` regardless." ✓ |
| P3 release-attach-only + README pointers | plan L141 (heading "CI-built, RELEASE-ATTACH ONLY") + L143 | RELEASE-ATTACH ONLY, NOT committed; the toolkit README + `docs/manual-gui/README.md` pointers linking the latest `manual-gui-v*` asset are named as what "satisfies 'beside Examples.pdf'". ✓ |
| P4 sole-channel + recorded-deferral-only | plan L150 | "the release attach is now the SOLE distribution channel"; deferral "allowed ONLY as an explicitly recorded decision in the shipping PR, never an omission (spec R0 I3)". ✓ |
| risk note RESOLVED-by-user | plan L172 | "~16 MiB PDF weight question is RESOLVED by the 2026-07-05 USER DECISION ... never committed ... no per-corpus-change PDF re-commit ever enters git history." ✓ |
| spec §3.2c amendment | spec L287–310 | surgical; see §2. ✓ |

The stretched "spec-locked (Examples.pdf precedent)" framing round-1 flagged is **gone** — replaced by the weight-non-comparability reasoning (215 KB text PDF vs ~16 MiB raster) + the manual-gui-stack's own release-attach convention (`m-format-gui-manual.pdf`). Round-1 I2 fully discharged.

## 2. SPEC AMENDMENT SURGICALITY + SUPERSESSION COVERAGE

The amendment is confined to spec §3.2(c) (L287–310). The supersession sentence (L305–310) explicitly names the three un-rewritten "committed `docs/gui_example.pdf`" restatements. I grepped every occurrence and checked each against the supersession:

- **§3 architecture diagram box** (L146 `docs/gui_example.pdf (committed deliverable)`) — NAMED ✓
- **§8** (L567–568 `committed docs/gui_example.pdf refreshed in the shipping PR`) — NAMED ✓
- **§9 P4** (L595 `committed docs/gui_example.pdf`) — NAMED ✓

Does anything ELSE still read as "commit the PDF" that the supersession does NOT catch? Checked exhaustively:
- L29 (`sitting beside docs/Examples.pdf`) — a *location* statement, resolved by the README-pointer clause. Not a commit assertion.
- L285 (`from the pinned GUI clone, committed`) — refers to corpus figure/transcript copies, which ARE committed. Correct under the amendment.
- L593 §9 P3 (`the path-filter decision for docs/gui_example.pdf, §3.2c`) — a workflow *path-filter* reference that defers to §3.2c (now authoritative + MOOT); NOT a "commit the PDF" assertion. The plan's P2.2 (L136) additionally resolves it MOOT explicitly, so no implementer is misled. Within surgical-amendment tolerance — benign observation, not a gap.
- L653 §12.1 (`builds from committed, gated sources`) — "committed **sources**", not committed PDF. Correct and consistent under the amendment.

**Conclusion:** every phrase that genuinely restates "commit the *PDF*" is caught by the supersession; the residual `docs/gui_example.pdf` mentions are either correct-as-written (committed sources/corpus, location statement) or self-redirect to the authoritative §3.2(c). The amendment is surgical and the supersession is comprehensive. ✓

## 3. I1 CENSUS — INTERNAL CONSISTENCY (A1 row-by-row) + P2.1 + STOP

Round-1 I1 remedy (i)–(iv) all present in P1.3 (plan L55–88):
- (i) `gui_render_emit`/`gui_render_faithfulness` reclassified OUT of "verified inert": L74 (`gui_render_emit` — NOT verified inert: expected RE-PIN under A1) + L75 (`gui_render_faithfulness` — green under both shapes, NOT an oracle). The "Expected INERT" list (L72) correctly no longer contains `gui_render_emit`. ✓
- (ii) mini-R0 MUST-CHOOSE A1-vs-A2 with book-fidelity consequence: L58–60 (A1 "structural render DEPICTS the taught '(none)' unlock"; A2 "does NOT depict ... requires NAMING and ACCEPTING that gap"). ✓
- (iii) per-artifact-surface census + STOP in those terms: 5-row table L78–84 + per-surface STOP L86 ("ANY movement outside this table ... → back to the mini-R0"). ✓
- (iv) name the `gui_render_emit.rs:95` source edit as an expected P1.3 GUI-repo change: L74 + table row 2 (L81). ✓
- hint-text NULL: L61 + L73. ✓

**A1 row-by-row consistency (census table L78–84 ↔ P2.1 L128 ↔ STOP L86/L128):**

| Census row (A1) | P2.1 intake expectation (A1) | Consistent? |
|---|---|---|
| 61-form gallery PNGs → ≈0 inert (closed form) | "ZERO `figures/gui/` PNGs (closed-form gallery inert)" | ✓ |
| `gui_render_emit.rs:95` → RE-PIN in **P1.3** (GUI-repo test source) | (GUI-leg surface; not a toolkit-intake item — correctly absent from P2.1) | ✓ |
| `gui_render_faithfulness` → green, not a census signal | (not an intake surface) | ✓ |
| toolkit `transcripts/gui/mnemonic-export-wallet.gui` → RE-PIN on toolkit leg at **P2.1** | "exactly ONE `.gui` transcript (mnemonic-export-wallet.gui), regenerated by `verify-examples-gui` from the new pin" | ✓ |
| `expected_gui_schema_inventory.json` anchors → ZERO iff opts/projection untouched | "anchor delta EXPECTED ZERO either way — verify by running, mini-catch-up only if RED" | ✓ |

STOP tripwire is expressed identically on both sides (census L86 "ANY movement outside this table"; P2.1 L128 "Any ripple beyond the P1.3 census table = STOP"), both keyed to the P1.3 census table as the single authority. The A2 branch (every surface inert, at the named doc-honesty gap) is carried consistently in both places. **Internally consistent A1 row-by-row.** ✓

## 4. THE DELIBERATE P2.2 → P2.1 CORRECTION — **ACCURATE, BLESSED**

Round-1 (review L38/L71/L74, inheriting the plan's own r1 L68 label) placed the toolkit `.gui` re-pin in "P2.2". The fold relocated it to **P2.1** with an inline "no ordering change" note (census table row 4, plan L83). Verified against the plan's own phase contents:

- **P2.1 = "provenance + pin bump + corpus intake"** (L124). The pin bump lives here: L127 `pinned-upstream.toml [mnemonic-gui].tag v0.55.0 → v0.56.0`. The `verify-examples-gui` **re-run that regenerates the .gui** lives here: L128 "F1 ripple lands HERE ... regenerated by `verify-examples-gui` from the new pin."
- **P2.2 = "build plumbing + the three lint phases + chapter scaffolds"** (L131). Its only `verify-examples-gui` mention is a **gate re-run** — L139 "`verify-examples-gui` + `verify-figures-gui` green at the new pin" — i.e. a verification that it stays green, NOT a re-pin.

So the sub-phase that actually runs `verify-examples-gui` *after the pin bump to regenerate* the `.gui` is **P2.1**, not P2.2. The round-1 "P2.2" was itself the error (carried from the plan's r1 L68 label); the fold correctly identified and corrected it, and the "no ordering change" claim holds (the correction renames the phase, moves no step's execution order; the sequence pin-bump → `.gui` re-pin → later P2.2 build-plumbing is preserved). **Correction is accurate — BLESSED.** No drift introduced.

## 5. MINORS m1–m5 — all folded

- **m1** (branch-protection assert→verify): plan L15 "mnemonic-gui master branch-protection: VERIFY AT SHIP, do not assert ... verified master UNPROTECTED; the required-`snapshots` rule was added later — the current state is checked at ship time (plan-R0 r1 m1)." ✓
- **m2** (pilot shots 4 vs §9's "5"): plan L104 "4 follows the authoritative spec-§5.3 table; spec-§9's '5 shots' is a loose parenthetical — §5.3 governs. Plan-R0 r1 m2." ✓
- **m3** (FOLLOWUP forward-reference acknowledgement): plan L45 "acknowledged, not a defect (plan-R0 r1 m3)." ✓
- **m4** (pinned-upstream.toml `[mnemonic].tag` documentary-only): plan L43 "documentary-only cross-cite per the file's own header ... nothing reads it as load-bearing — plan-R0 r1 m4." ✓
- **m5** (`.gitignore` already covers `tutorial/**` recursively): plan L106 "the existing `tests/snapshots/**/*.new.png` / `*.diff.png` / `*.old.png` globs are recursive, not forms-scoped (verified at plan-R0 r1 m5); confirmed covered, no extension needed." ✓

## 6. NEW CRITICAL / IMPORTANT — none

Adversarial pass over the folded text found no new C/I:
- No self-contradiction reintroduced in the P1.3 gate lists (`gui_render_emit` cleanly moved from the INERT list L72 to its own RE-PIN-under-A1 line L74; `per-flag choices` correctly listed as a mini-R0 *confirmation* item under A1, not asserted inert).
- No P2.1/P2.2 re-pin contradiction (P2.1 regenerates; P2.2 only re-asserts green).
- The A2 doc-honesty gap remains explicitly named-and-accept-required, not silently dropped.
- The one benign residual (§9 P3's path-filter reference, L593) is within surgical-amendment scope and is independently resolved by the plan's P2.2 MOOT clause — a Minor observation at most, not blocking.

---

## GATE INSTRUCTION

**GREEN at 0C/0I. BUILD MAY BEGIN (P1.1 — pin bump v0.74.0→v0.75.0 + FOLLOWUP filings).** Both round-1 Importants discharged, the USER release-attach-only decision threaded consistently, the spec amendment surgical with a comprehensive supersession, the I1 census internally consistent A1-row-by-row with P2.1 + the per-surface STOP, the P2.2→P2.1 correction accurate and blessed, all 5 Minors folded. No re-dispatch required. Proceed to implementation under the standing per-phase R0 + post-impl cadence.

---

## CITES (verified this round)

- plan L3, L5 (header/status — USER decision provenance); L15 (m1); L43 (m4); L45 (m3); L58–61, L71–75, L78–86 (I1 P1.3 census + gate reclassification + 5-row table + hint-text NULL); L83 (P2.2→P2.1 correction, "no ordering change"); L104 (m2); L106 (m5); L124–128 (P2.1 pin bump + F1 ripple/`verify-examples-gui` regeneration + A1/A2 expectations + STOP); L131, L136, L139 (P2.2 build plumbing / path-filter MOOT / `verify-examples-gui` green-gate re-run); L141, L143 (P3 release-attach-only + README pointers); L150 (P4 sole-channel + recorded-deferral); L172 (risk note RESOLVED-by-user).
- spec L146 (§3 diagram box), L287–310 (§3.2c amendment + supersession), L567–568 (§8), L595 (§9 P4) — the named restatements; L593 (§9 P3 path-filter, benign residual); L653 (§12.1 "committed sources", correct).
