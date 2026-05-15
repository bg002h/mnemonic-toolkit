# Phase P2.4 sub-batch 5a (Track M — 40-mnemonic chapter overview + final-word) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1` (mnemonic-toolkit)
**Scope:** §3.2 P2.4 sub-batch 5a (first schema-driven sub-batch of the user-chosen 5-sub-batch split for batch 5):
- `docs/manual-gui/src/40-mnemonic/41-overview.md` (NEW, ~95 LOC) — chapter overview; 10-subcommand index grouped by 5 families.
- `docs/manual-gui/src/40-mnemonic/47-final-word.md` (NEW, ~225 LOC) — `mnemonic final-word` chapter; SPEC §2.3 chapter shape (H1 + subcommand outline + 3 per-flag sections + 10 per-language-variant sections + worked example + refusals + advisories).
- `docs/manual-gui/tests/check_gui_schema_coverage.py` (PATCH, +10/-2) — exempts `-outline`-suffixed anchors from the orphan check so phase-4 schema-coverage and phase-5 outline-coverage do not contradict each other.

**Verdict:** **ITERATE 0C / 1I / 0N / 1n.**

The chapter-overview prose is well-organized; the final-word chapter is comprehensive (10/10 source-faithful items in the verification matrix); the lint-tool patch is sound and minimal. **One Important finding** in the chapter overview: the "five secret-bearing, five public-input subcommands" numerical claim is wrong against the actual `should_confirm_run` predicate at `mnemonic-gui/src/secrets.rs:80-105` — 9 or 10 of the 10 subcommands actually trigger the modal under realistic form input. **One Nitpick** in final-word: the refusal table row 4 is byte-faithful to the CLI manual baseline but technically incomplete relative to the actual CLI surface (two distinct refusals fire for non-phrase node types depending on whether the node parses; the manual collapses to one). Acceptable as fold; CLI-manual-followup worth filing.

---

## Important

### I-1 — "Five secret-bearing, five public-input" claim is source-unfaithful (41-overview.md:71-74)

**Where:** `docs/manual-gui/src/40-mnemonic/41-overview.md` lines 71-74:

> "Five of the ten subcommands consume secret-bearing inputs (the master `ms1`, a `--passphrase`, etc.) and therefore trigger the run-confirm modal at click-Run time; the other five are entirely public-input subcommands and **Run** fires the subprocess immediately."

**Why source contradicts:** Per `mnemonic-gui/src/secrets.rs:80-105` (`should_confirm_run`), the modal fires when ANY of:
- (a) a flag with `secret: true` has a non-empty value
- (b) a slot row's subkey is in `SECRET_SLOT_SUBKEYS`
- (c) a NodeValueComposite's node value is in `node_type_is_secret(node)`

Counting against the schema:

| Subcommand | Modal-trigger reason |
|---|---|
| bundle | `--passphrase` (secret:true) + slot rows |
| verify-bundle | `--passphrase` + slot rows |
| convert | `--passphrase`, `--bip38-passphrase` + `--from phrase/entropy/...` |
| export-wallet | slot rows can be secret-bearing (allows_slots: true) |
| derive-child | `--passphrase` + `--from phrase=` |
| final-word | `--from phrase=` (NodeValueComposite phrase) |
| seed-xor-split | `--from phrase=` |
| seed-xor-combine | `--share` (secret:true) + composite phrase |
| slip39-split | `--passphrase` + `--from phrase/entropy=` |
| slip39-combine | `--passphrase` + `--share` (secret:true) |

That is **9-10**, not 5. Even narrowing to "has at least one `secret:true` flag" gives 7 (bundle, verify-bundle, convert, derive-child, seed-xor-combine, slip39-split, slip39-combine).

**Fix:** rewrite as a non-numeric framing — e.g., "Most of the ten subcommands consume secret-bearing inputs and therefore trigger the run-confirm modal; only `export-wallet` (intended as watch-only) consistently fires Run without the modal when slot rows hold public xpubs". OR replace the precise number with a per-subcommand reference back to each chapter's "modal trigger" note.

