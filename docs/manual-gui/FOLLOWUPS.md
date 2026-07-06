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
     - `30-tour/31-first-launch.md:17` (three-panel layout) — **RESOLVED
       2026-06-29: now a gated `include="gui/mnemonic-bundle.gui"` render**
     - `30-tour/31-first-launch.md:87` (mk-tab form) — **RESOLVED
       2026-06-29: now a gated `include="gui/mk-inspect.gui"` render**
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
  - The total = 46 output-shaped fences: 19 gated (P4-output cycle),
    **2 form mockups now gated as generated renders (2026-06-29, the
    form-mockup leg — see Status)**, 25 residual non-gateable (this
    entry; was 27).
- **Status:** **partially RESOLVED (2026-06-29) — the FORM-MOCKUP leg is
  closed.** The generated-GUI-form-renders cycle (Leg 2 P5) added a
  headless `gui-render` + the `verify-examples-gui` gate, so the two
  full-window FORM mockups (`30-tour/31-first-launch.md:17`, `:87`) are
  now replaced by generated, gated structural renders — and every
  subcommand form additionally carries its render, gated against drift.
  (These mockups had silently DRIFTED from the real GUI before
  replacement, vindicating the gate.) See
  `manual-gui-generated-form-renders`. The REMAINDER is narrowed +
  remains **won't fix** for now — the output-panel `argv:`/`exit:`/
  `stdout:` framing, the run-confirm modal, the help-icon `?` snippets,
  the ellipsized/truncated illustrations, the URL-formula, and the
  input-paste reference blocks are NOT form structure, so `gui-render`
  does not cover them; gating them would need an output-panel/chrome
  render harness or deterministic goldens for non-deterministic content.
- **Tier:** `v1.1+`

### `gui-word-card-from-help-mislabels-secret-input` (CROSS-REPO → mnemonic-gui)

- **Surfaced:** 2026-06-29 (generated-GUI-form-renders Leg-2 post-impl review).
- **Where:** `mnemonic-gui/src/schema/mnemonic.rs` (the `word-card` `--from`
  flag `help:` string; and `--decode-plate`).
- **What:** the GUI schema's `word-card --from` help calls it a
  **"BIP-39 mnemonic"** (`phrase=`/`ms1=`/`entropy=`…) and carries
  `secret: false`. But `word-card` operates on **PUBLIC** material only —
  `--from` takes an `mk1` (xpub) or `md1` (descriptor) card; the secret
  `ms1` is **excluded** (verified against `mnemonic-toolkit` word-card
  source: it re-encodes a public card). The mislabel invites a user to
  paste a **seed phrase into an unmasked, no-run-confirm field** — a real
  GUI secret-hygiene footgun. The manual (`4n-word-card.md`) correctly
  documents the PUBLIC behavior and does NOT propagate this help string.
- **Companion:** file the matching entry in `mnemonic-gui/`'s FOLLOWUPS;
  the GUI-side fix (correct the `--from`/`--decode-plate` help) is a
  separate mnemonic-gui cycle. When it ships, the GUI `schema_mirror`
  surface changes → the manual's next GUI-pin bump re-gates.
