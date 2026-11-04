**⚠ Proposed Important: V5 (compare-cost prints wrong wsh figures and a false "Δ values are correct") and V7 (bundle prints a false threshold and recovery line on non-plain descriptors, one case understated). Both CONFIRMED; no Critical proposed.**

# Triage: toolkit V5 (compare-cost wsh) and V7 (bundle recovery text), 2026-11-03

Brief: `coordinator/briefs/triage-tk-v5-v7.md`. Reports under triage:
`coordinator/agent-reports/architect-2026-11-03-miniscript-13-2.md` §Evidence V5, V7
(V6 is context). Verdict rule: RULES.md "Triage of reported bugs". Grades are
proposals only, by the likelihood test quoted in the brief. No production code
is fixed; the scratch fixes below are uncommitted in the worktree.

## Setup (all executed)

- Worktree `/scratch/code/shibboleth/tk-triage-v5-v7`, branch `triage/v5-v7`, base
  `origin/master` = `02052a17` (mnemonic-toolkit 0.105.1). Toolchain 1.85.0
  (`cargo --version` = `cargo 1.85.0`), `CARGO_TARGET_DIR=/scratch/cache/tk-triage-v5-v7-target`.
- Baseline, captured once (`/scratch/cache/tk-triage-v5-v7-logs/baseline.log`):
  `cargo nextest run --locked --workspace` -> **4185 tests run: 4185 passed, 38 skipped**.
- Oracle: Bitcoin Core **v31.1.0** (`/scratch/bitcoin-signet/core-31.1/bin/bitcoind`), private
  regtest datadir `/scratch/cache/tk-triage-v5-v7-logs/regtest`, ports 19991/19992. The
  `bitcoind` on PATH is Bitcoin Satellite v0.2.4 (Core v25.0.0), which has no `multi_a`, so it
  was not used; the running signet node was not touched. The oracle script
  (`/scratch/cache/tk-triage-v5-v7-logs/oracle/oracle.py`, `spendsets.py`) funds a fresh UTXO
  at the descriptor, signs with only the chosen keys held privately (`utxoupdatepsbt` ->
  `walletprocesspsbt` -> `finalizepsbt`), decodes the final tx, and computes the input weight
  as 4 x (36 + varint(scriptSig) + scriptSig + 4) + serialized witness bytes. It cross-checks
  its arithmetic against Core's own `decoderawtransaction` `weight` on every run
  (`weight_formula_matches_core: true`). Throwaway regtest keys only; nothing secret in the repo.

## V5: compare-cost omits the wsh witness script

**Headline verdict: CONFIRMED.**

Reported reason: `Plan::witness_size()` sums the satisfaction template only; the wsh witnessScript
(and its varint) is missing, so every `wsh vB` is short by `(varint(len)+len)/4`, and every
`delta` is overstated by the same amount.

Code (unmodified, `02052a17`): `cost/enumerate.rs:267` `Ok(plan) => Some(plan.witness_size())`;
`cost/format.rs:56` `total_wu = SEGWIT_INPUT_BASE_WU + witness_bytes` (`SEGWIT_INPUT_BASE_WU = 164`,
`enumerate.rs:50`). The vendored `Plan::witness_size` (`vendor/miniscript/src/plan.rs:258-264`)
returns `witness_size(self.template)` only.

### Evidence: Core vs tool (input vB, ceil(WU/4))

| spend | Core 31.1 (WU / vB) | tool before | tool with scratch fix |
|---|---|---|---|
| wsh(pk) | 273 / 69 | 60 | 69 |
| wsh(or_d(pk,pk)) via 1st / 2nd key | 311 / 78, 312 / 78 | 60, 60 | 78, 79 |
| wsh(and_v(v:pk,pk)) | 380 / 95 | 78 | 96 |
| wsh(multi(2,3 keys)) | 416 / 104 | 78 | 105 |
| wsh(multi(3,5 keys)) | 556 / 139 | 97 | 140 |
| wsh(thresh(2,pk,s:pk,s:pk)) | 422 / 106 | 78 | 106 |
| tr(NUMS,pk) | 299 / 75 | 75 | 75 |
| tr(NUMS,or_d(pk,pk)) via 1st / 2nd key | 336 / 84, 337 / 85 | 84, 85 | 84, 85 |
| tr(NUMS,multi_a(2,3 keys)) | 435 / 109 | 109 | 109 |
| tr(NUMS,multi_a(3,5 keys)) | 569 / 143 | 143 | 143 |
| sh(wsh(pk)) | 413 / 104 | 60 (wsh figure) | 69 (wsh figure) |
| sh(wsh(multi(2,3))) | 556 / 139 | 78 (wsh figure) | 105 (wsh figure) |

