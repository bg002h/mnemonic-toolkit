# Plan: skip build and test steps for documentation-only changes (2026-10-11)

Owner go-ahead (2026-10-11, in session): asked "is there a way to make CI not
run when the PR is just documentation or plans or any non-compile code?",
then "yes" to a follow-up plan. On the scope question the owner chose:
"Five, skip gui" (descriptor-mnemonic, mnemonic-toolkit, mnemonic-key,
mnemonic-secret, mnemonic-engrave). On the guard question: "One script,
copied". mnemonic-transaction is also touched, but only because its
`release.yml` comes from the same generator (D4). mnemonic-gui is out of
scope.

**Precondition: the CI hardening PRs (design/ci-hardening/) are merged in all
seven repos.** This plan edits the same files. Every base below is the
default branch **after** those merges. Until then, do not start.

## Why (measured)

- Docs mostly land by a staging push (`ci/staging`, then the default branch),
  not by PR. Of the last 40 commits on each default branch, 27 (key), 31
  (engrave), 6 (dm) and 2 (secret) touched only documentation. Among merged
  PRs, 35 of 149 were documentation-only (design/agent-reports/ci-docs-skip-recon.md, items 4 and 7).
- A documentation-only staging push today waits for the full required
  checks: 5–7 min in key, ~10 min in the toolkit (its `test (macos-latest)`).
  The toolkit's `rust.yml` path-filters `push`, so a docs-only staging push
  there gets no `test`/`clippy` context at all, and the push needs the admin
  bypass.
- After this change, a documentation-only change still produces every
  required context. Each guarded job runs a checkout, the guard and (in one
  job per repo) the guard's self-test, then skips its remaining steps and
  reports success.

## The safety rule

The guard writes `docs_only=true` only when it can prove that **every file
that differs from a reference commit, whose tree already passed the required
checks,** is documentation as that repo's `ci/docs-only-paths.txt` defines
it. Any doubt writes `docs_only=false`. The guarded steps carry
`if: steps.docs.outputs.docs_only != 'true'`, so they also run when the guard
writes nothing. The reference commit is:

| Event | Reference commit |
|---|---|
| `pull_request` into the default branch | `github.event.pull_request.base.sha` |
| `pull_request` into any other branch | none: full run |
| `push` to the default branch | `github.event.before` (the previous tip) |
| `push` to any other branch (`ci/staging`) | the current tip of `origin/<default branch>` |
| anything else (tags, dispatch, schedule) | none: full run |

A push to `ci/staging` is compared with the default branch, never with its
own previous tip, because that tip may never have passed. That closes the
stale-base hole in engrave's current detector (recon H7). The comparison is
tree to tree, so a depth-1 checkout is enough.

