# Decision: how to land three docs-only commits on toolkit `master` (2026-09-04)

Stand-in architect ruling for the controller. Question: the staged SHA
`35b8779d` cannot earn `examples` / `test (ubuntu-latest)` / `clippy` via the
`ci/staging` push ritual because all three workflows are path-filtered on
`push`. Take (A) bypass, (B) widen the filters, (C) fake a gated-path touch, or
something better?

## Decision

**None of A, B, C. Take D: the staging-PR form of the ritual.** Push the tip to
`ci/staging`, open a PR against `master` marked "staging only, not intended to
merge", let the `pull_request` triggers earn the three required contexts on the
exact SHA, then fast-forward `master` with a plain `git push origin master` and
confirm the output carries no "Bypassed rule violations". Delete `ci/staging`;
GitHub marks the PR merged on its own once its head is reachable from `master`.

This is not an invention. It is precedent PR #68 (2026-09-02, engrave
`design/agent-reports/f324-close-report.md`), which landed `21b6696e` and
`d39d9626` -- the direct parents of the three commits in question -- for the
identical reason (a CI-file-only diff that touched no gated path). PR #65 and
PR #66 (2026-08-31) are the rebase-merge variant of the same idea, for a
docs-only and a report-only commit respectively.

## Why the rule CAN be satisfied

The push agent's "structurally cannot satisfy" was true of the **push** event
only. Both workflows deliberately carry **no `paths:` filter on
`pull_request:`**:

- `.github/workflows/rust.yml:45-52` -- "NO paths -- this workflow's
  `test`/`clippy` contexts are REQUIRED status checks ... a path-filtered
  required check wedges a docs-only PR forever ... Running the full matrix on a
  docs-only PR is green (unchanged code) -- the cost is CI minutes, not
  correctness."
- `.github/workflows/examples.yml:53` -- "NO paths -- a REQUIRED check must
  report on every PR", per the authoritative ruling
  `design/agent-reports/examples-pdf-branch-protection-ruling.md` §7.

A `pull_request` check-run binds to the PR **head SHA**, and `strict: false`
asks only whether the pushed SHA carries a passing context. Measured: the
`origin/master` tip `d39d9626` carries `clippy`, `test (ubuntu-latest)` and
`examples` = `success` today, all from PR #68's `pull_request` runs (the push
event never fired for it), and the f324 close report quotes the subsequent
`git push origin HEAD:master` output `d6277006..d39d9626` with no bypass line.
The rule was satisfied, not bypassed, on an unchanged SHA. That is exactly the
ritual, with the PR event standing in for the push event.

## Why not A, B, C

- **A (direct push, accept the bypass).** The bypass hatch is the operator's,
  not automation's. `rust.yml:59-61` says so in the repo itself: "enforce_admins
  is false here DELIBERATELY -- the maintainer's own escape hatch -- ... the
  no-bypass rule binds agents, not the human." The operator's 2026-08-15 ruling
  is verbatim "You are not permitted to bypass, but I am." The workflow comments
  that describe docs-only direct pushes as "covered by admin bypass" describe
  the human's path; they do not grant it to an agent. And since D exists, there
  is no necessity argument for A at all. Never propose flipping
  `enforce_admins` -- that setting is not the problem and stays as it is.
- **B (widen the push-side `paths:`).** `examples.yml:28-34` and
  `rust.yml:31-33` state the push filter is "DELIBERATELY LEFT IN PLACE" /
  "UNCHANGED and deliberate", and `SPEC_test_hardening_gating_and_wc_codec_ci.md`
  C1 chose to keep it on purpose. Widening it is a CI change with its own review
  cost, against documented intent, would run the full matrix on every docs push
  (the cost the filter exists to avoid), and is unnecessary because the PR
  trigger already covers the case.
- **C (touch a gated path to force the jobs).** A fabricated change to game a
  filter: it falsifies history and spends the heavy matrix to certify nothing.
  Forbidden on its face.

## D-ff versus D-rebase

