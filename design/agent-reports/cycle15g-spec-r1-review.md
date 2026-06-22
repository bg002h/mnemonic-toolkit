# R0 REVIEW (round 1) — cycle-15 Lane G (`mnemonic-gui` secret-residue zeroize)

**Spec:** `/scratch/code/shibboleth/wt-tk-master/design/BRAINSTORM_cycle15g_gui_secret_residue_zeroize.md`
**Verified against:** `origin/master @ 5ce9d53` (v0.46.0 + 1 sweep-report commit; all `src/**` line numbers byte-identical to the v0.46.0 tag — confirmed). `zeroize 1.8.2` + `zeroize_derive` resolved in lockfile.

**Verdict: NOT GREEN — 1 Important finding (I1).** Plus 4 Minor. The architecture (whole-holder zeroize + zeroize-on-drop + `.password(true)` masking) is sound and the priority axis (run-holder scrub completeness) is materially correct, but the spec prescribes a `PendingConfirm` struct shape that **will not compile** against the existing move-out destructure at `main.rs:1064` — the exact site D2 leans on. That must be corrected before the plan-doc.

---

## Citation verification (all PASS, 2 trivial line-drifts)

Every cited line verified against `5ce9d53`:
- `runner.rs:18-31` RunResult: `argv:20`, `mask:27`, `exit_code:29`, `stdout:30`, `stderr:31` — **exact match.** Note: `stdout`/`stderr` are `Vec<u8>`, NOT `String` (the review prompt's axis-1(a) phrasing said "`String`" — the spec correctly treats them as `Vec<u8>` in T1, line 146). No defect; flagging for the plan-doc's `Zeroize` impl which must call `.zeroize()` on `Vec<u8>` (valid — `Vec<u8>: Zeroize`).
- `main.rs` last_run/pending_confirm sites: `104`, `92`, `114`, `319/320/324`, `1045`, `1064`, `1092`, `1096`, `1206`, `1226` — all exact. **One drift:** spec §2:54 cites the store as `:1222`; actual is `:1223` (the `result.mask = mask;` insertion at `:1222` pushes the store down one). The sweep report has it right (`:1223`). MINOR (M1).
- `on_exit` `:1121`, sweep `:1144-1145`, save-FIRST/no-`mem::take` ordering — exact, and the load-bearing comment block is present verbatim.
- `widget.rs:653` plain `text_edit_singleline(value)`, comment `:648`, argv-secret predicate at `:663` — exact.
- `tree_form.rs:697`/`:717` plain key widgets, `xprv_hint` `:785`, `is_xprv_like` reachable — exact. `is_xprv_like` lives at `tree_model.rs:675` (spec's NOTE is correct).
- Infra: `Cargo.toml:20` `zeroize="1"`, `:74` `egui_kittest="0.31"`, `secrets.rs:175` predicate, `:279-288` SecretBuffer Zeroize+Drop, `:294` `zeroize_form_state` — exact.
- **`mem::replace`/`mem::take`/`.take()` scan:** NONE on `last_run` or `pending_confirm_argv`. Every replace/clear is a plain `= Some(..)`/`= None`, so the old value drops in place → axis 1(c) is **clean**: zeroize-on-Drop genuinely covers all replace/clear sites with no move-into-holder escape path.

---

## IMPORTANT

### I1 — `PendingConfirm` struct + `Drop` will not compile against the move-out destructure at `main.rs:1064` (E0509)

D2 / §3 (spec:106, 128) and Open-Q-1 (:196) prescribe: promote the tuple to `struct PendingConfirm { argv, mask, stdin }` with `Zeroize`+`Drop`, claiming "the `.clone()` at `:1064` then auto-scrubs its transient too." **As literally specified this does not compile.** The current site is:

```
if let Some((argv, mask, stdin)) = self.pending_confirm_argv.clone() { ... spawn_and_capture(self, argv, mask, stdin); }
```

This **moves** the tuple's fields out of the cloned `Option`. Rust **forbids moving fields out of a type that implements `Drop`** (E0509 "cannot move out of type which implements the `Drop` trait"). The equivalent `let Some(PendingConfirm { argv, mask, stdin }) = ...clone()` against a `Drop` struct is a hard compile error. Verified consumer: `spawn_and_capture(app, argv: Vec<String>, mask: Vec<bool>, stdin: Option<Vec<u8>>)` (`main.rs:1190`) takes **owned** values, and the Run arm at `:1092` moves all three in.

**Required change (pick one, spec must specify):**
- **(a) Recommended — bind the whole struct, re-clone inner fields on Run.** `let Some(pending) = self.pending_confirm_argv.clone()`; render the modal borrowing `pending.argv`/`pending.mask`; on Run call `spawn_and_capture(self, pending.argv.clone(), pending.mask.clone(), pending.stdin.clone())`. The cloned `pending` (a full `PendingConfirm`) drops at end of scope → its `Drop` fires → the per-frame transient **is** scrubbed, preserving D2's hygiene goal. Cost: one extra inner-field clone on the Run click path (acceptable — Run is a one-shot user action, and that inner clone is itself owned and short-lived; note for completeness it is NOT itself drop-scrubbed unless `spawn_and_capture` is also adjusted, but it is immediately consumed by spawn).
- **(b)** Keep `PendingConfirm` a plain `Zeroize` struct WITHOUT `Drop`; scrub explicitly at `:1092`/`:1096` + exit sweep, and accept that the `:1064` clone does not auto-scrub (the residue the spec is trying to close). This is strictly weaker — it reintroduces exactly the `:1064` transient the spec calls "the trap."

The cleanest reconciliation is **(a)**: it keeps `Drop`-based auto-scrub for the holder AND the per-frame clone, and only the briefly-live Run-path inner clones are un-drop-scrubbed (a one-shot, immediately-spawned residue — far smaller than the per-frame modal clone). The spec/plan must state the consumption-pattern change (whole-struct bind + inner re-clone on Run) explicitly, because the naive field-destructure is a compile error, and a hurried implementer following §3 verbatim will hit E0509 and may "fix" it by dropping `Drop` (silently reverting to the weaker (B) the spec rejected).

> Note this does **not** affect `RunResult`: nothing moves fields out of `last_run` (it's read by-ref at `:470` and replaced/cleared by plain assignment), so `impl Drop for RunResult` is clean. The E0509 hazard is `PendingConfirm`-specific, driven by the `spawn_and_capture` owned-move consumer.

---

## Axis-by-axis findings (the priority: run-holder scrub completeness)

**1(a) RunResult field coverage + Vec<String> element scrub — CORRECT.** All five secret-bearing fields covered by whole-holder zeroize: `argv: Vec<String>`, `stdout: Vec<u8>`, `stderr: Vec<u8>` (secret-bearing); `mask: Vec<bool>` (D6, harmless). `Vec<String>: Zeroize` in zeroize 1.x scrubs **each element's `String` bytes then clears the outer Vec** (`String: Zeroize` is live — used by M9 / `zeroize_form_state:300`), so element bytes ARE scrubbed, not just the outer Vec. `exit_code: Option<i32>` is non-secret, correctly untouched. **Verified sound.**

**1(b) The `:1064` per-frame clone drop-scrub — the claim is directionally right but mechanism-broken; see I1.** The clone exists (confirmed `:1064`), it is per-frame while the modal is open, and a `Drop` impl WOULD scrub it — but only under consumption pattern (a). Under the spec's literal move-out destructure it doesn't compile. Resolve via I1.

**1(c) Move-without-drop escape paths — NONE.** Confirmed by the `mem::*`/`.take()` scan: no holder is moved/replaced without the old value dropping in place. Clean.

**1(d) `mask` display-only, not a scrub boundary — CORRECT and correctly NOT relied upon.** The spec (§3:92, D1, D6) explicitly treats `mask` as a render hint, scrubs the whole holder regardless, and zeroizes `mask` too for uniformity. Verified against `runner.rs:21-26` doc ("Used only to mask the last-run argv: display; never affects what is spawned") and `main.rs:1082-1087` render use. Fail-closed, correct.

**1(e) Whole-scrub of stdout/stderr — RATIFIED.** Erring toward scrub is hygiene-correct. Confirmed no functional need to retain captured output post-display: `last_run.stdout/stderr` is rendered in the output pane (`:470`) and is **never re-read for logic** — it is not serialized (`RunResult` is `#[derive(Debug)]`-only, no `Serialize`), not fed back into assembly, not persisted (sweep report §1 confirms RAM-only). Some flows (an `ms1`-emitting subcommand) genuinely emit secret-class stdout; whole-scrub is the right fail-closed call. **Bless it.** (One nuance for the plan: scrub happens on Drop/exit, i.e. AFTER the user has dismissed/replaced the result — the output pane reads the live `String` while displayed, then the holder scrubs on next-run-replace or exit. No display regression.)

**2. PendingConfirm struct-vs-explicit — RATIFY the struct+Drop approach (with I1's consumption-pattern correction).** Struct+Drop covers all clear paths (`:1092`, `:1096`, exit) robustly vs explicit `= None` which is miss-prone. Consumers: the only consumers of `PendingConfirm` are the set at `:1045`, the destructure at `:1064`, and the two clears — all in `main.rs`, all surveyed. Promoting tuple→struct touches exactly those sites. **Does not break consumers** EXCEPT the `:1064` destructure (I1). With I1's fix, struct+Drop is correct and superior.

**3. Widget masking correctness — CORRECT.**
- Slug 3 predicate `node_type_is_argv_secret(node)` (`secrets.rs:175` → `SECRET_NODE_TYPES_ARGV`) correctly identifies secret-typed composite nodes (phrase/entropy/xprv/wif/minikey/bip38/electrum-phrase/seedqr per sweep §3) and leaves xpub/path/number plain — verified it's the SAME predicate already used at `:663`/`:672` and in `should_confirm_run:246`, so render-mask co-fires with paste-warn + run-confirm (no split-brain; T6 pins it). **Does NOT mask public fields.**
- Slug 4 predicate `is_xprv_like` (`tree_model.rs:675`): `key_part[1..4] == "prv"` after `rsplit(']')` — true for xprv/tprv (and `[origin]xprv…`), false for xpub-family, empty, raw-hex (verified by the in-file test vectors `:826-833`). Correctly masks only mis-pasted private keys; watch-only xpubs stay readable. Matches the established v0.38.0 slot pattern and `SecretLineEdit` (`.password(true)`).
- `Role::PasswordInput` kittest is the right RED assertion — it's exactly the v0.38.0 slot test idiom (`slot_secret_mask_v0_38_0.rs` T2, verified). **Correct.**

**4. SemVer + gates — CORRECT.** GUI MINOR 0.46.0→0.47.0 (new hygiene behavior, no breaking API). PR + 5-target CI before tag (GUI is PR-gated). NO toolkit pin (pure internal app-state + render; reuses pinned v0.60.0 taxonomy). **NO `schema_mirror` impact confirmed:** no clap flag/option/subcommand/dropdown add/remove/rename — `RunResult`/`PendingConfirm` zeroize and `.password(true)` are internal state + render only; `src/schema/mnemonic.rs` untouched. NEVER cargo-fmt (no fmt gate). Drift tests against pinned v0.60.0 binary (`MNEMONIC_BIN`). M9 + on-disk redactor correctly framed as complete-within-scope (these gaps are surfaces neither reaches — verified against sweep §"two defenses": `zeroize_form_state` iterates only `state.values/slots/positionals/tree`, never `last_run`/`pending`; redactor owns the persist path, and these holders aren't serialized). **Not re-done — extended. Correct.**

**5. No behavior change beyond masking/scrubbing — CONFIRMED.** Input still accepted (widgets keep `text_edit_singleline` semantics, just `.password(true)` glyphs); bytes flow to the same form-state `String`; holders keep cleartext while live and scrub only on replace/clear/exit. Run path, copy buttons, run-confirm modal masking, paste-warn, persist-redaction, M9 sweep all unchanged. Verified.

---

## MINOR (non-blocking; fold into the plan-doc)

- **M1 — line drift.** §2:54 cites the `last_run = Some(result)` store as `:1222`; actual `:1223` (the `result.mask = mask;` at `:1222` shifts it). Update the citation.
- **M2 — tree-render testability seam under-specified (axis-3 T7).** `render_payload` (`tree_form.rs:685`) and `render_node` (`:601`) are **private**; the only `pub` tree-render entry is `render(ui, state, bin)` (`:431`). The slot kittest had a clean `pub fn slot_editor::render` seam; tree_form does not. T7 (slug-4 `Role::PasswordInput`) must either drive through the public `render` with a constructed `FormState`/`TreeState` carrying a Key/KeyQuorum node, OR the plan must expose a narrow `pub(crate)` render-payload seam. The composite T5 is fine (reachable via `pub fn widget::render`/`render_with_dispatch`). The plan-doc (D7 testability seam) should name the concrete tree seam — this is the one place the slot-test analogy doesn't transfer 1:1.
- **M3 — `.password(true)` + paste `response` capture interaction (slug 3).** The composite arm captures `let response = ui.text_edit_singleline(value);` and reads `response.changed()` for paste-warn (`:653-665`). Switching to `egui::TextEdit::singleline(value).password(true)` must preserve the `Response` so paste detection still fires (`TextEdit::singleline(..).password(true).show(ui)` returns the `Response` via `.response`, or `ui.add(TextEdit::…)` returns `Response`). Trivially preservable, but the plan should pin the exact form so the existing paste-warn doesn't silently regress (a kittest/regression assertion on paste-warn co-firing would be belt-and-suspenders).
- **M4 — prompt vs spec on stdout/stderr type.** The review prompt phrased the fields as "`String`"; they are `Vec<u8>` (`runner.rs:30-31`). The spec is correct. Plan's `Zeroize` impl scrubs `Vec<u8>` (valid). No action beyond not transcribing the prompt's "String" into the plan.

---

## Required to reach GREEN

Fold **I1** (specify the `PendingConfirm` consumption-pattern change — whole-struct bind + inner re-clone on Run — so the struct+`Drop` actually compiles; do NOT silently fall back to no-`Drop`). Recommend also folding M1–M3 (M2 is the most consequential for the plan's TDD seam). Then **persist this review verbatim** to `design/agent-reports/cycle15g-...-review.md`, re-dispatch the architect on the folded spec, and re-run R0. Per the standing gate, **no plan-doc advance, no implementer dispatch until 0C/0I.** The architecture is otherwise sound and the priority axis (run-holder scrub completeness) is verified correct — once I1 is folded this lane should converge quickly.
