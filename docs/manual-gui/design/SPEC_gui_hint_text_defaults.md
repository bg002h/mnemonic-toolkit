# SPEC — `hint_text` ghosting for schema-defaulted Text/Path fields (fix `gui-prefilled-default-text-appends-on-type`)

- **Status:** DRAFT — awaiting R0 (0C/0I gate before any implementation).
- **Date:** 2026-07-01.
- **Target repo (Leg 1):** `mnemonic-gui`, recon at `master @ 7e9dcca7b740b138e7133c44a9709c4f9010aa66` (v0.54.0 + 1 docs commit). All `mnemonic-gui` file:line cites below are against that SHA.
- **Consumer repo (Leg 2):** `mnemonic-toolkit/docs/manual-gui/` (this manual; pin currently `mnemonic-gui-v0.54.0`, `pinned-upstream.toml:30`).
- **FOLLOWUP under fix:** `mnemonic-gui/FOLLOWUPS.md:968-977` `gui-prefilled-default-text-appends-on-type` (tier `ux`, filed by the PR-#24 P5 harness sweep).
- **egui:** 0.31 (`mnemonic-gui/Cargo.toml:48`); mechanics verified against registry source `egui-0.31.1`.

---

## 1. Goal

A flag whose schema declares a `default_value` currently pre-fills its widget
with the default as REAL editable text. A user who clicks and types without
first clearing APPENDS: `compare-cost --feerate` `1.0`+`5` → `1.05`;
`export-wallet --output` `-`+path → `-/path`. Replace the pre-fill with egui
`hint_text` ghosting for the affected widget kinds: the default DISPLAYS as a
ghost, typing REPLACES, and an empty field means "the CLI applies its own
default".

## 2. Recon ground truth (R0 inputs — every claim verified at source)

### 2.1 CORRECTION to the working premise: argv already omits at-default values

The cycle brief assumed "today a defaulted flag is passed EXPLICITLY
(`--feerate 1.0` in argv)". **That is false.** Since v0.10.0 (D33), the argv
assembler suppresses any value equal to the flag's schema default:

- `emit_one` gate: `src/form/invocation.rs:392-399` (`if is_at_default(flag, value) { return; }`).
- Per-kind compare table: `src/form/invocation.rs:45-95` — Text
  `s.is_empty() || s == default_str` (`:61`), Path `p == default_str`
  (`:64` — the `-` sentinel matches literally), Dropdown `s == default_str` (`:84`).
- Pinned by test `cell_import_wallet_select_descriptor_default_suppressed`
  (`tests/kittest_import_wallet_form.rs:224-267`): seeded `--select-descriptor all`
  MUST be absent from argv; `active-receive` MUST emit.

**Consequence:** for an untouched field, this fix changes **zero argv bytes**.
Today: buffer `"1.0"` → suppressed. Post-fix: buffer `""` + ghost → suppressed
(Text: `emit_one`'s `!v.is_empty()` guard `invocation.rs:402-403`; Path: empty
early-return `invocation.rs:473-475`). The masked copy-command preview
(`render_copy_command_masked`, built from the same argv) is likewise unchanged.
The only behavioral delta is the keystroke→value mapping: typing `5` into
`--feerate` now yields `5`, not `1.05`.

### 2.2 The seed/render path (current behavior)

- Per-frame seed: `src/form/widget.rs:220-229` (`render_with_dispatch`) — a flag
  absent from `state.values` seeds `default_flag_value_for_flag(flag)` and the
  result is pushed back into `state.values` (the persistence-visible store).
- The flag-aware default resolver — **THE single source of truth** (`src/form/flag_defaults.rs:63-86`):
  - Text/Dropdown/Path with `default_value: Some(s)` → concrete
    `Text(s)`/`Dropdown(s)`/`Path(s)` (`:67-70`) — **this is the papercut**.
  - Number/Range/Timestamp/TaggedOrIndexed → **`Unset` regardless of schema
    default** (`:81-84`); the widget renders a `Set` affordance
    (`widget.rs:460-474`), so these kinds have NO pre-fill and NO papercut.
- Editable binding: Text arm `widget.rs:452-454` (`ui.text_edit_singleline(s)`);
  Path arm `widget.rs:658-663` (same, plus the `stdio` button that writes `-`).
- All `default_flag_value_for_flag` call sites (exhaustive grep):
  widget scalar seed `widget.rs:223`; required-repeating row seed
  `widget.rs:314`; Unset-shape recovery `widget.rs:674`; emit-side
  `seeded_fixture` `render_emit.rs:142,150`; emit value column
  `render_emit.rs:604`.

### 2.3 The census — flags with `default_value`, by kind (parsed from the 4 schema files)

**82 total** (`mnemonic` 69 / `ms` 8 / `mk` 3 / `md` 2):

| Kind | Count | Pre-fills today? | Papercut? |
|---|---|---|---|
| Dropdown | 40 | yes (seeded selection) | **No** — selection replaces; never typed into |
| Number | 34 | no (`Unset` + `Set` affordance) | **No** |
| Text | 4 | **yes (editable text)** | **YES** |
| Path | 2 | **yes (editable text)** | **YES** |
| Range | 1 | no (`Unset`) | No |
| Timestamp | 1 | no (`Unset`) | No |

**The papercut surface is exactly 6 flags** (matching the FOLLOWUP's list),
each verified against its toolkit clap declaration:

| Flag | GUI schema (kind, default) | Toolkit clap source |
|---|---|---|
| `mnemonic compare-cost --feerate` | Text `"1.0"` — `src/schema/mnemonic.rs:2261` | `crates/mnemonic-toolkit/src/cmd/compare_cost.rs:32` `default_value_t = 1.0` |
| `mnemonic import-wallet --select-descriptor` | Text `"all"` — `src/schema/mnemonic.rs:2554` | `cmd/import_wallet.rs:165` `default_value = "all"` |
| `mnemonic nostr --timestamp` | Text `"0"` — `src/schema/mnemonic.rs:3551` | `cmd/nostr.rs:112` `default_value = "0"` |
| `ms derive --account` | Text `"0"` — `src/schema/ms.rs:279-287` | `mnemonic-secret/crates/ms-cli/src/cmd/derive.rs:43-44` `default_value_t = 0` |
| `mnemonic export-wallet --output` | Path `"-"` — `src/schema/mnemonic.rs:1476` | `cmd/export_wallet.rs:272` `default_value = "-"` |
| `mnemonic restore --output` | Path `"-"` — `src/schema/mnemonic.rs:652` | `cmd/restore.rs:242` `default_value = "-"` |

All 6 are `required: false` (verified per entry). **Zero secret flags carry a
`default_value`** (census result; independently enforced by the always-run
`secret_flags_never_carry_a_default_value` test in
`tests/gui_form_snapshots.rs` and the I3 never-persist class).

Affected forms (= the full regen blast radius): `mnemonic-compare-cost`,
`mnemonic-import-wallet`, `mnemonic-nostr`, `mnemonic-export-wallet`,
`mnemonic-restore`, `ms-derive` — **6 of the 61**.

### 2.4 Number/DragValue: verified NO papercut (the (d) question)

Two independent reasons:

1. Number (and Range/Timestamp/TaggedOrIndexed) defaults seed `Unset`
   (`flag_defaults.rs:81-84`) — nothing is pre-filled; the user opts in via the
   `Set` button (`widget.rs:460-474`), which seeds `seeded_value_for` (the
   range `min`, `widget.rs:381` — see §9 non-goal N4).
2. Even inside an active `DragValue`, typing REPLACES: egui 0.31.1's DragValue
   click-to-edit path explicitly select-alls the text before focus —
   `egui-0.31.1/src/widgets/drag_value.rs:620-628` sets
   `CCursorRange::two(CCursor::default(), CCursor::new(value_text.chars().count()))`.
   `DragValue` has **no `hint_text` API** (grep of `drag_value.rs`: none), and
   needs none — the Number-field answer is: **keep DragValue semantics
   unchanged; Number is out of scope.**

`TextEdit::hint_text` exists at
`egui-0.31.1/src/widgets/text_edit/builder.rs:205` and renders only while the
buffer is empty. **In-repo precedent:** the SlotEditor already ships it —
`src/form/slot_editor.rs:57`
(`egui::TextEdit::singleline(&mut row.value).hint_text(hint)`), gated by
`tests/slot_editor_path_hint_text.rs`.

### 2.5 conditional() rules vs seeded defaults (the (c) question)

Full read of all 17 conditional fns (`src/form/conditional.rs`) + a grep of
`src/` for the 6 flag names: **no conditional rule — and no other `src/`
consumer — reads ANY of the 6 Text/Path defaulted flags.** Specifically:

- `compare_cost` reads `--miniscript`/`--descriptor` (`conditional.rs:968-985`) — neither has a default; NOT `--feerate`.
- `export_wallet` reads `--descriptor`/`--template` (+ taproot/threshold rules, `conditional.rs:589-625`) — NOT `--output`/`--timestamp`/`--range`.
- `restore` reads `--md1` only (`conditional.rs:1006-1012`) — NOT `--output`.
- `import-wallet`, `nostr`, `ms derive` have **no conditional fn at all** (they are not among the 17).

**Flag-NAME collisions (grep caveat):** two of the 6 names also exist as
*different subcommands'* flag entries — `--account` is ALSO bundle's
Number-kind flag, which has a conditional `PinValue` rule
(`conditional.rs:229-248`) plus `main.rs:306` (hand-seed) / `main.rs:762`
(path-hint reader) consumers; `--timestamp` is ALSO export-wallet's
Timestamp-kind flag. Conditional fns and FormState are subcommand-scoped, so
the zero-consumers claim above holds — but only under that scoping. An
implementer grepping `src/` for the bare flag names WILL hit these entries;
they are not false alarms against this spec.

Conditional rules that DO key on seeded defaults are exclusively
**Dropdown-fed** (out of fix scope, seeding unchanged):

- `bundle`/`verify_bundle`/`export_wallet` `template_is_in(...)`
  (`conditional.rs:180-185`) reads `--template` — seeded `"bip44"` via the
  **kind-only `opts[0]` fallback** (`flag_defaults.rs:30-32` +
  `TEMPLATES` `src/schema/mnemonic.rs:69-80`; bundle `--template` has
  `default_value: None` — the brief's "seeded --template=bip44" example is
  real but comes from the opts[0] fallback, not from a schema `default_value`).
- `slip39_combine` reads `--to` (schema default `"entropy"`,
  `mnemonic.rs:1808`) at `conditional.rs:897-900` — and its predicate already
  treats `None` the same as `Some("entropy")`, so it is seed-robust by design.
- `md_encode`/`md_compile` read `--context`; `derive_child` reads
  `--application`; `build_descriptor` reads `--archetype` (leading-`""`
  sentinel) — all Dropdowns, none in scope.

**Therefore: scoping the fix to Text/Path preserves ALL conditional behavior
identically** — the `has_value` flip (true→false on a fresh form for the 6
flags, per `flag_value_is_present` `src/schema/mod.rs:497-501`) has zero
readers.

### 2.6 Omitted-vs-explicit equivalence (the (e) question)

Moot for argv (per §2.1 the flag is ALREADY omitted today when untouched), but
load-bearing for the ghost's truthfulness: the ghost claims "leave empty and
the CLI does `<default>`". That holds — but NOT by a blanket
"mirror-construction" guarantee. Precisely:

- **5 of the 6 flags mirror the `mnemonic gui-schema` v5 JSON strings
  exactly** (the v5 emitter extracts them from clap itself,
  `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:1184`
  `extract_default_value(arg, &kind)`).
- **`compare-cost --feerate` is a DELIBERATE double override.** The v5 JSON
  emits it as kind `number` with default `1` (clap's `default_value_t = 1.0`
  renders via `f64::to_string()` → `"1"`); the GUI hand-mirror deviates on
  BOTH axes — `FlagKind::Text` + `Some("1.0")` — documented at
  `src/schema/mnemonic.rs:2231-2236`: the GUI's Number kind is i64-only while
  the toolkit parser is f64, so a Text widget is the only lossless entry
  surface. The ghost `1.0` remains semantically truthful (the clap default IS
  1.0; typing the literal `1.0` parses identically).
- **Future-drift trap — do NOT "restore mirror fidelity" here.** A future
  pass that flips feerate back to `number`/`1` believing it corrects mirror
  drift would BREAK the documented i64-vs-f64 override, and no gate would
  catch it: `schema_mirror` gates flag NAMES + dropdown value enums, NOT
  kinds and NOT `default_value` strings (no `default_value` reference in
  `tests/schema_mirror.rs`).

Ghost-truthfulness is therefore anchored on the **§2.3 live clap-attribute
spot-checks of all 6 flags** (+ the §9 N3 trust model), not on a mechanical
mirror guarantee. No clap `conflicts_with`/`requires` rule can distinguish the
two states post-fix any differently than today, because the emitted argv is
byte-identical (untouched → omitted, both before and after). The ghost
inherits exactly the trust model the pre-fill has today; no new gate is
introduced (see §9 N3).

### 2.7 The downstream mirror/emit chain (the (f) inputs)

- `render_emit.rs::seeded_fixture` (`:102-158`) mirrors the widget seed via the
  SAME resolver (`default_flag_value_for_flag`, `:142,150`) — P3-R0 ruling A.
- The `.gui` value column does NOT read seeded state at all: `flag_value_str`
  (`render_emit.rs:592-632`) calls `default_flag_value_for_flag(flag)`
  directly (`:604`). Changing the resolver therefore updates widget, fixture,
  and emit **in one place, drift-free by construction**.
- The faithfulness gate (`tests/gui_render_faithfulness.rs`) compares
  presence/disabled/control-class only — "default/placeholder TEXT" is
  explicitly OUT of its tree-observable axes (`:27-30`). It re-runs but needs
  no change.
- Committed exact-ASCII pins: `tests/gui_render_emit.rs` pins export-wallet
  verbatim (`:83-116`, includes `--output ... path(stdio) -> -`) — MUST be
  updated. `mnemonic inspect` / `build-descriptor` / `mk inspect` pins contain
  no defaulted Text/Path flag.
- PNG corpus: `tests/snapshots/forms/<tab>-<sub>.png` (61 files; suite
  `tests/gui_form_snapshots.rs`, kittest dify threshold 0.6; CI job
  `snapshots` in `.github/workflows/build.yml:87` — the repo's only
  hard-required check). 6 files change.
- Manual (this repo): `transcripts/gui/*.gui` (61, gated by
  `tests/verify-examples-gui.sh` via `make verify-examples-gui`,
  `Makefile:292-297`, census `EXPECTED_GUI_RENDER_COUNT` = 61) and
  `figures/gui/*.png` (61 byte-copies, gated by lint phase 9
  `verify-figures-gui`). 6 + 6 files change. Pin-bump step 0 verifies the
  tag-push `snapshots` check-run succeeded (provenance anchor —
  `mnemonic-gui/FOLLOWUPS.md:1014-1040`
  `gui-form-snapshot-corpus-manual-consumer`).

### 2.8 Harness seed tables (the (4) question)

- The P5 sweep's append-dodge lives at `tests/ui_harness_sweep.rs:158-172`
  (doc: "`type_text` then APPENDS to that pre-filled text … a HARNESS
  artifact") + `:191-198` (the `Text("")`/`Path("")` re-seed inside
  `prepared_eligible_base`). Post-fix the re-seed models nothing — see §7 for
  its removal (which converts all I1 Text/Path round-trips into permanent
  append-regression coverage).
- `tests/ui_harness/mod.rs` `base_state` (`:148-176`) seeds only *context*
  flags (`addresses` `--from`/`--network` as empty) — none of the 6; unaffected.
- I2 (`tests/ui_harness_i2_conditional.rs`) exercises the 17 conditional subs;
  per §2.5 none keys on the 6 flags; its two `default_value:` mentions
  (`:602,612`) are synthetic-schema constructions. Unaffected.
- The sweep's non-default-injection generator uses `flag.default_value`
  (`ui_harness_sweep.rs:99,114-131,424`) read-only from the schema. Unaffected.

## 3. Fix design (per-kind scope, grounded in §2)

**Scope: `FlagKind::Text` and `FlagKind::Path { .. }` with
`default_value: Some(_)` — the 6 flags. Every other kind is untouched.**

1. **Resolver (the single edit that moves all mirrors at once):**
   `src/form/flag_defaults.rs::default_flag_value_for_flag` — the
   `FlagKind::Text` / `FlagKind::Path` arms (`:67-70`, currently
   `Text(default_str)` / `Path(default_str)`) fall through to the kind-only
   empty defaults (`Text("")` / `Path("")`), like the four Unset kinds already
   do. The `Dropdown` arm keeps mapping the schema default (`:69` — dropdown
   seeding is correct UX and feeds conditionals, §2.5). Doc-comment updated to
   state the new contract: "Text/Path schema defaults are DISPLAY-ONLY
   (hint_text ghost) + emission-time (`is_at_default`); they never enter
   `state.values`."
2. **Widget ghost:** `render_row`'s Text arm (`widget.rs:452-454`) and Path arm
   (`widget.rs:658-663`) render
   `ui.add(egui::TextEdit::singleline(s).hint_text(d))` when
   `flag.default_value == Some(d)` (plain `text_edit_singleline` otherwise —
   zero visual change for the other ~all Text/Path flags). Ghost text is the
   **literal `default_value` string** — no editorializing (mirror-truth +
   determinism); the flag's `help` tooltip continues to carry semantics (e.g.
   `-` = stdout). Precedent: `slot_editor.rs:57`.
3. **argv:** NO assembler change. `is_at_default` (`invocation.rs:45-95`)
   stays — a user who TYPES the literal default still gets it suppressed,
   exactly like today; untouched-empty emits nothing (§2.1). The `stdio`
   button on Path (`widget.rs:660-662`) still writes a literal `-` (visible as
   real text, suppressed from argv by `is_at_default` — same net argv as
   today).
4. **Stale persisted-state normalization (one-time migration):** pre-fix
   autosaves carry the seeded defaults in per-subcommand `values`
   (`src/persistence.rs:79-120` persists non-secret `state.values`). On LOAD
   (the persistence-restore path, NOT per-frame), drop any Text/Path entry
   whose value string equals its flag's `default_value`. Mechanics:
   (a) resolve each entry's flag schema via a per-subcommand lookup keyed by
   the persisted `tab:sub` key; (b) **fail-open** — an unknown subcommand or
   unknown flag name keeps its entry verbatim (never destructive on a lookup
   miss); (c) **kind-scoped to Text/Path entries only** — bundle's
   `--account` `Number(0)` hand-seed and every Number-kind default are
   untouched; (d) post-fix autosaves legitimately carry `Text("")`/`Path("")`
   entries — these do NOT equal the default and MUST NOT be dropped (third
   §7 migration test vector). Semantics-preserving: argv-identical by D33
   (both forms emit nothing) and visibility-identical because no rule reads
   those flags (§2.5). Load-time-only, so a user typing the literal default
   mid-session keeps their visible text (a per-frame clear would fight the
   keyboard).
5. **Emit depiction:** with the resolver change, `flag_value_str`
   (`render_emit.rs:604-614`) would render the 6 rows as `<empty>` — losing
   the ghost the user actually sees (the render's charter is "the screen the
   user sees on load", `render_emit.rs:72-81`). Add one arm: empty Text/Path
   with `flag.default_value == Some(d)` renders **`<hint:d>`** (ASCII,
   deterministic; grammar sibling of `<empty>`/`<unset>`/`<masked>`/`<pinned: …>`),
   e.g. `--feerate  text  -> <hint:1.0>`, `--output  path(stdio) -> <hint:->`.
   The faithfulness gate is indifferent (placeholder text excluded, §2.7);
   `seeded_fixture` needs no separate edit (same resolver).

## 4. argv semantics — omit-when-empty + the equivalence argument

Already the shipped D33 design; restated as the invariant the fix preserves:

- Untouched field → no `state.values` deviation → flag ABSENT from argv →
  the CLI applies its clap default. True before AND after this fix (§2.1);
  the ghost merely makes the existing semantics visible ("empty = CLI
  default").
- Ghost string == clap default, anchored on the 6 live clap-attribute
  spot-checks (§2.3) — NOT on mirror-construction: 5 of 6 mirror the v5 JSON
  strings exactly (`gui_schema.rs:1184`), while `--feerate` is a deliberate
  Text/`"1.0"` override of the v5 JSON's `number`/`1` (GUI Number is
  i64-only, toolkit parser is f64 — `src/schema/mnemonic.rs:2231-2236`;
  see §2.6).
- Reachable argv space is unchanged; only keystrokes→value changes
  (append → replace). No omitted-vs-explicit divergence class is introduced
  because explicit-default emission does not exist today either.

## 5. conditional() preservation

Per §2.5: zero conditional rules (and zero other `src/` consumers) read the 6
flags; all default-keyed conditionals are Dropdown-fed and Dropdown seeding is
untouched. The I2 suite (17 subs × 6 Visibility effects) re-runs as the
regression net. `seeded_fixture`'s fixed-point loop is unchanged in shape; its
inputs change only for the 6 flags, whose visibility maps are constant.

## 6. Downstream regen/re-gate chain — explicit phases

Mirrors the visual track's Leg-1 → tag → Leg-2 shape, smaller.

**Phase G — mnemonic-gui (one PR; GUI = PR + CI-green before tag):**
1. Implement §3 items 1-5 (TDD; tests first per §7).
2. Update the export-wallet exact-ASCII pin (`tests/gui_render_emit.rs:83-116`);
   add/adjust pins per §7.
3. Regenerate the snapshot corpus:
   `GUI_SNAPSHOTS=1 WGPU_BACKEND=gl LIBGL_ALWAYS_SOFTWARE=1 UPDATE_SNAPSHOTS=1
   cargo test --test gui_form_snapshots` (the sanctioned local path,
   `gui_form_snapshots.rs:37-40`); expect **exactly 6 changed PNGs** (§2.3
   list) — any other diff is a red flag for scope creep.
4. Flip `gui-prefilled-default-text-appends-on-type` → resolved in the
   shipping commit (status-flip discipline).
5. PR → all checks green (the hard-required `snapshots` job arbitrates the
   corpus at threshold 0.6) → merge → bump `Cargo.toml` `0.54.0 → 0.55.0` +
   CHANGELOG → tag `mnemonic-gui-v0.55.0`. The tag-push `snapshots` run is
   Leg 2's provenance anchor. `schema_mirror` is untouched (no flag
   name/kind/enum change — this is exempt from the paired-schema-PR rule; no
   `src/schema/*` edit at all).

**Phase M — mnemonic-toolkit `docs/manual-gui/` (this repo; no toolkit code):**
1. Step 0: verify the `mnemonic-gui-v0.55.0` tag-push `snapshots` check-run
   concluded `success` (`FOLLOWUPS.md::gui-form-snapshot-corpus-manual-consumer`
   contract).
2. `pinned-upstream.toml` `[mnemonic-gui] tag` → `mnemonic-gui-v0.55.0`; the
   four CLI-tag fields are UNCHANGED (no CLI surface change at the new tag —
   record that in the header comment, per the v0.54.0 precedent).
3. Regenerate `transcripts/gui/` via the pinned `gui-render --emit-all`
   (`make verify-examples-gui` fails-closed until done); expect exactly 6
   changed `.gui` files.
4. Byte-copy the 6 changed PNGs from the pinned clone's
   `tests/snapshots/forms/` into `figures/gui/` (lint phase 9
   `verify-figures-gui` byte-compares + 61-census both directions).
5. Prose: update `src/40-mnemonic/4c-import-wallet.md:162-164` ("free-form
   text input pre-filled with `all`" → ghost-hint wording: "shows `all` as
   placeholder text; leave empty to accept it"); sweep the book for other
   pre-fill claims (`grep -rn "pre-fill\|prefill\|pre-populated" src/`).
   NOTE: `src/50-md/53-encode.md:41-42` ("spin-box pre-filled with `5`") is
   **already stale today** (Number renders `Unset` + `Set`); fix it in the
   same pass but as a pre-existing-inaccuracy correction (§9 N4 companion).
6. `make verify-examples-gui && make lint && make` (all 9 lint phases) →
   commit. Manual CI = `.github/workflows/manual-gui.yml`.

**Phase R — mandatory post-impl whole-diff adversarial review** (both legs),
then FOLLOWUP flips wherever not already done in the shipping commits.

## 7. Test impact (mnemonic-gui)

Must update:
- `tests/gui_render_emit.rs:83-116` export-wallet exact pin (`-> -` becomes `-> <hint:->`).
- `tests/snapshots/forms/`: 6-PNG corpus regen (no test-code change).
- `tests/ui_harness_sweep.rs:158-198`: REMOVE the Text/Path `Text("")`/`Path("")`
  re-seed + its doc block — post-fix, type-without-clearing is the real user
  path, so all Text/Path I1 round-trips become permanent append-regression
  coverage (the sweep found this bug; now it guards the fix).

New tests (TDD, written first):
- **Append regression (THE test):** kittest form-level render of
  `compare-cost`, type `5` into `--feerate` WITHOUT clearing → argv contains
  `--feerate 5` (not `1.05`, not `1.0`).
- **Schema-derived pre-fill invariant** (sweep-style, not a 6-count
  change-detector): for EVERY Text/Path flag with `default_value` across all
  61 subs, the first-render buffer is empty (`state.values` entry is
  `Text("")`/`Path("")` or absent) AND assembled argv omits the flag.
- **Ghost presence:** exact-ASCII pin of `compare-cost` (`-> <hint:1.0>`) in
  `gui_render_emit.rs`; plus a widget-level kittest asserting the AccessKit
  text-input node's VALUE is empty on first render (hint text is not the
  value). Anti-tautology anchor: egui paints `hint_text` WITHOUT entering the
  text buffer, so the AccessKit assert is `value == ""` while the snapshot
  PNG simultaneously shows the ghost — that pair ties the `.gui` `<hint:d>`
  notation to real widget behavior rather than to the renderer's own output.
- **Persistence migration:** a persisted state carrying
  `("--output", Path("-"))` loads with the entry dropped; a persisted
  `("--output", Path("/tmp/x"))` survives verbatim; a persisted
  `("--output", Path(""))` (a legitimate post-fix autosave entry) survives
  verbatim — empty does not equal the default and is NOT dropped (§3.4d).

Verified-unaffected (rationale on file): `gui_render_faithfulness.rs` (§2.7);
`kittest_import_wallet_form.rs` Cell 5 (hand-built states, pins `is_at_default`
which is unchanged); `default_form_state.rs` (bundle Dropdown/Number);
`template_aware_seed.rs` (Dropdown/Number); I2/I3 suites (§2.8, §8);
`slot_editor_path_hint_text.rs` (separate widget, untouched). Full
`cargo test -p mnemonic-gui` runs at every gate (standing rule; local runs
`--jobs 2`).

## 8. Secret-flag interaction

None, by construction and by gate: **zero secret flags carry a
`default_value`** (census §2.3; enforced always-run by
`secret_flags_never_carry_a_default_value` in `tests/gui_form_snapshots.rs`,
and the I3 never-persist class re-verifies no secret ever enters persisted
state). Secret Text flags never route through the changed path at all — the
widget dispatch sends them to `SecretLineEdit`/`secret_widgets` before the
seed site (`widget.rs:95-122`), and the resolver change only affects flags
WITH a schema default. `hint_text` therefore can never display secret
material.

## 9. Non-goals

- **N1 — Dropdown seeding UX** (incl. `opts[0]` fallback seeds like bundle
  `--template = bip44`): unchanged; selections replace, no papercut.
- **N2 — Number/Range/Timestamp/TaggedOrIndexed:** already `Unset`-gated; no
  change (DragValue select-all on click, §2.4).
- **N3 — a `default_value`-string drift gate** (schema_mirror gates names, not
  default values): the ghost inherits the existing hand-mirror + paired-PR
  trust model. Candidate future FOLLOWUP; out of scope.
- **N4 — `Set` affordance seeds `min`, not the schema default**
  (`widget.rs:379-394` — e.g. clicking `Set` on `--gap-limit` yields the range
  min, not `20`): pre-existing, surfaced by this recon; file as a new GUI
  FOLLOWUP (`gui-number-set-affordance-ignores-schema-default`) rather than
  widening this cycle. The stale `53-encode.md:41-42` prose (§6 M5) is its
  documentation shadow.
- **N5 — select-all-on-focus** for pre-filled fields (the FOLLOWUP's
  alternative): rejected in favor of hint ghosting — ghosting also
  communicates "empty = CLI default", which select-all does not.
- **N6 — toolkit CLI / sibling-codec changes:** none (no flag surface change
  anywhere; `mnemonic-toolkit` is touched only under `docs/manual-gui/`).

## 10. FOLLOWUP resolution

- `mnemonic-gui/FOLLOWUPS.md::gui-prefilled-default-text-appends-on-type`
  (`:968-977`) → **resolved** in the Phase G shipping commit, citing this spec
  + the append-regression test + the 6-flag census.
- New GUI FOLLOWUP filed in the same commit: N4
  (`gui-number-set-affordance-ignores-schema-default`).
- This manual's FOLLOWUPS.md gains no new entry (Phase M is routine pin-bump +
  regen under existing gates); the Phase M commit message cites the spec.
