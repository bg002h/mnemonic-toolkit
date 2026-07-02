# mnemonic-gui user manual — CHANGELOG

Versioned changelog for the user manual under `docs/manual-gui/`.
The manual is a separate artifact from the CLI manual under
`docs/manual/`; the two ship under their own tag namespaces
(`manual-gui-v*` vs `manual-v*`) and track independent version
numbers.

## [Unreleased]

**Minor — pixel screenshots of all 61 GUI forms + the `verify-figures-gui`
byte-gate + GUI pin bump `mnemonic-gui-v0.53.0` → `mnemonic-gui-v0.54.0`
(Leg 2 of the visual-screenshot track).** The GUI-Forms gallery now pairs
every structural render with a **screenshot of the real GUI form**: each
section in `75-gui-forms/751…754` opens with its
`figures/gui/<tab>-<sub>.png` (61 total — dark theme, the GUI's on-launch
default; 2x scale; egui content only, no OS titlebar; frame = flags +
positionals + `Run`), copied **byte-for-byte** from the pinned GUI's
`egui_kittest` snapshot corpus (`tests/snapshots/forms/` at
`mnemonic-gui-v0.54.0`, whose tag-push `snapshots` CI run — GREEN,
verified pre-bump — arbitrates the pixels on the pinned rasterizer).

- **New gate `verify-figures-gui`** (`make lint` phase 9/9, banners
  renumbered from /8): byte-compares every committed figure against the
  pinned clone's snapshot corpus with a **both-direction census** at 61 —
  an orphan manual figure, a missing figure, or any byte drift fails,
  offending stems named, fail-closed.
- **Image-path mechanics:** chapters reference figures FILE-relative
  (`../../figures/gui/…`, the form lychee resolves); pandoc-HTML gets
  `--resource-path`, xelatex gets `\graphicspath` + image-scaling Gin
  defaults + non-floating `[H]` figures (61 floats would queue), and a
  guarded `\pandocbounded` shim covers pandoc ≥ 3.2.1. A green
  `make html` proves nothing about embeds (pandoc warns + exits 0 on a
  missing image) — the load-bearing check is the positive embed census:
  61 `data:image/png` embeds in the built HTML (CI-enforced in the
  `manual-gui.yml` build job).
- **Pin bump is otherwise inert:** the v0.53.0 → v0.54.0 delta is the
  GUI-side snapshot suite + its permanent `snapshots` CI job (+ a
  Cargo.lock delta of exactly 12 dev-graph packages) and the PR-#26
  word-card help-text fix — no CLI-surface change. The tag's four
  implied CLI pins are identical, so the manual's CLI pins and
  `manual-gui.yml`'s verify-examples tags are untouched;
  `gui-schema-coverage` unchanged at 982 anchors / 61 subcommands;
  `verify-examples-gui` still 61/61 byte-identical (structural renders
  untouched).

**Minor — the 61 GUI form renders consolidated into a dedicated "GUI Forms"
Part (restructuring; no render-content change).** The per-subcommand form
renders (added the prior leg) are moved OUT of the 61 subcommand chapters
INTO a new Part `75-gui-forms/` — 4 per-tab chapters (mnemonic 32 / md 10 /
ms 10 / mk 9) — so all the GUI screens read as one dedicated reference. Each
subcommand chapter keeps a one-line cross-link to its form (`> **GUI form:**
see [GUI Forms › …]`); its prose, flag anchors, and CLI-output transcript are
unchanged. The 3 forms with a conditional-`(required)` caveat (`inspect`/
`repair` = at-least-one; `ms encode` = exactly-one/XOR) keep their caveat,
reworded to point at the cross-linked render. The tour keeps its 2 inline
renders.

- **New gate `gui-form-xref`** (`tests/check_gui_form_xref.py`, `make lint`
  phase 8/8): bidirectional, fail-closed — for each `transcripts/gui/*.gui`
  stem, EXACTLY ONE `{#gui-form-<tab>-<sub>}` gallery anchor + EXACTLY ONE
  `](#gui-form-…)` cross-link, no orphans. So the 61 cross-links are a gated
  invariant, not a by-construction hope (lychee skips intra-doc fragments).
- **Gate-inert by construction:** the `.gui` render files are UNCHANGED
  (`verify-examples-gui` 61/61 byte-identical); `gui-schema-coverage` is
  unchanged at 982 anchors / 61 subs (the gallery uses prose-shaped
  `gui-form-*` anchors, exempt from the schema orphan check). `make lint` 8/8,
  HTML/PDF build clean.

