# v0.3.1 End-of-Phase Architect Review r1

**Date:** 2026-05-05
**Scope:** end-of-phase review of `mnemonic-toolkit` v0.3.1 (`tr(K, sortedmulti_a(...))` unblock via `[patch.crates-io]`)
**Plan:** `design/IMPLEMENTATION_PLAN_v0_3_1_sortedmulti_a.md`
**Files reviewed:** workspace `Cargo.toml`, `crates/mnemonic-toolkit/Cargo.toml`, `crates/mnemonic-toolkit/src/parse_descriptor.rs` (full), `design/SPEC_mnemonic_toolkit_v0_3.md` (§4.9.a, §4.10, §11), `design/FOLLOWUPS.md` (`tr-sortedmulti-a-via-upstream`), `CHANGELOG.md` (v0.3.1 entry)

**Verdict: 0C / 0I / 1L — green light for commit + tag + release.**

---

## Critical (0)

None.

## Important (0)

None.

## Low (1)

**L-1 — SPEC §4.9.a Layer 2 had a duplicate `Terminal::Multi` bullet.**

`design/SPEC_mnemonic_toolkit_v0_3.md` §4.9.a Layer 2 (the "Already handled by md-cli's walker" list) contained two consecutive bullets for `Terminal::Multi(thresh)` (lines 129 and 130). Copy-paste duplicate introduced during the v0.3.1 Layer 2 patch. Does not affect implementation or build; SPEC-prose nit only.

**Resolution:** fixed inline as part of the r1 verdict fold-in (deduplicated to a single `Carried from v0.2` bullet) before commit.

---

## Findings by scrutiny area

**1. Walker refactor fidelity.** All three plan changes landed correctly.
- `walk_wsh` (line 380): calls `walk_miniscript_node(w.as_inner(), km, false)` directly. Matches plan.
- `walk_wsh_inner`: confirmed deleted.
- `walk_sh` `ShInner::Wsh` arm: calls `walk_miniscript_node(w.as_inner(), ...)`. Architect r1 I-1 satisfied.
- `walk_sh` `ShInner::SortedMulti` arm: replaced by post-#915 explanatory comment.

**2. New `Terminal::SortedMulti` arm correctness.** Lines 482-487 and 495-500 use `&thresh.data().iter().collect::<Vec<_>>()` verbatim, matching the existing `Terminal::Multi` / `Terminal::MultiA` arms and satisfying architect r1 I-2 exactly.

**3. `wsh(sortedmulti(...))` regression.** `arm_sorted_multi_via_wsh` directly asserts `Tag::SortedMulti` via the new Layer-2 arm. `walk_wsh_sortedmulti_root` and `walk_sh_sortedmulti_root` cover both wrappers. v0.2 fixture matrix provides implicit byte-equality assurance. Coverage sufficient.

**4. SPEC patch surface coverage.** All five plan-required locations patched:
- §4.9.a Layer 1: three retracted bullets replaced with post-#915 prose.
- §4.9.a Layer 2: `Terminal::SortedMulti` + `Terminal::SortedMultiA` arms added.
- §4.10: deferral parenthetical removed; now "supported in v0.3.1".
- Final-note: "v0.3.0 historical note (retracted in v0.3.1)" framing present.
- Revision Round 8: detailed entry at §11.

L-1 nit was the only deviation found.

**5. CHANGELOG entry shape.** Sections present and in order: What's new / Mechanism / Future cleanup / Wire-bit-identical / Test corpus / Out of scope / Architect-review history. Matches v0.3.0 and v0.2.0 shape.

**6. FOLLOWUP reframe.** `tr-sortedmulti-a-via-upstream` correctly reclassified to `v0.3.2`. v0.3.2 cleanup steps documented with `gh api` watch command.

**7. Hidden v0.3.1 risks.** All three (force-push, miniscript 13.0.1 conflict, cargo publish blocker) acceptable for a patch release.

**8. Wire-bit-identical claim.** 159 unit tests + 2 ignored, 0 failed. v0.2 fixture matrix SHA pin in CHANGELOG. `descriptor_tr_sortedmulti_a_matches_template_tr_sortedmulti_a_md1` provides strongest signal.

**9. Plan adherence.** Steps 0-7 complete. No skipped steps.

**10. Tag-push blockers.** None.

---

**Architect-review history (cumulative for v0.3.1 cycle):**

| round | scope | verdict |
|---|---|---|
| r1 | sketch review (proposal before formal plan) | 0C / 3I / 4L → 5 action items folded into formal plan |
| r2 | formal plan re-review | 0C / 1I / 2L → 3 doc-fixes folded inline (Step 0 rev verification, Step 4 §6.8 grep sub-bullet, Step 3 schematic-import note) |
| r3 (this) | end-of-phase implementation review | 0C / 0I / 1L → L-1 duplicate `Terminal::Multi` bullet fixed inline |

**Go for tag.** Commit the working tree (stage paths explicitly per `feedback_avoid_git_add_all`), tag `mnemonic-toolkit-v0.3.1`, push master + tag, ship GitHub release with notes from CHANGELOG v0.3.1.
