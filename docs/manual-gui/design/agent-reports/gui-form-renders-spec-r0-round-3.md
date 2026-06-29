# R0 Review — SPEC_generated_gui_form_renders.md (Round 3)

**Reviewer:** opus architect (mandatory pre-implementation R0 gate; 0C/0I required).
**Artifact:** `mnemonic-toolkit/docs/manual-gui/design/SPEC_generated_gui_form_renders.md` (draft → R0 round-3).
**Prior:** round-1 RED 0C/4I/5m (folded); round-2 RED 0C/2I/4m (folded — `gui-form-renders-spec-r0-round-2.md`).
**Verified against:** mnemonic-gui `master` @ `01520a58` (Cargo.toml v0.52.0; PR-#24 harness present) + manual-gui infra at toolkit `master`. All facts below are live-grep-confirmed at `01520a58`.

---

## VERDICT: RED — 0 Critical / 1 Important / 2 Minor-Nit

**Not yet converged — but the gap is one residual instance of the *same class* round-2 opened, not a fresh defect.** The round-2 count fold (I-R2-1) is now **fully correct** (verified below). The round-2 slot-extraction fold (I-R2-2) was applied — but the **sweep that fold rested on was itself incomplete**: round-2 asserted *"the comprehensive sweep found no other non-gated consumer of any of the 5 gated modules; widget/archetype_form/**secret_widget have zero non-gated consumers**."* That assertion is **false**. The non-gated `schema::FormState` (in `src/schema/mod.rs`, which MUST stay non-gated — `conditional.rs`, `secrets.rs`, `persistence.rs` all consume it) embeds `secret_widget::SecretLineEdit` (`schema/mod.rs:322`). So A1 as written **still fails `cargo build --bin gui-render --no-default-features`** — gating `secret_widget.rs` leaves `SecretLineEdit` unresolved. The fix is the identical mechanical extraction the slot fold already established; no redesign, A1 ruling stands. This is precisely the load-bearing buildability question the round-3 brief flagged, and it lands on the Important side because it breaks the headline deliverable's compile.

What round-1/round-2 settled and the latest folds did NOT damage (re-confirmed below): the A-over-B ruling, the count (now 61/32), §2/§3/§7 sub-surface scoping, I3 narrowing, the predicate relocation, the slot extraction (correct as far as it goes), the ASCII glyph fix, the §5 ordering sentence. Do not re-litigate.

---

## CRITICAL (0)

None. No funds/correctness sink; architecture sound; A1 is the right realization.

---

## IMPORTANT (1)

### I-R3-1 — A1 STILL does not compile under `--no-default-features`: gating `secret_widget.rs` orphans `secret_widget::SecretLineEdit`, which the NON-gated `schema::FormState` consumes. Round-2's sweep missed it (and wrongly declared `secret_widget` has zero non-gated consumers). §3.

§3's A1 now gates the 5 egui modules (incl. `secret_widget.rs`), relocates the 4 predicates, and extracts the slot data types (`SlotSubkey`/`SlotRow`/`SlotState`). That fixes the slot edge. But it does **not** fix the structurally-identical `secret_widget` edge, which round-2's sweep declared absent.

**The edge is real and load-bearing — grep-verified at `01520a58`:**

- `src/schema/mod.rs:322` — `FormState.secret_widgets: BTreeMap<String, Vec<crate::form::secret_widget::SecretLineEdit>>`. This is a **field of `FormState`**, the central form-model struct.
- `FormState` lives in `src/schema/mod.rs`, declared `pub mod schema;` **unconditionally** in `lib.rs` (no `cfg(feature)` gating anywhere in `lib.rs`). It MUST stay non-gated: the emit-mode's core, `src/form/conditional.rs`, imports `crate::schema::FormState` (`conditional.rs:13`) and **every** predicate takes `&FormState` (`bundle(state: &FormState)`, `build_descriptor(state: &FormState)`, …). The spec itself (§3) says emit "derives the §2 render from `schema/` + `conditional(state)`." No `FormState`, no emit-mode.
- `SecretLineEdit` is a **pure egui-free data type**: `pub struct SecretLineEdit { buf: Zeroizing<Vec<u8>> }` (`secret_widget.rs:42-44`). Its egui coupling is confined to **methods** — `show(&mut self, ui: &mut egui::Ui, …)` (`:78`) and `paste_warn_id() -> egui::Id` (`:36`). The struct/data is egui-free; the module imports `eframe::egui` (`:26`).
- The non-gated `secrets.rs` ALSO depends on it transitively: `secrets.rs:323` iterates `state.secret_widgets.values_mut().flatten()` to zeroize each `SecretLineEdit` buffer (the very `zeroize_form_state` hygiene path), and `:235` reads `state.secret_widgets.get(...)`. And `secrets::flag_is_secret` is the symbol §6/m2 reuses — so `secrets.rs` is unconditionally non-gated by design.

So gate `secret_widget.rs` wholesale (which A1 must — it imports `eframe::egui` and uses `egui::TextEdit`/`Ui`/`Id`/`Event`; **not** gating it defeats A1's "pulls **none** of eframe/egui/wgpu/winit" claim) and the non-gated `schema/mod.rs` + `secrets.rs` fail to resolve `SecretLineEdit` under `--no-default-features` → **the headline A1 build does not compile.** This is the same failure mode I-R2-2 caught for slot_editor — round-2 fixed one of the two mixed-content modules and explicitly (wrongly) cleared the other.

**Why Important, not Minor:** it breaks the compile of the central deliverable (`gui-render --no-default-features`) — the build the whole `verify-examples-gui` gate stands on. Round-2 graded the identical slot break Important; parity demands the same here. Recoverable without redesign.

**Fold (mirror the slot extraction exactly):**
1. Extract `SecretLineEdit` (the `struct` + its egui-free inherent methods — `new`/`from_text`/`default`/the buffer accessors/zeroize hook) into an **egui-free module** (e.g. `form/secret_model.rs`), and keep the egui-coupled `show(&mut egui::Ui, …)` + `paste_warn_id() -> egui::Id` in the gated `secret_widget.rs` (inherent `impl` blocks may be split across modules within one crate — same trick the slot/tree splits use). Re-point `schema/mod.rs:322` (and confirm `secrets.rs`'s `state.secret_widgets` access resolves through `FormState`, which it will once the field's type resolves).
2. **Correct the slot extraction's re-point list too:** §3 names only `persistence.rs` + `secrets.rs` as the reason for the slot extraction, but the **primary** non-gated consumer is `schema::FormState` itself — `schema/mod.rs:293` (`pub slots: …::SlotState`), `:353`, `:363`. List the full non-gated consumer set: `schema/mod.rs` (FormState — BOTH `SlotState` and `SecretLineEdit`), `persistence.rs:30`, `secrets.rs:137`.
3. **Generalize the A1 invariant** (round-2 recommended this and it was not implemented): state it as *"relocate every egui-free item consumed by a non-gated module — the 4 mode-predicates, the slot data types, AND the secret-widget data type"* — and add the standing rule *"a gated module mixing egui widgets with egui-free data must split its data into an egui-free sibling (as `tree_form→tree_model` already does)."* That converts the recurring whack-a-mole into a checkable invariant so the next mixed module is caught structurally, not by a fresh sweep.

(For completeness: the comprehensive re-sweep below confirms `secret_widget` + `slot_editor` are the **only** two such edges. `widget` is consumed only by gated modules + doc-comments; `tree_form` only by gated `main.rs`; `archetype_form` only by gated `main.rs`/`tree_form` plus its 2 predicates already in the relocation list. So after this fold the build path is complete — no third surprise.)

---

## MINOR / NIT (2)

- **m-R3-1 — §3 still calls positionals "(both simple)", contradicting §2's corrected "real bounded work".** The m-R2-2 fold updated §2 to size positionals correctly (*"real bounded work: `render_whole_form` renders no positionals today, so the faithfulness leg must add them"*), but §3's faithfulness-anchor sentence still reads *"extend the harness to also assert positionals + the action bar (**both simple**)."* Stale parenthetical; the two sections now disagree on the same leg. Make §3 consistent: action bar = trivial, **positionals = bounded real work** (render targetable positional handles into the kittest tree). Cosmetic; no gate impact.
- **m-R3-2 — §5's mnemonic-gui leg list does not name the egui-free extractions.** §5 lists *"the `gui` feature-gate + predicate relocation + the `gui-render` binary + the faithfulness test"* but omits the slot **and** (per I-R3-1) secret-widget data-type extractions. Arguably subsumed under "the `gui` feature-gate," but since these extractions are the load-bearing buildability work, name them in §5's lockstep list so the executing instance can't drop them. Fold alongside I-R3-1.

---

## CONFIRMED SOUND — folds that landed correctly (do NOT re-litigate)

- **I-R2-1 (the count) — NOW FULLY CORRECT.** Verified four ways at `01520a58`:
  - `grep -c 'SubcommandSchema {'`: mnemonic **32** / md **10** / ms **10** / mk **9** = **61**.
  - non-comment `^\s*conditional:` (one per `SubcommandSchema`): 32 / 10 / 10 / 9 = **61**.
  - Harness live-green gate: `tests/ui_harness_sweep.rs:351` `assert_eq!(n_subs, 61, "expected exactly 61 subcommands across the 4 CLIs")` (and `:9` / `:333` strings: "all 61 subcommands (mnemonic 32 + md 10 + …)"). Schema untouched since the harness merged → authoritative.
  - §1 now states **61 (mnemonic 32 + md 10 + ms 10 + mk 9)**, explicitly notes the "~64/mnemonic-35" was a miscount of comment lines carrying `conditional: None`, and carries NO reconciliation narrative. `grep '64|35|reconcil|excluded subcommands'` over the spec → only the corrective sentence; the fabrication is gone.
  - **§1 ↔ §4 consistent.** §1's census now reads *"REDs if ANY subcommand lacks a committed render — **including subcommands with no identity flag** (they still have a form to render; the harness's no-identity filter affects only round-trip *check* coverage, never the form set)"*, matching §4's *"every `schema_for(tab)` subcommand."* The zero-identity-exclusion trap I-R2-1 warned about is closed — no exclusion re-introduced. (Mechanism verified: `run_full_sweep` inserts every sub into `per_sub` unconditionally; the no-identity filter governs only `subs_with_cover`, never `n_subs`.)
- **I-R2-2 (slot extraction) — correct as far as it goes.** The added `SlotSubkey`/`SlotRow`/`SlotState` extraction is the right move and the right precedent (`tree_form→tree_model`). It just under-listed its consumers and left the parallel `secret_widget` case un-swept (→ I-R3-1).
- **The predicate relocation (round-1 I1) remains necessary + sufficient for the 4 predicates.** `conditional.rs` imports only `crate::schema`; `is_render_suppressed` calls exactly `tree_form::{tree_enabled,suppressed_in_tree_mode}` + `archetype_form::{active_archetype,suppressed_in_archetype_mode}`, all egui-free bodies. Relocation valid.
- **m-R2-1 (ASCII) — FOLDED.** The render example (spec lines 13-19) uses `->`, `[ ]`, `<empty>`, `<masked>`, `[ Run ]` — strictly ASCII. The only non-ASCII bytes in the file are in **prose** (em-dash, `§`, `==`, `→`, and the m4 sentence that *deliberately* lists the forbidden `▸`/`›`/`⌀`). The gated `.gui` render content is ASCII-clean.
- **m-R2-3 (fixture mode) / m-R2-4 (ordering) — FOLDED.** §6 now specifies a *"generic/default-mode base per such form"* for build-descriptor; §5's *"Ordering (m4)"* sentence states the hard sequencing (GUI tag pushed first, manual pins it second). Both resolved.
- **Pipeline facts re-spot-checked:** `form/invocation.rs` has **0** egui imports → `render_copy_command_masked` reuse + leaving `invocation.rs` un-gated is correct (m2 sound). `verify-examples.sh` symlink, `pinned-upstream.toml` GUI pin v0.49.0-vs-model-v0.52.0, and `manual-gui.yml` clone-at-pinned-tag pattern all unchanged from round-2's verification.

---

## GENERATION-SOURCE RULING — unchanged

(A), realized as A1 (feature-gated headless emit-mode), remains correct. The only A1 defect is the still-incomplete egui-free extraction set (I-R3-1, the `secret_widget` twin of the slot case); it does not touch the ruling.

---

## Gate

**0 Critical / 1 Important → RED.** Fold I-R3-1 (extract `secret_widget::SecretLineEdit` to an egui-free module + re-point `schema/mod.rs:322`; correct the slot re-point list to include `schema/mod.rs:293/353/363`; generalize the A1 invariant + add the "split mixed egui/data modules" standing rule) and the 2 minors, then re-dispatch for round-4. The count fold (I-R2-1) is verified converged; this is the last buildability edge — once `secret_widget` is extracted, the comprehensive sweep shows no further non-gated→gated edge, so convergence at round-4 is expected.
