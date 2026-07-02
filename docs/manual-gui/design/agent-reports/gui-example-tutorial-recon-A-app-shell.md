# gui_example.pdf cycle — RECON report A (app-shell extraction map)

> Provenance: recon subagent report, persisted verbatim per house convention.
> Cycle: `gui_example_tutorial` (spec: `docs/manual-gui/design/SPEC_gui_example_tutorial.md`).
> Dispatched 2026-07-02 by the RECON+SPEC author. Sources verified against
> mnemonic-gui `master@0d4429d` (v0.55.0) working tree.

---

# Recon Report A — Driving the REAL full window under egui_kittest

Repo: `/scratch/code/shibboleth/mnemonic-gui` (mnemonic-gui v0.55.0, `src/main.rs`). All line citations are against the current working tree.

## 1. `src/main.rs` structure — the eframe::App and its panels

**Binary entry / OS runtime** (`src/main.rs`):
- `fn main() -> eframe::Result<()>` at `main.rs:37`. Resolves `persistence::default_state_path()` (`:47`), loads state (`:48`), builds `egui::ViewportBuilder` (`:53-60`), `eframe::NativeOptions` (`:62-65`), and calls `eframe::run_native(...)` (`:66-70`) with `MnemonicGuiApp::new(cc, loaded_state, state_path)`.

**The App struct**: `struct MnemonicGuiApp` at `main.rs:98` (bin-private; NOT `pub`).

**`impl eframe::App for MnemonicGuiApp`** at `main.rs:383`; `fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)` at `main.rs:384`. Note the `_frame` param is underscore-unused — verified no `frame.` access anywhere in the body (the only "frame" token in the body is a comment at `:572`). **The update body depends only on `ctx` and `self`, never on `eframe::Frame`.**

Panels enumerated in `update()`:
- **Per-frame geometry snapshot** (not a panel): `ctx.input(...)` at `main.rs:395-403`.
- **Autosave timer**: `main.rs:411-421`.
- **Tab bar** — `egui::TopBottomPanel::top("tabs")` at `main.rs:424`; the tab strip is a `ui.horizontal` loop over `CliTab::ALL` at `main.rs:428-451` (heading `:426`, per-tab button `:435`, grey-out fill `:437`, `add_enabled` `:439`, missing-binary tooltip `:441-443`, active marker `◀` `:449`).
- **Output panel** — `egui::TopBottomPanel::bottom("output")` at `main.rs:456`. Toggle checkboxes `:458-460`; error line reads `self.last_run_error` `:470`; result block reads `self.last_run` `:473`; masked argv line `:478-485`; `render_exit_badge` `:487`; **stdout `ScrollArea::vertical().id_salt("stdout").max_height(180.0)`** `:490-495`; **stderr `ScrollArea::vertical().id_salt("stderr").max_height(120.0)`** `:499-504`; "(no run yet)" fallback `:507`.
- **Central form** — `egui::CentralPanel::default()` at `main.rs:512`. Contains:
  - Pinned-version line + **subcommand selector** `egui::ComboBox::from_label("subcommand")` at `main.rs:525`, iterating `sch.subcommands` as `selectable_label`s `:528-536`; per-subcommand `?` help icon `:545-554`.
  - Active-subcommand resolution + `FormState` entry `main.rs:558-591`.
  - **The form ScrollArea**: `egui::ScrollArea::vertical().show(ui, ...)` at `main.rs:593`, spanning `:593-889`. Inside: build-descriptor mode selector `:601-603`; the generic **flag-widget loop** `:624-710` whose per-flag render is `widget::render_with_dispatch(...)` at `main.rs:677`; archetype sub-form `:692-708`; SlotEditor `form::slot_editor::render` `:782`; positional widgets `:832-880`; tree form `form::tree_form::render` `:887`.
  - **Action bar** (`ui.horizontal`) `main.rs:975-1026`: Copy POSIX/Windows/spec buttons, and the **`Run` button** at `main.rs:1023`; `Preview:` label `:1027`; click handling `:1029-1056`.

There is **no `SidePanel`** — the subcommand list is a `ComboBox` dropdown inside the CentralPanel, not a side list. (Note for the goal: "subcommand selector list" today = a combo popup, not a persistent side list.)

