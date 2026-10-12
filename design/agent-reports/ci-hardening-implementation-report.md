# CI hardening implementation report (2026-10-11)

Outcome: all seven branches `ci/hardening-2026-10-11` are pushed and have a DRAFT PR; every gate run was GREEN, actionlint had no new findings anywhere, and CI is green on all seven PRs (0 failures). No stop-and-ask point was hit.

## Per repo

| Repo | Worktree | Commit | PR | Gate counts | actionlint before/after | CI |
|---|---|---|---|---|---|---|
| descriptor-mnemonic | /scratch/code/shibboleth/dm-worktrees/ci-hardening | 15df6a36d1cec7203508dc2b514b092efb099100 | https://github.com/bg002h/descriptor-mnemonic/pull/36 | files=6 T=16 X=0 C=4 K_steps=0 K_env=0 | 0 / 0 lines | 38 pass, 4 skipping, 0 fail |
| mnemonic-toolkit | /scratch/code/shibboleth/tk-worktrees/ci-hardening | 608eaf7b9b6e53087164108d35a1dfac9d2ce3ac | https://github.com/bg002h/mnemonic-toolkit/pull/102 | files=19 T=42 X=6 C=12 K_steps=10 K_env=9 | 0 / 0 | 65 pass, 6 skipping, 0 fail |
| mnemonic-gui | /scratch/code/shibboleth/gui-worktrees/ci-hardening | 892f5dccc7a0de490fc53b218b8678ac515ab7c1 | https://github.com/bg002h/mnemonic-gui/pull/39 | files=2 T=8 X=0 C=2 K_steps=0 K_env=0 | 0 / 0 | 26 pass, 2 skipping, 0 fail |
| mnemonic-key | /scratch/code/shibboleth/mk-worktrees/ci-hardening | 56637f87c7a64e1e52a83c99da6d5fedcb396163 | https://github.com/bg002h/mnemonic-key/pull/12 | files=5 T=13 X=0 C=3 K_steps=0 K_env=0 | 0 / 0 | 42 pass, 6 skipping, 0 fail |
| mnemonic-secret | /scratch/code/shibboleth/ms-worktrees/ci-hardening | 7b470eaab56d020b58c339655fef7eb8dc892dc4 | https://github.com/bg002h/mnemonic-secret/pull/11 | files=5 T=20 X=0 C=3 K_steps=0 K_env=0 | 0 / 0 | 38 pass, 4 skipping, 0 fail |
| mnemonic-transaction | /scratch/code/shibboleth/mt-worktrees/ci-hardening | c2f123faba93bff93092e1a93b08695e05ed3231 | https://github.com/bg002h/mnemonic-transaction/pull/2 | files=2 T=4 X=0 C=1 K_steps=0 K_env=0 | 0 / 0 | 14 pass, 2 skipping, 0 fail |
| mnemonic-engrave | /scratch/code/shibboleth/me-worktrees/ci-hardening | 232bd0e164ed0637207f55884b52a1f4253755d2 | https://github.com/bg002h/mnemonic-engrave/pull/18 | files=1 T=5 X=0 C=1 K_steps=0 K_env=0 | 0 / 0 | 16 pass, 2 skipping, 0 fail |

All counts equal the plan's table. Every repo's GATE line was `GATE: GREEN`. Bases at the plan's SHAs (verified after `git fetch`). Toolkit counts include its re-run (below).

Engrave `release-gen` (run immediately before the engrave commit, after all five generated repos were committed):
```
release-gen descriptor-mnemonic: identical
release-gen mnemonic-secret: identical
release-gen mnemonic-key: identical
release-gen mnemonic-transaction: identical
release-gen mnemonic-toolkit: identical
GATE: GREEN
```
`emit.py` was not run. The engrave commit changes `.github/workflows/release.yml` (T 5, C 1) and `scripts/release-workflows/gen.py` (3 lines).

CI "required checks": the repos name no required-check set that I could see; all checks reported by `gh pr checks` were pass or skipping (skipping = jobs gated off on PRs, e.g. release/publish jobs). Every PR got two runs per workflow (push and pull_request event); both passed.

## Toolkit cache timings (Install steps, seconds)

Miss = first PR run (pull_request event, `manual` run 38182692639 attempt 1); hit = attempt 2 of the same run after `gh run rerun`.

| Step | Miss | Hit |
|---|---|---|
| Install mk-cli | 43 | 8 |
| Install md-cli | 55 | 3 |
| Install ms-cli | 27 | 2 |
| Restore cached sibling CLIs | 0 | 0 |

(Total of the three: 125 s miss, 13 s hit.) Attempt 2 concluded success. Other first-run (miss) timings: push-event `manual` run 38182688768: mk 54, md 71, ms 37; `quickstart` PR run 38182692662: 49/70/35; `technical-manual` PR run 38182692652: 51/69/33; `manual-gui` PR run 38182692645: GUI-pinned CLI tier 164, mnemonic binary 114, gui-render 93. Install pandoc + texlive on these runs: 112-230 s (well inside the 15 min step timeout). The Post Restore steps ran on the miss (cache saved). The manual-gui, quickstart and technical-manual caches were not re-run for a hit (the plan asked for `manual` only).

## Stop-and-ask questions

None. No count differed, no job/step already had `timeout-minutes`/`env:`, no listed file had `concurrency:`, no gate RED, no actionlint finding, no CI failure.

## Things done that the plan did not literally say

- Edited T and C with a small script kept in the session scratchpad (not in any repo): it inserts `timeout-minutes: 60` after every 4-space `runs-on:` line and the C block above `jobs:`. For files that have a `pull_request` trigger and already a top-level `concurrency:` (every `release.yml` except engrave's) it skips C, as the plan says; the plan's STOP only applies to files on its list. X, K1-K4 and gen.py were edited by separate small Python snippets; the gate verified all of it.
- The script confirmed the line above every C block is blank (checked in each diff) and that no `release.yml` other than engrave's got C.
- Toolkit commit: the gate was run on the worktree against `origin/master`; the worktree also holds the earlier design commits, which the gate ignores. The toolkit PR therefore also shows those existing design commits (plan, gate, reviews) as part of the branch.
- PR bodies: wording written by me (what changed, measured reasons from the plan, gate output, plan link to the toolkit branch's blob URL). All end with the Claude Code line.
- Pushed with an explicit `git push -u origin ci/hardening-2026-10-11` (worktrees were created tracking the base branch, so a bare `git push` would have refused).
- Re-ran only the toolkit PR's `manual` run, once (as instructed).
- Main checkouts untouched. Main toolkit checkout has 39 uncommitted files and main engrave 1; these predate this work and I did not touch them.
- Nothing merged, no PR marked ready, no commit amended, no force-push.

## Reading notes for the whole-diff reviewer

- Not covered by the gate and read by me in the diffs: comment wording, blank lines, placement of `env:` above `run:` (after the step's comments), K1 above the technical-manual section comment, K4 comment edit. technical-manual K4 line is now longer than the following wrapped comment lines, as the plan allowed.
- The persisted commit messages hold the gate output; the same text is in each PR body.
