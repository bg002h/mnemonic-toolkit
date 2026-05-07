# Phase 6 — feature-dev:code-reviewer review, round 1

**Date:** 2026-05-07
**Branch:** `manual/v0_1` (Phase 6 author commit)
**Verdict:** Not converged. 2 critical / 2 important / 0 nits.

All findings are in chapter 57.

## Critical

### C1 — Chapter 57: BIP-388 JSON field name is wrong

The example used `"policy_template"`. Toolkit emits `"description_template"` (verified at `wallet_export.rs:370` and `:423`).

**Fix applied:** Renamed in chapter 57 example block AND chapter 37's example block (same bug; same field name in 30-workflows/37-wallet-export.md).

### C2 — Chapter 57: BIP-388 placeholder notation wrong

Example showed `@N/<0;1>/*`; toolkit emits `@N/**` (verified at `wallet_export.rs:313` and `:417`).

**Fix applied:** Changed placeholders to `@0/**`, `@1/**`, `@2/**` in both chapter 57 and chapter 37 examples.

## Important

### I1 — Chapter 57 (and 37): `--format sparrow` / `--format specter` are stub refusals in v0.8

Both flags exist on the binary but return `ExportWalletFormatStub` errors. The chapter table claimed each was a usable Sparrow-/Specter-native format.

**Fix applied:**

- Chapter 57's "Choosing one format" table: replaced Sparrow / Specter rows with `--format bip388` (with parenthetical note about the deferred stubs).
- Chapter 37's "Sparrow native format" and "Specter native format" sections collapsed into a single "Sparrow + Specter (currently via BIP-388)" section that documents the deferral and recommends `--format bip388` for both. Mermaid flowchart updated to mark the native variants as "deferred stub".

### I2 — Chapter 57: Bitcoin Core BIP-388 wallet_policy import claim was overstated

Chapter said "Bitcoin Core 25+ supports importing both shapes" — Bitcoin Core's `importdescriptors` does NOT consume BIP-388 wallet_policy JSON. The bip388 format targets hardware wallets and third-party coordinators.

**Fix applied:** Rewrote the prose to clarify that Bitcoin Core consumes `--format bitcoin-core` only; `--format bip388` targets hardware wallets / third-party coordinators (Coldcard, Ledger, Foundation Passport, Sparrow's BIP-388 import path). Removed "Bitcoin Core 24" from the format-selection table.

## Verification of correct elements

| Check | Status |
|---|---|
| Chapter 54 neutrality (no ranking, focus on when each fits) | OK |
| Chapter 54 + 57 density (≤4 pages spirit) | OK |
| Chapter 55 K=N anti-pattern correctly qualified | OK |
| Chapter 55 three-axis orthogonality | OK |
| Chapter 56 `--passphrase` / `--bip38-passphrase` per-edge table | OK (verified vs cli-help/mnemonic-convert.txt) |
| Chapter 56 `(entropy, bip38)` BREAKING note | OK (vs CHANGELOG v0.8) |
| codex32 K-of-N status (planned for ms-codec v0.2) | OK (vs CLAUDE.md) |
| SLIP-39 RS1024 attribution | OK (chapter says "partial per-character" — reasonable shorthand) |

## Convergence assessment

After applying C1+C2+I1+I2 fixes (chapter 57 + chapter 37), Phase 6 is at 0C/0I. Chapters 51-56 were factually clean. No round-2 dispatch needed.
