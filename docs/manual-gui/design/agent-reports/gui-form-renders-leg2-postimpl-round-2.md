# GUI-form-renders cycle — Leg-2 POST-IMPLEMENTATION round-2 convergence review

**Scope:** tight, narrow re-review confirming the two round-1 Importants (both docs-only, plan-scoped "at ship") are now folded correctly, with no drift. The implemented artifact (61 renders + `verify-examples-gui` gate + CI job 1c) was already GREEN in round-1 and is NOT re-litigated here — only the two owed ship-records.
**Branch:** `feat/manual-gui-form-renders` @ `9da835a0` (tip; fold commit).
**Fold under review:** `9da835a0` — `docs(manual-gui): Leg-2 post-impl I1+I2 — CHANGELOG P5 entry + FOLLOWUPS records`.
**Reviewer:** opus architect (independent; verified with REAL tool calls — read both folded files, the tour source, the parent-tree diff, and the cspell config/baseline).
**Date:** 2026-06-29

---

## VERDICT: **GREEN — 0 Critical / 0 Important**

Both round-1 Importants are folded completely and accurately, with no drift and no overclaim. **The leg is PR-ready.** Two non-blocking Minors are noted below for optional polish; neither gates ship and neither affects any technical gate.

---

## Critical

**None.**

---

## Important

**None.** Both round-1 Importants are resolved:

### I1 (CHANGELOG P5 headline) — FOLDED, accurate
`docs/manual-gui/CHANGELOG.md` `[Unreleased]`:
- **Header reframed to the whole leg**, not just P4: "*generated, gated GUI form renders in the manual + a GUI pin catch-up … Leg 2 of the generated-GUI-form-renders cycle. The manual now SHOWS the real GUI…*" (lines 11-20). The marquee feature is now the lede, exactly as I1 demanded.
- **P5 bullet group present and correct** (lines 39-62): the 61 generated structural renders (`transcripts/gui/<tab>-<sub>.gui`, emitted by the pinned headless `gui-render` from `mnemonic-gui-v0.53.0 --no-default-features`, embedded via `include="gui/<tab>-<sub>.gui"`, `<masked>` sentinel, seeds defaults as the GUI does on load); the `verify-examples-gui` fail-closed gate (regen with the **pinned** `gui-render` + `diff` == committed + census 61 + independent secret-unmask scan; `tests/verify-examples-gui.sh` + Makefile target + `manual-gui.yml` **job 1c**); the tour-mockup replacement **with the specific drift it fixed** (`--template bip84` vs the real `bip44`, stale slot-row count); the at-least-one `(required)` caveat landing on exactly `{inspect, repair, ms encode}`; and the closing "lint 7/7 / verify-examples / verify-examples-gui / HTML+PDF build clean" line.
- Every factual claim matches round-1's independently-verified ground truth (pinned tag, job 1c, census 61, secret-scan, the {inspect, repair, ms-encode} required-set). **No inaccuracy, no overclaim.** The release job's `--notes-file CHANGELOG.md` will now ship the feature that defines the cycle.

### I2 (FOLLOWUPS — all three records) — FOLDED, accurate
`docs/manual-gui/FOLLOWUPS.md`:

**(a) `manual-gui-output-blocks-non-gateable-residual` Status narrowed** (lines 110-124): now reads "**partially RESOLVED (2026-06-29) — the FORM-MOCKUP leg is closed**," explicitly names the two replaced mockups (`30-tour/31-first-launch.md:17`, `:87`), credits the gate, notes the mockups "had silently DRIFTED … vindicating the gate," points to the new `manual-gui-generated-form-renders` entry, and **correctly narrows the remainder to still-won't-fix** (output-panel `argv:`/`exit:`/`stdout:` framing, run-confirm modal, help-icon `?` snippets, ellipsized/truncated illustrations, URL-formula, input-paste echoes — none of which are form structure, so `gui-render` does not cover them). The narrowing rationale is sound and not overclaimed.
  - **Verified the Status is TRUE, not overclaim:** the tour source now carries `include="gui/mnemonic-bundle.gui"` (line 21) and `include="gui/mk-inspect.gui"` (line 82) at the original mockup positions; a box-drawing/checkbox grep over `31-first-launch.md` finds only prose describing the `◀` active-tab convention — no surviving hand-drawn full-window form fence. The two mockups really were replaced by gated renders.

**(b) `manual-gui-generated-form-renders` RESOLVED entry filed** (lines 151-166): accurate — all 61 forms, headless `gui-render` shipped in `mnemonic-gui-v0.53.0 --no-default-features`, schema + `conditional()` default-seeding, committed under `transcripts/gui/`, `include=`-embedded, gated by `verify-examples-gui` (regen + `diff` == committed + census 61 + secret-unmask scan, fail-closed, `manual-gui.yml` job 1c), `<masked>` sentinel, cross-repo Leg-1/Leg-2 framing. Consistent with round-1's verified facts; `RESOLVED 2026-06-29`.