---

## Nitpicks

### n-1 — Refusal table row 4 is byte-faithful to CLI manual but technically incomplete (47-final-word.md:213)

The `--from` variant other than `phrase=` row collapses two distinct CLI refusals:
- `parse_from_input` refusal for unknown node tokens (e.g. `bogus=value` → "unknown --from node `bogus`; expected one of: phrase, entropy, ...")
- `final_word.rs:56-61` refusal for known-but-wrong node tokens (e.g. `entropy=value` → "final-word --from only accepts phrase=<value> or phrase=-; got entropy=")

The manual's prose matches the CLI manual baseline at `docs/manual/src/40-cli-reference/41-mnemonic.md:283`. Acceptable as fold; worth filing a CLI-manual-followup to correct both manuals in lockstep.

---

## Notes (PASS)

- **All 10 subcommand names, family categorization, and cross-references in 41-overview.md** are byte-correct against `schema/mnemonic.rs:1009-1090`.
- **Pinned format** `Pinned: mnemonic 0.13.0` matches runtime banner at `schema/mnemonic.rs:1095-1099`.
- **Final-word subcommand anchor + 3-bullet outline** match `len(FINAL_WORD_FLAGS) = 3`.
- **--from PHRASE_ONLY claim** matches `schema/mnemonic.rs:892`.
- **--language 10-variant outline** matches `LANGUAGES` const at `schema/mnemonic.rs:38-49`; spelling correct (`simplifiedchinese` no hyphen); cross-tab divergence with `ms` (`chinese-simplified`) noted via forward-reference to `[ms encode --lang]` (Direction-A miss when ms-tab schema lands; not a sub-batch-5a issue).
- **--json-out 6-field envelope schema** byte-faithful to `final_word.rs:148-156` `FinalWordJson` struct field order.
- **Refusals table rows 1-3 + 4-byte-faithful** match `final_word.rs::map_final_word_error` and CLI manual.
- **Advisories table** matches CLI manual rows 1-3.
- **GUI subprocess piping claim** (`runner.rs` Stdio::piped() suppresses TTY-vs-pipe advisory) verified at `mnemonic-gui/src/runner.rs:58`.
- **Modal-trigger claim** for `--from phrase=` (NodeValueComposite phrase node) verified by `should_confirm_run` clause (c) at `secrets.rs:96-103`.
- **`:::danger` admonition** is appropriate severity, source-faithful, and explicitly says the candidate list is itself secret-class.

---

## Lint-tool patch verification

The `check_gui_schema_coverage.py` patch correctly exempts `-outline`-suffixed anchors via `if anchor.endswith("-outline"): return False` early-return in `is_schema_shaped`. The exemption is necessary because:
- `expected_outlines` in `check_outline_coverage.py` derives `<sub>-outline` and `<flag>-outline` anchor names (lines 74-92).
- `build_expected` in `check_gui_schema_coverage.py` derives only schema-shaped (sub/flag/variant) anchors — outline anchors are not in the expected set.
- Without exemption, every batch with a subcommand-with-≥2-flags or enumerated-flag-with-≥2-variants would trigger 1+1 false orphans.

Side-effect risk (do-not-block, file as FOLLOWUP): if a future sibling-codec ever names a real schema flag literally `--outline` (kebab → `outline`), its anchor `<tab>-<sub>-outline` would silently become unchecked. No current-cycle concern.

---

## Lint state

- Phases 1-3 GREEN.
- Phase 4 schema-coverage RED at **445 missing** (was 459 baseline → -14 = 1 sub + 3 flags + 10 variants for final-word). No orphans.
- Phase 5 outline-coverage RED at **57 missing** (was 59 baseline → -2 = 1 subcommand-outline + 1 flag-outline for --language).
- Phases 6-7 WARN-skip (90-appendices arrives batch 10).
- HTML 16 H1 chapters (was 14 → +2 for overview + final-word).
- PDF 50 pages (was 42 → +8 for the two new chapters).

After folding I-1, R1 should LOCK cleanly.
