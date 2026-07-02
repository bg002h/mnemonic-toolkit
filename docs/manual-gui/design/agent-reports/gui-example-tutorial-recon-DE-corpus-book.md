# gui_example.pdf cycle — RECON report D+E (corpus conventions + book build/gates)

> Provenance: recon subagent report, persisted verbatim per house convention.
> Cycle: `gui_example_tutorial` (spec: `docs/manual-gui/design/SPEC_gui_example_tutorial.md`).
> Dispatched 2026-07-02 by the RECON+SPEC author. Sources verified against
> toolkit `master@4c401b16` and mnemonic-gui `master@0d4429d` (v0.55.0) working trees.

---

# Recon D+E — GUI screenshot machinery & the book/gate stack

Search breadth: very thorough. All cites are `file:line` against current working trees of `/scratch/code/shibboleth/mnemonic-toolkit` (toolkit) and `/scratch/code/shibboleth/mnemonic-gui` (GUI). UNVERIFIED items are flagged.

## Orientation (the key fact that reframes the whole cycle)

The recent "generated + gated GUI figures" cycle is the **visual-screenshot track**, spec at `toolkit:docs/manual-gui/design/SPEC_gui_visual_screenshot_track.md`. Its existing corpus is **FORM-ONLY** renders (flag grid + positionals + `[ Run ]`), auto-sized to content, on a **blank** fixture — explicitly NOT the whole window. Its own non-goals list rules out exactly what `gui_example.pdf` wants: "non-form chrome (output panel / modal … OS-native window decorations)" (`SPEC_gui_visual_screenshot_track.md:44`) and it renders the blank on-load state, never a filled form or post-Run output. So the tutorial book **reuses the plumbing/gates/determinism conventions but needs a genuinely new render path.**

---

## Part D — screenshot conventions + window sizing

### D1. The render/snapshot machinery, harness options, and the gate

**Which repo renders the images:** mnemonic-gui, via an `egui_kittest` snapshot test — `mnemonic-gui/tests/gui_form_snapshots.rs`. (There is also a *headless, egui-free* `gui-render` binary at `mnemonic-gui/src/bin/gui_render.rs`, but that emits the **text** structural `.gui` renders, not PNGs — different track, see E/§8.)

**Harness builder options** (`mnemonic-gui/tests/gui_form_snapshots.rs:76-90`):
```
Harness::builder()
    .with_pixels_per_point(2.0)          // PIXELS_PER_POINT const, line 71
    .build_ui_state(|ui, state| {         // a UI-CLOSURE harness, NOT the eframe App
        ui_harness::render_whole_form(ui, tab, sub, state);   // flag grid
        ui_harness::render_positionals(ui, sub, state);        // positionals
        ui_harness::render_action_bar(ui);                     // [ Run ]
    }, base);
harness.fit_contents();                   // viewport SHRUNK-TO-CONTENT
```
- **pixels_per_point:** fixed `2.0` (`gui_form_snapshots.rs:71`).
- **theme:** egui default **dark** — the GUI sets no custom visuals (`SPEC_gui_visual_screenshot_track.md:22`); documented to readers at `toolkit:docs/manual-gui/src/75-gui-forms/750-overview.md:9`.
- **fonts:** crate-embedded (`eframe … features=["default_fonts", …]`, `mnemonic-gui/Cargo.toml:47`); no system fonts (`SPEC …:22`).
- **cursor blink:** disabled by kittest at construction; animations settle via run-to-quiescence (`SPEC …:22`).
- **kittest version:** `egui_kittest 0.31.1`, features `wgpu` + `snapshot` (`mnemonic-gui/Cargo.toml:112`, `Cargo.lock`); egui/eframe `0.31`.
- The shared render helpers live in `mnemonic-gui/tests/ui_harness/mod.rs:419` (`render_whole_form`), `:522` (`render_positionals`), `:534` (`render_action_bar`).

**Where PNGs land:** `mnemonic-gui/tests/snapshots/forms/<tab>-<sub>.png` (const `SNAPSHOT_DIR`, `gui_form_snapshots.rs:68`). 61 committed. kittest also drops `<name>.new.png` on every gated run (used as the census tripwire); `.new/.diff/.old` are gitignored.

