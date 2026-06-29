# Post-implementation whole-diff review — `feat/manual-gui-forms-part`

**Reviewer:** opus architect (independent, adversarial)
**Scope:** whole-diff of `feat/manual-gui-forms-part` (`42f48551` P1 gate + `3b4fb1ee` P2 content) off `master@8a51277b` — the "GUI Forms" dedicated-Part restructuring of `docs/manual-gui/`.
**Date:** 2026-06-29
**Worktree reviewed:** `.claude/worktrees/agent-ae9c2222917aef982` (branch checkout).

---

## Verdict

**GREEN — PR-ready on substance: 0 Critical / 0 Important.**

The restructuring is correct, complete, and surgically scoped. All five live gates pass; the 61 renders are byte-identical to the pinned `gui-render`; the new `gui-form-xref` gate is sound in both directions and keyed on the full multi-hyphen stem; no `:::danger`/admonition or unrelated prose was touched; the tour is unchanged. Two ship-checklist deliverables enumerated in the plan are not yet in the branch (CHANGELOG `[Unreleased]` entry + the `manual-gui-form-renders-dedicated-part` FOLLOWUP RESOLVED record) — both are TAG/ship-gated, not PR-open-gated, and are listed under **What's owed before PR** as Minor/process. They do not block opening the PR but MUST land before merge/tag.

---

## Critical

None.

## Important

None.

## Minor / Nit

1. **(Owed, not a defect) CHANGELOG `[Unreleased]` entry for THIS leg is absent.** `docs/manual-gui/CHANGELOG.md` `[Unreleased]` currently describes only the *predecessor* leg (the generated-form-renders embed + GUI pin catch-up). The restructuring + the new `gui-form-xref` gate added by this branch have no CHANGELOG line. `IMPLEMENTATION_PLAN_gui_forms_dedicated_part.md:29` lists this as a deliverable. The manual CHANGELOG is gated by `changelog-check` on the **tag**, not on PR-open, so this is owed-before-tag, not a PR blocker.
2. **(Owed, not a defect) FOLLOWUP `manual-gui-form-renders-dedicated-part` has no FOLLOWUPS.md record.** The slug is referenced in `SPEC_gui_forms_dedicated_part.md:43` and `IMPLEMENTATION_PLAN_…:29` as "RESOLVED at ship", but `docs/manual-gui/FOLLOWUPS.md` contains no such entry (only the predecessor `manual-gui-generated-form-renders` — already RESOLVED 2026-06-29 at line 159). Per the standing discipline (flip FOLLOWUP status in the shipping commit), the entry should be created-and-RESOLVED in the ship commit. Owed-before-merge.
3. **Nit (informational, no action):** the tour (`30-tour/31-first-launch.md`) keeps inline renders for `mnemonic-bundle` and `mk-inspect`, both of which ALSO have a gallery entry — so those two `.gui` files are now included at two sites. This is intentional per SPEC §5 and inert (the gate ignores bare `include=` fences; `verify-examples-gui` compares per-`.gui`-file regardless of include-site count). No change needed.

---

## Gate-bites re-verification (all run locally, `RUSTUP_TOOLCHAIN=stable` [rustc 1.95.0], `MANUAL_GUI_UPSTREAM_ROOT=/scratch/code/shibboleth/mnemonic-gui`)

`env RUSTUP_TOOLCHAIN=stable make -C docs/manual-gui lint …` — **8/8 GREEN**:

| Phase | Result |
|---|---|
| 1/8 markdownlint | 0 errors (92 files) |
| 2/8 cspell | 0 issues (92 files) |
| 3/8 lychee `--offline` | 1942 total, **0 errors**, 5 excluded |
| 4/8 gui-schema-coverage | **OK: 982 schema anchors (61 subcommands) all present** — UNCHANGED by removing renders ✓ |
| 5/8 outline-coverage | OK: 129 outlines, correct bullet counts |
| 6/8 glossary-coverage | OK |
| 7/8 index bidirectional | OK |
| 8/8 **gui-form-xref** | **OK: 61 forms each with 1 gallery anchor + 1 cross-link (0 duplicates, 0 orphans)** ✓ |

`make md` / `make html` build clean (pandoc; only the benign `--self-contained` deprecation warning).

**Built-HTML anchor audit** (`build/m-format-gui-manual.html`):
- `id="gui-form-*"`: **61 unique / 61 total** — no pandoc auto-id collision (no `-1` suffixes). ✓
- `href="#gui-form-*"`: 61 unique, 122 total (= 61 in-body cross-links + 61 auto-TOC entries); **0 dangling** (every href resolves to an id). ✓
- Tab/reference anchors all resolve: `gui-forms-reference` (1 id / 5 href), `gui-forms-{mnemonic,md,ms,mk}` (1 id / 2 href each). ✓

**`gui-form-xref` soundness — perturbation matrix** (run on a scratch copy of `src/`; worktree left pristine, all perturbations restored):

