# R0 review — IMPLEMENTATION_PLAN_gui_forms_dedicated_part.md (round 1)

**Reviewer:** opus architect (mandatory plan-R0 gate, 0C/0I).
**Artifact:** `design/IMPLEMENTATION_PLAN_gui_forms_dedicated_part.md`
**Spec:** `design/SPEC_gui_forms_dedicated_part.md` (R0-GREEN, 2 rounds).
**Source verified against:** manual-gui @ toolkit `master` (working tree). All file:line cites below are live greps, not spec snapshots.

## Verdict

**RED — 0 Critical / 1 Important / 5 Minor-Nit.**

The plan's TDD gate-first structure, single-source generator, anchor/exempt reasoning, placement, and gate-compatibility analysis are all sound and source-confirmed. One Important defect: the removal instruction for the "at-least-one caveat" prose is factually wrong against source and, executed literally, destroys correct per-subcommand documentation and/or leaves dangling spatial references that no gate catches. Fixable with a scoped plan amendment (carve out 3 named chapters); does not require re-opening spec R0.

---

## Critical

None.

---

## Important

### I-1 — The "remove the at-least-one caveat prose, centralized in 750-overview.md" instruction is wrong against source; literal execution loses verified documentation or leaves dangling references.

**Where:** Plan P2 bullet 2 — *"REMOVE the include block (+ its render-specific lead-in/at-least-one caveat prose, now centralized in `750-overview.md`)"*; Plan P2 bullet 1 — *"the at-least-one `(required)` caveat stated ONCE here"*. Inherited from SPEC §2 (*"the at-least-one caveat, stated ONCE here instead of per-form"*) and §4.

**The facts (source-verified):** A full scan of all 61 subcommand renders for prose appearing between the render's closing fence and the next heading finds **exactly 3** chapters with render-referential caveat prose — and they are **not** a single uniform "at-least-one caveat":

- `src/40-mnemonic/4h-inspect.md:41` — *"**At-least-one input (not a conjunction).** The `(required)` markers on `--ms1` / `--mk1` / `--md1` above are conditional-sourced…"*  (at-least-one, 3 cards)
- `src/40-mnemonic/4i-repair.md:31` — identical at-least-one wording (`--ms1`/`--mk1`/`--md1`)
- `src/60-ms/63-encode.md:28` — *"**Exactly-one input (not a conjunction).** The `(required)` markers on `--phrase` and `--hex` above are conditional-sourced…"*  (**XOR / exactly-one**, different flags)

These three:
1. Are **heterogeneous** — two distinct constraint kinds (at-least-one vs exactly-one/XOR over different flag sets). They cannot be "stated ONCE" in `750-overview.md` without information loss.
2. Are **render-referential** — each says "the `(required)` markers … **above** are conditional-sourced". After the render moves to the gallery Part, "above" dangles (no gate catches a prose spatial reference: lychee `--offline` skips `#`-fragments, cspell/markdownlint/pandoc do not flag it).
3. Are **not derivable from the stem list** — they are hand-written, varied, and (for the 3) sit *after* the render block; the plan's "script the edit from the one stem list" cannot find or correctly rewrite them.
4. Are **verified-correct, load-bearing documentation.** The prior cycle's own R0 record establishes this: `gui-form-renders-p5-r0-round-1.md:90-95` proves the caveat lands on *exactly* {inspect, repair, ms encode}, that inspect/repair map to `three_way_card_at_least_one` (`conditional.rs:926-960`) and ms-encode to the mutually-exclusive `ms_encode` (`conditional.rs:730-746`), and that the manual *correctly distinguishes* XOR from at-least-one and *correctly omits* the caveat on conjunctive groups (verify-bundle, md-compile, etc.). `...leg2-postimpl-whole-diff-review.md:66` records the same. So "centralize once / delete" would erase documentation a prior gate verified as precise.

**Why Important (not Critical):** affects 3/61 chapters, breaks no build/gate/funds/secret surface, and is fully fixable by a scoped plan edit. But literal execution of the plan as written yields a real user-facing doc-correctness regression that no gate detects — exactly the "interleaved with non-render-specific prose" hazard the review brief flags.

