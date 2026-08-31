# DOCS pass — `md` CLI reference catch-up (mdcli-mini wallet-form-converter cycle)

Cross-repo docs pass, P7 step 2 of the release cycle, ruled into this phase by
the operator on 2026-08-31. Target repo: `mnemonic-toolkit`, worktree
`mnemonic-toolkit-mddocs` (branch `mdcli-mini-docs`), off `master` at
`2ecb0010` (build: opt-level 2 on the test and dev profiles).

Authority: the built binary at
`/scratch/code/shibboleth/descriptor-mnemonic-mdcli-mini/target/debug/md`,
used read-only — `--help` at top level and on every subcommand, plus live
invocations to confirm the refusal/warning behaviors named below (the
`--help` text alone doesn't show runtime message bodies or exit codes).

## What changed

File: `docs/manual/src/40-cli-reference/42-md.md`.

- Intro subcommand count corrected `Eight` → `Twelve` (matches what this
  chapter now documents; see "found beyond scope" below — the true binary
  count is higher still).
- **New section `md descriptor`**: full flag table (shares
  `--template`/`--key`/`--fingerprint`/`--path`/`--from-mk1`/
  `--from-mk1-file`/`--seat`/`--network`/`--chain`/`--change`/`--json` with
  `md address`, cross-referenced rather than duplicated) plus its own
  `--emit md1` and `--verify-against <md1|FILE>`, with the exit-code
  contract (`0` spend-equal / `5` NOT spend-equal / `1`,`2` errors)
  documented as the load-bearing part of that flag, per the brief.
- **New section `md decompose`**: positional (`-` for stdin), `--in <FILE>`,
  `--emit` with its full six-value set (`all`/`template`/`keys`/
  `fingerprints`/`descriptor`/`commands` — the checklist named only
  `template|keys|commands`; the binary's actual set is wider and the doc
  reflects the wider set), the `/**` BIP-389 shorthand acceptance, and the
  receive/change-pair and `listdescriptors`-JSON refusals.
- **`md address`**: flag table rewritten from a 9-row stub to the full
  13-row current surface (added `--from-mk1`, `--from-mk1-file`, `--seat`,
  origin-notated `--key`; documented the inline-origin > `--path` >
  `--key`-bracket precedence, verified against source comments — see below).
- **`md encode`**: added `--experimental` (mechanically required — see gate
  output); corrected `--group-size`/`--separator` text, which had drifted
  (the manual still described a "display grouping" and hyphen/comma
  separators; the binary now groups only the stderr engraving card and
  accepts `space`/literal-space only).
- **`md verify`**: added `--experimental`.
- **Refusal-behavior prose** (checklist items, each confirmed by running the
  binary, not just reading `--help`):
  - BIP-388 repeated-key admission taxonomy: full explanation under
    `md encode` (refuse, `unsupported:` prefix, exact quoted message),
    full explanation of the WARN side under `md decode` (`md: warning:`
    prefix, same body text, exit 0), one-line cross-references at
    `md inspect`, `md bytecode`, `md verify`. Confirmed live against the
    frozen fixture `crates/md-cli/tests/fixtures/n1/r-n1a-keyed.txt`:
    `md decode` on that card exits 0 with `md: warning: @0 appears at 2
    use sites...`; `md descriptor` on the same card exits 1 with
    `md: unsupported: @0 appears at 2 use sites...`.
  - mk1/md1 positional mix-ups: documented under `md descriptor` (applies
    equally to `md address`). Confirmed live: an mk1 string on the
    positional and an md1 string among `--from-mk1`'s values each produce
    a distinct named redirect (quoted in the doc), not a generic codec
    error.
- `docs/manual/tests/cli-subcommands.list`: added `md descriptor` and
  `md decompose` lines. Without this the flag-coverage step never invokes
  `--help` on either subcommand, so it would report zero coverage for
  everything just added — the enumeration is what makes the gate real going
  forward, per the list file's own header comment.

## Gate output

Ran the actual repo gate, not a re-derivation of it:

```
bash tests/lint.sh SRC_DIR=... TESTS_DIR=... \
  MNEMONIC_BIN=mnemonic MD_BIN=<mdcli-mini debug binary path> MS_BIN=ms MK_BIN=mk
```

```
[lint] === 1/6 markdownlint ===   Summary: 0 error(s)
[lint] === 2/6 cspell ===         Issues found: 0 in 0 files.
[lint] === 3/6 lychee ===         296 Total, 0 Errors
[lint] === 4/6 flag-coverage ===  (silent — no `err` lines emitted)
[lint] === 5/6 glossary-coverage ===  (silent)
[lint] === 6/6 index bidirectional === (silent)

[lint] OK
```

`MD_BIN` was pointed at the mdcli-mini debug binary directly (the authority
for this pass), not at `md` on `$PATH` (which is shell-aliased to `mkdir -p`
on this machine) and not at the Makefile's `cargo run` default (which builds
the *original* `descriptor-mnemonic` checkout, lacking this cycle's surface).
Independently cross-checked flag-coverage per-subcommand before the full run
(extracting `--[a-z][a-z0-9-]+` from each `md <sub> --help` and grepping
`42-md.md` for each token, mirroring `tests/lint.sh`'s own logic): all 12
`md` rows in `cli-subcommands.list` — `encode decode inspect address
bytecode compile vectors verify repair gen-man descriptor decompose` —
report every flag documented.

## Found stale/out of scope (reported, not fixed)

- **CI's flag-coverage never actually checks the current surface.**
  `.github/workflows/manual.yml` installs `md-cli` from
  `descriptor-mnemonic-md-cli-v0.11.2` (a released tag), not the
  mdcli-mini surface. Until a new `md-cli` release is cut and that
  workflow's pinned tag is bumped, CI's own flag-coverage step will run
  against the OLD binary — it degrades to a silent WARN ("no flags parsed")
  for `descriptor`/`decompose` (unrecognized-subcommand text has no `--`
  tokens to extract) rather than a hard failure, so this pass's additions
  are locally gated but not yet CI-gated. Release-engineering concern, out
  of scope for a docs pass.
- **`md`'s own `--help` summary for `address` is now blank** — an empty
  `about` string at the top level and on `md address --help`, while the
  sentence that used to describe `address` ("Derive bitcoin addresses from
  a wallet-policy-mode descriptor...") now opens `descriptor`'s doc
  comment instead. Reads like a doc-comment that moved with the refactor
  and was never given a replacement. The manual keeps its own (still-true)
  one-line description for `address`, so no reader-facing gap resulted, but
  the source-level gap is real. Not mine to fix (source lives in
  `descriptor-mnemonic-mdcli-mini`, out of scope for this repo).
- **`md gui-schema`** exists at the top level (`Emit a machine-readable
  JSON description of this CLI's flag surface...`) and is undocumented in
  the manual and absent from `cli-subcommands.list`. Not in the checklist
  for this pass; `mnemonic gui-schema` is already covered elsewhere in the
  manual, so `md`'s sibling is the gap. Left alone.
- **`--in <FILE>` / `--out <FILE>`** exist on `encode`/`decode`/`inspect`/
  `bytecode`/`repair`/`decompose` (SPEC §6b file-input/output) and are not
  called out as their own flag rows for `encode`/`decode`/`inspect`/
  `bytecode`/`repair` (only `decompose`'s got a row, since it's new this
  pass). The flag-coverage gate reports these as present only because
  `--in` is a literal substring of `--index` (already documented on
  `address`) and `--out` of other rows elsewhere in the file — a known
  weakness of the gate's plain substring match, not a defect I introduced.
  Genuinely worth its own row on each command; left as a found gap since it
  wasn't in this cycle's checklist and the checklist's own explicit
  boundary was `descriptor`/`address`/`decompose`/the two refusal classes.
- **`--unspendable-key`'s description on `md encode`** looks copy-pasted
  from `md compile`'s (identical wording in the manual for both) but the
  binary's actual `encode --help` text for it now differs slightly in
  emphasis (doesn't restate "other forms rejected" as flatly). Left
  unchanged — not factually wrong, just not verbatim, and outside the
  checklist.

## Verification notes

- The BIP-388 repeated-key refuse/warn split was cross-checked against
  `crates/md-cli/src/parse/reuse.rs`'s `Disposition` enum and its call
  sites (`build.rs` → `Refuse` for `encode`/`descriptor`/`address`;
  `decode.rs`/`bytecode.rs`/`inspect.rs`/`verify.rs` → `Warn`) before being
  written up, then confirmed live against the frozen fixture cards — not
  taken from the brief's framing alone. One correction made mid-pass: the
  brief's implicit framing put `md verify --template` on the refuse side;
  the test `verify_template_warns_and_completes_on_a_refused_shape`
  (`crates/md-cli/tests/n1_admission_taxonomy.rs:450`) and `verify.rs:57`'s
  `Disposition::Warn` call both show it is a READ verb here, not a mint
  verb — corrected before committing.
- The inline-origin > `--path` > `--key`-bracket precedence (documented on
  `md address`, inherited by `md descriptor`) was verified against
  `crates/md-cli/src/cmd/build.rs:191` and `crates/md-cli/src/parse/path.rs:60-61`'s
  own comments ("precedence is inline > --path > bracket") before being
  stated as fact.

## Final state

- Branch: `mdcli-mini-docs`, worktree `mnemonic-toolkit-mddocs`.
- Commit: `95e3723d93d1366549dd5507349d9592a271ebcb` — "docs(md): catch up
  42-md.md with the mdcli-mini wallet-form-converter surface" (2 files
  changed: `docs/manual/src/40-cli-reference/42-md.md`,
  `docs/manual/tests/cli-subcommands.list`).
- Not pushed anywhere, per the brief.
