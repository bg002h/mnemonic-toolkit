# P1.2 per-phase R0 — secret-field reveal (👁) toggle core + STOP-5 census ruling — Round 1

- **Reviewer:** opus architect (adversarial; secret-hygiene-first-class; re-ran the gates).
- **Under review:** `mnemonic-gui` branch `feat/tutorial-surfaced-fixes`, P1.1 `4d4ce75` + P1.2 `d8607a2`, off `origin/master 40156b0`.
- **Authority:** plan §P1.2 (`design/IMPLEMENTATION_PLAN_tutorial_surfaced_fixes_batch.md`) + `design/SPEC_gui_secret_reveal_toggle.md` + its R0 (`reveal-toggle-spec-r0-round-1.md`).
- **Gate:** hard R0 — 0 Critical / 0 Important. **STOP-5** (any pinned-artifact movement outside the phase censuses) had to be ruled before the gate could pass.
- **What I ran:** regenerated all 61 `.gui` via `gui-render --emit-all` (P1.2 build) and diffed vs the committed toolkit corpus; ran the whole `gui_form_snapshots` gallery census (GL software, `device_type==Cpu`); byte-compared all 61 `.new.png` vs goldens; ran the full package suite + every load-bearing binary explicitly; a RED-first neuter of `reveal_toggle`; and verified the M-1 tap-no-latch seam against **egui 0.31.1 source**.

---

## VERDICT: **GREEN** — 0 Critical / 0 Important.

Do the single **32-form gallery (PNG) re-pin** for P1.2, then advance to **P1.3**. The hygiene core is sound (display-only + never-persist + auto-hide ×4 + tap-no-latch + Id-seam all verified). The faithfulness scoping is correct and non-vacuous. STOP-5 is ruled below: **the PNG/figure mover set is 32 (Option A ratified); the `.gui` structural-render mover set STAYS 28** — the task's "28 `.gui` → 32 `.gui`" half of the directive is itself a wrong premise and is corrected here. All findings are Minor.

---

## STOP-5 RULING (the census/premise correction)

### (a) The 4 extra movers are genuinely masked-on-load composites — CONFIRMED. But there is NO `.gui` delta on them.

`mnemonic-final-word`, `mnemonic-seed-xor-split`, `mnemonic-seedqr-encode`, `mnemonic-ms-shares-split` each render a `NodeValueComposite --from` whose **default (first) node is argv-secret**, so the real render masks the value field on load and now (v0.57.0) gains the eye:

| Form | composite flag | node array | default node | `node_type_is_argv_secret` |
|---|---|---|---|---|
| final-word | `--from` | `PHRASE_ONLY` (`schema/mnemonic.rs:2054`) | `phrase` | yes |
| seed-xor-split | `--from` | `PHRASE_ONLY` | `phrase` | yes |
| seedqr-encode | `--from` | `PHRASE_ONLY` | `phrase` | yes |
| ms-shares-split | `--from` | `MS_SHARES_FROM_NODES=["phrase","entropy"]` (`:1889`) | `phrase` | yes |

**The R0-m6 premise ("composite on-load nodes are non-secret") was factually WRONG** — the implementer's finding is correct.

**But the task's parenthetical "check that the eye is the ONLY delta on those 4 `.gui`" rests on a false assumption: those 4 `.gui` do NOT change at all.** Verified empirically — the P1.2-regenerated `.gui` for all four is **byte-identical** to the committed corpus. Reason: the emit renders a composite value as `phrase=<empty>` (`render_emit.rs:657-661`), never `<masked>`, and `flag_has_reveal_eye`/the ` [reveal]` marker fire only on `flag_is_secret(flag)` — which is `false` for the composite `--from` flag (`schema/mnemonic.rs:1245/2226`, `secret: false`; not in `SECRET_FLAG_NAMES`). The composite (site #3) eye is deliberately **NOT depicted in emit** per R0 ruling 4's value-conditional carve-out. So the delta on those 4 forms is **PNG-only**.

Empirical `.gui` census (P1.2 emit vs committed): **exactly 28 movers** — the same site-#1 `<masked>` set (`grep -rl '<masked>' = 28`). The 4 composite forms are absent from it.

### (b) Option A (ratify 32) is CORRECT — do NOT suppress the eye on an empty masked field.

