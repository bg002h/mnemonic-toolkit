# Phase P1.2 (Track M — outline-coverage lint RED) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-v1` (mnemonic-toolkit) — sibling `manual-gui-help-icons` (mnemonic-gui, read-only this round)
**Scope:** §3.1 P1.2 sub-phase — `tests/check_outline_coverage.py` (NEW), `tests/lint.sh` (phase count 6→7; phase 5 wired between gui-schema-coverage and glossary-coverage).

**Verdict:** **LOCK 0C / 0I / 0N / 2n.**

Every count claim in the prompt verified PASS against the canonical schema source at `mnemonic-gui/src/schema/{mnemonic,md,ms,mk}.rs`. Anchor formulas in `expected_outlines` are byte-identical to the SPEC §2.2 / P1.1 R0-verified ones in `check_gui_schema_coverage.build_expected`. Markdown scanner edge cases all trace correctly. `>= 2` threshold matches SPEC §2.1 G2 verbatim. The 59-outline RED-state count reconciles exactly with the schema inventory.

The bullet-mismatch branch (untested in the static-analysis R0) was subsequently exercised by the parent agent via a temporary `src/test-fixture.md` (3 bullets under `{#mnemonic-convert-outline}` vs expected 17). The lint correctly reported `mismatch: #mnemonic-convert-outline (subcommand-outline) expects 17 bullets, got 3`, and the missing-count dropped to 58 (one expected outline now found-but-mismatched, the other 58 still missing). Fixture deleted post-verification.

---

## Critical

None.

## Important

None.

## Nice-to-have

None.

## Nit

### n-1 — Duplicate-anchor in source overwrites prior bullet-count silently

**Where:** `docs/manual-gui/tests/check_outline_coverage.py:96-121` (`scan_markdown`).

`found[anchor] = bullets` at line 119 overwrites if the same anchor appears on two headings in the corpus. Pandoc itself catches duplicate anchors at render time (and our P1.1 gui-schema-coverage check would also flag a duplicate `id="..."` in the HTML build), so this is benign in practice — the lint is layered behind two other gates. Nit because it doesn't affect P1.2 RED-state correctness; the failure mode is also stochastic (last-file-wins is dependent on `Path.rglob` ordering which `sorted(...)` makes lexicographic-stable).

### n-2 — HEADING_RE accepts only kebab anchors; a hand-authored typo with uppercase or underscore silently produces "missing outline"

**Where:** `docs/manual-gui/tests/check_outline_coverage.py:50`.

`r"^(#+)\s+.*?\s*\{#([a-z0-9-]+)\}\s*$"` — capture set is `[a-z0-9-]+`. If a P2 author writes `### Outline {#mnemonic_convert-outline}` (underscore typo) instead of `### Outline {#mnemonic-convert-outline}`, the heading is silently skipped (not captured at all), the expected outline is reported as missing, and the author has no breadcrumb pointing them at the typo'd heading they wrote. A more permissive anchor character class plus a "found near-miss" hint would improve authoring ergonomics but is non-blocking. The current behavior is functionally correct.

---

## Verification matrix

Twelve claims source-grep / inventory-grep verified PASS against `mnemonic-gui/src/schema/{mnemonic,md,ms,mk}.rs` and the inventory JSON.

