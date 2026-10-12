# CI docs-skip implementation report (2026-10-11)

Plan: design/ci-docs-skip/PLAN_ci_docs_skip_2026-10-11.md (toolkit branch design/ci-docs-skip, head 1528e52d). All six PRs are DRAFT and unmerged. No stop point was hit. No post-merge probe was run.

Method note: the workflow text edits for D1/D2/D4/D5 were applied by a line-based script (scratchpad edit_wf.py, edit_engrave.py) transcribing the plan; D4 gen.py/emit.py and the engrave test-job detect step by script too. Every result was then checked by the plan's gate and by reading git diff.

## descriptor-mnemonic
- Base B (origin default branch, contains hardening merge): `0da734f2eff6e4bdc1c1676bb9241ee59431ee32`
- Commit: `bd9e2d7810153e5214706bca405f32504b9e970d` on branch ci/docs-skip (worktree /scratch/code/shibboleth/dm-worktrees/ci-docs-skip)
- PR (draft): https://github.com/bg002h/descriptor-mnemonic/pull/37
- After-commit check: git status --porcelain empty; git diff --name-only B HEAD equals the gate's changed-files list (MATCH)
- actionlint before: 0 lines; after: 0 lines (no new findings)
- Gate output:
```
files descriptor-mnemonic: ci/docs-only.sh identical
files descriptor-mnemonic: ci/docs-only.test.sh identical
files descriptor-mnemonic: ci/docs-only-paths.txt identical
files descriptor-mnemonic: ci/docs-only.sh mode 100755
files descriptor-mnemonic: ci/docs-only.test.sh mode 100755
files descriptor-mnemonic: ci/docs-only.sh: eol: lf
files descriptor-mnemonic: ci/docs-only.test.sh: eol: lf
files descriptor-mnemonic: ci/docs-only-paths.txt: eol: lf
GATE: GREEN
paths-check descriptor-mnemonic: design/SPEC_x.md -> docs_only=true
paths-check descriptor-mnemonic: design/agent-reports/r.md -> docs_only=true
paths-check descriptor-mnemonic: docs/a.md -> docs_only=true
paths-check descriptor-mnemonic: bip/bip-x.mediawiki -> docs_only=true
paths-check descriptor-mnemonic: README.md -> docs_only=true
paths-check descriptor-mnemonic: CHANGELOG.md -> docs_only=true
paths-check descriptor-mnemonic: design/display-grouping-vectors.tsv -> docs_only=false
paths-check descriptor-mnemonic: design/display-grouping-vectors.tsv.sha256 -> docs_only=false
paths-check descriptor-mnemonic: crates/md-cli/README.md -> docs_only=false
paths-check descriptor-mnemonic: vendor/x/README.md -> docs_only=false
paths-check descriptor-mnemonic: ci/docs-only-paths.txt -> docs_only=false
paths-check descriptor-mnemonic: .github/workflows/ci.yml -> docs_only=false
paths-check descriptor-mnemonic: Cargo.lock -> docs_only=false
GATE: GREEN
descriptor-mnemonic: files=6 jobs=7 steps=30 combined=2 selftests=2
descriptor-mnemonic: changed files (6): .gitattributes .github/workflows/ci.yml .github/workflows/release.yml ci/docs-only-paths.txt ci/docs-only.sh ci/docs-only.test.sh
GATE: GREEN
```
- CI (`gh pr checks --watch` exit 0). Final table:
```
     32 pass
      2 skipping
      2 non-pass: assemble + release (skipping)
```
- Guard log line seen (every guarded job, both the pull_request and push runs; 22 job logs):
```
     22 docs-only: full run (not documentation: .gitattributes)
```
  Guarded job names that logged it:
  - cargo clippy
  - cargo doc
  - cargo fmt
  - cargo test (macos-latest)
  - cargo test (ubuntu-latest)
  - cargo test (windows-latest)
  - freebsd compile-gate (whole-crate)
  - musl compile/test (aarch64-unknown-linux-musl)
  - musl compile/test (x86_64-unknown-linux-musl)
  - test (macos-latest)
  - test (ubuntu-latest)

