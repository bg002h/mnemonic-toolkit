# CI docs-skip recon (read-only)

Scope: six repos, base refs fetched with `git fetch -q origin` on 2026-10-11.
Refs and SHAs: descriptor-mnemonic `origin/main` 957a0831; mnemonic-toolkit `origin/master` 4d703bb4;
mnemonic-gui `origin/master` fc25b2b; mnemonic-key `origin/main` 340d9da; mnemonic-secret
`origin/master` 8fef227; mnemonic-engrave `origin/master` 21136827.

Method: workflows parsed with PyYAML straight from `git show <ref>:<path>`; job and step line numbers
from the raw blobs. Item 3 was measured by extracting each ref with `git archive` and (a) resolving every
`include_str!` / `include_bytes!` / `include!` target (paren-matched, `concat!(env!("CARGO_MANIFEST_DIR"), ..)`
resolved to the crate root), (b) grepping `.rs`/`.go`/`.sh`/`.py`/`.yml` for runtime reads of `design/`, `docs/`,
`README`, `CHANGELOG`, `../..`. Item 4 used `gh pr list -R bg002h/<repo> --state merged -L 40 --json number,files`.
Citations are `path:line` inside the repo at the stated ref unless prefixed with another repo.

## Headline

1. **mnemonic-engrave already ships this feature.** `release.yml:98-128` has a "docs-only detection" step and
   `docs_only != 'true'` gates on steps 4-16 of the required `test (rust + go)` job. It uses
   `^(design/|[^/]+\.md$)` minus `design/hardware/`. The recon below treats it as prior art; its one defect is
   in Hazard H7.
2. **The toolkit's `rust.yml` cannot get a pure step-level skip yet.** Its `push:` trigger is path-filtered
   (`rust.yml:33-48`), so a docs-only push to `ci/staging` draws no `test`/`clippy` context at all.
   Only the `pull_request:` trigger is unfiltered (`rust.yml:49-56`).
3. **Two repos have a required-context name collision.** mnemonic-toolkit and mnemonic-secret each have a
   second workflow (`release.yml`) whose `test (ubuntu-latest)` job runs on `pull_request` under the same name
   as the required context (H1). Guarding only `rust.yml` leaves a full test run on every PR.
4. **`design/` and `*.md` are not safe allow-lists in every repo.** Tests or required steps read:
   `README.md` (toolkit, gui), `design/SPEC_*.md` (toolkit, key), `design/DESIGN_secret_channels_and_new_forms.md`
   and all of `design/measurements/` (gui), `design/display-grouping-vectors.tsv(.sha256)` (dm, toolkit, key, secret),
   `crates/mk-cli/CHANGELOG.md` (key), and `vendor/**/*.md` (280-382 checksummed files in each vendored repo).
5. **GUI has a self-referential trap.** Its tests parse its own workflows: a step-level `if:` on the
   secret-channels real-binary step fails the T10 OS gate (H6).
6. Docs-only PRs are 35 of 149 across the six repos (23.5%), but most docs-only work reaches these repos by
   direct push to `ci/staging` or the default branch, not by PR (commit-level figures in Item 4).

---

## 1. descriptor-mnemonic (`origin/main`)

### 1.1 Required checks

Workflow `ci.yml`. Triggers `ci.yml:3-24`: `push: branches [main, 'ci/**']`; `pull_request:` with no filters.
Workflow env `RUSTFLAGS: "-D warnings"` (`ci.yml:26-28`).

**`cargo test (ubuntu-latest)`**: job `test`, `ci.yml:31-59`.
- Name expression `ci.yml:32`: `cargo test (${{ matrix.os }})`; `runs-on: ${{ matrix.os }}` (33).
- Matrix `ci.yml:34-37`: `fail-fast: false`, `os: [ubuntu-latest, windows-latest, macos-latest]`. Only the ubuntu leg is required.
- Steps in order:
  1. `actions/checkout@v4` (unnamed)
  2. "Clone patched miniscript fork (md-codec-local-stack)" (`shell: bash`, `git clone ... ../rust-miniscript-fork`, `git checkout b308afd`)
  3. `dtolnay/rust-toolchain@1.85.0` (unnamed)
  4. `Swatinem/rust-cache@v2` (unnamed)
  5. "Install cargo-nextest (pinned, checksum-verified)"
  6. `cargo nextest run --workspace --all-targets --all-features --no-fail-fast` (unnamed, `ci.yml:57`)
  7. `cargo test --workspace --doc --all-features` (unnamed, `ci.yml:59`)

**`cargo clippy`**: job `clippy`, `ci.yml:61-75`; name `ci.yml:62`; not a matrix job; `ubuntu-latest`.
- Steps: 1 checkout@v4; 2 "Clone patched miniscript fork (md-codec-local-stack)"; 3 rust-toolchain@1.85.0 (components clippy);
  4 rust-cache@v2; 5 `cargo clippy --workspace --all-targets --all-features -- -D warnings` (`ci.yml:75`).

### 1.2 Run on `pull_request`, not required

- `ci.yml`: `cargo test (windows-latest)`, `cargo test (macos-latest)` (other legs of job `test`);
  `cargo fmt` (77; its step 2 runs `cd design && sha256sum -c display-grouping-vectors.tsv.sha256`, line 83);
  `cargo doc` (89); `freebsd compile-gate (whole-crate)` (105); `musl compile/test (x86_64-unknown-linux-musl)` and
  `musl compile/test (aarch64-unknown-linux-musl)` (136; matrix `include` at 171-179, runners `ubuntu-latest` / `ubuntu-24.04-arm`).
- `release.yml` (`pull_request:` unfiltered, line 55): `test (ubuntu-latest)`, `test (macos-latest)` (job `test`, 100; matrix `os: [ubuntu-latest, macos-latest]`);
  `build md (<name>)` x5: linux-x86_64, linux-aarch64, macos-x86_64, macos-aarch64, windows-x86_64 (129); `assemble + release` is job-`if`-skipped on PR (246-249).
  Name `test (ubuntu-latest)` differs from required `cargo test (ubuntu-latest)`, so there is no collision here.
- Path-filtered `pull_request` (these never appear on a non-matching PR; they cannot be required):
  `bitcoind-differential.yml:35-42` (job `differential`, 56); `fuzz-smoke.yml:26-30` (job `build` "cargo fuzz build (compile gate)", 40; job `smoke` is schedule/dispatch only, 64-67);
  `vendor-freshness.yml:40-47` (job "vendor/ satisfies Cargo.lock (offline)", 53).
- Not on PR: `man-pages.yml` (tags/dispatch).

### 1.3 Non-.rs files that are compiled or tested (must never count as docs)

- `include_str!`/`include_bytes!`/`include!` to non-.rs targets (measured):
  - `crates/md-cli/tests/fixtures/seating/` (35 files), `.../pathological/` (2), `.../decompose/` (1), `.../n1/` (3). They are included both by tests and by `src/` test modules, e.g. `crates/md-cli/src/seat/satisfy.rs:313-338`, `seat/p0_shapes.rs:54-296`, `seat/partition.rs:382-557`, `seat/input.rs:1080-1320`.
  - `crates/md-codec/tests/vectors/bip341-wallet-test-vectors.json` (`tests/bip341_wallet_vectors.rs:40`) and one more vectors file; `crates/md-codec/tests/golden/pre_refactor_encodings.json` (`internal_key_refactor.rs:26`) and `pre_refactor_ids.json` (`liana_unspendable.rs:310`); `crates/md-codec/tests/fixtures/liana/cases.json` (`crates/md-cli/tests/liana_cases.rs`).
