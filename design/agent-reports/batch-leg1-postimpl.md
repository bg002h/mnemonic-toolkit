# batch `tutorial_surfaced_fixes_batch` — Leg 1 (GUI) whole-leg post-implementation review

- **Reviewer:** opus architect — mandatory, non-deferrable post-implementation whole-diff review (adversarial, independent). Every gate re-run by me, not trusted from the per-phase reports.
- **Under review:** `mnemonic-gui` branch `feat/tutorial-surfaced-fixes`, tip `4cd878a`, off `origin/master 40156b0`. Whole-leg diff `40156b0..4cd878a` = 73 files, +1572/−64.
- **Authority:** the batched plan (`IMPLEMENTATION_PLAN_tutorial_surfaced_fixes_batch.md`), the two specs (`SPEC_gui_secret_reveal_toggle.md`, `SPEC_restore_template_none_affordance.md`), the per-phase R0s (`batch-p1.{2,3,4}-r0-round-1.md`, all GREEN).
- **Environment:** `RUSTUP_TOOLCHAIN=stable` (repo default is nightly; GUI MSRV 1.88), `--jobs 2`; visual gates `WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1` with `device_type==Cpu` (llvmpipe) asserted by the harness; pinned CLI on `$PATH` = `mnemonic 0.75.0` (matches `pinned-upstream.toml` / `mnemonic-toolkit-v0.75.0`).

---

## VERDICT: GREEN — P1.5 tag cleared. 0 Critical / 0 Important / 1 Minor (latent, non-blocking — M-1, file-as-FOLLOWUP).

The three changes compose into one correct system; only the two intended features change shipped behavior; the whole moved corpus is secret-clean; no gate regressed. The version/CHANGELOG/pin/FOLLOWUP-flip obligations are correctly still OUTSTANDING (not wrongly pre-applied in the leg). Proceed to P1.5 (release commit) then tag `mnemonic-gui-v0.57.0`.

---

## 1. Coherence — the 3 changes compose (RULING: coherent, no phase contradicts another)

- **Reveal (P1.2, `d8607a2`) — display-only 👁.** Moved the 32-form gallery (`8cbcc34`, R-A re-pin) — verified `git show 8cbcc34` = **exactly 32 `tests/snapshots/forms/*.png`, binary-only** (0 non-PNG, 0 src). 32 = 28 `<masked>`-on-load site-#1 forms ∪ 4 composite-default-secret forms (`final-word`, `seed-xor-split`, `seedqr-encode`, `ms-shares-split`), `export-wallet` correctly absent — matches the P1.2-R0-ratified Option-A set. The `.gui` structural set stays 28 (composite eye not depicted); PNG set is 32. No conflict.
- **Restore `(none)` (P1.3, `390df12`) — A1-append.** Touches only `src/schema/mnemonic.rs` (new `RESTORE_TEMPLATES` = 10 shared `TEMPLATES` in order + trailing `""`) + its test. `opts[0]` stays `bip44`, `default_value` stays `None`, shared `TEMPLATES`/`SINGLE_SIG`/`MULTISIG` untouched.
- **Combined re-drive (P1.4, `4cd878a`) — test/manifest-only.** `git show 4cd878a` touches **zero `src/`** (correct for P1.4); 26 tutorial PNGs re-pinned (4 reveal `-form` + 10 restore `{form,run}` + 12 eye-chrome). The manifest switches the 6 restore steps to Template `(none)` (the `""` sentinel) AND marks the 4 capture-true secret-value steps `reveal: true`. Both fixed flows are exercised by the SAME tutorial re-drive — the reveal shows what to type, the restore `(none)` teaches the clean md1 restore. No phase undoes another; the two features are orthogonal (reveal = ctx-transient display flag; restore = schema Dropdown value) and jointly demonstrated.

Whole-leg `src/` set = exactly the **6** expected files: `secret_widget.rs`, `slot_editor.rs`, `widget.rs`, `render_emit.rs`, `app_window.rs`, `schema/mnemonic.rs`.

