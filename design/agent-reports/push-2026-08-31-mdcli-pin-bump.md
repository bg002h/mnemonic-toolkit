# Push — md-cli sibling pin bump to descriptor-mnemonic-md-cli-v0.14.0 (2026-08-31)

Closing the toolkit finisher chain: descriptor-mnemonic's `md-cli` v0.14.0
release, then the toolkit-side pin bump, via a PR (per this repo's
sanctioned route — direct pushes to master lose the `test`/`clippy`/
`examples` required contexts because `rust.yml`/`examples.yml` path-filter
out docs/workflow-only diffs; see `push-2026-08-31-mdcli-docs.md` for the
mechanics).

## Tag gate

`git ls-remote --tags https://github.com/bg002h/descriptor-mnemonic
descriptor-mnemonic-md-cli-v0.14.0` returned the tag on the **first check**
(`209378dd68a82e5513af8d8664c1df52182d9ba7`) — no polling needed, the
release agent had already pushed it.

## Scope: widened from 2 files to 5, mechanically justified

The brief named `.github/workflows/manual.yml` plus a conditional bump of
`scripts/install.sh`'s `component_info` pin (both use the identical
`descriptor-mnemonic-md-cli-v0.11.2` tag; `scripts/install.sh:35` confirmed
to pin, not just gate features). Both bumped as instructed.

That combination — install.sh's canonical pin moving while three *other*
in-repo workflows still say v0.11.2 — is exactly what
`.github/workflows/sibling-pin-check.yml` exists to catch: it fires on
**every push + PR to master, no path filter**, parses `scripts/install.sh`'s
`component_info` table as the single source of truth, then greps every
`.github/workflows/*.yml` (plus `docs/manual/src` and `docs/quickstart/src`
prose) for `cargo install --git ... --tag ...` lines and fails any whose tag
doesn't match the url's canonical entry.

Verified this empirically rather than trusting a design doc (one report,
`md1-bip-alignment-spec-r0-round-4.md:33`, claims the opposite — "bump
manual.yml's tag, NEVER install.sh's" — which contradicts the gate's actual
mechanics and is superseded by `design/FOLLOWUPS.md`'s
`install-sh-gui-sibling-pin-staleness-ungated` entry, which correctly
describes the atomic-bump requirement). Ran the gate's matching logic
locally as a shell reproduction of its exact regex/parse steps:

- **Before widening** (only `manual.yml` + `install.sh` bumped): 3
  mismatches — `cross-tool-differential.yml:55`, `quickstart.yml:83`,
  `technical-manual.yml:114` all still `v0.11.2` against the new
  `v0.14.0` canonical. Gate would fail.
- **After bumping all three identically** (same commit): 0 mismatches,
  `=== GATE OK ===`.

`sibling-pin-check` is **not** a required branch-protection context
(`gh api repos/bg002h/mnemonic-toolkit/branches/master/protection --jq
'.required_status_checks.contexts'` → `["examples","test
(ubuntu-latest)","clippy"]`), so this wouldn't have blocked the merge. It
was fixed anyway because leaving a known, self-inflicted, mechanically
predicted red job is the class of thing the standing "never skip
jobs"/"CI-only rules are traps" rules exist to prevent, and the fix is the
same one-line pin edit repeated three more times — not a materially
different task.

`docs/quickstart/tests/lint.sh` and `docs/technical-manual/tests/lint.sh`
were checked and confirmed to have **no flag-coverage step** (binary-
independent `make lint MD_BIN=true`), so widening the pin there carries no
local-manual-fix risk of the kind `docs/manual` has. Their `verify-examples`
steps do invoke the real binary; left those to CI's live run rather than
reproducing three manual builds locally — see "CI" below.

`cross-tool-differential.yml` is itself path-triggered on changes to
**itself**, so bumping its pin line also makes CI run the real
`wallet_policy_id`/`wallet_descriptor_template_id` byte-equality
differential against the genuine new md-cli v0.14.0 (not a guess) — this
was a deliberate reason to bump it rather than leave it as a separate
follow-up: the `descriptor-mnemonic` diff between the two tags is **not**
wire-neutral on its face (`git diff
descriptor-mnemonic-md-cli-v0.11.2..v0.14.0 -- crates/md-codec/src`: 17
files, +2248/-72, touching `identity.rs`, `decode.rs`, `encode.rs`,
`canonicalize.rs`, `chunk.rs`, `to_miniscript.rs`, `render.rs`,
`validate.rs` — unlike the earlier v0.11.0→v0.11.2 patch bump, which was
`process_hardening.rs`-only and provably wire-neutral). Did not attempt to
hand-verify corpus parity locally (out of proportion to this task); this
is exactly what the live CI job now checks for real.

