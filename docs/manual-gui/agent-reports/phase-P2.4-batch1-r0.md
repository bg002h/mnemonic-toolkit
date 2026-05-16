# Phase P2.4 batch 1 (Track M — 00-frontmatter + 00-disclaimer) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1` (mnemonic-toolkit)
**Scope:** §3.2 P2.4 batch 1 — `docs/manual-gui/src/00-frontmatter.md` (NEW, 82 LOC), `docs/manual-gui/src/00-disclaimer.md` (NEW, 45 LOC), `docs/manual-gui/.cspell.json` (4-word append). All other §3.2 P2.4 batches deferred.

**Verdict:** **ITERATE 1C / 1I / 0N / 2n.**

The two new chapter files start at H1, the §1.4-mapped six-part chapter structure is coherent, the §2.5 version pins are byte-identical to `pinned-upstream.toml`, the manual-frontmatter URL in the markdown source matches the three other §2.4 pin sites. **But the HTML build pipeline silently mutilates the rendered manual:** `pandoc/filters/wrap-long-code.lua:65-70` unconditionally emits raw-LaTeX `\brktt{...}` for any inline-`Code` span with a non-space run ≥12 chars, regardless of output format; the html5 writer drops raw-LaTeX inlines. Result: every long-token code span in the frontmatter (including the canonical-example anchor URL that this chapter is supposed to publish as the contract URL for the §2.4 help-icon scheme) is silently elided from the rendered HTML. The markdown source is faithful; the rendered artifact at `build/m-format-gui-manual.html` is broken and would ship broken to GitHub Pages if M-P2.5 PE.6 fired today. This is a P0-inherited bug in the HTML build path that batch-1 is the first chapter content to expose.

The Track M / Track G byte-identity pin verification (claim 7) passes at the **source** level (4 sites agree byte-for-byte) but FAILS at the **rendered** level: the manual-side pin is **absent** from the built HTML, which makes the user-visible help-icon-contract section silently broken.

A secondary content issue: the "help-icon contract" section at `src/00-frontmatter.md:62-63` enumerates the Option-C affordance classes as "Dropdown / NodeValueComposite / repeating-field flag" — but the implemented `needs_help_icon` predicate at `mnemonic-gui/src/form/widget.rs:27-32` matches `Dropdown ∪ NodeValueComposite ∪ TaggedOrIndexed ∪ repeating`. The constellation has exactly one TaggedOrIndexed flag (`mnemonic export-wallet --taproot-internal-key` per plan §1.4 line 171), and it gets a `?` button per P2.2's GREEN implementation. The frontmatter prose understates the contract and would mislead a reader who clicks the `?` next to `--taproot-internal-key` and tries to look up its source in the manual.

---

## Critical

### C-1 — `wrap-long-code.lua` strips ALL ≥12-char code-span tokens from the HTML render

**Where:**
- Filter source: `/scratch/code/shibboleth/mnemonic-toolkit/docs/manual-gui/pandoc/filters/wrap-long-code.lua:65-70`
- Affected pipeline: `Makefile:177-189` — `make html` target loads `$(PDF_FILTER_ARGS)` (line 186) which includes `--lua-filter wrap-long-code.lua` (line 76).
- Concrete user-visible breakage in this batch (sample, from `build/m-format-gui-manual.html`):
  - Line 242: `<p> is the cross-platform desktop GUI overlay …`  ← `` `mnemonic-gui` `` (12 chars) stripped at the very first word of the chapter.
  - Line 250: `for the CLI surfaces themselves see the companion manual.` ← `` `mnemonic-toolkit` `` stripped.
  - Line 256–257: `tracking and the CLI versions` ← `` `mnemonic-gui v0.3.0` `` stripped.
  - Line 259: `each tag attaches` ← `` `manual-gui-v*` `` stripped.
  - Line 261, 354–355: `pushes the rendered HTML to .` / `re-deploys the HTML render to GitHub Pages.` ← the GitHub Pages URL is stripped both times it appears.
  - **Line 346: the canonical help-icon-contract example URL `https://bg002h.github.io/mnemonic-toolkit/manual-gui/#mnemonic-convert-from` is stripped entirely**, leaving the broken sentence `at the anchor for that exact flag — for the --from dropdown on mnemonic convert, for example.` This is the URL the user is supposed to recognise as the worked anchor; the rendered manual just elides it.
  - Line 354: `freshly-built PDF asset ()` ← `` `m-format-gui-manual-${MANUAL_GUI_VERSION}.pdf` `` stripped.
  - Line 355–357: `bumps ; new chapters bump ; copyedits and typo fixes bump . Independent of semver — a new release` ← `manual-gui-MAJOR`, `manual-gui-MINOR`, `manual-gui-PATCH`, and `mnemonic-gui` all stripped.
  - Line 359 (build banner): `Built from commit <code>e8f9ff9</code> on .` ← build timestamp `2026-05-15T18:16:04Z` stripped.

