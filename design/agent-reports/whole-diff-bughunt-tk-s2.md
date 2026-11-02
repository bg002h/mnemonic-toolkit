**⚠ 5 Important: survivors that remove FIX behaviour, each with a RED row that kills it.**

# Whole-diff bughunt tk G1+G2, slice 2 (Sonnet, tests-can-fail + survivor grading)

Head `ad1761b5`. All 35 missed entries (31 distinct mutants; the tool lists some twice) were re-applied by sed in a scratch worktree (detached at ad1761b5, removed after) and run against the RED rows below. Each row PASSES on real code and FAILS on the mutant (observed). Rows live in scratch files `tests/s2_red.rs`, `gate.rs` unit test `s2_red_gate_counts_every_hash_kind`, and two rows appended to `tests/bughunt_s1.rs`; their text is below.

## Important

**I1. repair.rs:1371 `^= m` -> `|= m` survives (2 entries). X3 (FIX corrected-share output) tests one substitution only.**
`bughunt_s5.rs` X3 (~line 330) corrects one symbol at one position; with `|=` the corrected symbol is wrong whenever `v & m != 0`, so most real errors yield a wrong "clean" share. Fix: replace the single case with a loop. RED row (run as printed in `s2_red.rs`):
`s2_red_x3_every_single_substitution_of_a_share_is_corrected` loops positions 4..len of `SHARE_1` x replacement chars `q p z x l`, runs `repair --ms1 -` with the damaged string on stdin, asserts exit 4 and stdout ends with `{SHARE_1}\n`.
Observed: real code PASS (0.45s); mutant `|=` FAIL.

**I2. repair.rs:1419-1425 `ms1_is_share` collapses to always-true (`-> true`, `sep + 1` -> `sep * 1`, `sep - 1`; 5 entries). No test asserts the seed-card VERIFY-ME text (advice lens).**
Any corrected seed card then prints the K-of-N share advice ("recombine it with the other shares") — wrong advice on a single-string seed card. Fix: add a RED row. `s2_red_corrected_seed_card_gets_seed_card_text`: damage `VALID_MS1` (position 21 -> 'x'), run `repair --ms1 -`, assert exit 4, stdout ends with `{VALID_MS1}\n`, stderr contains `a corrected seed card cannot be self-verified` and does not contain `K-of-N`.
Observed: real PASS; `-> true`, `sep - 1`, `sep * 1` all FAIL.

**I3. cost/enumerate.rs:444/445/446 (Hash256, Ripemd160, Hash160 arms deleted; 6 entries) and descriptor_builder/gate.rs:627/628/629 (3 entries). FIX #28 tests are sha256 only.**
With an arm deleted the repeated digest of that kind is not counted/deduped: compare-cost fails "no satisfying conditions" on real inputs (observed) and the gate cap diverges. Fix rows:
- `s2_red_28b_compare_cost_repeated_digest_all_kinds` (`s2_red.rs`): for hash256/ripemd160/hash160 run `compare-cost --miniscript 'or_i(and_v(v:pk(A),F(H)),and_v(v:pk(B),F(H))) --json`, assert exit 0 and labels `["A + preimage(h0)","B + preimage(h0)"]`, `--max-conditions 8` exit 0 and `7` exit 3.
- `s2_red_gate_counts_every_hash_kind` (gate.rs unit): for each kind build `{"and_v":[{"wrap":{"w":"v","sub":{"<k>":"<h>"}}},{"pk":A}]}`, assert `validate_with_cap(.,4)` ok and `(.,3)` errs with `OverEnvelope`.
Observed: real PASS both; each of the six enumerate/gate arm deletions FAIL (enumerate: compare-cost row; gate: unit row).

**I4. wallet_import/coldcard_multisig.rs:485 `c.len() == 1 && ...` -> `||` survives. FIX #19 refusal of non-`m/45'` single-component paths untested.**
With `||` any one-component path (e.g. `m/44'`) is imported as BIP-45. Fix row `s2_red_19c_single_component_non_45_path_refused`: blob with `Derivation: m/44'`, `Format: P2SH`, two cosigners, `import-wallet --blob - --format coldcard-multisig`; assert exit 2 and stderr exactly `error: import-wallet: coldcard-multisig: parse error: derivation path has only 1 components; need ≥2 for BIP-48 coin-type inference\n`.
Observed: real PASS; mutant FAIL.

**I5. verify_bundle.rs:2847 (`(*slot as usize) < n` -> `>`) and 2857 (`unplaced_groups += 1` -> `*= 1`). FIX #12 rules untested.**
2857: a stray card no longer blames any slot (slot 1 stays "not supplied" instead of "xpub absent from descriptor policy"); #12 only asserts slots 0 and 2. 2847: scan fallback never places cards, so cards supplied out of slot order all fail. Fix rows (appended to `tests/bughunt_s1.rs`):
- `s2_red_multisig_scan_places_cards_out_of_slot_order`: for k in 1..=8 build the #13-style wallet, `groups.reverse()` (supplied C,B,A), assert every `mk1_decode[i]` passes.
- `s2_red_multisig_stray_card_blames_xpub_absent_on_the_empty_slot`: the #12 setup (A, stray, C); exactly one `mk1_decode[i]` fails and its detail contains `absent from descriptor policy`.
Observed: real PASS both; 2847 mutant FAILs the first row (the original unreversed order did not kill it, so reversal is required); 2857 mutant FAILs the second.

## Equivalent (no action)
- repair.rs:1401 `check_is_share -> Ok(())` (2 entries): redundant with the downstream check.
- verify_bundle.rs:2348, 2355, 2395, 2401, 2414, 2440 "delete field passed": `VerifyCheck::default().passed == true`, the deleted value was `true`.

## Minor (one line each)
- M1 verify_bundle.rs:2837 `gi < n` -> `>`: differs only when the policy repeats an xpub (the scan fallback finds the same slot otherwise); guard is redundant with `pubkeys.get(gi)`.
- M2 gate.rs:656 `1 + rel_b + rel_t` -> `-`: pre-existing, not a FIX rule; untested relative-timelock count.
- M3 verify_bundle.rs:2413 (name), 2415, 2434, 2441 (detail), 2442 (decode_error): moved `ms1_card_checks` text, untested on the new #37 call site (failure branches); add assertions on `ms1_decode` detail for a damaged ms1 in the single-sig template test.

## Nits
None.

0 / 5 / 3 / 0
