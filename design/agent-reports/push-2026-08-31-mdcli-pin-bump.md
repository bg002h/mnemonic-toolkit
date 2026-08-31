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

## PR / CI / merge

Recorded in a follow-up append to this same file once the PR is open and
CI concludes (see the push agent's next actions) — placeholder left
intentionally rather than guessing outcomes.
