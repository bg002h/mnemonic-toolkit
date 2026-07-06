# batch `tutorial_surfaced_fixes_batch` — P1.3 per-phase R0 (round 1)

**Scope:** restore `--template` `(none)` unset affordance (A1-APPEND, `RESTORE_TEMPLATES`) + verify the R-A 32-form gallery re-pin landed clean.
**Reviewer:** opus architect, adversarial (gates re-run, not trusted).
**Under review:** mnemonic-gui branch `feat/tutorial-surfaced-fixes`, commit `390df12` (restore `(none)`) atop `8cbcc34` (R-A 32-form gallery re-pin).
**Authority:** plan §P1.3 (`design/IMPLEMENTATION_PLAN_tutorial_surfaced_fixes_batch.md:109-114`) + `design/SPEC_restore_template_none_affordance.md` (+ its R0) + the P1.2 R0 (`design/agent-reports/batch-p1.2-r0-round-1.md`, which ratified the 32/Option-A gallery set).
**Toolchain:** stable 1.95.0 (GUI MSRV 1.88; `RUSTUP_TOOLCHAIN=stable`), `--jobs 2`.

---

## VERDICT: **GREEN — advance to P1.4.** 0 Critical / 0 Important / 1 informational (non-blocking).

Every gate re-run by the reviewer, not read from the transcript. RED-first was reproduced empirically (0/5 at the parent schema). Gallery inertness confirmed under the real Cpu software rasterizer (15.66 s render, 0 diffs). The tree was left clean at `390df12`.

---

## 1. The A1-APPEND ruling — CORRECT

The `390df12` diff touches exactly two files (`src/schema/mnemonic.rs` +49, `tests/restore_template_none.rs` +313) — no other surface moved.

- **New `RESTORE_TEMPLATES` const** (`src/schema/mnemonic.rs:141`), sited immediately beside `EXPORT_WALLET_TEMPLATES` (`:103`). Body = the 10 shared `TEMPLATES` values **in order** (`bip44 … tr-sortedmulti-a`) **+ a trailing `""`** unset sentinel. Doc-comment mirrors the export-wallet append-not-prepend rationale and adds the restore-specific semantics (single-sig `(none)` = emit all four; md1 `(none)` = drop the single-sig template the CLI refuses at exit 2).
- **`RESTORE_FLAGS[--template]`** (`:581`): `Dropdown(TEMPLATES)` → `Dropdown(RESTORE_TEMPLATES)`; **`default_value` STAYS `None`**; `opts[0] == "bip44"` (virgin default unchanged). Help text gains a non-load-bearing `(none)` note (no gate pins restore help text — grep-confirmed).
- **The 3 other `TEMPLATES` consumers STAY shared:** bundle (`:347`), verify-bundle (`:911`), convert (`:1316`) — all still `Dropdown(TEMPLATES)`. **export-wallet stays on `EXPORT_WALLET_TEMPLATES`** (`:1470`). The complete `--template` Dropdown consumer set was enumerated; only restore moved.
- **`TEMPLATES` itself unchanged at 10 values** (`:69`).
- **No widget change** — verified `src/form/widget.rs:528/:535` already maps `""` → `(none)` via `display_or` for both the closed `selected_text` seam and each open-popup `selectable_value`. The `""` sentinel is handled generically (archetype/export-wallet precedent).
- **No conditional change** — `src/form/conditional.rs:1006` (`restore`) still emits the single rule (`not(has_value --md1) → {--from, Required}`); it never reads `--template`.

**Ruling: the A1-APPEND is faithful to F1/`EXPORT_WALLET_TEMPLATES` and to spec §2. The restore↔bundle 11-vs-10 asymmetry is intended and scoped; `TEMPLATES` and its partition consumers are untouched.**

## 2. The 5 TDD cells — genuinely RED-first, real assertions (not tautologies)

**Empirical RED-first proof:** reverted ONLY `src/schema/mnemonic.rs` to the parent `8cbcc34` (restore back on `Dropdown(TEMPLATES)`, 10 values), kept the new test, ran `cargo test --test restore_template_none` →
`test result: FAILED. 0 passed; 5 failed`. Confirms the report's "0/5 pre-impl". Cells 1/2/3 panic at the `select_template(_, "(none)")` step (no such node exists at parent), cell 4 at the open-popup `(none)` count, cell 5 at `opts.len()==11`. Schema restored to `390df12`; tree clean.

- **Cell 5 (`restore_templates_append_census`) — real.** Asserts `RESTORE_TEMPLATES.len()==11`, `opts[0]=="bip44"`, `opts.last()==Some("")`, `opts[..10]==bundle_opts` (cross-checked against the LIVE `bundle --template` opts, not a self-referential copy), `bundle_opts.len()==10`, and — via an independent `render_whole_form_harness` settle — that a virgin restore materialises `--template=="bip44"`. Two independent oracles (const shape + rendered virgin state) → not a tautology; it is the PREPEND tripwire.
- **Cell 3 (`has_value_tracks_transitions_and_conditional_is_template_independent`) — real, non-vacuous.** `visibility_projection` calls the production `sub.conditional` over every flag. `p_bip44 == p_none == p_bip49` genuinely tests template-independence (would go RED if the conditional ever read `--template`). The non-vacuity guard `visibility_projection(with_md1) != p_none` differs from `p_none` in exactly one input — the presence of `--md1` — so the `assert_ne!` is 100 % attributable to `--md1` reactivity (with `--md1` present, `--from` drops its `Required` effect). Verified against `conditional.rs:1006`. Not a constant/tautology.

