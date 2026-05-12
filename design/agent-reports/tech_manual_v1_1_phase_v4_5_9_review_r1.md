# Review Report: tech-manual v1.1 §V.4.5.9 + §V.4.5.10 — r1

**Reviewer:** code-reviewer (r1)
**Date:** 2026-05-12
**Review surface:**

- `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md` lines 373–680 (§V.4.5.9 + §V.4.5.10)
- `docs/technical-manual/src/60-back-matter/61-glossary.md` (8 vendor entries + 5 new symbol entries)
- `docs/technical-manual/src/60-back-matter/62-index-table.md` (+5 rows)
- `docs/technical-manual/src/60-back-matter/65-troubleshooting.md` (4 ExportWallet rows)
- `docs/technical-manual/.cspell.json` (+5 entries)

**Ground truth read in full:** all 8 vendor emitters, `wallet_export/mod.rs`, `cmd/export_wallet.rs`, `error.rs` ExportWallet variants, plus all 5 supporting documentation files.

## Critical

None.

## Important

### I-1. Glossary `ELECTRUM_SEED_VERSION_PIN`: "Defined" section pointer is wrong

**Confidence: 90**

**Offense:** `61-glossary.md:141` — entry ends with `Defined §V.4.3.8.`

**Contradicting ground truth:** §V.4.3.8 is the "Miscellaneous support modules" overview table; the full definitional treatment of `ELECTRUM_SEED_VERSION_PIN = 17` (rationale, empirical validation date, migration-chain explanation) lives at §V.4.5.9.7 (`54-mnemonic-toolkit-api.md:586`). Every other glossary entry for a symbol fully treated in §V.4.5.9.x cites §V.4.5.9.x as its "Defined" pointer.

**Fix applied:** `Defined §V.4.3.8.` → `Defined §V.4.5.9.7.`

### I-2. §V.4.5.9.3 Coldcard: cross-reference cites wrong sibling section number for Jade

**Confidence: 88**

**Offense:** `54-mnemonic-toolkit-api.md:487` — prose reads "Multisig text is the byte-identical input format accepted by Blockstream Jade (§V.4.5.9.6)."

**Contradicting ground truth:** Jade is documented in §V.4.5.9.4 (`54-mnemonic-toolkit-api.md:489`); §V.4.5.9.6 is the Specter section. Intent is clear but the number misnavigates the reader.

**Fix applied:** `§V.4.5.9.6` → `§V.4.5.9.4`.

## Low

None above confidence threshold.

## Nit

### N-1. cspell entry `"XONLY"` claimed unused as a bare word — FALSE POSITIVE

**Confidence: 80** (reviewer)

**File:** `.cspell.json:231`

**Reviewer claim:** `XONLY` appears only inside backtick-quoted spans (`` `NUMS_XONLY_HEX` ``), so the cspell `ignoreRegExpList` `` "`[^`]+`" `` should suppress it; the dictionary entry is dead.

**Empirical refutation (post-edit `make lint` run):** After removing the entry, cspell flagged `src/50-rust-api/54-mnemonic-toolkit-api.md:442:516 - Unknown word (XONLY)`. cspell's tokenizer splits the suppressed backtick span's snake_case identifier into sub-tokens that re-enter the word-checker. The entry IS load-bearing.

**Resolution:** Revert. `XONLY` retained in cspell dictionary.

## Verification Summary

### `--format` selector names (8/8 verified)

All 8 sub-section headers match `cmd/export_wallet.rs:21-39` `CliExportFormat::#[value(name = ...)]` attributes verbatim: `bitcoin-core`, `bip388`, `coldcard`, `jade`, `sparrow`, `specter`, `electrum`, `green`.

### Emitter struct line citations (8/8 verified)

`bitcoin_core.rs:14`, `bip388.rs:21`, `coldcard.rs:21`, `jade.rs:21`, `sparrow.rs:28`, `specter.rs:28`, `electrum.rs:40`, `green.rs:23` — all match.

### `collect_missing` behaviors (8/8 verified)

- Bitcoin Core / BIP-388 / Coldcard / Jade / Electrum / Green: always empty.
- Sparrow: `[Threshold]` when `is_multisig() && !threshold_user_supplied` (`sparrow.rs:31-48`).
- Specter: `[WalletName]` when `!wallet_name_was_user_supplied` (`specter.rs:31-38`).

### Matrix cells sampled (36 of 64)

All sampled cells match chapter's matrix. Footnotes [a]–[g] each verified against source.

### Five new glossary symbol entries

`EmitInputs` (mod.rs:327), `MissingField` (mod.rs:224), `TimestampArg` (mod.rs:122), `WalletFormatEmitter` (mod.rs:316), `WalletScriptType` (mod.rs:143) — all `pub(crate)` as claimed.

### Troubleshooting rows (4/4 verified)

All 4 `ExportWallet*` variants exist in `error.rs` with section pointers resolving to relevant sections.

### Refusal-mode classification

Chapter consistently distinguishes `BadInput` (shape incompatibility) from `ExportWalletMissingFields` (missing user-supplied flag). All routing verified.

### Schema-version / `seed_version` distinction

Chapter correctly distinguishes Electrum's `seed_version` (pinned to 17) from the toolkit's `schema_version` (BundleJson field). No conflation.

### Vendor flags (all verified present in `cmd/export_wallet.rs`)

No phantom flags. All cited at correct arg-attribute lines.

## Open Questions (toolkit-source hygiene — not chapter findings)

Drafter-flagged source-side doc-comment drifts. These are pre-existing and do not affect chapter accuracy:

1. `wallet_export/mod.rs:42-44` — SPEC §3 mismatch in comment text
2. `wallet_export/mod.rs:1-12` — mod doc-comment lists only 3 of 8 submodules
3. `cmd/export_wallet.rs:3-5` — cites v0.7 SPEC despite v0.8 realisation

File in toolkit's `design/FOLLOWUPS.md` if folding into the release cycle.

## Verdict

- [x] 2 findings folded inline (I-1 + I-2); 1 nit reverted (false-positive). Lint 6/6 OK. Ready for r2 verification.
