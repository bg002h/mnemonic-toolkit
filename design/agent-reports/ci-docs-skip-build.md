
# Re-run after: ci: skip a docs-only push to a PR that already passed (rule A')

PR #112. Applies ci-skip-constellation SPEC.md (GREEN at e8f831d), Part A.

## Change

```
 .github/workflows/release.yml |  13 +++-
 .github/workflows/rust.yml    |   6 ++
 ci/code-docs-check.sh         | 165 ++++++++++++++++++++++++++++++++++++++++++
 ci/code-docs-check.test.sh    | 128 ++++++++++++++++++++++++++++++++
 ci/code-docs-exempt.txt       |  19 +++++
 ci/docs-only-paths.txt        |   5 ++
 ci/docs-only.sh               | 158 +++++++++++++++++++++++++++++-----------
 ci/docs-only.test.sh          | 103 ++++++++++++++++++++++++--
 ci/shared-ci.sha256           |   4 +
 9 files changed, 549 insertions(+), 52 deletions(-)
```

## Gate

`gate-public: 0 failure(s)`

## Local checks

```
ci/docs-only.sh: OK
ci/docs-only.test.sh: OK
ci/code-docs-check.sh: OK
ci/code-docs-check.test.sh: OK
docs-only.test.sh: 73 passed, 0 failed
code-docs-check.test.sh: 41 passed, 0 failed
code-docs check: ok (431 files scanned, 2483 docs files)
```

## L1 (code head 0a19219988a839400a9cf1f19a90782bf574cab5)

```
run 40040371954    sibling-pin-check              pull_request   success    ok
run 40040371957    examples                       pull_request   success    ok
run 40040371944    quickstart                     pull_request   success    ok
run 40040371943    manual-gui                     pull_request   success    ok
run 40040371941    technical-manual               pull_request   success    ok
run 40040371976    manual                         pull_request   success    ok
run 40040371947    release                        pull_request   success    ok
run 40040371973    rust                           pull_request   success    ok
run 40040367844    sibling-pin-check              push           success    ok
  sibling-pin-check/sibling pins match install.sh  success    ok
  examples/examples                                success    ok
  quickstart/quickstart                            success    ok
  manual-gui/changes                               success    ok
  manual-gui/manual-gui                            success    ok
  manual-gui/release                               skipped    ok (allowed skip)
  manual-gui/lint                                  skipped    ok (allowed skip)
  manual-gui/verify-examples-gui                   skipped    ok (allowed skip)
  manual-gui/verify-examples                       skipped    ok (allowed skip)
  manual-gui/build                                 skipped    ok (allowed skip)
  technical-manual/technical-manual                success    ok
  manual/manual                                    success    ok
  release/release test (ubuntu-latest)             success    ok
  release/build mnemonic (macos-aarch64)           success    ok
  release/build mnemonic (macos-x86_64)            success    ok
  release/build mnemonic (windows-x86_64)          success    ok
  release/assemble + release                       skipped    ok (allowed skip)
  rust/test (release, ubuntu-latest, mlock einval) success    ok
  rust/clippy                                      success    ok
  rust/test (macos-latest)                         success    ok
  rust/miri (mlock unsafe)                         success    ok
  rust/musl build+test (x86_64-unknown-linux-musl) success    ok
  rust/g6 invariant (cross-repo mlock.rs)          success    ok
  rust/musl build+test (aarch64-unknown-linux-musl) success    ok
  rust/install.sh harnesses (man-step + MSRV guard) success    ok
  rust/lib cross-platform check (x86_64-pc-windows-msvc, windows-latest) success    ok
  rust/test (ubuntu-latest)                        success    ok
  rust/lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest) success    ok
  rust/lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest) success    ok
  rust/fmt (pinned 1.95.0)                         success    ok
  sibling-pin-check/sibling pins match install.sh  success    ok
pr-checks: PASS (mnemonic-toolkit 0a19219988a839400a9cf1f19a90782bf574cab5)
```

## L2 (docs head fdc92e9a3980226f09188a34fe51c95b1430560d, previous head 0a19219988a839400a9cf1f19a90782bf574cab5)

