# Plan: CI hardening across seven public repos (2026-10-11)

Owner go-ahead (2026-10-11, in session): "can you handle implementing the other
repos few safety and waste fixes?". The fixes that sentence names were listed
to the owner as: job timeouts, a step timeout on the texlive install, caching
the toolkit's from-source sibling-CLI installs, and cancelling superseded runs.
seedhammer is **out of scope** (owner ruling: it is an upstream fork; leave its
workflows as upstream wrote them). mnemonic-refugium is out of scope (its CI
already has `cancel-in-progress` and runs on self-hosted runners).

## Why (measured, not estimated)

- No job in any of the seven repos sets `timeout-minutes`, so GitHub's default
  of 360 minutes applies. mnemonic-toolkit run 37768820065 (manual-gui, PR from
  `claude/fold-i-manual`, 2026-10-08) spent **15,316 s (4 h 15 m)** in the
  step "Install pandoc + texlive" (an apt hang); that step normally takes
  97–164 s.
- The slowest successful job found is mnemonic-toolkit `rust.yml`
  `musl build+test (aarch64-unknown-linux-musl)` at **24.2 min** (run
  36210086244, 2026-09-26; all 1,287 successful rust.yml jobs in its last 100
  successful runs measured). That job has taken about 5 min since 2026-09-26.
  Across the last 300 runs of each repo, no other successful job took over
  14.0 min. 60 minutes is about 2.5x the slowest job ever seen.
- None of the seven repos cancels a superseded pull-request run. The only
  `concurrency` blocks are the `release.yml` release-serialisation blocks
  (`cancel-in-progress: false`), which this plan does not touch.
- mnemonic-toolkit builds the sibling CLIs (`mk`, `md`, `ms`, and in
  manual-gui also `mnemonic` and `gui-render`) from source in every doc job:
  measured in run 38117798692 (manual): mk 39 s, md 56 s, ms 30 s; run
  38117798600 (manual-gui): CLI tier 122 s + mnemonic 83 s, gui-render 89 s.
- Machine-verified 2026-10-11: with `CARGO_INSTALL_ROOT` pointing at a root
  that already holds the pinned package, a repeated
  `cargo install --git https://github.com/bg002h/mnemonic-secret --tag ms-cli-v0.20.1 ms-cli`
  exits 0 in 0.29 s ("Ignored package ... is already installed"). The first
  install took 23 s on a 24-core box. So restoring a cached install root
  skips the build in each install step; cargo still fetches the sibling's git
  repo, so a hit costs seconds, not zero. A changed tag or feature set still
  reinstalls. A changed rustc does **not** (review r1 I3, reproduced with
  1.85 → 1.88), so the keys hash `rust-toolchain.toml` where the toolchain
  comes from it.

## Repos, base branches, worktrees, branch name

| Repo | Base | Worktree path (create) |
|---|---|---|
| descriptor-mnemonic | `origin/main` | `/scratch/code/shibboleth/dm-worktrees/ci-hardening` |
| mnemonic-toolkit | `origin/master` | `/scratch/code/shibboleth/tk-worktrees/ci-hardening` |
| mnemonic-gui | `origin/master` | `/scratch/code/shibboleth/gui-worktrees/ci-hardening` |
| mnemonic-key | `origin/main` | `/scratch/code/shibboleth/mk-worktrees/ci-hardening` |
| mnemonic-secret | `origin/master` | `/scratch/code/shibboleth/ms-worktrees/ci-hardening` |
| mnemonic-engrave | `origin/master` | `/scratch/code/shibboleth/me-worktrees/ci-hardening` |
| mnemonic-transaction | `origin/main` | `/scratch/code/shibboleth/mt-worktrees/ci-hardening` |

**mnemonic-toolkit already has its worktree and branch**:
`/scratch/code/shibboleth/tk-worktrees/ci-hardening` on
`ci/hardening-2026-10-11`, holding this plan, the gate and the review reports.
Work there. In each of the **other six** repos: `git fetch origin`, then
`git worktree add -b ci/hardening-2026-10-11 <path> <base>`. Never work in a
main checkout. Never under `/tmp`.

