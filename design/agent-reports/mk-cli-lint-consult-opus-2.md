# Consult: decisions on PR #101, the kept branch, the PR 79 gate and what is still owed (opus, round 2)

Scope: Brian asked for decisions, not advice ("Ask opus not advise me"). Each
item below gets one verdict. I made read-only checks only. The facts settled in
consult 1 were not re-derived.

## Checks I ran (2026-10-11)

- PR #100: `CLOSED`, not merged. The branch `docs/mk-cli-lint` is still on
  origin at `e95043d3`, and the PR has 2 comments.
- PR #101: `OPEN`, draft, head `ee6a6160`, `MERGEABLE`, merge state `BLOCKED`
  (draft and pending checks). It has 0 reviews, 0 review threads and 0
  comments. It changes three files: `design/FOLLOWUPS.md` and the two agent
  reports. CI: 4 jobs were still `IN_PROGRESS` (test ubuntu-latest ×2,
  test macos-latest, musl x86_64). Every completed job was SUCCESS or SKIPPED.
- The FOLLOWUPS citations in #101 hold against `origin/master`. The pin is at
  `manual.yml:110`. The 10 markers are at `44-mk-cli.md` 43-46, 134, 183, 241,
  412, 503 and 553. The convention paragraph is at 16-18 and the pin text at
  line 12. R2, R4, R6 and R7 are in `ci/doc-flag-lint.test.sh`, and the mk-cli
  pin is in `scripts/install.sh:61`. The diff contains no model ids and no email.
- A set of tag builds of all three pinned sibling CLIs already exists at
  `/scratch/cache/ci-tools/bin/{md,mk,ms}`. Its `.crates.toml` and
  `.crates2.json` show `mk-cli-v0.13.0`, `descriptor-mnemonic-md-cli-v0.20.3`
  (features `cli-compiler`) and `ms-cli-v0.20.1`. These match
  `manual.yml:110/118/123` exactly. Papercuts row 32 (2026-10-06) set it up for
  this same trap.
- Papercuts row 41 (2026-10-11) gives the fix as "reinstall from tag with
  --force". That contradicts the decision Brian accepted: leave the shared
  `~/.cargo/bin/mk` alone.
- Refugium PR 79 is the draft plan PR "build addendum for spec PR 76 (fold I
  readings)", at head `c1601634`. Its record
  `design/agent-reports/plan-fold-i-readings-r1-fold.md:35` says the full
  toolkit `make -C docs/manual lint` "exits 2 on 10 `44-mk-cli.md` marker
  FAILs … The same 10 fail on clean `4d703bb4`, so they are pre-existing". That
  is wrong. Master is green at the pins, and the failure came from the local
  binary. The plan text itself (around line 554) correctly leaves the full
  `make lint` to the toolkit's CI. The status file
  `coordinator/status/mr-fold-i-readings-build.md` does not mention any of this.

## A. Mark #101 ready and merge it yourself: **DO**, once three conditions hold

Brian has removed himself as the approver ("Ask opus not advise me"). His
original prompt already authorised a session merge on the conditions below, and
a docs-only, revertible PR carries no risk that justifies waiting on him again.

This rests on reading "Ask opus not advise me" as his answer to the offer of
#101. If that message answered something else, the verdict is DON'T, and Brian
must type: `merge #101 at ee6a6160`.

Required first, in this order:

1. **CI green on `ee6a6160`.** Every check must be SUCCESS or SKIPPED, with
   nothing pending. Four jobs were still running when I looked.
2. **An independent whole-diff review of #101 on Opus at 0 C / 0 I.** The
   controller dispatches a fresh Opus subagent. It must not be the controller,
   and it must not be me: I am the decider, and my predecessor's consult is in
   the diff. Brief it to review only `git diff origin/master...ee6a6160`, with
   no fresh audit. Give it these settled facts so it does not re-derive them:
   the citations above are machine-checked; master is green at the pins; the
   reports are verbatim agent output and are not to be edited. It writes its
   report to
   `/scratch/code/shibboleth/mt-lint-followups/design/agent-reports/mk-cli-lint-pr101-review-round-1.md`.
   **Do not commit that report, or this one, onto #101.** #101 has been
   offered, so its head stays `ee6a6160`. Persist both reports on a new pushed
   branch with no PR, for example `docs/mk-cli-lint-reports`, cut from
   `ee6a6160`, with one persist commit per report. Record that branch in the
   status file.
   - If the review finds **any C or I**: do not merge and do not fold onto
     #101. Tell Brian loudly (⚠ on the first line). A fold would add commits to
     an offered PR, and that needs Brian.
   - Minors and Nits stay in the review report, which carries them; no FOLLOWUPS
     commit is made on #101.
