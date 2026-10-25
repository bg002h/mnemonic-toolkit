
# Re-run after: ci: doc gates watch rust-toolchain.toml; doc-gate A'

PR #113. Copies ci-skip-constellation reference/after-b/T1 (SPEC-after-b.md).

## Change

```
 .github/workflows/examples.yml         |   7 +-
 .github/workflows/manual-gui.yml       |   9 +-
 .github/workflows/manual.yml           |   7 +-
 .github/workflows/quickstart.yml       |   7 +-
 .github/workflows/technical-manual.yml |   7 +-
 ci/doc-gate-guard.sh                   | 154 +++++++++++++++++++++---
 ci/doc-gate-guard.test.sh              | 210 ++++++++++++++++++++++++++++++++-
 7 files changed, 377 insertions(+), 24 deletions(-)
```

## Gate

`make-after-b-reference --check: PASS (12 files)`

## Local checks

```
ci/docs-only.sh: OK
ci/docs-only.test.sh: OK
ci/code-docs-check.sh: OK
ci/code-docs-check.test.sh: OK
docs-only.test.sh: 73 passed, 0 failed
code-docs-check.test.sh: 41 passed, 0 failed
code-docs check: ok (431 files scanned, 2485 docs files)
158 passed, 0 failed
PASS T1 .github/workflows/examples.yml: exactly the §3.1-§3.2 edits
PASS T1 .github/workflows/manual.yml: exactly the §3.1-§3.2 edits
PASS T1 .github/workflows/quickstart.yml: exactly the §3.1-§3.2 edits
PASS T1 .github/workflows/technical-manual.yml: exactly the §3.1-§3.2 edits
PASS T1 .github/workflows/manual-gui.yml: exactly the §3.1-§3.2 edits
after-b-check T1: PASS
```

## L1 (code head 13c33a5c3ada992fa85c84cf9338b256f6b88ac6)

```
run 40221064398    examples                       pull_request   success    ok
run 40221064383    quickstart                     pull_request   success    ok
run 40221064397    technical-manual               pull_request   success    ok
run 40221064428    release                        pull_request   success    ok
run 40221064402    manual-gui                     pull_request   success    ok
run 40221064441    rust                           pull_request   success    ok
run 40221064380    manual                         pull_request   success    ok
run 40221064382    sibling-pin-check              pull_request   success    ok
run 40221060356    sibling-pin-check              push           success    ok
  examples/examples                                success    ok
  quickstart/quickstart                            success    ok
  technical-manual/technical-manual                success    ok
  release/build mnemonic (macos-aarch64)           success    ok
  release/release test (ubuntu-latest)             success    ok
  release/build mnemonic (macos-x86_64)            success    ok
  release/build mnemonic (windows-x86_64)          success    ok
  release/assemble + release                       skipped    ok (allowed skip)
  manual-gui/changes                               success    ok
  manual-gui/build                                 success    ok
  manual-gui/lint                                  success    ok
  manual-gui/verify-examples-gui                   success    ok
  manual-gui/verify-examples                       success    ok
  manual-gui/manual-gui                            success    ok
  manual-gui/release                               skipped    ok (allowed skip)
  rust/miri (mlock unsafe)                         success    ok
  rust/test (macos-latest)                         success    ok
  rust/fmt (pinned 1.95.0)                         success    ok
  rust/test (release, ubuntu-latest, mlock einval) success    ok
  rust/g6 invariant (cross-repo mlock.rs)          success    ok
  rust/musl build+test (aarch64-unknown-linux-musl) success    ok
  rust/lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest) success    ok
  rust/install.sh harnesses (man-step + MSRV guard) success    ok
  rust/test (ubuntu-latest)                        success    ok
  rust/clippy                                      success    ok
  rust/musl build+test (x86_64-unknown-linux-musl) success    ok
  rust/lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest) success    ok
  rust/lib cross-platform check (x86_64-pc-windows-msvc, windows-latest) success    ok
  manual/manual                                    success    ok
  sibling-pin-check/sibling pins match install.sh  success    ok
  sibling-pin-check/sibling pins match install.sh  success    ok
pr-checks: PASS (mnemonic-toolkit 13c33a5c3ada992fa85c84cf9338b256f6b88ac6)
```

