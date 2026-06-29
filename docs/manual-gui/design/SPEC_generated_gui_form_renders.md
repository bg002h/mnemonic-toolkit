# SPEC — generated, gated structural renders of every GUI form in the manual

**Status:** draft → R0 round-2. **Tier:** docs / test-infra (cross-repo: mnemonic-gui + mnemonic-toolkit/docs/manual-gui). **No funds surface; secret-display hygiene IS in scope.**
**Source:** mnemonic-gui `master` (v0.52.0; the egui_kittest harness from PR #24); manual-gui (`mnemonic-toolkit/docs/manual-gui/`). **R0 r1 RED (4I/5m) folded** (`design/agent-reports/gui-form-renders-spec-r0-round-1.md`).

## 1. Goal
A generated, GATED structural text-render of **every GUI subcommand form** embedded in the GUI manual — extending the manual's existing CLI-transcript "prose == output, gated" discipline to the GUI surface, and replacing the hand-authored `30-tour/*` GUI-form ASCII mockups. The **count is derived dynamically** (`schema_for(tab).subcommands.len()` summed at the pinned GUI tag), NOT hard-coded — **61** (mnemonic 32 + md 10 + ms 10 + mk 9; the same total the harness's `sweep_census` asserts, `tests/ui_harness_sweep.rs`). A **census gate** REDs if ANY subcommand lacks a committed render — **including subcommands with no identity flag** (they still have a form to render; the harness's no-identity filter affects only round-trip *check* coverage, never the form set). (I4 — corrected in R0 r2: the earlier "~64/mnemonic-35" miscounted comment lines containing `conditional: None`; there is no 61-vs-64 gap.)

## 2. Scope of a "structural render" (precise — I2)
A deterministic monospace text representation of a form's **flag grid** as the GUI presents it, under a fixed documented fixture state, per flag in render order: name, kind (text/number/dropdown[opts]/boolean/path), required/secret markers, visible/enabled/pinned state (from `conditional(state)`), default/placeholder. PLUS the **action bar** (Run button — trivial) and the **positionals** (real bounded work: `render_whole_form` renders no positionals today, so the faithfulness leg must add them — m-R3-1). The **bespoke sub-surfaces** (SlotEditor grid, tree builder, archetype param sub-form) are rendered as a single labeled placeholder line (e.g. `[ slot editor: N rows ]`, `[ descriptor tree builder ]`) and are **NOT** field-level faithfulness-gated this cycle (documented limitation; a follow-on may extend them). Example (ASCII-only):

```text
[ mnemonic > inspect ]
  --ms1     text     (required, secret)   -> <masked>
  --mk1     text                          -> <empty>
  --reveal-secret  checkbox               -> [ ] off
  --json    checkbox                      -> [ ] off
  [ Run ]
```

**Glyphs strictly ASCII** (`-> <empty> <masked> [ ] [x] [Run] >`) so the render survives the markdown AND pandoc→xelatex mono renders without glyph loss (m4 — no `▸`/`›`/`⌀`/box-drawing).

## 3. Generation source — RULED: (A) realized as **A1 (feature-gated headless emit-mode)**
Decisive axis = **independent regeneration** (the manual's `verify-examples-gui` must regenerate+diff, or the gate degrades to "file==file" — gating nothing, like (B) would). The GUI form model is egui-free; only 5 modules + `main.rs` touch egui. **A1:**
- In mnemonic-gui: put the 5 egui form modules (`form/widget.rs`, `tree_form.rs`, `slot_editor.rs`, `archetype_form.rs`, `secret_widget.rs`) + `main.rs` behind a **default-on `gui` feature**.
- **GENERAL INVARIANT (the load-bearing extraction — I-R2-2 + I-R3-1):** every *egui-free data type* that lives inside one of the 5 gated modules but is consumed by an *unconditional, non-gated* module MUST be extracted into an egui-free home; the gated module keeps only its egui-dependent methods/widgets. **The empirical completeness gate is that `cargo build --bin gui-render --no-default-features` COMPILES** — the plan's first task. Known instances (the architect's full sweep found exactly these two + no third):
  - **Slot types** `SlotState`/`SlotSubkey`/`SlotRow` out of `slot_editor.rs` (mirroring the existing `tree_form`→`tree_model` split). Consumers that pin them non-gated: `schema/mod.rs` (`FormState.slots`, the primary consumer), `persistence.rs`, `secrets.rs`.
  - **`SecretLineEdit`** (pure `struct { buf: Zeroizing<Vec<u8>> }`; only its `show(&mut egui::Ui)`/`paste_warn_id()->egui::Id` touch egui) out of `secret_widget.rs` into an egui-free module — keep `show`/`paste_warn_id` gated. Consumers non-gated: `schema/mod.rs:322` (`FormState.secret_widgets`), `secrets.rs:323` (zeroize).
- **Also relocate the 4 pure mode-predicates** (`tree_enabled`, `suppressed_in_tree_mode`, `active_archetype`, `suppressed_in_archetype_mode`) to egui-free homes (e.g. `schema/`/`app.rs`).
- **And relocate the canonical default/placeholder resolver** `widget::default_flag_value_for_flag` to an egui-free home (n-R4-1): the emit-mode reuses it as the SINGLE SOURCE OF TRUTH for a flag's default/placeholder, rather than reimplementing it (which would drift; the faithfulness gate would catch the drift, but reuse is the clean design + avoids a new non-gated→gated edge when the emit-mode lands).
- After these extractions, `cargo build --bin gui-render --no-default-features` pulls **none** of eframe/egui/wgpu/winit (I1 fix). The emit-mode derives the §2 render from `schema/` + `conditional(state)` + the relocated mode-predicates — light, headless, deterministic.
- **Faithfulness anchor:** an egui_kittest test (reusing the PR-#24 enumerator + `render_whole_form`) asserts `gui-render`'s emit == the ACTUAL rendered form, for the flag-grid scope `render_whole_form` covers; extend the harness to also assert positionals + the action bar (the action bar is trivial; positionals are real bounded work — `render_whole_form` renders none today, m-R3-1). The sub-surface placeholder lines are out of the faithfulness gate (per §2). This proves the manual's renders match what the GUI renders, at the gated scope.
- (B harness-rendered-cross-repo and C schema-only-no-faithfulness REJECTED — see r1 report.)

## 4. The manual pipeline (mirror the CLI-transcript gate)
- Renders committed under `docs/manual-gui/transcripts/gui/<tab>-<sub>.gui`.
- A fenced block carries `include="gui/<tab>-<sub>.gui"` → the existing content-agnostic `include-transcript.lua` (fail-closed) drops the body in — **no new filter**. Keep each swapped chapter's `gui-schema-coverage` anchors + `outline-coverage` blocks (m3).
- **New `verify-examples-gui`** — a SEPARATE script/Make target (do NOT edit the shared `verify-examples.sh`, which is a symlink to `manual/tests/verify-examples.sh` for `.cmd`/`.out` pairs, m1). It builds/installs the **pinned** `gui-render --no-default-features`, regenerates the `<tab>-<sub>.gui` set, and **diffs == committed** (fail-closed) — the gate the residual lacked. The manual-gui CI (`manual-gui.yml`) already clones the pinned GUI source for `gui-schema-coverage`, so building `gui-render` there is in-pattern.
- **Census gate:** assert every `schema_for(tab)` subcommand has a committed `.gui` (mirror the harness `sweep_census`).

## 5. Cross-repo lockstep
- **mnemonic-gui:** the `gui` feature-gate + **the egui-free extractions** (slot types `Slot{State,Subkey,Row}`, `SecretLineEdit`, the 4 mode-predicates — §3's load-bearing buildability work, do not drop) + the `gui-render` binary + the faithfulness test; new GUI tag (PR+CI, no crates.io).
- **manual-gui:** the `transcripts/gui/*.gui` set, the chapter `include=`s, the `30-tour/*` form-mockup replacement, `verify-examples-gui` + census, **`pinned-upstream.toml` bump** to the new GUI tag AND the 4 `*-tag-implied` CLI pins in lockstep (currently the GUI pin is `mnemonic-gui-v0.49.0` vs model-at-v0.52.0 — this cycle bumps it, m5).
- **Ordering (m4):** the mnemonic-gui leg (the `gui` feature-gate + `gui-render` + faithfulness test) ships + is TAGGED **first**; only then does the manual-gui leg pin that tag and gate against it (the manual's `verify-examples-gui` builds the *pinned* `gui-render`, so the tag must exist before the manual pins it).

## 6. Determinism & secret hygiene (first-class)
Fixed public fixtures per form (reuse the harness `sweep_candidate_bases` valid bases); no RNG/timestamp/PATH in the render. A form with a MODE (e.g. `build-descriptor`: archetype vs tree vs spec) renders one mode per fixture — specify a **generic/default-mode base** per such form so the committed render is the canonical screen (m3); a follow-on may add per-mode renders. **Secret fields:** value rendered as a FIXED masked sentinel (`<masked>`), never cleartext — reuse `secrets::flag_is_secret` + `invocation::render_copy_command_masked` (m2); fixtures are never real keys; the gate's diff output therefore cannot leak a secret. The emit-mode/test never logs a secret value.

## 7. What it catches / non-goals
- **Catches:** the manual's GUI form depiction drifting from the real GUI (a flag added/removed/renamed/re-ordered, a conditional default/visibility change, a secret-masking regression in the documented surface) — fail-closed at `verify-examples-gui` + census.
- **Non-goals / residual (I3 — narrowed):** this closes only the **form-mockup leg** of `manual-gui-output-blocks-non-gateable-residual` (Class-1 form mockups). The output-panel framing (`argv:`/`exit:`/`stdout:` — partly CLI-transcript-covered), the run-confirm modal, help-icon `?` illustrations, and the other residual classes (2–5) remain — file the narrowed residual. Also out: pixel screenshots (user chose structural; a wgpu-snapshot track is separate); the bespoke sub-surface field-level fidelity (§2); UX/layout judgment.

## 8. Companion / tracking
Partially resolves `manual-gui-output-blocks-non-gateable-residual` (form-mockup leg); file the narrowed remainder. FOLLOWUP `manual-gui-generated-form-renders`. Builds on `gui-automated-ui-functionality-harness` (enumerator + render path reused). Cross-repo companion in mnemonic-gui (the `gui` feature-gate + `gui-render` + faithfulness test).
