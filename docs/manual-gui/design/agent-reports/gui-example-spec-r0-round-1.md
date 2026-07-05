# R0 review — SPEC_gui_example_tutorial.md — round 1

- **Reviewer:** opus-tier architect (adversarial R0, spec gate).
- **Date:** 2026-07-04.
- **Artifacts reviewed:** `docs/manual-gui/design/SPEC_gui_example_tutorial.md` (full), all four recon reports
  (`gui-example-tutorial-recon-{A,B,C,DE}.md`, full), with load-bearing citations re-verified against
  mnemonic-gui `master@0d4429d` and toolkit `master@c621a81a` (= spec's `4c401b16` + the spec commit itself)
  working trees, plus the egui_kittest 0.31.1 registry source and the v0.74.0/v0.75.0 toolkit tags.

## Verdict: **RED — 0 Critical / 3 Important** (+4 Minor)

No Critical findings: every fact the user-locked contract rests on verified clean against source. The three
Importants are contract-hardening and ship-mechanics gaps — all cheap folds, none re-opens a locked decision.
Fold → re-dispatch per house rule.

---

## Explicit verification results (the facts everything hangs on)

### V1. Synchronous-runner claim (recon B) — **VERIFIED, exactly as claimed**

- `runner::run_with_stdin` (`src/runner.rs:172`) → `Command::new` (`:197`) + `.env("MNEMONIC_FORCE_TTY","1")`
  (`:199`) → `.spawn()` (`:208`) → `child.wait_with_output()` (`:228`). Fully blocking; grep confirms no
  tokio/thread/channel/poll in the run path; runner sets no `current_dir` (inherits cwd).
- `spawn_and_capture` (`src/main.rs:1220`) calls `runner::run_with_stdin(argv, stdin)` **inline** and assigns
  `app.last_run = Some(result)` / `last_run_error` in the same call — and it executes inside `update()`
  (Run click at `main.rs:~1054`; modal Run at `~1110`). Completion therefore lands in `last_run` in the
  **same frame** as the click. Confirmed timing-free pane: no timestamp/duration/PID/spinner anywhere in
  `main.rs:456-509`; the only Instant/Duration is the unrelated autosave timer.
- Argv echo deterministic: `assemble_argv` sets `argv[0] = schema.cli_name`, bare name, "No absolute path"
  (`src/form/invocation.rs:115-116, :159`); mask sentinel `SECRET_MASK = "••••"` (`:137`). The click path
  never rewrites argv[0] (the I4 overwrite pattern is test-only, and the spec correctly excludes it, §6.4).
- One verified nuance the spec doesn't state: `spawn_and_capture` **pre-probes the real `$PATH`**
  (`path_detect::detect(bin)`, `main.rs:~1232`) before spawning, independent of the injected `AppState` —
  the pinned CLI must genuinely be on `$PATH` at click time (it is, per the tutorial-snapshots job design).
  Feeds finding **I1**.
- `RunResult` zeroize-on-drop verified (`runner.rs:49-59`); `PendingConfirm` likewise (`:85-96`).

### V2. App-shell extraction map (recon A) — **VERIFIED, behavior-preserving as mapped**

- `struct MnemonicGuiApp` bin-private at `main.rs:98`; `impl eframe::App` at `:383`;
  `fn update(&mut self, ctx, _frame)` at `:384` — `_frame` appears **only** in the signature across the
  entire impl body (grep over `:384-1180`: zero uses). The update body is `Frame`-free as claimed.
- All OS side-effects confirmed in `main()`/`new(cc,…)` at the cited lines: window-capture protection via
  `cc.window_handle()` (`:166-174`, already warn-and-degrade on Err), 1 Hz keepalive thread (`:195-202`),
  `#[cfg(unix)]` signal-hook SIGINT/SIGTERM + `#[cfg(windows)]` ctrlc (`:217-256`), `AppState::detect_all()`
  (`:263`). The wrapper order the spec prescribes (three cc-effects → `detect_all` → pure init) matches
  today's execution order — the split is genuinely order-preserving.
- `main.rs` already references the library by path (`mnemonic_gui::platform::…`, `mnemonic_gui::form::…`) —
  the lift is mechanical relocation, not a module-graph untangle. The gui-feature gating precedent exists
  (`src/form/mod.rs` gates `widget/slot_editor/…` on `feature = "gui"`); `--no-default-features` is named as
  an explicit per-phase check (the right lesson from the form-renders cycle).
- egui_kittest 0.31.1 registry source confirms the Context-closure fixed-size path:
  `HarnessBuilder::with_size` (`builder.rs:32`), `with_pixels_per_point` (`:41`), `build_state` (`:113`);
  `Cargo.toml:112` confirms features `["wgpu","snapshot"]` — no `eframe` feature, so the spec's
  no-`build_eframe` strategy is consistent with the dependency surface.
- Riskiest seam assessment: the **constructor split** is the load-bearing refactor (signal handlers are
  process-global and `ctrlc::set_handler` cannot install twice; keepalive is a forever-thread per instance)
  — the spec correctly makes `new_headless` omit all three. Second seam: `impl eframe::App` also carries
  `on_exit` (`main.rs:~1145`, geometry snapshot + `scrub_app_run_holders`) — lifted wholesale it stays in
  the eframe impl and kittest never calls it; harmless (RunResult scrubs on drop regardless). No finding.
- Recon B's "recipe A BLOCKED" ↔ recon A composition note is honest and correct: blocked *today*, unblocked
  by the A-extraction; the spec adopts the composed path with (B)/seeded-`last_run` demoted to spike
  fallbacks that only a USER decision may adopt (§4 STOP). Locked contract honored.

### V3. Two-click secret-confirm drive — **VERIFIED + spec'd + taught**

`needs_confirm = secrets::should_confirm_run(sub, state)` at `main.rs:954` (fn at `secrets.rs:215`); Run
click either stashes `PendingConfirm` (`:~1048`) or spawns immediately (`:~1054`); modal
`egui::Window::new("Confirm secret-bearing run")` at `:~1082` with its own Run (`:~1110`) → `spawn_and_capture`.
The pending flag is set inside the CentralPanel closure and the modal renders later **in the same update
pass**, so the `-modal` shot is capturable on the click frame. The spec drives it (S2(ii), §3.1(b)) AND
ships it as tutorial content — J1 `-modal` shot + J2 all-seeds `-modal` shot (§5.3), with the §7(b)
no-plaintext assertion over the modal token list before capture. The teaching-moment box is checked.

### V4. Hygiene taxonomies (recon C) — **VERIFIED, pointed at the right lists**

- `SECRET_SLOT_SUBKEYS = ["phrase","seedqr","entropy","ms1","xprv","wif"]` (`secrets.rs:69-70` canonical
  fallback, toolkit-imported at `:35` with compile-time drift pins). `xpub`/`mk1` are NOT members → a
  cosigner/public slot row renders plain by construction (`slot_editor.rs:46-52` gates `.password(true)` on
  `row.subkey.is_secret_bearing()`); persist-layer DROPS secret slot rows (`persistence.rs:~114`); composite
  `phrase=` masking keys on `node_type_is_argv_secret` (`widget.rs:604-606`). The spec's §7 correctly rides
  these two taxonomies and explicitly disclaims the I3 64-flag census — matching recon C's precision note.
- Demo-data-only rule is **machine-asserted** (§7 bullet 1: driver allowlist assertion, fail-loud), not
  prose. Seed-phrase publication verified in `Examples.md` at the cited lines (209/215, 279/312, 284/329,
  1491-1493 — all three world-known vectors) and `gen.sh:26-28` literals.
- Committed-artifact leak channels checked: transcripts persist `stdout/stderr/exit` only — **no argv
  file**, so the unmasked in-memory argv never lands in either repo; the masked pane/modal renders are
  assertion-guarded; derived secret material in transcripts (e.g. J1's ms1 card for S0) is exactly what
  Examples.pdf already prints and §7(c) states this. Clean.

### V5. Book/gates/pins (recon D/E) — **VERIFIED with two ship-mechanics gaps (→ I3)**

- `gen.sh:22` hard version assert (`[ "$VER" = "mnemonic 0.55.3" ] || FATAL`) — the spec's cite is correct
  (recon DE's `:25` was off by three; spec-side re-verification evidently fixed it — good discipline).
- lint.sh is 9 phases (`step "1/9"…"9/9"` verified); phases 1-3 are `$SRC_DIR`-scoped, so the spec's
  "verify globs" note for extending them to `tutorial/` is right, AND phases 4-9 will not accidentally
  sweep the new dir. `verify-figures-gui` at `lint.sh:204+` as cited.
- Pins verified: toolkit `docs/manual-gui/pinned-upstream.toml:33` = `mnemonic-gui-v0.55.0`;
  mnemonic-gui `pinned-upstream.toml` = toolkit v0.74.0 + md/ms/mk-cli v0.11.0/v0.13.0/v0.11.0.
  `schema-mirror.yml:59-64` (pinned install) + `:127-133` (bare-name env) as cited; `build.yml:87-147`
  snapshots recipe as cited.
- **P1 pin-bump delta v0.74.0→v0.75.0 — verified near-zero, pre-scoped now:** the only
  `crates/mnemonic-toolkit/src` changes between the tags are `cmd/inspect.rs` + `error.rs` (the md1
  `template:` line + INSPECT_SCHEMA_VERSION 1→2). **No clap-surface change → the schema_mirror flag-name
  delta is ZERO**; the sibling mirrors (`archetype_schema_mirror`, `spec_nodes_mirror` [pins
  spec_schema_version==1, untouched], `xpub_search_schema_mirror`, …) are likewise unaffected. The GUI's
  real-CLI cells don't consume `inspect --json` (i4-mnemonic drives `decode-address`), so the wire-shape
  bump is inert for GUI tests. The mandatory catch-up is exactly: the 1 pin line + the
  `pinned_version: "mnemonic 0.74.0"` schema string (`schema/mnemonic.rs:4620`). That string renders ONLY
  in the real window (`main.rs:518`) — NOT in the 61-form corpus or `.gui` renders — so the bump does
  **not** invalidate the existing corpora, and §3.2(e)'s "one pin anchors three corpora" claim is coherent.
  Recommend the spec state this expected delta (see M4) so P1 doesn't rediscover it.
- Examples.pdf disclaimers (§2 version-skew, §10 non-goals, FOLLOWUP `examples-pdf-un-ci-gated`) are honest
  and sufficient; the un-CI'd/manual nature of the Examples model and the decision NOT to inherit it are
  correctly load-bearing in the §5.2 placement rationale.
- Chapter math re-checked: steps 1+1+7+5+8+3 = 25; shots 1+3+15+10+16+6 = 51 (each journey = 2×steps
  + modals where marked). Correct.

### V6. Spike coverage (S1-S5) — **adequate; STOP condition honors the locked contract**

Every unknown that could invalidate the user-locked contract is spiked with a measurable exit:
whole-window determinism incl. cross-backend (S1), live-click populated pane + modal drive + masked-argv
snapshot (S2), scroll mechanics with the named guaranteed fallback `ScrollArea::vertical_scroll_offset`
(S3 — correctly framed as mechanism-ratification, not feasibility), size + corpus byte/time budget (S4),
in-window AccessKit drive incl. repeating rows + SlotEditor (S5). S1/S2 failure → STOP + user escalation
with the fallback menu, and the spec states in terms that **no downgrade may be adopted silently**. Nothing
implementation-shaped precedes the spike (P0 is a throwaway worktree; ratifications flow into a plan that
gets its own R0). Two spike-scope nits → M2.

---

## Findings

### Critical — none.

### Important

**I1. The harness lacks the `gen.sh:22`-style pinned-tier hard assert — the "Pinned:" label cannot catch a
wrong-tier binary.** The window's `Pinned: mnemonic 0.75.0` line renders a **schema constant**
(`main.rs:518` ← `schema/mnemonic.rs:4620`), not anything probed from the binary; and `spawn_and_capture`
resolves bare `argv[0]` against the **real `$PATH`** at click time. So a local regen
(`UPDATE_SNAPSHOTS`-style) against a stale/wrong `mnemonic` produces panes + transcripts from tier X under
an honest-looking 0.75.0 label — detectable only downstream, as an undiagnosed CI byte/pixel diff (or not
at all, when tiers happen to agree). The house already solved this exact problem for Examples.pdf:
`gen.sh:22` hard-asserts `mnemonic --version` before capturing anything — and the spec cites that line
twice without adopting the pattern. **Fold:** add to §6 (determinism contract) + §3.1(b): at harness start,
run `<cli> --version` for every CLI a step will spawn and hard-fail unless it matches the pinned tier
(derived from `pinned-upstream.toml` / the schema `pinned_version`); this also machine-guards the label's
honesty in every shot.

**I2. The same-frame-completion invariant is relied on but never pinned as a named assertion/tripwire.**
§3.1(b)/§6.5 cite recon B's finding as a design fact, and the per-step assertion list (exit codes, chaining,
hygiene) omits the one assertion the entire populated-pane contract rests on. If a future GUI cycle makes
the runner async (a spinner/cancel button is a plausible UX evolution), the failure surfaces as a confusing
whole-corpus pixel diff ("pane empty in 25 .new.png files") rather than a named invariant breach — and the
seeded-`last_run` "fix" someone might reach for is precisely the downgrade the user reserved for themselves.
**Fold:** (a) per-step, immediately after the click frame (before any further `harness.run()`), assert
`app.last_run.is_some()` (or `last_run_error` for the detect-fail path) with a message naming the invariant
("runner must complete in the Run-click frame — populated-pane contract, SPEC §6.5; any async-runner change
is a USER-decision downgrade"); (b) state in §6 that synchronous in-frame completion is a **pinned
invariant** of the tutorial corpus, with a pointer comment at `spawn_and_capture` so the future refactorer
finds the contract from the code side.

**I3. Ship mechanics under-enumerated: the release-attach path needs a `manual-gui-v*` tag the spec never
names, and CHANGELOG sites are unlisted.** `manual-gui.yml` is tag-triggered on `manual-gui-v*` (verified;
latest existing tag `manual-gui-v1.1.0`) — §3.2(c)/P4's "release job attaches gui_example.pdf" is dormant
until such a tag is cut, and the phase plan names only `mnemonic-gui-v0.56.0`. Separately, both
`mnemonic-gui/CHANGELOG.md` and `docs/manual-gui/CHANGELOG.md` exist and the spec touches CHANGELOG only
for the S3 scroll-seam note. Standing project experience: version-site checklists are NOT gate-enforced and
this is the historical post-tag re-cut generator. **Fold:** P2 adds the mnemonic-gui CHANGELOG entry
(incl. the scroll-seam doc if ratified); P4 names the manual-gui tag that ships the attach (e.g.
`manual-gui-v1.2.0`) — or explicitly defers the attach to the next manual-gui tag as a stated decision —
and adds the `docs/manual-gui/CHANGELOG.md` entry to the P3/P4 checklist.

### Minor

**M1. No-shot transcript runs: production path unspecified.** J2 devices-1/2 converts and the J4 NUMS
bundle/restore are "prose + transcript only", and the J3/J4 restore chains need `--json` bundle outputs
that the shot-bearing human-readable steps may not produce — so the manifest will contain runs with zero
shots. The spec should state that these are ordinary manifest steps executed through the same GUI-driven
Run path (shots: 0), so every gated transcript still satisfies "populated by a real pinned-CLI execution
in the harness" and the run-census count stays manifest-derived. One sentence in §3.1(b)/§5.3.

**M2. Spike exits should name two drives explicitly:** (i) the ComboBox **popup** subcommand selection —
every step depends on it, popups live on a separate egui layer, and S2 only *implicitly* exercises it; make
it an S2/S5 exit criterion. (ii) Chapter 0 / J1 start from the **fresh-app demo seed** (`main.rs:300-316`:
`mnemonic:bundle` pre-filled `--network=mainnet`, `--template=bip84`, `--account=0`, plus one **empty Xpub
slot row** whose subkey must be flipped to `phrase` for J1) — deterministic, but it is baseline state the
S5 SlotEditor drive and the Chapter-0 orientation shot must account for.

**M3. §6 determinism contract omits the demo seed.** Item 3 pins toggles-from-fresh-defaults but not the
demo-seeded `mnemonic:bundle` FormState, which is part of the deterministic fresh-app baseline rendered in
Chapter 0 and mutated by J1. Add it (one line), so a future change to the demo seed is understood to move
tutorial pixels.

**M4. Record the verified P1 delta + cite nits.** (a) State in §3.1(e) that the expected v0.74.0→v0.75.0
schema-mirror delta is **zero flags** (verified: only `cmd/inspect.rs` + `error.rs` changed; i4 uses
`decode-address`, not `inspect`), with the catch-up scoped to the pin line + `pinned_version` string — turning
"resolve any accumulated delta" from an open risk into a budgeted edit. (b) Cosmetic: recon DE's `gen.sh:25`
is actually `:22` (spec already correct); recon A's modal-Run `:1108` is `:~1110` at HEAD. (c) Optional:
add `docs/gui_example.pdf` to `manual-gui.yml`'s path filter, or note that PDF-only touches don't trigger
the workflow (gates never gate PDF bytes, so this is documentation, not a gate gap).

---

## Locked-decision compliance check

1. ALL Examples.pdf journeys GUI-style — §5.3 covers J1-J5 + orientation; external/Appendix-B steps are
   prose callouts per §10, consistent with recon C's NO-GUI enumeration. ✓
2. Whole-window filled screenshots, generated + gated — §1.1(2) + §3 + §8; the "no side list" recon-fidelity
   note is honest (combo, not side list — verified `main.rs:525`). ✓
3. Post-Run panes populated by real pinned-CLI runs in the harness — §1.1(3) + §3.1(b) + S2; seeded-
   `last_run` and pane-mirror exist only in the STOP fallback menu, gated on a USER decision, stated
   verbatim ("None of these may be adopted silently"). ✓

## Gate instruction

RED at 0C/3I: fold I1-I3 (M1-M4 at author's discretion but all are one-liners), persist this review, and
re-dispatch the R0 round-2 convergence review before any plan/implementation work.