"Before" is the table the CLI prints today (`mnemonic compare-cost --miniscript 'pk(A)'`
prints `60 | 75 | +15`; `multi(2,A,B,C)` prints `78 | 109 | +31`). The residual +1 vB on
multi-signature rows is the tool's 72/73-byte dummy ECDSA signature against Core's
low-R 71-byte one, inside the tool's own printed "+-1" note. The error is exactly
`varint(script_len)+script_len` WU (pk: 36 WU = 9 vB; multi 2-of-3: 106 WU = 26.5 vB).

Consequences, as printed today: 60 vB is below the 68 vB of a P2WPKH input (no P2WSH spend can
be); `delta` for `pk(A)` is +15 against a real +6, for `multi(2,A,B,C)` +31 against a real +5
(the tool's own note says "Δ values are correct"; the manual at
`docs/manual/src/40-cli-reference/41-mnemonic.md:4596-4602` says `witness_size()` "includes ...
the serialized witnessScript" and that absolute `wsh vB` "match[es] Sparrow / Bitcoin Core / mempool";
SPEC_compare_cost_v0_26_0.md:180-183, 213 makes the same claim). Direction: wsh looks cheaper than
it is and taproot looks dearer than it is.

### Tests (committed, RED on `02052a17`; commits `a34bd61e` and `123c258d`)

`crates/mnemonic-toolkit/tests/cli_compare_cost_wsh_witness_script.rs`, 14 rows through the
public CLI (`assert_cmd`, `--json`). Expected figures are the Core spends above, pinned in the
file's header table.

- On unmodified code (`v5-red.log`): **14 tests, 5 passed, 9 failed**. The 9 failures all assert a
  wsh figure or a delta and print e.g. `got [("A", 60, 75, 15)]`, `got [("A + B", 78, 109, 31), ...]`:
  `v5_wsh_pk_vbytes_match_core`, `v5_descriptor_wsh_pk_vbytes_match_core`,
  `v5_wsh_or_d_vbytes_match_core`, `v5_wsh_and_v_vbytes_match_core`,
  `v5_wsh_multi_2of3_vbytes_match_core`, `v5_wsh_multi_3of5_vbytes_match_core`,
  `v5_wsh_thresh_2of3_vbytes_match_core`, `v5_p2wsh_input_never_cheaper_than_p2wpkh_input`
  (`pk(A): wsh vB 60 is below the 68 vB P2WPKH input`), `v5_delta_matches_core`.
  The 5 passes are the control rows (tr column x4, sh(wsh) strip x1).
- With the scratch fix (below): the 14 pass, plus the 55 existing `cli_compare_cost` rows
  (`v5-green.log`: **69 tests run: 69 passed, 1 skipped**). Nothing in the existing suite pinned an
  absolute wsh figure, as the architect said.
- Test-authoring slip, fixed before commit: my first `or_d` row asserted exactly 78 and failed once
  the fix landed (the second branch is 79 with the tool's 73-byte dummy sig); widened to 78..=79
  and the RED run was repeated on unmodified code.

### Scratch fix (uncommitted; `git diff` is also saved to `/scratch/cache/tk-triage-v5-v7-logs/scratch-fixes-v5-v7.patch`)

`cost/enumerate.rs::build_and_plan`, new lines at `:266-297`: for `Descriptor::Wsh(w)` add
`varint(w.inner_script().len()) + len`; tr gets nothing (its template already carries the leaf
script and control block). Plus a self-check that `total <= desc.max_weight_to_satisfy().to_wu() + 1`.

Facts for the FIX document:

1. **The self-check needs `+ 1`.** `Wsh::max_weight_to_satisfy` is "relative to the empty witness"
   (`vendor/miniscript/src/descriptor/segwitv0.rs:76-83`: it counts `varint(elems) - varint(0)`),
   `Plan::witness_size` includes the one-byte stack count. My first attempt without `+ 1` panicked on
   23 existing rows (`witness 110 exceeds max_weight_to_satisfy 109`). With `+ 1` all 69 rows pass.
   mr's doc says the same ("that function's 1-WU stack-count convention",
   `mnemonic-refugium cost.rs@5ed5b66f:1-30`).
