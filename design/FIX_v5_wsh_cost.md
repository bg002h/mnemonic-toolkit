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
- The 30-row RED filter (§3.1) on base: `30 tests run: 11 passed, 19 failed, 4235 skipped`; with Appendices A
  and B applied: `30 tests run: 30 passed, 4235 skipped`.
- The E9 file alone (§3.2) on base: `12 tests run: 5 passed, 7 failed, 0 skipped`; with Appendix B only:
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
changes, and `41-mnemonic.md:4616` stays true as written (E9-a).
