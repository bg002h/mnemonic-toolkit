# R0 Review — IMPLEMENTATION_PLAN_generated_gui_form_renders.md (Round 2)

**Reviewer:** opus architect (mandatory pre-implementation plan-R0 gate; 0C/0I required, BEFORE any code). Round-2 = confirm convergence after the round-1 fold (0C/2I/7m) + hunt for fold-introduced drift. Settled points NOT re-litigated.
**Artifact:** `mnemonic-toolkit/docs/manual-gui/design/IMPLEMENTATION_PLAN_generated_gui_form_renders.md` (rev: "plan-R0 r1 RED (2I/7m) folded").
**Spec:** `SPEC_generated_gui_form_renders.md` (R0-GREEN).
**Verified live at this session:** manual-gui infra at toolkit `master` (`pinned-upstream.toml`, `tests/lint.sh` 7-phase, `tests/check_gui_schema_coverage.py`, `tests/check_outline_coverage.py`, `tests/extract_gui_schema.py`, `tests/expected_gui_schema_inventory.json`, `.github/workflows/manual-gui.yml`, `transcripts/`) + mnemonic-gui `master` `pinned-upstream.toml` (v0.52.0). Every fact below is grep/file-confirmed at those checkouts.

---

## VERDICT: GREEN — 0 Critical / 0 Important / 5 Minor-Nit