## L2 (docs head fb67da362994cce73789e6b5c4f855e08ca6cef3, previous head 13c33a5c3ada992fa85c84cf9338b256f6b88ac6)

- run 40222098721 job 121255686829 `examples/examples`: `doc-gate-guard: relevant=false (A': no watched path changed since the previous PR head 13c33a5c3ada992fa85c84cf9338b256f6b88ac6, the merged base 101f9c7530c24ccc5dc663f5af2fe3bcb2ad4471 is an ancestor of it, and 'examples' passed on it)`
- run 40222098685 job 121255686658 `manual/manual`: `doc-gate-guard: relevant=false (A': no watched path changed since the previous PR head 13c33a5c3ada992fa85c84cf9338b256f6b88ac6, the merged base 101f9c7530c24ccc5dc663f5af2fe3bcb2ad4471 is an ancestor of it, and 'manual' passed on it)`
- run 40222098707 job 121255686784 `quickstart/quickstart`: `doc-gate-guard: relevant=false (A': no watched path changed since the previous PR head 13c33a5c3ada992fa85c84cf9338b256f6b88ac6, the merged base 101f9c7530c24ccc5dc663f5af2fe3bcb2ad4471 is an ancestor of it, and 'quickstart' passed on it)`
- run 40222098682 job 121255686647 `technical-manual/technical-manual`: `doc-gate-guard: relevant=false (A': no watched path changed since the previous PR head 13c33a5c3ada992fa85c84cf9338b256f6b88ac6, the merged base 101f9c7530c24ccc5dc663f5af2fe3bcb2ad4471 is an ancestor of it, and 'technical-manual' passed on it)`
- run 40222098745 job 121255686868 `manual-gui/changes`: `doc-gate-guard: relevant=false (A': no watched path changed since the previous PR head 13c33a5c3ada992fa85c84cf9338b256f6b88ac6, the merged base 101f9c7530c24ccc5dc663f5af2fe3bcb2ad4471 is an ancestor of it, and 'manual-gui' passed on it)`

# Re-run after: ci: guard the release build jobs with ci/docs-only.sh

PR #114. Copies ci-skip-constellation reference/after-b/B-tk (SPEC-after-b.md).

## Change

```
 .github/workflows/release.yml | 30 ++++++++++++++++++++++++------
 1 file changed, 24 insertions(+), 6 deletions(-)
```

## Gate

`make-after-b-reference --check: PASS (12 files)`

## Local checks

```
ci/docs-only.sh: OK
ci/docs-only.test.sh: OK
ci/code-docs-check.sh: OK
ci/code-docs-check.test.sh: OK
docs-only.test.sh: 73 passed, 0 failed
code-docs-check.test.sh: 41 passed, 0 failed
code-docs check: ok (431 files scanned, 2487 docs files)
PASS B-tk .github/workflows/release.yml: exactly the §4.2 edits to build
after-b-check B-tk: PASS
```

## L1 (code head 642f0588100fde1c4d6124d65e2cea07132e3296)