- run 40041424623 job 120713148248 `rust/clippy`: `docs-only: all 1 files changed since the previous PR head 0a19219988a839400a9cf1f19a90782bf574cab5 are documentation, the merged base afc9c24fade561f9ae2ff1fc831e00e1de07f30e is an ancestor of it, and 'clippy' passed on it; skipping the guarded steps`
- run 40041424623 job 120713148460 `rust/musl build+test (x86_64-unknown-linux-musl)`: `docs-only: all 1 files changed since the previous PR head 0a19219988a839400a9cf1f19a90782bf574cab5 are documentation, the merged base afc9c24fade561f9ae2ff1fc831e00e1de07f30e is an ancestor of it, and 'musl build+test (x86_64-unknown-linux-musl)' passed on it; skipping the guarded steps`
- run 40041424623 job 120713148530 `rust/test (release, ubuntu-latest, mlock einval)`: `docs-only: all 1 files changed since the previous PR head 0a19219988a839400a9cf1f19a90782bf574cab5 are documentation, the merged base afc9c24fade561f9ae2ff1fc831e00e1de07f30e is an ancestor of it, and 'test (release, ubuntu-latest, mlock einval)' passed on it; skipping the guarded steps`
- run 40041424623 job 120713148546 `rust/fmt (pinned 1.95.0)`: `docs-only: all 1 files changed since the previous PR head 0a19219988a839400a9cf1f19a90782bf574cab5 are documentation, the merged base afc9c24fade561f9ae2ff1fc831e00e1de07f30e is an ancestor of it, and 'fmt (pinned 1.95.0)' passed on it; skipping the guarded steps`
- run 40041424623 job 120713148547 `rust/test (ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head 0a19219988a839400a9cf1f19a90782bf574cab5 are documentation, the merged base afc9c24fade561f9ae2ff1fc831e00e1de07f30e is an ancestor of it, and 'test (ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40041424623 job 120713148551 `rust/install.sh harnesses (man-step + MSRV guard)`: `docs-only: all 1 files changed since the previous PR head 0a19219988a839400a9cf1f19a90782bf574cab5 are documentation, the merged base afc9c24fade561f9ae2ff1fc831e00e1de07f30e is an ancestor of it, and 'install.sh harnesses (man-step + MSRV guard)' passed on it; skipping the guarded steps`
- run 40041424623 job 120713148554 `rust/lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head 0a19219988a839400a9cf1f19a90782bf574cab5 are documentation, the merged base afc9c24fade561f9ae2ff1fc831e00e1de07f30e is an ancestor of it, and 'lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40041424623 job 120713148587 `rust/lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head 0a19219988a839400a9cf1f19a90782bf574cab5 are documentation, the merged base afc9c24fade561f9ae2ff1fc831e00e1de07f30e is an ancestor of it, and 'lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40041424623 job 120713148589 `rust/miri (mlock unsafe)`: `docs-only: all 1 files changed since the previous PR head 0a19219988a839400a9cf1f19a90782bf574cab5 are documentation, the merged base afc9c24fade561f9ae2ff1fc831e00e1de07f30e is an ancestor of it, and 'miri (mlock unsafe)' passed on it; skipping the guarded steps`
- run 40041424623 job 120713148664 `rust/musl build+test (aarch64-unknown-linux-musl)`: `docs-only: all 1 files changed since the previous PR head 0a19219988a839400a9cf1f19a90782bf574cab5 are documentation, the merged base afc9c24fade561f9ae2ff1fc831e00e1de07f30e is an ancestor of it, and 'musl build+test (aarch64-unknown-linux-musl)' passed on it; skipping the guarded steps`
- run 40041424623 job 120713148666 `rust/lib cross-platform check (x86_64-pc-windows-msvc, windows-latest)`: `docs-only: all 1 files changed since the previous PR head 0a19219988a839400a9cf1f19a90782bf574cab5 are documentation, the merged base afc9c24fade561f9ae2ff1fc831e00e1de07f30e is an ancestor of it, and 'lib cross-platform check (x86_64-pc-windows-msvc, windows-latest)' passed on it; skipping the guarded steps`
- run 40041424623 job 120713148685 `rust/test (macos-latest)`: `docs-only: all 1 files changed since the previous PR head 0a19219988a839400a9cf1f19a90782bf574cab5 are documentation, the merged base afc9c24fade561f9ae2ff1fc831e00e1de07f30e is an ancestor of it, and 'test (macos-latest)' passed on it; skipping the guarded steps`
- run 40041424581 job 120713148436 `release/release test (ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head 0a19219988a839400a9cf1f19a90782bf574cab5 are documentation, the merged base afc9c24fade561f9ae2ff1fc831e00e1de07f30e is an ancestor of it, and 'release test (ubuntu-latest)' passed on it; skipping the guarded steps`