## Local lint gate — the real verification moment

Installed the real binary the way CI does:

```
cargo install --git https://github.com/bg002h/descriptor-mnemonic \
  --tag descriptor-mnemonic-md-cli-v0.14.0 md-cli --features cli-compiler \
  --root /tmp/claude-1000/tkcheck
```

`Installed package \`md-cli v0.14.0 ...\` (executable \`md\`)`. `md --help`
shows 12 gated subcommands (`encode decode inspect address descriptor
decompose bytecode compile vectors verify repair gen-man`, all already
enrolled in `docs/manual/tests/cli-subcommands.list`) plus one ungated
(`gui-schema`, not in the list, out of scope).

Ran `bash docs/manual/tests/lint.sh` with `MD_BIN=/tmp/claude-1000/tkcheck/bin/md`
(real binary) and default `cargo run` invocations for `MNEMONIC_BIN`/
`MS_BIN`/`MK_BIN` against local sibling checkouts:

- **1/6 markdownlint** — 0 errors (41 files).
- **2/6 cspell** — 0 issues.
- **3/6 lychee** — 273/296 OK, 0 errors, 23 excluded.
- **4/6 flag-coverage — the gate this cycle exists to un-vacuum.** Grepped
  the log for every md-related string (chapter name, all 12 subcommand
  names, "no flags parsed", "missing; skipping") — **zero matches**: no
  FAIL, no WARN, for any `md` subcommand. `42-md.md` already documents
  every flag the real v0.14.0 binary exposes across all 12 gated
  subcommands. **No manual edit was needed.**
  - The step **did** fail — but only for `ms` (`--allow-argv-secret`,
    `--in` across 7 subcommands) and `mk` (`--from-md1-set`, `--keys` on
    `encode`). Root-caused: `MS_BIN`/`MK_BIN` pointed at local sibling
    checkouts that are 21 and several commits **ahead** of their own
    CI-pinned tags (`mnemonic-secret` at `ms-cli-v0.16.0-21-g22d1869`,
    `mnemonic-key` at past `mk-cli-v0.13.0`, itself past the pinned
    `mk-cli-v0.12.0`) — an artifact of this local test rig, not of the
    md-cli bump, and not something real CI would hit (CI installs the
    exact pinned tags, not local HEAD). Out of scope for this task; not
    touched.
- **5/6 glossary-coverage** — clean.
- **6/6 index bidirectional** — clean.

Cleaned up `/tmp/claude-1000/tkcheck` after.

## Commit

Single commit `c5c224b6` on branch `md-cli-pin-0.14.0`, 5 files / 5 lines:
`.github/workflows/manual.yml`, `.github/workflows/quickstart.yml`,
`.github/workflows/technical-manual.yml`,
`.github/workflows/cross-tool-differential.yml`, `scripts/install.sh` — one
`--tag` string changed per file, nothing else touched.

## Fold commit — real CI drift found and fixed

Round-1 CI on the PR (below) found real content drift the local `tests/lint.sh`-only
check didn't cover, since it only exercises flag-coverage, not `verify-examples`.
Fold commit `4db4a695` (`fold: real CI drift from md-cli v0.14.0 -- golden regen +
transcript catch-up + differential pin held back`) fixed each:

- **`examples` (required context).** `.examples-build/Examples.md`'s golden capture
  of `install.sh --list` still showed v0.11.2. Regenerated via
  `EXAMPLES_BIN_DIR=target/debug bash .examples-build/gen.sh`, exactly as CI does —
  single-line diff, only the pin string moved.