**Converged.** Both round-1 Importants are folded soundly and the folds did not introduce a Critical or Important drift. I1 → a real new **P4 catch-up phase** correctly sequenced BEFORE the renders, with the schema-coverage debt, inventory regen, and the 3-version-site lockstep enumerated. I2 → a single egui-free `render_fixture(tab,sub) -> FormState` (canonical `FormState::default()`) consumed by **both** the P2 emit and the P3 faithfulness render — the `emit == render` gate is now definitionally over one S. All 7 minors (m1 `[[bin]]` required-features, m2 hyphen bin name, m3 three version-sites, m4 tree-observable projection, m5 relocate the `default_flag_value_for` pair, m6 PR-#24 harness re-point, m7 stable byte encoding) are present in the plan text. A1 and the two-leg ordering stand.

The 5 Minors below are recommendations (the highest-value is N1 — add `make verify-examples` to P4's task+gate, because the pin bump also drags the **CLI-transcript** tier v0.70→v0.74). None blocks the gate; all are self-correcting at CI/build. **Implementation may begin.**

---

## CRITICAL (0)

None. No funds/correctness sink (docs/test-infra tier; secret hygiene handled — fixtures are `FormState::default()`/public, secrets rendered as the fixed `<masked>` sentinel via `secrets::flag_is_secret`, the gate diff cannot leak).

## IMPORTANT (0)

None. Both folds verified coherent (see per-question findings). No new Important surfaced.

---

## Convergence verification (the 5 charged questions)

### Q1 — I1 catch-up (P4): soundly sequenced + scoped. ✓ (one Minor: N1)

- **Sequencing is correct.** P4 (catch-up + pin bump) is its own phase in Leg 2, gated `make lint` green, landing BEFORE P5's render embed + `verify-examples-gui`. Confirmed why this ordering is load-bearing: `check_gui_schema_coverage.py:170` extracts the inventory **LIVE** from the cloned pinned upstream (`extract_gui_schema.extract(args.upstream_root)`), so the instant `pinned-upstream.toml` bumps to the new tag, `lint.sh` phase 4 sees `word-card`/`gen-man`×4/`inspect --json` in the schema and REDs until the manual carries matching anchors. P4 authors them first → phase 4 green → P5's renders land on a green base. Right call.
- **Schema-coverage scope is right.** Live `mnemonic-gui/pinned-upstream.toml` confirms the new tag's tier and the magnitude: the 5 undocumented subcommands are real and the manual has **zero** `word-card|gen-man` hits today. P4 names all five + the per-flag anchors + "word-card full section / gen-man stubs / inspect `--json`". Correct.
- **The 3 version-sites are complete and the new GUI tag's implied pins are knowable, not guessed.** Live `mnemonic-gui/pinned-upstream.toml` pins `mnemonic-toolkit-v0.74.0` / `descriptor-mnemonic-md-cli-v0.11.0` / `ms-cli-v0.13.0` / `mk-cli-v0.11.0`; manual `pinned-upstream.toml` currently mirrors v0.70.0/v0.7.0/v0.8.0/v0.9.0 and the `verify-examples` job (`manual-gui.yml:155,158,161,172`) hardcodes the same v0.70.0 tier. So P4's lockstep = (1) `pinned-upstream.toml` GUI tag + 4 implied pins → the v0.74.0 tier, (2) the `verify-examples` job's hardcoded CLI tags → same tier, (3) the new `gui-render` install tag → the new GUI tag. Confirmed.
- **Inventory regen:** see N2 — it is **hygiene, not gate-load-bearing** (nothing reads the committed JSON), but folding it is harmless/correct.
- **outline/glossary/index obligations:** see N3 — `word-card` (~10 flags ≥ 2) WILL require an `### Outline {#mnemonic-word-card-outline}` block (`check_outline_coverage.py:75`); glossary/index fire only if the author marks defined-terms/`\index{}`. All are union-gated by `make lint` (P4's stated gate runs all 7 phases), so the gate empirically backstops completeness — but enumerate them so "word-card full section" is sized to include them.

### Q2 — I2 shared fixture: coherent. ✓

- **One source, both consumers.** P1 defines `render_fixture(tab,sub) -> FormState` egui-free in `src`; P2 emits from `conditional(render_fixture(tab,sub))`; P3 renders "all 61 forms (same `render_fixture`)". Because both legs obtain S from the identical function, `emit == render` is **definitionally** over the same S regardless of what `render_fixture` returns — that is exactly the property I2 demanded. Coherent.
- **`FormState::default()` is a valid base for all 61.** Confirmed against round-1's grep (it is `sweep_candidate_bases`'s first element for every form, and spec §6's "generic/default-mode base" = that blank-form screen). It renders the canonical "first-open" form (required markers + `<empty>` values), which is documentable for all 61 incl. the 17 mode forms (default mode shown; per-mode renders are an accepted §6 follow-on).
- **Non-default-screen handling survives.** The plan simplifies to `FormState::default()` and drops round-1's explicit "per-form override table," but this does NOT break the invariant: any future non-default screen is encoded INSIDE `render_fixture`, so emit and render stay co-sourced. (The simplification also collapses the m6 harness ripple — `sweep_candidate_bases` need not relocate; only the slot types + `SecretLineEdit` do, which P1/m6 already cover.) Sound.

### Q3 — m4 faithfulness re-scope: sound; no ungated render content. ✓ (one precision Minor: N4)

- **The non-observable claim is correct.** Path and Text both render `Role::TextInput` (path-vs-text invisible to AccessKit); the `required` marker has no distinct widget; default/placeholder hint-text is not robustly AccessKit-exposed. Asserting only presence/`is_disabled`/Role-class/`PasswordInput`/positional-presence/action-bar is the right, non-thrashing comparator.
- **No column is gated by nothing.** Every `.gui` column is gated by AT LEAST `verify-examples-gui`'s byte-diff regen-determinism (P5), which re-emits from the GUI's own model and diffs == committed — so path-vs-text/required/default cannot silently drift from the GUI's schema/`default_flag_value_for_flag`. The observable subset is ADDITIONALLY tied to the live rendered tree by P3. Correctness-vs-real-CLI of the non-observable columns rests on the GUI schema's `FlagKind`/required + the shared default resolver — which is the GUI's own source of truth and the best achievable; it is a pre-existing property, not a regression this cycle introduces. (Precision nit N4: P3 cites "schema_mirror" as a co-gate for those columns; schema_mirror gates flag-NAMES + dropdown-value enums, NOT `FlagKind`/required/default — the real co-gate is the regen-determinism byte-diff + the shared schema/resolver source.)

### Q4 — Fold drift: none material. ✓ (two wording Minors: N2, N5)

- **render_fixture P1↔P2↔P3:** consistent (defined P1, consumed P2 + P3). No dangling ref.
- **`[[bin]] main required-features=["gui"]` vs `--no-default-features`:** coherent. P1's gate `cargo build -p mnemonic-gui --no-default-features` (no `--bin`) skips `main` (required-features unmet), builds `lib` + the non-gated `gui-render` — a superset of spec §3's `--bin gui-render` form. No contradiction.
- **P4 pin bump vs P5 gui-render install — same tag:** yes, both carry the new GUI tag. One wording wrinkle (N5): P4 enumerates "the new `gui-render` install line" as version-site #3, but that line is physically authored in P5 (the `verify-examples-gui` step doesn't exist until then). Tag VALUE is identical and same-leg, so no coherence break — just state that P4 bumps sites #1+#2 (both exist today) and P5 creates site #3 pinned to the same GUI tag.
- **Leg ordering:** Leg 1 tags first; P4 "AFTER the GUI tag" + line 47 "don't start P4 until it's pushed." Correct.
- **GUI MINOR + new bin non-interaction:** re-confirmed. New `gui-render` bin + default-on `gui` feature do not touch the toolkit `gui-schema` surface that GUI's `schema_mirror` pins; `pin_coherence`/`--locked --no-default-features` unaffected (lock is a superset). v0.52.0 → v0.53.0 MINOR is right.

### Q5 — New Critical/Important: none.

The only new finding of substance is N1 (CLI-transcript tier drift), assessed Minor below — likely no-op, mechanical if not, and fail-closed at CI.

---

## MINOR / NIT (5 — recommended, non-blocking)

- **N1 (highest-value) — P4's pin bump also drags the CLI-transcript `verify-examples` (Job 1b) gate; add `make verify-examples` to P4's task + gate.** The forced implied-pin bump is not just the GUI tag: it moves the CLI tier v0.70.0→**v0.74.0** (toolkit), v0.7.0→v0.11.0 (md), v0.8.0→v0.13.0 (ms), v0.9.0→v0.11.0 (mk) — and `manual-gui.yml`'s **Job 1b `verify-examples`** installs that hardcoded tier and runs `make verify-examples` against **17 committed `.out` goldens** (bundle/verify-bundle/convert/derive-child/seed-xor/slip39/xpub-search/ms-*/mk-*). P4's stated gate is only "`make lint` + `make md`/`html`" — it does NOT run `verify-examples`, yet Job 1b runs in the same LEG-2 manual-PR CI and the pin bump feeds it. **Why only Minor, not Important:** drift is plausibly a no-op (no version strings in the goldens — grep clean; the intervening cycles ADDED subcommands rather than changing existing-command output, and the secret-keymat sweep proved byte-identical), and if drift does occur it is mechanical golden-regen, fail-closed (caught at CI before merge, cannot silently ship wrong). **Fold:** add "run `make verify-examples` against the bumped tier; regenerate any drifted `.out` goldens (and, if an output FORMAT changed, the surrounding prose — flag as scope expansion)" to P4's task list and gate, so the implementer is not surprised at Job 1b.

- **N2 — `expected_gui_schema_inventory.json` regen is hygiene, not gate-unblocking; say so.** Confirmed: `check_gui_schema_coverage.py` extracts the inventory LIVE from the upstream clone (`:170`); NO Makefile target / lint phase / CI step diffs the committed JSON against a fresh extract (grep-confirmed — only `extract_gui_schema.py`'s docstring references it). So the actual gate-unblock for phase 4 is (a) the `pinned-upstream.toml` bump + (b) authoring the sections/anchors — both in P4. Regenerating the snapshot keeps the committed reference current (good), but the plan should not imply it is what turns `gui-schema-coverage` green (round-1 line 37 carried that slight over-attribution into the plan).

- **N3 — enumerate the outline/glossary/index obligations inside "word-card full section."** `check_outline_coverage.py:75` requires an `### Outline {#mnemonic-word-card-outline}` block for any subcommand with ≥2 flags — word-card (~10) needs one; gen-man (1 flag) does not. Glossary (phase 6) and index-bidirectional (phase 7) fire only if the author introduces marked defined-terms / `\index{}` (word-card's RS-ECC/RAID/parity vocabulary is a candidate). All are union-gated by `make lint`, so the gate backstops completeness — but list "outline block + glossary/index entries for any new terms" so word-card is sized honestly.

- **N4 — m4 precision: drop/qualify the "schema_mirror" co-gate citation for path/required/default.** Per CLAUDE.md, `schema_mirror` gates clap flag-NAMES + dropdown-value enums, NOT `FlagKind` (path vs text) / required / default. The non-observable columns are gated against drift by `verify-examples-gui`'s byte-diff + the shared schema/`default_flag_value_for_flag` source — say that instead of "+ schema_mirror" (which is accurate only for the name/dropdown aspects). No content is left ungated; this is wording.

- **N5 — version-site #3 is created in P5, not P4.** P4 lists three version-sites but the `gui-render` install line is authored in P5's `verify-examples-gui` step. Reword: P4 bumps sites #1 (`pinned-upstream.toml`) + #2 (the `verify-examples` job's hardcoded CLI tags); P5 creates site #3 (the `gui-render` install) pinned to the same GUI tag. Same tag value throughout the leg — purely a phrasing alignment.

---

## CONFIRMED FOLDED (spot-checked; do NOT re-litigate)

- **All 7 round-1 minors present in the plan:** m1 `[[bin]] for main with required-features=["gui"]` (line 10); m2 `[[bin]] name="gui-render"` (line 15); m3 "the 3 lockstep version-sites" (line 29); m4 "TREE-OBSERVABLE projection ONLY" (lines 19–20); m5 `default_flag_value_for_flag AND default_flag_value_for` (line 11); m6 "the PR-#24 harness consumers (`tests/ui_harness/*`)" (line 11); m7 "Stable byte encoding (LF, UTF-8→ASCII-only…) (m7)" (line 16).
- **I1 → P4** is a distinct, correctly-ordered phase (line 27) with the pin bump + 5-subcommand authoring + inventory regen + 3-site lockstep; user-accepted scope.
- **I2 → `render_fixture`** is one egui-free `src` fn, canonical `FormState::default()`, consumed by both P2 emit and P3 render (lines 12, 16, 20).
- **Phase/gating + review discipline** unchanged from round-1's CONFIRMED-SOUND set: tests-first per phase, two post-impl whole-diff reviews (one per leg), cross-repo hard-dep stated, `verify-examples-gui` SEPARATE from the symlinked `verify-examples.sh`, `include-transcript.lua` content-agnostic reuse. GUI PR+CI-before-tag ritual honored.

---

## Gate

**0 Critical / 0 Important → GREEN. Converged.** The two folds (I1 → P4 catch-up phase; I2 → shared `render_fixture`) are sound and introduced no Critical/Important drift; all 7 minors are incorporated. The 5 Minors above are recommended polish — N1 (add `make verify-examples` against the bumped CLI tier to P4) is the highest-value and worth folding before P4 executes, but it is self-correcting at CI and does not hold the gate. **Implementation may begin** (Leg 1 P1 first).
