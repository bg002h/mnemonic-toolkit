# Push report: toolkit master to 90cb039f via the staging-PR ritual (2026-09-06)

Followed the staging-PR form of the push ritual per
`design/agent-reports/push-toolkit-4fc30009.md` (precedent PRs #69-#73).

## Pre-push state

- `git status --short | grep -v '^??'`: empty (no modified/staged tracked files).
- Local `master` tip: `90cb039f0c39719c3510263df977c314d77d37a4`.
- `origin/master` before push: `4fc30009a35c5387e44a0d90c3f9e2d5d27eca7a`.
- `git log --oneline --first-parent origin/master..master`: 2 commits —
  `aef5ee3e` (push report for `4fc30009`), `90cb039f` (manual: a workflow
  section for a refused payload, and the phrase: record now has a producer,
  F-492/F-495).
- `git diff --stat origin/master..master`: 3 files changed, 204
  insertions(+), 3 deletions(-) — `design/agent-reports/push-toolkit-4fc30009.md`,
  `docs/manual/src/30-workflows/3B-payload-unlock-refusals.md` (new),
  `docs/manual/src/40-cli-reference/43-ms.md`.
  `git diff --name-only origin/master..master | grep -vE '^(docs/|design/)'`:
  empty — confirmed docs/records only, no crate code.
- Untracked files: 38 (`^??`), left untouched throughout — never staged,
  never referenced by `git add`. The `cycle-prep-recon-*.md` files in the
  repo root were not touched.

Matched the brief's description exactly; proceeded.

## Staging PR

- `git push -f origin master:refs/heads/ci/staging` — new branch created at
  `90cb039f0c39719c3510263df977c314d77d37a4`.
- `gh pr create --repo bg002h/mnemonic-toolkit --base master --head ci/staging`
  → **PR #74**: https://github.com/bg002h/mnemonic-toolkit/pull/74

## Check-run conclusions on tip SHA `90cb039f0c39719c3510263df977c314d77d37a4`

Source: `gh api repos/bg002h/mnemonic-toolkit/commits/90cb039f.../check-runs`
— separate workflow triggers: run id `34052094286` ("rust", the bulk of
jobs), `34052094345` (`examples`), `34052094281` (`build`/manual), and two
`sibling pins match install.sh` runs `34052091714` / `34052094311`.

| Check | Job id | Conclusion |
|---|---|---|
| build | 101537407552 | success |
| **clippy** | 101537407751 | **success** |
| **examples** | 101537407795 | **success** |
| fmt (pinned 1.95.0) | 101537407829 | success |
| g6 invariant (cross-repo mlock.rs) | 101537407722 | success |
| install.sh harnesses (man-step + MSRV guard) | 101537407790 | success |
| lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest) | 101537407944 | success |
| lib cross-platform check (x86_64-pc-windows-msvc, windows-latest) | 101537407779 | success |
| lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest) | 101537407921 | success |
| miri (mlock unsafe) | 101537407731 | success |
| musl build+test (aarch64-unknown-linux-musl) | 101537407847 | success (completed after the required contexts; confirmed via a foreground `gh run watch 34052094286` tail showing "Complete job") |
| musl build+test (x86_64-unknown-linux-musl) | 101537407835 | success |
| sibling pins match install.sh (run 34052091714) | 101537401107 | **failure** |
| sibling pins match install.sh (run 34052094311) | 101537407754 | **failure** |
| test (macos-latest) | 101537407770 | success |
| test (release, ubuntu-latest, mlock einval) | 101537407760 | success |
| **test (ubuntu-latest)** | 101537407738 | **success** |

**Required contexts** (`examples`, `test (ubuntu-latest)`, `clippy`): all
`success`, confirmed by a foreground Monitor polling
`gh api repos/bg002h/mnemonic-toolkit/commits/90cb039f.../check-runs` every
30s until all three showed a non-pending status, then re-confirmed with a
direct `gh api .../check-runs` pull. All three finished on the **first
attempt** — no rerun of the known-flaky
`permutation_search::tests::cap_estimate_with_synthetic_slow_evaluator_exceeds_ceiling`
test was needed (the job never failed).

**`sibling pins match install.sh`**: `failure` on both runs — pre-existing
and by-design non-required (per `design/FOLLOWUPS.md`
`sibling-pin-check-red-by-design`), not caused by this push, not blocking,
matches precedent PRs #69-#73 exactly.

**`musl build+test (aarch64-unknown-linux-musl)`**: still `in_progress` when
the required contexts were confirmed green; not in the required-context
list, so the final push (below) was not blocked on it. Confirmed `success`
in a later check-runs pull.

## Push output (verbatim, `git push origin master`)

```
To github.com:bg002h/mnemonic-toolkit.git
   4fc30009..90cb039f  master -> master
```

## Bypass check

No "Bypassed rule violations" line anywhere in the push output. **Rule
satisfied, not bypassed.**

## origin/master after push

`git fetch origin && git rev-parse origin/master` =
`90cb039f0c39719c3510263df977c314d77d37a4` — equal to the local tip.

## PR #74 final state

`gh pr view 74 --repo bg002h/mnemonic-toolkit --json state,mergedAt,url` →
`{"mergedAt":"2026-09-06T18:44:36Z","state":"MERGED","url":"https://github.com/bg002h/mnemonic-toolkit/pull/74"}`.
GitHub auto-marked PR #74 **MERGED** once its head (`ci/staging`) became
reachable from `master` via the fast-forward — same behavior as PR #69-#73.
No explicit close was needed or attempted.

## Staging ref cleanup

`git push origin --delete ci/staging` succeeded (`- [deleted] ci/staging`);
`git ls-remote origin refs/heads/ci/staging` returned empty — ref absent.

## Deviation from precedent

None. All three required contexts were green on the first attempt; no rerun
of the flaky `permutation_search` test was needed.

## Verdict

**SUCCESS**
