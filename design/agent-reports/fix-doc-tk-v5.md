**⚠ Important: V5 and E9 are both graded I (each CONFIRMED). Ruling 6's self-check rule fails on the corpus, with 7 thresh-free shapes at L = mw + 2, so STOP-1 blocks Part A2. E9 (Part B) is ready to build alone.**

# FIX-V5 (+E9) author report

- Deliverable: `design/FIX_v5_wsh_cost.md`, about 1194 lines, with appendices A-G embedded.
- Gate script: `design/agent-reports/fix-doc-tk-v5-evidence/doc-gate.sh`.
- Branch: `fix/v5-wsh-cost`, head `c129176d`, base `ac3ff70a`. Pushed to origin. No PR.
- Branch diff from ac3ff70a: 20 files, +2492/-5, no `src/` file changed. The test files are:
  - `cli_compare_cost_v5_fix.rs` (+112, new)
  - `cli_compare_cost_wrapper_chain.rs` (+276; this is the E9 triage file plus row 26)
  - `cli_compare_cost_wsh_witness_script.rs` (+20/-5: rustfmt of the triage file only, commit 010b6f8b)

## What the FIX decides

**Part A (V5): the wsh witness weight.**
- The cost becomes `witness_size() + varint_len(s) + s - varint_len(n) + varint_len(n+1)` (Appendix A).
- A1b adds the ruling-4 `sh(wsh)` note, in `notes[]` and as a text `note:` line.

**Part A2: the ruling-5/6 cost self-check.**
- Exit 3. R5's text is `compare-cost: cost self-check failed: largest wsh witness {L} WU, max_weight_to_satisfy + 1 = {M1} WU; no figures printed (this is a toolkit bug)`.
- It is written out as Appendix F but held by STOP-1.

