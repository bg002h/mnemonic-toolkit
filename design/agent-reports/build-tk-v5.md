# Build report: FIX_v5_wsh_cost Parts A + B (E9)

Branch `build/v5-wsh-cost` from `origin/master` 12e013bc; FIX at f68e9bb1. Cherry-picks: header line applied; the
last pick (f68e9bb1) conflicted only in `design/FOLLOWUPS.md` (both sides append) and both kept. PR: #134 (draft).
No STOP hit. Nothing was decided beyond the FIX.

## Section 6 steps
1. RED: exit 100, `30 tests run: 11 passed, 19 failed, 4223 skipped` (doc quotes 4235; M-1 in `fix-v5-r1-minors`).
2. Part B: `12 tests run: 12 passed, 0 skipped`; F1 appended; GREEN commit 9b1be357.
3. Part A: patches A, C, D, E applied; tutorial transcript and `50-j4-taproot-twin.md` hunks reverted;
   `30 tests run: 30 passed, 4223 skipped`; `make audit` replay OK; Examples.md regenerated equals Appendix E;
   F2-F4 appended; GREEN commit ce2e9a54.
4. Gate below. 5. No version/CHANGELOG change.

## Section 7 gate (logs `/scratch/cache/tk-build-v5-logs/`)
| item | command | exit | key line |
|---|---|---|---|
| 1 | `cargo +1.95.0 fmt --all -- --check` | 0 | no diff |
| 2 | `cargo metadata --locked`; `cargo build --locked --tests --workspace` | 0 | |
| 3 | `cargo nextest run --locked --workspace --no-fail-fast` | 0 | `4215 tests run: 4215 passed, 38 skipped` |
| 4 | `cargo test --locked --workspace --doc` | 0 | |
| 5 | `cargo clippy --locked --all-targets -- -D warnings` | 0 | |
| 6 | mlock g2_1 (eperm env), g2_3 (einval env), g2_4 (off env), g6 invariant; six install-*.test.sh; docs-only; code-docs-check x2 | 0 | all 0 (first g2_1/g2_3 attempt lacked the workflow `env:`; rerun with it passed) |
| 6 | CI-only: release g2_3, miri, lib cross-platform, musl | not run | green CI on the PR |
| 7 | `ci/doc-gate-guard.test.sh` | 0 | |
| 8 | `make audit` (pinned mk/md/ms) | 0 | `[verify-examples] OK (62 transcripts pass)`; `OK anchor-check: 9 danglers ... match baseline` |
| 9 | `ci/doc-flag-lint.test.sh` | 0 | `15 passed, 0 failed` |
| 10 | gen.sh then `git diff --exit-code` (Examples.md staged) | 0 | no diff |
| 11 | `make pdf`, `make html` (MERMAID_FILTER=skip) | 0 | |
| 12 | manual-gui `make lint` | 2 | 3 FAILs (gui-schema-coverage, verify-figures-gui, verify-tutorial-figures) = the doc's phases 4, 9, 10, environmental (red on base per FIX); `verify-tutorial-transcripts: OK (95 artifacts byte-identical)` |
| 12 | `make html` / `make verify-examples-gui` / `doc-gate-guard` | 0 | |
| 12 | `make verify-examples` | 2 | 17 checked; drift only in ms-/mk- sibling transcripts (MK/MS_BIN=true, rustc warning text); no compare-cost transcript. Not compared against base. |
| 13 | row filter | 0 | `30 tests run: 30 passed, 4223 skipped` |
| 14 | mutation-gate | see below | |

Full nextest is 0 failed (STOP-7: V7 already fixed on master, so no red rows).

## FOLLOWUPS
F1 `compare-cost-wrapper-chain-label-scan` closed by 9b1be357; F2 `compare-cost-wsh-witness-script-omitted` closed by
ce2e9a54 (shas written in the report commit, since a commit cannot name its own sha); F3 and F4 filed `open`.
`fix-v5-r1-minors` M-1/M-4 not applied.