## 3. Batch-inertness — the load-bearing P1.3 claim, re-verified (4/4)

Restore must add ZERO surface movement atop the post-reveal (R-A) baseline. All four re-run by the reviewer:

- **(a) `gui_form_snapshots` = 0 movement.** Ran `GUI_SNAPSHOTS=1 WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1` (NO `UPDATE_SNAPSHOTS`), `--jobs 2`: **exit 0, 2 passed, 15.66 s** (a real 61-form render, not the early-return skip; `device_type==Cpu` assert is unconditional under `GUI_SNAPSHOTS=1` and held → llvmpipe/Cpu confirmed). **0 `.diff.png` produced; restore PNG byte-stable** at its R-A baseline (closed combo, `opts[0]=bip44` virgin). The help-text append does NOT move the PNG (help is a tooltip, not inline) — empirically confirmed by the byte-clean compare. This doubles as the item-4 idempotency check (committed PNGs re-render byte-identical). `.new.png` scratch renders (gitignored) were cleaned; tree left clean.
- **(b) `gui_render_emit` — 0 restore pins + green.** grep of `tests/gui_render_emit.rs` → zero `restore` hits (restore is not among the inspect/build-descriptor/export-wallet/compare-cost pins). Suite: **15 passed**.
- **(c) `gui_schema_conditional_drift` floor `("restore",1)` — unchanged-green.** `:311` still `("restore", 1)`; suite **5 passed**. (The render-scoped ruling holds — no projected rule added.)
- **(d) `schema_mirror` — unchanged-green.** **21 passed.** The `""` sentinel is a GUI-only Dropdown **VALUE**, not a flag NAME; the flag-name gate never compares values (export-wallet/`EXPORT_WALLET_TEMPLATES` precedent). Template-partition (`SINGLE_SIG`/`MULTISIG`) derives from `is_multisig()`, not per-flag opts → inert.

**No drift gate tripped and the gallery did not move → the inertness claim stands.**

## 4. The R-A re-pin landed clean (32 set)

`git show 8cbcc34 --name-only` = **exactly 32 files, ALL `tests/snapshots/forms/*.png`** (0 non-conforming: no `.gui`, no `.new/.diff.png`, no source; `32 files changed, 0 insertions(+), 0 deletions(-)` = binary-only).

Membership verified against the P1.2-R0-ratified Option-A set:
- **32 = the 28 `<masked>`-emit forms ∪ the 4 composite-default-secret forms.** The 4 composites all PRESENT: `mnemonic-final-word`, `mnemonic-seed-xor-split`, `mnemonic-seedqr-encode`, `mnemonic-ms-shares-split`.
- **`mnemonic-addresses` PRESENT** (its `--language`-wide eye is sub-dify-0.6 but its bytes changed — the full-UPDATE regen caught it, not threshold selection).
- **`mnemonic-export-wallet` ABSENT** (0 slot rows on load, no visible secret widget → unmoved).
- **No GL-local noise form wrongly committed** — `mk-encode` and `xpub-search-address-of-xpub` both ABSENT (correctly reverted).

Idempotency independently confirmed by §3(a)'s byte-clean compare-only re-render.

## 5. Full suite + toolchain gates

- **`cargo test --jobs 2` (stable 1.95.0):** **667 passed; 0 failed** across 77 binaries; exit 0. New `restore_template_none` = **5/5**. (Captured with a full log — an initial run had been `| tail -80`-truncated, masking the true count/exit; re-run un-piped to get ground truth.)
- **`cargo clippy --all-targets -- -D warnings`:** exit 0.
- **`cargo clippy --no-default-features -- -D warnings`:** exit 0.
- **`cargo build --no-default-features`** (headless, zero wgpu/winit): exit 0.
- **Tutorial corpus UNTOUCHED** — `git diff 8cbcc34..390df12 -- tests/snapshots/tutorial tests/tutorial` = empty (that re-drive is P1.4/R-B, correctly deferred).

## 6. Findings by severity

- **Critical:** none.
- **Important:** none.
- **Minor / informational (non-blocking):** SPEC `§2.1` prose enumerates the shared-`TEMPLATES` consumers as "bundle/verify-bundle" (2), but there are **3** (bundle/verify-bundle/**convert**). The IMPLEMENTED `RESTORE_TEMPLATES` doc-comment and plan §P1.3 both correctly list all three, and the code leaves all three on the shared const — so the implementation is right; only the spec-body prose undercounts. No action required for P1.3; optionally correct the spec prose if it is lifted again.

---

## Gate-by-gate ledger (all reviewer-run)

| gate | result |
|---|---|
| full suite `--jobs 2` (stable) | 667 passed / 0 failed / exit 0 |
| `restore_template_none` (5 cells) | 5/5 GREEN; 0/5 at parent (RED-first proven) |
| `gui_form_snapshots` (Cpu, no update) | 0 movement, 0 `.diff.png`, exit 0 |
| `gui_render_emit` | 15 passed; 0 restore pins |
| `gui_schema_conditional_drift` | 5 passed; floor `("restore",1)` intact |
| `schema_mirror` | 21 passed (VALUE inert) |
| `gui_render_faithfulness` | 2 passed |
| `hint_text_defaults` | 10 passed |
| clippy `--all-targets` / `--no-default-features` | exit 0 / exit 0 |
| headless build `--no-default-features` | exit 0 |
| tutorial corpus | UNTOUCHED (P1.4) |
| R-A re-pin | 32 PNGs, clean set, idempotent |

**GATE: GREEN. Cleared to advance to P1.4 (the combined tutorial re-drive).** Repo left clean at `390df12`.