The composite eye already gates on the correct predicate — `is_secret_node` (`widget.rs:604,611-618`): the eye exists iff the field is *currently* secret-masked, and it clears the stale latch on a secret→non-secret node switch (cell8b). This is exactly the same "eye-on-empty-masked" behaviour the 28 site-#1 forms show on load (empty `<masked>` field + eye). **Option B (`&& !value.is_empty()`) would introduce a value-condition site #1 does not have** — an inconsistent affordance plus a jarring appear-on-type UX, for zero hygiene gain (revealing an empty field discloses nothing). Ratify 32.

### (c) 32 is the COMPLETE mover set (no 33rd); export-wallet UNMOVED — CONFIRMED.

Enumerated **all 9** `NodeValueComposite` flags and their owning forms; the default node is argv-secret in every case, so the eye-gaining set is: site-#1 `<masked>` forms (28, incl. `mnemonic-addresses`) ∪ {composite forms not already in the 28}. The composite forms already in the 28 (`convert`, `derive-child`, `slip39-split`, `seed-xor-combine`, `seedqr-decode` — each has a site-#1 secret flag) add no new mover. The composite forms **not** in the 28 are exactly the 4 above ⇒ **28 + 4 = 32. No 33rd.**

`mnemonic-export-wallet`: `.gui` byte-identical **and** gallery PNG byte-identical ⇒ **UNMOVED** (`[ slot editor: 0 rows ]` on load, no masked widget). Confirmed.

