# mnemonic-gui user manual — CHANGELOG

Versioned changelog for the user manual under `docs/manual-gui/`.
The manual is a separate artifact from the CLI manual under
`docs/manual/`; the two ship under their own tag namespaces
(`manual-gui-v*` vs `manual-v*`) and track independent version
numbers.

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
- `.cspell.json`: added `exfiltration`, `sentinel`, `unredacted`.
- **(P1–PE, in progress)** the +506 new schema anchors are authored
  per-tab across the v1.1 modernization phases; gated GREEN at PE.

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
