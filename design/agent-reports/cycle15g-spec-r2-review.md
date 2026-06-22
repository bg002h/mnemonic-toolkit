# R0 REVIEW (round 2) — cycle-15 Lane G (`mnemonic-gui` secret-residue zeroize)

**Spec:** `/scratch/code/shibboleth/wt-tk-master/design/BRAINSTORM_cycle15g_gui_secret_residue_zeroize.md`
**Prior round:** `/scratch/code/shibboleth/wt-tk-master/design/agent-reports/cycle15g-spec-r1-review.md` (NOT GREEN — 1 Important I1 + 4 Minor M1–M4)
**Verified against:** `origin/master @ 5ce9d53` (re-fetched 2026-06-21; `git rev-parse origin/master` → `5ce9d53`). All `src/**` line numbers re-grepped live; confirmed byte-identical to the v0.46.0 tag (the sole delta over the tag is `+50` lines to `FOLLOWUPS.md`).

**Verdict: GREEN — 0 Critical / 0 Important.** The I1 fold is correct AND complete, no fold introduced new Critical/Important drift, and every load-bearing citation re-verifies against live `origin/master`. One residual cosmetic line-citation imprecision is noted as Minor (non-blocking). The lane may advance to the plan-doc and its own R0 loop.

---

## Task 1 — I1 fold (§3.2) verification: CORRECT and COMPLETE

The §3.2 fold is sound. Verified against `main.rs:1064-1097` (read live):

- **Avoids E0509.** The current site `:1064` is `if let Some((argv, mask, stdin)) = self.pending_confirm_argv.clone()` and the Run arm at `:1093` *moves* `argv, mask, stdin` into `spawn_and_capture` (an owned consumer, sig confirmed at `:1190-1195`). Under a `Drop`-bearing `PendingConfirm` struct, field-destructure-then-move is E0509. The prescribed pattern — `let Some(pending) = …clone()`, borrow `pending.argv`/`pending.mask` in the modal, `spawn_and_capture(self, pending.argv.clone(), pending.mask.clone(), pending.stdin.clone())` on Run — binds the whole value (no field move-out) and clones owned copies into the consumer. This compiles: binding a `Drop` value by a single name is legal; only *moving fields out of it* is E0509. **Correct.**
- **Modal-body compatibility confirmed live.** The modal at `:1081-1088` reads `argv.iter()` / `mask.get(i)` **by-ref only** — so "render the modal borrowing `pending.argv`/`pending.mask`" works with zero change to modal semantics. The fold's claim is grounded in the actual code.
- **Hygiene goal preserved.** The cloned `pending` (a full `PendingConfirm`) drops at scope end on every path (Run, Cancel, modal-still-open), firing `Drop::drop → zeroize`, scrubbing the per-frame transient — which is *the* residue this cycle closes. §3.2 and D2 state this correctly.
- **Residual correctly characterized as acceptable.** The Run-path inner clones (`pending.argv.clone()` …) are owned, one-shot, immediately consumed by the OS process spawn, and not themselves drop-scrubbed. §3.2:129 characterizes this as "a far smaller residence than the per-frame modal clone … not worth threading `Drop` through `spawn_and_capture`'s owned params." This is an accurate and proportionate trade — the residue strictly shrinks vs. the status quo (the per-frame clone now scrubs; only a one-shot Run-click clone remains, transiently).
- **The "do not delete `Drop` to silence E0509" guard (§3.2:130) is present and load-bearing** — it pins compile-green + the T3/T4 scrub tests as the guard against a hurried implementer reverting to shape (B). Good.
- **`RunResult` `impl Drop` is genuinely E0509-clean.** Verified `last_run` is read by `ref` at `:470` (`if let Some(ref result) = self.last_run`) and every replace/clear (`:1206/:1222/:1226`) is a plain assignment, with no `mem::take`/`.take()`/`mem::replace` anywhere on `last_run` (grep returned none). No field is ever moved out of `last_run`, so `impl Drop for RunResult` compiles cleanly. §2:57 and §3.2's note state this correctly.

## Task 2 — M1 disposition: SPEC IS CORRECT; r1 review was the misread

Independently re-grepped `git show origin/master:src/main.rs | nl -ba | sed -n '1218,1228p'` @ `5ce9d53`:
- `:1221` = `result.mask = mask;`
- `:1222` = `app.last_run = Some(result);` ← **the store**
- `:1223` = `app.last_run_error = None;`

