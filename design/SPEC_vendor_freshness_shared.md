# SPEC: one shared vendor-freshness gate for five repos

Status: DRAFT, one open point (STOP S-1, "Shared-file pinning" below). Every
other point is decided here. Source: script audit 2026-11-08 row 5 and its
Reasons "5." (coordinator `agent-reports/architect-script-audit-2026-11-08.md`).
Supersedes, for check (4) only, `design/SPEC_vendor_freshness_ci_guard.md`
(its trust-on-first-use digest, §3.3 and §5).

Repos and the SHAs every measurement below was taken at:

| repo | default branch | SHA |
|---|---|---|
| mnemonic-toolkit (toolkit) | master | 3753b5c7bf90bcdb5354dbd18a7612cde087406a |
| mnemonic-refugium (mr, private) | master | 08b6ccb7e4c1ad8bc5f96065593ab77b66eda91a |
| mnemonic-key (key) | main | 613f8e3482e6f2c6a0ba2e639ad40142d62492be |
| mnemonic-secret (secret) | master | 7a62298732e5bbf0b7d35d223659eaae5318537a |
| descriptor-mnemonic (dm) | main | 0d96d2c89272be2b50df644987205b292c3afb33 |

The gate `design/spec-vendor-freshness/gate.sh` runs every command and checks
every expected-output block in this spec against those five checkouts
("Build gate" at the end).

## 1. Why

Each of the five repos commits a `vendor/` tree that the release build reads
with `--offline --locked`, and each runs its own copy of
`ci/repro/vendor-freshness.sh` (toolkit 311 lines, mr 130, key 93, secret 79,
dm 88). The copies have drifted apart:

- toolkit runs four checks: (1) resolution under an empty CARGO_HOME, (2) every
  file's sha256 against `.cargo-checksum.json`, (3) every registry crate's
  `package` digest against the Cargo.lock checksum, and (4) a digest of each
  git source's manifest, written by hand (trust on first use) into the
  script's `GROUNDED_GIT_SOURCES` array at every pin bump.
- mr runs (1) under the caller's CARGO_HOME, then fetches each git source from
  upstream at the pinned rev and compares bytes (`ground()`). It has no (2) or
  (3) for registry crates.
- key, secret and dm run (1) only, under the caller's CARGO_HOME.

So key, secret and dm pass the F-354 class: a git source vendored from the
wrong rev is self-consistent, so it resolves (1) and matches its own manifest
(2). Only a comparison against upstream at the Cargo.lock rev sees it. Their
non-hermetic CARGO_HOME is F-675: a warm `~/.cargo/git` hides a missing
source stanza. Section 9 shows each of these failures running.

The survivor is toolkit's script, with its lock-driven source stanzas, checks
(1)-(3) and hermetic home, and with check (4) replaced by mr's live upstream
re-derivation. That replacement needs no edit to the script at a pin bump. The
per-repo facts move into a data file. The script and its mutation test become
shared files, byte-identical in all five repos.

## 2. Files

The reference implementation is committed with this spec. Each repo's PR copies
it byte for byte; no PR edits it.

