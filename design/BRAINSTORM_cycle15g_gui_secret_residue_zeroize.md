# BRAINSTORM — cycle-15 Lane G: SECRET GUI-RESIDUE zeroize leg (`mnemonic-gui`)

- **Repo:** `mnemonic-gui` (the downstream `mnemonic`-toolkit GUI consumer).
- **Cycle / Lane:** cycle-15, Lane G — "secret GUI-residue zeroize."
- **Type:** brainstorm SPEC. **DESIGN ONLY — no code in this document.** Feeds the mandatory R0 reviewer-loop (see §10).
- **Source pin:** all citations re-grepped live against `origin/master` @ **`5ce9d53`** (`design(sweep): file GUI secret-key-material hygiene FOLLOWUP slugs`), whose only delta over the shipped **v0.46.0** tag (`1999323`, Merge PR #14 cycle-11a) is `+50` lines to `FOLLOWUPS.md` — i.e. **all `src/**` line numbers below are byte-identical to v0.46.0**. `git fetch -q origin` performed 2026-06-21.
- **Recon basis:** `/scratch/code/shibboleth/wt-tk-master/design/agent-reports/sweep-keymat-mnemonic-gui.md` (keymat sweep). FOLLOWUP slugs in `mnemonic-gui/FOLLOWUPS.md` (entries at `:40`, `:53`, `:66`, `:78`).
- **Lens:** secret-memory-hygiene is a FIRST-CLASS bar in the m-format constellation; a GUI has extra residue surfaces (app-level run holders, on-screen widgets) that the CLI does not.
- **Review history:** R0-r1 review persisted verbatim at `design/agent-reports/cycle15g-spec-r1-review.md` (NOT GREEN — 1 Important **I1** + 4 Minor). This is the **R1-folded** revision: I1 (the `PendingConfirm` E0509 move-out-of-`Drop` hazard → §3.2 + D2), M2 (tree-render seam → §6/T7 + D7), M3 (composite `Response` preservation → §4 + T5) all folded. **M1 was a reviewer misread** (the reviewer claimed the store is `:1223`; re-grep confirms `app.last_run = Some(result)` is at **`:1222`** and `:1223` is `app.last_run_error = None` — the spec's original `:1222` is correct; see §2 note). M4 was already correct in the spec (`stdout`/`stderr` are `Vec<u8>`). Re-dispatch for R0-r2.

---

## 0. Framing — what this is NOT

The two shipped defenses are **complete within their declared scope** and this cycle does **not** re-do them:

- **M9 `TreeNode::zeroize_keys`** (v0.46.0, `src/form/tree_model.rs`, wired into the exit sweep at `secrets.rs:332-333`) — scrubs the descriptor-builder model tree's `key`/`keys[i]` through all children. Complete for the *model-side tree*.
- **On-disk redactor** (`blank_non_extended_public_keys` + `redact_for_persistence`) — complete for the *persist path*.

The four gaps in scope are in surfaces **NEITHER defense reaches**:

1. **app-level run holders OUTSIDE `FormState`** — the exit sweep (`main.rs:1144-1145`) iterates only `self.form_state.values_mut()`, so `app.last_run` and `app.pending_confirm_argv` are structurally invisible to it (slugs 1 & 2);
2. **on-screen widget cleartext** — two secret-input `text_edit_singleline` widgets render without `.password(true)` (slugs 3 & 4).

**Frame the fix as extending coverage to those surfaces, not re-doing M9.**

---

## 1. Scope — the four slugs (one cycle, one PR)

| # | slug | severity | gap class | fix family |
|---|------|----------|-----------|------------|
| 1 | `gui-last-run-result-argv-stdout-not-zeroized` | **MED** | in-RAM residue (app holder, no disk) | scrub-on-replace + cover in exit sweep |
| 2 | `gui-pending-confirm-argv-not-zeroized` | **MED** | in-RAM residue (app holder, no disk) | scrub-on-clear + cover in exit sweep |
| 3 | `gui-composite-secret-value-rendered-cleartext-onscreen` | **LOW/MED** | on-screen widget cleartext | `.password(true)` gated on argv-secret node |
| 4 | `gui-tree-key-field-rendered-cleartext-onscreen` | **LOW** | on-screen widget cleartext | `.password(true)` gated on `is_xprv_like` |

Slugs 1–2 = the **run-holder scrub** family. Slugs 3–4 = the **widget-masking** family. Both bundle naturally; ship together.

### Out of scope — deliberately NOT re-filed
- The clipboard copy of the real (secret) argv (`main.rs:1030/1036`, `ctx.copy_text`) — **informed-consent design** (buttons relabeled "— reveals secret" v0.39.0; no egui clipboard-clear API). Accepted residue, like the allocator/OS-snapshot caveats.
- `gui-tree-key-egui-undo-ring-residue` (deferred undo-ring residue) — a DISTINCT class from slug 4 (which is the live-widget cleartext render); not addressed here.
- `composite-paste-warn-parity` / `gui-slot-value-no-paste-warn` (the paste-warn affordance) — sibling but separate; the v0.45.0 H3 work covered the funds-leak surface; NOT in this masking cycle.
- `gui-secret-buffer-allocator-residue` — already shipped (Phase B.1, `FOLLOWUPS.md:725`).
- v0.45.0-fixed items (`gui-runner-debug-logs-unmasked-secret-argv`, minikey H3 surfaces).

---

## 2. Verified citations (live `origin/master` @ `5ce9d53`)

### Slug 1 — `last_run`
- `src/main.rs:104` — `last_run: Option<runner::RunResult>` (field decl).
- `src/main.rs:319` — `last_run: None` (constructor).
- `src/main.rs:1206` — `app.last_run = None` (cleared on run start).
- `src/main.rs:1222` — `app.last_run = Some(result)` (**stored — the replace site; old value drops non-scrubbing**). (R0-r1 M1 claimed `:1223`; re-grepped — `:1221` is `result.mask = mask;`, `:1222` is the store, `:1223` is `app.last_run_error = None`. Spec's `:1222` is correct; M1 is a reviewer misread, no change.)
- `src/main.rs:1226` — `app.last_run = None` (cleared on spawn failure).
- `src/main.rs:1190-1195` — `fn spawn_and_capture(app: &mut MnemonicGuiApp, argv: Vec<String>, mask: Vec<bool>, stdin: Option<Vec<u8>>)` — takes **owned** values (drives the I1 consumption-pattern constraint, §3.2). `last_run` is read by-ref only at `:470` (`if let Some(ref result) = self.last_run`), never moved-out — so `impl Drop for RunResult` is E0509-clean.
- `src/runner.rs:18-31` — `pub struct RunResult { argv: Vec<String> (:20), mask: Vec<bool> (:27), exit_code: Option<i32> (:29), stdout: Vec<u8> (:30), stderr: Vec<u8> (:31) }`. `#[derive(Debug)]` only — **no `Zeroize`, no `Drop`**.
- Exit sweep: `src/main.rs:1121` `fn on_exit`, scrub at `:1144-1145` (`for state in self.form_state.values_mut() { secrets::zeroize_form_state(state); }`) — **never visits `last_run`**.

### Slug 2 — `pending_confirm_argv`
- `src/main.rs:92` — `type PendingConfirm = (Vec<String>, Vec<bool>, Option<Vec<u8>>);` (argv, mask, spec_stdin).
- `src/main.rs:114` — `pending_confirm_argv: Option<PendingConfirm>` (field decl).
- `src/main.rs:324` — `pending_confirm_argv: None` (constructor).
- `src/main.rs:1045` — `self.pending_confirm_argv = Some((argv, mask, spec_stdin));` (set when `needs_confirm`).
- `src/main.rs:1064` — `if let Some((argv, mask, stdin)) = self.pending_confirm_argv.clone()` (**per-frame `.clone()` while modal open — a transient residue copy dropped non-scrubbing at end of `update()`**).
- `src/main.rs:1092` — `self.pending_confirm_argv = None;` (Run path; the moved-out `argv`/`stdin` clone goes into `spawn_and_capture`).
- `src/main.rs:1096` — `self.pending_confirm_argv = None;` (Cancel path — plain drop, no zeroize).

### Slug 3 — composite value widget
- `src/form/widget.rs:653` — `let response = ui.text_edit_singleline(value);` in the `NodeValueComposite` arm (`FlagValue::NodeValueComposite { node, value }` at `:625`).
- In-source comment `:647-648` — *"The value field is a plain (non-password) text edit; mirror …"* (the phrase begins at `:647`).
- The argv-secret predicate is ALREADY in this arm: `:663` `crate::secrets::node_type_is_argv_secret(node.as_str())` (used today for paste-warn gating).
- Contrast (the established pattern): `src/form/secret_widget.rs:84` — `ui.add(egui::TextEdit::singleline(&mut transient).password(true))` (SecretLineEdit, backed by a `Zeroizing` transient). Slots are `.password(true)` since v0.38.0; Text flags route to SecretLineEdit.

### Slug 4 — tree key widgets
- `src/form/tree_form.rs:697` — `ui.text_edit_singleline(&mut node.key);` (PayloadShape::Key arm).
- `src/form/tree_form.rs:717` — `ui.text_edit_singleline(&mut node.keys[i]);` (PayloadShape::KeyQuorum loop).
- `src/form/tree_form.rs:699` / `:723` — `xprv_hint(ui, &node.key)` / `xprv_hint(ui, &node.keys[i])` (amber warning, does NOT mask).
- `src/form/tree_form.rs:785` — `fn xprv_hint`; the predicate it uses, `pub fn is_xprv_like` (the reusable masking gate), lives at **`src/form/tree_model.rs:675`** (NOTE: in `tree_model.rs`, not `tree_form.rs`).

### Shared infrastructure (already present — no new deps)
- `Cargo.toml:20` — `zeroize = "1"` (runtime dep).
- `Cargo.toml:74` — `egui_kittest = "0.31"` (dev-dep, drives frames for `Role::PasswordInput` assertions).
- `src/secrets.rs:135` — `use zeroize::Zeroize;`.
- `src/secrets.rs:279-288` — `SecretBuffer` impls `Zeroize` + `Drop` (the established zeroize-on-drop pattern to mirror).
- `src/secrets.rs:294` — `pub fn zeroize_form_state(state: &mut FormState)` (the exit sweep body).
- `src/secrets.rs:175` — `pub fn node_type_is_argv_secret(node: &str) -> bool` (slug-3 gate, `SECRET_NODE_TYPES_ARGV`).

---

## 3. Key decision — whole-holder zeroize vs redact-then-store / thread-secrecy

The run holders carry the secret as a **cleartext token inside a larger `Vec<String>` / `Vec<u8>`** (`--passphrase <seed>`, `--from phrase=<seed>`, `@N.phrase=<seed>`, plus `spec_stdin` and captured `stdout`). `mask: Vec<bool>` is **display-only** — it governs which token renders `SECRET_MASK` on screen; it does NOT scrub the stored bytes.

**Resolved: zeroize the WHOLE holder.** Rationale (simplest-correct, the standing preference):
- Any element of the argv/stdin/stdout MAY carry a secret (the mask is a render hint, not a security boundary, and could drift); scrubbing the whole `Vec<String>`/`Vec<u8>` is fail-closed.
- It mirrors `zeroize_form_state`, which already blanks every string-bearing value (not just secrets) precisely because per-element secrecy is fragile.
- Redact-then-store would lose the real argv the copy buttons + run path legitimately need (the holder must keep cleartext WHILE live); the fix is to scrub it **when it leaves scope** (replace / clear / exit), not to never store it.
- Threading secrecy through `RunResult` per-field is more code for no security gain over whole-holder scrub.

`Vec<String>` and `Vec<u8>` both implement `Zeroize` (zeroize 1.x); `String: Zeroize` is live (used by M9). So whole-holder scrub is a `.zeroize()` on each field.

### Two implementation shapes for the run-holder scrub (R0 to pick)
- **(A) zeroize-on-drop:** `impl Zeroize for RunResult` (zeroes all four `Vec`s) + `impl Drop for RunResult { fn drop(&mut self){ self.zeroize() } }`. Then the replace at `main.rs:1222`, the clears at `:1206/:1226`, and the `pending_confirm` clears at `:1092/:1096` scrub **automatically** on drop; the per-frame `.clone()` at `:1064` ALSO scrubs when its transient drops at end of `update()`. The exit sweep gains an explicit `app.last_run.take()` / scrub the pending holder so RAM is overwritten as early as possible at shutdown (matching the SecretBuffer/secret_widgets "drop ahead of the map's own Drop" rationale in `zeroize_form_state`).
- **(B) explicit scrub-on-replace/clear + exit sweep:** no `Drop`; an explicit `scrub()`/`zeroize()` method called at every replace/clear site and from `on_exit`. More call-sites, easy to miss one (the `.clone()` transient at `:1064` is the trap).

**RESOLVED: (A) zeroize-on-drop, because the per-frame `.clone()` at `main.rs:1064` is exactly the kind of residence (B) silently misses.** For `RunResult`: `impl Zeroize + Drop for RunResult` — clean, nothing moves fields out of `last_run` (read by-ref only at `:470`). For `pending_confirm_argv` (a bare tuple `PendingConfirm`): **promote to a small struct with `Zeroize`+`Drop`** (`struct PendingConfirm { argv: Vec<String>, mask: Vec<bool>, stdin: Option<Vec<u8>> }`), mirroring (A). **`mask: Vec<bool>` is non-secret; zeroizing it is harmless and keeps "whole-holder" uniform.**

### 3.2 — I1 fold: `PendingConfirm` + `Drop` consumption pattern (MUST follow — avoids E0509)

**Hazard (R0-r1 I1, BLOCKING-resolved here):** the current consume site at `main.rs:1064` is `if let Some((argv, mask, stdin)) = self.pending_confirm_argv.clone() { … spawn_and_capture(self, argv, mask, stdin); }` — it **moves the inner fields out** of the cloned value. Rust **forbids moving fields out of a type that implements `Drop`** (compile error **E0509** "cannot move out of type which implements the `Drop` trait"). So `let Some(PendingConfirm { argv, mask, stdin }) = …clone()` against a `Drop`-bearing struct **will not compile**. The consumer `spawn_and_capture` (`main.rs:1190`) takes **owned** `Vec<String>`/`Vec<bool>`/`Option<Vec<u8>>`, and the Run arm at `:1092` moves all three in.

**REQUIRED consumption-pattern change (the implementer MUST apply this; the naive field-destructure is a hard error):** bind the **whole struct**, then **re-clone the inner fields on Run**:

```rust
// at :1064 — bind the whole cloned PendingConfirm (do NOT destructure its fields)
if let Some(pending) = self.pending_confirm_argv.clone() {
    // … modal renders BORROWING pending.argv / pending.mask …
    // Run arm:
    self.pending_confirm_argv = None;
    spawn_and_capture(self, pending.argv.clone(), pending.mask.clone(), pending.stdin.clone());
    // `pending` (a full PendingConfirm) drops at end of scope → its Drop fires
    // → the per-frame transient IS scrubbed (D2's hygiene goal preserved).
}
```

- The cloned `pending` drops at scope end → `Drop` scrubs the per-frame transient — **the residue this cycle is closing.**
- Only the briefly-live Run-path inner clones (`pending.argv.clone()` etc.) are un-drop-scrubbed; they are owned, one-shot, and immediately consumed by `spawn`/the OS process spawn — a far smaller residence than the per-frame modal clone. (Acceptable; not worth threading `Drop` through `spawn_and_capture`'s owned params.)
- **DO NOT "fix" an E0509 by deleting `Drop`** — that silently reverts to shape (B), reintroducing the `:1064` transient the spec rejects. The plan-doc and per-phase TDD must pin this (a compile-green build + the T3/T4 scrub tests are the guard).

> **Exit-sweep ordering caveat (load-bearing, R0 must preserve):** `on_exit` (`main.rs:1121`) saves state.json FIRST, then sweeps (`:1144-1145`), because `zeroize_form_state` blanks ALL string values and sweeping first would persist a gutted form. The run-holder scrub additions go in the **same post-save sweep block** — but note `last_run`/`pending_confirm` are NOT serialized (no `Serialize`), so their scrub order vs the save is irrelevant; place them with the existing sweep for locality. Do NOT `mem::take(self.form_state)` (R0-r1 I2 from the M9 cycle).

---

## 4. Widget-masking approach (slugs 3 & 4)

Both are the same class: a secret-input `text_edit_singleline` that should be `.password(true)`. **No behavior change beyond on-screen masking** — the widget still accepts the same typed/pasted input; the bytes still flow to the same form-state `String`; only the on-screen glyphs become `•`. Persist-redaction, RAM-zeroize, paste-warn, and run-confirm are unchanged.

- **Slug 3 (composite):** at `widget.rs:653`, render `egui::TextEdit::singleline(value).password(true)` **GATED** on `secrets::node_type_is_argv_secret(node.as_str())` — the SAME predicate already evaluated three lines below at `:663`. Non-secret composite nodes (xpub / path / number) stay plain (so users can read a watch-only xpub). This mirrors the v0.38.0 slot masking.
  - **Decision: inline `.password(true)`, NOT a full SecretLineEdit swap.** The composite value lives in the form-state `String` (`FlagValue::NodeValueComposite{value}`), which the exit sweep (`secrets.rs:303`) + persist-redactor already scrub. A SecretLineEdit swap would re-home the buffer into `secret_widgets` and ripple through assembly/persistence for no hygiene gain (the buffer is already swept). Inline `.password(true)` is the minimal, fail-safe change. R0 may revisit if it prefers buffer unification.
  - **M3 fold — preserve the `Response` (paste-warn must not regress).** The arm captures `let response = ui.text_edit_singleline(value);` and reads `response.changed()` for the paste-warn detection at `:654-665`. The masking change MUST keep returning that same `Response`: replace with `let response = ui.add(egui::TextEdit::singleline(value).password(<gate>));` — `ui.add(..)` returns the `Response` identically. Do NOT switch to a `.show(ui)` form that buries the response, and do NOT drop the `let response =` binding. The gate `<gate>` = `node_type_is_argv_secret(node.as_str())` evaluated ONCE (hoist it above the `text_edit` so the same boolean drives both `.password(<gate>)` and the existing `:663` paste-warn condition — avoids double-evaluation drift). T5 asserts masking; **add a regression assertion that paste-warn still co-fires** for a secret composite (belt-and-suspenders, mirrors the T6 split-brain pin).
- **Slug 4 (tree key):** at `tree_form.rs:697` and `:717`, render `.password(true)` **GATED** on `tree_model::is_xprv_like(&node.key)` / `is_xprv_like(&node.keys[i])`. The amber `xprv_hint` already keys off the same predicate, so masking and the hint co-fire. Watch-only xpubs (the canonical input) stay readable; only a mis-pasted xprv-shaped string masks.
  - **Edge: input-then-mask flicker.** `is_xprv_like` becomes true only once enough of the xprv prefix is typed; the field un-masks while empty/partial and masks once it looks like an xprv. This is acceptable (a paste lands as one event → masks immediately; manual typing of an xprv into a watch-only builder is non-canonical). R0 to confirm vs always-mask (rejected: always-mask would hide the watch-only xpub the field is FOR).

---

## 5. Resolved-decisions table

| # | Decision | Resolution | Rationale |
|---|----------|------------|-----------|
| D1 | Whole-holder zeroize vs redact-then-store vs thread-secrecy | **Whole-holder** (zeroize entire argv/stdin/stdout/stderr) | Any element may carry a secret; mask is display-only, not a boundary; fail-closed; mirrors `zeroize_form_state` |
| D2 | Run-holder scrub mechanism | **(A) zeroize-on-drop** — `impl Zeroize+Drop for RunResult`; promote `PendingConfirm` to a struct with the same. **The `:1064` consumer MUST bind the whole struct + re-clone inner fields on Run (NOT field-destructure) — see §3.2** | Auto-scrubs the per-frame `.clone()` at `main.rs:1064` that explicit-scrub (B) would miss; field-destructure of a `Drop` type is E0509 (I1) |
| D3 | Exit-sweep coverage of run holders | Add `last_run` + `pending_confirm` scrub to the **post-save** sweep block in `on_exit` | Early RAM overwrite at shutdown; holders aren't serialized so order vs save is moot; preserve save-FIRST/no-`mem::take` from M9 cycle |
| D4 | Composite secret masking mechanism | **Inline `.password(true)`** gated on `node_type_is_argv_secret(node)` | Value already lives in the swept form-state String; SecretLineEdit swap = ripple for no hygiene gain |
| D5 | Tree-key masking gate | `.password(true)` gated on `tree_model::is_xprv_like(key)` | Watch-only xpub stays readable; only mis-pasted xprv masks; co-fires with existing amber hint |
| D6 | `mask: Vec<bool>` (non-secret) — zeroize it too? | **Yes** (harmless) | Keeps "whole-holder" uniform; avoids per-field secrecy threading |
| D7 | Testability seam | Route scrub logic through **public lib-crate methods** (`RunResult::zeroize` via the trait; a `PendingConfirm` scrub) so pure-logic RED tests assert without driving the bin crate. **Tree-render mask test (T7) uses the public `tree_form::render` with a constructed Key-node `FormState` (M2 fold, §6/T7); `pub(crate)` render seam only as a last resort** | `widget_secret.rs` documents that `main.rs` app-state fields are private to integration tests; mirror `SlotRow::zeroize_if_secret`. Composite (T5) is reachable via `pub widget::render`; tree (T7) has no public payload seam — drive the top-level `render` |
| D8 | SemVer | **GUI MINOR 0.46.0 → 0.47.0** | New hygiene behavior (scrub + mask); no breaking API |
| D9 | Toolkit pin bump? | **No** | Pure internal app-state + widget change; independent of Lanes M/T; reuses existing toolkit-pinned taxonomy |
| D10 | `schema_mirror` impact? | **None** | No clap flag / option / subcommand / dropdown-value add/remove/rename — internal state + render only (§7) |
| D11 | `cargo fmt`? | **NEVER fmt the GUI** | No fmt CI gate in mnemonic-gui; touch only the changed lines |
| D12 | Composite `.password` must preserve the paste-warn `Response` (M3 fold) | Use `ui.add(egui::TextEdit::singleline(value).password(gate))` (returns `Response` identically); hoist `gate = node_type_is_argv_secret(node)` to one eval; add a paste-warn-still-fires regression assertion | The arm reads `response.changed()` for paste-warn at `:654-665`; a careless `.show(ui)` swap or dropped binding would silently regress paste detection |

---

## 6. Per-slug RED tests (TDD — written before impl, observably RED → GREEN)

> Seam discipline (D7): app-state fields in `main.rs` are private to integration tests (documented in `tests/widget_secret.rs`), so each scrub must expose a **public lib-crate seam** the test asserts directly — mirror `SlotRow::zeroize_if_secret` (`slot_secret_mask_v0_38_0.rs` T1). `RunResult` lives in the `runner` lib module (public), so it IS reachable.

### Slug 1 — `RunResult` zeroize-on-drop / scrub
- **T1 (pure logic, no harness):** build `RunResult{ argv: vec!["mnemonic".into(), "--passphrase".into(), "<seed>".into()], stdout: b"secret".to_vec(), … }`; call the new `.zeroize()` (or drop a `Zeroizing`-wrapped copy and re-read backing capacity per the SecretBuffer test idiom); assert `argv`, `stdout`, `stderr`, `stdin` are all empty/zeroed. **RED today: `RunResult` has no zeroize method.** Add to `tests/secrets.rs` or a new `tests/run_holder_zeroize.rs`.
- **T2 (replace-site, optional kittest):** drive one secret-bearing run, replace `last_run`, assert the prior holder's backing was scrubbed — likely deferred to the seam (bin-crate not harness-isolable); the unit T1 + a code-review assertion that `:1222` replace + `on_exit` cover it is the testable floor.

### Slug 2 — `pending_confirm_argv` scrub-on-clear
- **T3 (pure logic):** with `PendingConfirm` promoted to a struct, build one with a secret argv + `Some(spec_stdin)`; call its `.zeroize()` (or drop); assert argv tokens + stdin bytes zeroed. **RED today: no scrub method; tuple has no Drop.**
- **T4 (exit-sweep coverage):** assert the exit-sweep helper (factored as a pure `scrub_app_run_holders(&mut last_run, &mut pending)` lib seam) zeroes both holders — RED until the seam exists and is wired into `on_exit`.

### Slug 3 — composite `.password(true)` (kittest, `Role::PasswordInput`)
- **T5:** mirror `slot_secret_mask_v0_38_0.rs` T2: render the `NodeValueComposite` widget for a SECRET node (`node = "phrase"`, value set) in an `egui_kittest::Harness`; assert the value field registers **`Role::PasswordInput`**. Render it for a NON-secret node (`node = "xpub"`); assert it does **NOT**. **RED today: `:653` is plain `text_edit_singleline`.**
- **T6 (split-brain pin):** assert the masking gate == the argv-secret gate for ALL composite node types (`node_type_is_argv_secret`), so render-mask and the paste-warn/argv classification can never diverge (mirrors slot T3).

### Slug 4 — tree-key `.password(true)` (kittest)
- **T7:** render PayloadShape::Key with `node.key = "<xprv-shaped>"` (an `is_xprv_like`-true string); assert `Role::PasswordInput`. Render with `node.key = "<xpub-shaped>"` (watch-only, `is_xprv_like` false); assert NOT masked. Same for `keys[i]` (KeyQuorum). **RED today: `:697/:717` plain.**
  - **M2 fold — tree-render seam (the slot-test analogy does NOT transfer 1:1).** `render_node` (`tree_form.rs:601`) and `render_payload` (`:685`) are **private**; the only `pub` tree-render entry is `pub fn render(ui, state, bin)` at `:431` (slug-3's composite is reachable via `pub fn widget::render`/`render_with_dispatch` at `widget.rs:81/473` — fine). The plan-doc MUST resolve T7's seam one of two ways: **(i)** drive through the public `tree_form::render` with a constructed `FormState` whose `TreeState` root carries a Key (or KeyQuorum) node holding the xprv-shaped string, and query the rendered tree for the key field's `Role::PasswordInput` (heavier harness setup but no API surface change — **preferred**); or **(ii)** expose a narrow `pub(crate)` render seam (e.g. `pub(crate) fn render_payload`) so the kittest can target just the payload widget. Pick (i) unless the constructed-tree harness proves intractable; (ii) widens the lib surface and should be a last resort. The plan-doc names the chosen seam concretely before P-tree TDD.

---

## 7. SemVer, PR gate, and drift-gate notes

- **SemVer: GUI MINOR `0.46.0 → 0.47.0`.** New hygiene behavior; no breaking API. Bump `Cargo.toml:3`, both READMEs (release-ritual version sites — silent drift surfaces), and any self-pin per the GUI release checklist. (Toolkit/GUI release-ritual version sites per `project_toolkit_release_ritual_version_sites`.)
- **PR-gated, NOT direct-FF:** mnemonic-gui ships via PR + **5-target CI green BEFORE tag** (GUI is the PR-gated repo in the constellation; codec/toolkit are direct-FF+tag). Open the PR; let CI go green on all 5 targets; then merge + tag `v0.47.0`.
- **NO toolkit pin bump** (D9). Independent of Lanes M/T (toolkit/`mnemonic-secret` etc.). Reuses the already-pinned `mnemonic-toolkit-v0.60.0` taxonomy (`SECRET_NODE_TYPES_ARGV`, `node_type_is_argv_secret`).
- **`schema_mirror` — NO impact (D10).** These are internal app-state (`RunResult`/`PendingConfirm` zeroize) + render (`.password(true)`) changes; **no clap flag / option / subcommand / dropdown-value addition, removal, or rename.** The `schema_mirror` gate enforces clap flag-NAME parity (+ dropdown value enums) and is untouched. The lockstep `mnemonic-gui/src/schema/mnemonic.rs` does NOT change. Confirm in the PR description (no `gui-schema` delta).
- **`canonicity_drift` / `schema_mirror` / `xpub_search_schema_mirror` / `archetype_schema_mirror` MUST run against the PINNED `mnemonic-toolkit-v0.60.0` binary, NOT a stale `$PATH mnemonic`.** Known false-fail mode: a stale `$PATH` toolkit (e.g. v0.56.0) makes `schema_mirror` mis-compare against the v0.60.0-pinned schema. Run with `MNEMONIC_BIN=<path-to-v0.60.0>` (and the sibling `MD_BIN`/`MS_BIN`/`MK_BIN` as the lint expects). CI uses the pinned binary; local runs must export it.
- **NEVER `cargo fmt` the GUI (D11).** mnemonic-gui has NO fmt CI gate; a blanket `cargo fmt` would reformat unrelated files with no gate to catch it. Edit only the changed lines.

---

## 8. Behavior-preservation invariant

No behavior change beyond masking + scrubbing:
- the composite / tree-key widgets accept the **same** typed/pasted input; the secret is `.password`-masked on screen only;
- the run path (`spawn_and_capture`), copy buttons (deliberate-reveal), run-confirm modal masking, paste-warn, persist-redaction, and the M9 tree-model sweep are **unchanged**;
- the run holders keep cleartext WHILE live (the copy + run paths need it); they are scrubbed only on replace / clear / exit.

---

## 9. Residual caveats (documented, NOT in scope)

- **Allocator residue** (`gui-secret-buffer-allocator-residue`) — `Zeroize` overwrites the bytes but the heap allocator may retain a copy. Accepted, already-tracked caveat; whole-holder zeroize inherits it.
- **egui undo ring** (`gui-tree-key-egui-undo-ring-residue`) — the `text_edit_singleline` undo ring retains keystrokes after model zeroize. `.password(true)` does NOT clear the undo ring; this slug-4 fix is the on-screen render only. Separate deferred mitigation.
- **OS snapshot / screenshot** (`gui-os-snapshot-secret-occlusion`) — Linux unmitigated. Out of class.
- **Clipboard** — deliberate informed-consent reveal (§1).

---

## 10. Mandatory R0 gate

Per `CLAUDE.md` (the standing hard gate): **this brainstorm SPEC must pass an opus-architect R0 review to 0 Critical / 0 Important BEFORE any implementation begins.** No code, no implementer dispatch, no plan-doc advance until GREEN. After each fold, **persist the review verbatim** to `design/agent-reports/cycle15g-...-review.md`, re-dispatch the architect, repeat until 0C/0I (folds can introduce drift — the reviewer-loop continues after every fold). Then the plan-doc gets its OWN R0 loop, then per-phase TDD with per-phase R0, then the mandatory whole-diff post-implementation adversarial review.

### R0-r1 dispositions (ratified — closed)
The following were OPEN at r1 and the r1 review **ratified** them; recording the resolutions so r2 need not re-litigate:
1. ✅ **D2 mechanism (`PendingConfirm` struct + `Drop`)** — ratified, WITH the I1 consumption-pattern correction (§3.2: bind whole struct + re-clone on Run; field-destructure of a `Drop` type is E0509). **This was the sole Important; now folded.**
2. ✅ **D4 composite masking depth** — inline `.password(true)` ratified sufficient (value already in the swept form-state String; SecretLineEdit swap = ripple for no hygiene gain).
3. ✅ **`stdout`/`stderr` whole-scrub** — **blessed.** Captured output is RAM-only (not serialized — `RunResult` is `Debug`-only), rendered at `:470` then never re-read for logic; some flows emit secret-class stdout; whole-scrub is fail-closed and correct. No display regression (the pane reads the live value while shown; scrub fires on next-run-replace / exit).
4. ✅ **Slug-4 flicker (D5)** — gated masking ratified (watch-only xpub must stay readable; paste lands as one event → masks immediately).
5. ✅ **T2 testability floor** — unit `RunResult::zeroize` test + code-review assertion of the `:1222`/`on_exit` wiring is an acceptable floor (bin-crate app state is not harness-isolable).

### Open questions FOR R0-r2
None substantive remain after the I1/M2/M3 fold. Confirm only:
- **(a)** the §3.2 consumption-pattern (whole-struct bind + Run-path inner re-clone) is the agreed shape and the plan-doc/TDD will pin "compile-green + T3/T4 scrub" as the guard against a `Drop`-deleting E0509 "fix";
- **(b)** the T7 tree-render seam choice — public-`render` constructed-tree (preferred) vs `pub(crate)` payload seam — can be deferred to the plan-doc, or should this spec pre-commit to (i)?