The spec's `:1222` is correct. The r1 review's claim of `:1223` was the misread. The fold's §9-note (line 9 / §2:55) correctly identifies this as a reviewer misread and makes **no source-citation change** — which is the right disposition. **Confirmed.**

## Task 3 — M2 fold (§6/T7 + D7): SOUND

Verified live: `tree_form.rs:431 pub fn render(...)`, `:601 fn render_node` (private), `:685 fn render_payload` (private). The fold correctly notes the slot-test analogy does **not** transfer 1:1 — `slot_secret_mask_v0_38_0.rs` drives `slot_editor::render` which IS `pub`, whereas tree_form's payload/node renderers are private. The two prescribed resolutions — (i) drive the public `tree_form::render` with a constructed Key-node `FormState`, preferred; (ii) a narrow `pub(crate)` payload seam as last resort — are both sound, and deferring the concrete pick to the plan-doc (with (i) preferred) is appropriate for a brainstorm SPEC. The composite T5 seam is correctly noted as already reachable via `pub render`/`pub render_with_dispatch` (`widget.rs:473/81`, both verified `pub`). **Sound.**

## Task 4 — M3 fold (§4 slug-3 + T5 + D12): CORRECT

Verified the composite arm at `widget.rs:653` captures `let response = ui.text_edit_singleline(value);` and reads `response.changed()`-equivalent paste detection in the `:654-665` block (gated on `node_type_is_argv_secret` at `:663`). The fold prescribes `let response = ui.add(egui::TextEdit::singleline(value).password(<gate>));` — and `ui.add(...)` returns a `Response` identically. This is **proven by the codebase itself**: `secret_widget.rs:84` is `let response = ui.add(egui::TextEdit::singleline(&mut transient).password(true));` — the exact idiom, already in use, already capturing the `Response`. So the paste-warn at `:654-665` does not regress. The gate-hoist (evaluate `node_type_is_argv_secret(node)` once, drive both `.password(gate)` and the `:663` condition) is correct and avoids double-eval drift. D12 records it consistently. **Correct.**

## Task 5 — Citation re-verification @ `5ce9d53`: ALL PASS (one cosmetic imprecision → Minor)

| Citation | Result |
|---|---|
| `runner.rs:18-31` (`argv:20, mask:27, exit_code:29, stdout:30, stderr:31`) | exact; `stdout/stderr` are `Vec<u8>` (M4 — spec correct) |
| `main.rs:92/104/114` (PendingConfirm tuple, last_run, pending_confirm field) | exact |
| `main.rs:1045/1064/1092/1096` (set / consume-destructure / Run-clear / Cancel-clear) | exact |
| `main.rs:1206/1222/1226` (clear-on-start / store / clear-on-fail) | exact (store = `:1222`, M1 confirmed) |
| `main.rs:1190-1195` `spawn_and_capture` owned sig; `:470` by-ref read; `:1121` on_exit; `:1144-1145` sweep | exact |
| `widget.rs:653` plain `text_edit_singleline`, `:663` predicate, `:625` arm | exact |
| `tree_form.rs:431 pub render / :601 render_node / :685 render_payload / :697 / :717 / :785 xprv_hint / :699/:723 hint calls` | exact |
| `tree_model.rs:675 pub is_xprv_like` (`[1..4]=="prv"` after `rsplit(']')`) | exact |
| `secrets.rs:135/175/279-288/294`; `:300/:302-303` (String + composite scrub) | exact |
| `Cargo.toml:3=0.46.0 / :20 zeroize="1" / :74 egui_kittest="0.31"` | exact |
| `secret_widget.rs:84` `.add(...).password(true)` contrast | exact |

**One cosmetic imprecision (Minor, see below):** §2:72 attributes the in-source comment *"The value field is a plain (non-password) text edit"* to `widget.rs:648`; live, that phrase begins at `:647` (`// The value field is a plain (non-password) text edit; mirror`) and spills into `:648`. The substantive citation (`:653` plain widget, `:663` predicate) is exact. Off-by-one on a comment-quote start line only; no logical impact.

## Task 6 — Resolved-decisions table (D1–D12) internal consistency: CONSISTENT