```
run 40233765262    examples                       pull_request   success    ok
run 40233759945    manual                         pull_request   success    ok
run 40233759908    technical-manual               pull_request   success    ok
run 40233759896    manual-gui                     pull_request   success    ok
run 40233759809    quickstart                     pull_request   success    ok
run 40233759966    sibling-pin-check              pull_request   success    ok
run 40233759920    release                        pull_request   success    ok
run 40233759882    rust                           pull_request   success    ok
run 40233756272    sibling-pin-check              push           success    ok
  examples/examples                                success    ok
  manual/manual                                    success    ok
  technical-manual/technical-manual                success    ok
  manual-gui/changes                               success    ok
  manual-gui/manual-gui                            success    ok
  manual-gui/verify-examples                       skipped    ok (allowed skip)
  manual-gui/verify-examples-gui                   skipped    ok (allowed skip)
  manual-gui/build                                 skipped    ok (allowed skip)
  manual-gui/lint                                  skipped    ok (allowed skip)
  manual-gui/release                               skipped    ok (allowed skip)
  quickstart/quickstart                            success    ok
  sibling-pin-check/sibling pins match install.sh  success    ok
  release/build mnemonic (macos-x86_64)            success    ok
  release/build mnemonic (windows-x86_64)          success    ok
  release/release test (ubuntu-latest)             success    ok
  release/build mnemonic (macos-aarch64)           success    ok
  release/assemble + release                       skipped    ok (allowed skip)
  rust/install.sh harnesses (man-step + MSRV guard) success    ok
  rust/musl build+test (x86_64-unknown-linux-musl) success    ok
  rust/fmt (pinned 1.95.0)                         success    ok
  rust/miri (mlock unsafe)                         success    ok
  rust/test (macos-latest)                         success    ok
  rust/test (release, ubuntu-latest, mlock einval) success    ok
  rust/musl build+test (aarch64-unknown-linux-musl) success    ok
  rust/lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest) success    ok
  rust/lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest) success    ok
  rust/test (ubuntu-latest)                        success    ok
  rust/lib cross-platform check (x86_64-pc-windows-msvc, windows-latest) success    ok
  rust/clippy                                      success    ok
  rust/g6 invariant (cross-repo mlock.rs)          success    ok
  sibling-pin-check/sibling pins match install.sh  success    ok
pr-checks: PASS (mnemonic-toolkit 642f0588100fde1c4d6124d65e2cea07132e3296)
```

## L2 (docs head bc9ce01474ab2787a02bc32bd758eb8a8aba727e, previous head 642f0588100fde1c4d6124d65e2cea07132e3296)

- run 40234810977 job 121293984951 `release/release test (ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'release test (ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40234810977 job 121293985146 `release/build mnemonic (macos-aarch64)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'build mnemonic (macos-aarch64)' passed on it; skipping the guarded steps`
- run 40234810977 job 121293985160 `release/build mnemonic (macos-x86_64)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'build mnemonic (macos-x86_64)' passed on it; skipping the guarded steps`
- run 40234810977 job 121293985192 `release/build mnemonic (windows-x86_64)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'build mnemonic (windows-x86_64)' passed on it; skipping the guarded steps`
- run 40234810813 job 121293984412 `rust/lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40234810813 job 121293984440 `rust/install.sh harnesses (man-step + MSRV guard)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'install.sh harnesses (man-step + MSRV guard)' passed on it; skipping the guarded steps`
- run 40234810813 job 121293984445 `rust/fmt (pinned 1.95.0)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'fmt (pinned 1.95.0)' passed on it; skipping the guarded steps`
- run 40234810813 job 121293984463 `rust/lib cross-platform check (x86_64-pc-windows-msvc, windows-latest)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'lib cross-platform check (x86_64-pc-windows-msvc, windows-latest)' passed on it; skipping the guarded steps`
- run 40234810813 job 121293984479 `rust/miri (mlock unsafe)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'miri (mlock unsafe)' passed on it; skipping the guarded steps`
- run 40234810813 job 121293984492 `rust/clippy`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'clippy' passed on it; skipping the guarded steps`
- run 40234810813 job 121293984512 `rust/test (macos-latest)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'test (macos-latest)' passed on it; skipping the guarded steps`
- run 40234810813 job 121293984515 `rust/lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40234810813 job 121293984554 `rust/test (release, ubuntu-latest, mlock einval)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'test (release, ubuntu-latest, mlock einval)' passed on it; skipping the guarded steps`
- run 40234810813 job 121293984570 `rust/musl build+test (x86_64-unknown-linux-musl)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'musl build+test (x86_64-unknown-linux-musl)' passed on it; skipping the guarded steps`
- run 40234810813 job 121293984582 `rust/test (ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'test (ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40234810813 job 121293984589 `rust/musl build+test (aarch64-unknown-linux-musl)`: `docs-only: all 1 files changed since the previous PR head 642f0588100fde1c4d6124d65e2cea07132e3296 are documentation, the merged base 63459eced79a0d749b632c5270e8bedc38c44c22 is an ancestor of it, and 'musl build+test (aarch64-unknown-linux-musl)' passed on it; skipping the guarded steps`

# Re-run after: ci: daily cross-repo drift check of the shared CI files

PR #115. Copies ci-skip-constellation reference/after-b/T2 (SPEC-after-b.md).

## Change

```
 .github/workflows/shared-drift.yml |  25 ++++++++
 ci/shared-drift.sh                 |  50 ++++++++++++++++
 ci/shared-drift.test.sh            | 114 +++++++++++++++++++++++++++++++++++++
 3 files changed, 189 insertions(+)
