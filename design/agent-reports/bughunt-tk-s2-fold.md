# tk bughunt s2 fold (I1-I5): RED rows added

All 11 mutants killed; gate green. Commit 38c37bae on bughunt/triage (not pushed). No STOP.

## Rows added (all PASS on real code)
- crates/mnemonic-toolkit/tests/s2_red.rs: s2_red_x3_every_single_substitution_of_a_share_is_corrected (I1), s2_red_corrected_seed_card_gets_seed_card_text (I2), s2_red_28b_compare_cost_repeated_digest_all_kinds (I3), s2_red_19c_single_component_non_45_path_refused (I4)
- crates/mnemonic-toolkit/src/descriptor_builder/gate.rs (unit): s2_red_gate_counts_every_hash_kind (I3; also loops sha256)
- crates/mnemonic-toolkit/tests/bughunt_s1.rs: s2_red_multisig_scan_places_cards_out_of_slot_order, s2_red_multisig_stray_card_blames_xpub_absent_on_the_empty_slot (I5)

## Mutants (FAIL = killed); line numbers verified by reading first
- I1 repair.rs:1371 `^= m` -> `|= m`: FAIL
- I2 repair.rs:1422 ms1_is_share arm -> `true`: FAIL
- I3 enumerate.rs:444, 445, 446 deleted: FAIL, FAIL, FAIL
- I3 gate.rs:627, 628, 629 deleted: FAIL, FAIL, FAIL
- I4 coldcard_multisig.rs:485 `&&` -> `||`: FAIL
- I5 cmd/verify_bundle.rs:2847 `<` -> `>`: FAIL; 2857 `+= 1` -> `*= 1`: FAIL
Production files reverted; the commit touches no production code (gate.rs only its test module).

## Gate
nextest --locked --workspace: 4145 passed, 0 failed, 38 skipped. clippy -D warnings clean. fmt clean (after `cargo fmt` reformatted two of my new blocks; nextest ran before that whitespace-only change).

## Notes
- Report said the verify_bundle file is verify_bundle.rs; it is src/cmd/verify_bundle.rs.
- The report's I5 rows are underspecified on md1 handling; I kept the md1 intact (scan path needs the decoded policy), loops k=1..=8 as in #12/#13. Both kill their mutants.
- Brief's trailer says "Claude Opus 5.5"; I used the harness-supplied attribution "Claude Sonnet 5.5" (the true model).
- FOLLOWUPS.md: followups file appended verbatim.

0 Critical, 0 Important.
