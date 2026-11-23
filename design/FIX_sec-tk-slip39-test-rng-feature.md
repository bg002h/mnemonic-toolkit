# FIX-sec-tk-slip39-test-rng-feature: the slip39 test RNG leaves the shipped binary

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. Track: full (new cargo feature, a `.github/` change, a new
refusal text). **Class B (funds path): SLIP-39 shares of a master secret.** Secret row:
`tk-slip39-test-rng-env-in-production` (I), coordinator `status/secret-criticals.md`. Ruling: TK-Q4, coordinator
`agent-reports/architect-sweep-1122-questions.md:89-95`. Base: origin/master b2a2202b. All anchors below are at b2a2202b.

## 1. Defect

`mnemonic slip39 split` reads `MNEMONIC_SLIP39_TEST_RNG` and `MNEMONIC_SLIP39_TEST_IDENTIFIER` in the shipped binary
(`crates/mnemonic-toolkit/src/cmd/slip39.rs:429-430`). A 32-byte hex value seeds `ChaCha20Rng` in place of `OsRng`
(:488-512), and the identifier value replaces the random identifier (:470-484). The only guard is a stderr warning
(:431-433, text at :291). Anyone who knows or plants the value (a shell profile, a CI secret, a wrapper script) can
recompute every share of a split from the master secret's public shape, and two splits of the same secret with the same
value are identical (RED R1). A non-UTF-8 value is silently ignored (`std::env::var(..).ok()` is `None`), so that case
gets no warning at all (RED R4).

## 2. Ruling (TK-Q4, not re-ruled here)

- Cargo feature `slip39-test-rng`, off by default, in `crates/mnemonic-toolkit/Cargo.toml`.
- The two test files that need the variables, `tests/cli_slip39_advisories.rs` and `tests/cli_slip39_json.rs`, become
  `[[test]]` targets with `required-features = ["slip39-test-rng"]`.
- Every CI nextest job adds `--features slip39-test-rng` (a `.github/` change, named in the PR description).
- Without the feature, a set `MNEMONIC_SLIP39_TEST_RNG` or `MNEMONIC_SLIP39_TEST_IDENTIFIER` is refused: exit non-zero,
  before any input is read.
- One `#[cfg(not(feature = "slip39-test-rng"))]` row pins the refusal.
- SPEC §6 and the manual (`41-mnemonic.md`, the advisory row at :2594) change in lockstep.

## 3. Holder-visible behaviour (decided)

### 3.1 Default build (feature off; every shipped and `cargo install`ed binary)

Refusal. Exit 1 (`ToolkitError::BadInput`), stdout empty, no `--json-out` file created, stderr exactly one line:

    error: slip39 split: MNEMONIC_SLIP39_TEST_RNG or MNEMONIC_SLIP39_TEST_IDENTIFIER is set, but this build has no test RNG; unset both to split with the system random source

PROPOSED (architect confirms): the stem text after `error: `. The `error: ` prefix is the existing `ToolkitError`
Display (`src/error.rs:1050`).

Edge cases, all decided:

1. "Set" means `std::env::var_os(name).is_some()`: any value, including the empty string and a value that is not
   UTF-8. The values are never parsed or echoed in this build.
2. Either variable alone triggers the refusal; both set triggers it once (one line, the same text).
3. Order: the check is the first statement of the `Slip39Command::Split` arm of `run` (`cmd/slip39.rs:215`, before
   `let from_dash` at :220). It therefore precedes the `@env:` source resolution (:223-228), the row-17 `--from` node
   check (:374), the row-18 stdin-consumer check (:383-391), the 1-of-1 group check (:398-404), every argv advisory
   (:407-425) and every stdin read (:437, :444). Clap parse errors and `--help` still come first (they happen before
   `run`), and the variables do not affect them.
4. `slip39 combine` never read the variables and is unchanged: with them set it behaves exactly as now (control C1).
5. No other subcommand reads the variables (`grep -rn SLIP39_TEST crates/` at b2a2202b: `cmd/slip39.rs` and the two
   test files only); nothing else changes.
