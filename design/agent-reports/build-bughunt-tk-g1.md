build: tk 661c8117; gate FAIL; 1 STOP

Worktree `/scratch/code/shibboleth/tk-worktrees/bughunt-triage`, branch `bughunt/triage`, base `a9476fe1`. Local only, nothing pushed. No version change. G2 untouched. Logs: `/scratch/cache/bughunt/tk/build-g1/`.

The one gate failure is manual job step 1/7 (markdownlint MD038) on the FIX's own 5.1 text; see `build-bughunt-tk-g1-STOP.md`. Everything else is green or matches the stated expectation.

## Commits (base a9476fe1)

| step | commit | closes 9.1 line |
|---|---|---|
| 1 RED rows (section 6 forms, six new rows, #14 and #7 ignored) and the section 7 flip; 33 run, 33 failed on unfixed code, each for the FIX's reason | aebd9017 | (10) |
| 2.1 restore.rs #1 #2 11b #10 #38 #40 #41 #42 #47 #53 | 9fa41d40 | (1) |
| 2.2 verify_bundle.rs #12 #13 #37 #39 #43 #44 #47 #52 | 127884ed | (2) |
| 2.3 bundle.rs #6 #9 #45 #46 #54 | ceeb2f53 | (3) |
| 2.4 parse_descriptor.rs #3 (flip is in step 1) | bad4a6fd | (4) |
| 2.5 synthesize.rs #11 / 11b | 72c292c3 | (5) |
| 2.6 electrum.rs #4, wallet_export/electrum.rs #5 | 4355ecef | (6) |
| 2.7 wc-codec pipeline.rs #8 and lib.rs rustdoc (2.24) | b685ec97 | (7) |
| 2.8 five spec edits (section 4) | 9e6af054 | (8) |
| 2.9 manual edits 5.1-5.14 | b709c8e5 | (9) |
| 3 FOLLOWUPS 9.1, 9.2, 9.3 before `bughunt-tk-fix-review-r1-minors` (still last) | 661c8117 | (11) |

`cargo fmt` (both toolchains) was run before the RED commit and the code commits; both check clean at the head. The reference patch was applied as a base; the code differing from it: #45 same-class text (I-3), #46 text (I-1), wc-codec `lib.rs` rustdoc. #37 already used the second `synthesize_unified(&resolved, ..)` call (I-2); manual 5.2 and 5.1 were applied from the FIX text (I-4), not from `manual-edits.py`.

## Gate

| check | result |
|---|---|
| `cargo +1.95.0 fmt --all -- --check`, `cargo fmt --all -- --check` (1.85.0) | both clean |
| `cargo clippy --locked --all-targets -- -D warnings` | clean |
| `cargo nextest run --locked --workspace --no-fail-fast` | 4152 run, 4118 passed, 34 failed, 22 skipped. All 34 failures are G2's RED rows (`bughunt_s3` 14, `bughunt_s4` 13 + 1 in the bin, `bughunt_s5` 6), the expected state before G2's build; no other failure |
| `cargo test --locked --doc --workspace` | ok (0 doc tests) |
| G2.1 eperm / G2.3 einval / G2.4 off fresh-subprocess steps | 1 passed each |
| miri `mlock::` (nightly) | 8 passed, 3 ignored |
| `ci/docs-only.test.sh`, shared-CI sha check, `code-docs-check` (+test), `shared-drift.test.sh` | pass |
| install-sh harnesses (msrv-guard, man-step, verify, signature, assets-gate, assets) | pass |
| g6 invariant (sibling `ms-cli-v0.20.2` from the local clone) | 2 passed |
| cross-platform `cargo check --lib`: freebsd | ok |
| not run: msvc (no `lib.exe`), aarch64-linux gnu (no cross gcc), both musl rows (no `musl-gcc`), the release-mode mlock row (no `--release` by rule). No platform code is in the diff |
| manual job, with the pinned mk-cli v0.13.0 installed from its tag: `make audit` | FAIL at lint 1/7: MD038 at `41-mnemonic.md:368:777` (the STOP). Other lint steps ran clean |
| `make verify-examples` | OK, 62 transcripts (incl. `41-bundle-inheritance-json`) |
| `make anchor-check` | OK, 9 danglers = baseline |
| `ci/doc-flag-lint.test.sh` | 15 passed, 0 failed |
| section 6 rows, each by exact name (`test(/(^|::)<row>$/)`) | 34 rows listed in the table: 32 matched and passed (1 run each), 2 matched and skipped (#14, #7: `#[ignore]`, 0 run by default; run with `--run-ignored only` both still FAIL, as the FIX says). Matches "32 run, 32 passed, 2 skipped". Toolkit `bughunt_s1`+`bughunt_s2`: 31 run, 31 passed, 1 skipped; wc-codec: #8 run and passed, #7 skipped |
| section 7 flip `lex_rejects_multipath_sans_wildcard` | 1 run, 1 passed |
| section 2 after-fix text, byte for byte: `cases.py` (90 cases) run on the built binary; all 215 `stdout:` / `stderr:` lines of the "after the fix" and "unchanged" transcripts found in the outputs, except 6 that are elided in the FIX itself (`…` mid-line, "22 lines in all") or are the signet case that `cases.py` lacks; that case and the Japanese #37 control are asserted by the RED rows (#45, #37), which pass. Exit codes are asserted by the rows | 

## 9.1 lines closed

(1) through (11), each by the commit in the table. FOLLOWUPS status of the entry stays `open` as the FIX's text gives it (the build does not edit it).

## Could not do

- The one STOP above (manual 5.1 wording vs markdownlint MD038). The manual is committed with the FIX's text unchanged, so the manual CI job would fail on the PR head until the wording is decided.
- The CI rows listed as "not run" above (msvc, aarch64 cross, musl, release mlock): not runnable on this machine without `--release` or missing toolchains.
- `coordinator/scripts/mutation-gate.sh` is the coordinator's step after this report; not run here.
