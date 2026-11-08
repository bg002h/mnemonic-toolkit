**⚠ 1 Important: S-1 pinning location undecided (STOP); the spec gate passes otherwise.**

# spec-vendor-freshness author report

Brief: coordinator briefs/spec-vendor-freshness.md. Worktree /scratch/cache/spec-vf,
branch spec/vendor-freshness-shared from mnemonic-toolkit origin/master 3753b5c7.

## Findings as I go
- shared-ci.sha256 is byte-identical in SIX public repos (toolkit, engrave,
  transaction, key, secret, dm; sha256 prefix cc22c1754b9f), and ci/shared-drift.sh
  compares all five files across all five other repos. engrave and transaction have
  no vendor/. The audit's pinning design (hash lines in each repo's shared-ci.sha256,
  drift for key/secret/dm only) cannot hold as written: STOP S-1.
- mnemonic-key's Cargo.lock has NO git source; toolkit's script fails closed on zero
  git sources, so the survivor must pass a lock with no git source and an empty data
  file (decided: it does).
- mr lacks docs/verify-reproducibility.md, which toolkit's refusal text names; the
  shared text drops the docs pointer (decided).
- Measured RED of the current scripts against the new test rows (gate C6): key,
  secret, dm and mr exit 0 on a hand edit to a registry crate (A5); secret and dm
  exit 0 on the F-354 shape (G4) and on a pin moved to an unfetchable rev (G5);
  toolkit's own current gate exits 0 on an unlisted file planted in a git source (G8).
- G9 with version 0.99.0 redded in check (1) in dm (md-codec requires
  miniscript ^13.0.0); the row now uses <major>.<minor>.<patch+1000>, semver-compatible.
- Proto headers no longer name ci/shared-ci.sha256 (depends on S-1): "pinned by its sha256".
- Decided beyond the audit's "Callers unchanged": the test runs in every repo's CI
  (key/secret/dm gain a "Mutation tests for the gate" step and ci/repro/** path
  filters; mr ci.yml gains one run line). A shared test that never runs proves nothing.
- Gate fixes (gate4-gate6). C2 died under pipefail when a data file has no rows
  (key); grep is now guarded. C6 always FAILed: gate.sh runs under `set -e`, so the
  backgrounded subshell quit on the test's non-zero exit before writing its
  `exit N` line. Every run line now captures `rc` explicitly (the green and test
  lines had the same latent bug, hidden only because they exit 0).
- Spec prose: the RED list now names G9 (secret and dm) and A8 to A12 (was A11).

## Final state
- Gate: `bash design/spec-vendor-freshness/gate.sh /scratch/cache/spec-vf-wt <log>`
  -> `gate: PASS`, rc=0, 53 PASS lines (run gate6); all five repo worktrees clean
  after. Every repo's test: 22 passed, 0 failed (key: 13 passed, 9 skipped, G rows
  with no git source). Spec blocks vf:red-*, vf:mutant-M1..M5, vf:proto-sha256
  and vf:refusal-* are filled from the gate's own logs by fill.py.
- `ci/code-docs-check.sh` ok on the branch (design/ is docs-only).
- Nothing committed or pushed. The spec files are untracked, with intent-to-add
  entries in the index (git add -N).

## Open (STOP)
- S-1 (Important, blocks pinning): shared-ci.sha256 and ci/shared-drift.sh span six
  repos, two of them (engrave, transaction) with no vendor/. Where do the two hash
  lines live, and which repos does drift compare them across? Section 10 of the
  spec states the question and leaves it undecided.

Counts: 0 Critical, 1 Important (S-1, an undecided product point), 0 Minor.
