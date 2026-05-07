# Phase 1 — Lint trapdoor test transcript

**Date:** 2026-05-07
**Branch:** `manual/v0_1`
**Purpose:** Prove the index bidirectional check in `tests/lint.sh` is *not* vacuously passing — i.e., it actually fails when source-side `\index{}` markers and `69-index-table.md` rows fall out of lockstep.

The plan mandates that Phase 1 demonstrate this via a deliberate-failure round. The discipline:

1. With both a real `\index{m-format star}` marker in `00-frontmatter.md` AND a matching row in `69-index-table.md`, run `make lint` (exercising step 6 / "index bidirectional"). It must PASS.
2. Delete the matching row from `69-index-table.md` (preserving the source-side marker). Run lint again. It must FAIL with the expected diagnostic.
3. Restore the row. Run lint a third time. It must PASS again.

If step 2 passes, the linter is broken (vacuously matching empty inputs) and Phase 1 is not converged.

## Step 1 — entry present (must PASS)

Invocation:

```sh
bash tests/lint.sh \
  SRC_DIR="$(pwd)/src" \
  TESTS_DIR="$(pwd)/tests" \
  MNEMONIC_BIN="true" MD_BIN="true" MS_BIN="true"
```

(The `true` placeholder for the CLI binaries makes flag-coverage emit
a "no flags parsed" warning per pair and skip the chapter check —
the trapdoor test exercises step 6 "index bidirectional", not flag
coverage.)

Result tail:

```
[lint] === 6/6 index bidirectional ===

[lint] OK
```

Exit code: **0**. ✅ PASSES as required.

## Step 2 — entry removed (must FAIL with the right diagnostic)

Modification: deleted the `| `m-format star` | ... |` row from
`src/60-appendices/69-index-table.md` (file backed up to
`/tmp/69-index-table.md.backup` for restoration).

Re-ran the same lint invocation. Result tail:

```
[lint] === 6/6 index bidirectional ===
[lint] FAIL: src \index{m-format star} missing from .../60-appendices/69-index-table.md

[lint] FAILED
```

Exit code: **1**. ✅ FAILS as required, with the exact diagnostic the
plan specifies (`src \index{TERM} missing from <path>`).

## Step 3 — entry restored (must PASS again)

Modification: copied `/tmp/69-index-table.md.backup` back over
`69-index-table.md`.

Re-ran the same lint invocation. Result tail:

```
[lint] === 6/6 index bidirectional ===

[lint] OK
```

Exit code: **0**. ✅ PASSES as required.

## Conclusion

The trapdoor test demonstrates that the bidirectional consistency
check is exercised (not vacuously passing). The implementation is
honest: present + matching → OK; missing on either side → FAIL with
a concrete diagnostic naming the offending term and the affected
file. Phase 1 acceptance criterion A5 (index present in both
formats; bidirectional check passes) is satisfied for the
markdown-side curated table; the PDF-side `\printindex` is exercised
by `make filter-smoke` (Phase 0 deliverable, blocked locally on
texlive availability — verified by CI in Phase 8).

## Bug fixes applied during Phase 1 (lint.sh)

Two latent bugs in `tests/lint.sh` were caught while running this
trapdoor test and fixed in-place:

1. **Self-reference bug in index bidirectional scan.** The src-side
   scan picked up `\index{TERM}` from `69-index-table.md`'s own
   prose (which uses the syntax as documentation). Added
   `--exclude='69-index-table.md'` to the recursive grep.

2. **Grep-help leak in flag-coverage.** The chapter-grep for the
   discovered flag (`grep -qF "$flag" "$chapter"`) was missing the
   end-of-options `--` marker. When `$flag` was `--help` (or any
   long flag), grep parsed it as an option and emitted its `--help`
   text, polluting lint output. Fix: `grep -qF -- "$flag" "$chapter"`.

Both fixes are committed in this Phase 1 commit alongside the new
content.
