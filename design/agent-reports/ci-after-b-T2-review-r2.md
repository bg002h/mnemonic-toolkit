r2: 0C/0I/0M/2N

# L3 review: mnemonic-toolkit PR #115 (after-Part-B T2), round r2

Head reviewed: dedee40980d6d0898c6fe6d50b9c712548ecf01e, whole artifact
(`ci/shared-drift.sh` 50 lines, `ci/shared-drift.test.sh` 122, `.github/workflows/shared-drift.yml` 25,
PR description). Fold under review: 9eead365 (`sha()` line 21 `|| exit 1`, test row D7).
Scratch: `/scratch/code/shibboleth/review-ab-T2-r2/`.

**The one question.** No. I found no path on which `ci/shared-drift.sh` prints
`shared-drift: PASS` (exit 0) while a shared file on another repo's default branch
differs from toolkit's checkout. The workflow runs the comparison on `schedule` and
`workflow_dispatch` (the step's only condition is `github.event_name != 'pull_request'`).
It does not break toolkit's CI. Findings from the fold commit: 1 (N-1 below, cosmetic).

## Critical

None.

## Important

None.

## Evidence run

- `TMPDIR=/scratch/code/shibboleth/review-ab-T2-r2/tmp bash ci/shared-drift.test.sh` on the
  head: `7 passed, 0 failed`, exit 0.
- PR #115 checks. On fold head 9eead365 and report head 790d27d6, every workflow run
  concluded success (shared-drift, rust, manual, manual-gui, release, examples, quickstart,
  technical-manual, sibling-pin-check). On dedee409, a docs-only report push, `rust` and
  `manual` were still in progress when I checked, and every completed run was success. The
  SKIPPED checks are the usual docs/release ones (lint, verify-examples, verify-examples-gui,
  build, release, assemble + release). Master's required contexts (`gh api
  .../branches/master/protection/required_status_checks`) do not include `shared-drift`
  or its job name.
- shared-drift run 40241598885 (pull_request) steps: the self-test was `success`, and
  `Compare the shared CI files with the other public repos` was `skipped`, as §5.3 specifies.

## Wrong-PASS paths examined (clean)

- `set -eu` without `pipefail`: the script has no pipelines. Each hash comes from
  `sha256sum < "$1"` inside `sha()`. The fold's `|| exit 1` makes the substitution subshell
  exit non-zero, and the parent's `here_sha=$(...)`/`there_sha=$(...)` assignment then trips
  `-e`. `local h` is declared on its own line, so `local` does not mask the status. An
  unreadable file also fails the redirection, which gives the same exit 1. A `return 1`
  variant behaves the same.
- Temp file reuse: `> "$tmp"` truncates before every fetch. A failed fetch reaches `continue`
  before any hash. A gh that writes part of a body and then exits non-zero also reaches
  `continue`.
- gh exits 0 on a non-raw body (a JSON directory listing, a symlink or submodule object, a
  followed 301): the bytes are not the shared file's, so the result is DRIFT and fails
  closed (live-checked in r1). An empty body is DRIFT, because no shared file is empty
  (D4). 4xx, a missing token and rate limits make gh exit non-zero, which gives `cannot fetch`.
- Default branch: a contents call without a ref reads the repo's default branch, which is
  what §5.1 asks for.
- Loop and URL mutants beyond the controller's 8: a URL that always names one repo, or one
  file, changes D2's problem count away from 1 or loses its DRIFT line, so D2 fails. Hashing
  `$tmp` on both sides falls under "comparison always ok".
- Running from a root other than toolkit's (for example
  `bash ../mnemonic-toolkit/ci/shared-drift.sh` from mnemonic-key) compares that checkout,
  not toolkit's. The PASS line stays true for what was compared, because it does not name
  toolkit. Only DRIFT lines mislabel the local side as "mnemonic-toolkit". The header and
  §5.1 say "Run from the repo root". Not raised.

## Test rows: can each fail?

D1 fails on the 25-line count or the PASS line. D2, D3, D5 and D6 fail under the
controller's mutants. D4 fails if an empty body counts as ok. D7 fails if `|| exit 1` is
removed (controller-verified). A mutant that drops `continue` after a failed fetch double-counts:
it prints ERROR and also DRIFT on the truncated temp file. D3 still passes, but that gives
no wrong PASS. I found no other row that survives removal of its own target logic.

## Re-grade of r1 deferred items whose text the fold touched

- r1 M-2 (D6 checks each argument's presence, not one call): stays Minor. The fold
  added D7 and left D6 unchanged. The contrived mutant (an extra
  `-H 'Accept: application/json'`) would make the live body JSON, which gives DRIFT and
  fails closed, not a wrong PASS.
- r1 N-1 ("missing locally" ordering not observable): stays a Nit. It is unchanged, and the
  only cost is a wasted API call.
- r1 M-1 (scheduled-workflow silence): its text is untouched. It stays deferred.

## Minor

None.

## Nit

- N-1 (fold): when `sha256sum` fails, the script stops before printing any line, so D7's ok line shows
  `()` and a real run's log carries only sha256sum's own stderr, or nothing if the tool exits
  silently. This fails closed, and it is what §5.1 specifies ("no PASS line").
- N-2: a `workflow_dispatch` on a non-default branch compares that branch's checkout, not
  toolkit master, and nothing in the log names the ref. The default dispatch ref is master.

## PR description

It states that the PR changes `.github/workflows/` and `ci/`, cites the spec and the
byte-identical `reference/after-b/T2/`, and ends with the Claude Code footer. No issue.
