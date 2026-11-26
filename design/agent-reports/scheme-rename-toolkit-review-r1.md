# scheme rename (toolkit) — independent adversarial review r1

Diff: `origin/claude/project-thread-utm1aa (40217d8)..origin/claude/project-thread-0jg5n7 (87758c5)`, one commit, 190 files.
Brief: `/mnt/project-files/cli-review/scheme-origin-path-plan-2026-11-26.md`.

**Verdict: 0 Critical, 2 Important, 6 Minor.** Apart from the spelling, I found no behaviour change: the accepted values, defaults, exit codes, stdout bytes and JSON keys are all the same.

## Critical

None.

## Important

**I1. A live doc generator still teaches `--template`.** The brief says to update live scripts.
- Files: `.examples-build/gen.sh` lines 151, 235, 239, 263, 267-290, 343, 349; `.examples-build/derive_keys.sh` lines 9-10; the generated `.examples-build/Examples.md` lines 40, 244-506.
- `examples.yml` rebuilds `Examples.md` from the source-built binary and diffs it. That diff stays green only because the hidden alias works.
- Failure scenario: the published Examples PDF keeps telling readers to type the hidden `--template`, and nothing flags it. This is the only user-facing doc left on the old spelling.

**I2. Chapter 45 still describes mr's printed kit text with the old flag (depends on the mr PR).**
- At `docs/manual/src/40-cli-reference/45-mr.md:3047`, the chapter says the kit text prints `` `--template-preset owner-heir` ``. That text comes from `mnemonic-refugium` `crates/mr-codec/src/kits.rs:4534`.
- The brief says mr's printed text moves to `--scheme`. Once it does, this sentence is wrong.
- The `manual-mr` lint does not check printed text, so nothing would catch it.
- I could not check the mr side: `mnemonic-refugium` `origin/claude/project-thread-0jg5n7` equals `master`, and has no rename yet. Reconcile this line with the mr PR.

## Minor

- **M1. The CHANGELOG oversells the JSON `flag` change.**
  - `ToolkitError::details()` at `error.rs:1019-1021` is `#[allow(dead_code)]`. Only unit tests call it. No output path prints the `flag` value. So changing `"--template"` to `"--scheme"` cannot be observed and breaks no contract.
  - I grepped mnemonic-gui, refugium-wallet and mnemonic-engrave: none of them parses a toolkit error `flag`.
  - The CHANGELOG line ("structured `flag` value … is `--scheme`") describes something nobody sees.
- **M2. One test's doc comment does not match the test.** `tests/cli_scheme_flag_alias.rs:218` says "A refusal reached through the old spelling", but `remedies_name_scheme` never passes the old spelling.
  - The alias tests cover only success paths.
  - I ran the error paths through the alias by hand (all exit 64, all naming `--scheme`): `--template=bip84`, `--archetype=kofn-recovery --spec x` (conflict), `--template bogus`, `--template X --scheme X` (duplicate), `bundle --template … --import-json` and `export-wallet --template … --descriptor/--from-import-json`.
- **M3. The new FOLLOWUP has no companion.** `design/FOLLOWUPS.md:10` `scheme-name-json-keys` is tier cross-repo but has no `Companion:` line. It also leaves out the xpub-search text output `(template=bip84, account=…)` at `path_of_xpub.rs:321`, `passphrase_of_xpub.rs:381`, `account_of_descriptor.rs:366` and `passphrase_search.rs:151`.
- **M4. Value name in chapter 43.** `43-ms.md:197` writes `--origin-path <T>`, but the settled ms value name is `<NAME>`.
- **M5. Leftover "preset" wording in chapter 45.** `45-mr.md:3726` still says "unknown preset … six presets" (prose only).
- **M6. mnemonic-gui will need updates at its next toolkit pin bump.**
  - When mnemonic-gui moves to a toolkit release with this rename, these fail: `tests/gui_schema_conditional_drift.rs`, `schema_mirror`, and `src/schema/archetypes.rs:306` (`find(|f| f.name == "--archetype")`).
  - At runtime the GUI still spawns `--template` and `--archetype`, and those work through the aliases.
  - This is expected under the planned paired GUI PR. `meta.template_groups` is unchanged.

