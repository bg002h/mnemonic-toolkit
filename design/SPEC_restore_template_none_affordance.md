# SPEC — restore `--template` `(none)` unset affordance (F1-shaped, A1-APPEND)

**Cycle:** `restore-form-single-sig-template-leaks-in-md1-mode`
**Kind:** cross-repo GUI-primary bugfix, F1-shaped. GUI-render-scoped ONLY (no toolkit `src/` / clap-surface / conditional-projection change), + a toolkit-manual ripple + a tutorial-corpus refresh.
**Template mirrored:** the export-wallet F1 `(none)` affordance (shipped 2026-07-05), ruled by `docs/manual-gui/design/agent-reports/gui-example-f1-mini-r0.md` and implemented at mnemonic-gui `52f7689`.

## Source SHAs (all cites grep-verified this authoring, 2026-07-05)
- **mnemonic-gui** `master @ cab940b` (`mnemonic-gui-v0.56.0`).
- **mnemonic-toolkit** `master @ 5294182f` (docs/manual-gui at `manual-gui-v1.2.0`; toolkit crate v0.75.0 line).
- FOLLOWUP source: mnemonic-gui `FOLLOWUPS.md:1064-1083` (filed at gui `1843abf`), slug `restore-form-single-sig-template-leaks-in-md1-mode`.

---

## 1. Problem statement + CONFIRMED premise

### 1.1 The papercut
mnemonic-gui's **restore** form renders `--template` as `FlagKind::Dropdown(TEMPLATES)` with `default_value: None` (`src/schema/mnemonic.rs:537-546`). The form-loop materialises a virgin Dropdown to `opts[0]` (`src/form/flag_defaults.rs`), so a fresh restore form always carries a present `--template = "bip44"` (single-sig). The user has **no way to clear it** — `TEMPLATES` has no `(none)`/`""` entry.

Restore's conditional models exactly ONE rule — `not(flag_present "--md1") → {--from, Required}` (`src/form/conditional.rs:1006-1012`). It **deliberately does NOT** model any `--template`↔`--md1` mutex. The toolkit projection `restore_conditional_rules()` emits the same single rule, pinned by `gui_schema_conditional_drift` with `("restore", 1)` in `SUBCOMMAND_FLOORS` (`tests/gui_schema_conditional_drift.rs:311`).

Consequently a GUI user driving restore in **`--md1` (multisig) mode** with the materialised single-sig default hits a hard refusal.

### 1.2 The premise — CONFIRMED EMPIRICALLY (recon #5)
The `mnemonic` CLI **rejects a single-sig `--template` in `--md1` mode with exit 2**. Exact site:

- `crates/mnemonic-toolkit/src/cmd/restore.rs:3068-3076` — inside `run_multisig` (the keyed-wallet-policy-md1 path reached via the `!args.md1.is_empty()` dispatch at `restore.rs:349`):
  ```rust
  if let Some(t) = args.template {
      if !t.is_multisig() {
          return Err(ToolkitError::ModeViolation {
              mode: "restore",
              flag: "--template",
              message: "--template (single-sig) does not apply in multisig --md1 mode; remove it",
          });
      }
  }
  ```
  `ModeViolation` maps to **exit 2** (`error.rs:618`). The gate fires **before** the md1 decode, so it triggers on the virgin `bip44` regardless of card completeness.

