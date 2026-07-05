# SPEC — `mnemonic-gui` reveal/show (👁) toggle on secret fields

- **Status:** DRAFT — awaiting mandatory opus R0 (0C/0I) before any implementation. NO code before GREEN.
- **User approval:** feature approved 2026-07-05.
- **Sensitivity:** SECRET-HYGIENE-FIRST-CLASS. This is a *deliberate secret-exposure affordance*; the hygiene model is the load-bearing part of this spec (per standing bar `feedback_secret_hygiene_first_class_bar`).
- **Repos:** primary = `mnemonic-gui` (widget + gates + GUI tag). Ripple = `mnemonic-toolkit` `docs/manual-gui/` (structural renders + figure gallery + prose + tutorial re-drive).
- **Source SHAs pinned for this spec's citations** (re-grep at plan-write time — citations decay every merge, per CLAUDE.md):
  - `mnemonic-gui` @ `cab940b` (`release: mnemonic-gui v0.56.0`), version `0.56.0`.
  - `mnemonic-toolkit` @ `97a494a9`; `docs/manual-gui` pins `mnemonic-gui-v0.56.0`; manual-gui at `v1.2.0`.
- **Batch target:** ship BATCHED — `mnemonic-gui-v0.57.0` + ONE tutorial re-drive + ONE `manual-gui-v1.3.0` (see §9).

---

## 1. Motivation

`mnemonic-gui` secret fields are hardcoded to `TextEdit::…password(true)` — always masked, no way to reveal what was typed. Two concrete problems:

1. **Usability / safety gap.** A user typing a 12–24-word BIP-39 seed (or an xprv, WIF, ms1 share, BIP-38 ciphertext, Electrum seed) into a fully-masked field cannot verify what they typed. A single seed typo = lost funds. The inability to *proofread* a hand-typed seed is a real self-custody hazard, not a cosmetic nit.
2. **Tutorial can't teach.** The `gui_example.pdf` tutorial (in flight) uses ONLY the three public, world-known BIP-39 test phrases as demo data. The `••••` masking hides the exact lesson — the reader cannot see what to type. A reveal affordance lets the tutorial re-drive show the (public) demo phrase in the filled-form screenshot.

A reveal toggle fixes both, but ONLY if it does so without weakening any of the existing hygiene surfaces (never-persist, run-confirm masking, argv-echo masking, paste-warn, OS-snapshot occlusion).

---

## 2. Recon findings (grounded in current source)

### 2.1 The secret buffer is held safely enough to reveal (no hygiene regression from reveal itself)

`src/form/secret_model.rs` — the egui-free `SecretLineEdit { buf: Zeroizing<Vec<u8>> }`:
- `secret_model.rs:29-32` — buffer is `Zeroizing<Vec<u8>>`, zeroed on drop.
- `secret_model.rs:34-40` — redacting `Debug` (prints only `len`, never bytes).
- `secret_model.rs:16-19` — deliberately NOT `Clone` (a clone is a second copy of the secret).
- `secret_model.rs:68-70` — `as_string()` returns `Zeroizing<String>` (type-level wrap obligation).

`src/form/secret_widget.rs` — the egui-coupled `show`:
- `secret_widget.rs:52-58` — per frame: copies `buf` into a transient `String`, renders `ui.add(egui::TextEdit::singleline(&mut transient).password(true))`, writes back on `response.changed()`, then `transient.zeroize()` at `:85`.
- `secret_widget.rs:19-27` (module doc) — the egui **undo-ring residue** is a documented second-tier gap (`FOLLOWUPS gui-secret-buffer-allocator-residue`), deferred beyond v0.2. Reveal does NOT change this posture (see §4.7).

**Finding:** the value is held safely enough to reveal. Reveal flips ONLY the render-time `.password(bool)` flag — it does not create a new copy, does not persist, does not touch the buffer's lifecycle. No hygiene regression is introduced *by revealing*. The pre-existing undo-ring caveat is orthogonal and unchanged.

### 2.2 There are **FIVE** masking sites, not two (critical scoping correction)

The task cited `secret_widget.rs:58` and `slot_editor.rs:52`. `grep -rn '\.password(' src/` at `cab940b` finds **five**:

