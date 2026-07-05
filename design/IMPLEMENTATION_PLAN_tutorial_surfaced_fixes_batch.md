# IMPLEMENTATION PLAN — batched cycle `tutorial_surfaced_fixes_batch`: secret-field reveal (👁) toggle + restore `--template` `(none)` affordance

**Two spec-R0-GREEN components, ONE coordinated two-tag release** (`mnemonic-gui-v0.57.0` → `manual-gui-v1.3.0`). The batch exists so the two expensive shared steps — the broad render/gallery re-pin and the tutorial corpus re-drive — each run **exactly once**.

- **Component A (reveal):** `design/SPEC_gui_secret_reveal_toggle.md` + R0 `design/agent-reports/reveal-toggle-spec-r0-round-1.md` (GREEN 0C/0I; OQ-1..5 ruled; minors M-1..M-6 folded herein).
- **Component B (restore `(none)`):** `design/SPEC_restore_template_none_affordance.md` + R0 `design/agent-reports/restore-template-spec-r0-round-1.md` (GREEN 0C/0I; minors m1–m6 folded herein).
- **Audit context:** `docs/manual-gui/design/agent-reports/tutorial-workaround-audit.md` — NO third code component. F0 secret-on-argv = MANAGED/accepted-design (do not touch); B1 file-picker = filed LOW/deferred GUI-side (`40156b0`), NOT in this batch; all other route-arounds = harness-artifact or intentional behavior.
- **Precedent mirrored:** the `gui_example.pdf` cycle (`docs/manual-gui/design/SPEC_gui_example_tutorial.md` + `IMPLEMENTATION_PLAN_gui_example_tutorial.md`; shipped `mnemonic-gui-v0.56.0` + `manual-gui-v1.2.0`). Same leg structure, same gates, same tag ordering.
- **Status:** DRAFT — awaiting mandatory opus plan-R0 (0C/0I) before ANY implementation (house hard gate). Review artifact: `design/agent-reports/tutorial-batch-plan-r0-round-N.md`.

**Source SHAs (all cites in this doc re-grep-verified at write time, 2026-07-05):**
- `mnemonic-toolkit` `origin/master @ 16bbe67b` (both specs + both spec-R0s committed; docs/manual-gui at the `manual-gui-v1.2.0` line, pin `mnemonic-gui-v0.56.0` at `docs/manual-gui/pinned-upstream.toml:55`; lint = **12 phases**, banners `lint.sh:121-373`).
- `mnemonic-gui` `origin/master @ 40156b0` = `cab940b` (`mnemonic-gui-v0.56.0`) + exactly one commit whose whole delta is `FOLLOWUPS.md +7` (`git diff --stat cab940b..40156b0` verified) — so every `src/`/`tests/` cite below is tag-identical.
- Tutorial corpus ground truth: **50 committed shots** (`tests/snapshots/tutorial/*.png` = 50; `manifest.rs:4` "25 shot-bearing steps / 50 committed shots (51 nominal; 1 modal trimmed)"). Toolkit copies: `docs/manual-gui/figures/tutorial/` = 50 PNGs, `transcripts/tutorial/` = 98 files. Gallery: 61 `.gui` + 61 PNGs; **28** `.gui` carry `<masked>` on load (`grep -rl '<masked>' docs/manual-gui/transcripts/gui/` = 28).

**Versions:** `mnemonic-gui` 0.56.0 → **0.57.0** (MINOR — reveal is a user-visible feature; restore `(none)` rides along). Manual release **`manual-gui-v1.3.0`**. **No toolkit crate bump, no crates.io publish, `docs/manual/` (non-GUI manual) untouched** — neither component changes any clap surface. The GUI's toolkit git-dep pin stays `mnemonic-toolkit-v0.75.0` (no `Cargo.toml`/`pinned_version` edit this cycle) → the GUI `schema-mirror` job runs against the identical binary and is trivially GREEN.

---

## GOTCHAS (read before every dispatch; repeat in every implementer brief)

