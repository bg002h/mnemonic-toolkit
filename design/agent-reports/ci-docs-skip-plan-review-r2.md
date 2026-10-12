# ci-docs-skip plan review r2 (independent, adversarial)

Artifact: toolkit branch `design/ci-docs-skip` @ bffe59de; fold under review `git diff 3e827495..bffe59de`.
Bases read: the post-hardening worktrees `/scratch/code/shibboleth/{dm,tk,mk,ms,me,mt}-worktrees/ci-hardening`.

**Verdict: 0 Critical / 1 Important / 2 Minor / 0 Nit.**

## r1 closure

| r1 | Closed? | How checked here |
|---|---|---|
| C1 | Yes | Re-walked r1's scenarios (strict:false merge on a moved base, admin push, a docs PR on a red or pending tip, the M3 paint-over): each now reaches `full` at the check-run step. A PR-event run on R tests the merge tree, not R. That is still sound: a SHA can become the default tip by fast-forward only if it descended from the base when its PR run happened, and then the merge tree is R's tree. |
| I1 | Yes | `allowed_removed()` now includes the combined `if:` lines. I ran `apply_text()` on the post-hardening toolkit, key, secret and dm `ci.yml`/`rust.yml`/`release.yml`. Each parses equal to `expected()`. The one difference is toolkit `lib-cross-platform` CHECK_NAME, where the plan's stated override applies. |
| I2 | Yes | `g6-invariant` is excluded in both repos. No other guarded job uses a checkout `path:` or `working-directory` before the guard. `expected()` on the post-hardening bases gives exactly the plan's counts in all six repos. |
| I3 | Yes | The four `.gitattributes` files end in `\n`. Appended rules come last, so they win (including over key's `* text=auto`). The CR strip and `check-attr --cached` are in place. |
| I4 | Yes | Base-ref check added, with a test. |
| I5 | Yes | No guarded step is a multi-line `- run: |` (grep over all guarded files). |
| I6 | Yes | Worktree == index is enforced, the removed-lines and changed-set checks read `--cached`, and the post-commit `status`/`diff --name-only` checks cover what the gate cannot. I found no false-PASS route that is not already caught: an unstaged or untracked file, `MM`, a deleted workflow, an extra file, or a forgotten gen.py. Engrave `release-gen` writes `__pycache__/`, but that is gitignored. |

GitHub semantics, measured with read-only API calls:
- `filter=latest` deduplicates **per check suite**, not per name. Toolkit `a97a80c84b` with `check_name='test (ubuntu-latest)'` returns 3 runs (release-PR, rust-PR, release-push suites). So "every run must be success" includes same-named runs from other workflows, and the rule is conservative.
- `gh api -X GET -f check_name=…` encodes `+`, spaces, commas and parentheses correctly: `test (rust + go)`, `musl build+test (…)` and `lib cross-platform check (x86_64-pc-windows-msvc, windows-latest)` all resolve.
- All six default tips are green on every guarded name. The only non-success runs are job-level-skipped jobs, and none of them shares a guarded name.
- Top-level `permissions: {contents: read, checks: read}` breaks no job in the four workflows. No step uses the token beyond checkout. Toolkit and secret g6 check out a *public* sibling, and the token's scope was always this repo. Key `release-on-tag` keeps its job-level grant. Fork PRs get a read-only token, which still includes `checks: read`.
- actionlint 1.7.12 on the base and on `expected()`-dumped workflows: 0 findings both sides, in all six repos.

---

## I1 — Important — the post-merge probe does not say to wait for the reference's own runs, so it fails as written

Location: PLAN "After the owner merges: the probe", steps 1–2. This text was not touched by the fold, but the fold's C1 change gave the probe a precondition it never states.

Failure scenario: the owner merges the PR. The implementer follows step 1 at once ("from a fresh clone of the new default tip") and pushes `ci/docs-only-probe`. The reference R is the new default tip, i.e. the merge commit. Its own push run, a full run because the change is code, is still in progress, or for the toolkit up to ~10 min of `test (macos-latest)`. Every guarded job then logs `docs-only: full run ('<name>' on <R> is in_progress/null, not completed/success)` and runs everything. Step 2's expected log line never appears. The same happens for any guarded job that is red or flaky on R. The plan says nothing about either case, so a Sonnet implementer stops.

A smaller part predates the fold. Step 2's "every later step shows as skipped" is false for engrave `test`: `SH2 demo install links resolve and are current` is unconditional by design, and `docs-only verdict` runs exactly when `docs_only == 'true'`.

Fix: insert a step 0: "Wait until every workflow run on the new default tip R has completed. Then list R's check runs (`gh api repos/<o>/<r>/commits/<R>/check-runs -X GET -f filter=latest -f per_page=100`). For every guarded job whose run on R is not `completed/success`, either re-run it until it is, or expect that job to log `full run ('<name>' on <R> is …)`; that is not a probe failure." In step 2, exempt engrave `test`'s `SH2 demo install links…` and `docs-only verdict` steps from "every later step shows as skipped".

## M1 — Minor — key `ci.yml`'s permissions comment becomes false after D5

Location: PLAN D5; key `ci.yml` lines 43–45: "Workflow-level permissions are not granted; each job declares its own. … build + fmt + vectors-roundtrip run with the default read-only token."

Failure scenario: after D5 the workflow grants `contents: read, checks: read` at the top level, and no job but `release-on-tag` declares its own. A later editor trusts the comment. The gate does not cover comments (NOT COVERED).

Fix: in D5, for key, replace those three comment lines with, e.g., "Workflow-level permissions: `contents: read` and `checks: read` (ci/docs-only.sh reads the reference commit's check runs). Only release-on-tag overrides them, with `contents: write` (release upload)." Also add those lines to the gate's allowed removed lines for key `ci.yml`.

## M2 — Minor — "this job's own check run" is really "every same-named run on R"

Location: PLAN safety rule ("asks the GitHub API for this job's own check run on the reference"); docs-only.sh header lines 21–24.

The query is by name. In toolkit and secret it also returns `release.yml`'s `test (ubuntu-latest)` for `rust.yml`'s `test (ubuntu-latest)`, and the reverse (measured above). It also returns PR-event suites. Requiring all of them is conservative. But if the guarded job's own workflow had no run on R, a green same-named job from another workflow would satisfy the check. I found no steady-state route to that: after D2, `rust.yml` runs on every push, and `[skip ci]` skips both workflows. So this is wording, not a hole. Fix: say "every latest check run named CHECK_NAME on R, from any workflow or event" in the plan and in the script header.

---

Fold accounting: all 3 findings come from the r1 fold commit bffe59de. I1 comes from the C1 change, whose new precondition the probe does not state; its engrave sub-point predates the fold. M1 comes from D5, and M2 from the C1 wording. The fold was authored by the controller session; this reviewer does not know its model or effort.
