# Fold 2, FIX-V7: export-wallet --from-import-json takes K from the emitted descriptor

Branch build/v7-bundle-recovery, draft PR #135. Fold commit f0987c0a on 4e111746. 0 Critical / 0 Important open from this work.

## Defect
`run_from_import_json` decoded K from `envelope.bundle.md1` (`.ok()` turning a decode error into None) while the script came from `envelope.bundle.descriptor`. A hand-edited envelope whose md1 and descriptor disagree exported the md1's K on the descriptor's keys.

## What changed
- `crates/mnemonic-toolkit/src/cmd/export_wallet.rs`: K is now computed from `canonical_descriptor_body` (the body `parsed_ms` is parsed from, so the same descriptor that is emitted). It goes through `wallet_import::pipeline::concrete_keys_to_placeholders` and `parse_descriptor`, exactly as `bundle --import-json` encodes it, then `crate::cmd::bundle::plain_multisig_threshold(&tree)`. The rule is reused, not duplicated. md1 and the threshold field are not consulted. An encode failure at either step is `ToolkitError::BadInput` ("cannot encode the envelope descriptor to derive K"), with no fallback to None.
- `crates/mnemonic-toolkit/tests/cli_bundle_recovery_threshold_text.rs`: new row `v7_export_wallet_k_ignores_disagreeing_md1`. The envelope's descriptor is a plain 3-of-3 sortedmulti; its md1 is replaced by the 2-of-3 md1 and its threshold field is 2. Sparrow, coldcard and electrum output must be byte-identical to the honest 3-of-3 envelope's, and the sparrow script must contain `sortedmulti(3,` and not `sortedmulti(2,`.
- No product decision was needed. FIX doc and SPEC were already amended in 4e111746.

## RED / GREEN
- RED on 4e111746 (output contains `Compiling mnemonic-toolkit`): `Summary [0.019s] 1 test run: 0 passed, 1 failed, 4144 skipped`; failed at the sparrow byte comparison.
- GREEN with the change: this row and `v7_export_wallet_takes_k_from_envelope_descriptor` both pass (`2 tests run: 2 passed`).

## Gate (files in /scratch/cache/tk-v7-fold2/)
- `cargo fmt --all -- --check`: clean.
- `cargo clippy --locked --all-targets -- -D warnings`: clean.
- `cargo nextest run --locked --workspace`: `Summary [4.354s] 4219 tests run: 4219 passed, 38 skipped`.
- FIX-V7 section 4 command as printed: `Summary [0.049s] 30 tests run: 30 passed, 4115 skipped`.

## Mutation gate (`mutation-gate.sh tk-build-v7 12e013bc`, fold head f0987c0a)
`caught 38, MISSED 5, timeout 0, unviable 0 (pass 1 exit 2)`, 9 min, 43 mutants. Out dir: /scratch/cache/tk-v7-fold2/mut/ (I set `MUTATION_GATE_OUT`).

Missed list; none is in `export_wallet.rs`, so the fold's own new code has no survivor. All five are in code from earlier V7 commits, not touched by this fold:
```
bundle.rs:1272:34: replace match guard c.len() == 1 with true in plain_multisig_threshold::only_child
bundle.rs:1384:32: replace > with < in build_unified_card
bundle.rs:1384:32: replace > with >= in build_unified_card
bundle.rs:1323:14: replace match guard leaf.tag != Tag::TapTree with true in plain_multisig_threshold
bundle.rs:1384:32: replace > with == in build_unified_card
```
(paths under crates/mnemonic-toolkit/src/cmd/)