**Why:** `wrap-long-code.lua:65-70` reads

```lua
Code = function(el)
  if not has_long_run(el.text) then return nil end
  local escaped = tex_escape(el.text)
  return pandoc.RawInline('latex', '\\brktt{' .. escaped .. '}')
end,
```

with `has_long_run` returning true for any inline-Code whose `%S+` run is ≥`MIN_RUN=12` chars. The function unconditionally emits `pandoc.RawInline('latex', ...)`. In the PDF pipeline this is correct (`\brktt` is defined in `pandoc/preamble.tex` and `\seqsplit`-splits the token). In the HTML pipeline the html5 writer drops raw-LaTeX inlines, silently destroying the content. The filter must be format-gated.

The threshold `MIN_RUN=12` matters: `mnemonic-gui` is exactly 12 chars, which is the trigger for the very first code span on the chapter's first line. Anything shorter (`md`, `ms`, `mk`, `mnemonic`, `--from`, `?`, etc.) survives.

**This is P0-inherited.** `wrap-long-code.lua` was copied verbatim from `docs/manual/pandoc/filters/wrap-long-code.lua` during M-P0.4. The CLI manual has no `make html` target (per FOLLOWUP `cli-manual-html-target` at `design/FOLLOWUPS.md:57-64`), so the bug never fires in the CLI build. M-P0.5 added the `make html` target without auditing whether the existing filters were format-safe; M-P2.4 batch 1 is the first chapter with long code spans, so it is the first chapter to expose the breakage.

**Fix:** gate the `Code` and `CodeBlock` handlers in `wrap-long-code.lua` on `FORMAT:match("latex")` so they no-op on HTML and gfm pipelines. Pattern is already present in `primer-box.lua:28` and is the project-canonical idiom. After the fix, rebuild `make html`, re-grep the rendered HTML for the canonical URL `https://bg002h.github.io/mnemonic-toolkit/manual-gui/#mnemonic-convert-from` to confirm it survives.

**Lockstep blast radius:** the GUI's `?` buttons open URLs against `MANUAL_BASE_URL`. They will resolve to anchors that exist in the HTML (anchors are headers and headers are untouched per `wrap-long-code.lua:63`). So the deep-link contract still functions in-flow. But the manual that the deep-link lands on has visibly broken prose with missing tokens scattered across every chapter that uses long backticked code spans. P3 cycle-wide LOCK cannot pass without fixing this.

---

## Important

### I-1 — Help-icon-contract section understates affordance classes (omits TaggedOrIndexed)

**Where:** `docs/manual-gui/src/00-frontmatter.md:62-63`:
> Every Dropdown / NodeValueComposite / repeating-field flag in the GUI renders with a `?` button next to its label.

**Why:** the implemented Option-C predicate at `mnemonic-gui/src/form/widget.rs:27-32` is `Dropdown ∪ NodeValueComposite ∪ TaggedOrIndexed ∪ repeating`. The §1.4 inventory pins exactly one TaggedOrIndexed flag in the constellation: `mnemonic export-wallet --taproot-internal-key` (plan line 171), and the P2.2 R0 verification confirms the 91-button accounting includes this flag's `?`-icon. The frontmatter's prose drops TaggedOrIndexed and would mislead a reader who clicks the `?` next to `--taproot-internal-key` and tries to map the affordance class back to the documented contract.

**Fix:** rephrase to "Every Dropdown / NodeValueComposite / TaggedOrIndexed / repeating-field flag …". The plan §1.6 line 240 is the canonical phrasing.

---

## Nice-to-have

None.

---

## Nit

### n-1 — `.cspell.json` duplicates `rekt`

`docs/manual-gui/.cspell.json` line 90 (pre-existing) and line 174 (newly appended). `cspell` tolerates duplicate keys silently. Drop the new duplicate.

### n-2 — `redeploys` cspell entry does not match the prose (`re-deploys`)

