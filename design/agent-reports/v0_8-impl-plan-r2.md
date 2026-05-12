# v0.8 IMPLEMENTATION_PLAN review — r2

Date: 2026-05-11
Reviewer: opus-architect (r2) via general-purpose agent

## R1 verification

**C-1 (tests/lint.sh path):** resolved. All four callsites converted to `make -C docs/manual lint MNEMONIC_BIN="$(pwd)/target/debug/mnemonic" MD_BIN=true MS_BIN=true MK_BIN=mk` — Phase 0 exit gate, Phase 1 step 8, Phase 6 final gauntlet, Verification block. Residual `tests/lint.sh` hits are only inside the iterative-review log explaining the fix (descriptive prose, not live invocations). No `bash tests/lint.sh` anywhere live.

**C-2 (v0.8.X tag collision):** resolved. Context paragraph rewritten to clarify this is the v0.8.1 cut; v0.8.0 (commit 7bb722a) is named as the prior `[BREAKING]` cut. Phase 6 step 2 uses `[0.8.1] — 2026-05-??`. Phase 6 step 5 uses `mnemonic-toolkit-v0.8.1`. CHANGELOG header text uses `## mnemonic-toolkit [0.8.1]`. Sibling header notes also use `v0.8.1`. No residual `v0.8.X` / `0.8.X` outside the iterative-review log.

**I-1 (wallet_export.rs:17-25 → 17-18):** resolved. Plan line 12 now reads `src/wallet_export.rs:17-18`. SPEC §3 also updated to `17-18`. No `17-25` outside the iterative-review log.

**I-2 (slip0132.rs:138 → 169):** resolved. Plan now cites `slip0132.rs:169` for `BIP84_REF_ZPUB`. No `slip0132.rs:138` anywhere outside the iterative-review log.

**I-3 (148-155 → 148-153 with sub-ranges):** resolved. All live citations normalized to `148-153` for the stub-arm match-block span — plan Context, Phase 1 step 6, Phase 3 step 4, Critical files summary; SPEC §2 and §12. Per-phase sub-ranges thread correctly: Phase 2 step 4 cites `148-150` for the Sparrow arm; Phase 3 step 4 cites `151-153` for Specter plus wildcard `154`. No `148-154` or `148-155` outside the iterative-review log.

**I-4 (MNEMONIC_BIN=true vacuous):** resolved. Phase 0 exit gate adds `cargo build --bin mnemonic`; passes `MNEMONIC_BIN="$(pwd)/target/debug/mnemonic"`; carries the explicit explanatory comment (cross-references FOLLOWUPS entry `lint-md-flag-coverage-vacuous-with-md_bin-true`). Phase 1 step 8 and Phase 6 final gauntlet and Verification block all mirror this pattern.

**I-5 (SPEC §13 electrum rows still cite Coldcard):** resolved. SPEC §13 table rows for `electrum_single.json` and `electrum_multi_2of4.json` now read "pinned to Phase 4 step 0 spike-observed byte shape" — matching SPEC §9's already-corrected narrative.

**I-6 (CHANGELOG breaking-change disambiguation):** resolved. Phase 6 step 2 now contains "**No breaking changes** — v0.7 stable `--format bitcoin-core` / `--format bip388` byte-exact fixtures continue to pass through the new submodule dispatch. (v0.8.0 at commit `7bb722a` was `[BREAKING]` per the v0.8 series header in `CHANGELOG.md`; this cut is additive.)"

**I-7 (wallet-export-industry-formats state-transition):** resolved. All three sites converted to additive `Resolution-extended:` notes on a still-resolved entry:
- Header line 5: "already `Status: resolved 3821f66` by v0.7 Phase 5; v0.8.1 cycle extends coverage from 2 → 8 formats via `Resolution-extended:` notes appended to the existing entry — no reopen".
- Phase 1 step 10: "Append a `Resolution-extended (v0.8.1 Phase 1):` line … do NOT reopen the entry; the FOLLOWUPS schema has no "reopen" state".
- Phase 5 step 6: "Append a final `Resolution-extended (v0.8.1 Phase 5):` line … entry stays `Status: resolved` (do NOT flip its status — there is no "RESOLVED-again" state)".