- **Degenerate agent returns:** implementer/watcher agents have returned 0-tool-use prompt-injection-shaped results and background CI-watchers have died silently — POLL git/CI ground truth inline between steps; verify `git log`/`gh` state before trusting any agent's claim; re-dispatch on anomaly. **NO background watchers/monitors in any brief.**
- **`make html` BEFORE `make lint`** on the manual-gui leg (shipped-cycle gotcha; lychee resolves md image links file-relative and does check them).
- **Fresh-tag-clone for every census:** `MANUAL_GUI_UPSTREAM_ROOT=<fresh clone of mnemonic-gui-v0.57.0>`; the sibling-checkout default (`../mnemonic-gui`) is the `.new.png` false-RED trap. Never run the toolkit-side gates against a dirty/wrong-ref sibling.
- **Review dispatches = opus** (standing model policy: architect/review = opus; implementers/recon = session default). Reports persist VERBATIM to `design/agent-reports/` BEFORE the fold-and-commit step. **Post-impl folds RE-ENTER the review loop** (re-dispatch a scoped convergence review; never self-verify and ship).
- **Commit trailers (all commits, both repos):** `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>` + `Claude-Session: https://claude.ai/code/session_01AXCxehvWYFoCE2TDpcV2Db` (read from the live harness prompt, per standing preference — do not copy stale trailers from older plan-docs). Stage paths explicitly — no `git add -A`.
- **GUI test runs:** `cargo test --jobs 2` ALWAYS (local linker-OOM); per-phase reviews run the FULL package suite, never targeted `--test` lists. Clippy BOTH configs (`--all-targets -- -D warnings` AND the `--no-default-features` headless build). **NO fmt in either repo** (toolkit mlock.rs g6 exemption; GUI has no fmt gate — don't fmt it).
- **Ship mechanics:** GUI = PR + CI-before-tag (branch protection contexts = `[snapshots]`, enforce_admins off → release commit may direct-push); `gh pr merge` local-cleanup can fail under a worktree while the server merge succeeds — verify via `gh pr view --json state,mergeCommit`; detached-HEAD pushes via `git push origin HEAD:<branch>`. Toolkit = direct-FF permitted for the manual leg.
- **kittest drives only `Focus`/`Click`/`SetValue`** — the latch arm is the ONLY harness-drivable reveal path (binding on test design). lavapipe self-reports "llvmpipe" — assert `device_type == Cpu`, never name/driver. Embed census greps `src="data:image/png` (pandoc ≥3.2.1 reorders img attrs).
- **Single implementer subagent per phase in a worktree; TDD; no parallel re-implementations.**

---

## BINDING RULINGS (carried from the two spec-R0s + one NEW ruling this plan makes)

1. **Reveal interaction = hybrid** (OQ-1 ratified): pointer **hold-to-reveal** primary (`is_pointer_button_down_on()`); **bounded latch** for keyboard/AccessKit/kittest/tutorial-capture; both feed ONE per-frame predicate; single-revealed-field invariant = one Context-transient `Option<egui::Id>` (sibling of `paste_warn_id()`, `secret_widget.rs:40-42`), NEVER a `FormState` field. Auto-hide on: Run dispatch, field blur, window-focus-loss (`ctx.input(|i| i.focused)`), tab/subcommand switch. **No timeout v1** (OQ-4).
2. **NEW RULING (reveal-R0 M-1, delegated to this plan): a pointer TAP does NOT latch.** Pointer interactions are hold-only; the latch arms ONLY on keyboard activation or AccessKit `Click`. Mechanism: suppress the latch when the eye's `clicked()` is pointer-caused (discriminate via the frame's pointer-release input vs an `AccessKitActionRequest`/keyboard activation); add a test cell — hold→release leaves NO latch (field masked next frame and stays masked). **Bounded fallback (recorded, not a user STOP):** if the discriminator proves unreliable under kittest + real-window testing, tap-latches is accepted (it stays inside the R0-accepted §4.5-bounded latch posture); record the downgrade in the phase report and in `gui-secret-reveal-latch-timeout`.
3. **Reveal scope = sites #1/#2/#3; DEFER #4/#5** (OQ-2 option (b), R0-ratified): `secret_widget.rs:58` + `slot_editor.rs:52` (secret arm only) + `widget.rs:606` (ANDed with `is_secret_node`). Tree sites `tree_form.rs:687`/`:711` deferred — FOLLOWUP `gui-secret-reveal-tree-key-sites` (tree keys in the shipped tutorial are public xpubs, never masked; `is_xprv_like` value-conditional masking would force fixture-value coupling into the faithfulness gate). Site #3's eye is tested by a dedicated kittest cell, NOT by the faithfulness gate (same fixture-coupling reason, R0-ratified).
4. **Depict the eye in the structural render** (OQ-3): a strictly-ASCII marker on the 28 masked-on-load secret rows (e.g. extend `  --passphrase  text  (secret) -> <masked>`, `transcripts/gui/mnemonic-restore.gui:14` shape); exact marker string decided + recorded in P1.2 (determinism contract `render_emit.rs:15-18`). Faithfulness gate models the adjacent `Role::Button` on BOTH sides + a non-vacuity negative.
5. **Reveal is display-only, everywhere else masked UNCONDITIONALLY:** run-confirm modal (`secrets.rs:215-252`), argv-echo/copy-command (`assemble_argv_with_secret_mask` / `render_copy_command_masked` — **`src/form/invocation.rs:152` / `:524`**, not secrets.rs — reveal-R0 M-2), paste-warn, persistence (`secret_widgets` `#[serde(skip)]`, `schema/mod.rs:320`), exit sweep. Sites #3/#4/#5 back plain swept `String`s, not `Zeroizing` buffers (M-3) — reveal introduces no new copy at any site. One-frame defocus race documented in prose, no modal (M-4). The eye is NEVER a schema-visible control (OQ-5; `schema_mirror` deser is name-only, `schema_check.rs:98-104`).
6. **Restore fix = A1-APPEND, NEW `RESTORE_TEMPLATES` const, render-scoped ONLY** (mirror F1/`EXPORT_WALLET_TEMPLATES` at `schema/mnemonic.rs:103`): 10 shared values in order + trailing `""`; `default_value` STAYS `None`; `opts[0]` stays `bip44`. Shared `TEMPLATES` (`:69`, 10 values) untouched — its other THREE consumers stay on it: bundle `:309`, verify-bundle `:866`, **convert `:1271`** (restore-R0 m1). Restore conditional rule set unchanged (1 rule, `conditional.rs:1006-1012`; floor `("restore", 1)` at `tests/gui_schema_conditional_drift.rs:311`). The md1-mode mutex projection is REJECTED/out-of-scope; the residual (virgin `bip44` still requires a manual `(none)` in md1 mode) is tracked by the m6 FOLLOWUP (§FOLLOWUPS).
7. **Premise (re-confirmed at `16bbe67b` this write):** single-sig `--template` in `--md1` mode → `ToolkitError::ModeViolation` → exit 2 — gate inside `run_multisig` at `crates/mnemonic-toolkit/src/cmd/restore.rs:3068-3076`; dispatch = `!args.md1.is_empty()` check at `:314`, `run_multisig` call at `:349` (m3 — both live lines); `ModeViolation => 2` at `error.rs:618`. `args.template` appears in `run_multisig` ONLY at the 3068 gate → a passing multisig template is inert; `(none)` (flag dropped) is byte-identical. Therefore **all tutorial restore transcripts are byte-stable across this batch** — a transcript invariant, gated (STOP-1).
8. **Tutorial allowlist bound is manifest-literal, NOT pixel-scan** (reveal-R0 §4 correction): `secret_allowlist_violations()` (`tests/tutorial/mod.rs:384`) over `SECRET_ALLOWLIST = [S0,S1,S2]` (`mod.rs:51`) already guarantees a revealed field can only ever display an allowlisted public phrase. **Do NOT widen the allowlist** (mask-set ⊆ {S0,S1,S2} preserved). The plan adds: (a) the ⊆-agreement assertion — the widget-mask classifiers (`flag_is_secret` / `is_secret_bearing` / `node_type_is_argv_secret`) agree with the checker's taxonomies (`SECRET_SLOT_SUBKEYS`/`SECRET_NODE_TYPES_ARGV` + flag census) for every reveal-in-scope field; (b) `secret_drive_count() > 0` non-vacuity survives; (c) the §8-test-9 negative.

