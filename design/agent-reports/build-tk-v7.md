**⚠ Deviations to note (no product-point STOP hit): V5 rows file removed; local `make lint` sibling-layout symbol misses.**

# Build report: toolkit FIX-V7 (bundle-card recovery lines)

Branch `build/v7-bundle-recovery` from `044162e7`, master `12e013bc` merged (`d99fcc1b`, no conflict noted). Draft PR: https://github.com/bg002h/mnemonic-toolkit/pull/135 (#135).

## Commits
- `d99fcc1b` merge master
- `dcacedd2` RED evidence
- `26807197` drop V5's RED rows file (`cli_compare_cost_wsh_witness_script.rs`)
- `06a8a046` GREEN (production, tests, row 24 narrowed per STOP-2 (b), fmt)
- `3fff1984` docs, manual, technical manual, spec sentence, CHANGELOG, FOLLOWUPS (F1, `v7-fix-review-minors`), GUI tutorial interim notes (two files)
- this report in a final commit

## RED / GREEN (red.txt, green-rows.txt, nextest.txt)
- RED: `30 tests run: 6 passed, 24 failed, 4125 skipped` (24 of 30 fail on base, as the doc measured)
- GREEN rows: `30 tests run: 30 passed, 4125 skipped`
- Full workspace: `4215 tests run: 4215 passed, 38 skipped`

## Gate (files in /scratch/cache/tk-build-v7-gate/)
- rust.yml: fmt (1.95.0) 0, build, nextest, doc tests, clippy -D warnings, mlock g2_1/g2_3/g2_4, g6 (2 passed), six install scripts, three ci scripts: all pass.
- manual.yml: doc-gate-guard 158 passed 0 failed; `make audit` lint OK, verify-examples 62 pass, anchor-check 9 danglers = baseline; doc-flag-lint mutation self-test 15 passed 0 failed; examples gen identical; pdf and html built.
- technical manual: CSpell 26 files 0 issues (25 before pdf/html); verify-examples 18 pass; symbol-ref-check in the CI-layout symlink workspace `OK (301 anchors checked, 427 skipped)`; pdf, html built.
- quickstart: lint OK, verify-examples 62 pass, pdf, html built.
- GUI manual (pinned mnemonic-gui-v0.63.0): html built, lint OK, CSpell 104 files.
- Not run: GUI manual verify-examples against the pinned CLI tier, gui-render jobs, release/fuzz/differential/CI-only workflows; mutation gate (coordinator's).

## Deviations
1. V5's rows file `cli_compare_cost_wsh_witness_script.rs` fails 9 rows on this branch (V5 behaviour is not in V7's base). The FIX header says not to take V5's rows, so it was `git rm`'d in its own commit. After that fmt exit 0 and nextest fully green. If V5 lands first the file returns with it.
2. A local `make lint` of the technical manual in the real sibling layout fails symbol-ref-check on `fresh_chunk_set_id` (two mk chapters, present on base, the FIX doc says CI skips them) and on `plain_multisig_threshold` (it resolves to the sibling master checkout, not this worktree). The symlink-workspace run the doc prescribes passes.
3. mlock g2 rows need CI's env (`MNEMONIC_TEST_MLOCK_FAIL_MODE`, `ulimit -l 65536`); first run without it failed, rerun as CI passes.

## STOPs
None.

0 Critical, 0 Important, 0 STOPs