## mnemonic-key
- Base B (origin default branch, contains hardening merge): `3c7a403fc254f2224fee4b70122a80521a874d72`
- Commit: `5ac93405a97bd81be2280870c6405733aa86d70b` on branch ci/docs-skip (worktree /scratch/code/shibboleth/mk-worktrees/ci-docs-skip)
- PR (draft): https://github.com/bg002h/mnemonic-key/pull/13
- After-commit check: git status --porcelain empty; git diff --name-only B HEAD equals the gate's changed-files list (MATCH)
- actionlint before: 0 lines; after: 0 lines (no new findings)
- Gate output:
```
files mnemonic-key: ci/docs-only.sh identical
files mnemonic-key: ci/docs-only.test.sh identical
files mnemonic-key: ci/docs-only-paths.txt identical
files mnemonic-key: ci/docs-only.sh mode 100755
files mnemonic-key: ci/docs-only.test.sh mode 100755
files mnemonic-key: ci/docs-only.sh: eol: lf
files mnemonic-key: ci/docs-only.test.sh: eol: lf
files mnemonic-key: ci/docs-only-paths.txt: eol: lf
GATE: GREEN
paths-check mnemonic-key: design/SPEC_x.md -> docs_only=true
paths-check mnemonic-key: docs/a.md -> docs_only=true
paths-check mnemonic-key: bip/bip-mnemonic-key.mediawiki -> docs_only=true
paths-check mnemonic-key: README.md -> docs_only=true
paths-check mnemonic-key: design/SPEC_mk_v0_1.md -> docs_only=false
paths-check mnemonic-key: design/display-grouping-vectors.tsv -> docs_only=false
paths-check mnemonic-key: design/display-grouping-vectors.tsv.sha256 -> docs_only=false
paths-check mnemonic-key: crates/mk-cli/CHANGELOG.md -> docs_only=false
paths-check mnemonic-key: vendor/x/README.md -> docs_only=false
GATE: GREEN
mnemonic-key: files=5 jobs=6 steps=28 combined=2 selftests=2
mnemonic-key: changed files (6): .gitattributes .github/workflows/ci.yml .github/workflows/release.yml ci/docs-only-paths.txt ci/docs-only.sh ci/docs-only.test.sh
GATE: GREEN
```
- CI (`gh pr checks --watch` exit 0). Final table:
```
     38 pass
      4 skipping
      2 non-pass: assemble + release (skipping)
      2 non-pass: release-on-tag (skipping)
```
- Guard log line seen (every guarded job, both the pull_request and push runs; 32 job logs):
```
     32 docs-only: full run (not documentation: .gitattributes)
```
  Guarded job names that logged it:
  - build (1.85 on macos-latest)
  - build (1.85 on ubuntu-latest)
  - build (1.85 on windows-latest)
  - build (beta on macos-latest)
  - build (beta on ubuntu-latest)
  - build (beta on windows-latest)
  - build (stable on macos-latest)
  - build (stable on ubuntu-latest)
  - build (stable on windows-latest)
  - fmt (pinned 1.95.0)
  - freebsd compile-gate (whole-crate)
  - musl compile/test (aarch64-unknown-linux-musl)
  - musl compile/test (x86_64-unknown-linux-musl)
  - test (macos-latest)
  - test (ubuntu-latest)
  - vectors-roundtrip

