# Push report: toolkit master to 4fc30009 via the staging-PR ritual (2026-09-06)

Followed the staging-PR form of the push ritual per
`design/agent-reports/push-toolkit-13f78a26.md` (precedent PRs #69-#72).

## Pre-push state

- `git status --short | grep -v '^??'`: empty (no modified/staged tracked files).
- Local `master` tip: `4fc30009a35c5387e44a0d90c3f9e2d5d27eca7a`.
- `origin/master` before push: `13f78a26da11342c82abb9ed8ab2e8b627065753`.
- `git log --oneline --first-parent origin/master..master`: 2 commits —
  `6cb55bb8` (push report for `13f78a26`), `4fc30009` (manual: preimage
  plates -- the two forms, §8.6's QR text, and `--pack-preimage`, H6 Task 13
  Step 3).
- `git diff --stat origin/master..master`: 2 files changed, 223
  insertions(+), 2 deletions(-) — `design/agent-reports/push-toolkit-13f78a26.md`,
  `docs/manual/src/40-cli-reference/43-ms.md`.
  `git diff --name-only origin/master..master | grep -vE '^(docs/|design/)'`:
  empty — confirmed docs/records only, no crate code.
- Untracked files: 38 (`^??`), left untouched throughout — never staged,
  never referenced by `git add`. The `cycle-prep-recon-*.md` files in the
  repo root were not touched.

Matched the brief's description exactly; proceeded.

## Staging PR

- `git push -f origin master:refs/heads/ci/staging` — new branch created at
  `4fc30009a35c5387e44a0d90c3f9e2d5d27eca7a`.
- `gh pr create --repo bg002h/mnemonic-toolkit --base master --head ci/staging`
  → **PR #73**: https://github.com/bg002h/mnemonic-toolkit/pull/73

## Check-run conclusions on tip SHA `4fc30009a35c5387e44a0d90c3f9e2d5d27eca7a`

Source: `gh api repos/bg002h/mnemonic-toolkit/commits/4fc30009.../check-runs`
(17 check runs — matches precedent's count — single workflow trigger, run id
`34033405864` for the bulk of jobs plus separate `examples` (`34033405910`),
`build` (`34033406022`), and two `sibling pins match install.sh` runs
`34033401748` / `34033405869`).

| Check | Job id | Conclusion |
|---|---|---|
| build | 101487024746 | success |
| **clippy** | 101487024410 | **success** |
| **examples** | 101487024190 | **success** |
| fmt (pinned 1.95.0) | 101487024276 | success |
| g6 invariant (cross-repo mlock.rs) | 101487024420 | success |
| install.sh harnesses (man-step + MSRV guard) | 101487024414 | success |
| lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest) | 101487024377 | success |
| lib cross-platform check (x86_64-pc-windows-msvc, windows-latest) | 101487024433 | success |
| lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest) | 101487024440 | success |
| miri (mlock unsafe) | 101487024393 | success |
| musl build+test (aarch64-unknown-linux-musl) | 101487024478 | success (completed after the required contexts; see below) |
| musl build+test (x86_64-unknown-linux-musl) | 101487024429 | success |
| sibling pins match install.sh (run 34033401748) | 101487013553 | **failure** |
| sibling pins match install.sh (run 34033405869) | 101487024323 | **failure** |
| test (macos-latest) | 101487024486 | success |
| test (release, ubuntu-latest, mlock einval) | 101487024376 | success |
| **test (ubuntu-latest)** | 101487024435 | **success** |

**Required contexts** (`examples`, `test (ubuntu-latest)`, `clippy`): all
`success`, confirmed via repeated `gh pr checks 73` polling and the
check-runs API. `examples` and `clippy` finished quickly (8s, 1m2s);
`test (ubuntu-latest)` ran long (7m9s, `in_progress` for several polling
cycles) but finished `success` on the **first attempt** — no rerun of the
known-flaky `permutation_search::tests::cap_estimate_with_synthetic_slow_evaluator_exceeds_ceiling`
test was needed.

**`sibling pins match install.sh`**: `failure` on both runs — pre-existing
and by-design non-required (per `design/FOLLOWUPS.md`
`sibling-pin-check-red-by-design`), not caused by this push, not blocking,
matches precedent PRs #69-#72 exactly.

**`musl build+test (aarch64-unknown-linux-musl)`**: still `in_progress` when
the required contexts were confirmed green; not in the required-context
list, so the final push (below) was not blocked on it. Confirmed `success`
in the final check-runs pull used to build the table above (completed before
this report was written).

`gh pr checks 73 --repo bg002h/mnemonic-toolkit` exited 1 throughout polling
(because of the non-required `sibling pins` failures and, earlier, pending
jobs), same as precedent's `gh pr checks` behavior.

## Push output (verbatim, `git push origin master`)

```
To github.com:bg002h/mnemonic-toolkit.git
   13f78a26..4fc30009  master -> master
```

## Bypass check

No "Bypassed rule violations" line anywhere in the push output. **Rule
satisfied, not bypassed.**

## origin/master after push

`git fetch origin && git rev-parse origin/master` =
`4fc30009a35c5387e44a0d90c3f9e2d5d27eca7a` — equal to the local tip.

## PR #73 final state

`gh pr view 73 --repo bg002h/mnemonic-toolkit --json state,mergedAt,url` →
`{"mergedAt":"2026-09-06T12:41:07Z","state":"MERGED","url":"https://github.com/bg002h/mnemonic-toolkit/pull/73"}`.
GitHub auto-marked PR #73 **MERGED** once its head (`ci/staging`) became
reachable from `master` via the fast-forward — same behavior as PR #69-#72.
No explicit close was needed or attempted.

## Staging ref cleanup

`git push origin --delete ci/staging` succeeded (`- [deleted] ci/staging`);
`git ls-remote origin refs/heads/ci/staging` returned empty — ref absent.

## Deviation from precedent

None. All three required contexts were green on the first attempt; no rerun
of the flaky `permutation_search` test was needed. `test (ubuntu-latest)` ran
longer than usual (7m9s) but did not fail, so no retry logic was invoked.

## Verdict

**SUCCESS**
