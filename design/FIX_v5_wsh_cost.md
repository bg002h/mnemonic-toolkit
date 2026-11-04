# FIX-V5: compare-cost wsh figures include the witness script (+ E9 wrapper chains)

Small-fix track (RULES.md "Process"). FIX branch `fix/v5-wsh-cost` (base `ac3ff70a`) stays as is, so its cited SHAs stay reachable. Build on `build/v5-wsh-cost`: `git fetch origin && git switch -c build/v5-wsh-cost origin/master && git cherry-pick a34bd61e 123c258d ac3ff70a && git cherry-pick ac3ff70a..<reviewed fix/v5-wsh-cost head>`. Do not take `d19256e8`.

**Status: AUTHOR DRAFT, not reviewed. Part A (V5) waits on STOP-1. Part B (E9) is complete; per grade e9
ruling (a) it ships alone, with the same text, if Part A waits (STOP-2 decides its branch and file name).**

Sources (coordinator `agent-reports/` unless a path is given):
- Grades: `grade-2026-11-03-tk-v5-v7.md` (V5 table :79-91, rulings 4-7 :120-145) and
  `grade-2026-11-03-tk-e9.md` (rulings (a) :66-88, (b) :90-114; lead L1 :116-122, OQ-1 :125).
- Triage: `tk-triage-v5-v7/design/agent-reports/triage-v5-v7.md` (RED file
  `cli_compare_cost_wsh_witness_script.rs`, on base) and `tk-triage-e9/design/agent-reports/triage-e9.md`
  (cc6a3882; RED file `cli_compare_cost_wrapper_chain.rs` from 3a2ceced).
- Evidence, on this branch: `design/agent-reports/fix-doc-tk-v5-evidence/` (patches, corpus, probes; §0.3).

**How this document was checked.**
- Every code, doc and transcript change is a patch file in the evidence directory, embedded byte for byte in
  the appendices. `bash design/agent-reports/fix-doc-tk-v5-evidence/doc-gate.sh` checks that each appendix
  fence equals its file, that §3's table equals `row-names.txt` and `row-alt.txt`, and that the build patches
  A-E apply to `ac3ff70a` and to `12e013bc`, each alone and all five in order (F alone to `ac3ff70a`). Its last
  line is `doc-gate: PASS` (22 `ok` lines); a planted byte in a fence and a renamed §3 row each made it print
  `doc-gate: FAIL`. It does not check figure correctness, Bitcoin Core agreement, that the patched tree builds,
  the GUI byte gate or runtime behaviour; the runs below do.
- The 30-row RED filter (§3.1) on this branch's head (no production patch): `30 tests run: 11 passed, 19 failed, 4235 skipped`; with Appendices A
  and B applied: `30 tests run: 30 passed, 4235 skipped`.
- The E9 file alone (§3.2) on the same head: `12 tests run: 5 passed, 7 failed, 0 skipped`; with Appendix B only:
  `12 tests run: 12 passed, 0 skipped`.
- The full suite with A and B: `4227 tests run: 4216 passed, 11 failed, 38 skipped`. The 11 failures are the
  FIX-V7 triage rows in `cli_bundle_recovery_threshold_text.rs`, red on base already (STOP-7). `cargo clippy
  --locked --all-targets -- -D warnings` is clean. `cargo +1.95.0 fmt --all -- --check` differs only in that
  V7 file (:50, :164), which is unformatted on base.
- With all five build patches (A, B, C, D, E) applied, and the pinned siblings of `manual.yml` (mk-cli-v0.13.0,
  md-cli v0.20.4 with `cli-compiler`, ms-cli-v0.20.2): `make audit` exits 0 with `[verify-examples] OK (62
  transcripts pass)` and `OK anchor-check: 9 danglers in current run match baseline (no new, no shrunk)`;
  `bash ci/doc-flag-lint.test.sh` reports `15 passed, 0 failed`; `.examples-build/gen.sh` output is byte
  identical to the patched `Examples.md`.
- `docs/manual-gui` `make lint`: phase 11 fails on the `tut-j4-20` byte gate as expected (STOP-5). Phases 4, 9
  and 10 fail on base too (environmental here; CI is the judge).
- The ruling-6 corpus (70 shapes, `corpus-rows-d-final.tsv`, counted by `count-plus-one.py`) is STOP-1's
  evidence. The rows were produced by a scratch harness (`corpus-scratch.patch`), never by production code.
- Log figures above are from `/scratch/cache/tk-fix-v5-logs/` on the author's machine (not committed):
  `rows-filter-base.log` 5182c2dc1a61, `rows-filter-committed-patches.log` 8f0b559a7c47, `e9-base.log`
  0c35cba1a1bb, `e9-standalone.log` ba212bf5bc65, `nextest-v5a-e9.log` f08c3037cb63, `clippy-v5a-e9.log`
  3dd47d5c84af, `make-audit-patched-pinmk.log` dd23b0b80fb1, `doc-flag-lint-patched.log` fd26173a8a70,
  `manual-gui-lint-patched.log` 1f2e8a9a8456, `manual-gui-lint-base.log` 48d3b51993c5 (first 12 hex of
  sha256).
- All runs used `PATH=$HOME/.cargo/bin:$PATH` (cargo 1.85.0, the `rust-toolchain.toml` pin), `--locked`, and no
  `--release`.
- Not run locally. These are the builder's gate items (§7): the musl release build and the other CI-only jobs,
  the regtest and differential jobs, and the mutation gate.

## 0. Inputs (binding)

### 0.1 Rulings

| id | source | binds |
|---|---|---|
| V5-1..V5-7 | grade v5-v7 :79-91 | each Important: the wsh column omits the witnessScript; V5-7 is the Δ column |
| R4 | grade v5-v7 :120-125 | the `sh(wsh)` note: its text (:123), in `notes[]` and as a text `note:` line, one notes-catalog row; SPEC :69 stays |
| R5 | grade v5-v7 :127-131 | new `CostSelfCheck` error, exit 3 (`Parse` stays 2); build-descriptor propagates it through its existing `?` (`build_descriptor.rs:477-486`, `:512-521`), exit 3 in human and `--json` views, no degraded preview; one manual exit-table row. Moved to F3 (STOP-1 ruling). |
| R6 | grade v5-v7 :133-139 | the self-check rule: thresh-free L == mw + 1, with a thresh every row <= mw + 1; STOP if a thresh-free shape breaks it; do not widen; keep the Core-measured rows. Moved to F3 (STOP-1 ruling). |
| R7 | grade v5-v7 :141-145 | the lockstep list (§5) |
| E9-a | grade e9 :66-88 | E9 is a separable section of this FIX; it ships alone if Part A STOPs; it adds no lockstep files |
| E9-b | grade e9 :90-114 | the wrapper-chain predicate and RED row `miniscript_path_accepts_label_spelled_like_chain` |

### 0.2 The previous author's RED work (restart brief: keep, fix or drop each)

| item | on disk | verdict |
|---|---|---|
| `tests/cli_compare_cost_v5_fix.rs` | 4 tests (the restart brief says 8; the file has 4) | KEEP all 4 unchanged. R4 rows 1-3 pin the note text exactly; row 4 covers the Δ sign (V5-7). |
| 45 lines in `tests/cli_compare_cost_wrapper_chain.rs` | 1 test, `miniscript_path_accepts_label_spelled_like_chain` | KEEP unchanged. It is E9-b's row as written: 3 spellings, `--json` exit 0, one condition, label `sdv + older(blocks)`, figures equal to the `--descriptor` oracle (never pinned), and the plain Condition column. |
| self-check rows for R5/R6 | no code exists on disk or in any commit | Named in §3.3 with their behaviour only; written by the builder once STOP-1 is ruled. |

The previous author's state is committed on this branch: `567dbf3d` (the two test files; `wrapper_chain.rs` is
the triage file from 3a2ceced plus the 45 lines) and `010b6f8b` (rustfmt only, of the base triage file
`cli_compare_cost_wsh_witness_script.rs`; no assertion changed).

### 0.3 Evidence files

`design/agent-reports/fix-doc-tk-v5-evidence/`:

| file | what |
|---|---|
| `fix-v5a-only.patch` | Appendix A: the witness-weight fix and the R4 note (production) |
| `fix-e9-production.patch` | Appendix B: the E9 predicate (production) |
| `fix-v5a-docs.patch` | Appendix C: SPEC, toolkit manual and GUI tutorial prose |
| `transcript-diffs.txt` | Appendix D: the transcripts whose figures move |
| `examples.diff` | Appendix E: `.examples-build/Examples.md` |
| `fix-v5-production.patch` | Appendix F: the HELD self-check (A + R5/R6). Do not apply (STOP-1). |
| `sh-wsh-pk.txt` | Appendix G: expected text output for `sh(wsh(pk(KA)))` after A |
| `row-names.txt`, `row-alt.txt` | the 30 RED row names, and the same names as one regex alternation |
| `corpus-rows-d-final.tsv`, `count-plus-one.py`, `corpus-scratch.patch` | STOP-1 corpus, its counter and the scratch harness that produced it |
| `d-wrapper-probe.txt` | Appendix F's build on the seven `d:` shapes, five `d:` shapes it refuses with exit 2, and the controls: exit code and stderr |
| `d-wrapper-v5a.txt` | Appendix A's wsh and tr vB for the four Core-spent shapes and `pk(A)` |
| `core-d-wrapper-figures.txt` | Core's regtest spends of the four shapes: witness item sizes, witness bytes, input WU and vB, Core's tx weight (keys shown as `A`) |
| `doc-gate.sh` | the fence and apply check above |

## 1. Part A: V5 spec delta

### A1. The wsh column includes the witnessScript (V5-1..V5-7)

**Defect.** `build_and_plan` (`cost/enumerate.rs:264`) costs a wsh condition as `plan.witness_size()`. For
`wsh(M)` that template omits the witnessScript item (`varint(s) + s` bytes, `s` = script length) and counts one
stack item too few, so every wsh figure is low (`wsh(pk)`: 60 vB printed, Bitcoin Core spends it at 273 WU =
69 vB) and the Δ column is wrong in size and, for large `multi`, in sign.

**Rule.** For `Descriptor::Wsh`, the wsh figure in WU is
`witness_size() + varint_len(s) + s - varint_len(n) + varint_len(n + 1)`, where `s` is the witnessScript length
in bytes, `n` the template's item count, and `varint_len(k) = VarInt(k as u64).size()`. Every other descriptor
keeps `witness_size()`. This equals upstream rust-miniscript #1046 (`rust-miniscript-fork` adf262b3
`src/plan.rs:258-277`). Code: Appendix A, `cost/enumerate.rs` hunk (`witness_weight_wu`).

**What the holder sees.** Every `wsh vB`, `wsh sats`, `Δ vB` and `Δ sats` figure, in text and `--json`
(`wsh_vbytes`, `delta_vbytes`, and the sats fields), from `compare-cost` and from the build-descriptor cost
preview. No flag, label, refusal or exit code changes. The `tr` column does not change. Moved figures in printed
docs: §4.2.

### A1b. The `sh(wsh)` note (R4)

When the input is `sh(wsh(M))` (the `ShInner::Wsh` arm of `cost/strip.rs`), compare-cost adds this note, text
exact (grade :123):

```text
input had a p2sh wrapper, sh(wsh(M)); each wsh figure is for bare wsh(M): add 35 vB per input (the scriptSig redeem-script push) and subtract 35 vB from each Δ
```