## mnemonic-secret
- Base B (origin default branch, contains hardening merge): `5a57fe8a28c43333a71555ac852b38730ec2709f`
- Commit: `ecae58cdbc2d728f0c1f54927459489dfeafb2c7` on branch ci/docs-skip (worktree /scratch/code/shibboleth/ms-worktrees/ci-docs-skip)
- PR (draft): https://github.com/bg002h/mnemonic-secret/pull/12
- After-commit check: git status --porcelain empty; git diff --name-only B HEAD equals the gate's changed-files list (MATCH)
- actionlint before: 0 lines; after: 0 lines (no new findings)
- Gate output:
```
files mnemonic-secret: ci/docs-only.sh identical
files mnemonic-secret: ci/docs-only.test.sh identical
files mnemonic-secret: ci/docs-only-paths.txt identical
files mnemonic-secret: ci/docs-only.sh mode 100755
files mnemonic-secret: ci/docs-only.test.sh mode 100755
files mnemonic-secret: ci/docs-only.sh: eol: lf
files mnemonic-secret: ci/docs-only.test.sh: eol: lf
files mnemonic-secret: ci/docs-only-paths.txt: eol: lf
GATE: GREEN
paths-check mnemonic-secret: design/SPEC_x.md -> docs_only=true
paths-check mnemonic-secret: docs/verify-reproducibility.md -> docs_only=true
paths-check mnemonic-secret: MIGRATION.md -> docs_only=true
paths-check mnemonic-secret: design/display-grouping-vectors.tsv -> docs_only=false
paths-check mnemonic-secret: design/display-grouping-vectors.tsv.sha256 -> docs_only=false
paths-check mnemonic-secret: crates/ms-cli/README.md -> docs_only=false
paths-check mnemonic-secret: vendor/x/README.md -> docs_only=false
GATE: GREEN
mnemonic-secret: files=5 jobs=11 steps=35 combined=2 selftests=2
mnemonic-secret: changed files (6): .gitattributes .github/workflows/release.yml .github/workflows/rust.yml ci/docs-only-paths.txt ci/docs-only.sh ci/docs-only.test.sh
GATE: GREEN
```
- CI (`gh pr checks --watch` exit 0). Final table:
```
     34 pass
      2 skipping
      2 non-pass: assemble + release (skipping)
```
- Guard log line seen (every guarded job, both the pull_request and push runs; 26 job logs):
```
     26 docs-only: full run (not documentation: .gitattributes)
```
  Guarded job names that logged it:
  - clippy
  - clippy (ms-codec)
  - fmt (pinned 1.95.0)
  - freebsd compile-gate (whole-crate)
  - history purge (recipes RUN under real shells)
  - miri (mlock unsafe)
  - musl compile/test (aarch64-unknown-linux-musl)
  - musl compile/test (x86_64-unknown-linux-musl)
  - test (macos-latest)
  - test (ms-codec)
  - test (release, ubuntu-latest, mlock einval)
  - test (ubuntu-latest)

