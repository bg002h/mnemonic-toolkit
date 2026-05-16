# Phase P2.3 (Track G — widget_help_icon kittest GREEN) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-help-icons` (mnemonic-gui)
**Scope:** §3.2 P2.3 sub-phase — post-P1.4-LOCK amendment to `mnemonic-gui/tests/widget_help_icon.rs`: `harness.run()` after `click()` changed to `harness.step()` (line 153); 20-line extension of the module-level docstring at lines 49-67 explaining the empirical rationale ("G-P2.3 amendment, post-P1.4-LOCK, empirically driven"). Excludes the signature-plumbing change at line 131 (G-P2.2 scope) and the 91-button widget integration in `src/form/widget.rs` + `src/main.rs` (G-P2.2 scope).

**Verdict:** **LOCK 0C / 0I / 1N / 0n.** Folded inline: §2.1 G7 plan amendment per I-1 recommendation.

The source-grep evidence is unambiguous: the docstring's mechanism narrative is byte-faithful to the egui_kittest 0.31.1 + egui 0.31.1 sources, the line numbers cited (`lib.rs:281`, `lib.rs:359`, `context.rs:2331`, `context.rs:1452-1453`, `data/output.rs:115`) all resolve correctly, and the choice of `step()` over `run_steps(1)` is semantically equivalent for this single-event use case. The amendment is a legitimate post-LOCK *additive* fold — the LOCKed P1.4 plan-prose at §3.1 P1.4 (lines 935-961) and §2.1 G7 (lines 496-549) explicitly never prescribed `run()` vs `step()` after click; they only specified the *read path*, leaving the frame-advance API as a downstream implementation choice.

**Runtime confirmation (closing the R0 tool-environment gap):** The reviewer was tool-scoped (read/grep only) and could not run cargo. The executor independently verified:

- `cargo test --test widget_help_icon` → `2 passed; 0 failed; 0 ignored` (both `cell_help_icon_read_path_sanity_probe` + `cell_help_icon_emits_open_url_for_mnemonic_convert_from` GREEN).
- Empirical step()-vs-run() one-off (since deleted): `step()` → `[OpenUrl(OpenUrl { url: "https://bg002h.github.io/mnemonic-toolkit/manual-gui/#mnemonic-convert-from", new_tab: true })]`; `run()` → `DEBUG run() steps: 3, DEBUG run() commands: []`. Matches the executor's quoted observation.
- `cargo test --test manual_anchor_coverage` → `5 passed; 0 failed; 1 ignored` (still default-skipped per `[[feedback-default-cargo-test-runs-sibling-dependent-tests]]`; not prematurely flipped to GREEN against missing HTML).

---

## Critical

None.

## Important

None blocking. One Important fold (recommended-not-blocking) folded inline post-LOCK — see I-1.

### I-1 (FOLDED inline) — §2.1 G7 plan-text amended to specify `step()` after click