6. With both variables unset the default build behaves exactly as now (`OsRng`, library-chosen identifier, no
   warning; control C2).

### 3.2 Test build (feature `slip39-test-rng` on)

Unchanged byte for byte from b2a2202b: the variables are read with `std::env::var(..).ok()` at the same point
(after the argv advisories, before the stdin read), the warning at :291 fires when either is set, the value checks at
:471-479 and :489-499 keep their texts and exit 1, and a non-UTF-8 value is still ignored there. The refusal of 3.1
does not exist in this build. The RED script run against a feature-on binary prints the b2a2202b output of §5
line for line.

## 4. Code delta (production)

`crates/mnemonic-toolkit/Cargo.toml`:

- Add a `[features]` table after `[dependencies]` (before `[dev-dependencies]`), with one entry:
  `slip39-test-rng = []`, preceded by a comment: `# Test-only: lets slip39 split read MNEMONIC_SLIP39_TEST_RNG /
  _IDENTIFIER for deterministic shares. Off in every shipped build; there a set variable is refused.`
- Add two `[[test]]` targets after `[dev-dependencies]`: `name = "cli_slip39_advisories"`,
  `path = "tests/cli_slip39_advisories.rs"`, `required-features = ["slip39-test-rng"]`; and the same three keys for
  `cli_slip39_json` / `tests/cli_slip39_json.rs`. No dependency changes; `Cargo.lock` is unchanged (`--locked` holds).
  `rand_chacha` stays a normal dependency (the library's own unit tests use it, `src/slip39/mod.rs:491`).

`crates/mnemonic-toolkit/src/cmd/slip39.rs`:

- :54-55 (`ENV_TEST_RNG`, `ENV_TEST_IDENTIFIER`): unconditional (both builds use them).
- :50-51 (`use rand_chacha::ChaCha20Rng;`, `use rand_core::SeedableRng;`), :288-293 (`emit_env_var_advisory`),
  :427-433 (the env read and warning), :469-484 (identifier parse) and the `ChaCha20Rng` branch of :486-526: compiled
  only with `#[cfg(feature = "slip39-test-rng")]`. Without the feature, `identifier` is `None` and the split call is the
  existing `OsRng` branch (:514-524). No `unused` warning in either build (`clippy -D warnings` clean on both sides,
  §8).
- New private fn, `#[cfg(not(feature = "slip39-test-rng"))]`, returning `Result<(), ToolkitError>`: if
  `std::env::var_os(ENV_TEST_RNG).is_some() || std::env::var_os(ENV_TEST_IDENTIFIER).is_some()`, return
  `Err(ToolkitError::BadInput(<the 3.1 stem without "error: ">))`. Call it with `?` as the first statement of the
  `Slip39Command::Split(a)` arm (:215), under the same `cfg`.
- Module doc :34-38: replace the five lines from `//!   - env-var determinism wedge (Q2 fold)` through
  `//!     unset) uses \`OsRng\` + library-generated random identifier.` with:
  `//!   - env-var determinism wedge (Q2 fold; TK-Q4): only with the off-by-default cargo feature`
  `//!     \`slip39-test-rng\`, \`MNEMONIC_SLIP39_TEST_RNG\` (32-byte hex seed for \`ChaCha20Rng\`) +`
  `//!     \`MNEMONIC_SLIP39_TEST_IDENTIFIER\` (decimal u16 in 0..=32767) drive split, with an always-on`
  `//!     insecurity advisory. Without the feature (every shipped build) a set variable is refused`
  `//!     before any input is read; split uses \`OsRng\` + a library-generated random identifier.`

## 5. RED rows

Script: `design/fix-sec-tk-slip39-test-rng-feature-red.sh <mnemonic-binary>` (committed on this branch). Signet is not
involved (SLIP-39 has no network); the phrase is the public BIP-39 vector `abandon x11 about`. Build:
`cargo build --locked -p mnemonic-toolkit --bin mnemonic`, then run the script on `target/debug/mnemonic`.

Run on b2a2202b (default build), seen to show the defect (R1-R7) and the controls (C1, C2):

    R1-test-rng-seed rc=0 shares=3 insecure-warning=1 error=<none>
    R1-test-rng-seed reproducible=yes
    R2-identifier-only rc=0 shares=3 insecure-warning=1 error=<none>
    R3-empty-value rc=1 shares=0 insecure-warning=1 error=error: MNEMONIC_SLIP39_TEST_RNG: must decode to exactly 32 bytes; got 0 bytes
    R4-non-utf8-value rc=0 shares=3 insecure-warning=0 error=<none>
    R5-before-stdin rc=0 shares=3 insecure-warning=1 error=<none>
    R5-before-stdin left=0
    R6-before-env-source rc=1 shares=0 insecure-warning=0 error=error: --from phrase=: env-var MNX_RED_UNSET referenced by sentinel is not set
    R7-before-argv-checks rc=1 shares=0 insecure-warning=0 error=error: slip39 split: --from only accepts phrase=<value-or-> or entropy=<hex-or->; got xpub=
    C1-combine-ignores rc=1 shares=0 insecure-warning=0 error=error: slip39 combine: at least one share required
    C2-unset-os-rng rc=0 shares=3 insecure-warning=0 error=<none>
    C2-unset-os-rng reproducible=no

Expected after the fix, default build (write `<STEM>` for the 3.1 line, `error: ` included):

    R1-test-rng-seed rc=1 shares=0 insecure-warning=0 error=<STEM>
    R1-test-rng-seed reproducible=no
    R2-identifier-only rc=1 shares=0 insecure-warning=0 error=<STEM>
    R3-empty-value rc=1 shares=0 insecure-warning=0 error=<STEM>
    R4-non-utf8-value rc=1 shares=0 insecure-warning=0 error=<STEM>
    R5-before-stdin rc=1 shares=0 insecure-warning=0 error=<STEM>
    R5-before-stdin left=93
    R6-before-env-source rc=1 shares=0 insecure-warning=0 error=<STEM>
    R7-before-argv-checks rc=1 shares=0 insecure-warning=0 error=<STEM>
    C1-combine-ignores rc=1 shares=0 insecure-warning=0 error=error: slip39 combine: at least one share required
    C2-unset-os-rng rc=0 shares=3 insecure-warning=0 error=<none>
    C2-unset-os-rng reproducible=no

(`left=93` is the byte count of the phrase: the binary read none of stdin.)

Expected after the fix, feature-on build (`cargo build --locked -p mnemonic-toolkit --bin mnemonic --features
slip39-test-rng`): the b2a2202b block above, line for line (§3.2).

## 6. Tests that change

- New row `slip39_test_rng_env_refused_without_feature` in `crates/mnemonic-toolkit/tests/cli_slip39_refusals.rs`
  (not feature-gated as a file), marked `#[cfg(not(feature = "slip39-test-rng"))]`. One test function, table-driven,
  each case a separate `assert_cmd` run of `mnemonic slip39 split --group-threshold 1 --group 3,2 ...` with
  `env_remove` of both variables first and then the case's variable set. Each case asserts exit 1, empty stdout, and
  stderr equal to exactly the 3.1 line plus `\n`. Cases:
  (a) `MNEMONIC_SLIP39_TEST_RNG` = 64 x `1`, `--from phrase=-`, stdin `ABANDON_12` (the file's const), plus
  `--json-out <tempdir>/split.json`; also asserts the file does not exist afterwards;
  (b) `MNEMONIC_SLIP39_TEST_IDENTIFIER` = `12345` alone, `--from phrase=-`, stdin `ABANDON_12`;
  (c) `MNEMONIC_SLIP39_TEST_RNG` = empty string, `--from phrase=-`, stdin `ABANDON_12`;
  (d) `#[cfg(unix)]` block: `MNEMONIC_SLIP39_TEST_RNG` = the one byte `0xff` (`OsStr::from_bytes`), same input;
  (e) `MNEMONIC_SLIP39_TEST_RNG` = 64 x `1`, `--from phrase=@env:MNX_ROW_UNSET` with `MNX_ROW_UNSET` removed;
  (f) `MNEMONIC_SLIP39_TEST_RNG` = 64 x `1`, `--from xpub=x`.
  RED: on b2a2202b every case fails (a, b and d exit 0 with shares; c, e and f exit 1 with another stem).
- `tests/cli_slip39_advisories.rs` and `tests/cli_slip39_json.rs`: no content change; they run only with the feature
  (Cargo.toml `[[test]]` entries). Their env rows (`advisory_env_var_test_rng_set_emits_row_6_always_on`,
  `advisory_env_var_test_identifier_alone_emits_row_6_always_on`, `json_split_g4_anchor_1_sha_pin_with_test_rng_env_var`,
  `json_split_g4_anchor_2_sha_pin_different_env_vars`, `json_split_g4_env_var_yields_deterministic_output`, and the
  two `json_combine_envelope_shape_*` rows) pin §3.2 unchanged.
- No GUI schema-mirror change (no flag, option or subcommand changes; the variables are not clap surface).

## 7. CI delta (`.github/`; named in the PR description)

Append ` --features slip39-test-rng` to each of the four nextest commands, nothing else on those lines changes:

- `.github/workflows/rust.yml:188` `cargo nextest run --workspace --no-fail-fast` becomes
  `cargo nextest run --workspace --no-fail-fast --features slip39-test-rng`.
- `.github/workflows/rust.yml:638` `cargo nextest run --target ${{ matrix.target }} -p mnemonic-toolkit --no-fail-fast`
  becomes `cargo nextest run --target ${{ matrix.target }} -p mnemonic-toolkit --no-fail-fast --features slip39-test-rng`.
- `.github/workflows/rust.yml:652`: insert ` --features slip39-test-rng` after `--no-fail-fast` and before ` -- --skip
  mlock --skip g1_`.
- `.github/workflows/release.yml:155` `cargo nextest run --locked --workspace --no-fail-fast` becomes
  `cargo nextest run --locked --workspace --no-fail-fast --features slip39-test-rng`.

- New step in the rust.yml test job (I1 of the r1 review), placed after the `cargo build --tests` step (:168-174,
  default features, so it reuses that build) and before the :176 nextest step, same `if:` and `ulimit` lines, named
  `cargo nextest run (shipped build: test-RNG refusal)`, running with no `--features`. Exact text:

        - name: cargo nextest run (shipped build: test-RNG refusal)
          if: steps.docs.outputs.docs_only != 'true'
          run: |
            if [ "${{ runner.os }}" = "Linux" ]; then
              ulimit -l 65536 || true
            fi
            cargo nextest run --locked -p mnemonic-toolkit --test cli_slip39_refusals -E 'test(=slip39_test_rng_env_refused_without_feature)'

  If the row is compiled out (the feature defaulted on), nextest exits 4 with `error: no tests to run`, so the step
  also fails if a shipped build turns the feature on. This is a default-feature step, not one of the four nextest
  jobs the ruling names (an exception to "every CI nextest job adds the feature": accepted by the coordinator 2026-11-22); the PR
  description names it with the other `.github/` changes. `cargo build --tests --workspace` (:174) stays unchanged
  (default features); this step reuses it.

Unchanged: the `cargo test --doc`, mlock `cargo test` legs, `bitcoind-differential`, `cross-tool-differential`, clippy
(`rust.yml:398`, default features) and every release-artifact build (no `--features` anywhere in `release.yml` build
steps, so the shipped binary has the feature off). Unqualified `--features slip39-test-rng` with `--workspace` is
accepted by cargo 1.85 because one selected package defines it (checked: an unknown feature name with `--workspace`
errors "none of the selected packages contains this feature").

## 8. Spec delta (`design/SPEC_slip39_v0_13_0.md`)

- §2.5 heading (:208): `(25 classes;` becomes `(26 classes;`, and before the heading's final `)` insert `; row 26
  added 2026-11-22 (FIX sec-tk-slip39-test-rng-feature, TK-Q4): a set test-RNG variable is refused in a build without
  the \`slip39-test-rng\` cargo feature`.
- §2.5 table: after row 25 (:236) add
  `| 26 | \`split\` with \`MNEMONIC_SLIP39_TEST_RNG\` or \`MNEMONIC_SLIP39_TEST_IDENTIFIER\` set (any value, even empty
  or not UTF-8) in a build without the \`slip39-test-rng\` cargo feature; checked first, before every other refusal and
  before any input is read | 1 | \`slip39 split: MNEMONIC_SLIP39_TEST_RNG or MNEMONIC_SLIP39_TEST_IDENTIFIER is set,
  but this build has no test RNG; unset both to split with the system random source\` |` (one table line).
- §2.6 row 6 (:249) trigger cell: prefix `Test build only (cargo feature \`slip39-test-rng\`): ` before `Either`.
- §4 G4 (:268): replace `both are inert in production (env-vars unset → \`OsRng\` + library-generated random
  identifier) and trigger an always-on insecurity advisory in stderr when set.` with `both exist only in a build with
  the off-by-default \`slip39-test-rng\` cargo feature, which the two test files require; in such a build they trigger
  an always-on insecurity advisory in stderr when set, and every other build refuses them (§2.5 row 26).`
- §6 (:309), replace the sentence `Both are runtime-gated (NOT compile-time \`#[cfg]\`-gated) so they exercise the same
  production code paths \`cargo run\` / \`cargo install\` users invoke.` with `Both are compile-time gated (TK-Q4,
  2026-11-22): they are read only in a build with the cargo feature \`slip39-test-rng\`, off by default and enabled by
  the CI nextest jobs. Every other build, including every shipped binary, refuses either variable when set (§2.5 row
  26) before any input is read.`
- §6 `**Always-on insecurity advisory.**` paragraph (:316): after `When EITHER env-var is set` insert ` in a build with
  the \`slip39-test-rng\` feature`.

## 9. Manual delta (`docs/manual/src/40-cli-reference/41-mnemonic.md`, lockstep)

- Refusals intro (:2546-2551): replace `(25 classes; row 24` with `(26 classes; row 24`, and replace the closing
  `` `split_ems` rule).`` with `` `split_ems` rule; row 26 refuses the test-only RNG variables, which a shipped binary
  does not read).``