- It is an entry of `--json` `notes[]` and a text line `note: <text>`. Strip notes come before the cost notes
  (`cost/mod.rs:166-169`), so it is the first note. The rows pin presence and exact text, not position.
- No other input gets it: `wsh(M)`, `tr(..)` and `--miniscript` do not (row
  `v5_sh_wsh_note_absent_for_wsh_tr_and_miniscript`).
- The `wsh` column stays the bare `wsh(M)` figure (SPEC :69 stays). 35 vB = 140 WU is the scriptSig push of the
  34-byte P2WSH redeem script, measured: `sh(wsh(pk))` 413 WU against `wsh(pk)` 273, `sh(wsh(multi))` 556
  against 416.
- Code: Appendix A, `cost/strip.rs` hunk (`pub const SH_WSH_NOTE`).
- Expected text output for `mnemonic compare-cost --descriptor 'sh(wsh(pk(02998512205ec6a5cdb77d5b4f7de63c560d1e846162612ee178c49e7b6cc44fb9)))'`,
  exit 0: Appendix G.

### A1c. Docs and transcripts (R7)

The prose, catalog row and transcripts are Appendices C, D and E; the file list is §5. The author wrote these
texts where the grade gives no wording; each is listed in §5.2 for the FIX review.

### A2. The cost self-check (R5, R6): not in this FIX

Architect 2026-11-04 ruled STOP-1 option (a). The self-check moves, with rulings 5 and 6 unchanged, to FOLLOWUP `compare-cost-wsh-cost-self-check` (§9 F3), owned by the next cycle that moves miniscript. The corpus misses come from the pinned `max_weight_to_satisfy`. It counts `d:`'s `1` push as 1 WU (`extra_props.rs:550`; fixed upstream by rust-miniscript #1015). It also folds k+1 satisfactions in `thresh` (`:902`; fixed by #1040). Appendix F is F3's reference code; do not apply it here.

## 2. Part B: E9 spec delta (separable)

**Defect** (grade e9 :20-23). `collect_abstract_labels` (`cost/translate.rs:156-241`) records the identifier
after every `(` or `,` as an abstract label unless it is in `KEYWORDS`, which lists single letters only, and
`replace_whole_word` then swaps it for a dummy key. A multi-letter wrapper chain (`ln:`, `sdv:`, `snl:`, `tv:`,
`sj:`) is mangled, and `--miniscript` exits 2 with `error: compare-cost: parse error: unknown wrapper «X:»`, where
`X` is a hex digit of the dummy key.

**Rule** (E9-b, one predicate in both places): *an identifier whose next non-whitespace character is `:` is a
wrapper chain.* `collect_abstract_labels` does not record it, and `replace_whole_word` does not replace an
occurrence whose next non-whitespace character is `:`. Code: Appendix B (`cost/translate.rs` only:
`fn is_wrapper_chain_at`, its use in both functions).

**What the holder sees.** `--miniscript` (and miniscript on stdin, `cmd/compare_cost.rs:74`, `:143`) accepts
every valid wrapper chain and gives the same figures as `--descriptor 'wsh(..)'` on the same shape. A label
spelled like a chain is still a label where it is not followed by `:` (`and_b(pk(sdv),sdv:older(10))` gives
one condition labelled `sdv + older(blocks)`). An invalid chain is still refused with exit 2
(`or_i(pk(A),sdv:pk(B))`). No flag, printed text or exit code changes for any other input. No lockstep file
changes, and `41-mnemonic.md:4616` stays true as written (E9-a). That line number is the grade's, at
`12e013bc`: it is the `--miniscript` flag row, which is `41-mnemonic.md:4614` at `ac3ff70a`.

## 3. RED rows

30 rows in three files. Test names are bare function names (the toolkit has no `tests/all.rs`), so the
exact-name filter is `test(/^<row>$/)`. The list is `row-names.txt` in the evidence directory, and
`row-alt.txt` is the same list joined with `|`; `doc-gate.sh` checks that both equal this table.

| # | file (`crates/mnemonic-toolkit/tests/`) | row | origin | production code at `ac3ff70a` (assert site: message) | after A+B |
|---|---|---|---|---|---|
| 1 | `cli_compare_cost_wsh_witness_script.rs` | `v5_wsh_pk_vbytes_match_core` | V5 triage | FAIL :80: `Core spends wsh(pk) as 273 WU = 69 vB; got [("A", 60, 75, 15)]` | PASS |
| 2 | same | `v5_wsh_or_d_vbytes_match_core` | V5 triage | FAIL :91: `Core: both or_d branches are 78 vB (+1 for the tool's 73-byte dummy sig); got [("A", 60, 84, 24), ("B", 60, 85, 25)]` | PASS |
| 3 | same | `v5_wsh_multi_2of3_vbytes_match_core` | V5 triage | FAIL :103: `Core 104 vB (+1 for the tool's 73-byte dummy sig); got [("A + B", 78, 109, 31), ...]` | PASS |
| 4 | same | `v5_wsh_multi_3of5_vbytes_match_core` | V5 triage | FAIL :115: `Core 139 vB (+<=2 for 73-byte dummy sigs); got [("A + B + C", 97, 143, 46), ...]` | PASS |
| 5 | same | `v5_wsh_and_v_vbytes_match_core` | V5 triage | FAIL :127: `Core 95 vB (+1 for the tool's 72/73-byte dummy sigs); got [("A + B", 78, 100, 22)]` | PASS |
| 6 | same | `v5_wsh_thresh_2of3_vbytes_match_core` | V5 triage | FAIL :139: `Core 106 vB (+1 for dummy sigs); got [("A + B", 78, 110, 32), ...]` | PASS |
| 7 | same | `v5_descriptor_wsh_pk_vbytes_match_core` | V5 triage | FAIL :150: `--descriptor wsh(pk(..)) must equal the Core spend; got [("key[0]", 60, 75, 15)]` | PASS |
| 8 | same | `v5_p2wsh_input_never_cheaper_than_p2wpkh_input` | V5 triage | FAIL :167: `pk(A): wsh vB 60 is below the 68 vB P2WPKH input` | PASS |
| 9 | same | `v5_delta_matches_core` | V5 triage | FAIL :180: `Δ vB for pk(A) must be 75 - 69 = 6; got [("A", 60, 75, 15)]` | PASS |
| 10 | same | `v5_tr_pk_vbytes_match_core` | V5 triage | PASS (control: tr is already right) | PASS |
| 11 | same | `v5_tr_or_d_vbytes_match_core` | V5 triage | PASS (control) | PASS |
| 12 | same | `v5_tr_multi_a_vbytes_match_core` | V5 triage | PASS (control) | PASS |
| 13 | same | `v5_descriptor_tr_nums_pk_vbytes_match_core` | V5 triage | PASS (control) | PASS |
| 14 | same | `v5_sh_wsh_descriptor_reports_the_wsh_column` | V5 triage | PASS (control: `sh(wsh(M))` costs as `wsh(M)`) | PASS |
| 15 | `cli_compare_cost_wrapper_chain.rs` | `oracle_rust_miniscript_parses_every_shape_in_both_contexts` | E9 triage 3a2ceced | PASS (oracle) | PASS |
| 16 | same | `oracle_descriptor_path_accepts_every_shape` | E9 triage 3a2ceced | PASS (oracle) | PASS |
| 17 | same | `miniscript_path_accepts_ln_chain` | E9 triage 3a2ceced | FAIL :114: ``compare-cost --miniscript 'or_i(pk(A),ln:pk(B))'` must accept valid miniscript (the --descriptor path and rust-miniscript do); got exit 2 stderr="error: compare-cost: parse error: unknown wrapper «9:»\n"`` | PASS |
| 18 | same | `miniscript_path_accepts_tv_chain` | E9 triage 3a2ceced | FAIL :114: `'or_i(pk(A),tv:pk(B))'`, exit 2, `unknown wrapper «0:»` | PASS |
| 19 | same | `miniscript_path_accepts_sdv_chain` | E9 triage 3a2ceced | FAIL :114: `'and_b(pk(A),sdv:older(10))'`, exit 2, `unknown wrapper «4:»` | PASS |
| 20 | same | `miniscript_path_accepts_snl_chain` | E9 triage 3a2ceced | FAIL :114: `'and_b(pk(A),snl:older(10))'`, exit 2, `unknown wrapper «b:»` | PASS |
| 21 | same | `miniscript_path_accepts_sj_chain` | E9 triage 3a2ceced | FAIL :114: `'and_b(pk(A),sj:pk(B))'`, exit 2, `unknown wrapper «f:»` | PASS |
| 22 | same | `miniscript_path_accepts_ln_chain_with_concrete_keys` | E9 triage 3a2ceced | FAIL :182: `concrete-key chain refused: stderr="error: compare-cost: parse error: unknown wrapper «9:»\n"` | PASS |
| 23 | same | `control_single_letter_wrappers_still_parse` | E9 triage 3a2ceced | PASS (control) | PASS |
| 24 | same | `control_real_labels_still_substituted` | E9 triage 3a2ceced | PASS (control) | PASS |
| 25 | same | `control_invalid_chain_still_refused` | E9 triage 3a2ceced | PASS (control: `or_i(pk(A),sdv:pk(B))` exits 2) | PASS |
| 26 | same | `miniscript_path_accepts_label_spelled_like_chain` | **new** (E9-b) | FAIL :249: ``compare-cost --miniscript 'and_b(pk(sdv),sdv:older(10))'` must accept a label spelled like a chain; got exit 2 stderr="error: compare-cost: parse error: unknown wrapper «4:»\n"`` | PASS |
| 27 | `cli_compare_cost_v5_fix.rs` | `v5_sh_wsh_note_in_json_notes` | **new** (R4) | FAIL :62: `notes[] must contain the ruling-4 sh(wsh) note exactly; got [...]` (the two existing notes only) | PASS |
| 28 | same | `v5_sh_wsh_note_in_text_note_lines` | **new** (R4) | FAIL :76: `text output must carry the line "note: input had a p2sh wrapper, ..."` | PASS |
| 29 | same | `v5_sh_wsh_note_absent_for_wsh_tr_and_miniscript` | **new** (R4) | PASS (guard). Plant: emitting the note for every input makes it fail at `cli_compare_cost_v5_fix.rs:90:9` (`plant-absent-for.log`) | PASS |
| 30 | same | `v5_delta_negative_for_multi_7_of_10` | **new** (V5-7) | FAIL :103: `7-of-10: tr must be cheaper than wsh (Δ < 0) once the witnessScript is counted; got [81, 81, ...]` | PASS |