## mnemonic-toolkit
- Base B (origin default branch, contains hardening merge): `fb2d1a9aa41830e9b6e5c115257cce31d748a322`
- Commit: `a41b546c25509de3d5bb92f230c16c23746eb2b7` on branch ci/docs-skip (worktree /scratch/code/shibboleth/tk-worktrees/ci-docs-skip-impl)
- PR (draft): https://github.com/bg002h/mnemonic-toolkit/pull/103
- After-commit check: git status --porcelain empty; git diff --name-only B HEAD equals the gate's changed-files list (MATCH)
- actionlint before: 0 lines; after: 0 lines (no new findings)
- Gate output:
```
files mnemonic-toolkit: ci/docs-only.sh identical
files mnemonic-toolkit: ci/docs-only.test.sh identical
files mnemonic-toolkit: ci/docs-only-paths.txt identical
files mnemonic-toolkit: ci/docs-only.sh mode 100755
files mnemonic-toolkit: ci/docs-only.test.sh mode 100755
files mnemonic-toolkit: ci/docs-only.sh: eol: lf
files mnemonic-toolkit: ci/docs-only.test.sh: eol: lf
files mnemonic-toolkit: ci/docs-only-paths.txt: eol: lf
GATE: GREEN
paths-check mnemonic-toolkit: design/SPEC_x.md -> docs_only=true
paths-check mnemonic-toolkit: docs/manual/src/a.md -> docs_only=true
paths-check mnemonic-toolkit: CHANGELOG.md -> docs_only=true
paths-check mnemonic-toolkit: README.md -> docs_only=false
paths-check mnemonic-toolkit: design/SPEC_convert_v0_6.md -> docs_only=false
paths-check mnemonic-toolkit: design/PLAN_v0_26_0_three_way_merge.md -> docs_only=false
paths-check mnemonic-toolkit: design/display-grouping-vectors.tsv -> docs_only=false
paths-check mnemonic-toolkit: crates/mnemonic-toolkit/README.md -> docs_only=false
paths-check mnemonic-toolkit: .examples-build/Examples.md -> docs_only=false
paths-check mnemonic-toolkit: vendor/x/README.md -> docs_only=false
paths-check mnemonic-toolkit: rust-toolchain.toml -> docs_only=false
paths-check mnemonic-toolkit: ci/docs-only.sh -> docs_only=false
GATE: GREEN
mnemonic-toolkit: files=19 jobs=9 steps=37 combined=3 selftests=2
mnemonic-toolkit: changed files (6): .gitattributes .github/workflows/release.yml .github/workflows/rust.yml ci/docs-only-paths.txt ci/docs-only.sh ci/docs-only.test.sh
GATE: GREEN
```
- CI (`gh pr checks --watch` exit 0). Final table:
```
     48 pass
     12 skipping
      2 non-pass: assemble + release (skipping)
      2 non-pass: build (skipping)
      2 non-pass: lint (skipping)
      2 non-pass: release (skipping)
      2 non-pass: verify-examples-gui (skipping)
      2 non-pass: verify-examples (skipping)
```
- Guard log line seen (every guarded job, both the pull_request and push runs; 26 job logs):
```
     26 docs-only: full run (not documentation: .gitattributes)
```
  Guarded job names that logged it:
  - clippy
  - fmt (pinned 1.95.0)
  - install.sh harnesses (man-step + MSRV guard)
  - lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest)
  - lib cross-platform check (x86_64-pc-windows-msvc, windows-latest)
  - lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest)
  - miri (mlock unsafe)
  - musl build+test (aarch64-unknown-linux-musl)
  - musl build+test (x86_64-unknown-linux-musl)
  - test (macos-latest)
  - test (release, ubuntu-latest, mlock einval)
  - test (ubuntu-latest)

## mnemonic-transaction
- Base B (origin default branch, contains hardening merge): `9e4539cda57567277f49a521a6afedc15043d0fd`
- Commit: `7915f365b7416975318b820e9ef428d771c51de2` on branch ci/docs-skip (worktree /scratch/code/shibboleth/mt-worktrees/ci-docs-skip)
- PR (draft): https://github.com/bg002h/mnemonic-transaction/pull/3
- After-commit check: git status --porcelain empty; git diff --name-only B HEAD equals the gate's changed-files list (MATCH)
- actionlint before: 0 lines; after: 0 lines (no new findings)
- Gate output:
```
files mnemonic-transaction: ci/docs-only.sh identical
files mnemonic-transaction: ci/docs-only.test.sh identical
files mnemonic-transaction: ci/docs-only-paths.txt identical
files mnemonic-transaction: ci/docs-only.sh mode 100755
files mnemonic-transaction: ci/docs-only.test.sh mode 100755
files mnemonic-transaction: ci/docs-only.sh: eol: lf
files mnemonic-transaction: ci/docs-only.test.sh: eol: lf
files mnemonic-transaction: ci/docs-only-paths.txt: eol: lf
GATE: GREEN
paths-check mnemonic-transaction: design/SPEC_mt_v0_1.md -> docs_only=true
paths-check mnemonic-transaction: README.md -> docs_only=true
paths-check mnemonic-transaction: design/vectors/mt1_v1_vectors.json -> docs_only=true
paths-check mnemonic-transaction: crates/mt-codec/src/test_vectors/mt1_v1.json -> docs_only=false
paths-check mnemonic-transaction: scripts/check-provenance.sh -> docs_only=false
GATE: GREEN
mnemonic-transaction: files=2 jobs=1 steps=5 combined=0 selftests=1
mnemonic-transaction: changed files (5): .gitattributes .github/workflows/release.yml ci/docs-only-paths.txt ci/docs-only.sh ci/docs-only.test.sh
GATE: GREEN
```
- CI (`gh pr checks --watch` exit 0). Final table:
```
     14 pass
      2 skipping
      2 non-pass: assemble + release (skipping)
```
- Guard log line seen (every guarded job, both the pull_request and push runs; 2 job logs):
```
      2 docs-only: full run (not documentation: .gitattributes)
```
  Guarded job names that logged it:
  - test (ubuntu-latest)