Two compliant shapes exist; prefer **D-ff** (staging PR + fast-forward push,
PR #68):

- keeps `87e594e0`, `0e4abdff`, `35b8779d` byte-for-byte -- the push report
  is named for the tip SHA and engrave continuity cites these;
- no `git reset --hard` afterwards, no orphaned local commits;
- `master` stays linear with no merge commit.

**D-rebase** (`gh pr merge --rebase`, PR #65/#66) is equally satisfied-not-
bypassed and is the fallback if the fast-forward push is refused for any
reason. It rewrites the three SHAs (GitHub always re-applies), so the report
filename would then name a SHA absent from `master`, and local `master` must
be reset to `origin/master` after an empty `git diff --stat` check -- the
procedure PR #65's report followed.

## What the PR's NON-required jobs will do (predicted, so nothing is a surprise)

- `sibling pins match install.sh`: **RED, pre-existing and deliberate.** It is
  already red on `origin/master` (`d39d9626`) and on `d6277006`. Cause:
  `.github/workflows/cross-tool-differential.yml:70-80` pins
  `descriptor-mnemonic-md-cli-v0.11.2` against install.sh's `v0.14.0`, with the
  comment "Bump this pin only alongside a fix to
  cli_cross_tool_differential.rs's corpus ... or its oracle". Not required, not
  caused here, not fixable in a docs push. It is also red on the deleted
  `ci/staging` run for `35b8779d` for the same reason.
- `build` (`manual.yml`; its `pull_request` paths include `docs/manual/**`, so
  it WILL run -- the one job that actually exercises these docs): **predicted
  GREEN.** CI installs ms-cli `v0.16.0` (`manual.yml:90`), the same version the
  local lint passed against (`ms 0.16.0`). At 0.16.0, `ms hashlock --help`
  exits 64 "unrecognized subcommand 'hashlock'"; `lint.sh:84` extracts `--help`
  from that error text and `43-ms.md` contains `--help` (8 occurrences), so the
  flag-coverage step emits no error -- the same mechanism PR #65's report
  measured. CI's mk-cli is `v0.12.0` vs local `0.13.0`; the check direction is
  help-flags ⊆ chapter (`lint.sh:93`), and all 13 flags of `mk encode --help`
  at 0.13.0 are present in `44-mk-cli.md`, so the 0.12.0 subset passes too.
  If `build` goes red anyway, that is a finding to assess before pushing (as
  PR #65 did), not a non-required job to merge past.
- The required trio runs against unchanged code and was green on PR #65 and
  PR #68.

## Procedure for the push agent (sonnet)

Do this first, so report-only commits do not need a round of their own (PR #66
was one commit): commit the untracked push report
`design/agent-reports/push-toolkit-35b8779d.md` and this file, each in its own
short commit, staging those two paths explicitly (the tree holds many other
pre-existing untracked files; do not sweep them in). Then freeze `master` and
stage the new tip:

```
git push origin master:refs/heads/ci/staging
gh pr create --repo bg002h/mnemonic-toolkit --base master --head ci/staging \
  --title "docs(manual): ms hashlock section + mk encode mirror (staging PR to trigger required checks)" \
  --body "Staging PR only -- triggers the pull_request-scoped required checks (test, clippy, examples) that the push-side paths: filter does not cover for a docs-only diff. Not intended to merge; master will be fast-forwarded once checks pass. Precedent: PR #68. Ruling: design/agent-reports/decision-toolkit-docs-push-path.md"
gh pr checks <n> --repo bg002h/mnemonic-toolkit --watch
gh api repos/bg002h/mnemonic-toolkit/commits/<tip-sha>/check-runs --jq '.check_runs[] | {name, conclusion}'
   # require examples, test (ubuntu-latest), clippy = success ON THE TIP SHA
git fetch origin master && test "$(git rev-parse origin/master)" = d39d96269ce352270189c11fabebf9ad070362b4
git push origin master          # expect d39d9626..<tip>; ANY "Bypassed rule violations" line = FAILURE, stop and report
git push origin --delete ci/staging
gh pr view <n> --json state     # expect MERGED (auto), else close it with a comment
```

No commits to `master` between the staging push and the final push.

## What to record

1. In the push agent's report (agent-written, own commit): PR number, per-job
   conclusions on the exact tip SHA, the verbatim `git push` output, the
   absence of the bypass string, `ci/staging` deleted, PR state.
2. **A ritual-doc follow-up, separate commit, not part of this push:** the
   ritual as written in `rust.yml:14-30`, `examples.yml:28-34` and the push
   brief describes only the push-event form, and two push agents in a row
   (`4388f1cf`, `35b8779d`) stopped on a case PR #68 had already solved two
   days earlier. Add the clause "when the diff touches no gated path, open a
   staging PR from `ci/staging` -- `pull_request` has no filter -- then
   fast-forward" to the toolkit's ritual comment and to the standing push
   brief / engrave memory (`push-via-sonnet-agent-automatically` currently
   says the primaries push directly).
3. **A toolkit `design/FOLLOWUPS.md` entry for the red `sibling-pin-check`,**
   if none exists: grep of FOLLOWUPS for `cross-tool-differential`, `v0.11.2`,
   `held back` finds only older slugs; the 2026-08-31 hold-back lives only in
   commit `45fe2ca1`'s message and the workflow comment. A required-adjacent
   gate that is red by design on every push deserves a filed owner.

## What I read (read-only; nothing modified, nothing committed, no `.jsonl`)

- `git status`, `git log origin/master..master`, `git diff --stat` in
  `/scratch/code/shibboleth/mnemonic-toolkit` (ahead 3 / behind 0; 5 files,
  +180).
- `gh api .../branches/master/protection` (contexts `examples`,
  `test (ubuntu-latest)`, `clippy`; `strict:false`; `enforce_admins:false`;
  `required_linear_history:false`; no PR-review requirement); repo merge
  methods (merge/squash/rebase all allowed).
- `gh api .../commits/{35b8779d,d39d9626,2f79f970,d8f06483}/check-runs`;
  `gh pr view 65/68`; `gh api .../pulls/{65,66,68}/commits`;
  `gh pr list --state merged --limit 6`.
- `.github/workflows/rust.yml` (:1-53), `examples.yml` (:1-55),
  `manual.yml` (:1-40, :79-90), `sibling-pin-check.yml` (:1-40),
  `cross-tool-differential.yml` (:70-90).
- `design/agent-reports/examples-pdf-branch-protection-ruling.md` §7;
  `design/agent-reports/push-toolkit-35b8779d.md` (untracked);
  `design/agent-reports/push-2026-08-31-mdcli-docs.md`;
  `design/SPEC_test_hardening_gating_and_wc_codec_ci.md` (C1, via grep);
  `design/FOLLOWUPS.md` (grep only);
  `docs/manual/tests/lint.sh` (:64-100); the `cli-subcommands.list` diff.
- `/scratch/code/shibboleth/mnemonic-engrave/design/agent-reports/f324-close-report.md`
  (:60-130).
- Operator memory: `push-bypass-is-an-asymmetry.md`, `never-skip-jobs.md`,
  `push-via-sonnet-agent-automatically.md`.
- Local binaries: `ms 0.16.0`, `mk 0.13.0`, `md 0.14.0`; ran
  `ms hashlock --help` (exit 64) and the `lint.sh:84` extraction against it;
  checked every `mk encode --help` flag against `44-mk-cli.md`.
