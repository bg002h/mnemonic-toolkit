# `docs/manual-gui/` follow-ups

Per-cycle work items, deferrals, and resolved entries for the
`mnemonic-gui` user manual.

## Conventions

Each entry has a short ID (kebab-case), a Surfaced date, a Where
location, a What description, a Status (`open` | `resolved <commit>`
| `monitoring` | `won't fix`), and a Tier:

- **`v1`**: scheduled for the in-progress v1.0 cycle.
- **`v1.1+`**: deferred to a future minor cycle.
- **`cross-repo`**: depends on coordination with `mnemonic-gui` and/or
  sibling repos. Mirrored by a companion entry in the affected
  sibling's `FOLLOWUPS.md`.
- **`v2+`**: deferred indefinitely; revisit only at major-version cut.

---

## Open items

### `gui-manual-html-mermaid-svg`

- **Surfaced:** 2026-05-15 (M-P2.4 batch 2)
- **Where:** `pandoc/filters/mermaid-cache-filter.lua` + `tools/render-mermaid-cache.py`
- **What:** mermaid blocks render as `\includegraphics` in the PDF
  target but as plain `<pre><code class="mermaid">…</code></pre>` in
  the HTML target after the M-P2.4-batch-2 format-gate fix. Plain
  pandoc standalone HTML doesn't auto-load mermaid.js, so a reader of
  the GitHub Pages render sees the source rather than the diagram.
  Fix is to (a) extend `render-mermaid-cache.py` to also emit `.svg`
  alongside `.pdf` per cache entry, (b) extend
  `mermaid-cache-filter.lua` to emit `pandoc.RawBlock("html", '<img
  src="figures/cache/<sha>.svg" alt="...">')` (or inline-SVG) in the
  HTML branch.
- **Status:** open
- **Tier:** `v1` — should ship before `manual-gui-v1.0.0` so the
  GitHub Pages render is presentable.

---

## Resolved items

(populated as items close)
