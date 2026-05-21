# cycle-prep recon — open `v0.28+` FOLLOWUPs (2026-05-20 post-A/B/C ship)

**HEAD at recon time:** `2ad13a9` (post-Cycle 4 manual-v0.3.0 ship).
**Sync state:** local master ≡ origin/master.
**Slugs verified:** 10 open FOLLOWUPs in the `v0.28+` tier band (as surfaced in this session's grep).

## Per-slug verification

### bsms-bip129-encryption-envelope
- **`crates/mnemonic-toolkit/src/wallet_import/bsms.rs`:** ACCURATE (file-level citation; characterization "plaintext-only 4-line parser" is structurally correct; 4-line logic spans L116-144).
- **`design/agent-reports/v0_27_0-phase-2-bip129-recon.md` §2:** ACCURATE — Section 2 (L18-39) covers BIP-129 STANDARD/EXTENDED envelopes.
- **Cross-cutting:** No drift. Both citations could be tightened with explicit line ranges for future readers, but body wording is acceptable as narrative.

### wallet-import-jade-seedqr
- **`crates/mnemonic-toolkit/src/wallet_import/jade.rs`:** ACCURATE — L20-25 documents Q1 lock + SeedQR deferral with explicit FOLLOWUP slug citation at L24.
- **`docs/manual/src/45-foreign-formats.md` §"What's NOT supported":** ACCURATE — section at L770; slug ref at L786.
- **Cross-cutting:** No drift.

### wallet-import-electrum-encrypted
- **`crates/mnemonic-toolkit/src/wallet_import/electrum.rs:312`:** ACCURATE — encrypted refusal at parse time with correct FOLLOWUP slug in stderr template.
- **`docs/manual/src/45-foreign-formats.md` §"Refusal stderr templates":** ACCURATE — refusal documented at L677 + parse-contract table at L664.
- **Cross-cutting:** No drift.

### wallet-import-format-mismatch-matrix-completion — **SCOPE DRIFT** (Important)
- **`crates/mnemonic-toolkit/src/cmd/import_wallet.rs:252` (BSMS arm):** ACCURATE — checks only BitcoinCore sniff.
- **`crates/mnemonic-toolkit/src/cmd/import_wallet.rs:261` (BitcoinCore arm):** ACCURATE — checks only BSMS sniff.
- **`crates/mnemonic-toolkit/src/cmd/import_wallet.rs:327` (ColdcardMultisig arm):** ACCURATE — checks BSMS + BitcoinCore.
- **`crates/mnemonic-toolkit/src/cmd/import_wallet.rs:460` (Sparrow arm):** ACCURATE — checks BSMS + BitcoinCore + ColdcardMultisig.
- **`crates/mnemonic-toolkit/src/cmd/import_wallet.rs:495` (Specter arm):** ACCURATE — checks BSMS + BitcoinCore + ColdcardMultisig + Sparrow.
- **Cross-cutting (Important):** FOLLOWUP body documents the P1C/P2C-era state but the matrix has since expanded — Coldcard arm now checks 5 formats (P3C); Electrum arm checks 6 (P6C); Jade arm checks all 7 sibling formats (P5C). The "inverse-wiring gap" claim in the FOLLOWUP body needs **re-validation against the now-wider matrix**. Some of the gaps may already be closed.

### bsms-import-taproot-refusal-parity — DRIFTED-by-3
- **`bsms.rs::BsmsParser::parse`:** ACCURATE — Parse function at L70; currently accepts taproot blobs (no early `Tr(_)` short-circuit).
- **`wallet_export/bsms.rs:69-76`:** **DRIFTED-by-3** — Emit-side refusal logic at L66-79 (cited 69-76). Refusal carries `BsmsTaprootRefused { script_type }` payload as documented.
- **`bsms.rs::extract_threshold` (L476):** ACCURATE — Function at L476; regex at L479 does NOT match `sortedmulti_a(` as documented.
- **`tests/cli_import_wallet_bsms.rs::bsms_2line_tr_nums_current_behavior_no_refusal` (L968):** ACCURATE.
- **Cross-cutting:** Minor 3-line drift on emit-side block; semantic claims hold.

### sparrow-taproot-descriptor-passthrough-import-support — DRIFTED-by-~100
- **`wallet_import/sparrow.rs`:** **DRIFTED-by-~100** — FOLLOWUP body cites "parse-step-6 taproot refusal" but the `script_template.contains("tr(")` short-circuit is actually at L311 (no parse-step-N marker present in current source).
- **`wallet_export/sparrow.rs:215-219`:** ACCURATE — `TrMultiA | TrSortedMultiA` emission of taproot descriptor-passthrough via `canonical_descriptor`.
- **Cross-cutting:** Semantic intent (parse-refusal location) remains traceable. Update the body's "parse-step-6" wording to `sparrow.rs:311` (literal line).

### coldcard-legacy-mk1-mk2-top-level-xpub-inference — **STATUS DRIFT** (potentially-already-resolved)
- **`crates/mnemonic-toolkit/src/wallet_import/coldcard.rs`:** ACCURATE — Parser has modern multi-path BIP envelope at L16-19/L100-103/L429-456.
- **Cross-cutting (Important):** The parser ALREADY HAS legacy mk1/mk2 fallback at L460-462 + SLIP-132 prefix inference at L471-494 (`infer_bip_from_xpub_prefix`). The heuristic the FOLLOWUP describes ("check per-path blocks first, then fallback to legacy top-level `xpub` + `xfp`") is implemented. The FOLLOWUP may be **partially resolved** as of an earlier-than-tracked commit. Re-classify: confirm whether the existing fallback covers the full mk1/mk2 firmware-history scope OR if it's narrower than required.

### green-emitter-multisig-refusal-template-only — DRIFTED-by-+35
- **`wallet_export/green.rs:30-44`:** ACCURATE — guard clause intact + refusal logic present.
- **`cmd/export_wallet.rs:603`:** **DRIFTED-by-+35** — `template: None` assignment in `run_from_import_json()` is at L638 in current source (cited 603).
- **Cross-cutting:** Line drift from cumulative code growth. Structural issue (multisig refusal bypassed in descriptor-mode due to `template == None`) remains valid.

### import-wallet-envelope-schema-version-narrative-drift
- **`crates/mnemonic-toolkit/src/cmd/import_wallet.rs:87`:** ACCURATE — `IMPORT_WALLET_ENVELOPE_SCHEMA_VERSION = "1"` const at L87.
- **`crates/mnemonic-toolkit/src/cmd/import_wallet.rs:975`:** ACCURATE — inner `BundleJson` literal `schema_version: "4"` at L974-975.
- **Cross-cutting:** Both constants present and dual-version pairing matches body narrative. No symbol renames.

### cross-format-refusal-matrix-include-coldcard-multisig
- **`tests/cli_export_wallet_from_import_json.rs:592-593`:** ACCURATE — `TEMPLATE_ONLY_DESTS` const at L592 (defn) + L593 (value).
- **`tests/cli_export_wallet_from_import_json.rs:815`:** ACCURATE — `REFUSAL_STDERR_PATTERNS` with `"requires --template"` substring at L815.
- **`tests/cli_export_wallet_from_import_json.rs:871`:** ACCURATE — cell count `assert_eq!(cell_count, 32, ...)` at L871.
- **Cross-cutting:** FOLLOWUP correctly identifies the refusal-text substring mismatch (`"requires --template"` vs new arm's `"requires a multisig --template"`). Both proposed solutions (broaden pattern OR tighten refusal text) remain valid.

---

## Aggregate findings

| Slug | Status | Notes |
|---|---|---|
| bsms-bip129-encryption-envelope | ✅ ACCURATE | file-level cites |
| wallet-import-jade-seedqr | ✅ ACCURATE | |
| wallet-import-electrum-encrypted | ✅ ACCURATE | |
| wallet-import-format-mismatch-matrix-completion | ⚠️ **SCOPE DRIFT** | body docs P1C/P2C-era; matrix has expanded |
| bsms-import-taproot-refusal-parity | 🔧 DRIFTED-by-3 | minor; emit-side block shifted |
| sparrow-taproot-descriptor-passthrough-import-support | 🔧 DRIFTED-by-~100 | body wording "parse-step-6" needs L311 lookup |
| coldcard-legacy-mk1-mk2-top-level-xpub-inference | ⚠️ **STATUS DRIFT** | parser ALREADY has legacy fallback at L460-494; may be partially resolved |
| green-emitter-multisig-refusal-template-only | 🔧 DRIFTED-by-+35 | line cite shifted from L603 → L638 |
| import-wallet-envelope-schema-version-narrative-drift | ✅ ACCURATE | |
| cross-format-refusal-matrix-include-coldcard-multisig | ✅ ACCURATE | (filed this session) |

**5 ACCURATE / 3 line-drifted / 2 substantive-drift (scope or status).**

## Highest-value findings for the next brainstorm

1. **`coldcard-legacy-mk1-mk2-top-level-xpub-inference` may already be resolved.** Parser has legacy-xpub fallback at `coldcard.rs:460-462` + SLIP-132 prefix inference at `:471-494`. Before any brainstorm picks this up, verify whether the existing implementation covers the full mk1/mk2 firmware-history scope OR is narrower than the FOLLOWUP body describes. If covered, flip Status to `resolved` + cite the implementing commit. If narrower, narrow the FOLLOWUP body to the residual gap.

2. **`wallet-import-format-mismatch-matrix-completion` body is outdated.** Inverse-wiring gap claim documents P1C/P2C-era narrow matrix; current matrix has expanded (Coldcard 5-format, Electrum 6-format, Jade 7-format coverage). Re-validate the gap before any brainstorm dispatches.

3. **3 line-drift fixes** (bsms-import-taproot/sparrow-taproot/green-emitter) are mechanical — the slug bodies should be amended with current line numbers at brainstorm-write time. Per `feedback-followups-md-line-numbers-presumed-stale` discipline: re-grep at brainstorm-write, don't trust the body verbatim.

4. **5 ACCURATE citations** (bsms-encryption / jade-seedqr / electrum-encrypted / schema-version-drift / coldcard-multisig-matrix) are ready-to-brainstorm with no body fixup needed.

## No-action recommendation

This is a pure recon dossier; no brainstorm or implementation triggered. Next-cycle scheduling is user-discretion.
