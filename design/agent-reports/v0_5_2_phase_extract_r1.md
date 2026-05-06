# v0.5.2 Phase Extract — code-reviewer r1

**Outcome:** 0C/1I/0L/0N → 0C/0I (after I-1 fix). APPROVED.

## Scope reviewed
4 paths in the v0.5.2 atomic refactor:
- `crates/mnemonic-toolkit/src/derive_slot.rs` (NEW) — extracted `derive_bip32_from_entropy` helper.
- `crates/mnemonic-toolkit/src/derive.rs` — `DerivedAccount` extended with `account_path` field; `derive_full` collapsed to call helper.
- `crates/mnemonic-toolkit/src/main.rs` — `mod derive_slot;` added.
- `crates/mnemonic-toolkit/src/cmd/bundle.rs` — `resolve_slots` phrase + entropy branches now call helper.

## Plan-fidelity verification
1. **Behavior preservation (phrase branch):** `derive_full` does parse_in → to_entropy → helper(entropy, language, ...). Helper does from_entropy_in(language, entropy) → to_seed. The round-trip parse_in → to_entropy → from_entropy_in is bijective for all BIP-39 languages; seed output is byte-identical. Tests pass: 230 lib + 44 integration.
2. **DerivedAccount field population:** all sites populate `account_path`. No partial construction.
3. **Helper visibility:** `pub(crate)` from `derive_slot.rs`; `mod derive_slot` declared in main.rs:6. Accessible from cmd/bundle via `crate::derive_slot::derive_bip32_from_entropy`.
4. **Path raw string equivalence:** `acc.account_path.to_string()` matches pre-refactor `template.derivation_path(network, account).to_string()`.
5. **Network cross-check:** moved to `derive_bip32_from_entropy`; fires for all callers.

## Important folded inline

**I-1 (unused imports in bundle.rs after refactor):** `Mnemonic` and `Xpriv` no longer referenced post-refactor (bundle.rs:167-168). Removed both; `DerivationPath` and `Fingerprint` retained (still live in xpub/wif branches).

## Test results
230 lib + 44 integration tests pass; 2 lib ignored (pre-existing). No new failures.
