
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
