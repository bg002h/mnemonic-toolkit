# Push report: toolkit master to 7e07088c via staging-PR ritual (2026-09-05)

Followed the staging-PR form of the push ritual per
`design/agent-reports/decision-toolkit-docs-push-path.md` (precedent PR #68).

## Pre-push state (step 1)

- `git status --short | grep -v '^??'`: empty (no modified tracked files).
- Local `master` tip: `7e07088c3a5fc33518b1dbd2f9fd0c02e9bb716b`.
- `git log --oneline origin/master..master`: 5 commits ahead (7e07088c,
  954eceb2, 35b8779d, 0e4abdff, 87e594e0).
- `git log --oneline master..origin/master | wc -l`: 0 (0 behind).

Matched expectations exactly; proceeded.

## Staging PR

- `git push -f origin master:refs/heads/ci/staging` — new branch created.
- `gh pr create --repo bg002h/mnemonic-toolkit --base master --head ci/staging`
  → **PR #69**: https://github.com/bg002h/mnemonic-toolkit/pull/69

## Check-run conclusions on tip SHA `7e07088c3a5fc33518b1dbd2f9fd0c02e9bb716b`

(`gh api repos/bg002h/mnemonic-toolkit/commits/7e07088c.../check-runs`)

| Check | Conclusion |
|---|---|
| build | success |
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
| sibling pins match install.sh (job 101243362454) | **failure** |
| sibling pins match install.sh (job 101243374378) | **failure** |
| test (macos-latest) | success |
| test (release, ubuntu-latest, mlock einval) | success |
| test (ubuntu-latest) | success |

**Required contexts** (`examples`, `test (ubuntu-latest)`, `clippy`): all
`success`. **`build`** (manual.yml, exercises the new docs): `success`, as
predicted. **`sibling pins match install.sh`**: `failure`, pre-existing and
by-design non-required (per `design/FOLLOWUPS.md`
`sibling-pin-check-red-by-design`) — not caused by this push, not blocking.

`gh pr checks 69 --watch` itself exited 1 (because of the non-required
sibling-pin failure), but its per-job table matched the API's per-SHA results
above.

## Push output (verbatim, `git push origin master`)

```
To github.com:bg002h/mnemonic-toolkit.git
   d39d9626..7e07088c  master -> master
```

Full output also saved at `/scratch/code/shibboleth/.tmp/push-toolkit-7e07088c.log`.

## Bypass check

No "Bypassed rule violations" line anywhere in the push output. **Rule
satisfied, not bypassed.**

## origin/master after push

`git fetch origin && git rev-parse origin/master` = `7e07088c3a5fc33518b1dbd2f9fd0c02e9bb716b`
— equal to the local tip.

## PR #69 final state

GitHub auto-marked PR #69 **MERGED** (`mergedAt: 2026-09-05T04:09:11Z`) once
its head (`ci/staging`) became reachable from `master` via the fast-forward —
exactly the behavior the decision doc predicted ("GitHub marks the PR merged
on its own once its head is reachable from master"). No explicit close was
needed or attempted, since the PR was not left open/unmerged to close.

## Staging ref check

`git push origin --delete ci/staging` succeeded; `git ls-remote origin
refs/heads/ci/staging` returns empty — ref absent.

## Verdict

**SUCCESS**