Commit policy: every change is a new commit; never amend, rebase or
force-push.

## The edits

Edit only files under `.github/workflows/`, plus
`mnemonic-engrave/scripts/release-workflows/gen.py` (G below). Do not reformat, reorder, re-quote or re-indent anything
else. Do not touch comments except the one named in K4.

### T — job timeout (all seven repos)

For every job that has a `runs-on:` key at job level (the line is
`    runs-on: ...`, exactly four spaces of indent), insert a new line directly
below that `runs-on:` line:

```yaml
    timeout-minutes: 60
```

- Do **not** add it to a job that has `uses:` (a reusable-workflow call; GitHub
  rejects `timeout-minutes` there). These are: descriptor-mnemonic
  `man-pages.yml` job `repro`; mnemonic-toolkit `man-pages.yml` job `repro` and
  `repro-drift.yml` job `drift`; mnemonic-key `musl-binaries.yml` job `repro`;
  mnemonic-secret `man-release.yml` job `repro`.
- Do **not** touch `runs-on:` lines indented more than four spaces. In
  mnemonic-toolkit `rust.yml` the lines at 12 spaces (`            runs-on:
  windows-latest` etc., in the `lib-cross-platform` matrix `include:`) are
  matrix values, not job keys.
- The reusable workflow `mnemonic-toolkit/.github/workflows/reproducible-musl-build.yml`
  is a normal file with normal jobs: its four jobs get T like any other.
  (Callers in other repos pin it by commit, so they keep the old file until
  they bump their pin. That is expected.)
- Every job in the seven repos currently lacks `timeout-minutes`; if you find
  one that already has it, STOP and report.

### X — texlive step timeout (mnemonic-toolkit only)