Rows 1-14 are on base already (rustfmt'd by 010b6f8b, formatting only). Rows 15-25 are 3a2ceced's file,
carried in by 567dbf3d; row 26 and rows 27-30 are the previous author's uncommitted work, kept unchanged (§0.2)
and committed in 567dbf3d. New rows: 5 (26-30); new rows that fail on base: 4 (26, 27, 28, 30); row 29 is a
guard that passes on base and was proved by the plant above.

The E9 rows never pin a figure: each compares `--miniscript` with the `--descriptor 'wsh(..)'` oracle on the
same binary, so Part A cannot move them (E9-a).

### 3.1 The FIX filter (all 30 rows)

From the worktree root:

```text
PATH=$HOME/.cargo/bin:$PATH cargo nextest run --locked --workspace --no-fail-fast -E "test(/^($(cat design/agent-reports/fix-doc-tk-v5-evidence/row-alt.txt))\$/)"
```

- On base (this branch's head, no production patch): exit 100, summary line
  `30 tests run: 11 passed, 19 failed, 4235 skipped`; the 19 failures are exactly rows 1-9, 17-22, 26-28 and 30.
- With Appendices A and B applied: exit 0, `30 tests run: 30 passed, 4235 skipped`.
- A matched count other than 30 is a FAIL, whatever the pass count.

### 3.2 Part B alone (E9 ships alone)

```text
PATH=$HOME/.cargo/bin:$PATH cargo nextest run --locked -p mnemonic-toolkit --test cli_compare_cost_wrapper_chain --no-fail-fast
```

- On this branch's head (no production patch): `12 tests run: 5 passed, 7 failed, 0 skipped` (rows 17-22 and
  26 fail as in the table). The file does not exist on `ac3ff70a`; this branch adds it.
- With Appendix B only: `12 tests run: 12 passed, 0 skipped`.
- If E9 ships alone on another base (STOP-2), re-run this on that base before quoting it; a GREEN expires when
  its inputs move (grade e9 :76-78).

### 3.3 Rows for F3 (not written in this FIX)

These rows move to F3 unchanged; no code exists for them. No code exists for them, on disk or in any commit, and Appendix F
contains no `#[test]` function. They are written by the builder once STOP-1 (and STOP-6, the test seam) are
ruled, in a new file `cli_compare_cost_v5_self_check.rs` or as `#[cfg(test)]` unit tests in `cost/` if STOP-6
rules a private seam. The ruling may rename or drop them; the FIX review then re-checks §3.

| row | behaviour pinned |
|---|---|
| `v5_self_check_failure_exits_3_with_ruling_5_text` | a forced self-check failure prints R5's template (§1 A2) with L and mw + 1 filled in, exactly, on stderr after `error: `, prints no figures, and exits 3 |
| `v5_self_check_thresh_free_requires_l_eq_mw_plus_1` | for a thresh-free shape the check passes when L == mw + 1 and fails (exit 3) when L != mw + 1 |
| `v5_self_check_thresh_allows_rows_up_to_mw_plus_1` | for a shape with `thresh`, every row <= mw + 1 passes; a row > mw + 1 fails (exit 3) |
| `v5_build_descriptor_propagates_cost_self_check` | `build-descriptor` propagates the same error unchanged through its `?` (`cost::run_compare_cost` calls at `cmd/build_descriptor.rs:477` and `:512`) and exits 3 in both the human and `--json` views, with no degraded preview (R5); STOP-4 decides what the human view may already have printed |

## 4. Tests and transcripts that change

No existing test assertion changes. With Appendices A and B applied, the full suite fails only the 11 FIX-V7
rows that are red on base already (STOP-7). Three test files carry wsh figures
(`cli_compare_cost.rs`, `cli_compare_cost_wsh_witness_script.rs`, `cli_compare_cost_wrapper_chain.rs`), and none
of their other assertions moves.

### 4.1 Test files on this branch (no production code)

| file (`crates/mnemonic-toolkit/tests/`) | change | commit |
|---|---|---|
| `cli_compare_cost_wsh_witness_script.rs` | on base; rustfmt only (formatting, no assertion or name change) | 010b6f8b |
| `cli_compare_cost_wrapper_chain.rs` | new on this branch: 3a2ceced's 11 rows (15-25) as-is, plus row 26 | 567dbf3d |
| `cli_compare_cost_v5_fix.rs` | new: rows 27-30 | 567dbf3d |

The toolkit has no `tests/all.rs` aggregator, so each file is its own test binary and no list file changes.

### 4.2 Transcripts and printed figures that move (Appendices D and E)

Every moved wsh figure goes up, because the witnessScript is now counted; every Δ (tr minus wsh) goes down by
the same amount. tr figures do not move.

| file | before (wsh, tr, Δ in vB; sats where printed) | after |
|---|---|---|
| `docs/manual/transcripts/41-compare-cost-orb.out` | A and B: `60 \| 84 \| +24 \| 1500 \| 2100 \| +600` | `78 \| 84 \| +6 \| 1950 \| 2100 \| +150` |
| `docs/manual/transcripts/41-compare-cost-ord.out` | A: `60 \| 85 \| +25`; B: `60 \| 86 \| +26` | A: `79 \| 85 \| +6`; B: `80 \| 86 \| +6` |
| `docs/manual/transcripts/41-compare-cost-tr-ik.out` | `60 \| 75 \| +15 \| 1500 \| 1875 \| +375` | `69 \| 75 \| +6 \| 1725 \| 1875 \| +150` |
| `docs/manual/transcripts/41-compare-cost-json.out` | `wsh_vbytes` 60, `delta_vbytes` 15, `wsh_sats` 60, `delta_sats` 15 | 69, 6, 69, 6 |
| `docs/manual-gui/transcripts/tutorial/tut-j4-20-compare-cost.stdout.txt` (moved to F4 (STOP-5)) | A + B: wsh 79 (Δ +68); C, D, E: 61 (+71); F + preimage(h0): 69 (+70) | 144 (+3); 126 (+6); 133 (+6) |
| `.examples-build/Examples.md` @@-1204 | the same J4 rows as `tut-j4-20` | the same as `tut-j4-20` |
| `.examples-build/Examples.md` @@-1224 | `68 \| 90 \| +22` | `84 \| 90 \| +6` |

The two J4 PNGs (`docs/manual-gui/figures/tutorial/tut-j4-20-compare-cost-form.png` and `-run.png`) show the
compare-cost panel; whether they are regenerated, and against which GUI binary, moved to F4 (STOP-5).

No `build-descriptor` transcript carries cost figures. The `build-descriptor --json` `cost` field embeds the
compare-cost JSON (`41-mnemonic.md:4864` at `ac3ff70a`), but no transcript shows it, so ruling 7's "build-descriptor
transcripts" item changes no file. Measured: `make audit` passes with only the four compare-cost transcripts
regenerated (128 transcript files checked).

## 5. Lockstep files (ruling 7)

Eight files change; none is production code. Appendix C holds the prose hunks, D the transcripts, E the examples.

| # | file | ruling 7 item, or why it is beyond ruling 7 | appendix, hunk |
|---|---|---|---|
| 1 | `docs/manual/src/40-cli-reference/41-mnemonic.md` | `:4596-4602` (the `witness_size()` prose), the notes catalog; plus the worked example at `:4660` (beyond R7: its `+24` moves) | C @@-4593, @@-4657, @@-4749 |
| 2 | `docs/manual/transcripts/41-compare-cost-orb.out` | compare-cost transcripts | D |
| 3 | `docs/manual/transcripts/41-compare-cost-ord.out` | compare-cost transcripts | D |
| 4 | `docs/manual/transcripts/41-compare-cost-tr-ik.out` | compare-cost transcripts | D |
| 5 | `docs/manual/transcripts/41-compare-cost-json.out` | compare-cost transcripts | D |
| 6 | `design/SPEC_compare_cost_v0_26_0.md` | `:180-183`, `:213` (lines `:180`, `:210` here); plus `:145` (the same formula in the step-3 sketch) and `:280` (the new note in the notes list), beyond R7 | C @@-145, @@-180, @@-210, @@-280 |
| 7 | `docs/manual-gui/tutorial/50-j4-taproot-twin.md` | `:241` (J4 step 20 prose; the changed line is `:240` at `ac3ff70a`); moved to F4 (STOP-5) | C @@-238 |
| 8 | `docs/manual-gui/transcripts/tutorial/tut-j4-20-compare-cost.stdout.txt` | its transcript; moved to F4 (STOP-5) | D |
| 9 | `.examples-build/Examples.md` | beyond R7: `examples.yml` regenerates it and fails on any diff | E |
| 10 | `docs/manual/src/40-cli-reference/42-md.md` | beyond R7: its `md compile` recipe prints a compare-cost table at `:471-472` | C @@-469 |

Ruling 7 items that change no file:
- The exit-code table (`41-mnemonic.md:4753-4767` at `ac3ff70a`) moves with A2 to F3 (STOP-1 ruling).
  The row A2 would add, not in Appendix C: `| wsh cost self-check failed against \`max_weight_to_satisfy\` (a
  toolkit bug; no figures printed) | \`3\` |`.
- The build-descriptor transcripts: none carries cost figures (§4.2).

### 5.1 Lockstep texts the builder does not write

Every changed line is in Appendices C, D and E. The builder applies them with `git apply` and regenerates the
transcripts with the repo's own targets (§6); a regenerated transcript that differs from Appendix D is a FAIL.

### 5.2 Texts the author wrote (the grade gives no wording); for the FIX review

1. `41-mnemonic.md` @@-4593: "API. `Plan::witness_size()` counts the witness items, their length prefixes and
   the stack-count varint, and for `tr` the tapscript and control block. For `wsh` the plan's template omits the
   witnessScript, so `compare-cost` adds it as one more witness item (the script plus its length prefix) and
   re-encodes the stack-count varint. Per-input"
2. `41-mnemonic.md` @@-4657: "tr costs `+6`" (was `+24`). The sentence's reason ("because the tr witness carries
   an extra 33-byte control block") is unchanged; the FIX review judges whether it
   still reads true at `+6`.
3. `41-mnemonic.md` notes catalog row: "| `input had a p2sh wrapper, sh(wsh(M)); …` | `--descriptor sh(wsh(M))`.
   The `wsh` column is costed for bare `wsh(M)`; the note gives the +35 vB per-input correction. |"
4. `SPEC_compare_cost_v0_26_0.md:145`: `wsh_witness_bytes: witness_weight_wu(&wsh_desc, &wp)`.
5. `SPEC_compare_cost_v0_26_0.md:180`: "- tapscript + control block (tr only; for wsh the template omits the
   witnessScript, which §4 adds as one more stack item)". SPEC §4 "Cost computation" starts at `:208`.
6. `SPEC_compare_cost_v0_26_0.md:210`: `let wsh_witness_bytes = witness_weight_wu(&wsh_desc, &wsh_plan); //
   witness_size() + varint(s) + s - varint(n) + varint(n + 1); s = witnessScript bytes, n = template items (the
   template omits the witnessScript)`.
