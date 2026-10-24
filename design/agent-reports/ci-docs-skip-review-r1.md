r1: 0C/0I/1M/0N

# L3 delta review r1: mnemonic-toolkit PR #112 (feat/ci-docs-skip, head 81668f0e)

Cursory delta review per brief `ci-skip-constellation/briefs/l3-mnemonic-toolkit-r1.md`.
Shared files, gate, L1/L2 were settled and not re-reviewed.

## 1. `ci/docs-only-paths.txt` additions and `ci/code-docs-exempt.txt`

- New `- ^docs/verify-reproducibility\.md$`: named in an error message at
  `ci/repro/vendor-freshness.sh:162`. Correct (conservative) protection.
- New `- ^CLAUDE\.md$`: named in `crates/mnemonic-toolkit/tests/design_artifacts_presence.rs`
  (lines 6, 21), a test the guarded `test` job runs. Correct; the `-` rule overrides
  `+ ^[^/]+\.md$`.
- Exempt files: grep of every workflow, `ci/*.sh`, `scripts/*.sh` and `crates/**/*.rs`
  finds no guarded step running `scripts/followup-reconcile.sh`,
  `scripts/spec-citation-gate.py`, `ci/doc-flag-lint.test.sh`, `ci/doc-gate-guard.test.sh`
  or `ci/doc-gate-paths.py` (the last three are run only from `manual.yml` and
  `ci/doc-gate-guard.sh`). None of the six exempt workflows contains `docs-only.sh`
  (0 hits each). No Rust test reads any of them (the only `.github/workflows` mentions
  in tests are in messages/comments about other workflows).
- `bash ci/code-docs-check.sh` on the head: `ok (431 files scanned, 2484 docs files)`.

No finding.

## 2. New step insertion sites

Matches SPEC §2.3: `rust.yml` `clippy` (after `Self-test the docs-only guard`, no `if:`)
and `release.yml` `test` (same `if: runner.os == 'Linux' && github.event.inputs.tag == ''`
as the self-test, as its second line). Both sit before the `Docs-only change?` step.
No finding.

## 3. `release.yml` `test` job

- `ref:` = `${{ github.event.inputs.tag || (startsWith(github.ref, 'refs/tags/') && github.ref) || github.sha }}`,
  `name:` = `release test (${{ matrix.os }})` + ` (tag)` suffix, `CHECK_NAME: release test (${{ matrix.os }})`.
  All three match SPEC §2.1 verbatim.
- Job id `test` unchanged; `needs: [test, build]` (line 249) still resolves.
- No script, doc or workflow refers to the old release-job name. `scripts/push-via-staging.sh`
  reads required contexts from branch protection at run time (no hard-coded names).
- Live branch protection on master: `["examples","test (ubuntu-latest)","clippy","manual",
  "quickstart","technical-manual","manual-gui","test (macos-latest)"]`. Both `test (...)`
  contexts are still produced by `rust.yml`'s `test` job (matrix ubuntu-latest, macos-latest).

**M1.** Before this PR, `release.yml`'s ubuntu job also emitted the required context
`test (ubuntu-latest)`. GitHub and `push-via-staging.sh` (latest check-run id wins) judged
whichever of the two runs finished last, so the release test was racily gating and could
also shadow a `rust.yml` failure. The rename fixes the shadowing but leaves `release test
(ubuntu-latest)` not required at all; SPEC §2.1's "The required checks are unaffected" is
silent on this. Suggest: after the first master push produces the new name, consider adding
`release test (ubuntu-latest)` to master's required contexts (release.yml runs on every
pull_request with no path filter, so it would not wedge).

## 4. PR description

Accurate: names rule A′, the code-docs check, the checkout/rename and why, that
`.github/` and `ci/` change, and the GREEN spec. Draft, head 81668f0e. The omission of the
repo's own paths/exempt lines is the already-filed PR 2/PR 4 nit; not re-raised. No finding.
