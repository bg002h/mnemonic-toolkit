# T1 author report: miniscript tripwire rule and stale 13.x text

Branch `fix/miniscript-tripwire-13-2` from origin/master 02052a17. No dependency bump
(`[patch.crates-io]` rev ff4732e5 unchanged). Class D, Sonnet author.

## What changed

1. **Tripwire (architect Q2).** Logic in `ci/miniscript-tripwire.py` (stdlib). The
   workflow `.github/workflows/miniscript-fork-tripwire.yml` now only fetches the sparse
   index and calls it with `BASELINE: '13.2.0'`; it also runs the classifier tests
   offline first and triggers on changes to the script, test, fixture and itself.
   **This edits `.github/`, as the brief authorizes.** The printed texts are the architect's.
2. **Tests** `ci/miniscript-tripwire.test.sh`, rows a-f from Q2 plus `row_g`. Fixture
   `ci/fixtures/miniscript-index-13x.jsonl` holds the four real 13.x index rows.
3. **Tracker text.** FOLLOWUPS umbrella `taproot-coverage-cycle-on-miniscript-gt-13-1-0`
   and companion `upstream-miniscript-taptree-depth2-display-asymmetry` corrected,
   plus `two-miniscripts-patch-load-bearing-stale-error` and T3-PARTIAL (i).
   Root, `crates/mnemonic-toolkit` and `fuzz` `Cargo.toml` comments rewritten (rev lines untouched).
4. **Manual lockstep** (`41-mnemonic.md`): the `tr(sortedmulti_a)` bullet no longer
   waits for a release; the "deeper trees refused" sentence is replaced (depth-2 restores,
   since toolkit 5f88071c, tests `left_heavy_3leaf_tr_restores_depth2` and
   `right_spine_3leaf_tr_also_restores_depth2`). The in-tree `sortedmulti_a` refusal sentence is kept (T3).
5. **New FOLLOWUPS.** `miniscript-pin-moves-double-count-mr-cost` (V6; owning phase: the
   next cycle that moves miniscript) and `fuzz-and-repro-miniscript-pin-realign`. Q1 text
   recorded verbatim: "Kept by decision 2026-11-03 (architect miniscript-13-2): no 13.x
   port; drop when one release carries #910, #915, #911 and #929; the tripwire watches for it."

## TDD evidence

RED: with `ci/miniscript-tripwire.py` absent, rows a-f all failed. GREEN: 7 of 7, also
against the real miniscript-13.2.0.crate (sha-checked against the fixture cksum via
`MINISCRIPT_132_CRATE`). Plants, each caught by the named row: drop `or new_major`
(row_d); `k < 3` in the control (row_e); tautological cksum check (row_f); `<=` to `<`
in step 4 (row_a); `new_major or len(found) >= 1` (row_g).

## Choices (nothing was an undecided product point)

- `row_g` is beyond the architect's six: 2/4 markers on the same major gives exit 0 plus
  the step-7 notice. It pins the "all four" half of the rule against a loosening.
- The fuzz/repro 95fdd1c pin realign needs a re-lock (a code change), so per the brief
  it is a FOLLOWUP, not done here. The fuzz-build gate step rewrites `fuzz/Cargo.lock`;
  that edit was reverted, not committed.
- Companion status set to `resolved` for the toolkit: a tracker-state call (the lift
  landed 2026-08-20); the upstream half stays as the entry describes.
- Coordinator mid-task defects, both fixed before push: (1) FOLLOWUPS text saying the
  depth-2 restore refusal was live (corrected against 5f88071c and `ensure_taptree_wellformed`);
  (2) a stray non-TOML line in `fuzz/Cargo.toml` (deleted; all three Cargo.toml parse with tomllib).
- Not touched: restore gates (T3), compare-cost/bundle (V5/V7).

## Gate (run once to a file, grep-read; late edits re-run per section)

All CI jobs the host can run were run: fmt, clippy, nextest, doctests, cross-freebsd,
the ci/ self-tests (docs-only, code-docs, doc-gate-guard, shared-drift, vf, residue),
vendor-freshness, fuzz-build, manual-pdf, manual-html, manual-audit, doc-flag-lint mutation.

Three sections failed on the first run, all environmental, none caused by this change:

- `manual-audit` rc=2 and `doc-flag-lint-mutation` rc=1: the host `mk` on PATH is a
  path install of a mnemonic-key dev checkout (it defines `--in`, `--keys`, ...) while CI
  installs tag `mk-cli-v0.13.0`. Installed that tag into a scratch root and re-ran with
  `MK_BIN` pointed at it: manual-audit rc=0, doc-flag-lint 15 passed 0 failed. Re-run
  again after the last manual edit: manual-audit rc=0.
- `cross-windows` rc=101: `cc-rs` cannot find MSVC `lib.exe` building secp256k1-sys.
  CI-only (it runs on windows-latest).

After the FOLLOWUPS and manual edits: `ci/docs-only.test.sh`, `ci/code-docs-check.test.sh`,
`ci/code-docs-check.sh` ("ok"), `ci/doc-gate-guard.test.sh`, `ci/shared-drift.test.sh`,
`ci/miniscript-tripwire.test.sh` (7/7) and the Cargo.toml parse all pass.
This change is not docs-only (it touches `.github/` and `ci/`), so the full job set applies.

**CI-only (not run locally):** test-release-mlock-einval (needs `--release`, forbidden);
macos test row; aarch64 cross-check; musl-build-test and its aarch64 row (no `musl-gcc`);
cross-windows (no MSVC); the `release.yml` pull_request matrix; the live-network
steps of the tripwire workflow (index fetch, crate download); `mr` lines of the
manual lint (`MR_BIN` unset, as in this repo's CI; mr is linted in mnemonic-refugium).

## Findings

None Critical or Important. Minor: the host's `mk` is a dev build, so the local manual
gate needs `MK_BIN` set to the pinned tag (not filed; a local-environment note).

0 Critical, 0 Important, 1 Minor, 0 Nit.