**Required fix (plan amendment):**
- Narrow the scripted removal to **only** the byte-uniform lead-in paragraph + the 3-line include fence (see §"Verified sound" — this part is genuinely mechanical and safe across all 61).
- **Carve out** `4h-inspect.md`, `4i-repair.md`, `60-ms/63-encode.md` from the "caveat" removal. **Retain** their caveat blocks in place; **reword** each block's "above" spatial reference to point at the cross-linked gallery form (e.g. "the `(required)` markers in the [GUI form](#gui-form-…)"). Do **not** centralize these into `750-overview.md`.
- Correct the plan/§ wording: there is no single "at-least-one caveat" — there are two kinds (at-least-one ×2, exactly-one/XOR ×1). `750-overview.md` may carry the *generic* explanation of conditional-`(required)` marker semantics, but the per-subcommand specific caveats stay in-chapter.

---

## Minor / Nit

### M-1 — Scope the `gui-form-xref` cross-link count to the subcommand chapters (exclude the gallery Part), or an overview cross-reference list trips the gate.
The check asserts "EXACTLY ONE `](#gui-form-S)` cross-link" (Plan P1; SPEC §6 says "in the subcommand chapters"). If a future `750-overview.md` adds a gallery TOC/cross-reference list (a natural thing), a whole-`src/` scan would count 2 links/stem → false RED. Spell out in P1 that the link-count scan excludes `src/75-gui-forms/` (anchor-count and link-count are already token-shape-disjoint — `{#…}` vs `](#…)` — so only an overview link list is at risk). Note also: bound each grep on the full delimited token (`{#gui-form-S}` / `](#gui-form-S)`) so no stem that is a textual prefix issue arises (none of the 61 stems is a strict prefix of another, and the closing delimiter makes it exact — confirm in the check).

### M-2 — Be explicit that the removal target is the lead-in paragraph + fence ONLY, never the per-chapter `:::danger` secret block.
Each chapter's secret-hygiene `:::danger` admonition (e.g. `42-bundle.md:12-24`, `4h-inspect.md:24-31`) sits *above* the uniform lead-in and is **not** render-specific (it's worked-example secret hygiene + the run-confirm `••••` modal note, distinct from the render's `<masked>` note). The scripted lead-in+fence removal won't touch it, but the plan should state this boundary so the implementer doesn't fold it into "render-specific prose".

### M-3 — `lint.sh` step labels are hard-coded `N/7`; adding the phase makes it 8.
Cosmetic: renumber the `step "X/7 …"` strings (lint.sh:56,64,76,84,99,113,130) to `/8` when inserting the new phase. Place the new phase as a **markdown-source** check (like outline-coverage, reads `src/` + the stem list) — it does **not** need `make html` and does **not** need `MANUAL_GUI_UPSTREAM_ROOT`; it needs `SRC_DIR` (already passed) + `TRANSCRIPTS_GUI`.

### M-4 — `TRANSCRIPTS_GUI` plumbing: confirm the derive path.
M-r2-1 offers "pass `TRANSCRIPTS_GUI=$SRC_DIR/../transcripts/gui` or derive it." Verified: `SRC_DIR = $(MANUAL_DIR)/src`, so `$SRC_DIR/../transcripts/gui = $(MANUAL_DIR)/transcripts/gui = $(TRANSCRIPTS)/gui` — correct. Cleaner: add `TRANSCRIPTS_GUI="$(TRANSCRIPTS_GUI)"` to the Makefile `lint:` arg block (Makefile:296-305, the var already exists at line 276) and parse it in lint.sh. Either works in CI (`manual-gui.yml:106` runs `make lint`, which passes `SRC_DIR`); the chain to the new gate is confirmed free.

### M-5 — `figures-cache-verify` and markdownlint H1 not addressed in P2's gate list.
`make lint` has `lint: figures-cache-verify` (Makefile:237). The 4 new gallery chapters contain no mermaid blocks (only `include=` renders) → no cache delta → unaffected; worth a one-line "no mermaid in gallery" note. Also ensure each new chapter opens with an H1 (`# …`) so markdownlint MD041/MD001 pass under the gallery `## <form>` headings.

---

## Verified sound (no action)

