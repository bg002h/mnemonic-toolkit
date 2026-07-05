# Spec R0 — `mnemonic-gui` secret-field reveal/show (👁) toggle — Round 1

- **Reviewer:** opus architect (adversarial, secret-hygiene-first-class scrutiny)
- **Spec under review:** `design/SPEC_gui_secret_reveal_toggle.md`
- **Ground-truth SHA:** `mnemonic-gui @ cab940b` (`release: mnemonic-gui v0.56.0`), version `0.56.0` — **confirmed** (`git rev-parse` HEAD is `40156b0`, one commit ahead of `cab940b`; the only delta is `FOLLOWUPS.md +7`, so all `src/`/`tests/` citations were verified directly against `cab940b`).
- **Toolkit ground-truth:** manual-gui at `docs/manual-gui/`, GUI pin `mnemonic-gui-v0.56.0`.
- **Gate:** hard R0 — 0 Critical / 0 Important required before any implementation.

---

## VERDICT: **GREEN** — 0 Critical / 0 Important. Write the plan-doc.

Every source citation in the spec verifies against `cab940b` (line numbers accurate to ±2; all sentinel/function/site cites land exactly). The hygiene model is sound: reveal is a pure per-frame `.password(bool)` flip with no new copy, no persisted state, and no path into the run-confirm / argv-echo / paste-warn / persistence / exit-sweep surfaces. The single-revealed-field invariant lives in Context-transient data (`Option<egui::Id>`), leaving the I3 never-persist net structurally untouched. The tutorial-allowlist control is present and load-bearing (`secret_allowlist_violations()` over manifest literals) and needs **no** widening. All five findings below are **Minor** (plan-time precision / tracked FOLLOWUP); none blocks writing the plan.

---

## Rulings on the five open R0 questions

### OQ-1 — hold-primary + bounded-latch hybrid vs latch-only → **KEEP THE HYBRID (ratified)**

The hybrid is sound. The latch does **not** reintroduce an unbounded "left-revealed field" risk the auto-hide triggers fail to cover, with one narrow, tracked exception (see Finding M-1). Reasoning:

- The **hold** arm (`eye_response.is_pointer_button_down_on()`) is strictly the most hygienic model for the dominant pointer case — release re-masks next frame, no state to leak. Nearly free (one bool read).
- The **latch** arm is a *hard requirement*, not a convenience: (a) press-and-hold is not an AccessKit action, so hold-only is inaccessible to keyboard/AT users; (b) the kittest harness drives only `Focus`/`Click`/`SetValue` (project memory), so a hold-only reveal is **undrivable by the faithfulness + hygiene gates**; (c) the tutorial's revealed-demo-phrase capture needs a deterministic reveal. Collapsing to latch-only would sacrifice the strictly-safer pointer path for no gain; collapsing to hold-only fails all three hard requirements. Keep both feeding one per-frame predicate.

**The 4 auto-hide triggers are sufficient and correctly sourced:**
- (1) Run dispatch, (2) secret-field blur/`lost_focus`, (3) window-focus-loss, (4) tab/subcommand switch.
- Trigger (3) is the safety-critical one (the OS-snapshot / App-Switcher / Task-View risk). Its source `ctx.input(|i| i.focused)` is **correct** — verified against `src/platform.rs:1-9` (the exact snapshot surface the spec targets) and consistent with the codec's own occlusion mitigations at `platform.rs:64` (macOS `NSWindowSharingNone`) / `:91` (Windows `WDA_EXCLUDEFROMCAPTURE`) / `:107-113` (Linux documented-unmitigated). The choice to clear the latch on defocus so the thumbnail captures a masked field is the right call.
- One residual: there is an inherent **one-frame race** at defocus (the compositor may thumbnail the last-presented frame before egui presents the re-masked frame). This does **not** widen the accepted posture — macOS/Windows already occlude the window via OS API regardless of reveal; Linux is already documented-unmitigated (`gui-os-snapshot-secret-occlusion`). Reveal is a bounded, user-initiated act. Acceptable; note it in the hygiene prose (Finding M-4).

