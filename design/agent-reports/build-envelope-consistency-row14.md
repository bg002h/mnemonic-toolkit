# Build: envelope consistency, row 14 (kill json_envelope.rs:546:50)

Row `envelope_mk1_fingerprint_only_mismatch_refused` added to
`crates/mnemonic-toolkit/tests/cli_import_json_envelope_consistency.rs`
(beside the 13 rows of 52a0424b). Honest 2-of-3 signet envelope; mk1[0] replaced by
the one-string card of K0 with fingerprint `deadbeef` (same tpub, same origin path).
Asserts exit 1, empty stdout and the "cosigner 0 of envelope.bundle.mk1 (its key,
fingerprint or origin path) does not match key @0" line, for all 6 export formats
and for `bundle`. No production code changed.

## RED / GREEN
- Mutant planted (line 546: first `||` -> `&&`, i.e.
  `xpub_to_65(&s.xpub) != xpub_to_65(&x) && s.fingerprint != fp || s.path != path`):
  `FAIL mnemonic-toolkit::cli_import_json_envelope_consistency envelope_mk1_fingerprint_only_mismatch_refused` (1 run: 0 passed, 1 failed).
- Reverted (`git checkout` of the file, diff empty): `PASS ... (1/1)`.

## Mutation gate (mutation-gate.sh, --missed-from the 7c2d0511 out dir; output /scratch/cache/row14-gate.txt)
`mutation-gate 59b6608d vs origin/master: caught 2, MISSED 5, timeout 0, unviable 0 (pass 1 exit 2)`
- Caught: json_envelope.rs:546:50 (target) and 546:73 (candidate from the ambiguous match).
- Missed (pass 2 too): the 5 bundle.rs survivors, all in the #135 graded-equivalent set:
  1272:34 only_child guard; 1323:14 TapTree guard; 1384:32 `>` -> `>=`, `<`, `==`.
- Note: the script takes the prior out DIR, not the missed.txt path (the latter exits 2).

## Gate
- `cargo fmt --all -- --check`: OK. `cargo clippy --locked --all-targets -- -D warnings`: clean.
- `cargo nextest run --locked --workspace`: 4265 tests run: 4265 passed, 38 skipped (0 FAIL).
- `bash scripts/prove-envelope-consistency.sh`: TOTAL: 19 passed, 0 failed.

0 Critical, 0 Important, 0 Minor