- **Status:** ✓ **RESOLVED 2026-06-29** (mnemonic-gui PR #26, master `2914496c`).
  The GUI `word-card` `--from`/`--decode`/`--decode-plate` help strings were
  corrected to mirror the toolkit CLI: `--from` is PUBLIC `mk1`/`md1` (NOT a
  secret / seed phrase). Help-text only (no flag-name/`secret` change →
  `schema_mirror` unaffected); GUI suite 622/0. **Tier:** `secret-hygiene`.

### `gui-example-tutorial-book` — toolkit leg of the `gui_example.pdf` tutorial-book cycle (tracking)

- **Surfaced:** 2026-07-05, cycle `gui_example_tutorial`
  (`design/SPEC_gui_example_tutorial.md`, R0-GREEN ×2 +
  `design/IMPLEMENTATION_PLAN_gui_example_tutorial.md`, plan-R0 GREEN). Filed at
  the toolkit leg's P2.1 per spec §11.
- **What (this repo's leg, plan-P2/P3):** consume the tagged tutorial corpus from
  `mnemonic-gui-v0.56.0` into `docs/manual-gui/` — pin bump v0.55.0 → v0.56.0
  (toolkit-implied v0.75.0); the F1 export-wallet `--template` `(none)` ripple (one
  `.gui` re-pin + the reference-manual `(none)` section + inventory regen); byte-copy
  50 figures → `figures/tutorial/` + 33 run transcripts → `transcripts/tutorial/`;
  the three new fail-closed lint phases (10 `verify-tutorial-figures`, 11
  `verify-tutorial-transcripts`, 12 `tutorial-xref`); the `tutorial/` chapter book +
  `gui-example-pdf`/`gui-example-html` Makefile targets + `manual-gui.yml` build
  wiring; then (P3) editorial prose + (P4) the `manual-gui-v*` release-attach.
- **Status:** RESOLVED 2026-07-05 — shipped as `manual-gui-v1.2.0` (P4 shipping
  commit). **Tier:** `cross-repo` (paired legs; GUI tag BEFORE the toolkit pin).
- **Companion:** mnemonic-gui `FOLLOWUPS.md::gui-example-tutorial-book` (GUI leg,
  filed at its P1.1) + toolkit `design/FOLLOWUPS.md::gui-example-tutorial-book`.

### `examples-pdf-un-ci-gated` — `docs/Examples.pdf` is generated but has zero CI gating and a hard-stale pin

- **Surfaced:** 2026-07-05, `gui_example_tutorial` recon E5 (spec §11). Filed at the
  toolkit leg's P2.1; catalog-only.
- **Where:** `.examples-build/gen.sh` (the `Examples.md`/`Examples.pdf` generator).
- **What:** `docs/Examples.pdf` is generated by hand-run `.examples-build/gen.sh` with
  **no CI gate at all** and a hard-stale `mnemonic 0.55.3` pin (`gen.sh:22`) while the
  tier is v0.75.0 — so the committed PDF can silently drift from the current CLI. The
  new tutorial book (this cycle) deliberately does NOT inherit that model: it is fully
  CI-gated (byte corpus gates + embed census) and release-attach-only. Candidate for a
  Cycle-C-FULL-style verify gate + regen. Out of scope this cycle.
- **Status:** open (catalog only; explicitly NOT this cycle). **Tier:** `v1.1+`.
- **Companion:** toolkit `design/FOLLOWUPS.md::examples-pdf-un-ci-gated`;
  `design/SPEC_gui_example_tutorial.md` §11 (filing mandate).

### `gui-secret-reveal-toggle` — toolkit-manual leg of the reveal (👁) + restore `(none)` batched cycle (tracking)

- **Surfaced:** 2026-07-05, batched cycle `tutorial_surfaced_fixes_batch`
  (`design/SPEC_gui_secret_reveal_toggle.md` + `design/SPEC_restore_template_none_affordance.md`,
  both R0-GREEN 0C/0I; `design/IMPLEMENTATION_PLAN_tutorial_surfaced_fixes_batch.md`,
  plan-R0 GREEN). Filed at the toolkit leg's P2.1 per the cross-repo mirror rule.
- **What (this repo's leg, plan-P2.1/P2.2):** consume the two user-visible deltas that
  rode `mnemonic-gui-v0.57.0` into `docs/manual-gui/` — pin bump v0.56.0 → v0.57.0
  (pin-NEUTRAL: toolkit-implied stays v0.75.0, md/ms/mk unchanged); re-pin the 28 `.gui`
  structural renders that gained the ` [reveal]` marker on their masked-on-load secret
  rows (`mnemonic-restore.gui` carries BOTH deltas — the ` [reveal]` row AND the
  `--template … ,(none)]` row); byte-copy the 32 moved gallery PNGs (the 28 `.gui`-movers
  ∪ 4 composite-visual-only `final-word`/`seed-xor-split`/`seedqr-encode`/`ms-shares-split`)
  + the 26 moved tutorial figures (4 reveal-plaintext + 10 restore `(none)` + 12 eye-chrome;
  corpus stays 50, transcripts BYTE-IDENTICAL); regen `expected_gui_schema_inventory.json`
  (restore `--template` gains the trailing `""` sentinel); add the REQUIRED restore `(none)`
  reference section (`src/40-mnemonic/4d-restore.md`) + the reveal-toggle prose + the
  tutorial rewrite (the six restore steps now teach the clean `(none)` md1 restore, not the
  workaround; the four reveal steps note the demo phrase is now visible). **Toolkit leg has
  ZERO `crates/` `src/` changes** (docs-only).
- **Status:** open — discharged by this leg (P2.1 + P2.2); flips RESOLVED in the
  `manual-gui-v1.3.0` shipping commit (P2.3). **Tier:** `cross-repo` (paired legs; GUI tag
  BEFORE the toolkit pin).
- **Companion:** mnemonic-gui `FOLLOWUPS.md::gui-secret-reveal-toggle` (GUI leg, RESOLVED
  mnemonic-gui-v0.57.0) + toolkit `design/FOLLOWUPS.md::gui-secret-reveal-toggle`.

---

## Resolved items

### `manual-gui-visual-screenshot-track` — RESOLVED 2026-07-01

Pixel screenshots of all **61** GUI forms embedded in the GUI-Forms
gallery, byte-gated against the pinned GUI. Cross-repo cycle, two legs:

- **Leg 1 (mnemonic-gui → `mnemonic-gui-v0.54.0`):** a P0 spike proved
  headless software wgpu rendering on real CI runners (lavapipe/Vulkan,
  Plan A; threshold 0.6 fleet-stable across ≥6 runner instances + a
  full backend swap); P1 shipped `tests/gui_form_snapshots.rs`
  (egui_kittest, all 61 forms, `fit_contents()` @ 2x, dark default
  theme, blank fixtures — masked secrets, the always-run
  `flag.secret ⇒ no default_value` hygiene assertion), the committed
  61-PNG corpus (`tests/snapshots/forms/`, 2.12 MiB plain — no LFS),
  and the permanent `snapshots` CI job (ran-at-all census 61), now a
  required check on GUI master.
- **Leg 2 (this manual):** GUI pin bump v0.53.0 → v0.54.0 (CLI pins
  unchanged — verified; schema-coverage zero delta at 982/61);
  `figures/gui/<tab>-<sub>.png` ×61 copied byte-for-byte from the tag's
  corpus; each gallery section embeds its screenshot above the
  structural render (file-relative paths + pandoc `--resource-path` +
  `\graphicspath`/Gin/`[H]` LaTeX alignments); **new lint phase 9/9
  `verify-figures-gui`** — byte-compare vs the pinned clone's corpus,
  census 61 BOTH directions (orphan baseline / coverage gap / byte
  drift all fail, stems named), fail-closed. HTML embeds gated by the
  positive census (61 `data:image/png`), since pandoc exits 0 on a
  missing image.

The separable HUMAN UX-review question ("is this layout confusing") is
NOT closed by pixels — it still needs a person; the run-output panel /
modal chrome remain the documented non-goal residual. Companion:
`mnemonic-gui/FOLLOWUPS.md` entry `gui-form-snapshot-corpus-manual-consumer`.

**Corpus-consumer contract re-exercised 2026-07-01** (hint-text-defaults
cycle, manual leg): pin bump v0.54.0 → v0.55.0 (`538d1a89`; tag-push
`snapshots` check-run = `success`, verified pre-bump per the contract's
provenance-anchor step 0); 6/61 figures + 6/61 structural renders
re-pinned (the `<hint:…>` ghost depiction for schema-defaulted Text/Path
fields, `design/SPEC_gui_hint_text_defaults.md`); CLI pins again
unchanged; `gui-schema-coverage` zero delta (982/61).

### `manual-gui-generated-form-renders` — RESOLVED 2026-06-29

Generated, gated structural renders of all **61** GUI subcommand forms
embedded in the manual. A headless `gui-render` binary (shipped in
`mnemonic-gui-v0.53.0`, built `--no-default-features`) emits each form's
ASCII structural render from the GUI's own `schema/` + `conditional()`
(seeding defaults exactly as the GUI does on load); the renders are
committed under `transcripts/gui/`, embedded via
`include="gui/<tab>-<sub>.gui"`, and **gated** by `verify-examples-gui`
(regenerate with the pinned `gui-render` + `diff` == committed +
census 61 + secret-unmask scan, fail-closed; `manual-gui.yml` job 1c).
Secret fields render a fixed `<masked>` sentinel. Cross-repo cycle:
mnemonic-gui Leg 1 (`gui-render` + egui_kittest faithfulness gate →
`mnemonic-gui-v0.53.0`) + manual-gui Leg 2 (catch-up + renders + gate).
Closes the **form-mockup leg** of
`manual-gui-output-blocks-non-gateable-residual`.

### `manual-gui-form-renders-dedicated-part` — RESOLVED 2026-06-29

Consolidated the 61 generated GUI form renders (added by
`manual-gui-generated-form-renders`) out of the per-subcommand chapters into a
dedicated **Part `75-gui-forms/`** (4 per-tab chapters: mnemonic 32 / md 10 /
ms 10 / mk 9), leaving a one-line cross-link in each subcommand chapter (its
prose + flag anchors + `.out` CLI-output transcript unchanged). Added a new
bidirectional fail-closed `gui-form-xref` gate (`tests/check_gui_form_xref.py`,
`make lint` phase 8/8) so the 61 cross-links + gallery anchors are a gated
invariant (lychee skips intra-doc fragments). Render-content-inert
(`verify-examples-gui` 61/61 byte-identical; `gui-schema-coverage` unchanged at
982/61 — the gallery uses prose-shaped `gui-form-*` anchors, schema-orphan
exempt). The 3 conditional-`(required)` caveats (inspect/repair at-least-one;
ms-encode exactly-one/XOR) retained + reworded to point at the cross-linked
render. Full R0 pipeline GREEN (spec ×2 + plan ×2 + post-impl whole-diff).