### OQ-2 — tree-site scope → **PRIMARY sites #1/#2/#3 now; DEFER #4/#5 with a filed FOLLOWUP (option b)**

Deferring the two build-descriptor tree-key sites is an **acceptable phased scope, not a Critical/Important gap**. All five sites verified at the cited lines (see the site table below). Justification for deferral:

- Default-masked still holds at **every** site → **no hygiene regression** from deferral, only a bounded UX inconsistency.
- Sites #4/#5 mask *conditionally* on `is_xprv_like` (dynamic, content-shaped). The tutorial's tree journey (J3 "4-tier wsh vault") uses **public xpubs** → `is_xprv_like` is false → the tree keys are **never masked**, so there is nothing to reveal there. The inconsistency is invisible in the shipped tutorial.
- **Additional technical justification the spec under-weights (reinforces deferral):** the faithfulness gate's emit-side projection `control_class(flag)` (`render_emit.rs:439-460`) is a *static per-flag* function returning `ControlClass::Secret`. Sites #4/#5 (and #3) render the eye *conditionally on the runtime value* (`is_xprv_like`/`is_secret_node`), which the static projection cannot cleanly predict from schema alone. Sites #1/#2 render the eye *unconditionally* on secret fields → trivially faithfulness-modellable. Including the value-conditional tree sites forces fixture-value coupling into the faithfulness gate. Bounding this cycle to #1/#2 (always-eye) + #3 (conditional but with existing `is_secret_node` infra, eye tested by the dedicated kittest cell §8 test #8 rather than the faithfulness gate) is the clean cut.
- **Required action:** file `gui-secret-reveal-tree-key-sites` in `mnemonic-gui/FOLLOWUPS.md` (per §9).

### OQ-3 — depict in `.gui` structural render vs chrome-only → **DEPICT (28 `.gui` re-pin)**

Depict a minimal ASCII marker on masked-on-load secret rows. The structural render is the manual's source-of-truth "what widgets are on screen"; omitting a real, always-present control on secret rows creates a gap the faithfulness gate cannot guard against and diverges the `.gui` from the PNG. Marker must be strictly ASCII (`render_emit.rs:15-18` determinism contract). **Re-pin quantum verified: exactly 28** `.gui` files carry the `<masked>` sentinel on load (`grep -rl '<masked>' docs/manual-gui/transcripts/gui/` → 28). Composite/tree/slot render `<empty>`/`[ slot editor: 0 rows ]` on load → no marker added there (0 additional `.gui`). The `.gui` regenerate mechanically via `gui-render --emit-all`. (If R0 had elected chrome-only: 0 `.gui`, faithfulness-gate-only — but depict is correct.)

### OQ-4 — no timeout in v1 → **ACCEPTABLE for v1**, with a tracked follow-up (Finding M-1)

No wall-clock timeout in v1 is the right call for determinism: the faithfulness + tutorial captures must be byte-reproducible (`render_emit.rs:15-18`), and a timer is a non-determinism source that would have to be gated out of every capture path anyway. The hold arm needs no timeout (release = hide); the latch is bounded by the four §4.5 triggers. **The one residual exposure** — a pointer *tap* (or keyboard actuation) latches, and if the user then leaves both the field and window focused and walks away, the plaintext persists until an eventual defocus — is narrow, user-initiated, and strictly inside the already-accepted OS-snapshot posture. Not Important. **Required action (M-1):** the plan must (a) rule on whether a pointer *tap* should latch at all (prefer hold-dominant for pointer; reserve latch for keyboard/AT/harness) and (b) file `gui-secret-reveal-latch-timeout` as a capture-gated fast-follow so the walk-away case is tracked, not silent.

### OQ-5 — `schema_mirror` needs no edit → **CONFIRMED (verified)**