The reference is **verified, not assumed** (owner ruling on review r1 C1,
"Verify the reference"). Branch protection is `strict: false`, so a PR
merged on a moved base, or an admin push, can put a tree that never passed
at the default tip. So, only after every changed file classifies as
documentation, the guard asks the GitHub API for this job's own check run on
the reference (`CHECK_NAME`, the job's rendered name; `filter=latest`) and
writes `docs_only=true` only if there is at least one such run and every one
is `completed` with conclusion `success`. A red, pending, skipped or missing
run, or any API error, means a full run. A reference that was itself
docs-skipped green is fine: its tree differs from a tested one only in
documentation. Any changed path that contains a newline also means a full
run.

## Files, machine-verified

All in this directory (`design/ci-docs-skip/` on toolkit branch
`design/ci-docs-skip`):

- `docs-only.sh`: the guard. In every repo it is copied byte for byte to
  `ci/docs-only.sh`, mode 100755.
- `docs-only.test.sh`: 42 scenarios against a scratch `file://` origin, with
  `gh` replaced by a stub (`DOCS_ONLY_GH`) that prints canned check runs.
  Copied byte for byte to `ci/docs-only.test.sh`, mode 100755. Run here: 42
  passed. Nine mutations of the guard each turned at least one scenario RED:
  deny rules ignored, staging compared with its own previous tip, a bad
  pattern read as no-match, an empty diff read as docs, no reference check,
  any conclusion accepted, no CR strip, no base-branch check, no newline check.
- `paths/<repo>.txt`: each repo's `ci/docs-only-paths.txt`, copied byte for
  byte. The protected (`-`) entries are the recon's item-3 paths. They were
  checked again here: every `design/`, `docs/`, `bip/` or root `*.md` path
  that a string literal in the repos' `.rs`/`.go` files names is protected,
  or is not a path read (a label or a message). `paths-check` (below) runs
  the real guard on 5–13 sample paths per repo. All six repos: GREEN.
- `docs-skip-gate.py`: the gate. Its self-test applies the plan to the
  toolkit, engrave and key bases by dumping the gate's own expected YAML
  (GREEN), then applies eleven mutations to each (all RED). It also applies
  the plan **textually**, line by line, to the descriptor-mnemonic base
  (GREEN, with the real removed-lines check), then deletes an unrelated
  comment line and adds a repeated `if:` key (both RED). The dumped cases
  prove the checks fire, not that the gate's expected YAML encodes the plan;
  the textual case is the one cross-check of that.
- Check-run names: every guarded job's check run is named by its `name:`
  exactly as rendered, except toolkit `lib-cross-platform`, whose `name:`
  names no matrix value, so GitHub appends them:
  `lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest)`.
  Read from the check-runs API on the six default tips and on toolkit PR 102's
  head, 2026-10-11.

## Repos, bases, worktrees, branch

| Repo | Base (after hardening merge) | Worktree (create) |
|---|---|---|
| descriptor-mnemonic | `origin/main` | `/scratch/code/shibboleth/dm-worktrees/ci-docs-skip` |
| mnemonic-toolkit | `origin/master` | `/scratch/code/shibboleth/tk-worktrees/ci-docs-skip-impl` |
| mnemonic-key | `origin/main` | `/scratch/code/shibboleth/mk-worktrees/ci-docs-skip` |
| mnemonic-secret | `origin/master` | `/scratch/code/shibboleth/ms-worktrees/ci-docs-skip` |
| mnemonic-engrave | `origin/master` | `/scratch/code/shibboleth/me-worktrees/ci-docs-skip` |
| mnemonic-transaction | `origin/main` | `/scratch/code/shibboleth/mt-worktrees/ci-docs-skip` |

In each repo: `git fetch origin`, then `git worktree add -b ci/docs-skip
<path> <base>`. Record `B=$(git -C <path> rev-parse HEAD)` per repo and use
that SHA as `<base>` for the gate. The design files stay on toolkit branch
`design/ci-docs-skip` (worktree `/scratch/code/shibboleth/tk-worktrees/ci-docs-skip`).
Copy from there. That copy of `docs-skip-gate.py` is the only authoritative
gate. Every change is a new commit: never amend, rebase or force-push.
Stage paths explicitly.

## The edits

### F — the guard files (all six repos)

```sh
S=/scratch/code/shibboleth/tk-worktrees/ci-docs-skip/design/ci-docs-skip
mkdir -p <wt>/ci
cp $S/docs-only.sh $S/docs-only.test.sh <wt>/ci/
cp $S/paths/<repo>.txt <wt>/ci/docs-only-paths.txt
chmod 755 <wt>/ci/docs-only.sh <wt>/ci/docs-only.test.sh
cat >> <wt>/.gitattributes <<'ATTR'
# ci/docs-only*: LF on every OS (Git Bash cannot run a CRLF script).
ci/docs-only.sh text eol=lf
ci/docs-only.test.sh text eol=lf
ci/docs-only-paths.txt text eol=lf
ATTR
git -C <wt> add .gitattributes ci/docs-only.sh ci/docs-only.test.sh ci/docs-only-paths.txt
git -C <wt> update-index --chmod=+x ci/docs-only.sh ci/docs-only.test.sh
```

The `cat >>` creates `.gitattributes` in engrave and mnemonic-transaction,
which have none. In the other four it appends after the file's last line;
each of those ends in a newline (checked). Windows checkouts would otherwise
CRLF-convert the three files, and Git Bash cannot run a CRLF script.

### D1 — guard every listed job (four repos)

The **guarded jobs** are:

| Repo | File | Jobs |
|---|---|---|
| descriptor-mnemonic | `ci.yml` | every job (6) |
| mnemonic-toolkit | `rust.yml` | every job except `g6-invariant`: 8 |
| mnemonic-key | `ci.yml` | every job except `release-on-tag` (it has a job-level `if:`): 5 |
| mnemonic-secret | `rust.yml` | every job except `g6-invariant`: 10 |

`g6-invariant` (toolkit and secret) is left alone: it checks out into a
subdirectory, so `ci/docs-only.sh` is not at the path the guard step runs.

In each guarded job:

1. Directly below the last line of the job's first step (it is
   `actions/checkout` in every one; if not, STOP), insert the guard step.
   `<NAME>` is the job's `name:` value, copied exactly as written on its
   `name:` line, except toolkit `lib-cross-platform`, where it is
   `lib cross-platform check (${{ matrix.target }}, ${{ matrix.runs-on }})`:

   ```yaml
         - name: Docs-only change? (skips the steps below when every changed file is documentation)
           id: docs
           shell: bash
           env:
             DEFAULT_BRANCH: ${{ github.event.repository.default_branch }}
             PR_BASE_SHA: ${{ github.event.pull_request.base.sha }}
             PUSH_BEFORE: ${{ github.event.before }}
             CHECK_NAME: <NAME>
             GH_TOKEN: ${{ github.token }}
           run: bash ci/docs-only.sh
   ```