1. **P1 gate-first RED holds.** Current tree has **zero** `gui-form-*` tokens (`grep -rn 'gui-form-' src/` → empty). So on the current tree the check finds 0 anchors + 0 links for all 61 stems → exits 1 with 61 missing anchors + 61 missing cross-links. The renders are inline today (61 subcommand `include=` blocks + 2 tour = 63 total `include="gui/`), carrying no `gui-form-*` token. The "prove it bites before content exists" claim is real.
2. **Check is grep-implementable, house-style.** Per stem `S` (full filename stem, M-r2-3): exactly-one `{#gui-form-S}` + exactly-one `](#gui-form-S)` + orphan direction (every `gui-form-*` token maps to a stem). Simpler than `check_outline_coverage.py` (no bullet counting). Full-stem keying + delimiter bounding handles `mnemonic-xpub-search-account-of-descriptor` correctly.
3. **Tour cannot perturb counts (M-r2-2).** `src/30-tour/31-first-launch.md:21,82` are bare `include="gui/mnemonic-bundle.gui"` / `gui/mk-inspect.gui` — neither a `{#gui-form-*}` nor a `](#gui-form-*)` token. Each stem keeps exactly one gallery anchor + one cross-link.
4. **Per-tab split exact:** mnemonic 32 / md 10 / ms 10 / mk 9 = 61 (`ls transcripts/gui/`), matching SPEC §2's 751-mnemonic(32)/752-md(10)/753-ms(10)/754-mk(9). Census `EXPECTED_GUI_RENDER_COUNT=61` (Makefile:277) unchanged.
5. **Anchor exemption bulletproof (SPEC §3 confirmed at source).** `is_schema_shaped` (`check_gui_schema_coverage.py:101-115`) is prefix-anchored (`anchor == shape or anchor.startswith(shape + "-")`); no tab is named `gui`, so every `gui-form-*` gallery anchor is exempt from Direction B. The explicit `{#gui-form-…}` on every heading is mandatory and confirmed: a forgotten anchor on a `## <tab> <sub>` heading auto-derives a schema-shaped id → orphan → fail-closed (the one backstop). Removing the renders drops no `id=` (renders are fenced code, carry none; Direction A is satisfied by prose) → `gui-schema-coverage` stays green.
6. **`outline-coverage` unaffected.** `scan_markdown` collects all `{#anchor}` headings but `expected_outlines` only looks up `*-outline` anchors; `gui-form-*` headings are collected-but-never-queried. The gallery carries no `<sub>-outline` → no obligation. For all 61 source chapters the `## …-outline`/next heading sits intact after the lead-in+fence removal (58 chapters have the heading immediately after; the 3 in I-1 have the caveat then the heading — preserve both).
7. **`verify-examples-gui` byte-identical.** `.gui` files unmoved; only `include=` sites relocate. `include-transcript.lua` resolves via absolute `TRANSCRIPTS_DIR` (Makefile:89) independent of the including file's location → moved includes resolve in `make md/html/pdf`.
8. **Placement `75-` correct.** `find … | LC_ALL=C sort` (Makefile:67) orders `75-gui-forms/` after `70-mk/` and before `80-troubleshooting/` (both dirs confirmed present); no Makefile/SOURCES edit. `75-` (with the CLI-reference Parts) vs `85-` (after troubleshooting) is editorial; the plan's choice is reasonable and harms no cross-reference.
9. **`750-overview.md` passes clean.** Prose-only Part intro: no `*-outline` obligation (outline-coverage is schema-sub-keyed), no glossary obligation (glossary-coverage greps the glossary file, lint.sh:118-124), no schema anchor. Needs only an H1 + cspell-clean prose.
10. **CI chain free.** `manual-gui.yml:106` `make lint …` → Makefile `lint` → `lint.sh` with `SRC_DIR` → new phase. `MANUAL_GUI_UPSTREAM_ROOT` env-set (manual-gui.yml:61); the new gate needs neither it nor the HTML build.
11. **Phasing right.** 2 phases (gate RED → content GREEN) is the correct TDD coupling; post-impl whole-diff review is placed; P2 is mechanical enough for one phase (generator + 4 gallery chapters + 58 uniform rewrites + the 3 hand-edits from I-1).

---

## Bottom line

Re-dispatch after folding **I-1** (carve out + retain + reword the 3 caveat chapters; correct the "single at-least-one caveat" framing) and ideally the Minors. Everything else is GREEN and source-confirmed. Convergence expected in one fold.
