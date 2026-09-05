# Push report: toolkit master to 67090e2a via staging-PR ritual (2026-09-05)

Followed the staging-PR form of the push ritual per
`design/agent-reports/push-toolkit-7e07088c.md` (precedent PR #69).

## Pre-push state

- `git -C /scratch/code/shibboleth/mnemonic-toolkit status --short | grep -v '^??'`: empty (no modified tracked files).
- Local `master` tip: `67090e2a5d13091425e4e7c1129b2bcae1881a19`.
- `origin/master` before push: `7e07088c3a5fc33518b1dbd2f9fd0c02e9bb716b`.
- `git log --oneline origin/master..master`: 1 commit ahead (`67090e2a` — a
  docs-only commit adding `design/agent-reports/push-toolkit-7e07088c.md`,
  confirmed via `git show --stat 67090e2a`: 1 file changed, 90 insertions,
  0 deletions).
- `git log --oneline master..origin/master | wc -l`: 0 (0 behind).
- Untracked `cycle-prep-recon-*.md` files present in repo root and left
  untouched throughout (never staged).

Matched expectations exactly; proceeded.

## Staging PR

- `git -C /scratch/code/shibboleth/mnemonic-toolkit push -f origin master:refs/heads/ci/staging` —
  new branch created.
- `gh pr create --repo bg002h/mnemonic-toolkit --base master --head ci/staging`
  → **PR #70**: https://github.com/bg002h/mnemonic-toolkit/pull/70

## Check-run conclusions on tip SHA `67090e2a5d13091425e4e7c1129b2bcae1881a19`

First pass (`gh api repos/bg002h/mnemonic-toolkit/commits/67090e2a.../check-runs`,
run `33946149247` attempt 1):

| Check | Conclusion |
|---|---|
| build/manual (not present this run) | n/a |
| clippy | success |
| examples | success |
| fmt (pinned 1.95.0) | success |
| g6 invariant (cross-repo mlock.rs) | success |
| install.sh harnesses (man-step + MSRV guard) | success |
| lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest) | success |
| lib cross-platform check (x86_64-pc-windows-msvc, windows-latest) | success |
| lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest) | success |
| miri (mlock unsafe) | success |
| musl build+test (aarch64-unknown-linux-musl) | success |
| musl build+test (x86_64-unknown-linux-musl) | success |
| sibling pins match install.sh (job 101252472981) | **failure** |
| sibling pins match install.sh (job 101252483050) | **failure** |
| test (macos-latest) | success |
| test (release, ubuntu-latest, mlock einval) | success |
| **test (ubuntu-latest)** (job 101252483016) | **failure** |

`test (ubuntu-latest)` (a required context) failed with:

```
thread 'permutation_search::tests::cap_estimate_with_synthetic_slow_evaluator_exceeds_ceiling' panicked at crates/mnemonic-toolkit/src/permutation_search.rs:1610:9:
expected ceiling refusal, got Ok(RunWithProgress { estimate: 3219.3697536s })
```

This is a wall-clock-calibration test asserting an estimate exceeds a ceiling;
the commit under push touches only a `.md` report file (verified above), so
this is CI-load-dependent flakiness, not a regression from this push.

Ran `gh run rerun 33946149247 --repo bg002h/mnemonic-toolkit --failed` (after
confirming via `gh api .../runs/33946149247 --jq .status` = `completed`, all
jobs finished, only `test (ubuntu-latest)` and the two `sibling pins` jobs at
`failure`). GitHub queued a new attempt (`run_attempt: 2`) for `test
(ubuntu-latest)`, job id `101255149739`. Polled
`gh api repos/bg002h/mnemonic-toolkit/actions/jobs/101255149739 --jq
'{status,conclusion}'` every 15s until `completed` — **result: `success`** in
7m26s, confirming the flake theory. The `sibling pins match install.sh` jobs
were not rerun (see below — pre-existing, non-required, unrelated).

Final state (`gh pr checks 70 --repo bg002h/mnemonic-toolkit` and per-job via
`gh api .../commits/67090e2a.../check-runs`):

| Check | Conclusion |
|---|---|
| clippy | success |
| examples | success |
| fmt (pinned 1.95.0) | success |
| g6 invariant (cross-repo mlock.rs) | success |
| install.sh harnesses (man-step + MSRV guard) | success |
| lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest) | success |
| lib cross-platform check (x86_64-pc-windows-msvc, windows-latest) | success |
| lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest) | success |
| miri (mlock unsafe) | success |
| musl build+test (aarch64-unknown-linux-musl) | success |
| musl build+test (x86_64-unknown-linux-musl) | success |
| sibling pins match install.sh (job 101252472981) | **failure** |
| sibling pins match install.sh (job 101252483050) | **failure** |
| test (macos-latest) | success |
| test (release, ubuntu-latest, mlock einval) | success |
| **test (ubuntu-latest)** (job 101255149739, attempt 2) | **success** |

**Required contexts** (`examples`, `test (ubuntu-latest)`, `clippy`): all
`success` after the rerun. **`sibling pins match install.sh`**: `failure`,
pre-existing and by-design non-required (per `design/FOLLOWUPS.md`
`sibling-pin-check-red-by-design`) — not caused by this push, not blocking,
matches precedent PR #69 exactly.

`gh pr checks 70` itself exited 1 (because of the non-required sibling-pin
failure), same as precedent's `gh pr checks 69 --watch` behavior.

## Push output (verbatim, `git -C /scratch/code/shibboleth/mnemonic-toolkit push origin master`)

```
To github.com:bg002h/mnemonic-toolkit.git
   7e07088c..67090e2a  master -> master
```

## Bypass check

No "Bypassed rule violations" line anywhere in the push output. **Rule
satisfied, not bypassed.**

## origin/master after push

`git -C /scratch/code/shibboleth/mnemonic-toolkit fetch origin && git rev-parse
origin/master` = `67090e2a5d13091425e4e7c1129b2bcae1881a19` — equal to the
local tip.

## PR #70 final state

GitHub auto-marked PR #70 **MERGED** (`mergedAt: 2026-09-05T05:32:31Z`) once
its head (`ci/staging`) became reachable from `master` via the fast-forward —
same behavior as PR #69. No explicit close was needed or attempted.

## Staging ref check

`git -C /scratch/code/shibboleth/mnemonic-toolkit push origin --delete
ci/staging` succeeded; `git ls-remote origin refs/heads/ci/staging` returns
empty — ref absent.

## Deviation from precedent

Precedent PR #69 needed no rerun; this push required one rerun of `test
(ubuntu-latest)` due to a flaky wall-clock-ceiling test unrelated to the
docs-only diff. Logged here rather than filed as a new FOLLOWUP, since the
test itself (`cap_estimate_with_synthetic_slow_evaluator_exceeds_ceiling`) is
pre-existing and outside this task's scope.

## Verdict

**SUCCESS**
