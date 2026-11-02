# Fold: architect rulings QT1-QT3 into FIX_convert_path.md

Outcome: all three STOPs ruled "proposal stands"; doc and test file updated; no STOP remains.

- STOP-1 (R4 before R3), STOP-2 (R4/R3 before 5.a), STOP-3 (R3 before R5): section 10 rewritten as ruled, with the final `run()` order. Section 3.5 code unchanged; the line after it now cites the ruling.
- Doc defect: STOP-2 example `--xpub-prefix vpub` (clap exit 64) replaced by `zpub`. No row had been built from it.
- Three guard rows added to `crates/mnemonic-toolkit/tests/cli_convert_path.rs` and to doc section 6: `r4_before_r3_xprv_to_xpub_path_and_account`, `r4_before_xpub_prefix_network`, `r3_before_r5_named_path_with_account`. Section 6's "avoid the overlaps" sentence replaced; section 8 and the build-impact line now say 30 rows.
- Unmodified production code (`/scratch/cache/convert-path/fold-red.txt`): 30 run, 2 passed (the two old guards), 28 failed. New rows fail for the stated reasons: r4_before_r3 exits 0 printing XPUB_84; r4_before_xpub_prefix_network exits 2 with the 5.a text instead of R4; r3_before_r5 exits 1 with the R5 text.
- Reference patch applied as a scratch change then reverted (`fold-green.txt`): 30/30 passed.
- Ambiguity, resolved by the ruling's explicit "(no `--network`)": its row text reads `--from P S ... (no --network)`, where `S` is `--network signet`. The row uses no `--network`. The tests-that-change list (section 7) is unchanged: none.
- Not folded (architect open points, not this doc): `--from xprv --to xpub --account 3` with no `--path` is silently accepted; needs its own triage.

Counts: Critical 0, Important 0; STOPs open 0.