2. **The self-check does refuse a double count** (the architect's claim, executed): planting
   `2 * (vi + n)` in the fix makes every wsh row fail with `compare-cost: witness 146 exceeds
   max_weight_to_satisfy 109` (`v5-plant-double.log`). In my scratch it is an `assert!` (exit 101);
   the FIX should make it a typed refusal.
3. **thresh caveat from mr (E28).** mr's doc says `max_weight_to_satisfy()` is not exact with a
   `thresh` at the pin (overestimates by about a signature per multisig sub or nesting level, may
   fall short by less than one) and uses a one-sided backstop there. I tried four thresh shapes
   (`thresh(2,pk,s:pk,s:pk)`, `thresh(1,multi(2,..),s:pk)`, an `or_d(multi, and_v(v:thresh,
   older))`, and the Core-measured two) with the `+ 1` check in place: none fired. That is four
   shapes, not proof; the FIX should adopt mr's backstop shape for thresh or test more nesting.
4. **Upstream master already fixes it** (`rust-miniscript-fork` `upstream/master` adf262b3,
   `src/plan.rs:258-277`): `template_size + varint_len(script_len) + script_len - varint_len(template.len())
   + varint_len(template.len()+1)`. The last two terms differ from mine only when the item count
   crosses 252/253, which the 100-item wsh standardness cap rules out. Bumping the pin to a miniscript
   with #1046 makes the toolkit double count unless the toolkit's own addition is removed in the same
   change; the self-check in item 2 is what would catch it (V6 is the same hazard for mr).

### V5 sibling rows

| # | case | verdict | evidence |
|---|---|---|---|
| V5-1 | wsh single key: `--miniscript pk(A)`, `--descriptor wsh(pk(KEY))`, never-below-68 invariant | CONFIRMED | 60 vs Core 69; RED on unmodified, GREEN with fix |
| V5-2 | wsh `or_d(pk,pk)` | CONFIRMED | 60/60 vs Core 78/78 |
| V5-3 | wsh `and_v(v:pk,pk)` | CONFIRMED | 78 vs Core 95 |
| V5-4 | wsh `multi(2,A,B,C)` | CONFIRMED | 78 vs Core 104 |
| V5-5 | wsh `multi(3,A..E)` | CONFIRMED | 97 vs Core 139 |
| V5-6 | wsh `thresh(2,pk,s:pk,s:pk)` | CONFIRMED | 78 vs Core 106 |
| V5-7 | `delta` column and the note "Δ values are correct" | CONFIRMED | +15 vs +6 (pk), +31 vs +5 (multi 2-of-3); `v5_delta_matches_core` |
| V5-8 | tr column: `pk`, `or_d`, `multi_a` 2-of-3 and 3-of-5, `--descriptor tr(NUMS,pk)` | FALSE POSITIVE (the tr side is not wrong) | passes on unmodified code and equals Core in all 5 shapes; planting `+36` on tr (`enumerate.rs:267`, `Some(plan.witness_size() + if matches!(desc, Descriptor::Tr(_)) {36} else {0})`) fails exactly the 4 tr rows and leaves the wsh rows' status unchanged (`v5-plant-tr.log`: 1 passed, 11 failed of 12 at that time) |
| V5-9 | `--descriptor sh(wsh(M))` | UNRESOLVED | the wrapper is stripped by design (SPEC_compare_cost_v0_26_0.md:69 "comparison ignores p2sh-redeem-script overhead"; Core measures sh(wsh(pk)) at 104 vB, 35 vB above the wsh figure, because of the 35-byte scriptSig push). The same V5 omission applies to its printed wsh figure and is fixed with the rest. Whether to model or merely name the sh overhead is a product decision (open question Q2); control row `v5_sh_wsh_descriptor_reports_the_wsh_column` pins today's behaviour |

### V5 grade proposal

The tool does not fail closed (exit 0, a wrong figure, a false "Δ values are correct" claim),
so condition (1) of the likelihood test fails and it cannot be Minor. **Proposed Important** for
V5-1..V5-7 together (one defect, one fix). Not Critical: a fee estimate and a wsh-vs-tr comparison,
no address, card or key binding is affected. V5-9, if the design stands: **Minor** (a documented
simplification the manual does not repeat).

Lockstep if fixed: manual `41-mnemonic.md:4596-4602` (its claim becomes true only after the fix;
the sh(wsh) line is new text), SPEC §4.

## V7: bundle recovery text overstates (and sometimes understates) the threshold

**Headline verdict: CONFIRMED.**

Reported reason: `bundle --descriptor` prints `# Threshold: K of N` and `# Recovery: any K of N
signing keys + md1 (template card).` from `extract_multisig_threshold`, which is not the real
spend condition for a non-plain policy.

Code (unmodified, `02052a17`): `cmd/bundle.rs:1250-1262` `extract_multisig_threshold` returns the K of
the FIRST `MultiKeys`/`Variable` node found depth-first (`Body::Children(c) => c.iter().find_map(..)`,
`:1255`), else `None`; `cmd/bundle.rs:1316-1318` reads it and `:1365-1368` falls back to N
(`.or(descriptor_threshold).or(if n > 1 { Some(n) } ...)`), feeding `format.rs:282` (`# Threshold`)
and `format.rs:350` (`# Recovery: any`). The `--json` branch does the same at `bundle.rs:957-963`
(`args.threshold.or(descriptor_threshold).unwrap_or(n as u8)`). `restore --md1` already knows the
walk is not a threshold: `restore.rs:4358` `threshold_field = if template_opt.is_some() { k_opt } else { None }`
and its comment at `:4352` ("for a decay vault `extract_multisig_threshold` returns only the FIRST k").

### Evidence: which key sets Core lets spend

For every shape every subset of the keys was tried as the only private keys in a wallet that
imported the descriptor (`spendsets.py`); a subset spends iff the Core-signed PSBT finalizes
(`matrix.out`, `matrix2.out`). Tool column = stderr of
`mnemonic bundle --network mainnet --descriptor <same shape, fixture xpubs>`, exit 0 in every row.

| descriptor | Core: minimal spending sets | printed `Threshold` / `Recovery: any` |
|---|---|---|
| wsh(sortedmulti(2,K0,K1,K2)) | any 2 of 3 | 2 of 3 / any 2 of 3 (correct) |
| wsh(multi(2,K0,K1,K2)) | any 2 of 3 | 2 of 3 (correct) |
| tr(NUMS,sortedmulti_a(2,K0,K1,K2)) | any 2 of 3 | 2 of 3 (correct) |
| tr(NUMS,multi_a(2,K0,K1,K2)) | any 2 of 3 | 2 of 3 (correct) |
| tr(NUMS,{pk(K0),pk(K1)}) | {0} {1} | **2 of 2** |
| tr(NUMS,{pk(K0),{pk(K1),pk(K2)}}) | {0} {1} {2} | **3 of 3** |
| wsh(or_d(pk(K0),pk(K1))) | {0} {1} | **2 of 2** |
| wsh(or_d(pk(K2),multi(2,K0,K1))) | {2} {0,1} | **2 of 3** |
| wsh(or_i(multi(2,K0,K1),pk(K2))) | {2} {0,1} | **2 of 3** |
| tr(NUMS,{multi_a(2,K0,K1),pk(K2)}) | {2} {0,1} | **2 of 3** |
| tr(NUMS,{sortedmulti_a(2,K0,K1),pk(K2)}) | {2} {0,1} | **2 of 3** |
| tr(K0,multi_a(2,K1,K2)) (non-NUMS internal key) | {0} {1,2} | **2 of 3** |
| wsh(and_v(v:pk(K0),or_d(pk(K1),pk(K2)))) | {0,1} {0,2} | **3 of 3** |
| tr(NUMS,{multi_a(2,K0,K1),multi_a(2,K2,K3)}) | {0,1} {2,3} | 2 of 4, "any 2 of 4" is false ({K0,K2} cannot spend) |
| wsh(or_d(multi(2,K0,K1),multi(2,K2,K3))) | {0,1} {2,3} | same |
| wsh(and_v(v:multi(2,K0,K1),multi(2,K2,K3))) | {0,1,2,3} only | **2 of 4 (understated: all four needed)** |
| wsh(thresh(2,pk(K0),s:pk(K1),s:pk(K2))) | any 2 of 3 | 2 of 3 (correct) |

The architect's three examples reproduce (rows 5, 8 and 10 above). The JSON `multisig.threshold`
carries the same false value in every bold row (`run.sh` output), so the same K feeds a wire field
that `MultisigInfoView` (`wallet_import/json_envelope.rs:174-176`, `threshold: u8`) reads back.
`tr(NUMS,{multi_a(2,K0,K1),multi_a(2,K1,K2)})` was refused by bundle (exit 2, BIP-388 distinct-key
violation), so key-sharing leaves were not usable for this sweep; `tr(NUMS,{and_v(v:pk(K2),
sortedmulti_a(1,K0,K1)),pk(K3)})` is architect V10 (T3's), not triaged here.

### Tests (committed, RED on `02052a17`; commit `d19256e8`)

`crates/mnemonic-toolkit/tests/cli_bundle_recovery_threshold_text.rs`, 12 rows (`assert_cmd`,
fixture xpubs already used by `cli_bundle_json_threshold.rs`). They assert only that the false claim
is gone, not what replaces it (open question Q1).

- On unmodified code (`v7-red.log`): **12 tests, 1 passed, 11 failed**, e.g.
  `tr(NUMS,{pk(K0),pk(K1)}): card prints the false '# Threshold: 2 of 2'`,
  `wsh(and_v(v:multi(2,K0,K1),multi(2,K2,K3))): card prints the false '# Threshold: 2 of 4'`,
  `tr(NUMS,{multi_a(2,K0,K1),multi_a(2,K2,K3)}): card claims any 2 of 4 keys recover`.
  The pass is the control over the four plain shapes.
- With the scratch fix: **12 passed** (`v7-green.log`).

### Scratch fix (uncommitted)

`cmd/bundle.rs`: new `is_plain_threshold_shape(tree)` at `:1263-1292` (true only for a `multi`-family
root under `wsh`, `sh`, `sh(wsh)`, or a NUMS-internal-key `tr` whose only leaf is that root), and
`build_unified_card` (`:1396-1411`) prints no threshold when `--threshold` is absent and the tree is
not plain. JSON left as is (Q1). This is a triage device, not the proposed design.

### V7 sibling rows

| # | case | verdict | evidence |
|---|---|---|---|
| V7-1 | `tr(NUMS,{pk,pk})` | CONFIRMED | "2 of 2"; either key spends |
| V7-2 | `tr(NUMS,{pk,{pk,pk}})` depth-2 | CONFIRMED | "3 of 3"; any one key spends |
| V7-3 | `wsh(or_d(pk,pk))` | CONFIRMED | "2 of 2"; any one |
| V7-4 | `wsh(or_d(pk(K2),multi(2,K0,K1)))` | CONFIRMED | "2 of 3"; K2 alone spends |
| V7-5 | `wsh(or_i(multi(2,..),pk))` | CONFIRMED | "2 of 3"; the pk alone spends |
| V7-6 | `tr(NUMS,{multi_a(2,..),pk})` | CONFIRMED | "2 of 3"; the pk alone spends |
| V7-7 | `tr(NUMS,{sortedmulti_a(2,..),pk})` | CONFIRMED | same |
| V7-8 | `tr(K0,multi_a(2,K1,K2))`, non-NUMS internal key | CONFIRMED | "2 of 3"; K0 alone spends (key path) |
| V7-9 | `wsh(and_v(v:pk,or_d(pk,pk)))` | CONFIRMED | "3 of 3"; two keys suffice |
| V7-10 | two disjoint multi leaves (tr and wsh forms), "any 2 of 4" | CONFIRMED | {K0,K2} cannot spend |
| V7-11 | `wsh(and_v(v:multi(2,..),multi(2,..)))` | CONFIRMED | "2 of 4" but all four are needed (understated) |
| V7-12 | the four plain shapes | FALSE POSITIVE (the lines are correct) | control passes on unmodified code; planting `if true { return false; }` at the top of `is_plain_threshold_shape` (scratch copy, `bundle.rs:1267`) fails exactly `v7_control_plain_shapes_keep_threshold_and_recovery_lines` and nothing else (`v7-plant-plain.log`) |
| V7-13 | `--json` `multisig.threshold` on the same shapes | UNRESOLVED | the false value is emitted (`run.sh`), but the right value is a product decision on a wire field (null? omitted? a new field?) that `--import-json` reads as `u8`; no test written (Q1) |
| V7-14 | `wsh(thresh(2,pk,s:pk,s:pk))` | UNRESOLVED | the printed "2 of 3" and "any 2 of 3" are true here; my scratch predicate would drop them. Whether a pure `thresh` keeps the line is a product decision (Q1); not tested |

### V7 grade proposal

The tool does not fail closed (exit 0 and a printed false claim on an engraving card), so the
likelihood test cannot reach Minor on condition (1). The path needs one unusual condition (a
non-plain descriptor given to `bundle --descriptor`; `bundle` accepts them all and `restore`
carries a general-policy arm for them). **Proposed Important** for V7-1..V7-11 together (one
root cause, one fix). I do not propose Critical: the md1/mk1 cards themselves are correct and the
wrong text is an advisory line. A grader should look at V7-11 first if considering Critical: it says
"any 2 of 4 signing keys + md1" for a wallet that needs all four, which is the direction that can
strand funds. V7-13 shares the grade of the defect it belongs to if the wire value is changed.

## V6 (context for V5's fix; not triaged)

`mnemonic-refugium cost.rs@5ed5b66f` (read, not run here): `witness_script_wu` (`:241`) adds
`len + varint_len(len)` for `Wsh | ShWsh`, the same quantity my scratch fix adds and Core confirmed
above; its doc comment states the same 1-WU stack-count convention I hit (V5 fact 1). I did not run
mr.

## Open questions (product decisions; not decided here)

- **Q1 (V7 text and wire).** What replaces the `# Threshold` and `# Recovery: any` lines for a
  non-plain descriptor: nothing, a minimum-keys phrase, or a pointer to the policy? What does
  `--json` `multisig.threshold` become (it is `u8`, read by `--import-json`)? Does a pure `thresh`
  keep its line? The brief says V7's text is decided at its FIX document.
- **Q2 (V5 sh(wsh)).** Add a note that sh(wsh) is priced as wsh, or model the 35-byte scriptSig push?
- **Q3 (V5 refusal).** The self-check should be a typed refusal with an exit code; wording and code
  are the FIX document's (mr uses `Error::CostSelfCheck`).

## Reproduce

```
cd /scratch/code/shibboleth/tk-triage-v5-v7
PATH=$HOME/.cargo/bin:$PATH CARGO_TARGET_DIR=/scratch/cache/tk-triage-v5-v7-target
cargo nextest run --locked -p mnemonic-toolkit --test cli_compare_cost_wsh_witness_script --test cli_bundle_recovery_threshold_text
python3 -I /scratch/cache/tk-triage-v5-v7-logs/oracle/oracle.py 'wsh(pk({K0}))' 0          # needs the regtest node
python3 -I /scratch/cache/tk-triage-v5-v7-logs/oracle/spendsets.py 'tr(NUMS,{pk({K0}),pk({K1})})' 2
```

Full suite with both scratch fixes and the 26 new rows (`both-fullsuite.log`):
**4211 tests run: 4211 passed, 38 skipped** (4185 baseline + 26).

Nothing machine-verified was left to a reviewer except the three open questions and the grades.
Not covered: the GUI schema mirror and GUI manual (no flag changes in either scratch fix), `restore`
on these cards (architect V8), and V10.

Counts: CONFIRMED 18, FALSE POSITIVE 2, UNRESOLVED 3; proposed C 0 / I 2 / M 1 / N 0