**(c) Cross-repo `gui-word-card-from-help-mislabels-secret-input` OPEN entry filed** (lines 127-145, marked `CROSS-REPO → mnemonic-gui`): accurately characterizes the GUI footgun — the GUI schema's `word-card --from` help calls it a "BIP-39 mnemonic" (`phrase=`/`ms1=`/`entropy=`) with `secret: false`, whereas `word-card` is PUBLIC-only (`mk1`/`md1`, `ms1` excluded), so the mislabel "invites a user to paste a seed phrase into an unmasked, no-run-confirm field." Correctly states **the manual is correct and does NOT propagate the bad help string** (no overclaim, matches round-1's highlighted finding verbatim in substance). `Status: open (cross-repo, GUI-side)`, `Tier: secret-hygiene`, with a `Companion:` line directing the matching `mnemonic-gui/FOLLOWUPS.md` entry — satisfying the CLAUDE.md cross-repo convention's toolkit-side obligation.

---

## No-drift ruling — CONFIRMED

- `git show --stat 9da835a0` → **exactly two files**: `CHANGELOG.md` (+34/−5) and `FOLLOWUPS.md` (+50/−8). The gated artifact (the 61 `.gui` renders, `verify-examples-gui.sh`, the Makefile target, `manual-gui.yml` job 1c, the lint suite, the schema/inventory) is **untouched** by the fold. Round-1's GREEN technical verdict stands unchanged.
- Working tree is **clean** (`git status --untracked-files=no` empty); branch is `feat/manual-gui-form-renders`. (Untracked `??` files exist only OUTSIDE `docs/manual-gui/` — other cycles' recon/plan files — as round-1 already noted; none are part of this leg.)

## Lint ruling — STILL GREEN (the fold cannot RED cspell/markdownlint)

The two files ARE linted by cspell + markdownlint, so a genuinely-new unknown word would RED. **The fold introduced ZERO new vocabulary:**
- A local cspell pass over the post-fold files flags only `roff`, `hyperref`, `goldens`, `gateable`, `ellipsized`/`Ellipsized`. **Every one of these pre-exists in the parent tree (`471661b9`)** that round-1 verified cspell-GREEN (`cspell 0 / 87 files`): `roff` (parent CHANGELOG:30, P4 gen-man), `hyperref` (parent CHANGELOG:104, the older 1.0.1 entry), and `goldens`/`gateable`/`ellipsized` (parent FOLLOWUPS:48,75,113 — the entry has shipped through gated PRs since 2026-06-23). These pass the real lint via the `.cspell.json` backtick/fence `ignoreRegExpList` in their real contexts; the local oracle is not config-faithful, but the **pre-existence argument is decisive**: the real lint already accepts these words on a tree it passed.
- Critically, the local pass flagged **nothing** in the fold's genuinely-new text — the new cross-repo entry (lines 127-145) and the resolved entry (lines 151-166) produced zero flags. The new Status prose reuses the same `goldens`/`ellipsized` vocabulary the prior won't-fix Status already carried.
- markdownlint: the fold reuses the file's existing heading/bullet/line-wrap structure; no new construct. Implementer reports `make lint` 7/7; consistent with inspection. (The full `make lint` schema/lychee phases need the sibling GUI checkout + network and are unaffected by these two prose files in any case.)

---

## Minor / Nit (non-blocking — do NOT gate GREEN)

1. **Internal-consistency residue in `manual-gui-output-blocks-non-gateable-residual`.** The fold corrected the **Status** (the authoritative current-state field) to mark `:17`/`:87` resolved and narrow the remainder, but left the historical **What** body intact: the Class-1 list (lines 59-60) still enumerates `31-first-launch.md:17`/`:87` as "have no binary to diff against," and the summary still reads "27 residual non-gateable" (line 108-109) rather than 25. Round-1's I2.1 had suggested striking the two lines and decrementing 27→25. The fold met the SUBSTANCE (Status accurately records the resolution + narrowing, conservatively, no overclaim) via a Status amendment instead. A reader landing on the Class-1 list without reading the Status would see stale data. Optional one-line polish: decrement to 25 and annotate the two list lines "(RESOLVED — now gated; see Status)". **Not Important:** the governing Status is correct and conservative, there is no overclaim, and no gate is affected.
2. **Cross-repo companion not in this worktree's scope.** The CLAUDE.md convention mirrors the cross-repo entry in BOTH repos' `FOLLOWUPS.md`. The toolkit-side record (which this PR owns) is present and accurate; its `Companion:` line directs the `mnemonic-gui/FOLLOWUPS.md` entry, which lands with the separate GUI-side cycle (as the entry itself frames). Track that the `mnemonic-gui` companion is filed in lockstep; out of scope for this leg's diff.

---

## Bottom line

Both round-1 Importants are **folded completely and accurately, with no drift and no overclaim**: the CHANGELOG now leads with the P5 headline (61 gated renders + `verify-examples-gui` + job 1c + the tour replacement and its fixed drift), and all three FOLLOWUPS records are present and faithful (residual narrowed + form-mockup leg RESOLVED; the new RESOLVED renders entry; the cross-repo word-card secret-hygiene footgun, with the manual correctly noted as already-correct). The fold touched only the two docs files; the gated artifact is unchanged; cspell/markdownlint stay green because no new vocabulary was introduced. **GREEN (0C/0I) — the leg is PR-ready.** Branch left clean (no edits made by this review).
