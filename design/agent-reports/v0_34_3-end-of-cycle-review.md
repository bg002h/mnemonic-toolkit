# v0.34.3 — End-of-cycle architect review (opus) — MANDATORY pre-tag gate

**Date:** 2026-05-22
**Cycle:** mnemonic-toolkit v0.34.3 — wallet-cluster FOLLOWUP hygiene (docs/test-only PATCH)
**Branch:** `v0.34.3-wallet-cluster-hygiene`
**Reviewer:** opus (feature-dev:code-reviewer), end-of-cycle gate
**Scope reviewed:** full cycle diff `/tmp/v0_34_3_cycle.diff` (commits `6c68547`..`a77d1cb`) + live source

---

## Critical
(none)

## Important
(none)

## Minor

- **Stale prose in the rewritten `bsms-bip129-full-cutover` "What" header line is now internally inconsistent with the rewritten Status line.** `design/FOLLOWUPS.md:2215` still reads `**What:** v0.28+ remaining sub-items (a) and (c) and (d):` — but the cycle's Status rewrite at `:2223` correctly states "ONLY sub-item (d) remains", and the sub-bullets mark (a)/(b)/(e) shipped v0.28.0, (c) shipped v0.31.0. The header line was already stale before this cycle and the rewrite did not touch it. Not a wrongly-closed slug, not blocking — authoritative Status line is correct — but a reader hitting L2215 first sees a contradiction. Fix: change `:2215` to `**What:** sub-items (a)–(e) below; only (d) remains open:`. Pre-existing drift surfaced (not introduced) by the cycle.

- **Plan-doc Task-4 Step-4 CHANGELOG text says "corrects the lock-regen discipline" but the shipped CHANGELOG says "applies".** `IMPLEMENTATION_PLAN...:171` vs shipped `CHANGELOG.md:11` ("applies" — the better wording). Harmless plan-vs-shipped delta; shipped text is correct. No action required.

---

## Verification summary (confirmed against live source)

- **Closures backed by shipped code.** `wallet-import-bsms-encrypted` → resolved: `import-wallet --bsms-encryption-token` real (`cmd/import_wallet.rs:227-228`), PBKDF2-SHA512 + AES-256-CTR + HMAC-SHA256 per BIP-129 §Encryption. `wallet-import-bsms-round-1` → resolved-superseded: `--bsms-round1` is a real repeating *verify* path (BIP-322), not assembly; DISPOSITION A sound. `bsms-bip129-full-cutover` → (d)-only: (a)/(b)/(e) shipped v0.28.0, (c) shipped v0.31.0; sub-item (d) genuinely pending — `6 =>` lenient arm at `wallet_import/bsms.rs:146` still parses-with-deprecation. Correct future-MINOR framing.
- **Duplicate stub deletion.** `grep "DUPLICATE STUB"` = 0; canonical `### bsms-bip129-full-cutover` survives once (`FOLLOWUPS.md:2208`); clean transition `bsms-taproot-emit` (:2478) → `wallet-import-taproot-internal-key` (:2480). No orphan.
- **Cite refreshes point at live symbols.** taproot-emit `:64-79` spans `BsmsEmitter::emit` (L64) → `P2tr|P2trMulti` refusal (L77-80); signet `:24-26` is the live doc-comment; `6 =>` arm `:146` live; `extract_threshold` guard `:496-497` live; parse-entry `tr(` refusal `:215` live.
- **Unit test.** `wallet_import/bsms.rs:555-569` compiles (`ToolkitError` via `use super::*` → module `use crate::error::ToolkitError`), `BsmsTaprootImportRefused` correct variant (`error.rs:51`), both inputs hit the L496-497 substring guard directly (not the L215 parse-entry refusal, only reachable via `parse()`). Assertion logic correct. (Note: `tr(NUMS,sortedmulti_a(...))` contains both `sortedmulti_a(` and `multi_a(`; OR short-circuits — identical result; second input exercises the `multi_a(` branch.)
- **CLAUDE.md edit accurate.** `mnemonic-gui/tests/schema_mirror.rs:91-121` (`assert_schema_matches_help`) compares flag-name SETS via set-difference, does NOT touch `--json` wire-shape. No overstatement.
- **Version consistency.** `Cargo.toml:3` = 0.34.3, `Cargo.lock:682` = 0.34.3, `install.sh:32` = `mnemonic-toolkit-v0.34.3`, `CHANGELOG.md:9` = `[0.34.3]`. All aligned; lock-regen lesson applied.
- **Scope discipline.** 9 files; only `.rs` change is a `#[cfg(test)]` fn inside `mod tests`. No production code path changed, no clap flag added/removed/renamed → PATCH + no GUI/manual lockstep correct. Nothing beyond the plan's 4 tasks.
- **Structural integrity.** No broken `###` heading, no orphan, schema-mirror slug narrowed consistently with the CLAUDE.md edit, extract-threshold slug Status flipped to resolved with correct cite-drift fix.

VERDICT: GREEN (0C/0I)

---

## Fold disposition (controller)
GREEN (0C/0I) → gate satisfied. Folded Minor #1 (`FOLLOWUPS.md:2215` "What:" header reworded for coherence with the rewritten Status line) — doc-only, zero spec/code impact, so no R0 re-dispatch (re-dispatch guards against drift from folding Critical/Important; there were none). Minor #2 needs no action (shipped CHANGELOG wording is the correct one).
