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
| `pull_request` | `github.event.pull_request.base.sha` |
| `push` to the default branch | `github.event.before` (the previous tip) |
| `push` to any other branch (`ci/staging`) | the current tip of `origin/<default branch>` |
| anything else (tags, dispatch, schedule) | none: full run |

A push to `ci/staging` is compared with the default branch, never with its
own previous tip, because that tip may never have passed. That closes the
stale-base hole in engrave's current detector (recon H7). The comparison is
tree to tree, so a depth-1 checkout is enough.

The assumption it rests on: the default branch's tip passed the required
checks. Branch protection gives that, except for an admin push that bypasses
it, which the operator does knowingly.

## Files, machine-verified

All in this directory (`design/ci-docs-skip/` on toolkit branch
`design/ci-docs-skip`):

- `docs-only.sh`: the guard. In every repo it is copied byte for byte to
  `ci/docs-only.sh`, mode 100755.
- `docs-only.test.sh`: 28 scenarios against a scratch `file://` origin.
  Copied byte for byte to `ci/docs-only.test.sh`, mode 100755. Run here: 28
  passed. Four mutations of the guard (deny rules ignored, staging compared
  with its own previous tip, a bad pattern read as no-match, an empty diff
  read as docs) each turned at least one scenario RED.
- `paths/<repo>.txt`: each repo's `ci/docs-only-paths.txt`, copied byte for
  byte. The protected (`-`) entries are the recon's item-3 paths. They were
  checked again here: every `design/`, `docs/`, `bip/` or root `*.md` path
  that a string literal in the repos' `.rs`/`.go` files names is protected,
  or is not a path read (a label or a message). `paths-check` (below) runs
  the real guard on 5–13 sample paths per repo. All six repos: GREEN.
- `docs-skip-gate.py`: the gate. Its self-test applies the plan to the
  toolkit, engrave and key bases (GREEN), then applies five mutations to each
  (all RED).

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
git -C <wt> add ci/docs-only.sh ci/docs-only.test.sh ci/docs-only-paths.txt
git -C <wt> update-index --chmod=+x ci/docs-only.sh ci/docs-only.test.sh
```

### D1 — guard every listed job (four repos)

The **guarded jobs** are:

| Repo | File | Jobs |
|---|---|---|
| descriptor-mnemonic | `ci.yml` | every job (6) |
| mnemonic-toolkit | `rust.yml` | every job (9) |
| mnemonic-key | `ci.yml` | every job except `release-on-tag` (it has a job-level `if:`): 5 |
| mnemonic-secret | `rust.yml` | every job (11) |

In each guarded job:

1. Directly below the job's first step (it is `actions/checkout` in every
   one; if not, STOP), insert the guard step:

   ```yaml
         - name: Docs-only change? (skips the steps below when every changed file is documentation)
           id: docs
           shell: bash
           env:
             DEFAULT_BRANCH: ${{ github.event.repository.default_branch }}
             PR_BASE_SHA: ${{ github.event.pull_request.base.sha }}
             PUSH_BEFORE: ${{ github.event.before }}
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
   `if:`, add, as the step's line directly below its `- name:` (or
   `- uses:` when it has no name):

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

## The gate

```sh
G=/scratch/code/shibboleth/tk-worktrees/ci-docs-skip/design/ci-docs-skip/docs-skip-gate.py
python3 $G files       <repo> <wt>
python3 $G paths-check <repo> <wt>
python3 $G workflows   <repo> <wt> <B>
# engrave, last, after all five generated repos are edited:
python3 $G release-gen <engrave-wt> descriptor-mnemonic=<dm-wt> mnemonic-secret=<ms-wt> mnemonic-key=<mk-wt> mnemonic-transaction=<mt-wt> mnemonic-toolkit=<tk-wt>
```

`workflows` builds the expected parsed YAML from `<B>` with D1–D4 applied
and asserts equality, rejecting repeated keys. It also asserts the counts
below, and that `git diff` removed exactly the lines D2 and D3 replace and
nothing else. Expected counts (machine-computed from the pre-hardening bases;
hardening adds no steps to these jobs):

| Repo | guarded jobs | steps given the condition | of which combined with an existing `if` | self-test steps |
|---|---|---|---|---|
| descriptor-mnemonic | 7 | 30 | 2 | 2 |
| mnemonic-toolkit | 10 | 40 | 3 | 2 |
| mnemonic-key | 6 | 28 | 2 | 2 |
| mnemonic-secret | 12 | 37 | 2 | 2 |
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
   `release.yml` part of D4. Run the gate (`files`, `paths-check`,
   `workflows`) and actionlint, then make one commit.
2. mnemonic-engrave last: F, D3, D4 (gen.py and emit.py), then `files`,
   `paths-check`, `workflows` and `release-gen`, then one commit.

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

For each of the six repos, from a fresh clone of the new default tip:
1. Create a commit that adds one line to `design/docs-only-probe.md` (new
   file). Push it to a new branch `ci/docs-only-probe`.
2. Wait for the push run. In every guarded job: the log shows
   `docs-only: all 1 changed files are documentation`, every later step shows
   as skipped, and the job succeeds. Record each required job's duration.
3. Delete the remote branch (`git push origin --delete ci/docs-only-probe`).
   This deletes a branch, so it needs the owner's go-ahead. Ask, naming the
   six branches. The commit never lands on the default branch.

## Stop-and-ask points

A guarded job whose first step is not a checkout; a later step with an `if:`
not in D1.3's list; a gate RED that is not your own slip; a count that
differs; a new actionlint line; a failing CI job; any base that does not yet
contain the hardening merge.