**Gating — threshold, not byte-identical:** kittest's native **dify threshold at the default 0.6** (`SnapshotOptions::new().output_path(SNAPSHOT_DIR)` with no override, `gui_form_snapshots.rs:156`; `try_snapshot_options`, `:174`). Comparison is threshold-diff because cross-backend/CPU byte drift is real (see D4).

**The CI gate/job** (`mnemonic-gui/.github/workflows/build.yml:87-147`, job name `snapshots`):
- `env: GUI_SNAPSHOTS: "1"`, `WGPU_BACKEND: vulkan` (`build.yml:110-112`).
- Software rasterizer = **lavapipe** via `apt-get install mesa-vulkan-drivers` (`build.yml:116-124`).
- Runs `GUI_SNAPSHOTS=1 cargo test --test gui_form_snapshots -- --nocapture` (`build.yml:133`).
- **Ran-at-all census:** `find tests/snapshots/forms -name '*.new.png' | wc -l` must equal 61 (`build.yml:135-139`).
- On failure uploads `**/*.diff.png` artifacts (`build.yml:141-147`).
- **No path filter** — fires on every PR / master push / `mnemonic-gui-v*` tag (the tag run is the manual leg's provenance anchor; `SPEC …:29`).
- **Adapter guard** inside the test: selected wgpu adapter MUST be `device_type == Cpu`, `backend` must honor `WGPU_BACKEND` (`gui_form_snapshots.rs:118-145`). Never keys on adapter name (lavapipe self-reports as "llvmpipe"; `gui_form_snapshots.rs:29-32`).
- Env-gate is EARLY-RETURN-SKIP so plain `cargo test` on dev machines is unaffected (`gui_form_snapshots.rs:109-116`).

### D2. Current image geometry — auto-sized, NOT fixed

Size is **auto-sized-to-content** (`fit_contents()`), so width AND height vary per form. Verified with `identify` on `mnemonic-gui/tests/snapshots/forms/*.png` (physical px @ ppp 2.0, `height×width`):

Tallest → shortest (representative):
| stem | h×w |
|---|---|
| mnemonic-verify-bundle | 1253×850 |
| mnemonic-restore | 1169×850 |
| mnemonic-convert | 957×1008 |
| mnemonic-bundle | 912×857 |
| mnemonic-export-wallet | 828×919 |
| mnemonic-build-descriptor | 827×778 |
| mnemonic-xpub-search-passphrase-of-xpub | 745×935 |
| md-encode | 660×828 |
| mnemonic-addresses | 576×812 |
| mk-encode | 533×829 |
| … | … |
| md-inspect / mk-inspect / mk-decode | 153×778 |
| ms-vectors (smallest, per spike) | 110×168 |

Widths span ~168–1008 px; heights ~110–1253 px. The identical `figures/gui/` copies in the toolkit have the same dimensions (verified: `toolkit:docs/manual-gui/figures/gui/mnemonic-verify-bundle.png` = 1253×850, 61 files).

### D3. Scrolling — the real shell vs the flat harness

The **real app shell** (`mnemonic-gui/src/main.rs`) is a 3-panel eframe window:
- Top tab strip: `TopBottomPanel::top("tabs")` with the mnemonic/md/ms/mk buttons + active `◀` marker (`main.rs:424-453`).
- Bottom output panel: `TopBottomPanel::bottom("output")` with show-cmdline/stdout/stderr checkboxes, `argv:` line, exit badge, and stdout/stderr each in their own `ScrollArea::vertical().max_height(180/120)` (`main.rs:456-509`; the "(no run yet)" placeholder at `:507`).
- Central form: `CentralPanel` with `Pinned: <ver>` + the **subcommand `ComboBox`** selector + `?` help button (`main.rs:512-555`), then the flag form inside `egui::ScrollArea::vertical().show(...)` (`main.rs:593`).

The snapshot harness renders the **form only, flat, with NO ScrollArea** and `fit_contents()`, so nothing clips today. For a **fixed whole-window** shot the geometry flips into a problem: the tallest journey forms are ~1253 px @ ppp 2.0 (`mnemonic verify-bundle`, 29 flag rows, ≈43 px/row; `gui_form_snapshots.rs:46-47`, spike M2 `visual-track-p0-spike.md:50-53`). The P0 spike measured that a **fixed 800×600 viewport (=1600×1200 physical) WOULD clip verify-bundle** — it is the only form that clips (`visual-track-p0-spike.md:53`). A whole-window shot that also spends vertical budget on the tab strip + output panel makes clipping worse for many more forms.

**kittest scroll control: UNVERIFIED — spike item.** `grep` for `scroll_to` / `scroll_to_me` / AccessKit `ScrollTo` across `mnemonic-gui/tests/` and `src/` returns **zero** precedent (only `ScrollArea` widget-construction sites). No existing test drives kittest scrolling. Note the render ceiling for the flat path: wgpu `max_texture_dimension_2d` (8192 px) caps `fit_contents()` at ≈190 rows and FAILS-LOUD rather than clipping (`gui_form_snapshots.rs:44-48`) — but that ceiling does not help a *fixed-window* shot, which needs either scroll actions (unverified) or per-step cropping / multi-shot. **Flag as a P0 spike for the new cycle.**

### D4. Determinism conventions already established (and where documented)

- **ppp = 2.0**, **threshold = 0.6**, **dark theme**, **embedded fonts** — as above.
- **CI COMPARES; it does not regenerate.** Regeneration is manual: `GUI_SNAPSHOTS=1 WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1 UPDATE_SNAPSHOTS=1 cargo test --test gui_form_snapshots` (`gui_form_snapshots.rs:35-37`; `build.yml:102-104`). The pinned CI job arbitrates the result at 0.6.
- **OS/renderer variance is real but sub-threshold.** P0 spike (`toolkit:docs/manual-gui/design/agent-reports/visual-track-p0-spike.md`): same-env renders byte-identical (4/4 jobs, `:23-24`); cross-env 20/62 PNGs byte-differ but **0 pixels exceed 0.6** across 558 comparisons incl. a full Vulkan↔GL backend swap (`:26-40, :72`). Corpus byte-SIZE is explicitly NOT an identity signal (`gui_form_snapshots.rs:39-42`; spike `:73`).
- **Remediation ladder** (never blanket-raise threshold; per-form `SnapshotOptions` with human sign-off; escalate to pinned-env regen): `SPEC_gui_visual_screenshot_track.md:31`.
- **Where documented:** spec §4 (`SPEC_gui_visual_screenshot_track.md:20-24`); the harness module-doc (`gui_form_snapshots.rs:1-54`); the reader-facing overview (`toolkit:docs/manual-gui/src/75-gui-forms/750-overview.md:9-12`); the latest-cycle spec cross-ref (`SPEC_gui_hint_text_defaults.md:214-225`).
- Timing/size budget (spike): full-61 render loop 16.5–22.1 s CI (9.6 s local); warm job ~1.5–2 min; 61 PNGs = ~2.12 MiB (`visual-track-p0-spike.md:57-63`).

---

## Part E — the book build + gates

### E5. How `docs/Examples.pdf` is built today

- Builder: `toolkit:.examples-build/gen.sh` (35 KB bash) → emits `Examples.md` to stdout by **running the real `mnemonic` binary and capturing verbatim input+output** ("every command is run and its full combined output shown", `gen.sh:2-4`), then `pandoc Examples.md --include-in-header=preamble.tex --listings --pdf-engine=xelatex -f markdown-smart -o Examples.pdf` (`gen.sh:10-14`).
- Preamble: `toolkit:.examples-build/preamble.tex` — `listings` + `xcolor`, a literate glyph map + active-catcode `≥/≤` hack so output unicode survives xelatex (`preamble.tex:1-18`).
- **Binary pin:** gen.sh hard-asserts `mnemonic 0.55.3` (`gen.sh:25`).
- **Generated vs hand-pasted:** GENERATED (binary-run capture), but **NOT verify-gated** — there is NO CI workflow for it. `grep` over `.github/workflows/` for `gen.sh` / `Examples.md` / `examples-build` returns nothing; there is no `examples.yml`. So it is a **manual, run-locally build** (contrast with manual-gui's `verify-examples`).
- **What's checked in:** only `gen.sh` is git-tracked under `.examples-build/` (`.examples-build/.gitignore:9-10` ignore `Examples.md`+`Examples.pdf`), but the **deliverable `docs/Examples.pdf` IS committed** (`git ls-files docs/Examples.pdf` → tracked; 215 KB).

### E6. How the GUI manual (`docs/manual-gui/`) is built + gated

- Build tool: **pandoc → xelatex** (NOT mdbook). `toolkit:docs/manual-gui/Makefile`: `md` (gfm, `:124`), `pdf` (`pandoc --to latex` → `xelatex` ×2 + optional `makeindex`, `:157-182`), `html` (`html5 --standalone --self-contained --resource-path=.:src/75-gui-forms`, `:200-213`). Figure paths are file-relative `../../figures/gui/<stem>.png`, resolved via `--resource-path` (`Makefile:190-206`).
- Lint/tests dir: `docs/manual-gui/tests/` — `lint.sh` (9 phases), `check_gui_form_xref.py`, `check_gui_schema_coverage.py`, `check_outline_coverage.py`, `verify-examples-gui.sh`, `verify-examples.sh` (symlink to `../../manual/tests/verify-examples.sh`).
- **9-phase `make lint`** (`lint.sh`): 1 markdownlint, 2 cspell, 3 **lychee --offline** (`:97-100`), 4 gui-schema-coverage, 5 outline-coverage, 6 glossary-coverage, 7 index, 8 **gui-form-xref** (every `transcripts/gui/*.gui` stem embedded exactly once; `:179-191`), 9 **verify-figures-gui** (`:193-248`).
- **Figure-embed census (two independent gates):**
  - `gui-form-xref` (`check_gui_form_xref.py`) ties every `.gui` stem to its embed site.
  - `verify-figures-gui` (`lint.sh:204-247`): byte-compares `figures/gui/*.png` against the pinned GUI clone's `tests/snapshots/forms/*.png`, census **both directions** (orphan-baseline fail, coverage-gap fail, byte-drift fail), expected 61.
  - HTML build census: `grep -o 'src="data:image/png' … | wc -l` must equal 61 (`manual-gui.yml:304-314`).
- **CI workflow:** `toolkit:.github/workflows/manual-gui.yml`. Jobs: **`lint`** (clones pinned GUI, runs `make lint`; `:33-107`), **`verify-examples`** (`:131-186`), **`verify-examples-gui`** (`:210-260`), **`build`** (`make pdf`+`html`, embed census, artifact upload; `:269-327`), **`release`** (tag-only; GitHub release + gh-pages; `:342-440`). Path-filtered to `docs/manual-gui/**` for branch/PR; tag pushes always run (`:9-20`).

**REUSABLE gates for a tutorial book vs NEW:**
- Reusable as-is: pandoc→xelatex Makefile skeleton, lychee, the `src="data:image/png"` embed census pattern, `include-transcript.lua` fenced-include mechanism, the pinned-clone plumbing, release/gh-pages job shape.
- Must be NEW/adapted: a `verify-tutorial-figures` gate (the existing `verify-figures-gui` byte-diffs a *1:1 form corpus*; a tutorial has *two shots per step* with a different stem scheme and a different upstream layout), a new xref checker (step↔figure), a new `EXPECTED_*_COUNT` census key, and either a new book target in the Makefile/CI or a new `gui_example.pdf` build dir mirroring `.examples-build` (see E7).

### E7. Cross-repo image-generation flow (where the tutorial corpus slots in)

The flow is **checked-in PNGs + a pinned tag**, NOT artifact fetch:
1. mnemonic-gui generates + **commits** the corpus at `tests/snapshots/forms/<tab>-<sub>.png`; the `snapshots` CI job (`build.yml:87`) arbitrates it at 0.6 and the tag-push run is the provenance anchor.
2. The manual **byte-copies** those PNGs into `toolkit:docs/manual-gui/figures/gui/<tab>-<sub>.png` and commits them.
3. The pin lives in `toolkit:docs/manual-gui/pinned-upstream.toml`: `[mnemonic-gui] tag = "mnemonic-gui-v0.55.0"` + 4 implied CLI pins (`pinned-upstream.toml:32-39`).
4. `manual-gui.yml`'s lint job `git clone --depth 1 --branch $PINNED_TAG mnemonic-gui` → `MANUAL_GUI_UPSTREAM_ROOT` (`manual-gui.yml:41-63`); `verify-figures-gui` byte-diffs `figures/gui/` against that clone's `tests/snapshots/forms/` (`lint.sh:204`).
5. Reverse direction: mnemonic-gui pins the toolkit for schema-mirror (`mnemonic-gui/pinned-upstream.toml:18-22`, `mnemonic-toolkit-v0.74.0`). Staleness is guarded by `toolkit:.github/workflows/gui-pin-drift-check.yml` and `sibling-pin-check.yml`.

**Where a tutorial corpus slots in:** generate the whole-window/two-shot PNGs in mnemonic-gui (new test + new corpus dir, e.g. `tests/snapshots/tutorial/`), gate them in the GUI's `snapshots` job (or a sibling job), tag a new `mnemonic-gui-v*`, bump the manual/book pin, byte-copy into a new figures dir, and add a new `verify-*-figures` census. The split-gate rationale (GUI owns rendering+threshold; docs repo owns byte-copy+census — no rasterizer in docs CI) is the pattern to inherit (`SPEC_gui_visual_screenshot_track.md:9-15`).

### E8. Existing design SPECs to build on + naming convention

`toolkit:docs/manual-gui/design/` holds the directly-relevant specs:

- **`SPEC_gui_visual_screenshot_track.md`** (the pixel-PNG precedent — READ IT). Key decisions:
  - Split-gate: Leg-1 (GUI) owns kittest render + 0.6 threshold; Leg-2 (manual) owns byte-copy + `verify-figures-gui` (`:9-15`).
  - Harness: `render_fixture` blank state, ppp 2.0, `fit_contents()` (P0-ratified over fixed-size), dark theme, embedded fonts (`:20-24`).
  - No-clipping acceptance bar; the real app scrolls but the harness doesn't (`:24`).
  - Secret hygiene as a first-class pixel-channel bar: `secret ⇒ default_value.is_none()` machine assertion + masked widgets + no-inject invariant (`:23`; enforced by `secret_flags_never_carry_a_default_value`, `gui_form_snapshots.rs:188-229`).
  - Corpus layout `tests/snapshots/forms/<tab>-<sub>.png` → `figures/gui/<tab>-<sub>.png`; caption `![<tab> <sub> — GUI form screenshot (dark theme, 2x)](../../figures/gui/<tab>-<sub>.png)` (`751-mnemonic.md:7`).
- **`SPEC_generated_gui_form_renders.md`** (the sibling structural-text track): A1 headless egui-free `gui-render` emit → `transcripts/gui/<tab>-<sub>.gui`, gated by `verify-examples-gui` (regenerate+byte-diff) + census 61 (`:23-44`). Count derived dynamically from `schema_for(tab).subcommands` = 61 (mnemonic 32 + md 10 + ms 10 + mk 9) (`:7`).
- Also present: `SPEC_gui_forms_dedicated_part.md` (the gallery Part structure), `SPEC_gui_hint_text_defaults.md` (latest cycle; documents the 6-form hint-text delta and re-cites the corpus/pin plumbing at `:214-225`), plus the plan files `IMPLEMENTATION_PLAN_gui_visual_screenshot_track.md` and the `design/agent-reports/visual-track-*` round reports (esp. the P0 spike, `agent-reports/visual-track-p0-spike.md`).

**Corpus/figure naming convention:** stem = `<tab>-<sub>` where `tab ∈ {mnemonic, md, ms, mk}` is `CliTab::bin_name()` and `<sub>` is the subcommand name (e.g. `mnemonic-verify-bundle`, `md-encode`, `mk-inspect`). The SAME stem is shared by three artifacts, cross-checked by `gui-form-xref`: `tests/snapshots/forms/<stem>.png` (GUI), `figures/gui/<stem>.png` (manual), and `transcripts/gui/<stem>.gui` (structural render). Anchors: `{#gui-form-<tab>-<sub>}` (`751-mnemonic.md:5`).

---

## Closing: reusable machinery vs must-build-new (for `gui_example.pdf`)

**REUSABLE machinery**
- egui_kittest 0.31.1 snapshot harness pattern: `Harness::builder().with_pixels_per_point(2.0)…` + `try_snapshot_options` + dify **0.6** threshold (`gui_form_snapshots.rs:76,156,174`).
- The whole CI recipe for headless deterministic rendering: lavapipe `mesa-vulkan-drivers` + `WGPU_BACKEND=vulkan` + `GUI_SNAPSHOTS=1`, adapter `device_type==Cpu` guard, `.new.png` census, `.diff.png` artifact upload, `UPDATE_SNAPSHOTS` regen (`build.yml:87-147`).
- Determinism conventions: ppp 2.0, dark theme, embedded fonts, threshold-not-byte, remediation ladder, secret-hygiene assertion (`SPEC_gui_visual_screenshot_track.md:20-24,31`; `gui_form_snapshots.rs:188`).
- Cross-repo split-gate + pin flow: commit corpus in GUI, byte-copy into docs, `pinned-upstream.toml` + `--branch $PINNED_TAG` clone + `verify-figures-gui` byte-diff/census (`manual-gui.yml:41-63`, `lint.sh:204-247`).
- Book build stack: pandoc→xelatex Makefile, `include-transcript.lua`, `--self-contained` HTML, `src="data:image/png"` embed census, lychee, release/gh-pages jobs (`Makefile`, `manual-gui.yml:269-440`).
- Reader-facing scaffolding for a figure gallery (`750-overview.md`, `751-…754`), `<tab>-<sub>` stem convention, `gui-form-*` anchors.
- The widget drive-dispatch primitives for filling inputs already exist (`ui_harness/mod.rs:282-334` — TextEdit/DragValue/ComboBox/Checkbox), and the real subprocess runner exists (`mnemonic-gui/src/runner.rs`).

**MUST BUILD NEW**
- A **whole-window render path** driving the real `eframe::App` (`MnemonicGuiApp::update`, `main.rs:383-384`) so the tab strip (`:424`), subcommand `ComboBox` (`:525`) and output panel (`:456`) appear. NO kittest precedent drives the App shell — every existing snapshot/harness test renders isolated UI closures. This is the largest new piece and likely a P0 spike (does kittest host a paneled eframe App headlessly under wgpu? UNVERIFIED).
- **Filled-form fixtures + per-step input driving** across the WHOLE form (current corpus is blank `render_fixture`; drive dispatch today targets one isolated flag, not a filled multi-field form).
- **Post-Run output capture** (second shot): populate the output panel — either run the pinned CLI via `runner.rs` (determinism: pin inputs, guard timestamps/paths) or inject a canned `RunResult`; enforce output-panel secret redaction (`render_copy_command_masked`, `main.rs:480`).
- **Fixed whole-window sizing + scrolling.** `fit_contents()` cannot express a fixed window with panels; tall forms (≥1253 px) exceed a fixed viewport → need kittest scroll actions (UNVERIFIED, D3) or per-step crop/multi-shot. Spike required.
- **Two-shots-per-step corpus + naming scheme** (e.g. `<journey>-<step>-{form,run}.png`), a new figures dir, a new `verify-tutorial-figures` census gate, a new step↔figure xref, and a new book build target (extend `docs/manual-gui/Makefile` or a fresh `.examples-build`-style dir + a new/extended CI workflow). Note `docs/Examples.pdf`'s generator is currently un-CI'd and manual (E5) — if `gui_example.pdf` follows that model it inherits that gap; if it follows manual-gui it gets full CI gating.