3. **No open threads.** Re-check at merge time. There were 0 threads and 0
   comments when I looked.

Then:

- Edit the PR body's "How" line, which currently says "Offered for Brian to
  merge; not merged by the session". Change it to: merged by the session after
  Brian delegated the decision; Opus review `mk-cli-lint-pr101-review-round-1.md`
  returned 0/0. Editing the body does not change the head.
- Run `gh pr ready 101`, then
  `gh pr merge 101 --merge --match-head-commit ee6a61602ef714331dc37170e0e83a630739c2d9`,
  with no `--delete-branch`.
- Confirm the merge with `gh pr view 101 --json state,mergeCommit`.

## B. Delete branch `docs/mk-cli-lint`: **DON'T**

The rules allow deleting a branch only on Brian's typed go-ahead naming it. He
said "Do not delete the branch". "Ask opus" names no deletion. Keeping the
branch costs nothing.

Leave the branch and do not raise it with Brian again. If he ever wants it
gone, he types `delete branch docs/mk-cli-lint`. Do not delete
`docs/mk-cli-lint-followups` after the merge either, because the same rule
applies.

## C. Refugium PR 79 gate: **DO** write one note; make no change to PR 79 itself

**Decision:** add a short note to
`/scratch/code/shibboleth/refugium-coordinator/coordinator/status/mr-fold-i-readings-build.md`,
the tracker for PR 79. Do not commit anything to PR 79 and do not edit its
persisted fold record.

The controller has not done enough yet. PR 79's own record says the toolkit
lint failures are "pre-existing" on master. The next PR 79 reviewer, or the
Sonnet implementer at step 5.2, could believe toolkit master is red. They might
then stop to ask, or "fix" the markers again, which would repeat PR #100.

Suggested note:

> 2026-10-11: The toolkit `make -C docs/manual lint` exit 2 recorded in
> `plan-fold-i-readings-r1-fold.md:35` (10 `44-mk-cli.md` marker FAILs) was not
> pre-existing on master. It came from a local `mk` dev build. Master is green
> in CI at the pins (mk-cli-v0.13.0; toolkit PR #100 closed, #101). No PR 79
> plan change is needed: the plan leaves the full `make lint` to toolkit CI,
> which is correct. For any local re-run, use
> `MD_BIN=/scratch/cache/ci-tools/bin/md MK_BIN=/scratch/cache/ci-tools/bin/mk MS_BIN=/scratch/cache/ci-tools/bin/ms`
> (tag builds that match `manual.yml`), with `MNEMONIC_BIN` and `MR_BIN` built
> from the tree under test. Give this as a settled fact in the next PR 79
> review brief.

PR 79's gate does not wait on any toolkit change.

## D. What is still owed

1. **Papercuts row 41: DO amend it.** Its fix ("reinstall from tag with
   --force") contradicts the decision Brian accepted. Replace the fix with:
   "run the lint with `MK_BIN` (and `MD_BIN`, `MS_BIN`) set to
   `/scratch/cache/ci-tools/bin/*` (tag builds, see row 32); do not reinstall
   the shared `~/.cargo/bin/mk`; any non-tag `mk` fails this lint whatever
   version it reports". Add "recurrence of row 32 (2026-10-06)". The file is not
   under git, so edit it directly.
2. **Status file `toolkit-lint-fix.md`: DO update it after A completes.** Fill
   in: #101 merged (merge commit), or held on a C/I; branches
   `docs/mk-cli-lint` and `docs/mk-cli-lint-followups` kept; the reports
   branch name; "Waiting on: nothing" (it currently waits on Brian, which is
   now stale). Also name `/scratch/cache/ci-tools/bin` as the reference
   binaries next to `mk-tag-rv`.
3. **Persist this report and the #101 review report: DO**, on the reports
   branch as described in A. They must not stay uncommitted in the worktree
   only.
4. **Rebuilding `~/.cargo/bin/mk`, the open question 3 from consult 1:
   DON'T.** Any non-tag build fails this lint, whatever version it reports, so
   a rebuild from main fixes nothing here. PR 79's record shows the same 10
   FAILs "vs pinned 0.14.0". A rebuild would also change a binary that other
   sessions share. The fix is the `*_BIN` path, and it is now recorded in
   FOLLOWUPS, papercuts and the PR 79 note.
5. **Nothing else is owed.** PR #100 already carries the closing comment, the
   FOLLOWUPS citations check out, and no CI or `.github/` change is involved.
