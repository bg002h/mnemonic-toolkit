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

### `manual-gui-output-blocks-non-gateable-residual`

- **Surfaced:** 2026-06-23 (Cycle-C P4 output-fidelity)
- **Where:** `src/30-tour/*`, `src/40-mnemonic/*`, `src/50-md/*`,
  `src/60-ms/*`, `src/70-mk/*` (output-shaped fences NOT wired to a
  `transcripts/<stem>.out` golden).
- **What:** P4 wired the 19 real-CLI-output worked-example fences to
  captured goldens (verified against the GUI-pinned CLI tier —
  toolkit-v0.70.0 / md-cli-v0.7.0 / ms-cli-v0.8.0 / mk-cli-v0.9.0 —
  by `make verify-examples`). The remaining 27 output-shaped `text`/
  `json` fences are **structurally non-gateable** — they are not raw
  CLI stdout/stderr — and are left as authored. They fall into five
  classes:

  1. **GUI window/screen ASCII mockups** (`+---+` box-drawing, `☑`/`☐`
     checkboxes, `▾` dropdown carets, `◀` active-tab markers, `••••`
     redaction sentinels, `←`/`↑` annotation arrows). These render the
     GUI chrome, not CLI output, and have no binary to diff against:
     - `30-tour/31-first-launch.md:17` (three-panel layout)
     - `30-tour/31-first-launch.md:87` (mk-tab form)
     - `30-tour/31-first-launch.md:116` (mk1 paste-field input echo)
     - `30-tour/31-first-launch.md:127` (live `Preview:` line)
     - `30-tour/32-run-and-output.md:20` (output-panel render — GUI
       `argv:`/`exit:`/`stdout:` framing, not raw stdout)
     - `30-tour/32-run-and-output.md:68` (output-panel non-zero-exit
       render — GUI `exit:`/`stderr:` framing)
     - `30-tour/32-run-and-output.md:103` (convert form mockup)
     - `30-tour/32-run-and-output.md:112` (run-confirm modal)
     - `30-tour/32-run-and-output.md:152` (output-panel `exit:`/`stdout:`
       framing of an `ms1` run — the raw value IS gated via
       `44-convert-phrase-to-ms1.out`, but the panel framing is GUI-drawn)
     - `30-tour/33-help-icons-and-deep-links.md:36` (per-subcommand `?`)
     - `30-tour/33-help-icons-and-deep-links.md:54` (per-flag `?`)
     - `30-tour/33-help-icons-and-deep-links.md:71` (slot-row `?`)
  2. **Ellipsized / truncated illustrations** (`...` / `…` placeholders
     standing in for full strings — diffing would fail by construction):
     - `40-mnemonic/42-bundle.md:391` (bundle `--json` schema —
       `"ms1": ["ms10entrsq..."]` etc.)
     - `40-mnemonic/44-convert.md:597` (xpub multi-output — `bc1q...`)
     - `40-mnemonic/47-final-word.md:153` (final-word `--json` schema —
       `"candidates": ["abandon", "ability", "above", "..."]`)
     - `40-mnemonic/4d-xpub-search-account-of-descriptor.md:261`
       (`xpub1.../0/*, ...`)
     - `40-mnemonic/4e-build-descriptor.md:303` (`…/<0;1>/*`,
       `#<checksum>` placeholders)
     - `40-mnemonic/4e-xpub-search-passphrase-of-xpub.md:255` (`xpub6...`)
     - `40-mnemonic/4f-xpub-search-path-of-xpub.md:223` (`xpub6...`)
     - `40-mnemonic/4g-xpub-search-address-of-xpub.md:146` (`bc1q...`)
     - `70-mk/73-encode.md:201` (`mk1qpydzkpqqsq…` — intentionally
       truncated; prose notes the per-invocation `chunk_set_id` makes the
       exact prefix non-deterministic, so a fixed golden is impossible)
  3. **URL-composition / formula illustrations** (not command output):
     - `30-tour/33-help-icons-and-deep-links.md:104` (URL formula)
  4. **Canonical-data / input-paste reference blocks** (the strings a
     user PASTES into a form field, or a chapter's "worked-example data
     convention" listing — inputs, not outputs):
     - `40-mnemonic/4e-xpub-search-passphrase-of-xpub.md:264`
       (candidates-file match line — requires an out-of-band file
       fixture; the always-emitted advisory at `:232` IS gated)
     - `50-md/51-overview.md:65` (canonical md1 reference listing)
     - `60-ms/61-overview.md:102` (canonical phrase reference)
     - `60-ms/61-overview.md:108` (canonical ms1 reference)
     - `60-ms/62-inspect.md` / `63-encode.md` / `64-decode.md` /
       `65-verify.md` / `69-derive.md`, `70-mk/71-overview.md:83`,
       `72-inspect.md` / `74-decode.md` paste-field input fences (the
       canonical ms1/mk1/phrase the step says to paste — these echo the
       INPUT; the matching OUTPUT fence in each chapter IS gated)
  - The total = 46 output-shaped fences: 19 gated (P4), 27 residual
    non-gateable (this entry).
- **Status:** won't fix — these are GUI chrome, schema illustrations,
  truncated examples, or input echoes by nature; gating them would
  require either rendering the GUI (out of scope for a doc-build) or
  fabricating deterministic goldens for non-deterministic / ellipsized
  content. Re-triage only if a future cycle adds a GUI-render harness
  (would let the `30-tour/*` mockups be screenshot-diffed).
- **Tier:** `v1.1+`

---

## Resolved items

(populated as items close)
