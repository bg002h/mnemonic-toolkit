# ci-docs-skip whole-diff review, round 2 (C1 fold re-review)

Reviewer: independent, Opus 5.5, high effort. Read-only; author of neither the fold nor round 1.

## Heads

| repo | PR | head (fold) | parent (r1 head) | file changed |
| --- | --- | --- | --- | --- |
| design (toolkit worktree ci-docs-skip) | — | d78d12d0 (fold 894a690c) | — | PLAN D4, docs-skip-gate.py |
| descriptor-mnemonic | #37 | acaa0c86 | bd9e2d78 | .github/workflows/release.yml |
| mnemonic-toolkit | #103 | 924e0a33 | a41b546c | .github/workflows/release.yml |
| mnemonic-key | #13 | 654d1f9d | 5ac9340 | .github/workflows/release.yml |
| mnemonic-secret | #12 | 02702991 | ecae58c | .github/workflows/release.yml |
| mnemonic-transaction | #3 | 8719a87d | 7915f36 | .github/workflows/release.yml |
| mnemonic-engrave | #19 | 22ef304f | 1423244e | scripts/release-workflows/gen.py |

`C=0 I=0 M=0 N=0`

## Verdict

**GREEN.** The fold closes C1 completely and correctly, and introduces no new defect.

## Findings

None at any severity.

## Fold accounting

0 C / 0 I / 0 M / 0 N stem from the fold (plan + gate by Opus 5.5; the six commits by Sonnet 5.5 high).

## What I checked and how

1. **The six diffs are the plan, byte for byte.** `git -C <wt> show HEAD` on each: every diff is the same 2 insertions and 1 deletion in job `test`. The self-test `if:` becomes `runner.os == 'Linux' && github.event.inputs.tag == ''`, and `if: github.event.inputs.tag == ''` is inserted between `name:` and `id: docs` of the guard step. Engrave's gen.py carries the identical lines; neither condition contains `${{`, so no f-string escaping is needed. This matches PLAN D4 lines 281-286.

2. **No other path checks out a different tree and then runs ci/docs-only\*.**
   - I grepped every workflow in all six worktrees for `ref:`, `repository:`, `workflow_dispatch`, `workflow_call` and `docs-only`.
   - Only release.yml in the five generated repos checks out `github.event.inputs.tag || github.ref`, in three jobs: `test`, `build` and `assemble`.
   - Only `test` runs the guard or the self-test. `build` and `assemble` contain no `docs-only` reference.
   - The other non-default checkouts touch no guarded job:
     - toolkit reproducible-musl-build.yml (`inputs.toolkit_ref`) has no guard.
     - toolkit rust.yml `g6-invariant` (mnemonic-secret at a pinned tag) has no guard step.
     - secret rust.yml `g6-invariant` (toolkit@master) has no guard step.
     - Even where a secondary checkout exists, the guard sits at steps[1], right after the primary checkout, and uses `path:` for the secondary.
   - Engrave's own release.yml has a `workflow_dispatch` with no inputs (`grep -c inputs` = 0). It checks out only the triggering ref and is outside GENERATED, so it correctly gets no RELEASE_IF.
   - No consumer reads the guard's output as `== 'false'`, and no `needs.*.outputs` reads it either. The only reads are `!= 'true'`, plus engrave's `== 'true'` "docs-only verdict" step. So an empty output from a skipped guard means a full run.

3. **The expression is correct.**
   - In all five repos the input is named `tag` and has no `type:`, so it defaults to string with `default: ''`. Only `sign_dry_run` is `type: boolean`.
   - GitHub docs (Expressions → Operators), verbatim: "GitHub performs loose equality comparisons. If the types do not match, GitHub coerces the type to a number." Null becomes `0`, and for strings "Note: empty string returns `0`." So `null == ''` is true.
   - Evidence from live runs on the fold heads (`gh run view <id> --json jobs`), for both `pull_request` and branch `push` events, where `github.event.inputs` is absent:
     - The guard step concluded **success** (it ran) in every release `test` leg of all five generated repos: dm 38195067313 and 38195064357; tk 38195075359 and 38195072056; mk 38195069620 and 38195066539; ms 38195072080 and 38195068348; mt 38195073167 and 38195070534.
     - The self-test was **success** on ubuntu and **skipped** on macos, as intended.
   - A tag push leaves `github.event.inputs` absent in the same way, so it evaluates identically.
   - A dispatch with an empty tag gives `'' == ''`, so the guard runs.
   - A dispatch with a tag set gives `'<tag>' == ''`, which is false, so both steps are skipped and every `!= 'true'` step runs. That is the full run C1 requires.

4. **The gate fold encodes the plan faithfully.**
   - Dict side: `guarded_step(name, guard_if)` rebuilds the step as `{"name", "if", **rest}`. GUARD's order is name, id, shell, env, run, so the result is name, if, id, …. The comparison is on parsed YAML and does not depend on key order. Duplicate keys are rejected by the loader (line 171).
   - Textual side: `GUARD_TEXT.replace("\n        id: docs", "\n        if: {gif}\n        id: docs", 1)` produces exactly the plan's text. SELFTEST_LINUX_TEXT matches the plan's self-test line.
   - RELEASE_IF is applied only to `release.yml:test` for repos in GENERATED, which is the five and excludes engrave. SELFTEST (for ci.yml/rust.yml) is unchanged.
   - The two new textual mutations assert their target line occurs exactly once before replacing it, then require RED.
   - I relied on the controller's stated results: 41-case PASS, and RED on the unfixed r1 dm head. I did not re-run them.

5. **No regression on the tag-push and PR paths.** Point 3 is direct evidence that the guard and Linux self-test still run on pull_request and push events in all five repos and in engrave, whose `test (rust + go)` guard and self-test show success in 38195078129 and 38195074409. The plan's table (line 47: "tags, dispatch, schedule | none: full run") and the rationale added to D4 agree with this behaviour.
