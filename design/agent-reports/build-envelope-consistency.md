# Build: import-json envelope consistency (E1, E2a, E2b, E3)

Branch `fix/import-json-envelope-consistency`; base `origin/build/v7-bundle-recovery` (PR #135 not yet in master; its head 93ea9667 merged in, which adds only a report file). Graded Important by the coordinator.

## RED / GREEN
- RED on unmodified code (base 8eefb841): 13 rows, 12 FAIL, 1 PASS (honest control).
- GREEN: 13 of 13 pass. Full workspace `cargo nextest run --locked --workspace`: 4264 passed, 38 skipped (re-run after the merge: same). fmt --check and clippy -D warnings clean.
- The 11 tests-that-change applied as the architect report lists.

## Proof script `scripts/prove-envelope-consistency.sh` (19 cases, both commands)
- Pre-change binary (built from 8eefb841): TOTAL 4 passed, 15 failed.
- Head: TOTAL 19 passed, 0 failed.
- The v0.27.0 as-released envelope is a legacy bare P2SH, so export-wallet is checked for descriptor, bitcoin-core and bip388 only (sparrow/coldcard/electrum have no template for it, a pre-existing refusal).

## Gate
- `make audit` in docs/manual: rc 2, 10 flag-coverage failures about `mk-cli after 0.13.0` markers in 44-mk-cli.md. Identical 10 failures on the unmodified base (8eefb841); not caused by this change, not touched here. Needs an owner (mk-cli pin bump to 0.14.0 in the manual).
- Mutation gate (54 mutants, 10 min): caught 48, MISSED 6.
  - 5 survivors are in bundle.rs (`plain_multisig_threshold` guards at 1272, 1323; `build_unified_card` `>` at 1384, three operator variants): code from PR #135, not this change.
  - `json_envelope.rs:546:50 replace || with && in check_envelope_consistency`: NOT equivalent. The E2b slot comparison (key, fingerprint, path) is only exercised by rows where all three fields differ (swapped card) or where E1 refuses first (descriptor-side path edit), so a mismatch in only one field is untested. A row that changes only the mk1 card's origin path would kill it. I did not add one: the brief fixes the 13 rows and the name is a product choice. Needs a decision (suggest filing owned by this PR).

## Not done
- `--self-check` default unchanged; FOLLOWUPS `deferred-minor` filed.

Counts: 0 Critical, 0 Important, 1 Minor (E2b single-field survivor).
