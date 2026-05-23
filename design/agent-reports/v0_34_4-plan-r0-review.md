# v0.34.4 — Plan-doc R0 architect review (opus) — MANDATORY pre-implementation gate

**Date:** 2026-05-22
**Cycle:** mnemonic-toolkit v0.34.4 — `import-wallet` format-mismatch matrix completion
**Branch:** `v0.34.4-format-mismatch-matrix`
**Reviewer:** opus (feature-dev:code-reviewer), R0
**Target:** `design/IMPLEMENTATION_PLAN_v0_34_4_format_mismatch_matrix.md` (verified against live source)

---

## Critical
(none)

## Important
(none)

## Minor

- **M1 — the 4 modified blocks' comments still reference the OLD slug name** (`wallet-import-format-mismatch-matrix-completion`) and "intentionally narrow" framing. Cosmetic. When Task 1 Step 3 updates those comments, point them at the new slug `wallet-import-format-mismatch-matrix-completion-discovered-gaps` + note the arm is now complete (v0.34.4). No functional impact. (`import_wallet.rs` coldcard/electrum/sparrow/specter block comments.)
- **M2 — plan's "~Lxxx" anchors verified EXACT, not drifted** (coldcard L587, electrum L693, sparrow L802, specter L837). No action; recorded as verified.

---

## Verification summary (all 8 gate points)

1. **10 missing pairs / count** — independently tallied from live source L473-877: coldcard→{Electrum,Jade}=2, electrum→{Jade}=1, sparrow→{Coldcard,Electrum,Jade,Specter}=4, specter→{Coldcard,Electrum,Jade}=3 = **exactly 10**. bsms/bitcoin-core/coldcard-multisig/jade each 7/7 complete. Plan's table correct. ✓
2. **Variant names** — `enum SniffOutcome` at `sniff.rs:62-73` = `{Ambiguous, BitcoinCore, Bsms, Coldcard, ColdcardMultisig, Electrum, Jade, NoMatch, Sparrow, Specter}`. Plan's new arms `SniffOutcome::{Electrum,Jade,Coldcard,Specter}` all spelled correctly. ✓
3. **Error variant shape** — `error.rs:184-187`: `ImportWalletFormatMismatch { supplied: String, sniffed: String }`; exit_code=1 (`error.rs:468`). Plan constructs `supplied`/`sniffed` `.to_string()`. ✓
4. **Harness + fixtures** — `assert_format_mismatch(user_format, fixture, detected_format)` exists (`cli_import_wallet_format_mismatch_matrix.rs:13`); signature matches the 10 one-liners. All 4 referenced fixtures exist on disk. ✓
5. **Sniff correctness (subtle)** — each reused fixture is already proven to sniff as its claimed format by existing PASSING cells in the same file (bsms/bitcoin-core/coldcard-multisig use the same fixture+detected_format). `sniff_format` (`sniff.rs:105-114`) returns a single format only on exactly-one vote, so those passing tests guarantee single-format verdicts → reuse-consistency holds for all 10. ✓
6. **Canonical arm ordering** — existing complete arms follow `Bsms,BitcoinCore,Coldcard,ColdcardMultisig,Electrum,Jade,Sparrow,Specter` (self-skipped); live insertion anchors match the plan (sparrow currently `Bsms,BitcoinCore,ColdcardMultisig`; specter `…,Sparrow`). Order functionally irrelevant (exclusive patterns) but anchors accurate. ✓
7. **Version artifacts** — `Cargo.toml:3`=0.34.3, `Cargo.lock:682`=0.34.3, `install.sh:32`=`mnemonic-toolkit-v0.34.3`, `CHANGELOG.md:9`=`[0.34.3]`. Bumps to 0.34.4 correct. ✓
8. **SemVer/lockstep** — match-arm-only + tests; no clap flag change. PATCH + no GUI/manual lockstep correct. ✓

VERDICT: GREEN (0C/0I)

---

## Fold disposition (controller)
GREEN (0C/0I) → gate satisfied, implementation may proceed. M1 (cosmetic comment slug-name) folded into Task 1 Step 3 during impl (comments reference the new slug + note completion). M2 needs no action. No plan-doc edit + no R0 re-dispatch (GREEN; no Critical/Important).
