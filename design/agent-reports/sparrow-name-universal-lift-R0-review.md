# R0 review — SPEC_sparrow_name_universal_lift.md (verbatim, persisted before fold)

Reviewer: feature-dev:code-reviewer (opus). Base `2a36ee6`. Cycle A — sparrow-name universal lift.

## VERDICT: RED — 3 Critical / 4 Important / 4 Minor
Spec NOT implementable as written. 3 audit-table rows misstate envelope wire-shape or source struct; 4th use-site fact (`wallet_name_was_user_supplied`) makes Specter target fail at runtime.

## Critical (3)

### C1 — `coldcard_multisig_source_metadata` is NOT emitted today
Spec claims envelope carries it; verified absent (`cmd/import_wallet.rs:1736-1882` emits 6 per-format fields; no `coldcard_multisig_source_metadata`). `ImportProvenance::ColdcardMultisig` returns `None` from every accessor. `ColdcardMultisigSourceMetadata.name: String` IS populated by parser at `:499-506` but discarded — never enters per-format emit chain. Implication: `resolved_wallet_name()` on coldcard-multisig envelope always returns None. Fix options: (a) extend wire-shape this cycle (add `coldcard_multisig_source_metadata()` accessor + emit block — additive, back-compat); (b) scope-cut coldcard-multisig, file sub-FOLLOWUP. Brainstorm's "fix the class" discipline argues (a).

### C2 — Jade key is `coldcard_compat.name`, not `multisig_name`
Verified `JadeSourceMetadata` (`wallet_import/jade.rs:65-79`) has `coldcard_compat: ColdcardMultisigSourceMetadata` + `jade_specific_fields: Vec<String>`. No `multisig_name` field. Wire path is `jade_source_metadata.coldcard_compat.name`. Accessor as written returns None always. Fix: generalize helper to take `&[&str]` path, or special-case jade.

### C3 — Coldcard single-sig has NO source-side wallet name
`ColdcardSourceMetadata` (`wallet_import/coldcard.rs:108-124`) fields: `chain, xfp, bip_derivation, raw_account, dropped_fields`. No `name`. Fixture's `"name": "p2wpkh"` inside `bip84` is BIP-derivation label, not wallet name. Spec fabricated this row. Fix: drop coldcard-singlesig from lift scope (same disposition as BSMS); update "7 name-carrying formats" count.

## Important (4)

### I1 — SpecterEmitter requires `wallet_name_was_user_supplied: true`; lift alone won't produce successful re-emit
`wallet_export/specter.rs:31-38` collect_missing pushes `MissingField::WalletName` when `!wallet_name_was_user_supplied`. After fix without flag update, `--from-import-json env --format specter` without `--wallet-name` errors `ExportWalletMissingFields` even though lifted name exists. Fix: at use-site flip flag when lift occurs (`args.wallet_name.is_some() || lifted.is_some()`) OR rename field `wallet_name_is_non_default`. Mechanical refactor; one consumer at `wallet_export/specter.rs:34`.

### I2 — Audit table citations off
`CoreSourceMetadata.wallet_name` at `wallet_import/mod.rs:328`, NOT `bitcoin_core.rs:220` (function parameter, unrelated). Jade cited `:67-` (doc-comment start), asserts nonexistent field. Re-grep all 7 audit-table citations against current source.

### I3 — `wallet_name_was_user_supplied` doc-comment becomes misleading
`cmd/export_wallet.rs:453-454` doc says "lets SpecterEmitter distinguish user-supplied from default" — post-fix it's also true on lift. Rename field this cycle OR update comment + spec.

### I4 — Electrum-multisig empty-x1-label test gap
`ElectrumSourceMetadata.wallet_name: Option<String>` — singlesig parser at `:105-108` filters empty → None, but multisig path (`x1/.label`) doesn't have this guarantee. Add `json_envelope_electrum_multisig_empty_x1_label_returns_none` cell.

## Minor (4)
- M1: chapter-45 path is `docs/manual/src/45-foreign-formats.md` (not under `40-cli-reference/`); spec re-cite at fold time.
- M2: schema_version disposition only implied — add one-line "Wire compatibility" note (stays `"1"`; back-compat).
- M3: "7 name-carrying" count → 6 (after C1+C3): sparrow, specter, jade, electrum, bitcoin-core, coldcard-multisig (conditional on C1.a). Update §Scope, §Tests, §Architecture throughout.
- M4: Test naming OK; lock the parametric set as `&[(format, path, name)]` slice literals so test fails RED if dispatch arms drift.

## GREEN
- Use-site `cmd/export_wallet.rs:693-696` accurate; `envelope` is `&ImportJsonEnvelope` in scope.
- `wallet_name_was_user_supplied` read at `:726`; only consumer is `wallet_export/specter.rs:34` (covered by I1).
- 6 transcripts in `docs/manual/transcripts/foreign-formats/` are the correct set.
- Sparrow `.out` IS the name/label diff (4-line context + 2 fields ≈ 146 bytes).
- `#[serde(default)] Option<serde_json::Value>` is correct back-compat idiom.
- `--locked` guard at `rust.yml:44-45` is real; Phase-6 list complete.
- v0.37.7 → v0.37.8 PATCH disposition correct.

## Required for GREEN
1. C1 — pick (a) extend wire-shape OR (b) scope-cut; spec must state explicit choice.
2. C2 — accessor walks `coldcard_compat.name`; generalize helper or special-case.
3. C3 — drop coldcard-singlesig.
4. I1 — flip flag on lift OR rename `wallet_name_is_non_default`.
5. I2 — re-grep all citations.
6. I3 — update doc-comment + spec narrative.
7. I4 — add electrum multisig empty-x1-label test cell.

Minors M1-M4 inline. Re-dispatch architect for R1 per "reviewer-loop continues after every fold."
