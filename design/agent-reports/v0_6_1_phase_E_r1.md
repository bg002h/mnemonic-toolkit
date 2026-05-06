# v0.6.1 Phase E code review — r1 (reviewer: feature-dev:code-reviewer)

**Verdict:** APPROVED 0 Critical / 0 Important.

## Fixture verification

- `TREZOR_24_MASTER_FINGERPRINT = "5436d724"` — confirmed by `derive.rs:79` unit test (`derive_master_fingerprint_stable`); pre-existing canonical.
- `TREZOR_24_ZERO_MS1_24WORD = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcwugpdxtfme2w"` — pinned across every v0.1 and v0.2 bundle test vector file.
- `SAMPLE_WIF_SENTINEL_FINGERPRINT = "751e76e8"` — correct hash160 of the compressed generator pubkey; first 4 bytes = the bitcoin genesis-address fingerprint.
- `TREZOR_24_BIP84_MAINNET_XPRV` — new in this diff; verified transitively by the passing `entropy_to_xprv_bip84_mainnet` test against the canonical BIP-84 derivation.

## Code-path analysis

- `entropy_to_xpub_bip84_mainnet` vs `phrase_to_xpub_bip84_mainnet` — both hit the `Phrase | Entropy` arm but distinct entries (`hex::decode` vs `Mnemonic::parse_in`); not redundant.
- `entropy_to_fingerprint_bip84_mainnet` — emits `derived.master_fingerprint` per `convert.rs:519-524`; assertion against `5436d724` is correct (master fingerprint, not account-xpub fingerprint).
- `xprv_to_fingerprint_account_xpriv` — cross-check assertion against `xpub → fingerprint` for the matching xpub fixture proves the xprv path goes through the same account-xpub fingerprint computation.
- `wif_to_fingerprint_co_tested_with_wif_to_xpub` — compound `--to xpub,fingerprint`; co-tests two edges per architect L-3.

## FOLLOWUP coverage audit

The v0.6.0 `convert-test-coverage-tightening` FOLLOWUP enumerated 6 missing direct edges + 3 round-trip loops. Phase E delivers exactly those 6 + 3. The 2 deferral-message tests are explicitly dropped (correctly, since Phase B shipped `phrase/entropy → wif`). Coverage closure is complete.

## Lows fixed inline

- `round_trip_entropy_to_ms1_to_entropy` intermediate assertion tightened from `starts_with("ms10entr")` to byte-exact `assert_eq!(ms1, TREZOR_24_ZERO_MS1_24WORD)` (constant added to round-trips file).

## Lows deferred

- `TREZOR_24_BIP84_MAINNET_XPRV` lacks an external-source citation comment. Logged informally; not worth a FOLLOWUP entry (the value is verifiable by anyone running the canonical BIP-84 derivation against the trezor-24 zero-entropy seed).
- `TREZOR_24_ZERO_ENTROPY_HEX_64` is duplicated across `cli_convert_happy_paths.rs` and `cli_convert_round_trips.rs`. Standard Rust integration-test convention (each `tests/*.rs` is a separate crate; no shared module). Not a defect.

## Cleared for Phase E commit

`cargo test --workspace` reports all tests pass; +6 new direct-edge tests in `cli_convert_happy_paths.rs` + 3 round-trip tests in new `cli_convert_round_trips.rs`.