Every step whose `name:` starts with `Install pandoc + texlive` gets a
step-level line directly below its `- name:` line, indented to the step's key
level (the same indent as the step's `run:`):

```yaml
        timeout-minutes: 15
```

There are exactly six such steps: `examples.yml` (1, named
`Install pandoc + texlive + DejaVu mono`), `manual-gui.yml` (2: jobs `lint`
and `build`), `manual.yml` (1), `quickstart.yml` (1), `technical-manual.yml`
(1). If the count differs, STOP and report.

### C — cancel superseded pull-request runs (all seven repos)

For every workflow file whose triggers include `pull_request` **and** that has
no top-level `concurrency:` key, insert this block directly above the
top-level `jobs:` line, followed by one blank line before `jobs:`:

```yaml
# Cancel a pull request's superseded run when a newer commit is pushed to it.
# Every other event gets a group of its own (the run id), so pushes, tags,
# schedules and dispatches are never cancelled.
concurrency:
  group: ${{ github.workflow }}-${{ github.event_name == 'pull_request' && github.event.pull_request.number || github.run_id }}
  cancel-in-progress: true
```

Copy it byte for byte. The expected files (26):

- descriptor-mnemonic: `bitcoind-differential.yml`, `ci.yml`, `fuzz-smoke.yml`, `vendor-freshness.yml`
- mnemonic-toolkit: `bitcoind-differential.yml`, `cross-tool-differential.yml`, `examples.yml`, `fuzz-smoke.yml`, `gui-pin-drift-check.yml`, `manual-gui.yml`, `manual.yml`, `quickstart.yml`, `rust.yml`, `sibling-pin-check.yml`, `technical-manual.yml`, `vendor-freshness.yml`
- mnemonic-gui: `build.yml`, `schema-mirror.yml`
- mnemonic-key: `ci.yml`, `fuzz-smoke.yml`, `vendor-freshness.yml`
- mnemonic-secret: `fuzz-smoke.yml`, `rust.yml`, `vendor-freshness.yml`
- mnemonic-engrave: `release.yml`
- mnemonic-transaction: `ci.yml`

Do **not** add it to any `release.yml` that already has `concurrency:`
(descriptor-mnemonic, mnemonic-toolkit, mnemonic-key, mnemonic-secret,
mnemonic-transaction). mnemonic-engrave's `release.yml` has none and is in the
list. If a file in the list already has top-level `concurrency:`, or a file
not in the list has a `pull_request` trigger and no `concurrency:`, STOP and
report.

### K — cache the toolkit's sibling-CLI install roots (mnemonic-toolkit only)

Use `actions/cache@v4` (the exact ref already used in `bitcoind-differential.yml`).
Every cache-restore step is skipped on tag refs, so release assets are always
built from fresh installs.

**K1. `manual.yml`, `quickstart.yml`, `technical-manual.yml`** (one job each:
`manual`, `quickstart`, `technical-manual`). In each, insert these two steps,
in this order, with one blank line between steps as the file already does.
In `manual.yml` and `quickstart.yml` they go directly above the line
`      - name: Install mk-cli`. In `technical-manual.yml` they go directly above
the comment line that starts `      # ---- Real-bin output-fidelity gate`
(the section comment that sits above `Install mk-cli`), so that comment stays
with the installs it introduces.

```yaml
      - name: Put cached sibling CLIs on PATH
        # The three Install *-cli steps below install into
        # $RUNNER_TEMP/sibling-clis (their CARGO_INSTALL_ROOT); this puts its
        # bin/ first on PATH. It also reads the runner image name for the
        # cache key, so a cache built on one image is never restored on
        # another.
        id: sibling
        if: steps.guard.outputs.relevant == 'true'
        run: |
          echo "$RUNNER_TEMP/sibling-clis/bin" >> "$GITHUB_PATH"
          echo "image=$ImageOS" >> "$GITHUB_OUTPUT"

      - name: Restore cached sibling CLIs
        # On a hit each `cargo install` below finds its pinned package
        # already installed and skips the build (it still fetches the git
        # repo); a changed tag or feature set reinstalls. cargo does not
        # reinstall for a newer rustc, so the key also hashes
        # rust-toolchain.toml. Skipped on tags, so release assets are always
        # built from fresh installs.
        if: steps.guard.outputs.relevant == 'true' && !startsWith(github.ref, 'refs/tags/')
        uses: actions/cache@v4
        with:
          path: ${{ runner.temp }}/sibling-clis
          key: sibling-clis-${{ steps.sibling.outputs.image }}-${{ github.job }}-${{ hashFiles('.github/workflows/<FILE>', 'rust-toolchain.toml') }}
```

where `<FILE>` is the file's own name (`manual.yml`, `quickstart.yml`,
`technical-manual.yml`).

**K2.** In the same three files, add to each of the three steps
`Install mk-cli`, `Install md-cli`, `Install ms-cli` a step-level `env:` block
placed directly above the step's `run:` line, at the step's key indent:

```yaml
        env:
          CARGO_INSTALL_ROOT: ${{ runner.temp }}/sibling-clis
```

None of those nine steps has an `env:` today; if one does, STOP and report.
Do not change the `run:` lines. `sibling-pin-check.yml` scans those exact
`cargo install --git ... --tag ...` lines; they must stay byte-identical.

**K3. `manual-gui.yml`**:
- In job `verify-examples`, directly above `      - name: Install GUI-pinned CLI tier (md / ms / mk) into an isolated root`, insert:

```yaml
      - name: Read runner image for the cache key
        id: img
        run: echo "image=$ImageOS" >> "$GITHUB_OUTPUT"

      - name: Restore cached GUI-pinned CLI tier
        # DOCBIN_ROOT below. On a hit the four `cargo install`s find their
        # pinned packages already installed and skip the build; a changed
        # tag reinstalls. cargo does not reinstall for a newer rustc, so the
        # key also hashes rust-toolchain.toml. Skipped on tags.
        if: ${{ !startsWith(github.ref, 'refs/tags/') }}
        uses: actions/cache@v4
        with:
          path: ${{ runner.temp }}/docbins
          key: docbins-${{ steps.img.outputs.image }}-${{ hashFiles('.github/workflows/manual-gui.yml', 'rust-toolchain.toml') }}
```

