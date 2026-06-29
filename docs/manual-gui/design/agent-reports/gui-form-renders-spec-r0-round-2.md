# R0 Review — SPEC_generated_gui_form_renders.md (Round 2)

**Reviewer:** opus architect (mandatory pre-implementation R0 gate; 0C/0I required).
**Artifact:** `mnemonic-toolkit/docs/manual-gui/design/SPEC_generated_gui_form_renders.md` (draft → R0 round-2).
**Prior:** round-1 RED 0C/4I/5m, all folded (`gui-form-renders-spec-r0-round-1.md`).
**Verified against:** mnemonic-gui `master` @ `01520a5` (Cargo.toml v0.52.0; PR-#24 harness present; `git log afcd28e..HEAD -- src/schema/` = EMPTY → schema unchanged since the harness merged) + manual-gui infra at toolkit `master`.

---

## VERDICT: RED — 0 Critical / 2 Important / 4 Minor-Nit

**Not converged.** Both round-2 Importants are **fold-introduced** in the two *load-bearing* items the round-2 brief flagged for scrutiny: the I4 count fold embedded a **wrong number + a fabricated reconciliation narrative** (the real count is **61, not ~64**; the harness census never "excluded subcommands with no identity flag" from its count — and believing it does leads straight to a census coverage-hole), and the I1/A1 fold's "**gate exactly these 5 modules + relocate exactly these 4 predicates**" is **incomplete** — it omits the egui-free `SlotState`/`SlotRow`/`SlotSubkey` types that two NON-gated modules consume, so `cargo build --bin gui-render --no-default-features` would **fail to compile** as specified. Both are recoverable without redesign (A1 remains the right call); re-dispatch after folding.

What round-1 settled and the folds did NOT damage (confirmed below): the A-over-B generation-source ruling, the §2/§3/§7 sub-surface scoping coherence, the I3 narrowing, and folds m1/m2/m3/m5. Do not re-litigate.

---

## CRITICAL (0)

None. No funds/correctness sink; the architecture is sound and A1 is the right realization.

---

## IMPORTANT (2)

### I-R2-1 — The I4 fold embedded a FALSE count (~64 / mnemonic 35) and a FABRICATED reconciliation narrative. The real count is 61; the harness census never excluded zero-identity subcommands from its total. §1.

§1 now reads: *"empirically ~64 (mnemonic 35 + md 10 + ms 10 + mk 9), to be pinned down at build (and reconciled with the harness sweep's '61', **which excluded subcommands with no identity flag**)."* Both halves are wrong against current `master`:

- **The count is 61, not ~64; mnemonic is 32, not 35.** Triple-confirmed at `01520a5`:
  - Real (non-comment) `conditional:` fields — exactly one per `SubcommandSchema`: mnemonic **32**, md **10**, ms **10**, mk **9** = **61** (`grep -E '^\s*conditional:' | grep -v ':\s*//'`).
  - `SubcommandSchema {` literal count: 32 / 10 / 10 / 9 = **61** (no `#[cfg(test)]` block exists in these data files to inflate it).
  - The harness's own census is a GREEN gate asserting this: `tests/ui_harness_sweep.rs:351` `assert_eq!(n_subs, 61, "expected exactly 61 subcommands across the 4 CLIs")`, where `n_subs = per_sub.len()` and `run_full_sweep` (`:273-300`) iterates `schema_for(tab).subcommands` for all 4 tabs. The schema is untouched since PR #24 merged (`git log afcd28e..HEAD -- src/schema/` empty), so this assert is live-green and authoritative.
  - **Origin of the error:** round-1's I4 computed "mnemonic = 23 None + 12 Some = 35" by counting `conditional: Some|None` and asserting it is "exactly one per SubcommandSchema." It is not — `conditional: None` also appears in **comment** lines (e.g. `src/schema/mnemonic.rs:4536`, `:4599`), inflating the None count by exactly 3 (32→35) and the total by 3 (61→64). The spec folded this miscount in as "empirical."

- **The reconciliation mechanism is fabricated.** There is no 61-vs-64 discrepancy to reconcile, and the harness census did **not** "exclude subcommands with no identity flag" from its count. `run_full_sweep` inserts **every** subcommand into `per_sub` — zero-identity subs land in `no_identity_surface` AND are still counted in `per_sub` (`:281` push to one bucket, `:296` insert to `per_sub` unconditionally). The "no identity flag" filter governs only which subs get an I1 round-trip **CHECK** (`subs_with_cover`), never the **COUNT** (`n_subs`). So the clause misattributes an invented gap to a mechanism that does not touch the total.

- **Why this is Important, not cosmetic — downstream mis-implementation risk + internal contradiction.** This is exactly the "plausible-but-wrong fact" class CLAUDE.md's R0 gate exists to catch. A plan-doc author lifting "mnemonic 35" / "the harness excluded subs with no identity flag" could legitimately conclude that the ~3 zero-identity subcommands are *legitimately un-renderable* and write the form-render **census to EXCLUDE no-identity-flag subcommands** — reintroducing the precise silent under-coverage I4 was filed to close (a subcommand with only positionals/secret/boolean flags still has a full flag grid to render and MUST get a `.gui` + census RED). It also contradicts §4, which correctly censuses *"every `schema_for(tab)` subcommand"* (all 61). §1 and §4 cannot both be right.

**Fold:** State the count as **61 (mnemonic 32 + md 10 + ms 10 + mk 9)** at the to-be-pinned tag — or better, drop the embedded number entirely and say "derived dynamically (currently 61 at the pinned tag)." **Delete** the "reconciled with the harness sweep's '61', which excluded subcommands with no identity flag" clause. Add one sentence making §4's intent explicit and load-bearing: *the census covers ALL `schema_for(tab).subcommands`, INCLUDING zero-identity-flag subcommands (they still render a flag grid / positionals / secret fields).* The dynamic-derivation + census core of the I4 fix is correct and self-correcting — only the prose annotation and the narrative are defective.

### I-R2-2 — The A1 fold is INCOMPLETE: gating `slot_editor.rs` breaks `--no-default-features`, because its egui-free data types (`SlotState`/`SlotRow`/`SlotSubkey`) are consumed by two NON-gated modules. §3.

§3 enumerates A1 as gating exactly the 5 egui form modules (incl. `slot_editor.rs`) behind `gui` and relocating exactly the 4 mode-predicates. The 4-predicate relocation is verified correct and sufficient *for the predicates* (see "confirmed sound" below). But the enumeration inherits round-1 I1's claim that the 4 predicates are *"the only model pieces currently trapped inside egui modules"* — and that claim is **incomplete**. A comprehensive `use crate::form::<gated>` sweep across `src/` finds two load-bearing non-gated→gated edges:

- `src/persistence.rs:30` — `use crate::form::slot_editor::SlotState;` (used at `:121`), and
- `src/secrets.rs:137` — `use crate::form::slot_editor::SlotSubkey;`

`persistence.rs` and `secrets.rs` are declared **unconditionally** in `lib.rs` (they are the redaction + state-persistence modules — and `secrets::flag_is_secret` is the very symbol §6/m2 reuses, so they MUST stay non-gated). `SlotState`/`SlotRow`/`SlotSubkey` are **egui-free pure-data types** (`SlotState { rows: Vec<SlotRow> }`; `SlotSubkey` is a plain enum mirroring `mnemonic-toolkit::slot_input::SlotSubkey`) that merely happen to live in the egui-importing `slot_editor.rs` (`use eframe::egui;` at `:11`). Gate `slot_editor.rs` wholesale and both non-gated modules fail to resolve `SlotState`/`SlotSubkey` under `--no-default-features` → the headline A1 build does not compile.

This is structurally **identical to the already-solved tree_form↔tree_model split** (the egui-free model — `TreeState`/`TreeNode`/… — lives in `tree_model.rs`; the egui widget in `tree_form.rs`). slot_editor never got that split. So A1 must do for slot_editor what the codebase already did for tree_form: **extract `SlotSubkey`/`SlotRow`/`SlotState` into an egui-free home** (e.g. `form/slot_model.rs`), alongside relocating the 4 predicates. Scope is bounded, mechanical, and well-precedented — but it is **additional, load-bearing A1 work the spec does not list**, and the round-2 brief explicitly asked for any "remaining non-optional egui pull A1 doesn't address." This is it (and it is the *only* one — the comprehensive sweep found no other non-gated consumer of any of the 5 gated modules; widget/archetype_form/secret_widget have zero non-gated consumers, and tree_form's data is already in tree_model).

**Fold:** Add the `slot_editor` model/widget split to A1's enumerated work: extract `SlotSubkey`/`SlotRow`/`SlotState` to an egui-free module and re-point `persistence.rs:30` + `secrets.rs:137`. Generalize the A1 invariant from "relocate the 4 predicates" to "relocate every egui-free item consumed by a non-gated module — the 4 predicates AND the slot data types," so a future gated module added with mixed egui/data content is caught by the same rule.

---

## MINOR / NIT (4)

- **m-R2-1 — §2 still uses a non-ASCII glyph while claiming ASCII-safe.** §2 asserts *"Glyphs ASCII-safe (`<empty>`/`<masked>`/`[ ]`/`[x]`/`[Run]`)"* but the example rows retain the non-ASCII `▸` (U+25B8) as the value separator (`▸ <masked>`, `▸ <empty>`, `▸ [ ] off`). The text gate diffs `.gui` content fine, but `▸` is exactly the kind of glyph m4 warned must survive pandoc→xelatex mono. Either drop `▸` for an ASCII marker (`->`, `:`) or add `▸` to the documented safe set with the preamble-font check. Cosmetic / PDF-only; gate unaffected.
- **m-R2-2 — "positionals + action bar … both simple" under-sizes the positional leg.** §3 extends the faithfulness anchor to positionals + action bar, "both simple." The action bar IS trivial (the `[Run]` button is static, presence-only). But `render_whole_form` (`tests/ui_harness/mod.rs:414-449`) currently renders **no positional widget** (its doc `:407` explicitly omits them), and the harness README documents why whole-form widget targeting is hard (no egui label↔input association). Rendering positionals into the kittest tree with targetable handles is bounded but real work, not "simple." The plan should size the positional leg as its own task.
- **m-R2-3 — sub-surface-bearing forms render only one mode per fixture.** §2's single "fixed documented fixture state" means `build-descriptor` renders in exactly one of generic / tree / archetype mode; the `[ descriptor tree builder ]` placeholder only appears under a tree-mode fixture, and the full flag grid only under a generic-mode fixture. The scoping is honest (the flag grid IS gated; the sub-surface is an explicit placeholder), but the plan should specify the fixture mode per such form (recommend generic-mode bases from `sweep_candidate_bases` to maximize the rendered, faithfulness-gated flag grid) so the render is not thin-by-accident.
- **m-R2-4 — §5 ordering correct but not stated as a hard constraint.** §5 lists mnemonic-gui work (new GUI tag, PR+CI) before manual-gui work (pin bump) — the right order, and consistent with the GUI "PR+CI-before-tag" release ritual. The hard sequencing ("the GUI tag must exist/be pushed before `pinned-upstream.toml` can pin it, and before manual-gui.yml's clone-at-pinned-tag can resolve") is structurally obvious but unstated; one explicit sentence removes any ambiguity for the executing instance.

---

## CONFIRMED SOUND — folds that landed correctly (do NOT re-litigate)

- **I1/A1 core — the predicate relocation is necessary AND sufficient for the emit-mode (modulo I-R2-2).** Verified the emit path is fully reachable without the `gui` feature: `conditional()` imports only `crate::schema` (`src/form/conditional.rs:13`; no tree_form/archetype_form/egui dependency). The render loop the emit-mode mirrors (`render_whole_form`) needs exactly `conditional()` + `is_render_suppressed`, and `is_render_suppressed` (`tests/ui_harness/mod.rs:376-399`, mirroring `src/main.rs:624-647`) calls precisely the 4 named predicates — `tree_form::tree_enabled`, `tree_form::suppressed_in_tree_mode`, `archetype_form::active_archetype`, `archetype_form::suppressed_in_archetype_mode`. All 4 bodies are egui-free (string compares + `state.tree`/`state.dropdown_value` + `ARCHETYPE_PARAM_FLAGS` from `schema/archetypes.rs` + `archetypes::find`), so relocating them to `tree_model.rs`/`schema` is valid. Module-level egui coupling is exactly the 6 files round-1 named (`grep 'use eframe|use egui'`); every other `eframe`/`egui` token in `src/` (app.rs, platform.rs, tree_model.rs, secrets.rs, conditional.rs) is a comment. The default-on `gui` feature is non-breaking for the normal GUI build (it gates the 5 modules + `main.rs`'s `required-features`).
- **The residual `mnemonic-toolkit` git dep is correctly out of scope of the "light" claim.** `gui-render --no-default-features` still compiles `mnemonic-toolkit` (non-optional dep, `Cargo.toml:45`, used by `schema/mnemonic.rs`) + secp256k1 — but that is exactly the weight of the `mnemonic` CLI the manual-gui CI **already** builds (`manual-gui.yml` "Install GUI-pinned mnemonic binary"), so "builds like a pinned CLI" is accurate. A1's claim is correctly scoped to "pulls **none of eframe/egui/wgpu/winit**," which holds; `platform.rs`'s `raw_window_handle` is a trait-only crate, not the graphics stack.
- **I2 scope coherence (§2 ↔ §3 ↔ §7).** Consistent: render = flag grid + positionals + action bar (faithfulness-gated) with sub-surfaces as ungated placeholder lines; §3's anchor = `render_whole_form` (flags) + new positional/action-bar assertions; §7 non-goals explicitly excludes "bespoke sub-surface field-level fidelity (§2)." `build-descriptor` is NOT a coverage hole dressed as a limitation — it has a substantial real flag grid that IS rendered and gated; only the tree/archetype internals are placeholdered.
- **I3 narrowing.** §1/§7/§8 now consistently claim only the form-mockup leg of `manual-gui-output-blocks-non-gateable-residual`, with the output-panel framing / run-confirm modal / help-icon / Classes 2–5 left as a filed narrowed remainder. Resolved.
- **Pipeline facts (m1/m3/m5) verified live:** `tests/verify-examples.sh` IS a symlink → `../../manual/tests/verify-examples.sh` (so the separate-script fold is correct); `pinned-upstream.toml` carries `[mnemonic-gui] tag = "mnemonic-gui-v0.49.0"` + the 4 `*-tag-implied` CLI pins (toolkit v0.70.0 / md-cli v0.7.0 / ms-cli v0.8.0 / mk-cli v0.9.0), matching §5's lockstep; `manual-gui.yml` already clones mnemonic-gui at the pinned tag and installs the full CLI tier, so building `gui-render` there is in-pattern. **m2** symbols exist: `secrets::flag_is_secret` (`secrets.rs:151`), `invocation::render_copy_command_masked` (`invocation.rs:524`).

---

## GENERATION-SOURCE RULING — unchanged

(A), realized as A1 (feature-gated headless emit-mode), remains correct. The decisive axis (independent regeneration via `verify-examples-gui`, vs (B)'s file==file degenerate gate) is settled and undamaged by the folds. The only A1 defect is the slot_editor extraction omission (I-R2-2), which does not alter the ruling.

---

## Gate

**0 Critical / 2 Important → RED.** Fold I-R2-1 (correct the count to 61 / mnemonic 32; delete the fabricated "excluded subcommands with no identity flag" reconciliation; make the census's all-subcommand coverage explicit) and I-R2-2 (add the slot_editor `SlotState`/`SlotRow`/`SlotSubkey` egui-free extraction to A1's work list and re-point `persistence.rs:30` + `secrets.rs:137`), then re-dispatch for round-3 (a fold can introduce fresh drift — the loop continues until GREEN). The 4 minors are recommended-but-non-blocking.