**Minor — generated, gated GUI form renders in the manual + a GUI pin
catch-up (`mnemonic-gui-v0.49.0` → `mnemonic-gui-v0.53.0`), Leg 2 of the
generated-GUI-form-renders cycle.** The manual now SHOWS the real GUI: a
generated structural render of every one of the 61 GUI subcommand forms
is embedded in its chapter and gated against drift — extending the
manual's "prose == output by construction" discipline (today
CLI-transcript-only) to the GUI surface. Plus the pin catch-up the bump
to a `gui-render`-capable GUI tag required: additive coverage of +5
newly-exposed subcommand sections, +17 schema anchors, +1 outline
target; 0 removed.

- **Pin bump.** `pinned-upstream.toml`: `[mnemonic-gui]` `v0.49.0` →
  `v0.53.0`; the four implied CLI tags re-pinned to what the GUI tag
  pins in its own schema-mirror map (`mnemonic-toolkit-v0.74.0`,
  `descriptor-mnemonic-md-cli-v0.11.0`, `ms-cli-v0.13.0`,
  `mk-cli-v0.11.0`). `.github/workflows/manual-gui.yml` `verify-examples`
  job re-pinned the same four CLI install tags in lockstep.
- **`mnemonic word-card`** — new full chapter
  (`src/40-mnemonic/4n-word-card.md`): the toolkit-v0.74.0
  steel-engravable BIP-39 word-card encoder/decoder (8 flags +
  repeating decode positional), with its `### Outline`, glossary entry,
  and index row.
- **`gen-man`** — new stub sections on all four CLI tabs
  (`src/40-mnemonic/4o-gen-man.md`, `src/50-md/5B-gen-man.md`,
  `src/60-ms/6b-gen-man.md`, `src/70-mk/7a-gen-man.md`): the
  `--out`-only roff man-page generator now exposed in each schema.
- **Inventory hygiene.** `tests/expected_gui_schema_inventory.json`
  regenerated from the v0.53.0 GUI schema source.
- **61 generated GUI form renders (the headline, P5).** Each GUI
  subcommand form is emitted as a deterministic ASCII structural render
  (`transcripts/gui/<tab>-<sub>.gui`) by the pinned headless `gui-render`
  binary (`mnemonic-gui-v0.53.0 --no-default-features`) and embedded in
  its chapter via `include="gui/<tab>-<sub>.gui"`. Secret fields render a
  fixed `<masked>` sentinel (the value is never sourced from form state);
  the render seeds flag defaults exactly as the GUI does on load, so it
  depicts the screen the user actually sees.
- **`verify-examples-gui` fidelity gate (`tests/verify-examples-gui.sh` +
  Makefile target + `manual-gui.yml` job 1c).** Regenerates the renders
  with the pinned `gui-render` and `diff`s == committed (fail-closed),
  plus a census (all 61) and an independent secret-unmask scan — so the
  manual's GUI depiction can never silently drift from the real GUI. This
  closes the **form-mockup leg** of
  `manual-gui-output-blocks-non-gateable-residual`.
- **Tour mockups replaced.** The two hand-drawn full-window form mockups
  in `src/30-tour/31-first-launch.md` (which had silently DRIFTED from
  the real GUI — e.g. `--template bip84` vs the real `bip44`, a stale
  slot-row count) were swapped for the generated, gated renders. An
  at-least-one `(required)` caveat was added to the three genuine
  at-least-one forms (`inspect`, `repair`, `ms encode`).
- `make lint` 7/7 GREEN (0-missing / 0-orphan against the v0.53.0 GUI
  schema), `verify-examples` + `verify-examples-gui` GREEN against the
  pinned bins, HTML + PDF build clean (renders appear, no empty fences).

## [1.1.0] - 2026-06-23

**Minor — manual-gui v1.1 modernization (GUI pin `mnemonic-gui-v0.3.0`
→ `mnemonic-gui-v0.49.0`).** Purely additive coverage: +28 subcommand
chapters/sections, +506 schema anchors, +69 outline targets; 0 schema
anchors removed, 0 chapters deleted.

- **Pin bump.** `pinned-upstream.toml`: `[mnemonic-gui]` `v0.3.0` →
  `v0.49.0`; the four implied CLI tags re-pinned to what the GUI tag
  pins in its own schema-mirror map (`mnemonic-toolkit-v0.70.0`,
  `descriptor-mnemonic-md-cli-v0.7.0`, `ms-cli-v0.8.0`, `mk-cli-v0.9.0`);
  stale line-number comment rewritten.