---

## THE DEPENDENCY GRAPH (the batch's core — what gates what, and why each expensive step runs once)

```
P1.1 FOLLOWUP filings ─┐
P1.2 REVEAL code final ─→ 61-form gallery re-pin (ONCE, in P1.2)   [gallery moves ONLY on reveal]
        │                  gui_render_emit pin re-pins (same phase)
        ▼
P1.3 RESTORE code final    [gallery/emit-INERT: asserted vs the post-reveal baseline]
        │
        ▼
P1.4 manifest edits ((none) switches + reveal drives + gate reconciliation)
        │
        ▼   ONE tutorial corpus regen (UPDATE_SNAPSHOTS + GUI_TUTORIAL_SNAPSHOTS), double-run byte-identity
        ▼
P1.5 whole-diff review → PR → CI → merge → release 0.57.0 → TAG mnemonic-gui-v0.57.0 → tag-run verify
        │  (GUI tag MUST precede the toolkit pin — paired-PR/pin rule)
        ▼
P2.1 pin bump v0.56.0→v0.57.0 → ONE toolkit-side regen from the FRESH TAG CLONE
        (28 .gui re-pin [restore.gui carries BOTH deltas] + 28 gallery PNG re-copy
         + tutorial figure re-copy [moved subset of 50] + inventory regen; transcripts ZERO delta)
        ▼
P2.2 prose (reference + tutorial) → make html → make lint 12/12 → gui_example rebuild + embed census
        ▼
P2.3 whole-diff review → merge → CHANGELOG + FOLLOWUP flips → TAG manual-gui-v1.3.0 → release-attach verify
```

**The two single-regen rules (violating either = the half-updated-corpus failure the batch exists to prevent):**
- **R-A:** `gui_form_snapshots` (61-gallery) is regenerated EXACTLY ONCE, inside P1.2, after the reveal widget + emit marker are final. P1.3 must not move it (restore is closed-form inert — `opts[0]`/`default_value` unchanged; asserted, cell 5 + census). No `UPDATE_SNAPSHOTS` run against the gallery at any other point.
- **R-B:** the tutorial corpus is regenerated EXACTLY ONCE, inside P1.4, after BOTH code changes are final AND the manifest carries both the `(none)` switches and the reveal drives. Any earlier regen bakes eye-chrome-without-`(none)` (or vice versa) into committed bytes and forces a second full re-drive. No partial/per-step regen commits.
- **Why reveal before restore:** the reveal is the phase that moves the gallery/emit surfaces; sequencing restore second makes its spec-§3 inertness claims testable against the FINAL baseline, and keeps the P1.2 28-form census cleanly attributable with nothing else in flight. (The restore spec's standalone claims "restore PNG byte-identical / `.gui` only re-pins the template line" are RESTATED for the batch: restore's gallery PNG and `.gui` DO move in this cycle — moved by the REVEAL (restore is one of the 28 masked-on-load forms, `--passphrase` row at `mnemonic-restore.gui:14`) — the restore-phase claim is "zero ADDITIONAL movement from the `(none)` append".)
- **Why the PR opens only after P1.4:** the `tutorial-snapshots` CI job (`build.yml:149`, env `GUI_TUTORIAL_SNAPSHOTS=1`) regenerates and byte-compares the corpus on every PR push; mid-branch pushes between P1.2 and P1.4 would RED it (stale corpus). Local phase gates stay GREEN throughout because both snapshot suites are env/rasterizer-gated and skipped by a plain `cargo test`.
- **Transcript invariant (both features):** reveal is display-only; `(none)` drops an inert flag (ruling 7) → the 98 `transcripts/tutorial/*` files and all run `.stdout/.stderr/.exit` bytes are IDENTICAL across the whole batch. This is asserted GUI-side in P1.4 (regen diff) and toolkit-side in P2.1 (`verify-tutorial-transcripts`); ANY delta = STOP-1.

