# SPEC R0 (round 1) — restore `--template` `(none)` unset affordance

- **Cycle:** `restore-form-single-sig-template-leaks-in-md1-mode`
- **Spec under review:** `design/SPEC_restore_template_none_affordance.md`
- **Reviewer:** opus architect, spec R0 hard gate (0C/0I required before any plan-doc / code).
- **Mode:** READ-ONLY, adversarial. Every load-bearing claim re-verified against live source (not the spec's assertions).
- **Source SHAs at review:** mnemonic-toolkit `master @ 97a494a9` (spec committed here); mnemonic-gui `master @ cab940b` (= `mnemonic-gui-v0.56.0`, matches the spec's cited SHA). docs/manual-gui at the `manual-gui-v1.2.0` line (latest tag confirmed `manual-gui-v1.2.0`).

---

## VERDICT: **GREEN — write the plan. 0 Critical / 0 Important.**

The premise is empirically re-confirmed at source. The design is a byte-for-byte mirror of the ratified export-wallet F1 (`EXPORT_WALLET_TEMPLATES` → `RESTORE_TEMPLATES`), every automated gate is inert, the render-scoped-only ruling is correct and precedented, and the Leg-3 byte-identical-transcript claim is SOUND (verified against the `run_multisig` code path, not asserted). Six Minor items follow — all non-blocking hygiene / tracking; fold into the plan-doc. No STOP condition reached; no toolkit `src/` / clap / conditional-projection pressure.

---

## PREMISE RE-CONFIRMATION (the whole fix hinges on it)

**CONFIRMED at source.** The md1-mode single-sig-`--template` refusal is real, exit 2, and fires on the virgin `bip44` before any card decode:

- `crates/mnemonic-toolkit/src/cmd/restore.rs:3068-3074` — inside `fn run_multisig` (spans `3050-3673`; next fn `resolve_seed_entropy` at 3674):
  ```rust
  if let Some(t) = args.template {
      if !t.is_multisig() {
          return Err(ToolkitError::ModeViolation {
              mode: "restore", flag: "--template",
              message: "--template (single-sig) does not apply in multisig --md1 mode; remove it",
          });
      }
  }
  ```
- `ModeViolation → exit 2`: `error.rs:618` (`ToolkitError::ModeViolation { .. } => 2`) — CONFIRMED exact line.
- Dispatch: `if !args.md1.is_empty()` at `restore.rs:314`; a **keyed wallet-policy md1** (`is_wallet_policy()==true`) fails both the `is_singlesig_template` and `is_multisig_template` template-completion predicates → falls through to `return run_multisig(...)` at `restore.rs:349` → the 3068 gate. (Spec §1.2 cites "349" for the `!args.md1.is_empty()` dispatch; the *check* is at 314 and the `run_multisig` *call* is at 349 — mechanism correct, see m3.)
- **Inertness of a passing template:** across the entire `run_multisig` body (3050-3673) `args.template` appears **only at 3068** (grep-verified). A **multisig** template (`is_multisig()==true`) clears the gate and is then **never consumed** — so a multisig `--template` value is purely inert, and clearing it (no `--template`) yields identical output. The keyless template-completion path `run_multisig_template_completion` (1369-3049) references `args.template` **zero** times. → `(none)`/no-template on any multisig-md1 restore is byte-identical to the multisig-template workaround.
- The empirical table in §1.1 (exit 2 for `bip44`/`bip84`; exit 0 for `wsh-sortedmulti` and for no-template, byte-identical) is consistent with the two complementary gates: **373 gate** (single-sig path, `md1` empty) refuses a *multisig* template with `bad(...)` (BadInput exit 1); **3068 gate** (md1 path) refuses a *single-sig* template (ModeViolation exit 2). `--format`'s requires-`--template` gate at `restore.rs:395-400` is in the **single-sig path only** (after the md1 dispatch returns at 349) → unreachable in md1 mode.

**Premise stands. The fix is not moot.** The virgin `bip44` in md1 mode is a real exit-2 trap; `(none)` is the honest escape.

---

## VERIFICATION OF EACH REVIEW ITEM

### (2) A1-APPEND design — CONFIRMED, and the "no other subcommand breaks" claim is TRUE (with one omission)
- `TEMPLATES` (10 values, no `""`) at `mnemonic-gui/src/schema/mnemonic.rs:69`. Restore `--template = FlagKind::Dropdown(TEMPLATES)`, `default_value: None` at **:539/:544** — CONFIRMED.
- The F1 precedent `EXPORT_WALLET_TEMPLATES` (`:103`) is exactly the 10 `TEMPLATES` in order + trailing `""`; its doc-comment already ratifies "the bundle 10 vs export-wallet 11 asymmetry is intended and scoped." `RESTORE_TEMPLATES` does not yet exist (grep-confirmed) → the new-const design is correct and precedented.
- **`Dropdown(TEMPLATES)` has FOUR consumers:** `309` (bundle), `539` (restore), `866` (verify-bundle), `1271` (convert). The plan re-points **only line 539**; bundle/verify-bundle/convert stay on the shared 10-value `TEMPLATES`. So no other subcommand's list breaks. **NOTE (m1):** spec §2.1 says `TEMPLATES` "feeds bundle/verify-bundle" and omits **convert** (`CONVERT_FLAGS`, `:1271`). Immaterial to correctness — convert stays on the shared const — but the plan-doc should enumerate all three untouched consumers.

### (3) Render-scoped-ONLY ruling + the UX question — CORRECT, mirrors ratified F1
- Restore's conditional is exactly ONE rule: `conditional.rs::restore` (**:1006-1012**) pushes `("--from", Required)` iff `!has_value("--md1")`; no `--template` rule. The floor `("restore", 1)` is at `tests/gui_schema_conditional_drift.rs:311` — CONFIRMED. A paired mutex projection (grey `--template` in md1) would force `("restore", 1) → 2` + a toolkit `src/` projected rule → not F1-shaped → correctly REJECTED / out-of-scope (spec §2.3).
- `(none)` alone fully resolves the papercut: selecting it writes `Dropdown("")` → `has_value` false → the flag simply drops from argv → the 3068 gate's `if let Some(t)` is skipped. No conditional rule needed. CONFIRMED sufficient.
- **UX ruling (is leaving the single-sig default present a Critical gap? — NO):** The virgin restore form still materialises `--template = bip44` (opts[0], `flag_defaults.rs:30-31`), so an md1 user must **manually** pick `(none)` or hit the exit-2 refusal. This is **identical in kind to the ratified F1** (export-wallet also keeps `bip44` virgin and requires a manual `(none)` to unlock `--descriptor`). It is a **strict improvement** over the status quo, where no honest escape existed (the only options were a multisig-template lie or the crash), and the CLI error already says "remove it." Not Critical, not Important. The one asymmetry worth tracking: restore's residual is a *hard exit-2 crash-on-Run* vs export-wallet's *soft disabled field* — so the "grey `--template` in md1 mode" nicety has more UX value here. **Recommendation (m6):** file that as a FOLLOWUP now (spec says "MAY"; upgrade to SHOULD-file) so the residual is tracked, not silently dropped.

### (4) Gate-inertness — CONFIRMED inert on every automated gate
- **`schema_mirror`** — GUI-side deser is name-only: `schema_check.rs:98-104` `struct GuiSchemaFlag { name: String }` with the explicit comment "Other fields (required, kind, choices) intentionally not deserialized." Dropdown VALUES are never compared; restore's flag-name set is unchanged. **INERT.** Also: Leg 1 does not bump the GUI's toolkit pin, so `schema_mirror` runs against the same v0.75.0 binary → GREEN.
- **Template-partition consts** (`SINGLE_SIG_TEMPLATES` / `MULTISIG_TEMPLATES`, `conditional.rs:30,40`) — derived from `CliTemplate::is_multisig()`, distinct from any per-flag opts. `RESTORE_TEMPLATES` is a new per-flag const. **INERT.** (Confirmed `tr-sortedmulti-a` and `wsh-sortedmulti` are both in `MULTISIG_TEMPLATES` → the tutorial workaround values clear the 3068 gate today.)
- **`gui_schema_conditional_drift`** — synthesises from the toolkit-projected restore rule (presence of `--md1`), never enumerates GUI `--template` opts; rule count stays `1 == ("restore", 1)`. **INERT.**
- **`gui_render_emit.rs`** — `grep -ni restore tests/gui_render_emit.rs` → **ZERO hits**. **INERT** (see item 5).
- **61-form gallery PNG** (`figures/gui/mnemonic-restore.png` + the GUI kittest `gui_form_snapshots`) — closed virgin form, opts[0]=bip44 unchanged, `default_value: None` unchanged → **byte-identical** (APPEND is load-bearing here; a PREPEND would move it — cell 5 tripwire). **INERT.**
- `argv_assembler*`, `hint_text_defaults`, `build_descriptor_*` value pins, `gui_render_faithfulness` — all inert (same reasoning as F1 §2/§4; restore is not special). CONFIRMED.

### (5) T3 — no `gui_render_emit.rs` restore pin — CONFIRMED
`gui_render_emit.rs` pins inspect (mnemonic/mk/ms), build-descriptor, export-wallet (`:95` the `--template` line), compare-cost, and references bundle (`:163`) + a plain form. **Restore is absent** (zero grep hits). So restore's GUI-side placement guard is the NEW TDD append-pin test (§5 cell 5), and its exact-ASCII structural render lives only in the toolkit `.gui` transcript (re-pinned in Leg 2). CONFIRMED. **NOTE (m2):** spec §2.2's emit-pin enumeration ("only inspect / build-descriptor / export-wallet / compare-cost") under-lists (also bundle + ms/mk inspect) — but the load-bearing claim (restore not pinned) is TRUE.

### (6) The 3-leg ripple + the Leg-3 BYTE-IDENTICAL claim — SOUND
- **Leg 2 pin + surfaces — verified:**
  - `docs/manual-gui/pinned-upstream.toml:55` = `tag = "mnemonic-gui-v0.56.0"` → bump to v0.57.0. CONFIRMED.
  - `docs/manual-gui/transcripts/gui/mnemonic-restore.gui:4` = `--template  dropdown[bip44,…,tr-sortedmulti-a]  -> bip44`; re-pin appends `,(none)`, `-> bip44` stays. Regenerated by `verify-examples-gui` (Makefile `:306-307`). CONFIRMED.
  - **Coverage anchor:** `check_gui_schema_coverage.py::kebab("") == ""` (`out.strip("-")` on empty → `""`), and `anchor(variant) = "mnemonic-restore-template" + "-" + kebab("")` = **`mnemonic-restore-template-`** (trailing dash). The `--template` section at `4d-restore.md:173`, outline anchors `bip44`(:185)…`tr-sortedmulti-a`(:194); a `(none)` variant → REQUIRED anchor `#mnemonic-restore-template-` else `gui-schema-coverage` REDs. The F1 precedent is already SHIPPED at `45-export-wallet.md:48-64` (the `(none)` prose + `#mnemonic-export-wallet-template-` anchor). CONFIRMED — REQUIRED edit correctly identified. The `:178-179` "Same 10 values as bundle --template" correction is also correctly flagged.
  - `expected_gui_schema_inventory.json` regen is **documentary / un-gated** — CONFIRMED by the shipped Leg-2 whole-diff review (`gui-form-renders-leg2-postimpl-whole-diff-review.md:54`: "No test/CI consumes it … It is a hygiene snapshot"). Does not RED.
- **Leg-3 byte-identical-transcript claim — CONFIRMED SOUND (not merely asserted):**
  - All 6 steps have `Drive::TypeMd1Chain { flag: "--md1", … }` → **all are `--md1` multisig restores; NONE is a genuine single-sig restore** (a single-sig restore has no `--md1`; dropping `--template` there WOULD change output to all-four). Manifest cites all exact: `tut-j2-08-restore` (:267-268, `wsh-sortedmulti`), `tut-j3-13` (:288-289, `wsh-sortedmulti`), `tut-j4-17` (:306-307, `tr-sortedmulti-a`), `tut-j4-nums` (:336-337, `tr-sortedmulti-a`, capture=false), `tut-j5-23-restore-descriptor` (:345-348, inline `wsh-sortedmulti` + `--format descriptor`), `tut-j5-24-restore-core` (:350-353, inline `wsh-sortedmulti` + `--format bitcoin-core`).
  - In `run_multisig`, `args.template` is referenced **only at the 3068 gate**; the current workaround values are all `∈ MULTISIG_TEMPLATES` → clear the gate → inert. `(none)` drops the flag → gate skipped → **identical stdout/stderr/exit**. In the keyless template-completion path, `args.template` is never referenced at all. → byte-identical either way.
  - **J5 `--format` in md1 mode:** the `--format`-requires-`--template` gate (`restore.rs:395`) is in the single-sig path (unreachable once the 314→349 md1 dispatch returns). `run_multisig` supports `--format` via `build_multisig_import_payload`. So `(none) + --format <X>` in md1 mode = exit 0, unchanged payload. CONFIRMED.
  - Therefore `verify-tutorial-transcripts` stays GREEN with **no transcript re-pin**; only the 5 shot-bearing steps' `-form`/`-run` PNGs (= **10 PNGs**, j4-nums excluded as capture=false) re-pin under `verify-tutorial-figures`. The spec's PNG list matches exactly. The spec's STOP condition #3 (a restore transcript changing under `(none)` → investigate) correctly guards a surprise.

### (7) Secret hygiene — CONFIRMED unaffected
`--template` is `secret: false` (`:538`); the appended `""` sentinel is presence-suppressed (never an argv value, never in a masked copy-command). Restore's classified-secret surface is untouched: `--passphrase` (`:667`, `secret: true`) and `--passphrase-stdin` (`:677`, `secret: true`) — both confirmed `secret: true`; plus `--from`'s composite secret nodes. No owned secret introduced; no zeroize/redaction change. (m5: spec §7 cites `:672`/`:683`; actual `:667`/`:677` — ~5-line drift, substance correct.)

### (8) Release plan — COHERENT, versions correct
Latest tags confirmed: `mnemonic-gui-v0.56.0` and `manual-gui-v1.2.0`. → `mnemonic-gui-v0.57.0` (MINOR: a user-visible schema-VALUE affordance addition, no crates.io publish — appropriate) → `manual-gui-v1.3.0` (MINOR content release). No `mnemonic`/`md`/`ms`/`mk` clap-surface change → no toolkit crate bump, no crates.io publish, `docs/manual/` (non-GUI manual) untouched, CLI-reference mirror invariant does not fire. All correct. Ordering (Leg 1+3a → tag GUI → Leg 2+3b → manual-gui release) is coherent.

### (9) Blocking issues for a plan-doc — NONE.

---

## FINDINGS BY SEVERITY

### Critical — NONE.
### Important — NONE.

### Minor
- **m1** — §2.1 undercounts the shared-`TEMPLATES` consumers: it feeds bundle (`:309`), verify-bundle (`:866`) **and convert (`:1271`)** — spec omits convert. Immaterial (only restore's `:539` re-points), but the plan-doc should enumerate all three untouched consumers to make the "no other subcommand breaks" claim explicit.
- **m2** — §2.2 emit-pin enumeration under-lists (`gui_render_emit.rs` also references bundle + ms/mk inspect). The load-bearing claim (restore not pinned) is TRUE; enumeration only.
- **m3** — §1.2 cites the md1 dispatch at `restore.rs:349`; the `!args.md1.is_empty()` check is at `:314` and the `run_multisig` call is at `:349`. Mechanism correct; the plan-doc should cite both live line numbers.
- **m4** — The reference `--format` section (`4d-restore.md:84-92`) states "`--format` with no `--template` → exit 2" (scoped to single-sig by the adjacent multisig sentence). After Leg-3 teaches `(none) + --format` in md1 mode, a one-line clarification that **md1 mode needs no `--template`** would keep reference↔tutorial consistent. Pre-existing, un-gated (coverage checks anchors, not prose), non-blocking — optional plan-doc line.
- **m5** — §7 passphrase citations (`:672`/`:683`) drift ~5 lines from actual (`:667`/`:677`); `secret: true` + untouched substance correct.
- **m6** (UX tracking) — render-scoped-only leaves the `bip44` virgin default present → md1 user must manually pick `(none)` to avoid the exit-2 refusal. Mirrors ratified F1; the mutex (grey `--template` in md1) is correctly out-of-scope. **SHOULD file** the "grey `--template` in md1 mode" FOLLOWUP now so the residual is tracked (spec's "MAY" → "SHOULD").

---

## EXPLICIT RULINGS (requested)

- **Render-scoped vs mutex UX:** RENDER-SCOPED `(none)` alone is correct and sufficient — NOT a Critical UX gap. It is byte-for-byte the ratified F1 pattern (virgin default present, manual `(none)` selection), a strict improvement over the no-escape status quo, and the mutex alternative is a forbidden toolkit-`src/` / floor-breaking change. File the greying FOLLOWUP (m6) to track the residual.
- **Leg-3 byte-identical-transcript claim:** SOUND / CONFIRMED at source. All 6 steps are `--md1` multisig restores passing a `MULTISIG_TEMPLATES` value that clears the 3068 gate but is never consumed; `(none)` drops the flag and skips the gate → identical stdout/stderr/exit. `--format` is unaffected in md1 mode (its requires-`--template` gate is single-sig-path-only). Only the 10 `-form`/`-run` PNGs re-pin; transcripts stay byte-stable. STOP-#3 correctly guards a surprise.

**GATE: GREEN. Proceed to the implementation plan-doc.** Fold m1–m6 (citation live-re-grep + enumerate all 3 `TEMPLATES` consumers + the m4 `--format` clarification + file the m6 FOLLOWUP). Per project convention, re-dispatch this reviewer after the plan-doc is drafted (reviewer-loop continues after every fold).