Two `egui::Window` modals rendered after the panels (see §6).

## 2. App-struct state vs FormState

`MnemonicGuiApp` fields (`main.rs:98-151`):
- `app_state: AppState` (`:99`) — per-CLI `Detected` + `active_tab` (defined in `src/app.rs:63-70`).
- `active_subcommand: BTreeMap<CliTab, String>` (`:102`).
- `form_state: BTreeMap<String, FormState>` (`:104`) — keyed `"cli:subcommand"` (`Self::form_key`, `main.rs:373-375`).
- `last_run: Option<runner::RunResult>` (`:105`) — the output-panel content source.
- `last_run_error: Option<String>` (`:106`).
- `show_cmdline / show_stdout / show_stderr: bool` (`:107-109`).
- `pending_confirm_argv: Option<PendingConfirm>` (`:117`) — run-confirm modal state.
- `pending_paste_warn: bool` (`:121`) — paste-warn modal flag.
- `last_template: BTreeMap<String, Option<String>>` (`:131`).
- `state_path: Option<PathBuf>` (`:136`).
- `last_autosave: std::time::Instant` (`:141`), `last_saved_snapshot: Option<String>` (`:142`).
- `window_size / window_position: Option<[f32;2]>` (`:149-150`).

There is **no `theme` field and no runner handle** — runs are synchronous (see §3). What lives in `FormState` (per-subcommand, owned in the map, NOT on the app struct): flag `values`, `slots`, `positionals`, `secret_widgets` (the secret-widget row buffers) — referenced throughout `main.rs:578-887` and enumerated in `tests/ui_harness/mod.rs:62-65`. `FormState` is **not `Clone`** (`main.rs:277-280`).

## 3. OS/eframe-runtime entanglement (what breaks headless)

All entanglement is in `main()` and in `MnemonicGuiApp::new(cc, ...)`; the `update()` body is essentially eframe-free.

Entangled points:
- `eframe::run_native` — `main.rs:66`. In `main()` only.
- `egui::ViewportBuilder` / `NativeOptions` — `main.rs:53-65`. In `main()` only.
- **`cc.window_handle()` → `platform::apply_window_capture_protection`** — `main.rs:166-168`; already defensive: on `Err` it just warns (`:169-174`). Only reachable via `CreationContext`.
- **Keepalive thread** — `std::thread::Builder...spawn(loop { sleep(1s); ctx.request_repaint() })` at `main.rs:195-202`. Clones `cc.egui_ctx`. Spawns an unbounded background thread.
- **Signal handlers** — `#[cfg(unix)]` `signal_hook::iterator::Signals::new([SIGINT, SIGTERM])` + `ctx.send_viewport_cmd(ViewportCommand::Close)` at `main.rs:217-243`; `#[cfg(windows)]` `ctrlc::set_handler` at `main.rs:245-256`. Global process state.
- `send_viewport_cmd` — only inside those handler threads (`:236`, `:250`); **not in `update()`**.
- `request_repaint` — only in the keepalive thread (`:200`); **not in `update()`**.
- `AppState::detect_all()` — `main.rs:263` → probes `$PATH` for all four CLIs (`src/app.rs:88-96`, `path_detect::detect` uses `std::env::var_os("PATH")` at `src/path_detect.rs:22`). Nondeterministic across machines → affects tab grey-out rendering.

