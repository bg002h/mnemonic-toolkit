# Triage E9: compare-cost refuses valid miniscript with a multi-letter wrapper chain

Lead: grade-2026-11-03-tk-v5-v7.md line 74 (E9). Brief: `briefs/triage-e9-e10.md`, section E9.
Repo: mnemonic-toolkit at origin/master 12e013bc, branch `triage/e9-wrapper-chain`,
worktree `/scratch/code/shibboleth/tk-triage-e9`. The worktree did not exist when I started; I created it
with `git worktree add -b triage/e9-wrapper-chain /scratch/code/shibboleth/tk-triage-e9 origin/master`.
RED test commit: 3a2ceced. Cargo 1.85.0 (matches `rust-toolchain.toml`). Raw logs, patch, `dk.py`, `matrix.sh`/`matrix.out` (the 16-shape matrix): `/scratch/cache/triage-e9/`.

## Verdict: CONFIRMED (a toolkit defect, not an rust-miniscript one)

- Fails on unmodified code for the reported reason: `crates/mnemonic-toolkit/tests/cli_compare_cost_wrapper_chain.rs`,
  11 rows, **5 passed, 6 failed** on 12e013bc (`red-unmodified.log`). Each failure is exit 2,
  `compare-cost: parse error: unknown wrapper «X:»`.
- Passes with the fix applied as a scratch change: **11 of 11 pass** (`green-with-fix.log`). The wider existing
  suites (`binary(/cost/)`, `test(/cost::/)`, `bughunt_s4`) also pass with it: 87 of 87 including the 11
  (`existing-with-fix.log`). The patch is `/scratch/cache/triage-e9/e9-scratch-fix.patch`; I reverted it.
  `git status` shows only the new test file and `git diff` is empty.

## Q1. Is the parse wrong? Yes. rust-miniscript at the pin accepts it; the toolkit mangles the input first

Executed evidence for `or_i(pk(A),ln:pk(B))` (A, B = compressed keys KA, KB):

| path | result |
|---|---|
| rust-miniscript at the pin, `Miniscript::<DefiniteDescriptorKey, Segwitv0>::from_str` and `<..., Tap>::from_str` (test row `oracle_rust_miniscript_parses_every_shape_in_both_contexts`) | Ok in both contexts, all 5 shapes |
| `compare-cost --descriptor 'wsh(or_i(pk(KA),ln:pk(KB)))'` | exit 0, table printed (60/86 vB, +26, two rows) |
| `export-wallet --descriptor 'wsh(or_i(pk([b8688df1/48'/0'/0'/2']xpubA/<0;1>/*),ln:pk([28645006/48'/0'/0'/2']xpubB/<0;1>/*)))' --format bitcoin-core` | exit 0, descriptor round-tripped with `ln:` intact |
| `compare-cost --miniscript 'or_i(pk(A),ln:pk(B))'` | **exit 2**, `unknown wrapper «9:»` |
| `compare-cost --miniscript 'or_i(pk(KA),ln:pk(KB))'` (concrete hex keys, no labels at all) | **exit 2**, same text |
| `echo 'or_i(pk(A),ln:pk(B))' \| compare-cost` (stdin path) | **exit 2**, same text |

BIP-379 (raw text saved to `/scratch/cache/triage-e9/bip379/bip-0379.md`, lines 57-61) confirms the syntax:
"The colon is dropped between subsequent wrappers; e.g. `dv:older(144)`". So `ln:` is a normal chain.

## Q2. Where is the bug

`crates/mnemonic-toolkit/src/cost/translate.rs`, `collect_abstract_labels` (:156). Its scan at :161
(`if c == '(' || c == ','`) reads the identifier that follows (:162-174). The only guard against taking a wrapper
for a label is the `KEYWORDS` list (:181-227), which holds the ten single-letter wrappers (`a s c t d v j n l u`,
:213-222) and no chain. So `ln` (after the `,`) is not a keyword and not hex/numeric, and is pushed as a label
(:236). `translate_miniscript` (:47) then runs `rewrite_labels_with` (:63, :68; :245) and `replace_whole_word`
(:258), which swap `ln` for that label's 66-hex dummy key. The string handed to `Miniscript::from_str` (:82) is
`<66-hex>:pk(<hex>)`. The toolkit's `--descriptor` path (`cost/strip.rs`, `Descriptor::from_str`) never runs this scan.

Why only chains of two or more letters: single letters are in `KEYWORDS`. Why only after `(` or `,`: a chain at the very
start of the string is never scanned, so `tv:pk(A)` and `ln:pk(B)` as the whole input parse.

## Q3. Is the garbled «9:» a separate defect? No: it is a symptom of the same defect

The «X:» is a hex digit of the dummy key that replaced the chain. rust-miniscript walks the wrapper string
back to front (`vendor/miniscript/src/miniscript/mod.rs:1044`, `for ch in frag_wrap.bytes().rev()`) and reports the first
byte that is not a wrapper letter, as a single char (`:1056`, `x => return Err(Error::UnknownWrapper(x.into()))`;
text at `vendor/miniscript/src/lib.rs:513`).

