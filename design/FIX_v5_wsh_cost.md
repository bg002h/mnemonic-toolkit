# FIX-V5: compare-cost wsh figures include the witness script (+ E9 wrapper chains)

Small-fix track (RULES.md "Process"). Repo mnemonic-toolkit, branch `fix/v5-wsh-cost`, base `ac3ff70a`
(the tip of `origin/triage/v5-v7`; it is not on master, which is `12e013bc`; see STOP-2 and STOP-7).

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
  fence equals its file and that the build patches apply to `ac3ff70a`, alone and together. It does not check
  figure correctness, Bitcoin Core agreement, the GUI byte gate or runtime behaviour; the runs below do.
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
| R5 | grade v5-v7 :127-131 | new `CostSelfCheck` error, exit 3 (`Parse` stays 2); build-descriptor propagates it through its existing `?` (`build_descriptor.rs:477-486`, `:512-521`), exit 3 in human and `--json` views, no degraded preview; one manual exit-table row |
| R6 | grade v5-v7 :133-139 | the self-check rule: thresh-free L == mw + 1, with a thresh every row <= mw + 1; STOP if a thresh-free shape breaks it; do not widen; keep the Core-measured rows |
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
| `d-wrapper-probe.txt`, `d-wrapper-v5a.txt` | Core's weight for `wsh(and_b(pk,sdv:older(10)))` and A's figure for it |
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

### A2. The cost self-check (R5, R6): HELD by STOP-1

R6's rule fails on the corpus (STOP-1), so none of A2 is built until it is ruled. For the review, the text R5
fixes, and what Appendix F measured:

- Variant `CompareCostError::CostSelfCheck { largest_wu: u64, max_weight_wu: u64 }`, exit `3`.
- Display, R5's template verbatim: `compare-cost: cost self-check failed: largest wsh witness {largest_wu} WU,
  max_weight_to_satisfy + 1 = {max_weight_wu + 1} WU; no figures printed (this is a toolkit bug)`.
- Check (Appendix F, `cost/enumerate.rs`): L = the largest planned wsh row in WU; mw =
  `wsh_desc.max_weight_to_satisfy()` in WU; thresh-free requires L == mw + 1; with a `thresh` every row must be
  <= mw + 1.
- build-descriptor: the error reaches the holder through the existing `?` at `build_descriptor.rs:477-486` and
  `:512-521`, exit 3, human and `--json`.
- Manual exit table (41-mnemonic.md, compare-cost "Exit codes"), author wording: `| wsh cost self-check failed
  against \`max_weight_to_satisfy\` (a toolkit bug; no figures printed) | \`3\` |`.
- Open beside STOP-1: STOP-3 (a `max_weight_to_satisfy` error), STOP-4 (the preview header prints before the
  `?`), STOP-6 (no input reaches exit 3, so no CLI row can see it).

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

### 3.3 Held rows (A2; not written until STOP-1 is ruled)

These rows have names and behaviour only. No code exists for them, on disk or in any commit, and Appendix F
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
| `docs/manual-gui/transcripts/tutorial/tut-j4-20-compare-cost.stdout.txt` | A + B: wsh 79 (Δ +68); C, D, E: 61 (+71); F + preimage(h0): 69 (+70) | 144 (+3); 126 (+6); 133 (+6) |
| `.examples-build/Examples.md` @@-1204 | the same J4 rows as `tut-j4-20` | the same as `tut-j4-20` |
| `.examples-build/Examples.md` @@-1224 | `68 \| 90 \| +22` | `84 \| 90 \| +6` |

The two J4 PNGs (`docs/manual-gui/figures/tutorial/tut-j4-20-compare-cost-form.png` and `-run.png`) show the
compare-cost panel; whether they are regenerated, and against which GUI binary, is STOP-5.

No `build-descriptor` transcript carries cost figures. The `build-descriptor --json` `cost` field embeds the
compare-cost JSON (`41-mnemonic.md:4864` at `ac3ff70a`), but no transcript shows it, so ruling 7's "build-descriptor
transcripts" item changes no file. Measured: `make audit` passes with only the four compare-cost transcripts
regenerated (128 transcript files checked).

## 5. Lockstep files (ruling 7)

Ten files change; none is production code. Appendix C holds the prose hunks, D the transcripts, E the examples.