| this spec's file | becomes, in each of the five repos |
|---|---|
| `design/spec-vendor-freshness/proto/vendor-freshness.sh` | `ci/repro/vendor-freshness.sh` (replaces the current one) |
| `design/spec-vendor-freshness/proto/vendor-freshness.test.sh` | `ci/repro/vendor-freshness.test.sh` (replaces toolkit's; new in the others) |
| `design/spec-vendor-freshness/proto/data/<repo>.txt` | `ci/repro/vendor-git-sources.txt` (new, per repo, not shared) |

The `vf:proto-sha256` block is the output of
`sha256sum vendor-freshness.sh vendor-freshness.test.sh`, run in
`design/spec-vendor-freshness/proto/` (gate C10). In each repo the same hashes
pin `ci/repro/vendor-freshness.sh` and `ci/repro/vendor-freshness.test.sh`.

<!-- vf:proto-sha256 -->
```
9a2761a6e50a5fda600347bd84af90f8b6c72edfd5eb731b263ff63813d11241  vendor-freshness.sh
14386f525d7278f0aa210b10f658c6d8d0b4dc0c0cbc0de28c954a7f44f86b27  vendor-freshness.test.sh
```

## 3. Interface and exit codes

- Invocation: `bash ci/repro/vendor-freshness.sh`, from any directory. The
  script changes to the repo root, which it finds from its own path.
- Arguments: none. Any argument is a refusal (A11).
- Environment:
  - `VF_SOURCES` names the data file. Its default is
    `ci/repro/vendor-git-sources.txt`, and a relative path is taken from the
    repo root.
  - `PATH` must provide `cargo`, `git` and `python3`. The script runs
    `python3 -I`.
  - No other variable is read. `CARGO_HOME` and git's global and system
    configuration are overridden; see section 6.
- Exit codes:
  - 0: every check passed.
  - 1: a refusal, or any check red.
  - No other code is used. A failure of bash or python3 before the script's
    own logic runs gives the interpreter's code; the test does not cover it.
- Output:
  - Progress and OK lines go to stdout.
  - Every refusal and red goes to stderr. Its first line starts
    `::error::vendor-freshness: `, and detail lines start `  - ` (6 spaces for
    a continuation).
  - Within an OK line, the separator is ASCII ` -- `.
- Scratch: `target/vendor-freshness.XXXXXX` under the repo root (`target/` is
  gitignored in all five repos). It is removed on every exit path.
- Network: (1)-(3) are offline. Check (4) fetches from each row's host and is
  skipped when Cargo.lock has no git source.

## 4. Data file

Grammar of `ci/repro/vendor-git-sources.txt` (the default `VF_SOURCES`):

- Lines that are empty, or whose first character is `#`, are ignored.
- Every other line is a row and must match
  `^([A-Za-z0-9_.+-]+)\|(https://[^|\s]+)\|(\.|[A-Za-z0-9_.+-]+(?:/[A-Za-z0-9_.+-]+)*)$`,
  and no path segment of field 3 may be `..`.
- Field 1 is the vendor dir name (`vendor/<field 1>`).
- Field 2 is the url exactly as Cargo.lock writes it after `git+` and before
  `?`.
- Field 3 is the crate's directory inside the upstream repo, or `.` for the
  repo root.
- Two rows may not name the same vendor dir (A12).
- Each row's url must be a url Cargo.lock pins from git (A8), and each url
  Cargo.lock pins from git must have a row (A6).
- Rows are processed in sorted vendor-dir order, whatever their order in the
  file.

Every repo keeps the file. It holds only its three comment lines when
Cargo.lock has no git source (key). The rows below were resolved from each
repo's Cargo.lock at the SHA above. Gate C2 checks that each block equals the
committed data file, and that its urls equal the lock's git urls.

<!-- vf:data-mnemonic-toolkit -->
```
# ci/repro/vendor-git-sources.txt -- one row per git source Cargo.lock pins, read
# by ci/repro/vendor-freshness.sh (check 4 re-derives each from upstream).
# Row: <vendor dir>|<url exactly as in Cargo.lock>|<crate dir in upstream, or .>
md-codec|https://github.com/bg002h/descriptor-mnemonic|crates/md-codec
miniscript|https://github.com/rust-bitcoin/rust-miniscript|.
```

<!-- vf:data-mnemonic-refugium -->
```
# ci/repro/vendor-git-sources.txt -- one row per git source Cargo.lock pins, read
# by ci/repro/vendor-freshness.sh (check 4 re-derives each from upstream).
# Row: <vendor dir>|<url exactly as in Cargo.lock>|<crate dir in upstream, or .>
md-codec|https://github.com/bg002h/descriptor-mnemonic|crates/md-codec
miniscript|https://github.com/rust-bitcoin/rust-miniscript|.
```

<!-- vf:data-mnemonic-key -->
```
# ci/repro/vendor-git-sources.txt -- one row per git source Cargo.lock pins, read
# by ci/repro/vendor-freshness.sh (check 4 re-derives each from upstream).
# Row: <vendor dir>|<url exactly as in Cargo.lock>|<crate dir in upstream, or .>
```

<!-- vf:data-mnemonic-secret -->
```
# ci/repro/vendor-git-sources.txt -- one row per git source Cargo.lock pins, read
# by ci/repro/vendor-freshness.sh (check 4 re-derives each from upstream).
# Row: <vendor dir>|<url exactly as in Cargo.lock>|<crate dir in upstream, or .>
mnemonic-io-lib|https://github.com/bg002h/mnemonic-engrave|crates/mnemonic-io-lib
```

<!-- vf:data-descriptor-mnemonic -->
```
# ci/repro/vendor-git-sources.txt -- one row per git source Cargo.lock pins, read
# by ci/repro/vendor-freshness.sh (check 4 re-derives each from upstream).
# Row: <vendor dir>|<url exactly as in Cargo.lock>|<crate dir in upstream, or .>
miniscript|https://github.com/rust-bitcoin/rust-miniscript|.
```

A later pin bump (a new rev for the same url) needs no edit to the data file
or the script. A new git source needs a new row, in the same PR as the
Cargo.lock change.

## 5. Refusals before any check, in this order

The script stops at the first refusal and exits 1. Each text below is the
whole `::error::` line after `::error::vendor-freshness: `. `<path>` is the
data file's path as given, so `ci/repro/vendor-git-sources.txt` by default.
The test row that proves each one is in the last column, and its exact output
is in section 8.

| # | condition | text | row |
|---|---|---|---|
| P1 | any argument | `takes no arguments; got: <args joined by one space>` | A11 |
| P2 | data file missing | `data file <path> not found. Every repo keeps one; it holds only comments when Cargo.lock has no git source.` | A10 |
| P3 | a row fails the grammar | `<path>:<line no>: malformed row '<line>'. A row is <vendor dir>\|<url as in Cargo.lock>\|<crate dir in upstream>: three fields, no spaces, an https:// url, and a crate dir that is '.' or a relative path without '..'.` | A9 |
| P4 | a vendor dir has a second row | `<path>:<line no>: vendor dir '<dir>' already has a row (line <first line no>).` | A12 |
| P5 | a Cargo.lock `source = "git+…"` that is not `git+<url>?rev=<40 hex>#<the same 40 hex>` | `Cargo.lock has a git source not pinned by a 40-hex rev: <the source value>. Pin it with rev = "<40-hex>" or take it from crates.io. Failing closed.` | A7 |
| P6 | one url at two revs | `Cargo.lock pins <url> at more than one rev (<rev a>, <rev b>). This gate re-derives one rev per url. Failing closed.` | A13 |
| P7 | a lock git url with no row | `Cargo.lock pins git source <url>?rev=<rev>, which has no row in <path>. A git source has no published tarball digest, so only check (4) can verify its vendored bytes. Add a row (<vendor dir>\|<url>\|<crate dir in upstream>) or drop the source.` | A6, G6 |
| P8 | a row whose url Cargo.lock does not pin from git | `<path>:<line no>: row '<dir>' names <url>, which Cargo.lock does not pin from git. Drop the row, so the gate's exemptions match the lock.` | A8 |
| P9 | `cargo` or `git` not on PATH | `<cargo\|git> not found on PATH.` | untested |

In the table, `\|` stands for a literal `|`. P7 and P8 apply in that order to
each url, sorted.

## 6. Checks and the hermetic environment

Each check prints its OK line only when it passes. Checks (2) and (3) collect
every defect, print them together, and exit 1 before (4) runs. The gate does
not touch the network for a tree it already knows is bad.

**(1) Resolution.**

- The script prints
  `vendor-freshness: (1/4) resolving Cargo.lock against committed vendor/ (offline, locked, empty CARGO_HOME; git sources: <dir>=<40-hex rev> ...|none) ...`.
- It runs `cargo metadata --format-version 1 --locked --offline` with
  `CARGO_HOME=<scratch>/cargo-home`, a new empty directory, and these
  `--config` values:
  - `source.crates-io.replace-with="vendored-sources"`;
  - for each lock git url, in sorted url order, the key
    `source."git+<url>?rev=<rev>"` with `.git="<url>"`, `.rev="<rev>"` and
    `.replace-with="vendored-sources"`;
  - `source.vendored-sources.directory="vendor"`.
- cargo's stdout is discarded and its stderr passes through.
- Nothing from the caller's CARGO_HOME is read (A2 plants a poisoned
  `config.toml` there and stays green).
- On failure, cargo's own stderr comes first, then
  `::error::vendor-freshness: vendor/ is out of sync with Cargo.lock: the --offline --locked release build cannot resolve a dependency from the committed vendor/ tree. Run 'cargo vendor --locked vendor/' and commit the result. This is the toolkit v0.74.0 release failure class, caught at PR time.`
- OK: `vendor-freshness: (1/4) OK -- vendor/ satisfies Cargo.lock.`

**(2) Integrity.** For every `vendor/*/.cargo-checksum.json`, every listed
file exists and its sha256 equals the recorded value. Defects:

- `<file>: MISSING (recorded in <manifest>)`
- `<file>: CONTENT MISMATCH`, followed by the lines `recorded <hex>` and
  `on disk  <hex>`, each indented 6 spaces
- `<manifest>: unreadable checksum manifest (<python exception>)`

OK: `vendor-freshness: (2/4) OK -- <n> files across <m> crates match their recorded sha256.`

**(3) Registry provenance.** For every vendored crate whose manifest has a
non-null `package`, that digest equals the `checksum` of the Cargo.lock entry
with the same name and version. The crates with a null `package` must be
exactly the data file's rows. Defects:

- `<crate dir>/Cargo.toml: could not read name/version`
- `<crate dir>: PROVENANCE MISMATCH for <name> <version>`, followed by the
  lines `.cargo-checksum.json package <hex>` and
  `Cargo.lock checksum          <hex>`, each indented 6 spaces
- `vendored crate(s) with NO registry anchor and no row in <path>: <dirs, comma-space>`,
  followed by these two lines, each indented 6 spaces:
  `A git or path source cannot be checked against Cargo.lock (no` and
  `published tarball digest). Add a row for it, or drop it.`

When (2) or (3) found anything:

- The heading is `::error::vendor-freshness: <k> content defect(s) in vendor/:`.
- One `  - <defect>` line follows for each defect.
- The script exits 1.

OK: `vendor-freshness: (3/4) OK -- <a> crates anchored to Cargo.lock checksums; <g> git source(s) exempt by a row in <path> (<dirs, comma-space>|none).`

