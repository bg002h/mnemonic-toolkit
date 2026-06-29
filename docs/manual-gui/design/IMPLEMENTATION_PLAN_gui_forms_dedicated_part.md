# IMPLEMENTATION PLAN — GUI forms dedicated Part

**Spec:** `SPEC_gui_forms_dedicated_part.md` (R0-GREEN, 2 rounds). **Status:** draft → plan-R0. Single-repo (manual-gui), single PR. Branch `feat/manual-gui-forms-part`.
**GOTCHAS:** local gates need `env RUSTUP_TOOLCHAIN=stable` (root pins 1.85 < GUI 1.88, for the `verify-examples-gui` regen step); build globs `src/**/*.md | LC_ALL=C sort` (new files auto-include); `make lint` runs the gate family.

## Phases (each: tests-first → impl → per-phase R0 to 0C/0I)

### P1 — the `gui-form-xref` gate (gate-first / TDD)
**Files:** `tests/check_gui_form_xref.py` (new, house-style, ~mirrors `check_outline_coverage.py`'s grep-family form — a MARKDOWN-SOURCE check over `src/`, NO `make html`, NO upstream-root needed, M-3) + wire into `tests/lint.sh` as a new phase, **renumbering the phase banners `N/7` → `N/8`** (M-3) + thread the transcripts path **from the Makefile** (M-4/M-7: `TRANSCRIPTS_GUI` ALREADY EXISTS at `Makefile:276` = `$(TRANSCRIPTS)/gui` — REUSE it in the `lint:` arg block, do NOT redefine; it's currently unexported so the `lint:` target must pass it through) + `.github/workflows/manual-gui.yml:106` picks it up free via `make lint`.
**The check (bidirectional, fail-closed):** enumerate the canonical stem list = the `*.gui` basenames in `transcripts/gui/` (key on the **FULL filename stem** — M-r2-3 — so `mnemonic-xpub-search-account-of-descriptor` is not mis-split at the first hyphen). For each stem `S`: assert EXACTLY ONE `{#gui-form-S}` anchor **across the gallery chapters (`src/75-gui-forms/`)** AND EXACTLY ONE `](#gui-form-S)` cross-link **in the subcommand chapters (`src/` EXCLUDING `src/75-gui-forms/`)** — M-1: scoping the link-count to NON-gallery chapters lets a future gallery-overview TOC/cross-ref list link each form without false-REDing the gate. Reverse/orphan clause: every `gui-form-*` anchor/link token must map to a real `.gui` stem. Exit 1 on any missing/extra/orphan, with the offending stem(s) named.
**Tests-first gate (prove it BITES before the content exists):** on the CURRENT tree (no gallery yet) the check must exit 1 reporting 61 missing anchors + 61 missing cross-links (the renders are still inline, no `gui-form-*` tokens). Add a self-test or a documented manual demonstration: introduce a typo'd cross-link in a scratch fixture → caught both directions. Confirm it does NOT false-trip on the tour's 2 inline includes.
**Gate:** the check runs in `make lint`, currently RED (expected — content lands in P2); `make lint`'s OTHER phases still green; the script is clean (no false positives on existing prose).

