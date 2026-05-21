# v0.28.7 Phase 6 end-of-cycle opus review

**Reviewer:** opus
**Date:** 2026-05-20
**Cycle:** mnemonic-toolkit-v0.28.7 (Wave 2 hardening)

## Verdict: GREEN

0 Critical / 0 Important. Recommendation: **PROCEED-TO-COMMIT** (then Phase 7 release tooling).

## Slug-by-slug substantiation

- **Slug 1** (`bsms-import-taproot-refusal-parity`): variant `BsmsTaprootImportRefused` correctly placed BEFORE `BsmsTaprootRefused` (alphabetical, `I` < `R`) at `error.rs:271`. Display arm L687, exit_code arm L477 (= 2), kind arm L534 — all 4 sites alphabetically before `BsmsTaprootRefused`. Parse-entry refusal at `bsms.rs:215` AFTER checksum verify but BEFORE `parse_descriptor` expensive parse — correct ordering. Defense-in-depth at `extract_threshold` L493 correct.
- **Slug 2** (`green-emitter-multisig-refusal-template-only`): `WalletScriptType::is_multisig()` at `wallet_export/mod.rs:176-181` covers exactly `P2shMulti | P2shP2wshMulti | P2wshMulti | P2trMulti`. `green.rs:36` refactored from template-gated to `inputs.script_type.is_multisig()` — preserves error message; cell_4 at `cli_export_wallet_green.rs:104` exercises descriptor-mode multisig refusal end-to-end.
- **Slug 3** (`wallet-import-format-mismatch-matrix-completion` Option B): 17 new `ImportWalletFormatMismatch` arms confirmed (BSMS: 6 new at L277-312; BitcoinCore: 6 new at L329-364; ColdcardMultisig: 5 new at L450-479). Matrix test asserts `"blob looks like"` (the correct stderr substring per `error.rs:648`) — Implementer B's deviation is semantically correct.
- **Slug 4** (`wallet-import-taproot-internal-key` Framing B Fix-α): `matches!(script_type, WalletScriptType::P2tr | WalletScriptType::P2trMulti)` at `export_wallet.rs:622` correctly uses parse-side detection; `WalletScriptType` added to `use` group at L16; placed AFTER `script_type_from_descriptor` at L612. Test cell covers 4 formats × 2 descriptors (P2tr + P2trMulti) = 8 sub-assertions.

## Quantitative verification

- Test math: 2008 + 1 (Slug1 NEW) + 1 (Slug2 cell_4) + 17 (Slug3 matrix) + 1 (Slug4 p_slug4) = **2028**. Confirmed.
- Both canary flips semantically correct: `core_sniff_smoke` exit 1 (was 2) post-Slug-3; `p11c_green_descriptor_passthrough_singlesig_passes_multisig_refused` post-Slug-2.
- Clippy clean.
- No GUI lockstep required: no clap/CLI surface added; no JSON wire-shape change; `ImportWalletFormatMismatch` already existed.

## Minor (deferred)

1. **Slug 1 Step 8 cell duplicates Step 7 coverage** — `tests/cli_import_wallet_bsms.rs:979-1007` (`bsms_tr_sortedmulti_a_refused_via_extract_threshold_guard`) uses the SAME fixture (`bsms-2line-tr-nums.txt`) as the renamed Step-7 cell. Because the parse-entry `tr(` guard at `bsms.rs:215` fires FIRST, the `extract_threshold` defense-in-depth at L493 is unreachable via this test path. Cell still passes; just provides no second-guard exercise. Functional behavior pinned by parse-entry guard.

## New FOLLOWUP to file

1. **`bsms-extract-threshold-defense-in-depth-direct-unit-test`** — Add a `#[cfg(test)] mod tests` unit test in `wallet_import/bsms.rs` that directly invokes `extract_threshold("tr(NUMS,sortedmulti_a(2,@0,@1))")` and asserts `Err(BsmsTaprootImportRefused)`. The integration test at `cli_import_wallet_bsms.rs:979` cannot reach the guard because the parse-entry refusal fires first; the guard at `bsms.rs:493` is therefore shipped untested. Low priority — purely defense-in-depth regression-guard gap.
