# T1 fold of review finding I-1 (toolkit PR 133)

Worktree `/scratch/code/shibboleth/tk-t1-tripwire`, branch `fix/miniscript-tripwire-13-2`, start head 04ef5405.
Source: `design/agent-reports/review-t1-miniscript-tripwire.md` I-1. Only I-1 folded; M-1..M-4 and N1..N6
filed unfixed as FOLLOWUPS `t1-miniscript-tripwire-review-minors` (`deferred-minor`, toolkit T3).
The fix is the reviewer's, compared against `/scratch/cache/tk-t1-review-fix/ci/` and applied as written:
the `crate_texts` and `main` change in `ci/miniscript-tripwire.py`, the decoy line, and row_h only. The
scratch copy's rows i, j, k (M-1) and the `fixtures/` dir (identical to the worktree's) were not taken.

Plant = `(r"^\s*SortedMulti\(Threshold<", ...)` loosened to `r"^\s*SortedMulti\("` in MARKERS (V11's unusable
marker; it matches `ShInner`/`WshInner` in the real 13.2.0). Harness: a copy of `ci/` with a symlink to
`vendor/`, plant applied by sed, `bash ci/miniscript-tripwire.test.sh` run in derived mode and again with
`MINISCRIPT_132_CRATE` = the real `miniscript-13.2.0.crate` (downloaded from static.crates.io,
sha256 `4d79651c...32a8b` = the fixture's 13.2.0 cksum).

## 1. RED: plant passes everything on head 04ef5405 (before any edit)
Plant line: `35:    (r"^\s*SortedMulti\(", "Terminal::SortedMulti (#915)"),`

| run | rows | result |
|---|---|---|
| derived tree, plant | a-g | PASS x7, `all 7 rows passed`, rc=0 |
| real 13.2.0 crate, plant | a-g | PASS x7, `all 7 rows passed`, rc=0 |
| derived tree, no plant | a-g | PASS x7, rc=0 |
| real 13.2.0 crate, no plant | a-g | PASS x7, rc=0 |

The loosened marker survives in both modes: I-1 reproduced.

## 2. RED: row_h and the decoy added, old script
Added the decoy line to the derived tree and `row_h` (a sha-correct 13.3.0 crate holding only
`miniscript-13.3.0/notsrc/a.rs`), default row list extended to row_h. `bash ci/miniscript-tripwire.test.sh`, rc=1:
`PASS row_a..row_g`, then for row_h
`FAIL: exit 0, want 1`, `FAIL: output lacks: ::error::miniscript-13.3.0.crate has no src/*.rs files, so a scan of it would mean nothing. Failing closed.`,
`FAIL: output has: ::notice::` (the old script printed the step-7 green notice, "lacks" all four), `FAIL row_h`, `3 assertion(s) failed`.
The production fail-open reproduced.

## 3. GREEN: fix applied (`crate_texts(data)` generic `<dir>/src/`, returns a list; `main` exits 1 on an empty list)
| run | a | b | c | d | e | f | g | h | result |
|---|---|---|---|---|---|---|---|---|---|
| derived tree, plant | P | **FAIL** | P | **FAIL** | P | P | P | P | `2 assertion(s) failed`, rc=1 |
| real 13.2.0 crate, plant | P | **FAIL** | P | **FAIL** | P | P | P | P | `2 assertion(s) failed`, rc=1 |
| derived tree, no plant | P | P | P | P | P | P | P | P | `all 8 rows passed`, rc=0 |
| real 13.2.0 crate, no plant | P | P | P | P | P | P | P | P | `all 8 rows passed`, rc=0 |

Live, as the workflow runs it (fresh sparse index, 90 rows): `--baseline 13.2.0` prints
`No release newer than the baseline. The [patch.crates-io] pin stays.`; `--baseline 13.1.0` downloads and
scans the real 13.2.0 crate and prints the step-7 notice (all four missing), rc=0.

## 4. Gate jobs the change touches (one run, output in a file)
The change is `ci/` only (no Rust), so `cargo nextest run --locked` was not run. Workflow
`miniscript-fork-tripwire.yml` runs the tripwire test; `ci/docs-only.test.sh`, `ci/code-docs-check.test.sh`
and `ci/code-docs-check.sh` cover the docs-only and code-docs gates; `ci/doc-gate-guard.test.sh` guards both.

| command | result |
|---|---|
| `bash ci/miniscript-tripwire.test.sh` | `all 8 rows passed`, rc=0 |
| `bash ci/docs-only.test.sh` | `73 passed, 0 failed`, rc=0 |
| `bash ci/code-docs-check.test.sh` | `41 passed, 0 failed`, rc=0 |
| `bash ci/code-docs-check.sh` | no output, rc=0 |
| `bash ci/doc-gate-guard.test.sh` | `158 passed, 0 failed`, rc=0 |
| parse `ci/miniscript-tripwire.py` | rc=0 |

Raw capture: `scratchpad/out/gate.txt` (session scratchpad, not committed).

## 5. Commits (all pushed to origin/fix/miniscript-tripwire-13-2)
1. `de3f9243` ci: tripwire scans the downloaded crate for real (review I-1)
2. `85c79bb5` followups: defer T1 review M-1..M-4 and N1..N6 to toolkit T3
3. this report

## Notes
- The test script's header comment still reads "RED rows (a)-(f), plus row_g"; row_h makes it stale. Not edited
  (outside the fix as specified); it is a Nit for T3.
- The reviewer's M-1 candidate rows i, j, k remain in `/scratch/cache/tk-t1-review-fix/ci/miniscript-tripwire.test.sh` and in the review report.

0 Critical, 0 Important, 0 Minor, 1 Nit (stale test-script header comment) found while folding; I-1 closed.