**Operational caveat — `mnemonic-addresses` is a *sub-threshold* mover.** The gallery THRESHOLD census reported only **31 failures**; `addresses` passed the dify-0.6 gate because its eye is a tiny fraction of its wide form (the long `--language` dropdown dominates width, so the eye adds no width and the aggregate pixel-diff is sub-threshold), yet its rendered bytes DO change (an eye is drawn — it is a site-#1 `<masked>` form). The **re-pin must regenerate via `UPDATE_SNAPSHOTS` (regen + git-diff), NOT threshold-failure selection**, or `addresses`'s figure silently stays at the stale no-eye v0.56.0 golden and the CI `snapshots` gate won't catch it (sub-threshold ⇒ passes). A GL-local regen additionally perturbs ~2 secret-less forms by sub-LSB noise (`mk-encode`, `xpub-search-address-of-xpub` — both have no secret field, confirmed); those must be `git checkout`-reverted so the commit is a clean **32**, per the known A5 "size ≠ identity" gotcha. The pinned CI rasterizer (lavapipe-Vulkan) that made the goldens produces a clean 32.

### (d) Count-correction directive (this is a census/premise fix, not a scope/user decision)

- **P1.2 R-A (GUI `gui_form_snapshots`): 28 → 32.** The plan's R0-m4 census RULE — `moved-PNG-set == grep -rl '<masked>'` — is **falsified** (the composite forms move the PNG without carrying `<masked>` in `.gui`). Corrected rule: **`moved-PNG-set == <masked>-census (28) ∪ composite-default-argv-secret set (4) = 32`**. Regenerate via UPDATE (see (c)).
- **P2.1 `docs/manual-gui/figures/gui/*.png`: 28 → 32** byte-copies. The plan's "same set as `.gui`" note is wrong — the figure set = the `.gui` 28 **∪** the 4 composite-only forms.
- **P2.1 `docs/manual-gui/transcripts/gui/*.gui`: STAYS 28.** Do NOT bump this to 32 — the 4 composite forms' `.gui` do not change (composite eye is not depicted, ruling 4). Bumping it to 32 would create 4 phantom `.gui` re-pins that are byte-identical no-ops and would break the P2.1 census. (The P1.1 `gui-secret-reveal-toggle` FOLLOWUP body already correctly states "28 masked-on-load `.gui` rows" — keep it.)
- Update the spec §2.6 / §7.2 and the plan's prose that conflate the two counts: **`.gui` = 28; figures/gallery = 32.**

---

## Hygiene-core verdict — **SOUND**

**1. Display-ONLY — CONFIRMED.** The only readers of reveal state (`reveal_field_key`/`revealed_field`/`clear_revealed_field`/`reveal_toggle`/`clear_reveal_on_*`) are `secret_widget.rs` (defs), `app_window.rs` (the two auto-hide seams), `slot_editor.rs` (site #2), `widget.rs` (site #3). **No masked/redacted surface reads it** — grep of `secrets.rs`, `form/invocation.rs` (`assemble_argv_with_secret_mask`/`render_copy_command_masked`), `persistence.rs` (`redact_for_persistence`) for `reveal` is empty (the one hit at `invocation.rs:518` is an unrelated doc-comment). cell6 proves it live: with the latch **armed** and the field on-screen revealed, every argv token bearing the FAKE secret keeps `mask==true`, the copy-command stays `••••` (both shell flavours), and the serialized redacted state does not contain it. `clear_revealed_field` is called on Run dispatch (`app_window.rs:1030`) **before** the confirm modal opens.

**2. Single-revealed-field / never-persist — AIRTIGHT.** Reveal is one Context-transient `Option<egui::Id>` under `reveal_field_key()` in `ctx.data_mut` (`secret_widget.rs:130-149`) — **not a `FormState` field at all**, so there is nothing to `#[serde(skip)]`; it cannot serialize and cannot survive restart. The I3 64-secret never-persist net is structurally unaffected — `ui_harness_i3_secret_nopersist` **7/7 green, unchanged**. Revealing B re-masks A: cell3 asserts `id_a != id_b` and exactly one `TextInput` throughout.

**3. Auto-hide ×4 — all wired.** (1) Run dispatch `clear_revealed_field` (`app_window.rs:1030`); (4) tab/subcommand switch `clear_reveal_on_form_change` keyed on `"<cli>:<sub>"` (`app_window.rs:552`); (2) field blur `clear_reveal_on_blur` (per-field, `secret_widget.rs:174-178`; only clears when *this* field is the revealed one); (3) window-focus-loss inside `reveal_toggle` via `ui.input(|i| i.focused)` (`:193-199`), and the final predicate is `window_focused && (latched || hold)` so even a physically-held eye re-masks on defocus. **Run + subcommand-switch are tested against the REAL `MnemonicGuiApp`** (cell4a/cell4d via `app_on_sub`); blur + focus-loss via the widget harness (cell4b/cell4c) — appropriate, since those two are widget-frame triggers, not app-window seams. The latch is cleared by Run **before** any `-run`/`-modal` capture (trigger 1).

**4. Pointer-tap-does-NOT-latch (M-1) — VERIFIED against egui 0.31.1 source; no fallback-downgrade needed.** `reveal_toggle` arms the latch iff `eye.clicked() && !eye.clicked_by(egui::PointerButton::Primary)` (`secret_widget.rs:206`). egui source: `clicked() = flags.contains(FAKE_PRIMARY_CLICKED) || clicked_by(Primary)` (`response.rs:153`); `clicked_by(Primary)` requires a real pointer primary click (`response.rs:166`); `FAKE_PRIMARY_CLICKED` is set **only** by keyboard Space/Enter-with-focus or an AccessKit `Click` action (`context.rs:1263-1277`). Therefore: pointer tap ⇒ `clicked_by(Primary)=true` ⇒ `!that=false` ⇒ **no latch** (the hold arm gives it the momentary reveal); AccessKit/keyboard ⇒ `FAKE_PRIMARY_CLICKED=true`, `clicked_by(Primary)=false` ⇒ **latches**. The FAKE_PRIMARY split is a sound, built-in discriminator (no input-event sniffing). cell5 confirms both arms.

**5. Id seam (M-6) — no cross-field bleed.** `field_id = ui.unique_id().with("…")` used as BOTH the latch key and the `TextEdit` explicit id (`secret_widget.rs:267`, `slot_editor.rs:57`, `widget.rs:612`). `Ui::unique_id()` is position-derived — stable across frames for a fixed layout, **unique per Ui instance** (each secret field is its own `ui.horizontal`); `ui.id()` (the stable id) collides between sibling horizontals, which is why the implementer chose `unique_id()` — correct. cell3 empirically asserts `id_a != id_b` for two coexisting `SecretLineEdit`s. A hierarchy shift changes `unique_id` ⇒ the latch key stops matching ⇒ the field **re-masks (fails safe)**, never leaks.

RED-first proof: neutering `reveal_toggle` to never-reveal (return-only) fails **7/12** cells — cell2/3/4b/4c/5/6/8b (every reveal-behaviour cell) — while the 5 masked-default / eye-present / clear-works cells correctly still pass (the safe default is preserved). Non-vacuous. Full suite + every gate green: `secret_reveal_toggle` 12, `gui_render_faithfulness` 2, `gui_render_emit` 15, `ui_harness_i3_secret_nopersist` 7, `schema_mirror` 21, `gui_schema_conditional_drift` 5, `schema_mirror_secret_drift` 1, `widget_secret_mask_cycle15g` 9. `schema_mirror`/`conditional_drift` unchanged ⇒ **no schema surface** (OQ-5 held). clippy/`--no-default-features` are the phase-gate's remaining CI arms (full `cargo test` exit 0).

---

## Faithfulness-scoping verdict — **CORRECT + non-vacuous**

The gate (`gui_render_faithfulness.rs:253-277`) models the eye **only** for `ControlClass::Secret` (site #1, always-eye): emit predicts `flag_has_reveal_eye(flag)`, the real isolated render must expose the `👁` button (queried by exact glyph label via `observe_reveal_eye`, **not** `Role::Button` — so a `?` help icon can't false-match). A dedicated non-vacuity negative (`reveal_eye_faithfulness_is_non_vacuous`) proves teeth (a projection omitting the eye would RED) **plus** a discrimination arm (`--json` non-secret carries no eye either side). The site-#3 composite eye is **carved out to kittest cell8b**, not this gate — correct per R0 ruling 4: `control_class` is a static per-flag projection and a value-conditional composite eye would force fixture-value coupling into the gate (this is the source of the "11 composite divergences" the implementer saw before scoping — they are the composite/value-conditional cases the gate cannot statically predict, legitimately excluded). The scoping does **not** hide a real site-#1 drift (those are checked unconditionally), and cell8b independently proves the composite eye tracks `is_secret_node` and clears the stale latch on a node switch.

Eye marker string ` [reveal]` — deterministic ASCII const (`render_emit.rs:589,690`), appended after `<masked>` on the 28 site-#1 rows (`is_secret && !is_secret_bool`) and secret positionals; `REVEAL_EYE_GLYPH="👁"` (U+1F441) is used only in the live GUI + the faithfulness label query, never in the ASCII `.gui`. Emit-pin re-pins are correct + complete: the 3 exact-ASCII pins (`inspect --ms1`, the bundle `--passphrase` slot-form, `ms inspect` positional) updated to `<masked> [reveal]`; the general secret-line loop uses substring `contains("-> <masked>")` which tolerates the marker.

---

## Findings

### Critical — none.
### Important — none.

### Minor (none blocks P1.2)

- **m1 — the census directive is asymmetric (the STOP-5 output above).** `.gui` STAYS 28; only the gallery/figures go 28→32. Fold into the plan (P1.2 R-A rule, P2.1 figure count = 32 / `.gui` count = 28) and the spec §2.6/§7.2 prose. Delivered in the ruling; this is a plan-doc edit, the P1.2 code is correct.
- **m2 — `addresses` is a sub-threshold PNG mover.** Re-pin via UPDATE regen (not threshold-failure selection), else its figure stays stale-no-eye and the sub-threshold CI gate won't catch it; revert the ~2 GL-local sub-LSB noise forms (`mk-encode`, `xpub-search-address-of-xpub`) so the commit is a clean 32. (See ruling (c).)
- **m3 — cell3 covers two `SecretLineEdit`s, not the two secret-slot-row nor the `--passphrase`+secret-positional case the plan named.** The invariant is structurally sound (single global ctx key + per-widget `unique_id`), but the two-slot-row collision sub-case is untested. Optional: add a two-secret-slot-row assertion (or the passphrase+positional combo) to cell3 for completeness. Not a hygiene gap.
- **m4 — the faithfulness gate checks masking but not the *eye* on secret positionals** (`gui_render_faithfulness.rs:283-301`); the secret-positional eye is asserted only in the emit-pin and is structurally guaranteed by `SecretLineEdit::show`. Optional: extend the positional block to `observe_reveal_eye`. Not a hygiene gap.

---

## Bottom line

The hygiene model holds under adversarial, first-class scrutiny — reveal is a pure per-frame `.password(!reveal)` flip with no path into any masked/redacted surface, bounded by four correctly-wired auto-hide triggers and the single-field Context-transient invariant; the tap-no-latch seam and the Id seam are verified against egui source and empirically. STOP-5 is ruled: **PNG/figures = 32 (Option A ratified, complete, export-wallet unmoved); `.gui` = 28 (unchanged — the task's "32 `.gui`" is a wrong premise, corrected here)**. **GREEN — commit the single 32-form gallery re-pin (UPDATE regen, clean to exactly 32), then advance to P1.3**, carrying the m1 count-correction into P2.1 (`figures` 28→32, `transcripts` stays 28). Repo left clean (neuter reverted; `.new.png`/`.diff.png` deleted; re-pin NOT committed).