**Not entangling** (safe under kittest's headless `egui::Context`):
- `ctx.copy_text(...)` at `main.rs:1033, 1039, 1043` — egui platform-output; harmless headless.
- `ui.ctx().open_url(...)` at `main.rs:550, 726` (and `src/form/widget.rs:66`) — egui output; harmless.
- No file dialogs and no clipboard crate: `grep` for `rfd|FileDialog|arboard|clipboard`/`Clipboard` finds **no** dependency — the only "clipboard" hits are comments (`main.rs:958, 1128`) and `ctx.copy_text`. UNVERIFIED-negative confirmed by absence in `Cargo.toml`.

## 4. The `gui`-feature-split precedent + existing test harness

**Feature split** (`Cargo.toml:14-40`): `default = ["gui"]`; `gui = ["dep:eframe", "dep:egui"]` (`:21-22`). The `mnemonic-gui` bin has `required-features = ["gui"]` (`:29-32`) — so `main.rs` (and thus `MnemonicGuiApp`) is compiled **only** when `gui` is on, but note it is gated at the **bin level**, not with in-file `#[cfg]` attributes. A second, ungated bin `gui-render` (`src/bin/gui_render.rs`) drives the egui-free emit path (`Cargo.toml:38-40`).

**Module gating** (`src/form/mod.rs`): unconditional egui-free modules `conditional, fixtures, flag_defaults, invocation, mode_predicates, render_emit, secret_model, slot_model, tree_model` (`:13-21`); **`#[cfg(feature = "gui")]`** modules `archetype_form, secret_widget, slot_editor, tree_form, widget` (`:24-33`). `src/lib.rs` re-exports `app, form, help, path_detect, persistence, platform, runner, schema, schema_check, secrets` (all `pub mod`, ungated — `lib.rs:7-16`).

**egui_kittest wiring** (`Cargo.toml:112`): `egui_kittest = { version = "0.31", features = ["wgpu", "snapshot"] }` — **the `eframe` feature is NOT enabled** (relevant to §5). Pinned to 0.31.1 (`Cargo.lock:1324-1326`).

**How tests build harnesses today — form-only, never the window.** Every kittest test uses a **`Ui`-scoped** closure: `Harness::new_ui_state(FnMut(&mut Ui, &mut FormState), state)` or `Harness::builder()...build_ui_state(...)`. I enumerated all 50+ construction sites; **zero** use a Context-level closure and **zero** construct `MnemonicGuiApp`. Key files:
- `tests/ui_harness/mod.rs` — the shared engine. `render_flag_harness` (`:183-195`) and `render_whole_form_harness` (`:458-469`) both call `Harness::new_ui_state`. `render_whole_form` (`:419-454`) re-implements `main.rs`'s per-flag loop **as a copy** (it explicitly mirrors "`src/main.rs:601-686`" and omits the SlotEditor/tree/archetype sub-surfaces — see the doc at `:405-418`). Positionals + action bar helpers `:487-536` are a **hand-copied mirror** of `main.rs:832` / `main.rs:1023` (`render_action_bar` just does `ui.add_enabled(true, Button::new("Run"))`, `:534-536`).
- `tests/gui_form_snapshots.rs` — the **existing whole-*form* screenshot suite** (61 forms). `snapshot_harness` (`:76-90`) uses `Harness::builder().with_pixels_per_point(2.0).build_ui_state(|ui, state| { render_whole_form; render_positionals; render_action_bar }, base)` then `harness.fit_contents()` (`:88`). Snapshots via `harness.try_snapshot_options(&name, &opts)` (`:174`), default dify threshold 0.6 (`:155`), corpus `tests/snapshots/forms/<tab>-<sub>.png` (`:68`). Env-gated on `GUI_SNAPSHOTS=1` (`:109`), with a CPU-adapter guard (`:126-133`). **This is the closest precedent to the goal, but it renders form crops in a `Ui` closure, NOT the tab bar / output panel / real panels.**
- `tests/ui_harness_i4_realcli.rs` — the real-CLI functional cells: assembles argv via `invocation::assemble_argv` (`:127`) from a driven harness `h.state()`, then calls `runner::run(argv)` directly (`:141`); binary resolved from `MNEMONIC_BIN`/`MD_BIN`/`MS_BIN`/`MK_BIN` on `$PATH`. It bypasses the app entirely.
- Corroboration that this is a known gap: `design/agent-reports/gui-v0-35-0-persistence-wiring-r0-round1-review.md:31` states "`MnemonicGuiApp` is bin-private (verified: no test constructs it; kittest cells drive closures — tests/widget_interaction.rs:241)".

## 5. The minimal extraction to drive the REAL window

**Verified capability**: egui_kittest 0.31.1 *can* step a plain Context closure — `Harness::new_state(FnMut(&egui::Context, &mut State), state)` (`egui_kittest-0.31.1/src/lib.rs:166`), `Harness::new(FnMut(&egui::Context))` (`lib.rs:465`), and builder equivalents `build_state` (`builder.rs:113`) / `build` (`builder.rs:193`). These call the closure to render into a real `egui::Context` with panels (see `AppKind::Context`/`ContextState` in `src/app_kind.rs:32-41`). It **also** has a native eframe path: `HarnessBuilder::build_eframe(FnOnce(&mut CreationContext)->State) where State: eframe::App` (`builder.rs:150-171`), which constructs a headless `eframe::CreationContext::_new_kittest` (verified present: `eframe-0.31.1/src/epi.rs:115`) + `eframe::Frame::_new_kittest` (`epi.rs:679`) and steps `app.update(ctx, frame)` (`app_kind.rs:42-47`) — but this requires enabling egui_kittest's `eframe` feature, which is currently off.

**Recommended minimal extraction (Context-closure strategy — avoids the eframe feature and `CreationContext` entirely):**

1. **Move `MnemonicGuiApp` out of the bin into the (gui-gated) library.** Lift the struct (`main.rs:98-151`), `impl MnemonicGuiApp` (`:153-376`), `impl eframe::App` (`:383-1178`), plus `AUTOSAVE_INTERVAL` (`:381`), `render_exit_badge` (`:1194-1215`), and `spawn_and_capture` (`:1220-1260`) into e.g. `src/app_window.rs` (or extend `src/app.rs`) wrapped in `#[cfg(feature = "gui")]` (it uses egui + the gui-gated `widget/slot_editor/tree_form/archetype_form/secret_widget` modules). Re-export from `main.rs`. This alone makes it reachable as `mnemonic_gui::…::MnemonicGuiApp`.

2. **Split the constructor.** Introduce a pure constructor that takes the already-resolved `(loaded: Option<PersistedState>, state_path: Option<PathBuf>)` **plus an injectable `AppState`**, and does the field init only (`main.rs:258-335`) — **excluding** the three OS side-effects (window-capture `:165-175`, keepalive thread `:195-202`, signal/ctrlc handlers `:217-256`). The existing `new(cc, loaded, state_path)` becomes a thin wrapper that runs those cc-dependent effects and then calls the pure constructor. Tests call the pure one.

3. **Extract the update body into an eframe-free method.** Add `fn ui(&mut self, ctx: &egui::Context)` containing today's `update` body verbatim (it already never touches `_frame`), and reduce `impl eframe::App::update` to `self.ui(ctx)`. Tests then drive `Harness::builder().with_size([920.0,720.0]).build_state(|ctx, app| app.ui(ctx), app_instance)` (default window size seed `[920.0,720.0]` is at `main.rs:52`).

4. **Seams for determinism (screenshot goal):**
   - *AppState injection* — construct with a fixed `AppState` (all four `Detected::Found` or a chosen mix) instead of `detect_all()` (`main.rs:263`), so tab grey-out is host-independent. `AppState` fields are `pub` (`src/app.rs:64-70`).
   - *Disk isolation* — pass `state_path = None` (already a param, `main.rs:157`); disables load/autosave/on_exit save.
   - *Output panel content* — pre-seed `app.last_run = Some(runner::RunResult{ argv, mask, exit_code, stdout, stderr })` (all fields `pub`, `src/runner.rs:19-33`) and/or `app.last_run_error`. This renders a populated output panel **without any subprocess** and needs no runner seam.

5. **Runner seam (only if scripting a live Run click).** `spawn_and_capture` (`main.rs:1220`) hard-calls the free fns `path_detect::detect` (`:1232`) and `runner::run_with_stdin` (`:1247`) — there is **no injection point**. For a scripted "click Run → observe output" test you must either put a real CLI on `$PATH` (I4-style, nondeterministic for pixels) or add a seam (e.g. a boxed runner fn on the struct). For pure screenshots, prefer pre-seeding `last_run` (item 4) and skip this.

**Genuinely hard bits** (why not just `build_eframe`): the real `new(cc)` fuses the signal-handler install (`main.rs:217-256`) and the forever keepalive thread (`:195-202`) with state init. If you drive the real `impl eframe::App` via `build_eframe`, you'd run those per-harness unless the constructor is split first — so the split in step 2 is mandatory either way. Given that, the Context-closure path (steps 1-3) is strictly less risky than enabling egui_kittest's `eframe` feature (which adds an eframe-feature/version integration surface). The `update` body's eframe-independence (verified) is what makes step 3 clean.

## 6. Output-panel content source + the run-confirm modal position

**Output panel content**: driven entirely by `self.last_run: Option<runner::RunResult>` (`main.rs:105`) and `self.last_run_error: Option<String>` (`main.rs:106`). `RunResult` fields consumed by the panel: `argv: Vec<String>`, `mask: Vec<bool>`, `exit_code: Option<i32>`, `stdout: Vec<u8>`, `stderr: Vec<u8>` (`src/runner.rs:19-33`), rendered at `main.rs:473-505`. Toggles `show_cmdline/show_stdout/show_stderr` gate the three sections. **Seeding `last_run` is sufficient to render a full output panel deterministically** (RunResult has a `Drop`/`Zeroize` impl at `runner.rs:48-61`, but is freely constructible with public fields).

**Run-confirm (secret-redaction) modal — YES, it sits between Run and execution, but conditionally:**
- Gate: `let needs_confirm = secrets::should_confirm_run(sub, state)` at `main.rs:954`. `should_confirm_run` (`src/secrets.rs:215-252`) returns true iff any secret-class flag/slot/positional/composite has a non-empty value.
- On Run click (`main.rs:1046-1056`): if `needs_confirm`, it sets `self.pending_confirm_argv = Some(PendingConfirm{argv, mask, stdin})` (opens the modal) and does **not** execute; else it calls `spawn_and_capture` immediately.
- The modal itself: `egui::Window::new("Confirm secret-bearing run")` at `main.rs:1081-1124`; its own **`Run`** button (`:1108`) calls `spawn_and_capture`, **`Cancel`** (`:1117`) clears the pending state.
- **Implication for scripted drive**: a **non-secret** form runs on one Run click; a **secret-bearing** form requires **two** clicks (form `Run` at `:1023` → modal `Run` at `:1108`). A driver must query the modal by title/label to click through.
- The separate **paste-warn modal** `egui::Window::new("Secret paste warning")` (`main.rs:1130-1142`, flag `pending_paste_warn` set from a ctx-data bus at `:1063-1068`) is informational/non-blocking and independent of the Run path.

---

## Extraction difficulty verdict: **MODERATE**

The update loop is architecturally clean for headless drive (no `eframe::Frame` dependency, no clipboard/file-dialog crates, no in-`update` viewport/repaint commands), and egui_kittest natively supports the Context-closure panels path — so a faithful whole-window harness is achievable. It is not CLEAN only because the App lives in a bin and its constructor fuses state init with process-global side-effects that must be surgically separated first.

**Top 3 risks:**
1. **App + update loop are bin-private in `main.rs`** (`main.rs:98`, `:383`), unreachable from `tests/`. Must be lifted into the `#[cfg(feature="gui")]` library (~1080 contiguous lines: struct `:98-151`, impls `:153-1178`, helpers `:1194-1260`). Mechanical but sizable, and it re-homes a 62KB file's core.
2. **`new(cc, …)` fuses pure state-init with three OS side-effects** — the SIGINT/SIGTERM/ctrlc handler install (`main.rs:217-256`, global process state that would hijack the test runner and `ctrlc::set_handler` errs if called twice) and the forever keepalive thread (`:195-202`, per-harness thread leak), plus `cc.window_handle()` capture (`:166-168`). A test-safe constructor that omits all three is required; this is the load-bearing refactor.
3. **Nondeterminism + no runner seam.** `AppState::detect_all()` (`main.rs:263`) makes tab grey-out depend on host `$PATH`, and `spawn_and_capture` hard-wires `runner::run_with_stdin` (`main.rs:1247`) with no injection point. Deterministic whole-window PNGs require injecting a fixed `AppState` and pre-seeding `last_run`; any test that scripts a *live* Run click inherits real-subprocess nondeterminism (and, for secret forms, the two-stage confirm modal at `main.rs:1081-1124`).