Empirical reproduction (release `mnemonic 0.74.0`, keyed 2-of-3 `wsh(sortedmulti(...))` md1 built from the tutorial's `MULTISIG_DESC`):

| invocation | exit | result |
|---|---|---|
| `restore --md1 <card> --template bip44` | **2** | `--template (single-sig) does not apply in multisig --md1 mode; remove it` |
| `restore --md1 <card> --template bip84` | **2** | same |
| `restore --md1 <full 6-chunk card> --template wsh-sortedmulti` | 0 | multisig restore (template inert — passes `!is_multisig()` gate, then never consumed) |
| `restore --md1 <full 6-chunk card>` (no template) | 0 | multisig restore — **byte-identical stdout AND stderr** to the `wsh-sortedmulti` run |

`grep args.template` across `restore.rs` → lines `373, 383, 502` (single-sig / non-md1 path) and `3068` (the md1-mode gate) ONLY. In `run_multisig` `args.template` appears **only** at the 3068 gate and is otherwise never consumed → a multisig `--template` value is purely inert in md1 mode; clearing it (`(none)`) yields identical output.

**Premise stands. The fix is not moot.** A materialised single-sig `bip44` in md1 mode is a real exit-2 trap; the only GUI escapes today are (a) selecting a MULTISIG template (inert but a lie — the user is not choosing a template) or (b) the refusal. The fix gives an honest escape: an appended `(none)` that clears `--template`.

### 1.3 Why `(none)` is also correct in single-sig (non-md1) mode
In the single-sig path, omitting `--template` means "emit all four" (`ALL_SINGLE_SIG = [bip44, bip49, bip84, bip86]`, `restore.rs:45-49`). So `(none)` is a legitimate, meaningful selection in single-sig mode too (broaden to all four), not merely an md1-mode escape hatch. The APPEND placement keeps `opts[0] = bip44` as the virgin default, so nothing about the current single-sig default behaviour changes.

---

## 2. Design — A1-APPEND (mirror F1 exactly)

**Append a trailing `""` UNSET sentinel to restore's `--template` opts, in a NEW restore-specific `RESTORE_TEMPLATES` const**, rendered as `(none)` via the existing `display_or` combo machinery. `default_value` stays `None`.

### 2.1 Why a NEW const (differs from a pure in-place append)
Restore currently shares the module-level `TEMPLATES` const (`src/schema/mnemonic.rs:69`, 10 values, no `""`) — `RESTORE_FLAGS[--template].kind = Dropdown(TEMPLATES)` (`:539`). `TEMPLATES` also feeds `bundle`/`verify-bundle` and (indirectly, via `CliTemplate::is_multisig()`) the `SINGLE_SIG_TEMPLATES`/`MULTISIG_TEMPLATES` partition consts. Appending `""` in place would leak the sentinel into every consumer and into the partition consts → a `schema_mirror` template-partition delta (out of scope) and a wrong bundle dropdown. **So, exactly as F1 did for export-wallet (`EXPORT_WALLET_TEMPLATES`), restore gets its own `RESTORE_TEMPLATES` const**; `TEMPLATES` stays 10 and untouched. This is the same 11-vs-10 asymmetry F1 established, now extended to a second per-flag const — intended and scoped.

### 2.2 EXACT GUI touch-points (mnemonic-gui)

| # | File / site | Change |
|---|---|---|
| **T1** | `src/schema/mnemonic.rs` — new `const RESTORE_TEMPLATES: &[&str]` beside `EXPORT_WALLET_TEMPLATES` (`:103`) | The 10 `TEMPLATES` values **in order** + a trailing `""` sentinel. Doc-comment mirrors `EXPORT_WALLET_TEMPLATES`' (append-not-prepend rationale; distinct from shared `TEMPLATES`; `(none)` display via `display_or`; the md1-mode-clear + all-four-single-sig semantics). |
| **T2** | `src/schema/mnemonic.rs` — `RESTORE_FLAGS[--template]` (`:537-546`) | `kind: Dropdown(TEMPLATES)` → `Dropdown(RESTORE_TEMPLATES)`. **`default_value` STAYS `None`** — `Some("bip44")` would trip `is_at_default` suppression and drop `--template` from a virgin single-sig run's argv. Virgin default still comes from `opts[0]` materialisation (= `bip44`). |
| — | `src/form/widget.rs` (combo) | **NO change.** Combo already iterates `*opts` and renders `""` as a selectable `display_or("(none)", opt)`; selecting it writes `Dropdown("")`. |
| — | `src/form/conditional.rs::restore` (`:1006-1012`) | **NO change.** Rule set stays 1 (`--from` Required unless `--md1`). Clearing `--template` needs no conditional rule — it just drops the flag from argv. |
| **T3** | `src/schema/mnemonic.rs` help text (optional, `:542`) | Optionally note the `(none)` affordance in the `--template` help string (non-load-bearing; keeps GUI tooltip honest). No gate depends on it. |

**No `gui_render_emit.rs` re-pin exists for restore.** Unlike F1 (which re-pinned the export-wallet exact-ASCII line at `tests/gui_render_emit.rs`), `gui_render_emit.rs` pins only `inspect / build-descriptor / export-wallet / compare-cost` — **restore is not among them** (grep-confirmed). So restore's GUI-side placement guard is the new TDD append-pin test (§5, cell 5), not a `gui_render_emit.rs` edit. Restore's exact-ASCII structural render lives only in the toolkit `.gui` transcript (§4, re-pinned there).

### 2.3 Render-scoped vs paired-projection — RULING
**RENDER-SCOPED ONLY.** Identical to F1. The alternative — a paired toolkit conditional projection adding `flag_present("--md1") → {--template, Disabled}` to `restore_conditional_rules()` so the GUI greys `--template` in md1 mode — is **REJECTED / out of scope** because:
1. It is a toolkit `src/` change (new projected rule) → not F1-shaped.
2. It forces `SUBCOMMAND_FLOORS` `("restore", 1) → ("restore", 2)` and a matching GUI `conditional::restore` rule, or the `gui_schema_conditional_drift` gate REDs (the FOLLOWUP names this exact trap).
3. `(none)` alone fully resolves the papercut with zero projection change.
A paired projection MAY be filed as a separate future FOLLOWUP (a "grey `--template` in md1 mode" UX nicety), but it is **not** this cycle.

---

## 3. Gate-inertness ruling (mirror F1 §2; restore is not special)

Appending `""` to restore's `--template` is **inert** on every automated gate:

- **`schema_mirror`** (flag-NAME gate, `tests/schema_mirror.rs`; upstream deser is `GuiSchemaFlag { name }` only — `src/schema_check.rs`). Restore's flag-name set is unchanged; Dropdown VALUES are never compared. **INERT.**
- **Template-partition drift** (`SINGLE_SIG_TEMPLATES`/`MULTISIG_TEMPLATES` vs toolkit `meta.template_groups`). Derived from `CliTemplate::is_multisig()`, NOT from any per-flag opts. `RESTORE_TEMPLATES` is a distinct const. **INERT.**
- **`gui_schema_conditional_drift`** synthesises satisfying states from the **toolkit-projected** rule (`not(flag_present --md1) → {--from, required}`), never enumerating the GUI's `--template` opts. Rule count stays 1 == `("restore", 1)` floor. **INERT.**
- **`build_descriptor_*` value pins** — build-descriptor `--archetype`/`--allow` only. **INERT.**
- **`gui_render_faithfulness`** — axes = presence/disabled/control-class/secret-masking; option-list contents explicitly out. **GREEN, not an oracle here.**
- **`hint_text_defaults`** — Dropdown with `default_value: None`; NULL interaction. **GREEN.**
- **`gui_form_snapshots` (61-form gallery `figures/gui/mnemonic-restore.png`)** — closed form, `opts[0] = bip44` virgin → byte-identical. **INERT (APPEND is load-bearing here).**
- **`argv_assembler*`** — explicit values still emit; materialised `--template bip44` still emitted on a virgin single-sig run. **INERT.**

The ONLY moved GUI-repo surfaces are the **new** TDD test file and the append behaviour itself. Placement (`opts[0]` must stay `bip44`) is the load-bearing cell — a future silent PREPEND would flip the virgin default to unset, enable "all four" on load, and move the gallery PNG. Cell 5 (§5) is the tripwire.

---

## 4. Cross-repo ripple — THREE legs + tag/release plan

### Leg 1 — GUI fix (mnemonic-gui)
T1–T3 (§2.2) + the TDD suite (§5). Full per-phase R0 + scoped post-impl on the diff. Gates: full `cargo test --jobs 2`, `schema_mirror`, `gui_schema_conditional_drift`, `gui_form_snapshots` (restore PNG byte-identical), `hint_text_defaults`, clippy `--all-targets` and `--no-default-features` (`-D warnings`), headless build. **Tag `mnemonic-gui-v0.57.0`.** (GUI = PR + CI-before-tag; no fmt gate; branch protection contexts = `[snapshots]`.) Note: the GUI fix also re-drives the tutorial-corpus (Leg 3a) **in the same tag** — see below; the tutorial-snapshots gate re-pins the affected restore PNGs in this tag.

### Leg 2 — Toolkit reference-manual ripple (mnemonic-toolkit `docs/manual-gui/`) — the F1-P2 analog
Bump the GUI pin and mirror F1's P2 exactly:

| surface | action |
|---|---|
| `docs/manual-gui/pinned-upstream.toml:55` | `[mnemonic-gui] tag = "mnemonic-gui-v0.56.0"` → `"mnemonic-gui-v0.57.0"`. The `[manual-gui]` `*-tag-implied` fields are **pin-neutral** (v0.57.0 pins the same `mnemonic-toolkit-v0.75.0` / md/ms/mk — this fix changes no CLI surface); re-verify against the v0.57.0 tag's own `pinned-upstream.toml`. |
| `docs/manual-gui/transcripts/gui/mnemonic-restore.gui:4` | RE-PIN. Regenerated by `verify-examples-gui` from the pinned `gui-render` (`Makefile:303-307`). Line becomes `--template  dropdown[bip44,…,tr-sortedmulti-a,(none)]  -> bip44` (append `,(none)`; `-> bip44` unchanged; every other line byte-identical). Doubles as the structural placement guard. |
| `docs/manual-gui/src/40-mnemonic/4d-restore.md` `--template` section (`:173-230`) | **REQUIRED** `(none)` addition, else `gui-schema-coverage` REDs. `check_gui_schema_coverage.py::build_expected` requires an anchor per variant; `kebab("") == ""` → required anchor `#mnemonic-restore-template-`. Add: (a) an Outline bullet `- [`(none)`](#mnemonic-restore-template-)` after `tr-sortedmulti-a` (`:194`); (b) a `### `(none)` {#mnemonic-restore-template-}` section. **Prose MUST differ from export-wallet's** — for restore `(none)` means "clear `--template`; in single-sig mode emit all four `bip44/49/84/86`, in `--md1` multisig mode it removes the single-sig template that the CLI rejects (exit 2) so the md1-driven multisig reconstruction runs cleanly." Also correct `:178-179` ("Same 10 values as `bundle --template`") to note restore now carries those 10 **plus** the GUI `(none)` sentinel (bundle stays 10). |
| `docs/manual-gui/tests/expected_gui_schema_inventory.json` (restore `--template`, near `:1949`) | DOCUMENTARY REGEN via `extract_gui_schema.py` (restore `--template` variants gains a trailing `""`). **No gate reads this file** (grep-confirmed: no `.rs`/lint consumer) → does NOT RED; regenerate for hygiene. |

### Leg 3 — Tutorial-corpus + prose refresh (spans BOTH repos)
The `gui_example.pdf` tutorial currently **routes around this exact bug**: every md1 restore step selects a MULTISIG template because the single-sig default is rejected in md1 mode (manifest comment `tests/tutorial/manifest.rs:11-14`). The tutorial is a usability instrument; the fix is not complete until it demonstrates the FIXED `(none)` flow rather than the workaround. **Verified: switching each restore step to `(none)` produces byte-identical run transcripts (stdout/stderr/exit), so `verify-tutorial-transcripts` stays GREEN with no transcript change; only the whole-window `-form.png` / `-run.png` change (the Template dropdown now reads `(none)`), which `verify-tutorial-figures` demands be re-pinned.**

**Affected steps — enumerated (all are keyed-multisig-md1 restores; NONE is a genuine single-sig restore, so ALL switch to `(none)`):**

| step (manifest stem) | journey | current workaround `--template` | shots captured? |
|---|---|---|---|
| `tut-j2-08-restore` (`manifest.rs:267-268`) | J2 2-of-3 | `wsh-sortedmulti` → `""` | yes (form+run) |
| `tut-j3-13-restore` (`:288-289`) | J3 11-key vault | `wsh-sortedmulti` → `""` | yes (form+run) |
| `tut-j4-17-restore` (`:306-307`) | J4 taproot | `tr-sortedmulti-a` → `""` | yes (form+run) |
| `tut-j4-nums-restore` (`:336-337`) | J4 NUMS | `tr-sortedmulti-a` → `""` | no (transcript-only feed) |
| `tut-j5-23-restore-descriptor` (`:345-348`) | J5 watch-only | `wsh-sortedmulti` (+`--format descriptor`) → `""` | yes (form+run) |
| `tut-j5-24-restore-core` (`:350-353`) | J5 watch-only | `wsh-sortedmulti` (+`--format bitcoin-core`) → `""` | yes (form+run) |

`--format` in md1 mode does NOT require `--template` (the "`--format` requires `--template`" gate at `restore.rs:395` is in the single-sig, non-md1 path; `run_multisig` supports `--format` via `build_multisig_import_payload`) — **empirically verified**: `(none) + --format descriptor` and `(none) + --format bitcoin-core` both exit 0 with stdout byte-identical to the current `wsh-sortedmulti` workaround. So J5 switches cleanly too.

**Leg 3a (mnemonic-gui, in the v0.57.0 tag):**
- `tests/tutorial/manifest.rs`: change `restore_drives!` call-sites' `$template` from the multisig value to `""` (J2-08, J3-13, J4-17, J4-NUMS) and the two inline J5 drives' `--template` value to `""`; update the `restore_drives!` doc-comment (`:136-143`) — remove the "multisig-`--template` route-around" framing, describe the `(none)` clear. Update the manifest header note (`:11-14`) from "papercut found → route around" to "papercut FIXED → the clean `(none)` md1 restore."
- Regenerate the affected tutorial screenshots via the tutorial-snapshots harness (kittest whole-window): `tut-j2-08-restore-{form,run}.png`, `tut-j3-13-restore-{form,run}.png`, `tut-j4-17-restore-{form,run}.png`, `tut-j5-23-restore-descriptor-{form,run}.png`, `tut-j5-24-restore-core-{form,run}.png`. These re-pin in the SAME v0.57.0 tag (BY DESIGN). `manifest-stems.txt` is unchanged (same step names). Run transcripts stay byte-identical → no transcript re-pin.

**Leg 3b (mnemonic-toolkit `docs/manual-gui/`, in Leg 2's manual-gui release):**
- Re-copy the changed tutorial figures into `docs/manual-gui/figures/tutorial/` (the 10 PNGs above); `verify-tutorial-figures` census demands the byte-updated copies. `transcripts/tutorial/` is unchanged (byte-identical outputs).
- Update the tutorial PROSE for the affected steps — remove the workaround explanation, describe the real `(none)` flow (mirroring how J2's export-wallet `(none)` is already taught at `docs/manual-gui/tutorial/30-j2-multisig.md:21-24,108-110,144`). Precise edits:
  - `30-j2-multisig.md:261-262` — "the **Template** drop-down is set to the wallet's `wsh-sortedmulti` family (a template is inert when restoring from an `md1` card …)" → "set the **Template** drop-down to **`(none)`** — the card carries the full policy, so no single-sig template applies (selecting one would be rejected in `--md1` mode)."
  - `40-j3-degrading-vault.md:169` — "Template set to the wallet's `wsh-sortedmulti` family" → `(none)`.
  - `50-j4-taproot-twin.md:313` — "Template `tr-sortedmulti-a`" → `(none)`.
  - J5 (`60-j5-watch-only.md`) — the two `restore --format` steps' Template mention → `(none)`.
- `tutorial-xref` still passes (same artifact stems embedded). Rebuild + re-attach `gui_example.pdf`.

### Release plan (ordered)
1. **Leg 1 + Leg 3a together** → PR on mnemonic-gui → CI GREEN (`snapshots` + `tutorial-snapshots`) → merge → **tag `mnemonic-gui-v0.57.0`**.
2. **Leg 2 + Leg 3b together** on mnemonic-toolkit → pin bump to v0.57.0 → `make -C docs/manual-gui` regen (`.gui`, inventory, tutorial figures) → all `docs/manual-gui/tests/lint.sh` phases GREEN (`gui-schema-coverage`, `verify-examples-gui`, `verify-tutorial-figures`, `verify-tutorial-transcripts`, `tutorial-xref`) → **release `manual-gui-v1.3.0`** (next after `manual-gui-v1.2.0`) with the rebuilt `gui_example.pdf` attached.

No `mnemonic`/`md`/`ms`/`mk` CLI change; no toolkit crate version bump; no crates.io publish. `docs/manual/` (the non-GUI manual) is untouched — restore's `--template` clap surface is unchanged, so the CLI-reference mirror invariant does not fire.

---

## 5. TDD test plan (mnemonic-gui, RED-first, always-run — mirror F1's 5 cells)

New file `tests/restore_template_none.rs` (mirrors `tests/export_wallet_template_none.rs`), all cells RED before T1/T2, GREEN after; not env-gated:

1. **Reachability / clear (the regression test).** Drive a virgin restore form → select `(none)` → assert the assembled argv carries **no `--template` token** and no `""` sliver, and `has_value("--template")` is false. (Restore has no descriptor-arm to unlock, so this cell asserts the *clear*, not an enable.)
2. **`(none)` never leaks `""`.** Selecting `(none)` yields no `--template` in argv and no `""` / empty-string artifact in the masked copy-command (both `ShellFlavor`s). Re-selecting `bip44` re-emits `--template bip44`.
3. **Materialisation / `has_value` both ways.** Virgin `has_value("--template")` true (materialised `bip44`); after `(none)`, false; after re-selecting a real value, true. Assert the `restore` conditional rule set is UNCHANGED across all three (still exactly `{--from Required}` when `--md1` empty; the transition touches no rule) — the render-scoped guarantee.
4. **`(none)` display.** The sentinel renders `(none)` in the OPEN popup (one row) and, via `display_or`, at the closed-combo seam — never a bare `""`; all 10 real rows still listed. Stored value is the raw `Dropdown("")`.
5. **★ Virgin-default stability (the APPEND census guard).** `RESTORE_TEMPLATES.len() == 11`, `opts[0] == "bip44"`, `opts.last() == Some("")`; `opts[..10]` equals the shared `TEMPLATES` in order; a virgin settled restore form materialises `--template == "bip44"`, `has_value` true. Assert `TEMPLATES.len() == 10` (shared const untouched; the sentinel is restore-only, distinct from `EXPORT_WALLET_TEMPLATES`). This tripwires a future silent PREPEND (which would move the gallery PNG + flip the virgin default). Demonstrate the prepend variant RED before landing.

**Adjacent-gate assessment (each RUN, expected status):** `schema_mirror` GREEN/INERT · `gui_schema_conditional_drift` GREEN/INERT (rule count 1) · template-partition GREEN/INERT · `gui_form_snapshots` GREEN/INERT (restore PNG byte-identical) · `hint_text_defaults` GREEN/NULL · `argv_assembler*` GREEN/INERT · tutorial-snapshots — restore `-form`/`-run` PNGs RE-PIN by design (Leg 3a), transcripts byte-stable.

---

## 6. Secret-hygiene note

The `--template` change is a **non-secret Dropdown** (`secret: false`) — masking is unaffected. Restore's classified-secret surface is untouched: `--passphrase` (`src/schema/mnemonic.rs:672`, `secret: true`), `--passphrase-stdin` (`:683`, `secret: true`), and `--from`'s composite secret nodes (`phrase`/`ms1`/`entropy`/`seedqr`) routed through the secret confirm-modal. The appended `""` sentinel never becomes a value that could carry secret material (it is presence-suppressed to nothing in argv), and it appears in no masked copy-command. Cell 2 pins the no-`""`-leak property. No new owned secret is introduced; no zeroize/redaction surface changes.

---

## 7. STOP conditions

STOP and re-review (back to this spec) if implementation surfaces any of:
- Movement of any OTHER form's gallery PNG, any OTHER subcommand's `.gui` / emit / coverage anchors, or `bundle`/`export-wallet` `--template` opts (the restore↔bundle 11-vs-10 asymmetry is intended and restore-scoped — do not "fix" bundle; the two per-flag sentinel consts `EXPORT_WALLET_TEMPLATES` + `RESTORE_TEMPLATES` are both intended).
- Any `gui_schema_conditional_drift` rule-set delta on restore (would signal an accidental conditional change → violates the render-scoped ruling).
- A tutorial restore transcript (`.stdout/.stderr/.exit`) changing under `(none)` (recon says byte-stable; a change means an md1-mode `(none)` behavioural difference not predicted here — investigate before re-pinning).
- Any need for a toolkit `src/` / clap-surface / conditional-projection edit (would mean the fix is NOT F1-shaped → escalate; this spec asserts render-scoped is sufficient).

---

## 8. Ruling summary
- **Premise:** CONFIRMED (recon #5) — single-sig `--template` in `--md1` mode → exit 2 (`restore.rs:3068-3076`); virgin `bip44` hits it. Fix is not moot.
- **Design:** A1-APPEND, new restore-specific `RESTORE_TEMPLATES` const (restore shares `TEMPLATES` today → needs its own const, exactly like F1's `EXPORT_WALLET_TEMPLATES`), `default_value` stays `None`, `opts[0]` stays `bip44`.
- **Scope:** RENDER-SCOPED ONLY — no toolkit `src/` / clap / conditional-projection change. Paired projection explicitly rejected/out-of-scope.
- **Gates:** all inert (flag-NAME `schema_mirror`, partition consts, conditional-drift, gallery PNG). The only re-pins: the new TDD file, the toolkit `.gui` line, the coverage `(none)` section, the inventory hygiene regen, and the tutorial restore PNGs.
- **Ripple:** 3 legs — GUI fix (tag `mnemonic-gui-v0.57.0`, incl. tutorial-corpus re-drive) → toolkit manual pin/`.gui`/coverage/inventory + tutorial figures/prose → release `manual-gui-v1.3.0` with rebuilt `gui_example.pdf`.
