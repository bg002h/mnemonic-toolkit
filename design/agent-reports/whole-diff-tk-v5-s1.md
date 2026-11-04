# Whole-diff review: toolkit V5 (+E9), slice 1 (executed advice and design)

Target: `tk-build-v5`, branch `build/v5-wsh-cost`, head d8abb6f8; diff `12e013bc..d8abb6f8` (PR #134, draft).
Reviewer: Opus 5.5, effort high. This is the first whole-diff review of this build, not a re-review, so no finding
traces to an earlier fix commit.
Binary: `target/debug/mnemonic` built `--locked` at d8abb6f8 (cargo 1.85.0, no `--release`).
Scratch: `/scratch/cache/whole-diff-tk-v5/s1/` (outputs, regtest datadir, `scripts/spend.sh`).

## Answer to the one question
Yes, within the tool's documented dummy-signature residual. Every wsh figure compare-cost prints now counts the
witnessScript and its length varint, and it matches Bitcoin Core 31.1 spends to within the 72-byte vs 71-byte ECDSA
dummy signature (+1 WU per signature, rounded). E9 chains (`ln:`, `snl:`, `sdv:`, `sj:`, `tv:`) are accepted on
`--miniscript` and give the same figures as the `--descriptor wsh(..)` oracle. Invalid chains are still refused with
exit 2. No valid policy I probed was refused. No Critical or Important.

## What I executed
- **Printed commands, as printed.** I ran the four transcript `.cmd` files (`41-compare-cost-{json,orb,ord,tr-ik}`) with
  `MNEMONIC_BIN` set to the head binary. stdout and stderr are byte-identical to the committed `.out` and `.err`.
  `42-md.md:469-473` (`compare-cost --miniscript 'or_d(pk(A),and_v(v:pk(B),older(144)))'`) prints `79|85|+6` and
  `80|86|+6`, as the doc says. The two `.examples-build/Examples.md` commands at :1200 and :1220 print the committed
  rows (144/126/126/126/133 and 84). Appendix G's `sh(wsh(pk(KA)))` command output equals `sh-wsh-pk.txt`
  byte for byte.
- **Formula against the authorities.**
  - BIP-144 (raw text saved to `s1/ext/bip144.txt`, :82): each item is a var_int length plus its bytes, behind a
    var_int count.
  - BIP-141 (`s1/ext/bip141.txt`, :102, :206): the witnessScript is the last witness item.
  - At the pin ff4732e, `Plan::witness_size` sums only the template plus `varint_len(n)` (`util.rs:50-51`). The
    wsh template is the miniscript satisfaction only (`segwitv0.rs:164-169`). The tr template carries the TapScript
    and the control block (`tr/mod.rs:500-501`).
  - `witness_weight_wu` equals upstream #1046 (`rust-miniscript-fork` adf262b3 `src/plan.rs:258-277`, merge
    20596a5), so F3's step (1) ("reduce to `witness_size()`") is correct.
- **Hand computation against the tool.** Every value matched:
  - `or_b(pk(A),s:pk(B))`: 148 WU, 78 vB, Δ +6.
  - `or_d…older(144)`: 79/85 and 80/86.
  - `multi(2,12 keys)`: 182/188/+6. The script is 411 bytes, so this is the varint-3 path.
  - `multi(11,12)`: 346/332/−14. `multi(12,12)`: 364/348/−16. `multi(7,10)`: 256/251/−5.
  - `and_b(pk(A),snl:after(100))`: 72/78.
  - The build-descriptor `kofn-recovery` preview with signet tpubs: 107/111 and 89/95. This equals
    `--miniscript` and the hand figures.
- **Core 31.1 regtest spends** (nix `bitcoind-31.1`; throwaway regtest keys; witness WU from `decoderawtransaction`):

  | shape | Core witness items (bytes) | Core witness WU / input vB | tool vB |
  |---|---|---|---|
  | `wsh(multi(2, 12 keys))` (script 411 B, varint 3) | 0, 71, 71, 411 | 560 / 181 | 182 (2 × 73-byte dummy sig) |
  | `wsh(thresh(2,pk,s:pk,snl:older(1)))`, A+B path | 1, 71, 71, 83 | 231 / 99 | 100 (A + B row) |
  | `wsh(and_v(v:pk,ln:older(1)))`, nSequence 1, `testmempoolaccept` allowed | 0, 71, 42 | 117 / 71 | 71 |

  Together with the triage's Core table (V5-1..V5-6: 69, 78/78, 95, 104, 139, 106 against the tool's
  69, 78/79, 96, 105, 140, 106 now), every residual is the dummy signature and never the script term.
