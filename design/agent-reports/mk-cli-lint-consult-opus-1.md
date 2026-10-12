# Consult: next steps for PR #100 and the "red make lint" task (opus, round 1)

Scope: one question. Given the settled facts in the brief, what should the
controller do next? Nothing was re-audited. I read only
`design/agent-reports/mk-cli-lint-pr100-review-round-1.md` (C1/I1/M1/M2/N1) and
grepped `origin/master:design/FOLLOWUPS.md`, which has no entry for an mk-cli pin
bump or for the lint's `--version` trust.

## Bottom line

The task is finished and needs no fix. On the CI-pinned binary, master is green.
The red came from the local environment: a dev build of mk-cli that labels
itself 0.13.0. Leaving the code alone is the correct outcome, and it does not
count as a failure to deliver. The diagnosis lands in the third bucket the rules
name, "mk-cli binary drifted". Because the pin in `manual.yml:110` is
authoritative, no judgement call about which side is right is needed.

Next steps:
1. Close PR #100 unmerged and keep its branch.
2. Leave `~/.cargo/bin/mk` alone. Point `MK_BIN` at the tagged binary.
3. Make no lint or chapter change. Record the review's residue in a
   FOLLOWUPS-only PR and offer it to Brian rather than merging it.
4. Tell Brian, with the ⚠ line the rules require. Unblock Refugium PR 79's gate
   by re-running it on the pinned binary.

## 1. Close PR #100 now and keep the branch

**Recommendation: close it now, without merging, and post a comment linking the
review report. Do not delete `docs/mk-cli-lint`.**

- The draft is wrong, not merely premature. C1 says it turns CI red and
  documents flags that the pinned release rejects. I1 says it breaks the mutation
  test's anchors no matter which binary runs. Brian pre-authorised merging "the
  fix". An open PR titled as that fix is a trap: a later session, or a skim
  reader, could merge it under that authorisation. Closing it removes the trap.
- Closing is within routine judgement and does not need Brian's word. It is
  reversible (it can be reopened). It touches no CI file and deletes no branch.
  The PR was never offered for merge. It withdraws the controller's own artifact
  and does not choose between two sides. Deleting the branch is the action that
  needs Brian's typed go-ahead, so ask for it in the reply and do not do it.
- First, commit this consult report to the branch as its own persist commit, so
  that both agent reports survive on the kept branch. This does not break the
  rule against adding commits after offering a PR for merge, because the PR was
  never offered. Do not push any fold to it.

## 2. Use `MK_BIN` with the tagged binary; do not reinstall

**Recommendation: do not run `cargo install --force` over `~/.cargo/bin/mk`.
Run the lint with `MK_BIN=/scratch/code/shibboleth/mk-tag-rv/root/bin/mk`,
which is already built from `mk-cli-v0.13.0` and has been verified to give
exit 0 with 10 exemptions.**

The review's C1 suggested reinstalling. I disagree, for these reasons:

- `~/.cargo/bin/mk` is shared by every session on the machine. It is the only
  installed `mk` that has `--in`, `--out`, `--keys` and `--from-md1-set`.
  Replacing it with 0.13.0 silently breaks any Refugium, mnemonic-key or toolkit
  work, or any script or test that uses those flags. The failure would be
  `unexpected argument`, which is the same class of drift as this one, running in
  the opposite direction and harder to trace. Swapping a shared tool that other
  sessions use is beyond "fix the lint". It is Brian's call.
- `make lint` already takes `MK_BIN=…`, which is how CI calls it
  (`mnemonic-toolkit/CLAUDE.md`). Pointing it at the tagged binary reproduces CI
  exactly and has no side effects.
- `/scratch/code/shibboleth/mk-tag-rv/` is no longer just a temporary build. It
  is now the reference binary. Keep it, and do not act on the review's
  housekeeping note that it should be deleted.
- Optional, for Brian: rebuilding the installed `mk` from current mnemonic-key
  main (after 1b8c524) makes it report `0.14.0-dev`. That removes the false
  label and keeps the new flags. It is still a change to shared state, so offer
  it and do not do it.

## 3. Make no code change; record a FOLLOWUP and offer it separately

**Recommendation: change nothing in `lint.sh`, the chapter, or CI. Open one
small docs-only PR that adds two `design/FOLLOWUPS.md` entries, and offer it to
Brian. Do not self-merge it.**

- The failures need no repo change, so under "change nothing outside what the
  failures need" the fix-PR is empty. A lint that warns when `--version`
  disagrees with the pin would fix M1, a Minor. The rule says minors go to
  FOLLOWUPS as `deferred-minor` and are not fixed in this round. It also
  operates on the manual.yml pin and reads a CI file, which edges toward the
  CI-change gate. Do not build it now.
