# Cycle 3 P0 STRICT-GATE recon — 2026-05-20

**Origin/master HEAD at recon time:** `885f522` (v0.28.6).
**Brainstorm spec source SHA:** `f9fbe6a` (`design/BRAINSTORM_v0_28_plus_residual_followups.md`).

## Summary

4 slugs reconned in parallel (read-only Explore subagents). 2 slugs (`bsms-import-taproot-refusal-parity` + `green-emitter-multisig-refusal-template-only`) confirm the brainstorm scope; 1 slug (`wallet-import-taproot-internal-key`) confirms Framing B (envelope-gate-only) per architect M2 prediction; 1 slug (`wallet-import-format-mismatch-matrix-completion`) **EXPANDS dramatically** — actual residual is 27 arms / ~35-37 test cells, not the 3-arm narrow set the FOLLOWUPS body listed.

---

## Slug 1 — `bsms-import-taproot-refusal-parity`

**Status:** GO. Minor citation drift; scope intact.

- `bsms.rs:70` — `BsmsParser::parse` entry — CORRECT.
- `bsms.rs:479` — `extract_threshold` regex — CORRECT (regex is `(?:thresh|multi|sortedmulti)\((\d+)` — does NOT include `sortedmulti_a`; side-channel confirmed).
- `wallet_export/bsms.rs:69-76` emit-side mirror — STALE (actual 78-84).
- Test-cell cross-reference in source comment cites `bsms.rs:419-421` — STALE (actual `extract_threshold` at 476-491).
- `BsmsTaprootRefused` variant in `error.rs:279` carries `script_type: WalletScriptType` field. **STRUCTURAL CONCERN:** import-side parser has no `WalletScriptType` in scope at `BsmsParser::parse` time. Plan-doc must resolve:
  - **Option α:** NEW variant `BsmsTaprootImportRefused` (no script_type field).
  - **Option β:** Reuse with `WalletScriptType::P2tr` synthetic value (lossy — can't distinguish taproot-singlesig vs taproot-multisig at import time).
  - **Recommended:** Option α — cleaner separation; mirrors emit/import variant split elsewhere.
- Existing import-side cell: `bsms_2line_tr_nums_current_behavior_no_refusal` at `tests/cli_import_wallet_bsms.rs:968`. Cycle 3 renames + flips assertion (exit 0 → exit 2).

---

## Slug 2 — `green-emitter-multisig-refusal-template-only`

**Status:** GO. Citations accurate; scope confirmed isolated to green.rs.

- `wallet_export/green.rs:30-44` — VERBATIM as FOLLOWUPS claims. Refusal guard is `if let Some(t) = inputs.template { if t.is_multisig() { return Err(...) } }`.
- `cmd/export_wallet.rs:603` — slight drift (~10 lines; `script_type_from_descriptor` is at ~L612). Surrounding code accurately described.
- `WalletScriptType::is_multisig` — **DOES NOT EXIST**. Must be added in `wallet_export/mod.rs:158` block. Variants to match: `P2shMulti | P2shP2wshMulti | P2wshMulti | P2trMulti`.
- Anti-pattern survey: **isolated to green.rs alone**. No other emitter has the same bug. (sparrow.rs:42 uses `if let Some(t) = inputs.template` in `collect_missing` — not a refusal context; electrum/coldcard/bip388/jade all have hard-require patterns, not optional-guards.)
- LOC estimate: ~12 (7-line replacement in green.rs + ~5-line new `impl` method in mod.rs).
- New regression cell needed: descriptor-mode multisig green refusal. Existing cells (`tests/cli_export_wallet_green.rs` 3 cells) cover singlesig + templated-multisig; no descriptor-mode case yet.

---

## Slug 3 — `wallet-import-format-mismatch-matrix-completion`

**Status:** SCOPE EXPANSION REQUIRES DECISION before plan-doc write.

### Current state per-arm at HEAD `885f522`

| `--format` arm | Refused via `ImportWalletFormatMismatch` | Width | Missing (silent fall-through) |
|---|---|---|---|
| `bsms` | BitcoinCore | 1 | coldcard, coldcard-multisig, electrum, jade, sparrow, specter |
| `bitcoin-core` | Bsms | 1 | coldcard, coldcard-multisig, electrum, jade, sparrow, specter |
| `coldcard` | Bsms, BitcoinCore, ColdcardMultisig, Sparrow, Specter | 5 | electrum, jade |
| `coldcard-multisig` | Bsms, BitcoinCore | 2 | coldcard, electrum, jade, sparrow, specter |
| `electrum` | Bsms, BitcoinCore, Coldcard, ColdcardMultisig, Sparrow, Specter | 6 | jade |
| `jade` | Bsms, BitcoinCore, Coldcard, ColdcardMultisig, Electrum, Sparrow, Specter | 7 | — (complete) |
| `sparrow` | Bsms, BitcoinCore, ColdcardMultisig | 3 | coldcard, electrum, jade, specter |
| `specter` | Bsms, BitcoinCore, ColdcardMultisig, Sparrow | 4 | coldcard, electrum, jade |

### Scope-drift finding

- FOLLOWUPS body (`design/FOLLOWUPS.md:~2589`) says: "narrow-arm residuals are now: BSMS (1), BitcoinCore (1), ColdcardMultisig (2)" — **STRUCTURALLY WRONG**.
- Actual gaps:
  - BSMS: 6 missing arms
  - BitcoinCore: 6 missing arms
  - ColdcardMultisig: 5 missing arms
  - Coldcard: 2 missing (NOT in FOLLOWUPS residual list)
  - Sparrow: 4 missing (NOT in FOLLOWUPS list)
  - Specter: 3 missing (NOT in FOLLOWUPS list)
  - Electrum: 1 missing (NOT in FOLLOWUPS list)
- Total: **27 new `ImportWalletFormatMismatch` return sites** needed for full matrix.
- Total: **~35-37 new test cells** to close the 56-cell off-diagonal matrix (current coverage ~19 cells).
- Effort estimate: 1-2 days alone (would likely dominate Cycle 3).

### Plan-doc options

- **Option A (full completion):** All 27 arms + 37 cells; Cycle 3 effort balloons from "3-5 days" → "5-7 days".
- **Option B (narrow to original residual set):** Close only BSMS / BitcoinCore / ColdcardMultisig per FOLLOWUPS body; file a new FOLLOWUP for the discovered Coldcard/Sparrow/Specter/Electrum gaps as scope-expansion finding.
- **Option C (full completion but defer to dedicated cycle):** Drop Slug 3 entirely from Cycle 3; file as separate cycle (Cycle 3.5 or Wave-3 add).

---

## Slug 4 — `wallet-import-taproot-internal-key`

**Status:** GO. Framing B (envelope-gate-only) confirmed; drop per-exporter framing from plan-doc.

- `cmd/export_wallet.rs:650` — EXACT line: `taproot_internal_key: None,` inside the single `EmitInputs` struct literal in `run_from_import_json`. Comment on lines 647-649 self-documents the FOLLOWUP.
- All 8 `wallet_import/*.rs` parsers surveyed: **NONE** carry taproot internal-key designation. Uniformly taproot-agnostic. Framing A (per-exporter fan-out) is wrong.
- Fix is a single-point change before `EmitInputs` is built. Two sub-options:
  - **Fix-α:** Add `BadInput` refusal at `run_from_import_json` if envelope-side descriptor body matches `tr(...)`.
  - **Fix-β:** Add `taproot_internal_key` field to envelope `BundleJson` so the field propagates through.
- **Recommended:** Fix-α for v0.28.7 (no wire-shape change; pure refusal); Fix-β stays open for v0.29+ wire-shape evolution.
- LOC estimate: ~15-20 (refusal block + 1 test cell asserting refusal of envelope with `tr(...)` body).

---

## Recommendations for plan-doc body

1. **Slug 1:** Use Option α (new `BsmsTaprootImportRefused` variant).
2. **Slug 2:** Add `WalletScriptType::is_multisig()` method in lockstep; refactor green.rs:30-44.
3. **Slug 3:** USER DECISION REQUIRED — Option A / B / C above. **Recommend Option B** (close original 3-arm narrow set + file new FOLLOWUP for discovered gaps).
4. **Slug 4:** Use Fix-α (refusal-only; no wire-shape change).
