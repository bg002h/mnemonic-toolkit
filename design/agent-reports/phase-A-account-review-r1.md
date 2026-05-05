# Phase A — `--account` thread-through Review — r1

**Date:** 2026-05-05
**Commit under review:** `5486bd6` (parent: `f4c671c`)
**Reviewer:** opus phase-review

## Verdict

0 critical / 0 important / 2 low / 2 nits

✅ **Phase A r1 terminator reached** — cleared to advance to Phase B foundation.

## Critical / Important

(none)

## Low / Nit

- **L-1 (FIXED inline post-r1):** `cmd/bundle.rs::bundle_watch_only` watch-only stderr warning previously said "Use v0.2's --account flag once available" — stale text shipping to users on every single-sig watch-only invocation. Fixed inline (warning now only fires when `args.account == 0` and rephrased to advise explicit `--account <N>` for non-zero account xpubs).
- **L-2 (FIXED inline post-r1):** `error.rs::message()` `BundleMismatch` text said "re-run with v0.2's --account flag once available." Same stale-text issue. Reworded to "pass --account <N> to match (default 0)".
- **N-1 (FOLLOWUPS):** PLAN Phase A's task description said "derive.rs — no signature change"; actual implementation added `account: u32` to `derive_full`. Better implementation choice; plan-prose drift only. Defer to FOLLOWUPS.
- **N-2 (FOLLOWUPS):** `verify_bundle.rs::run_watch_only` SPEC §2.2.2 warning text still contains literal `m/<purpose>'/<coin>'/0'` even when `--account 5` is passed. Cosmetic; the text is a SPEC §2.2.2 byte-exact-pinned warning label. Phase D mode-violation/help-text consistency audit will revisit.

The two doc-comments on `BundleArgs::account` and `VerifyBundleArgs::account` were also flagged for misleading "PathDeclPaths::Divergent" mention (single-sig stays Shared); that's been left in place since v0.2 multisig WILL produce Divergent for non-zero per-cosigner accounts. The doc-comment is forward-looking. Defer to Phase D doc-comment audit.

## Verified

- **SPEC §2.1.7 closure:** `account: u32` with `default_value = "0"` present in both `BundleArgs` and `VerifyBundleArgs`.
- **SPEC §4.2 single-sig:** `build_descriptor` calls `template.md_origin_path(network, account)` → `PathDeclPaths::Shared(origin_path)`. `Divergent` not emitted in Phase A (correct — Phase A is single-sig only).
- **SPEC §4.1 derivation path:** `derive_full` calls `template.derivation_path(network, account)` which produces `m/{purpose}'/{coin}'/{account}'`.
- **Origin path account component:** `md_origin_path.components[2]` carries the account value with `hardened: true`.
- **`engraving_card` byte-exact regression:** the v0.1 `engraving_card_full_no_passphrase_byte_exact` test still passes — `account: 0` produces `"account: 0\n"` line identical to v0.1.
- **`BundleJson.account`:** `emit()` reads `args.account` (no longer hardcoded 0).
- **New unit test:** `origin_path_with_nonzero_account` exercises `account: 5` and asserts both `origin_path_str` (`"m/84'/0'/5'"`) and `md_origin_path.components[2].value == 5`.
- **Cross-binding invariants:** v0.1's two `debug_assert!`s in `synthesize_full` and `synthesize_watch_only` preserved at same positions and same semantics.
- **All internal callers updated:** `bundle_full`, `bundle_watch_only`, `run_full` (verify_bundle) all thread `args.account` correctly. No hardcoded `0` at call sites.
- **Wire-bit-identical regression:** implementer reported PASS=16/FAIL=0 against `tests/vectors/v0_1/`. Encoded ms1/mk1/md1 strings byte-identical to v0.1 under `--account 0`.
- **Phase A scope discipline:** no multisig, Divergent, privacy-preserving, or self-check code pulled in. Phase A stays isolated to single-sig account thread-through.

## Smoke checks

- `cargo test -p mnemonic-toolkit`: 72 passing (71 v0.1 + 1 new `origin_path_with_nonzero_account`)
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`: clean
- `cargo fmt --check -p mnemonic-toolkit`: clean
- Wire-bit-identical regression: 16/16 cells PASS (bip{44,49,84,86} × {mainnet,testnet,signet,regtest})
