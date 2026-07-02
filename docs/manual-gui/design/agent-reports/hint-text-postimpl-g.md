# Post-implementation whole-diff review — hint-text-defaults, Phase G (mnemonic-gui)

- **Reviewer:** opus-tier architect, adversarial post-impl gate (0C/0I required before merge + tag).
- **Date:** 2026-07-01.
- **Under review:** `mnemonic-gui` branch `fix/hint-text-defaults`, single commit `a0edc1cd` off master `7e9dcca7`; PR #28 (OPEN). Diff: 14 files, +481/−42, 6 binary PNGs.
- **Authority:** `mnemonic-toolkit/docs/manual-gui/design/SPEC_gui_hint_text_defaults.md` + `hint-text-spec-r0-round-1.md` (RED 0C/1I/4m) → `hint-text-spec-r0-round-2.md` (**GREEN 0C/0I** — "Cleared for build"). R0 ×2 confirmed converged.
- **Verification basis:** full source read at `a0edc1cd` (not the implementer's report), full local suite run, live CI check-rollup, branch-protection API, PNG inspection, independent schema census, and a red-proof of the new tests against master `7e9dcca7`.

## Verdict

**GREEN — 0 Critical / 0 Important / 3 Minor (none blocking). Merge + tag `mnemonic-gui-v0.55.0` cleared.**

- Full suite at `a0edc1cd`: **73 test binaries, 635 tests, 0 failures, EXIT 0** (`cargo test --jobs 2`; log includes `gui_form_snapshots`, `gui_render_emit`, `gui_render_faithfulness`, `schema_mirror`, `persist_redaction_v0_34_0`, `secret_taxonomy_pin`, `ui_harness_i1..i4`, `ui_harness_sweep`, `hint_text_defaults`, all `persistence*`).
- Live CI on PR #28 head `a0edc1cd`: 12/12 completed checks SUCCESS (`release` correctly SKIPPED on a PR). Branch protection API confirms **`snapshots` is the sole hard-required context** — passed (run 28566924975).
- Test-fn delta vs master: **+11 / −0** (641 vs 630).

## 1. Spec conformance §3.1–§3.4 (+ §3 item 5) — VERIFIED

- **§3.1 resolver:** `src/form/flag_defaults.rs::default_flag_value_for_flag` — Text/Path arms now fall through to the kind-only empty defaults (`Text("")`/`Path("")`), merged with the pre-existing Boolean/NodeValueComposite fallback arm (behavior for those two is byte-identical to before). **Dropdown arm intact** (`FlagValue::Dropdown(default_str)`); **Number/Range/Timestamp/TaggedOrIndexed arms intact** (`FlagValue::Unset`). Read the whole function at `a0edc1cd` — the ONLY behavioral change is Text/Path-with-default. One-resolver atomicity holds: widget seed (`widget.rs:223`), repeating-row seed (`:314`), Unset-shape recovery (`:674`), emit `seeded_fixture` + value column (`render_emit.rs`) all route through this single fn; none re-implements the mapping.
- **§3.2 ghost:** `widget.rs` `render_row` Text arm and Path arm each render `ui.add(egui::TextEdit::singleline(..).hint_text(d))` **only when `flag.default_value` is `Some(d)`**; the `None` branch is the pre-existing `ui.text_edit_singleline` — no-default fields byte-unchanged. Path `stdio` button still writes a literal `-` (visible text, argv-suppressed by `is_at_default` — unchanged).
- **§3 item 5 sentinel:** `render_emit.rs::flag_value_str` — empty Text/Path with a default renders `format!("<hint:{d}>")`; non-empty renders the string; no-default empty renders `<empty>`. Dropdown split into its own arm with the exact pre-existing behavior. Grammar sibling of `<empty>`/`<unset>`/`<masked>`/`<pinned: …>` as specified; all live payloads ASCII (`1.0`, `all`, `0`, `-`). Both ASCII pins in place: export-wallet `-> -` → `-> <hint:->` (the only pre-existing pin containing a defaulted Text/Path flag — all other pins unchanged and green) + NEW compare-cost pin `--feerate text -> <hint:1.0>` (`exact_render_defaulted_text_flag_ghosts_hint`).
- **§3.4 normalization — all four mechanics traced in `persistence.rs::normalize_loaded_form_values`:**
  - (a) per-subcommand lookup keyed by the persisted `"<cli>:<sub>"` key — matches the writer `main.rs::form_key` (`format!("{}:{}", tab.bin_name(), sub)`); end-to-end validated by real `save→load` migration tests.
  - (b) **FAIL-OPEN traced at every miss:** malformed key (no `:`) → `continue`; `CliTab::from_bin_name` miss → `continue`; unknown sub → `continue`; unknown flag → `retain` keep; no `default_value` → keep; kind/value shape mismatch → `_ => true` keep. Pinned by `migration_fails_open_on_unknown_subcommand_and_flag` (passes).
  - (c) **kind-scoped:** match arms are exactly `(FlagKind::Text, FlagValue::Text)` and `(FlagKind::Path{..}, FlagValue::Path)`. Bundle's `--account` `Number(0)` hand-seed: `src/main.rs` untouched by the diff (verified `git diff … -- src/main.rs` empty) AND `migration_leaves_number_kind_entries_untouched` round-trips it verbatim.
  - (d) empty-survives: `s != default_str` — `""` never equals a non-empty default; `migration_keeps_empty_path_value` pins it.
  - Placement: load-time only, AFTER JSON-parse + schema-version checks, in-memory mutation only — `load()` gains **no write path** (the `.json.bak` rename on corrupt/mismatch is pre-existing).

## 2. argv invariant — ZERO delta, verified three ways

1. **Code:** `src/form/invocation.rs` has an **empty diff** vs master. `is_at_default` Text arm `s.is_empty() || s == default_str` → an EMPTY Text buffer was suppressed pre-fix and post-fix; empty Path is suppressed by `emit_one`'s `p.is_empty()` early-return (Path `is_at_default` is `p == default_str`, false for `""` — the emit guard covers it). Pre-fix seeded `"1.0"`/`"-"` → suppressed by `is_at_default`; post-fix `""` → suppressed by the empty guards. Same net argv: nothing.
2. **Census (independent, parsed from the 4 schema files at `a0edc1cd`):** exactly **6** Text/Path flags carry a `default_value` — `compare-cost --feerate` (`1.0`), `import-wallet --select-descriptor` (`all`), `nostr --timestamp` (`0`), `ms derive --account` (`0`), `export-wallet --output` (`-`), `restore --output` (`-`) — **all `required: false`, all `repeating: false`**, so no required/repeating emission path can change either. The resolver change touches no other flag. Emission changed for **zero** flags.
3. **Tests:** `defaulted_text_path_flags_never_prefill_and_never_emit_untouched` sweeps every defaulted Text/Path flag across all 61 subs (empty first-render buffer + argv omits); `typing_the_literal_default_is_suppressed_like_untouched` pins that typing the literal default still suppresses. Consumer check: zero hits for `--feerate`/`--select-descriptor` in `conditional.rs`/`main.rs`; zero `"--output"` readers in `src/` outside schema + the new doc comment — spec §2.5's zero-consumers claim re-verified at `a0edc1cd`.

## 3. Corpus re-pin — exactly 6, spec-exact

`git diff --stat 7e9dcca7..a0edc1cd -- tests/snapshots/` → exactly 6 PNGs: `mnemonic-compare-cost`, `mnemonic-export-wallet`, `mnemonic-import-wallet`, `mnemonic-nostr`, `mnemonic-restore`, `ms-derive` — a 1:1 match with the §2.3 blast radius; **no other PNG byte-moved**. Spot-opened two:
- `mnemonic-compare-cost.png`: `--feerate` field EMPTY with a **dimmed `1.0` ghost** (visibly grayer than the label text; `--miniscript`/`--descriptor` fields plain-empty for contrast).
- `mnemonic-export-wallet.png`: `--output` field EMPTY with a **dimmed `-` ghost**, stdio button intact.
The `snapshots` CI job (the required check, dify threshold 0.6) arbitrated the re-pin green. **No manual files in this diff** — all 14 files are `mnemonic-gui`-side; the 61 `.gui` transcripts + `figures/gui/` copies are Phase M work (correctly absent here).

## 4. Behavior preservation elsewhere

- `tests/gui_render_faithfulness.rs`, `tests/gui_form_snapshots.rs`, `src/form/conditional.rs`, `src/schema/*`, `src/main.rs`, `src/form/invocation.rs`, `tests/ui_harness/mod.rs`: **empty diffs** — untouched, all green in the suite run.
- `schema_mirror` untouched and green (no flag name/kind/enum change — paired-schema-PR rule not triggered).
- Sweep workaround removal is clean: `prepared_eligible_base` dropped its `kind: IdentityKind` param + the `Text("")`/`Path("")` re-seed; both call sites (`i1_cell`, `i1_leaf_value_proptest`) updated; the unused `FlagValue` import removed; clippy `-D warnings` green on CI. The stripped flag now seeds through the REAL production path — every Text/Path I1 round-trip is a permanent append tripwire, exactly as claimed.
- Suite delta +11/−0 test fns; 0 removed anywhere in the diff.

## 5. Red-proof (adversarial TDD verification against master `7e9dcca7`)

Injected `tests/hint_text_defaults.rs` verbatim into a master worktree (all helper APIs — `render_flag_harness`, `flag_of`, `sub_of`, `schema_for`, `FormState::from_pairs` — pre-exist on master; `ui_harness/mod.rs` is untouched by the PR): **6 of 10 FAIL on master, 4 pass.**
- FAIL (the behavior-bearing six): append regression — master argv is literally `["mnemonic","compare-cost","--feerate","1.05"]` (the exact papercut); literal-default-typed (pre-fill made it `1.01.0` → emitted); the 61-sub sweep (pre-filled buffers); the AccessKit `value==""` anchor (master value `"1.0"`); both migration-DROP vectors.
- PASS-on-master (by design): the 4 survival/scope-guard vectors (`keeps_non_default`, `keeps_empty`, `fails_open`, `number_kind_untouched`) assert preservation — master preserves trivially; they exist to fence the NEW code's scope, and they'd catch an overreaching migration. Not tautologies.
Worktree removed after the run; both repos left clean.

## 6. Secret hygiene

- **No new persistence WRITE path:** `save()` is untouched and serializes through `serialize_redacted` (on-disk JSON never contains secret-class entries; I3 + `persist_redaction_v0_34_0` + `secret_taxonomy_pin` all green). The normalization runs on LOAD, mutates in-memory state only, and can only ever **drop** entries.
- Zero secret flags carry a `default_value` (census; enforced always-run by `secret_flags_never_carry_a_default_value`, `tests/gui_form_snapshots.rs:189` — present and green). Secret Text flags are dispatched to the secret widgets before the changed `render_row` arms; `hint_text` payloads are schema literals (`1.0`, `all`, `0`, `-`).
- New test fixtures contain no secret material (paths/text: `/tmp/x`, `all`, `-`, `""`, `Number(0)`).

## 7. FOLLOWUP records + commit hygiene

- `gui-prefilled-default-text-appends-on-type` → **RESOLVED** text is accurate on every claim I checked (resolver/ghost/migration mechanics, 6-flag census, sweep-workaround removal, R0 ×2 → GREEN with real report paths, N5 rejection note).
- New `gui-number-set-affordance-ignores-schema-default` entry: cites verified **line-exact** at `a0edc1cd` — `seeded_value_for` at `src/form/widget.rs:379` with the `Number → *min` arm at `:381` (takes `&FlagKind`, cannot see the default); `--gap-limit` at `src/schema/mnemonic.rs:3133-3143` (`min: 0`, `default_value: Some("20")`); the argv-affecting analysis is correct (`Set` → 0 ≠ 20 → `is_at_default` false → `--gap-limit 0` emits).
- Commit trailer present (`Co-Authored-By` + `Claude-Session`). No fmt churn — every hunk is purposeful; no `src/schema/` or unrelated-file drift; nothing outside the spec's §3/§6/§7/§10 scope.

## 8. Minor findings (non-blocking)

- **m1 — "10 new tests" undercounts by one.** The new file has 10, but `exact_render_defaulted_text_flag_ghosts_hint` in `tests/gui_render_emit.rs` makes the true delta **+11** (641 vs 630 `#[test]` fns). Bookkeeping only; no action needed beyond not propagating "10" into the CHANGELOG as an exact count.
- **m2 — spec §7 wording vs implementation:** the spec says the append test is a "form-level" kittest render of compare-cost; the implementation uses the single-flag `render_flag_harness` (which routes through the production `render_one_flag` → `render_with_dispatch` path, conditional visibility included, with full-form argv assembly). Coverage-equivalent — and single-flag rendering is what makes `get_by_role(Role::TextInput)` unique on a 3-text-input form. Record as an accepted deviation; no code change wanted.
- **m3 — spec header still says `Status: DRAFT — awaiting R0`** despite round-2 GREEN. Toolkit-side file, not part of this PR; flip to R0-GREEN (with the round-2 cite) in the Phase M commit.

## 9. Owed at ship (correctly ABSENT from this PR)

Confirmed none of the following is in the diff (no `Cargo.toml`/`Cargo.lock`/`CHANGELOG.md`/`README` change; version at `a0edc1cd` is 0.54.0):
1. Merge PR #28 (GUI = PR + CI-green before tag; `snapshots` is the required check — green).
2. Release commit on master: `Cargo.toml` 0.54.0 → **0.55.0** + `Cargo.lock` + `CHANGELOG.md` + README self-pin (the `readme_pin_coherence` + `pin_coherence` suites will fail-closed if missed). Protected-master posture: admin bypass, as worked for v0.54.0. NO `cargo fmt` (GUI has no fmt gate; standing rule).
3. Tag `mnemonic-gui-v0.55.0`; **verify the tag-push `snapshots` check-run concludes `success`** — it is Phase M's provenance anchor (`FOLLOWUPS.md::gui-form-snapshot-corpus-manual-consumer`).
4. Phase M (toolkit `docs/manual-gui/`): pin bump to v0.55.0 (4 CLI tags unchanged, note in header comment), regen exactly 6 `.gui` transcripts, byte-copy exactly 6 PNGs into `figures/gui/`, prose sweep (`4c-import-wallet.md:162-164` pre-fill wording + the pre-existing stale `53-encode.md:41-42` spin-box prose), `make verify-examples-gui && make lint && make`; fold m3 there.
5. Phase R for the M leg after it lands.

## Cites

- Diff: `git -C /scratch/code/shibboleth/mnemonic-gui diff 7e9dcca7..a0edc1cd` (14 files, +481/−42).
- Sources read at `a0edc1cd`: `src/form/flag_defaults.rs` (resolver), `src/form/widget.rs` (Text/Path arms; `seeded_value_for:379-381`), `src/form/render_emit.rs::flag_value_str`, `src/persistence.rs::{load,normalize_loaded_form_values,save}`, `src/form/invocation.rs::{is_at_default,emit_one}` (diff-empty), `src/main.rs::form_key`, `tests/hint_text_defaults.rs`, `tests/ui_harness/mod.rs::{render_flag_harness,render_one_flag}`, `tests/ui_harness_sweep.rs`, `tests/gui_render_emit.rs`, `tests/gui_form_snapshots.rs:189`, `src/schema/mnemonic.rs:3133-3143`.
- Suite log: 73 binaries / 635 passed / 0 failed / EXIT 0 (local, `--jobs 2`). Red-proof log: 6 failed / 4 passed on master, incl. the literal `--feerate 1.05` argv.
- CI: PR #28 rollup (12 SUCCESS + release SKIPPED) at head `a0edc1cd`, run 28566924975; branch protection `required_status_checks.contexts == ["snapshots"]`.
- R0 chain: `hint-text-spec-r0-round-1.md` (RED 0C/1I/4m) → `hint-text-spec-r0-round-2.md` (GREEN 0C/0I).