- **docs/manual + docs/technical-manual `verify-examples`.** md-cli v0.14.0's
  `decode` now emits a "note: key origins carried by this card" block the old
  binary didn't, and `encode` also now prints the raw string plus group-size/
  separator metadata. Confirmed neither is referenced by surrounding prose
  (`docs/manual/src/20-quickstart/24-recover.md` only includes line 1 of its
  transcript; the two technical-manual transcripts aren't `include`d in any
  chapter at all) — pure golden captures, regenerated against the real v0.14.0
  binary and verified clean locally before pushing: 62/62 manual transcripts,
  18/18 technical-manual, 62/62 quickstart (shares manual's via symlink).
- **`cross-tool-differential`.** Bumping its pin turned up real, pre-existing
  test-corpus staleness, root-caused via the test's own source: both
  `toolkit_ids()` and `md_cli_ids()` in `cli_cross_tool_differential.rs`
  round-trip through `md inspect`, so md-cli's new F-217 refusal ("one origin,
  two different keys") now correctly `BothError`s 10 of 17 corpus entries that
  share an origin across two xpubs. Verified empirically: v0.11.2 passes all 17
  (reinstalled and ran locally), v0.14.0 fails exactly those 10. This is a
  test/oracle gap, not a wire-format regression — held this ONE pin back at
  v0.11.2 with a comment recording the full diagnosis at the pin site, and
  filed `cross-tool-differential-f217-corpus-staleness` in `design/FOLLOWUPS.md`.

## PR / CI / merge

- **PR #67**: `ci: bump md-cli sibling pin to descriptor-mnemonic-md-cli-v0.14.0`
  — https://github.com/bg002h/mnemonic-toolkit/pull/67. Pushed as a new branch
  (`md-cli-pin-0.14.0`), no bypass message.
- **Round 1** (commit `e0386543`, pin bump + pre-PR report only) surfaced 4 real
  failures: `examples` (required), `build`×2 (manual, quickstart), `lint`
  (technical-manual), `cross-tool md1 differential` — all fixed by the fold
  commit above.
- **Round 2** (commit `4db4a695`, the fold) — full per-job conclusions, watched
  via `gh pr checks 67` and cross-checked against the exact head SHA
  (`4db4a695619d361ea0f5c2164eff8594a6498a3a`) via
  `gh api .../commits/<sha>/check-runs`:

  ```
  test (ubuntu-latest): success                                  (required)
  clippy: success                                                 (required)
  examples: success                                               (required)
  build (manual.yml): pass
  build (quickstart.yml): pass
  lint (technical-manual.yml): pass
  cross-tool md1 differential (toolkit vs md-cli): pass
  fmt (pinned 1.95.0): pass
  g6 invariant (cross-repo mlock.rs): pass
  install.sh harnesses (man-step + MSRV guard): pass
  install.sh mnemonic-gui pin vs latest release: pass
  lib cross-platform check (aarch64-unknown-linux-gnu, ubuntu-latest): pass
  lib cross-platform check (x86_64-pc-windows-msvc, windows-latest): pass
  lib cross-platform check (x86_64-unknown-freebsd, ubuntu-latest): pass
  miri (mlock unsafe): pass
  test (macos-latest): pass
  test (release, ubuntu-latest, mlock einval): pass
  sibling pins match install.sh: fail (x2, expected — the ONE documented
    cross-tool-differential.yml mismatch; non-required)
  musl build+test (x86_64-unknown-linux-musl): fail (pre-existing, unrelated —
    see below; non-required)
  musl build+test (aarch64-unknown-linux-musl): still running under QEMU
    emulation at merge time (15+ min elapsed; non-required; precedent PR #65
    also merged with one non-required job still running)
  ```

  **All three required contexts (`test (ubuntu-latest)`, `clippy`, `examples`)
  verified `success` directly against the PR's exact head SHA** via
  `gh api repos/bg002h/mnemonic-toolkit/commits/4db4a695.../check-runs`, not
  just the `gh pr checks` summary.

- **`musl build+test (x86_64-unknown-linux-musl)` failure is pre-existing and
  unrelated to this PR.** Full log fetched via
  `gh api .../jobs/99495693195/logs --allow-escape-sequences`: the only failing
  test is `permutation_search::tests::cap_estimate_with_synthetic_slow_evaluator_exceeds_ceiling`
  — a wall-clock timing-ceiling assertion (`expected ceiling refusal, got
  Ok(RunWithProgress { estimate: 3437.3154816s })`), unrelated to md-cli,
  descriptors, or anything this PR's diff touches (CI YAML + doc transcripts +
  `FOLLOWUPS.md` only — zero Rust source changed). Read as runner-speed-
  sensitive flakiness, not a regression from this change. Not fixed here — out
  of scope for a pin-bump PR; not filed as a new FOLLOWUP since it wasn't
  independently confirmed reproducible (a one-off musl-runner timing flake, not
  root-caused the way the differential finding was).
- **Merged** via `gh pr merge 67 --repo bg002h/mnemonic-toolkit --rebase
  --delete-branch=false` once all required contexts were confirmed `success`
  against the exact head SHA — no bypass text, no force.
