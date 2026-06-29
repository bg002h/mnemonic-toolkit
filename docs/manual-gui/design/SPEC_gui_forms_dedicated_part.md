# SPEC — extract the 61 GUI form renders into a dedicated "GUI Forms" Part

**Status:** draft → R0. **Tier:** docs (manual-gui restructuring). **No funds/secret surface change** (the renders already mask secrets; this only RELOCATES them).
**Source:** manual-gui @ toolkit `master` (post-`06348ff1`, the shipped per-subcommand-embedded renders). User-chosen layout: **a new Part** (one chapter per tab) + **a cross-link pointer** left in each subcommand chapter.

## 1. Goal
Move the 61 generated GUI form renders out of the per-subcommand chapters into a dedicated **Part — "GUI Forms Reference"** (4 chapters: mnemonic / md / ms / mk). Each subcommand chapter keeps a one-line cross-link to its form. The CLI-output transcripts (`.out`, gated by `verify-examples`) and all prose stay in the subcommand chapters. The `verify-examples-gui` fidelity gate + census are preserved unchanged (placement-agnostic). The 30-tour walkthrough KEEPS its 2 inline renders (narrative context).

## 2. New Part structure
- New directory **`src/75-gui-forms/`** (the build globs `find src -name '*.md' | LC_ALL=C sort`, so a `75-` prefix auto-orders AFTER `70-mk/` and before `80-troubleshooting/` — no Makefile/SOURCES edit; placement = "after the per-tab CLI reference parts, before troubleshooting/appendices"). R0 may move to `85-` (after troubleshooting) — placement is an R0/user-tunable detail.
- 4 chapters, one per tab, each opening with a Part/chapter intro then a section per subcommand form:
  - `75-gui-forms/751-mnemonic.md` (32 forms), `752-md.md` (10), `753-ms.md` (10), `754-mk.md` (9).
  - Per form: a `## <tab> <sub> {#gui-form-<tab>-<sub>}` heading + a 1-line lead (what the form does) + the `include="gui/<tab>-<sub>.gui"` block. Optionally a part-overview chapter `750-overview.md` (intro + what a structural render shows + the secret-masking note + the at-least-one caveat, stated ONCE here instead of per-form).

## 3. The anchor convention — THE load-bearing gate constraint
`gui-schema-coverage` (`tests/check_gui_schema_coverage.py`) is **bidirectional**: (A) every schema `<tab>-<sub>`/`<tab>-<sub>-<flag>` needs exactly one `id="..."`; (B) every HTML anchor that *looks* schema-shaped (`<tab>-<sub>`…) must map to a real schema entry — **prose anchors are exempt by construction (only schema-shaped anchors are orphan-checked)**.
- **RULE: the GUI-forms form-headings use PROSE-shaped anchors `gui-form-<tab>-<sub>` (with the `gui-form-` prefix), NEVER the schema-shaped `<tab>-<sub>`.** This avoids BOTH (i) a duplicate-ID collision with the subcommand chapter's existing `id="<tab>-<sub>"` (pandoc would auto-suffix → broken links + a possible Direction-A miss) AND (ii) orphan-flagging by Direction B (the `gui-form-` prefix makes the anchor non-schema-shaped → exempt). The subcommand chapters' schema-anchors are UNTOUCHED (the renders carry no anchors; only prose does), so Direction A stays satisfied.
- **R0-r1 VERIFIED bulletproof:** `is_schema_shaped` (`check_gui_schema_coverage.py:101-115`) is **prefix-anchored** (`anchor == shape or anchor.startswith(shape + "-")`), NOT substring; no tab is named `gui`, so no schema shape prefixes `gui-form-…` → all 61 gallery anchors exempt. Direction A is satisfied by prose alone TODAY (the gate is green with the renders present, which carry no `id=`), so removing them drops no expected id.
- **EVERY gallery heading MUST carry its explicit `{#gui-form-<tab>-<sub>}`** — a heading `## mnemonic bundle` WITHOUT it auto-derives `id="mnemonic-bundle"` → a schema-shaped ORPHAN that FAILS the gate (this is the one fail-closed backstop; the `gui-form-xref` check §6 adds the leading one).

## 4. Per-subcommand chapter edits
- **Remove** the `include="gui/<tab>-<sub>.gui"` fenced block (currently ~line 28 of each subcommand chapter) + its render-specific lead-in/caveat prose.
- **Add** a one-line cross-link where the render was, e.g.:
  `> **GUI form:** see [GUI Forms › mnemonic › bundle](#gui-form-mnemonic-bundle).`
  Target `#gui-form-<tab>-<sub>` — **SINGLE-SOURCED**: both the gallery heading anchor (§2) AND this cross-link target are derived from the one canonical stem list = `ls transcripts/gui/*.gui` (`<tab>-<sub>`). The implementation generates the 61 gallery sections and the 61 cross-links from that list (not hand-typed twice), so the two ends agree by construction; the `gui-form-xref` check (§6) is the gate that PROVES it (closes I-r1-1).
