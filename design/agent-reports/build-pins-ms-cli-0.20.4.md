# Build report: toolkit PR "pins: ms-cli-v0.20.4"

Base: origin/master 67470572a2e74568c3b2875819deab891da9fc90 (did not move at fetch). Branch `pins/ms-cli-v0.20.4`.
Worktree: /scratch/cache/wt-toolkit-pins-ms-0.20.4. Logs: /scratch/cache/toolkit-pins-ms-0.20.4-logs/.
PR: #159 (draft) https://github.com/bg002h/mnemonic-toolkit/pull/159. Content head 1cb763ae; this report gets a final PR-number commit on top (see `git log`).

## What was done (pins and texts)
- Pin texts applied with the architect's `apply.py` (read-only, run from outside against this tree: tag `ms-cli-v0.20.4`, version `0.20.4`): install.sh:58, the three workflow mirrors (manual.yml, quickstart.yml, technical-manual.yml), pinned-upstream.toml (:127 and :116-117), 82-binary-and-launch.md (:48, :76, the two `{#installer-ahead}` ms bullets = A/B9), 43-ms.md (A1 `--out` paragraph, A2 split row, A3/Q5 *When stdout fails*). Applied by a one-off script (logs/docs.py): CHANGELOG.md and docs/manual-gui/CHANGELOG.md entries (report texts), design/FOLLOWUPS.md (`ms-manual-when-stdout-fails` closed; `ms-split-out-refuses-existing-manual` filed `open`).
- Not touched: 67-split.md, 64-decode.md, 65-verify.md (B5-B8 wait for the GUI re-pin), manual-gui.yml (stays `ms-cli-v0.20.1`).
- `first_signed` for ms stays `0.20.2` (install.sh:187-195): it is the first ms release whose assets carry a `.minisig` (install.sh:183-184, "ms 0.20.2 (2026-11-01) is the first pinned release that is signed"); the 0.20.4 pin is after it, so no entry changes (the file's rule: fill it in the commit that moves a pin onto the first signed release, which 99310923 did).

## RED row
- Plant: install.sh moved to `ms-cli-v0.20.4` alone, then `python3 docs/manual-gui/tests/check_cli_pins.py --manual-dir docs/manual-gui --install-sh scripts/install.sh` -> rc=1, `pinned-upstream.toml [installer-ahead] ms-cli-tag-implied = ms-cli-v0.20.2, but install.sh pins ms-cli-v0.20.4` (logs/red.log). GREEN is below.

## tk2 comparison
`diff` of tk2 (/scratch/cache/architect-ms-pin-bump/tk2) against this tree, per file apply.py touches: IDENTICAL for scripts/install.sh, manual.yml, quickstart.yml, technical-manual.yml, pinned-upstream.toml, 82-binary-and-launch.md, 43-ms.md (`diff -q` empty for all seven). The pin PR's other files (CHANGELOG.md, manual-gui CHANGELOG.md, FOLLOWUPS.md) differ from tk2 only by the report's added texts (apply.py does not write them); .examples-build/Examples.md by the regeneration. tk2's 64-decode.md/67-split.md edits are B5-B8 and are deliberately NOT in this tree.

## Examples golden
`EXAMPLES_BIN_DIR=target/debug bash .examples-build/gen.sh > Examples.md`: rc 0; `git diff --stat`: 1 file, 4 insertions, 4 deletions = exactly the ms lines :109, :135, :136, :139 (tag, asset name, `ms 0.20.4`). Nothing else.

## Gate lines (each captured once in the logs dir; "base" = unmodified origin/master worktree /scratch/cache/wt-toolkit-pins-ms-0.20.4-base, same binaries)
Binaries: ms 0.20.4 from the release tarball (sha256 checked OK against SHA256SUMS.x86_64, `ms --version` = `ms 0.20.4`), as MS_BIN only; mnemonic debug build of this tree; local md 0.20.5 and mk 0.14.0 (NOT the pins md 0.20.4, mk 0.13.0).
- sibling-pin-check step 1 (workflow script extracted): rc=0, 11 pins OK incl. `OK .github/workflows/{manual,quickstart,technical-manual}.yml:{175,150,181}: ms-cli-v0.20.4`, `OK all sibling pins match scripts/install.sh`.
- sibling-pin-check step 2 / check_cli_pins.py: `OK: cli-pin-consistency: ... ms 0.20.1 ...; installer ahead: ms 0.20.4; 79 version mention(s) in 103 files`. Plant (82:48 left at `ms-cli-v0.20.2`): rc=1, `82-binary-and-launch.md:48: names ms 0.20.2 ('ms-cli-v0.20.2'), but the pin is 0.20.1`. (RED row above, GREEN here.)
- gui-pin-drift-check: rc=0 (a warning only: GUI pin v0.63.0 lags the latest v0.63.1; not this PR's).
- install-verify / install-signature / install-assets-gate (offline): each `OK`. install-assets (live, against the pinned releases): `ok 32 of 32 mapped assets passed the signed-pin gate`, includes `ms 0.20.4: signed, first_signed 0.20.2 <= pin`; `[install-assets.test] OK`. install-msrv-guard, install-man-step: rc=0.
- examples: gen.sh regenerated and written; the gate `git diff --exit-code` compares the committed file with the same regeneration, so it holds once committed (not re-run in CI form).
- ci/docs-only.test.sh, shared-ci.sha256 check, ci/code-docs-check(.test).sh: rc=0. (This diff is NOT docs-only for ci/docs-only.sh: scripts/install.sh and .github/ are outside ci/docs-only-paths.txt, so rust.yml and release.yml run in full.)
- rust.yml test equivalents: `cargo nextest run --locked --workspace --no-fail-fast -j 8`: `4267 tests run: 4267 passed, 38 skipped`; `cargo test --locked --workspace --doc` rc=0; g2_1 (eperm), g2_3 (einval), g2_4 (off): rc=0; g6 `cargo test --locked -p mnemonic-toolkit --test mlock_g6_invariant -- --include-ignored` with SIBLING_REPO_PATH = `git archive` of ms-cli-v0.20.4 (= 9b3332e0): `2 passed`; `cargo clippy --locked --all-targets -- -D warnings` rc=0; `cargo +1.95.0 fmt --all -- --check` rc=0.
- manual.yml: `make audit` (MS_BIN = release ms 0.20.4): FAILS at the lint step on BOTH this tree and base with the identical sorted set of 12 `[lint] FAIL` lines, of which 10 are `(unreleased: mk-cli after 0.13.0: --in)` markers in 44-mk-cli.md from the local mk 0.14.0, none for ms (the architect's known set). `make audit` stops at lint, so `make verify-examples` and `make anchor-check` were run separately: anchor-check `OK ... 9 danglers match baseline` on both; verify-examples fails on both with the identical 9 lines, all `44-mk-*` / `qs-26-recover-mk1` drift from mk 0.14.0, none ms. `ci/doc-flag-lint.test.sh` (mutation test, local bins, MS_BIN=release ms 0.20.4): `10 passed, 5 failed` on BOTH this tree and base, the same 5 (baseline, R3, R4, R6, G0), every one caused by the local mk 0.14.0 / md 0.20.5 markers (`unreleased: mk-cli after 0.13.0`, and on base also one md `--md-only` marker); no ms marker. With the pinned mk/md it would be the CI result, which I could not reproduce here.
- quickstart.yml: lint `[lint] OK`; verify-examples fails identically on base (8 mk-related FAIL lines from mk 0.14.0). technical-manual.yml: lint `[lint] OK`. html for manual, quickstart, technical-manual: rc=0; pdf for the three: rc=0.
- manual-gui.yml: `make html` then `make lint` (MANUAL_GUI_UPSTREAM_ROOT = clone of mnemonic-gui-v0.63.0): `[lint] OK` on this tree and base; verify-examples against the GUI-pinned tier (md 0.20.3, ms 0.20.1, mk 0.13.0, mnemonic 0.105.1 built with `cargo install --locked` in an isolated root): `OK (17 transcripts pass)` on both; `make pdf`, `gui-example-pdf`, `gui-example-html`: rc=0.

## NOT RUN
- manual-gui.yml `verify-examples-gui` (builds the pinned gui-render, unaffected by this diff).
- rust.yml: `miri`, `lib cross-platform check` (windows/aarch64/freebsd), `musl build+test` (x86_64 and aarch64), `test (release ... mlock einval)` (needs `--release`, forbidden by the rules), the install.sh-harness step as a whole job beyond the individual scripts above; release.yml `build`/`assemble` (cross builds, signing) and its `test` job on macOS/Windows legs. bitcoind-differential, cross-tool-differential, fuzz-smoke, shared-drift, vendor-freshness do not trigger on this diff's paths.
- `ci/doc-flag-lint.test.sh` and `make audit` with the PINNED mk 0.13.0 / md 0.20.4: the local mk/md differ (see the manual.yml line), so the CI-faithful result is not reproduced; the same-both-ways comparison is given instead.

## Remaining older ms tags/versions (`git grep -n 'ms-cli-v0\.20\.\|ms 0\.20\.'`)
- manual-gui.yml:175, :230 (`ms-cli-v0.20.1`): GUI tier, Q4, stays.
- pinned-upstream.toml:93, :113 and manual-gui pages 11, 12, 61, 82:28, 82:49, 94: the GUI's own pin ms 0.20.1, stays (GUI tier rule).
- install.sh:183 (`ms 0.20.2` in the first_signed note) stays: history of the first signed release.
- crates/mnemonic-toolkit/tests/bughunt_s5.rs:36 (`ms 0.20.2` in a test comment): names the release that fixed that behaviour; history.
- check_cli_pins.py:15: a docstring example (`ms-cli-v0.20.0`).
- design/ and CHANGELOG entries (filtered from the listing): history of earlier pins.

## STOPs / questions
- None blocking. One substitution note (Minor): in the new FOLLOWUPS heading `<NEW_TAG>` appears after the words "ms-cli", which read literally gives "ms-cli ms-cli-v0.20.4"; I wrote "to ms-cli-v0.20.4" (drop the duplicate word). The Status line says "by the pin commit `pins: ms-cli-v0.20.4`" (the PR title form) in place of "`pins: ms-cli <NEW_TAG>`".
- Noted, not fixed (architect N1): pinned-upstream.toml:116 still says "two CLIs" (three are ahead).

Counts: Critical 0, Important 0, Minor 1, Nit 1.