7. `SPEC_compare_cost_v0_26_0.md:280` (new list item): the R4 note text verbatim, prefixed "sh(wsh) advisory:".
8. `50-j4-taproot-twin.md:240`: "(`+3` to `+6` vB)" (was `+68` to `+71`). The rest of the paragraph ("a Taproot
   script-path spend reveals a leaf plus its control block … slightly dearer fallbacks") is unchanged and still
   true at these figures.
9. `42-md.md:471-472`: the two table rows, `79 | 85 | +6` and `80 | 86 | +6`, as the binary prints them.

## 6. Build steps (in this order)

FIX branch `fix/v5-wsh-cost` (base `ac3ff70a`) stays as is, so its cited SHAs stay reachable. Build on `build/v5-wsh-cost`: `git fetch origin && git switch -c build/v5-wsh-cost origin/master && git cherry-pick a34bd61e 123c258d ac3ff70a && git cherry-pick ac3ff70a..<reviewed fix/v5-wsh-cost head>`. Do not take `d19256e8`. Use `export
PATH=$HOME/.cargo/bin:$PATH`; `cargo --version` must print `cargo 1.85.0 (d73d2caf9 2024-12-31)`. Never pass
`--release`. Send every run to a file and grep it; never run a suite twice to count. `E` below is
`design/agent-reports/fix-doc-tk-v5-evidence`.

1. RED. Run §3.1 on the branch head before any patch. Expect exit 100 and a summary line of
   `30 tests run: 11 passed, 19 failed, …`; the 19 failures are the rows §3 marks as failing on base. Quote the
   line in the RED evidence.
2. Part B (E9), its own commit:
   - `git apply $E/fix-e9-production.patch` (Appendix B; touches `cost/translate.rs` only).
   - Run §3.2. Expect `12 tests run: 12 passed, 0 skipped`.
   - Append F1 (§9) to `design/FOLLOWUPS.md`.
   - Commit as the E9 GREEN commit; its message quotes the §3.2 summary line.
3. Part A (V5). STOP-1 is ruled option (a) and STOP-5 option (ii) (architect 2026-11-04). Under any other ruling,
   the ruling's text replaces this step and the FIX review re-checks §3, §4 and §5 first.
   - `git apply $E/fix-v5a-only.patch` (Appendix A), then `git apply $E/fix-v5a-docs.patch` (Appendix C), then
     `git apply $E/transcript-diffs.txt` (Appendix D), then `git apply $E/examples.diff` (Appendix E).
   - Revert Appendix D's tutorial hunk (`git checkout -- docs/manual-gui/transcripts/tutorial/tut-j4-20-compare-cost.stdout.txt`) and Appendix C's `50-j4-taproot-twin.md` hunk (`git checkout -- docs/manual-gui/tutorial/50-j4-taproot-twin.md`); append F4 (§9).
   - `cargo build --bin mnemonic`.
   - Run §3.1. Expect exit 0 and `30 tests run: 30 passed, …`.
   - Check the transcripts against the patched binary: §7 items 8 and 10. A replayed transcript that differs from
     Appendix D, or a regenerated `Examples.md` that differs from Appendix E's result, is a FAIL; do not
     hand-edit a transcript to match.
   - Append F2 (§9) to `design/FOLLOWUPS.md`.
   - Commit as the V5 GREEN commit; its message quotes the §3.1 summary line.
4. Run the whole gate (§7) on the final head, then the mutation gate (§7 item 14).
5. Do not change the version, `CHANGELOG.md` or any release file (§8).

## 7. Gate (full toolkit CI; the builder runs all of it on the final head)

Capture each run once to a file and grep it. "Expected" is the result on the final head with Parts A and B
applied. STOP-7 decides the base.

`rust.yml`:

1. `cargo +1.95.0 fmt --all -- --check` (`rust.yml:118`). Expected: exit 0; no diff may name a file this FIX touches.
2. `cargo metadata --locked --format-version 1 > /dev/null` (`:148`) and `cargo build --locked --tests
   --workspace` (`:171`). Expected: exit 0.
3. `cargo nextest run --locked --workspace --no-fail-fast` (`:185`). Expected: 0 failed.
4. `cargo test --locked --workspace --doc` (`:190`). Expected: exit 0 (measured at the head:
   `Doc-tests mnemonic_toolkit` and `Doc-tests wc_codec`, each `test result: ok. 0 passed; 0 failed; …`).
5. `cargo clippy --locked --all-targets -- -D warnings` (`:395`). Expected: exit 0, no warnings.
6. The rest of `rust.yml`, locally where it runs here, else as green CI on the PR:
   - the mlock rows `cargo test -p mnemonic-toolkit --lib mlock::tests::g2_1 -- --include-ignored` and the
     same for `g2_3` and `g2_4` (`:205`, `:218`, `:234`);
   - `cargo test -p mnemonic-toolkit --test mlock_g6_invariant -- --include-ignored` (`:524`);
   - `cargo build --bin mnemonic` (`:263`), then the six `sh scripts/install-*.test.sh` steps
     (`:268`-`:301`);
   - `bash ci/docs-only.test.sh` (`:375`), `bash ci/code-docs-check.test.sh` and `bash ci/code-docs-check.sh`
     (`:380`-`:381`);
   - CI only: the release `g2_3` row (`:335`), miri (`:365`), the lib cross-platform check (`:397`) and the
     musl build and test (`:526`).
   Expected: each exits 0.

`manual.yml` (the manual job and its mutation self-tests), with the pinned `mk` (`mk-cli-v0.13.0`), `md`
(`descriptor-mnemonic-md-cli-v0.20.4`, `--features cli-compiler`) and `ms` (`ms-cli-v0.20.2`) first in PATH:

7. `bash ci/doc-gate-guard.test.sh` (`manual.yml:61`). Expected: exit 0.
8. From `docs/manual`: `make audit MNEMONIC_BIN="$PWD/../../target/debug/mnemonic" MD_BIN=md MS_BIN=ms MK_BIN=mk
   FIXTURES_DIR="$PWD/../../crates/mnemonic-toolkit/tests/fixtures/wallet_import"` (`:186`). Expected:
   `[lint] OK`, `[verify-examples] OK (62 transcripts pass)` and `OK anchor-check: …` (measured with A, B, C
   and D applied and the pinned `mk`). The transcript replay is the check that the four `41-compare-cost-*.out`
   files equal the patched binary's output.
9. `MNEMONIC_BIN="$PWD/target/debug/mnemonic" MD_BIN="$(command -v md)" MS_BIN="$(command -v ms)"
   MK_BIN="$(command -v mk)" bash ci/doc-flag-lint.test.sh` (`:200`-`:203`, the doc-flag-lint mutation
   self-test). Expected: `15 passed, 0 failed` (measured with A, B, C and D applied).
10. `examples.yml`: `EXAMPLES_BIN_DIR="$PWD/target/debug" bash .examples-build/gen.sh >
    .examples-build/Examples.md || exit 1`, then `git diff --exit-code -- .examples-build/Examples.md`
    (`examples.yml:116`-`:125`). Expected: no diff after Appendix E is applied.
11. From `docs/manual`: `make pdf MERMAID_FILTER=skip` and `make html MERMAID_FILTER=skip` (`:210`, `:226`).
    Expected: exit 0.

`manual-gui.yml`, `quickstart.yml`, `technical-manual.yml`:

12. From `docs/manual-gui`: `make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=true`
    (`manual-gui.yml:160`), `make html` (`:156`), `make verify-examples` (`:252`) and `make
    verify-examples-gui` (`:349`); plus `bash ci/doc-gate-guard.test.sh` (`:64`). The lint's
    `verify-tutorial-transcripts` phase compares the tutorial transcripts byte for byte with the pinned
    `mnemonic-gui-v0.63.0` corpus (`manual-gui.yml:75-78`); with Appendix D's tutorial hunk applied it fails
    (measured: `[lint] FAIL: verify-tutorial-transcripts: byte drift vs the pinned corpus for:
    tut-j4-20-compare-cost.stdout.txt`), and STOP-5 decides what passes here. `quickstart.yml` and
    `technical-manual.yml` run on CI; neither book mentions `compare-cost` (grep empty), so no figure there
    moves. Expected: green, phase 11 included (the tutorial transcript is unchanged).

The FIX rows and the mutation gate:

13. The FIX rows by exact name: §3.1's command. Expected: exit 0, exactly **30 tests run: 30 passed**. If E9
    ships alone (STOP-2), §3.2's command: exactly **12 tests run: 12 passed**. Quote the line in the GREEN
    commit message. A short or empty count is a FAIL, whatever the pass count.
14. `bash /scratch/code/shibboleth/refugium-coordinator/coordinator/scripts/mutation-gate.sh <worktree> origin/master`
    after items 1-5 pass. Production files touched: `cost/enumerate.rs`, `cost/strip.rs` (Part A) and
    `cost/translate.rs` (Part B). Its missed list goes into the review brief. Risk class: A (a new printed note,
    the manual and transcripts) and C (cost code); the coordinator may raise it.

Every other workflow that runs on the PR (`release.yml`'s build matrix and `sibling-pin-check.yml` run on
every `pull_request`) must be green on the PR head. `fuzz-smoke.yml`, `bitcoind-differential.yml` and
`cross-tool-differential.yml` are path-filtered and none of their paths is touched here (`cost/` is not in
any list), so they do not run; `man-pages.yml`, `changelog-check.yml` and `install-pin-check.yml` run only on
tags. None of these is a local step of this gate.

## 8. Out of scope

- **L1 / OQ-1** (grade e9 :116-125; L1 :118-122, OQ-1 :125). Lowercase labels that spell a single-letter wrapper
  (`or_b(pk(a),s:pk(b))`, `multi(2,a,b,c)`) are refused with `compare-cost: parse error: key too short`. Whether
  to accept them is an undecided product point that needs triage and a ruling. Appendix B leaves `KEYWORDS`
  (`translate.rs:213-222`) unchanged, so this FIX does not move L1 either way (not measured after B).
- **FIX-V7** (`cmd/bundle.rs` recovery-threshold text). It is a separate FIX by ruling 7 (grade v5-v7 :141-145);
  only its effect on this FIX's base is here (STOP-7).
- **E10**, mr's thresh backstop `largest <= BASE + mw + 73` (grade v5-v7 :74, :136, :152). It is mr code, not
  toolkit code, and has its own triage.
- **The vendored `cast_dupif` undercount** (`vendor/miniscript/src/miniscript/types/extra_props.rs:550`, against
  `or_i` at :763). It is an upstream rust-miniscript defect. It is fixed upstream by #1015; F3.
- The pinned `threshold` folds k+1 satisfactions (`extra_props.rs:902`), fixed upstream by #1040 (architect 2026-11-04).
- **Release work**: version, `CHANGELOG.md`, tag, install line (§6 step 5).

## 9. FOLLOWUPS (the builder appends these to `design/FOLLOWUPS.md` in the GREEN commit)

F1 (Part B, E9; appended by §6 step 2, closed by the GREEN commit that turns rows 15-26 green):

```
### `compare-cost-wrapper-chain-label-scan` — `compare-cost --miniscript` refuses valid multi-letter wrapper chains (repo: **mnemonic-toolkit**; owning phase: **FIX_v5_wsh_cost build, Part B**)
- Surfaced: triage E9 (`triage/e9-wrapper-chain` cc6a3882; RED file 3a2ceced); grade `grade-2026-11-03-tk-e9.md`: CONFIRMED, grade **I**, rulings (a)-(b).
- Build impact (1): `cost/translate.rs` treats an identifier whose next non-whitespace character is `:` as a wrapper chain, in both `collect_abstract_labels` and `replace_whole_word` (FIX §2, Appendix B). FIX §3 rows 15-26 (`cli_compare_cost_wrapper_chain.rs`, 12 tests) pass.
- Status: `owed-to-build`, closed by this build's GREEN commit (write its sha here).
- Tier: `toolkit`.
```

F2 (Part A, V5; appended by §6 step 3, only after STOP-1 and STOP-5 are ruled):

```
### `compare-cost-wsh-witness-script-omitted` — `compare-cost` wsh figures omit the witnessScript and its length (repo: **mnemonic-toolkit**; owning phase: **FIX_v5_wsh_cost build, Part A**)
- Surfaced: triage V5 (`triage/v5-v7` ac3ff70a); grade `grade-2026-11-03-tk-v5-v7.md`: V5-1..V5-7 each **I**, rulings 4-7.
- Build impact (1): the wsh witness weight counts the witnessScript and its length varint (`witness_weight_wu`, FIX §1 A1, Appendix A). FIX §3 rows 1-14 and 30 pass.
- Build impact (2): the ruling-4 `sh(wsh)` note, in `notes[]` and as a text `note:` line (FIX §1 A1b). FIX §3 rows 27-29 pass.
- Build impact (3): the lockstep files of FIX §5. The regenerated transcripts equal Appendix D, and `.examples-build/Examples.md` equals Appendix E's result. The GUI tutorial J4 step 20 (FIX §5 rows 7-8) moved to `manual-gui-j4-20-compare-cost-v5-figures` (architect 2026-11-04 STOP-5).
- Build impact (4): none in this build; the ruling-5/6 self-check moved to `compare-cost-wsh-cost-self-check` (architect 2026-11-04 STOP-1).
- Status: `owed-to-build`, closed by this build's GREEN commit (write its sha here).
- Tier: `toolkit`.
```

### `compare-cost-wsh-cost-self-check` — compare-cost's ruling-5/6 wsh cost self-check, held until the pinned `max_weight_to_satisfy` is exact for `d:` and `thresh` (repo: **mnemonic-toolkit**; owning phase: **the next cycle that moves miniscript**)
- Surfaced: FIX_v5_wsh_cost STOP-1, ruled option (a) by `architect-2026-11-04-fix-v5-stops.md`. Rulings 5 and 6 of `grade-2026-11-03-tk-v5-v7.md` stand unchanged; only their build moved here.
- Why held: at pin ff4732e5 `max_weight_to_satisfy` counts `d:`'s `1` push as 1 WU (`extra_props.rs:550`; fixed by rust-miniscript #1015) and folds k+1 satisfactions in `thresh` (`:902`; fixed by #1040), so 11 of the FIX corpus's 63 shapes (7 thresh-free with `d:`, 4 thresh) break ruling 6 although the printed figure is right.
- Action, in the cycle that moves miniscript: (1) if the new rev carries #1046, reduce `witness_weight_wu` (`cost/enumerate.rs`) to `witness_size()`, since #1046 counts the witness script (FIX rows 1-9 fail on a double count); (2) re-run the FIX corpus (`corpus-scratch.patch`, `count-plus-one.py` in the FIX evidence dir) on the new rev; (3) if the rev carries #1015 and #1040, every thresh-free shape has L == mw + 1 and every thresh row is <= mw + 1, rule FIX STOP-3, STOP-4 and STOP-6, then build rulings 5-6 from FIX Appendix F with the FIX §3.3 rows; otherwise STOP.
- Status: `open`.
- Tier: `toolkit` (upstream-dependent).

### `manual-gui-j4-20-compare-cost-v5-figures` — at the GUI manual's next re-pin, J4 step 20's compare-cost transcript, figures and prose take FIX-V5's wsh figures (repo: **mnemonic-toolkit** `docs/manual-gui` + **mnemonic-gui** `tests/snapshots`; owning phase: **the GUI-manual / mnemonic-gui pin bump to the toolkit release carrying FIX_v5_wsh_cost Part A**)
- Surfaced: FIX_v5_wsh_cost STOP-5, ruled by `architect-2026-11-04-fix-v5-stops.md`. Ruling 7's GUI item (`grade-2026-11-03-tk-v5-v7.md`) is met here, not in the fix PR: `manual-gui.yml` byte-compares `docs/manual-gui/transcripts/tutorial/` and `figures/tutorial/` with the pinned mnemonic-gui corpus, which reflects the pinned toolkit release.
- What: mnemonic-gui regenerates `tests/snapshots/tutorial/tut-j4-20-compare-cost.stdout.txt` and the two J4 step-20 PNGs against that toolkit release; the re-pin copies them into `docs/manual-gui/` and changes `50-j4-taproot-twin.md:241` "(`+68` to `+71` vB)" to the range the regenerated transcript prints (FIX Appendix D predicts 144 (+3), 126 (+6), 133 (+6), i.e. "(`+3` to `+6` vB)").
- Status: `open`.
- Tier: `docs`, `cross-repo`.

## 10. STOPs (undecided points; the builder does not choose)

### STOP-1. Ruling 6's rule fails on the corpus (blocks Part A)

Ruling 6 (grade v5-v7 :133-139): "If the FIX's corpus finds a thresh-free shape with L != mw + 1, STOP and
report: do not widen the rule." The corpus finds seven such shapes.

- **Corpus.** `corpus-rows-d-final.tsv` has 70 lines and 63 unique planned shapes, produced by the scratch
  harness `corpus-scratch.patch` on Appendix F's build. `python3 -I count-plus-one.py corpus-rows-d-final.tsv`
  prints:

  ```text
  lines 70 by status {'OK': 64, 'ERRTRANSLATE': 4, 'SKIPPED': 2}; unique OK shapes 63
  thresh-free 51: L == mw+1 44, L != mw+1 7
    free  L=123 mw=121 +1  and_b(pk(@A),sdv:after(500000))
    free  L=120 mw=118 +1  and_b(pk(@A),sdv:older(10))
    free  L=122 mw=120 +1  and_b(pk(@A),sdv:older(144))
    free  L=120 mw=118 +1  and_v(v:pk(@A),dv:older(144))
    free  L=158 mw=156 +1  andor(pk(@A),dv:older(144),pk(@B))
    free  L=159 mw=157 +1  or_d(pk(@A),and_v(v:pk(@B),dv:older(144)))
    free  L=159 mw=157 +1  or_i(pk(@A),and_v(v:pk(@B),dv:older(144)))
  thresh 12: offsets L-(mw+1) {-144: 1, -72: 6, -71: 1, 1: 4}
    thresh L=236 mw=234 +1  thresh(2,pk(@A),s:pk(@B),sln:older(32768))
    thresh L=237 mw=235 +1  thresh(2,pk(@A),s:pk(@B),snl:after(1700000000))
    thresh L=345 mw=343 +1  thresh(3,pk(@A),s:pk(@B),s:pk(@C),snl:older(12960))
    thresh L=455 mw=453 +1  thresh(4,pk(@A),s:pk(@B),s:pk(@C),s:pk(@D),sln:older(1000))
  ```

  Each "+1" is L - (mw + 1). All seven thresh-free misses contain a `d:` wrapper. The four `thresh` rows at +1
  also break ruling 6's thresh clause (every row <= mw + 1). They contain no `d:`, and their cause is not traced.
- **What the CLI does.** On Appendix F's build, all seven `d:` shapes exit 3 with R5's text, for example
  `and_b(pk(A),sdv:older(10))`: `largest wsh witness 120 WU, max_weight_to_satisfy + 1 = 119 WU`. Controls
  without `d:` (`s:pk`, `snl:older(10)`, `a:older(10)`) exit 0 (`d-wrapper-probe.txt`). So Appendix F would
  refuse valid, common inputs with "this is a toolkit bug".
- **Which side is right (the `d:` rows).** The tool's L is right and the vendored `max_weight_to_satisfy` is
  1 WU low. Whole vB cannot decide this: Core's regtest spends of `wsh(and_b(pk,sdv:older(10)))`,
  `wsh(and_v(v:pk,dv:older(144)))`, `wsh(and_v(v:pk,older(144)))` and `wsh(and_b(pk,snl:older(10)))` are 71, 71,
  70 and 71 vB, Appendix A prints the same (`d-wrapper-v5a.txt`), but an input of 283 WU and one of 284 WU both
  round up to 71 vB. Core's witness items decide it (`core-d-wrapper-figures.txt`, a key-free extract of the
  regtest log). For `wsh(and_b(pk,sdv:older(10)))` Core's items are 1, 71 and 43 bytes, and its weight agrees
  with W = 119 = 1 (item count) + (1 + 1) + (1 + 71) + (1 + 43). So the `1` that `d:` pushes costs 2 WU. Core's
  signature is 71 bytes; the vendored size model assumes 73 with its length prefix
  (`vendor/miniscript/src/miniscript/types/extra_props.rs:91`), one more than Core's 1 + 71. With that
  signature the same witness is 1 + 2 + 73 + 44 = 120 WU, which is the corpus's L for the shape (row
  `and_b(pk(@A),sdv:older(10))`: L 120, mw 118). L = mw + 1 would need 119, i.e. a 1-WU `d:` push, which is
  not how a witness item serialises. The control without `d:`, `wsh(and_v(v:pk,older(144)))`, has items 71
  and 39 (W = 113 = 1 + 72 + 40), so the signature alone accounts for L - W = 1. In the `snl:` shape the
  `l:` push is empty (item 0 bytes, 1 WU) and the corpus has L = mw + 1 (120, 119). The cause is
  `cast_dupif` (`extra_props.rs:544`), which adds `max_witness_stack_size + 1` (:550) for the `1` push;
  `or_i` adds `2 +` (:763) for its `1` push and `1 +` (:756) for its empty push.