**Part B (E9), separable.**
- Rule: an identifier whose next non-whitespace character is `:` is a wrapper chain.
- That one predicate (`is_wrapper_chain_at`) is used in both `collect_abstract_labels` and `replace_whole_word` (`cost/translate.rs` only, Appendix B).
- No lockstep file changes. The `--miniscript` flag row is `41-mnemonic.md:4616` at 12e013bc (the grade's citation) and `:4614` at ac3ff70a. It stays true as written.

## Review of the previous author's uncommitted RED work (keep, fix, drop)

**`cli_compare_cost_v5_fix.rs`: KEEP all of it, unchanged.**
- It holds 4 tests, not the 8 the restart brief says.
- They are §3 rows 27-30.

**The 45 staged lines in `cli_compare_cost_wrapper_chain.rs`: KEEP unchanged.**
- They are one row, `miniscript_path_accepts_label_spelled_like_chain` (§3 row 26, E9 ruling (b)).
- It takes its figures from the `--descriptor` oracle on the same binary and never pins them.

**Self-check rows: none existed.**
- No code was written for them. They are named and held in §3.3:
  - `v5_self_check_failure_exits_3_with_ruling_5_text`
  - `v5_self_check_thresh_free_requires_l_eq_mw_plus_1`
  - `v5_self_check_thresh_allows_rows_up_to_mw_plus_1`
  - `v5_build_descriptor_propagates_cost_self_check`

**Commits.** The test files are in 567dbf3d. 010b6f8b is rustfmt only.

## RED rows (§3), each run against unmodified code

There are 30 rows:
- 1-14: the V5 triage file `cli_compare_cost_wsh_witness_script.rs`.
- 15-25: the E9 triage file (3a2ceced).
- 26: new (E9-b).
- 27-30: new (`cli_compare_cost_v5_fix.rs`).

| tree | row filter (all 30 by exact name) | E9 file alone |
|---|---|---|
| base ac3ff70a | `30 tests run: 11 passed, 19 failed, 4235 skipped` | `12 tests run: 5 passed, 7 failed` |
| base + A + B | `30 passed` | |
| base + B only | | `12 passed` |

- On base, the failing rows are 1-9, 17-22, 26-28 and 30, each for its stated reason.
- 4 of the 5 new rows fail on base (26, 27, 28, 30).
- Row 29 is a guard: it checks the note does not appear on bare `wsh`, so it passes on base. To show it can fail, the note was planted on bare `wsh` and the row failed at `cli_compare_cost_v5_fix.rs:90:9`.

## STOP-1: the corpus result (ruling 6)

**Corpus counts.**
- 70 lines: OK 64, ERRTRANSLATE 4, SKIPPED 2. That gives 63 unique OK shapes.
- Thresh-free: 51 shapes. 44 have L = mw + 1. **7 have L = mw + 2, and all 7 contain `d:`.**
- Thresh: 12 shapes. Their offsets L - (mw + 1) are {-144: 1, -72: 6, -71: 1, +1: 4}.
  - The 4 at +1 break ruling 6's thresh clause too.
  - They contain no `d:`, and I have not traced their cause.

**Core evidence (WU level).**
- For `and_b(pk(A),sdv:older(10))`, Core's witness items are [1, 71, 43]. W = 119, and the `d:` push costs 2 WU.
- So the tool's L = 120 is right, and `max_weight_to_satisfy` is 1 WU low.
- Cause: the vendored `cast_dupif` (`vendor/miniscript/src/miniscript/types/extra_props.rs:544`) adds `+1` at `:550`. `or_i` adds `2 +` at `:763` (and `1 +` at `:756`).
- vB cannot tell the two apart: 283 WU and 284 WU both round to 71 vB.

**Effect on the CLI.** With Appendix F applied, all 7 `d:` shapes exit 3, while the controls without `d:` exit 0.

**Options, none chosen.**
- (a) Ship A without the self-check.
- (b) Widen the rule. Ruling 6 forbids this.
- (c) Patch the vendored `cast_dupif`.
- (d) Base the check on something else.

## STOPs (§10), one line each

1. **STOP-1**: ruling 6's rule fails on the corpus, as above. It blocks A2, and with it the F2 build-impact item (4).
2. **STOP-2**: shipping E9 alone needs a branch name, a file name and a base. B applies to both bases.
3. **STOP-3**: how a `max_weight_to_satisfy` Err maps to an exit (only matters if A2 ships).
4. **STOP-4**: whether build-descriptor prints its human header before the `?` (only matters if A2 ships).
5. **STOP-5**: the GUI tutorial J4 step 20 byte gate against the pinned mnemonic-gui-v0.63.0. Options: (i) a companion PR plus a tag plus a pin bump, (ii) defer, (iii) something else. `manual-gui` phase 11 fails on `tut-j4-20` until this is ruled.
6. **STOP-6**: there is no CLI test seam to reach exit 3 (only matters if A2 ships).
7. **STOP-7**: the order of V5 and V7, and the build base.
   - ac3ff70a carries the FIX-V7 red rows, and it is the tip of `triage/v5-v7`, not on master (12e013bc).
   - Patches A-E apply to master, but the file holding rows 1-14 is not on master.

## Lockstep (§5) and texts for the FIX review (§5.2)

**10 lockstep files, none of them production code.**
- `41-mnemonic.md`.
- The 4 compare-cost transcripts.
- `SPEC_compare_cost_v0_26_0.md`.
- The J4 tutorial page and its transcript.
- `.examples-build/Examples.md`.
- `42-md.md`.

**Files that do not change.**
- The exit-table row is added only with A2, so it is held. Its text is given in §5.
- No build-descriptor transcript carries cost figures: `make audit` passes with only the 4 compare-cost transcripts regenerated.

**9 texts I wrote, because the grade gives no wording.** Each needs a judgement in the FIX review:
1. The `witness_size()` prose in `41-mnemonic.md`.
2. "tr costs `+6`" (was `+24`).
3. The notes-catalog row.
4. SPEC `:145`.
5. SPEC `:180`.
6. SPEC `:210`.
7. SPEC `:280`.
8. J4 `:240`: "`+3` to `+6` vB".
9. The `42-md.md` table rows.

**No existing assertion changes (§4).**

## Gate figures (base + A + B; run once each to a file, then grepped)

**Workspace nextest:** `4227 tests run: 4216 passed, 11 failed, 38 skipped`.
- The 11 failures are the FIX-V7 triage rows, which already fail on base.

**Other checks.**
- clippy (`-D warnings`): clean.
- `cargo +1.95.0 fmt --check`: it differs only in the V7 triage file.
- `make audit`: OK (62 transcripts).
- doc-flag-lint: 15 passed.

**doc-gate.sh: 22 ok lines, PASS.**
- Coverage:
  - each fence equals its evidence file;
  - the §3 table equals `row-names.txt` and `row-alt.txt`;
  - A-E each apply alone and all in order to both ac3ff70a and 12e013bc;
  - F applies alone to ac3ff70a.
- Plants: changing one byte in fence G, or renaming row 7, each gave FAIL with exit 1.
- It does not check: figure correctness, Core agreement, the build or tests of the patched tree, the GUI byte gate, the manual job, runtime output or prose.

**Not run locally**, because these are builder gate items (§7):
- musl;
- regtest;
- the differential job;
- `coordinator/scripts/mutation-gate.sh <worktree> ac3ff70a`.

## Out of scope (§8) and FOLLOWUPS (§9)

**Out of scope (§8).**
- L1/OQ-1 (grade e9 :116-125).
- FIX-V7.
- E10.
- The vendored `cast_dupif` undercount (unless STOP-1 rules (c)).
- The cause of the 4 thresh shapes at +1.
- Release work.

**FOLLOWUPS (§9).** `design/FOLLOWUPS.md` is not edited.
- §9 carries F1 (E9) and F2 (V5) for the builder to append.
- F2 is appended only after STOP-1 and STOP-5 are ruled.

## Hygiene

- The branch diff has no key material, email address or model id (scan: 0 matches).
- The regtest wrapper evidence is quoted by figures only.

Counts: RED rows 30 (new 5, fail-on-base 4); lockstep files 10; STOPs 7
