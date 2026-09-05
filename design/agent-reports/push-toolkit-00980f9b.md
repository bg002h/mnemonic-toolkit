# Push report: toolkit master to 00980f9b via the staging-PR ritual (2026-09-05)

Followed the staging-PR form of the push ritual per
`design/agent-reports/push-toolkit-67090e2a.md` (precedent PRs #69 and #70).

## Pre-push state

- `git status --short | grep -v '^??'`: empty (no modified/staged tracked files).
- Local `master` tip: `00980f9b8915f22879d501e7e38a08f46a144a2f`.
- `origin/master` before push: `67090e2a5d13091425e4e7c1129b2bcae1881a19`.
- `git log --oneline --first-parent origin/master..master`: 3 commits on
  master's own line — `13ac4e9f` (push report), `6cf3ecd8` (followup entry),
  `00980f9b` (no-ff merge of `h3-hashlock-device-manual`, parents `6cf3ecd8`
  + `6b2b4048`). The merge brings in two more commits (`2c5f31cd`,
  `6b2b4048`) reachable only via that second parent — 5 commits total in the
  full range, 3 on the first-parent line, matching the task's description.
- `git diff --stat origin/master..master`: 4 files changed, 319 insertions(+),
  0 deletions — `design/FOLLOWUPS.md`, `design/agent-reports/push-toolkit-67090e2a.md`,
  `docs/manual/.cspell.json`, `docs/manual/src/40-cli-reference/43-ms.md`.
  `git diff --name-only origin/master..master | grep -vE '^(docs/|design/)'`:
  empty — confirmed docs/records only, no crate code.
- Untracked `cycle-prep-recon-*.md` and other untracked `design/`,
  `docs/manual-gui/design/agent-reports/` files present (38 total) and left
  untouched throughout (never staged, never referenced by `git add`).

Matched expectations exactly; proceeded.

## Staging PR

- `git push -f origin master:refs/heads/ci/staging` — new branch created at
  `00980f9b8915f22879d501e7e38a08f46a144a2f`.
- `gh pr create --repo bg002h/mnemonic-toolkit --base master --head ci/staging`
  → **PR #71**: https://github.com/bg002h/mnemonic-toolkit/pull/71

## Check-run conclusions on tip SHA `00980f9b8915f22879d501e7e38a08f46a144a2f`

Single pass, no rerun needed (`gh api repos/bg002h/mnemonic-toolkit/commits/00980f9b.../check-runs`
and `gh pr checks 71 --repo bg002h/mnemonic-toolkit`), run ids `33974986876`
(main workflow), `33974986902` (build), `33974986899` (examples), and two
separate `sibling pins` runs `33974984912` / `33974986886`:

| Check | Job id | Conclusion |
|---|---|---|
| build | 101330000395 | success |
| clippy | 101330000415 | success |
| **examples** | 101330000396 | **success** |
| fmt (pinned 1.95.0) | 101330000260 | success |
| g6 invariant (cross-repo mlock.rs) | 101330000520 | success |
| install.sh harnesses (man-step + MSRV guard) | 101330000437 | success |
| lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest) | 101330000569 | success |
| lib cross-platform check (x86_64-pc-windows-msvc, windows-latest) | 101330000488 | success |
| lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest) | 101330000370 | success |
| miri (mlock unsafe) | 101330000446 | success |
| musl build+test (aarch64-unknown-linux-musl) | 101330000463 | success |
| musl build+test (x86_64-unknown-linux-musl) | 101330000597 | success |
| sibling pins match install.sh (run 33974984912) | 101329994635 | **failure** |
| sibling pins match install.sh (run 33974986886) | 101330000249 | **failure** |
| test (macos-latest) | 101330000381 | success |
| test (release, ubuntu-latest, mlock einval) | 101330000564 | success |
| **test (ubuntu-latest)** | 101330000462 | **success** |
| **clippy** | 101330000415 | **success** |

**Required contexts** (`examples`, `test (ubuntu-latest)`, `clippy`): all
`success` on the **first attempt** — no rerun of the known-flaky
`permutation_search::tests::cap_estimate_with_synthetic_slow_evaluator_exceeds_ceiling`
test (or anything else) was needed this time.

**`sibling pins match install.sh`**: `failure` on both runs — pre-existing and
by-design non-required (per `design/FOLLOWUPS.md`
`sibling-pin-check-red-by-design`), not caused by this push, not blocking,
matches precedent PRs #69/#70 exactly.

`gh pr checks 71 --repo bg002h/mnemonic-toolkit` exited 1 (because of the
non-required sibling-pin failure), same as precedent's `gh pr checks`
behavior.

## Push output (verbatim, `git push origin master`)

```
To github.com:bg002h/mnemonic-toolkit.git
   67090e2a..00980f9b  master -> master
```

## Bypass check

No "Bypassed rule violations" line anywhere in the push output. **Rule
satisfied, not bypassed.**

## origin/master after push

`git fetch origin && git rev-parse origin/master` =
`00980f9b8915f22879d501e7e38a08f46a144a2f` — equal to the local tip.

## PR #71 final state

`gh pr view 71 --repo bg002h/mnemonic-toolkit --json state,mergedAt,url` →
`{"mergedAt":"2026-09-05T15:47:44Z","state":"MERGED","url":"https://github.com/bg002h/mnemonic-toolkit/pull/71"}`.
GitHub auto-marked PR #71 **MERGED** once its head (`ci/staging`) became
reachable from `master` via the fast-forward — same behavior as PR #69/#70.
No explicit close was needed or attempted.

## Staging ref check

`git push origin --delete ci/staging` succeeded (`- [deleted] ci/staging`);
`git ls-remote origin refs/heads/ci/staging` returned empty — ref absent.

## Deviation from precedent

None. Unlike the `67090e2a` push (one rerun of a flaky wall-clock ceiling
test), this push's required contexts were all green on the first attempt.

## Verdict

**SUCCESS**
