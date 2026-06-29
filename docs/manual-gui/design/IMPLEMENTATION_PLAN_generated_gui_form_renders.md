# IMPLEMENTATION PLAN — generated, gated GUI form renders (+ manual catch-up)

**Spec:** `SPEC_generated_gui_form_renders.md` (R0-GREEN, 4 rounds). **Status:** draft → plan-R0 round-2. **Plan-R0 r1 RED (2I/7m) folded.**
**Two legs, ordered:** mnemonic-gui (refactor + `gui-render` + faithfulness test → **new GUI tag**) ships FIRST; then manual-gui (catch-up + pin bump, then the renders + gate). Each phase: tests-first → impl → per-phase R0 to 0C/0I; a post-impl whole-diff PER LEG before each merge.
**GOTCHAS:** GUI clippy `--all-targets -D warnings` gated (`#![allow(dead_code)]` on shared test mods); GUI has NO fmt gate; `cargo test --jobs 2` (linker-OOM); Leg-1 needs a release TAG (the manual pins it).

## LEG 1 — mnemonic-gui (branch `feat/gui-render-form-emit`)

### P1 — egui-free extraction refactor + the shared `render_fixture` (the buildability gate)
**Files (mnemonic-gui):** `Cargo.toml` (default-on `gui` feature; gate `eframe`/`egui`/`egui-wgpu`/`winit` under it; **`[[bin]]` for `main` with `required-features=["gui"]`** — m1, else `--no-default-features` fails "main not found"); `lib.rs` (`#[cfg(feature="gui")]` on the 5 form modules + the `main` bin); the extractions to egui-free homes:
- `Slot{State,Subkey,Row}` ← `slot_editor.rs`; `SecretLineEdit` (struct + egui-free methods; keep `show`/`paste_warn_id` gated) ← `secret_widget.rs`; `default_flag_value_for_flag` **AND `default_flag_value_for`** (m5) ← `widget.rs`; the 4 mode-predicates ← egui-free home (`schema/`/`app.rs`). Re-point ALL non-gated consumers (`schema/mod.rs` FormState.slots/.secret_widgets, `persistence.rs`, `secrets.rs`) **and the PR-#24 harness consumers** (`tests/ui_harness/*` import moved types — update imports, m6).
- **`render_fixture(tab, sub) -> FormState`** (I2): ONE egui-free shared canonical fixture source, consumed by BOTH the emit binary (P2) and the faithfulness test (P3). Canonical base = `FormState::default()` (already `sweep_candidate_bases`'s first element for all 61 forms — no big relocation needed). Egui-free, in `src`.
**Gate:** `cargo build -p mnemonic-gui --no-default-features` **COMPILES** (verify `cargo tree --no-default-features` shows no eframe/egui/wgpu/winit); default build (`gui` on) + **full existing suite green** (`cargo test --jobs 2`, incl. the PR-#24 harness/`schema_mirror`/`persist_redaction`/`secret_taxonomy_pin` — behavior-preserving); clippy `-D warnings` clean both configs.

### P2 — the `gui-render` emit binary
**Files:** new `src/bin/gui_render.rs` + **`[[bin]] name="gui-render"`** (hyphen, m2), NOT gated. CLI `gui-render --form <tab> <sub>` + `--emit-all <dir>`. Emits the §2 ASCII render from `schema/` + `conditional(render_fixture(tab,sub))` + `default_flag_value_for_flag`: flag grid (name/kind/required/secret/visible/enabled/pinned) + positionals + action bar + sub-surface placeholder lines. Secret fields → fixed `<masked>` sentinel (reuse `flag_is_secret`; never cleartext). **Stable byte encoding** (LF, UTF-8→ASCII-only, sorted-where-applicable) for cross-machine regen determinism (m7).
**Gate:** unit tests asserting the exact ASCII for ≥3 representative forms (incl. a secret-bearing + a mode form like `build-descriptor` showing the placeholder); `--emit-all` writes 61 files; byte-deterministic across runs + machines; `--no-default-features` binary builds + runs.

### P3 — the egui_kittest faithfulness test (tree-observable projection)
**Files:** new `tests/gui_render_faithfulness.rs`. For all 61 forms (same `render_fixture`): assert `gui-render`'s emit agrees with the ACTUAL rendered form **on the TREE-OBSERVABLE projection ONLY** (m4): flag presence, disabled/hidden, Role-class, `PasswordInput` for secrets, positional presence, action bar. **NOT** path-vs-text / required-marker / default-text (not AccessKit-recoverable — those are covered by P5's regen-determinism + `schema_mirror`). Reuse the PR-#24 enumerator + `render_whole_form`; extend it to render+observe positionals + the action bar. Sub-surface placeholders out of the faithfulness gate. Census: all 61.
**Gate:** faithfulness test green; full suite + clippy green.

### LEG-1 RELEASE: P1–P3 per-phase R0 + a leg-1 post-impl whole-diff → PR → CI green → merge → **tag** (GUI MINOR — new `gui-render` bin + `gui` feature; GUI app behavior-preserving). No crates.io.

## LEG 2 — manual-gui (branch `feat/manual-gui-form-renders`; AFTER the GUI tag)

### P4 — manual catch-up + pin bump (I1 — the forced, budgeted prerequisite)
Bumping the pin to the new GUI tag exposes 5 subcommands the manual omits; the **zero-tolerance bidirectional `gui-schema-coverage`** gate REDs until documented. So:
**Files:** `pinned-upstream.toml` → new GUI tag + the 4 implied CLI pins (knowable from `mnemonic-gui/pinned-upstream.toml`: toolkit-v0.74.0 / md-v0.11.0 / ms-v0.13.0 / mk-v0.11.0 — **note these also move the CLI-transcript tier v0.70→v0.74**); the `verify-examples` job's hardcoded tags (version-site #2); new manual sections + per-flag anchors for **`word-card`** (mnemonic, ~10 flags — full section, **incl. its `outline-coverage` block + glossary + index entries**, not just schema anchors), **`gen-man`** (×4 CLIs — stubs), **`inspect --json`**; regenerate `expected_gui_schema_inventory.json` (hygiene). (The 3rd lockstep version-site — the `gui-render` install line — is created in P5, m3-wording.)
**Gate:** `make lint` (incl. `gui-schema-coverage` bidirectional + `outline-coverage` + glossary + index) green against the bumped pin; **`make verify-examples` green** (the implied-pin bump moves the CLI-transcript tier feeding the SEPARATE Job-1b `.out` gate over 17 goldens — likely a no-op since intervening cycles ADDED subcommands rather than changing existing output, but must be confirmed; N1); `make md`/`html` build green.

### P5 — generate + embed + gate the 61 renders + replace mockups
**Carry (P2-R0 Minor-1):** the `(required)` marker is conditional-sourced, so at-least-one prompt groups (e.g. `mnemonic inspect`'s ms1/mk1/md1, all static `required=false`) render as all-`(required)` though the toolkit accepts any ONE — add a one-line manual caveat near those renders so the grid isn't read conjunctively.
**Files:** install/build the **pinned** `gui-render --no-default-features` (mirror the manual's pinned-CLI install; CI `manual-gui.yml` Job-1b `cargo install` is the in-pattern hook — m3); `gui-render --emit-all docs/manual-gui/transcripts/gui/` → 61 `<tab>-<sub>.gui`, committed; chapters get fenced `include="gui/<tab>-<sub>.gui"` (reuse `include-transcript.lua`); the `30-tour/*` hand-drawn GUI-form mockups replaced by the `include=`d renders (keep `gui-schema-coverage`/`outline` anchors); new **`verify-examples-gui`** script + Make target (SEPARATE from the symlinked `verify-examples.sh`) that builds the pinned `gui-render` + regenerates + **diffs == committed** (fail-closed); a **census** assert (every `schema_for(tab)` sub has a `.gui`, incl. zero-identity subs); extend `manual-gui.yml` to build `gui-render` + run `verify-examples-gui`.
**Gate:** `make verify-examples-gui` + census green; `make lint`/`md`/`html` green; `manual-gui.yml` passes.

### LEG-2 RELEASE: P4–P5 per-phase R0 + a leg-2 post-impl whole-diff → manual PR → CI → merge.

## FOLLOWUPs (at ship)
RESOLVE the form-mockup leg of `manual-gui-output-blocks-non-gateable-residual` + `manual-gui-generated-form-renders`; FILE the narrowed remainder (output-panel/modal/Classes 2–5); cross-repo companion in mnemonic-gui.

## Risk / sequencing
- **P1 = the risk gate** (real GUI module refactor); the `--no-default-features` compile + behavior-preserving full-suite are the proof; the spec's general invariant + build-gate absorb any new edge.
- **P4 = the unbudgeted-now-budgeted catch-up** (word-card is the bulk; gen-man/inspect-json small) — author it before the renders so the pin bump is coverage-green.
- **Cross-repo hard dep:** Leg 2 needs the Leg-1 GUI tag; don't start P4 until it's pushed.
- **Determinism + secret hygiene** first-class (P2): fixed `render_fixture`, masked secrets, no leak in render or gate diff.