- **The rows that stay either way.** Ruling 6 keeps the Core-measured rows (rows 1-14), since only they catch a
  double count on a `thresh` (E7).
- **Options (none chosen):**
  - (a) Ship Part A as Appendix A (the formula and the R4 note) plus §5's lockstep, without the self-check.
    Rulings 5 and 6 are then unmet in this FIX. The ruling also decides whether A2 is dropped or refiled, and to
    which owner. §6 step 3 is written for this path.
  - (b) Widen the rule, for example L == mw + 1 + (the number of `d:` wrappers). Ruling 6 forbids this. It would
    need a new architect ruling, and it would still not explain the four `thresh` rows.
  - (c) Correct the undercount: patch the vendored `cast_dupif` to count 2 (`vendor/`, risk class B, and
    `vendor-freshness`), or carry an upstream fix, then re-run the corpus. This does not touch the four `thresh`
    rows, which have no `d:`.
  - (d) Use another basis for the check (a new rule text, for example a check that does not use
    `max_weight_to_satisfy`).
- **Needed:** a ruling naming the option. For (a), the ruling also names A2's fate and owner. For (b)-(d), it
  gives the new rule text, and the four `thresh` +1 rows are traced before any self-check ships.

### STOP-2. E9 alone: branch, file name and base

If Part A waits on STOP-1, Part B ships alone with the same text (grade e9 ruling (a), :68-69). Undecided:

- the branch name;
- the FIX file name: this document with Part A marked held, or a new file carrying §2, §3.2, Appendix B and F1
  unchanged;
- the base: `origin/master` (`12e013bc`), or `ac3ff70a`, which is only on `origin/triage/v5-v7` and carries
  FIX-V7's red rows (STOP-7).

Measured facts: Appendix B applies to both bases (`doc-gate.sh`). `translate.rs` is identical on both (grade e9
:76). `cli_compare_cost_wrapper_chain.rs` is on neither base. On this branch it is `567dbf3d`'s file, which is
`3a2ceced`'s file plus the 45-line row (`git diff --stat 3a2ceced 567dbf3d -- crates/mnemonic-toolkit/tests/cli_compare_cost_wrapper_chain.rs` shows 45 insertions). On any base,
the steps are: add that file (`git show 567dbf3d:crates/mnemonic-toolkit/tests/cli_compare_cost_wrapper_chain.rs`),
run §3.2 to see `12 tests run: 5 passed, 7 failed`, apply Appendix B, run §3.2 to see `12 passed`, and append F1.

### STOP-3. A `max_weight_to_satisfy` error (contingent on A2)

Appendix F :25-27 maps an `Err` from `max_weight_to_satisfy()` to `CompareCostError::NoSatisfyingConditions`
(exit 3, `compare-cost: no satisfying conditions for this miniscript`, `cost/mod.rs:82-84`, `:98`). That mapping
is a placeholder. Rulings 5 and 6 do not say what the holder sees when the bound itself cannot be computed. The
choices include `NoSatisfyingConditions`, `CostSelfCheck` (which has no figure to print for M1) or a new
variant with its own text. Moot under STOP-1 option (a).

