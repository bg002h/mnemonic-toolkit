# v0.6.1 Phase B code review — r1 (reviewer: feature-dev:code-reviewer)

**Verdict:** APPROVED 0 Critical / 0 Important.

## Correctness checks passed

- `compressed: true` mandated by BIP-32 §4 and enforced (`convert.rs:504`).
- `network.network_kind()` propagates correctly to WIF version byte via `bitcoin::PrivateKey.network: NetworkKind` (verified against bitcoin-0.32.x source at `crypto/key.rs:402`).
- `passphrase` flows through to `Mnemonic::to_seed(passphrase)` inside `derive_bip32_at_path`; PBKDF2 IS traversed.
- Missing-path refusal returns `ToolkitError::ConvertRefusal` (exit 2), not `BadInput` (exit 1), matching SPEC §2 path-requirement note.
- `edge_uses_pbkdf2` predicate at `convert.rs:351-360` correctly includes `Wif`; ignored-passphrase warning correctly suppressed on this edge.
- `needs_derive` correctly excludes `Wif` — `--template` is not required for the phrase/entropy → wif edge.
- Byte-exact refusal stderr matches SPEC §2 and test assertions.
- `is_secret_bearing(Wif) == true` so §7 secret-on-stdout warning fires on phrase → wif output.
- `derive_bip32_at_path` signature and secp-context pattern are consistent with the pre-existing sibling `derive_bip32_from_entropy`.

## Nits (deferred to FOLLOWUPS or addressed in this commit)

- **Addressed in this commit:** `convert.rs:505` redundant `.into()` on `network.network_kind()` — `CliNetwork::network_kind()` returns `NetworkKind` directly; `.into()` was a no-op identity. Dropped.
- **Deferred to FOLLOWUPS** (not introduced by Phase B): `convert.rs:382` and `:385` have duplicate `// 8)` step labels in the `run` function dispatch; the second should be `// 9)`. Pre-existed Phase B; logged as `convert-run-step-numbering-duplicate-8` for v0.6.2+.

## Cleared for Phase B commit

`cargo test --workspace` reports 230 lib + integration tests pass; +5 new integration tests added in this phase (`phrase_to_wif_bip84_leaf_mainnet`, `entropy_to_wif_bip84_leaf_mainnet`, `phrase_to_wif_passphrase_does_not_emit_ignored_warning` in `cli_convert_happy_paths.rs`; `refusal_phrase_to_wif_missing_path`, `refusal_entropy_to_wif_missing_path` in `cli_convert_refusals.rs`).
