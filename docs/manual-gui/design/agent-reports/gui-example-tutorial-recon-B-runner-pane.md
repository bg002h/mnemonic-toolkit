# gui_example.pdf cycle — RECON report B (runner + populated output pane)

> Provenance: recon subagent report, persisted verbatim per house convention.
> Cycle: `gui_example_tutorial` (spec: `docs/manual-gui/design/SPEC_gui_example_tutorial.md`).
> Dispatched 2026-07-02 by the RECON+SPEC author. Sources verified against
> mnemonic-gui `master@0d4429d` (v0.55.0) working tree.
> Delivery note: the agent's first final-report emission was lost to an API
> stream timeout; the agent was resumed and re-emitted this report from intact
> context (no re-investigation). Flagged per the dispatch-failure convention.

---

# RECON B — mnemonic-gui: can egui_kittest drive form → Run → real subprocess → settle → snapshot a GENUINELY POPULATED output pane, deterministically?

Repo is at `/scratch/code/shibboleth/mnemonic-gui` (cwd was `mnemonic-toolkit`; I investigated the gui repo).

## Section 1 — subprocess spawn + completion path
The runner is **fully synchronous + blocking**. No tokio, no thread, no channel, no `poll_promise`, no `Arc<Mutex>`.
- `src/runner.rs:172` `run_with_stdin` → `:197` `Command::new(...)` (`std::process::Command`) → `:208` `.spawn()` → `:228` `child.wait_with_output()?` (blocks; drains stdout+stderr in parallel). `:147` `run` delegates with `stdin=None`.
- GUI caller: `src/main.rs:1220` `spawn_and_capture` → `:1247` `runner::run_with_stdin(argv, stdin)` **inline**. Ok → `app.last_run = Some(result)` (`:1252`); Err → `app.last_run_error` (`:1257`). This executes **inside `MnemonicGuiApp::update()`** (impl at `src/main.rs:383`).
- Consequence: completion lands in the **same frame** as the Run/confirm click. No cross-frame drain, no channel recv in `update()`, no polled promise. Completion does **not** require multiple `update()` calls.
- `request_repaint`: the only one is a 1 Hz Wayland keepalive from a background thread (`src/main.rs:196-201`). There is **no** repaint tied to run-start or run-completion (none needed). No "running"/spinner state is rendered; the UI simply blocks during the subprocess.

## Section 2 — output pane state + render; full content enumeration
State: `MnemonicGuiApp.last_run: Option<runner::RunResult>` (`src/main.rs:105`) + `last_run_error: Option<String>` (`:106`). `RunResult` = argv: Vec<String>, mask: Vec<bool>, exit_code: Option<i32>, stdout: Vec<u8>, stderr: Vec<u8> (`src/runner.rs:19-33`). Written same-frame as completion (§1).
Render: `src/main.rs:456-509` (`TopBottomPanel::bottom("output")`). Every pane element, classified:
1. Three checkboxes "show command-line"/"show stdout"/"show stderr" (`:458-460`). Labels **deterministic**. Checked-state from persisted toggles; with **no** persisted file all default **true** (`src/main.rs:271`) → a fresh app shows argv+exit+stdout+stderr.
2. `last_run_error` colored_label "subprocess error: {err}" (`:471`) — **absent on success**; {err} is OS-nondeterministic but unreached for exit-0.
3. `argv:` line, only if `show_cmdline` (`:478-485`): `render_copy_command_masked(&result.argv, &result.mask, Posix)` (`src/form/invocation.rs:524`); secret tokens → fixed "••••" (`invocation.rs:137`). **Deterministic** except argv[0] (§6).
4. Exit badge (`:487` → `render_exit_badge` `src/main.rs:1194`): "exit: 0" | green "exit: 5 — Repair Applied (BCH auto-fire…)" | "exit: {n}" | "exit: (killed)". **Deterministic** (exit is a pure function of a fixed input).
5. stdout block if `show_stdout && !empty` (`:488-495`): "stdout:" + `ScrollArea::vertical().id_salt("stdout").max_height(180.0)` + `ui.monospace(String::from_utf8_lossy(stdout))`. **Deterministic for decode/inspect vectors** (pure function of input, no RNG/timestamp); nondeterministic for any generate/RNG/timestamp subcommand.
6. stderr block if `show_stderr && !empty` (`:497-505`): "stderr:" + `ScrollArea id_salt "stderr" max_height 120` + monospace. Decode warnings/notes; deterministic for fixed vectors (often empty).
7. else "(no run yet)" (`:507`).
**Absent content (grep-verified across main.rs):** no timestamps, no durations/"took 1.2s", no spinner frames, no PID, no elapsed/Instant in the pane (the only Instant/Duration are the unrelated autosave timer at `:141/:331/:381/:411/:420`). The pane is essentially timing-free.
Truncation/wrapping: no truncation; ScrollArea clips to `max_height` (180/120 pts), so a long stdout renders only its first visible slice in a snapshot; monospace wraps at egui default. Clipping is deterministic for fixed content.

