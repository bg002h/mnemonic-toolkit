# PR #100 (docs/mk-cli-lint, head 19dca6b7) — adversarial review, round 1

Reviewer: independent, opus. Base: origin/master 4d703bb4. Scope: the 10-marker
removal in `docs/manual/src/40-cli-reference/44-mk-cli.md`.

## Verdict

**Do not merge. The premise is false.** The 10 markers are correct against the
CI-pinned `mk-cli-v0.13.0`. The local lint failure comes from a stale local
`mk`, not from the manual. Merging this PR turns the manual workflow red and
makes the chapter document flags that the release it tells users to install
rejects.

## What was checked (all executed, not inferred)

1. **Diff shape is mechanically clean.** `git diff -U0` gives 10 `-` and 10 `+` rows.
   If you strip ` (unreleased: mk-cli after 0.13.0: <flag>)` from each `-` row, it
   becomes byte-identical to its `+` row (`diff` reports no differences). The
   pipe-field count is 4 on every row before and after. No other file or text changed.
2. **Local lint, installed `mk` (`~/.cargo/bin/mk`, reports `mk 0.13.0`).** PR
   head gives `[lint] OK`, EXIT=0. origin/master (temporary worktree, since
   removed) gives EXIT=2 with exactly 10 `[lint] FAIL: stale marker in 44-mk-cli.md`
   lines (`--in` × 7 verbs, plus `--from-md1-set`, `--keys`, `--out` on encode).
   This matches the brief.
3. **The installed `mk` is NOT the pinned release.** `~/.cargo/.crates.toml`
   records it as
   `mk-cli 0.13.0 (path+file:///scratch/code/shibboleth/mnemonic-key/crates/mk-cli)`.
   That is a path build, not the tag. Its mtime is 2026-09-23 18:25:59 -0700. That
   predates mnemonic-key `1b8c524` (2026-09-24 03:47:10 -0700), "mk-cli:
   0.14.0-dev -- a main build no longer passes for 0.13.0 (F-677)". So it is a
   `main` build that still called itself 0.13.0, which is exactly the F-677
   hazard. `git log mk-cli-v0.13.0..HEAD -- crates/mk-cli` in mnemonic-key lists
   `3f5fc52 P3 row "the md1 set flag": --from-md1-set FILE` *after* the tag.
   `git show mk-cli-v0.13.0:crates/mk-cli/src/cmd/encode.rs` has no
   `keys`/`in`/`out`/`from_md1_set` args.
4. **Lint and mutation test against the real pin.** I built the binary the way
   CI does: `cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.13.0 mk-cli`.
   Remote tag `^{}` = 0feaaaa9, which matches the local tag. Results:
   | head | `make lint` (tag mk) | `ci/doc-flag-lint.test.sh` (tag mk) |
   |---|---|---|
   | origin/master | EXIT=0, `10 flag exemption(s)` | `12 passed, 0 failed`, EXIT=0 |
   | PR 19dca6b7 | EXIT=2, 10 × `flag --X is documented for \`mk V\` … but the pinned binary does not define it` | `FAIL baseline`, then `mutation anchor not found … '(unreleased: mk-cli after 0.13.0: --in)'` / `FAIL R2 … mutation did not apply`, EXIT=1 |
5. **Item 3 of the brief.** `mk <verb> --help` does list all 10 flags, but only on the
   stale local binary. On the tagged binary, `mk encode --keys x` prints
   `error: unexpected argument '--keys' found`, and `mk decode --in x` prints
   `error: unexpected argument '--in' found`.
6. **CI parity (item 5).** `.github/workflows/manual.yml:110` installs
   `--tag mk-cli-v0.13.0`, and step "Audit manual" runs `make audit` (which
   includes `lint`) with `MK_BIN=mk`. CI therefore behaves like row "PR 19dca6b7"
   in the table, not like the local run. `MR_BIN` appears 0 times in
   manual.yml, so the `MR_BIN unset` WARN is identical in both. It is the only
   WARN in either local run.

## Critical