### P2 — build the gallery + rewire the 61 subcommand chapters (make the gate GREEN)
**Files:**
- New Part `src/75-gui-forms/`: `750-overview.md` (Part intro — what a structural render shows, the secret-`<masked>` note, AND the GENERAL convention that `(required)` markers are conditional-sourced — but NOT the 3 specific per-form constraints; those stay in their chapters) + `751-mnemonic.md` / `752-md.md` / `753-ms.md` / `754-mk.md`. Each new chapter **opens with an H1** (markdownlint, M-5). **Generate** each per-tab chapter from the stem list: per form a `## <tab> <sub> {#gui-form-<tab>-<sub>}` heading + a 1-line lead + the `include="gui/<tab>-<sub>.gui"` block. EVERY heading carries its explicit `{#gui-form-…}` (the auto-id-collision backstop, spec §3).
- The 61 subcommand chapters — **PRECISE removal (I-1):** remove ONLY the byte-uniform **lead-in paragraph + the 3-line render fence**; ADD the one-line cross-link `> **GUI form:** see [GUI Forms › <tab> › <sub>](#gui-form-<tab>-<sub>).` where the fence was. **NEVER touch the per-chapter `:::danger` secret block** (M-2) or any other prose. **Anchor the removal on the LEAD-IN paragraph (which appears exactly 61×, NEVER in the tour), not on the fence/include-path alone (M-6):** `mnemonic-bundle.gui`/`mk-inspect.gui` are ALSO included in the tour with an identical fence, and a fence/stem-keyed strip would silently remove a tour render — a GATE-INVISIBLE break (the xref still balances 1 anchor+1 link; census still counts 61 distinct `.gui`). Equivalently, exclude `src/30-tour/` from the edit. **Both the gallery anchors and the cross-links are generated from the SAME stem list** (single-sourced, spec §4) — script the edit, don't hand-type 122 sites.
- **The 3 caveat chapters are CARVED OUT of the blind script (I-1):** `40-mnemonic/4h-inspect.md:41` + `40-mnemonic/4i-repair.md:31` carry an **at-least-one** caveat (`--ms1`/`--mk1`/`--md1`); `60-ms/63-encode.md:28` carries a DIFFERENT **exactly-one / XOR** caveat (`--phrase`/`--hex`). These were verified precise by the prior cycle's R0 (mapped to real `conditional.rs` arms) and are render-referential ("the `(required)` markers … **above**"). For these 3: RETAIN the caveat block, but **reword its "above" reference** to point at the cross-linked gallery form (the render is no longer above). Do NOT delete/centralize them.
- The tour (`30-tour/31-first-launch.md`): UNCHANGED (keeps its 2 inline renders; `figures-cache-verify` unaffected — no mermaid in the gallery, M-5).
**Gate (all green):**
- `gui-form-xref` GREEN (61/61 anchors + 61/61 cross-links, no orphan).
- `gui-schema-coverage` GREEN (the `gui-form-*` anchors are prose-exempt; the subcommand schema/flag anchors are UNTOUCHED — verify the count is still 982 anchors / 61 subs).
- `verify-examples-gui` + census GREEN (the `.gui` files are unchanged; only `include=` sites moved — `env RUSTUP_TOOLCHAIN=stable make verify-examples-gui`).
- `outline-coverage` GREEN (gallery chapters carry no schema `<sub>` anchors → exempt; confirm no miscount), `glossary`/`index` GREEN, `markdownlint`/`cspell`/`lychee` GREEN.
- `make md` + `make html` + `make pdf` build clean; the gallery renders APPEAR (spot-check the built HTML: a `gui-form-*` heading + its render); a sample cross-link resolves to its gallery anchor in the built HTML.

### Post-impl + ship
- **Post-impl whole-diff review** (mandatory) over the whole `feat/manual-gui-forms-part` diff (the gate + 4 gallery chapters + 61 chapter rewrites).
- CHANGELOG `[Unreleased]` entry (the restructuring + the new `gui-form-xref` gate). FOLLOWUP `manual-gui-form-renders-dedicated-part` RESOLVED.
- PR → `manual-gui.yml` CI green → merge. (No GUI re-pin; same `mnemonic-gui-v0.53.0`. No new binary build in CI for this — but `verify-examples-gui` job 1c still builds the pinned `gui-render`; unaffected.)

## Risk / sequencing
- **P1 gate-first** is the TDD anchor: it must RED on the current tree (proving it measures the real invariant) before P2 makes it GREEN.
- **Single-source the 122 edits** from the stem list (a generator script) — hand-editing invites the exact typo the `gui-form-xref` gate exists to catch (so the gate + the generator are belt-and-suspenders).
- **No `.gui` content change, no GUI re-pin** — `verify-examples-gui` must remain byte-identical green (a RED there means an accidental render edit).
- Determinism + secret hygiene: unchanged (renders already mask; relocation is inert on content).