## Section 3 — run-confirm modal / secret redaction
A confirm step exists **between Run click and spawn, but only when secrets are present**.
- Gate: `src/main.rs:954` `needs_confirm = secrets::should_confirm_run(sub, state)`. On Run click (`:1046`): needs_confirm → stash `pending_confirm_argv = Some(PendingConfirm{argv,mask,stdin})` (`:1048`) and defer; else `spawn_and_capture(...)` immediately (`:1054`).
- `should_confirm_run` (`src/secrets.rs:215`): true iff any secret-class flag has a value, or a secret slot row, or a secret positional widget, or a secret NodeValueComposite. So a public-vector decode with a **secret positional** still confirms (e.g. `ms decode`'s `ms1` positional is `secret: true`).
- Modal (`src/main.rs:1081-1124`): `egui::Window::new("Confirm secret-bearing run")`, label `secrets::RUN_CONFIRM_MODAL_PREFIX` ("This invocation passes secret-bearing arguments to " — `src/secrets.rs:200`), "Argv:" then each token masked by its mask bit (`:1098-1105`). Buttons "Run" (`:1108` → clears pending, `spawn_and_capture` with cloned argv/mask/stdin) and "Cancel" (`:1117`).
- Test click-through: target "Run" inside the window, click, run(). Simplest deterministic cell avoids the modal by using a **non-secret** subcommand (`mnemonic decode-address`, `md decode`, `mk decode`).

## Section 4 — existing I4 real-CLI cells
File: `tests/ui_harness_i4_realcli.rs`. They prove subprocess execution but **do not render**.
- Invocation: **not** through the app update loop. They drive only `--json` through the P1 widget harness (`render_flag_harness` + `drive`, `:123-124`), call the GUI's own `assemble_argv` (`:127`), overwrite `argv[0] = bin` (`:140`), and call `runner::run(argv)` **directly** (`:141`). No MnemonicGuiApp, no output pane, no `update()`.
- Wait for completion: none — `runner::run` blocks to completion. No busy loop, no timeout.
- Assertions: `exit_code == Some(0)` (`:150`), stdout parses as JSON (`:162`), per-cell field checks (mnemonic `:180`; md `:204`; ms `:226`; mk `:259`). All fields are pure functions of fixed **public** vectors (`:52-72`); explicitly no RNG/timestamp (`:24-29`).
- Pinned binary: env `MNEMONIC_BIN`/`MD_BIN`/`MS_BIN`/`MK_BIN` via `pinned_bin()` (`:83`); unset → **early-return-skip** (`:105-112`), never `#[ignore]`.
- CI provisioning: `.github/workflows/schema-mirror.yml` `cargo install --locked --git … --tag …` for all four CLIs (`:59-88`), then `cargo test --workspace` with `MNEMONIC_BIN=mnemonic` / `MD_BIN=md` / `MS_BIN=ms` / `MK_BIN=mk` (`schema-mirror.yml:127-133`) — **bare names resolved via $PATH**. Pins: mnemonic-toolkit-v0.74.0, descriptor-mnemonic-md-cli-v0.11.0, ms-cli-v0.13.0, mk-cli-v0.11.0 (`pinned-upstream.toml`).
- `runner_integration.rs` mirrors the pattern (assemble_argv → `argv[0]=MNEMONIC_BIN` → `runner::run`, `:93-102`); also non-rendering.

## Section 5 — marrying them into a populated-pane render
**The blocker:** the entire pane render (`src/main.rs:456-509`), `spawn_and_capture` (`:1220`), `render_exit_badge` (`:1194`), and `MnemonicGuiApp` itself all live in `src/main.rs` — the **binary** crate. Integration tests link the **library** crate `mnemonic_gui`, whose root (`src/lib.rs:7-16`) exports app/form/help/path_detect/persistence/platform/runner/schema/schema_check/secrets — **not `main`**. Tests cannot name `MnemonicGuiApp`, call `update()`, or reach the pane. No test uses `new_eframe`/`build_eframe`/`eframe::App` (grep: 0 hits in `tests/`; the only `impl eframe::App` is `src/main.rs:383`). Every existing harness is `Harness::new_ui_state`/`build_ui_state` over a `FormState` closure (`tests/ui_harness/mod.rs:189, :463`; `tests/gui_form_snapshots.rs:78-90`) mirroring fragments of the update loop. The action-bar mirror `render_action_bar` (`tests/ui_harness/mod.rs:534-536`) renders a **dead** `[Run]` button — `ui.add_enabled(true, Button::new("Run"))` with **no `.clicked()` and no spawn**. There is **no output-pane mirror** anywhere.

Two recipes:
- **(A) Real production `update()`** via `Harness::builder().build_eframe(|cc| MnemonicGuiApp::new(cc,…))`: **BLOCKED today** — MnemonicGuiApp is binary-private, and `new()` (`src/main.rs:154`) calls `cc.window_handle()` for capture-protection (`:166-174`; offscreen kittest has no window → warn/degrade), spawns a keepalive thread (`:196-202`), installs SIGINT/SIGTERM handlers (`:217-251`). Requires lifting MnemonicGuiApp into the lib + guarding those side-effects.
- **(B) A new mirror harness** (the sanctioned pattern — how `render_whole_form` mirrors the form loop): a `build_ui_state` closure whose State carries `Option<runner::RunResult>` (`RunResult` is public: `mnemonic_gui::runner`) and re-mirrors `main.rs:456-509`. Because the runner is **synchronous**: call `runner::run(argv)` (blocks), store the RunResult in state, then a **single `harness.run()`** renders a genuinely populated pane. kittest's run()/step() are more than sufficient — no channel to drain, no multi-frame settle, **no thread/channel timing to flake**. (Caveat: the mirrored pane render could drift from `main.rs`; there is no faithfulness guard for the pane like there is for the form grid.)

Normalization for recipe B: argv[0] echo (§6); restrict to decode/inspect vectors so stdout/exit are pure functions of input. No timing strings to scrub.

## Section 6 — cwd / env / paths
- **cwd:** runner sets no `.current_dir()` (grep: 0 hits in `src/runner.rs`) → inherits the test process cwd. No pane content derives from cwd for decode vectors.
- **env:** only `.env("MNEMONIC_FORCE_TTY", "1")` (`src/runner.rs:199`); no `env_clear`/`env_remove` → inherits parent env. Deterministic.
- **temp files:** runner writes none. Tree-mode pipes spec JSON via stdin (`--spec -`), not a temp path (`src/main.rs:928-943`).
- **binary path echo (the one real normalization target):** production `assemble_argv` sets `argv[0] = schema.cli_name`, a **bare name**, "No absolute path" (`src/form/invocation.rs:115-116, :159`) → production pane echoes "mnemonic decode-address …" (deterministic). BUT the I4/runner_integration pattern **overwrites** `argv[0]` with the pinned bin (`ui_harness_i4_realcli.rs:140`; `runner_integration.rs:100`). If a render harness reuses that, the `argv:` line shows `MNEMONIC_BIN` verbatim: bare "mnemonic" under CI (`schema-mirror.yml:129`) = deterministic; an absolute `$HOME/.cargo/bin/mnemonic` locally = **host-specific**. Normalize argv[0] (or force a bare name) before snapshotting.

## VERDICT: FEASIBLE-WITH-NORMALIZATION (via a new pane-mirror harness)
Driving the **real production `update()`** pane is **BLOCKED as-is** (MnemonicGuiApp is binary-private; its `new()` needs a window handle and spawns threads/signal handlers; the existing harness Run button is a dead mirror that never spawns). But a genuinely-populated pane **is** feasible via the repo's established mirror-harness pattern:

**Drive recipe:** `Harness::builder().build_ui_state(closure, State{last_run: Option<RunResult>})` where the closure re-mirrors `src/main.rs:456-509`; drive the form + `--json` exactly like the I4 cells; call `runner::run(argv)` (synchronous, blocks to completion); assign the RunResult into state; call `harness.run()` once; snapshot. Because completion is in-frame (no channel/thread/poll), the settle is single-pass and non-flaky.

**Strings to normalize:**
1. `argv[0]` in the `argv:` echo — force/rewrite to a bare "mnemonic"/"md"/"ms"/"mk" (else absolute local paths differ from CI's bare names). The only genuinely host-specific token.
2. Scope inputs to the four I4 decode vectors (no RNG/timestamp) so stdout/exit are pure functions of fixed input.
No duration/timestamp/PID/spinner strings exist to scrub; env/cwd/temp-path leakage: none for decode vectors.

**Gotchas:** (a) a secret-positional subcommand (`ms decode`) trips the confirm modal — prefer non-secret `decode-address`/`md`/`mk` cells or click the modal's "Run"; (b) the mirrored pane render must be kept in sync with `main.rs:456-509` (no faithfulness guard exists for the pane); (c) `ScrollArea max_height` clips long stdout in the snapshot (deterministic but partial).

---

> Synthesis note by the spec author (not part of the verbatim report): recipe (A)
> "BLOCKED today" is *unblocked by the app-shell extraction that recon A maps out*
> — the two reports compose: after the A-extraction, the real `update()`/`ui()`
> path is drivable and recipe (B)'s mirror becomes the fallback, not the primary.
> The spec adopts A-extraction + real Run-click drive as primary, with (B) and
> seeded-`last_run` as spike fallbacks.