- Runtime reads by tests: `crates/md-codec/tests/vectors/` (323 files, `read_dir` at `skeleton_key_conformance.rs:43-47`, `crates/md-cli/tests/cmd_descriptor.rs:13-17`, `corpus_origin_consistency.rs:22-27`, `vector_corpus.rs:22`); `crates/md-codec/tests/fixtures/coordinator/evidence.jsonl` (`tests/coordinator.rs:17-21`, `md-cli/tests/cli_coordinator_verdict.rs:28-31`, `crates/xtask/src/main.rs:20-25`); `tests/fixtures/network` (`network_version_bytes.rs:42-47`); `crates/md-cli/tests/fixtures` (`cmd_decompose_roundtrip.rs:45-49`, `acceptance_walks.rs:87-91`); `crates/md-cli/tests/golden/compose_pre_unspendable.json` (`cli_compose_unspendable.rs:46-51`); 32 insta `.snap` files under `crates/md-cli/tests/snapshots/`.
- **`design/display-grouping-vectors.tsv`** is read by `crates/md-codec/tests/display_grouping_conformance.rs:30-34` (`"/../../design/display-grouping-vectors.tsv"`). Its `.sha256` is checked by `ci.yml:83` (job `fmt`, not required). `design/` is therefore NOT pure docs here.
- Build/config inputs (non-.rs): `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` (1.85.0), `.cargo/config.toml`, `Cross.toml`, `crates/*/Cargo.toml`, `fuzz/**` (own workspace), `vendor/**` (127 dirs, each with `.cargo-checksum.json` listing its `.md` files), `.gitattributes` (LF rule for `crates/md-codec/tests/vectors/*.json`, needed for SHA-lock tests), `.github/workflows/**`.
- Checked and NOT read: README doc-tests (no `#![doc = include_str!]` in the first-party tree; `readme = "README.md"` in `crates/md-codec/Cargo.toml:9`, `crates/md-cli/Cargo.toml:9` is metadata only); `build.rs` (none; the two `build.rs` paths are modules `crates/md-cli/src/cmd/build.rs`, `crates/md-codec/src/coordinator/build.rs`); `docs/` (comment-only mentions at `crates/md-cli/src/cmd/compose.rs:577`, `format/json.rs:12,18,353`, `tests/cli_compose.rs:227`); `bip/`; root `README.md`/`CHANGELOG.md`/`MIGRATION.md`/`CLAUDE.md`; every other `design/*.md`.

### 1.4 Last merged PRs docs-only

