# R0 review — SPEC_gui_hint_text_defaults.md (round 1)

- **Reviewer:** opus-tier architect, adversarial R0 (gate: 0C/0I before any plan/code).
- **Artifact:** `docs/manual-gui/design/SPEC_gui_hint_text_defaults.md` (DRAFT, 2026-07-01).
- **Ground truth verified against:** `mnemonic-gui` working tree == `master @ 7e9dcca7b740b138e7133c44a9709c4f9010aa66` (exact match to the spec's recon SHA; clean checkout), egui registry source `egui-0.31.1`, toolkit working tree (`docs/manual-gui/`, `crates/mnemonic-toolkit/`), `mnemonic-secret/crates/ms-cli`, and the **live `mnemonic gui-schema` v5 JSON** (v0.74.0 release binary).

## Verdict

**RED — 0 Critical / 1 Important / 4 Minor.**

The recon is solid: every load-bearing claim was independently re-derived and held — the census is exact, the argv-suppression mechanics are exactly as cited, the one-resolver architecture is real and exhaustive, the conditional/consumer sweep is complete, and the blast radius (6 PNGs / 6 `.gui` / 6 figures / 1 exact-ASCII pin) is complete and correct. The "smaller than feared" conclusion **survives adversarial re-derivation**. The single Important is a false statement in the §2.6/§4 truthfulness argument (the "mirror-construction" premise is empirically wrong for `--feerate`), which does not change the design but must be corrected because it sets a concrete future-drift trap. Fold + re-dispatch per the standing reviewer-loop rule.

---

## Recon-claim verification results

### Claim 1 — argv already omits at-default values: **VERIFIED**

- `is_at_default` at `src/form/invocation.rs:45`; compare arms exactly as cited: Text `s.is_empty() || s == default_str` at **:61**, Path `p == default_str` at **:64**, Dropdown at **:84**.
- `emit_one` at **:392**, D33 gate `if is_at_default(flag, value) { return; }` at **:398-400**. Text `!v.is_empty()` guard at **:402-403**; Path `p.is_empty()` early-return at **:474** (within the cited :473-475).
- Pinning test `cell_import_wallet_select_descriptor_default_suppressed` at `tests/kittest_import_wallet_form.rs:225-267`: seeded `Text("all")` asserted ABSENT from argv; `active-receive` asserted present. Matches the spec's description.
- **Edge — user types the literal default:** suppressed identically pre/post fix (`is_at_default` is untouched; Text string-compare, Path `-` literal match). Same for the Path `stdio` button writing a literal `-` (visible text, suppressed at emission). Zero argv delta.
- **Edge — empty vs absent:** for every kind, empty and absent produce identical argv. Text: `!v.is_empty()` guard; Path: `p.is_empty()` return; Dropdown: `!v.is_empty()` guard; Boolean presence-only; Composite empty-suppressed. **No flag exists where empty-string vs absent differ in argv.**
- The only observable flip is `flag_value_is_present` (`src/schema/mod.rs:496-505`: Text/Path present iff non-empty) going true→false on a fresh form for the 6 — reader count is zero (Claim 4).

### Claim 2 — census 82 / per-kind counts / exactly 6 Text+Path: **VERIFIED (independently re-derived)**

- Re-parsed all 4 schema tables (kind-tracking awk over `FlagSchema` blocks): `default_value: Some` count = **mnemonic 69 / ms 8 / mk 3 / md 2 = 82**; per-kind **40 Dropdown / 34 Number / 4 Text / 2 Path / 1 Range / 1 Timestamp**. Byte-for-byte match with the spec's table.
- The 6 stems re-derived: `compare-cost --feerate` Text "1.0" (`mnemonic.rs:2262-2270`), `import-wallet --select-descriptor` Text "all" (`:2554-2570`, block-open at :2554 as cited), `nostr --timestamp` Text "0" (`:3552-3560`), `export-wallet --output` Path "-" (`:1477-1486`), `restore --output` Path "-" (`:653-662`), `ms derive --account` Text "0" (`ms.rs:279-287`). All six `required: false` AND `repeating: false` (all scalar — the ghost patch's scalar `render_row` arms suffice).
- Toolkit clap side verified live: `compare_cost.rs:32` `default_value_t = 1.0`; `import_wallet.rs:~165` `default_value = "all"`; `nostr.rs:112` `default_value = "0"`; `export_wallet.rs:272` and `restore.rs:242` `default_value = "-"`; `ms-cli derive.rs:43-44` `default_value_t = 0`.
- **Zero secret flags carry a default:** confirmed by block-scan of all 4 schema files (the one scanner hit was a regex block-boundary artifact; direct read shows every `ms.rs --json` is `secret: false, default_value: None`) and by the enforcing test `secret_flags_never_carry_a_default_value` (`tests/gui_form_snapshots.rs:189`) — verified **always-run** (deliberately NOT behind the `GUI_SNAPSHOTS` env gate, per its own doc comment).

### Claim 3 — Number/DragValue has NO papercut: **VERIFIED (both halves)**

- Half 1: `flag_defaults.rs:63` resolver; Text/Dropdown/Path arms at **:68-70**; Number/Range/Timestamp/TaggedOrIndexed → `Unset` at **:81-84**. Set affordance at `widget.rs:~460-474` (button "Set" → `seeded_value_for`); nothing pre-fills.
- Half 2: egui 0.31.1 registry source `drag_value.rs` click-to-edit branch (~:616-628) sets `CCursorRange::two(CCursor::default(), CCursor::new(value_text.chars().count()))` — select-all before focus; typing REPLACES. `hint_text` does not exist in `drag_value.rs` (grep: zero hits). `TextEdit::hint_text` at `text_edit/builder.rs:205`.
- In-repo precedent confirmed: `slot_editor.rs:57` `TextEdit::singleline(&mut row.value).hint_text(hint)` + `tests/slot_editor_path_hint_text.rs` exists.

### Claim 4 — no conditional/src consumer reads any of the 6: **VERIFIED (with a name-collision caveat → Minor m1)**

- Exactly **17** conditional fns enumerated (`bundle:193, verify_bundle:384, convert:471, export_wallet:589, build_descriptor:656, derive_child:703, ms_encode:730, mk_encode:756, md_encode:780, md_compile:813, md_address:831, slip39_split:855, slip39_combine:882, repair:949, inspect:958, compare_cost:968, restore:1006`). No fn for import-wallet, nostr, or ms derive.
- Bodies read: `compare_cost` reads only `--miniscript`/`--descriptor` (:968-985); `export_wallet` reads `--descriptor`/`--template`/taproot/threshold (:589-625) — NOT `--output`; `restore` reads only `--md1` (:1006-1012); `slip39_combine`'s `--to` predicate treats `None` == `Some("entropy")` (:897-900) — seed-robust as claimed.
- src-wide grep for the 6 names: the only hits outside `src/schema/` are **bundle's Number-kind `--account`** (`conditional.rs:229-248` PinValue rule; `main.rs:306` hand-seed; `main.rs:762` path-hint reader) — a *different subcommand's flag entry*; conditional fns and FormState are subcommand-scoped, so the claim holds. `--output`/`--timestamp`/`--feerate`/`--select-descriptor`: zero src consumers.
- `is_render_suppressed` (`render_emit.rs:523-544`) keys only on `--slot`/allows_slots and build-descriptor tree/archetype modes — none of the 6.

### Claim 5 — one-resolver fix / no fourth consumer / re-pin enumeration: **VERIFIED**

- Exhaustive grep for `default_flag_value_for_flag`: `widget.rs:223` (scalar seed), `:314` (required-repeating seed), `:674` (Unset-shape recovery), `render_emit.rs:142,150` (`seeded_fixture`), `:604` (`flag_value_str`), plus re-export `widget.rs:27` / import `render_emit.rs:30`. **No fourth consumer** — `main.rs`, `persistence.rs`, `conditional.rs`, and the test suite never call it. Editing the Text/Path arms moves widget seed, emit fixture, and value column atomically, as claimed.
- Faithfulness gate: `gui_render_faithfulness.rs:26-30` explicitly excludes "default/placeholder TEXT" from its axes — survives by construction. ✓
- Re-pin enumeration is **complete**: of the exact-ASCII pins in `gui_render_emit.rs`, only export-wallet (`:83-116`, contains `--output  path(stdio)  -> -`) covers one of the 6 forms; the others pin mnemonic-inspect/build-descriptor/mk-inspect/bundle-placeholder/ms-inspect (none among the 6). Transcript grep: **exactly 6** `.gui` files carry the seeded defaults (`mnemonic-{compare-cost,import-wallet,nostr,export-wallet,restore}.gui`, `ms-derive.gui`) → 6 PNG + 6 `.gui` + 6 `figures/gui` + 1 pin is the full blast radius.

### Claim 6 — persistence normalization: **VERIFIED as designed; mechanics under-specified (Minor m2)**

- Pre-fix autosaves do carry the seeded defaults (`redact_for_persistence` `persistence.rs:77+` persists non-secret `state.values`; per-frame seed pushes the default back into `state.values` at `widget.rs:223-229`). Load hook exists: `persistence::load` at `:323`, called from `main.rs:48`.
- Eating a USER-typed value equal to the default (at next load): **acceptable and correctly reasoned** — argv byte-identical (`is_at_default` suppresses both states), visibility-identical (zero readers, Claim 4), and display near-identical (the ghost shows the same string the user typed). Load-time-only scoping avoids fighting the keyboard. Sound.
- Secret interaction: **disjoint by construction** — the redaction layer is name/class-based and strips secrets *before* write; zero secret flags carry defaults (gate-enforced); normalization operates on the non-secret residue only.

### Claim 7 — hint-text UX contract + `<hint:d>` sentinel: **VERIFIED**

- Sentinel grammar siblings confirmed in `flag_value_str` (`<empty>`, `<unset>`, `MASKED`, `<pinned: …>`); `<hint:d>` is unambiguous. All 6 hint payloads are ASCII (`1.0`, `all`, `0`, `-`, `-`, `0`); the always-run ASCII form test passes by inspection; `verify-examples-gui` byte-gates regenerate-vs-committed from the same pinned binary → self-consistent; mdBook code-fence inclusion is literal-safe.
- Value-column honesty: `-> <hint:1.0>` (vs old `-> 1.0`) is the honest depiction under the render charter ("the screen the user sees on load", `render_emit.rs:72-81`) — the ghost IS what the user sees on load.

### Claim 8 — cycle shape + ripple completeness: **VERIFIED**

- Phase G matches the GUI ship ritual (PR + CI-green before tag). `snapshots` CI job at `build.yml:87-140` (no path filter, `.new.png` count check). `Cargo.toml` version `0.54.0` ✓ → `0.55.0`. `schema_mirror` untouched-claim ✓ (no `default_value` reference in `tests/schema_mirror.rs`; no `src/schema/*` edit).
- Phase M: `pinned-upstream.toml` `[mnemonic-gui] tag = "mnemonic-gui-v0.54.0"` (line **30**, spec says :31 — nit m3); `Makefile:292-297` `verify-examples-gui` + `EXPECTED_GUI_RENDER_COUNT ?= 61` (`:288`); `transcripts/gui/` = 61 files; `figures/gui/` = 61 files; lint phase 9 `verify-figures-gui` (`tests/lint.sh:23,193`); `.github/workflows/manual-gui.yml` exists. Prose: `4c-import-wallet.md:162-166` "free-form text input pre-filled with `all`" ✓; `53-encode.md:41-42` "spin-box pre-filled with `5`" confirmed **stale today** (md encode `--group-size` is Number → `Unset` + `Set`) — correcting it as a pre-existing-inaccuracy is right.
- FOLLOWUP cites: `mnemonic-gui/FOLLOWUPS.md:968-977` entry matches (lists exactly the 6 flags); `:1014-1040` corpus-consumer contract real, incl. the step-0 tag-push `snapshots` provenance check.
- **Bonus 1 (sweep workaround removal): SOUND.** `prepared_eligible_base` (`ui_harness_sweep.rs:~172-200`) strips the under-test flag then re-seeds `Text("")`/`Path("")`; post-fix the strip alone yields the same empty first-render buffer, so removal is behavior-identical AND converts every Text/Path I1 round-trip into a permanent append-regression tripwire (a re-introduced pre-fill would make `type_text` append and fail the round-trip). Safe for no-default Text/Path too (the re-seed was already a no-op there). The other harness tables are untouched as claimed (`ui_harness/mod.rs:148-176` base_state seeds only addresses context flags; I2's two `default_value:` mentions at `:602,612` are both `None` in synthetic schemas).
- **Bonus 2 (N4 filed-not-fixed): RIGHT SCOPE CALL, example verified.** `--gap-limit` is `Number { min: 0 … }` with `default_value: Some("20")` (`mnemonic.rs:3133-3143`); `seeded_value_for` seeds `min` (`widget.rs:379-381`) → clicking Set yields 0, which then **emits** `--gap-limit 0` (0 ≠ 20, not suppressed). Fixing it changes argv behavior — correctly out of this UX cycle.

---

## Findings

### Critical

None.

### Important

**I1 — §2.6/§4 "mirror-construction" premise is empirically FALSE for `--feerate`; reword so ghost-truthfulness rests on the spot-checks, and document the override to defuse a future-drift trap.**
Evidence: the live `mnemonic gui-schema` v5 JSON (v0.74.0 binary) emits `compare-cost --feerate` as kind **`"number"`** with default_value **`1`** (a JSON number — clap's `default_value_t = 1.0` renders via `f64::to_string()` → `"1"`). The GUI hand-mirror deliberately deviates on BOTH axes: `FlagKind::Text` + `Some("1.0")`, documented at `src/schema/mnemonic.rs:2231-2236` ("`--feerate` is a Text widget rather than `FlagKind::Number` because … the GUI's Number kind is i64-only"). So the spec's sentences "the GUI schema's `default_value` strings are hand-mirrored from `mnemonic gui-schema` v5 JSON, which extracts them from clap itself" (§2.6) and "Ghost string == clap default by mirror-construction (`gui_schema.rs:1184`)" (§4) are wrong for 1 of the 6.
Impact: the ghost `1.0` **remains semantically truthful** (the clap default IS 1.0; typing the literal `1.0` parses identically), so the DESIGN is unchanged — but a spec whose §2 is titled "every claim verified at source" must not carry a false verification chain, and this specific falsehood is a trap: a future "restore mirror fidelity" pass could flip feerate to number/`1` believing it corrects drift, breaking the documented i64-vs-f64 override (schema_mirror would not catch it — it gates neither kinds nor default strings).
Fold: reword §2.6 + §4 to (a) state that 5 of the 6 mirror the v5 strings exactly, (b) call out the feerate kind+string override with the `mnemonic.rs:2231-2236` cite and the v5-emits-`number 1` fact, (c) anchor ghost-truthfulness on the §2.3 live clap-attribute spot-checks + the N3 trust model rather than a mechanical mirror guarantee. One paragraph; no design change.

### Minor

**m1 — §2.5: note the flag-NAME collisions so implementer greps don't false-alarm.** `--account` also exists as bundle's Number-kind flag with a conditional `PinValue` rule (`conditional.rs:229-248`) plus `main.rs:306`/`main.rs:762` consumers; `--timestamp` also exists as export-wallet's Timestamp-kind flag. Both are different subcommand-scoped flag entries; the "zero consumers" claim survives, but the spec's src-grep statement is only true under subcommand scoping — say so.

**m2 — §3.4: specify the normalization mechanics.** (a) per-subcommand schema lookup from the persisted `tab:sub` key; (b) fail-open on unknown subcommand/flag (keep the entry); (c) kind-scoped to Text/Path entries only (leaves bundle's `--account` `Number(0)` hand-seed and all Number defaults untouched); (d) post-fix autosaves legitimately carry `Text("")`/`Path("")` entries which must NOT be dropped (they don't equal the default) — add that as a third test vector next to the two in §7.

**m3 — cite nit:** `pinned-upstream.toml` `[mnemonic-gui] tag` is line **30**, not 31.

**m4 — §7 ghost-presence test (suggestion):** state explicitly that egui paints `hint_text` without entering the text buffer, so the AccessKit assert is "value == ''" while the snapshot shows the ghost — this is the anti-tautology anchor tying the `.gui` `<hint:d>` notation to real widget behavior.

---

## Gate

**RED (0C / 1I / 4m).** Fold I1 (and Minors as desired), persist this review, re-dispatch for round 2 per the standing "reviewer-loop continues after every fold" rule. The fix design itself needed no change under adversarial verification — all folds are spec-text corrections.
