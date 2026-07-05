# F1 pre-implementation mini-R0 — export-wallet `--template` unset affordance (A1 vs A2, BINDING)

- **Reviewer:** opus-tier architect (F1 mini-R0; the cycle's ONLY src-behavior change; RATIFICATION 4 refine-within-constraints authority).
- **Date:** 2026-07-05.
- **Mode:** READ-ONLY on both repos. Read from `master` via `git show` (GUI working tree is on `feat/gui-example-leg1`, a parallel implementer's branch — NOT touched). `master` and `feat/gui-example-leg1` both at `0d4429d` at review time.
- **Source SHAs (grep-verified this review):** mnemonic-gui `master@0d4429d` (v0.55.0 base); mnemonic-toolkit `master@be876803` (docs/manual-gui).
- **Artifacts:** `IMPLEMENTATION_PLAN_gui_example_tutorial.md` P1.3 (L55–88) + P2.1 (L124–128); `gui-example-p0-r0-review.md` RATIFICATION 4; plan-R0 r1/r2.

---

## VERDICT: **A1 — with a BINDING correction: APPEND the sentinel (opts[0] stays `bip44`), NOT the archetype's prepend. The census gains TWO toolkit-side surfaces the plan under-counted.**

A1 is the book-honest choice and it satisfies every binding constraint. But the plan's A1 encoding is **wrong on the load-bearing census cell** (it prescribes the archetype's leading-`(none)` render `dropdown[(none),bip44,…]`, which is self-contradictory with its own "gallery inert / virgin `opts[0]`=bip44" claim) and **under-counts A1's re-pin surfaces by two** (the frozen inventory JSON + a REQUIRED reference-manual `(none)` section — else `gui-schema-coverage` REDs at P2.1). Both are resolved below; both honor the zero-toolkit-src / zero-clap-surface / no-shared-`TEMPLATES` invariants. Two Important findings on the P1.3 brief (I settle them here — that is this mini-R0's job).

---

## 1. THE RULING — A1, APPEND placement

### Choice: **A1** (sentinel enumerated in the per-flag `opts`, render depicts `(none)`), NOT A2.
Decision rule from the brief: *"if A1's choices-gate check comes back inert, A1 is the book-honest choice."* The choices-gate (`schema_mirror`) is **INERT under A1 — definitively** (§2 below). A1's additional ripples are **not forbidden divergences** — they do not touch the toolkit projection, do not force a toolkit `src/` change, do not change the clap surface, and are **precedented byte-for-byte by the existing `--archetype` `""` sentinel**. A2's alternative is an **un-gated doc-honesty gap** (`gui_render_faithfulness` axes exclude option-list contents — `gui_render_faithfulness.rs:1-30`, re-verified): the tutorial teaches "select `(none)` to unlock Descriptor" while the reference manual's `--template` section, the `gui-render` structural emit, and the `.gui` transcript would all show `dropdown[bip44,…]` with no `(none)`. Given the house first-class doc-honesty bar (+ the standing "all doc CLI-output binary-identical" directive), the book-honest option wins when achievable within constraints — and it is.