- In job `verify-examples-gui`, directly above `      - name: Install pinned gui-render (headless, --no-default-features)`, insert:

```yaml
      - name: Read runner image for the cache key
        id: img
        run: echo "image=$ImageOS" >> "$GITHUB_OUTPUT"

      - name: Restore cached GUI-pinned gui-render
        # GUIBIN_ROOT below. The pinned tag comes from pinned-upstream.toml,
        # so the key hashes it too. gui-render builds with
        # RUSTUP_TOOLCHAIN=stable, and a new stable release does not rebuild
        # it; only a changed tag does. Skipped on tags.
        if: ${{ !startsWith(github.ref, 'refs/tags/') }}
        uses: actions/cache@v4
        with:
          path: ${{ runner.temp }}/guibin
          key: guibin-${{ steps.img.outputs.image }}-${{ hashFiles('.github/workflows/manual-gui.yml', 'docs/manual-gui/pinned-upstream.toml') }}
```

No other change in `manual-gui.yml` beyond T, X and C. Its install steps
already use `--root`.

**K4.** In `technical-manual.yml`, the comment line
`        # MD/MS/MK resolve via $PATH to the cargo-installed ~/.cargo/bin`
becomes
`        # MD/MS/MK resolve via $PATH to the cargo-installed $RUNNER_TEMP/sibling-clis/bin`.
That is the only comment edit in the plan. (If that changes the line's wrap,
leave the following comment lines as they are.)

`cross-tool-differential.yml` gets no K change (one install, low value).

### G — the release-workflow generator (mnemonic-engrave only)

The `release.yml` files of descriptor-mnemonic, mnemonic-secret, mnemonic-key,
mnemonic-transaction and mnemonic-toolkit are generated by
`mnemonic-engrave/scripts/release-workflows/emit.py` from the template in
`gen.py` (review r1 I1; at base all five are byte-identical to its output).
So T in those five files must also be made in the template, or the next
`emit.py` run strips it. In the engrave worktree's
`scripts/release-workflows/gen.py`, insert the line `    timeout-minutes: 60`
directly below each of its three lines that start with `    runs-on:`
(jobs `test`, `build`, `assemble`). Change nothing else in it. Do **not**
run `emit.py`: it writes into the main checkouts under
`/scratch/code/shibboleth/<repo>/`. The gate's `release-gen` mode runs the
generator in memory instead.

mnemonic-engrave's own `release.yml` is hand-written (not generated) and gets
T and C like any other workflow.

**Work order:** edit, gate and commit the six other repos first, in any
order. Do mnemonic-engrave last, and run `release-gen` immediately before its
commit (it needs all five generated repos already edited).

**Merge order:** the engrave PR and the five repos' PRs are consistent only as
a set. Merge them the same day; nothing breaks in between (CI never runs
`emit.py`), but the generator and its outputs disagree until all six are in.

## The gate (run before every commit that touches workflows)

The only authoritative gate is the toolkit worktree's
`design/ci-hardening/ci-hardening-gate.py`. Run it from there; never copy it
from anywhere else. For each repo `<name>` with worktree `<wt>` and base
`<base>` from the table above:

```sh
G=/scratch/code/shibboleth/tk-worktrees/ci-hardening/design/ci-hardening/ci-hardening-gate.py
python3 $G workflows <name> <wt> <base>
```

It builds the expected parsed YAML of every workflow from the base by
applying exactly the edits above (T, X, C, and K in their exact step order),
and asserts each edited file equals it (types included, so `1` is not
`true`; a repeated mapping key is rejected). It also asserts the file set is unchanged, the counts equal this
table, and that `git diff` removed no line except the one K4 comment line.
For the generator, once the engrave and the five generated repos are edited:

```sh
python3 $G release-gen <engrave-wt> descriptor-mnemonic=<dm-wt> mnemonic-secret=<ms-wt> mnemonic-key=<mk-wt> mnemonic-transaction=<mt-wt> mnemonic-toolkit=<tk-wt>
```