| # | Site | What it masks | Mask condition | Reveal scope |
|---|------|---------------|----------------|--------------|
| 1 | `src/form/secret_widget.rs:58` | secret Text flags, secret positionals, secret repeating rows (`--passphrase`, `--bip38-passphrase`, `--ms1`, `--share`, secret positionals) | **always** (`.password(true)`) | **PRIMARY** |
| 2 | `src/form/slot_editor.rs:52` | per-row secret slot values (`@N.phrase`/`seedqr`/`entropy`/`ms1`/`xprv`/`wif`) | on `row.subkey.is_secret_bearing()` | **PRIMARY** |
| 3 | `src/form/widget.rs:606` | the `NodeValueComposite` value cell (`--from <node>=<value>`, `--share`, etc.) | on `node_type_is_argv_secret(node)` (node-dependent) | **PRIMARY** |
| 4 | `src/form/tree_form.rs:687` | build-descriptor tree node single `key` | on `is_xprv_like(node.key)` (value-shaped, dynamic) | **SECONDARY** (see §5.2) |
| 5 | `src/form/tree_form.rs:711` | build-descriptor tree `keys[i]` (KeyQuorum) | on `is_xprv_like(node.keys[i])` (dynamic) | **SECONDARY** |

Sites 3–5 are **conditionally** masked (node/xprv-shaped), so they do not render masked with public/empty data on load; they matter when a user actually types secret material. A reveal that covers sites 1–2 but leaves 3–5 always-toggle-less is a UX inconsistency (some masked fields reveal, some don't) but NOT a hygiene regression (default-masked still holds everywhere). §5.2 rules on scope.

### 2.3 Slot rows (`slot_editor.rs`)

`slot_editor.rs:28-64` — `render(ui, state, path_hint)` walks `state.rows`; the `@` index DragValue, the subkey ComboBox, the `=` label, then the value `TextEdit`. The **mutual exclusion** at `:51-63` is load-bearing: `if row.subkey.is_secret_bearing() { …password(true) } else { …(Path,hint) | plain }`. `.password` and `.hint_text` never combine (a `Path` subkey is never secret). The reveal must slot into the *secret arm only* and must not perturb the `(Path, hint)` arm.

### 2.4 OS-snapshot exposure (`src/platform.rs`)

`platform.rs:3-9` (module doc) + the impls: macOS App Switcher / Windows Task View can snapshot the visible window for live thumbnails.
- macOS: mitigated via `NSWindowSharingType::NSWindowSharingNone` (`platform.rs:64`).
- Windows: mitigated via `WDA_EXCLUDEFROMCAPTURE` (`platform.rs:91`).
- Linux/BSD: **no compositor analogue** — documented-unmitigated at `FOLLOWUPS gui-os-snapshot-secret-occlusion` (`platform.rs:107-113`).

**A revealed secret is plaintext on-screen and is therefore capturable by any window-snapshot API for the duration of the reveal.** The reveal model MUST bound that duration and MUST NOT widen the existing exposure posture (§4.7).

### 2.5 The surfaces reveal must NOT weaken

- **Never-persist (I3).** `tests/ui_harness_i3_secret_nopersist.rs:1-45` — for every classified secret × subcommand, a FAKE fixture must be ABSENT from (1) persisted state (`redact_for_persistence` → serialize), (2) the masked-argv confirm-modal / preview (`assemble_argv_with_secret_mask` → `render_copy_command_masked`; every token bearing it carries `mask == true`), (3) the `--spec -` stdin tree path. `secret_widgets` is `#[serde(skip)]` (`src/schema/mod.rs:319-322`) — type-level never-persist; `persistence.rs:135` reconstructs it empty on load.
- **Run-confirm modal** (`src/secrets.rs:200-201`, `should_confirm_run` `:215-252`) — shows the argv preview MASKED (`••••`), always.
- **Argv-echo masking** — `assemble_argv_with_secret_mask` / `render_copy_command_masked` (consumed by I3).
- **Paste-warn** (`src/secrets.rs:170-211`, threshold `PASTE_WARN_THRESHOLD = 8`; the `paste_warn_id()` chokepoint at `secret_widget.rs:40-42`, fired in `secret_widget.rs:77-81` and `widget.rs:610-621`).
- **Exit sweep** — `secrets::zeroize_form_state` (`src/secrets.rs:294-335`) walks `secret_widgets.values_mut().flatten()` and zeroes every row.

Reveal is **display-only on the input widget**; every surface above stays masked/redacted *unconditionally, independent of reveal state* (§4.2).

### 2.6 The structural-render / faithfulness / figure ripple surface (recon #5)

- **Structural renders (`gui-render` emit → `*.gui`).** `src/form/render_emit.rs` emits the deterministic ASCII form render; a secret value renders as the fixed `MASKED = "<masked>"` sentinel (`render_emit.rs:38-39`, `flag_value_str` `:598-600`, `positional_body` `:667`). The committed renders live in the TOOLKIT at `docs/manual-gui/transcripts/gui/*.gui` (**61 files**, one per form).
- **Faithfulness gate.** `tests/gui_render_faithfulness.rs` — for all 61 forms, the emit-side `project_form` (`render_emit.rs:465-518`) prediction of per-flag `ControlClass` (incl. `ControlClass::Secret` = `Role::PasswordInput`, `render_emit.rs:366-384` + `control_class` `:439-460`) is cross-checked against the REAL AccessKit tree read off the production egui render. A new eye control adjacent to the `PasswordInput` node **changes the AccessKit tree** and MUST be modelled on both sides or the gate REDs.
- **Figure gallery.** `docs/manual-gui/figures/gui/*.png` — **61** whole-window PNG screenshots (basenames 1:1 with the `.gui` files, verified). A new 👁 widget changes the pixels of every form whose secret input is visible on load.

**Enumeration of the ripple surface (from the committed `.gui` renders at `docs/manual-gui/transcripts/gui/`):**

- **Forms with a value-bearing masked input** (`<masked>` sentinel = SecretLineEdit path, site #1): **28 forms** —
  `mnemonic-addresses`, `mnemonic-bundle`, `mnemonic-convert`, `mnemonic-derive-child`, `mnemonic-electrum-decrypt`, `mnemonic-import-wallet`, `mnemonic-inspect`, `mnemonic-ms-shares-combine`, `mnemonic-nostr`, `mnemonic-repair`, `mnemonic-restore`, `mnemonic-seedqr-decode`, `mnemonic-seed-xor-combine`, `mnemonic-silent-payment`, `mnemonic-slip39-combine`, `mnemonic-slip39-split`, `mnemonic-verify-bundle`, `mnemonic-xpub-search-account-of-descriptor`, `mnemonic-xpub-search-passphrase-of-xpub`, `mnemonic-xpub-search-path-of-xpub`, `ms-combine`, `ms-decode`, `ms-derive`, `ms-encode`, `ms-inspect`, `ms-repair`, `ms-split`, `ms-verify`.
- **Forms with a slot-editor sub-surface** (site #2 possible): **4** — `mnemonic-bundle`, `mnemonic-export-wallet`, `mnemonic-import-wallet`, `mnemonic-verify-bundle`. Only `mnemonic-export-wallet` is slot-only-secret (its `.gui` shows `[ slot editor: 0 rows ]` on load — no masked widget visible on load).
- **Union of secret-bearing forms = 29** (28 masked-input ∪ export-wallet).
- Composite (site #3) and tree (sites #4–5) are node/xprv-conditional and do NOT render `<masked>` on load with public/empty data, so they add nothing to the on-load masked count.

**Finding (blocker check):** the secret buffer is held safely enough to reveal without introducing a new hygiene regression. No blocker to proceeding. The design work is entirely in *bounding the exposure the reveal deliberately creates* and in *scoping the large but mechanical manual re-pin*.

---

## 3. Design goals & non-goals

**Goals**
- G1. Let a user proofread a hand-typed secret (default-masked; reveal is always a deliberate act).
- G2. Let the tutorial re-drive reveal a PUBLIC demo phrase in the filled-form screenshot.
- G3. Zero weakening of never-persist / run-confirm / argv-echo / paste-warn / exit-sweep / OS-occlusion.

**Non-goals**
- N1. Reveal in the run-confirm modal, the argv/copy-command preview, the `--json` output pane, or any persisted surface. Those stay masked/redacted **always**.
- N2. Closing the egui undo-ring residue (`gui-secret-buffer-allocator-residue`) — orthogonal, unchanged.
- N3. New Linux compositor occlusion (`gui-os-snapshot-secret-occlusion`) — orthogonal, unchanged; reveal does not widen it beyond §4.7.

---

## 4. Hygiene model (the core)

### 4.1 Default = masked, always

Every secret field renders masked on load and after every auto-hide event. Reveal is NEVER the default and NEVER survives any state transition. No persisted, serialized, or restart-surviving state can hold "revealed" (see §4.6).

### 4.2 Ephemeral scope — reveal affects the INPUT widget display ONLY

Reveal flips exactly one thing: the per-frame `.password(bool)` argument on the *input* `TextEdit` of the currently-actuated secret field. It changes:
- nothing persisted (secret_widgets is `#[serde(skip)]`; nothing about reveal is stored in `FormState` at all — §4.6);
- nothing in the run-confirm modal (masked always);
- nothing in the argv echo / copy-command preview (masked always);
- nothing about paste-warn (still fires on over-threshold paste regardless of reveal);
- nothing about the exit sweep or the buffer lifecycle.

Formally: reveal is a pure function of a transient "is this field being actuated right now" signal into the single `.password` render flag of that one field. It has NO other observable effect.

### 4.3 Interaction ruling — **hold-to-reveal (press-and-hold) primary, bounded-latch fallback for keyboard/AT + harness**

**Ruling:** the interaction is **press-and-hold** for pointer users — the strictly-more-hygienic default — with a **bounded latched** arm for keyboard/AccessKit actuation and for the test/tutorial capture harness. Both arms feed ONE per-frame reveal predicate.

- **Pointer (primary):** reveal is active for the frames the eye control is physically held: `reveal_this_frame = eye_response.is_pointer_button_down_on()`. On pointer release the field re-masks on the very next frame. There is no latched state to leave revealed — the strictly most hygienic model, satisfying the standing first-class bar.
- **Keyboard / AccessKit (fallback):** press-and-hold is NOT an AccessKit action, and the kittest harness can drive only `Focus`/`Click`/`SetValue` (per project memory: "kittest Node has only Focus/Click"). A hold-only reveal would be **inaccessible to keyboard/AT users AND undrivable by the faithfulness/hygiene gates and the tutorial capture**. So AccessKit `Click` on the eye control arms a **bounded latched reveal** for that ONE field, which auto-hides on the first of the §4.5 triggers. This is the path the tests and the tutorial exercise (§7, §8).

**Why not latch-only?** A latched toggle risks a left-revealed field between actions. The pointer arm avoids that entirely for the common case; the latch arm is bounded by the aggressive §4.5 auto-hide triggers so its exposure window is confined to "the user is actively attending to this field."

**Why not hold-only?** Hold-only fails three hard requirements: (a) keyboard/AT accessibility, (b) drivability by the kittest faithfulness + hygiene gates, (c) deterministic capture for the tutorial's revealed-demo-phrase screenshot. The fallback latch, bounded by §4.5, closes all three at bounded exposure cost.

**Single-revealed-field invariant (mandatory).** At most ONE secret field is revealed at any instant across the whole app. Actuating field B immediately re-masks field A. Implemented by a single Context-transient `Option<egui::Id>` ("which field, if any, is revealed") — see §4.6. This caps blast radius and makes the auto-hide reasoning global rather than per-widget.

> **Open R0 question OQ-1:** keep the hybrid (hold + bounded-latch), or collapse to latch-only for a single code path? Recommendation: **keep the hybrid** — the hold arm is nearly free (one `is_pointer_button_down_on()` read) and is the most hygienic for the dominant pointer case; the latch arm is required regardless for a11y + gates + capture. R0 to ratify.

### 4.4 No timeout in v1 (justified)

A wall-clock timeout auto-hide is intentionally **omitted** in v1: it introduces a non-determinism source into the faithfulness/tutorial capture (which must be byte-reproducible — `render_emit.rs:14-18`, tutorial determinism §6 of the tutorial spec) and buys little over the §4.5 event triggers. The hold arm needs no timeout (release IS the hide); the latch arm is bounded by blur/window-focus-loss/Run/tab-switch. If a timeout is later wanted it must be gated OUT of any capture path. (Tracked as a possible fast-follow, not v1.)

### 4.5 Auto-hide triggers (latch arm) — enumerated + justified

The latched reveal for the armed field is force-cleared (→ masked) on the FIRST of:

1. **Run click** — a secret is about to be passed to the child process; the confirm modal opens masked. Leaving the input revealed behind/around the modal is gratuitous exposure. *Clear on Run dispatch.*
2. **Secret-field blur / focus-out** — the user moved on; a revealed-but-unfocused field is pure standing exposure. *Clear when the revealed field loses keyboard focus* (`response.lost_focus()` / focus moved elsewhere).
3. **Window loses focus / app-switch** — this is the OS-snapshot risk (§2.4): App Switcher / Task View thumbnail the window at exactly the app-switch moment. egui exposes window focus via `ctx.input(|i| i.focused)` (`egui 0.31 RawInput.focused`, confirmed `data/input.rs:78`). *Clear the instant the window loses focus* — so the thumbnail captures a masked field. This is the most safety-critical trigger.
4. **Tab / subcommand switch** — navigating to another CLI tab or subcommand form abandons the field context. *Clear on any `CliTab` change or active-subcommand change.*

The pointer (hold) arm satisfies all four **by construction**: releasing the pointer (which happens on blur, app-switch, Run, or navigation) drops `is_pointer_button_down_on()` to false → masked next frame. The explicit triggers exist for the latch arm.

### 4.6 No new persisted / serialized / model state

Reveal state lives ONLY in egui **Context transient data** (`ctx.data_mut`), keyed analogously to `paste_warn_id()` (`secret_widget.rs:40-42`) — a single `Option<egui::Id>` under a well-known key (the single-revealed-field invariant, §4.3). This guarantees:
- it is NOT a field of `FormState` → cannot be serialized → the I3 never-persist net is structurally unaffected (§6);
- it does not survive app restart (Context data is per-process);
- it is not captured by `redact_for_persistence` (there is nothing to redact).

`SecretLineEdit` (the egui-free model in `secret_model.rs`) gains **no** reveal field — reveal is a UI-frame concern, kept out of the model that `FormState` owns.

### 4.7 Snapshot-risk handling (§2.4 reckoning)

- While revealed, the plaintext is on-screen and capturable by a window-snapshot for the reveal duration. Reveal is bounded (hold = physical hold; latch = §4.5 triggers, *including app-switch → clear*), and is always a deliberate user act.
- The existing occlusion posture applies **unchanged**: macOS `NSWindowSharingNone` + Windows `WDA_EXCLUDEFROMCAPTURE` already suppress the OS screenshot APIs for this window; Linux remains documented-unmitigated (`gui-os-snapshot-secret-occlusion`). Reveal does NOT widen this surface — it is display of content the user is already actively editing.
- The §4.5(3) window-focus-loss trigger is specifically chosen so the app-switch thumbnail (the exact snapshot risk called out in `platform.rs`) captures a **re-masked** field on the latch arm; the hold arm re-masks on the same event because the pointer is no longer held.
- The paste-warn modal copy (`secrets.rs:182-196`) already educates the user on the snapshot/argv/undo-ring caveats; a short reveal note is added to the hygiene prose (§7.3), not a new modal (avoid modal fatigue).

---

## 5. Widget design

### 5.1 The eye control

- A compact, focusable icon button (👁 / 👁‍🗨) rendered **immediately adjacent** to the secret input, inside the same `ui.horizontal` row (so it shares the row layout with the label + field). ASCII-free glyph in the live GUI; the ASCII structural render uses an ASCII marker (§7.1).
- Behavior: `is_pointer_button_down_on()` → hold-reveal (arm 1); AccessKit `Click` → arm/re-arm the bounded latch and set the single `Option<Id>` to this field's Id (arm 2); clicking the eye of the currently-latched field toggles it back to masked.
- Hover text: e.g. "Hold to reveal (masked by default)". The label/field remain exactly as today when not actuated.
- The reveal predicate for a field = `data.reveal_id == Some(this_id)` OR `eye_response.is_pointer_button_down_on()`; the `.password(!reveal)` flag is computed from it. All existing `response.changed()` / paste-warn / write-back logic is untouched.

### 5.2 Scope over the five masking sites (§2.2)

- **PRIMARY (this cycle): sites #1, #2, #3.**
  - #1 `SecretLineEdit::show` (`secret_widget.rs`) — the reveal predicate replaces the hardcoded `.password(true)` at `:58`. Covers secret Text flags, secret positionals, secret repeating rows uniformly (all route through `show`).
  - #2 `slot_editor.rs` — an eye control on the *secret arm only* (`:52`), per-row (each slot row has its own Id → its own eye). The `(Path, hint)` arm is untouched.
  - #3 `widget.rs:606` composite value — the reveal predicate ANDs with the existing `is_secret_node` gate (reveal only ever un-masks a field that is *currently* secret-masked; a non-secret node has no eye).
- **SECONDARY (decision OQ-2): sites #4, #5 (build-descriptor tree key/keys).** These are `is_xprv_like`-conditional. Two options:
  - (a) **Include** for hygiene-affordance consistency (a reveal that covers some masked fields but not the tree keys is a confusing surface). Higher effort — `tree_form.rs` is node-recursive; each `key`/`keys[i]` needs its own eye + Id, and the tutorial's J3 "4-tier wsh vault" journey touches the tree.
  - (b) **Defer** to a fast-follow FOLLOWUP. Defensible because: the tree key mask is *conditional* on xprv-shape, the tutorial's tree journey uses PUBLIC xpubs (never masked, so no reveal needed there), and default-masked still holds (no hygiene regression from deferral) — only a UX inconsistency.
  - **Recommendation: (a) include all five for a consistent affordance**, since batching the re-pin once (§9) makes the marginal cost of the two extra sites low; but if R0 elects to bound scope, (b) is acceptable with a filed FOLLOWUP `gui-secret-reveal-tree-key-sites`. R0 to rule.

### 5.3 AccessKit representation (faithfulness gate)

- The eye control is a `Role::Button` node adjacent to the `Role::PasswordInput` (or, when revealed on the latch arm, a `Role::TextInput`) node in the AccessKit tree.
- The faithfulness gate (`gui_render_faithfulness.rs`) MUST be extended so both sides model the eye:
  - emit side (`render_emit.rs`): the `ControlClass::Secret` projection gains a "has adjacent reveal button" fact (or a new companion assertion in `project_form`), so the emit *predicts* the eye node.
  - real side: the harness reads the adjacent `Role::Button` off the isolated secret render and asserts its presence.
- This makes the gate **stronger** (it now proves the GUI actually renders a reveal control on every secret field), not merely non-breaking. The default (unactuated) render is still `PasswordInput` — the faithfulness fixture never actuates reveal (`FormState::default()`, no secret injected), so the masked default is what the gate observes, matching the §4.1 invariant.

---

## 6. Interaction with the never-persist I3 regression

Reveal is **display-only** (§4.2) and stores nothing in `FormState` (§4.6):
- `secret_widgets` remains `#[serde(skip)]`; the reveal `Option<Id>` lives in Context data, never serialized. The 64-secret never-persist net (`ui_harness_i3_secret_nopersist.rs`) is therefore **structurally unaffected** — its three assert surfaces (persisted state / masked-argv preview / stdin) never read reveal state.
- **New test required** (defense-in-depth, §8): an explicit assertion that toggling reveal ON does NOT change `redact_for_persistence` output, does NOT change `assemble_argv_with_secret_mask` masking, and does NOT change `render_copy_command_masked` (`••••` preserved). i.e. reveal is proven orthogonal to every I3 surface, not merely assumed. This is the anti-regression proof that the reveal cannot leak into a persisted/preview/argv surface.

---

## 7. Ripple scoping

### 7.1 GUI (`mnemonic-gui`) — the implementing repo

- `src/form/secret_widget.rs` (#1), `src/form/slot_editor.rs` (#2), `src/form/widget.rs` (#3) [+ `src/form/tree_form.rs` #4/#5 if OQ-2 = include].
- The reveal predicate + single-`Option<Id>` Context-transient key (a sibling of `paste_warn_id()`), incl. the §4.5 auto-hide clears wired at: Run dispatch (`app_window.rs` run path), field blur, window-focus-loss (`ctx.input(|i| i.focused)`), tab/subcommand switch (`app_window.rs` tab handling).
- `src/form/render_emit.rs` — depict the reveal affordance in the structural render (§7.2 decision) + extend `project_form` / `ControlClass::Secret` for the faithfulness gate (§5.3).
- `tests/gui_render_faithfulness.rs` — extend both projection sides for the eye node.
- New hygiene tests (§8).
- **GUI tag `mnemonic-gui-v0.57.0`.** Bumping the toolkit's GUI pin to v0.57.0 fires `schema_mirror` — but reveal adds **no flag/option/subcommand/dropdown-value** (it is pure UI chrome on existing widgets), so `schema_mirror` should NOT need a schema edit. **Confirm at plan time** that no schema surface changed (if the eye is ever modelled as a schema-visible control, that would be a mistake — it must not be).

### 7.2 Toolkit manual re-pin (quantified — the BIG ripple)

- **Structural renders (`docs/manual-gui/transcripts/gui/*.gui`).** DECISION **OQ-3:** depict the reveal marker in the ASCII render, or treat it as chrome (like the help hover, which is NOT rendered)?
  - **Recommendation: depict** a minimal ASCII marker on secret rows (e.g. append ` [reveal]` or add `reveal` to the state suffix; strictly ASCII per `render_emit.rs:14-18`). Rationale: the structural render is the manual's source-of-truth "what widgets are on screen"; omitting a real control creates a gap the faithfulness gate cannot guard, and the marginal cost is negligible (the `.gui` files regenerate mechanically via `gui-render --emit-all`).
  - **Quantified re-pin: 28 `.gui` files** (the site-#1 masked-input forms, §2.6). Composite/tree/slot render as sub-surface placeholders or `node=<empty>` on load, so they do NOT gain a `<masked>` marker on load (0 additional `.gui`). If OQ-3 = "chrome / don't depict", then **0 `.gui` change** but the faithfulness gate still changes (§5.3).
- **Figure gallery (`docs/manual-gui/figures/gui/*.png`).** Every form whose secret input is visible on load gains a 👁 in its whole-window screenshot → **~28 PNG re-pins** (the site-#1 forms; `export-wallet` shows `[ slot editor: 0 rows ]` on load → likely no visible secret widget → no change). These regenerate via the visual-screenshot harness and are byte-verified + secret-scanned + census-gated (per the `verify-figures-gui` / gallery machinery). This is the expensive part.
- **Reference-manual prose.** Any prose stating "secret fields are masked" gains a "hold-to-reveal" note. Known sites: the tour (`docs/manual-gui/src/30-tour/31-first-launch.md`) and the GUI-forms part (`docs/manual-gui/src/75-gui-forms/751-mnemonic.md` … `754-mk.md`), plus a short hygiene note near the OS-snapshot/paste-warn discussion. Grep `masked|password|••••` under `docs/manual-gui/src/` at plan time for the exact set.
- **Lockstep:** re-pin the GUI tag in `docs/manual-gui/pinned-upstream.toml` (currently `mnemonic-gui-v0.56.0` → `v0.57.0`) and re-verify anchor/census gates.

### 7.3 Tutorial re-drive (combined with the restore fix)

- The tutorial capture corpus is **50 PNG shots** at `mnemonic-gui/tests/snapshots/tutorial/` driven by `tests/gui_tutorial_snapshots.rs` (env-gated `GUI_TUTORIAL_SNAPSHOTS=1`). The manual-side tutorial source is `docs/manual-gui/tutorial/*.md` (J1–J5).
- Today, "secret fields auto-mask in filled shots (hygienic by construction)". The reveal re-drive **deliberately reveals the PUBLIC demo phrase** in the `*-form` shots whose captured step types a secret (e.g. `tut-j1-01-bundle-single-sig-form`, the restore `*-form` shots, etc.), so the reader sees what to type (G2).
- **Gate reconciliation (mandatory):** the tutorial harness already runs a **secret-allowlist checker + fixture watch-only scan** (`gui_tutorial_snapshots.rs:12-18`). Revealing a demo phrase means a plaintext phrase now legitimately appears in a committed PNG — the scan must be taught that **only the allowlisted PUBLIC world-known demo phrases may appear revealed**, and that any *other* secret appearing revealed = RED. This keeps "no real secret ever in a committed shot" intact while permitting the intentional public-phrase reveal. The plan MUST update the allowlist gate accordingly (not loosen it wholesale).
- The re-drive re-captures the full corpus deterministically; the diff gate arbitrates which shots visually changed. **ONE combined re-drive** with the restore `(none)` fix and any audit-confirmed fixes (§9), so the expensive capture + PDF rebuild happen once.

---

## 8. Test plan

All tests written BEFORE impl (per-phase TDD). New/updated:

1. **Masking-default.** A freshly-rendered secret field (site #1/#2/#3, and #4/#5 if included), unactuated, renders `.password(true)` / `Role::PasswordInput`. Proves default-masked (§4.1). (Faithfulness gate already covers the AccessKit side across all 61 forms; add a focused unit assertion.)
2. **Reveal-flips (latch arm).** AccessKit `Click` on the eye → the field's next-frame render is `.password(false)` / `Role::TextInput` and the AccessKit tree exposes the (public, fake) buffer text. Driven via kittest Click (the drivable path). Assert the buffer is UNCHANGED (reveal is display-only, not a mutation).
3. **Single-revealed-field invariant.** Actuate field A, then field B → A is re-masked, B revealed; the Context `Option<Id>` holds exactly one. (Where two secret fields coexist, e.g. `--passphrase` + a secret positional, or two slot rows.)
4. **Auto-hide triggers** (§4.5), each as a discrete cell driving the latch then the trigger, asserting re-mask:
   - (a) Run click → masked; (b) blur/focus-out → masked; (c) window-focus-loss (`RawInput.focused = false`) → masked; (d) tab/subcommand switch → masked.
5. **Never-persist unaffected (§6).** With reveal ARMED: `redact_for_persistence` output unchanged; `assemble_argv_with_secret_mask` tokens still `mask == true`; `render_copy_command_masked` still `••••`. Proves reveal cannot leak into persisted/preview/argv surfaces. Uses FAKE fixtures + coordinate-only failure messages (per I3 harness hygiene, `ui_harness_i3_secret_nopersist.rs:30-45`).
6. **Faithfulness-gate update (§5.3).** Both projection sides model the eye `Role::Button` adjacent to `PasswordInput`; the gate stays GREEN across all 61 forms AND newly proves the eye is present on every secret field. Include a **non-vacuity negative**: a projection that omits the eye REDs against the real render.
7. **Slot-arm isolation.** The `(Path, hint)` arm of `slot_editor.rs` is unaffected (no eye on a non-secret slot row); the eye appears only on `is_secret_bearing()` rows.
8. **Composite gating (#3).** The eye appears only when `is_secret_node` is true; switching the composite node from secret→non-secret removes the eye and the field becomes plainly readable (no stale reveal state leaking a now-non-secret field, and vice-versa).
9. **Tutorial-gate negative** (tutorial repo side): a NON-allowlisted secret appearing revealed in a shot REDs the secret-allowlist checker; an allowlisted public demo phrase passes.

Per project memory `feedback_r0_review_run_full_package_suite`: R0/per-phase reviews run the FULL `cargo test -p mnemonic-gui` suite (reveal ripples into faithfulness / render-emit / persistence gates outside any one phase's targets), plus the toolkit `verify-figures-gui` / structural-render census on the re-pin side.

---

## 9. Batch coordination

This feature ships BATCHED to make the expensive re-pin + PDF rebuild happen **once**:

- **ONE GUI tag `mnemonic-gui-v0.57.0`** carrying: (a) this reveal toggle, (b) the restore `(none)` fix (`design/SPEC_restore_template_none_affordance.md`, the F1-shaped `restore-form-single-sig-template-leaks-in-md1-mode` fix already queued in memory), (c) any audit-confirmed fixes ready in the same window.
- **ONE tutorial re-drive** (§7.3) covering both the reveal (public-phrase reveal) and the restore-fix (`--template (none)` affordance) screenshot deltas.
- **ONE `manual-gui-v1.3.0`** release: re-pin `pinned-upstream.toml` v0.56.0 → v0.57.0, regenerate the ~28 `.gui` renders + ~28 PNG figures + the tutorial corpus + the standalone `gui_example.pdf`, update prose, one PDF rebuild.
- Sequencing: GUI leg (PR + CI incl. `snapshots`/`tutorial-snapshots` gates + post-impl adversarial review) → merge → tag `mnemonic-gui-v0.57.0` → toolkit leg (pin bump fires `schema_mirror` — expected GREEN, no schema delta — plus the manual re-pin + tutorial re-drive) → tag `manual-gui-v1.3.0`.
- Cross-repo FOLLOWUPS: file `gui-secret-reveal-toggle` in `mnemonic-gui/FOLLOWUPS.md` with a `Companion:` line into the toolkit manual-gui FOLLOWUPS for the re-pin/tutorial obligation (per CLAUDE.md cross-repo mirror rule). If OQ-2 = defer tree sites, file `gui-secret-reveal-tree-key-sites`.

---

## 10. Open questions for R0

- **OQ-1** (§4.3): keep the hybrid hold + bounded-latch, or collapse to latch-only? Rec: keep hybrid.
- **OQ-2** (§5.2): cover all 5 masking sites, or PRIMARY 3 now + defer tree sites #4/#5? Rec: cover all 5; deferral acceptable with a FOLLOWUP.
- **OQ-3** (§7.2): depict the reveal marker in the ASCII structural render (28 `.gui` re-pin), or treat as chrome (0 `.gui`, faithfulness-gate-only)? Rec: depict.
- **OQ-4** (§4.4): confirm NO timeout in v1 (determinism). Rec: no timeout.
- **OQ-5** (§7.1): confirm `schema_mirror` needs no edit (reveal adds no schema surface). Verify at plan time.

---

## 11. Blocker assessment

**No blocker.** The secret value is held safely enough to reveal (`Zeroizing<Vec<u8>>`, redacting Debug, non-Clone, `Zeroizing<String>` extraction — §2.1). Reveal introduces no new copy and no new persisted state; it flips a per-frame render flag bounded by the §4.5 auto-hide triggers and the single-revealed-field invariant. The pre-existing undo-ring residue (`gui-secret-buffer-allocator-residue`) and the Linux OS-snapshot gap (`gui-os-snapshot-secret-occlusion`) are orthogonal and unchanged; reveal does not widen them (§4.7). The real work is (a) rigorously bounding the deliberate exposure (§4) and (b) the quantified-but-mechanical manual re-pin + tutorial re-drive (§7), batched to run once (§9).
