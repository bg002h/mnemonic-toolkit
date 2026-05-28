# SPEC — manual-prose-execution gate (full sweep)

- **Date:** 2026-05-27
- **Source SHA:** `badb619` (toolkit `master`)
- **Status:** approved design (brainstorm + opus architect blueprint); **pending R0 to 0C/0I before any implementation.**
- **Type:** docs/test-only (no clap surface, no GUI lockstep, no Rust code, zero binary change). **Commit-to-master, no version bump** (R0 C1 fold: v0.34.3 was actually a tagged PATCH — wrong precedent. Correct precedent is **`9294723`** (v0.37.6 hygiene commit: --locked CI guard + test-fixture fix) — pure CI/test/docs with zero binary change, shipped as commit-to-master no-bump.) Resolves FOLLOWUP `manual-prose-command-execution-gate`.

## Purpose
Close the manual-prose-execution gap surfaced by the v0.28.1 and v0.36.4 documentation audits: the manual lint validates flag names/spelling/links/glossary/index but never **executes** the documented recipes, so docs can silently rot (and have).

**Key reframing from the brainstorm exploration:** the harness is already built and CI-wired. `docs/manual/tests/verify-examples.sh` runs `.cmd`/`.out`/`.err` triples through `bash` with binary/fixture substitutions + per-recipe `mktemp -d` isolation + byte-exact diff. `Makefile audit` = lint + verify-examples; `manual.yml` runs it on every push/PR touching the manual; **sibling binaries (md/ms/mk) are already installed in CI via install.sh-pinned tags** (v0.36.4 fix, `manual.yml:72-88` ↔ `install.sh:35,38,41`). 14 transcripts pass today: 6 top-level (quickstart/inheritance) + 8 in `cross-format-recipes/`. The actual gap is **transcript coverage** of chapter-45's per-format round-trip examples (`transcripts/foreign-formats/` exists but is empty), plus two small defense-in-depth wins.

## Scope (three pieces)

### Piece 1 — 6 chapter-45 transcripts in `transcripts/foreign-formats/`
Chapter `45-foreign-formats.md` has exactly 6 `### Round-trip example` subsections (one per foreign format). v0.37.0's `export-wallet-from-import-json-template-format-reemit` fix unblocked the previously-impossible recipes. Author all 6:

| # | Stem | Source line | Fixture |
|---|---|---|---|
| 1 | `roundtrip-sparrow-singlesig` | `45-foreign-formats.md:305-320` | `sparrow-singlesig-p2wpkh.json` |
| 2 | `roundtrip-specter-singlesig` | `:404-411` | `specter-singlesig-p2wpkh.json` |
| 3 | `roundtrip-coldcard-singlesig` | `:480-487` | `coldcard-singlesig-bip84-mainnet.json` |
| 4 | `roundtrip-coldcard-multisig` | `:563-572` | `coldcard-ms-2of3-p2wsh-with-xfp.txt` |
| 5 | `roundtrip-jade-multisig` | `:640-647` | `jade-multisig-2of3-p2wsh.json` |
| 6 | `roundtrip-electrum-singlesig` | `:752-759` | `electrum-standard-bip84-mainnet.json` |