2. In exactly one job per repo, the **self-test job** (descriptor-mnemonic
   `clippy`, mnemonic-toolkit `clippy`, mnemonic-key `fmt`, mnemonic-secret
   `clippy`), insert this step between the checkout and the guard step:

   ```yaml
         - name: Self-test the docs-only guard
           shell: bash
           run: bash ci/docs-only.test.sh
   ```

3. Give every step **after** the guard step the condition. If the step has no
   `if:`, add it as the line directly below the step's first line, the one
   that starts with `- ` (`- name:`, `- uses:` or `- run:`; every `- run:`
   step in these files is a single line):

   ```yaml
           if: steps.docs.outputs.docs_only != 'true'
   ```

   If the step already has `if: <E>`, replace that line with
   `if: (<E'>) && steps.docs.outputs.docs_only != 'true'`, where `<E'>` is
   `<E>` with a surrounding `${{` … `}}` removed. The nine such steps:
   - descriptor-mnemonic `musl-check`: `${{ matrix.musl_native }}` → `(matrix.musl_native) && …`, and `${{ matrix.arm_native }}` → `(matrix.arm_native) && …`
   - mnemonic-toolkit `lib-cross-platform`: `matrix.target == 'aarch64-unknown-linux-gnu'` → `(matrix.target == 'aarch64-unknown-linux-gnu') && …`; `musl-build-test`: the same two matrix flags.
   - mnemonic-key `musl-check` and mnemonic-secret `musl-check`: the same two matrix flags.

   Do not touch the checkout step, the self-test step or the guard step.

### D2 — mnemonic-toolkit `rust.yml` push trigger

Delete the `paths:` key under `on.push` and its whole list: the 16 lines from
`    paths:` through `      - 'scripts/install-*.test.sh'`. Without that
filter a docs-only staging push could not earn `test`/`clippy`. The guard now
does the filter's job, and does it safely.

Replace these three comment lines in the `push:` comment block:

```
    # agents, not the human. The `paths:` filter below is UNCHANGED and
    # deliberate: a staged commit must touch a gated path to build, which every
    # code change does.
```

with:

```
    # agents, not the human. There is no `paths:` filter: every job runs and
    # reports, and ci/docs-only.sh skips the build and test steps when every
    # changed file is documentation (ci/docs-only-paths.txt).
```

and these three lines in the `pull_request:` comment:

```
    # green (unchanged code) — the cost is CI minutes, not correctness. The
    # push-side path filter above keeps code-push efficiency; docs-only DIRECT
    # pushes to master are covered by admin bypass (enforce_admins:false).
```

with:

```
    # green (unchanged code). ci/docs-only.sh now skips the build and test
    # steps of a documentation-only change on push and pull_request alike.
```

(The em dash in the first replaced line is U+2014, as in the file.)

### D3 — mnemonic-engrave `release.yml`

- Job `test`: replace the whole step `- name: docs-only detection` (from
  that line through the last line of its `run: |` block, keeping the blank
  line after it) with the self-test step from D1.2 followed by the guard step
  from D1.1. Then, in every later step of the job, replace
  `steps.detect.outputs.docs_only` with `steps.docs.outputs.docs_only`: the
  13 `!= 'true'` conditions and the one `== 'true'` condition of
  `docs-only verdict`. Do not give `SH2 demo install links resolve and are
  current` a condition: it stays unconditional, as its comment says. The
  `signing key copies agree with minisign.pub` step stays before the
  self-test, unconditional.
- Job `picotool-probe`: apply D1.1 and D1.3 (no self-test).
- `<NAME>` is `test (rust + go)` in `test` and `picotool argv probe` in
  `picotool-probe`.