- **Keep** all prose, flag-reference anchors (the schema anchors), and the `.out` CLI-output transcript include. The subcommand chapter is now prose + CLI output + the form cross-link.

## 5. Tour handling
`30-tour/31-first-launch.md` KEEPS its 2 inline renders (`include="gui/mnemonic-bundle.gui"`, `gui/mk-inspect.gui`) — the walkthrough needs the form shown in context. A `.gui` may be included in BOTH the tour and the gallery (`include-transcript.lua` embeds the file; no single-inclusion constraint; the census gates file EXISTENCE, not inclusion count). The tour's inline renders carry NO `gui-form-*` heading anchor (they're mid-prose), so no collision. **They also cannot perturb the `gui-form-xref` counts (§6):** that gate keys on `{#gui-form-*}` anchors and `](#gui-form-*)` links — the tour's bare `include="gui/…"` matches neither token, so each stem still has exactly one gallery anchor + one cross-link (M-r2-2).

## 6. Gate-compatibility (must all stay green)
- **`verify-examples-gui`** — placement-agnostic: it regenerates the `transcripts/gui/*.gui` with the pinned `gui-render` + diffs == committed + census 61. The `.gui` FILES are unchanged; only their `include=` sites move. GREEN by construction. (CI job 1c untouched.)
- **`gui-schema-coverage`** — GREEN via §3 (prose-shaped gallery anchors exempt; subcommand schema-anchors untouched; the cross-link is a normal markdown link, not an anchor).
- **`outline-coverage`** — keyed on `<sub>-outline` anchors in subcommand-documenting chapters; the gallery chapters carry NO schema `<sub>` anchors (only `gui-form-*`), so they need no per-form outline. R0 verifies the gallery chapters don't accidentally trip an outline requirement; add a chapter-level outline if the gate expects one per top-level chapter.
- **NEW GATE `gui-form-xref` (closes I-r1-1 — the cross-link verification hole).** lychee runs `--offline` with NO `--include-fragments` (no `.lychee.toml`), so it SKIPS the bare `#gui-form-*` intra-doc fragments; pandoc/LaTeX don't hard-fail dangling internal links; `include-transcript.lua` fail-closes only on bad `.gui` STEMS, not bad cross-link fragments. So WITHOUT a dedicated check, a typo on any of the ~122 edit sites ships a dead link no gate catches. Add a small (~15-line, house-style) **bidirectional** lint phase (`tests/check_gui_form_xref.py`, wired into `tests/lint.sh` + `manual-gui.yml`): enumerate the canonical stem list `transcripts/gui/*.gui`; assert for each `<tab>-<sub>` — (i) EXACTLY ONE `{#gui-form-<tab>-<sub>}` anchor exists across the gallery chapters, AND (ii) EXACTLY ONE `](#gui-form-<tab>-<sub>)` cross-link exists in the subcommand chapters; AND no `gui-form-*` anchor/link exists without a matching `.gui` stem (orphan direction). Fail-closed (exit 1 on any missing/extra/orphan). This makes the 61 cross-links a GATED invariant, not a by-construction hope.
- **`lychee`** (`--offline` over `src/`) + **`markdownlint`** + **`cspell`** — the 61 cross-links + 4 new chapters must pass; cross-link anchors deterministic (§3). `make md`/`html`/`pdf` build clean (fail-closed `include-transcript.lua` would FAIL on a wrong `include=` stem — so a moved render that points at a bad stem breaks the build, a free correctness check).
- **`glossary` / `index`** bidirectional — the gallery chapters add no glossary/index obligations beyond what already exists (the subcommands are already documented); confirm no new orphan.

## 7. Non-goals
No change to: the `.gui` render CONTENT or the `gui-render` binary (Leg-1, untouched); the CLI-output `.out` transcripts; the schema-anchor/flag-reference structure; the separate CLI manual (`docs/manual/`). Not a visual/screenshot change. Not re-pinning the GUI (no `pinned-upstream.toml` change — same `mnemonic-gui-v0.53.0`).

## 8. Tracking
FOLLOWUP `manual-gui-form-renders-dedicated-part` (RESOLVED at ship). Companion CHANGELOG `[Unreleased]` entry. Builds on the just-shipped `manual-gui-generated-form-renders`.
