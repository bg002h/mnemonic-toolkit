# R0 Review — SPEC_generated_gui_form_renders.md (Round 4)

**Reviewer:** opus architect (mandatory pre-implementation R0 gate; 0C/0I required).
**Artifact:** `mnemonic-toolkit/docs/manual-gui/design/SPEC_generated_gui_form_renders.md` (draft → R0 round-4).
**Prior:** r1 RED 0C/4I/5m → r2 RED 0C/2I/4m → r3 RED 0C/1I/2m (all folded).
**Verified against:** mnemonic-gui `master` @ `01520a58ae29323bbdb29c6b2a6ee5e6712c67c0` (Cargo.toml v0.52.0; PR-#24 harness present) + manual-gui infra at toolkit `master`. Every fact below is live-grep-confirmed at `01520a58`.

---

## VERDICT: GREEN — 0 Critical / 0 Important / 2 Minor-Nit (non-blocking)

**Converged.** Round-3's prediction holds: after the `SecretLineEdit` extraction, an independent comprehensive re-sweep of all 5 gated egui modules finds **no third non-gated→gated compile edge**. A1 is buildable as specified. The two r3→r4 minors (m-R3-1 stale "both simple", m-R3-2 §5 extraction-naming) are folded cleanly, the general invariant + build-compiles gate are sound and concrete (not hand-wavy), and §2/§3/§5 are mutually consistent. The count (61/32) and census remain correct. The 2 nits below are forward-looking implementation-guidance only; they do **not** break the compile, are backstopped by the faithfulness gate, and do **not** hold the gate.

---

## CRITICAL (0)

None.

## IMPORTANT (0)

None.

---

## THE LOAD-BEARING CONVERGENCE CHECK — independent comprehensive A1 buildability sweep

Method: at `01520a58`, established the **true** egui-coupling set (strict code-ref grep, comments stripped), then for EACH of the 5 gated modules enumerated every cross-module reference and classified each consumer as gated / non-gated / comment-only.

**(0) The gating set is exactly right.** Strict `\b(eframe|egui)\b` code-refs (doc/line-comments excluded) appear in **only**: `form/{widget, tree_form, slot_editor, archetype_form, secret_widget}.rs` + `main.rs`. Confirmed egui-FREE (zero non-comment refs): `app.rs` (its own header says the eframe loop lives in `main.rs`; it is the data layer), `platform.rs`, `secrets.rs`, `schema/mod.rs`, `form/{conditional, invocation, tree_model}.rs`. So gating the 5 form modules + `main.rs` is necessary **and sufficient** to keep egui out of the `--no-default-features` lib path. `gui-render` is a separate bin, so `main.rs`/`app.rs` egui status is irrelevant to it.

**(1) slot_editor — COVERED.** egui-free pub surface: `SlotSubkey`, `SlotRow`, `SlotState` (+ inherent methods `as_str`/`is_secret_bearing`/`rows_sorted`/`persistable_rows`/`to_slot_argv*`/`detect_slot_index_gaps`/`remove_row`). Non-gated consumers: `schema/mod.rs:293,353,363` (`FormState.slots: SlotState` — the **primary** consumer), `persistence.rs:30` (`SlotState`), `secrets.rs:137` (`SlotSubkey`). All three are named in §3's slot extraction. The egui-free inherent methods ride along with their data type; only `render(ui: &mut egui::Ui, …)` stays gated. ✓

**(2) secret_widget — COVERED (the r3 fix).** egui-free pub surface: `SecretLineEdit` (`struct { buf: Zeroizing<Vec<u8>> }` + `new`/`from_text`/`as_string`/`is_empty`/`zeroize`). egui-coupled (stay gated): `show(&mut egui::Ui,…)` `:78`, `paste_warn_id()->egui::Id` `:36`. Non-gated consumers: `schema/mod.rs:322` (`FormState.secret_widgets: …Vec<SecretLineEdit>`), and transitively `secrets.rs:235/323` + `persistence.rs:135` (through the `FormState` field). All named in §3. ✓

**(3) widget — NO non-gated edge.** egui-free pub exports `display_or`, `default_flag_value_for`, `default_flag_value_for_flag`, `seeded_value_for`, `RepeatAnnotation` are consumed **only** by gated `archetype_form`/`tree_form` and by `widget` itself (lazy default-seeding at `:217/:308/:741`). Zero non-gated consumers (grep-confirmed; the apparent `schema/mod.rs:322` hit is the `secret_widget::` substring, and `invocation.rs:279` is a comment). ✓

**(4) tree_form — NO non-gated edge beyond the covered predicates.** Current non-gated references (`tree_model.rs:675`, `invocation.rs:552`, `schema/mod.rs:338`) are **all doc-comments**. Real callers are gated `main.rs`. The only egui-free exports the (future, non-gated) emit-mode needs are the mode-predicates `tree_enabled`/`suppressed_in_tree_mode` — already in §3's relocation list. ✓

**(5) archetype_form — NO non-gated edge beyond the covered predicates.** Real callers: gated `tree_form`/`main.rs`. Emit-mode-needed egui-free exports = `active_archetype`/`suppressed_in_archetype_mode` — already in the relocation list. ✓

**Predicate relocation is correct and its target homes are valid.** The 4 predicates are defined in gated `tree_form.rs`/`archetype_form.rs` and (at `01520a58`) called only from gated `main.rs`/`tree_form`. The relocation is **forward-looking** — the new non-gated emit-mode must reproduce mode-suppression to compute visible/enabled/pinned state — which is exactly what §3 says ("emit derives … from `schema/` + `conditional(state)` + the relocated mode-predicates"). (Note: there is no `is_render_suppressed` symbol in the tree at this SHA — a harmless imprecision in r3's prose; the spec body never relies on it.) §3's suggested homes "`schema/`/`app.rs`" are both verified egui-FREE, and both can see `FormState` (which the `&FormState` predicates need), so no circular-dep. ✓

**Conclusion: the two named extractions (`Slot{State,Subkey,Row}` + `SecretLineEdit`) + the 4 predicate relocations cover EVERY non-gated→gated egui-free dependency in the crate. `cargo build --bin gui-render --no-default-features` will compile. No third edge. A1 is buildable.**

---

## FOLD-CLEANLINESS — r3→r4

- **I-R3-1 (extract `SecretLineEdit`) — FOLDED, correct.** §3 line 28 extracts `SecretLineEdit` to an egui-free home, keeps `show`/`paste_warn_id` gated, names the exact non-gated consumers (`schema/mod.rs:322`, `secrets.rs:323`) — matches grep. The slot re-point list was corrected to lead with the **primary** consumer `schema/mod.rs (FormState.slots)` (r3 fold item #2). ✓
- **General invariant + build-compiles gate — SOUND, not hand-wavy.** §3 line 26 states the invariant as a structural rule ("every egui-free data type … consumed by an unconditional non-gated module MUST be extracted; the gated module keeps only its egui-dependent methods/widgets") with a **concrete, checkable empirical gate**: `cargo build --bin gui-render --no-default-features` COMPILES, named as "the plan's first task." This converts the recurring whack-a-mole into a CI-checkable invariant (r3 fold item #3). ✓
- **m-R3-1 (positionals) — FOLDED.** The stale "(both simple)" is gone (grep: NONE). §3 line 31 now reads "the action bar is trivial; positionals are real bounded work — `render_whole_form` renders none today," consistent with §2 line 10's "real bounded work." ✓
- **m-R3-2 (§5 names the extractions) — FOLDED.** §5 line 41 lists "the egui-free extractions (slot types `Slot{State,Subkey,Row}`, `SecretLineEdit`, the 4 mode-predicates — §3's load-bearing buildability work, do not drop)." §3↔§5 consistent. ✓
- **Count (61/32) + census — still correct.** Schema source at `01520a58`: `mnemonic.rs` 32 + `md.rs` 10 + `ms.rs` 10 + `mk.rs` 9 = **61**; harness `ui_harness_sweep.rs:351` `assert_eq!(n_subs, 61, …)`. §1 carries 61 with the I4 corrective sentence and no 64/35 reconciliation narrative. §1↔§4↔§5 census wording consistent (every `schema_for(tab)` subcommand, no-identity included). The r3→r4 fold did not touch any count/census text. ✓
- **No new contradiction / dangling section-ref introduced.** ✓

---

## MINOR / NIT (2 — non-blocking, do NOT hold the gate)

- **n-R4-1 — emit-mode default/placeholder resolution is unstated; consider naming `widget::default_flag_value_for_flag` as the single source of truth.** §2 includes "default/placeholder" in the render scope, and the GUI computes a flag's initial value lazily via the egui-FREE `widget::default_flag_value_for_flag(flag)` (`widget.rs:217/308/741`) — which currently lives in the **gated** `widget.rs`. This is **not a compile edge** (zero non-gated consumers today, so it does not break `--no-default-features`), and it is **backstopped**: the faithfulness test (`emit == actual rendered form`) goes RED if `gui-render` re-derives defaults and drifts from `default_flag_value_for_flag`. So correctness cannot silently ship wrong. But for a clean single-source-of-truth — exactly the precedent §3 already set by forward-relocating the mode-predicates the emit-mode needs — the spec could optionally note that `gui-render` should reuse `default_flag_value_for_flag` (relocating it egui-free) rather than re-implement the FlagKind→default mapping (Dropdown→opts[0], Unset kinds, schema-`default_value` passthrough). Implementation-guidance nicety; no redesign; the faithfulness gate makes it safe either way.
- **n-R4-2 — provenance-tag cosmetics.** §2's positionals parenthetical tags the conclusion "— m2)" while §3 tags the same conclusion "— m-R3-1"; both sections AGREE on substance ("real bounded work"), so this is a stale provenance marker, not a contradiction. Optional cleanup.

---

## CONFIRMED-SOUND CARRY-FORWARDS (settled r1–r3; re-spot-checked, NOT re-litigated)

A-over-B/C ruling; A1 realization; §2/§3/§7 sub-surface single-placeholder scoping; I3 residual narrowing; ASCII glyph set (render block strictly `-> [ ] [x] <empty> <masked> [ Run ]`); §5 ordering (GUI tag first, manual pins second); §4 separate `verify-examples-gui` (no edit to the symlinked `verify-examples.sh`); §6 fixed-fixture determinism + `<masked>` secret hygiene reusing `secrets::flag_is_secret` + `invocation::render_copy_command_masked` (`invocation.rs` confirmed 0 egui imports); `pinned-upstream.toml` GUI pin v0.49.0-vs-model-v0.52.0 bump in lockstep with the 4 CLI pins.

---

## Gate

**0 Critical / 0 Important → GREEN. Converged.** The buildability question that drove r2 (slot) and r3 (secret_widget) is closed: the comprehensive sweep confirms `Slot{State,Subkey,Row}` + `SecretLineEdit` + the 4 mode-predicates are the complete extraction set, and `cargo build --bin gui-render --no-default-features` will compile. Folds are clean; the general invariant + compile gate make the next mixed module catchable structurally. The 2 nits are optional polish, not blockers. **Spec is cleared to advance to the implementation-plan stage.**