## Checked and clean

- **clap ids:** every `requires`, `conflicts_with` and `conflicts_with_all` was moved to the `scheme` id; there are no dangling `"template"` or `"archetype"` ids.
- **build-descriptor:** a sub-flag without `--scheme` gives exit 64, "required: --scheme".
- **bundle:** a missing `--scheme` gives exit 64.
- **Correctly kept:** the `md descriptor --template` text at `taproot_override_classify.rs:96` and in chapters 41 and 42 still names md's own flag, which is right.
- **src:** no string literal meaning the scheme still prints `--template` or `--archetype`.
- **Tests:** in the 114 test files, the + and − line counts differ only where rustfmt reflowed lines. No assertion was dropped: I counted 6 removed and 44 added assert lines, the extra ones from the new alias file.
- **Transcripts:** all 15 changed `.cmd` files, re-run against the built binary, are byte-identical to their `.out` and `.err` files.

## Commands and counts

- `CARGO_BUILD_JOBS=2 cargo test --locked --workspace --no-fail-fast`, output in `/home/user/scratch-tk-review/test.log`:
  - 23 result blocks
  - **4547 passed, 0 failed, 33 ignored**, exit 0
  - all 9 `cli_scheme_flag_alias` tests ok
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: exit 0, 0 warnings.
- `cargo fmt --all --check`: exit 0.
- The manual `lint.sh flag-coverage` was **not** run, because md, ms, mk and mr are not installed here. Instead I checked by hand that each section of chapter 41 lists `--scheme` and `--add-path <PATH>`.

## Fold

Folded on `claude/project-thread-0jg5n7` in the commit after 87758c5.

- **I1, fixed.** `.examples-build/gen.sh` and `derive_keys.sh` now type `--scheme`. I regenerated `Examples.md` the way `examples.yml` does it (`EXAMPLES_BIN_DIR=target/debug bash .examples-build/gen.sh`). The diff against the committed file is the 12 lines that carry the flag spelling and nothing else. The rendered `docs/Examples.pdf` was not rebuilt, because the LaTeX toolchain is not installed here.
- **I2, deferred as asked.** `45-mr.md:3047` still quotes mr's printed kit text with `--template-preset owner-heir`. The coordinator will reconcile it when the mr PR changes that text.
- **M1, fixed.** I dropped the CHANGELOG claim about the structured `flag` value, because no output path prints it. The source values stay `--scheme`, which is consistent and invisible.
- **M2, fixed.**
  - The doc comment of `remedies_name_scheme` now says what the test does: a runtime refusal about a missing scheme names `--scheme`.
  - New test `refusals_are_identical_under_the_alias` covers four refusals: an export-wallet conflict with `--descriptor`, a bundle with an unknown scheme value, a build-descriptor conflict with `--spec`, and a restore given a multisig scheme at runtime.
  - Each asserts byte-identical stdout, stderr and exit code under both spellings, that stderr names `--scheme`, and that stderr does not contain the old spelling.
- **M3, fixed.** `scheme-name-json-keys` now has a `Companion:` line for mnemonic-gui, and lists the xpub-search text output `(template=…, account=…)`.
- **M4, fixed.** Chapter 43 now reads `--origin-path <NAME>`.
- **M5, fixed in prose.** Chapter 45 now reads "An unknown scheme is refused with the six schemes". That sentence is the chapter's own prose, not a quote of mr's output. No mr rename branch existed to check against: mnemonic-refugium `origin/claude/project-thread-0jg5n7` is absent or equal to master.
- **M6, no action.** The GUI PR is in progress.

## Fold, I2 (controller, after the mr PR's commit 2184d6eff)

mr-codec `kits.rs` now prints `` `--scheme {name}` `` in the kit's "these" list, so chapter 45's description of that line reads `` `--scheme owner-heir` ``. M5's wording ("An unknown scheme is refused with the six schemes") matches the mr branch's help/refusal wording.
