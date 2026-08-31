# Push — mdcli-mini docs catch-up to mnemonic-toolkit master (2026-08-31)

Closing the toolkit-docs loop for the mdcli-mini wallet-form-converter cycle.
Source: worktree `mnemonic-toolkit-mddocs`, branch `mdcli-mini-docs` at
`fa576b13a34b8e6f6ac38966d2932d9b3dc2edbd` (2 commits: `95e3723d` the docs
content, `fa576b13` its persisted pass report
`design/agent-reports/DOCS-mdcli-mini-toolkit-pass.md`).

## Merged

Main toolkit checkout was clean at tip `2ecb0010454d91f9dda1c428bec744c27cce38f5`
("build: opt-level 2 on the test and dev profiles"). `fa576b13` was a
descendant with merge-base `2ecb0010` == master's own tip, so
`git merge --ff-only fa576b13a34b8e6f6ac38966d2932d9b3dc2edbd` fast-forwarded
cleanly: `2ecb0010..fa576b13`, 3 files changed (`42-md.md`,
`cli-subcommands.list`, the new report).

## CI-redness assessment (manual.yml / md-cli v0.11.2 pin)

`.github/workflows/manual.yml`'s `build` job installs `md-cli` from the
released tag `descriptor-mnemonic-md-cli-v0.11.2` (line 86:
`cargo install --git https://github.com/bg002h/descriptor-mnemonic --tag
descriptor-mnemonic-md-cli-v0.11.2 md-cli --features cli-compiler`), which
predates the mdcli-mini cycle and lacks `md descriptor` / `md decompose`.
The docs commit enrolled both in
`docs/manual/tests/cli-subcommands.list`, so CI's flag-coverage step
(`docs/manual/tests/lint.sh` step 4/6, invoked via `make audit` → `make lint`
→ `bash tests/lint.sh`) now runs `<old-md-bin> descriptor --help` and
`<old-md-bin> decompose --help` against that stale binary.

**Empirically built the actual v0.11.2 binary** (git worktree at the local
`descriptor-mnemonic` clone's `descriptor-mnemonic-md-cli-v0.11.2` tag,
`cargo build -p md-cli --features cli-compiler`, matching CI's install
features) rather than inferring behavior. Findings:

- `md descriptor --help` and `md decompose --help` both exit 2 with clap's
  unrecognized-subcommand error (`error: unrecognized subcommand
  'descriptor'` / `'decompose'`, `tip: a similar subcommand exists...`,
  `Usage: md <COMMAND>`, `For more information, try '--help'.`).