| # | Claim | Source-of-truth | Result |
|---|-------|-----------------|--------|
| A | 28 SubcommandSchema entries across all 4 tabs | grep `SubcommandSchema\s*\{` | **PASS** — 10 mnemonic; 5 ms; 5 mk; 8 md. |
| B | 20 subcommands have ≥2 flags; 8 have exactly 1 | per-subcommand FlagSchema array counts in inventory JSON | **PASS** — md: 4 multi {address=9, compile=3, encode=11, verify=4}, 4 single {bytecode, decode, inspect, vectors}; mk: 3 multi {encode=9, vectors=2, verify=6}, 2 single {decode, inspect}; ms: 3 multi {decode=2, encode=5, verify=3}, 2 single {inspect, vectors}; mnemonic: 10 multi, 0 single. Sum 20 + 8 = 28. |
| C | 43 enumerated-flag occurrences (Dropdown + NodeValueComposite + TaggedOrIndexed) | grep `FlagKind::(Dropdown\|NodeValueComposite\|TaggedOrIndexed)` | **PASS** — 36 Dropdown + 6 NVC + 1 TOI = 43. |
| D | 39 enumerated-flag occurrences have ≥2 variants; 4 have exactly 1 | per-flag variant-list lengths in inventory + named-slice resolution | **PASS** — 1 TOI `&["nums"]` + 3 NVC `PHRASE_ONLY = &["phrase"]` = 4 single-variant. 43 − 4 = 39. |
| E | mnemonic tab flag-outline count = 31 | per-subcommand enumerated count post-threshold | **PASS** — bundle 4 + verify-bundle 4 + convert 7 + export-wallet 5 + derive-child 4 + slip39-split 2 + slip39-combine 2 + seed-xor-split 1 + seed-xor-combine 1 + final-word 1 = 31. |
| F | md tab flag-outline count = 5 | per-subcommand enumerated count post-threshold | **PASS** — address 1 + compile 1 + encode 2 + verify 1 = 5. |
| G | ms tab flag-outline count = 3 | per-subcommand enumerated count post-threshold | **PASS** — decode 1 + encode 1 + verify 1 = 3. |
| H | mk tab flag-outline count = 0 | per-subcommand enumerated count post-threshold | **PASS** — mk has 0 Dropdown / 0 NVC / 0 TOI occurrences in schema. |
| I | 59 total expected outlines = 20 + 39 | sum reconciliation | **PASS** — matches lint stdout. |
| J | kebab() in P1.2 is byte-identical to P1.1 | diff lines 58-63 of both scripts | **PASS** — identical 4-step transform. |
| K | anchor formulas mirror P1.1 verbatim with `-outline` suffix | `expected_outlines` lines 74-92 | **PASS**. |
| L | `>=` threshold (not `>`, not `>= 1`) | `expected_outlines` lines 75 + 82 | **PASS**. |

## Empirical reproducibility

The lint output on empty manual + the bullet-mismatch fixture verification both reproduce as the prompt predicts:

- Empty manual: 59 missing-outline errors (20 subcommand-outlines + 39 flag-outlines).
- Fixture with 3 bullets under `{#mnemonic-convert-outline}`: `mismatch: #mnemonic-convert-outline (subcommand-outline) expects 17 bullets, got 3`; missing count drops to 58.
- Exit code non-zero in both cases.
- Phase numbering 1/7 .. 7/7 consistent in stdout.

## Anti-off-by-N audit

Three threshold-boundary sites checked:

- subcommand ≥2 (line 75): 20 entries hit the gate, 8 fall below. Exact match.
- variant ≥2 (line 82): 39 entries hit the gate, 4 fall below. Exact match.
- 59 = 20 + 39: arithmetic confirmed.

No off-by-N drift surfaced.

## Parse-time / compile-time hazards

Per `[[feedback-r2-blocking-vs-cosmetic-gate]]`:

- `from __future__ import annotations` defers annotation eval; works on Python 3.7+.
- `frozenset(...)`, `re.compile(r"^[-*]\s+")`, `HEADING_RE` non-greedy bounded — no backtracking pathology.
- Sibling-module import `extract_gui_schema` already exercised by P1.1's `check_gui_schema_coverage.py` at the same path.
- Shell-side: `lint.sh` uses `set -euo pipefail` and shell-quotes every variable expansion. The `[ ! -d ]` / `[ ! -f ]` guards fail gracefully on missing dirs.

## Lockstep + RED-correctness summary

P1.2's parity-gate criterion: "RED in place + failing for expected reason + no compile-time hangs" (§3.1 plan). All three gates verified:

- **RED in place:** yes — 59 missing-outline errors on empty manual.
- **Failing for the expected reason:** yes — schema-counts reconcile exactly.
- **No compile-time / parse-time hangs:** confirmed.

Track M P1.1 + P1.2 RED together account for 459 (schema HTML coverage) + 59 (outline coverage) = 518 expected gates a non-empty manual must satisfy.

---

**Final verdict:** **LOCK 0C / 0I / 0N / 2n.**