I computed each label's dummy key independently of the toolkit (pure-Python secp256k1 over
`sha256("compare-cost-dummy-key:<label>:0")`, script `/scratch/cache/triage-e9/dk.py`) and took the last hex digit:

| chain/label | last digit | observed text |
|---|---|---|
| `ln` | 9 | `unknown wrapper «9:»` |
| `sdv` | 4 | `unknown wrapper «4:»` |
| `snl` | b | `unknown wrapper «b:»` |
| `tv` | 0 | `unknown wrapper «0:»` |
| `sj` | f | `unknown wrapper «f:»` |
| `vc` | 9 | `unknown wrapper «9:»` (`and_v(vc:pk_k(A),pk(B))`) |

When the last digit is itself a wrapper letter the message changes shape, which also shows the text is not a message the
toolkit chose: `ac` ends in `a`, giving `typecheck: fragment «a:pk_k(KEY)» cannot wrap a fragment of type K`; `dv` ends in `c`, giving
`typecheck: fragment «c:older(10)» cannot wrap a fragment of type B`. The text vanishes with the root cause (GREEN run above).
The terseness is rust-miniscript's `UnknownWrapper(char)` at the pin (upstream shape); I do not propose a separate
toolkit item for it.

## Corrections to the lead's wording (they change what a fix must test)

1. **Not all the lead's examples are valid miniscript.** Through the oracle path, `or_i(pk(A),sdv:pk(B))`,
   `or_i(pk(A),snl:pk(B))` and `and_v(tv:pk(A),pk(B))` are rust-miniscript **typecheck failures** (`dv:pk(..)`: "needs to be `z`, needs to consume zero elements"; `and_v(tv:pk(..),pk(..))`:
   "cannot accept children of types B and B"). The toolkit's exit 2 is the right outcome there and only
   the reason is wrong. The chains themselves are valid in other shapes; the RED rows use:
   `or_i(pk(A),ln:pk(B))`, `or_i(pk(A),tv:pk(B))`, `and_b(pk(A),sdv:older(10))`, `and_b(pk(A),snl:older(10))`,
   `and_b(pk(A),sj:pk(B))`. All five exit 0 on `--descriptor`, valid in Segwitv0 and Tap.
   Row `control_invalid_chain_still_refused` pins that an invalid chain stays refused.
2. **`tv:` is context-dependent.** `tv:pk(A)` alone parses (exit 0); the chain fails only after a `(` or `,`.
3. **Wider than "wrapper chain of two or more letters" after a comma:** any such chain after `(` or `,`, in `--miniscript`
   and the stdin form, with abstract labels or concrete hex keys alike.
4. A few valid-in-wsh shapes exit 3 on `--descriptor` for a different, correct reason (Tap malleability), e.g.
   `thresh(2,pk(A),s:pk(B),sj:pk(C))`. Not part of this defect.

## Fix note (proposal only; no production code committed)

Scratch patch (4 lines, `e9-scratch-fix.patch`): in `collect_abstract_labels`, after the `buf.is_empty()` check (:175-177),
`continue` when the next char is `:`. It turns the 6 RED rows GREEN.

**It is incomplete for one case I measured:** a user label spelled like a chain. With the patch applied,
`compare-cost --miniscript 'and_b(pk(sdv),sdv:older(10))'` still exits 2 `«4:»`, while the equivalent concrete
`--descriptor wsh(and_b(pk(<key for sdv>),sdv:older(10)))` exits 0. `replace_whole_word` (:258) also replaces the chain
occurrence of a known label. A complete fix must make that function skip a hit followed by `:`. No RED row for this
case is in the test file (it needs a label equal to a chain, and the lead did not ask); it is a one-line add for the fix PR.

## Impact facts for grading

- Fails closed: exit 2, 0 bytes on stdout (measured), no wrong figure.
- Workaround exists: `--descriptor 'wsh(<miniscript with concrete keys>)'` computes the same figures (the RED rows
  assert the `--miniscript` figures equal the `--descriptor` oracle's).
- The refusal text points the user at the wrong cause (a wrapper "unknown" that is a valid, documented one).
- I did not check whether the GUI compare-cost panel or the manual claim wrapper-chain support. The v0_26_0 SPEC
  restricts nothing here and says `--miniscript` takes a "bare miniscript fragment".

## Open questions

- **OQ-1 (grade).** My brief does not quote the likelihood test, so I did not apply it and propose no C/I/M/N grade.
  The grader's own line 80 refers to "condition (1)" of it without the text. The facts above (fails closed, workaround
  exists, valid input refused, misleading reason) are what a grade would turn on. Under the plain severity words in
  `CLAUDE.md` it is a "real defect, missing case"; whether the likelihood test lowers a fail-closed refusal to Minor is
  the coordinator's call.
- **OQ-2 (owner).** E9 was filed "not graded here". A fix PR needs an owner phase; I filed nothing.

## Not done

Did not audit other label-scan inputs beyond those listed; did not run the full toolkit suite or manual lint; did not
touch `.github/` or the manual. The scratch fix is not committed.

Counts: verdict CONFIRMED; proposed C/I/M/N 0/0/0/0 (grade withheld, OQ-1)