- In both jobs, add the line `      checks: read` directly below the job's
  `      contents: read` line (D5).
- No other job changes.

### D4 — the release-workflow generator, and the five generated `release.yml`

In `mnemonic-engrave/scripts/release-workflows/gen.py`, job `test` of the
template:

- Directly below the `test` job's checkout step, insert the block below.
  The checkout's last line is
  `          ref: ${{{{ github.event.inputs.tag || github.ref }}}}`. That line
  occurs three times in gen.py (jobs `test`, `build`, `assemble`); use the
  one inside `test`, the first. Each of the four lines named next also
  occurs exactly once inside `test`; edit only that occurrence.

  ```
        - name: Self-test the docs-only guard
          if: runner.os == 'Linux'
          shell: bash
          run: bash ci/docs-only.test.sh
        - name: Docs-only change? (skips the steps below when every changed file is documentation)
          id: docs
          shell: bash
          env:
            DEFAULT_BRANCH: ${{{{ github.event.repository.default_branch }}}}
            PR_BASE_SHA: ${{{{ github.event.pull_request.base.sha }}}}
            PUSH_BEFORE: ${{{{ github.event.before }}}}
            CHECK_NAME: test (${{{{ matrix.os }}}})
            GH_TOKEN: ${{{{ github.token }}}}
          run: bash ci/docs-only.sh
  ```

  The `{{{{`/`}}}}` are the f-string escapes the template already uses; they
  render as `${{ … }}`.
- Add the line `        if: steps.docs.outputs.docs_only != 'true'`
  directly below each of these template lines:
  `      - uses: dtolnay/rust-toolchain@master`,
  `      - name: Install cargo-nextest (pinned, checksum-verified)`,
  `      - name: cargo nextest run`,
  `      - name: cargo test --doc`.
- Directly above the `test` job's `    strategy:` line (the first of the
  template's `    strategy:` lines; edit only the one inside `test`), insert:

  ```
      permissions:
        contents: read
        checks: read
  ```

  The workflow's top-level permissions (`contents: write`, `id-token: write`,
  `attestations: write`) then no longer reach `test`, which needs none of them.

In `mnemonic-engrave/scripts/release-workflows/emit.py`, `TEST_SETUP["mt"]`:
add the line `        if: steps.docs.outputs.docs_only != 'true'` directly
below `      - name: Install the shells the history-purge tests drive`.

(Build-gated here: these gen.py and emit.py edits, applied to scratch
copies, make the generator's output parse equal to the gate's expected
`release.yml` in all five repos.)

Then make the **same** text change by hand in the `test` job of
`release.yml` in descriptor-mnemonic, mnemonic-toolkit, mnemonic-key,
mnemonic-secret and mnemonic-transaction (with `${{`/`}}` in place of the
escapes). The gate's `release-gen` mode proves the six agree. Do not run
`emit.py`: it writes into the main checkouts.

### D5 — let the guard read check runs

The guard step's `GH_TOKEN` needs `checks: read`. Directly above the line
`jobs:` in descriptor-mnemonic `ci.yml`, mnemonic-toolkit `rust.yml`,
mnemonic-key `ci.yml` and mnemonic-secret `rust.yml` (none has a top-level
`permissions:`; if one does, STOP), insert these four lines, the last one
blank:

```yaml
permissions:
  contents: read
  checks: read

```

mnemonic-key `release-on-tag` keeps its own job-level `contents: write`. The
release.yml `test` jobs get theirs in D4, and engrave's two jobs in D3.

## The gate

```sh
G=/scratch/code/shibboleth/tk-worktrees/ci-docs-skip/design/ci-docs-skip/docs-skip-gate.py
python3 $G files       <repo> <wt>
python3 $G paths-check <repo> <wt>
python3 $G workflows   <repo> <wt> <B>
# engrave, last, after all five generated repos are edited:
python3 $G release-gen <engrave-wt> descriptor-mnemonic=<dm-wt> mnemonic-secret=<ms-wt> mnemonic-key=<mk-wt> mnemonic-transaction=<mt-wt> mnemonic-toolkit=<tk-wt>
```

`workflows` builds the expected parsed YAML from `<B>` with D1–D5 applied
and asserts equality, rejecting repeated keys. It also asserts the counts
below; that `git diff` removed exactly the lines D1.3, D2 and D3 replace and
nothing else; that `.gitattributes` gained exactly F's four lines; and that
the files differing from `<B>` are exactly the planned set, which it prints.
`workflows`, `files` and `release-gen` refuse a worktree with unstaged
changes or untracked files, so they certify exactly what the commit holds. Expected counts (machine-computed from the pre-hardening bases;
hardening adds no steps to these jobs):