---

## LEG 1 — mnemonic-gui (branch `feat/reveal-and-restore-none` off `master@40156b0`)

### P1.1 — FOLLOWUP filings + tracking entries (no src; one commit)

**Files:** `FOLLOWUPS.md` (GUI repo) — file: `gui-secret-reveal-tree-key-sites` (ruling 3; cites `tree_form.rs:687`/`:711` + the fixture-coupling rationale), `gui-secret-reveal-latch-timeout` (reveal-R0 M-1: capture-gated fast-follow; records the tap-does-not-latch ruling + the bounded fallback), `restore-md1-template-mutex-projection` (restore-R0 m6: the "grey `--template` in md1 mode" UX nicety — a future PAIRED toolkit-projection change: would force `("restore",1)→2` + a toolkit `src/` rule; explicitly NOT this cycle), and the cycle tracking entry `gui-secret-reveal-toggle` with a `Companion:` line into the toolkit's manual-gui FOLLOWUPS (filed P2.1) per the cross-repo mirror rule. The already-open `restore-form-single-sig-template-leaks-in-md1-mode` (`FOLLOWUPS.md:1064`) gets a "fix in flight, this cycle" note now and flips RESOLVED in the P1.5 release commit (status-flip discipline).
**Gates:** none beyond lint-by-eye; full suite untouched-green.

### P1.2 — reveal core (sites #1/#2/#3 + emit + faithfulness + hygiene tests + the ONE gallery re-pin)

**Tests FIRST (all always-run, not env-gated; spec §8 cells 1–8 + the ruling-2 cell):**
1. Masking-default (unactuated → `.password(true)` / `Role::PasswordInput`) at #1/#2/#3.
2. Reveal-flips via AccessKit `Click` (kittest-drivable latch path): next frame `.password(false)`, AccessKit exposes the FAKE public buffer text; buffer UNCHANGED (display-only, no mutation).
3. Single-revealed-field invariant (actuate A then B → A re-masked; the `Option<Id>` holds exactly one; exercise on a form with `--passphrase` + a secret positional, and on two slot rows).
4. Auto-hide ×4 discrete cells: Run click / blur / `RawInput.focused=false` / tab+subcommand switch → re-mask.
5. **Ruling-2 cell:** pointer hold→release leaves NO latch (masked next frame, stays masked); keyboard/AccessKit `Click` DOES latch.
6. Never-persist orthogonality (defense-in-depth over the I3 net): with the latch ARMED — `redact_for_persistence` output unchanged; every `assemble_argv_with_secret_mask` token still `mask == true`; `render_copy_command_masked` still `••••` (both `ShellFlavor`s). FAKE fixtures, coordinate-only failure messages (I3 harness hygiene).
7. Faithfulness both-sides + **non-vacuity negative** (an emit projection omitting the eye REDs against the real render).
8. Slot-arm isolation (`(Path, hint)` arm eye-free; eye only on `is_secret_bearing()` rows) + composite gating (eye tracks `is_secret_node`; node switch secret→non-secret removes the eye AND clears any stale latch for that Id).