| Perturbation | Expected | Observed |
|---|---|---|
| typo a cross-link (`…-bundle)` → `…-bundlex)`) | RED | exit=1 "cross-link(s) missing" ✓ |
| delete a gallery anchor (`{#gui-form-md-encode}`) | RED | exit=1 "gallery anchor(s) missing" ✓ |
| duplicate a cross-link (2nd `ms-encode` link) | RED | exit=1 "cross-link(s) duplicated" ✓ |
| orphan cross-link to non-existent stem | RED | exit=1 "orphan cross-link(s)" ✓ |
| orphan gallery anchor to non-existent stem | RED | exit=1 "orphan gallery anchor(s)" ✓ |

The gate is not a rubber stamp: it catches missing/typo'd/duplicate/orphan in BOTH directions. It keys on the **full filename stem** (`canonical_stems = p.stem`; regex `[a-z0-9-]+`), so multi-hyphen subs like `mnemonic-xpub-search-account-of-descriptor` are never mis-split — the baseline GREEN includes that stem. Cross-links are correctly scoped to NON-gallery chapters (0 cross-links inside `src/75-gui-forms/`), and the tour's bare `include=` fences carry no anchor/link token so they cannot perturb counts (verified: `grep gui-form- src/30-tour/` → none).

---

## Byte-identical-renders confirmation

`env RUSTUP_TOOLCHAIN=stable make -C docs/manual-gui verify-examples-gui` →
**`OK (61/61 renders match the pinned gui-render; no secret leak)`.**
`git diff master..feat/manual-gui-forms-part -- docs/manual-gui/transcripts/` is **empty** — no `.gui` source file changed. The relocation is inert w.r.t. render content; secret hygiene is unchanged (renders already mask classified-secret fields as `<masked>`; no cross-link or prose inlines key material).

---

## Caveat-correctness ruling

All three caveat chapters retain their distinct constraint semantics — the distinction is **preserved, not flattened**:

- `40-mnemonic/4h-inspect.md` — **at-least-one** ("`--ms1`/`--mk1`/`--md1` … required only until you fill *any one* … Supply at least one"). ✓ `:::danger` block untouched.
- `40-mnemonic/4i-repair.md` — **at-least-one** (identical wording). ✓ `:::danger` untouched.
- `60-ms/63-encode.md` — **exactly-one / XOR** ("`--phrase` and `--hex` … required only until you fill *one*. Provide a phrase **or** raw hex (mutually exclusive), not both"). ✓ The `Disabled`/`Hidden` conditional prose above it is untouched.

Each "above" reference was reworded from "the `(required)` markers … **above**" to "… **in the GUI form linked above**", correctly redirecting the reader from the (now-removed) inline render to the gallery cross-link directly preceding the caveat. No other prose in these three files changed.

Whole-diff surgical-scope proof: across all 61 subcommand chapters the residue of removed non-{fence,lead-in,caveat-reword} lines is **empty**, and the residue of added non-{cross-link,caveat-reword} lines is **empty**. Exactly 61 `include="gui/…"` fences removed from the subcommand chapters, 0 added there (all 61 now live only in the gallery); 0 `:::`-fence lines added or removed anywhere. The tour diff is empty and it still carries its 2 inline renders.

Gallery completeness: `751-mnemonic` 32 forms, `752-md` 10, `753-ms` 10, `754-mk` 9 = 61, each `## \`<cmd>\` {#gui-form-<tab>-<sub>}` with its `include` fence; matches the 61 `.gui` stems exactly. `750-overview.md` reads well, states the masking + all-`abandon` test-vector caveat, names the three conditional-required forms (inspect/repair/ms encode) consistently with the caveat chapters, and adds no glossary/index obligation (its only terms — `<masked>`, `(required)`, `[disabled]`, control kinds — are explained in-place).

---

## What's owed before PR / merge

1. **Add a `docs/manual-gui/CHANGELOG.md` `[Unreleased]` entry** for the dedicated-Part restructuring + the new `gui-form-xref` lint phase (plan deliverable `IMPLEMENTATION_PLAN_…:29`). TAG-gated by `changelog-check`; add before merge so the tag is clean.
2. **Create + RESOLVE the `manual-gui-form-renders-dedicated-part` FOLLOWUP** in `docs/manual-gui/FOLLOWUPS.md` (referenced by SPEC:43 / PLAN:29; flip in the shipping commit per standing discipline).

Neither is a code/content defect and neither blocks opening the PR; both are routine pre-merge/pre-tag checklist items. No other action required.

---

## Bottom line

GREEN. The implementation matches the R0-GREEN plan, all gates bite and pass, the new gate is genuinely sound, the 61 renders are byte-identical, and the change is surgically scoped with the caveat semantics preserved. Fold the two owed checklist items (CHANGELOG entry + FOLLOWUP RESOLVED) into the PR before merge and ship.