## 2. Behavior-preservation — ONLY the 2 intended features change shipped behavior (RULING: preserved)

**(a) Reveal is DISPLAY-ONLY — independently proven.** `grep` of all `src/` for the reveal-state accessors (`revealed_field`/`reveal_field_key`/`clear_revealed_field`/`clear_reveal_on_*`/`reveal_toggle`) returns readers in EXACTLY four files: `secret_widget.rs` (defs), `slot_editor.rs` (#2), `widget.rs` (#3), `app_window.rs` (2 auto-hide seams). **No masking/redaction/persistence surface reads reveal:** `grep reveal src/secrets.rs src/form/invocation.rs src/persistence.rs` → the single hit `invocation.rs:518` is a doc-comment ("the deliberate-reveal copy" = the pre-existing real copy-command), not a functional read. Reveal is **not** a `FormState`/`schema` field (`grep reveal src/schema/mod.rs` empty) → structurally cannot serialize. The run-confirm modal, argv echo, copy-command, paste-warn, persistence and exit-sweep all stay masked/redacted unconditionally. Confirmed live: the reveal `-modal`/`-run` shots are re-masked (opened, §3) and `ui_harness_i3_secret_nopersist` is **7/7 unchanged**. Auto-hide is wired at 4 triggers (Run dispatch `app_window.rs`; tab/sub switch `clear_reveal_on_form_change`; blur `clear_reveal_on_blur`; window-focus-loss inside `reveal_toggle` — final predicate `window_focused && (latched || hold)`). Tap-no-latch is the built-in `clicked() && !clicked_by(Primary)` FAKE_PRIMARY discriminator (P1.2 R0 verified against egui 0.31.1 source; no fallback downgrade taken).

**(b) Restore `(none)` is A1-append.** `opts[0]=="bip44"`, `default_value: None` (virgin still materializes `bip44`), the 3 shared-`TEMPLATES` consumers (bundle/verify-bundle/convert) + export-wallet untouched, restore `conditional.rs` keys only on `--md1` (never reads `--template`). `restore_template_none` 5/5 (RED-first proven at P1.3: 0/5 at parent).

**(c) Nothing else changes runtime behavior.** `app_window.rs` (+9) = two additive `clear_*` calls at existing seams, no existing logic modified. `render_emit.rs` = additive emit projection (`flag_has_reveal_eye` + ` [reveal]` ASCII marker on masked rows/secret positionals) — headless emit path only, no live-GUI behavior. `widget_secret_mask_cycle15g.rs` (+6/−4) = benign source-assertion pin update (`.password(is_secret_node&&!reveal)` — reveal only ANDs, never widens maskedness). No accidental change to any non-secret widget (the composite eye gates on `is_secret_node`; slot eye on the secret arm only; the `(Path,hint)` arm untouched).

## 3. WHOLE-CORPUS SECRET HYGIENE (RULING: CLEAN — no leak; the final pre-tag gate passes)

**PNGs opened and read visually:**
- `tut-j1-01-bundle-single-sig-form` — slot `@0.phrase` shows **S0 in plaintext** (scrolled `…abandon abandon about`), single field; `--passphrase` empty; **Preview line `--slot ••••` masked**. Exactly one field revealed.
- `tut-j2-07-bundle-all-seeds-form` — slot `@0`/`@1` MASKED (`••••••••`), slot `@2` (last-driven) **S2 plaintext** (`…letter advice cage above`); Preview `--slot •••• --slot •••• --slot ••••`. **Single-revealed-field invariant visible.**
- `tut-j1-01-…-run` + `tut-j2-07-…-run` — form fields re-masked, argv echo masked, stderr is the **value-free** `secret material on argv (--slot @N.phrase=)` advisory, stdout carries only the intended PUBLIC `ms1/mk1/md1` cards. Run auto-hid the latch. No plaintext.
- Gallery `mnemonic-inspect` — the secret `--ms1` field carries the 👁 eye with an **empty/masked** value (gallery uses no real secret; `--reveal-secret` is an unrelated real flag checkbox).
- Eye-chrome catch-up `tut-ch0-00-orientation-form` — `--passphrase` row gained the 👁 next to an empty masked field; no revealed value.

**Transcripts + fixtures:** `grep` of all committed `tests/snapshots/` for `abandon abandon`/`legal winner`/`letter advice`/`zoo zoo` → none; `\b[xt]prv[0-9A-Za-z]{50,}` → none. `fixtures_carry_no_secret_material` GREEN. The plaintext seeds exist ONLY as rasterized pixels in the 4 allowlisted reveal `-form` PNGs.

**Gate shape:** allowlist stays `[S0,S1,S2]`. The parameterized `assert_no_plaintext` loosens ONLY the filled-form checkpoint (`if Some(val)==revealed { continue }`) — the populated-pane and confirm-modal checkpoints stay UNCONDITIONALLY strict; the skipped value is itself allowlist-gated by `secret_values_are_allowlisted`. Word-disjointness (`abandon`/`legal`/`letter`) keeps the other-secret probes teeth. `allowlist_checker_bites_on_non_allowlisted_secret` proves the checker BITES (incl. the reveal-marked arm). The reveal never reaches a non-demo secret anywhere.

## 4. No-regression (RULING: no regression)

Ground-truth (un-piped) `cargo test --jobs 2` (stable): **670 passed, 0 failed** across 77 binaries. Per-binary:

| gate | result | meaning |
|---|---|---|
| full suite | 670 passed / 0 failed | — |
| `secret_reveal_toggle` | 12 | reveal core |
| `restore_template_none` | 5 | restore `(none)` |
| `gui_render_faithfulness` | 2 | eye modelled non-vacuously (site-#1 always-eye; composite carved to kittest; discrimination arm) |
| `gui_render_emit` | 15 | ` [reveal]` marker pins |
| `ui_harness_i3_secret_nopersist` | 7 (**unchanged**) | 64-secret never-persist net structurally unaffected |
| `schema_mirror` | 21 (**unchanged**) | reveal adds no flag name; restore `""` is a GUI-only Dropdown VALUE — the gated template parity is the SINGLE_SIG/MULTISIG partition (consts unchanged), not per-flag opts (export-wallet `""` precedent) |
| `gui_schema_conditional_drift` | 5 (**unchanged**) | restore floor `("restore",1)` intact |
| `widget_secret_mask_cycle15g` | 9 | composite mask-gate source pin |
| `gui_form_snapshots` (Cpu, no-update byte-compare) | 2 passed, 61 forms, 0 diff | 32-form baseline **byte-stable + idempotent** |
| `gui_tutorial_snapshots` (Cpu, 50-PNG byte-compare, 145 s) | 12 passed incl. reveal-census + budget + BITE negatives | corpus **byte-deterministic**; 50 shots / 27.1 MiB ≤ 32 MiB |
| clippy `--all-targets` / clippy `--no-default-features` | exit 0 / exit 0 | — |
| build `--no-default-features` (headless) | exit 0 | zero wgpu/winit |

The faithfulness gate models the eye on both sides (`flag_has_reveal_eye` vs `observe_reveal_eye` by exact glyph) with a dedicated non-vacuity negative — non-vacuous. Tree is clean; my byte-compare scratch `.new.png` (gitignored) were removed.

## 5. M-1 latent minor — RULE: FILE a hardening FOLLOWUP (do NOT expand P1.5 with code)

The filled-form loosening skips by VALUE-equality (`Some(val)==revealed`), so a future reveal-marked step driving the SAME phrase into two secret fields would skip the still-masked twin's probe. **Inert today:** no reveal-marked step repeats a phrase (j1-01=S0; j2-02/03=S0; j2-07=S0/S1/S2 distinct); the single-latch invariant is core-enforced (one ctx-transient `Option<Id>`, `set_revealed_field` overwrites) so only ONE field can render plaintext; the pane/modal checkpoints are unconditionally strict; and the 61-form masking gallery would catch a genuine masked-field leak.

**Recommendation: FILE, not harden-now.** Rationale: (a) it is a TEST-ORACLE robustness gap, not shipped behavior, and structurally cannot fire today; (b) a real hardening (scope the skip to the last-matching drive INDEX, not value) needs a red-proven twin-phrase test to be meaningful, which re-enters the P1.4 review loop — whereas P1.5 is meant to be code-free ship-prep; (c) three independent nets already cover the real leak surface. The FOLLOWUP entry (e.g. `gui-tutorial-reveal-skip-value-vs-index`) lands in the P1.5 release commit alongside the status flips. It does NOT gate the tag.

## 6. What's owed at the release commit (P1.5) — NONE wrongly pre-applied

Confirmed the leg is still at pre-release state (correct — status-flip discipline flips at ship):
- `Cargo.toml` version = **`0.56.0`**; `Cargo.lock` `mnemonic-gui` = **`0.56.0`** — NOT prematurely bumped.
- `README.md` install pin = **`mnemonic-gui-v0.56.0`** — NOT prematurely bumped.
- CHANGELOG top entry = `[0.56.0]` — no `0.57.0` entry yet.
- FOLLOWUPS: `restore-form-single-sig-template-leaks-in-md1-mode` = **"FIX IN FLIGHT (this cycle)"**; `gui-secret-reveal-toggle` = **"open (flips to RESOLVED in v0.57.0 shipping commit)"** — both correctly still un-flipped.

**Owed at the P1.5 release commit:**
1. `0.56.0 → 0.57.0` in `Cargo.toml` + `Cargo.lock`.
2. CHANGELOG `[0.57.0]` entry — SemVer-MINOR; cover: reveal hygiene model (display-only; hold-primary + bounded-latch fallback; single-revealed-field; auto-hide ×4; **no wall-clock timeout**; **tap-does-not-latch**), restore `--template (none)` A1-append, the combined tutorial re-drive (4 reveal + 6 restore-`(none)` + eye-chrome), gates green (suite/clippy×2/headless/snapshots/tutorial-snapshots).
3. `README.md` self-pin `v0.56.0 → v0.57.0`.
4. Flip **both** FOLLOWUPS → RESOLVED: `restore-form-single-sig-template-leaks-in-md1-mode` and `gui-secret-reveal-toggle`.
5. (from §5) file the M-1 hardening FOLLOWUP.
6. The tag run MUST verify `snapshots` == success AND `tutorial-snapshots` == success (both re-run GREEN here).

Note the deferred fast-follows correctly filed and NOT closed this cycle: `gui-secret-reveal-tree-key-sites` (#4/#5), `gui-secret-reveal-latch-timeout`, `restore-md1-template-mutex-projection`.

## 7. Findings by severity

- **Critical:** none.
- **Important:** none.
- **Minor:** **M-1** (latent, defense-in-depth) — filled-form `assert_no_plaintext` skip is value-equality; a future twin-phrase reveal step could skip a masked twin's probe. Inert today (single-latch core-enforced; pane/modal strict; gallery net). Recommendation: FILE (§5); lands in P1.5; does not gate the tag. Cite: `tests/gui_tutorial_snapshots.rs` filled-form checkpoint + `tests/tutorial/mod.rs::revealed_value`.
- **Informational** (carried from P1.3 R0, spec-prose only): `SPEC_restore_template_none_affordance.md §2.1` undercounts the shared-`TEMPLATES` consumers as 2 (bundle/verify-bundle) — there are 3 (+convert). The implemented `RESTORE_TEMPLATES` doc-comment and plan list all 3 correctly; code is right. Optional spec-prose fix if lifted again.

## Bottom line

Whole-leg diff is coherent, behavior-preserving except the two intended features, and secret-clean across the entire moved corpus. 670/0 suite, both visual gates byte-deterministic and GREEN, I3/schema_mirror/conditional_drift unchanged, clippy×2 + headless clean. Version/CHANGELOG/pin/FOLLOWUP-flip obligations are correctly still outstanding. **GREEN — clear to author the P1.5 release commit and tag `mnemonic-gui-v0.57.0`.** Repo left clean (scratch `.new.png` removed; HEAD `4cd878a`; `git status` empty).