**C1 — 44-mk-cli.md:43,44,45,46,134,183,241,412,503,553 (the 10 rows): removes
correct markers. CI goes red, and the manual documents flags the pinned release
rejects.**
- Failure: on CI's pinned `mk-cli-v0.13.0`, the PR head fails `make lint` (exit 2,
  10 FAILs) and `ci/doc-flag-lint.test.sh` (exit 1). Master passes both, so the
  PR converts a green manual workflow into a red one.
- Advice lens: the chapter's own install line (44-mk-cli.md:12,
  `cargo install … --tag mk-cli-v0.13.0 --bin mk`) gives the user the tagged
  0.13.0. Once the marker is gone, they follow e.g. the `mk encode --keys` row
  and get `error: unexpected argument '--keys' found`. The same happens for
  `--in` on all 7 verbs, and for `--out` and `--from-md1-set`. The marker was
  the only text warning them.
- The PR description's claim "the pinned mk-cli 0.13.0 … already defines" these
  flags is false. It was measured with a mislabelled dev binary.
- Fix: close PR #100 without merging. Fix the environment instead: run
  `cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.13.0 mk-cli --force`,
  so the local `mk` is the pinned release, then re-run
  `make -C docs/manual lint … MK_BIN=mk` (expect EXIT=0, 10 exemptions). Remove
  the markers only in the PR that bumps the CI pin
  (`.github/workflows/manual.yml:110`, `scripts/install.sh`, 44-mk-cli.md:12) to
  an mk-cli release that ships these flags. That PR must also rename the
  `after 0.13.0` markers for any flags still unreleased at the new pin.

## Important

**I1 — ci/doc-flag-lint.test.sh:75-77, 83-85, 91-93, 95-97: mutations R2, R4, R6
and R7 anchor on these exact marker strings.** R2 and R4 anchor on
`(unreleased: mk-cli after 0.13.0: --in)`; R6 and R7 anchor on
`(unreleased: mk-cli after 0.13.0: --keys)`.
- Failure: after this PR, `edit_file` raises `mutation anchor not found` and the
  script `exit 1`s at R2. The CI step "Mutation-test the flag lint" fails
  independently of C1. This holds whichever `mk` is used, because the anchor
  text no longer exists.
- Fix: this PR should not land (C1). For the eventual pin-bump PR that
  legitimately removes these markers, add a bullet to that PR's checklist:
  "re-anchor R2/R4/R6/R7 in ci/doc-flag-lint.test.sh on a marker that is still
  live at the new pin. If none remains, inject a synthetic marked row into the
  copied src/ (as R1 does for an unmarked row), so the reverse-direction
  mutations keep a target."

## Minor

- M1 — docs/manual/tests/lint.sh:195: `PINNED[$bin]` is taken from
  `<bin> --version`, so a non-release build that never bumped its version
  passes as the pinned release. That is how this PR's false premise arose.
  mk-cli closed it at source in 1b8c524 (F-677). File a FOLLOWUP to have the lint
  WARN when `--version` disagrees with the tag pinned in manual.yml, or to
  document in the Makefile `lint` help that local runs must use the tagged
  binaries.
- M2 — 44-mk-cli.md:16-18 (marker-convention paragraph): **not a defect** at the
  correct outcome (markers kept, so the paragraph is accurate). Had the PR's
  premise held, the paragraph would describe a convention with zero instances.
  That is Minor at most, and it would be removed in the pin-bump PR. No other
  `unreleased` or `after 0.13.0` text exists anywhere in `docs/manual/src`
  besides the 10 markers and this paragraph (grep: no other chapter matches).

## Nit

- N1 — PR #100 commit/description: the claim "CI tag mk-cli-v0.13.0; installed
  `mk` is 0.13.0" conflates two different binaries. State the provenance
  (`.crates.toml` source) whenever a lint result is attributed to "the pinned"
  binary.

## Housekeeping

- The temporary master worktree `/scratch/code/shibboleth/mt-lint-master-rv` was
  removed (`git worktree list` shows 0 matches).
- `/scratch/code/shibboleth/mk-tag-rv/` (the tagged mk build: `root/bin/mk` +
  `target/`) is left in place. A hook blocks `rm -rf`, so the controller or user
  must delete it. Its `root/bin/mk` is usable for reproducing step 4 meanwhile.

1 / 1 / 2 / 1