**Impl files:** `src/form/secret_widget.rs` (predicate replaces `.password(true)` at `:58`; **`ui.horizontal` wrap + stable per-field `egui::Id` via the TextEdit `Response.id`** — reveal-R0 M-6; call sites `app_window.rs:826`, `widget.rs:116`, `widget.rs:159` must stay layout-sane); `src/form/slot_editor.rs` (secret arm `:52` only, per-row Id); `src/form/widget.rs` (`:606`, predicate ∧ `is_secret_node`); the reveal key + clear-trigger wiring (Run dispatch path + tab handling in `app_window.rs`; `ctx.input(|i| i.focused)`); `src/form/render_emit.rs` (ASCII marker on `MASKED`-bearing rows — `flag_value_str` `:598-600`, positional `:667`; `project_form`/`control_class` `:439-460` gain the eye fact); `tests/gui_render_faithfulness.rs` (both sides + negative); `tests/gui_render_emit.rs` — re-pin exactly the exact-ASCII pins whose forms carry secret rows (expected: mnemonic-inspect, ms-inspect, the bundle reference; enumerate empirically, record in the phase report).
**The ONE gallery re-pin (R-A):** `UPDATE_SNAPSHOTS=1 cargo test --test gui_form_snapshots` — the expected mover set is **census-derived, not a literal** (R0-m4): `moved-PNG-set == grep -rl '<masked>' docs/manual-gui/transcripts/gui/` (= 28 at `2ed3d369`; the number is the census output, and STOP-5 fires on ANY deviation rather than trusting the literal). export-wallet expected UNMOVED (`[ slot editor: 0 rows ]` on load). **P1.2 must explicitly confirm no composite form (`widget.rs:606`) renders an on-load eye** (R0-m6 watch-item — a composite whose on-load node defaults to a secret `node_type` would be the likely 29th-mover; verify each composite form's on-load node is non-secret). Any 29th mover or any unmoved expected-mover = STOP-5.
**Gates:** all new cells green; FULL suite `--jobs 2`; clippy both configs; `--no-default-features` build; `schema_mirror` / `gui_schema_conditional_drift` / partition / archetype gates untouched-green (the eye adds no schema surface — ruling 5). **Scoped post-impl review (opus)** → GREEN before P1.3.

### P1.3 — restore `RESTORE_TEMPLATES` (small, strictly after P1.2)

**Tests FIRST — new `tests/restore_template_none.rs` mirroring `tests/export_wallet_template_none.rs`, all 5 spec-§5 cells RED before impl:** (1) `(none)` clears → no `--template` token, `has_value` false; (2) no `""` sliver in argv or masked copy-command (both flavors); re-select `bip44` re-emits; (3) materialisation both ways + conditional rule set UNCHANGED across all three states; (4) `(none)` display via `display_or` (open popup + closed seam), stored raw `Dropdown("")`, all 10 real rows listed; (5) ★ append census: `RESTORE_TEMPLATES.len()==11`, `opts[0]=="bip44"`, `opts.last()==Some("")`, `opts[..10]==TEMPLATES`, `TEMPLATES.len()==10`, virgin form materialises `bip44` — demonstrate the PREPEND variant RED before landing.
**Impl:** T1 new `RESTORE_TEMPLATES` beside `EXPORT_WALLET_TEMPLATES` (`schema/mnemonic.rs:103`), doc-comment mirroring its append-not-prepend rationale + the restore semantics (single-sig `(none)` = emit all four, `restore.rs:45-49`; md1 `(none)` = drop the flag the CLI refuses); T2 `:539` `Dropdown(TEMPLATES)` → `Dropdown(RESTORE_TEMPLATES)` (`default_value` STAYS `None`); T3 optional help-text note. Consumers `:309`/`:866`/`:1271` untouched (ruling 6).
**Batch-inertness assertion:** re-run `gui_form_snapshots` WITHOUT update — zero movement (restore PNG already at its post-reveal baseline); `gui_render_emit` untouched (restore has no pin — grep-confirmed zero hits, restore-R0 item 5); conditional-drift floor `("restore",1)` unchanged. (restore-R0 **m5** — the spec's `--passphrase` citation drift, actual `:667`/`:677` — is spec-internal precision, NOT plan-carried; no plan action.)
**Gates:** FULL suite; clippy both configs. **Scoped post-impl review (opus)** → GREEN before P1.4.

### P1.4 — the COMBINED tutorial re-drive (the batch's shared expensive step; sub-plan below)

**(a) `(none)` switches — EXACTLY these six restore sites** (`tests/tutorial/manifest.rs`): the `restore_drives!` call sites `tut-j2-08-restore` (`:268`), `tut-j3-13-restore` (`:289`), `tut-j4-17-restore` (`:307`), `tut-j4-nums-restore` (`:337`, capture:false) — macro `$template` args → `""` — and the two inline J5 drives `tut-j5-23-restore-descriptor` (`:346`) / `tut-j5-24-restore-core` (`:351`) → `value: ""`. Update the `restore_drives!` doc-comment (`:136-137`, "route-around" → the `(none)` clear) and the manifest header note (`:11-14`, "papercut found → route around" → "papercut FIXED → the clean `(none)` md1 restore"). **DO NOT touch the two other `--template wsh-sortedmulti` drive sites — `convert_drives!` (`:129-133`) and `tut-j2-07-bundle-all-seeds` (`:236`) are GENUINE template selections (convert/bundle consume them), not route-arounds.**
**(b) Reveal drives (G2 — the reader sees what to type):** designation is RULE-derived, manifest-explicit, census-enforced: every `capture: true` step with ≥1 secret-classified drive carries an explicit reveal marker (a new `Drive` arm or per-step field — implementer's encoding choice) actuating the LAST-driven secret field's eye via AccessKit `Click` (the latch arm) after all field drives, BEFORE the `-form` capture; the latch persists through wheel-scroll `-form2` offsets (no blur) and is auto-hidden by the Run click before `-run`/`-modal` captures (so ONLY `-form` shots show plaintext; modal + pane shots stay masked by trigger 1). A mod.rs census asserts (predicate = **`Drive::secret_value().is_some()`**, `mod.rs:169-177` — NOT `Step::is_secret()`, which ORs `secret_modal` and would over-select): no step with a secret-value drive lacks the marker, no step without one carries it; the resulting set is **exactly these FOUR** capture:true secret-value steps — `tut-j1-01-bundle-single-sig` (slot @0 = S0); **`tut-j2-02-convert-fingerprint` + `tut-j2-03-convert-xpub` (both `capture:true` — they drive the site-#3 composite `--from phrase=` eye, S0)** [R0-m1 correction: these are NOT capture:false]; `tut-j2-07-bundle-all-seeds` (last slot = S2). The six restore steps use `TypeMd1Chain` → `secret_value()==None` → zero markers (correct — a restore form types a public `--md1` card, no seed). Under the single-revealed-field invariant the all-seeds step reveals exactly ONE slot row — the other rows visibly masked is itself the teaching image. **Word-disjointness (R0-m3, ruling-8 collision-safety):** S0/S1/S2 have distinct first words (`abandon` / `legal` / `letter`) and S2 contains neither other seed's first word, so a revealed allowlisted value can never satisfy another secret's word-probe at the parameterized checkpoint.
**(c) Gate reconciliation (ruling 8):** parameterize `assert_no_plaintext` at the **filled-form checkpoint ONLY** (`gui_tutorial_snapshots.rs:436-438`): for a reveal-marked step, the revealed field's own allowlisted value is permitted there; word-probes for every OTHER secret stay strict; the populated-pane (`:509`) and confirm-modal (`:576-579`) checkpoints stay UNCONDITIONALLY strict. Add the ⊆-agreement assertion + `secret_drive_count() > 0` survival + the negative (a non-allowlisted secret driven ⇒ RED; a reveal-marked step whose value ∉ allowlist ⇒ RED). `SECRET_ALLOWLIST` stays exactly `[S0,S1,S2]`.
**(d) The ONE regen (R-B):** `UPDATE_SNAPSHOTS=1 GUI_TUTORIAL_SNAPSHOTS=1 cargo test --test gui_tutorial_snapshots --jobs 2` — then a SECOND full run byte-identical (aggregate sha256). Expected movement: `-form` shots of reveal-marked steps (plaintext + eye), every shot whose window shows a secret field (eye chrome), the 10 restore `-form`/`-run` PNGs (`(none)` in the Template combo). Count/stems constant: 50 shots, `manifest-stems.txt` UNCHANGED (assert). **Transcripts: ZERO byte delta** (ruling 7) — any delta = STOP-1. `BUDGET_HARD_MIB = 32.0` (`gui_tutorial_snapshots.rs:73`) assertion green (eye + text pixels ≈ noise; report the new total).
**Gates:** FULL suite; both snapshot suites green against the new baselines locally; clippy both configs. **Scoped post-impl review (opus)** → GREEN before P1.5.

### P1.5 — ship Leg 1

**Leg-1 whole-diff adversarial review (opus, FULL suite)** → GREEN → PR (body: reveal = pure view-chrome, zero schema surface; restore = render-scoped F1 mirror; paired-PR rule satisfied vacuously — no toolkit clap change; NO toolkit pin bump this cycle) → CI green (clippy / headless / msrv / `snapshots` 61-with-28-new-baselines / `tutorial-snapshots` 50-shot censuses) → merge (verify via `gh pr view --json state,mergeCommit`) → release commit 0.56.0→0.57.0 (`Cargo.toml` + `Cargo.lock` + `CHANGELOG.md` — reveal hygiene model incl. the no-timeout + tap-no-latch rulings; restore `(none)`; corpus re-drive — + README self-pin; **flip `restore-form-single-sig-template-leaks-in-md1-mode` + `gui-secret-reveal-toggle` → RESOLVED here**) → **tag `mnemonic-gui-v0.57.0`** → **tag-run verification:** `snapshots` + `tutorial-snapshots` check-runs == `success` via `gh api repos/bg002h/mnemonic-gui/commits/<tag-sha>/check-runs` (explicit remote URL — Leg 2 executes where `origin` is the toolkit; fail-closed), recorded in the phase report.

---

## LEG 2 — mnemonic-toolkit `docs/manual-gui/` (branch `feat/manual-gui-v0.57.0-repin`; ONLY after the tag-run verification)

### P2.1 — pin bump + the ONE toolkit-side regen intake + companion FOLLOWUPs

**Step 0 (fail-closed):** P1.5's tag-run verification == success for BOTH snapshot jobs.
**Files:** `docs/manual-gui/pinned-upstream.toml:55` → `tag = "mnemonic-gui-v0.57.0"`; the `*-tag-implied` fields EXPECTED unchanged (`toolkit-tag-implied = mnemonic-toolkit-v0.75.0`, md/ms/mk idem — this cycle bumps no CLI pin) — VERIFY against the tag's own `pinned-upstream.toml`, don't assume. `MANUAL_GUI_UPSTREAM_ROOT` = fresh v0.57.0 clone (GOTCHA).
**Regen (single pass, census-bounded):**
| Surface | Expected |
|---|---|
| `transcripts/gui/*.gui` via `make verify-examples-gui` (`Makefile:305-307`) | **exactly 28 re-pins** (the masked-on-load set — eye marker on secret rows). `mnemonic-restore.gui` carries BOTH deltas: `:4` `--template dropdown[bip44,…,tr-sortedmulti-a,(none)] -> bip44` (format precedent: `mnemonic-export-wallet.gui:2`) AND the `:14` `--passphrase` marker row. |
| `figures/gui/*.png` (phase 9 `verify-figures-gui`) | **28 byte-copies** from the pinned clone (same set). |
| `figures/tutorial/*.png` (phase 10) | re-copy all 50 from the pinned clone; git shows the moved subset (= the P1.4 report's list). Count stays 50; stems census green. |
| `transcripts/tutorial/*` (phase 11) | **ZERO delta** (98 files byte-identical) — any delta = STOP-1. |
| `tests/expected_gui_schema_inventory.json` via `extract_gui_schema.py` | restore `--template` (`:1949` block) gains trailing `""` — documentary/un-gated (grep-confirmed no `.rs`/lint consumer; shipped-review precedent), regen for hygiene. |
Any mover outside this table = STOP-5 (unexplained drift).
**FOLLOWUPs (toolkit side):** file the `Companion:` entries for `gui-secret-reveal-toggle` (re-pin/tutorial obligation — this leg IS the discharge; flips at P2.3) in `docs/manual-gui/FOLLOWUPS.md` + `design/FOLLOWUPS.md` per convention.

### P2.2 — prose (reference + tutorial) + build + all 12 lint phases

**Reference-manual edits:**
- `src/40-mnemonic/4d-restore.md` — **REQUIRED** or `gui-schema-coverage` (phase 4) REDs: Outline bullet after `tr-sortedmulti-a` (`:194`) + a `### (none) {#mnemonic-restore-template-}` section (`kebab("") == ""` → trailing-dash anchor; F1 precedent shipped at `45-export-wallet.md:48-64`). Prose MUST differ from export-wallet's: single-sig `(none)` = emit all four `bip44/49/84/86`; md1 `(none)` = removes the single-sig template the CLI refuses (exit 2). Rewrite `:178-181` ("Same 10 values as bundle" → 10 + the GUI-only sentinel; the "a multisig value refuses" sentence scoped to single-sig mode). Fold restore-R0 **m4** into the `--format` section (`:84-92`): one line — md1 mode needs NO `--template` for `--format`.
- `src/40-mnemonic/45-export-wallet.md:48-56` — **staleness fix:** "the 11-vs-10 asymmetry is intended and export-wallet-scoped" is falsified by this cycle → "export-wallet + restore carry the sentinel; bundle/verify-bundle/convert stay 10".
- Reveal prose pass (un-gated content — the whole-diff review is the oracle): grep-driven (`grep -rn "masked\|••••\|password" src/`) targeting claims INVALIDATED by the reveal ("always masked, no way to reveal"). Primary sites: `10-foundations/14-secret-handling.md`, `30-tour/31-first-launch.md`, `30-tour/32-run-and-output.md`, `80-troubleshooting/84-secrets-and-os.md`, `75-gui-forms/750-overview.md` (+ per-CLI 751–754 intros). Content: hold-to-reveal primary / bounded latch (keyboard) / auto-hide triggers / the M-4 one-frame defocus note / run-confirm-modal-etc. stay masked always. Check `90-appendices/93-dropdown-reference.md` (its shared-`TEMPLATES` section `:30-36` lists bundle/verify-bundle/convert/export-wallet — restore appears absent; verify at impl; F1 precedent left it untouched — optional parity note only, un-gated) + glossary "reveal" term if cspell/glossary-coverage demands.
**Tutorial prose (the changed steps + the reveal teaching):** `tutorial/30-j2-multisig.md:255-265` (drop the "for consistency" disguise → "set **Template** to **`(none)`** — the card carries the full policy; a single-sig template would be refused in `--md1` mode"); `40-j3-degrading-vault.md:169`; `50-j4-taproot-twin.md:313`; `60-j5-watch-only.md:50` (+ the `~:85` J5-24 mention) — all → `(none)`. Reveal notes where shots now show plaintext: Ch0 conventions (`10-ch0-orientation.md`, near the `:120-124` argv-warning block) + the J1/J2 secret-typing steps — "the demo phrase is shown via the eye (hold-to-reveal); these are public test vectors; real phrases: verify, then release". Grep `mask` across `tutorial/*.md` for any remaining stale masking description. `tutorial-xref` (phase 12) unaffected (same stems).
**Gates:** `make html` FIRST, then `make -C docs/manual-gui lint` **12/12** (`RUSTUP_TOOLCHAIN=stable`, fresh-clone `MANUAL_GUI_UPSTREAM_ROOT`); `make gui-example-pdf gui-example-html` + embed census == 50; open the PDF — spot-check ≥3 figures incl. one revealed-phrase `-form` shot (plaintext legible at print size) and one restore `-form` shot (`(none)` visible in the Template combo); existing book targets untouched-green. **Scoped post-impl review (opus)** → GREEN.

### P2.3 — ship Leg 2 (second tag)

**Leg-2 whole-diff review (opus)** → GREEN → PR or direct-FF (toolkit house rule; zero `src/` changes — no version-site ritual, no vendor, no ToolkitError surface) → `manual-gui.yml` green (lint / verify-examples / verify-examples-gui / build incl. gui_example embed census) → merge → shipping commit: finalize `docs/manual-gui/CHANGELOG.md` (dated v1.3.0 entry: reveal depiction + `(none)` + the tutorial now teaching the fixed flow) + **flip the toolkit-side companion FOLLOWUPs → RESOLVED** → **tag `manual-gui-v1.3.0`** → verify the release job attached BOTH the manual PDF and the rebuilt **`gui_example.pdf`** (release-attach is the PDF's SOLE channel — precedent decision; gh-release TOCTOU gotcha: `|| true` pattern, fix-forward + re-tag if the attach fails) → ship report to the user (+ the standing post-cycle FOLLOWUP-burndown offer: the three new slugs from P1.1).

---

## REVIEW CADENCE (all reviews opus; all persisted VERBATIM to `design/agent-reports/` BEFORE folds; folds re-enter the loop)

| Gate | Artifact |
|---|---|
| THIS plan-doc R0 | `tutorial-batch-plan-r0-round-N.md` → GREEN 0C/0I before ANY implementation |
| P1.2 reveal scoped post-impl | `tutorial-batch-p1.2-postimpl-round-N.md` (FULL suite) |
| P1.3 restore scoped post-impl | `tutorial-batch-p1.3-postimpl-round-N.md` |
| P1.4 re-drive scoped post-impl | `tutorial-batch-p1.4-postimpl-round-N.md` |
| Leg-1 whole-diff (pre-merge) | `tutorial-batch-leg1-whole-diff-review.md` |
| P2.1+P2.2 post-impl | `tutorial-batch-p2-postimpl-round-N.md` |
| Leg-2 whole-diff (pre-merge) | `tutorial-batch-leg2-whole-diff-review.md` |

If Agent-API dispatch fails mid-session: flag explicitly, defer the review to API recovery — never substitute inline self-review.

## FOLLOWUPs LEDGER (file in P1.1 / P2.1; flip in the shipping commits)

| Slug | Repo | Action |
|---|---|---|
| `restore-form-single-sig-template-leaks-in-md1-mode` (`FOLLOWUPS.md:1064`, open) | mnemonic-gui | FLIP → RESOLVED in the P1.5 release commit |
| `gui-secret-reveal-toggle` (+ toolkit `Companion:`) | both | FILE P1.1/P2.1; FLIP at P1.5/P2.3 |
| `gui-secret-reveal-tree-key-sites` | mnemonic-gui | FILE (deferred sites #4/#5) |
| `gui-secret-reveal-latch-timeout` | mnemonic-gui | FILE (capture-gated fast-follow; records the tap-no-latch ruling + any fallback downgrade) |
| `restore-md1-template-mutex-projection` | mnemonic-gui | FILE (the "grey `--template` in md1" paired-projection nicety; restore-R0 m6 "SHOULD-file") |
| `gui-secret-buffer-allocator-residue`, `gui-os-snapshot-secret-occlusion`, `gui-path-flag-no-file-picker` (B1) | — | UNTOUCHED (orthogonal / deferred / not this batch) |

## TWO-TAG RELEASE SEQUENCE (strict)

1. Leg 1 complete → **`mnemonic-gui-v0.57.0`** → tag-run check-runs verified `success` (`snapshots` + `tutorial-snapshots`).
2. Only then Leg 2 → **`manual-gui-v1.3.0`** → release-attach verified (`gui_example.pdf` + manual PDF).
3. No toolkit crate tag; no crates.io; `docs/manual/` untouched (no CLI-surface change → the mirror invariant does not fire).

## STOP / ESCALATION LEDGER (user decisions — never implementer choices)

1. **Any tutorial restore transcript byte-delta under `(none)`** (or ANY transcript delta batch-wide) → STOP: contradicts the verified `run_multisig` inertness (ruling 7) — investigate before any re-pin.
2. **Any toolkit `src/` / clap / conditional-projection edit pressure** (mutex arm, floor bump, `gui_schema.rs` projection, eye-as-schema-control) → STOP: both specs are render-scoped by R0 ruling.
3. **Any `schema_mirror` / `gui_schema_conditional_drift` / partition-gate delta** → STOP (accidental schema surface).
4. **Corpus regen > `BUDGET_HARD_MIB` 32 MiB** → STOP (the ceiling is a user decision from the gui_example P1.5 STOP).
5. **Any pinned-artifact movement outside the phase censuses** (a 29th gallery form; an unexpected `.gui`/emit-pin/anchor; a `manifest-stems.txt` delta) → STOP back to the owning phase's review.
6. **Faithfulness gate cannot model the eye for sites #1/#2 without fixture-value coupling** → STOP (site #3 is already carved out to a kittest cell by R0; #1/#2 failing would break OQ-3's depict ruling).
7. **Cross-backend (GL↔Vulkan) drift > dify 0.6 on the NEW shot classes** (eye glyph, revealed text — pick a glyph proven in egui's bundled fonts; assert `device_type == Cpu`) after the remediation ladder → STOP with evidence.
8. **Any pressure to widen `SECRET_ALLOWLIST` beyond `[S0,S1,S2]`** or to weaken the modal/pane/persist checkpoints → STOP (secret-hygiene first-class bar).
9. *Recorded-downgrade (NOT a user STOP):* pointer-tap/AT-click indistinguishable in egui → tap-latches fallback per ruling 2, recorded in the phase report + `gui-secret-reveal-latch-timeout`.
