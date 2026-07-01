# R0 PLAN review — GUI visual screenshot track — round 3 (2026-07-01, scoped fold-fidelity check)

**Artifact:** `docs/manual-gui/design/IMPLEMENTATION_PLAN_gui_visual_screenshot_track.md` (post-fold of round-2's 0C/1I/4m).
**Round 2:** `agent-reports/visual-track-plan-r0-round-2.md`. **Charter (narrow):** verify the five round-2 folds (I7 ×2 edits, m7, m8, m9, m10) landed verbatim-in-substance, hunt fold-introduced drift in the surrounding passages only. Round-2's verified-faithful list (I1–I6, m1–m6, drift hunt) is settled and was not re-litigated.

## Verdict: GREEN — 0 Critical / 0 Important

All five folds are faithful to their prescriptions and introduce no new drift. **The plan is converged. P0 may begin.**

---

## Fold verification — ALL FAITHFUL

- **I7 ✓ (both edits, disposition fully restored).**
  - *Edit 1 (P0 trigger paragraph, line 10):* now reads "`workflow_dispatch` in the spike file is inert pre-merge (default-branch rule) and DIES WITH THE FILE in P1 (I7) — the ladder's dispatch-regen workflow stays DEFERRED-DOCUMENTED." The round-2-flagged "may sit in the file as a post-merge convenience" clause is GONE — nothing any longer points toward merging the spike workflow. Matches the prescription (the added "(I7)" tag and capitalization are cosmetic).
  - *Edit 2 (P1 file list, line 23):* "**Spike-artifact removal (I7):** delete `tests/spike_form_snapshots.rs` + the P0 spike workflow file in P1 (the rasterizer recipe is ABSORBED into `build.yml`'s `snapshots` job); nothing spike-named reaches the merge." Matches the prescription; names both artifacts; states both halves.
  - *Absorb+remove pair coherent:* line 22's build.yml item carries the absorb half ("P0's proven rasterizer recipe"); line 23 carries the remove half. Together nothing spike-named survives the merge, so the P0 dispatch trigger can no longer be mistaken post-merge for the ladder's deferred dispatch-regen workflow — the ladder-disposition sentence in line 24 ("DEFERRED-DOCUMENTED — post-merge-only by construction … NOT a P1 deliverable and nothing in P1 cites it as existing") stands unambiguous and uncontradicted. Line 10's mention cites it as *staying deferred*, not as existing — consistent. The P0 spike *report* (`visual-track-p0-spike.md`) lives toolkit-side and is not "spike-named content reaching the GUI merge" — no tension.
- **m7 ✓** — Line 24's red-corpus sequence now reads "flipping the PR's own job to `UPDATE_SNAPSHOTS=1` **and adding a corpus-upload-artifact step** (m7 — the failure-path `.diff.png` upload never fires on a passing UPDATE run)". Rationale included; the upload step rides the same TEMPORARY commit, so "revert the flip" removes it too — sequence stays mechanical end-to-end (flip+upload → download artifact → commit corpus → revert → never merge red).
- **m8 ✓** — P2 step 0 (line 31) uses the explicit URL: `git ls-remote https://github.com/bg002h/mnemonic-gui mnemonic-gui-v0.54.0 | cut -f1` inside the `gh api …/check-runs` command, with the m8 annotation naming the wrong-remote hazard ("Leg 2 executes in the toolkit repo where `origin` is mnemonic-toolkit"), the fail-closed property ("empty SHA → API error ≠ success"), and the lightweight-tag/`^{}` note. Mechanics re-checked: missing tag → empty SHA → malformed API path → error ≠ `success` (fail-closed as stated); lightweight tag → commit SHA directly → check-runs resolves; ls-remote tail-matching finds `refs/tags/mnemonic-gui-v0.54.0` from the bare name. Sound.
- **m9 ✓** — Line 35's resolution space now opens "(m9 — BOTH branches need a consumer-config alignment; no bare form works alone)" and distributes the alignment over both branches: file-relative form "+ an alignment for pandoc-HTML/xelatex", OR manual-root-relative "+ an alignment: `--resource-path` … `\graphicspath{{../}}` … or copy-figures-into-`build/`". The round-2 misreading (file-relative needing no alignment) is no longer expressible. Coherent with the surrounding "no bare path form satisfies all 5 consumers" + decide-in-phase + embed-census-as-gate framing, all unchanged.
- **m10 ✓** — Line 26's release-commit item carries "**m10: once the required-`snapshots` rule is live, a protected master rejects direct pushes of unchecked commits — the release commit rides the standing PR flow or an explicit admin bypass, decided at ship**", placed exactly where the surprise would fire (between the branch-protection ship-step and the tag). Chain ordering unchanged and coherent: merge → protection ship-step → release commit (PR flow / admin bypass) → tag → verify tag-push snapshots run.

## Drift hunt (surrounding passages)

- **P1 file-list ordering coherent:** promotion refactor → snapshot test → corpus → build.yml job (absorb) → spike removal (remove, adjacent to its absorb site) → red-corpus mitigation → branch-protection. The insertion reads naturally; no cross-reference broken.
- **P0 section internally consistent:** step 3 still describes the spike workflow neutrally ("The CI workflow (PR-event, per above)"); its fate is now carried once in line 10 and once in line 23 — no residual "convenience"/"throwaway" ambiguity in either direction.
- No other passage touched by the five edits changed meaning; gate lines, GOTCHAS, Leg-2 items, and the Risk section are byte-wise consistent with round-2's verified state.

## Critical

None.

## Important

None.

## Minor / Nit (non-blocking, optional)

- **n1 — Stale status line (bookkeeping, pre-existing omission — NOT edit-introduced).** Line 3 still reads "**Status:** draft → plan-R0 round-2" and cites only the r1 fold. A one-line refresh ("plan-R0 round-3 GREEN; r2 (1I/4m) folded — `agent-reports/visual-track-plan-r0-round-2.md`") would keep the audit trail self-describing. May ride the P0 commit; does not gate.

---

## Gate disposition

**GREEN at 0C/0I.** The round-2 blocker (I7) is fully resolved — the spike-artifact absorb+remove disposition is restored in both places, the misleading clause is dead, and the ladder-disposition sentence is unambiguous. All four minors folded faithfully with no new drift. The reviewer loop has converged. **P0 (the spike on real runners) may begin.**