`.cspell.json:173` declares `redeploys`. Prose at `00-frontmatter.md:76` uses `re-deploys` (hyphenated) → cspell splits → `re`, `deploys` both pass. Dead-code entry. Either drop or change prose to unhyphenated.

---

## Verification trace

1. **Heading level (claim 1).** Both H1. `src/00-frontmatter.md:1` = `# About this manual`; `src/00-disclaimer.md:1` = `# Read this first — UNTESTED ALPHA SOFTWARE`. Matches AUTHORING.md:13-19. PASS.

2. **`:::primer` / `:::danger` use (claim 2).** Neither file uses an admonition. AUTHORING.md:146-148 says `:::danger` is required when a chapter introduces/first uses the seed; the frontmatter only declares the convention (does not run a worked example), so the looser reading accepts it. The first actual `:::danger` will appear in 30-tour or 40-mnemonic. PASS (accepted reading).

3. **Explicit `{#anchor-id}` IDs (claim 3).** Per-subcommand-shape chapters require explicit IDs; top-level chapter files use pandoc-auto-slugs which `tests/lint.sh::gui-schema-coverage` does not check. PASS.

4. **Forward-link validity (claim 4).** Bare prose cross-manual refs; lychee can't trip. FOLLOWUP `gui-manual-cross-refs-to-cli-manual` tiered `v1.1+`. PASS.

5. **Six-part chapter structure (claim 5).** Six rows map 1:1 to §1.4 lines 184-194. PASS.

6. **Version pinning (claim 6).** All five pins byte-identical to `pinned-upstream.toml` lines 19, 22-25. PASS.

7. **Four pin sites byte-identical (claim 7).** **Source-level PASS; render-level FAIL — block on C-1 fix.** All four sources agree, but the manual-side pin is stripped from `build/m-format-gui-manual.html` by C-1.

8. **CLI frontmatter copy-faithfulness (claim 8).** Two-track audience preserved + GUI-adapted; living-document phrasing parallel; six-row table GUI-shaped; MIT preserved. PASS.

9. **Disclaimer copy-faithfulness (claim 9).** UNTESTED ALPHA framing preserved; "GUI-affordance UX feedback" added to acceptable; "Why this page is here" extended with GUI OS-integration concerns. PASS.

10. **Index marker hygiene (claim 10).** `\index{mnemonic-gui}` + `\index{m-format constellation}` at frontmatter lines 3-4. `tests/lint.sh:132` warn-skips while `99-index-table.md` is absent. PASS (warn-skip is intentional batch-1 state).

11. **cspell additions (claim 11).** `affordance` + `affordances` needed; `redeploys` dead-code (n-2); `rekt` duplicates baseline line 90 (n-1). PASS modulo n-1 / n-2.

---

## Prose-command execution

The reviewer's tool-environment lacked Bash. Inferences below rest on direct reads of the build artifact + lint-driver source:

- `make lint` phases 1-3: PASS based on `tests/lint.sh:55-81` + new files' shape.
- Phases 4-5: RED at P1 baseline 459/59 (no batch-1 delta to schema/outline coverage).
- Phases 6-7: WARN-skip per `tests/lint.sh:118-127,131-155` (90-appendices absent).
- `make html`: ran (artifact present at `build/m-format-gui-manual.html`, fresh mtime). Build succeeded but produced the corrupted prose documented in C-1.
- Spot-check: all 9 expected pandoc-auto-anchors present in built HTML at lines 202, 212, 220, 226, 241, 262, 278, 342, 352. Anchor layer intact — only inline code-spans mutilated.

---

## Final verdict

**ITERATE 1C / 1I / 0N / 2n.**

C-1 (`wrap-long-code.lua` HTML mutilation) must fix before any batch-1 sign-off; without it, every chapter with long backticked tokens — and in particular the very chapter (00-frontmatter) that publishes the canonical help-icon-contract URL — ships visibly broken. The fix is small (format-gate the `Code` and `CodeBlock` handlers on `FORMAT:match("latex")` per the `primer-box.lua:28` idiom) and unblocks the rest of M-P2.4.

I-1 (TaggedOrIndexed omission) is a content-precision fix per plan §1.6 line 240.

n-1 / n-2 are cspell cleanups.

After the C-1 fix + I-1 reword + the two cspell cleanups, batch 1 is ready for R1 architect-review. The plan §3.5 per-batch reviewer-LOCK gate (0C/0I) requires the C-1 fix before promotion to batch 2 (`10-foundations/`).
