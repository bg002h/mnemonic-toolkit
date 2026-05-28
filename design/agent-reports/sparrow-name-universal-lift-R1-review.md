# R1 re-review — SPEC_sparrow_name_universal_lift.md (verbatim, post-fold)

Reviewer: feature-dev:code-reviewer (opus). R0 was RED 3C/4I/4M (`sparrow-name-universal-lift-R0-review.md`); folded; this is the re-dispatch.

## VERDICT: GREEN — 0 Critical / 0 Important
All folds correctly applied. No new drift. Mandatory pre-impl R0 gate satisfied (RED 3C/4I/4M → fold → R1 GREEN 0C/0I).

## Fold verification
- **C1 (coldcard-multisig emit-side)**: `ImportProvenance::ColdcardMultisig(_)` at `wallet_import/mod.rs:152`; `coldcard_source_metadata()` model at `:179-191` parallels exactly. Emit blocks at `cmd/import_wallet.rs:1736-1882` (5 per-format); new block fits between coldcard-singlesig (ends `:1778`) and electrum (starts `:1779`). `ColdcardMultisigSourceMetadata` (`coldcard_multisig.rs:96-111`) carries `name: String`, `policy: PolicyKOfN { k, n }`, `script_format: ColdcardMsFormat` — matches jade emit projection pattern at `cmd/import_wallet.rs:1825-1826`. No existing test asserts ABSENCE (`grep coldcard_multisig_source_metadata` returns only spec/R0). Additive emit = back-compat.
- **C2 (jade nested path)**: Emit block `cmd/import_wallet.rs:1820-1834` writes `jade_source_metadata: { coldcard_compat: { name, ... }, jade_specific_fields: [] }`. Accessor's `&["coldcard_compat", "name"]` walks this exact path.
- **C3 (coldcard-singlesig dropped)**: `ColdcardSourceMetadata` (`coldcard.rs:112-124`) — no `name`. Deserializer omits `coldcard_source_metadata`; integration matrix = 6 cells; §Per-format audit lists 6 yes + 2 explicit fall-through.
- **I1 + I3 (rename `wallet_name_was_user_supplied` → `wallet_name_is_non_default`)**: 6 sites total — `cmd/export_wallet.rs:479,726`, `wallet_export/mod.rs:501-503,504`, `wallet_export/specter.rs:34`, `tests/cli_export_wallet_specter.rs:114`. All covered by "throughout."
- **I2 (citations)**: `CoreSourceMetadata.wallet_name` at `wallet_import/mod.rs:328` (struct `:317-329`); spec cite `:318-329` covers it. Other audit-table citations within tolerance.
- **I4 (electrum-multisig empty-x1 test)**: Multisig parser at `electrum.rs:759-763` filters `!s.is_empty()` today — cell guards deserializer's own filter against future parser drift. Defensive but valid.

## No new drift
Use-site `cmd/export_wallet.rs:693-696` accurate; Phase-6 list complete; SemVer-PATCH disposition correct (additive Optional wire-shape; no clap surface change → no GUI lockstep); manual lockstep references `docs/manual/src/45-foreign-formats.md` + 6 transcripts. All 9 occurrences of "6" coherent; no stale "7" except in §R0 history narrative.

## Residual sub-threshold (do not block GREEN)
- M-sub-1: `wallet_export/mod.rs:501-503` and `tests/cli_export_wallet_specter.rs:114` are doc-comment sites not explicitly listed in §4 — covered by "rename throughout" but worth verifying during implementation sweep.
- M-sub-2: I4 cell narrative "parser path may emit empty" is narrowly inaccurate (current parser filters), but cell remains defensively valid.

**Cleared to implement.** Test count: 7 unit + 6 integration + 1 explicit-override + 1 specter-target = 15 cells.