### Placement: **APPEND** — `Dropdown([bip44, …, tr-sortedmulti-a, ""])`, sentinel LAST.
The archetype (`--archetype`) **prepends** `""` (`ARCHETYPES[0] == ""`) AND sets `default_value: Some("")` because it *wants* unset-as-default (`build_descriptor_schema.rs:117-129`; render shows `dropdown[(none),decaying-multisig,…] -> <empty>`, `gui_render_emit.rs:58`). Export-wallet `--template` must NOT be unset-by-default — its virgin default must stay `bip44` to keep the 61-form gallery PNG byte-stable and the taught "clear Template" flow honest. Since materialization takes `opts.first()` (`flag_defaults.rs:29-31`, re-verified) and `default_value` stays `None`, **the sentinel must be appended** so `opts[0]` remains `bip44`. (See §3 — this is the census's load-bearing cell.)

### EXACT touch-points (chosen variant)

| # | File / fn | Change |
|---|---|---|
| T1 | `src/schema/mnemonic.rs` — new per-flag const, e.g. `const EXPORT_WALLET_TEMPLATES: &[&str] = &["bip44",…,"tr-sortedmulti-a",""];` | The 10 `TEMPLATES` values **in order** + a trailing `""` sentinel. **Distinct from the shared `TEMPLATES` const** (`:69`) — honors "NO `""` in shared `TEMPLATES`". `TEMPLATES` continues to feed bundle/verify-bundle + the `SINGLE_SIG`/`MULTISIG` partition consts, untouched. |
| T2 | `src/schema/mnemonic.rs` — `EXPORT_WALLET_FLAGS[--template]` (`:1384`) | `kind: FlagKind::Dropdown(TEMPLATES)` → `Dropdown(EXPORT_WALLET_TEMPLATES)`. **`default_value` STAYS `None`** (do NOT set `Some("bip44")` — that would trip `is_at_default` suppression and drop `--template` from a virgin run's argv → BOTH-required refusal, a functional break; and it is unnecessary since `opts[0]`=bip44 already materializes correctly). |
| — | `src/form/widget.rs` (combo, `:520-540`) | **NO change.** The combo already iterates `*opts` and renders any `""` entry as a selectable `display_or("(none)", opt)`; selecting it writes `Dropdown("")`. Machinery reused verbatim. |
| — | `src/form/conditional.rs::export_wallet` (`:585-610`) | **NO change.** Rules are presence-based (`has_value` on `--template`/`--descriptor`). Selecting `(none)` → `Dropdown("")` → `has_value` false (`schema/mod.rs:388`, present iff non-empty) → the `--descriptor Disabled` push simply doesn't fire; the `!has_descriptor && !has_template` arm marks BOTH `Required` — exactly the toolkit's projected mutex. Constraint "conditional RULES unchanged" honored by construction. |
| T3 | `tests/gui_render_emit.rs:95` (export-wallet exact-ASCII pin) | RE-PIN: `--template … dropdown[bip44,…,tr-sortedmulti-a,(none)]  -> bip44`. Append `,(none)` at the END of the opts token; value column stays `-> bip44`; `--descriptor` stays `[disabled]`; every other line byte-identical. This pin **doubles as the placement guard** (a prepend would read `dropdown[(none),bip44,…]` and this assert would catch it). |

Constraints check: no `""` in shared `TEMPLATES` (T1 is a new const); clap flag-name set unchanged; the mirrored/**projected** template value-set (the toolkit's `single_sig`/`multisig` partition + gui-schema choices) untouched; conditional rules unchanged. **All three RATIFICATION-4 binding constraints honored.**

---

## 2. THE CHOICES-GATE INERTNESS ANSWER (definitive, with cites)

**The `schema_mirror` "dropdown value enum" comparison the brief worried about DOES NOT EXIST as a GUI↔toolkit choices comparison. `schema_mirror` gates flag NAMES ONLY. A1's sentinel is INERT on it.**

- `schema_mirror`'s live comparison (`assert_schema_matches_help`, `schema_mirror.rs:52-124`) maps `sub.flags.iter().map(|f| f.name)` on the GUI side and set-compares against the upstream flag-name set. The upstream side deserializes gui-schema JSON via `GuiSchemaFlag { name }` only — **`schema_check.rs:98-103`: "Other fields (required, kind, choices) intentionally not deserialized."** So gui-schema's `choices` are **never read**, never compared. Confirmed independently by `build_descriptor_schema.rs:6-8,117`: *"`schema_mirror` compares flag NAMES only — kinds, `repeating`, and Dropdown values are NOT gate-checked."*
- The toolkit **does** emit real `choices` for export-wallet `--template` (`gui_schema.rs:243` `choices: Option<Vec<String>>`; the inventory snapshot shows the 10 values) — but because the GUI never deserializes that field, adding a GUI-side sentinel cannot diverge it. The CLAUDE.md phrase "plus dropdown value enums" refers to the **separate partition consts**, not a per-flag choices compare (next bullet).
- **Template-partition drift** (`schema_mirror.rs:596-717`): compares `conditional::SINGLE_SIG_TEMPLATES` / `MULTISIG_TEMPLATES` consts against the toolkit's `meta.template_groups`. These are **distinct consts**, derived from `CliTemplate::is_multisig()` (`gui_schema.rs:272-288`) — NOT the export-wallet `--template` opts. T1's new per-flag const does not touch them. **INERT.**
- **`gui_schema_conditional_drift`**: synthesizes satisfying states from the **toolkit's projected rule** (`synthesize_satisfying`: `FlagPresent` → pushes `Text("exemplar")`; `DropdownValueIn` → `values.first()` from the toolkit's list — `gui_schema_conditional_drift.rs:104-113,330-352`). It never enumerates the GUI's `opts`. Export-wallet's mutex is `FlagPresent`-based (`gui_schema.rs:387-406`, `Predicate::FlagPresent{--template|--descriptor} → Disabled`). Appending `""` is invisible to it. **INERT.**
- **`build_descriptor_*` value pins** (`build_descriptor_schema.rs`): cover build-descriptor's `--archetype`/`--allow` only. Export-wallet `--template` is **not** byte-pinned anywhere (grep-confirmed: no test asserts export-wallet opts == `TEMPLATES`). **INERT.**
- **`gui_render_faithfulness`**: axes = per-flag presence/disabled/control-class + secret-masking; option-list contents explicitly OUT (`gui_render_faithfulness.rs:1-30`). Green under both A1 and A2; **not an oracle** for this change (cannot catch A2's under-depiction).

**Conclusion:** the choices-gate is inert → per the brief's rule, **A1 is the book-honest choice.** No compliant A1 shape forces a toolkit `src/` change.

---

## 3. THE DEFAULT-SELECTION / CENSUS-CELL RULING (the load-bearing cell)

**Ruled precisely:** the 61-form gallery PNG for export-wallet is **byte-stable (inert) IFF the sentinel is APPENDED. Under the plan's prescribed prepend it MOVES.** APPEND is mandatory.

Mechanism (re-verified at source):
- Virgin materialization: `render_with_dispatch` write-back (`widget.rs:220-229`) on an absent flag pushes `default_flag_value_for_flag(flag)`. With `default_value: None`, that falls to `default_flag_value_for(&Dropdown(opts))` = **`Dropdown(opts.first())`** (`flag_defaults.rs:29-31,63-66`).
- **APPEND** (`opts = [bip44,…,""]`): `opts[0] == "bip44"` → virgin `Dropdown("bip44")` → `has_value("--template")` **true** → `export_wallet` pushes `--descriptor Disabled` → the closed dropdown renders `bip44`, `--descriptor` `[disabled]`, the multisig-only flags `[disabled]` — **identical to today's steady state** (`gui_render_emit.rs:93-115`). Gallery `figures/gui/mnemonic-export-wallet.png` = **byte-identical**. The reader genuinely must "clear Template to unlock Descriptor" → the taught flow is honest.
- **PREPEND** (`opts = ["",bip44,…]`, the archetype-literal the plan prescribes): `opts[0] == ""` → virgin `Dropdown("")` → `has_value` **false** → `--descriptor` **enabled** on virgin, closed dropdown shows `(none)`, and `template_is_in(SINGLE_SIG)` false → `--threshold` / `--multisig-path-family` / `--taproot-internal-key` **un-disable**, the `(required)` pair markers reappear. The **whole export-wallet steady-state render changes on ~5 lines AND the gallery PNG moves.** This directly contradicts the plan's own census row-1 ("virgin `opts[0]` materialization keeps the closed render at `bip44`") — the plan wants both leading-`(none)` in the render AND `opts[0]`=bip44, which is impossible (render order == opts order == materialization `opts[0]`).

**Cell ruling: seed still lands `bip44` ⇒ gallery PNG byte-stable ⇒ APPEND. The alternative (prepend, or `default_value: Some("bip44")`) is REJECTED** — prepend moves the PNG and rewrites the steady-state; `default_value: Some("bip44")` breaks the virgin run via `is_at_default` suppression.

---

## 4. CONFIRMED / AMENDED TDD TEST LIST (P1.3)

The plan's 4 are kept and sharpened; **one new test (#5) is load-bearing for the append variant.** All in the always-run suite (`cargo test --jobs 2`), not env-gated.

1. **Reachability (F1 regression).** Drive a virgin export-wallet form → select the `(none)` entry → type a descriptor into `--descriptor` → assert: `--descriptor` **enabled** (mutex released), assembled argv **carries** `--descriptor <text>`, and **no `--template` token**. RUN → GREEN (new).
2. **`(none)` never leaks `""`.** Selecting `(none)` yields **no `--template` in the assembled argv** and no `""` in the copy-command string / argv echo (empty `Dropdown` is presence-suppressed — `has_value` false + the `argv_assembler_disabled_suppression.rs:247` "dropdown value must NOT leak" discipline). Re-selecting a real value re-emits `--template <v>` and re-disables `--descriptor`. RUN → GREEN (new; guards the emit path for the `None`-default empty-Dropdown case).
3. **Materialization / `has_value`.** Virgin `has_value("--template")` **true**; after `(none)`, **false**; the mutex tracks it both ways. RUN → GREEN (new).
4. **`(none)` display.** The sentinel renders `(none)` in both the open list and the closed combo (`display_or`), never a bare `""`. RUN → GREEN (new).
5. **★ Virgin-default stability (NEW — census guard).** Assert a virgin export-wallet form materializes `--template == "bip44"` (NOT `""`), `has_value("--template")` true, and `--descriptor` `Disabled` on load. This pins the **APPEND** placement against a future refactor silently prepending the sentinel (which would flip the virgin default, move the gallery PNG, and desync the taught flow). RUN → GREEN (new). *(The `gui_render_emit.rs:95` re-pin in T3 is the redundant structural guard — a prepend would fail its exact-ASCII assert.)*

**Adjacent-gate assessment (each NAMED, RUN, status):**

| Gate | Status under A1-append |
|---|---|
| `schema_mirror` (flag-name, all 4 CLIs) | **RUN → GREEN, INERT** (flag-names only; §2). |
| `gui_schema_conditional_drift` | **RUN → GREEN, INERT** (presence-based; synth reads toolkit values, not GUI opts). The variant-(b) tripwire — asserts zero rule-set delta. |
| template-partition (`single_sig_…` / `multisig_templates_const_matches_meta`) | **RUN → GREEN, INERT** (separate consts). |
| `build_descriptor_*` value pins | **RUN → GREEN, INERT** (build-descriptor only). |
| `gui_render_faithfulness` | **RUN → GREEN** (option-list not an axis) — not a census signal. |
| `hint_text_defaults` | **RUN → GREEN, NULL** interaction (Dropdown, `default_value: None`; hint-text covers hinted *Text* flags — `gui_render_emit.rs:132-135`). |
| `gui_form_snapshots` (61-form gallery) | **RUN → GREEN, INERT** (closed form, `opts[0]`=bip44). |
| `argv_assembler*` (export-wallet/bundle) | **RUN → GREEN, INERT** (set explicit values; materialized `--template bip44` still emitted). |
| `gui_render_emit.rs:95` (export-wallet) | **EXPECTED RE-PIN** (T3; append `,(none)`) — a NAMED P1.3 GUI-repo test-source edit. |

**Full-suite + clippy (both configs) + `--no-default-features` after the edit; scoped opus post-impl review on the F1 diff → GREEN before P1.4.**

---

## 5. RE-PIN CENSUS (per-surface, A1-append) — CORRECTED

The plan lists {gallery=0, emit re-pin, `.gui` re-pin, inventory=**ZERO**}. A1-append actually moves **two more** toolkit-side surfaces. Both are precedented exactly by the existing `--archetype` `""` sentinel; **neither is a toolkit `src/` / clap-surface / projection change.**

| Artifact surface | A1-append ruling |
|---|---|
| 61-form gallery PNG (`figures/gui/mnemonic-export-wallet.png`) | **INERT (0).** Closed form; `opts[0]`=bip44 virgin (§3). |
| `gui_render_emit.rs:95` exact-ASCII pin (GUI test source) | **RE-PIN — P1.3** (T3; append `,(none)`). |
| toolkit `transcripts/gui/mnemonic-export-wallet.gui` | **RE-PIN — P2.1** (regenerated by `verify-examples-gui` from the v0.56.0 pin; `Makefile:267-276`). |
| **`docs/manual-gui/tests/expected_gui_schema_inventory.json`** | **DOCUMENTARY REGEN — P2.1** (NOT the plan's "ZERO"). `extract_gui_schema.py` reads the GUI schema **source** (`src/schema/*.rs`) and records each Dropdown's resolved `variants`; the `--archetype` `""` is already captured as `""` (inventory L680). Under append, export-wallet `--template` variants gains a trailing `""`. **No programmatic gate reads this file** (grep-confirmed: no `.rs`/lint consumer) → it does **not** RED; regenerate for accuracy/hygiene. |
| **`docs/manual-gui/src/40-mnemonic/45-export-wallet.md` (+ built HTML)** | **REQUIRED EDIT — P2.x, else `gui-schema-coverage` REDs.** `check_gui_schema_coverage.py::build_expected` adds an anchor per variant; the `""` variant → required anchor `#mnemonic-export-wallet-template-` (`kebab("")==""`). The manual currently has NO `(none)` entry for `--template` → **missing-anchor RED at P2.1**. Fix = mirror the `--archetype` precedent (`4e-build-descriptor.md:75,82`): add `- [` `(none)` `](#mnemonic-export-wallet-template-)` to the `--template` Outline + a `### ` `(none)` ` {#mnemonic-export-wallet-template-}` section. **Prose MUST differ from `--archetype`'s** — for export-wallet `(none)` is an appended affordance, NOT the default (bip44 is); wording e.g. *"The empty-string unset sentinel — no template selected; selecting it clears `--template` so `--descriptor` becomes the required/enabled field."* Also correct the `:48` cross-reference ("Same 10 values as `bundle --template`") — export-wallet now carries those 10 **plus** the GUI `(none)` sentinel (bundle stays 10). |
| `expected_gui_schema_inventory.json` as a GATE | **No RED** (no consumer). |
| all other `gui-schema-coverage` anchors | **INERT.** |

**STOP-tripwire recalibration (supersedes plan L86/L128 "expected ZERO"):** under A1-append the EXPECTED movement set is exactly {gallery INERT; `gui_render_emit.rs:95` RE-PIN (P1.3); `mnemonic-export-wallet.gui` RE-PIN (P2.1); `expected_gui_schema_inventory.json` one-entry regen (P2.1); `45-export-wallet.md` `(none)` section (P2.x)}. **Movement of any OTHER form's gallery PNG, any OTHER subcommand's emit/`.gui`/anchors, bundle's `--template` opts, or any partition/conditional delta = STOP** (back to this mini-R0). The bundle↔export-wallet `--template` asymmetry (11 vs 10) after A1 is **intended and scoped** — do not "fix" bundle.

---

## FINDINGS on the P1.3 brief

### Critical — none.

### Important (both SETTLED by this ruling — no open items block P1.3)

- **I1 — the plan's A1 render placement is wrong and self-contradictory (census-cell).** P1.3 L59 prescribes `dropdown[bip44,…]` → `dropdown[(none),bip44,…]` (archetype-literal, sentinel FIRST) while census row-1 (L80) asserts the gallery stays `bip44` via "virgin `opts[0]` materialization." These are mutually exclusive: sentinel-first ⇒ `opts[0]==""` ⇒ virgin unset ⇒ gallery moves + `--descriptor` enabled + ~5-line steady-state rewrite. **RESOLVED:** APPEND (`dropdown[bip44,…,tr-sortedmulti-a,(none)]`), `default_value` stays `None` (§1, §3).
- **I2 — the plan's A1 census under-counts by two toolkit-side surfaces.** The plan expects `expected_gui_schema_inventory.json` = ZERO and never lists a reference-manual edit. In fact `extract_gui_schema.py` reads GUI schema **source** and `check_gui_schema_coverage.py` requires a manual anchor **per variant** — so A1's `""` forces (a) an inventory regen (documentary, no RED) and (b) a REQUIRED `(none)` section in `45-export-wallet.md` or `gui-schema-coverage` REDs at P2.1. Both are precedented by `--archetype`. **RESOLVED:** named in §5; STOP-tripwire recalibrated so they are EXPECTED, not scope-smell STOPs.

### Minor
- **m1.** `render_emit`/`extract_gui_schema.py` read the **same** `FlagKind::Dropdown(opts)` — so "depict `(none)` in the emit" and "hide `(none)` from the inventory/coverage" are **mutually impossible**. This is why A1 unavoidably touches the inventory + manual anchor; it is not a defect, just the coupling that forces I2's fuller census (and rules out any "narrower A1" that shows `(none)` in the render yet stays inventory-inert).
- **m2.** Keep `default_value: None` (do NOT set `Some("bip44")`): `Some("bip44")` would suppress `--template` at-default → virgin export-wallet run emits neither `--template` nor `--descriptor` → both-required refusal (functional break). Append + `None` is the only run-preserving, census-preserving shape.

**GATE: the P1.3 design is SETTLED — A1, APPEND. The implementer may execute the touch-point list (T1–T3 + the 5 TDD tests) under the standing per-phase R0 + scoped post-impl cadence.** No STOP-ledger condition (§4/§6.5 SAME-FRAME; toolkit-src pressure; F1-cannot-satisfy) is reached: A1-append lives entirely in GUI `form/`+schema rendering/materialization + doc re-pins, with zero toolkit `src/` / clap-surface / projection change.

---

## CITES (all live-verified 2026-07-05)

- mnemonic-gui `master@0d4429d`: `src/schema/mnemonic.rs:69` (`TEMPLATES`, 10 vals, no `""`), `:1384` (export-wallet `--template` = `Dropdown(TEMPLATES)`, `default_value: None`), `:4163-4164` (choices not gate-checked); `src/form/flag_defaults.rs:29-31,63-66` (Dropdown default = `opts.first()`); `src/schema/mod.rs:388` (`has_value` = present iff non-empty); `src/form/widget.rs:220-229` (materialize write-back), `:520-540` (combo iterates `opts`, `display_or("(none)")`); `src/form/conditional.rs:585-610` (`export_wallet`, presence-based mutex + both-Required arm); `src/schema_check.rs:98-103` (gui-schema deser = NAME only); `tests/schema_mirror.rs:52-124` (flag-NAME gate), `:596-717` (partition consts, separate); `tests/gui_render_emit.rs:58` (archetype prepend `dropdown[(none),…] -> <empty>`), `:83-115` (export-wallet pin), `:95` (the `--template` line); `tests/gui_render_faithfulness.rs:1-30` (axes exclude option-list); `tests/gui_schema_conditional_drift.rs:104-113,330-352` (synth from toolkit-projected values); `tests/build_descriptor_schema.rs:6-8,117-129` (schema_mirror = NAMES only; `--archetype` `""` sentinel precedent + `default_value: Some("")`).
- mnemonic-toolkit `master@be876803`: `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:243` (`choices` emitted), `:272-288` (partition from `is_multisig()`), `:387-406` (export-wallet `FlagPresent → Disabled`); `docs/manual-gui/tests/extract_gui_schema.py:1-40,153-213` (reads GUI schema source, resolves const → variants); `docs/manual-gui/tests/expected_gui_schema_inventory.json:679-687` (`--archetype` `""` captured), `:1321-1338` (export-wallet `--template` = 10 vals); `docs/manual-gui/tests/check_gui_schema_coverage.py::build_expected` (anchor per variant, `kebab("")==""`); `docs/manual-gui/tests/lint.sh:105-117` (phase 4 gui-schema-coverage); `docs/manual-gui/src/40-mnemonic/4e-build-descriptor.md:73-86` (`(none)` outline+section precedent), `45-export-wallet.md:42-72` (`--template` outline+per-variant sections, no `(none)`).