Only 35 merged PRs exist (window 2026-04-29 to 2026-10-11). **7 of 35**: #8, #15, #16, #20, #22, #28, #34.
Areas: design/ only (#8, #15, #16, #28, #34), root `*.md` only (#22), both (#20). No PR hit the 100-file API cap. No near-misses on the item-3 path.
Commit-level (first-parent, last 40 on `origin/main`): 6 docs-only.

---

## 2. mnemonic-toolkit (`origin/master`)

### 2.1 Required checks

| Required context | Workflow : job | Line | Matrix |
|---|---|---|---|
| `test (ubuntu-latest)`, `test (macos-latest)` | `rust.yml` : `test` | job 113, name 114 | `rust.yml:116-118`: `fail-fast: false`, `os: [ubuntu-latest, macos-latest]`; name `test (${{ matrix.os }})` |
| `clippy` | `rust.yml` : `clippy` | 294-301 | none |
| `examples` | `examples.yml` : `examples` | 43 | none |
| `manual` | `manual.yml` : `manual` | 29 | none |
| `quickstart` | `quickstart.yml` : `quickstart` | 29 | none |
| `technical-manual` | `technical-manual.yml` : `technical-manual` | 52 | none |
| `manual-gui` | `manual-gui.yml` : `manual-gui` (aggregate, `if: always()`) | 403-424 | none |

Triggers of `rust.yml` (`rust.yml:13-56`): `push: branches [main, master, 'ci/**']` **with `paths:`** (`crates/**`, `.gitattributes`, `Cargo.toml`, `Cargo.lock`, `.github/workflows/rust.yml`, `scripts/install.sh`, `scripts/install-*.test.sh`, lines 33-48); `pull_request:` with NO paths (49-56, with an explicit comment that a path-filtered required check wedges).

**`rust.yml` `test` steps (`rust.yml:120-207`):**
1 `actions/checkout@v6`; 2 "Verify Cargo.lock is up to date (--locked guard)"; 3 "Show ulimit -l (diagnostic)"; 4 "Install cargo-nextest (pinned, checksum-verified)"; 5 "cargo build --tests"; 6 "cargo nextest run (default tests; G2.1 #[ignore]-gated)"; 7 "cargo test --doc"; 8 "G2.1 eperm fault injection (fresh subprocess; env set before cargo)"; 9 "G2.3-debug einval debug_assert (fresh subprocess)"; 10 "G2.4 off control (fresh subprocess)". There is no toolchain step; `rust-toolchain.toml` drives rustup.

**`rust.yml` `clippy` steps (`rust.yml:297-301`):** 1 `actions/checkout@v6`; 2 "cargo clippy --all-targets -- -D warnings".

**Doc gates.** All five already use "always-run job, guard step, heavy steps `if: steps.guard.outputs.relevant == 'true'`", via `ci/doc-gate-guard.sh`. Triggers: `push: branches [main, master, 'ci/**'] + tags`, `pull_request:` unfiltered, `workflow_dispatch`.
- `examples` (`examples.yml:49-...`, guard line 59 `--book .examples-build`): 1 "Checkout" (fetch-depth 0, line 53); 2 "Detect relevant changes (always runs; ...)"; 3 "Install pandoc + texlive + DejaVu mono"; 4 "Install pinned Rust toolchain (1.85.0)"; 5 "Rust build cache"; 6 "Build mnemonic binary (debug)"; 7 "Regenerate the golden Examples.md"; 8 "Gate - golden must match the freshly regenerated output" (`git diff --exit-code -- .examples-build/Examples.md`, line 111); 9 "Build PDF"; 10 "Upload golden + PDF as workflow artifacts"; 11-12 tag-only release steps.
- `manual` (`manual.yml:35-`; guard line 60 `--book docs/manual`): 1 Checkout (fetch-depth 0, line 39); 2 "Self-test the doc-gate guard (always runs, BEFORE the guard decides)" (`bash ci/doc-gate-guard.test.sh`, line 52); 3 "Detect relevant changes ..."; 4-19 gated: install pandoc+texlive, Node.js, npm tools, lychee, mk-cli, md-cli, ms-cli, build mnemonic, "Audit manual (lint + verify-examples with real mnemonic binary)", "Mutation-test the flag lint (ci/doc-flag-lint.test.sh)", Build/Verify PDF, Build/Verify HTML, two uploads; 20-22 tag-only release steps.
- `quickstart` (`quickstart.yml:35-`; guard line 47 `--book docs/quickstart`): Checkout; Detect relevant changes; then 16 gated steps (install pandoc+texlive, Node, npm tools, lychee, mk-cli, md-cli, ms-cli, build mnemonic, Lint quickstart, "Verify worked-example output fidelity (real binaries)", Build/Verify PDF, Build/Verify HTML, two uploads); 3 tag-only release steps.
- `technical-manual` (`technical-manual.yml:58-`; guard line 73 `--book docs/technical-manual`): Checkout; Detect; gated: install make, Node, npm lint tools, lychee, "Lint technical manual", mk/md/ms-cli, build mnemonic, "Pre-build cargo-example targets (md-codec-examples)", "Verify technical-manual examples (real current-tier bins)", pandoc+texlive, Build/Verify PDF, Build/Verify HTML, two uploads; 3 tag-only release steps.
- `manual-gui` (`manual-gui.yml`): job `changes` (37; checkout fetch-depth 0 line 45; guard line 49 `--book docs/manual-gui`); gated jobs `lint` (67), `verify-examples` (167), `verify-examples-gui` (248), `build` (309), each `if: needs.changes.outputs.relevant == 'true'`; aggregate `manual-gui` (403) with `needs: [changes, lint, verify-examples, verify-examples-gui, build]`, `if: always()`, one step "All gate jobs passed, or were skipped because nothing gated changed" (lines 407-424) that demands `skipped` when `RELEVANT != true`; `release` (441) tag-only.

### 2.2 Run on `pull_request`, not required

- `rust.yml`: `fmt (pinned 1.95.0)` (96); `install.sh harnesses (man-step + MSRV guard)` (209); `test (release, ubuntu-latest, mlock einval)` (256); `miri (mlock unsafe)` (277); `lib cross-platform check` x3 (303; matrix `include` 316-336: x86_64-pc-windows-msvc/windows-latest, aarch64-unknown-linux-gnu/ubuntu-latest, x86_64-unknown-freebsd/ubuntu-latest); `g6 invariant (cross-repo mlock.rs)` (368); `musl build+test (x86_64-unknown-linux-musl)` and `(aarch64-unknown-linux-musl)` (418; matrix 474-486).
- `release.yml` (`pull_request:` unfiltered, line 54): **`test (ubuntu-latest)`** (job `test`, 99-106; `release.yml:104` matrix `os: [ubuntu-latest]`; steps checkout@v4, rust-toolchain@master, install nextest, "cargo nextest run", "cargo test --doc") **same name as the required `rust.yml` context (H1)**; `build mnemonic (macos-x86_64|macos-aarch64|windows-x86_64)` (128); `assemble + release` skipped by job `if` on PR (216).
- `sibling-pin-check.yml` (`pull_request: branches [master]`, no paths): `sibling pins match install.sh` (46).
- Path-filtered `pull_request`: `bitcoind-differential.yml:58-71`; `cross-tool-differential.yml:24-28`; `fuzz-smoke.yml:38-45`; `gui-pin-drift-check.yml:41-44`; `vendor-freshness.yml:36-43` (jobs `vendor-freshness` 49, `repro-script-tests` 70).
- Not on PR: `changelog-check`, `install-pin-check`, `man-pages` (tags); `miniscript-fork-tripwire`, `repro-drift` (schedule/dispatch); `reproducible-musl-build.yml` (`workflow_call` only).

### 2.3 Non-.rs files that are compiled or tested

- Compile-time `include_str!` to non-.rs: `.examples-build/degrade2.desc` (`crates/mnemonic-toolkit/tests/cli_bundle_md1_template_multisig.rs:201`); **`design/SPEC_convert_v0_6.md`** (`crates/mnemonic-toolkit/src/slip0132.rs:382`, inside a `#[test]`; the test slices on `<!-- BEGIN: slip0132-info-line -->` markers, lines 383-392); `src/slip39/slip39_english.txt` (from `src/slip39/wordlist.rs`); `src/wordlists/*.txt` (4 files, from `src/wordlists/mod.rs`); `crates/mnemonic-toolkit/tests/{bip39_trezor_vectors.json, fixtures/*.json, fixtures/bip352/send_and_receive_test_vectors.json, fixtures/descriptor_builder/* (15 files), vectors/*.json}`.
- Runtime reads by tests of repo files outside `crates/`:
  - `README.md` (root, `tests/readme_version_current.rs:27` list `["README.md", "../../README.md"]`; asserts `<!-- toolkit-version: {version} -->`) and `crates/mnemonic-toolkit/README.md` (same line, and `tests/readme_dice_kat_parity.rs:32`, asserts the dice KAT string). Root `*.md` is therefore NOT docs.
  - `design/PLAN_v0_26_0_three_way_merge.md` presence (`tests/design_artifacts_presence.rs:13,18`).
  - `design/display-grouping-vectors.tsv` (`tests/display_grouping_conformance.rs:36`, `tests/cli_p3_grouping_surface.rs:382`).
  - sibling repo `../../../mnemonic-secret/crates/ms-cli/src/mlock.rs` (`tests/mlock_g6_invariant.rs:25`, `#[ignore]`d, run by job `g6-invariant`).
- Build/config inputs: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `Cross.toml`, `Dockerfile.repro`, `.gitattributes` (rust.yml:35-41 documents it as test-outcome-changing), `vendor/**` (169 dirs, 382 `.md` files under it), `ci/repro/**`, `scripts/install.sh` and `scripts/install-*.test.sh` (read by non-required `install-sh-harnesses`, and by `rust.yml:393` in `g6-invariant`), `fuzz/**` and `crates/wc-codec/fuzz/**` (own workspaces).
- Not read by Rust tests: anything under `docs/` (grep clean). `docs/` is consumed by the five doc gates through `ci/doc-gate-guard.sh`. `.examples-build/Examples.md` is the `examples` gate's golden (`examples.yml:111`) and is a `*.md`.
- No `build.rs`, no `go:embed`, no README doc-tests (`crates/mnemonic-toolkit/Cargo.toml:11` `readme =` is metadata).

### 2.4 Last 40 merged PRs docs-only

Window 2026-07-02 to 2026-10-11. **13 of 40**: #66, #70, #72, #73, #74, #75, #79, #82, #83, #85, #96, #97, #98.
- design/ only: #66, #70, #75, #79, #82, #83, #85. design/ + docs/: #72, #73, #74. **docs/ only: #96, #97, #98**; these touch the five doc gates' inputs, so doc gates must still run while code gates may skip.
- Strict `design/`-only: 7 of 40. None hit the 100-file cap; no near-misses.
- Commit-level: the local clone is shallow (`git rev-parse --is-shallow-repository` = true), so only 25 first-parent commits were measurable; 3 docs-only. Treat as indicative.

### 2.5 `ci/doc-gate-guard.sh`

**Inputs.** CLI: one or more `--book <dir>` (required, else exit 64, line 63), optional `--also '<ERE>'` (lines 56-62). Env: `GITHUB_EVENT_NAME`, `GITHUB_REF`, `GITHUB_BASE_REF` (PR), `GITHUB_EVENT_PATH` (to read `before`), `GITHUB_OUTPUT` (defaults to `/dev/stdout`), `DEFAULT_BRANCH` (**defaults to `master`**, line 68). Requires `git` and `python3`.
**Output.** One line `relevant=true|false` appended to `$GITHUB_OUTPUT` via `decide` (lines 71-75), then `exit 0`. Only usage errors exit non-zero (64).
**Event handling (lines 88-120).**
- `pull_request`: base = merge-base of `HEAD` and `origin/$GITHUB_BASE_REF`, fetching that branch first (lines 91-94; helper 79-86 unshallows if the merge-base is missing).
- `push` to a tag (`refs/tags/*`): `relevant=true` (97).
- `push` to the default branch (`refs/heads/$default`, `main`, `master`; line 98): base = the event's `before` read from `$GITHUB_EVENT_PATH` (99-110); empty/zero `before` or unfetchable `before` gives true.
- `push` to any other branch (`ci/staging`, topics; 111-114): base = merge-base with `origin/$default`; failure gives true.
- anything else (dispatch, schedule): `relevant=true` (117-119).
**Computation (lines 122-150).** `ci/doc-gate-paths.py --rev $base --rev HEAD <books>` derives the watched set from both trees (symlink targets, `../` references in non-Markdown files); `git diff --no-renames --name-only $base HEAD`; for each changed file, true if it is under a watched directory (`*/`) or equals a watched file, or matches `--also` (unanchored `grep -qE`). Here-strings are used, never `printf | grep -q`, to avoid the SIGPIPE-under-pipefail false skip (lines 134-136).
**Fail-safe rules (header lines 42-44 plus code).** Err only toward running: tags, dispatch, unfetchable base, no merge-base, python/git error, empty derivation all give `true`. `[skip ci]` is fail-closed by absence of the workflow (lines 46-51).
**Tests.** `ci/doc-gate-guard.test.sh` has 21 `expect` scenarios; it reads each workflow's `bash ci/doc-gate-guard.sh ...` line out of `.github/workflows/<wf>.yml` (line 48) over `WFS=(manual quickstart technical-manual manual-gui)` (line 28); `manual.yml:52` runs it on every run of the `manual` job.

**Reusable for the inverse question ("did anything OUTSIDE an allow-list change?") without modification: no.**
- Its only question is "did anything inside the watched set or `--also` match change?". `--also` is a single ERE tested per changed file with `grep -qE` (lines 146-148). ERE has no negation or lookahead, so "not under design/ and not `*.md`" cannot be expressed. Only a positive deny-list of code paths can be passed.
- `--book` is mandatory and always triggers the python derivation (line 125); an inverse call would need a dummy book.
- `DEFAULT_BRANCH` defaults to `master` (line 68); on a `main`-default repo (dm, key) a `ci/staging` push computes `merge_base_with master`, fails to fetch, and returns `true` (safe, but never skips).
- What can be reused as is: the event-to-base logic (lines 77-120) and the fail-safe conventions. A modification needs either a new mode (e.g. an `--allow '<ERE of docs paths>'` that reports `relevant=true` when ANY changed file does not match) or extraction of lines 54-120 into a sourced library. `ci/doc-gate-guard.test.sh` needs new scenarios in either case.
- The guard needs history: doc workflows check out with `fetch-depth: 0` (e.g. `manual.yml:39`); `rust.yml` uses the default depth 1, so the guard would unshallow (line 84) on every run (toolkit has `vendor/`, 194 MB extracted).

---

## 3. mnemonic-gui (`origin/master`)

### 3.1 Required checks

Triggers: `build.yml:` `push: branches [master, 'ci/**'] + tags mnemonic-gui-v*`, `pull_request: branches [master, 'release/**']` (no paths), `workflow_dispatch`; `schema-mirror.yml`: same push and pull_request branch filters.

| Required context | Workflow : job | Line | Matrix |
|---|---|---|---|
| `snapshots` | `build.yml` : `snapshots` | 103 (name 124) | none |
| `clippy` | `build.yml` : `clippy` | 27 (name 32) | none |
| `headless (no-default-features)` | `build.yml` : `headless` | 48 (name 59) | none |
| `schema-mirror gate` | `schema-mirror.yml` : `schema-mirror` | 27 (name 28) | none |
| `x86_64-unknown-linux-gnu` | `build.yml` : `build` | 282 (name 283) | `build.yml:287-`: `fail-fast: false`; name `${{ matrix.target }}`; 7 `include` legs (x86_64/aarch64 linux-gnu, x86_64/aarch64 linux-musl, x86_64-pc-windows-msvc, x86_64/aarch64-apple-darwin). Required leg: `x86_64-unknown-linux-gnu`, `cross: true` |

Steps (names only):
- `clippy`: 1 `actions/checkout@v5`; 2 install-rust (`rust-toolchain@stable`); 3 cargo-cache (`Swatinem/rust-cache@v2`); 4 `cargo clippy --all-targets -- -D warnings`.
- `headless`: 1 checkout; 2 install-rust; 3 cargo-cache; 4 `cargo build --no-default-features`; 5 `cargo clippy --no-default-features -- -D warnings`.
- `snapshots`: 1 checkout; 2 install-lavapipe; 3 install-rust; 4 cargo-cache; 5 snapshot-suite; 6 census-66 (`build.yml:153-156`, counts `*.new.png`, expects 66); 7 upload-diffs `[if: failure()]`.
- `build` (`build.yml:329-`): 1 checkout; 2 compute-version; 3 install-rust; 4 cargo-cache; 5 install-musl-tools `[matrix.musl_native]`; 6 install-cross `[matrix.cross]`; 7 build-native `[!matrix.cross]`; 8 build-cross `[matrix.cross]`; 9 glibc-floor `[endsWith(matrix.target,'-linux-gnu')]`; 10 package-unix; 11 package-windows; 12 upload-artifact `[tag or dispatch]`.
- `schema-mirror` (`schema-mirror.yml:30-`, 22 steps): checkout-mnemonic-gui; install-rust; cargo-cache; Install cargo-nextest; parse-pinned-upstream; install-mnemonic-toolkit; install-md-cli; install-ms-cli; install-mk-cli; install-pinned-release-binaries; install-shells-and-pyyaml; secret-channels-real-binary-tests (`:136-139`); secret-channels-os-gate (`:142-143`); secret-channels-regen-check (`:147-150`); secret-channels-design-gate (`:153-154`); smoke-gui-schema-mnemonic / -md / -ms / -mk; cargo-test-schema-mirror; cargo-test-full-suite; cargo-test-doc.

### 3.2 Run on `pull_request`, not required

`build.yml`: `msrv (1.88.0)` (78), `tutorial-snapshots` (167; installs a pinned toolkit), the other six `build` legs (aarch64-unknown-linux-gnu, x86_64-unknown-linux-musl, aarch64-unknown-linux-musl, x86_64-pc-windows-msvc, x86_64-apple-darwin, aarch64-apple-darwin); `release` (421) is skipped by job `if` on PR. There are no path-filtered workflows.

### 3.3 Non-.rs files that are compiled or tested

- `include_str!`: `src/form/channels/data.rs:27,30,33` compiles in `design/measurements/secret-channels/channel_table.json`, `channel_policy.json`, `reinterpret.json`. So **`design/measurements/` is compiled code input.**
- Test/CI reads under `design/`: the whole `design/measurements/secret-channels/` (66 files + `fixtures/` 11 + `fixtures/ci/` 18; `tests/secret_channels_common/mod.rs:17`, `tests/secret_channels_t8.rs:20`, `tests/secret_channels_t1.rs:653`); **`design/DESIGN_secret_channels_and_new_forms.md`** (`tests/secret_channels_t8.rs:163-164`, and `check_design_tables.py:11` which regenerates and diffs its generated blocks; run at `schema-mirror.yml:153-154`; `regen_check.py:44` copies it too). It is a `.md` under `design/` that gates.
- `README.md`: `tests/readme_pin_coherence.rs:25,32,52,60` (install-pin lines must equal `pinned-upstream.toml` and `Cargo.toml`). Root `README.md` is a test input.
- `pinned-upstream.toml`: `src/schema_check.rs:62`, `tests/pin_coherence.rs:20-29`, `tests/schema_mirror.rs:459-466`, `tests/secret_channels_t1.rs:648`; also parsed by workflow step `parse-pinned-upstream`.
- **The workflows themselves**: `tests/schema_mirror.rs:166-197` (asserts step names `parse-pinned-upstream`, `install-mnemonic-toolkit`, `install-md-cli`, `install-ms-cli`, `install-mk-cli`, `cargo-test-schema-mirror`, `smoke-gui-schema-*`, and `steps.pins.outputs.*_tag` substrings in `schema-mirror.yml`) and `:261-300` (asserts `name: compute-version`, `shell: bash`, absence of `env.REF_NAME` in `build.yml`); `design/measurements/secret-channels/os_gate.py:3-12,121-135` (run by `tests/secret_channels_t10.rs:32-51`).
- Test data: `tests/snapshots/forms/` (66 PNG), `tests/snapshots/tutorial/` (145), `tests/fixtures/**`, `tests/tutorial/**` (incl. `manifest-stems.txt`, read by `build.yml:259-271`), `tests/ui_harness/`.
- Build/config: `Cargo.toml`, `Cargo.lock`. No `rust-toolchain.toml`, `build.rs`, `vendor/`, `go:embed`.
- Not read: `docs/onboarding/*.md`, `screenshots/*.png`, `FOLLOWUPS.md`, `CHANGELOG.md`, `CLAUDE.md`, `cycle-prep-recon-*.md`, the other 29 `design/*.md` plus `design/agent-reports/`, `scripts/secret-channels-mutations.py` (not in CI). `tests/manual_anchor_coverage.rs:136` reads a sibling toolkit build but is `#[ignore]`d.

### 3.4 Last merged PRs docs-only

37 merged PRs (window 2026-05-13 to 2026-08-09). **0 of 37.** Near-miss: #20 touches only `README.md` (a test input here). No 100-file cap hits.
Commit-level (first-parent, last 40 on `origin/master`): 11 docs-only. So this repo's docs-only work lands by direct push, not PR.

---

## 4. mnemonic-key (`origin/main`)

### 4.1 Required check

**`build (stable on ubuntu-latest)`**: `ci.yml` job `build`, `ci.yml:51-92`; name `ci.yml:52` `build (${{ matrix.toolchain }} on ${{ matrix.os }})`; matrix `ci.yml:54-58`: `fail-fast: false`, `toolchain: [stable, beta, '1.85']` x `os: [ubuntu-latest, macos-latest, windows-latest]` (9 legs; one required).
Triggers `ci.yml:` `push: branches [main, 'ci/**'] + tags mk-cli-v*, mk-codec-v*`; `pull_request:` unfiltered.
Steps (`ci.yml:60-91`): 1 `actions/checkout@v4`; 2 "Install Rust ${{ matrix.toolchain }}" (rust-toolchain@master, components clippy); 3 `Swatinem/rust-cache@v2` (key `${{ matrix.toolchain }}-${{ matrix.os }}`); 4 "Build" (`cargo build --workspace`); 5 "Install cargo-nextest (pinned, checksum-verified)"; 6 "Test" (`cargo nextest run --workspace --no-fail-fast`); 7 "Doc-tests"; 8 "Clippy" (`cargo clippy --workspace --all-targets -- -D warnings`). Clippy is a step here, not a job.

### 4.2 Run on `pull_request`, not required

- `ci.yml`: the other 8 legs of `build`; `fmt (pinned 1.95.0)` (101; step 2 `cd design && sha256sum -c display-grouping-vectors.tsv.sha256`, line 108); `vectors-roundtrip` (119, `needs: build`); `freebsd compile-gate (whole-crate)` (159); `musl compile/test (x86_64-unknown-linux-musl|aarch64-unknown-linux-musl)` (199; matrix 227-); `release-on-tag` (274, job `if` tag only).
- `release.yml` (`pull_request:` unfiltered): `test (ubuntu-latest)`, `test (macos-latest)` (95; matrix 100), `build mk (macos-x86_64|macos-aarch64|windows-x86_64)` (124), `assemble + release` (job-`if` skipped, 208). No name collision with `build (stable on ubuntu-latest)`.
- Path-filtered PR: `fuzz-smoke.yml` (`build`, paths `fuzz/**`, `crates/mk-codec/src/**`); `vendor-freshness.yml`.

### 4.3 Non-.rs files that are compiled or tested

- `include_str!`: `crates/mk-cli/CHANGELOG.md` (`crates/mk-cli/tests/version_is_honest.rs:17`; asserts a `## [x.y.z]` heading for any release version); `crates/mk-codec/src/test_vectors/{csid_ext_v0.1.json, +1}` (`csid_ext.rs`); `v0.1.json` (via `VECTOR_FILE`, `canonical_payload.rs:30`, `error_coverage.rs:137`, `vectors.rs:60`).
- Runtime reads: **`design/SPEC_mk_v0_1.md`** (`crates/mk-cli/tests/spec_cites_resolve.rs:25`; derives valid `§x.y` headings, asserts `§3.3` exists and `§3.5.4` does not); `design/display-grouping-vectors.tsv` (`tests/p3_declines.rs:154`, `src/format.rs:205`); its `.sha256` (`ci.yml:108`, job `fmt`, not required). `vectors-roundtrip` reads `crates/mk-codec/src/test_vectors/v0.1.json` (`ci.yml:149-150`).
- Build/config: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `Cross.toml`, `.gitattributes`, `vendor/**` (134 dirs, 287 `.md` files), `fuzz/**`, `ci/repro/**`.
- Not read: `bip/`, `docs/*.md`, root `README.md`/`CHANGELOG.md`/`CLAUDE.md`, the other `design/*.md`, `crates/*/README.md`, `crates/mk-codec/CHANGELOG.md`.

### 4.4 Last merged PRs docs-only

Only 11 merged PRs exist (window 2026-04-29 to 2026-10-05). **3 of 11**: #3, #7, #10 (#3 design/ + root `*.md`; #7 and #10 design/ only). No near-misses.
Commit-level (last 40 on `origin/main`): 27 docs-only. Docs land overwhelmingly by direct push.

---

## 5. mnemonic-secret (`origin/master`)

### 5.1 Required checks

All in `rust.yml`. Triggers `rust.yml:` `push: branches [main, master, 'ci/**']` (no paths), `pull_request:` unfiltered.

| Required context | Job | Line | Matrix |
|---|---|---|---|
| `test (ubuntu-latest)`, `test (macos-latest)` | `test` | 156 (name 157) | `rust.yml:160-163`: `os: [ubuntu-latest, macos-latest]`, `fail-fast: false` |
| `clippy` | `clippy` | 278-288 | none |
| `test (ms-codec)` | `test-ms-codec` | 109 (name 114) | none |
| `clippy (ms-codec)` | `clippy-ms-codec` | 147-154 | none |

Steps:
- `test` (`rust.yml:163-`): 1 `actions/checkout@v4`; 2 "Show ulimit -l (diagnostic)"; 3 "Install cargo-nextest (pinned, checksum-verified)"; 4 "cargo build --tests"; 5 "cargo nextest run (default tests; G2.1 ...)"; 6 "G2.1 eperm fault injection ..."; 7 "G2.3-debug einval debug_assert ..."; 8 "G2.4 off control ...".
- `clippy` (`:281-288`): 1 checkout@v4; 2 **"conformance-vector checksum pin"** (`cd design && sha256sum -c display-grouping-vectors.tsv.sha256`, line 285); 3 "cargo clippy --all-targets -- -D warnings" (`cargo clippy --all-targets -p ms-cli`, line 288).
- `test-ms-codec` (`:116-145`): 1 checkout@v4; 2 "hashlock reproduction preflight (a missing tool fails HERE, never a test that could be skipped)"; 3 "Install cargo-nextest"; 4 "cargo nextest run -p ms-codec"; 5 "cargo test -p ms-codec --doc"; 6 "hashlock reproduction test RAN (by name)" (`:138-145`, greps `/tmp/repro.log` for `test result: ok. 1 passed`).
- `clippy-ms-codec`: 1 checkout@v4; 2 "cargo clippy -p ms-codec --all-targets -- -D warnings".

### 5.2 Run on `pull_request`, not required

- `rust.yml`: `fmt (pinned 1.95.0)` (92); `test (release, ubuntu-latest, mlock einval)` (238); `miri (mlock unsafe)` (260); `history purge (recipes RUN under real shells)` (290); `g6 invariant (cross-repo mlock.rs)` (336); `freebsd compile-gate (whole-crate)` (367); `musl compile/test (x86_64-unknown-linux-musl|aarch64-unknown-linux-musl)` (400; matrix 440-).
- `release.yml` (`pull_request:` unfiltered): **`test (ubuntu-latest)`** (job `test`, 103-104; matrix `os: [ubuntu-latest]` at 108) **same name as the required `rust.yml` context (H1)**; `build ms (macos-x86_64|macos-aarch64|windows-x86_64)` (132); `assemble + release` job-`if` skipped (228).
- Path-filtered PR: `fuzz-smoke.yml` (`build`; paths `fuzz/**`, `crates/ms-codec/src/**`); `vendor-freshness.yml`.
- Not on PR: `man-release.yml` (tags/dispatch).

### 5.3 Non-.rs files that are compiled or tested

- `include_str!`: `crates/ms-cli/vectors/v0.1.json` (`crates/ms-cli/src/cmd/vectors.rs:11`, and 3 files in that dir); `crates/ms-codec/tests/vectors/{hashlock-v0.8.json, v0.1.json}` (`hashlock_kat.rs:16`, `hashlock_qr_text.rs:31`, `hashlock_derivation.rs:93,179`, `crates/ms-cli/tests/vectors_parity.rs:11`).
- **`design/display-grouping-vectors.tsv`** read by `crates/ms-cli/src/format.rs:358-360` (test module) and `crates/ms-cli/tests/whitespace_only_separator.rs:119-137` (runs `sha256sum -c` against the `.sha256`). The required `clippy` job also checks it (`rust.yml:285`).
- Build/config: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `Cross.toml`, `.gitattributes`, `vendor/**` (131 dirs, 285 `.md`), `fuzz/**`, `ci/repro/**`, `scripts/` (`plan-build-gate-ms.sh`, `plan-handwire-ms-hashlock.py`, `push-via-staging.sh`; none referenced by a required job).
- Not read: `docs/verify-reproducibility.md`, `design/*.md` and `design/agent-*`, root `README.md`/`CHANGELOG.md`/`MIGRATION.md`/`CLAUDE.md`, `cycle-prep-recon-*.md`, `crates/*/README.md`, `crates/ms-codec/CHANGELOG.md`.

### 5.4 Last merged PRs docs-only

Only 9 merged PRs exist (window 2026-05-08 to 2026-05-13). **4 of 9**: #1, #3, #4, #10 (#1 design/ + root `*.md`; #3 root `*.md`; #4, #10 design/). No near-misses.
Commit-level (last 40 on `origin/master`): 2 docs-only.

---

## 6. mnemonic-engrave (`origin/master`)

Only workflow: `.github/workflows/release.yml`. Triggers `release.yml:` `push: tags v*, branches [master, 'ci/**']`; `pull_request:` unfiltered; `workflow_dispatch`.

### 6.1 Required checks

**`test (rust + go)`**: job `test`, `release.yml:67-258`; name line 68; not a matrix job; `permissions: contents: read`; `env: ME_REQUIRE_GO: '1'`.
Steps (docs-only gating is already present):
1. "Checkout (with submodules)" (`actions/checkout@v4`, `submodules: true`)
2. "signing key copies agree with minisign.pub" (`sh scripts/check-minisign-pin.sh`, line 96; ALWAYS runs; the script checks `.github/workflows/release.yml` x2, `README.md` x2, `scripts/release-workflows/gen.py` x1, `check-minisign-pin.sh:49-52`)
3. "docs-only detection" (`id: detect`, lines 98-128)
4-16, each `if: steps.detect.outputs.docs_only != 'true'`: Install Rust toolchain; fmt; clippy; clippy (library, no default features); Set up Go; Install the shells the history-purge gates execute; Install cargo-nextest; "Rust test suite (locked; Go on PATH + ME_REQUIRE_GO=1 -> oracles run)"; Rust doc-tests; "Go tests (me-preview sidecar)"; "Go build + tests (ndef-roundtrip oracle)"; "refugium-otp e2e (fake picotool)"; "shellcheck (OTP scripts)"
17. "SH2 demo install links resolve and are current" (line 246; deliberately NOT gated, network, `./demo/sh2/check-install-links.sh`)
18. "docs-only verdict" (`if: docs_only == 'true'`, lines 256-258)

**Existing rule (`release.yml:98-128`).** BEFORE = `github.event.before`, overridden for `pull_request` by `github.event.pull_request.base.sha` (107). FULL (docs_only=false) on: empty/zero BEFORE ("new ref"), BEFORE unfetchable (`git fetch --no-tags --depth=1 origin $BEFORE`), diff failure, empty diff, any path matching `^design/hardware/`, or any path not matching `^(design/|[^/]+\.md$)` (here-string `grep -qvE`, per F-695). Output polarity is fail-closed (`!= 'true'`).

**`picotool argv probe`**: job `picotool-probe`, `release.yml:272-309`; name line 273; not a matrix job; `permissions: contents: read`. Steps: 1 Checkout (no submodules); 2 "Install Nix (pinned by commit SHA)" (`cachix/install-nix-action`); 3 "Build the pinned picotool (patched) and the stock control" (`nix build .#picotool`, `.#picotool-unpatched`); 4 "Probe every builder argv shape against the real binary" (`bash scripts/test/picotool-argv-probe.sh`); 5 "seal --sign --clear works and verifies (patched)"; 6 "seal --sign --clear fails 248 on stock 2.3.1 (negative control)". **It has no docs-only gating**: every docs PR builds Nix and both picotools. Inputs: `flake.nix`, `flake.lock`, `nix/patches/**`, `scripts/test/**`, `scripts/lib/**`.

### 6.2 Run on `pull_request`, not required

`go-build` "build me-preview (all targets)" (311); `rust-build` "build me (linux-x86_64|linux-aarch64|macos-aarch64|macos-x86_64|windows-x86_64)" (381; matrix 386-); `assemble + sign + release` (463; job `if` tag/dispatch, skipped on PR; `needs: [test, go-build, rust-build, picotool-probe]`).

### 6.3 Non-.rs/.go files that are compiled or tested

- Include macros (all inside `crates/me-cli/`): `tests/vectors/bundle-md1-mk1.json` (`tests/cli.rs:374,944`), `tests/vectors/*.ndef` x4 (`tests/golden.rs:19-40`, plus md1/mk1 variants), `testdata/show_public_records_pre_s2.txt` (`tests/sysw_cli.rs:2654`), `testdata/seal_vectors.json` (`src/seal/mod.rs:552`), `testdata/sysw_vectors.json` (`src/sysw/vectors.rs:26`), `testdata/{descriptor_seam_vectors,codex32_seam_vectors,record_class_vectors}.json` (read via `std::fs` in `tests/descriptor_seam.rs:48`, `descriptor_as.rs:15`, `descriptor_refusals.rs:43`, `expect_kinds.rs:281`, `record_corpus.rs:34`, `codex32_seam.rs:28`, `sysw_composer_records.rs:427`). `tests/descriptor_refusals.rs:41` includes itself.
- Cross-language: `crates/me-cli/tests/cross_lang.rs:21` runs `../../firmware/ndef-roundtrip` (Go); `tests/preview_cross_lang.rs:70` runs `../../preview` (Go). Go modules: `preview/go.mod` (`replace seedhammer.com => ../third_party/seedhammer`, a **git submodule gitlink**, `.gitmodules`), `firmware/ndef-roundtrip/go.mod`. No `go:embed` anywhere.
- **`design/hardware/`** (`retail-otp.json` + `captures/`): read by `scripts/refugium-otp.sh:169`; `scripts/test/run-e2e-otp.sh:139-145,176,312-318,560-561,669-678`. Already excluded by the existing rule.
- `README.md`: checked by the always-running `scripts/check-minisign-pin.sh:51` (needs the pin at least twice). No other test reads it.
- Build/config: `Cargo.toml` (workspace `exclude` lists `design/measurements/mt-size-probe` and `harnesses/liana`, which are Cargo crates living under `design/` and `harnesses/` but NOT built by the workspace), `Cargo.lock`, `flake.nix`, `flake.lock`, `nix/patches/**`, `.gitmodules` + gitlink `third_party/seedhammer`, `scripts/test/fixtures/**` (e.g. `seal/blinky.uf2`), `preview/go.sum`.
- Not read by any required step: all other `design/**` (1955 `.md` files plus evidence, journeys, measurements, vectors and `.py`/`.html`/`.json` under it), root `*.md` (`CLAUDE.md`, `DecisionsMadeByClaude.md`, `IMPLEMENTATION_LOG_P*.md`, `cycle-prep-recon-*.md`), `.claude/**`. `scripts/test/shot-server-security-test.py:51` reads `design/journeys/shot_server.py` but no workflow runs it.

### 6.4 Last merged PRs docs-only

17 merged PRs (window 2026-07-10 to 2026-10-07). **8 of 17**: #7, #8, #9, #10, #12, #13, #16, #17 (all design/ only; none of them touched `design/hardware/`). No near-misses.
Commit-level (last 40 on `origin/master`): 31 docs-only. Docs land overwhelmingly by direct push to `ci/staging` then `master`.

---

## 7. PR docs-only tally

| Repo | Merged PRs available | Docs-only | PR numbers |
|---|---|---|---|
| descriptor-mnemonic | 35 | 7 | 8, 15, 16, 20, 22, 28, 34 |
| mnemonic-toolkit | 40 | 13 | 66, 70, 72, 73, 74, 75, 79, 82, 83, 85, 96, 97, 98 |
| mnemonic-gui | 37 | 0 | none (#20 README-only is a near-miss) |
| mnemonic-key | 11 | 3 | 3, 7, 10 |
| mnemonic-secret | 9 | 4 | 1, 3, 4, 10 |
| mnemonic-engrave | 17 | 8 | 7, 8, 9, 10, 12, 13, 16, 17 |
| **Total** | **149** | **35 (23.5%)** | |

Definition used: every changed file is under `design/` or ends in `.md`, minus these item-3 paths:
dm `design/display-grouping-vectors.tsv*`; toolkit `README.md`, `crates/mnemonic-toolkit/README.md`, `design/SPEC_convert_v0_6.md`, `design/PLAN_v0_26_0_three_way_merge.md`, `design/display-grouping-vectors.tsv`;
gui `README.md`, `design/DESIGN_secret_channels_and_new_forms.md`, `design/measurements/**`;
key `crates/mk-cli/CHANGELOG.md`, `design/SPEC_mk_v0_1.md`, `design/display-grouping-vectors.tsv*`;
secret `design/display-grouping-vectors.tsv*`; engrave `design/hardware/**`.
Caveats: `gh` caps `files` at 100 per PR (no docs-only candidate hit it; the toolkit and gui windows contained PRs with exactly 100 files that were not docs-only); where fewer than 40 PRs are listed, that is every merged PR the repo has. The toolkit docs-only list includes #96-#98 which touch only `docs/**/*.md`, the doc-gate inputs. Commit-level supplement (first-parent diff vs first parent, last 40 on the default branch): dm 6, toolkit 3 of 25 (shallow clone), gui 11, key 27, secret 2, engrave 31.

---

## 8. Hazards (anything that makes a docs-skip unsafe or ineffective)

**H1. Duplicate required-context names (toolkit, secret).** `release.yml` job `test` runs on `pull_request` and is named `test (${{ matrix.os }})` with `os: [ubuntu-latest]`, i.e. `test (ubuntu-latest)`. toolkit: `release.yml:99-104`; secret: `release.yml:103-108`. This is the same string as the required context from `rust.yml` (toolkit 113-118, secret 156-163). Guarding only `rust.yml` leaves a full nextest run on every docs PR under the same context name, and which check run satisfies the rule is ambiguous. `scripts/push-via-staging.sh` resolves duplicates by the highest check-run id per required context (comment at lines ~134-136). Both jobs need the same guard, or `release.yml`'s `test` must be renamed or made non-PR. dm and key do not collide (`cargo test (...)` / `build (... on ...)`).

**H2. toolkit `rust.yml` push trigger is path-filtered (`rust.yml:33-48`).** A docs-only commit pushed to `ci/staging` runs no `rust.yml` job, so `test (...)` and `clippy` never appear and `push-via-staging.sh` stops ("required context(s) NEVER RAN", per `ci/doc-gate-guard.sh:11-13`). The comment at `rust.yml:50-56` records the PR side as solved and the push side as "covered by admin bypass". Skipping steps for docs-only commits requires removing that filter. The filter also omits `rust-toolchain.toml`, `Cross.toml`, `vendor/**`, `ci/repro/**`, `Dockerfile.repro`, which affect builds.

**H3. Output polarity.** The toolkit doc gates use `if: steps.guard.outputs.relevant == 'true'` (fail-open: an empty output skips the work). A code-skip must use the engrave polarity (`docs_only != 'true'`, fail-closed). Every guard error path must write `false` only on a positive docs-only proof; a crash that exits 0 without writing must run the tests.

**H4. Allow-list membership.** Neither `design/**` nor `*.md` is docs in all repos: toolkit `README.md` x2, `design/SPEC_convert_v0_6.md`, `design/PLAN_v0_26_0_three_way_merge.md`; gui `README.md`, `design/DESIGN_secret_channels_and_new_forms.md`, `design/measurements/**` (including `.md` files such as `table.md`, `bytes.md`, `t3.md` consumed by `check_design_tables.py`); key `design/SPEC_mk_v0_1.md`, `crates/mk-cli/CHANGELOG.md`; the four-repo `design/display-grouping-vectors.tsv` (+ `.sha256`) in dm/toolkit/key/secret (it is not `.md` but is under `design/`); engrave `design/hardware/**`. Each repo needs its own protected list; a shared generic rule would be wrong in at least five of six.

**H5. `vendor/**/*.md` is not docs.** 280, 382, 287 and 285 `.md` files live under `vendor/` in dm, toolkit, key and secret, listed with hashes in each crate's `.cargo-checksum.json`. A `*.md` rule that is not anchored away from `vendor/` would skip the vendor-integrity and offline-build checks. Also keep `.gitattributes`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/**`, `Cross.toml`, `Dockerfile.repro`, `flake.nix`, `flake.lock`, `nix/**`, `.gitmodules` and submodule gitlinks on the "affects build" side.

**H6. GUI tests inspect the GUI's own workflows (`mnemonic-gui`).**
(a) `design/measurements/secret-channels/os_gate.py:121-135` ignores any step carrying `if:` or `continue-on-error` and requires a job without a job-level `if` and without a `needs:` chain through an `if:` job. `channel_policy.json` has `private_channels_on: ["linux"]`, so `tests/secret_channels_t10.rs:32-51` requires an unconditional step running `cargo test|nextest --test secret_channels_t2 --test secret_channels_t3prime --test secret_channels_t7` with `MNEMONIC_BIN` (today `schema-mirror.yml:136-139`). Adding a step-level `if:` to that step fails T10 (it runs in `cargo-test-full-suite`, `schema-mirror.yml:192-198`).
(b) `tests/schema_mirror.rs:166-197,261-300` assert step-name substrings in both workflows; renaming or restructuring steps breaks them.
(c) `build.yml:153-156` (`census-66`, expects 66 `*.new.png`) fails if `snapshot-suite` is skipped without it.
So the GUI cannot use a plain `if:` on its test steps without also changing the T10 gate or placing the skip inside a step's script.

**H7. Engrave's existing detector bases pushes on `github.event.before` (`release.yml:101,109-115`).** For a push to an existing `ci/staging` whose previous tip is an unchecked commit, a docs-only commit on top computes docs-only and skips tests, although the tree is not identical to a green parent. The toolkit guard documents this exact case (I-3) and uses the merge-base with the default branch for non-default pushes (`ci/doc-gate-guard.sh:30-38,111-114`). Also: the `picotool argv probe` job has no gating; `README.md` and the minisign pin check stay unconditional by design; do not duplicate the existing step when planning engrave.

**H8. Base-ref computation.** For `push` the base must be (default branch: previous tip; any other branch: merge-base with the default branch), not `before`. `DEFAULT_BRANCH` defaults to `master` in the toolkit guard (line 68): dm and key use `main`, so `ci/staging` pushes would return `true` (safe, no savings) unless `DEFAULT_BRANCH=main` is exported. Tags, `workflow_dispatch`, and schedule must return full run. Docs-only commits arrive mostly by direct push to `ci/**` (key 27/40, engrave 31/40, gui 11/40), so the push path carries most of the value, and PR-only testing would miss it.

**H9. History depth and cost.** `rust.yml`, `ci.yml` and the other code jobs use `actions/checkout` with the default depth 1. A guard needs the base commit; the toolkit guard unshallows when the merge-base is missing (`ci/doc-gate-guard.sh:82-85`). Doc workflows already use `fetch-depth: 0`. Using `--depth=1` fetch of `before` as engrave does fails the merge-base requirement for non-default branches.

**H10. Matrix and aggregate semantics.** Every matrix leg is its own job, so the guard step is needed in each required leg (toolkit/secret `test` x2, dm `test` x3, key `build` x9, gui `build` x7); the non-required legs still run in full unless also guarded. Do not use job-level `if:` on a required matrix job: a skipped dynamically named matrix job reports an unexpanded name and wedges. The toolkit `manual-gui` aggregate (`manual-gui.yml:403-424`) shows the pattern that tolerates skips, and its `RELEVANT != true` branch demands `skipped`.

**H11. Steps coupled to skipped steps.** secret `test-ms-codec` step 6 greps `/tmp/repro.log` produced by its `cargo test` (`rust.yml:138-145`); the preflight step 2 (`rust.yml:118-123`) is meant to be unconditional ("a missing tool fails HERE"). gui `census-66` (above). toolkit `test` step 2 (`cargo metadata --locked`). Anything that consumes an earlier skipped step's output must share its `if`.

**H12. Doc gates must keep running.** In the toolkit, docs are consumed by five required doc gates that call the same guard family; a code guard must not hide a docs change from them (they read `docs/**`, `.examples-build/**`, `crates/**`, `Cargo.*`; `.examples-build/Examples.md` is a `*.md` golden at `examples.yml:111`, and `.examples-build/degrade2.desc` is compiled into a Rust test, `cli_bundle_md1_template_multisig.rs:201`). Conversely a docs-only PR (e.g. toolkit #96-#98, `docs/` only) should skip code jobs while the doc gates run.

**H13. Self-reference.** `.github/workflows/**`, `ci/**`, the guard script and its test must be classified as code. Workflows are test inputs in gui (H6) and are copied into the toolkit guard's own `--also` lists (`manual.yml:60`, etc.).

**H14. `[skip ci]`, new refs, zero SHAs.** A `[skip ci]` commit runs no workflow, so no required context appears (fail-closed, per `ci/doc-gate-guard.sh:46-51`). New refs and zero `before` SHAs must run the full suite (the toolkit guard and engrave detector both do).

**H15. Cross-repo coupling not covered by path rules.** toolkit `g6-invariant` (`rust.yml:368-417`) and secret `g6-invariant` (`rust.yml:336-`) compare `mlock.rs` across repos at a pin read from `scripts/install.sh`; gui installs pinned sibling CLIs (`schema-mirror.yml`); toolkit `sibling-pin-check.yml` and `gui-pin-drift-check.yml` read `scripts/install.sh` and `docs/manual-gui`. A docs-only change in one repo cannot change these inputs in another, but a docs-only classification must not extend to `scripts/install.sh` or `pinned-upstream.toml`.

**H16. Platform detail.** The matrix includes `windows-latest` (dm `test`, key `build`) and `macos-latest` legs; a bash guard step on Windows legs needs `shell: bash` and `python3`. The required legs are ubuntu (all repos) plus macOS (toolkit, secret), where `python3` is present.
