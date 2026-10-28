# ci-manual-mr-bin build report

Brief: `ci-manual-mr-bin` (toolkit `manual` CI checks mr's chapter). Implementer
run, branch `ci/manual-mr-bin` from `origin/master` `4091d6f8`, worktree
`/scratch/code/shibboleth/mt-ci-mr-bin`. Draft PR: bg002h/mnemonic-toolkit#116.

## What changed

Commit `b5046f95`, two files:

- `.github/workflows/manual.yml`
  - New step "Install mr", after "Install ms-cli":
    `cargo install --git https://github.com/bg002h/mnemonic-refugium --rev 6b210222c03e16f53f79ead4836b51d7339f4945 mr-cli`
    (package `mr-cli`, binary `mr`, confirmed in `crates/mr-cli/Cargo.toml`;
    `6b210222` resolved with `git rev-parse` in the refugium checkout and
    confirmed an ancestor of its `origin/master`). Same `if:` guard and
    `CARGO_INSTALL_ROOT` as the other installs; the existing cache key hashes
    `manual.yml`, so the new step is covered by the same cache.
  - `MR_BIN=mr` in "Audit manual"; `MR_BIN="$(command -v mr)"` in
    "Mutation-test the flag lint" (the neighbouring lines' style in each step).
  - Two comments updated: "The four Install steps ..." and "a changed tag, rev
    or feature set reinstalls".
- `design/FOLLOWUPS.md`: one entry, `verify-examples-skips-mr`, status `open`,
  owning phase none (ownerless residue), tier `tooling`.

Manual text changed: none. The lint passes on `45-mr.md` and on the `mr` lines
of `cli-subcommands.list` against the pinned `mr`, so there was nothing to fix
and the 20-line STOP threshold was never in play.

## Item 3 of the brief (other pin-list checks)

- `sibling-pin-check.yml`: its detector matches only `cargo install ... --git
  <url> --tag <tag>` lines and resolves them against `scripts/install.sh`. The
  new line uses `--rev`, so it is not scanned and raises no warning. The PR run
  of that job is green.
- `install-pin-check.yml`: toolkit self-pin on `mnemonic-toolkit-v*` tags only.
  Not affected.
- `scripts/install.sh`: its pin table has no mr component and the installer
  fetches release assets by tag; mnemonic-refugium has no tags. Adding mr would
  need a choice (tag scheme, release assets, signing); nothing requires it, so
  it was not touched.
- `ci/doc-gate-guard.sh`: `manual.yml` is already in its `--also` set, so the
  workflow change makes the guard answer relevant; no edit needed.
- `ci/code-docs-check.sh`: `manual.yml` is in `ci/code-docs-exempt.txt`; ran
  clean.

Item 3 itself needed no choice; the STOP below is about repository access.

## Observation (not changed)

`ci/doc-flag-lint.test.sh` does not pass `MR_BIN` to `lint.sh` itself, and its
usage line lists four binaries. The lint reads `MR_BIN` from the environment,
so the variable set by the workflow step is inherited and the baseline case
checks the mr chapter. The script has no mr mutation case. Both left as is.

## Gate

Local, with the tagged sibling CLIs installed from the pins `manual.yml` uses
(`mk-cli-v0.13.0`, `descriptor-mnemonic-md-cli-v0.20.3` with `cli-compiler`,
`ms-cli-v0.20.1`), `mr` from the pinned rev (`mr --version`:
`0.1.0-dev (6b210222)`), and `cargo build --bin mnemonic` (debug) from this
tree. Each install used its own `CARGO_TARGET_DIR` under the worktree's
gitignored `target/`; output captured once per command to a file.

| Command | Exit |
|---|---|
| `cargo install` x4 (mk, md, ms, mr; pinned tags and rev) | 0, 0, 0, 0 |
| `cargo build --bin mnemonic` | 0 |
| `make audit MNEMONIC_BIN=... MD_BIN=md MS_BIN=ms MK_BIN=mk MR_BIN=mr FIXTURES_DIR=...` (as the "Audit manual" step) | 0 |
| `MNEMONIC_BIN=... MD_BIN=... MS_BIN=... MK_BIN=... MR_BIN=... bash ci/doc-flag-lint.test.sh` (as the mutation step) | 0 |
| `bash ci/doc-gate-guard.test.sh` | 0 |
| `bash ci/code-docs-check.sh` | 0 |

Results: audit lint 7/7 OK, `verify-examples` 62 transcripts pass, anchor-check
9 danglers match baseline. Mutation test: 12 passed, 0 failed. The string
"MR_BIN unset" appears 0 times in both outputs. Guard test: 158 passed.

Negative control (a copy of the manual, not the repo): renaming `--card` in the
`mr decode` Flags table of `45-mr.md` makes the lint FAIL in both directions
(`flag --card for mr decode is not documented in its section` and `flag
--zz-no-such-flag is documented for mr decode ... but the pinned binary does not
define it`) with `MR_BIN` set; without it the lint prints the "MR_BIN unset"
WARN. So the mr check is live.

## verify-examples and mr

`docs/manual/tests/verify-examples.sh` does not cover mr: no `$MR_BIN`
substitution, `mr` absent from the bare-binary guard, the Makefile
`verify-examples` target passes four `*_BIN` values, and `docs/manual/transcripts/`
has no mr transcript. Not extended. FOLLOWUPS entry `verify-examples-skips-mr`
filed in the same PR.

## CI

The `manual` job is RED on the PR head `b5046f95`, and the cause is not the
manual: `mnemonic-refugium` is a PRIVATE repository (`gh repo view
bg002h/mnemonic-refugium`: `isPrivate: true`; mnemonic-key, descriptor-mnemonic,
mnemonic-secret, mnemonic-gui and mnemonic-toolkit are all PUBLIC). The runner
has no credentials for it, so "Install mr" fails at the clone:

```
Updating git repository `https://github.com/bg002h/mnemonic-refugium`
error: failed to clone into: /home/runner/.cargo/git/db/mnemonic-refugium-60cac43a25820f08
  revision 6b210222c03e16f53f79ead4836b51d7339f4945 not found
  failed to authenticate when downloading repository
```

Every step before it (mk-cli, md-cli, ms-cli installs, pandoc, texlive, npm
tools, lychee) passed; the audit and mutation steps never ran (skipped after
the failed install).

- PR `manual` run (fail, 7m7s):
  https://github.com/bg002h/mnemonic-toolkit/actions/runs/40607552973
- push `manual` run on `ci/manual-mr-bin` (fail, 5m23s, same step):
  https://github.com/bg002h/mnemonic-toolkit/actions/runs/40607518732
- Green on the same head: sibling-pin-check, examples, quickstart,
  technical-manual, manual-gui, release. The `rust` workflow was still running
  when this report was written (not waited for: the blocker is the `manual`
  job, and nothing in `rust` reads `manual.yml`).

Local evidence that the manual text is fine once `mr` is installable: see Gate.

## STOP

Open choice, not made here: how the `manual` job gets read access to the
private `mnemonic-refugium` repository. The brief's `cargo install --git ...
--rev ... mr-cli` line cannot work unauthenticated, and the toolkit's workflows
have no precedent for private-repo access (the only `repository:` checkouts
are public repos; the only secrets are the minisign keys). Options for Brian:

1. Make `mnemonic-refugium` public. The brief's line then works unchanged.
2. Add a read-only credential for `mnemonic-refugium` as a repository secret on
   `bg002h/mnemonic-toolkit` (a fine-grained PAT with Contents: read on that one
   repo, or a deploy key), and change "Install mr" to use it (for example a
   `git config --global url."https://x-access-token:${TOKEN}@github.com/".insteadOf
   "https://github.com/"` line plus `CARGO_NET_GIT_FETCH_WITH_CLI=true`, scoped to
   that step). Needs the secret created by the owner; a pull request from a fork
   gets no secret, so the required `manual` context would go red for such PRs.
3. Keep the install out of CI: make the mr lint step conditional on the secret
   or on repository visibility (weaker: the check silently does not run when
   the credential is absent, which is the state this task set out to remove).

Nothing was merged or marked ready; the PR stays a draft. No manual text or
`scripts/install.sh` was changed to work around this.