Reveal adds **zero** clap surface — no flag, option, subcommand, or dropdown value. The eye is pure GUI view-chrome on existing widgets. `mnemonic gui-schema` (the toolkit clap structure the GUI's `schema_mirror` compares against) is unchanged, so `schema_mirror` and `gui_schema_conditional_drift` are trivially GREEN. Both gate files confirmed present at `cab940b` (`tests/schema_mirror.rs`, `tests/gui_schema_conditional_drift.rs`). The only failure mode is *modelling the eye as a schema-visible control* — which §7.1 explicitly forbids; the plan must not do so. **Minor precision (M-5):** §7.1's phrasing "bumping the toolkit's GUI pin to v0.57.0 fires `schema_mirror`" is imprecise — `schema_mirror` lives GUI-side and runs against the pinned toolkit binary; this feature changes neither the toolkit clap surface nor adds a GUI-schema control, so there is nothing for it to catch. Conclusion (no schema delta, GREEN) is correct.

---

## Hygiene-model verdict (the load-bearing core) — **SOUND**

**1. The secret buffer is held safely enough to reveal — CONFIRMED (site #1).** `src/form/secret_model.rs` (read in full): `SecretLineEdit { buf: Zeroizing<Vec<u8>> }` zeroed on drop (`:29-32`), redacting `Debug` printing only `len` (`:34-40`), **non-`Clone` by deliberate omission** with the rationale in the module doc (`:16-19`; this transitively drops `FormState`'s `Clone`, confirmed at `schema/mod.rs:316-319`), `as_string() -> Zeroizing<String>` (`:68-70`). The reveal flips only the render-time `.password(bool)` on the *already-existing* transient `String` inside `show` (`secret_widget.rs:57-58`, zeroed at `:85`) — it creates **no new copy** and touches nothing in the buffer lifecycle. *Caveat noted (Finding M-3): the §2.1 "held safely" argument is site-#1-specific; sites #3/#4/#5 back onto plain swept `String`s, not `Zeroizing` buffers — reveal still introduces no new copy there, but the framing should acknowledge it.*

**2. Reveal is display-ONLY; every masked/redacted surface stays independent — CONFIRMED.**
- Run-confirm modal: `should_confirm_run` (`secrets.rs:215-252`, read) fires on any secret; the preview is masked via `assemble_argv_with_secret_mask` / `render_copy_command_masked` — **which live in `src/form/invocation.rs:152` / `:524`, not `secrets.rs`** (spec §2.5/§6 name them without a file; plan should cite the real path — Finding M-2). None of these read reveal state.
- Paste-warn: `paste_warn_id()` chokepoint (`secret_widget.rs:40-42`), fired at `:77-81` and `widget.rs:613-624`; independent of reveal.
- Exit sweep: `zeroize_form_state` (`secrets.rs:294-335`) walks the widgets; independent of reveal.
- Persistence: `redact_for_persistence` (`persistence.rs:77`); `secret_widgets` is `#[serde(skip)]` (`schema/mod.rs:320-322`) and reconstructed empty on load (`persistence.rs:135`). Reveal writes nothing here.
- Spec §8 test #5 (reveal-armed ⇒ `redact_for_persistence`/argv-mask/`render_copy_command_masked` all unchanged) is the correct defense-in-depth proof. Keep it mandatory.

**3. Single-revealed-field invariant in Context-transient data — AIRTIGHT.** One `Option<egui::Id>` under a well-known key, sibling of `paste_warn_id()` (`secret_widget.rs:40-42`), in `ctx.data_mut`. It is **not** a `FormState` field → cannot serialize → the I3 net (`tests/ui_harness_i3_secret_nopersist.rs`, 64 classified secrets × the persist/argv-preview/stdin surfaces) is structurally unaffected. `SecretLineEdit` gains no reveal field. Confirmed against the I3 harness header (`:1-45`) — its three assert surfaces never read reveal state.

**4. Tutorial-gate reconciliation — SOUND, with a precision correction.** The load-bearing control is **already present**: `tests/tutorial/mod.rs:384 secret_allowlist_violations()` iterates `MANIFEST` × drives and RED-flags any secret-classified drive whose value is not in `SECRET_ALLOWLIST` (`= &[S0, S1, S2]`, the three published phrases, `mod.rs:51`). Because a revealed field can only ever display a value the manifest drove into it, and only allowlisted values may be secret-driven, **a revealed secret in a committed PNG can only ever be an allowlisted public phrase — by construction — and the allowlist needs no widening**. This satisfies §7.3's intent while keeping "no real secret in a committed shot" intact.

  **Precision correction (fold into the plan):** §7.3's framing — "teach the *scan* to permit only allowlisted phrases revealed" — is slightly off. There is **no PNG-pixel scanner to teach**; the bound comes from the *manifest-literal* allowlist checker. The airtight requirement the plan must carry is: **the widget-mask classification set must be a subset of the allowlist's secret-classification set for any field capturable-while-revealed**, so nothing revealable escapes the allowlist check. Concretely: the allowlist checker classifies "secret drives" via `SECRET_SLOT_SUBKEYS` / `SECRET_NODE_TYPES_ARGV` (+ the flag census); the widget masks via `flag_is_secret` (#1), `is_secret_bearing` (#2), `node_type_is_argv_secret` (#3). The plan must add an assertion that these agree for every reveal-in-scope field, and must confirm `secret_drive_count() > 0` non-vacuity survives. Deferring #4/#5 (OQ-2) removes the one place the sets could diverge (`is_xprv_like` tree keys vs the node taxonomy) — reinforcing deferral. Do **not** loosen the allowlist; add the §8 test #9 negative (non-allowlisted secret revealed ⇒ RED).

**5. Faithfulness gate becomes stronger, non-vacuous — CONFIRMED conceptually.** `render_emit.rs` models `ControlClass::Secret` = `Role::PasswordInput` (`:367`, `control_class` `:439-460`, `project_form` `:465`); the gate (`tests/gui_render_faithfulness.rs`, present) cross-checks emit vs real AccessKit across all 61 forms. Adding an adjacent `Role::Button` eye to both projection sides proves the GUI actually renders a reveal control on every secret field. The default fixture (`FormState::default()`, no secret injected) still observes the masked `PasswordInput` — matching the §4.1 default-masked invariant. §8 test #6's non-vacuity negative (a projection omitting the eye REDs) is the right anti-tautology guard. Keep it.

---

## Ripple-count confirmation (all verified against ground truth)

| Quantity | Spec | Ground truth | Status |
|---|---|---|---|
| `.password(` real masking sites | 5 | 5 (`secret_widget.rs:58`, `slot_editor.rs:52`, `widget.rs:606`, `tree_form.rs:687`, `tree_form.rs:711`; 2 extra grep hits at `widget.rs:596/601` are comments) | ✅ |
| Committed `.gui` structural renders | 61 | 61 (`git ls-files docs/manual-gui/transcripts/gui/*.gui`) | ✅ |
| Figure gallery PNGs | 61 | 61 (`git ls-files docs/manual-gui/figures/gui/*.png`) | ✅ |
| Masked-input forms (`<masked>` on load, site #1) → `.gui`/PNG re-pin | 28 | 28 (`grep -rl '<masked>'`) | ✅ |
| Union of secret-bearing forms | 29 | 28 ∪ export-wallet; export-wallet `.gui` shows `[ slot editor: 0 rows ]` (no masked widget on load) | ✅ |
| Slot-editor forms with rows on load | 0 | all 4 (bundle/export-wallet/import-wallet/verify-bundle) show `0 rows` → no eye on load → ~28 PNG holds | ✅ |
| Tutorial capture shots | 50 | **50** (`tests/snapshots/tutorial/*.png`, both `cab940b` and working tree) | ✅ (see note) |
| `SECRET_ALLOWLIST` size | 3 (S0/S1/S2) | 3 (`tests/tutorial/mod.rs:51`) | ✅ |

**Tutorial-count note:** the spec's **50 is correct against ground truth**. Project memory's "keep-all-51-shots / 51-shot corpus" and the intermediate commit `ac349d1` ("51 whole-window shots") are **stale** — the v0.56.0 release commit `cab940b` itself says "50-shot corpus" and the committed file count is 50. The plan should size the re-drive at **50**, not 51.

The site table maps exactly to the spec's §2.2 (sites #1–#5 with the correct gate predicates: `.password(true)` #1/#2, `.password(is_secret_node)` #3 where `is_secret_node = node_type_is_argv_secret(node.as_str())`, `.password(key_is_xprv)` #4, `.password(k_is_xprv)` #5). The `slot_editor.rs:51-63` secret-arm / (Path,hint)-arm mutual exclusion is exactly as §2.3 describes.

---

## Findings

### Critical — none.
### Important — none.

### Minor (plan-time precision / tracked follow-ups; none blocks the plan)

- **M-1 (OQ-4 residual) — pointer-tap-latch + no-timeout walk-away.** A pointer tap or keyboard actuation arms the latch; if the user then leaves both field and window focused and walks away, plaintext persists until an eventual defocus. Bounded, user-initiated, inside the accepted OS-snapshot posture. **Action:** the plan rules on whether a pointer *tap* should latch (prefer hold-dominant for pointer; reserve latch for keyboard/AT/harness) **and** files `gui-secret-reveal-latch-timeout` as a capture-gated fast-follow FOLLOWUP.

- **M-2 — argv-mask function file path.** `assemble_argv_with_secret_mask` / `render_copy_command_masked` live in `src/form/invocation.rs:152` / `:524`, not `src/secrets.rs`. §2.5/§6 name them without a file; the plan must cite the real path so the §8 test #5 wiring is correct.

- **M-3 — "held safely" framing is site-#1-specific.** §2.1's `Zeroizing`/redacting-Debug/non-Clone safety argument covers only site #1 (`SecretLineEdit`). Sites #3/#4/#5 back onto plain (swept, xprv-redacted-on-persist) `String`s. Reveal introduces **no new copy** at any site (it flips a flag on an existing `&mut String`), so there is no regression — but the plan should state this explicitly rather than let §2.1 imply all sites are `Zeroizing`-backed.

- **M-4 — one-frame defocus race in the hygiene prose.** The §4.5(3) defocus→clear has an inherent one-frame window before egui presents the re-masked frame; it does not widen the accepted posture (macOS/Windows OS occlusion + Linux documented-unmitigated). Document it in the §7.3 hygiene note; do not add a modal.

- **M-5 — `schema_mirror` firing-mechanism wording.** §7.1's "bumping the toolkit's GUI pin … fires `schema_mirror`" is imprecise (`schema_mirror` is GUI-side, over the pinned toolkit binary; this feature touches neither side). Conclusion (no schema delta) is correct; tighten the prose so no one is tempted to model the eye as a schema control.

- **M-6 (implementation-shape note for the plan, not a spec defect) — eye adjacency + stable per-field Id.** `SecretLineEdit::show(&mut self, ui, label, help)` (`secret_widget.rs:52`) currently emits `ui.label(label)` then `ui.add(TextEdit)` and is invoked at `app_window.rs:826`, `widget.rs:116`, `widget.rs:159`. To render the eye *adjacent* (§5.1) the content must share a horizontal layout, and the single-revealed-field invariant needs a **stable per-field `egui::Id`** across frames (use the `TextEdit` `Response.id`; ensure uniqueness where two secret fields coexist — e.g. `--passphrase` + a secret positional, or repeating rows via `widget.rs:159`). The plan must nail Id derivation + the `ui.horizontal` wrapping.

---

## Bottom line

The spec is well-grounded, the hygiene model is defensible under first-class scrutiny, and every source citation checks out against `cab940b`. All five open questions are ruled (hybrid / PRIMARY-3-defer-2 / depict / no-timeout-with-tracked-follow-up / no-schema-delta). No Critical or Important finding exists. **GREEN — proceed to write the implementation plan-doc**, folding the six Minor precision items above (they are plan-content, not spec re-review triggers). Per the reviewer-loop discipline, re-dispatch this reviewer after the plan-doc is drafted (and after any spec fold that materially changes §4/§7).