```

## Gate

`make-after-b-reference --check: PASS (12 files)`

## Local checks

```
ci/docs-only.sh: OK
ci/docs-only.test.sh: OK
ci/code-docs-check.sh: OK
ci/code-docs-check.test.sh: OK
docs-only.test.sh: 73 passed, 0 failed
code-docs-check.test.sh: 41 passed, 0 failed
code-docs check: ok (434 files scanned, 2488 docs files)
shared-drift.test.sh: 6 passed, 0 failed
```

## L1 (code head 6a19b8b6b1571349e5011153aa66a9057174cc92)

```
run 40237523759    manual-gui                     pull_request   success    ok
run 40237523634    technical-manual               pull_request   success    ok
run 40237523704    shared-drift                   pull_request   success    ok
run 40237523658    examples                       pull_request   success    ok
run 40237523640    quickstart                     pull_request   success    ok
run 40237523616    sibling-pin-check              pull_request   success    ok
run 40237523625    manual                         pull_request   success    ok
run 40237523606    release                        pull_request   success    ok
run 40237523654    rust                           pull_request   success    ok
run 40237520048    sibling-pin-check              push           success    ok
  manual-gui/changes                               success    ok
  manual-gui/manual-gui                            success    ok
  manual-gui/release                               skipped    ok (allowed skip)
  manual-gui/build                                 skipped    ok (allowed skip)
  manual-gui/lint                                  skipped    ok (allowed skip)
  manual-gui/verify-examples                       skipped    ok (allowed skip)
  manual-gui/verify-examples-gui                   skipped    ok (allowed skip)
  technical-manual/technical-manual                success    ok
  shared-drift/shared CI files match the other public repos success    ok
  examples/examples                                success    ok
  quickstart/quickstart                            success    ok
  sibling-pin-check/sibling pins match install.sh  success    ok
  manual/manual                                    success    ok
  release/release test (ubuntu-latest)             success    ok
  release/build mnemonic (windows-x86_64)          success    ok
  release/build mnemonic (macos-x86_64)            success    ok
  release/build mnemonic (macos-aarch64)           success    ok
  release/assemble + release                       skipped    ok (allowed skip)
  rust/g6 invariant (cross-repo mlock.rs)          success    ok
  rust/test (ubuntu-latest)                        success    ok
  rust/miri (mlock unsafe)                         success    ok
  rust/lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest) success    ok
  rust/fmt (pinned 1.95.0)                         success    ok
  rust/test (release, ubuntu-latest, mlock einval) success    ok
  rust/lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest) success    ok
  rust/lib cross-platform check (x86_64-pc-windows-msvc, windows-latest) success    ok
  rust/install.sh harnesses (man-step + MSRV guard) success    ok
  rust/test (macos-latest)                         success    ok
  rust/musl build+test (x86_64-unknown-linux-musl) success    ok
  rust/clippy                                      success    ok
  rust/musl build+test (aarch64-unknown-linux-musl) success    ok
  sibling-pin-check/sibling pins match install.sh  success    ok