| # | file | ruling 7 item, or why it is beyond ruling 7 | appendix, hunk |
|---|---|---|---|
| 1 | `docs/manual/src/40-cli-reference/41-mnemonic.md` | `:4596-4602` (the `witness_size()` prose), the notes catalog; plus the worked example at `:4660` (beyond R7: its `+24` moves) | C @@-4593, @@-4657, @@-4749 |
| 2 | `docs/manual/transcripts/41-compare-cost-orb.out` | compare-cost transcripts | D |
| 3 | `docs/manual/transcripts/41-compare-cost-ord.out` | compare-cost transcripts | D |
| 4 | `docs/manual/transcripts/41-compare-cost-tr-ik.out` | compare-cost transcripts | D |
| 5 | `docs/manual/transcripts/41-compare-cost-json.out` | compare-cost transcripts | D |
| 6 | `design/SPEC_compare_cost_v0_26_0.md` | `:180-183`, `:213` (lines `:180`, `:210` here); plus `:145` (the same formula in the step-3 sketch) and `:280` (the new note in the notes list), beyond R7 | C @@-145, @@-180, @@-210, @@-280 |
| 7 | `docs/manual-gui/tutorial/50-j4-taproot-twin.md` | `:241` (J4 step 20 prose; the changed line is `:240` at `ac3ff70a`) | C @@-238 |
| 8 | `docs/manual-gui/transcripts/tutorial/tut-j4-20-compare-cost.stdout.txt` | its transcript | D |
| 9 | `.examples-build/Examples.md` | beyond R7: `examples.yml` regenerates it and fails on any diff | E |
| 10 | `docs/manual/src/40-cli-reference/42-md.md` | beyond R7: its `md compile` recipe prints a compare-cost table at `:471-472` | C @@-469 |

Ruling 7 items that change no file:
- The exit-code table (`41-mnemonic.md:4753-4767` at `ac3ff70a`) changes only with A2, which is held (STOP-1).
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

Work on branch `fix/v5-wsh-cost` in a git worktree that is not under `/tmp`. Use `export
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
3. Part A (V5). Do not start until STOP-1 and STOP-5 (§10) are ruled. The steps below are the ones that apply
   under STOP-1 option (a) (ship V5a, the formula and the note, without the self-check). Under any other ruling,
   the ruling's text replaces this step and the FIX review re-checks §3, §4 and §5 first.
   - `git apply $E/fix-v5a-only.patch` (Appendix A), then `git apply $E/fix-v5a-docs.patch` (Appendix C), then
     `git apply $E/transcript-diffs.txt` (Appendix D), then `git apply $E/examples.diff` (Appendix E).
   - The tutorial transcript `tut-j4-20-compare-cost.stdout.txt` in Appendix D, and the J4 step-20 PNGs, follow
     the STOP-5 ruling. If that ruling keeps the GUI tutorial out of this FIX, revert that file's hunk
     (`git checkout -- docs/manual-gui/transcripts/tutorial/tut-j4-20-compare-cost.stdout.txt`) and the
     `50-j4-taproot-twin.md` hunk of Appendix C, and record the deferral as the ruling says.
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
applied. STOP-7 decides the base. If the build runs on `ac3ff70a` before FIX-V7's GREEN is in it, the only
allowed failures are V7's: items 1 and 3 then show exactly the V7 lines quoted there, and anything else is a
FAIL. Once V7's GREEN is in the base, item 1 must be clean and item 3 must show 0 failed.

`rust.yml`:

1. `cargo +1.95.0 fmt --all -- --check` (`rust.yml:118`). Expected: exit 0. On `ac3ff70a` before V7 the only
   diffs are in `crates/mnemonic-toolkit/tests/cli_bundle_recovery_threshold_text.rs` (V7's file, at its
   lines 50 and 164); no diff may name a file this FIX touches.
2. `cargo metadata --locked --format-version 1 > /dev/null` (`:148`) and `cargo build --locked --tests
   --workspace` (`:171`). Expected: exit 0.
3. `cargo nextest run --locked --workspace --no-fail-fast` (`:185`). Expected: 0 failed. On `ac3ff70a` before V7:
   exactly 11 failed, all in `cli_bundle_recovery_threshold_text` and all named `v7_*` (measured with A and B
   applied: `4227 tests run: 4216 passed, 11 failed, 38 skipped`).
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
    moves. Expected: every job green on the PR, item 12's lint as STOP-5 rules.

The FIX rows and the mutation gate:

13. The FIX rows by exact name: §3.1's command. Expected: exit 0, exactly **30 tests run: 30 passed**. If E9
    ships alone (STOP-2), §3.2's command: exactly **12 tests run: 12 passed**. Quote the line in the GREEN
    commit message. A short or empty count is a FAIL, whatever the pass count.
14. `bash /scratch/code/shibboleth/refugium-coordinator/coordinator/scripts/mutation-gate.sh <worktree> ac3ff70a`
    after items 1-5 pass. Production files touched: `cost/enumerate.rs`, `cost/strip.rs` (Part A) and
    `cost/translate.rs` (Part B). Its missed list goes into the review brief. Risk class: A (a new printed note,
    the manual and transcripts) and C (cost code); the coordinator may raise it.

Every other workflow that runs on the PR (`release.yml`'s build matrix and `sibling-pin-check.yml` run on
every `pull_request`) must be green on the PR head. `fuzz-smoke.yml`, `bitcoind-differential.yml` and
`cross-tool-differential.yml` are path-filtered and none of their paths is touched here (`cost/` is not in
any list), so they do not run; `man-pages.yml`, `changelog-check.yml` and `install-pin-check.yml` run only on
tags. None of these is a local step of this gate.
