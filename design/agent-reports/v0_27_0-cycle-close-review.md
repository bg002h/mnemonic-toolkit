# v0.27.0 End-of-Cycle Holistic Architect Review

**Date:** 2026-05-19
**Reviewer:** opus-4-7 (feature-dev:code-reviewer)
**Verdict:** YELLOW pre-fold → GREEN post-fold (1 Important + 2 Minor folded in-cycle)

## Cycle-level acceptance verification

| Gate | Status | Evidence |
|---|---|---|
| Cargo.toml version bump | PASS | `crates/mnemonic-toolkit/Cargo.toml:3` = `0.27.0` |
| CHANGELOG audit | PASS | All 6 closed FOLLOWUPs cited in `### Closed FOLLOWUPS`; 5 `### Added` + 2 `### Changed` entries trace back to FOLLOWUPs/NEW features |
| FOLLOWUPS Status flips | PASS | All 6 closed FOLLOWUPs have `**Status:** resolved` with closure narratives. All 5 new FOLLOWUPs filed with complete fields. |
| Integration cell preserves descriptor + xpubs byte-for-byte | PASS | `cross_format_bsms_to_bitcoin_core_to_import_round_trip` asserts all 3 cosigner xpubs + all 3 origin paths preserved verbatim in Bitcoin Core output |
| Manual CLI reference covers 7 new flags + inspect schema_version | PASS | All 7 flags + inspect JSON envelope documented; `make lint` flag-coverage gate green |
| 1536 toolkit tests + clippy + manual lint | PASS | 1536 passing, 0 failing; clippy clean; manual lint OK |

## Findings (all folded in-cycle)

### IMPORTANT 1 — Manual chapter 3A misrepresented BIP-129 Round-1 record schema and signing digest scope (confidence 95) — FOLDED

**File:** `docs/manual/src/30-workflows/3A-bsms-round1-verify.md`

**Issue 1 (folded):** the chapter showed the 5-line Round-1 record with line 3 = DERIVATION_PATH and line 4 = KEY. BIP-129 spec (verified upstream + per `bsms_round1.rs:31-71`) is:
- L1 `BSMS 1.0`
- L2 `<TOKEN_HEX>`
- L3 `<KEY>` carrying `[fingerprint/derivation-path]xpub_or_pubkey` (derivation path embedded inside line 3, not a separate line)
- L4 `<description>` (≤80 char text)
- L5 `<SIGNATURE>`

**Fold:** schema diagram rewritten + cell text below diagram clarifies that the path is INLINE in `[...]`.

**Issue 2 (folded):** chapter said the BIP-322 digest is computed over `TOKEN_HEX` alone. BIP-129 spec (line 81: *"sign the first four lines"*) + implementation `bsms_round1.rs::signed_body` confirms the signed body is the 4-line body joined by `\n` (no trailing newline).

**Fold:** digest formula corrected; source-of-truth citation added.

**Issue 3 (folded):** lines 48-50 described per-record output as "stderr NOTICE per record on verification failure" but the code block at lines 52-57 showed the standalone-mode stdout summary. Stderr NOTICE fires only on failure.

**Fold:** prose rewritten to correctly attribute stdout-summary vs stderr-NOTICE channels.

### MINOR 1 — CHANGELOG date stamp 2026-05-19 — CONFIRMED OK

CHANGELOG entry header reads `## mnemonic-toolkit [0.27.0] — 2026-05-19`. Today is 2026-05-19. Cycle: v0.26.0 (2026-05-18) → v0.27.0 (2026-05-19) is a 1-day cycle, which is fine.

### MINOR 2 — `--bsms-round1 <FILE>` flag-table cell did not enumerate the `-` rejection — FOLDED

**File:** `docs/manual/src/40-cli-reference/41-mnemonic.md:695`

**Issue:** bundle / export-wallet entries at `:45-46` and `:643-644` explicitly call out `<FILE|->` accepting both file paths and stdin. The `--bsms-round1` entry showed `<FILE>` only; the rejection of `-` (deliberate v0.27.0 limitation per chapter 3A) was not surfaced in the CLI reference.

**Fold:** appended "v0.27.0 accepts a file path only — stdin form `-` is rejected, supply a file path per record (FOLLOWUP: multi-record stdin intake)" to the cell text.

## Items verified PASS

- **Cycle-level coherence (headline deliverable).** `cross_format_bsms_to_bitcoin_core_to_import_round_trip` at `tests/cli_export_wallet_from_import_json.rs:285-327` is genuine end-to-end: BSMS Round-2 blob → `import-wallet --json` → `export-wallet --from-import-json - --format bitcoin-core` → asserts both Bitcoin Core descriptors contain all 3 cosigner xpubs verbatim + all 3 origin-paths.
- **5 new FOLLOWUPs filed.** All 5 present with required fields:
  - `cross-format-conversion-matrix-expansion`
  - `bsms-taproot-emit`
  - `bsms-bip129-full-cutover`
  - `wallet-import-taproot-internal-key`
  - `plan-smoke-step4-ms1-on-bundle-not-supported`
- **All 6 closed FOLLOWUPs Status-flipped.** Per-phase commits flipped them in-line; backstop sweep finds no gaps.
- **Manual chapter 39 (cross-format-conversion).** Recipes 1/2/3 + multi-entry-envelope section + supported-destinations table consistent with shipped flag surfaces.
- **Inspect JSON envelope manual.** Correctly describes `schema_version: "1"` backfill via `InspectEnvelope` wrapper + notes RepairJson already at v0.22.0.
- **gui-schema drift gate.** No NEW subcommands added; 7 new flags auto-emit via gui-schema macro infrastructure. `pinned-upstream.toml` correctly not bumped per plan §4.6.

## Final verdict

**GREEN.** Both Important + 2 Minor findings folded pre-tagging.

The cycle's headline cross-format conversion contract is genuinely closed end-to-end. The closure machinery (CHANGELOG + FOLLOWUP Status flips + new FOLLOWUPs) is in excellent shape — the per-phase Status-flip discipline lesson from v0.25.0 held firm this cycle. No Critical findings; no code-level issues; no missing test coverage.