- **Secret-redaction prose sync (GUI `v0.39.0`+).** Chapters 11, 14,
  32, 42, and 84 corrected from the v0.3.0-era "modal renders argv in
  plaintext" framing to the shipped `••••`-sentinel redaction:
  secret-bearing argv VALUES are masked in both the run-confirm modal
  and the output-panel `argv:` echo; multi-row / slot secret rows carry
  a per-row mask bit; the cold-node operational practice is demoted from
  load-bearing to general hygiene. Residual flag-name exposure and the
  spawned-argv (`/proc/<pid>/cmdline`) exposure preserved as
  still-true caveats.
- **Per-tab overview counts** corrected (mnemonic 10→30, md 8→9,
  ms 5→9, mk 5→8) plus pinned-version labels.
- **import-wallet orphan reconcile.** Renamed 5 prose/walkthrough
  anchors out of the schema-shaped namespace (`iw-*`) and removed the
  `--select-descriptor` enumerated-dropdown anchors/outline (it is a
  0-variant flag at v0.49.0) to clear the orphan-direction lint at the
  re-pin.
- `.cspell.json`: added `exfiltration`, `nostr`, `sentinel`,
  `unredacted`.
- **Output-fidelity gating (P4).** 19 worked-example fences across the
  `mnemonic`-tab chapters are wired to GUI-pinned golden transcripts
  (`{.text/.json include="<stem>.out"}`, fail-closed via
  `pandoc/filters/include-transcript.lua`) and re-run against the pinned
  binaries by `make verify-examples` (17 transcripts) in CI — so the
  documented argv/output for the gated forms is drift-checked, not just
  the schema anchors.
- **Modernization phases (P1–PE) complete.** The +506 new schema
  anchors were authored per-tab across the v1.1 phases and gated GREEN:
  `make lint` is 7/7 (0-missing / 0-orphan against the v0.49.0 GUI
  schema), `verify-examples` GREEN against the pinned bins, and HTML +
  PDF build clean. Closes FOLLOWUP
  `gui-run-confirm-modal-secret-redaction-manual-companion` (the
  redaction prose was blocked on this modernization).

## [1.0.2] - 2026-06-07

**Patch — prose sync with toolkit v0.47.3** (GUI pin unchanged at
`mnemonic-gui-v0.3.0`).

- `45-export-wallet.md`: `--timestamp` default reworded `now` → `0`
  (rescan from genesis), matching the toolkit v0.47.3 default flip
  (summary + prose + JSON example).
- `.cspell.json`: added `rescan`.
- Routine doc re-capture (output-class advisory wording).

## [1.0.1] - 2026-05-15

**Patch — post-ship cosmetic fixes.** Surfaced by a fresh local
rebuild after v1.0.0 shipped.

- **PDF glyph fix.** Replaced U+2715 (✕ multiplication-X) with
  U+00D7 (× multiplication sign) in 2 locations
  (`src/30-tour/31-first-launch.md` and
  `src/40-mnemonic/42-bundle.md`). DejaVu Serif Bold lacks U+2715,
  so the ✕ rendered as a "?" replacement char in the v1.0.0 PDF.
- **2 stale intra-document cross-references** fixed (LaTeX
  hyperref caught them; the lint suite's lychee phase doesn't
  validate intra-document anchors):
  - `src/10-foundations/11-what-is-mnemonic-gui.md`: 2 links to
    `#how-the-gui-relates-to-the-four-clis` updated to
    `#relation-to-cli` (the v1.0 cycle's batch-9 named-anchor
    add).
  - `src/90-appendices/92-flag-index.md`: removed fabricated
    `md verify --json` link — schema's `VERIFY_FLAGS` does NOT
    include `--json` (verified against
    `mnemonic-gui/src/schema/md.rs:183-216`).

No content changes; identical anchor set; lint still 7/7 GREEN.
PDF stays at 223 pages.

## [1.0.0] - 2026-05-15

**First release.** Complete coverage of the `mnemonic-gui` v0.3.0
GUI surface across all four CLI tabs (`mnemonic`, `md`, `ms`,
`mk`), with bidirectional schema-coverage lint enforced on every
PR and a help-icon contract that deep-links every `?` button to
its matching anchor in the rendered HTML.

### Highlights

- **~3,400 lines of markdown** across **51 source files**
  rendering to a **223-page PDF** + a single-file HTML at
  `https://bg002h.github.io/mnemonic-toolkit/manual-gui/`.
- **459 schema anchors** + **59 outlines** + **44 indexed terms**
  enforced by `make lint` (7 phases: markdownlint, cspell,
  lychee, gui-schema-coverage, outline-coverage,
  glossary-coverage, index bidirectional).
- **3-job CI workflow** at `.github/workflows/manual-gui.yml`
  (lint / build / release) — gh-pages deploy on `manual-gui-v*`
  tag push.