### STOP-4. build-descriptor's human header before the `?` (contingent on A2)

Ruling 5 says build-descriptor propagates the error "unchanged through its existing `?`
(`build_descriptor.rs:477-486, 512-521`), exit 3, in both the human and `--json` views, with no degraded
preview." In the human view the preview header is printed before that `?` runs, so a self-check failure would
leave the header on stdout above the exit-3 error. Undecided: whether the header counts as a "degraded preview"
(moving the cost call before the header changes build-descriptor's print order), or whether the header may stay.
Moot under STOP-1 option (a).

### STOP-5. GUI tutorial J4 step 20 against the pinned GUI

Ruling 7 puts the GUI tutorial J4 step 20 prose (`50-j4-taproot-twin.md:241`) and its transcript in lockstep.
`docs/manual-gui/pinned-upstream.toml:107-108` pins `[mnemonic-gui] tag = "mnemonic-gui-v0.63.0"`, and
`manual-gui.yml` phases 10 and 11 (:66-80; the byte compare is :75-78) compare the tutorial figures and
transcripts byte for byte against that pinned clone's snapshots. Appendix D's `tut-j4-20-compare-cost.stdout.txt`
hunk therefore fails phase 11 (§7 item 12) until the GUI side moves. Options (none chosen):

- (i) a mnemonic-gui companion PR that regenerates the J4 step-20 snapshots and PNGs, a new mnemonic-gui tag, and
  a pin bump in `pinned-upstream.toml` (the tag needs Brian's go-ahead);
- (ii) defer the GUI tutorial lockstep: revert Appendix D's tutorial hunk and the `50-j4-taproot-twin.md` hunk
  of Appendix C, and file the deferral with an owner (a departure from ruling 7);
- (iii) another route the ruling names.

### STOP-6. No CLI test seam for exit 3 (contingent on A2)

Once STOP-1 is ruled so that every corpus shape passes the check, no valid input reaches exit 3, so no CLI row
can show R5's text. §3.3's first two rows need a seam. The choices are a `#[cfg(test)]` unit test that calls the
check with forged L and mw (no production surface), or a fault-injection hook in production (a new surface).
Moot under STOP-1 option (a).

### STOP-7. V5 and V7 order, and the build base

`ac3ff70a` is the tip of `origin/triage/v5-v7`, not of master (`12e013bc`; master is not its ancestor; the four
commits are `a34bd61e`, `d19256e8`, `123c258d`, `ac3ff70a`). It carries FIX-V7's triage file
`cli_bundle_recovery_threshold_text.rs`: 11 rows red on base, and the file is not rustfmt-clean (:50, :164).
On `ac3ff70a`, `cargo nextest run` and `cargo fmt --check` therefore fail whatever this FIX does (§7). Appendices
A-E apply to `origin/master` too, but rows 1-14's file `cli_compare_cost_wsh_witness_script.rs` is not on master.
Undecided: whether this FIX builds after FIX-V7's GREEN is in `ac3ff70a`, builds on `ac3ff70a` with V7's lines as
the only allowed failures (§7's rule), or rebases onto master with the triage file cherry-picked; and which PR
merges first.

## Appendices (embedded byte for byte from the evidence directory; `doc-gate.sh` checks each fence)

### Appendix A. Part A production code (`fix-v5a-only.patch`)

Applies to `ac3ff70a` and `12e013bc`. Held by STOP-1 (a) until the ruling.

<!-- evidence: fix-v5a-only.patch -->
~~~~diff
diff --git i/crates/mnemonic-toolkit/src/cost/enumerate.rs w/crates/mnemonic-toolkit/src/cost/enumerate.rs
index 4d5fe524..e0628767 100644
--- i/crates/mnemonic-toolkit/src/cost/enumerate.rs
+++ w/crates/mnemonic-toolkit/src/cost/enumerate.rs
@@ -16,10 +16,11 @@
 
 use std::collections::BTreeSet;
 
+use miniscript::bitcoin::consensus::encode::VarInt;
 use miniscript::bitcoin::hashes::{hash160, ripemd160, sha256};
 use miniscript::bitcoin::{absolute, relative};
 use miniscript::descriptor::{DefiniteDescriptorKey, DescriptorPublicKey};
-use miniscript::plan::Assets;
+use miniscript::plan::{Assets, Plan};
 use miniscript::{Descriptor, ForEachKey, Miniscript, Segwitv0};
 
 use super::translate::{pubkey_to_label_segv0, Translated};
@@ -264,11 +265,36 @@ fn build_and_plan(
         }
     }
     match desc.clone().plan(&a) {
-        Ok(plan) => Some(plan.witness_size()),
+        Ok(plan) => Some(witness_weight_wu(desc, &plan)),
         Err(_) => None,
     }
 }
 
+/// Witness weight (WU) of one planned spend. For `wsh(M)` the plan's
+/// template omits the witnessScript, so add it as one more stack element
+/// (the expression of rust-miniscript PR #1046): `varint(script) + script`,
+/// and re-encode the stack-count varint for `t + 1` elements. `tr` templates
+/// already carry the leaf script and control block.
+fn witness_weight_wu(
+    desc: &Descriptor<DefiniteDescriptorKey>,
+    plan: &Plan<DefiniteDescriptorKey>,
+) -> usize {
+    let template_wu = plan.witness_size();
+    match desc {
+        Descriptor::Wsh(wsh) => {
+            let script_len = wsh.inner_script().len();
+            let elems = plan.witness_template().len();
+            template_wu + varint_len(script_len) + script_len - varint_len(elems)
+                + varint_len(elems + 1)
+        }
+        _ => template_wu,
+    }
+}
+
+fn varint_len(n: usize) -> usize {
+    VarInt(n as u64).size()
+}
+
 fn is_minimal(
     wsh_desc: &Descriptor<DefiniteDescriptorKey>,
     tr_desc: &Descriptor<DefiniteDescriptorKey>,
diff --git i/crates/mnemonic-toolkit/src/cost/strip.rs w/crates/mnemonic-toolkit/src/cost/strip.rs
index 3efd1a6c..c4c0abf6 100644
--- i/crates/mnemonic-toolkit/src/cost/strip.rs
+++ w/crates/mnemonic-toolkit/src/cost/strip.rs
@@ -13,6 +13,9 @@ use super::translate::Translated;
 use super::CompareCostError;
 use super::NUMS_XONLY_HEX;
 
+/// Grade ruling 4: `notes[]` entry for a `sh(wsh(M))` input.
+pub const SH_WSH_NOTE: &str = "input had a p2sh wrapper, sh(wsh(M)); each wsh figure is for bare wsh(M): add 35 vB per input (the scriptSig redeem-script push) and subtract 35 vB from each Δ";
+
 /// Strip the wrapper from a user-supplied descriptor string; return a
 /// `Translated` ready for the enumerator (plus any advisory note). Concrete
 /// hex pubkeys are preserved; no abstract labels are detected (those only
@@ -68,7 +71,7 @@ pub fn translate_descriptor(input: &str) -> Result<(Translated, Option<String>),
         Descriptor::Sh(sh) => match sh.into_inner() {
             ShInner::Wsh(wsh) => {
                 let m_segv0 = wsh.into_inner();
-                translated_from_segv0(input, m_segv0).map(|t| (t, None))
+                translated_from_segv0(input, m_segv0).map(|t| (t, Some(SH_WSH_NOTE.to_string())))
             }
             ShInner::Wpkh(_) | ShInner::Ms(_) => Err(CompareCostError::UnsupportedWrapper(
                 "sh(non-wsh)".to_string(),
~~~~

### Appendix B. Part B production code (`fix-e9-production.patch`)

Applies to `ac3ff70a` and `12e013bc`, alone or after A.

<!-- evidence: fix-e9-production.patch -->
~~~~diff
diff --git i/crates/mnemonic-toolkit/src/cost/translate.rs w/crates/mnemonic-toolkit/src/cost/translate.rs
index 1bcf9e44..fdb5863a 100644
--- i/crates/mnemonic-toolkit/src/cost/translate.rs
+++ w/crates/mnemonic-toolkit/src/cost/translate.rs
@@ -175,6 +175,10 @@ fn collect_abstract_labels(input: &str) -> Vec<String> {
             if buf.is_empty() {
                 continue;
             }
+            let rest = chars.peek().map_or("", |&(j, _)| &input[j..]);
+            if is_wrapper_chain_at(rest) {
+                continue;
+            }
             let is_hex_pubkey =
                 (buf.len() == 64 || buf.len() == 66) && buf.chars().all(|c| c.is_ascii_hexdigit());
             let is_numeric = buf.chars().all(|c| c.is_ascii_digit());
@@ -265,7 +269,7 @@ fn replace_whole_word(haystack: &str, needle: &str, replacement: &str) -> String
             let before_ok = i == 0 || !is_ident_char(bytes[i - 1]);
             let after_ok =
                 i + needle_b.len() == bytes.len() || !is_ident_char(bytes[i + needle_b.len()]);
-            if before_ok && after_ok {
+            if before_ok && after_ok && !is_wrapper_chain_at(&haystack[i + needle_b.len()..]) {
                 result.push_str(replacement);
                 i += needle_b.len();
                 continue;
@@ -277,6 +281,14 @@ fn replace_whole_word(haystack: &str, needle: &str, replacement: &str) -> String
     result
 }
 
+/// Grade ruling (b) (E9): an identifier whose next non-whitespace character
+/// is `:` is a wrapper chain (`ln:`, `sdv :`), never a key label. BIP-379 puts
+/// wrapper letters only immediately before `:`, and no BIP-380 key form
+/// contains `:`. `rest` is the input after the identifier.
+fn is_wrapper_chain_at(rest: &str) -> bool {
+    rest.chars().find(|c| !c.is_whitespace()) == Some(':')
+}
+
 fn is_ident_char(b: u8) -> bool {
     b.is_ascii_alphanumeric() || b == b'_'
 }
~~~~

### Appendix C. Part A docs: manual and lockstep text (`fix-v5a-docs.patch`)

<!-- evidence: fix-v5a-docs.patch -->
~~~~diff
diff --git i/design/SPEC_compare_cost_v0_26_0.md w/design/SPEC_compare_cost_v0_26_0.md
index 0ff207ba..c4c4b754 100644
--- i/design/SPEC_compare_cost_v0_26_0.md
+++ w/design/SPEC_compare_cost_v0_26_0.md
@@ -145,7 +145,7 @@ let mut rows = Vec::new();
     let tr_plan  = tr_desc.clone().plan(&provider);
     if let (Ok(wp), Ok(tp)) = (wsh_plan, tr_plan) {
         if is_minimal(&cfg, &assets, &wsh_desc, &tr_desc) {    // shrinkage check
-            rows.push(Row { label: cfg.label(), wsh_witness_bytes: wp.witness_size(), tr_witness_bytes: tp.witness_size() });
+            rows.push(Row { label: cfg.label(), wsh_witness_bytes: witness_weight_wu(&wsh_desc, &wp), tr_witness_bytes: tp.witness_size() });
             if rows.len() >= hard_cap { break 'outer; }
         }
     }
@@ -180,7 +180,7 @@ Step 1's precheck means: with default `hard_cap=4096`, the user must have `|sign
 rust-miniscript's `Plan` returns a *non-malleable, optimal* witness for a given `AssetProvider`'s assets. `Plan::witness_size() -> usize` is the byte count of the witness data for that satisfaction in that context (Segwitv0 or Tap), including:
 - per-stack-item length prefixes
 - stack-count varint
-- scriptCode (wsh: serialized script on the stack) or tapscript + control block (tr)
+- tapscript + control block (tr only; for wsh the template omits the witnessScript, which §4 adds as one more stack item)
 - script-context-specific signature sizes (73 for ECDSA, 64 for Schnorr SIGHASH_DEFAULT)
 
 (`Plan::satisfaction_weight()` returns weight units; per §4, we use `witness_size()` which is byte-count — equivalent to weight-units for witness data since each witness byte contributes 1 wu.)
@@ -210,7 +210,7 @@ Each minimal configuration is labeled compactly:
 For each minimal configuration produced by §3:
 
 ```
-let wsh_witness_bytes = wsh_plan.witness_size();           // includes scriptCode (the witnessScript)
+let wsh_witness_bytes = witness_weight_wu(&wsh_desc, &wsh_plan); // witness_size() + varint(s) + s - varint(n) + varint(n + 1); s = witnessScript bytes, n = template items (the template omits the witnessScript)
 let tr_witness_bytes  = tr_plan.witness_size();            // includes tapscript + control block
 
 // 41 = 36-byte outpoint + 1-byte scriptSig-length-0 + 4-byte sequence;
@@ -280,6 +280,7 @@ Header lines: echo the extracted miniscript, the wsh-vs-tr wrapper comparison st
 - feerate-zero advisory (§4)
 - soft-cap-reached advisory: `"enumeration capped at 256 conditions; raise --max-conditions to see more"`
 - concrete-keys advisory: `"input had concrete keys; cost is identical to the abstract case"`
+- sh(wsh) advisory: `"input had a p2sh wrapper, sh(wsh(M)); each wsh figure is for bare wsh(M): add 35 vB per input (the scriptSig redeem-script push) and subtract 35 vB from each Δ"`
 
 The JSON envelope sets exit 0 on success even when notes are non-empty.
 
diff --git i/docs/manual-gui/tutorial/50-j4-taproot-twin.md w/docs/manual-gui/tutorial/50-j4-taproot-twin.md
index 9838bb37..3603ca3b 100644
--- i/docs/manual-gui/tutorial/50-j4-taproot-twin.md
+++ w/docs/manual-gui/tutorial/50-j4-taproot-twin.md
@@ -238,7 +238,7 @@ spending condition)** and paste the folded miniscript into
 The panel (standard output only — this subcommand prints no standard
 error) tabulates, per spend condition, the witness size in virtual
 bytes and satoshis for `wsh` against `tr`, and the delta. Every
-condition costs **more** on the script-path (`+68` to `+71` vB) because
+condition costs **more** on the script-path (`+3` to `+6` vB) because
 a Taproot script-path spend reveals a leaf plus its control block — the
 trade Taproot makes is a *much* cheaper, private cooperative key-path
 against slightly dearer fallbacks. The closing notes flag that
diff --git i/docs/manual/src/40-cli-reference/41-mnemonic.md w/docs/manual/src/40-cli-reference/41-mnemonic.md
index 0b7e0358..93c31835 100644
--- i/docs/manual/src/40-cli-reference/41-mnemonic.md
+++ w/docs/manual/src/40-cli-reference/41-mnemonic.md
@@ -4593,9 +4593,11 @@ See [Consensus-masked relative timelocks](#consensus-masked-relative-timelocks)
 for the non-blocking `older()` advisory this command emits on intake.
 
 Cost is computed via rust-miniscript v13's `Descriptor::plan(...)`
-API; `Plan::witness_size()` returns the full witness-data byte count
-(witness items + length prefixes + stack-count varint + the
-serialized witnessScript or tapscript + control block). Per-input
+API. `Plan::witness_size()` counts the witness items, their length
+prefixes and the stack-count varint, and for `tr` the tapscript and
+control block. For `wsh` the plan's template omits the witnessScript,
+so `compare-cost` adds it as one more witness item (the script plus its
+length prefix) and re-encodes the stack-count varint. Per-input
 costs include the constant 41 vB SegWit-input overhead (36-byte
 outpoint + 1-byte scriptSig-length-zero + 4-byte sequence) so the
 absolute `wsh vB` and `tr vB` numbers match what Sparrow / Bitcoin
@@ -4657,7 +4659,7 @@ Stdout:
 PLACEHOLDER — generated from transcripts/41-compare-cost-orb.out at build
 ```
 
-Either A or B can sign alone; both pay the same cost. tr costs `+24`
+Either A or B can sign alone; both pay the same cost. tr costs `+6`
 vB more per spend than wsh because the tr witness carries an extra
 33-byte control block.
 
@@ -4749,6 +4751,7 @@ in plaintext output) carry advisory text. Known entries:
 | `input had concrete keys; cost is identical to the abstract case` | input contained no abstract labels. |
 | `input contains hash-preimage fragments; …` | input has at least one `sha256` / `hash256` / `ripemd160` / `hash160` leaf. |
 | `input had a non-NUMS internal key IK; …` | (v0.28.0) `--descriptor tr(IK, {M})` with `IK ≠ NUMS`. The advisory carries the IK hex; the JSON envelope's `keypath_spend` field carries the keypath-spend cost (`{ internal_key_xonly_hex, vbytes: 58, sats }`); plaintext output adds a `Keypath-spend (via IK …): 58 vB \| <SATS> sats` annotation line below the per-condition table. |
+| `input had a p2sh wrapper, sh(wsh(M)); …` | `--descriptor sh(wsh(M))`. The `wsh` column is costed for bare `wsh(M)`; the note gives the +35 vB per-input correction. |
 
 ### Exit codes
 
diff --git i/docs/manual/src/40-cli-reference/42-md.md w/docs/manual/src/40-cli-reference/42-md.md
index 84d0e69a..c23632c0 100644
--- i/docs/manual/src/40-cli-reference/42-md.md
+++ w/docs/manual/src/40-cli-reference/42-md.md
@@ -469,8 +469,8 @@ md encode --from-policy 'or(pk(@0),and(pk(@1),older(144)))' \
 #    with abstract labels (@0→A, @1→B); cost is key-agnostic:
 mnemonic compare-cost --miniscript 'or_d(pk(A),and_v(v:pk(B),older(144)))'
 # Condition         | wsh vB | tr vB |  Δ vB
-# A                 |     60 |    85 |   +25
-# B + older(blocks) |     60 |    86 |   +26
+# A                 |     79 |    85 |    +6
+# B + older(blocks) |     80 |    86 |    +6
 ```
 
 To bring the wallet up watch-only or emit importable artifacts, compile with
~~~~

### Appendix D. Part A transcripts: expected regeneration diff (`transcript-diffs.txt`)

The builder regenerates the transcripts with the repo's tool; the result must equal this diff.

<!-- evidence: transcript-diffs.txt -->
~~~~diff
--- a/docs/manual/transcripts/41-compare-cost-orb.out
+++ b/docs/manual/transcripts/41-compare-cost-orb.out
@@ -4,7 +4,7 @@
 
 Condition | wsh vB | tr vB |  Δ vB | wsh sats | tr sats | Δ sats
 ----------+--------+-------+-------+----------+---------+-------
-A         |     60 |    84 |   +24 |     1500 |    2100 |   +600
-B         |     60 |    84 |   +24 |     1500 |    2100 |   +600
+A         |     78 |    84 |    +6 |     1950 |    2100 |   +150
+B         |     78 |    84 |    +6 |     1950 |    2100 |   +150
 
 note: per-condition vbytes are rounded individually; absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct
--- a/docs/manual/transcripts/41-compare-cost-ord.out
+++ b/docs/manual/transcripts/41-compare-cost-ord.out
@@ -4,7 +4,7 @@
 
 Condition         | wsh vB | tr vB |  Δ vB | wsh sats | tr sats | Δ sats
 ------------------+--------+-------+-------+----------+---------+-------
-A                 |     60 |    85 |   +25 |       60 |      85 |    +25
-B + older(blocks) |     60 |    86 |   +26 |       60 |      86 |    +26
+A                 |     79 |    85 |    +6 |       79 |      85 |     +6
+B + older(blocks) |     80 |    86 |    +6 |       80 |      86 |     +6
 
 note: per-condition vbytes are rounded individually; absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct
--- a/docs/manual/transcripts/41-compare-cost-tr-ik.out
+++ b/docs/manual/transcripts/41-compare-cost-tr-ik.out
@@ -5,7 +5,7 @@
 
 Condition | wsh vB | tr vB |  Δ vB | wsh sats | tr sats | Δ sats
 ----------+--------+-------+-------+----------+---------+-------
-key[0]    |     60 |    75 |   +15 |     1500 |    1875 |   +375
+key[0]    |     69 |    75 |    +6 |     1725 |    1875 |   +150
 
 Keypath-spend (via IK f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9): 58 vB | 1450 sats
 
--- a/docs/manual/transcripts/41-compare-cost-json.out
+++ b/docs/manual/transcripts/41-compare-cost-json.out
@@ -10,12 +10,12 @@
   "conditions": [
     {
       "label": "key[0]",
-      "wsh_vbytes": 60,
+      "wsh_vbytes": 69,
       "tr_vbytes": 75,
-      "delta_vbytes": 15,
-      "wsh_sats": 60,
+      "delta_vbytes": 6,
+      "wsh_sats": 69,
       "tr_sats": 75,
-      "delta_sats": 15
+      "delta_sats": 6
     }
   ],
   "keypath_spend": null,
--- a/docs/manual-gui/transcripts/tutorial/tut-j4-20-compare-cost.stdout.txt
+++ b/docs/manual-gui/transcripts/tutorial/tut-j4-20-compare-cost.stdout.txt
@@ -4,11 +4,11 @@
 
 Condition             | wsh vB | tr vB |  Δ vB | wsh sats | tr sats | Δ sats
 ----------------------+--------+-------+-------+----------+---------+-------
-A + B + older(blocks) |     79 |   147 |   +68 |       79 |     147 |    +68
-C + older(512s)       |     61 |   132 |   +71 |       61 |     132 |    +71
-D + older(512s)       |     61 |   132 |   +71 |       61 |     132 |    +71
-E + older(512s)       |     61 |   132 |   +71 |       61 |     132 |    +71
-F + preimage(h0)      |     69 |   139 |   +70 |       69 |     139 |    +70
+A + B + older(blocks) |    144 |   147 |    +3 |      144 |     147 |     +3
+C + older(512s)       |    126 |   132 |    +6 |      126 |     132 |     +6
+D + older(512s)       |    126 |   132 |    +6 |      126 |     132 |     +6
+E + older(512s)       |    126 |   132 |    +6 |      126 |     132 |     +6
+F + preimage(h0)      |    133 |   139 |    +6 |      133 |     139 |     +6
 
 note: per-condition vbytes are rounded individually; absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct
 note: input contains hash-preimage fragments; preimage-known rows are enumerated assuming the user can supply each preimage (cost only — no preimage knowledge is implied)
~~~~

### Appendix E. Part A `.examples-build/Examples.md` (`examples.diff`)

<!-- evidence: examples.diff -->
~~~~diff
--- a/.examples-build/Examples.md
+++ b/.examples-build/Examples.md
@@ -1204,11 +1204,11 @@
 
 Condition             | wsh vB | tr vB |  Δ vB | wsh sats | tr sats | Δ sats
 ----------------------+--------+-------+-------+----------+---------+-------
-A + B + older(blocks) |     79 |   147 |   +68 |       79 |     147 |    +68
-C + older(512s)       |     61 |   132 |   +71 |       61 |     132 |    +71
-D + older(512s)       |     61 |   132 |   +71 |       61 |     132 |    +71
-E + older(512s)       |     61 |   132 |   +71 |       61 |     132 |    +71
-F + preimage(h0)      |     69 |   139 |   +70 |       69 |     139 |    +70
+A + B + older(blocks) |    144 |   147 |    +3 |      144 |     147 |     +3
+C + older(512s)       |    126 |   132 |    +6 |      126 |     132 |     +6
+D + older(512s)       |    126 |   132 |    +6 |      126 |     132 |     +6
+E + older(512s)       |    126 |   132 |    +6 |      126 |     132 |     +6
+F + preimage(h0)      |    133 |   139 |    +6 |      133 |     139 |     +6
 
 note: per-condition vbytes are rounded individually; absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct
 note: input contains hash-preimage fragments; preimage-known rows are enumerated assuming the user can supply each preimage (cost only — no preimage knowledge is implied)
@@ -1224,7 +1224,7 @@
 
 Condition        | wsh vB | tr vB |  Δ vB | wsh sats | tr sats | Δ sats
 -----------------+--------+-------+-------+----------+---------+-------
-F + preimage(h0) |     68 |    90 |   +22 |       68 |      90 |    +22
+F + preimage(h0) |     84 |    90 |    +6 |       84 |      90 |     +6
 
 note: per-condition vbytes are rounded individually; absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct
 note: input contains hash-preimage fragments; preimage-known rows are enumerated assuming the user can supply each preimage (cost only — no preimage knowledge is implied)
~~~~

### Appendix F. HELD self-check (A2, R5/R6): do not apply (`fix-v5-production.patch`)

Appendix A plus the R5/R6 self-check. It is the build that produced `d-wrapper-probe.txt`. Do not apply it:
STOP-1 holds A2.

<!-- evidence: fix-v5-production.patch -->
~~~~diff
diff --git i/crates/mnemonic-toolkit/src/cost/enumerate.rs w/crates/mnemonic-toolkit/src/cost/enumerate.rs
index 4d5fe524..a494fc83 100644
--- i/crates/mnemonic-toolkit/src/cost/enumerate.rs
+++ w/crates/mnemonic-toolkit/src/cost/enumerate.rs
@@ -16,10 +16,11 @@
 
 use std::collections::BTreeSet;
 
+use miniscript::bitcoin::consensus::encode::VarInt;
 use miniscript::bitcoin::hashes::{hash160, ripemd160, sha256};
 use miniscript::bitcoin::{absolute, relative};
 use miniscript::descriptor::{DefiniteDescriptorKey, DescriptorPublicKey};
-use miniscript::plan::Assets;
+use miniscript::plan::{Assets, Plan};
 use miniscript::{Descriptor, ForEachKey, Miniscript, Segwitv0};
 
 use super::translate::{pubkey_to_label_segv0, Translated};
@@ -173,6 +174,16 @@ pub fn enumerate_minimal_conditions(
         return Err(CompareCostError::NoSatisfyingConditions);
     }
 
+    // Grade ruling 6: two-sided self-check of the wsh column against
+    // rust-miniscript's own `max_weight_to_satisfy` (which omits the
+    // BIP-144 stack-count byte, hence the +1).
+    let max_weight_wu = wsh_desc
+        .max_weight_to_satisfy()
+        .map_err(|_| CompareCostError::NoSatisfyingConditions)?
+        .to_wu();
+    let wsh_rows_wu: Vec<usize> = rows.iter().map(|r| r.wsh_witness_bytes).collect();
+    wsh_self_check(&wsh_rows_wu, max_weight_wu, has_thresh(&translated.segv0))?;
+
     Ok(EnumerationReport {
         rows,
         soft_cap_reached: soft_reached,
@@ -264,11 +275,68 @@ fn build_and_plan(
         }
     }
     match desc.clone().plan(&a) {
-        Ok(plan) => Some(plan.witness_size()),
+        Ok(plan) => Some(witness_weight_wu(desc, &plan)),
         Err(_) => None,
     }
 }
 
+/// Witness weight (WU) of one planned spend. For `wsh(M)` the plan's
+/// template omits the witnessScript, so add it as one more stack element
+/// (the expression of rust-miniscript PR #1046): `varint(script) + script`,
+/// and re-encode the stack-count varint for `t + 1` elements. `tr` templates
+/// already carry the leaf script and control block.
+fn witness_weight_wu(
+    desc: &Descriptor<DefiniteDescriptorKey>,
+    plan: &Plan<DefiniteDescriptorKey>,
+) -> usize {
+    let template_wu = plan.witness_size();
+    match desc {
+        Descriptor::Wsh(wsh) => {
+            let script_len = wsh.inner_script().len();
+            let elems = plan.witness_template().len();
+            template_wu + varint_len(script_len) + script_len - varint_len(elems)
+                + varint_len(elems + 1)
+        }
+        _ => template_wu,
+    }
+}
+
+fn varint_len(n: usize) -> usize {
+    VarInt(n as u64).size()
+}
+
+/// Grade ruling 6. `rows_wu` is every wsh row's witness weight (WU);
+/// `max_weight_wu` is `max_weight_to_satisfy().to_wu()` of `wsh(M)`.
+/// Without a `thresh`, the largest row must equal `max_weight_wu + 1`;
+/// with one, every row must be at most `max_weight_wu + 1`.
+fn wsh_self_check(
+    rows_wu: &[usize],
+    max_weight_wu: u64,
+    has_thresh: bool,
+) -> Result<(), CompareCostError> {
+    let largest_wu = rows_wu.iter().copied().max().unwrap_or(0) as u64;
+    let bound = max_weight_wu + 1;
+    let ok = if has_thresh {
+        largest_wu <= bound
+    } else {
+        largest_wu == bound
+    };
+    if ok {
+        Ok(())
+    } else {
+        Err(CompareCostError::CostSelfCheck {
+            largest_wu,
+            max_weight_wu,
+        })
+    }
+}
+
+/// `true` if any fragment of `m` is a `thresh(..)`.
+fn has_thresh(m: &Miniscript<DefiniteDescriptorKey, Segwitv0>) -> bool {
+    use miniscript::miniscript::decode::Terminal;
+    m.iter().any(|sub| matches!(sub.node, Terminal::Thresh(..)))
+}
+
 fn is_minimal(
     wsh_desc: &Descriptor<DefiniteDescriptorKey>,
     tr_desc: &Descriptor<DefiniteDescriptorKey>,
diff --git i/crates/mnemonic-toolkit/src/cost/mod.rs w/crates/mnemonic-toolkit/src/cost/mod.rs
index 49dda306..611f2d44 100644
--- i/crates/mnemonic-toolkit/src/cost/mod.rs
+++ w/crates/mnemonic-toolkit/src/cost/mod.rs
@@ -53,6 +53,9 @@ pub enum CompareCostError {
     ConditionsTooMany { raw: usize, cap: usize },
     /// Miniscript has zero satisfying conditions (degenerate). Exit 3.
     NoSatisfyingConditions,
+    /// The wsh column failed its self-check against
+    /// `max_weight_to_satisfy` (grade ruling 6). Exit 3.
+    CostSelfCheck { largest_wu: u64, max_weight_wu: u64 },
 }
 
 impl std::fmt::Display for CompareCostError {
@@ -82,6 +85,14 @@ impl std::fmt::Display for CompareCostError {
             CompareCostError::NoSatisfyingConditions => {
                 write!(f, "compare-cost: no satisfying conditions for this miniscript")
             }
+            CompareCostError::CostSelfCheck {
+                largest_wu,
+                max_weight_wu,
+            } => write!(
+                f,
+                "compare-cost: cost self-check failed: largest wsh witness {largest_wu} WU, max_weight_to_satisfy + 1 = {} WU; no figures printed (this is a toolkit bug)",
+                max_weight_wu + 1
+            ),
         }
     }
 }
@@ -95,7 +106,8 @@ impl CompareCostError {
             | CompareCostError::UnsupportedWrapper(_)
             | CompareCostError::MultiLeafTr
             | CompareCostError::ConditionsTooMany { .. }
-            | CompareCostError::NoSatisfyingConditions => 3,
+            | CompareCostError::NoSatisfyingConditions
+            | CompareCostError::CostSelfCheck { .. } => 3,
         }
     }
 }
diff --git i/crates/mnemonic-toolkit/src/cost/strip.rs w/crates/mnemonic-toolkit/src/cost/strip.rs
index 3efd1a6c..c4c0abf6 100644
--- i/crates/mnemonic-toolkit/src/cost/strip.rs
+++ w/crates/mnemonic-toolkit/src/cost/strip.rs
@@ -13,6 +13,9 @@ use super::translate::Translated;
 use super::CompareCostError;
 use super::NUMS_XONLY_HEX;
 
+/// Grade ruling 4: `notes[]` entry for a `sh(wsh(M))` input.
+pub const SH_WSH_NOTE: &str = "input had a p2sh wrapper, sh(wsh(M)); each wsh figure is for bare wsh(M): add 35 vB per input (the scriptSig redeem-script push) and subtract 35 vB from each Δ";
+
 /// Strip the wrapper from a user-supplied descriptor string; return a
 /// `Translated` ready for the enumerator (plus any advisory note). Concrete
 /// hex pubkeys are preserved; no abstract labels are detected (those only
@@ -68,7 +71,7 @@ pub fn translate_descriptor(input: &str) -> Result<(Translated, Option<String>),
         Descriptor::Sh(sh) => match sh.into_inner() {
             ShInner::Wsh(wsh) => {
                 let m_segv0 = wsh.into_inner();
-                translated_from_segv0(input, m_segv0).map(|t| (t, None))
+                translated_from_segv0(input, m_segv0).map(|t| (t, Some(SH_WSH_NOTE.to_string())))
             }
             ShInner::Wpkh(_) | ShInner::Ms(_) => Err(CompareCostError::UnsupportedWrapper(
                 "sh(non-wsh)".to_string(),
~~~~

### Appendix G. Expected text output for `sh(wsh(pk(KA)))` after A (`sh-wsh-pk.txt`)

<!-- evidence: sh-wsh-pk.txt -->
~~~~text
Input:     sh(wsh(pk(02998512205ec6a5cdb77d5b4f7de63c560d1e846162612ee178c49e7b6cc44fb9)))
Extracted: pk(02998512205ec6a5cdb77d5b4f7de63c560d1e846162612ee178c49e7b6cc44fb9)
Wrapper comparison: wsh(M)  vs  tr(NUMS, {M})
Feerate: 1.0 sat/vB

Condition | wsh vB | tr vB |  Δ vB | wsh sats | tr sats | Δ sats
----------+--------+-------+-------+----------+---------+-------
key[0]    |     69 |    75 |    +6 |       69 |      75 |     +6

note: input had a p2sh wrapper, sh(wsh(M)); each wsh figure is for bare wsh(M): add 35 vB per input (the scriptSig redeem-script push) and subtract 35 vB from each Δ
note: per-condition vbytes are rounded individually; absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct
note: input had concrete keys; cost is identical to the abstract case
~~~~