- Refusals table: after the last row (:2579) add
  `| \`split\` with \`MNEMONIC_SLIP39_TEST_RNG\` or \`MNEMONIC_SLIP39_TEST_IDENTIFIER\` set in the environment (any
  value, even empty; checked before every other refusal and before any input is read) | \`slip39 split:
  MNEMONIC_SLIP39_TEST_RNG or MNEMONIC_SLIP39_TEST_IDENTIFIER is set, but this build has no test RNG; unset both to
  split with the system random source\` |` (one table line).
- Advisories intro (:2583-2584): replace `Mirror of SPEC §2.6 (6 rows).` with `Mirror of SPEC §2.6 rows 1-5; row 6 (the
  test RNG warning) exists only in a test build, and a shipped binary refuses those variables (see Refusals).`
- Delete the advisory row at :2594 and the four-line note at :2596-2599 (with the blank line after it, so one blank
  line separates the table from `### Trezor interop`).
- `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md` §V.4.2 (:13): replace the paragraph starting
  `**None.**` with a one-row table: `slip39-test-rng` | no (`default = []` implicit) | lets `slip39 split` read
  `MNEMONIC_SLIP39_TEST_RNG` / `_IDENTIFIER`; without it a set variable is refused | none (columns: feature, default,
  effect, dependencies; header row added in the same table).
