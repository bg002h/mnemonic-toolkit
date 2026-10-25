r1: 0C/0I/4M/3N

# L3 review r1: mnemonic-toolkit PR #113 (T1), head 85f56872

Reviewer: Opus, independent whole-diff adversarial pass, per
`ci-skip-constellation/briefs/l3-ab-T1-r1.md`. The question: can any change
give a wrong `relevant=false`? Answer: no path found. Every surviving
mutant below either fails closed in real use or is an equivalent mutant.

## Method

- Read the whole diff (`git diff origin/master...85f56872`), SPEC-after-b
  §3, §6 and §7, and SPEC.md §2.1.
- Dumped all five workflows with PyYAML. Every step after each guard step
  has `relevant` in its `if:`, except the tag-only release steps, and tags
  always answer `relevant=true`. Each manual-gui inner job has
  `if: needs.changes.outputs.relevant == 'true'`. CHECK_NAME, PUSH_BEFORE,
  GH_TOKEN and `checks: read` are in place as §3.2's table says. No
  workflow has top-level `permissions:`. The manual-gui `changes` job only
  checks out and runs the guard, so `contents: read` is enough.
- Mutation run. I made 21 single-point mutants of `ci/doc-gate-guard.sh`,
  each committed in its own clone, and ran each against the full
  `ci/doc-gate-guard.test.sh`, 21 in parallel. Baseline: 158 passed,
  0 failed. Scripts:
  `/scratch/code/shibboleth/review-ab-T1-r1/scripts/mutants.py` and
  `run-mutant.sh`. Logs: `/scratch/code/shibboleth/review-ab-T1-r1/mut/*.log`.