- **The persisted docs-pass report's stated mechanism was inaccurate.** It
  claimed this degrades to `lint.sh`'s "no flags parsed; skipping" WARN
  branch (line 85: `if [ -z "$flags" ]`) because the error text has "no `--`
  tokens to extract." That is false: the error text's closing line contains
  the substring `--help` (from `try '--help'`), and `lint.sh` line 84's
  regex (`grep -oE -- '--[a-z][a-z0-9-]+'`) extracts exactly that —
  confirmed by literally running `lint.sh`'s line-84 command against the
  built v0.11.2 binary: `flags parsed: [--help]` for both subcommands, not
  empty.
  - **The bottom-line conclusion (no hard failure) is still correct**, but
    via a different code path: `--help` is already documented in
    `42-md.md` (line 9: "Every subcommand below accepts `--help` (`-h`) for
    inline help.", plus two flag-table rows), so `lint.sh`'s per-flag
    existence check (line 93: `grep -qF -- "$flag" "$chapter"`) finds it and
    the loop emits no `err`. Net effect matches the report's conclusion,
    reached for the wrong reason.
- Ran the **actual `make audit`** gate (lint + verify-examples +
  anchor-check — everything CI's `build` job's "Audit manual" step runs) in
  the merged worktree with `MD_BIN` pointed at the real v0.11.2 binary:
  - `bash tests/lint.sh ... MD_BIN=<v0.11.2 binary> MS_BIN=ms MK_BIN=mk` →
    `[lint] OK`, all 6 steps silent/clean.
  - `bash tests/verify-examples.sh ... MD_BIN=<v0.11.2 binary>` — first
    attempt against a stale `~/.cargo/bin/mnemonic` FAILED (transcript
    drift on `mnemonic bundle` grouping format) — a pre-existing local
    environment staleness, not a CI-relevant defect (CI always builds
    `mnemonic` fresh from HEAD per `manual.yml`'s own "Build mnemonic
    binary (debug)" step). Rebuilt `mnemonic` from the merged worktree's
    HEAD (`cargo build --bin mnemonic`) and reran: `[verify-examples] OK
    (62 transcripts pass)`. The docs commit added no new transcript files,
    so `md descriptor`/`decompose` are not exercised by this step at all.
  - `make html MERMAID_FILTER=skip` + `bash tests/anchor-check.sh` →
    `OK anchor-check: 10 danglers in current run match baseline (no new, no
    shrunk)`.
  - **Conclusion: `make audit` (the entirety of what CI's `build` job's
    critical step runs) passes clean against the exact old binary CI
    installs.** This was verified live by opening PR #65 and letting the
    real `manual.yml` workflow run against the real pinned tag — see
    "Pushed" below for its actual conclusion, not just the local
    reproduction.

## A second, unrelated blocker found and routed around

`rust.yml` and `examples.yml` — which produce the branch protection's three
*required* contexts (`test (ubuntu-latest)`, `clippy`, `examples`; confirmed
via `gh api repos/bg002h/mnemonic-toolkit/branches/master/protection`,
`required_status_checks.contexts`) — both gate their `push:` trigger behind
a `paths:` filter (`crates/**`, `Cargo.toml`, `Cargo.lock`,
`.gitattributes`, and their own workflow file) that **excludes
`docs/manual/**` entirely**, on every branch the trigger lists including
`ci/**`. A docs-only diff therefore cannot earn any of the three required
contexts via a direct push or via the repo's own `ci/staging` staging ritual
(described in both workflow files' comments) — that ritual's `paths:` filter
applies identically to `ci/staging`.

Confirmed this is not hypothetical by checking a real historical docs-only
commit already on master, `aa5e1ae5a4fa3a65b49ad0308a6411ae7e5e0f1c`
(`docs(manual): a bare 'ms derive --template bip48' is now accepted`):
`gh api repos/bg002h/mnemonic-toolkit/commits/<sha>/status` returns
`{"state":"pending","total_count":0}` — zero statuses ever posted, of any
kind — and its `check-runs` list contains only one unrelated job
(`sibling pins match install.sh`). That commit is on master today, meaning
it landed via a push that never earned the required contexts.

Given `enforce_admins: false` and `rust.yml`'s own comment ("enforce_admins
is false here DELIBERATELY — the maintainer's own escape hatch ... the
no-bypass rule binds agents, not the human"), a direct `git push origin
master` under these (the operator's own, `bg002h`) credentials would very
likely succeed only via GitHub's admin-bypass path, printing "Bypassed rule
violations" — an outcome the constellation-wide standing rule treats as a
failure to report, not a success, specifically because it binds agents, not
the human.

**Routed around it via a PR instead of a raw push.** `rust.yml`'s and
`examples.yml`'s `pull_request:` triggers deliberately carry **no** `paths:`
filter (their own comments: a path-filtered *required* check would "wedge
forever" on a PR), so opening a PR lets all three required contexts run for
real, plus `manual.yml` itself (its `pull_request:` trigger's paths filter
does include `docs/manual/**`), all merged through GitHub's normal
protected-merge flow with no bypass. This repo has 64 prior merged PRs, so
it is an established convention here, not an improvised one.

## Pushed

- Pushed branch `mdcli-mini-docs` (`fa576b13`) directly to `origin` as a
  **new branch**, not to `master` — clean, no bypass warning printed.
- Opened PR #65: `docs(md): catch up 42-md.md with the mdcli-mini
  wallet-form-converter surface` —
  https://github.com/bg002h/mnemonic-toolkit/pull/65
- Watched all checks via `gh pr checks 65 --repo bg002h/mnemonic-toolkit`,
  judged per-job. Final snapshot (16/17 concluded; 1 non-required job still
  running at merge time, see below):

  ```
  build: pass                                                    (manual.yml — the exact job under assessment)
  clippy: pass                                                   (required)
  examples: pass                                                 (required)
  fmt (pinned 1.95.0): pass
  g6 invariant (cross-repo mlock.rs): pass
  install.sh harnesses (man-step + MSRV guard): pass
  lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest): pass
  lib cross-platform check (x86_64-pc-windows-msvc, windows-latest): pass
  lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest): pass
  miri (mlock unsafe): pass
  musl build+test (aarch64-unknown-linux-musl): pending at merge time (unrelated cross-build; no path touching docs/manual affects it; not a required context)
  musl build+test (x86_64-unknown-linux-musl): pass
  sibling pins match install.sh: pass (x2, PR + push-triggered runs)
  test (macos-latest): pass
  test (release, ubuntu-latest, mlock einval): pass
  test (ubuntu-latest): pass                                     (required)
  ```

  **`build` (manual.yml) passing for real, against the genuine
  `descriptor-mnemonic-md-cli-v0.11.2` install, is the live confirmation of
  the CI-redness assessment above** — not just the local `make audit`
  reproduction. All three required contexts (`test (ubuntu-latest)`,
  `clippy`, `examples`) are green.

- Merged via `gh pr merge 65 --repo bg002h/mnemonic-toolkit --rebase
  --delete-branch=false` (rebase, not squash/merge-commit, to keep the two
  original commits and their messages distinct — closest available
  approximation to `--ff-only` through the PR API, which has no true
  fast-forward merge method). GitHub's rebase-merge preserves commit content
  and order but assigns new SHAs (it always re-applies commits, even when
  the branch is already directly rebase-able onto the base):
  `95e3723d` → `225bb582` (docs content), `fa576b13` → `c8c83623` (report
  commit). Verified `git diff --stat` between the pre-merge local
  fast-forward state (`fa576b13`) and the new `origin/master`
  (`c8c83623`) is **empty** — byte-identical trees, only commit metadata
  differs.
- `gh pr view 65` confirms `state: MERGED`, `mergedAt:
  2026-08-31T06:52:43Z`, `mergeCommit.oid: c8c83623fac7c1bb58b7dbe00e9110bbc2ec7a06`.
- `git ls-remote origin refs/heads/master` → `c8c83623fac7c1bb58b7dbe00e9110bbc2ec7a06`,
  matching. Local master reconciled: `git fetch origin master` then
  `git reset --hard origin/master` (safe — tree-identical to the local
  fast-forward state it replaced, confirmed via the empty `git diff --stat`
  above before resetting).

## Cleanup

- `git worktree remove /scratch/code/shibboleth/mnemonic-toolkit-mddocs` —
  removed cleanly (worktree was clean, no uncommitted changes).
- Local branch `mdcli-mini-docs`: `git branch -d` refused ("not fully
  merged") because git's ancestry check compares commit SHAs, and the
  GitHub rebase-merge produced new SHAs for content git's local check
  doesn't recognize as an ancestor of the old local pointer. Force-deleted
  with `git branch -D mdcli-mini-docs` after independently confirming (the
  empty `git diff --stat` above) the content is fully present on
  `origin/master` — safe, no work lost.
- Remote branch: `git push origin --delete mdcli-mini-docs` — deleted
  cleanly.
