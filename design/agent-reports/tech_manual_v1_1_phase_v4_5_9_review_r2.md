# Review Report: tech-manual v1.1 §V.4.5.9 + §V.4.5.10 — r2

**Reviewer:** code-reviewer (r2)
**Date:** 2026-05-12

## Review surface

- `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md` lines 373–762
- `docs/technical-manual/src/60-back-matter/61-glossary.md`
- All 8 vendor emitter sources in `crates/mnemonic-toolkit/src/wallet_export/`
- `.git/packed-refs` (tag existence)

## Fold Verification (r1)

- **I-1** `61-glossary.md:141` now reads `Defined §V.4.5.9.7.` — byte-exact. PASS.
- **I-2** `54-mnemonic-toolkit-api.md:487` now reads `Blockstream Jade (§V.4.5.9.4)` — byte-exact. PASS.
- **N-1** `XONLY` cspell entry retained (false-positive revert confirmed). PASS.

## Critical

None.

## Important

### I-A. Glossary `TaprootInternalKey`: "Defined" pointer wrong

**Confidence: 88**
**Offense:** `61-glossary.md:373` — `Defined §V.4.3.8.`

§V.4.3.8 is the miscellaneous-support-modules table; `TaprootInternalKey` is a line item there but receives no enum-variant description, usage rationale, or emitter context. Full definitional treatment lives at §V.4.5.9 and sub-sections (`--taproot-internal-key` flag, role in every taproot-capable emitter). All five sibling symbols added this cycle (`EmitInputs`, `MissingField`, `TimestampArg`, `WalletFormatEmitter`, `WalletScriptType`) cite `§V.4.5.9` or `§V.4.5.9.x`.

**Fold:** `Defined §V.4.3.8.` → `Defined §V.4.5.9.` (applied)

### I-B. Glossary `wallet-export`: "Defined" pointer wrong

**Confidence: 88**
**Offense:** `61-glossary.md:417` — `Defined §V.4.3.8.`

Same pattern as I-A. The comprehensive definitional section is §V.4.5.9 (eight vendor sub-sub-sections, emitter trait contract, EmitInputs field reference, collect_missing semantics, ExportWalletMissingFields routing) + §V.4.5.10 (compatibility matrix).

**Fold:** `Defined §V.4.3.8.` → `Defined §V.4.5.9.` (applied)

### I-C. Glossary `Jade (wallet-export format)`: `v0.8.2` tag does not exist

**Confidence: 92**
**Offense:** `61-glossary.md:185` — `shipped in v0.8.2`

`git tag --list 'mnemonic-toolkit-*'` returns `v0.5.0` through `v0.8.1` only — no `v0.8.2`. HEAD's `crates/mnemonic-toolkit/` content is byte-identical with `mnemonic-toolkit-v0.8.1` (empty `git log v0.8.1..HEAD -- crates/mnemonic-toolkit/`). The Jade emitter at `wallet_export/jade.rs` is part of v0.8.1.

**Fold:** `shipped in v0.8.2` → `shipped in v0.8.1` (applied)

### I-D. Glossary `Coldcard (wallet-export format)`: same non-existent tag

**Confidence: 92**
**Offense:** `61-glossary.md:101` — `master-xpub wiring landed in v0.8.2`

Same root cause as I-C. The `--slot @0.master_xpub=<base58>` plumbing at `coldcard.rs:102,212-216` is part of v0.8.1.

**Fold:** `master-xpub wiring landed in v0.8.2` → `master-xpub wiring landed in v0.8.1` (applied)

## Low / Nit

None above threshold.

## Stragglers folded by parent agent (out of r2 scope but caught pre-r3)

After r2 returned, a sweep `grep -rn 'v0\.8\.2' src/` surfaced two more reader-facing chapter references that r2 had misattributed to the CHANGELOG:

- `54-mnemonic-toolkit-api.md:145` — `v0.8.1+v0.8.2 vendor-emitter expansion` → `v0.8.1 vendor-emitter expansion`
- `54-mnemonic-toolkit-api.md:746` — collapsed the three-clause "v0.8.1 ... v0.8.1+v0.8.2 ... v0.8.2 ..." progression to a single v0.8.1 attribution
- `54-mnemonic-toolkit-api.md:761` (cspell comment) — `"Jade" (v0.8.2 ...)` → `"Jade" (v0.8.1 ...)`

All applied. Final sweep confirms 0 v0.8.2 references remain in `src/`.

## Complement Matrix Audit (r1 unchecked 28 of 64 cells)

Verified the cells r1 did not sample:

- `bip388` column (8/8): `format_bip388_wallet_policy` handles all 10 `CliTemplate` variants exhaustively.
- `specter` column (8/8): `emit_specter_wallet_json` is descriptor-passthrough with no per-shape branching.
- `bitcoin-core` `pkh`: no shape guard in `format_bitcoin_core_importdescriptors`.
- `electrum` singlesig (incl. `bip86`): `P2tr` variant returns neutral `xpub` form without error.
- `green` descriptor-passthrough: multisig guard skipped when `template = None`.

All complement cells match the chapter matrix.

## JSON Output Block Audit (8/8)

All eight representative output blocks well-formed; key names and structural shape match emitter source.

## Section-Anchor Consistency

All `§V.4.5.9.X` and `§V.4.5.10` cross-references resolve correctly post-fold.

## Verdict

- [x] All 4 r2 findings + 3 parent-agent stragglers folded inline. Lint 6/6 OK. Ready for r3 verification.