It must print `identical` for all five. `python3 $G self-test /scratch/code/shibboleth/mnemonic-toolkit 4d703bb4`
proves the gate itself: GREEN on the plan, RED on each of 14 known-bad
mutations (review r1's M1–M8, with three variants of M6, and four more).

Expected counts (bases as of descriptor-mnemonic `957a0831`,
mnemonic-toolkit `4d703bb4`, mnemonic-gui `fc25b2b`, mnemonic-key `340d9da`,
mnemonic-secret `8fef227`, mnemonic-engrave `21136827`, mnemonic-transaction
`e0200b6`; if a base has moved and the gate goes RED on counts, STOP and
report):

| Repo | T | X | C | K_steps | K_env |
|---|---|---|---|---|---|
| descriptor-mnemonic | 16 | 0 | 4 | 0 | 0 |
| mnemonic-toolkit | 42 | 6 | 12 | 10 | 9 |
| mnemonic-gui | 8 | 0 | 2 | 0 | 0 |
| mnemonic-key | 13 | 0 | 3 | 0 | 0 |
| mnemonic-secret | 20 | 0 | 3 | 0 | 0 |
| mnemonic-engrave | 5 | 0 | 1 | 0 | 0 |
| mnemonic-transaction | 4 | 0 | 1 | 0 | 0 |

(T was machine-counted two ways, `grep -c '^    runs-on:'` and parsing the YAML
for jobs with `runs-on`; both agree. K_steps is 2 per K1 file plus 2 per K3
job.)

NOT COVERED by the gate: the wording of added comments, blank lines,
quoting, YAML 1.1 spellings (`6_0`, `yes`), key order inside a mapping, and
where added lines sit relative to existing comments (C above `jobs:`, `env:`
above `run:`, K1 above the technical-manual section comment). Read `git diff`
yourself for those before committing.

Also run `actionlint .github/workflows/*.yml` in each worktree before and after
the edit, and confirm the after-output has no line the before-output lacks.

## Commits and PRs

- mnemonic-toolkit: the plan, the gate and the review reports are already
  committed on the branch (one new commit per report and per fold). Add
  nothing to `design/`.
- Every repo, then one commit with the workflow edits, message:

  ```
  ci: job timeouts, cancel superseded PR runs[, texlive step timeout, cached sibling-CLI roots]

  Per design/ci-hardening/PLAN_ci_hardening_2026-10-11.md (mnemonic-toolkit).
  T: <n> jobs, timeout-minutes 60
  C: <n> workflows, cancel superseded PR runs
  (toolkit also) X: 6 texlive steps, timeout-minutes 15
  (toolkit also) K: 10 cache/PATH steps, CARGO_INSTALL_ROOT on 9 install steps
  (engrave also) G: gen.py template gets the same 3 job timeouts

  Gate: <paste the gate's counts line and GATE line>
  (in the engrave commit, also the release-gen lines)
  actionlint: no new findings
  ```

  (the bracketed part only in the toolkit). End each message with the
  attribution lines the session requires.
- Push `ci/hardening-2026-10-11` and open a **draft** PR against the base
  branch, titled `ci: job timeouts and cancel superseded PR runs` (toolkit:
  `ci: job timeouts, cancel superseded PR runs, cache sibling-CLI installs`).
  PR body: what changed, the measured reasons above, the gate output, and a
  link to the plan.
- After CI finishes on each PR, record per PR: overall conclusion, and for the
  toolkit the durations of the `Install mk-cli` / `md-cli` / `ms-cli` steps on
  the first run (a cache miss) — then re-run the toolkit `manual` workflow once
  and record the same steps on the hit.
- Do not merge. Do not mark ready. The owner merges after the whole-diff review.

## Stop-and-ask points

Stop and report instead of choosing, if: a count differs from the table or the
stated lists; a job already has `timeout-minutes`; a listed file already has
`concurrency:`; a K-target step already has `env:`; the gate goes RED and the
cause is not an edit you made by mistake; actionlint shows a new finding; any
CI job fails on the PR.