**Format = triple** (`.cmd`+`.out`+`.err`), even if `.err` is empty (a presence sentinel — `verify-examples.sh:85` only checks file presence; content may be empty; M2 fold), so future stderr drift surfaces. Substitution-aware (`$FIXTURES_DIR`, `$MNEMONIC_BIN`); no bitcoin-core wrap needed (chapter-45 doesn't import bitcoin-core in any of these recipes).

**Capture, never author.** Every `.out`/`.err` = the live output of running the `.cmd` against `target/debug/mnemonic` at cycle tip. Authoring expected output by hand has zero upside and unbounded downside (typos masquerade as drift). **Transcripts mirror chapter-45 prose recipes LITERALLY** (R0 I1 fold) — do not add commands the docs don't have:
- **Cells #1 (sparrow) + #4 (coldcard-MS)** end in `diff`/`diff <(jq -S ...) <(jq -S ...)` per chapter prose → `.out` captures the live diff output (empty for clean round-trip; non-empty for documented dropped fields — both pinned as expected).
- **Cells #2/#3/#5/#6 (specter/coldcard-SS/jade/electrum)** end at `export-wallet > X_re.<ext>` per chapter prose (no terminal `diff` or `cat`) → both commands redirect to files → `.out` is empty → gate asserts both commands exit 0 (catches v0.36.4-class "recipe is impossible" regressions but not content drift). Enhancing the chapter prose to add comparators for #2/#3/#5/#6 is a separate FOLLOWUP, NOT in this cycle.

**Cells with expected non-empty `.out` diffs (documented canonicalize behavior, NOT bugs):**
- **Sparrow (#1)**: round-trips clean (`name` lifted to `label` and preserved verbatim).
- **Coldcard-MS (#4)**: text-format reordering — fixture has comment lines + `Derivation:` BEFORE `Format:`; re-emit normalizes.

Lockstep prose addenda (R0 I2 fold — placement corrected; original `:317`/`:411` landed INSIDE the code fences): add one clarifying sentence AFTER the sparrow close-fence (`45-foreign-formats.md:321`, currently blank) noting that documented dropped fields produce a non-empty but expected diff; and AFTER the coldcard-MS close-fence (`:572` if blank; place between close-fence and next H3) noting text-format reordering. (Specter `:412` addendum dropped — cell #2 has no diff per I1.)

### Piece 2 — Harness sentinel hardening (`verify-examples.sh`)
The harness's `=true` silent-default fallback (`verify-examples.sh:46-49`) is a "vacuous pass" hatch: if a future CI config drops `MD_BIN=md` (or someone runs `make audit` without env), md/ms/mk-using transcripts silently `=true` → exit 0. Harden:
```diff
- : "${MNEMONIC_BIN:=true}"
- : "${MD_BIN:=true}"
- : "${MS_BIN:=true}"
- : "${MK_BIN:=true}"
+ : "${MNEMONIC_BIN:?MNEMONIC_BIN is required (path to mnemonic binary)}"
+ : "${MD_BIN:?MD_BIN is required (path to md binary)}"
+ : "${MS_BIN:?MS_BIN is required (path to ms binary)}"
+ : "${MK_BIN:?MK_BIN is required (path to mk binary)}"
```
Makefile defaults populate all four env vars (`Makefile:42-45` — M1 fold; all 4 binaries are `?=` defaulted, not just 3); `make audit` still works. Direct script invocation now errors with a clear message instead of vacuously passing.

Plus file new FOLLOWUP `manual-yml-sibling-pin-vs-install-sh-drift-gate` for a static install.sh↔manual.yml sibling-pin gate (defense-in-depth for the closed `manual-yml-and-install-sh-sibling-gui-pin-staleness` symptom; not fixed this cycle).

### Piece 3 — lychee `--include-fragments` (**DEFERRED this cycle per pre-flight; FOLLOWUP filed**)
**Cycle update (2026-05-27):** Phase 1 mandatory pre-flight (per R0 I4) revealed Piece 3 needs deeper rework than originally scoped. Lychee per-file `--include-fragments` against `src/` reports **97 errors / 127 references** — mostly *false positives* from pandoc cross-file concat (lychee per-file mode doesn't know the manual is a single concatenated document at render time; e.g. `45-foreign-formats.md#mnemonic-import-wallet` references an anchor in `41-mnemonic.md` that resolves at concat-time). Re-running against the **build output** `build/m-format-manual.md` (`make md` first) yields **174 errors / 603 references** — now real danglers (URL-encoded spaces, missing `worked-example-*` definitions, slug-rule mismatches) but well beyond a "trivial inline fix" or a reasonable `--exclude` list. Per R0 I4's "broader backlog → FOLLOWUP" guidance, deferred to its own cycle with empirical evidence and build-output approach pre-designed. See FOLLOWUP `manual-anchor-dangler-backlog-cleanup`. The pre-existing `lint.sh:57` (lychee `--offline` only) is unchanged this cycle.

Original one-line plan (preserved for reference): `lychee --offline --no-progress` → `lychee --offline --include-fragments --no-progress` (`lint.sh:57`). Lychee v0.24.2 (pinned in `manual.yml:68` + `Dockerfile.build:55`) supports `--include-fragments` (added v0.24.0 — R0 I3 fold: the option is enum-valued `{none|anchor-only|text-only|full}` at v0.24+; bare `--include-fragments` flag defaults to `anchor-only`, exactly what we want — validates `[label](#anchor)` against local `{#id}` ids; text-fragments are NOT validated, which is correct intent). Works in `--offline` mode (lychee parses anchor IDs from local `.md` files; no network).

**Phase 1 step ordering — pre-flight is MANDATORY, not optional** (R0 I4 fold). 124 `](#anchor)` references across 22 manual files; pandoc auto-anchor heuristics may not match lychee's parse; probability all 124 resolve clean is low. Phase 1 sequence:
1. **Trial-run locally FIRST** (before any edit): `cd docs/manual && lychee --offline --include-fragments --no-progress src` and enumerate any reported danglers.
2. Triage: ≤2 trivial typos → fix inline this cycle. Broader backlog → file FOLLOWUP `manual-anchor-dangler-backlog-cleanup` with `path:line` citations + add temporary `--exclude '#<dangler>'` so the gate ships GREEN.
3. THEN edit `lint.sh:57`.

## Completeness & non-redundancy
- **Completeness:** `grep -c '^### Round-trip example' 45-foreign-formats.md` = 6; matrix covers all 6 exactly once. BSMS Round-2 has no `### Round-trip example` (descriptor-only round-trip, prose `:138-163`, correctly excluded).
- **Non-redundancy:** the 8 existing `cross-format-recipes/` transcripts test A→B *conversion* correctness (envelope as bridge); the 6 new transcripts test A→A *canonicalize fidelity* (the load-bearing claim of `:781-808`). Orthogonal: A→B can succeed while A→A drifts. Plus Coldcard-multisig (#4) has no cross-format-recipes analogue.

## Risk flags
1. Specter/Coldcard-SS/Coldcard-MS recipes produce non-empty diffs by design (see Piece 1); capture pins them as expected, prose addendum clarifies.
2. Taproot already walled off `--from-import-json` (`:356-361`); none of the 6 fixtures is taproot.
3. CI complications: none expected (sibling binaries already installed; lychee flag is a token addition; transcripts use only `mnemonic`).
4. **Test-only-cycle-becomes-product-fix escape hatch:** per the v0.37.4 lesson, if recipe capture reveals an unexpected diff that's a real defect (not documented behavior), promote to a code-fix PATCH.

## Mechanism
Canonical capture, no authoring. The capture procedure runs each `.cmd` body through `bash` with env substitutions to a fresh `mktemp -d`, redirecting stdout/stderr to candidate `.out`/`.err` files for review, then `mv` into place. Verification = `make -C docs/manual audit MNEMONIC_BIN=… MD_BIN=md MS_BIN=ms MK_BIN=mk FIXTURES_DIR=…` reports 14 transcripts pass.

## Build sequence
- **Phase 0** — R0 architect review of this spec to 0C/0I (persist verbatim). DONE: R0 RED 1C/4I/3M → R1 GREEN; both persisted.
- **Phase 1** — Piece 3 pre-flight + deferral decision. DONE: 174 build-output danglers → FOLLOWUP filed.
- **Phase 2** — Piece 1 (6 transcripts): author `.cmd` files, capture `.out`/`.err`, prose addendum, `make audit` green; commit.
- **Phase 3** — Piece 2 (harness hardening + new FOLLOWUP `manual-yml-sibling-pin-vs-install-sh-drift-gate`); commit.
- **Phase 4** — FOLLOWUP flip (resolve `manual-prose-command-execution-gate`) + commit-to-master, **no version bump** (correct precedent: `9294723` v0.37.6 hygiene commit — pure CI/test/docs with zero binary change. R0 C1 fold).

No GUI lockstep (no clap surface change). No Cargo.lock churn (no Rust code). The new `--locked` CI guard (v0.37.6 hygiene) is irrelevant — no version bump.

## R0 history
R0 (`design/agent-reports/manual-prose-gate-R0-review.md`): RED 1C/4I/3M → folded. C1 (precedent corrected to `9294723`), I1 (cells mirror prose literally — #2/#3/#5/#6 exit-0 gate, no diff; only #1+#4 capture diff), I2 (addendum line numbers corrected `:317`→`:321`, specter addendum dropped per I1), I3 (lychee anchor-only default clarified), I4 (pre-flight promoted to mandatory Phase 1 step (i)), M1/M2/M3 inline. Load-bearing Piece 2 safety check (Makefile defaults all 4 binaries) PASSES. R1 re-dispatch pending.
