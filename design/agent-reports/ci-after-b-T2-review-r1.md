r1: 0C/1I/2M/1N

# L3 review: mnemonic-toolkit PR #115 (after-Part-B T2), round r1

Head reviewed: 287593f041d8f85ee20be0b349585d1403c57f75
(`git diff origin/master...287593f0`: shared-drift.yml 25, shared-drift.sh 50,
shared-drift.test.sh 114, build report 82 lines). Scratch work:
`/scratch/code/shibboleth/review-ab-T2-r1/`.

**The one question.** In the workflow, no: every wrong-PASS path I found is
stopped by the self-test step, which runs first. Run on its own, yes: one
path exists, I-1 below. The workflow runs the comparison on `schedule` and
`workflow_dispatch` and does not break toolkit's CI. PR #115's checks are
green, the only skipped checks are the allowed ones, and `shared-drift` is
not in master's required contexts (checked with
`gh api .../branches/master/protection/required_status_checks`).

## Important

### I-1: `sha` fails open. With `sha256sum` missing or failing, every pair prints `ok` and the script prints `shared-drift: PASS`

`sha()` runs `h=$(sha256sum < "$1")`. It is called as `here_sha=$(sha "$f")`
and `there_sha=$(sha "$tmp")`. Outside POSIX mode, bash turns off `-e` inside
command substitutions. So when `sha256sum` fails, the function keeps going,
`h` stays empty, `printf` exits 0, and both hashes come out as the empty
string. They compare equal, so the pair prints `ok`. `set -eu` (with or
without `pipefail`) does not catch this. This is the errexit gap that
brief item 1 points at, and it shows up here rather than in a pipeline (the
script has no pipelines).

Reproduction: put a `gh` stub that always prints `DIFFERENT-BYTES` on a PATH
that has bash, mktemp, rm, cat, env, dirname and printf but no sha256sum,
then run the script from the worktree root:

```
ci/shared-drift.sh: line 21: sha256sum: command not found
...
ok    mnemonic-transaction ci/shared-ci.sha256
shared-drift: PASS (5 repos, 5 files)
```

All 25 remote files differ, yet the result is PASS with exit 0.

Grading. Likelihood test condition (1) fails, because the result is a wrong
PASS. That makes this an Important, not a Minor. It is not a Critical,
because it cannot happen in the workflow today. Under the same PATH,
`ci/shared-drift.test.sh` fails (D2 and D4 report `exit 0`, giving
`4 passed, 2 failed`). The self-test step runs before the comparison step,
so the job goes red. `ubuntu-latest` also ships coreutils. What remains
exposed is a standalone or local run, for example on a macOS host without
coreutils, or with a broken `sha256sum`. In that case the tool reports
"every pair matches" without having compared anything. The header lists
`sha256sum` as a requirement but nothing enforces it.

Fix (verified). In `sha()`, change line 21 to:

```
  h=$(sha256sum < "$1") || exit 1
```

When sha256sum fails, the substitution subshell now exits 1. The
assignment `here_sha=$(sha "$f")` then returns non-zero, and the parent's
`set -e` aborts with no PASS line. With this change the reproduction above
exits 1 (`fixed, no sha256sum: rc=1`), and `shared-drift.test.sh` still
reports `6 passed, 0 failed`. To lock it in, add an optional row D7: put a
`sha256sum` stub that exits 1 at the front of PATH, then expect a non-zero
exit and no `shared-drift: PASS` line. The spec's §5.1 and §5.2 tables and
`reference/after-b/T2/` change with the fix.

## Minor

- M-1: Several kinds of silence produce no failing run, so they give no
  signal. GitHub disables a public repo's scheduled workflows after 60 days
  with no repo activity, and it sends scheduled-run failure emails only to
  the last person who edited the cron line. Neither shows up as a red run.
  Inherent to §5.3's "a failing scheduled run is the signal". Worth one
  line under §6.4.
- M-2: D6 checks that each argument shows up somewhere in `$GH_ARGS`, not
  that they belong to one call or appear in order. A mutant that adds a
  second `-H 'Accept: application/json'` would still pass D6. This is
  contrived, and the live run's 25 `ok` lines show the raw body arrives
  today.

## Nit

- N-1: §5.1 says "missing locally" is "checked first", before the fetch.
  No row can see that ordering: D5 passes whether the local check runs
  before or after the `gh` call. It would only matter as a wasted API call.

## Checked and clean (no finding)

- Temp file reuse. `> "$tmp"` truncates before every fetch, and a failed
  fetch hits `continue` before anything is hashed. A gh that exits 0 with
  an empty body produces DRIFT, because no local shared file is empty.
- gh non-200. I ran gh against the live API. A 404 exits 1, which gives
  `cannot fetch`. A directory path returns JSON with exit 0, which gives
  DRIFT (fails closed). An out-of-repo symlink or a submodule returns a
  JSON object in the same way, which also gives DRIFT. A renamed repo
  answers 301, which gh follows to the real content. The raw media type
  serves files up to 100 MB, and the shared files are KB-sized.
- Default branch. Contents API calls without a ref read the repo's default
  branch, which is exactly what the spec asks for. All five repos are
  PUBLIC and not archived (`gh repo view`): main/master as recorded.
- `permissions: contents: read` with `GH_TOKEN: github.token` can read
  other public repos. Precedent: toolkit's `gui-pin-drift-check.yml` uses
  the same permissions and `secrets.GITHUB_TOKEN` to run
  `gh release list -R bg002h/mnemonic-gui`, and its scheduled run
  succeeded on 2026-10-23, 24 and 25. 25 calls per day is far under the
  token's rate limit.
- Workflow triggers. Comparison runs on schedule and dispatch, and the
  self-test runs on all three events. The `pull_request` `paths:` filter
  covers the three T2 files. Nothing is required, so there is no
  interaction with existing required checks. `actions/checkout@v4`
  matches 14 existing uses.
- Test rows D1-D6. Each row has a mutant that turns it red: D2, D3, D5 and
  D6 via the controller's mutants; D4 via an "empty body treated as ok"
  mutant; D1 via the 25-line count. `shared-drift.test.sh` ran
  `6 passed, 0 failed` here (TMPDIR under scratch).
- PR description. It says it changes `.github/workflows/` and `ci/`, cites
  the spec and the reference copy, and ends with the footer.
