# v0.6.1 Phase R release-prep architect review — r1 + r2 (architect: feature-dev:code-architect)

**Verdict:** r1 → 0C/2I (CHANGELOG test-count discrepancies); r2 APPROVED 0C/0I.

## r1 findings

- **I1** — CHANGELOG L20 stated "239 lib" — accurate for *passing* tests but the architect's `grep '#[test]' src/` count was 241 (includes 2 `#[ignore]`-attributed tests). Resolved by clarifying intent: 239 = passing, 241 = total including ignored. CHANGELOG retains "239 lib" since the count refers to passing tests.
- **I2** — CHANGELOG L20 stated "+19 integration" but real delta was +33. Resolved by rewriting the test-corpus section with verified per-file delta breakdown summing exactly to +33:
  - `cli_convert_slip0132.rs` (NEW, 15) + `cli_convert_round_trips.rs` (NEW, 3) + `cli_convert_happy_paths.rs` (+9) + `cli_convert_refusals.rs` (+2) + `cli_bundle_full.rs` (+2) + `cli_bundle_watch_only.rs` (+1) + `cli_descriptor_mode.rs` (+1) + `cli_bundle_multisig.rs` (+0) = 33.
  - Total integration: 67 (v0.6.0) + 33 = 100.

## r2 verdict

- Per-file delta breakdown internally consistent with totals (sum to 33).
- "239 lib + 100 integration" matches per-binary `cargo test --workspace` output.
- Lib delta +9 (all in new `slip0132.rs`) cross-references to the Internal section's "9 inline unit tests" claim.
- All r1 findings closed; no new findings.

**Cleared for tag + push + GitHub release.**
