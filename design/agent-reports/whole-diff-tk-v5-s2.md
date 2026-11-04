**⚠ 1 Important (survivors that remove V5's witness-weight terms); 0 Critical.**

# Whole-diff review, toolkit V5 (+E9) build, slice 2 (tests-can-fail)

Target `/scratch/code/shibboleth/tk-build-v5`, branch `build/v5-wsh-cost`, head d8abb6f8 (diff `12e013bc..d8abb6f8`). Reviewer: Sonnet 5.5, high. First round of slice 2, not a re-review. Target worktree untouched (only this report added). All mutants, plants and new tests ran in a reflink copy of the head, `/scratch/cache/whole-diff-tk-v5/s2/head` (logs `mut-m*.log`, `plant-p*.log`, `mut-results.txt`, `plant-results.txt` beside it). cargo-mutants was not run; each of the 12 listed survivors was applied by hand to the scratch copy at its reported line:col (the tool listed them; I only replayed them to run a kill test). Gate on head, my scratch copy, before any change: the four compare-cost test binaries, 85 tests run, 85 passed.

## Summary of the 12 survivors

| # | survivor (enumerate.rs / translate.rs) | verdict | killed by |
|---|---|---|---|
| 1 | 287:25 `+`->`*` (`tw * vl(sl)`, drops the `vl(sl)` term) | real gap, I-1 | `v5_pin_wsh_pk_witness_wu_exact` (110 -> 109) and the 7-of-10 pin |
| 2 | 287:50 `+`->`*` (`vl(sl) * sl`, same loss) | real gap, I-1 | same two |
| 3 | 287:63 `-`->`+` (+2 WU) | real gap, I-1 | same two |
| 4 | 287:63 `-`->`/` (+1 WU) | real gap, I-1 | same two |
| 5 | 288:17 `+`->`*` (-1 WU) | real gap, I-1 | same two |
| 6 | 295:5 `varint_len` -> 0 | real gap, I-1 | same two |
| 7 | 295:5 `varint_len` -> 1 | real gap, I-1 | the 7-of-10 pin only (a 343-byte script has a 3-byte varint; pk(A) cannot see it) |
| 8 | 288:36 `elems + 1` -> `elems * 1` | EQUIVALENT | none possible, proof below |
| 9 | 288:36 `elems + 1` -> `elems - 1` | EQUIVALENT | none possible, proof below |
| 10 | translate.rs 271:19 `+`->`*` | real gap, M-1 | `v5_pin_replace_whole_word_label_at_end_of_input` |
| 11 | translate.rs 271:19 `+`->`-` | real gap, M-1 | both translate pins |
| 12 | translate.rs 271:36 `==`->`!=` | real gap, M-1 | both translate pins |

Equivalence proof for #8 and #9: `varint_len(n)` is 1 for n <= 252 and 3 for 253..=65535. `elems` is `plan.witness_template().len()`, the stack-element count of one planned wsh spend. `varint_len(elems + 1)` differs from `varint_len(elems)` only at elems = 252, and from `varint_len(elems - 1)` only at elems in {252, 253}. elems is at least 1 (`--miniscript 'older(10)'` and `and_v(v:older(10),older(12))` both exit 2 "All spend paths must require a signature"), so `elems - 1` cannot underflow. elems is at most 100: `Wsh::new` runs `Segwitv0::top_level_checks` (vendor/miniscript/src/miniscript/context.rs:324, :547-560), which caps witness items at `MAX_STANDARD_P2WSH_STACK_ITEMS` = 100, and the script is capped at 3600 bytes (a 99-branch `thresh(1,pk,s:pk,...)` already hits "cannot be larger than 3600 bytes"). So for every reachable input both mutants return the original value. (Consequence, Nit N-1: the pair `- varint_len(elems) + varint_len(elems + 1)` nets to 0 for every input the tool accepts.)

## Plants (each FIX fix's defect put back; RED row fails for the stated reason)

| plant | rows that fail | stated reason seen |
|---|---|---|
| p1: `witness_weight_wu` -> `plan.witness_size()` (V5 Part A reverted) | 10: `v5_wsh_pk/or_d/multi_2of3/multi_3of5/and_v/thresh_2of3_vbytes_match_core`, `v5_descriptor_wsh_pk_vbytes_match_core`, `v5_p2wsh_input_never_cheaper_than_p2wpkh_input`, `v5_delta_matches_core`, `v5_delta_negative_for_multi_7_of_10` | `Core spends wsh(pk) as 273 WU = 69 vB; got [("A", 60, 75, 15)]`; 7-of-10 `got [81, 81, ...]`. Matches FIX §3 rows 1-9, 30 exactly. |
| p2: sh(wsh) note -> `None` | `v5_sh_wsh_note_in_json_notes`, `v5_sh_wsh_note_in_text_note_lines` | `notes[] must contain the ruling-4 sh(wsh) note exactly; got [...]` (the two existing notes). Rows 27, 28. |
| p3a: drop the `collect_abstract_labels` guard | `miniscript_path_accepts_ln_chain_with_concrete_keys` only | exit 101, not a refusal: debug_assert `segv0 + tap key walks must yield labels in the same order` (enumerate.rs:432). See M-2. |
| p3b: drop the `replace_whole_word` guard | `miniscript_path_accepts_label_spelled_like_chain` only | `got exit 2 stderr="error: compare-cost: parse error: unknown wrapper «4:»"`. Row 26. |
| p4: emit the sh(wsh) note for every input (mod.rs) | `v5_sh_wsh_note_absent_for_wsh_tr_and_miniscript` | `--descriptor wsh(pk(...)): the sh(wsh) note must not appear; got ["input had a p2sh wrapper, ...`, at `cli_compare_cost_v5_fix.rs:90:9`. Row 29. |

All five plants fail their rows; none passes. Rows 17-21 (the five abstract-key chain shapes) do not fail under either E9 half alone (p3a, p3b), because each half masks the other; they only fail when both guards are gone (the FIX evidence shows them red on base).

## Findings

### I-1 Important: the exact witness-weight terms of V5 are unpinned; 7 of 12 survivors are real (enumerate.rs:287-288, :295)

The V5 rule (doc comment enumerate.rs:272-277, FIX Appendix A) is exact in WU: add `varint(script) + script` and re-encode the stack-count varint. The only tests that exercise it read vB, `ceil((164 + WU)/4)`, which hides 1-2 WU. Rows 1-9 pin Core vB; `v5_delta_negative_for_multi_7_of_10` pins a sign. So a mutant that drops the length-varint byte (#1, #2, #6, #5), adds 1-2 WU (#3, #4), or hard-codes a 1-byte varint (#7) passes all 85 tests. Mutant #7 is a user-visible defect class: a witnessScript of 253 bytes or more (every multi(k,n>=8), e.g. the 7-of-10 row the FIX adds to show the sign flip) would be 2 WU short; the test's own comment says "wsh 256 vB, tr 251 vB, so Δ = -5" but the assertion `all(d < 0)` cannot see it, and even a vB pin would not (859 WU and 857 WU both print 256 vB). Per the brief a survivor that removes a FIX behaviour is at least Important.

Fix text: add these two unit tests to a `#[cfg(test)]` module at the end of `crates/mnemonic-toolkit/src/cost/enumerate.rs`. They are unit tests because `witness_weight_wu` is private and `cost` is only `pub` under `cfg(fuzzing)` (lib.rs:156-157). Hand derivation, independent of the code: pk(A) = stack count 1 + (1 + 72 dummy sig) + (1 + 35 script) = 110 WU (Core measures 109 with a 71-byte sig); multi(7, 10 keys) script = 1 + 10*34 + 1 + 1 = 343 bytes, so witness = count 1 + empty dummy 1 + 7*(1 + 72) + (3 + 343) = 859 WU.

```rust
#[cfg(test)]
mod v5_wu_pin_tests {
    use super::*;
    use crate::cost::translate::{build_tr_descriptor, build_wsh_descriptor, translate_miniscript};

    /// Exact `wsh_witness_bytes` (WU) of every row of `--miniscript <m>`.
    fn wsh_wu(m: &str) -> Vec<usize> {
        let t = translate_miniscript(m).expect("translate");
        let w = build_wsh_descriptor(t.segv0.clone()).expect("wsh");
        let tr = build_tr_descriptor(t.tap.clone()).expect("tr");
        enumerate_minimal_conditions(&t, &w, &tr, 10_000)
            .expect("enumerate")
            .rows
            .iter()
            .map(|r| r.wsh_witness_bytes)
            .collect()
    }

    /// wsh(pk(K)): stack count 1 + (1 + 72 dummy sig) + (1 + 35 script) = 110 WU.
    /// Core signs the same spend at 109 (71-byte low-R sig, 1 fewer).
    #[test]
    fn v5_pin_wsh_pk_witness_wu_exact() {
        assert_eq!(wsh_wu("pk(A)"), vec![110]);
    }

    /// multi(7, 10 keys): script = 1 + 10*34 + 1 + 1 = 343 bytes, so its length
    /// varint is 3 bytes (>= 253). Witness = count 1 + empty dummy 1 +
    /// 7*(1 + 72) + (3 + 343) = 859 WU for each of the C(10,7) = 120 signer sets.
    #[test]
    fn v5_pin_wsh_multi_7_of_10_witness_wu_exact_three_byte_varint() {
        let got = wsh_wu("multi(7,A,B,C,D,E,F,G,H,I,J)");
        assert_eq!(got.len(), 120);
        assert!(got.iter().all(|w| *w == 859), "got {got:?}");
    }
}
```

RED, run as printed on each mutant (apply the mutant at the listed line:col to the head, then):
`PATH=$HOME/.cargo/bin:$PATH cargo nextest run --locked -p mnemonic-toolkit --no-fail-fast -E 'test(/v5_pin_/)'`

Observed (head GREEN: `4 tests run: 4 passed` with the two translate pins below; mutants):

| mutant | result |
|---|---|
| 287:25 `+`->`*` | 2 failed: `v5_pin_wsh_multi_7_of_10_witness_wu_exact_three_byte_varint`, `v5_pin_wsh_pk_witness_wu_exact` |
| 287:50 `+`->`*` | same 2 failed |
| 287:63 `-`->`+` | same 2 failed |
| 287:63 `-`->`/` | same 2 failed |
| 288:17 `+`->`*` | same 2 failed |
| 295:5 body `0` | same 2 failed |
| 295:5 body `1` | 1 failed: `v5_pin_wsh_multi_7_of_10_witness_wu_exact_three_byte_varint` only |
| 288:36 `*` and `-` | 0 failed (equivalent, see above) |

Because the 3-byte varint is only visible to the 343-byte test, keep both tests. The existing 85 tests still pass with the new ones added (`89 tests run` with the translate pins).

### M-1 Minor: translate.rs `replace_whole_word` end-of-input and word-boundary arms are unpinned (translate.rs:271, mutants #10-#12)

Pre-existing arithmetic of the `after_ok` guard (not a spec or FIX rule; the E9 change only ANDs a third term at :272), so Minor. A label at the very end of the input (`pk(A`) indexes `bytes[i + len]` out of bounds under `*`/`-`/`!=`; and under `!=` a label that prefixes a fragment name (`or` in `or_i(...)`) is wrongly substituted, because the longest-first ordering in `rewrite_labels_with` is the only other defence. Fix text: append inside `mod tests` of `crates/mnemonic-toolkit/src/cost/translate.rs`:

```rust
    #[test]
    fn v5_pin_replace_whole_word_label_at_end_of_input() {
        // A label that ends the input is still replaced (and never indexes
        // past the end).
        assert_eq!(replace_whole_word("pk(A", "A", "X"), "pk(X");
    }

    #[test]
    fn v5_pin_replace_whole_word_skips_fragment_prefix() {
        // `or` is a label here, and the prefix of the fragment `or_i`: only the
        // whole-word occurrence is replaced.
        assert_eq!(
            replace_whole_word("or_i(pk(or),pk(B))", "or", "X"),
            "or_i(pk(X),pk(B))"
        );
    }
```

RED (mutants applied, same command as above): 271:19 `*` -> 1 failed (`..._label_at_end_of_input`); 271:19 `-` -> 2 failed (both); 271:36 `!=` -> 2 failed (both). Head GREEN.

### M-2 Minor: the `collect_abstract_labels` half of E9 is pinned by one row, and fails as a panic

With the :178-181 guard removed (p3a) five of six chain rows stay green; only `miniscript_path_accepts_ln_chain_with_concrete_keys` fails, and it fails because the phantom label trips the I3 `debug_assert` (exit 101), which a release build would not fire (by reading: the phantom label would then flip `concrete_keys` to false and drop the "input had concrete keys" note; not run). Fails closed in a debug build and the suite is red, so Minor. Suggested row: assert the `notes[]` of `--miniscript 'or_i(pk(A),ln:pk(B))' --json` has no "concrete keys" entry and of the concrete-key form has one. Not run.

### M-3 Minor: row 30 comment promises more than it asserts (cli_compare_cost_v5_fix.rs:93-111)

Comment: "wsh 256 vB, tr 251 vB, so Δ = -5 for every one of the 120 signer sets ... 2-of-3 stays tr-dearer (Δ +4)". Assertions: `all(d < 0)` and `all(d > 0)`. I ran the binary: 7-of-10 prints 256/251/-5 for the sampled rows and 2-of-3 prints Δ +4 on all three. Tighten to `assert!(big.iter().all(|d| *d == -5))` and `== 4`, which pin the figures the comment states. (This still does not kill #7; the WU pins do.)

### N-1 Nit: enumerate.rs:287-288 nets `- varint_len(elems) + varint_len(elems + 1)` to zero for every accepted input

See the equivalence proof. Either drop the pair and the `elems` binding, or keep it and add one line to the doc comment saying it is zero below 252 elements (the stack-count varint stays one byte, since policy caps the stack at 100). Keeping it is harmless.

## Advice lens

New advice text in the diff: `SH_WSH_NOTE` (strip.rs:16-17) "each wsh figure is for bare wsh(M): add 35 vB per input (the scriptSig redeem-script push) and subtract 35 vB from each Δ". Checked against the Core table in `cli_compare_cost_wsh_witness_script.rs`: sh(wsh(pk(K0))) 413 WU / 104 vB vs wsh(pk(K0)) 273 WU / 69 vB: +140 WU = +35 vB per input; Δ = tr - wsh falls by 35. Correct for every `sh(wsh(M))` input (the scriptSig is the fixed 35-byte push of the 34-byte `0x0020<32>` redeem script). No command is printed; no card or seat binding involved. No finding. The note is only emitted for `sh(wsh(M))` (p4 shows row 29 fails if it leaks).

## Not covered

Manual `make lint` and the build report's "3 environmental FAILs" (settled, outside this lens); text and JSON transcripts of the manual (checked by the manual CI job, not nextest); the FIX document's prose; Core-measured numbers themselves (taken from the test header). The two equivalence proofs rest on the vendored miniscript's top-level checks, read in source, not on an exhaustive input sweep.

0 C / 1 I / 3 M / 1 N