| Repo | guarded jobs | steps given the condition | of which combined with an existing `if` | self-test steps |
|---|---|---|---|---|
| descriptor-mnemonic | 7 | 30 | 2 | 2 |
| mnemonic-toolkit | 9 | 37 | 3 | 2 |
| mnemonic-key | 6 | 28 | 2 | 2 |
| mnemonic-secret | 11 | 35 | 2 | 2 |
| mnemonic-engrave | 2 | 18 | 0 | 1 |
| mnemonic-transaction | 1 | 5 | 0 | 1 |

(Each generated `release.yml` adds one guarded job and one Linux-only
self-test to its repo's numbers.)

NOT COVERED by the gate: added comment text, blank lines, quoting, key order
and YAML 1.1 spellings. Read `git diff` for those. Also run actionlint before
and after; the after-output may have no new line. It does not prove the guard
works on GitHub's runners. The PR's own CI runs the guard on every OS leg
(Linux, macOS, Windows): a code change, so it must print `full run` and run
everything. The post-merge probe below proves the skip.

## Work order, commits, PRs

1. descriptor-mnemonic, mnemonic-key, mnemonic-secret, mnemonic-toolkit,
   mnemonic-transaction, each: F, then D1 (and D2 for the toolkit), then the
   `release.yml` part of D4, then D5. Stage every edited path explicitly.
   Run the gate (`files`, `paths-check`, `workflows`) and actionlint, then
   make one commit.
2. mnemonic-engrave last: F, D3, D4 (gen.py and emit.py), then stage, then
   `files`, `paths-check`, `workflows` and `release-gen`, then one commit.

After each commit, `git status --porcelain` must print nothing, and
`git diff --name-only <B> HEAD` must list exactly the files the gate's
`changed files` line printed. Otherwise STOP.

Commit message:

```
ci: skip build and test steps when a change is documentation only

Per design/ci-docs-skip/PLAN_ci_docs_skip_2026-10-11.md (mnemonic-toolkit).
ci/docs-only.sh writes docs_only=true only when every file that differs
from an already-checked reference commit matches ci/docs-only-paths.txt;
every guarded step runs unless it does.
<one line per D-item applied, with the gate's counts>

Gate: <the files, paths-check and workflows GATE lines; engrave also release-gen>
actionlint: no new findings
```

ending with the session's attribution lines. Push `ci/docs-skip` and open a
**draft** PR titled `ci: skip build and test steps for documentation-only
changes`. The body states what changed, the safety rule table, the gate
output, and links this plan. Wait for CI on each PR. Every required check
must pass, and each guarded job's log must show
`docs-only: full run (not documentation: …)`. A failing job is a stop point.
Do not merge.

## After the owner merges: the probe

For each of the six repos, from a fresh clone of the new default tip R:
0. Wait until every workflow run on R has completed. Then list R's check runs:
   `gh api -X GET repos/bg002h/<repo>/commits/<R>/check-runs -f filter=latest -f per_page=100 --jq '.check_runs[] | "\(.name) \(.status)/\(.conclusion)"'`.
   Note every guarded job whose run on R is not `completed/success`. Do not
   re-run anything.
1. Create a commit that adds one line to `design/docs-only-probe.md` (new
   file). Push it to a new branch `ci/docs-only-probe`.
2. Wait for the push run. In every guarded job not noted in step 0: the log
   shows `docs-only: all 1 changed files are documentation`, every later step
   shows as skipped (except, in engrave `test`, `SH2 demo install links
   resolve and are current` and `docs-only verdict`, which run), and the job
   succeeds. A job noted in step 0 must instead log
   `docs-only: full run ('<NAME>' on <R> is …, not completed/success)`; that
   is the guard working, not a probe failure. Record each required job's
   duration. Anything else is a stop point.
3. Delete the remote branch (`git push origin --delete ci/docs-only-probe`).
   This deletes a branch, so it needs the owner's go-ahead. Ask, naming the
   six branches. The commit never lands on the default branch.

## Stop-and-ask points

A guarded job whose first step is not a checkout; a later step with an `if:`
not in D1.3's list; a workflow D5 names that already has a top-level
`permissions:`; a gate RED that is not your own slip; a count that
differs; a new actionlint line; a failing CI job; any base that does not yet
contain the hardening merge.