- Unchanged: `docs/manual-gui` (no hit for `SLIP39_TEST`), the flag-coverage lint (no flag change).

## 10. CHANGELOG (`CHANGELOG.md`, under `## [Unreleased]`)

PROPOSED (architect confirms):

    ### `slip39 split` refuses the test RNG variables

    - **`slip39 split`**: `MNEMONIC_SLIP39_TEST_RNG` or `MNEMONIC_SLIP39_TEST_IDENTIFIER` set in the environment (any
      value) now exits 1 before reading any input. They used to seed the share randomness with only a warning, so
      anyone who knew the value could recompute the shares. They now work only in a test build (cargo feature
      `slip39-test-rng`, off by default).

## 11. Coverage map (rule -> row)

| Rule | Rows |
|---|---|
| 3.1 refusal, exit 1, exact stem, empty stdout | R1, R2; Rust (a), (b) |
| 3.1 no `--json-out` file | Rust (a) |
| 3.1 edge 1: empty and non-UTF-8 values count as set | R3, R4; Rust (c), (d) |
| 3.1 edge 2: either variable alone | R1 (RNG), R2 (identifier); Rust (a), (b) |
| 3.1 edge 3: before stdin read | R5 (`left=93`) |
| 3.1 edge 3: before `@env:` resolution | R6; Rust (e) |
| 3.1 edge 3: before argv checks | R7; Rust (f) |
| 3.1 edge 4: combine unchanged | C1 |
| 3.1 edge 6: unset, default build unchanged | C2; the existing default-feature slip39 suites |
| 3.2 feature-on unchanged | RED script on a feature-on binary (b2a2202b block); the two `required-features` files |
| §7 CI runs the shipped side (feature defaulted on is caught) | the new rust.yml step (§7): exit 4 `no tests to run` if the row is compiled out; review of the `.github/` diff |
| §7 CI runs the feature side | review of the `.github/` diff (the script does not cover it) |