## mnemonic-engrave
- Base B (origin default branch, contains hardening merge): `a2e129d313ee092e8ae86b6482159ab3e54274c2`
- Commit: `1423244e98041bfe131203f3fa61a37276d53f36` on branch ci/docs-skip (worktree /scratch/code/shibboleth/me-worktrees/ci-docs-skip)
- PR (draft): https://github.com/bg002h/mnemonic-engrave/pull/19
- After-commit check: git status --porcelain empty; git diff --name-only B HEAD equals the gate's changed-files list (MATCH)
- actionlint before: 0 lines; after: 0 lines (no new findings)
- Gate output:
```
files mnemonic-engrave: ci/docs-only.sh identical
files mnemonic-engrave: ci/docs-only.test.sh identical
files mnemonic-engrave: ci/docs-only-paths.txt identical
files mnemonic-engrave: ci/docs-only.sh mode 100755
files mnemonic-engrave: ci/docs-only.test.sh mode 100755
files mnemonic-engrave: ci/docs-only.sh: eol: lf
files mnemonic-engrave: ci/docs-only.test.sh: eol: lf
files mnemonic-engrave: ci/docs-only-paths.txt: eol: lf
GATE: GREEN
paths-check mnemonic-engrave: design/SPEC_x.md -> docs_only=true
paths-check mnemonic-engrave: design/agent-reports/r.md -> docs_only=true
paths-check mnemonic-engrave: README.md -> docs_only=true
paths-check mnemonic-engrave: design/hardware/retail-otp.json -> docs_only=false
paths-check mnemonic-engrave: design/hardware/captures/a.txt -> docs_only=false
paths-check mnemonic-engrave: docs/a.md -> docs_only=false
paths-check mnemonic-engrave: scripts/refugium-otp.sh -> docs_only=false
paths-check mnemonic-engrave: flake.lock -> docs_only=false
GATE: GREEN
mnemonic-engrave: files=1 jobs=2 steps=18 combined=0 selftests=1
mnemonic-engrave: changed files (7): .gitattributes .github/workflows/release.yml ci/docs-only-paths.txt ci/docs-only.sh ci/docs-only.test.sh scripts/release-workflows/emit.py scripts/release-workflows/gen.py
GATE: GREEN
release-gen descriptor-mnemonic: identical
release-gen mnemonic-secret: identical
release-gen mnemonic-key: identical
release-gen mnemonic-transaction: identical
release-gen mnemonic-toolkit: identical
GATE: GREEN
```
- CI (`gh pr checks --watch` exit 0). Final table:
```
     16 pass
      2 skipping
      2 non-pass: assemble + sign + release (skipping)
```
- Guard log line seen (every guarded job, both the pull_request and push runs; 4 job logs):
```
      4 docs-only: full run (not documentation: .gitattributes)
```
  Guarded job names that logged it:
  - picotool argv probe
  - test (rust + go)

## Notes
- Non-pass checks are all `skipping`: tag/dispatch-only jobs (assemble + release, release-on-tag, release, lint, verify-examples*, build in toolkit's skipped workflow) that the plan does not guard. Zero failures.
- Jobs with no guard line are exactly the unguarded ones: build matrices, g6-invariant (toolkit, secret), toolkit non-rust.yml workflows, mnemonic-transaction ci.yml 'test (rust)'.
- Both a pull_request and a push run exist per commit; both show the full-run line.
- Per-job durations were not recorded (the plan asks only for the probe).

## Stop points hit
None.