**(4) Git-source provenance (mr's re-derivation).**

- With no row: the script prints
  `vendor-freshness: (4/4) OK -- Cargo.lock has no git source; nothing to re-derive.`
  and exits 0.
- Otherwise it prints
  `vendor-freshness: (4/4) re-deriving <n> git source(s) from upstream at the Cargo.lock rev ...`.
- For each row in sorted vendor-dir order, in `<scratch>/upstream/<dir>`, with
  `GIT_TERMINAL_PROMPT=0 GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null`,
  the script runs:
  1. `git init -q`;
  2. `git fetch -q --depth 1 <url> <rev>`;
  3. `git -c advice.detachedHead=false checkout -q FETCH_HEAD`;
  4. `git rev-parse HEAD`, which must equal the rev.
- It then compares `vendor/<dir>` with `<checkout>/<field 3>`:
  - the vendored file set equals the manifest's `files` keys;
  - every file except `Cargo.toml` is byte-identical to upstream's;
  - Cargo.toml's name and version equal upstream's. Each is the first line
    matching `^name = "…"` or `^version = "…"`. A value absent on either side
    reads as `none`, and an absent upstream version counts as a mismatch.
- `cargo vendor` rewrites Cargo.toml, so that file's bytes are not compared.
- Defects, each prefixed `vendor/<dir>: `:
  - `missing`
  - `cannot fetch <url> at <rev>`
  - `fetched HEAD is not <rev>`
  - `<field 3> is not a crate directory in <url> at <rev>`
  - `file set differs from .cargo-checksum.json: <first 10 differing paths, comma-space>`
  - `<file>: not in upstream at <rev8>`
  - `<file>: differs from upstream at <rev8>`
  - `Cargo.toml: package name <n> is not upstream's <n>`
  - `Cargo.toml: version <v> is not upstream's <v>`
  - after 20 defects in one row: `... and <k> more`
- A passing row prints
  `vendor-freshness: (4/4) vendor/<dir> matches <url> at <rev8> (<n> files).`
- When any row has a defect:
  - the heading is
    `::error::vendor-freshness: (4/4) <k> git-source defect(s): a vendored git source is not upstream's tree at the Cargo.lock rev. A tree vendored from another rev passes (1)-(3); only this check sees it (F-354).`;
  - one `  - <defect>` line follows for each defect;
  - the script exits 1.
- OK: `vendor-freshness: (4/4) OK -- <dir>@<rev8>, ... re-derived from upstream.`

The texts are the proto's, byte for byte. Where this section paraphrases a
text, the proto decides.

Untested texts (the test file's header gives the same list and its reason):

- check (3)'s PROVENANCE MISMATCH and unanchored-set lines (cargo or the
  data-file refusals red first on every mutation that reaches them);
- P9 (no cargo or git);
- an unreadable manifest;
- `missing` in (4) (check (1) reds first);
- `fetched HEAD is not`;
- a package-name mismatch;
- the `... and <k> more` cap.

**Green output at the five SHAs.** These are measured. "Blocking waiting for
file lock" lines from a concurrent cargo are dropped, and `exit <code>` is
appended.

<!-- vf:green-mnemonic-toolkit -->
```
vendor-freshness: (1/4) resolving Cargo.lock against committed vendor/ (offline, locked, empty CARGO_HOME; git sources: md-codec=cf35d61af0f058029df841f4882945e53f86cd4b miniscript=ff4732e5f75aa555682343cb180fa72ee3e8e9d5) ...
vendor-freshness: (1/4) OK -- vendor/ satisfies Cargo.lock.
vendor-freshness: (2/4) OK -- 7772 files across 169 crates match their recorded sha256.
vendor-freshness: (3/4) OK -- 167 crates anchored to Cargo.lock checksums; 2 git source(s) exempt by a row in ci/repro/vendor-git-sources.txt (md-codec, miniscript).
vendor-freshness: (4/4) re-deriving 2 git source(s) from upstream at the Cargo.lock rev ...
vendor-freshness: (4/4) vendor/md-codec matches https://github.com/bg002h/descriptor-mnemonic at cf35d61a (396 files).
vendor-freshness: (4/4) vendor/miniscript matches https://github.com/rust-bitcoin/rust-miniscript at ff4732e5 (98 files).
vendor-freshness: (4/4) OK -- md-codec@cf35d61a, miniscript@ff4732e5 re-derived from upstream.
exit 0
```

<!-- vf:green-mnemonic-refugium -->
```
vendor-freshness: (1/4) resolving Cargo.lock against committed vendor/ (offline, locked, empty CARGO_HOME; git sources: md-codec=6cf6599ab71eb37adef836641095fb3c8756784f miniscript=ff4732e5f75aa555682343cb180fa72ee3e8e9d5) ...
vendor-freshness: (1/4) OK -- vendor/ satisfies Cargo.lock.
vendor-freshness: (2/4) OK -- 5126 files across 95 crates match their recorded sha256.
vendor-freshness: (3/4) OK -- 93 crates anchored to Cargo.lock checksums; 2 git source(s) exempt by a row in ci/repro/vendor-git-sources.txt (md-codec, miniscript).
vendor-freshness: (4/4) re-deriving 2 git source(s) from upstream at the Cargo.lock rev ...
vendor-freshness: (4/4) vendor/md-codec matches https://github.com/bg002h/descriptor-mnemonic at 6cf6599a (433 files).
vendor-freshness: (4/4) vendor/miniscript matches https://github.com/rust-bitcoin/rust-miniscript at ff4732e5 (98 files).
vendor-freshness: (4/4) OK -- md-codec@6cf6599a, miniscript@ff4732e5 re-derived from upstream.
exit 0
```

<!-- vf:green-mnemonic-key -->
```
vendor-freshness: (1/4) resolving Cargo.lock against committed vendor/ (offline, locked, empty CARGO_HOME; git sources: none) ...
vendor-freshness: (1/4) OK -- vendor/ satisfies Cargo.lock.
vendor-freshness: (2/4) OK -- 6642 files across 134 crates match their recorded sha256.
vendor-freshness: (3/4) OK -- 134 crates anchored to Cargo.lock checksums; 0 git source(s) exempt by a row in ci/repro/vendor-git-sources.txt (none).
vendor-freshness: (4/4) OK -- Cargo.lock has no git source; nothing to re-derive.
exit 0
```

<!-- vf:green-mnemonic-secret -->
```
vendor-freshness: (1/4) resolving Cargo.lock against committed vendor/ (offline, locked, empty CARGO_HOME; git sources: mnemonic-io-lib=6c24e62823e6c1ac02aa3862cd6020674bf58544) ...
vendor-freshness: (1/4) OK -- vendor/ satisfies Cargo.lock.
vendor-freshness: (2/4) OK -- 6285 files across 131 crates match their recorded sha256.
vendor-freshness: (3/4) OK -- 130 crates anchored to Cargo.lock checksums; 1 git source(s) exempt by a row in ci/repro/vendor-git-sources.txt (mnemonic-io-lib).
vendor-freshness: (4/4) re-deriving 1 git source(s) from upstream at the Cargo.lock rev ...
vendor-freshness: (4/4) vendor/mnemonic-io-lib matches https://github.com/bg002h/mnemonic-engrave at 6c24e628 (10 files).
vendor-freshness: (4/4) OK -- mnemonic-io-lib@6c24e628 re-derived from upstream.
exit 0
```

<!-- vf:green-descriptor-mnemonic -->
```
vendor-freshness: (1/4) resolving Cargo.lock against committed vendor/ (offline, locked, empty CARGO_HOME; git sources: miniscript=ff4732e5f75aa555682343cb180fa72ee3e8e9d5) ...
vendor-freshness: (1/4) OK -- vendor/ satisfies Cargo.lock.
vendor-freshness: (2/4) OK -- 6616 files across 127 crates match their recorded sha256.
vendor-freshness: (3/4) OK -- 126 crates anchored to Cargo.lock checksums; 1 git source(s) exempt by a row in ci/repro/vendor-git-sources.txt (miniscript).
vendor-freshness: (4/4) re-deriving 1 git source(s) from upstream at the Cargo.lock rev ...
vendor-freshness: (4/4) vendor/miniscript matches https://github.com/rust-bitcoin/rust-miniscript at ff4732e5 (98 files).
vendor-freshness: (4/4) OK -- miniscript@ff4732e5 re-derived from upstream.
exit 0
```

## 7. Removed with the old scripts

These items go, in every repo that has them:

- toolkit's `GROUNDED_GIT_SOURCES` array and its hand re-grounding at each pin
  bump;
- mr's `ground()`, together with its separate name, version and exemption
  lists;
- the key, secret and dm resolution-only bodies;
- each port's "two-block" or "three-block" stanza logic.

The `vendor-freshness.test.sh` text replaces toolkit's 115-line test. mr, key,
secret and dm had no test file.

## 8. The mutation test

`bash ci/repro/vendor-freshness.test.sh` copies the tracked tree into a scratch
directory once per row, plus the working copies of the gate and the data file.
It applies one mutation, runs the gate there, and asserts both the exit code
and a pattern naming the check that fired.

- The scratch parent is `VF_TEST_TMPDIR`, else `TMPDIR`, else `/tmp`.
- `VF_TEST_VERBOSE=1` prints the gate's output under each row.
- The last line is `vendor-freshness.test: <p> passed, <f> failed, <s> skipped`.
- Exit 0 when f = 0.
- It needs rsync, python3, git, cargo, and network access for the G rows.

Subjects are chosen from the tree, so the file runs unchanged in every repo:

- R is the first vendor dir, in sorted order, with a non-null `package` whose
  crate name occurs once in Cargo.lock.
- G is the first row of the data file. With no row, every G row prints
  `skip` and does not count as passed.
- Each subject's edited file is the first `.rs` entry, in sorted order, of its
  manifest.
- OTHER_REV is `0000000000000000000000000000000000000001`.

| row | mutation | exit | pattern (regex on combined output) |
|---|---|---|---|
| A1 | none (control) | 0 | `\(4/4\) OK` |
| A2 | the caller's CARGO_HOME holds a `config.toml` that is not TOML | 0 | `\(4/4\) OK` |
| A3 | `vendor/R` removed | 1 | `vendor/ is out of sync with Cargo.lock` |
| A4 | R's Cargo.lock version set to 0.99.0 | 1 | `vendor/ is out of sync with Cargo.lock` |
| A5 | `// tampered` appended to R's edited file | 1 | `vendor/R/<file>: CONTENT MISMATCH` |
| A6 | a `git+https://github.com/example/elsewhere?rev=OTHER_REV#OTHER_REV` package appended to Cargo.lock | 1 | `has no row in` |
| A7 | the same package with `?branch=main` | 1 | `not pinned by a 40-hex rev` |
| A8 | row `ghost\|https://github.com/example/ghost\|.` appended | 1 | `does not pin from git` |
| A9 | line `not a row` appended | 1 | `malformed row` |
| A10 | data file removed | 1 | `data file ci/repro/vendor-git-sources.txt not found` |
| A11 | the gate run with the argument `extra` | 1 | `takes no arguments` |
| A12 | the ghost row appended twice | 1 | `vendor dir 'ghost' already has a row` |
| A13 | the elsewhere package at OTHER_REV and at `…02` | 1 | `more than one rev` |
| G1 | `vendor/G` removed | 1 | `vendor/ is out of sync with Cargo.lock` |
| G2 | G's Cargo.lock version set to 0.99.0 | 1 | `vendor/ is out of sync with Cargo.lock` |
| G3 | G's edited file tampered | 1 | `vendor/G/<file>: CONTENT MISMATCH` |
| G4 | G's edited file tampered and its manifest digest rewritten to match (F-354) | 1 | `vendor/G: <file>: differs from upstream at <rev8>` |
| G5 | G's rev replaced by OTHER_REV in Cargo.lock and every Cargo.toml outside vendor/ and target/ | 1 | `vendor/G: cannot fetch` |
| G6 | G's row dropped | 1 | `has no row in` |
| G7 | G's field 3 set to `no-such-dir` | 1 | `is not a crate directory in` |
| G8 | an unlisted `src/vf_planted.rs` planted in `vendor/G` | 1 | `file set differs from .cargo-checksum.json: src/vf_planted.rs` |
| G9 | G's version set to `<major>.<minor>.<patch+1000>` in Cargo.lock and in its vendored Cargo.toml, with the manifest digest rewritten | 1 | `vendor/G: Cargo.toml: version <that> is not upstream's` |

Why G9's version takes that form: it stays semver-compatible with any `^`
requirement, so check (1) still resolves and only (4) can see it. Measured: dm
requires `miniscript = "^13.0.0"`, and with 0.99.0 the row redded in (1)
instead.

Each block below is measured, and is the row lines of a
`VF_TEST_VERBOSE=1` run with `exit <code>` appended (gate C4).

<!-- vf:test-mnemonic-toolkit -->
```
subjects: R=adler2 (adler2, benches/bench.rs) G=md-codec (md-codec, examples/dump_encodings.rs, cf35d61a)
ok   - A1 control: the unmodified tree is green (exit 0)
ok   - A2 hermetic CARGO_HOME: the caller's config.toml is never read (exit 0)
ok   - A3 registry crate adler2 removed -> check 1 (exit 1)
ok   - A4 registry crate adler2 stale (lock bumped) -> check 1 (exit 1)
ok   - A5 registry crate adler2 file edited -> check 2 (exit 1)
ok   - A6 git source with no row -> refused (exit 1)
ok   - A7 git source pinned by branch -> refused (exit 1)
ok   - A8 row Cargo.lock does not pin -> refused (exit 1)
ok   - A9 malformed data-file row -> refused (exit 1)
ok   - A10 data file missing -> refused (exit 1)
ok   - A11 an argument -> refused (exit 1)
ok   - A12 two rows for one vendor dir -> refused (exit 1)
ok   - A13 one git url at two revs -> refused (exit 1)
ok   - G1 git source md-codec removed -> check 1 (F-675) (exit 1)
ok   - G2 git source md-codec stale (lock bumped) -> check 1 (exit 1)
ok   - G3 git source md-codec file edited -> check 2 (exit 1)
ok   - G4 md-codec self-consistent wrong-rev byte -> check 4 (F-354) (exit 1)
ok   - G5 md-codec pin moved to an unfetchable rev -> check 4 (exit 1)
ok   - G6 md-codec row dropped -> refused (exit 1)
ok   - G7 md-codec crate dir wrong -> check 4 (exit 1)
ok   - G8 md-codec unlisted file planted -> check 4 (exit 1)
ok   - G9 md-codec another version vendored self-consistently -> check 4 (exit 1)
vendor-freshness.test: 22 passed, 0 failed, 0 skipped
exit 0
```

<!-- vf:test-mnemonic-refugium -->
```
subjects: R=anstream (anstream, examples/dump-stream.rs) G=md-codec (md-codec, examples/dump_encodings.rs, 6cf6599a)
ok   - A1 control: the unmodified tree is green (exit 0)
ok   - A2 hermetic CARGO_HOME: the caller's config.toml is never read (exit 0)
ok   - A3 registry crate anstream removed -> check 1 (exit 1)
ok   - A4 registry crate anstream stale (lock bumped) -> check 1 (exit 1)
ok   - A5 registry crate anstream file edited -> check 2 (exit 1)
ok   - A6 git source with no row -> refused (exit 1)
ok   - A7 git source pinned by branch -> refused (exit 1)
ok   - A8 row Cargo.lock does not pin -> refused (exit 1)
ok   - A9 malformed data-file row -> refused (exit 1)
ok   - A10 data file missing -> refused (exit 1)
ok   - A11 an argument -> refused (exit 1)
ok   - A12 two rows for one vendor dir -> refused (exit 1)
ok   - A13 one git url at two revs -> refused (exit 1)
ok   - G1 git source md-codec removed -> check 1 (F-675) (exit 1)
ok   - G2 git source md-codec stale (lock bumped) -> check 1 (exit 1)
ok   - G3 git source md-codec file edited -> check 2 (exit 1)
ok   - G4 md-codec self-consistent wrong-rev byte -> check 4 (F-354) (exit 1)
ok   - G5 md-codec pin moved to an unfetchable rev -> check 4 (exit 1)
ok   - G6 md-codec row dropped -> refused (exit 1)
ok   - G7 md-codec crate dir wrong -> check 4 (exit 1)
ok   - G8 md-codec unlisted file planted -> check 4 (exit 1)
ok   - G9 md-codec another version vendored self-consistently -> check 4 (exit 1)
vendor-freshness.test: 22 passed, 0 failed, 0 skipped
exit 0
```

<!-- vf:test-mnemonic-key -->
```
subjects: R=aho-corasick (aho-corasick, src/ahocorasick.rs) G=none
ok   - A1 control: the unmodified tree is green (exit 0)
ok   - A2 hermetic CARGO_HOME: the caller's config.toml is never read (exit 0)
ok   - A3 registry crate aho-corasick removed -> check 1 (exit 1)
ok   - A4 registry crate aho-corasick stale (lock bumped) -> check 1 (exit 1)
ok   - A5 registry crate aho-corasick file edited -> check 2 (exit 1)
ok   - A6 git source with no row -> refused (exit 1)
ok   - A7 git source pinned by branch -> refused (exit 1)
ok   - A8 row Cargo.lock does not pin -> refused (exit 1)
ok   - A9 malformed data-file row -> refused (exit 1)
ok   - A10 data file missing -> refused (exit 1)
ok   - A11 an argument -> refused (exit 1)
ok   - A12 two rows for one vendor dir -> refused (exit 1)
ok   - A13 one git url at two revs -> refused (exit 1)
skip - G1 git source <none> removed -> check 1 (F-675) (no row in ci/repro/vendor-git-sources.txt)
skip - G2 git source <none> stale (lock bumped) -> check 1 (no row in ci/repro/vendor-git-sources.txt)
skip - G3 git source <none> file edited -> check 2 (no row in ci/repro/vendor-git-sources.txt)
skip - G4 <none> self-consistent wrong-rev byte -> check 4 (F-354) (no row in ci/repro/vendor-git-sources.txt)
skip - G5 <none> pin moved to an unfetchable rev -> check 4 (no row in ci/repro/vendor-git-sources.txt)
skip - G6 <none> row dropped -> refused (no row in ci/repro/vendor-git-sources.txt)
skip - G7 <none> crate dir wrong -> check 4 (no row in ci/repro/vendor-git-sources.txt)
skip - G8 <none> unlisted file planted -> check 4 (no row in ci/repro/vendor-git-sources.txt)
skip - G9 <none> another version vendored self-consistently -> check 4 (no row in ci/repro/vendor-git-sources.txt)
vendor-freshness.test: 13 passed, 0 failed, 9 skipped
exit 0
```

<!-- vf:test-mnemonic-secret -->
```
subjects: R=aho-corasick (aho-corasick, src/ahocorasick.rs) G=mnemonic-io-lib (mnemonic-io-lib, src/channel.rs, 6c24e628)
ok   - A1 control: the unmodified tree is green (exit 0)
ok   - A2 hermetic CARGO_HOME: the caller's config.toml is never read (exit 0)
ok   - A3 registry crate aho-corasick removed -> check 1 (exit 1)
ok   - A4 registry crate aho-corasick stale (lock bumped) -> check 1 (exit 1)
ok   - A5 registry crate aho-corasick file edited -> check 2 (exit 1)
ok   - A6 git source with no row -> refused (exit 1)
ok   - A7 git source pinned by branch -> refused (exit 1)
ok   - A8 row Cargo.lock does not pin -> refused (exit 1)
ok   - A9 malformed data-file row -> refused (exit 1)
ok   - A10 data file missing -> refused (exit 1)
ok   - A11 an argument -> refused (exit 1)
ok   - A12 two rows for one vendor dir -> refused (exit 1)
ok   - A13 one git url at two revs -> refused (exit 1)
ok   - G1 git source mnemonic-io-lib removed -> check 1 (F-675) (exit 1)
ok   - G2 git source mnemonic-io-lib stale (lock bumped) -> check 1 (exit 1)
ok   - G3 git source mnemonic-io-lib file edited -> check 2 (exit 1)
ok   - G4 mnemonic-io-lib self-consistent wrong-rev byte -> check 4 (F-354) (exit 1)
ok   - G5 mnemonic-io-lib pin moved to an unfetchable rev -> check 4 (exit 1)
ok   - G6 mnemonic-io-lib row dropped -> refused (exit 1)
ok   - G7 mnemonic-io-lib crate dir wrong -> check 4 (exit 1)
ok   - G8 mnemonic-io-lib unlisted file planted -> check 4 (exit 1)
ok   - G9 mnemonic-io-lib another version vendored self-consistently -> check 4 (exit 1)
vendor-freshness.test: 22 passed, 0 failed, 0 skipped
exit 0
```

<!-- vf:test-descriptor-mnemonic -->
```
subjects: R=aho-corasick (aho-corasick, src/ahocorasick.rs) G=miniscript (miniscript, examples/big.rs, ff4732e5)
ok   - A1 control: the unmodified tree is green (exit 0)
ok   - A2 hermetic CARGO_HOME: the caller's config.toml is never read (exit 0)
ok   - A3 registry crate aho-corasick removed -> check 1 (exit 1)
ok   - A4 registry crate aho-corasick stale (lock bumped) -> check 1 (exit 1)
ok   - A5 registry crate aho-corasick file edited -> check 2 (exit 1)
ok   - A6 git source with no row -> refused (exit 1)
ok   - A7 git source pinned by branch -> refused (exit 1)
ok   - A8 row Cargo.lock does not pin -> refused (exit 1)
ok   - A9 malformed data-file row -> refused (exit 1)
ok   - A10 data file missing -> refused (exit 1)
ok   - A11 an argument -> refused (exit 1)
ok   - A12 two rows for one vendor dir -> refused (exit 1)
ok   - A13 one git url at two revs -> refused (exit 1)
ok   - G1 git source miniscript removed -> check 1 (F-675) (exit 1)
ok   - G2 git source miniscript stale (lock bumped) -> check 1 (exit 1)
ok   - G3 git source miniscript file edited -> check 2 (exit 1)
ok   - G4 miniscript self-consistent wrong-rev byte -> check 4 (F-354) (exit 1)
ok   - G5 miniscript pin moved to an unfetchable rev -> check 4 (exit 1)
ok   - G6 miniscript row dropped -> refused (exit 1)
ok   - G7 miniscript crate dir wrong -> check 4 (exit 1)
ok   - G8 miniscript unlisted file planted -> check 4 (exit 1)
ok   - G9 miniscript another version vendored self-consistently -> check 4 (exit 1)
vendor-freshness.test: 22 passed, 0 failed, 0 skipped
exit 0
```

**Exact refusal and red output.** Each block holds a row's lines from the
verbose run that start `::error::`, `  - ` or 6 spaces (gate C5).
mnemonic-toolkit is the reference repo. secret G4 shows the same text in a
repo whose single git source is not a miniscript fork.

<!-- vf:refusal-mnemonic-toolkit-A3 -->
```
::error::vendor-freshness: vendor/ is out of sync with Cargo.lock: the --offline --locked release build cannot resolve a dependency from the committed vendor/ tree. Run 'cargo vendor --locked vendor/' and commit the result. This is the toolkit v0.74.0 release failure class, caught at PR time.
```

<!-- vf:refusal-mnemonic-toolkit-A5 -->
```
::error::vendor-freshness: 1 content defect(s) in vendor/:
  - vendor/adler2/benches/bench.rs: CONTENT MISMATCH
      recorded d67bef1c7f36ed300a8fbcf9d50b9dfdead1fd340bf87a4d47d99a0c1c042c04
      on disk  5bd552f033c6ed8ac18f2ca580dd7989c4b27568335efc7344e035b7d748136e
```

<!-- vf:refusal-mnemonic-toolkit-A6 -->
```
::error::vendor-freshness: Cargo.lock pins git source https://github.com/example/elsewhere?rev=0000000000000000000000000000000000000001, which has no row in ci/repro/vendor-git-sources.txt. A git source has no published tarball digest, so only check (4) can verify its vendored bytes. Add a row (<vendor dir>|https://github.com/example/elsewhere|<crate dir in upstream>) or drop the source.
```

<!-- vf:refusal-mnemonic-toolkit-A7 -->
```
::error::vendor-freshness: Cargo.lock has a git source not pinned by a 40-hex rev: "git+https://github.com/example/elsewhere?branch=main#0000000000000000000000000000000000000001". Pin it with rev = "<40-hex>" or take it from crates.io. Failing closed.
```

<!-- vf:refusal-mnemonic-toolkit-A8 -->
```
::error::vendor-freshness: ci/repro/vendor-git-sources.txt:6: row 'ghost' names https://github.com/example/ghost, which Cargo.lock does not pin from git. Drop the row, so the gate's exemptions match the lock.
```

<!-- vf:refusal-mnemonic-toolkit-A9 -->
```
::error::vendor-freshness: ci/repro/vendor-git-sources.txt:6: malformed row 'not a row'. A row is <vendor dir>|<url as in Cargo.lock>|<crate dir in upstream>: three fields, no spaces, an https:// url, and a crate dir that is '.' or a relative path without '..'.
```

<!-- vf:refusal-mnemonic-toolkit-A10 -->
```
::error::vendor-freshness: data file ci/repro/vendor-git-sources.txt not found. Every repo keeps one; it holds only comments when Cargo.lock has no git source.
```

<!-- vf:refusal-mnemonic-toolkit-A11 -->
```
::error::vendor-freshness: takes no arguments; got: extra
```

<!-- vf:refusal-mnemonic-toolkit-A12 -->
```
::error::vendor-freshness: ci/repro/vendor-git-sources.txt:7: vendor dir 'ghost' already has a row (line 6).
```

<!-- vf:refusal-mnemonic-toolkit-A13 -->
```
::error::vendor-freshness: Cargo.lock pins https://github.com/example/elsewhere at more than one rev (0000000000000000000000000000000000000001, 0000000000000000000000000000000000000002). This gate re-derives one rev per url. Failing closed.
```

<!-- vf:refusal-mnemonic-toolkit-G4 -->
```
::error::vendor-freshness: (4/4) 1 git-source defect(s): a vendored git source is not upstream's tree at the Cargo.lock rev. A tree vendored from another rev passes (1)-(3); only this check sees it (F-354).
  - vendor/md-codec: examples/dump_encodings.rs: differs from upstream at cf35d61a
```

<!-- vf:refusal-mnemonic-toolkit-G5 -->
```
::error::vendor-freshness: (4/4) 1 git-source defect(s): a vendored git source is not upstream's tree at the Cargo.lock rev. A tree vendored from another rev passes (1)-(3); only this check sees it (F-354).
  - vendor/md-codec: cannot fetch https://github.com/bg002h/descriptor-mnemonic at 0000000000000000000000000000000000000001
```

<!-- vf:refusal-mnemonic-toolkit-G7 -->
```
::error::vendor-freshness: (4/4) 1 git-source defect(s): a vendored git source is not upstream's tree at the Cargo.lock rev. A tree vendored from another rev passes (1)-(3); only this check sees it (F-354).
  - vendor/md-codec: no-such-dir is not a crate directory in https://github.com/bg002h/descriptor-mnemonic at cf35d61af0f058029df841f4882945e53f86cd4b
```

<!-- vf:refusal-mnemonic-toolkit-G8 -->
```
::error::vendor-freshness: (4/4) 2 git-source defect(s): a vendored git source is not upstream's tree at the Cargo.lock rev. A tree vendored from another rev passes (1)-(3); only this check sees it (F-354).
  - vendor/md-codec: file set differs from .cargo-checksum.json: src/vf_planted.rs
  - vendor/md-codec: src/vf_planted.rs: not in upstream at cf35d61a
```

<!-- vf:refusal-mnemonic-toolkit-G9 -->
```
::error::vendor-freshness: (4/4) 1 git-source defect(s): a vendored git source is not upstream's tree at the Cargo.lock rev. A tree vendored from another rev passes (1)-(3); only this check sees it (F-354).
  - vendor/md-codec: Cargo.toml: version 0.47.1000 is not upstream's 0.47.0
```

<!-- vf:refusal-mnemonic-secret-G4 -->
```
::error::vendor-freshness: (4/4) 1 git-source defect(s): a vendored git source is not upstream's tree at the Cargo.lock rev. A tree vendored from another rev passes (1)-(3); only this check sees it (F-354).
  - vendor/mnemonic-io-lib: src/channel.rs: differs from upstream at 6c24e628
```

## 9. RED: every port's current proof still fails

The ports recorded these RED proofs in their commits. Each maps to a row of
the shared test, and that row reds under the new gate:

| port | recorded RED proof | row |
|---|---|---|
| toolkit | control | A1 |
| toolkit | md-codec removed | G1 |
| toolkit | lock stale | G2 |
| toolkit | pin moved (the text changes from "pin MOVED" to `cannot fetch`) | G5 |
| toolkit | file edited | G3 |
| toolkit | self-consistent wrong rev (the text changes to `differs from upstream`) | G4 |
| toolkit | ungrounded git source | A6, G6 |
| toolkit | poisoned CARGO_HOME | A2 |
| key | stale vendor | A3, A4 |
| key | vendor/mnemonic-io-lib hidden | A3 (key's lock has no git source now) |
| key | a second unknown git source in the lock | A6 |
| secret | stale vendor | A3, A4, G2 |
| secret | vendor/mnemonic-io-lib moved aside | G1 |
| dm | stale vendor | A3, A4, G2 |
| dm | an uncovered git source | A6 |
| mr | none recorded (no test file) | all rows |

Gate C6 installs each repo's CURRENT script (from `git show <SHA>:ci/repro/vendor-freshness.sh`),
the new test and the data file, and runs the test. In every repo the test
exits 1, and the rows below are among those where the current script gave the
wrong exit (`<row>:<exit it gave>`). They are the defects each port misses
today:

- A5: a hand edit to a registry crate.
- G3 and G4: a hand edit, and the F-354 shape, in a git source.
- G5: a pin moved to an unfetchable rev.
- G8: a planted file. toolkit's own current gate misses this one.
- G9: another version of a git source vendored self-consistently (secret and
  dm).
- A8 to A12: data-file and argument refusals, which the current scripts do not
  have.
- A2: a red under a poisoned CARGO_HOME, where the gate should stay green.

<!-- vf:red-mnemonic-toolkit -->
```
A8:0 A9:0 A10:0 A11:0 A12:0 G6:0 G7:0 G8:0
```

<!-- vf:red-mnemonic-refugium -->
```
A2:1 A5:0 A8:0 A9:0 A10:0 A11:0 A12:0 G6:0 G7:0
```

<!-- vf:red-mnemonic-key -->
```
A2:1 A5:0 A8:0 A9:0 A10:0 A11:0 A12:0
```

<!-- vf:red-mnemonic-secret -->
```
A2:1 A5:0 A8:0 A9:0 A10:0 A11:0 A12:0 G3:0 G4:0 G5:0 G6:0 G7:0 G8:0 G9:0
```

<!-- vf:red-descriptor-mnemonic -->
```
A2:1 A5:0 A8:0 A9:0 A10:0 A11:0 A12:0 G3:0 G4:0 G5:0 G6:0 G7:0 G8:0 G9:0
```

**The test can catch a weakened gate.** Gate C7 installs five mutants of the
proto in mnemonic-toolkit. Each block lists rows the mutant must turn into
FAIL:

- M1 drops the hermetic CARGO_HOME;
- M2 drops check (4);
- M3 drops the file-set comparison;
- M4 drops the byte comparison;
- M5 drops the version comparison.

<!-- vf:mutant-M1 -->
```
A2
```

<!-- vf:mutant-M2 -->
```
G4 G5 G7 G8 G9
```

<!-- vf:mutant-M3 -->
```
G8
```

<!-- vf:mutant-M4 -->
```
G4
```

<!-- vf:mutant-M5 -->
```
G9
```

## 10. Shared-file pinning (STOP S-1, open)

The audit says "pinned in each repo's `ci/shared-ci.sha256` and added to
toolkit `ci/shared-drift.sh` `files` for the public repos (key, secret, dm)".
Measured facts make that design inconsistent as written:

- `ci/shared-ci.sha256` is byte-identical in six public repos: toolkit,
  mnemonic-engrave, mnemonic-transaction, key, secret and dm. Its four lines
  are docs-only.sh, docs-only.test.sh, code-docs-check.sh and
  code-docs-check.test.sh.
- `ci/shared-drift.sh` compares one files list across one repos list:
  ```
  repos=(descriptor-mnemonic mnemonic-engrave mnemonic-key mnemonic-secret mnemonic-transaction)
  files=(ci/docs-only.sh ci/docs-only.test.sh ci/code-docs-check.sh ci/code-docs-check.test.sh ci/shared-ci.sha256)
  ```
  The file `ci/shared-ci.sha256` is in that list. Its test
  (`ci/shared-drift.test.sh`) expects 25 ok lines and
  `shared-drift: PASS (5 repos, 5 files)`.
- engrave and transaction have no `vendor/` and no vendor-freshness gate.

So two hash lines added to `ci/shared-ci.sha256` must then be added in engrave
and transaction too, or that file drifts. Either those repos carry the gate
files (inert), or the vendor files get a pin file of their own. That is a
product choice about what engrave and transaction carry, and the brief does
not decide it.

- **Option A:**
  - All six public repos carry `ci/repro/vendor-freshness.sh` and
    `ci/repro/vendor-freshness.test.sh`. engrave and transaction carry them
    unrun and with no data file.
  - `ci/shared-ci.sha256` gains their two lines in all six.
  - shared-drift's `files` gains the two paths: 35 ok lines,
    `shared-drift: PASS (5 repos, 7 files)`.
  - Two extra PRs (engrave, transaction).
- **Option B (recommended):**
  - A second pin file, `ci/shared-vendor.sha256`, holding the two lines, in
    toolkit, key, secret and dm.
  - It is checked by a step `sha256sum --check --strict ci/shared-vendor.sha256`
    in each `vendor-freshness.yml`, before the gate step.
  - `ci/shared-drift.sh` gains a second group: repos (descriptor-mnemonic,
    mnemonic-key, mnemonic-secret) and files (the two scripts and
    `ci/shared-vendor.sha256`).
  - Its test's counts change by the corresponding amounts. They are measured
    in the toolkit PR.
  - The existing shared set is untouched, and no repo carries a gate it never
    runs.

Under either option, mr (private, absent from shared-drift) appends the two
hash lines to its own `ci/shared-ci.sha256`. That file holds only the two
code-docs lines today and is checked in mr's CI. Section 11's per-repo lists
are written for option B. Under option A, replace "`ci/shared-vendor.sha256`
(new)" with "two lines appended to `ci/shared-ci.sha256`", drop the separate
sha256sum step (rust.yml and release.yml already check that file), and add the
engrave and transaction PRs.

## 11. Per-repo changes

Every PR in this section carries these items. "The three files" means the
section 2 copies: the script, the test and the data file.

1. The three files, copied byte for byte. `sha256sum` of the two shared files
   must equal the `vf:proto-sha256` block.
2. The pin (section 10).
3. Script catalog rows, purpose text exactly:
   - `vendor-freshness.sh`: `Four-check vendor/ gate (shared file): resolution under an empty CARGO_HOME, file integrity, registry provenance, and each git source re-derived from upstream at its Cargo.lock rev (rows in ci/repro/vendor-git-sources.txt).`
   - `vendor-freshness.test.sh`: `Mutation tests for vendor-freshness.sh (shared file; needs rsync, git, cargo and network).`
   - Usage column: `bash ci/repro/vendor-freshness.sh` and
     `bash ci/repro/vendor-freshness.test.sh`.
4. A `design/FOLLOWUPS.md` entry, slug `vendor-freshness-shared`, status
   `open`, with one Companion line per repo PR. The last PR to merge sets it
   to `resolved`.
5. The PR description says it changes `.github/` (allowed in a reviewed PR).

Job names, step names that exist today, and workflow names do not change.
They may be required checks.

**mnemonic-toolkit (PR-T1):**

- The three files. toolkit's current test is replaced.
- `ci/shared-vendor.sha256` (new).
- `.github/workflows/vendor-freshness.yml`:
  - The push and pull_request path lists already hold `ci/repro/**`, so they
    are unchanged.
  - Add the pin step `- name: Shared vendor gate files are unmodified` with
    `run: sha256sum --check --strict ci/shared-vendor.sha256`, before
    `Resolve Cargo.lock against committed vendor/ (offline, locked)`.
  - Replace the comment above `Mutation tests for the gate` (lines 66-70,
    "F-675: prove the gate can fail. Seven cases ...") with:
    ```
          # F-675: prove the gate can fail. The shared test's rows
          # (design/SPEC_vendor_freshness_shared.md section 8) assert the exit
          # status AND which check fired, against a control that must stay
          # green. Runs after the real gate so a red there is reported first.
    ```
- `ci/docs-only-paths.txt`: delete line 20
  (`# Named in ci/repro/vendor-freshness.sh (ci/code-docs-check.sh).`) and
  line 21 (`- ^docs/verify-reproducibility\.md$`). The new script names no
  docs file, and gate C8 shows `ci/code-docs-check.sh` exits 0 with them
  deleted.
- `design/SPEC_vendor_freshness_ci_guard.md`: insert one line after its title:
  `> Superseded for check (4), §3.3 and §5 by design/SPEC_vendor_freshness_shared.md (live re-derivation from upstream; no GROUNDED_GIT_SOURCES).`
- `scripts/README.md` rows 27 and 28: item 3's texts.
- Unchanged: `crates/mnemonic-toolkit/Cargo.toml:85` and
  `crates/mnemonic-toolkit/tests/p3_io_lib_pin.rs:8`. Their "hand-ground"
  words now describe writing the data-file row.

**mnemonic-refugium (PR-R):**

- The three files.
- Two lines appended to `ci/shared-ci.sha256`, in the order script, test.
- `.github/workflows/ci.yml`, job `vendor-freshness` (line 274): after
  `- run: bash ci/repro/vendor-freshness.sh` (line 283), add
  `- run: bash ci/repro/vendor-freshness.test.sh`. The job's name and comment
  are unchanged.
- `scripts/README.md` row 14 and a new row after it: item 3's texts.
- `CLAUDE.md:103` and `README.md:25` stay as they are; they remain true.
- `scripts/plan-*-gate.sh` files that tail the old script's log are historical
  and unchanged.

**mnemonic-key (PR-K), mnemonic-secret (PR-S), descriptor-mnemonic (PR-D),**
each:

- The three files.
- `ci/shared-vendor.sha256` (new).
- `.github/workflows/vendor-freshness.yml`:
  - In both path lists, `'ci/repro/vendor-freshness.sh'` becomes
    `'ci/repro/**'`.
  - Lines 5-6 (`# REDs iff the committed ...` and its continuation) become
    these two lines:
    ```
    # REDs on any check of ci/repro/vendor-freshness.sh (shared file; spec:
    # mnemonic-toolkit design/SPEC_vendor_freshness_shared.md).
    ```
  - Lines 17-19 are deleted: the `#` line and the two-line
    `# CODEC two-block form: ...` paragraph after it. All three repos have the
    same lines there.
  - Add the pin step as in toolkit, before the step
    `Resolve Cargo.lock against committed vendor/ (offline, locked)`.
  - After that step, add:
    ```
          - name: Mutation tests for the gate
            run: bash ci/repro/vendor-freshness.test.sh
    ```
  - The step comment `# \`cargo metadata\` resolution only — no compile, no musl target, no Docker.`
    becomes `# Four checks; check (4) fetches each git source from upstream. No compile, no musl target, no Docker.`
- `ci/docs-only-paths.txt`: delete the comment
  `# Named in ci/repro/vendor-freshness.sh (ci/code-docs-check.sh).` and the
  rule `- ^docs/verify-reproducibility\.md$` that follows it (key line 18,
  secret line 15, dm line 16, each with its comment on the line before).
- Script catalog: key `SCRIPTS.md` row 9, secret `scripts/README.md` row 9,
  dm `scripts/README.md` row 9, each with item 3's text, plus a new row for
  the test after it.
- secret only: in `.github/workflows/man-release.yml`, lines 66-69 become:
  ```
    # Derives the rev from Cargo.lock, as ci/repro/vendor-freshness.sh does, and
    # fails closed on an empty result, so the `repro` job below never carries a
    # hand-written copy of the rev that could drift from the lockfile the pin
    # actually lives in.
  ```
- Unchanged: key `crates/mk-cli/Cargo.toml:55` and dm
  `crates/md-cli/Cargo.toml:46` "hand-ground" comments.

**Coordinator (PR-C, coordinator branch):** `scripts/catalog/<repo>.tsv` rows
for the five repos take item 3's texts. Regenerate with
`python3 scripts/scripts-catalog.py --write`.

## 12. Rollout order and PR list

The shared files are byte-identical from the first merge on. Each repo's gate
is self-contained (it reads only its own tree and upstream), so the order
matters only for shared-drift.

1. **PR-T1, mnemonic-toolkit.** Everything listed for toolkit, without the
   shared-drift change. The daily shared-drift job compares only the existing
   shared set, so it stays green.
2. **PR-R, PR-K, PR-S, PR-D,** in any order, each independently: mr, key,
   secret and dm. Each PR's own CI runs the gate and the test on its own tree.
3. **PR-T2, mnemonic-toolkit,** after PR-K, PR-S and PR-D have merged:
   - shared-drift.sh gains the vendor group (option B) or two files
     (option A);
   - its test gains the new counts and a mutation row for a drifted
     `ci/repro/vendor-freshness.sh` in one repo;
   - the FOLLOWUPS entry is resolved.

   Merged earlier, shared-drift would be red until all three repos landed.
4. **PR-C, coordinator.** Catalog rows, after 1 and 2.

Option A adds PR-E (mnemonic-engrave) and PR-X (mnemonic-transaction) to step
2, and PR-T2 must follow them as well.

PR list: PR-T1, PR-R, PR-K, PR-S, PR-D, PR-T2, PR-C, plus PR-E and PR-X under
option A.

Each PR's build gate is its repo's full CI. Before it is reviewed it also runs:

- `bash ci/repro/vendor-freshness.sh`, whose output equals this spec's
  `vf:green-<repo>` block, unless the repo's Cargo.lock has moved since the
  SHA above. If it has, re-derive the data rows from the new lock and record
  the new output in the PR.
- `bash ci/repro/vendor-freshness.test.sh`, with 0 failed.

## 13. Build gate

```
bash design/spec-vendor-freshness/gate.sh <checkouts> <logs>
```

- `<checkouts>/<repo>` is a clean checkout of each of the five repos at the
  SHA above.
- It needs bash, python3, git, rsync, cargo 1.85.0 (rustup) and network
  access to github.com.
- It prints PASS or FAIL for each check C1-C10, then `gate: PASS` or
  `gate: FAIL (<n>)`.

The run on this spec's head is in the author report
(`design/agent-reports/spec-vendor-freshness-author.md`).

NOT covered by the gate:

- the `.github/` workflow edits;
- the `ci/shared-ci.sha256`, `ci/shared-vendor.sha256` and `ci/shared-drift.sh`
  edits (pinning is OPEN, S-1);
- the catalog and FOLLOWUPS edits;
- a real network outage (G5 stands in);
- mr's self-hosted runner having rsync;
- whether green CI on each PR matches this machine.
