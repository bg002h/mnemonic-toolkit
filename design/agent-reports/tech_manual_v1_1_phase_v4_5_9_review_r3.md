# Review Report: tech-manual v1.1 §V.4.5.9 + §V.4.5.10 — r3

**Reviewer:** code-reviewer (r3)
**Date:** 2026-05-12
**Scope:** Confirmation-only. Verify all r2-disposition folds + 3 parent-agent stragglers landed; sweep for straggler patterns; check `TaprootInternalKey` visibility claim.

## Fold Verification

All seven fold points verified by Read of cited lines:

- `61-glossary.md:101` — `master-xpub wiring landed in v0.8.1` — PASS
- `61-glossary.md:185` — `shipped in v0.8.1` — PASS
- `61-glossary.md:373` — ends `Defined §V.4.5.9.` — PASS
- `61-glossary.md:417` — ends `Defined §V.4.5.9.` — PASS
- `54-mnemonic-toolkit-api.md:145` — `v0.8.1 vendor-emitter expansion` — PASS
- `54-mnemonic-toolkit-api.md:746` — single v0.8.1 attribution — PASS
- `54-mnemonic-toolkit-api.md:761` (cspell comment) — `(v0.8.1 wallet-export emitter)` — PASS

## Straggler Sweeps

- `v0\.8\.2` across `src/` — 0 matches. PASS.
- `§V\.4\.3\.8` as a "Defined §X" pointer in glossary — 0 occurrences. (One narrative occurrence in `63-release-history.md` is non-pointer prose, not a finding.)
- `§V.4.5.9.4` / `§V.4.5.9.6` cross-reference routing — three total occurrences (`61-glossary.md:185` Jade→.4, `61-glossary.md:353` Specter→.6, `54-mnemonic-toolkit-api.md:487` Coldcard→Jade=.4). All correct.

## TaprootInternalKey Visibility Drift Check

Glossary (`61-glossary.md:373`) claims `pub enum` at `wallet_export/mod.rs:68`.
Source (`mod.rs:68`) declares `pub enum TaprootInternalKey {` — bare `pub` confirmed. PASS. (This is one of few `wallet_export/` items with bare `pub` visibility; the five sibling symbols added this cycle are correctly documented as `pub(crate)` in their own entries.)

## Critical / Important / Low / Nit

None.

## Verdict

- [x] 0C / 0I / 0L / 0N — chapter is tag-ready (advance to cycle-exit)
