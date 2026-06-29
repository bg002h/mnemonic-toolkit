# R0 Review — SPEC_generated_gui_form_renders.md (Round 1)

**Reviewer:** opus architect (mandatory pre-implementation R0 gate; 0C/0I required).
**Artifact:** `mnemonic-toolkit/docs/manual-gui/design/SPEC_generated_gui_form_renders.md`
**Scope:** cross-repo docs + test-infra (mnemonic-gui + mnemonic-toolkit/docs/manual-gui). No funds surface; secret-display hygiene IS in scope.
**Verified against:** mnemonic-gui `master` (Cargo.toml v0.52.0, PR-#24 harness present) + manual-gui infra at toolkit `master`.

---

## VERDICT: RED — 0 Critical / 4 Important / 5 Minor-Nit

The architecture is sound and the recommended option (A) is the right call, BUT a **load-bearing premise is false as the code stands** (the "fast, light, NO-GPU binary the manual CI can build like a pinned CLI") and three scoping claims are over-broad/stale. All four are recoverable without abandoning option (A); none force a redesign. Re-dispatch after folding.

**Generation-source ruling: (A), specifically A1 (feature-gated / extracted headless emit-mode).** Evidence and the A-vs-B reasoning are in the dedicated section below.

---

## CRITICAL (0)

None. The design does not contain a funds- or correctness-sinking flaw, and option (A) survives scrutiny.

---

## IMPORTANT (4)

### I1 — The "light, headless, NO-GPU binary" premise is FALSE as written (build-time eframe/wgpu/winit). §3(A) / §3 closing sentence / §4.

§3 sells (A) as "a small headless command … with NO `egui_kittest` and NO GPU … the manual's `make verify-examples-gui` builds/installs the *pinned* `gui-render` (exactly as verify-examples installs pinned CLIs)." This conflates **runtime** with **build-time**. Runtime-headless is true (it opens no window). Build-time is NOT: in cargo, dependencies are per-package, and a `gui-render` binary in the `mnemonic-gui` crate links the lib crate, whose `pub mod form;` (`src/lib.rs:8`) pulls in the five egui-importing modules. So `cargo install --git mnemonic-gui --bin gui-render` (no extra flags) compiles the **entire** `eframe` + `egui` + `wgpu` + `winit` graphics stack plus the `mnemonic-toolkit` git dep — a large, slow build, the opposite of the four light CLIs the manual CI installs today (`secp256k1-sys` C dep only).

Evidence the egui coupling is real but **narrow**:
- egui-importing lib modules (load-bearing `use eframe::egui` / `egui::`): `src/form/widget.rs:15`, `src/form/tree_form.rs:35`, `src/form/slot_editor.rs:11`, `src/form/archetype_form.rs:27`, `src/form/secret_widget.rs:26`, plus the binary `src/main.rs:9`. That is the COMPLETE set (`grep -rln 'use eframe\|use egui'`).
- The form **model is egui-free**: `src/schema/*` (zero egui hits), `src/form/conditional.rs` (imports only `crate::schema` + `serde_json` + `regex`; its single "egui" hit at `:153` is a comment), `src/form/tree_model.rs`, `src/secrets.rs` (egui mentions are comments only), `src/app.rs`, `src/form/invocation.rs`, `src/persistence.rs`, `src/runner.rs`, `src/path_detect.rs` — all import no egui.
- The dep is non-optional: Cargo.toml `:18` `eframe = { version = "0.31", … "wgpu" … }` and `:19 egui = "0.31"` are plain `[dependencies]`, no feature gate, no `optional = true`.

**Consequence:** §3's central premise that (A) is buildable "exactly as verify-examples installs pinned CLIs" does not hold without structural change. The spec must rule one of:
- **(A1, recommended):** put `eframe`/`egui` behind a default-on `gui` feature, `#[cfg(feature="gui")]` the five form modules + `main.rs` (`[[bin]]` main with `required-features=["gui"]`), and build `gui-render` with `--no-default-features` so it pulls NONE of the graphics stack. Feasibility is HIGH and bounded: the egui surface is exactly 5 modules + main; the only model pieces currently trapped inside egui modules are the four pure mode-predicates `tree_form::tree_enabled` (`:60`), `tree_form::suppressed_in_tree_mode` (`:72`), `archetype_form::active_archetype` (`:49`), `archetype_form::suppressed_in_archetype_mode` (`:42`) — relocate them to egui-free homes (`tree_model.rs` is already egui-free; an `archetype` model split). `flag_is_secret` is already egui-free (`secrets.rs:151`); the masked-render helpers live in the egui-free `invocation.rs`. This realizes the spec's stated "light binary" property.
- **(A-but-heavy, acceptable fallback):** ship `gui-render` in the GUI crate as-is and accept the manual CI compiling the full graphics stack. This IS feasible — the GUI's own `build.yml` compiles eframe/wgpu/winit on bare `ubuntu-latest` via `dtolnay/rust-toolchain` with **no apt graphics deps** — so no exotic system packages are needed, but it adds a multi-minute Rust build to manual CI and contradicts the "light" framing.

The spec must pick one and stop describing `gui-render` as unconditionally light. Leaving the premise unqualified is the Important defect.

### I2 — The faithfulness anchor (`render_whole_form`) is a FLAG-GATE projection only; §2's "positional … action bar," sub-surfaces, and the output panel are NOT covered by it.

§3 promises the egui_kittest test "asserts the emit-mode output == the ACTUAL rendered form (the harness's AccessKit tree) for all 61." But the PR-#24 harness's `render_whole_form` (`tests/ui_harness/mod.rs:414-449`) is **explicitly not a faithful full render** — its own doc (`:405-413`) states it "**Deliberately omits the bespoke SUB-SURFACES** (`SlotEditor`, the archetype param sub-form, the tree builder, positional widgets)." It iterates `sub.flags` applying `is_render_suppressed` + Hidden-skip + the `add_enabled_ui(!Disabled)` gate, and nothing else. So:
- An emit-mode validated as `emit == render_whole_form` proves equivalence **only at the flag-gate level** (name / kind / required / disabled / hidden), which `conditional()` does drive — that part is genuinely sound and complete for the flag set.
- It does NOT cover what §2 explicitly lists as in-scope: **positionals** (`md encode [TEMPLATE]`, `md address [PHRASES]`, the card positionals — schema carries `PositionalArgSchema`, but `render_whole_form` renders no positional widget) and **the action bar** (`[ Run ]` / Copy — static GUI chrome `render_whole_form` never emits).
- It does NOT cover the **dominant sub-surfaces** of several forms: `build-descriptor` (tree-builder + archetype param sub-form + the 9 `ARCHETYPE_PARAM_FLAGS`), `bundle`/`restore`/etc. (`--slot` SlotEditor grid), and the run-confirm modal. For `build-descriptor` a flag-only render is actively misleading (the form is mostly the tree/archetype surface).

The harness README (`tests/ui_harness/mod.rs:344-362`) documents WHY whole-form per-widget targeting is hard (egui attaches no label↔input association), which is also why extending the faithfulness test to those sub-surfaces is non-trivial. The spec must (a) **define precisely** what a "structural render" covers (recommend: the flag/positional gate-level structure + a static action-bar line), and (b) for the sub-surface-bearing forms, either extend the harness + add sub-surface fixtures, or scope the sub-surface OUT with a documented in-render note — and NOT claim §2's "positional … action bar" are render-faithful when the current anchor renders neither.

### I3 — "Closes `manual-gui-output-blocks-non-gateable-residual`" is over-claimed; a schema+conditional emit-mode closes only the form-mockup leg of Class 1.

§1/§8 claim the cycle closes the residual. The residual (`docs/manual-gui/FOLLOWUPS.md:41-130`) enumerates **27 fences in 5 classes**. A render derived from `schema/` + `conditional()` can close only the **form-structure** members of Class 1 (e.g. `31-first-launch.md:87` mk-tab form, `:116` paste field, `32-run-and-output.md:103` convert form mockup). It CANNOT produce:
- **Class 1 output/chrome remainder:** the output-panel framing (`32-run-and-output.md:20`, `:68`, `:152` — GUI `argv:`/`exit:`/`stdout:`/`stderr:` framing wrapping CLI output), the **run-confirm modal** (`:112`), the **help-icon `?`** illustrations (`33-…:36`, `:54`, `:71`), the three-panel layout overview (`31-…:17`), the live `Preview:` line (`31-…:127`). The output panel is neither a pure form render (emit-mode can't produce it) nor a raw CLI transcript (the GUI re-frames stdout with `argv:`/`exit:`/`stdout:` prefixes + the show-stdout/stderr header) — §4/§7's "reuses the CLI transcript" does not account for the GUI framing, which stays hand-authored.
- **Classes 2–5 entirely:** ellipsized illustrations (`…`/`...`), URL-formula blocks, and canonical input-paste reference blocks — none are form structure.

The residual's own Status note (`FOLLOWUPS.md:124-128`) anticipated a future harness enabling the mockups to be **screenshot-diffed (pixel)**; the spec deliberately chose STRUCTURAL (§7 non-goals), which addresses the form-structure mockups but not via the mechanism the residual envisioned for the output/modal chrome. Fold: narrow the claim to "closes the form-mockup leg," and **file a narrowed residual** for the output-panel framing + modal + help-icon chrome + Classes 2–5 (or expand scope explicitly to generate those, which is materially larger work).

### I4 — The "61 forms" figure is stale; the current pinned schema has 64 subcommands. Derive dynamically + add a census gate.

The spec hard-codes **61** in its title, §1, §3, §4, §5. The PR-#24 harness comment it lifts this from (`tests/ui_harness/mod.rs:599-642`, `tests/ui_harness_sweep.rs:9`) says `61 = mnemonic 32 + md 10 + ms 10 + mk 9` — but that is a snapshot that has **decayed** (CLAUDE.md's documented citation-decay hazard). Counting `conditional: Some|None` (exactly one per `SubcommandSchema`) in the current source:
- mnemonic: 23 None + 12 Some = **35** (not 32)
- md 10, ms 10, mk 9 — **TOTAL = 64**, not 61. (17 conditional subs, matching the sweep's "17"; the 3 new mnemonic subs are `conditional: None`.)

A plan that authors exactly 61 fixtures/renders under-generates by 3, and the new GUI pin (≥ current master) may add more. Fold: **derive the count from `schema_for(tab).subcommands.len()` summed over the 4 tabs at the pinned tag**; never hard-code. And **adopt a census gate** mirroring the harness's `sweep_census` (`tests/ui_harness_sweep.rs`, README `:17` "census REDs if a sub drops to 0"): `verify-examples-gui` must enumerate ALL subcommands from the pinned schema and FAIL if any lacks a committed `.gui` — this is what turns "added a subcommand, forgot its form render" into a hard RED rather than silent under-coverage.

---

## MINOR / NIT (5)

- **m1 — Don't mutate the shared `verify-examples` runner.** `docs/manual-gui/tests/verify-examples.sh` is a **symlink** to `../../manual/tests/verify-examples.sh` (shared across 3 books; it diffs `.cmd`→run→`.out` pairs, which the per-form emit-all does not fit). §4's "`verify-examples-gui` … joined into `verify-examples`" must add a **separate** script + `make verify-examples-gui` target (and have CI / an aggregate target call both); do not edit the symlinked shared script.
- **m2 — Mandate reuse of the existing secret-mask path; make secret fixtures structurally leak-proof.** §2/§6 are correctly scoped, but should require the emit-mode to mask via the SAME code the GUI uses (`secrets::flag_is_secret` at `secrets.rs:151` + the v0.39.0 secret-mask-preview / `invocation::render_copy_command_masked`), so the faithfulness test proves the mask matches and there is no second masking implementation to drift. Additionally, secret-bearing fixtures should carry only a **fixed masked sentinel** (never even fake-but-key-shaped bytes), so the committed `.gui` golden AND the gate's failure-diff output are structurally incapable of emitting key-shaped material — defense-in-depth consistent with the first-class secret-hygiene bar.
- **m3 — Preserve the existing anchor/outline lints.** New generated-render sections must still satisfy `gui-schema-coverage` (G1 `id="…"` anchors, `lint.sh` phase 4 / `check_gui_schema_coverage.py`) and `outline-coverage` (G2, phase 5). A `.gui` code fence carries no anchor — when swapping a hand-mockup for an `include=`d render, do NOT drop the section's `{#…}` heading / `### Outline` block. The form-render is additive to, not a replacement for, the anchored prose.
- **m4 — Verify Unicode render tokens survive pandoc→xelatex.** The §2 example uses `▸ • ⌀ ☐ ℹ ⚠`. The text gate diffs `.gui` content (fine), but `make pdf` (xelatex) must have these glyphs in the monospace font or the PDF tofus/halts. Confirm against `pandoc/preamble.tex`'s mono font during the plan (PDF-cosmetic only; does not affect the gate).
- **m5 — Lockstep the implied CLI pins + nit on include path.** The new GUI tag bump must also re-pin `[manual-gui] *-tag-implied` in `pinned-upstream.toml` (the file documents this four-field lockstep) and the manual-gui.yml clone/install tags. Nit: `transcripts/gui/<tab>-<sub>.gui` with fence `include="gui/<tab>-<sub>.gui"` resolves correctly via `TRANSCRIPTS_DIR` (the lua filter is content/extension-agnostic — confirmed `include-transcript.lua` `CodeBlock`), so "No new filter" (§4) holds.

---

## GENERATION-SOURCE RULING (the §3 decision the spec asked R0 to settle)

**Recommendation: (A), realized as A1 (feature-gated / extracted headless emit-mode).** Reject (B); (C) already rejected and correctly so.

**Why (A) over (B) — the decisive axis is INDEPENDENT REGENERATION, not source-vs-binary purity.** (A)'s value is that the manual's own `verify-examples-gui` can *regenerate and diff* the 61/64 renders, an independent leading-ish gate that mirrors the CLI-transcript discipline. (B) (harness-rendered files committed in the GUI repo, manual consumes them) cannot regenerate manual-side without building+running the egui_kittest harness; its manual-side gate therefore degrades to "the `include=`d file equals the committed file" — trivially true, gating nothing — so drift detection falls back entirely on the GUI-CI gate + the pin bump, i.e. a **lagging indicator exactly like `schema_mirror`** (CLAUDE.md's documented failure mode). Note the spec's own stated objection to (B) — "couples the manual build to a GUI source checkout" — is weaker than it implies: the manual **already** clones the pinned GUI *source* and runs `check_gui_schema_coverage.py` against it (`manual-gui.yml` Job 1, "Clone mnemonic-gui at pinned tag"). So source-coupling is not the differentiator; regeneration strength is.

**Feasibility of (A1) — HIGH, bounded.** The form model is egui-free (schema/*, `conditional.rs`, `app.rs`, `invocation.rs`, `persistence.rs`, `secrets.rs`, `runner.rs`, `tree_model.rs` import no egui — verified by grep). The egui surface is exactly **5 form modules + `main.rs`** (`widget.rs`/`tree_form.rs`/`slot_editor.rs`/`archetype_form.rs`/`secret_widget.rs`). Gating those behind a default-on `gui` feature + relocating the four pure mode-predicates (`tree_enabled`, `suppressed_in_tree_mode`, `active_archetype`, `suppressed_in_archetype_mode`) to egui-free homes makes `cargo install --bin gui-render --no-default-features` pull none of `eframe`/`egui`/`wgpu`/`winit` — a genuinely light binary the manual CI builds like a CLI. This is the realization that makes the spec's own "light binary" claim true.

**If A1's refactor is judged too large for one cycle, A-but-heavy is an acceptable fallback** (build the full GUI binary in manual CI): the GUI `build.yml` proves eframe/wgpu/winit compiles on `ubuntu-latest` with no extra apt deps, so a new manual-CI job `cargo install --git mnemonic-gui --tag <new> --bin gui-render` works — at the cost of a multi-minute build and the "light" framing. Either path preserves the independent-regeneration gate that makes (A) the right choice over (B). The faithfulness egui_kittest test (the AccessKit-tree anchor) lives in the GUI repo's dev-deps in both A1 and A-but-heavy, so the manual binary never carries it. **The spec must commit to A1 (or explicitly to A-but-heavy) and correct the unqualified "light/NO-GPU/build-like-a-CLI" language in §3.**

---

## What is already SOUND (do not re-litigate)

- The manual pipeline reuse is correct: `include-transcript.lua` is content/extension-agnostic and fail-closed (verified), so `.gui` files need no new filter; `TRANSCRIPTS_DIR` resolution works for `transcripts/gui/…`.
- `conditional()` provably drives the real per-flag render (PR-#24 `render_whole_form` + `tests/conditional_visibility.rs` + `gui_schema_conditional_drift.rs`), so the flag-gate level of the structural render IS faithfully anchorable.
- Determinism holds for the flag-gate render: it derives from static schema + `conditional(fixed fixture)` + static defaults/dropdown-opts; no RNG/timestamp/PATH enters form *structure* (the GUI's PATH-sniffing `path_detect` is a runtime convenience, not schema). The harness already runs run-to-stable for the egui_kittest path.
- Per-form fixture authoring is realistic and largely PRE-DONE: `sweep_candidate_bases` (`tests/ui_harness/mod.rs:647-734`) already encodes minimal-valid bases for all subcommands (+ seeded variants for the 17 conditional subs); the plan can seed the form fixtures from it. Every subcommand has a documentable minimal-valid state (the sweep proves it).
- Secret-hygiene scoping (mask value, public/zero-entropy fixtures, mirror the `:::danger` zero-entropy convention) is first-class-correct in intent; m2 only asks it reuse the existing mask code + a leak-proof sentinel.