§2.1 G7 fold-history paragraph (plan lines 535-542) extended with a third-fold entry naming the G-P2.3 step()-amendment, the source line numbers (`egui_kittest-0.31.1/src/lib.rs:225-234` for `step`; `lib.rs:251` for the `self.output = output` overwrite; `lib.rs:305-322` for `run`'s loop body), and the empirical observation (step() → 1 OpenUrl; run() → 3 steps, []). This closes the latent drift gap the reviewer identified: a future kittest reuse from §2.1 G7 alone (without opening the test file) could plausibly re-introduce the bug. With the fold, SPEC and test agree at the plan level.

## Nice-to-have

### N-1 — Docstring claim "A click on a stateful egui button typically triggers a follow-up immediate repaint (animation / hover state)" is the *probable* but not source-line-pinned root cause

The docstring at line 53-54 attributes the multi-frame run() to "animation / hover state". The empirically verified fact ("step() → 1 OpenUrl; run() → 3 steps, [] commands") is what matters and is logically consistent with the source mechanism (`_step` overwriting `self.output` at `egui_kittest-0.31.1/src/lib.rs:251`). However, the *specific cause* of the additional 2 frames after the click frame is implementation-detail of egui's response-handling and button-state machinery — the reviewer did not pin the exact `request_repaint()` call site driving the loop. Stylistic-only: the docstring's "typically triggers a follow-up immediate repaint" is appropriately hedged with "typically" and the operational claim is the empirical 3-step observation, not the precise mechanism. No action required.

## Nit

None.

---

## Verification trace

1. **Source-faithfulness of step() vs run() rationale (claim 1):**
   - `egui_kittest-0.31.1/src/lib.rs:281` is `pub fn run(&mut self) -> u64` — matches docstring claim.
   - `lib.rs:305-322` (`try_run`) confirms run's body loops `self.step()` then breaks if `repaint_delay != Duration::ZERO` — matches docstring "loops step() until repaint_delay != ZERO".
   - `lib.rs:225-234` (`pub fn step`) takes `self.kittest.take_events()`; if empty, runs `_step(false)` once; else runs `_step(false)` once per event. For a click that queues exactly one `Event::ActionRequest` (verified at `kittest-0.1.0/src/node.rs:80-86`, single `self.event(...)` call), step() runs exactly one frame.
   - `lib.rs:240-252` (`_step`) at line 251 writes `self.output = output;` — this is the smoking gun. Every `_step` overwrites the prior frame's full output, which is why no-op frames after the click frame replace the populated `commands` Vec with an empty one (since `viewport.output` is drained at end_pass).
   - `egui-0.31.1/src/context.rs:2331` confirms `let mut platform_output: PlatformOutput = std::mem::take(&mut viewport.output);` — the per-frame drain.

   **Pass: rationale is byte-faithful to source.**

2. **Read-path source verification (claim 2):**
   - `context.rs:1452-1453`: `pub fn open_url(&self, open_url: crate::OpenUrl) { self.send_cmd(crate::OutputCommand::OpenUrl(open_url)); }` — writes only to `commands`, never touches deprecated `open_url` field.
   - `data/output.rs:115`: `#[deprecated = "Use `Context::open_url` or `PlatformOutput::commands` instead"]` annotates `pub open_url: Option<OpenUrl>` at line 116. Acceptable cite.

   **Pass: read-path docstring claims match source.**

3. **Empirical evidence reproducibility (claim 3):** Reviewer source-traced the mechanism but could not run cargo from the dispatch. Executor independently reproduced via one-off debug test (since deleted):
   - `harness.step()` after click → `[OpenUrl(OpenUrl { url: "https://bg002h.github.io/mnemonic-toolkit/manual-gui/#mnemonic-convert-from", new_tab: true })]`.
   - `harness.run()` after click → 3 steps returned, commands `[]`.
   Matches the docstring's quoted observation at line 62. **Pass with executor-side runtime confirmation.**

4. **Plan amendment shape (claim 4):**
   - Docstring lines 49-67 self-label as "G-P2.3 amendment (post-P1.4-LOCK, empirically driven)" and cite `egui_kittest-0.31.1/src/lib.rs:281` (correct) and `lib.rs:359` (correct).
   - The prose at lines 53-54 attributes the multi-frame run to "animation / hover state" — see N-1; correctly hedged with "typically".
   - The prose at lines 56-59 is byte-accurate against `lib.rs:359-361` + `context.rs:2331`.
   - The conclusion at lines 59-62 is source-correct given the single-event `click()` action: `take_events` returns `[ActionRequest]` (len 1), and step's for-loop runs `_step` exactly once for it (`lib.rs:230-233`).

   **Pass: line numbers + prose are source-faithful; no overclaiming.**

5. **`harness.step()` choice over `harness.run_steps(1)` (claim 5):**
   - `lib.rs:340-346` defines `pub fn run_steps(&mut self, steps: usize) { for _ in 0..steps { self.step(); } }` — literally "calls step() x times". `run_steps(1)` is byte-equivalent to a single `step()` call.
   - The test uses bare `step()`. Both are correct; `step()` is marginally clearer for "exactly one frame" semantics. No issue.

   **Pass: API choice is semantically equivalent and clear.**

6. **P1.4 cell + sanity probe BOTH GREEN (claim 6):** Source-traced as PASS by reviewer; executor confirmed via `cargo test --test widget_help_icon`: `2 passed; 0 failed; 0 ignored`.

7. **No new clippy warnings from the kittest amendment (claim 7):** The amendment changes one expression (`harness.run()` → `harness.step()`) and extends a `//!` docstring. Neither introduces new clippy lint categories. Pre-existing 3 `doc_overindented_list_items` errors at `tests/manual_anchor_coverage.rs:25-29` (per P2.1 R0 verification 11) are out of scope and stay deferred to P3 cycle-wide LOCK.

---

## Plan-amendment-discipline assessment

The user's question: *"Is the step() amendment a legitimate post-LOCK fold (additive clarification to a LOCKed test)? Or should it have prompted reopening the P1.4 LOCK for an R*n round?"*

**Legitimate post-LOCK additive fold.**

Rationale:
1. The LOCKed P1.4 prose (§3.1 P1.4 + §2.1 G7) explicitly never pinned the post-click frame-advance API. The plan-prose treated the frame-advance as a downstream implementation detail.
2. The amendment is additive (extends docstring) and a one-line code change (`run()` → `step()`); it does not reopen any LOCKed claim.
3. The amendment is *empirically driven* per `[[feedback-architect-must-run-prose-commands]]`: the executor encountered an empirical failure and corrected the implementation.

If the LOCKed prose had pinned `harness.run()` post-click, then yes, reopening for R*n would be necessary. As-is, the amendment is *consistent with* the LOCKed prose (which is silent on this point) and only refines the downstream implementation choice. I-1 (folded) extends the SPEC to record this choice so future re-use from §2.1 G7 alone stays correct.

---

**Final verdict:** **LOCK 0C / 0I / 1N / 0n.** G-P2.3 kittest GREEN sub-phase complete. Both `cell_help_icon_read_path_sanity_probe` and `cell_help_icon_emits_open_url_for_mnemonic_convert_from` PASS. The step()-amendment is source-faithful to egui_kittest 0.31.1 + egui 0.31.1 at every cited file:line, and is a legitimate post-P1.4-LOCK additive fold per the cycle's `[[feedback-architect-must-run-prose-commands]]` discipline. I-1 SPEC fold applied inline at plan lines 535-542 to close the latent drift gap.

P2 parity gate for Track G reached: 91-button integration shipped (G-P2.2), kittest GREEN (G-P2.3), helper module + URL scheme LOCKed (G-P2.1). Track M (P2.4 + P2.5) is the remaining P2 work before P3 cycle-wide LOCK + PE.