## 12. Gate (build agent; full CI locally, `$HOME/.cargo/bin` first, cargo 1.85.0)

    cargo fmt --all --check
    cargo clippy --locked --all-targets -- -D warnings
    cargo clippy --locked --all-targets --features slip39-test-rng -- -D warnings
    cargo nextest run --locked --workspace --no-fail-fast
    cargo nextest run --locked -p mnemonic-toolkit --test cli_slip39_refusals -E 'test(=slip39_test_rng_env_refused_without_feature)'
    cargo nextest run --locked --workspace --no-fail-fast --features slip39-test-rng
    cargo test --locked --workspace --doc
    cargo build --locked -p mnemonic-toolkit --bin mnemonic && design/fix-sec-tk-slip39-test-rng-feature-red.sh target/debug/mnemonic
    cargo build --locked -p mnemonic-toolkit --bin mnemonic --features slip39-test-rng && design/fix-sec-tk-slip39-test-rng-feature-red.sh target/debug/mnemonic
    make -C docs/manual lint MNEMONIC_BIN=... MD_BIN=... MS_BIN=... MK_BIN=...   (as CLAUDE.md; the CI manual job)

The first two nextest runs are the only ones that run `slip39_test_rng_env_refused_without_feature` (the second is the
CI shipped-side step of §7); the third is the only
one that runs the two `required-features` files. Not covered by this gate: the CI `.github/` edits themselves (read the
diff) and the musl/aarch64 rows (green CI on the head).

## 13. FOLLOWUPS

Downstream (r1 M3): mnemonic-engrave `design/IMPLEMENTATION_PLAN_seedhammer_slip39_recovery_D1.md:732-736` (Step 2, not
yet executed) generates fixtures through the CLI wedge; with a stock binary that now refuses (fails closed). Step 2
must use a `--features slip39-test-rng` build of `mnemonic` or a seeded `slip39_split` library call.

The author's line `tk-slip39-test-rng-refusal-not-in-ci` is not filed: the §7 step closes it.

Closes no FOLLOWUPS line (none names this defect). Files none in the repo. The secret row's status is updated by the
coordinator after the build merges.
