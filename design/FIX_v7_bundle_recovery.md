# FIX-V7: bundle card recovery summary for general policies

Status: STOP-1..3 ruled (architect-2026-11-04-fix-v7-stops.md); awaiting the FIX review. FIX branch `fix/v7-bundle-recovery` (base `ac3ff70a`) stays as is, so its cited SHAs stay reachable. Build on `build/v7-bundle-recovery` (architect STOP-7, `architect-2026-11-04-fix-v5-stops.md`):
`git fetch origin && git switch -c build/v7-bundle-recovery origin/master && git cherry-pick d19256e8 ac3ff70a && git cherry-pick ac3ff70a..<reviewed fix/v7-bundle-recovery head>`.
Do not take `a34bd61e` or `123c258d` (V5's rows). The evidence below was measured on an equivalent tree: `origin/master` `12e013bc` plus `d19256e8`, `ac3ff70a` and `283132e7`, cherry-picked.
Merge order: whichever of the V5 and V7 PRs is first green with 0C/0I merges first. The second merges `origin/master` in, re-runs all of section 7, regenerates `.examples-build/Examples.md` with `gen.sh`, and resolves the `design/FOLLOWUPS.md` append. T3 waits for this FIX.

## 1. Inputs and scope

- Grade and rulings (bind): coordinator `agent-reports/grade-2026-11-03-tk-v5-v7.md`, rulings 1, 2, 3 and 7. V7-1..V7-11 are Critical.
- Small-fix track: no new flag, type or command. One repo (mnemonic-toolkit) plus its manual lockstep.
- Code files (ruling 7): `crates/mnemonic-toolkit/src/cmd/bundle.rs`, `src/format.rs` (card), `src/format.rs` `MultisigInfo` (JSON writer), `src/wallet_import/json_envelope.rs` `MultisigInfoView` (JSON reader), `src/cmd/export_wallet.rs` (reader consumer), `src/cmd/import_wallet.rs` (emitter).
- The builder works from this document and the committed evidence directory `design/agent-reports/fix-doc-tk-v7-evidence/` (the evidence dir below). Its patches are the measured GREEN; this document's texts are normative where they differ.
- Not in scope: T3 (lifting the `sortedmulti_a` whole-leaf gate) waits for this FIX to merge.

## 2. Spec delta (exact texts)

### 2.1 Plain predicate (ruling 1, ruling 3)

A descriptor is **plain** when exactly one of these holds. Otherwise it is **general**.

1. The root is `wsh`, `sh`, or `sh(wsh(..))`, and its only child is a k-of-n node.
2. The root is `tr` with internal key `NUMS` (md `InternalKey::NumsPoint`), and its script tree is a single leaf (not a branch) that is a k-of-n node.

A **k-of-n node** is one of the following:

- `multi`, `sortedmulti`, `multi_a` or `sortedmulti_a`, with its k;
- a **pure-key `thresh`**, with its k.

A pure-key `thresh` is one where every argument is `pk(K)` or `pkh(K)` under zero or more `s:`/`a:` wrappers and nothing else. Any other wrapper (`n:`, `l:`, `v:`, `d:`, `j:`, `u:`, `t:`) or argument (`multi`, `older`, `after`, a hash, a nested combinator) makes the `thresh` general.

Further rules:

- A `tr` internal key that is a real key (md `InternalKey::Slot`), or the Liana unspendable key (`InternalKey::LianaUnspendable`), makes the descriptor general, whatever the leaves are.
- A taptree with two or more leaves is general.
- k is the k of the k-of-n node. n is the bundle's signing-key count as today (the cosigner count), unchanged.
- **Template mode** (`mnemonic bundle` without `--descriptor`/`--descriptor-file`/`--import-json`) is always plain, with k = `--threshold` if given, else n. This is unchanged.

### 2.2 Engraving card (stderr), `mnemonic bundle`

| case | `# Threshold:` line | `# Recovery:` line |
|---|---|---|
| n == 1 (any policy) | absent (unchanged) | absent (unchanged) |
| n > 1, plain | `# Threshold: {k} of {n}` (unchanged) | `# Recovery: any {k} of {n} signing keys + md1 (template card).` (unchanged) |
| n > 1, general | **absent** | **`# Recovery: general policy, not k-of-n (the md1 policy decides which key sets can spend); keep all {n} signing keys, any hash preimages it names, and md1 (template card).`** |

The general Recovery line is one line: there is no wrap and the line ends with `\n`. It sits where the plain Recovery line sits today. Every other card line is unchanged. Exit code unchanged (0).

### 2.3 Bundle JSON (`mnemonic bundle --json`)

- `multisig.threshold`: JSON integer k for plain, JSON `null` for general. Its type becomes `Option<u8>`, serialized as `null` rather than omitted. Every other field is unchanged. `multisig` stays absent when n == 1, as today.
- Reader: `MultisigInfoView.threshold` becomes `Option<u8>`. An envelope with `"threshold": null` is accepted by every `--import-json` / `--from-import-json` consumer:
  - `mnemonic bundle --import-json`;
  - `mnemonic export-wallet --from-import-json`.
- `mnemonic bundle --import-json` ignores the envelope's `threshold`, as today, and now also ignores `--threshold`, which used to set k on this path (architect STOP-2 (b)). It recomputes k from the descriptor by 2.1, so a stale integer or a `--threshold` value still yields the descriptor's card, at exit 0.
- `export-wallet --from-import-json` passes no threshold (`None`) from a null envelope.
  - `--format descriptor` / `bitcoin-core` / `bip388` output is byte-identical to the same envelope carrying an integer.
  - `--format sparrow` and the other template formats keep their existing general-policy refusal, at exit 1, with the unchanged text:
    `error: --from-import-json: --format sparrow cannot represent a general wallet policy (timelocks/hashlocks/non-multisig miniscript); it is a plain k-of-n multisig format. Use --format descriptor / bitcoin-core / bip388 for faithful descriptor passthrough.`

### 2.4 `mnemonic import-wallet`

- After the input is parsed and the entry selected, every entry's threshold is recomputed from its descriptor by 2.1. It is k when plain, none when general, and it replaces whatever the source format carried. This applies, for example, to BSMS, which declares a k for a decaying policy.
- `--json`: `[i].bundle.multisig.threshold` is k or `null` by the same rule (via 2.3's type).
- Text summary: the existing `none` arm prints `bundles[{i}].threshold=none` and `threshold=none` for general entries; plain entries print the integer as today.
- n == 1 entries print as today. `wsh(multi(1,K0))` is plain, k = 1. A single-key entry prints `none`.

## 3. Production change (what, not how)

- `bundle.rs`: add the 2.1 predicate as one function over the md `tree::Node`. It returns `Option<u8>`: k when plain, `None` when general.
  - Descriptor mode and the `--import-json` path take k from it.
  - Template mode keeps `--threshold`, or n when the flag is absent.
  - A hint implementation is the evidence dir's `fix-v7-production.patch` (adds `pub(crate) fn plain_multisig_threshold(root: &md_codec::tree::Node) -> Option<u8>`). It is not normative, and on the `--import-json` path it ignores `--threshold`, which is one of STOP-2's two options, not a ruling.
- `format.rs`:
  - `MultisigInfo.threshold: Option<u8>`.
  - The unified card prints the 2.2 table.
- `json_envelope.rs`:
  - `MultisigInfoView.threshold: Option<u8>`.
  - Its one unit test `bundle_json_view_round_trips_every_field_of_bundle_json` changes two lines (717 `threshold: 2,` to `threshold: Some(2),`; 763 `assert_eq!(m.threshold, 2);` to `assert_eq!(m.threshold, Some(2));`).
- `export_wallet.rs`: read the threshold as `Option` (`and_then`).
- `import_wallet.rs`: the 2.4 recompute. The emitter uses the recomputed value.
- `bundle --import-json` ignores `--threshold` (STOP-2 (b)), as the hint patch does.

## 4. RED rows

All 30 rows are committed (`d19256e8`: rows 1-12; `283132e7`: rows 13-30). Rows 1-29 are in `crates/mnemonic-toolkit/tests/cli_bundle_recovery_threshold_text.rs`; row 30 is a unit test in `crates/mnemonic-toolkit/src/format.rs` (`mod tests`). The names are in the evidence dir's `row-names.txt`; `fix-rows-filter.txt` is the nextest filter `test(/(^|::)(row1|...|row30)$/)` over them.

Command (from the worktree root):

`PATH=$HOME/.cargo/bin:$PATH cargo nextest run --locked -p mnemonic-toolkit -E "$(cat design/agent-reports/fix-doc-tk-v7-evidence/fix-rows-filter.txt)"`

- On the build base before the GREEN commit: the summary line reads `30 tests run: 6 passed, 24 failed, <S> skipped`, and the 24 failures are exactly the rows marked FAIL below, each for the reason given. Measured: `Summary [   0.023s] 30 tests run: 6 passed, 24 failed, 4111 skipped`.
- After the GREEN commit: `30 tests run: 30 passed, <S> skipped`, no failure. Measured: `Summary [   0.045s] 30 tests run: 30 passed, 4111 skipped`.
- `<S>` moves with the base; the matched count 30 is the invariant. A matched count other than 30 is a FAIL.
- The RED run's output must contain `Compiling mnemonic-toolkit`. A reused `CARGO_TARGET_DIR` can hold a test binary built from a patched tree, and nextest then reports a false `30 passed` on base (measured once on this branch). If the line is missing, `touch crates/mnemonic-toolkit/src/*.rs` and re-run, or use a fresh target dir. Re-measured after a forced rebuild: `Summary [   0.069s] 30 tests run: 6 passed, 24 failed, 4125 skipped`, the same 24 failures.

The "base" column is the measured outcome on unmodified production code. Reasons cite the test file's assertion line (`t:` = `cli_bundle_recovery_threshold_text.rs`).

| # | row | origin | base | reason on base |
|---|---|---|---|---|
| 1 | `v7_tr_two_pk_leaves_not_2_of_2` | triage | FAIL | t:96 `assert_no_false_claim`: card prints the false `# Threshold: K of N` |
| 2 | `v7_tr_three_pk_leaves_depth2_not_3_of_3` | triage | FAIL | t:96, as row 1 |
| 3 | `v7_wsh_or_d_two_keys_not_2_of_2` | triage | FAIL | t:96, as row 1 |
| 4 | `v7_wsh_or_d_key_or_multi_not_2_of_3` | triage | FAIL | t:96, as row 1 |
| 5 | `v7_wsh_or_i_multi_or_key_not_2_of_3` | triage | FAIL | t:96, as row 1 |
| 6 | `v7_tr_multi_a_leaf_plus_pk_leaf_not_2_of_3` | triage | FAIL | t:96, as row 1 |
| 7 | `v7_tr_sortedmulti_a_leaf_plus_pk_leaf_not_2_of_3` | triage | FAIL | t:96, as row 1 |
| 8 | `v7_tr_non_nums_internal_key_plus_multi_a_not_2_of_3` | triage | FAIL | t:96, as row 1 |
| 9 | `v7_wsh_and_v_with_or_branch_not_3_of_3` | triage | FAIL | t:96, as row 1 |
| 10 | `v7_two_disjoint_multi_leaves_not_any_2_of_4` | triage | FAIL | t:167: card claims any 2 of 4 keys recover, but {K0,K2} cannot spend |
| 11 | `v7_and_of_two_multis_not_2_of_4_understated` | triage | FAIL | t:96, as row 1 |
| 12 | `v7_control_plain_shapes_keep_threshold_and_recovery_lines` | triage | PASS | control: plain `wsh`/`tr` shapes keep both lines |
| 13 | `v7_general_shapes_print_exact_general_recovery_line` | new | FAIL | t:218 `assert_general_card`: a general policy must print no `# Threshold:` line |
| 14 | `v7_non_pure_thresh_gets_general_line` | new | FAIL | t:218, as row 13 |
| 15 | `v7_archetypes_get_general_line` | new | FAIL | t:218, as row 13 |
| 16 | `v7_e3_timelocked_inheritance_gets_general_line` | new | FAIL | t:218, as row 13 |
| 17 | `v7_bundle_json_threshold_null_for_general_policy` | new | FAIL | t:424: `multisig.threshold` left `Number(2)`, right `Null` |
| 18 | `v7_import_wallet_text_threshold_none_for_general` | new | FAIL | t:433: stdout carries `bundles[0].threshold=2`, not `=none` |
| 19 | `v7_import_wallet_json_threshold_null_for_general` | new | FAIL | t:453: left `Some(Number(2))`, right `Some(Null)` |
| 20 | `v7_null_threshold_envelope_accepted_by_bundle_import_json` | new | FAIL | t:467: exit 1, `error: --import-json: envelope JSON parse: invalid type: null, expected u8 at line 1 column 2490` |
| 21 | `v7_null_threshold_envelope_accepted_by_export_wallet_descriptor` | new | FAIL | t:494: null envelope refused, `error: --from-import-json: envelope JSON parse: invalid type: null, expected u8` |
| 22 | `v7_null_threshold_envelope_template_format_still_refuses_general_policy` | new | FAIL | t:519: the parse error of row 21 appears where the sparrow general-policy refusal (2.3) is expected |
| 23 | `v7_stale_envelope_threshold_recomputed_by_bundle_import_json` | new | FAIL | t:218, as row 13 (an integer threshold in the envelope for a general descriptor) |
| 24 | `v7_import_json_threshold_flag_cannot_override_card_k` | new | FAIL | t:240: plain 2-of-3 envelope with `--threshold 1`: want exactly one line `# Threshold: 2 of 3`, left 0, right 1 (the card prints `# Threshold: 1 of 3`). See STOP-2 |
| 25 | `v7_pure_key_thresh_keeps_any_k_of_n` | new | PASS | guard (ruling 3): `thresh(2,pk,s:pk,s:pk)` keeps both lines |
| 26 | `v7_sh_plain_shapes_keep_threshold_and_recovery_lines` | new | PASS | guard: `sh(multi)`, `sh(wsh(sortedmulti))` keep both lines |
| 27 | `v7_single_key_prints_no_threshold_or_recovery_line` | new | PASS | guard: n == 1 unchanged |
| 28 | `v7_bundle_json_threshold_k_for_plain` | new | PASS | guard: JSON integer k for plain |
| 29 | `v7_import_wallet_threshold_k_for_plain` | new | PASS | guard: import-wallet integer k for plain |
| 30 | `format::tests::unified_card_general_policy_prints_general_recovery_line_without_threshold` | new | FAIL | `format.rs:510`: the card does not contain the general Recovery line (the card has no general arm) |

Counts: 30 rows; 18 new (13-30), of which 13 fail on base (13-24, 30); 24 fail on base in all; 6 PASS guards (12, 25-29).

Row 24 accepts both STOP-2 outcomes (a refusal, or a card with `# Threshold: 2 of 3`); it fails on base only because the card takes k from the flag. When STOP-2 is ruled, the builder narrows row 24 to the ruled outcome in the GREEN commit, with the exact text the ruling gives (section 8).

## 5. Tests that change

Measured with the evidence dir's production, tests and docs patches applied: the whole workspace passes (`4215 tests run: 4215 passed, 38 skipped`). Exactly these existing tests change. No other test, insta snapshot or fixture changes.

| # | file | test | change (evidence) |
|---|---|---|---|
| 1 | `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs` | `bsms_2_line_happy_path` | `stdout.contains("threshold=2")` becomes `stdout.contains("threshold=none")`, and its comment becomes `// Summary should list 2 cosigners + testnet + threshold=none (general policy: thresh with an sln:older leaf, FIX-V7).` (`fix-v7-tests.patch` @112) |
| 2 | same file | `bsms_decaying_multisig_n_32768` | `"threshold=2"` becomes `"threshold=none"` (@300) |
| 3 | `crates/mnemonic-toolkit/tests/cli_import_wallet_roundtrip.rs` | `fixture_bsms_2line_decay_144_parses_clean` | `"threshold=2"` becomes `"threshold=none"` (@149) |
| 4 | `crates/mnemonic-toolkit/src/wallet_import/json_envelope.rs` (`mod tests`) | `bundle_json_view_round_trips_every_field_of_bundle_json` | 717 `threshold: 2,` becomes `threshold: Some(2),`; 763 `assert_eq!(m.threshold, 2);` becomes `assert_eq!(m.threshold, Some(2));` (`fix-v7-production.patch` @714, @760) |

Rows 1-3 are BSMS decaying policies (a `thresh` with an `sln:older` leaf), so they are general by 2.1 and 2.4. The BSMS file still declares k = 2, but import-wallet now ignores that value.

Transcripts that change (replayed by `make audit`, section 7 item 8; never hand-edited to match):

- `docs/manual/transcripts/41-bundle-inheritance-cards.err`:
  - the line `# Threshold: 3 of 3` is removed;
  - `# Recovery: any 3 of 3 signing keys + md1 (template card).` becomes `# Recovery: general policy, not k-of-n (the md1 policy decides which key sets can spend); keep all 3 signing keys, any hash preimages it names, and md1 (template card).`
- `docs/manual/transcripts/41-bundle-inheritance-json.out` line 41: `"threshold": 3,` becomes `"threshold": null,`.
- `.examples-build/Examples.md` (generated by `gen.sh`, section 7 item 10), in its 11-key descriptor bundle:
  - `# Threshold: 3 of 11` is removed;
  - the Recovery line becomes the general line with `keep all 11 signing keys` (`examples.diff`).

GUI tutorial transcripts that the fix moves are not changed in this PR (STOP-1). In `docs/manual-gui/transcripts/tutorial/`, measured against the patched binary:

| file | today | under this FIX |
|---|---|---|
| `tut-j3-12-engrave.stderr.txt` | line 2 `# Threshold: 3 of 11`; line 17 `# Recovery: any 3 of 11 signing keys + md1 (template card).` | line 2 removed; the general line with `keep all 11 signing keys` |
| `tut-j4-16-engrave.stderr.txt` | line 2 `# Threshold: 3 of 12`; line 18 `# Recovery: any 3 of 12 signing keys + md1 (template card).` | line 2 removed; the general line with `keep all 12 signing keys` |
| `tut-j3-restore-feed-bundle-json.stdout.txt` | `"threshold": 3` (11 cosigners) | `"threshold": null` |
| `tut-j4-restore-feed-bundle-json.stdout.txt` | `"threshold": 3` (12 cosigners) | `"threshold": null` |

The two restore-feed bundle-json files are not in ruling 7's list; this author found them. These tutorial bundle-json transcripts are measured unchanged: `tut-j2-restore-feed`, `tut-j4-nums-feed` and `tut-j5-22`. mnemonic-gui does not read `multisig.threshold` (`rg '"threshold"|\.threshold\b' src` in mnemonic-gui `c9914ca` matches only archetype form parameters), so no GUI code depends on the type change.

## 6. Lockstep files and build steps

### 6.1 Lockstep files (ruling 7)

Eight files change in the fix PR, besides the production code and the section 5 tests. The evidence dir holds the hunks: `fix-v7-docs.patch` (files 1-7) and `examples.diff` (file 8).

| # | file | ruling 7 item, or why it is beyond ruling 7 | hunk |
|---|---|---|---|
| 1 | `design/SPEC_mnemonic_toolkit_v0_5.md` | `:179` (the Recovery line); also the Threshold line at `:168`, now "plain k-of-n only", plus a new "Plain k-of-n (FIX-V7, 2026-11-03)" paragraph carrying 2.1 | @165, @176 |
| 2 | `docs/manual/src/40-cli-reference/41-mnemonic.md` | `:671-674` (the inheritance example's prose and include range 1-11 becomes 1-10) and the bundle JSON field docs (a new `multisig.threshold` is `null` bullet); beyond R7: the import-wallet `bundle` summary line at `:1698` names `threshold` as k or `null` | @666, @759, @1698 |
| 3 | `docs/manual/transcripts/41-bundle-inheritance-cards.err` | its transcript | section 5 |
| 4 | `docs/manual/transcripts/41-bundle-inheritance-json.out` | `:41` | section 5 |
| 5 | `docs/technical-manual/src/40-bundle-formation/41-bundle-anatomy.md` | ruling 7 | @111, @121, @133 |
| 6 | `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md` | ruling 7 (`MultisigInfo.threshold` type and the JSON field) | @340, @733 |
| 7 | `docs/technical-manual/.cspell.json` | beyond R7: file 5's new text uses the word `preimages`, which the technical manual's cspell lint rejects without it | @165 |
| 8 | `.examples-build/Examples.md` | beyond R7: `examples.yml` regenerates it and fails on any diff | `examples.diff` |

Ruling 7 items not changed in this PR (STOP-1): `tut-j3-12` and `tut-j4-16` (section 5), with their PNGs (`docs/manual-gui/figures/tutorial/tut-j3-12-engrave-run.png`, `tut-j4-16-engrave-run.png`) and their prose (`docs/manual-gui/tutorial/40-j3-degrading-vault.md:130-161`, `50-j4-taproot-twin.md:103-128`). `manual-gui.yml:75-78` byte-compares `docs/manual-gui/transcripts/tutorial/` and `figures/tutorial/` with the pinned `mnemonic-gui-v0.63.0` corpus, which reflects the pinned toolkit release (0.105.1). A change here fails the GUI book's lint, the same constraint as V5's STOP-5.

### 6.2 Build steps (in this order)

Prerequisite: STOP-1, STOP-2 and STOP-3 (section 8) are ruled. Until then the builder does not start. Each ruling's text replaces the step it names, and the FIX review re-checks sections 2-6 first.

Build on `build/v7-bundle-recovery` (header), in a git worktree that is not under `/tmp`. Set `export PATH=$HOME/.cargo/bin:$PATH`; `cargo --version` must print `cargo 1.85.0 (d73d2caf9 2024-12-31)`. Never pass `--release`. Send every run to a file and grep it, and never run a suite twice to count. `E` below is `design/agent-reports/fix-doc-tk-v7-evidence`.

1. RED. Run section 4's command on the branch head before any patch. Expect exit 100 and `30 tests run: 6 passed, 24 failed, …`. The 24 failures must be the rows section 4 marks FAIL, and the output must contain `Compiling mnemonic-toolkit`. Quote the summary line in the RED evidence.
2. Production and tests:
   - `git apply $E/fix-v7-production.patch`, then `git apply $E/fix-v7-tests.patch`.
   - STOP-2 is ruled (b): the patch already ignores the flag. Narrow row 24 as architect-2026-11-04-fix-v7-stops.md STOP-2 gives.
   - `cargo build --bin mnemonic`.
   - Run section 4's command. Expect exit 0 and `30 tests run: 30 passed, …`.
3. Docs and transcripts:
   - `git apply $E/fix-v7-docs.patch`, then `git apply $E/examples.diff`.
   - Apply architect T8 and T9 (asserting replaces; each old string occurs once). T11 is dropped: no cards made by 0.105.1 or earlier exist (Brian 2026-11-04).
   - Insert the two STOP-1 interim notes (architect-2026-11-04-fix-v7-stops.md T1). Then `git status --short docs/manual-gui` lists exactly `tutorial/40-j3-degrading-vault.md` and `tutorial/50-j4-taproot-twin.md`.
   - Check the transcripts against the patched binary: section 7 item 8.
   - Then `git add .examples-build/Examples.md` and run section 7 item 10. Its `git diff` compares the worktree with the index, so an unstaged `examples.diff` is itself a diff.
   - A replayed transcript that differs from `fix-v7-docs.patch`, or a regenerated `Examples.md` that differs from `examples.diff`'s result, is a FAIL. Do not hand-edit a transcript to match.
4. Append F1 (section 8, STOP-1, in the form its ruling gives) to `design/FOLLOWUPS.md`. STOP-3 is ruled (b): `git apply $E/fix-v7-changelog.patch`, then architect T10's insert (2) only. T10 (1) is dropped: no cards made by 0.105.1 or earlier exist (Brian 2026-11-04).
5. Commit as the V7 GREEN commit. Its message quotes the section 4 summary line.
6. Run the whole gate (section 7) on the final head, then the mutation gate (section 7 item 14).
7. Do not change the version or any release file.

## 7. Gate (full toolkit CI; the builder runs all of it on the final head)

Capture each run once to a file and grep it. "Measured" is this author's run on the header's equivalent tree with the four evidence patches applied (`git diff` of that tree equals `fix-v7-production.patch`, `fix-v7-docs.patch` and `examples.diff` byte for byte; logs in `/scratch/cache/tk-fix-v7-logs/bb-*`). `cargo --version`: `cargo 1.85.0 (d73d2caf9 2024-12-31)`.

`rust.yml`:

1. `cargo +1.95.0 fmt --all -- --check` (`rust.yml:118`). Expected: exit 0. Measured: exit 0.
2. `cargo metadata --locked --format-version 1 > /dev/null` and `cargo build --locked --tests --workspace`. Expected: exit 0. Measured: exit 0.
3. `cargo nextest run --locked --workspace --no-fail-fast` (`:185`). Expected: 0 failed. Measured: `Summary [   4.627s] 4215 tests run: 4215 passed, 38 skipped`.
4. `cargo test --locked --workspace --doc` (`:190`). Expected: exit 0. Measured: `Doc-tests mnemonic_toolkit` and `Doc-tests wc_codec`, each `test result: ok. 0 passed; 0 failed; …`.
5. `cargo clippy --locked --all-targets -- -D warnings`. Expected: exit 0, no warnings. Measured: exit 0.
6. The rest of `rust.yml`, locally where it runs here, else as green CI on the PR:
   - the mlock rows `cargo test -p mnemonic-toolkit --lib mlock::tests::g2_1 -- --include-ignored`, and the same for `g2_3` and `g2_4` (`:205`, `:218`, `:234`). Measured: each `test result: ok. 1 passed`.
   - `mlock_g6_invariant` (`:524`). It needs the mnemonic-secret sibling at the ms-cli tag that `scripts/install.sh` pins (`ms-cli-v0.20.2`, `install.sh:58`). CI checks it out at `:513`-`:515` and sets `SIBLING_REPO_PATH` (`:518`). Without the sibling the test fails 2 of 2 (measured), so run it locally as:
     - `git clone --depth 1 --branch ms-cli-v0.20.2 https://github.com/bg002h/mnemonic-secret /scratch/cache/tk-v7-build-ms-sibling`
     - `SIBLING_REPO_PATH=/scratch/cache/tk-v7-build-ms-sibling cargo test -p mnemonic-toolkit --test mlock_g6_invariant -- --include-ignored`
     - Measured: `test result: ok. 2 passed; 0 failed`.
   - `cargo build --bin mnemonic`, then `sh scripts/install-<x>.test.sh` for each of `assets-gate`, `assets`, `man-step`, `msrv-guard`, `signature` and `verify` (`:268`-`:301`). Measured: each exit 0.
   - `bash ci/docs-only.test.sh`, `bash ci/code-docs-check.test.sh` and `bash ci/code-docs-check.sh`. Measured: each exit 0.
   - CI only: the release `g2_3` row, miri, the lib cross-platform check, and the musl build and test.
   - Expected: each exits 0.

`manual.yml` (the manual job and its mutation self-tests). The pinned CLIs go first in PATH. `--version` does not identify them, so install the tags into a fresh root, or reuse the V5 build's root if it exists:

```
R=/scratch/cache/tk-v7-build-ci-tools
CARGO_TARGET_DIR="$R/target" cargo install --root "$R" --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.13.0 mk-cli
CARGO_TARGET_DIR="$R/target" cargo install --root "$R" --git https://github.com/bg002h/descriptor-mnemonic --tag descriptor-mnemonic-md-cli-v0.20.4 md-cli --features cli-compiler
CARGO_TARGET_DIR="$R/target" cargo install --root "$R" --git https://github.com/bg002h/mnemonic-secret --tag ms-cli-v0.20.2 ms-cli
export PATH="$R/bin:$HOME/.cargo/bin:$PATH"
```

7. `bash ci/doc-gate-guard.test.sh` (`manual.yml:61`). Expected: exit 0. Measured: `158 passed, 0 failed`.
8. Run from `docs/manual`: `make audit MNEMONIC_BIN="$PWD/../../target/debug/mnemonic" MD_BIN=md MS_BIN=ms MK_BIN=mk FIXTURES_DIR="$PWD/../../crates/mnemonic-toolkit/tests/fixtures/wallet_import"` (`:186`).
   - Expected: `[lint] OK`, `[verify-examples] OK (62 transcripts pass)` and `OK anchor-check: …`.
   - Measured: `[lint] OK`, `[verify-examples] OK (62 transcripts pass)`, `OK anchor-check: 9 danglers in current run match baseline (no new, no shrunk)`.
   - The transcript replay is the check that `41-bundle-inheritance-cards.err` and `41-bundle-inheritance-json.out` equal the patched binary's output.
9. `MNEMONIC_BIN="$PWD/target/debug/mnemonic" MD_BIN="$(command -v md)" MS_BIN="$(command -v ms)" MK_BIN="$(command -v mk)" bash ci/doc-flag-lint.test.sh` (`:200`-`:203`, the doc-flag-lint mutation self-test). Expected: `15 passed, 0 failed`. Measured: the same.
10. `examples.yml`: run `EXAMPLES_BIN_DIR="$PWD/target/debug" bash .examples-build/gen.sh > .examples-build/Examples.md || exit 1`, then `git diff --exit-code -- .examples-build/Examples.md`. Expected: no diff, once `examples.diff` is applied and staged (section 6.2 step 3). Measured: the regenerated file's diff from base equals `examples.diff`.
11. Run from `docs/manual`: `make pdf MERMAID_FILTER=skip` and `make html MERMAID_FILTER=skip` (`:210`, `:226`). Expected: exit 0. Measured: both built.

`technical-manual.yml`, `quickstart.yml`, `manual-gui.yml`:

12. Each book's gate:
    - **Technical manual**, from `docs/technical-manual`:
      - Run `make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=true` (`technical-manual.yml:124`-`:129`), `make verify-examples`, `make pdf` and `make html`.
      - Expected and measured: `CSpell: Files checked: 26, Issues found: 0 in 0 files.` (lockstep file 7 adds `preimages`), `[lint] OK`, `[verify-examples] OK (18 transcripts pass)`, and both books built.
      - The lint's symbol-ref-check resolves source citations only in CI's checkout layout (`technical-manual.yml:69`-`:73`). In a plain worktree it checks 0 anchors. The patched `41-bundle-anatomy.md:136` cites `bundle.rs::plain_multisig_threshold`, so the production function keeps that exact name.
      - To check it locally, emulate CI's layout:
        - `W=/scratch/cache/tk-v7-build-symref-ws; mkdir -p "$W"; ln -s "$PWD" "$W/mnemonic-toolkit"`
        - `python3 "$W/mnemonic-toolkit/docs/technical-manual/tests/symbol-ref-check.py" SRC_DIR="$W/mnemonic-toolkit/docs/technical-manual/src"`
        - Expected and measured: `[symbol-ref-check] OK (301 `::` anchors checked, 427 skipped)`.
      - A local `make lint` with all sibling checkouts present also reports two `fresh_chunk_set_id` misses in the mk chapters. Base has them too, they are not in this FIX's files, and CI skips them.
    - **Quickstart**, from `docs/quickstart`, with the pinned CLIs: `make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=mk` (`quickstart.yml:166`) and `make verify-examples`. Expected and measured: `[lint] OK`, `[verify-examples] OK (62 transcripts pass)`.
    - **GUI manual**: under `docs/manual-gui` only the two STOP-1 interim notes change (tutorial prose, outside the byte-compared folders). Its `verify-examples` replays with the pinned toolkit `mnemonic-toolkit-v0.105.1` (`manual-gui.yml:237`-`:257`), not this PR's binary, and its lint byte-compares the tutorial corpus with `mnemonic-gui-v0.63.0`. Expected: green on the PR, as on base. STOP-1 is ruled (ii): the two interim notes are the only change, and `make lint` (`manual-gui.yml:160`) and `make html` (`:156`) are re-run locally.

The FIX rows and the mutation gate:

13. The FIX rows by exact name: section 4's command. Expected: exit 0, exactly **30 tests run: 30 passed**. Quote the line in the GREEN commit message. A short or empty count is a FAIL, whatever the pass count. Measured: `Summary [   0.045s] 30 tests run: 30 passed, 4111 skipped`.
14. Run `bash /scratch/code/shibboleth/refugium-coordinator/coordinator/scripts/mutation-gate.sh <worktree> origin/master` after items 1-5 pass.
    - Production files touched: `cmd/bundle.rs`, `format.rs`, `wallet_import/json_envelope.rs`, `cmd/export_wallet.rs` and `cmd/import_wallet.rs`.
    - Its missed list goes into the review brief.
    - Risk class: A (printed card text, the import-wallet summary, the manual and transcripts) and C (the JSON type change). `bitcoind-differential.yml` names `cmd/bundle.rs` and `cmd/export_wallet.rs` as funds-path inputs, so the coordinator may raise this to B.

Every other workflow that runs on the PR must be green on the PR head:

- `release.yml`'s build matrix and `sibling-pin-check.yml` (every `pull_request`).
- `bitcoind-differential.yml`: path-filtered, and this FIX touches two of its paths, `src/cmd/bundle.rs` and `src/cmd/export_wallet.rs` (`bitcoind-differential.yml:50`, `:55`). It runs the pinned bitcoind v27.0 on CI; it is not a local step.

Two workflows do not run, because none of their paths is touched: `fuzz-smoke.yml` (`parse_descriptor.rs`, `lib.rs`, `fuzz/`, `wc-codec`) and `cross-tool-differential.yml` (`parse_descriptor.rs`). `man-pages.yml`, `changelog-check.yml` and `install-pin-check.yml` run only on tags.

## 8. STOPs (undecided points; the builder does not choose)

The builder does not start until all three are ruled (section 6.2's prerequisite).

### STOP-1. GUI tutorial items of ruling 7 against the pinned GUI

Ruling 7 lists these GUI tutorial items:

- `tut-j3-12` and `tut-j4-16`: the stderr transcripts in section 5, and their PNGs `docs/manual-gui/figures/tutorial/tut-j3-12-engrave-run.png` and `tut-j4-16-engrave-run.png`;
- their prose, `docs/manual-gui/tutorial/40-j3-degrading-vault.md:130-161` and `50-j4-taproot-twin.md:103-128`.

Section 5 also found the j3/j4 restore-feed bundle-json transcripts. `manual-gui.yml:75-78` byte-compares `transcripts/tutorial/` and `figures/tutorial/` with the pinned `mnemonic-gui-v0.63.0` corpus, so any change here fails the GUI book's lint. This is the pattern of FIX-V5's STOP-5, which the architect ruled as a deferral (2026-11-04, V5 option (ii), owned by V5's F4).

Two points are open:

1. **Deferral.** This FIX defers these items the same way. That is a departure from ruling 7 and needs a ruling.
2. **Interim note.** Until the GUI re-pin, the GUI book keeps printing the C-graded false Recovery claim (`# Recovery: any 3 of 11 signing keys + md1 (template card).` on j3's decaying vault and `any 3 of 12` on j4's taproot twin, section 5). The architect left an interim note to this FIX's ruling (`architect-2026-11-04-fix-v5-stops.md:107-109`). Options (none chosen):
   - (i) no interim note; F1 alone tracks the items;
   - (ii) an interim prose note in `40-j3-degrading-vault.md` and `50-j4-taproot-twin.md`, outside the byte-compared directories, with wording and anchors the ruling gives. Section 7 item 12 then re-runs the GUI book's `make lint` and `make html`.

F1 (proposed; section 6.2 step 4 appends it in the form the ruling gives, status `open`):

```
### `manual-gui-tutorial-v7-general-recovery` — at the GUI manual's next re-pin, the J3 and J4 engraving cards take FIX-V7's general-policy Recovery line (repo: **mnemonic-toolkit** `docs/manual-gui` + **mnemonic-gui** `tests/snapshots`; owning phase: **the GUI-manual / mnemonic-gui pin bump to the toolkit release carrying FIX_v7_bundle_recovery**)
- Surfaced: FIX_v7_bundle_recovery STOP-1, ruled by `architect-2026-11-04-fix-v7-stops.md`. Ruling 7's GUI items (`grade-2026-11-03-tk-v5-v7.md`) are met here, not in the fix PR: `manual-gui.yml` byte-compares `docs/manual-gui/transcripts/tutorial/` and `figures/tutorial/` with the pinned mnemonic-gui corpus, which reflects the pinned toolkit release.
- What: mnemonic-gui regenerates the snapshots behind `tut-j3-12-engrave.stderr.txt` and `tut-j4-16-engrave.stderr.txt`, their two PNGs, and the j3/j4 restore-feed bundle-json snapshots against that toolkit release. The re-pin copies them into `docs/manual-gui/` and deletes the two interim notes FIX-V7 put after the engrave stderr blocks of `40-j3-degrading-vault.md` and `50-j4-taproot-twin.md` (architect STOP-1): the regenerated cards print the general line (FIX section 2.2) and no `# Threshold` line.
- Status: `open`.
- Tier: `docs`, `cross-repo`.
```

F1 names the toolkit-side copies; the mnemonic-gui snapshot paths are whatever its tutorial corpus maps to them.

### STOP-2. `bundle --import-json` with `--threshold`

On the `--import-json` path today, `--threshold` sets the card's k (row 24: a plain 2-of-3 envelope with `--threshold 1` prints `# Threshold: 1 of 3`). That is a false recovery claim, and ruling 1 takes k from the descriptor. Descriptor mode already refuses the flag: `ModeViolation { mode: "descriptor", flag: "--threshold", message: DESCRIPTOR_WITH_THRESHOLD }`, exit 2, text `--threshold is meaningful only with a multisig --template; descriptor mode encodes K directly.` (`bundle.rs:195`, `:304-309`). The `--import-json` dispatch (`bundle.rs:277`) runs before that check. Options (none chosen):

- (a) refuse: `ModeViolation`, exit 2, with a mode name and message text the ruling gives (for example, the same `mode: "descriptor"` and message as descriptor mode, or a new `mode: "import-json"` and text);
- (b) ignore the flag: the card takes k from the descriptor (plain) or prints the general line. The hint patch does this.

Row 24 accepts both outcomes until the ruling. Section 6.2 step 2 narrows it to the ruled outcome.

### STOP-3. CHANGELOG entry

`CHANGELOG.md` has an `## [Unreleased]` block (`:9`), and FIX-V5 (its section 8) leaves it unchanged. Options (none chosen):

- (a) no change in this PR; the release prep writes the entry;
- (b) apply `E/fix-v7-changelog.patch`: two `### Fixed` bullets, for the card lines and for JSON `multisig.threshold` `null` (its text is in the evidence dir; the ruling may edit it).

### Note (not a STOP): ruling 3 departs from two existing behaviours

Ruling 3 is binding and is built as written. A pure-key `thresh` is plain and keeps "any k of n". This departs from:

- the triage scratch predicate (`scratch-fixes-v5-v7.patch`), which made every `thresh` general;
- `restore`, which writes a `null` top-level threshold for any `thresh`.

So `bundle --json` writes an integer k for `wsh(thresh(2,pk(K0),s:pk(K1),s:pk(K2)))` (row 25), where `restore` writes `null` for the same descriptor. Aligning `restore` is out of this FIX's scope (ruling 7 does not list it). This FIX files nothing for it, and the reviewer may raise it.

## 9. Coverage map (ruling -> rows)

| ruling | what | rows (section 4) |
|---|---|---|
| 1 | plain shapes keep both lines | 12, 26 (guards) |
| 1 | general shapes: no Threshold line, the exact general Recovery line | 1-11 (triage), 13, 15, 16, 30 |
| 1 | `tr` with a non-NUMS internal key is general | 8 |
| 1 | n == 1 unchanged | 27 |
| 1 | the card's k comes from the descriptor, not a stale envelope or a flag | 23, 24 (STOP-2) |
| 2 | `bundle --json` `multisig.threshold`: k for plain, `null` for general | 17, 28 |
| 2 | second emitter: `import-wallet` JSON `null` and text `threshold=none` | 18, 19, 29 |
| 2 | readers accept `null`: `bundle --import-json`, `export-wallet` | 20, 21 |
| 2 | `export-wallet`'s k-of-n formats still refuse a general policy | 22 |
| 3 | pure-key `thresh` is plain | 25 |
| 3 | any other `thresh` is general | 14 |
| 7 | lockstep files | section 6.1 (8 files), section 7 items 8, 10 and 12; GUI items: STOP-1 |

Rulings 4-6 are FIX-V5's. Every row maps to at least one ruling, and every V7 ruling maps to at least one row.
