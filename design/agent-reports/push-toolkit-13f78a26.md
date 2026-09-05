# Push report: toolkit master to 13f78a26 via the staging-PR ritual (2026-09-05)

Followed the staging-PR form of the push ritual per
`design/agent-reports/push-toolkit-00980f9b.md` (precedent PRs #69, #70, #71).

## Pre-push state

- `git status --short | grep -v '^??'`: empty (no modified/staged tracked files).
- Local `master` tip: `13f78a26da11342c82abb9ed8ab2e8b627065753`.
- `origin/master` before push: `00980f9b8915f22879d501e7e38a08f46a144a2f`.
- `git log --oneline --first-parent origin/master..master`: 2 commits on
  master's own line — `46b40bbb` (push report for `00980f9b`), `13f78a26`
  (no-ff merge of `h5-manual`, parents `46b40bbb` + `b48af1c1`). The merge
  brings in one more commit (`b48af1c1`) reachable only via that second
  parent — 3 commits total in the full range, 2 on the first-parent line,
  matching the task's description ("a push report and a no-ff merge of the
  docs branch `h5-manual`").
- `git diff --stat origin/master..master`: 2 files changed, 128 insertions(+),
  9 deletions(-) — `design/agent-reports/push-toolkit-00980f9b.md`,
  `docs/manual/src/40-cli-reference/43-ms.md`.
  `git diff --name-only origin/master..master | grep -vE '^(docs/|design/)'`:
  empty — confirmed docs/records only, no crate code.
- Untracked files: 38, all `^??` (matches precedent's count), left untouched
  throughout — never staged, never referenced by `git add`.

Matched expectations exactly; proceeded.

## Staging PR

- `git push -f origin master:refs/heads/ci/staging` — new branch created at
  `13f78a26da11342c82abb9ed8ab2e8b627065753`.
- `gh pr create --repo bg002h/mnemonic-toolkit --base master --head ci/staging`
  → **PR #72**: https://github.com/bg002h/mnemonic-toolkit/pull/72

## Check-run conclusions on tip SHA `13f78a26da11342c82abb9ed8ab2e8b627065753`

Source: `gh api repos/bg002h/mnemonic-toolkit/commits/13f78a26.../check-runs`
(17 check runs, single workflow trigger, run id `33988583350` for the bulk of
jobs plus separate `examples` (`33988583490`), `build` (`33988583426`), and
two `sibling pins match install.sh` runs `33988579875` / `33988583354`).

| Check | Job id | Conclusion |
|---|---|---|
| build | 101366581179 | success |
| **clippy** | 101366581002 | **success** |
| **examples** | 101366581473 | **success** |
| fmt (pinned 1.95.0) | 101366580965 | success |
| g6 invariant (cross-repo mlock.rs) | 101366581161 | success |
| install.sh harnesses (man-step + MSRV guard) | 101366581114 | success |
| lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest) | 101366581118 | success |
| lib cross-platform check (x86_64-pc-windows-msvc, windows-latest) | 101366581132 | success |
| lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest) | 101366581051 | success |
| miri (mlock unsafe) | 101366580833 | success |
| musl build+test (aarch64-unknown-linux-musl) | 101366580963 | success (completed after the required contexts; see below) |
| musl build+test (x86_64-unknown-linux-musl) | 101366581038 | success |
| sibling pins match install.sh (run 33988579875) | 101366570856 | **failure** |
| sibling pins match install.sh (run 33988583354) | 101366580870 | **failure** |
| test (macos-latest) | 101366581048 | success |
| test (release, ubuntu-latest, mlock einval) | 101366580985 | success |
| **test (ubuntu-latest)** | 101366580971 | **success** |

**Required contexts** (`examples`, `test (ubuntu-latest)`, `clippy`): all
`success` on the **first attempt** (confirmed via `gh pr checks 72` at
~2 minutes post-trigger, and again via the check-runs API) — no rerun of the
known-flaky
`permutation_search::tests::cap_estimate_with_synthetic_slow_evaluator_exceeds_ceiling`
test (or anything else) was needed.

**`sibling pins match install.sh`**: `failure` on both runs — pre-existing and
by-design non-required (per `design/FOLLOWUPS.md`
`sibling-pin-check-red-by-design`), not caused by this push, not blocking,
matches precedent PRs #69/#70/#71 exactly.

**`musl build+test (aarch64-unknown-linux-musl)`**: still `in_progress` when
the required contexts were confirmed green; polled separately via
`gh api .../check-runs/101366580963` until it completed — `success`. It is
not in the required-context list, so the final push (below) was not blocked
on it, but it is recorded here for completeness since it did finish green
before the report was written.

`gh pr checks 72 --repo bg002h/mnemonic-toolkit` exited 1 (because of the
non-required `sibling pins` failures), same as precedent's `gh pr checks`
behavior.

## Push output (verbatim, `git push origin master`)

```
To github.com:bg002h/mnemonic-toolkit.git
   00980f9b..13f78a26  master -> master
```

## Bypass check

No "Bypassed rule violations" line anywhere in the push output. **Rule
satisfied, not bypassed.**

## origin/master after push

`git fetch origin && git rev-parse origin/master` =
`13f78a26da11342c82abb9ed8ab2e8b627065753` — equal to the local tip.

## PR #72 final state

`gh pr view 72 --repo bg002h/mnemonic-toolkit --json state,mergedAt,url` →
`{"mergedAt":"2026-09-05T20:17:38Z","state":"MERGED","url":"https://github.com/bg002h/mnemonic-toolkit/pull/72"}`.
GitHub auto-marked PR #72 **MERGED** once its head (`ci/staging`) became
reachable from `master` via the fast-forward — same behavior as PR #69/#70/#71.
No explicit close was needed or attempted.

## Staging ref cleanup

`git push origin --delete ci/staging` succeeded (`- [deleted] ci/staging`);
`git ls-remote origin refs/heads/ci/staging` returned empty — ref absent.

## Deviation from precedent

None on the required-context path: all three required contexts were green on
the first attempt, no rerun needed. The only difference from the `00980f9b`
report is that the non-required `musl build+test (aarch64-unknown-linux-musl)`
job was still running when the required contexts were confirmed and had to be
polled separately to completion (it finished `success`); this did not delay
or affect the final push, which proceeded once the required contexts were
green.

## Verdict

**SUCCESS**
