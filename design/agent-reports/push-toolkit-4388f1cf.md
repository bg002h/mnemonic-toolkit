# Push attempt: mnemonic-toolkit master @ 4388f1cf — ABORTED

## Task
Push `master` of `/scratch/code/shibboleth/mnemonic-toolkit` to `origin`,
respecting branch protection, using the `ci/staging` staging ritual if
required contexts exist.

## Precondition check (step 1)
- `git status --short | grep -v '^??'` — empty (no modified tracked files).
- Untracked `cycle-prep-recon-*.md` / `design/*` / `docs/manual-gui/design/*`
  files present, left untouched as instructed.
- `git log --oneline origin/master..master` (using the **locally cached**
  `origin/master` ref, no fetch yet) — 2 commits, tip `4388f1cf`:
  - `4388f1cf` docs(manual): mirror mk encode --from-md1-set / --keys / --in / --out
  - `ab838d98` docs(manual): ms hashlock chapter section + inspect's preimage kind
  - Both confirmed via `git show --stat` to touch only `docs/manual/**`
    (`.cspell.json`, `43-ms.md`, `44-mk-cli.md`, `tests/cli-subcommands.list`)
    — docs/manual only, as expected.
- This step's literal checks all passed.

## Branch protection discovered (step 2)
```
{"contexts":["examples","test (ubuntu-latest)","clippy"],"strict":false,"enforce_admins":false}
```
Required contexts: `examples`, `test (ubuntu-latest)`, `clippy`.
`.github/workflows/rust.yml` (produces `test (ubuntu-latest)`, `clippy`) and
`.github/workflows/examples.yml` (produces `examples`) both trigger on
`branches: [master, main, 'ci/**']` — the `ci/**` staging ritual applies.

## Staging push and the divergence found
Pushed `git push origin master:refs/heads/ci/staging` — succeeded (new branch
`ci/staging` = `4388f1cf`).

While waiting for CI, only a `sibling-pin-check` run appeared for the commit
(concluded `failure`; not a required context, so not gating) — `rust`/`examples`
never triggered. Investigating turned up the real problem: **an explicit
`git fetch origin` showed `origin/master` at `d39d9626`, which is NOT an
ancestor relationship with local `master` — the two have DIVERGED**, not
simply "master is ahead":

```
git log --oneline origin/master..master   (after fetch)
  4388f1cf docs(manual): mirror mk encode ...
  ab838d98 docs(manual): ms hashlock chapter section ...

git log --oneline master..origin/master   (after fetch)
  d39d9626 ci: extend F-324 git_source stanza to cc-validate.sh and remap-off-negative.sh  (2026-09-02 04:07:18 -0700)
  21b6696e ci: add generic git_source_url/git_source_rev inputs to reproducible-musl-build.yml (F-324)  (2026-09-02 04:01:12 -0700)

git merge-base --is-ancestor origin/master master  ->  NOT an ancestor (diverged)
merge-base(master, origin/master) = d8f06483b82a4c0f0b8da86aeb50d31942c31481
```

Local `master`'s two docs commits (dated 2026-09-04) were built on a stale
base that predates `21b6696e`/`d39d9626` (dated 2026-09-02) — commits that are
genuinely present on `origin/master` and absent from local history. This
predates this task; nothing in this session created it. It also explains why
`rust.yml`/`examples.yml` never ran on the `ci/staging` push: that branch was
itself built on the stale base and would not have been a faithful test of the
real tip regardless.

A plain `git push origin master` at this point would be rejected by git
as non-fast-forward (safe — no data-loss risk), but per the task's explicit
"Otherwise STOP; push nothing" instruction, and since the actual precondition
("2 unpushed commits" implicitly assuming no other divergence) does not hold,
**no push of `master` was attempted.**

## Cleanup performed
- `git push origin --delete ci/staging` — deleted (it was built on the stale
  base and represented nothing worth keeping).
- `git fetch origin` — confirms `origin/master` = `d39d9626` (unchanged by
  this session), staging ref absent (`git ls-remote origin refs/heads/ci/staging`
  returns nothing).

## What was NOT done
- `master` was never pushed to `origin/master`.
- No merge, rebase, or reconciliation of the divergence was attempted (out of
  scope for this task; needs an operator/controller decision on whether to
  rebase the two local docs commits onto current `origin/master` or handle
  otherwise).
- No other repo touched; no `.jsonl` read; nothing committed.

## Bypass check
N/A — master was never pushed.

## origin/master after
`d39d96269ce352270189c11fabebf9ad070362b4` (unchanged from before this
session's actions; local `master` remains at `4388f1cf6e5c57948ddff0a0a4bc50fcc7fb42ec`,
2 commits not on `origin/master` and diverged from it by 2 commits the other way).

## Verdict
**FAILURE** — push not performed. Local `master` and `origin/master` have
diverged (2 commits each way); this must be reconciled (rebase/merge) before
a push can proceed. Root cause is a stale local base predating this session,
not an error introduced by this push attempt.