- FOLLOWUP entry A, `manual-lint-version-trust` (status `deferred-minor`, no
  owning phase): this is M1. `lint.sh:195` takes the pin from `<bin> --version`,
  so a mislabelled dev build passes as the pinned release. Options are (a) WARN
  when `--version` disagrees with the `manual.yml` pin, or (b) say in the
  Makefile `lint` help that local runs need the tagged binaries via `*_BIN=`.
  Add a companion note citing the at-source fix in mnemonic-key 1b8c524 (F-677).
  No mirror entry in mnemonic-key is needed, because its side is already fixed.
  Fold N1 into this entry as one line: attribute lint results to a binary by its
  `.crates.toml` provenance.
- FOLLOWUP entry B, `mk-cli-pin-bump-marker-removal` (owning phase: the next
  mk-cli pin-bump PR): this is the legitimate form of PR #100. It is a checklist
  for that PR:
  - bump `manual.yml:110`, `scripts/install.sh` and `44-mk-cli.md:12` together;
  - drop the 10 markers only for flags the new pin ships, and rename `after
    0.13.0` on any that remain unreleased;
  - re-anchor mutations R2, R4, R6 and R7 in `ci/doc-flag-lint.test.sh` (I1), or
    inject a synthetic marked row;
  - revisit the marker-convention paragraph at `44-mk-cli.md:16-18` (M2).

  This entry is gating for that future PR, not a minor. Without it, the C1 and
  I1 lessons are lost when PR #100 closes.
- The same PR can also carry the two agent reports onto master. That makes the
  branch safe to delete later.
- Why offer rather than merge: the pre-authorisation covered "the fix". A
  FOLLOWUPS-only PR is residue, not a fix. Offering costs Brian one click and
  keeps the consent boundary exact.
- Append one row to `/scratch/code/papercuts.md` (standing directive), for
  example: `2026-10-11 · manual lint red locally (10 stale-marker FAILs in
  44-mk-cli.md) · installed mk was a mnemonic-key main path-build labelled 0.13.0;
  run lint with MK_BIN=<tagged mk-cli-v0.13.0>; check ~/.cargo/.crates.toml
  provenance · mnemonic-toolkit`.

## 4. The reply to Brian, and the Refugium PR 79 gate

The rules require any Critical or Important finding to go on the first line, in
bold with ⚠. The outcome goes immediately after it.

> **⚠ Review of my draft fix (PR #100) found 1 Critical / 1 Important — the fix
> was wrong, so I closed it unmerged. Nothing was merged.**
>
> Master's manual lint is **not red in CI**. Its pin, `mk-cli-v0.13.0`, gives
> `make lint` exit 0 (10 exemptions) and `doc-flag-lint.test.sh` 12/0. The red
> was local only. `~/.cargo/bin/mk` is a path build of mnemonic-key main from
> 2026-09-23, made before 1b8c524 bumped it to 0.14.0-dev, so it says 0.13.0 but
> has the 10 unreleased flags. The chapter's `unreleased` markers are correct.
> PR #100 removed them, which would have turned CI red and documented flags that
> the pinned release rejects (C1). It also broke the mutation-test anchors (I1).
>
> What I did: closed #100 and kept branch `docs/mk-cli-lint`; left your
> installed `mk` untouched; re-ran the lint against the tagged binary
> (`MK_BIN=/scratch/code/shibboleth/mk-tag-rv/root/bin/mk`).
>
> For you to decide:
> 1. Delete branch `docs/mk-cli-lint`? It needs your typed go-ahead.
> 2. Merge PR #<n>? It is FOLLOWUPS only: the M1 `--version` trust is
>    deferred-minor, and the checklist is owed to the next mk-cli pin bump.
> 3. Rebuild `~/.cargo/bin/mk`? Options: from the tag (loses the new flags that
>    other sessions may use), from current main (it then honestly says
>    0.14.0-dev), or leave it as is.

The Refugium PR 79 gate should re-run the toolkit manual lint with `MK_BIN`
pointed at the tagged `mk-cli-v0.13.0` binary, the same pin as `manual.yml:110`.
It should record the exit 0 and the 10 exemptions in its gate output, together
with a one-line note that the earlier exit 2 was a local-binary artifact. It
should then proceed. PR 79 should not wait on any toolkit change, because there
isn't one.

Caveat: only `mk` has been shown to drift. If the gate is already re-running,
it should do a five-second provenance check of the other three binaries
(`~/.cargo/.crates.toml`: path build or tag). It should not assume they match
their pins.