- **E9 probes** (`--miniscript` against `--descriptor 'wsh(<concrete>)'`):
  - Identical rows: `and_b(pk(A),snl:after(100))`, `andor(pk(A),ln:older(144),pk(B))`,
    `and_v(v:pk(A),ln:older(144))`, `or_d(pk(A),and_v(v:pk(B),ln:older(144)))`, `and_b(pk(A),sdv:older(10))`,
    `thresh(2,pk(A),snl:older(144),s:pk(B))` and `and_b(pk(A),a:pk(B))`.
  - Refused on both paths, as expected for invalid miniscript: `or_i(pk(A),sdv:pk(B))`,
    `or_d(pk(A),snl:pk(B))`, `and_v(tv:pk(A),pk(B))`, `or_b(pk(A),sj:pk(B))` (malleable) and
    `and_b(pk(ln),ln:older(10))`.
  - `and_b(pk(A),xyz:pk(B))` gives `unknown wrapper «z:»`, which names the user's own letter and no dummy-key hex.
  - Spaced spellings `s :pk(B)` and `, s:pk(B)` are accepted.
- **Fidelity to the FIX.** The production `+/-` lines equal Appendices A and B (sorted-line diff: identical). The
  doc hunks equal Appendix C, apart from the F4-reverted GUI tutorial line, plus Appendix D's transcripts.
- **sh(wsh) note, followed literally.**
  - `sh(wsh(pk))`: 69 + 35 = 104, which is Core's 413 WU. Δ +6 − 35 = −29 = 75 − 104.
  - `sh(wsh(multi(2,2 keys)))` on stdin: the note is present and first in `notes[]`. 96 + 35 = 131, which equals
    (384 + 140) / 4.
- **Stale figures elsewhere.** Neither manual, the GUI manual, the READMEs nor mnemonic-gui `src` carries another
  wsh figure or another "witness_size includes the witnessScript" claim. The only remaining one is the F4-deferred
  `tut-j4-20` transcript and `50-j4-taproot-twin.md:241`, which the architect's STOP-5 ruling covers.

## Findings

### Critical
None.

### Important
None.

### Minor
- **M-1 (advice lens, ruling-4 text).** `cost/strip.rs:17` `SH_WSH_NOTE` (the grade's text verbatim) gives the
  correction in vB only.
  - Reproduction: `echo 'sh(wsh(multi(2,02998512205ec6a5cdb77d5b4f7de63c560d1e846162612ee178c49e7b6cc44fb9,03dff1d77f2a671c5f36183726db2341be58feae1da2deced843240f7b502ba659)))' | target/debug/mnemonic compare-cost --json --feerate 25`
    prints `wsh_sats` 2400 and `delta_sats` 100. The real values are 3275 and −775.
  - A holder who applies "subtract 35 vB from each Δ" to the `Δ sats` column gets 65.
  - It does not fail closed, but it misstates fees only and needs three conditions: a `sh(wsh)` input, a feerate
    other than 1, and reading the sats columns rather than vB.
  - Fix: this needs a coordinator ruling, because the text is ruling 4's. Proposed text: "…add 35 vB per input (the
    scriptSig redeem-script push; 35 × feerate in the sats columns) and subtract the same from each Δ".
  - File as a FOLLOWUP owned by the next compare-cost FIX.

### Nit
- **N-1.** `design/FOLLOWUPS.md` F3 (`compare-cost-wsh-cost-self-check`, step 1) says "FIX rows 1-9 fail on a double
  count". Row 8 (`v5_p2wsh_input_never_cheaper_than_p2wpkh_input`, `cli_compare_cost_wsh_witness_script.rs:168`,
  `row.1 >= 68`) is a lower bound and passes on a double count. It should read "rows 1-7 and 9".
- **N-2.** `SPEC_compare_cost_v0_26_0.md:119` ("`witness_size()` … that's what §4 reads") and `:186` ("we use
  `witness_size()`") are now stale for wsh, where §4 reads `witness_weight_wu`.
- **N-3.** The `41-mnemonic.md` notes-catalog row for the new note names only `--descriptor sh(wsh(M))`. A
  `sh(wsh(..))` descriptor on stdin gets the note too (verified above).
- **N-4.** For a `sh(wsh)` input, the output prints "subtract 35 vB from each Δ" and then "Δ values are correct" one
  line later. The rounding note predates this diff; reword it when M-1 is ruled.

## Note for s2 (not a finding)
The `varint_len` → 0/1 survivors (`enumerate.rs:295`) sit on the varint-3 path, which no test reaches. My Core spend
above gives an oracle: `compare-cost --miniscript 'multi(2,A,B,C,D,E,F,G,H,I,J,K,L)' --json` prints 66 conditions,
all `wsh_vbytes` 182, `tr_vbytes` 188 and `delta_vbytes` 6. Core spends it at 560 witness WU (181 vB, +1 for two
73-byte dummy sigs). The run is fast and under the default cap.

C / I / M / N: 0 / 0 / 1 / 4
