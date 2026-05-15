# Phase P1.1 (Track M — gui-schema-coverage lint RED) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1` (mnemonic-toolkit) — sibling `manual-gui-help-icons` (mnemonic-gui, read-only this round)
**Scope:** §3.1 P1.1 sub-phase — `tests/extract_gui_schema.py`, `tests/check_gui_schema_coverage.py` (NEW), `tests/expected_gui_schema_inventory.json` (regenerated), `tests/lint.sh` (phase 4 swapped, env-var argv added, appendix paths shifted 60→90).

**Verdict:** **LOCK 0C / 0I / 2N / 1n.**

The RED is failing for the right reason. Every source-grep claim from the prompt verified PASS. The extractor's tuple-struct NodeValueComposite fix is sound and the regenerated inventory's variant total (270) matches my hand-count exactly. The lint produces the documented 459-anchor missing-set on both empty-build and build-banner-only HTML; orphan-direction check yields 0 false positives against the 4 prose anchors emitted by `make html` on an empty `src/`. No Critical / Important survivors.

---

## Critical

None.

## Important

None.

## Nice-to-have

### N-1 — `cli-subcommands.list` is now an unreferenced inheritance from CLI-manual P0 copy

**Where:** `docs/manual-gui/tests/cli-subcommands.list`

The file was copied verbatim from `docs/manual/tests/cli-subcommands.list` during M-P0.2 (`cp -r docs/manual/{pandoc,tests} docs/manual-gui/`). With P1.1's lint phase swap (flag-coverage → gui-schema-coverage), nothing in this repo reads it any longer. Grepping confirms no consumer: `lint.sh` no longer iterates it; the new gui-schema-coverage path reads the GUI repo's schema modules directly. The file also carries content that is wrong-for-this-manual: it lists `mnemonic seed-xor split` / `mnemonic seed-xor combine` (the CLI's two-token form) and `mnemonic gui-schema`, none of which the GUI exposes as subcommands.

Recommendation: delete in P1.1 now while the rationale is fresh. Deferring risks future authors editing it under the impression it's still a source of truth.

### N-2 — AUTHORING.md still references CLI-manual appendix paths (60-/69-) while lint.sh uses GUI paths (90-/99-)

**Where:** `docs/manual-gui/AUTHORING.md` lines 116 (`src/60-appendices/69-index-table.md`) and 168 (`src/60-appendices/61-glossary.md`).

The lint.sh checks `$SRC_DIR/90-appendices/91-glossary.md` and `$SRC_DIR/90-appendices/99-index-table.md`. The README.md was correctly updated to `90-appendices/` in P0, but AUTHORING.md was missed. P2 authors following AUTHORING.md verbatim would create files at the wrong paths and the lint would silently `WARN: missing; skipping`.

This is sub-Important because the lint's missing-file fallback is warn-and-skip, not fail-loudly — but per `[[feedback-r2-blocking-vs-cosmetic-gate]]`, the gap between authoring guidance and lint enforcement is the kind of silent-drift hazard the feedback rule cautions against. Recommend folding into P1.1 with a sed update on AUTHORING.md to bring its two path references in line with lint.sh + README.md + SPEC §1.4.

## Nit

### n-1 — `lint.sh` argv parsing accepts `MNEMONIC_BIN`/`MD_BIN`/`MS_BIN`/`MK_BIN` but never uses them

**Where:** `docs/manual-gui/tests/lint.sh` lines 34-37.

The argv parser sets these four variables, the Makefile passes them at lines 251-254, and the header comment at line 22-24 notes "unused by gui-schema-coverage; reserved for future GUI-side worked-example phases." The "reserved" annotation makes this intentional, so it's truly a nit.

---

## Verification matrix

Twelve claims source-grep verified PASS against `mnemonic-gui/src/schema/{mnemonic,md,ms,mk}.rs` and the inventory JSON.

| # | Claim | Source-of-truth | Result |
|---|-------|-----------------|--------|
| A | 28 SubcommandSchema entries total (10/8/5/5) | `grep -c SubcommandSchema\s*\{\s*name:` across 4 schema files | **PASS** — mnemonic.rs:1009-1090 (10); md.rs:401-465 (8); ms.rs:184-225 (5); mk.rs:227-268 (5). |
| B | 161 FlagSchema entries total | hand-summed per-subcommand counts from inventory | **PASS** — md: 31 ; mk: 19 ; mnemonic: 99 ; ms: 12. Sum = 161. |
| C | 270 enumerated-flag variants total | hand-summed variant-list lengths from inventory | **PASS** — md: 16 ; mk: 0 ; mnemonic: 224 ; ms: 30. Sum = 270. Matches §3.1 RED-expected count (28 + 161 + 270 = 459). |
| D | 6 NodeValueComposite occurrences, total variants = 20 | grep `FlagKind::NodeValueComposite` | **PASS** — mnemonic.rs:409 (NODE_TYPES=13), 685 (`&["xprv","phrase"]`=2), 771 (SLIP39_FROM_NODES=2), 897 (PHRASE_ONLY=1), 942 (PHRASE_ONLY=1), 979 (PHRASE_ONLY=1). Sum 20. |
| E | NodeValueComposite tuple-struct fix lands both shapes | `_classify_kind` lines 167/171 in `extract_gui_schema.py` | **PASS** — line 167 matches `NodeValueComposite(CONST)`; line 171 matches `NodeValueComposite(&[...])`. Both shapes present in source covered. |
| F | 1 TaggedOrIndexed occurrence (mnemonic export-wallet --taproot-internal-key) | grep `FlagKind::TaggedOrIndexed` | **PASS** — mnemonic.rs:662 `FlagKind::TaggedOrIndexed(&["nums"])`. |
| G | 36 Dropdown occurrences total (mnemonic 28 + md 5 + ms 3 + mk 0) | grep `FlagKind::Dropdown` | **PASS** — sum 36 (per-line tally in dispatch reply). |
| H | `_collect_subcommands` regex captures all 28 entries despite intermediate `human_name:` line | trace `[^}]*?flags:` against actual block shape | **PASS** — no `}` appears between `name:` and `flags:` in any block. |
| I | `_collect_flag_array` decl regex handles single-line array decls | trace against `const INSPECT_FLAGS: ... = &[FlagSchema { ... }];` | **PASS** — non-greedy + MULTILINE + `\];\s*$` works for both single-line and multi-line shapes. |
| J | brace-depth-aware splitter handles nested `Path { stdio_sentinel: ... }` / `Number { min, max }` | trace `_split_flagschema_blocks` against 11 Path + 8 Number occurrences | **PASS** — depth-counter handles nesting; inventory entry-count matches source occurrence-count. |
| K | kebab() implementation matches SPEC §2.2 verbatim | `check_gui_schema_coverage.py:58-63` | **PASS** — lowercase → non-alphanumeric→`-` → collapse → strip leading/trailing, all four steps in order. |
| L | anchor-derivation formula in `build_expected` faithful to SPEC §2.2 | `check_gui_schema_coverage.py:66-88` | **PASS** — sub = `tab + "-" + kebab(name)`; flag = `sub + "-" + name.lstrip("-")`; variant = `flag + "-" + kebab(value)`. |

## Empirical reproducibility

The empirical lint output is mechanically reproducible without re-running the script:

- `make html` produces an HTML file with exactly 4 `id="..."` attributes: `title-block-header`, `TOC`, `toc-build-banner`, `build-banner`. None match any of the 28 `<tab>-<sub>` shape prefixes.
- `missing = expected - found = 459 - 0 = 459`.
- `orphans = (found - expected) filtered to is_schema_shaped = 0`.
- Exit code 1.

Without `make html`: HTML file absent, WARN emitted on stderr, `found = set()`, missing=459, orphans=0, exit 1.

## Anti-off-by-N audit

Per `[[feedback-r0-must-read-source-off-by-n]]`, three potential drift sites checked: subcommand 28 (PASS), flag 161 (PASS), variant 270 (PASS). The handoff's stale "50 missing variants" predates the P1.1 NodeValueComposite fix; the current inventory + the P1.1 prompt + my hand-count agree on 270. No off-by-N pattern surfaced.

## Lockstep + RED-correctness summary

P1's parity-gate criterion is "all 5 RED suites in place; each failing for the expected reason; no compile-time hangs" (§3.1 plan). For Track M P1.1 specifically:

- RED in place: yes — `lint.sh` exits 1 on empty + on build-banner-only manual.
- Failing for the expected reason: yes — 459 missing schema anchors, 0 orphans. The failure cleanly distinguishes "schema → HTML" direction (459 missing) from the orthogonal "HTML → schema" direction (0).
- No compile-time / parse-time hangs: confirmed. Regex is anchored + non-greedy; brace-walker is bounded by string length.

---

**Final verdict:** **LOCK 0C / 0I / 2N / 1n.** Track M P1.1 RED is sound and reproducible. Track G P1.4 + P1.5 can advance independently per the parity gate. The two N-findings (cli-subcommands.list orphan; AUTHORING.md path drift) folded into P1.1 commit-set.