D1–D12 all align with the prose post-fold. Cross-checks: D2 matches §3/§3.2 (struct+Drop + the explicit §3.2-cross-reference for the consumption pattern); D7 matches §6/T7 (public-render-preferred / `pub(crate)` last-resort); D12 matches §4-slug-3/T5 (`ui.add(...).password(gate)` + hoist + regression assertion). Numbering note: the table runs D1–D7, then **D12**, then D8–D11 (D12 inserted out of numeric order, adjacent to D7 by topic). Slightly unusual ordering but every D is present, unique, and internally consistent — cosmetic, not a defect.

## Task 7 — SemVer / gates: ALL CORRECT

- **SemVer GUI MINOR 0.46.0 → 0.47.0** — `Cargo.toml:3` is `0.46.0` live; new hygiene behavior, no breaking API → MINOR correct.
- **PR-gate + 5-target CI before tag** — correct; GUI is the PR-gated constellation repo.
- **No toolkit pin bump (D9)** — correct; pure internal app-state + render, reuses pinned `mnemonic-toolkit-v0.60.0` taxonomy.
- **No `schema_mirror` impact (D10)** — independently verified: `schema/mnemonic.rs` (pinned to v0.60.0) references none of `RunResult`/`PendingConfirm`/`last_run`; no clap flag/option/subcommand/dropdown-value add/remove/rename. Correct.
- **Never `cargo fmt` the GUI (D11)** — correct; no fmt CI gate in mnemonic-gui.
- **Pinned-v0.60.0-binary drift-test note (§7)** — correct and matches the known `MNEMONIC_BIN=v0.60.0` false-fail caveat for `schema_mirror`/`canonicity_drift`.

## Task 8 — §10 R0-r1 dispositions: ACCURATE, NO OVER-CLAIMING

Cross-checked all 5 ratified items against the r1 review text:
1. **D2 mechanism (struct+Drop) WITH the I1 consumption-pattern correction** — r1 §I1 "(a) Recommended" + axis-2 "RATIFY the struct+Drop approach (with I1's consumption-pattern correction)." Accurately stated as the sole Important, now folded. OK
2. **D4 inline `.password(true)`** — r1 axis-3 / the prompt's ratification of inline-not-SecretLineEdit. OK
3. **stdout/stderr whole-scrub blessed** — r1 axis-1(e) "RATIFIED … Bless it." OK
4. **Slug-4 flicker / gated masking** — r1 axis-3 slug-4 (`is_xprv_like` masks only mis-pasted privkeys; watch-only stays readable). OK
5. **T2 testability floor** — r1 axis-3 / M4-adjacent: unit `RunResult::zeroize` + code-review assertion of `:1222`/`on_exit` wiring as an acceptable floor. OK

No item over-claims beyond what r1 actually ratified.

---

## CRITICAL
None.

## IMPORTANT
None.

## MINOR (non-blocking; fold opportunistically into the plan-doc)

- **m2-r2 (new, cosmetic).** §2:72 cites the comment phrase *"The value field is a plain (non-password) text edit"* at `widget.rs:648`; live it begins at `:647`. Off-by-one on a comment-quote start line; the substantive `:653`/`:663` citations are exact. Update if convenient when the plan-doc lifts §2.
- **d-table ordering (cosmetic).** The Resolved-decisions table runs D1–D7, D12, D8–D11 (D12 inserted adjacent to its topical sibling D7). All decisions present/unique/consistent; renumber to monotonic only if a future editor finds the out-of-order D12 confusing.

Neither Minor blocks the gate.

---

## Verdict

**GREEN (0 Critical / 0 Important).** The I1 fold is correct (avoids E0509 via whole-struct bind + Run-path re-clone), complete (modal-body borrow path verified compatible, residual correctly characterized, the "don't delete `Drop`" guard pinned), and introduced no new Critical/Important drift. M1 is correctly dispositioned as a prior-reviewer misread (`:1222` confirmed by independent re-grep). M2 (tree-render seam) and M3 (`Response`-preserving `ui.add`) folds are both sound and grounded in live code (`secret_widget.rs:84` is the proof-of-idiom for M3; `slot_secret_mask` is the proof for the M2 seam-asymmetry note). All load-bearing citations re-verify against `origin/master @ 5ce9d53`. SemVer/PR-gate/no-toolkit-pin/no-schema_mirror/never-fmt/pinned-binary notes are all correct.

Per the standing gate, this brainstorm SPEC has converged to 0C/0I. **The lane may now advance to the plan-doc, which gets its own R0 loop.** Persist this review verbatim to `design/agent-reports/cycle15g-spec-r2-review.md` before proceeding.