- **Lockstep companion** in `mnemonic-gui` (branch
  `manual-gui-help-icons`): URL helper module, per-subcommand /
  per-Dropdown / per-NodeValueComposite `?` button rendering,
  egui_kittest cell pinning the click → `OpenUrl` command path,
  and a `manual_anchor_coverage` Rust test asserting every
  schema-derived URL fragment resolves in the rendered HTML.

### Pinned upstream tags

- `mnemonic-toolkit-v0.13.0` — `mnemonic` CLI
- `descriptor-mnemonic-md-cli-v0.5.0` — `md` CLI
- `ms-cli-v0.2.1` — `ms` CLI
- `mk-cli-v0.3.1` — `mk` CLI
- `mnemonic-gui-v0.3.0` — the GUI itself

(See `docs/manual-gui/pinned-upstream.toml` for the
machine-readable form.)

### Chapter coverage

| Range | Coverage |
|---|---|
| `00-frontmatter` + `00-disclaimer` | Front-matter; secret-class disclaimer |
| `10-foundations/` (4 files) | What the GUI is; relation to the four CLIs; bundle/card/slot mental model; secret-handling threat model |
| `20-install/` (3 files) | Linux / macOS / Windows install + wgpu/egui graphics-stack notes |
| `30-tour/` (3 files) | First-launch walkthrough; **Run** + output-panel; help-icons + deep-links |
| `40-mnemonic/` (10 subcommand chapters + overview) | `mnemonic` tab: bundle, verify-bundle, convert, export-wallet, derive-child, final-word, seed-xor-split/combine, slip39-split/combine |
| `50-md/` (8 subcommand chapters + overview) | `md` tab: inspect, encode, decode, verify, bytecode, vectors, compile, address |
| `60-ms/` (5 subcommand chapters + overview) | `ms` tab: inspect, encode, decode, verify, vectors |
| `70-mk/` (5 subcommand chapters + overview) | `mk` tab: inspect, encode, decode, verify, vectors |
| `80-troubleshooting/` (4 files) | Binary-missing + launch issues; form-fill + output errors; secrets + OS hygiene |
| `90-appendices/` (5 files) | Glossary; alphabetical flag index; Dropdown/NodeValueComposite/TaggedOrIndexed reference; release history; bidirectional `\index{}` table |

### What's NOT in scope (v1.1 deferrals)

Tracked in `design/FOLLOWUPS.md` at this repo and at the
`mnemonic-gui` repo's `FOLLOWUPS.md`:

- `gui-manual-cross-refs-to-cli-manual` — bidirectional links
  between this manual and the CLI manual where concepts overlap
  (currently one-way: this manual references the CLI manual but
  not vice versa).
- `cli-manual-html-target` — HTML render of the CLI manual to
  match this manual's pipeline; only PDF + Markdown today.
- `gui-manual-localization` — non-English translations.
- `gui-help-icon-per-flag-affordance` — per-flag `?` buttons (in
  addition to the existing per-subcommand / per-Dropdown /
  per-NodeValueComposite / per-repeating-field icons).
- `gui-manual-base-url-runtime-override` — `--manual-base-url`
  runtime flag if the build-time `MNEMONIC_GUI_MANUAL_BASE_URL`
  env-var override proves insufficient.
- `mk-vectors-pretty-out-help-mismatch` — upstream `mk-cli`
  help-text claim that `--pretty` is ignored under `--out`
  (source actually honors it; lockstep fix needed across mk-cli
  source + mnemonic-gui schema + this manual's note).

### Reviewer-loop record

Ten reviewer-locked batches under M-P2.4 (40-mnemonic 5
sub-batches; 50-md 3 sub-batches; 60-ms 1 sub-batch; 70-mk 1
sub-batch; 80-troubleshooting 1 sub-batch; 90-appendices 1
sub-batch) plus M-P0/M-P1/M-P2.5/P3 reviewer passes. R0
ITERATE → fold → R1 LOCK on every batch except where R0
returned LOCK on nits-only. Drift catches across the cycle
included the `ms-encode-lang` → `ms-encode-language` anchor
rewire, the `mk-cli` false `conflicts_with` tri-cite drift, the
fabricated `winit-wayland-masked-input` FOLLOWUP citation, the
appendix flag-index fabrication catch (rewritten from schema
source), and the gh-pages publish-path-vs-URL-helper mismatch.

### Companion PRs

- Track M (this repo): see PR opened against `master` from
  branch `manual-gui-v1`.
- Track G (`bg002h/mnemonic-gui`): see PR opened against
  `master` from branch `manual-gui-help-icons`. **Both PRs MUST
  merge in lockstep** per the cycle's
  `[[feedback-manual-gui-lockstep]]` mirror invariant.
