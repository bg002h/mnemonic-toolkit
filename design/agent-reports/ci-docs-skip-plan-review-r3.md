# ci-docs-skip plan review r3 (narrow: the r2 fold of the post-merge probe)

**Verdict: 0 Critical / 0 Important / 2 Minor / 2 Nit.**

Scope: `git diff HEAD~1 HEAD -- design/ci-docs-skip/` at e65e5c74 (plan section "After the owner merges: the probe"). One question: does it close r2 I1, and does the new text add a Critical or Important defect?

## r2 I1: closed

- Step 0 makes the implementer wait for every run on R, list R's check runs, and note each non-success guarded job without re-running it. Step 2 gives noted jobs the expected line `docs-only: full run ('<NAME>' on <R> is …, not completed/success)`. That line matches what `docs-only.sh` prints: with one documentation file, classification passes and the check-run test is the first one that can fail. `$base` is the full R sha. The in-progress, red and flaky cases from r2 now all have a stated outcome.
- The engrave `test` exemption names exactly the two steps that run after the guard on a docs-only change: `SH2 demo install links resolve and are current`, which is unconditional, and `docs-only verdict`, which has `== 'true'` and only echoes. The `signing key copies agree…` step and the self-test both come before the guard. In the generated `release.yml` `test` template, every step after the checkout is one of the four D4 lines or mt's TEST_SETUP step, and each gets the condition. So no other job has a step after the guard that runs.

Measured, read-only:
- All six repos' `ci.yml`/`rust.yml`/`release.yml` trigger on push to `'ci/**'`, so `ci/docs-only-probe` runs every guarded workflow. After D2 the toolkit's `rust.yml` has no `paths:` filter, so R's own push run exists.
- Under the real guard's rule logic, `design/docs-only-probe.md` matches a `+` rule and no `-` rule in all six `paths/<repo>.txt`.
- Check-run counts with `filter=latest` on today's default tips: dm 19, toolkit 17, key 49, secret **100**, engrave 9, mt 8 (see N1).

## M1 — Minor — step 0 cannot tell same-named guarded jobs apart

Location: probe step 0, "Note every guarded job whose run on R is not `completed/success`".

Failure scenario: in toolkit and secret, `rust.yml`'s `test (ubuntu-latest)` and the generated `release.yml`'s `test (ubuntu-latest)` have the same check-run name. On secret's tip, `check_name='test (ubuntu-latest)'` returns 4 runs from 4 suites. The step 0 `--jq` prints only `name status/conclusion`, with no workflow. Say `release.yml`'s `test (ubuntu-latest)` is red on R and `rust.yml`'s is green. The implementer cannot tell which job the red line belongs to. If it notes only one of them, the guard still sees a red run for both (it queries by name, as r2 M2 measured). Both log `full run (…completed/failure…)`, so the un-noted one hits "Anything else is a stop point". That stop is fail-safe, not a false PASS. It only happens when one of those runs is red on R, which is why this is Minor and not Important.

Fix: add to step 0: "Noting is by check-run name. If any line for a guarded job's name is not `completed/success`, note every guarded job with that name. In toolkit and secret, `rust.yml` `test` and `release.yml` `test` share `test (ubuntu-latest)`."

## M2 — Minor — a noted job's own result, and what to report about it, are not stated

Location: probe step 2, "A job noted in step 0 must instead log … that is the guard working, not a probe failure."

Failure scenario: a job that was red on R does a full run on the probe and probably fails again. The text says nothing about whether that failure is acceptable or is "anything else", and nothing about how to report that the skip went unproven for that job. A literal reader may either stop, or report the probe complete without saying that job's skip went unexercised. Nothing goes wrong either way. The expected line still proves that the job's CHECK_NAME resolves to its check runs, and the gate machine-checks the step conditions.

Fix: append: "A noted job's own pass or fail is not a probe result. List the noted jobs in the probe report as 'skip not exercised (R not green)'."

## N1 — Nit — step 0 lists one page only

Location: step 0, `-f per_page=100` with no `--paginate`.

Secret's current tip has exactly 100 latest check runs. A daily scheduled `cargo fuzz` workflow adds 4 more to the default tip each day, so tomorrow's listing would be cut silently. A fresh merge commit R carries about 20–40 runs, so this only bites if the probe waits days after the merge. A missing line leads to a stop point at worst. Fix: add `--paginate`.

## N2 — Nit — `<NAME>` is the unrendered `name:` value, but logs and step 0 show rendered names

Location: step 2's `'<NAME>' on <R>`. D1 defines `<NAME>` as the literal `name:` text, e.g. `test (${{ matrix.os }})`. The guard log and the step 0 list show `test (ubuntu-latest)`. Fix: write "`<NAME>` as rendered for that matrix leg".

---

Fold accounting: all 4 findings (2M/2N) come from the r2 fold commit e65e5c74, and all from its new step 0 and step 2 text. M1 is r2 M2's same-name measurement surfacing in the new step 0. The fold was authored by the controller session; this reviewer does not know its model or effort.