**L-1 (Phase 3 step 4 duplicate):** resolved. Phase 3 reviewer-loop step is now numbered `5.`. The four-step body is now: 1 (RED fixtures), 2 (impl `specter.rs`), 3 (pin refusal fixture), 4 (wire CliExportFormat::Specter); reviewer-loop is 5.

**L-2 (clap shape ambiguous):** resolved. Phase 1 step 7 now specifies: `Option<String>` clap-derive shape; post-parse default resolution `let wallet_name = args.wallet_name.clone().unwrap_or_else(|| format!("{}-{}", template_human_name(template), account));`; specter-required check via `SpecterEmitter::collect_missing` returning `MissingField::WalletName` based on a new `wallet_name_was_user_supplied: bool` field on `EmitInputs` added in Phase 0 step 6.

**N-1 (electrum wallet_path):** resolved. Phase 4 step 0 now passes `--wallet_path /tmp/electrum-spike-single.json` to `electrum --offline restore <xpub>`, with a parallel `--wallet_path /tmp/electrum-spike-multi.json` for the multisig case. The "inspect the resulting wallet file at `~/.electrum/wallets/<name>`" wording is gone.

## Additional checks

- **MissingField variant count:** SPEC §4 enumerates exactly 7 variants — matches SPEC iterative-review log "shrinks from 9 to 7". The plan does not enumerate independently; no drift.
- **Iterative-review log accuracy:** the new R1 entry in the plan log faithfully describes every edit actually applied. No fabricated resolutions.
- **No residual `bash tests/lint.sh` live invocations.**
- **No residual `v0.8.X` / `0.8.X` live mentions.**
- **No residual `17-25`, `148-154`, `148-155`, `slip0132.rs:138` live mentions.**

## New findings

### R2-L1 — Phase 0 step 6 mentions `MissingField` + `ToolkitError` but does not list the `wallet_name_was_user_supplied: bool` field that Phase 1 step 7 promises is added "alongside the rest of `EmitInputs`"

**Location:** `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md` Phase 0 step 6, cross-referenced from Phase 1 step 7.

**Evidence:** Phase 1 step 7 says: "The specter-required check happens later in `SpecterEmitter::collect_missing` via a `wallet_name_was_user_supplied: bool` field on `EmitInputs` (add this field in Phase 0 step 6 alongside the rest of `EmitInputs`)." Phase 0 step 6 reads: "Add `MissingField` enum + `build_missing_fields_refusal` + `ToolkitError::ExportWalletMissingFields { format, missing }` variant. Zero new behavior tests (validator does not fire yet in v0.7 paths)." It does not mention `EmitInputs` at all — `EmitInputs` is introduced implicitly in Phase 0 step 4 ("thread `EmitInputs` through dispatch") and defined in SPEC §12. The reader following Phase 1 step 7's pointer to "Phase 0 step 6 alongside the rest of `EmitInputs`" finds no `EmitInputs` mention there. Cosmetic only; the implementing agent will resolve it by reading SPEC §12, but the cross-reference is slightly misleading.

**Fix:** In Phase 0 step 6, append: "Define `EmitInputs` struct per SPEC §12, including the `wallet_name_was_user_supplied: bool` field used by Phase 1 step 7 / Phase 3's `SpecterEmitter::collect_missing`." OR in Phase 1 step 7, change "(add this field in Phase 0 step 6 alongside the rest of `EmitInputs`)" to "(add this field to `EmitInputs` during the Phase 0 module-reorganization commit, per SPEC §12)".

## Summary

Total NEW: 0C / 0I / 1L / 0N

- All 12 R1 findings verified resolved with concrete evidence.
- One new Low surfaced (R2-L1): Phase 0 step 6 does not explicitly mention `EmitInputs` or the `wallet_name_was_user_supplied` field, despite Phase 1 step 7 cross-referencing it. Cosmetic; implementing agent will resolve via SPEC §12. Recommend a one-line tightening.
- No fold-in regressions, no new typos, no cross-document drift introduced by the R1 fold.
- SPEC consistency confirmed (electrum-table cross-reference fixed; line refs aligned; MissingField count of 7 matches enumeration).

Convergence: YES — 0C/0I. IMPLEMENTATION_PLAN ready for STOP-and-check-user gate. (Optional R2-L1 fix is cosmetic and may be folded in the same session or deferred to a follow-up.)