| mutant | change | result |
| --- | --- | --- |
| c1 | drop the base-ref check | killed by AP19 |
| c2 | drop the CHECK_NAME check | killed by AP11 |
| c3hex | allow A-F | killed by AP9 |
| c3len | drop the 40/64 length check | **survives** (M3) |
| c4 | drop the parent count | killed by AP14 |
| c4second | merged = second parent | killed by AP18 |
| c5 | drop the present-or-fetch check | survives (fails closed later; see N2) |
| c6 | `w2=$watched` (main-flow set) | **survives** (M4) |
| c7empty | drop the empty-diff check | killed by AP17 |
| c7hit | drop the watched test in A′ | killed by AP12, AP13, AP16, S12-AP (15 rows) |
| c7prhead | diff B..HEAD^2 instead of B..HEAD | killed by AP16 |
| c8rc | ignore compare's exit code | **survives** (M1) |
| c8status | accept any status | killed by AP5, AP6 |
| c9rc | ignore check-runs' exit code | **survives** (M1) |
| c9empty | drop `[ -n "$runs" ]` | survives (equivalent; N1) |
| c9first | check only the first run line | killed by AP4 |
| c9post | drop `-X GET` | **survives** (M2) |
| c9jq | `.check_runs[0]` | **survives** (M2) |
| c9nofilter | drop `-f filter=latest` | killed by AP18 |
| event | try A′ on every event | killed by AP15 |
| noz | drop `-z` | killed by 8 S12 rows (matches the controller's 150/8) |

(c7hit's first run died when its clone failed on a git maintenance.lock
race. Rerun alone: 143 passed, 15 failed.)

## Checked; no finding

- **Main flow after the refactor.** `first_hit` keeps the old loop's order
  (changed files outer, watched list inner, then `--also`), its three
  reason texts and its here-strings. It returns 1 exactly where the old loop
  fell through to `decide false`. A grep error (exit 2) reads as no match,
  as before. `a_prime` can reach `relevant=false` only through all nine
  conditions. A crash inside it under `set -u` exits non-zero with no
  output, which is a red job and not a skip. No old `true` path now reaches
  `false`, except through A′.
- **A′ soundness, given the settled design.** With M an ancestor of B
  (cond. 8) and a forward-only default branch, every earlier merged base
  was an ancestor of B. So B's test merge had B's own tree. Cond. 7 diffs
  B against the test merge, not the new PR head, so a base that gained
  watched content is caught (AP16, and c7prhead is killed). If HEAD were
  ever a fast-forward to a PR-side merge commit, so that its first parent
  is the PR's own parent, cond. 7 still sees any master change merged in
  since B. That gives no wrong `false`.
- **Cond. 4 header parse.** I looked at this under the PR R lesson. Header
  continuation lines (`gpgsig`, `mergetag`) start with a space, so they
  never read as empty or as `parent `. The header ends at the first empty
  line. `cat-file` keeps the real parents even in a shallow clone
  (technical-manual checks out at depth 1). `rev-parse HEAD^2` would
  misreport there, because a shallow clone grafts the parents away. Keep it.
- **`changed_files`.** `-z` with `tr` and pipefail: a failing `git diff`
  stays a failure. `doc-gate-paths.py` already uses `ls-tree -z`, so it
  has no quotepath sibling.
- **`--jq` outputs and run lines.** Both are compared by exact equality:
  `ahead|identical` and `completed/success`, with `null` printed for a
  pending run's conclusion. No hand-parsing of GitHub syntax remains.
- **manual-gui `--also` without `crates/`.** This is pre-existing and
  correct: its CLI tier is pinned and `cargo install`ed, not built from this
  tree (manual-gui.yml header).

## Minors

- **M1.** AP7 cannot fail when either exit-code check is removed (c8rc and
  c9rc both survive). `FAKE_RC=1` fails both stub calls at once, and the
  stub still prints its normal reply. Neither removal gives a wrong `false`
  in real use: the downstream checks need an exact `ahead`/`identical` or
  `completed/success` on stdout, and a failing `gh api` does not print one.
  Fix: give each call its own exit code (`FAKE_COMPARE_RC`,
  `FAKE_RUNS_RC`) and add one row per call.
- **M2.** The stub ignores `-X GET` and the `--jq` program, and AP18 does
  not pin them (c9post and c9jq survive). Dropping `-X GET` fails closed:
  gh POSTs, the call errors, and A′ goes dead. A jq that read only
  `.check_runs[0]` would let one passing run hide a pending one on B, and
  the suite would stay green. That is not reachable today, because the
  code's jq is correct, and L2 ran it live. Fix, removing the
  interpretation rather than adding a case: add `-X`, `GET`, `--jq` and the
  exact `.check_runs[] | "\(.status)/\(.conclusion)"` to AP18's
  `grep -qxF` list.
- **M3.** No row exercises cond. 3's length check (c3len survives). A
  shorter hex prefix still names B, or fails as ambiguous, so it gives no
  wrong `false`. Fix: add a row with before = `${P1:0:39}`, want true.
- **M4.** No row tells cond. 6's B-tree watched set apart from the main
  flow's merge-base-tree set (c6 survives). I found no wrong `false`. A
  reference that exists only in B's tree can stop existing only through a
  change to a file in the book, or to another file the set already
  watches, and that change is itself a hit. File it, or add a row in which
  B's book references a file that neither the merge-base nor HEAD
  references.

## Nits

- **N1.** AP3 cannot fail when `[ -n "$runs" ]` is removed (equivalent
  mutant). `<<<""` yields one empty line, and the loop already refuses it.
  The explicit check only chooses the reason text.
- **N2.** AP10 (40 zeros) cannot fail under any single removal. Cond. 3
  accepts zeros by design, and conds 5, 6 and 7 each refuse them. The row
  is a fail-closed check, not a cond. 3 check.
- **N3.** S10a compares CHECK_NAME with the workflow file's name, not with
  the job's check name. If a job later gains a `name:`, S10a still passes,
  and A′ then finds no runs, so it fails closed. §8's spec gate covers this
  at build.
