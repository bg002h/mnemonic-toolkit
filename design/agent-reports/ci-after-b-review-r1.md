r1: 0C/0I/0M/1N

# L3 review r1: mnemonic-toolkit PR #114 (B-tk), head bc0c4511ea718e8510975eafee42bf5443a22e0e

The one question: can the change skip `build`'s build steps on an event whose
build output is used, or on a change that is not documentation, or break the
release (tag) path? **No.** Details per brief item below.

## 1. Steps that must still run, and what assemble needs

- `build` (release.yml:158-263) has no attestation, signing, cache or required
  check. Its only output used downstream is the `mnemonic-<target>` artifact,
  read only by `assemble` (release.yml:302-305, 373). No other workflow
  downloads it (grep of `.github/workflows/` for `download-artifact` hits only
  manual-gui.yml, its own artifacts).
- `assemble` runs only when `startsWith(github.ref, 'refs/tags/') ||
  inputs.tag != '' || inputs.sign_dry_run == 'true'` (release.yml:268). Every
  such event gets a full build:
  - tag push: the docs step runs (`null == ''` is true), event `push`, ref
    `refs/tags/...` hits `*) full "push to ..."` (docs-only.sh:198);
  - dispatch with `tag`: the docs step is skipped, `steps.docs.outputs.docs_only`
    is empty, every guarded `if:` is true;
  - dispatch without `tag` (including `sign_dry_run`, and a dispatch run from a
    tag ref): event `workflow_dispatch` hits `*) full` (docs-only.sh:204).
  Evidence, run against the PR's `ci/docs-only.sh` with a failing `gh` stub
  (scratch under /scratch/code/shibboleth/review-ab-B-tk-r1/):
  ```
  [workflow_dispatch refs/heads/master] output: docs_only=false
  [push refs/tags/mnemonic-toolkit-v9.9.9] output: docs_only=false
  [workflow_dispatch refs/tags/mnemonic-toolkit-v9.9.9] output: docs_only=false
  ```
  So the spec §4 premise (dispatch and tag runs always run in full) holds for
  this workflow, including the sign_dry_run consumer of the artifacts.
- On a docs-only skip (PR, default-branch push, `ci/**` push), `assemble` is
  skipped, and `build` concludes success, so nothing reads the missing artifact.

## 2. CHECK_NAME versus the real check-run name

CHECK_NAME is `build mnemonic (${{ matrix.name }})`, rendered per leg in the
step env, so each leg looks up only its own name; the check-runs API
`check_name` filter is exact, so another leg cannot satisfy it. No other job in
`.github/` is named `build mnemonic (...)`. On PR and branch-push events the
job's name has no suffix and equals CHECK_NAME. A leg that was itself
docs-skipped green on R is accepted by design (guard header, settled Part A
induction).

## 3. Permissions

`contents: read, checks: read` covers checkout, the guard's `git fetch` and
compare API (contents) and check-runs API (checks). `upload-artifact@v4` uses
the runtime token, not GITHUB_TOKEN scopes. No step in `build` attests, signs,
writes a release or requests an OIDC token; those are all in `assemble`, which
keeps the workflow-level grant (it has no job `permissions:`).

## 4. The ` (tag)` suffix and the checkout ref

| event | checked out | check-run name (sha it attaches to) | A′/guard lookup |
| --- | --- | --- | --- |
| pull_request | `github.sha`, the test-merge commit (same tree as `refs/pull/N/merge` at event time; two-parent, as A′ needs) | `build mnemonic (X)` (merge sha) | CHECK_NAME, matches |
| push master / `ci/**` | `github.sha` (no longer the moving branch ref, so the tree is the sha the check attaches to) | `build mnemonic (X)` (pushed sha) | matches |
| tag push | `github.ref` (the tag) | `build mnemonic (X) (tag)` (tag commit) | guard runs full; never looked up |
| dispatch, `tag` set | the input tag | `build mnemonic (X) (tag)` (dispatch ref's sha, not the tag's) | guard step skipped |
| dispatch, no `tag`, branch ref | `github.sha` | `build mnemonic (X)` (that sha, built in full) | valid evidence for that tree |
| dispatch, no `tag`, tag ref | `github.ref` | `build mnemonic (X) (tag)` | full |

A ` (tag)` run can never satisfy an untagged lookup (exact-name filter), which
is what stops a backfill dispatch's run, built from the tag's tree but attached
to the dispatch ref's sha, from vouching for that sha. The reverse never
arises: tag and dispatch runs never consult the guard's skip path. The name
expression is the one the `test` job has carried since Part A; no tag run of
release.yml has occurred since then, so the suffix is not yet seen live in this
repo (not a defect: the expression is the standard `cond && a || b` form and
the empty-string fallback renders the untagged name, as L1/L2 show).

## 5. PR description

Accurate: states it changes `.github/workflows/`, the three guard cases, the
permissions, the `github.sha` / ` (tag)` change, and ends with the footer. It
correctly does not claim a `ci/` change (none in the diff).

## Minors and nits

- N1: the description says "the release build jobs" / "Each job" (plural);
  this PR guards one job, `build` (the plural is the shared B-* subject line).