pr-checks: PASS (mnemonic-toolkit 6a19b8b6b1571349e5011153aa66a9057174cc92)
```

## L2 (docs head db63cc6bd23621029e671525df8fb347ac498e13, previous head 6a19b8b6b1571349e5011153aa66a9057174cc92)

- no live skip test for T2 (SPEC-after-b.md §7 step 5: T1 and B-* only)

# Re-run after: Fold L3 r1 I-1: shared-drift sha() fails closed; test row D7

PR #115. Copies ci-skip-constellation reference/after-b/T2 (SPEC-after-b.md).

## Change

```
 .github/workflows/shared-drift.yml              |  25 +++++
 ci/shared-drift.sh                              |  50 ++++++++++
 ci/shared-drift.test.sh                         | 122 ++++++++++++++++++++++++
 design/agent-reports/ci-after-b-T2-review-r1.md | 120 +++++++++++++++++++++++
 design/agent-reports/ci-after-b-build.md        |  82 ++++++++++++++++
 5 files changed, 399 insertions(+)
```

## Gate

`make-after-b-reference --check: PASS (12 files)`

## Local checks

```
ci/docs-only.sh: OK
ci/docs-only.test.sh: OK
ci/code-docs-check.sh: OK
ci/code-docs-check.test.sh: OK
docs-only.test.sh: 73 passed, 0 failed
code-docs-check.test.sh: 41 passed, 0 failed
code-docs check: ok (434 files scanned, 2489 docs files)
shared-drift.test.sh: 7 passed, 0 failed
```

## L1 (code head 9eead365075fdd32222c64ba390cdee013b2b0fc)

```
run 40240410780    shared-drift                   pull_request   success    ok
run 40240410730    sibling-pin-check              pull_request   success    ok
run 40240411000    technical-manual               pull_request   success    ok
run 40240410635    quickstart                     pull_request   success    ok
run 40240410675    examples                       pull_request   success    ok
run 40240410606    manual-gui                     pull_request   success    ok
run 40240410624    release                        pull_request   success    ok
run 40240410609    manual                         pull_request   success    ok
run 40240410693    rust                           pull_request   success    ok
run 40240406323    sibling-pin-check              push           success    ok
  shared-drift/shared CI files match the other public repos success    ok
  sibling-pin-check/sibling pins match install.sh  success    ok
  technical-manual/technical-manual                success    ok
  quickstart/quickstart                            success    ok
  examples/examples                                success    ok
  manual-gui/changes                               success    ok
  manual-gui/manual-gui                            success    ok
  manual-gui/verify-examples-gui                   skipped    ok (allowed skip)
  manual-gui/lint                                  skipped    ok (allowed skip)
  manual-gui/verify-examples                       skipped    ok (allowed skip)
  manual-gui/build                                 skipped    ok (allowed skip)
  manual-gui/release                               skipped    ok (allowed skip)
  release/release test (ubuntu-latest)             success    ok
  release/build mnemonic (macos-aarch64)           success    ok
  release/build mnemonic (macos-x86_64)            success    ok
  release/build mnemonic (windows-x86_64)          success    ok
  release/assemble + release                       skipped    ok (allowed skip)
  manual/manual                                    success    ok
  rust/test (ubuntu-latest)                        success    ok
  rust/lib cross-platform check (x86_64-pc-windows-msvc, windows-latest) success    ok
  rust/test (release, ubuntu-latest, mlock einval) success    ok
  rust/musl build+test (x86_64-unknown-linux-musl) success    ok
  rust/clippy                                      success    ok
  rust/g6 invariant (cross-repo mlock.rs)          success    ok
  rust/miri (mlock unsafe)                         success    ok
  rust/lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest) success    ok
  rust/install.sh harnesses (man-step + MSRV guard) success    ok
  rust/test (macos-latest)                         success    ok
  rust/musl build+test (aarch64-unknown-linux-musl) success    ok
  rust/fmt (pinned 1.95.0)                         success    ok
  rust/lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest) success    ok
  sibling-pin-check/sibling pins match install.sh  success    ok
pr-checks: PASS (mnemonic-toolkit 9eead365075fdd32222c64ba390cdee013b2b0fc)
```
